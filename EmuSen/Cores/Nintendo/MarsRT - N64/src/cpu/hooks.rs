//! The debugger's hooks: tables the host pushes down, and what an observed frame records for it to read back.
//! Nothing here calls the host; a frame that meets a table ends early with a reason. See Mars_Native.md §6.5.

use std::collections::HashMap;

/// Why an observed frame stopped, as bits; zero is the field's end.
pub mod stop {
    pub const FRAME: u32 = 0;
    /// The next instruction's address is in a breakpoint range.
    pub const BREAKPOINT: u32 = 1;
    /// The host asked to stop before every instruction.
    pub const EACH: u32 = 2;
    /// The call stack's depth reached the target, or passed the guard.
    pub const DEPTH: u32 = 4;
    /// A store landed in a data-breakpoint range; the store has run.
    pub const DATA: u32 = 8;
    /// An interrupt was entered; the next instruction is the handler's first.
    pub const INTERRUPT: u32 = 16;
    /// A log is full and must be drained.
    pub const RING: u32 = 32;
}

/// A stretch of one memory, numbered as `ffi::space` numbers memories, both ends included.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    pub space: u32,
    pub start: u32,
    pub end: u32,
}

/// One byte a processor store left, in the memory it landed in, and the storing instruction's address.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Write {
    pub space: u32,
    pub address: u32,
    pub value: u32,
    pub pc: u32,
}

/// A call pushed (`kind` 1, with its source and target) or a return popped (`kind` 0).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Call {
    pub kind: u32,
    pub source: u32,
    pub target: u32,
}

/// C#'s `CallStackRegistry.MaxDepth`: a runaway chain stops growing here, and its returns still pop.
pub const MAX_DEPTH: usize = 512;
/// Entries a log holds before the frame stops to have it drained.
pub const LOG_CAPACITY: usize = 1 << 16;
/// One bit per 24-bit address, as C#'s `CoverageRegistry` keeps it.
pub const COVERAGE_BYTES: usize = 1 << 21;

#[derive(Clone, Debug)]
pub struct Hooks {
    /// Calls and returns tracked; off, the stack stands where the last observed frame left it.
    pub calls: bool,
    /// Processor stores reported byte by byte, as C#'s `MemoryBus.Report`, into the ranges below.
    pub writes: bool,
    /// Stop after an interrupt is entered.
    pub interrupts: bool,
    /// Stop before every instruction.
    pub each: bool,
    /// Instructions charged to the innermost call, as C#'s `NoteInstruction`.
    pub profiling: bool,
    /// A `jr ra` ran; the pop waits for its delay slot, as C#'s `_returnAfterSlot`.
    pub return_after_slot: bool,
    pub interrupt_taken: bool,
    pub data_pending: bool,
    /// Stop once the depth is at or under this; `i32::MIN` is unarmed, as C#'s `_stepDepthTarget`.
    pub depth_target: i32,
    /// Stop once the depth is over this; `-1` is unarmed.
    pub depth_guard: i32,
    /// Enabled breakpoints, both ends included, compared as C# compares its `int`s.
    pub breakpoints: Vec<(i32, i32)>,
    pub watch_ranges: Vec<Range>,
    pub break_ranges: Vec<Range>,
    /// `(source, target)` of every open call, innermost last.
    pub stack: Vec<(u32, u32)>,
    pub unmatched_returns: i64,
    pub calls_log: Vec<Call>,
    pub writes_log: Vec<Write>,
    /// The bitmap while coverage is armed, drained and cleared by the host.
    pub coverage: Option<Box<[u8]>>,
    pub covered: i64,
    /// Instructions by the routine they ran in, zero outside any call; drained by the host.
    pub profile: HashMap<u32, i64>,
    owner: u32,
    run: i64,
    /// The frame an observed loop is inside, so a stop the host resumes from does not start the frame's clock again.
    pub frame_open: bool,
    pub frame_start: i64,
    pub frame_fields: i64,
}

impl Default for Hooks {
    fn default() -> Self {
        Hooks {
            calls: false,
            writes: false,
            interrupts: false,
            each: false,
            profiling: false,
            return_after_slot: false,
            interrupt_taken: false,
            data_pending: false,
            depth_target: i32::MIN,
            depth_guard: -1,
            breakpoints: Vec::new(),
            watch_ranges: Vec::new(),
            break_ranges: Vec::new(),
            stack: Vec::new(),
            unmatched_returns: 0,
            calls_log: Vec::new(),
            writes_log: Vec::new(),
            coverage: None,
            covered: 0,
            profile: HashMap::new(),
            owner: 0,
            run: 0,
            frame_open: false,
            frame_start: 0,
            frame_fields: 0,
        }
    }
}

impl Hooks {
    /// The flags set at once, the profile's run closed first so no instruction is charged to a routine under the new rules.
    pub fn configure(&mut self, calls: bool, writes: bool, interrupts: bool, each: bool, coverage: bool, profiling: bool) {
        self.flush();
        self.calls = calls;
        if !calls {
            self.return_after_slot = false;
        }
        self.writes = writes;
        self.interrupts = interrupts;
        self.each = each;
        if coverage != self.coverage.is_some() {
            self.coverage = coverage.then(|| vec![0u8; COVERAGE_BYTES].into_boxed_slice());
        }
        self.profiling = profiling;
        self.owner = self.stack.last().map_or(0, |f| f.1);
    }

    /// True while any table could stop, record or report: the frame loop the host chooses is the observed one.
    pub fn armed(&self) -> bool {
        self.calls || self.writes || self.interrupts || self.each || self.profiling || self.coverage.is_some() || !self.breakpoints.is_empty()
    }

    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// `CallObserver`: pushed as the call executes, so its delay slot is already a frame deeper.
    #[inline(never)]
    pub fn note_call(&mut self, source: u64, target: u64) {
        let (source, target) = (source as u32, target as u32);
        self.calls_log.push(Call { kind: 1, source, target });
        if self.stack.len() < MAX_DEPTH {
            self.stack.push((source, target));
        }
        self.flush();
    }

    /// `ReturnObserver`: popped once the return's delay slot has run; a pop with nothing open is counted, not swallowed.
    #[inline(never)]
    pub fn note_return(&mut self) {
        self.calls_log.push(Call { kind: 0, source: 0, target: 0 });
        if self.stack.pop().is_none() {
            self.unmatched_returns += 1;
        }
        self.flush();
    }

    /// The profile's open run charged to its routine, and the routine that owns the next run named.
    pub fn flush(&mut self) {
        if self.run != 0 {
            *self.profile.entry(self.owner).or_insert(0) += self.run;
            self.run = 0;
        }
        self.owner = self.stack.last().map_or(0, |f| f.1);
    }

    /// What the observed loop records before each instruction: the coverage bit and the profiler's count.
    #[inline(always)]
    pub fn record(&mut self, pc: u64) {
        if let Some(bits) = &mut self.coverage {
            let index = (pc as u32 & 0xFF_FFFF) as usize;
            bits[index >> 3] |= 1 << (index & 7);
            self.covered += 1;
        }
        if self.profiling {
            self.run += 1;
        }
    }

    /// Why the loop should stop before the instruction at `pc`, every reason that holds; the one-shot reasons are consumed.
    #[inline(always)]
    pub fn stop_before(&mut self, pc: u64) -> u32 {
        let mut why = stop::FRAME;
        if self.each {
            why |= stop::EACH;
        }
        let depth = self.stack.len() as i32;
        if (self.depth_target != i32::MIN && depth <= self.depth_target) || (self.depth_guard >= 0 && depth > self.depth_guard) {
            why |= stop::DEPTH;
        }
        if self.data_pending {
            self.data_pending = false;
            why |= stop::DATA;
        }
        if self.interrupt_taken {
            self.interrupt_taken = false;
            why |= stop::INTERRUPT;
        }
        if self.writes_log.len() >= LOG_CAPACITY || self.calls_log.len() >= LOG_CAPACITY {
            why |= stop::RING;
        }
        if !self.breakpoints.is_empty() && self.covers(pc) {
            why |= stop::BREAKPOINT;
        }
        why
    }

    /// `Breakpoint.Covers` on the address as C# holds it, a signed 32-bit integer.
    #[inline(always)]
    pub fn covers(&self, pc: u64) -> bool {
        let address = pc as u32 as i32;
        self.breakpoints.iter().any(|&(start, end)| address >= start && address <= end)
    }

    /// One byte of a store, as `IWriteObserver.OnWrite` would receive it: logged where a watch or a data breakpoint covers it.
    #[inline(never)]
    pub fn note_write(&mut self, space: u32, address: u32, value: u8, pc: u64) {
        let hit = |r: &Range| r.space == space && address >= r.start && address <= r.end;
        let watched = self.watch_ranges.iter().any(hit);
        let breaks = self.break_ranges.iter().any(hit);
        if watched || breaks {
            self.writes_log.push(Write { space, address, value: value as u32, pc: pc as u32 });
        }
        if breaks {
            self.data_pending = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_return_with_nothing_open_is_counted_and_the_depth_never_goes_negative() {
        let mut h = Hooks::default();
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
        assert_eq!(h.profile[&0], 3);
        assert_eq!(h.profile[&0x200], 1);
    }

    #[test]
    fn a_stop_consumes_its_one_shot_reasons_and_reports_every_reason_at_once() {
        let mut h = Hooks { each: true, data_pending: true, interrupt_taken: true, ..Hooks::default() };
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
}
