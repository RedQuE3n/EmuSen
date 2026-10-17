//! The drive, asked at the controller's host interface (psx-spx "CDROM Controller I/O Ports") while the kernel owns no
//! operation and so leaves the controller alone (Diamond_OpenKernel_Phase2.md §12.1): GetID's answer read by its flag
//! bits, never by whole bytes (Diamond_Shell.md §2). IRQ2 stays masked; the interrupt flags are polled.

use diamond_guest::mem::{rd8, wr8};

const HSTS: u32 = 0xBF80_1800;
const COMMAND: u32 = 0xBF80_1801;
const RESULT: u32 = 0xBF80_1801;
const HINT: u32 = 0xBF80_1803;
const PATIENCE: u32 = 2_000_000;

/// What the drive holds, as hand-over v1 numbers it in Exec's second parameter.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Held {
    Empty,
    Audio,
    Refused,
    /// A licensed data disc: the shell returns and the kernel boots it.
    Data,
    /// Spinning up, the lid open, or no answer yet: asked again later.
    Unknown,
}

impl Held {
    pub fn from_reason(reason: u32) -> Held {
        match reason {
            1 => Held::Audio,
            2 => Held::Refused,
            _ => Held::Empty,
        }
    }
}

/// The interrupt the controller raised, 0 for none before patience runs out, its response read and acknowledged.
fn response(out: &mut [u8; 8]) -> u32 {
    wr8(HSTS, 1);
    let mut n = 0;
    let int = loop {
        let int = rd8(HINT) & 7;
        if int != 0 || n >= PATIENCE {
            break int;
        }
        n += 1;
    };
    let mut i = 0;
    while rd8(HSTS) & 0x20 != 0 && i < 16 {
        let b = rd8(RESULT) as u8;
        if i < 8 {
            out[i] = b;
        }
        i += 1;
    }
    wr8(HSTS, 1);
    wr8(HINT, 0x07);
    int
}

/// GetID (1Ah): INT3 then INT2 for a licensed data disc, INT5 with the flags otherwise (psx-spx "GetID").
pub fn identify() -> Held {
    let mut r = [0u8; 8];
    wr8(HSTS, 1);
    wr8(HINT, 0x1F);
    wr8(HINT, 0x40);
    wr8(HSTS, 0);
    let mut n = 0;
    while rd8(HSTS) & 0x80 != 0 && n < PATIENCE {
        n += 1;
    }
    wr8(COMMAND, 0x1A);
    match response(&mut r) {
        3 => {}
        5 if r[0] & 0x10 == 0 && r[1] == 0x80 => return Held::Unknown,
        _ => return Held::Unknown,
    }
    match response(&mut r) {
        2 => Held::Data,
        5 if r[1] & 0x40 != 0 => Held::Empty,
        5 if r[1] & 0x10 != 0 => Held::Audio,
        5 if r[1] == 0x80 && r[0] & 0x08 != 0 => Held::Refused,
        _ => Held::Unknown,
    }
}
