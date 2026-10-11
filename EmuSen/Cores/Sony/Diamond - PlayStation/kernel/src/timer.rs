//! psx-spx's "BIOS Timer Functions" B(02h)-B(06h), as the register writes it gives. Diamond_OpenKernel.md §4.12.

use crate::layout::io;
use crate::mem::{rd, rd16, wr, wr16};

fn port(t: u32, offset: u32) -> u32 {
    io::TIMER + t * 0x10 + offset
}

/// B(02h).
pub extern "C" fn init_timer(t: u32, reload: u32, flags: u32) -> u32 {
    if t > 2 {
        return 0;
    }
    wr16(port(t, 4), 0);
    wr16(port(t, 8), reload);
    let mut mode = if flags & 1 << 4 == 0 { 0x48 } else { 0x49 };
    if flags & 1 == 0 {
        mode |= 0x100;
    }
    if flags & 1 << 12 != 0 {
        mode |= 0x10;
    }
    wr16(port(t, 4), mode);
    1
}

/// B(03h).
pub extern "C" fn get_timer(t: u32) -> u32 {
    if t > 2 { 0 } else { rd16(port(t, 0)) }
}

fn irq_bit(t: u32) -> Option<u32> {
    match t {
        0..=2 => Some(1 << (4 + t)),
        3 => Some(1),
        _ => None,
    }
}

/// B(04h): 1 for the timers, 0 for vblank, as documented.
pub extern "C" fn enable_timer_irq(t: u32) -> u32 {
    if let Some(b) = irq_bit(t) {
        wr(io::I_MASK, rd(io::I_MASK) | b);
    }
    (t <= 2) as u32
}

/// B(05h).
pub extern "C" fn disable_timer_irq(t: u32) -> u32 {
    if let Some(b) = irq_bit(t) {
        wr(io::I_MASK, rd(io::I_MASK) & !b);
    }
    1
}

/// B(06h).
pub extern "C" fn restart_timer(t: u32) -> u32 {
    if t > 2 {
        return 0;
    }
    wr16(port(t, 0), 0);
    1
}
