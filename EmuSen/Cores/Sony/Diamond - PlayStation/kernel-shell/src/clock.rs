//! The shell's frame: a vblank, known by the kernel's root counter 3 event (class F2000003h, spec 2, psx-spx "Event
//! Classes"), which the kernel delivers on each vblank once the pads have enabled its interrupt.

use crate::bios;

static mut VBLANK: u32 = 0;
static mut FRAMES: u32 = 0;

pub fn start() {
    unsafe {
        VBLANK = bios::open_event(0xF200_0003, 2, bios::EV_READY, 0);
        bios::enable_event(VBLANK);
    }
}

pub fn stop() {
    unsafe { bios::close_event(VBLANK) };
}

/// The next vblank waited for; the frames counted since the shell started.
pub fn frame() -> u32 {
    unsafe {
        while bios::test_event(VBLANK) == 0 {}
        FRAMES += 1;
        FRAMES
    }
}

pub fn frames() -> u32 {
    unsafe { FRAMES }
}
