//! Beryl's Sega Master System VDP: the 315-5124 (the Mark III and the first Master System), the 315-5246 (the Master
//! System II), the 315-5378 (the Game Gear) and the TMS9918A they descend from (the SG-1000), in mode 4 and the
//! TMS9918's modes. The caller owns the clock: `Vdp::run` brings the chip to a time counted in its own clock, the master
//! clock over five, two to a dot. Its picture is the chip's colour codes, one a pixel, so that the console's DAC levels
//! stay the console's. `compose` is mode 4's composition of a line, which the Genesis's mode 4 shares, and `tms` the
//! TMS9918's. Beryl_SmsVdp.md is the record; MacDonald's VDP documents, SMS Power!'s VDP pages, Sega's Mark III
//! manual, Texas Instruments' TMS9918A data manual and Young's TMS9918A document the sources.

pub mod compose;
mod state;
#[cfg(test)]
mod tests;
pub mod timing;
pub mod tms;
mod vdp;

pub use vdp::{STATUS_BITS, Vdp};

/// What one chip of the family does differently (Beryl_SmsVdp.md §2.2). The Genesis's is for `compose` alone: its chip
/// is its console's own (Endou_Plan.md §3.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Variant {
    /// The byte a state names the chip by.
    pub id: u8,
    /// The TMS9918's table registers act as AND masks over mode 4's addresses [R SMS Power!'s "VDP Registers"], and over
    /// the Graphics II tables' [R MacDonald §15; Young §3.3].
    pub masks: bool,
    /// The 224-line and the 240-line displays [R MacDonald §6; §14 for the Game Gear's lack of 240].
    pub lines_224: bool,
    pub lines_240: bool,
    /// Register 1's bit 0 zooms mode 4's sprites [R MacDonald §13: on the Genesis "this bit does nothing"].
    pub zoom: bool,
    /// Every sprite on a line zoomed in both directions; else only the first N - 4 of N horizontally (D-6).
    pub zoom_all: bool,
    /// The Game Gear's 64-byte CRAM, written a word at a time through a latch in GG mode [R MacDonald §5].
    pub gg: bool,
    /// Register 0's bit 5, the left column's mask, has an effect [R MacDonald §7: none on the Game Gear].
    pub left_mask: bool,
    /// The fine scroll's first pixels show the backdrop (D-7); else the wrapped column, as Nephrite draws the Genesis's.
    pub fine_blank: bool,
    /// The right columns' lock by the column counter [R MacDonald §9]; else by the pixel, as Nephrite draws the Genesis's.
    pub lock_by_counter: bool,
    /// The TMS9918A itself: no mode 4, eight registers, the status's fifth sprite and the 4K VRAM mode [R TI §2].
    pub tms: bool,
}

impl Variant {
    /// The 315-5124: the Mark III and the first Master System.
    pub const SMS1: Variant = Variant { id: 1, masks: true, lines_224: false, lines_240: false, zoom_all: false, ..Variant::SMS2 };
    /// The 315-5246: the Master System II.
    pub const SMS2: Variant = Variant {
        id: 2,
        masks: false,
        lines_224: true,
        lines_240: true,
        zoom: true,
        zoom_all: true,
        gg: false,
        left_mask: true,
        fine_blank: true,
        lock_by_counter: true,
        tms: false,
    };
    /// The 315-5378: the Game Gear, "identical to an SMS2 VDP" but for its CRAM and window [R SMS Power!'s "GG VDP"].
    pub const GG: Variant = Variant { id: 3, lines_240: false, gg: true, left_mask: false, ..Variant::SMS2 };
    /// The TMS9918A: the SG-1000's, with the 315-5124's masks in Graphics II [R TI; Young; MacDonald §15].
    pub const TMS9918A: Variant = Variant { id: 4, tms: true, ..Variant::SMS1 };
    /// The 315-5313's mode 4, for `compose`: no zoom, no taller displays, no masks, and Nephrite's fine scroll and lock
    /// (Nephrite_Disputes.md D-7, Beryl_SmsVdp.md §9).
    pub const GENESIS: Variant = Variant { id: 5, lines_224: false, lines_240: false, zoom: false, fine_blank: false, lock_by_counter: false, ..Variant::SMS2 };

    /// The chips `Vdp` models; the Genesis's is not one.
    pub fn of_id(id: u8) -> Option<Variant> {
        [Variant::SMS1, Variant::SMS2, Variant::GG, Variant::TMS9918A].into_iter().find(|v| v.id == id)
    }
}

/// The Game Gear's LCD in GG mode: the central 160 by 144 of a 256 by 192 picture [R SMS Power!'s "GG VDP"; Sega's Game
/// Gear VDP manual, lines 24 and 167 the LCD's top and bottom], as (x, y, width, height). A 224-line display's is lines
/// 40-183, `gg_window_top`'s (Endou_Disputes.md D-21); Endou's `gg` module draws it.
pub const GG_WINDOW: (u32, u32, u32, u32) = (48, 24, 160, 144);

/// The display's line at the top of the Game Gear's LCD in GG mode: the window stays where it is in the frame, so a
/// 224-line display, which starts 16 lines higher, shows it from line 40 (Endou_Disputes.md D-21).
pub const fn gg_window_top(height: u32) -> u32 {
    GG_WINDOW.1 + (height - 192) / 2
}
