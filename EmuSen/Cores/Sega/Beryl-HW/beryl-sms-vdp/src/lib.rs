//! Beryl's Sega Master System VDP: the 315-5124 (the Mark III and the first Master System), the 315-5246 (the Master
//! System II) and the 315-5378 (the Game Gear) in mode 4. The caller owns the clock: `Vdp::run` brings the chip to a
//! time counted in its own clock, the master clock over five, two to a dot. Its picture is CRAM entries, one a pixel,
//! so that the console's DAC levels stay the console's. `compose` is the composition of a line, written to be shared
//! with the Genesis's mode 4. Beryl_SmsVdp.md is the record; MacDonald's "Sega Master System VDP documentation", SMS
//! Power!'s VDP pages and Sega's Mark III manual the sources.

pub mod compose;
mod state;
#[cfg(test)]
mod tests;
pub mod timing;
mod vdp;

pub use vdp::{STATUS_BITS, Vdp};

/// What one chip of the family does differently in mode 4. The TMS9918A's modes and the Genesis's map are stage 4's,
/// each a further constant here (Endou_Plan.md §3.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Variant {
    /// The byte a state names the chip by.
    pub id: u8,
    /// The TMS9918's table registers act as AND masks over mode 4's addresses [R SMS Power!'s "VDP Registers"].
    pub masks: bool,
    /// The 224-line and the 240-line displays [R MacDonald §6; §14 for the Game Gear's lack of 240].
    pub lines_224: bool,
    pub lines_240: bool,
    /// Every sprite on a line zoomed in both directions; else only the first N - 4 of N horizontally (D-6).
    pub zoom_all: bool,
    /// The Game Gear's 64-byte CRAM, written a word at a time through a latch in GG mode [R MacDonald §5].
    pub gg: bool,
    /// Register 0's bit 5, the left column's mask, has an effect [R MacDonald §7: none on the Game Gear].
    pub left_mask: bool,
}

impl Variant {
    /// The 315-5124: the Mark III and the first Master System.
    pub const SMS1: Variant = Variant { id: 1, masks: true, lines_224: false, lines_240: false, zoom_all: false, gg: false, left_mask: true };
    /// The 315-5246: the Master System II.
    pub const SMS2: Variant = Variant { id: 2, masks: false, lines_224: true, lines_240: true, zoom_all: true, gg: false, left_mask: true };
    /// The 315-5378: the Game Gear, "identical to an SMS2 VDP" but for its CRAM and window [R SMS Power!'s "GG VDP"].
    pub const GG: Variant = Variant { id: 3, masks: false, lines_224: true, lines_240: false, zoom_all: true, gg: true, left_mask: false };

    pub fn of_id(id: u8) -> Option<Variant> {
        [Variant::SMS1, Variant::SMS2, Variant::GG].into_iter().find(|v| v.id == id)
    }
}

/// The Game Gear's LCD in GG mode: the central 160 by 144 of a 256 by 192 picture [R SMS Power!'s "GG VDP"]. Its use
/// is the Game Gear's stage (Endou_Plan.md §7, J1).
pub const GG_WINDOW: (u32, u32, u32, u32) = (48, 24, 160, 144);
