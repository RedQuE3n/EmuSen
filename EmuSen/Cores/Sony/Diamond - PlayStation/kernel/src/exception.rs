//! The exception path after the entry has saved the registers: the four priority chains (psx-spx "Priority
//! Chains"), the exit through the hook structure, unresolved exceptions, and the kernel's own chain elements: the
//! syscall handler, the timer and vblank handlers, and the default interrupt handler. Diamond_OpenKernel.md §4.

use crate::layout::{TABLES, io, table};
use crate::mem::{call, rd, wr};
use crate::{blocks, entry, event};

pub mod tcb {
    pub const fn reg(n: u32) -> u32 {
        0x08 + 4 * n
    }
    pub const EPC: u32 = 0x88;
    pub const SR: u32 = 0x94;
    pub const CAUSE: u32 = 0x98;
}

/// The exception stack, 1000h bytes in the kernel's RAM ("No Nested Exceptions").
#[unsafe(no_mangle)]
pub static mut EXCEPTION_STACK: [u32; 0x400] = [0; 0x400];

/// ResetEntryInt's structure, in setjmp's layout, and the one exceptions currently leave through.
static mut DEFAULT_EXIT: [u32; 12] = [0; 12];
static mut EXIT: u32 = 0;

/// ChangeClearRCnt's flags for timers 0-2 and vblank, and SetIrqAutoAck's for IRQ0-IRQ10.
static mut CLEAR_RCNT: [u32; 4] = [1, 1, 1, 0];
static mut AUTO_ACK: [u32; 11] = [0; 11];

/// The kernel's own chain elements: next, second function, first function, unused.
static mut SYSCALL_ELEMENT: [u32; 4] = [0; 4];
static mut TIMER_ELEMENTS: [[u32; 4]; 4] = [[0; 4]; 4];
static mut DEFINT_ELEMENT: [u32; 4] = [0; 4];

fn excb(priority: u32) -> u32 {
    rd(TABLES + table::EXCB) + 8 * (priority & 3)
}

/// Called from the entry with the TCB the registers went to; never returns.
#[unsafe(no_mangle)]
extern "C" fn exception_dispatch(tcb: u32) -> ! {
    let code = rd(tcb + tcb::CAUSE) >> 2 & 0x1F;
    if code == 0 {
        let epc = rd(tcb + tcb::EPC);
        if rd(epc) & 0xFE00_0000 == 0x4A00_0000 {
            wr(tcb + tcb::EPC, epc + 4);
        }
    }
    for priority in 0..4 {
        let mut element = rd(excb(priority));
        while element != 0 {
            let next = rd(element);
            let first = rd(element + 8);
            if first != 0 {
                let r = call(first, 0, 0, 0, 0);
                let second = rd(element + 4);
                if r != 0 && second != 0 {
                    call(second, r, 0, 0, 0);
                }
            }
            element = next;
        }
    }
    if code != 0 {
        event::deliver_event(0xF000_0010, 0x1000);
        call(rd(crate::layout::A_TABLE + 4 * 0x40), 0, 0, 0, 0);
        unsafe { entry::return_from_exception() }
    }
    unsafe { entry::longjmp(EXIT, 1) }
}

/// C(02h): the element becomes the chain's first.
pub extern "C" fn sys_enq_int_rp(priority: u32, element: u32) -> u32 {
    let head = excb(priority);
    wr(element, rd(head));
    wr(head, element);
    0
}

/// C(03h): the element removed wherever it is in the chain.
pub extern "C" fn sys_deq_int_rp(priority: u32, element: u32) -> u32 {
    let mut link = excb(priority);
    while rd(link) != 0 {
        if rd(link) == element {
            wr(link, rd(element));
            return 0;
        }
        link = rd(link);
    }
    0
}

/// B(18h).
pub extern "C" fn reset_entry_int() -> u32 {
    unsafe {
        let exit = &raw mut DEFAULT_EXIT as u32;
        for i in 0..12 {
            wr(exit + 4 * i, 0);
        }
        wr(exit, entry::return_from_exception as *const () as u32);
        wr(exit + 4, &raw const EXCEPTION_STACK as u32 + 0x1000 - 4);
        EXIT = exit;
        exit
    }
}

/// B(19h).
pub extern "C" fn hook_entry_int(address: u32) -> u32 {
    unsafe { EXIT = address };
    0
}

fn element(e: *mut [u32; 4], first: extern "C" fn() -> u32) -> u32 {
    let a = e as u32;
    wr(a, 0);
    wr(a + 4, 0);
    wr(a + 8, first as *const () as u32);
    wr(a + 12, 0);
    a
}

/// C(01h).
pub extern "C" fn enqueue_syscall_handler(priority: u32) -> u32 {
    sys_enq_int_rp(priority, element(&raw mut SYSCALL_ELEMENT, syscall_handler))
}

/// C(00h): timers 0, 1 and 2, then vblank, so that the chain reads vblank first as psx-spx lists it.
pub extern "C" fn enqueue_timer_and_vblank_irqs(priority: u32) -> u32 {
    let handlers: [extern "C" fn() -> u32; 4] = [timer0_handler, timer1_handler, timer2_handler, vblank_handler];
    for (i, h) in handlers.into_iter().enumerate() {
        let e = unsafe { (&raw mut TIMER_ELEMENTS as *mut [u32; 4]).add(i) };
        sys_enq_int_rp(priority, element(e, h));
    }
    0
}

/// C(0Ch).
pub extern "C" fn init_def_int(priority: u32) -> u32 {
    sys_enq_int_rp(priority, element(&raw mut DEFINT_ELEMENT, default_handler))
}

/// C(0Dh).
pub extern "C" fn set_irq_auto_ack(irq: u32, flag: u32) -> u32 {
    if irq < 11 {
        unsafe { AUTO_ACK[irq as usize] = (flag != 0) as u32 };
    }
    0
}

/// C(0Ah): the old flag.
pub extern "C" fn change_clear_rcnt(t: u32, flag: u32) -> u32 {
    if t > 3 {
        return 0;
    }
    unsafe {
        let old = CLEAR_RCNT[t as usize];
        CLEAR_RCNT[t as usize] = flag;
        old
    }
}

pub fn pending() -> u32 {
    rd(io::I_STAT) & rd(io::I_MASK)
}

pub fn acknowledge(bit: u32) {
    wr(io::I_STAT, !(1 << bit));
}

extern "C" fn syscall_handler() -> u32 {
    let tcb = blocks::current_tcb();
    if rd(tcb + tcb::CAUSE) >> 2 & 0x1F != 8 {
        return 0;
    }
    wr(tcb + tcb::EPC, rd(tcb + tcb::EPC) + 4);
    let sr = rd(tcb + tcb::SR);
    match rd(tcb + tcb::reg(4)) {
        0 => {}
        1 => {
            wr(tcb + tcb::reg(2), (sr & 0x404 == 0x404) as u32);
            wr(tcb + tcb::SR, sr & !0x404);
        }
        2 => wr(tcb + tcb::SR, sr | 0x404),
        3 => {
            wr(tcb + tcb::reg(2), 1);
            wr(rd(TABLES + table::PCB), rd(tcb + tcb::reg(5)));
        }
        _ => {
            event::deliver_event(0xF000_0010, 0x4000);
        }
    }
    unsafe { entry::return_from_exception() }
}

fn counter(t: u32, irq: u32) -> u32 {
    if pending() & 1 << irq != 0 {
        event::deliver_event(0xF200_0000 + t, 2);
        if unsafe { CLEAR_RCNT[t as usize] } != 0 {
            acknowledge(irq);
            unsafe { entry::return_from_exception() }
        }
    }
    0
}

extern "C" fn timer0_handler() -> u32 {
    counter(0, 4)
}
extern "C" fn timer1_handler() -> u32 {
    counter(1, 5)
}
extern "C" fn timer2_handler() -> u32 {
    counter(2, 6)
}
extern "C" fn vblank_handler() -> u32 {
    counter(3, 0)
}

/// The events psx-spx's "Default IRQ Handler Events" lists for IRQ0-IRQ10, IRQ6 with timer 1's class as listed.
const DEFAULT_CLASSES: [u32; 11] = [
    0xF000_0001,
    0xF000_0002,
    0xF000_0003,
    0xF000_0004,
    0xF000_0005,
    0xF000_0006,
    0xF000_0006,
    0xF000_0008,
    0xF000_000B,
    0xF000_0009,
    0xF000_000A,
];

extern "C" fn default_handler() -> u32 {
    let p = pending();
    for (irq, class) in DEFAULT_CLASSES.iter().enumerate() {
        if p & 1 << irq != 0 {
            event::deliver_event(*class, 0x1000);
            if unsafe { AUTO_ACK[irq] } != 0 {
                acknowledge(irq as u32);
            }
        }
    }
    0
}
