//! A vector unit's micro mode as the rest of the machine drives it: the interface stage S12 defines and S13 fills
//! (Sapphire_Native.md §21.6). The EE starts VU0 with VCALLMS, VCALLMSR, and VU1 by writing CMSAR1; it waits for a running
//! microprogram before a macro instruction or an interlocked transfer (VU User's Manual §1.3, §5.4.5, §5.4.6). Times are
//! EE cycles; an implementation runs its unit lazily, as far as each call asks.

use super::Units;

pub trait Micro {
    /// Begins the microprogram at doubleword `addr` at cycle `at`; false when micro mode is not built.
    fn start(&mut self, units: &mut Units, addr: u16, at: u64) -> bool;
    /// Runs the microprogram as far as cycle `t`.
    fn run_to(&mut self, units: &mut Units, t: u64);
    /// Whether a microprogram runs, as of the last call.
    fn busy(&self) -> bool;
    /// Runs the microprogram to its end (the E bit's delay slot) and returns that cycle, or `at` when none runs.
    fn finish(&mut self, units: &mut Units, at: u64) -> u64;
    /// Runs the microprogram to its first pair with the M bit set, or its end, and returns that cycle, from which
    /// QMTC2.I and CTC2.I may write (§5.4.6); `at` when none runs.
    fn release(&mut self, units: &mut Units, at: u64) -> u64;
    /// FBRST's force break (§1.3.3).
    fn force_break(&mut self, units: &mut Units, at: u64);
    /// FBRST's reset (§1.3).
    fn reset(&mut self);
    /// The unit's VPU-STAT bits but DIV, in VU0's positions (§5.1.3).
    fn stat(&self) -> u32;
}

/// Micro mode before S13: never running, and refusing a start, which halts the interpreter with the starting word.
#[derive(Clone, Copy, Debug, Default)]
pub struct Unbuilt;

impl Micro for Unbuilt {
    fn start(&mut self, _: &mut Units, _: u16, _: u64) -> bool {
        false
    }
    fn run_to(&mut self, _: &mut Units, _: u64) {}
    fn busy(&self) -> bool {
        false
    }
    fn finish(&mut self, _: &mut Units, at: u64) -> u64 {
        at
    }
    fn release(&mut self, _: &mut Units, at: u64) -> u64 {
        at
    }
    fn force_break(&mut self, _: &mut Units, _: u64) {}
    fn reset(&mut self) {}
    fn stat(&self) -> u32 {
        0
    }
}
