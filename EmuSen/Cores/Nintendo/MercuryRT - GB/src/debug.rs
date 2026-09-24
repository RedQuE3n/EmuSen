//! The debugger's hooks: tables the host pushes down, and what an observed frame records for it to read back.
//! Nothing here calls the host; a frame that meets a table ends early with a reason. See Mercury_Native.md §8.5.

use std::collections::BTreeMap;

use crate::cpu::CpuBus;
use crate::memory::bus::MemoryBus;

/// Why an observed frame stopped, as bits; zero is the frame's end.
pub mod stop {
    pub const FRAME: u32 = 0;
    /// The next instruction's address is in a breakpoint range.
    pub const BREAKPOINT: u32 = 1;
    /// The host asked to stop before every instruction.
    pub const EACH: u32 = 2;
    /// The call stack's depth is at or under the target, or over the guard.
    pub const DEPTH: u32 = 4;
    /// A store landed in a data-breakpoint range; the store has run.
    pub const DATA: u32 = 8;
    /// An interrupt was dispatched; the next instruction is the handler's first.
    pub const INTERRUPT: u32 = 16;
    /// A log is full and must be drained.
    pub const RING: u32 = 32;
}

/// A stretch of one memory, numbered as `ffi`'s spaces are, both ends included.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    pub space: u32,
    pub start: u32,
    pub end: u32,
}

/// One byte a CPU store left, in the space C#'s `MemoryBus.Write` reports it under, and the storing instruction's address.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Write {
    pub space: u32,
    pub address: u32,
    pub value: u32,
    pub pc: u32,
}

/// A call pushed (`kind` 1), an interrupt dispatched (`kind` 2), each with its source and target, or a return popped (`kind` 0).
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
/// One bit for each of the CPU's 64K addresses: the first 8K of C#'s 24-bit `CoverageRegistry` bitmap.
pub const COVERAGE_BYTES: usize = 0x10000 / 8;

#[derive(Clone, Debug)]
pub struct Hooks {
    /// Calls, interrupt dispatches and returns tracked; off, the stack stands where the host last set it.
    pub calls: bool,
    /// CPU stores reported byte by byte, as C#'s `IWriteObserver`, into the ranges below.
    pub writes: bool,
    /// Stop after an interrupt is dispatched.
    pub interrupts: bool,
    /// Stop before every instruction.
    pub each: bool,
    /// Instructions charged to the innermost call, as C#'s `NoteInstruction`.
    pub profiling: bool,
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
    /// The target of every open call, innermost last: the registry's, pushed down with the tables, then moved by what runs.
    pub stack: Vec<u16>,
    pub calls_log: Vec<Call>,
    pub writes_log: Vec<Write>,
    /// The bitmap while coverage is armed, drained and cleared by the host.
    pub coverage: Option<Box<[u8]>>,
    pub covered: i64,
    /// Instructions by the routine they ran in, zero outside any call; drained by the host. Not a `HashMap`: that one changed the
    /// plain `MemoryBus::read`'s code, which shares its hasher with the ROM-patch table (Mercury_Native.md §8.5.5).
    pub profile: BTreeMap<u32, i64>,
    owner: u32,
    run: i64,
    /// The budget the frame an observed loop is inside began with, kept while the host continues it past a stop it refused.
    pub budget: i64,
    /// Entries a log holds before it stops the frame; `LOG_CAPACITY` except in the crate's own tests.
    pub capacity: usize,
}

impl Default for Hooks {
    fn default() -> Self {
        Hooks {
            calls: false,
            writes: false,
            interrupts: false,
            each: false,
            profiling: false,
            interrupt_taken: false,
            data_pending: false,
            depth_target: i32::MIN,
            depth_guard: -1,
            breakpoints: Vec::new(),
            watch_ranges: Vec::new(),
            break_ranges: Vec::new(),
            stack: Vec::new(),
            calls_log: Vec::new(),
            writes_log: Vec::new(),
            coverage: None,
            covered: 0,
            profile: BTreeMap::new(),
            owner: 0,
            run: 0,
            budget: 0,
            capacity: LOG_CAPACITY,
        }
    }
}

impl Hooks {
    /// The flags set at once, the profile's run closed first so no instruction is charged to a routine under the new rules.
    pub fn configure(&mut self, calls: bool, writes: bool, interrupts: bool, each: bool, coverage: bool, profiling: bool) {
        self.flush();
        self.calls = calls;
        self.writes = writes;
        self.interrupts = interrupts;
        self.each = each;
        if coverage != self.coverage.is_some() {
            self.coverage = coverage.then(|| vec![0u8; COVERAGE_BYTES].into_boxed_slice());
        }
        self.profiling = profiling;
    }

    /// The registry's stack, innermost last, so a depth and an owner here are the registry's own.
    pub fn set_stack(&mut self, targets: &[u16]) {
        self.flush();
        self.stack.clear();
        self.stack.extend(targets.iter().take(MAX_DEPTH));
        self.owner = self.stack.last().map_or(0, |&t| t as u32);
    }

    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// `CallStackRegistry.NotePush`: the push past the cap is logged for the registry's entry points, and not stacked.
    #[inline(never)]
    pub fn note_call(&mut self, source: u16, target: u16, interrupt: bool) {
        if interrupt && self.interrupts {
            self.interrupt_taken = true;
        }
        if !self.calls {
            return;
        }
        self.calls_log.push(Call { kind: if interrupt { 2 } else { 1 }, source: source as u32, target: target as u32 });
        self.flush();
        if self.stack.len() < MAX_DEPTH {
            self.stack.push(target);
        }
        self.owner = self.stack.last().map_or(0, |&t| t as u32);
    }

    /// `CallStackRegistry.NotePop`: a pop with nothing open is logged for the registry to count, and moves nothing here.
    #[inline(never)]
    pub fn note_return(&mut self) {
        if !self.calls {
            return;
        }
        self.calls_log.push(Call::default());
        self.flush();
        self.stack.pop();
        self.owner = self.stack.last().map_or(0, |&t| t as u32);
    }

    /// The profile's open run charged to the routine it ran in.
    pub fn flush(&mut self) {
        if self.run != 0 {
            *self.profile.entry(self.owner).or_insert(0) += self.run;
            self.run = 0;
        }
    }

    /// What C#'s loop records before each step: the coverage bit and the profiler's count.
    #[inline(always)]
    pub fn record(&mut self, pc: u16) {
        if let Some(bits) = &mut self.coverage {
            bits[(pc >> 3) as usize] |= 1 << (pc & 7);
            self.covered += 1;
        }
        if self.profiling {
            self.run += 1;
        }
    }

    /// Why the loop should stop before the step at `pc`, every reason that holds; the one-shot reasons are consumed.
    #[inline(always)]
    pub fn stop_before(&mut self, pc: u16) -> u32 {
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
        if self.writes_log.len() >= self.capacity || self.calls_log.len() >= self.capacity {
            why |= stop::RING;
        }
        if self.covers(pc) {
            why |= stop::BREAKPOINT;
        }
        why
    }

    /// `Breakpoint.Covers` on the address as C# passes it to `ShouldBreak`, the program counter widened to an `int`.
    #[inline(always)]
    pub fn covers(&self, pc: u16) -> bool {
        let address = pc as i32;
        self.breakpoints.iter().any(|&(start, end)| address >= start && address <= end)
    }

    /// One byte of a store, as `IWriteObserver.OnWrite` receives it: logged where a watch or a data breakpoint covers it.
    #[inline(never)]
    pub fn note_write(&mut self, space: u32, address: u32, value: u8, pc: u16) {
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

    /// The storing instruction's address, known only once the step that stored has set it, written into what that step logged.
    #[inline(always)]
    pub fn stamp(&mut self, from: usize, pc: u16) {
        for w in &mut self.writes_log[from..] {
            w.pc = pc as u32;
        }
    }
}

/// The bus as the observed step sees it: every access passes to the machine's bus, and stores, calls and returns are noted on the way.
/// The plain step is monomorphised on `MemoryBus` itself, so none of this is in it.
pub struct ObservedBus<'a> {
    pub bus: &'a mut MemoryBus,
    pub hooks: &'a mut Hooks,
}

impl CpuBus for ObservedBus<'_> {
    #[inline(always)]
    fn read(&mut self, address: u16) -> u8 {
        self.bus.read(address)
    }

    #[inline(always)]
    fn write(&mut self, address: u16, data: u8) {
        let reported = if self.hooks.writes { self.bus.reported_space(address) } else { None };
        self.bus.write(address, data);
        if let Some((space, offset)) = reported {
            self.hooks.note_write(space, offset, data, 0);
        }
    }

    #[inline(always)]
    fn tick(&mut self, cycles: i32) {
        self.bus.tick(cycles)
    }

    fn stop(&mut self) {
        self.bus.stop()
    }

    fn note_call(&mut self, source: u16, target: u16, interrupt: bool) {
        self.hooks.note_call(source, target, interrupt);
    }

    fn note_return(&mut self) {
        self.hooks.note_return();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_return_with_nothing_open_is_logged_and_the_depth_never_goes_negative() {
        let mut h = Hooks { calls: true, ..Hooks::default() };
        h.note_return();
        assert_eq!(h.depth(), 0);
        h.note_call(0x10, 0x20, false);
        h.note_return();
        assert_eq!(h.depth(), 0);
        assert_eq!(h.calls_log, vec![Call::default(), Call { kind: 1, source: 0x10, target: 0x20 }, Call::default()]);
    }

    #[test]
    fn the_stack_stops_growing_at_the_cap_and_its_returns_still_pop() {
        let mut h = Hooks { calls: true, ..Hooks::default() };
        for i in 0..MAX_DEPTH as u16 + 3 {
            h.note_call(i, i, false);
        }
        assert_eq!(h.depth(), MAX_DEPTH);
        assert_eq!(h.calls_log.len(), MAX_DEPTH + 3);
        h.note_return();
        assert_eq!(h.depth(), MAX_DEPTH - 1);
    }

    #[test]
    fn nothing_is_tracked_or_logged_while_calls_are_off_but_an_interrupt_still_stops() {
        let mut h = Hooks { interrupts: true, ..Hooks::default() };
        h.note_call(1, 0x40, true);
        h.note_return();
        assert!(h.calls_log.is_empty());
        assert_eq!(h.depth(), 0);
        assert_eq!(h.stop_before(0x40), stop::INTERRUPT);
    }

    #[test]
    fn a_breakpoint_range_compares_as_the_int_the_registry_keeps() {
        let h = Hooks { breakpoints: vec![(0x150, 0x152), (-5, -1)], ..Hooks::default() };
        assert!(h.covers(0x151));
        assert!(!h.covers(0x153));
        assert!(!h.covers(0xFFFF));
    }

    #[test]
    fn the_profile_charges_each_run_to_the_routine_open_while_it_ran() {
        let mut h = Hooks::default();
        h.configure(true, false, false, false, false, true);
        h.record(0);
        h.record(0);
        h.note_call(0x100, 0x200, false);
        h.record(0);
        h.note_return();
        h.record(0);
        h.flush();
        assert_eq!(h.profile[&0], 3);
        assert_eq!(h.profile[&0x200], 1);
    }

    #[test]
    fn a_stack_pushed_down_sets_the_depth_and_the_owner() {
        let mut h = Hooks::default();
        h.configure(true, false, false, false, false, true);
        h.set_stack(&[0x300, 0x400]);
        h.record(0);
        h.note_return();
        h.record(0);
        h.flush();
        assert_eq!((h.profile[&0x400], h.profile[&0x300], h.depth()), (1, 1, 1));
    }

    #[test]
    fn a_stop_consumes_its_one_shot_reasons_and_reports_every_reason_at_once() {
        let mut h = Hooks { each: true, data_pending: true, interrupt_taken: true, calls: true, ..Hooks::default() };
        assert_eq!(h.stop_before(0), stop::EACH | stop::DATA | stop::INTERRUPT);
        assert_eq!(h.stop_before(0), stop::EACH);
        h.each = false;
        h.depth_target = 0;
        assert_eq!(h.stop_before(0), stop::DEPTH);
        h.note_call(1, 2, false);
        assert_eq!(h.stop_before(0), stop::FRAME);
        h.depth_guard = 0;
        assert_eq!(h.stop_before(0), stop::DEPTH);
    }

    #[test]
    fn a_write_is_logged_where_a_range_covers_it_and_only_a_break_range_stops() {
        let mut h = Hooks::default();
        h.watch_ranges.push(Range { space: 3, start: 0x10, end: 0x13 });
        h.break_ranges.push(Range { space: 5, start: 0x20, end: 0x20 });
        h.note_write(3, 0x13, 7, 0x150);
        assert_eq!(h.writes_log.len(), 1);
        assert!(!h.data_pending);
        h.note_write(3, 0x14, 7, 0);
        h.note_write(4, 0x10, 7, 0);
        assert_eq!(h.writes_log.len(), 1);
        h.note_write(5, 0x20, 9, 0);
        assert_eq!(h.writes_log.len(), 2);
        assert!(h.data_pending);
        assert_eq!(h.writes_log[0], Write { space: 3, address: 0x13, value: 7, pc: 0x150 });
    }

    #[test]
    fn a_full_log_stops_the_frame() {
        let mut h = Hooks { capacity: 2, ..Hooks::default() };
        h.watch_ranges.push(Range { space: 3, start: 0, end: 0xFFFF });
        h.note_write(3, 0, 1, 0);
        assert_eq!(h.stop_before(0), stop::FRAME);
        h.note_write(3, 0, 1, 0);
        assert_eq!(h.stop_before(0), stop::RING);
    }
}
