//! psx-spx's "BIOS Joypad Functions": the buffers, the PadCardIrq element at priority 2 and PAD_init2's and PAD_dr's
//! interface; on each vblank both ports read over SIO0 as "Controller and Memory Card Signals" describes the driver.
//! Diamond_OpenKernel.md §4.13.

use crate::entry;
use crate::exception::{self, acknowledge, pending};
use crate::mem::{rd, rd8, wr, wr8};
use crate::{card, sio};

const I_MASK: u32 = 0x1F80_1074;

static mut BUFFERS: [u32; 2] = [0; 2];
static mut ENABLED: u32 = 0;
static mut STARTED: bool = false;
static mut PADS: bool = false;
static mut CLEAR: u32 = 1;
static mut BUTTON_DEST: u32 = 0;
static mut HIDDEN: [[u8; 0x22]; 2] = [[0; 0x22]; 2];
static mut ELEMENT: [u32; 4] = [0; 4];

/// SetConf's reset forgets the element with the chains.
pub fn forget() {
    unsafe {
        STARTED = false;
        PADS = false;
    }
    card::forget();
}

/// B(12h): the buffers zero-filled to their sizes and remembered, the pad enabled, the element not yet queued; SIO0
/// reset and set to 8 bits at the kernel's 0088h (psx-spx "Signals": about 250 kHz) [C on where].
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
    sio::reset();
    1
}

/// B(13h): the element queued and vblank enabled in I_MASK, without which a program that calls only PAD_init2, as
/// Peter Lemon's `Cube` does on a console, would never see its buttons (Diamond_OpenKernel.md §4.13) [A].
pub extern "C" fn start_pad2() -> u32 {
    unsafe { PADS = true };
    queue();
    1
}

/// The element the pads and the cards share, queued once at priority 2, with vblank enabled.
pub fn queue() {
    wr(I_MASK, rd(I_MASK) | 1);
    unsafe {
        if !STARTED {
            let e = &raw mut ELEMENT as u32;
            wr(e + 4, 0);
            wr(e + 8, pad_handler as *const () as u32);
            exception::sys_enq_int_rp(2, e);
            STARTED = true;
        }
    }
}

/// The element removed once neither the pads nor the cards use it.
pub fn unqueue() {
    unsafe {
        if STARTED && !PADS && !card::started() {
            exception::sys_deq_int_rp(2, &raw const ELEMENT as u32);
            STARTED = false;
        }
    }
}

/// Whether the shared element is queued.
pub fn queued() -> bool {
    unsafe { STARTED }
}

/// B(14h).
pub extern "C" fn stop_pad2() -> u32 {
    unsafe { PADS = false };
    unqueue();
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

/// B(4Ah)'s pad half: InitCARD2's flag enables the pads' reads.
pub fn enable(pad_enable: u32) {
    unsafe { ENABLED = pad_enable };
}

/// A controller's read into `buffer` (psx-spx "B(12h)"): status 00h and the ID, then the halfwords its ID counts, up
/// to 22h bytes in all; status FFh when no /ACK came or the second ID byte was not 5Ah.
fn read_port(port: u32, buffer: u32) {
    let control = sio::select(port);
    let mut status = 0xFF;
    let (_, present) = sio::exchange(control, 0x01, sio::TIMEOUT);
    if present {
        let (id, more) = sio::exchange(control, 0x42, sio::TIMEOUT);
        let (second, data) = sio::exchange(control, 0x00, sio::TIMEOUT);
        if more && data && second == 0x5A {
            let halfwords = if id & 0xF == 0 { 16 } else { id & 0xF };
            for i in 0..2 * halfwords {
                let last = i + 1 == 2 * halfwords;
                let (v, _) = sio::exchange(control, 0x00, if last { 0 } else { sio::TIMEOUT });
                wr8(buffer + 2 + i, v);
            }
            wr8(buffer + 1, id);
            status = 0;
        }
    }
    wr8(buffer, status);
    sio::deselect();
}

extern "C" fn pad_handler() -> u32 {
    if pending() & 1 == 0 {
        return 0;
    }
    unsafe {
        if ENABLED != 0 {
            for (port, b) in BUFFERS.into_iter().enumerate() {
                if b != 0 {
                    read_port(port as u32, b);
                }
            }
        }
        card::on_vblank();
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
