//! Port 1's digital buttons through the kernel's pad buffers (InitPAD2, StartPAD2), active-low as psx-spx's
//! "Controllers - Standard Digital/Analog Controllers" gives them, here active-high, with the presses of each frame.

use crate::bios;
use diamond_guest::mem::{rd8, wr8};

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

static mut BUFFERS: [[u8; 0x22]; 2] = [[0; 0x22]; 2];

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

/// The grammar's repeat (EmuSen_LunaP_Kit.md): the directions and shoulders press again after 400 ms held and then
/// every 80 ms, counted in frames; nothing else repeats.
pub struct Repeat {
    held: u32,
    frames: u32,
}

const REPEATS: u32 = UP | DOWN | LEFT | RIGHT | L1 | R1 | L2 | R2;
const FIRST: u32 = 24;
const NEXT: u32 = 5;

impl Repeat {
    pub const fn new() -> Repeat {
        Repeat { held: 0, frames: 0 }
    }

    /// The presses of this frame, from the buttons held now.
    pub fn step(&mut self, now: u32) -> u32 {
        let new = now & !self.held;
        if now & REPEATS != self.held & REPEATS || new != 0 {
            self.frames = 0;
        } else if now & REPEATS != 0 {
            self.frames += 1;
        }
        self.held = now;
        let again = self.frames >= FIRST && (self.frames - FIRST) % NEXT == 0;
        new | if again { now & REPEATS } else { 0 }
    }
}
