//! The control blocks in the table of tables (psx-spx "BIOS Control Blocks"): their allocation from the kernel's
//! memory by SetConf, GetConf, and the current thread through the PCB. Diamond_OpenKernel.md §4.4.

use crate::layout::{self, EVCB_SIZE, EXCB_SIZE, PCB_SIZE, TABLES, TCB_SIZE, table};
use crate::mem::{rd, wr};
use crate::{cdrom, exception, heap, pad};

static mut EVENTS: u32 = 0;
static mut THREADS: u32 = 0;
static mut STACKTOP: u32 = 0;

fn entry(offset: u32, address: u32, size: u32) {
    wr(TABLES + offset, address);
    wr(TABLES + offset + 4, size);
}

fn zeroed(size: u32) -> u32 {
    let a = heap::alloc_kernel_memory(size);
    if a != 0 {
        let mut i = 0;
        while i < size {
            wr(a + i, 0);
            i += 4;
        }
    }
    a
}

/// A(9Ch): the kernel's memory reset, the blocks allocated anew with the boot thread in TCB 0, the default handlers
/// enqueued at the priorities psx-spx gives (syscall 0, timers and vblank 1, the default handler 3).
pub extern "C" fn set_conf(events: u32, threads: u32, stacktop: u32) -> u32 {
    heap::sys_init_memory(layout::KERNEL_MEMORY, layout::KERNEL_MEMORY_SIZE);
    let excb = zeroed(EXCB_SIZE);
    entry(table::EXCB, excb, EXCB_SIZE);
    let pcb = zeroed(PCB_SIZE);
    entry(table::PCB, pcb, PCB_SIZE);
    let tcbs = zeroed(threads * TCB_SIZE);
    entry(table::TCB, tcbs, threads * TCB_SIZE);
    for i in 0..threads {
        wr(tcbs + i * TCB_SIZE, 0x1000);
    }
    if tcbs != 0 {
        wr(tcbs, 0x4000);
    }
    wr(pcb, tcbs);
    let evcbs = zeroed(events * EVCB_SIZE);
    entry(table::EVCB, evcbs, events * EVCB_SIZE);
    unsafe {
        EVENTS = events;
        THREADS = threads;
        STACKTOP = stacktop;
    }
    exception::enqueue_syscall_handler(0);
    exception::enqueue_timer_and_vblank_irqs(1);
    exception::init_def_int(3);
    pad::forget();
    cdrom::forget();
    0
}

/// A(9Dh).
pub extern "C" fn get_conf(events: u32, threads: u32, stacktop: u32) -> u32 {
    unsafe {
        wr(events, EVENTS);
        wr(threads, THREADS);
        wr(stacktop, STACKTOP);
    }
    0
}

pub fn events() -> u32 {
    unsafe { EVENTS }
}

pub fn threads() -> u32 {
    unsafe { THREADS }
}

pub fn stacktop() -> u32 {
    unsafe { STACKTOP }
}

pub fn current_tcb() -> u32 {
    rd(rd(TABLES + table::PCB))
}
