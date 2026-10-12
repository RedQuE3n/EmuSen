//! SIO0's byte exchange as psx-spx's "Controller and Memory Card Signals" and its "Emulation Note" describe the
//! kernel's driver, shared by the pads and the memory cards: a byte sent, about 100 cycles waited, IRQ7 acknowledged in
//! I_STAT and then in CR, the next IRQ7 awaited, the reply read on RXRDY. Diamond_OpenKernel.md §4.13 and §13.

use crate::mem::{rd, rd8, rd16, wr, wr8, wr16};

const DATA: u32 = 0x1F80_1040;
const STAT: u32 = 0x1F80_1044;
const MODE: u32 = 0x1F80_1048;
const CTRL: u32 = 0x1F80_104A;
const BAUD: u32 = 0x1F80_104E;
const I_STAT: u32 = 0x1F80_1070;
/// CR: TXEN, DTR and DSRIEN; bit 13 the port; bit 4 acknowledges.
const SELECT: u32 = 0x1003;
/// About 100 µs of polls, psx-spx's timeout for /ACK, and the 100 cycles waited before acknowledging the last one.
pub const TIMEOUT: u32 = 1000;
/// A memory card's /ACK after a read's 5Ch byte, about 31,000 cycles late ("Memory Card Read/Write Commands").
pub const LATE: u32 = 20_000;
const SETTLE: u32 = 50;

/// SIO0 reset and set to 8 bits at the kernel's 0088h (psx-spx "Signals": about 250 kHz).
pub fn reset() {
    wr16(CTRL, 0x40);
    wr16(MODE, 0x0D);
    wr16(BAUD, 0x88);
}

fn spin(n: u32) {
    for i in 0..n {
        unsafe { core::ptr::read_volatile(&i) };
    }
}

/// Port 0 or 1 selected; the control word the exchanges use.
pub fn select(port: u32) -> u32 {
    let control = SELECT | (port & 1) << 13;
    wr16(CTRL, control);
    control
}

pub fn deselect() {
    wr16(CTRL, 0);
}

/// One byte sent and its reply taken; `wait` polls for the next IRQ7 that long (0 for a last byte, which none
/// follows): the reply, and whether /ACK came.
pub fn exchange(control: u32, byte: u32, wait: u32) -> (u32, bool) {
    wr8(DATA, byte);
    spin(SETTLE);
    wr(I_STAT, !0x80);
    wr16(CTRL, control | 0x10);
    let mut acked = false;
    for _ in 0..wait {
        if rd(I_STAT) & 0x80 != 0 {
            acked = true;
            break;
        }
    }
    for _ in 0..TIMEOUT {
        if rd16(STAT) & 2 != 0 {
            break;
        }
    }
    (rd8(DATA), acked)
}
