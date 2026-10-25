//! A vector unit's micro mode as the rest of the machine drives it: the interface stage S12 defined and S13 fills with
//! `super::pipe::Pipe` (Sapphire_Native.md §21.6, §23). The EE starts VU0 with VCALLMS and VCALLMSR, VU1 by writing
//! CMSAR1, and VIF starts either with its MSCAL family (S14); the EE waits for a running microprogram before a macro
//! instruction or an interlocked transfer (VU User's Manual §1.3, §5.4.5, §5.4.6). Times are EE cycles, which are the
//! VU's (ps2tek, "VU Architecture": "294.912 MHz (same as EE)"); a pair's time is its T stage. An implementation runs
//! its unit lazily, as far as each call asks.

use super::Units;

pub trait Micro {
    /// Begins the microprogram at doubleword `addr`, its first pair's T stage at cycle `first`; false when micro mode
    /// cannot start.
    fn start(&mut self, units: &mut Units, addr: u16, first: u64) -> bool;
    /// Runs the microprogram as far as cycle `t`, and lands the flags its pairs left for later cycles up to `t`.
    fn run_to(&mut self, units: &mut Units, t: u64);
    /// Whether a microprogram runs, as of the last call.
    fn busy(&self) -> bool;
    /// Runs the microprogram to its end (the E bit's delay slot, a D or T bit, or a bound) and returns the T stage of
    /// its last pair, or of the last microprogram's when none runs; none when none has run since a reset.
    fn finish(&mut self, units: &mut Units, at: u64) -> Option<u64>;
    /// Runs the microprogram to its first pair with the M bit set, or its end, and returns the cycle QMTC2.I and CTC2.I
    /// may issue in (§5.4.6); none when none runs.
    fn release(&mut self, units: &mut Units, at: u64) -> Option<u64>;
    /// FBRST's force break (§1.3.3).
    fn force_break(&mut self, units: &mut Units, at: u64);
    /// FBRST's reset (§1.3).
    fn reset(&mut self);
    /// The unit's VPU-STAT bits VBS, VDS, VTS, VFS and VGW, in VU0's positions (§5.1.3); the divider's and the EFU's
    /// the caller reads off the units.
    fn stat(&self) -> u32;
    /// The T stage of the last pair run since a reset, without running anything (VIF's waits, S14).
    fn last(&self) -> Option<u64> {
        None
    }
    /// Whether `run_to(t)` would do nothing but move the unit's clock, and if so that move made: no microprogram
    /// running and nothing left to land (the board's shortcut, Sapphire_Native.md §35.7).
    fn idle_to(&mut self, _units: &mut Units, _t: u64) -> bool {
        false
    }
}

/// Micro mode not built: never running, and refusing a start, which halts the interpreter with the starting word; kept
/// for a machine that leaves micro mode out.
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
    fn finish(&mut self, _: &mut Units, _: u64) -> Option<u64> {
        None
    }
    fn release(&mut self, _: &mut Units, _: u64) -> Option<u64> {
        None
    }
    fn force_break(&mut self, _: &mut Units, _: u64) {}
    fn reset(&mut self) {}
    fn stat(&self) -> u32 {
        0
    }
    fn idle_to(&mut self, _: &mut Units, _: u64) -> bool {
        true
    }
}
