//! Port 1's digital buttons through the kernel's pad buffers (InitPAD2, StartPAD2), active-low as psx-spx's
//! "Controllers - Standard Digital/Analog Controllers" gives them, here active-high, with the presses of each frame.

use crate::bios;
use diamond_guest::mem::{rd8, wr8};

pub const SELECT: u32 = 1 << 0;
pub const START: u32 = 1 << 3;
pub const UP: u32 = 1 << 4;
pub const RIGHT: u32 = 1 << 5;
pub const DOWN: u32 = 1 << 6;
pub const LEFT: u32 = 1 << 7;
pub const L2: u32 = 1 << 8;
pub const R2: u32 = 1 << 9;
pub const L1: u32 = 1 << 10;
pub const R1: u32 = 1 << 11;
pub const TRIANGLE: u32 = 1 << 12;
pub const CIRCLE: u32 = 1 << 13;
pub const CROSS: u32 = 1 << 14;
pub const SQUARE: u32 = 1 << 15;

static mut BUFFERS: [[u8; 0x22]; 2] = [[0; 0x22]; 2];
static mut HELD: u32 = 0;

pub fn start() {
    unsafe {
        let (p1, p2) = (&raw mut BUFFERS[0] as u32, &raw mut BUFFERS[1] as u32);
        bios::init_pad2(p1, 0x22, p2, 0x22);
        bios::start_pad2();
        for i in 0..0x22 {
            wr8(p1 + i, 0xFF);
        }
    }
}

pub fn stop() {
    unsafe { bios::stop_pad2() };
}

/// The buttons held now: none while the buffer's status byte says no pad answered.
pub fn held() -> u32 {
    let b = unsafe { &raw const BUFFERS[0] } as u32;
    if rd8(b) != 0 { 0 } else { !(rd8(b + 2) | rd8(b + 3) << 8) & 0xFFFF }
}

/// The buttons pressed since the last call.
pub fn pressed() -> u32 {
    let now = held();
    let p = now & !unsafe { HELD };
    unsafe { HELD = now };
    p
}
