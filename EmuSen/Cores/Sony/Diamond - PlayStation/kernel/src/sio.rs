//! SIO0's byte exchange as psx-spx's "Controller and Memory Card Signals" and its "Emulation Note" describe the
//! kernel's driver, shared by the pads and the memory cards: a byte sent, about 100 cycles waited, IRQ7 acknowledged in
//! I_STAT and then in CR, the next IRQ7 awaited, the reply read on RXRDY. Inlined into each caller, the pads' in the
//! kernel's RAM and the cards' in the ROM. Diamond_OpenKernel.md §4.13, Diamond_OpenKernel_Phase2.md §13 and §25.3.

use crate::mem::{rd, rd8, rd16, wr, wr8, wr16};

const DATA: u32 = 0x1F80_1040;
const STAT: u32 = 0x1F80_1044;
const MODE: u32 = 0x1F80_1048;
const CTRL: u32 = 0x1F80_104A;
const BAUD: u32 = 0x1F80_104E;
const I_STAT: u32 = 0x1F80_1070;
/// CR: TXEN, DTR and DSRIEN; bit 13 the port; bit 4 acknowledges.
const SELECT: u32 = 0x1003;
/// Cycles a bit takes at the kernel's rate, BR 0088h with MR's factor 1 ("Serial Interfaces (SIO)").
const BIT: u32 = 0x88;
/// Cycles of one poll of IRQ7 or RXRDY from the kernel's RAM, 9.5 as Diamond measures them, taken low so that no wait
/// ends early (Diamond_OpenKernel_Phase2.md §25.3) [M].
const POLL: u32 = 9;
/// Polls for /ACK until 100 µs (3,387 cycles) past the byte's last clock, psx-spx's timeout for the kernel's driver
/// ("Controller and Memory Card Signals"); a device acknowledges about 1,500 cycles after a byte begins ("Timings").
pub const TIMEOUT: u32 = (8 * BIT + 3_387) / POLL;
/// A memory card's /ACK after a read's 5Ch byte, about 31,000 cycles late ("Memory Card Read/Write Commands").
pub const LATE: u32 = 20_000;
/// Spins of about 13 cycles each, the "100 cycles or so" the driver waits before acknowledging an old IRQ7 ("Emulation
/// Note") [M].
const SETTLE: u32 = 8;

/// SIO0 reset and set to 8 bits at the kernel's 0088h (psx-spx "Signals": about 250 kHz).
#[inline(always)]
pub fn reset() {
    wr16(CTRL, 0x40);
    wr16(MODE, 0x0D);
    wr16(BAUD, 0x88);
}

#[inline(always)]
fn spin(n: u32) {
    for i in 0..n {
        unsafe { core::ptr::read_volatile(&i) };
    }
}

/// Port 0 or 1 selected; the control word the exchanges use.
#[inline(always)]
pub fn select(port: u32) -> u32 {
    let control = SELECT | (port & 1) << 13;
    wr16(CTRL, control);
    control
}

#[inline(always)]
pub fn deselect() {
    wr16(CTRL, 0);
}

/// One byte sent and its reply taken; `wait` polls for the next IRQ7 that long (0 for a last byte, which none
/// follows): the reply, and whether /ACK came.
#[inline(always)]
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
