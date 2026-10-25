//! The clocks (Sapphire_Plan.md §5.1, Sapphire_Disputes.md D-0): one master, the EE core's cycle, and every other unit
//! an exact integer of it, so nothing drifts. The figures are ps2tek's family, the one place a ruling on D-0 changes.
//! Sapphire_Native.md §20.3 records which timings are the documents' and which are argued.

/// The EE core's clock, 294.912 MHz [R ps2tek "EE Architecture"]: the scheduler's unit.
pub const EE_HZ: u64 = 294_912_000;

/// EE cycles in one cycle of BUSCLK, 147.456 MHz [R EE User's Manual §2.1.2, §4.2 (Tn_MODE.CLKS)].
pub const BUS: u64 = 2;

/// A television standard's line and field, in bus cycles and lines [R ps2tek "EE Timers": "NTSC: 9370 BUSCLK cycles per
/// scanline", "262 scanlines per frame (VBOFF: 240 | VBON: 22)"; PAL 9436, 312, 286 | 26].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Standard {
    pub line: u64,
    pub lines: u64,
    /// Lines with V-blank low, from the field's start; the rest have it high.
    pub active: u64,
}

pub const NTSC: Standard = Standard { line: 9370, lines: 262, active: 240 };
pub const PAL: Standard = Standard { line: 9436, lines: 312, active: 286 };

/// The bus cycles at each line's start during which the stand-in holds H-blank high: a choice [C], about a sixth of
/// the line as an analogue picture's blanking is, until the PCRTC's own timing replaces it at S20.
pub const HBLANK: u64 = 1600;

/// EE cycles in one IOP cycle: the IOP at 36.864 MHz, an eighth of the EE's [R ps2tek "CDVD Reads and Seeks":
/// "IOP_CLOCK is ~36,864,000 Hz"; D-0's family]. Sapphire_Native.md §22.
pub const IOP: u64 = 8;

/// The IOP's timer 0 on its external clock, the pixel clock of 13.5 MHz "regardless of screen mode" [R ps2tek "IOP
/// Timers"], as EE cycles a tick: 294.912 / 13.5 = 8192 / 375 exactly, so its ticks are counted as a ratio.
pub const PIXEL: (u64, u64) = (8192, 375);
