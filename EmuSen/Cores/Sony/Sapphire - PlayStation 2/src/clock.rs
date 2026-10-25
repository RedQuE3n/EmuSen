//! The clocks (Sapphire_Plan.md §5.1, Sapphire_Disputes.md D-0): one master, the EE core's cycle, and every other unit
//! an exact integer of it, so nothing drifts. The figures are ps2tek's family, the one place a ruling on D-0 changes.
//! Sapphire_Native.md §20.3 records which timings are the documents' and which are argued.

/// The EE core's clock, 294.912 MHz [R ps2tek "EE Architecture"]: the scheduler's unit.
pub const EE_HZ: u64 = 294_912_000;

/// EE cycles in one cycle of BUSCLK, 147.456 MHz [R EE User's Manual §2.1.2, §4.2 (Tn_MODE.CLKS)].
pub const BUS: u64 = 2;

/// The television standards by their old names, now the PCRTC's video modes (Sapphire_Native.md §29.3), which own the
/// lines and fields; the board is made with one.
pub use crate::pcrtc::timing::{Mode as Standard, NTSC, PAL};

/// The PCRTC's video clock VCK in NTSC, PAL and 480p, 54 MHz (four of the 13.5 MHz pixel clock), as VCK ticks per EE
/// cycle: 294.912 MHz × 375 / 2048 = 54 MHz exactly [A, Sapphire_Native.md §29.3].
pub const VCK_54: (u64, u64) = (375, 2048);

/// VCK in 1080i, 74.25 MHz: 294.912 MHz × 12375 / 49152 [A, §29.3].
pub const VCK_74: (u64, u64) = (12375, 49152);

/// EE cycles in one IOP cycle: the IOP at 36.864 MHz, an eighth of the EE's [R ps2tek "CDVD Reads and Seeks":
/// "IOP_CLOCK is ~36,864,000 Hz"; D-0's family]. Sapphire_Native.md §22.
pub const IOP: u64 = 8;

/// The IOP's timer 0 on its external clock, the pixel clock of 13.5 MHz "regardless of screen mode" [R ps2tek "IOP
/// Timers"], as EE cycles a tick: 294.912 / 13.5 = 8192 / 375 exactly, so its ticks are counted as a ratio.
pub const PIXEL: (u64, u64) = (8192, 375);
