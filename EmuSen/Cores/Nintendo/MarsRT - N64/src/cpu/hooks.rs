//! The debugger's hooks: `emusen_native::debug::Hooks`, the tables and logs every Rust core shares, and what MarsRT's
//! observed loop keeps beside them, the return that waits for its delay slot and the observed frame's clock. Nothing here
//! calls the host; a frame that meets a table ends early with a reason. See Mars_Native.md §6.5 and §7.3.

use std::ops::{Deref, DerefMut};

pub use emusen_native::debug::{Call, LOG_CAPACITY, MAX_DEPTH, Range, Write, flag, kind, stop};

/// The processors' coverage widths: the CPU's a 24-bit address, as C#'s `CoverageRegistry` keeps it, the signal processor's
/// its 4 KB IMEM.
pub const WIDTHS: [u32; 2] = [24, 12];
/// The CPU's bitmap, one bit per 24-bit address.
pub const COVERAGE_BYTES: usize = 1 << 21;

#[derive(Clone, Debug)]
pub struct Hooks {
    pub shared: emusen_native::debug::Hooks,
    /// A `jr ra` ran; the pop waits for its delay slot, as C#'s `_returnAfterSlot`.
    pub return_after_slot: bool,
    /// The frame an observed loop is inside, so a stop the host resumes from does not start the frame's clock again.
    pub frame_open: bool,
    pub frame_start: i64,
    pub frame_fields: i64,
}

impl Default for Hooks {
    fn default() -> Self {
        Hooks { shared: emusen_native::debug::Hooks::new(&WIDTHS), return_after_slot: false, frame_open: false, frame_start: 0, frame_fields: 0 }
    }
}

impl Deref for Hooks {
    type Target = emusen_native::debug::Hooks;
    fn deref(&self) -> &Self::Target {
        &self.shared
    }
}

impl DerefMut for Hooks {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.shared
    }
}

impl Hooks {
    /// The CPU's flags set at once, the depths and the signal processor's coverage left as they stand.
    pub fn configure(&mut self, calls: bool, writes: bool, interrupts: bool, each: bool, coverage: bool, profiling: bool) {
        let bit = |on: bool, b: u32| if on { b } else { 0 };
        let rsp = self.shared.coverage.get(1).is_some_and(Option::is_some);
        let flags = bit(calls, flag::CALLS)
            | bit(writes, flag::WRITES)
            | bit(interrupts, flag::INTERRUPTS)
            | bit(each, flag::EACH)
            | bit(profiling, flag::PROFILING)
            | bit(coverage, 1 << flag::COVERAGE)
            | bit(rsp, 1 << (flag::COVERAGE + 1));
        let (target, guard) = (self.depth_target, self.depth_guard);
        self.shared.configure(flags, target, guard);
        self.settle_flags();
    }

    /// A return still waiting for its slot is dropped once calls are no longer tracked, so it cannot pop a frame later.
    pub fn settle_flags(&mut self) {
        if !self.shared.calls {
            self.return_after_slot = false;
        }
    }

    /// `CallObserver`: pushed as the call executes, so its delay slot is already a frame deeper.
    #[inline(never)]
    pub fn note_call(&mut self, source: u64, target: u64) {
        self.shared.note_call(source as u32, target as u32, kind::CALL);
    }

    /// What the observed loop records before each instruction: the coverage bit and the profiler's count.
    #[inline(always)]
    pub fn record(&mut self, pc: u64) {
        self.shared.record(pc as u32);
    }

    /// Why the loop should stop before the instruction at `pc`, every reason that holds; the one-shot reasons are consumed.
    #[inline(always)]
    pub fn stop_before(&mut self, pc: u64) -> u32 {
        self.shared.stop_before(pc as u32)
    }

    /// `Breakpoint.Covers` on the address as C# holds it, a signed 32-bit integer.
    pub fn covers(&self, pc: u64) -> bool {
        self.shared.covers(pc as u32)
    }

    /// One byte of a store, as `IWriteObserver.OnWrite` would receive it: logged where a watch or a data breakpoint covers it.
    #[inline(never)]
    pub fn note_write(&mut self, space: u32, address: u32, value: u8, pc: u64) {
        self.shared.note_write(space, address, value, pc as u32);
    }

    /// The CPU's coverage bitmap while it is armed.
    pub fn coverage_bits(&mut self) -> Option<&mut [u8]> {
        self.shared.coverage.first_mut().and_then(Option::as_mut).map(|c| &mut c.bits[..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_return_with_nothing_open_is_counted_and_the_depth_never_goes_negative() {
        let mut h = Hooks::default();
        h.configure(true, false, false, false, false, false);
        h.note_return();
        assert_eq!((h.depth(), h.unmatched_returns), (0, 1));
        h.note_call(0x10, 0x20);
        h.note_return();
        assert_eq!((h.depth(), h.unmatched_returns), (0, 1));
        assert_eq!(h.calls_log.len(), 3);
    }

    #[test]
    fn the_stack_stops_growing_at_the_cap_and_its_returns_still_pop() {
        let mut h = Hooks::default();
        h.configure(true, false, false, false, false, false);
        for i in 0..MAX_DEPTH as u64 + 3 {
            h.note_call(i, i);
        }
        assert_eq!(h.depth(), MAX_DEPTH);
        h.note_return();
        assert_eq!(h.depth(), MAX_DEPTH - 1);
    }

    #[test]
    fn a_breakpoint_range_compares_as_the_signed_int_the_registry_keeps() {
        let mut h = Hooks::default();
        h.breakpoints.push((0xA400_0040u32 as i32, 0xA400_0048u32 as i32));
        assert!(h.covers(0xFFFF_FFFF_A400_0044));
        assert!(!h.covers(0xFFFF_FFFF_A400_004C));
        assert!(!h.covers(0x0000_0044));
    }

    #[test]
    fn the_profile_charges_each_run_to_the_routine_open_while_it_ran() {
        let mut h = Hooks::default();
        h.configure(true, false, false, false, false, true);
        h.record(0);
        h.record(0);
        h.note_call(0x100, 0x200);
        h.record(0);
        h.note_return();
        h.record(0);
        h.flush();
        assert_eq!(h.profile, [(0, 3), (0x200, 1)]);
    }

    #[test]
    fn a_stop_consumes_its_one_shot_reasons_and_reports_every_reason_at_once() {
        let mut h = Hooks::default();
        h.configure(true, false, false, true, false, false);
        h.data_pending = true;
        h.interrupt_taken = true;
        assert_eq!(h.stop_before(0), stop::EACH | stop::DATA | stop::INTERRUPT);
        assert_eq!(h.stop_before(0), stop::EACH);
        h.each = false;
        h.depth_target = 0;
        assert_eq!(h.stop_before(0), stop::DEPTH);
        h.note_call(1, 2);
        assert_eq!(h.stop_before(0), stop::FRAME);
    }

    #[test]
    fn a_write_is_logged_where_a_range_covers_it_and_only_a_break_range_stops() {
        let mut h = Hooks::default();
        h.watch_ranges.push(Range { space: 0, start: 0x10, end: 0x13 });
        h.break_ranges.push(Range { space: 1, start: 0x20, end: 0x20 });
        h.note_write(0, 0x13, 7, 0xA400_0040);
        assert_eq!(h.writes_log.len(), 1);
        assert!(!h.data_pending);
        h.note_write(0, 0x14, 7, 0);
        assert_eq!(h.writes_log.len(), 1);
        h.note_write(1, 0x20, 9, 0);
        assert_eq!(h.writes_log.len(), 2);
        assert!(h.data_pending);
        assert_eq!(h.writes_log[0], Write { space: 0, address: 0x13, value: 7, pc: 0xA400_0040 });
    }

    #[test]
    fn configure_keeps_the_signal_processor_s_coverage_and_drops_a_waiting_return_with_calls_off() {
        let mut h = Hooks::default();
        h.shared.configure(1 << (flag::COVERAGE + 1), i32::MIN, -1);
        h.configure(true, false, false, false, true, false);
        assert!(h.coverage[1].is_some() && h.coverage_bits().is_some_and(|b| b.len() == COVERAGE_BYTES));
        h.return_after_slot = true;
        h.configure(false, false, false, false, false, false);
        assert!(!h.return_after_slot && h.coverage[0].is_none() && h.coverage[1].is_some());
    }
}
