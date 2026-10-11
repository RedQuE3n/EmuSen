//! psx-spx's "BIOS Joypad Functions": the buffers, the PadCardIrq element at priority 2 and PAD_init2's and PAD_dr's
//! interface. Reading the pads over SIO0 is stage 6's: until then each vblank marks both buffers FFh, no pad
//! answering. Diamond_OpenKernel.md §4.13.

use crate::entry;
use crate::exception::{self, acknowledge, pending};
use crate::mem::{rd8, wr, wr8};

static mut BUFFERS: [u32; 2] = [0; 2];
static mut ENABLED: u32 = 0;
static mut STARTED: bool = false;
static mut CLEAR: u32 = 1;
static mut BUTTON_DEST: u32 = 0;
static mut HIDDEN: [[u8; 0x22]; 2] = [[0; 0x22]; 2];
static mut ELEMENT: [u32; 4] = [0; 4];

/// SetConf's reset forgets the element with the chains.
pub fn forget() {
    unsafe { STARTED = false };
}

/// B(12h): the buffers zero-filled to their sizes and remembered, the pad enabled, the element not yet queued.
pub extern "C" fn init_pad2(buf1: u32, size1: u32, buf2: u32, size2: u32) -> u32 {
    for (b, n) in [(buf1, size1), (buf2, size2)] {
        for i in 0..n {
            wr8(b + i, 0);
        }
    }
    unsafe {
        BUFFERS = [buf1, buf2];
        ENABLED = 1;
    }
    1
}

/// B(13h).
pub extern "C" fn start_pad2() -> u32 {
    unsafe {
        if !STARTED {
            let e = &raw mut ELEMENT as u32;
            wr(e + 4, 0);
            wr(e + 8, pad_handler as *const () as u32);
            exception::sys_enq_int_rp(2, e);
            STARTED = true;
        }
    }
    1
}

/// B(14h).
pub extern "C" fn stop_pad2() -> u32 {
    unsafe {
        if STARTED {
            exception::sys_deq_int_rp(2, &raw const ELEMENT as u32);
            STARTED = false;
        }
    }
    1
}

/// B(15h): only types 20000000h and 20000001h; the kernel's own buffers, FFh-filled and then InitPAD2's.
pub extern "C" fn pad_init2(kind: u32, button_dest: u32, _a: u32, _b: u32) -> u32 {
    if kind != 0x2000_0000 && kind != 0x2000_0001 {
        return 0;
    }
    let (b1, b2) = unsafe { (&raw mut HIDDEN[0] as u32, &raw mut HIDDEN[1] as u32) };
    for i in 0..0x22 {
        wr8(b1 + i, 0xFF);
        wr8(b2 + i, 0xFF);
    }
    init_pad2(b1, 0x22, b2, 0x22);
    start_pad2();
    unsafe { BUTTON_DEST = button_dest };
    2
}

fn buttons(buffer: u32) -> u32 {
    if buffer == 0 || rd8(buffer) != 0 {
        return 0xFFFF;
    }
    let half = rd8(buffer + 2) << 8 | rd8(buffer + 3);
    match rd8(buffer + 1) {
        0x41 => half,
        0x23 => half | 0x07C7,
        _ => 0xFFFF,
    }
}

/// B(16h): pad 1's buttons low, pad 2's high, each with its first byte high; stored at button_dest even when it is 0.
pub extern "C" fn pad_dr() -> u32 {
    let (b1, b2) = unsafe { (&raw const HIDDEN[0] as u32, &raw const HIDDEN[1] as u32) };
    let v = buttons(b1) | buttons(b2) << 16;
    wr(unsafe { BUTTON_DEST }, v);
    v
}

/// B(5Bh): whether the pad handler acknowledges vblank and returns; the old flag.
pub extern "C" fn change_clear_pad(flag: u32) -> u32 {
    unsafe {
        let old = CLEAR;
        CLEAR = flag;
        old
    }
}

/// B(4Ah): its pad flag only; the cards are phase 2.
pub extern "C" fn init_card2(pad_enable: u32) -> u32 {
    unsafe { ENABLED = pad_enable };
    0
}

extern "C" fn pad_handler() -> u32 {
    if pending() & 1 == 0 {
        return 0;
    }
    unsafe {
        if ENABLED != 0 {
            for b in BUFFERS {
                if b != 0 {
                    wr8(b, 0xFF);
                }
            }
        }
        if BUTTON_DEST != 0 {
            pad_dr();
        }
        if CLEAR != 0 {
            acknowledge(0);
            entry::return_from_exception()
        }
    }
    0
}
