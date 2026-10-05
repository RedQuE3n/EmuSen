//! The debugger's hooks as data: the tables the host pushes down, and what an observed frame records for it to drain.
//! Nothing here calls the host, and nothing here knows how a core's loop observes; a frame that meets a table ends early
//! with its reasons. The mechanism (MercuryRT's observed bus, MarsRT's `HOOKED` interpreter, MoonRT's observing flag)
//! stays in each core. See EmuSen_NativeCores.md §3.14.

use std::collections::BTreeMap;

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

/// The flags `emusen_native_debug_set` takes; coverage of processor n is bit `COVERAGE + n`.
pub mod flag {
    pub const CALLS: u32 = 1;
    pub const WRITES: u32 = 2;
    pub const INTERRUPTS: u32 = 4;
    pub const EACH: u32 = 8;
    pub const PROFILING: u32 = 16;
    pub const COVERAGE: u32 = 8;
}

/// The flags `emusen_native_debug_run_frame` takes.
pub mod run {
    /// The first instruction is not checked: the host has already decided to run it.
    pub const UNCHECKED: u32 = 1;
    /// The frame is the one a stop left open, so its budget stands.
    pub const CONTINUE: u32 = 2;
}

/// What a call-log entry is: a return, a call, or an interrupt of C#'s `CallFrameKind` entered.
pub mod kind {
    pub const RETURN: u32 = 0;
    pub const CALL: u32 = 1;
    pub const IRQ: u32 = 2;
    pub const NMI: u32 = 3;
    pub const BRK: u32 = 4;
    pub const COP: u32 = 5;
}

/// A stretch of one memory, numbered as the core's spaces are, both ends included.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    pub space: u32,
    pub start: u32,
    pub end: u32,
}

/// One byte a store left, in the space the C# core reports it under, and the storing instruction's address.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Write {
    pub space: u32,
    pub address: u32,
    pub value: u32,
    pub pc: u32,
}

/// A call-log entry: `kind`, then the source and target of a call or an interrupt; a return's are zero.
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

/// One processor's coverage: a bit an address over its width, and the steps recorded since the last drain.
#[derive(Clone, Debug)]
pub struct Coverage {
    pub bits: Box<[u8]>,
    pub covered: i64,
    mask: u32,
}

impl Coverage {
    fn new(width: u32) -> Coverage {
        Coverage { bits: vec![0u8; (1usize << width) / 8].into_boxed_slice(), covered: 0, mask: ((1u64 << width) - 1) as u32 }
    }
}

#[derive(Clone, Debug)]
pub struct Hooks {
    /// Calls, interrupts and returns tracked; off, the stack stands where the host last set it.
    pub calls: bool,
    /// Stores reported byte by byte into the ranges below.
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
    /// Every open call as (source, target), innermost last: the registry's, pushed down, then moved by what runs.
    pub stack: Vec<(u32, u32)>,
    /// Returns with nothing open, which move nothing and are still logged.
    pub unmatched_returns: i64,
    pub calls_log: Vec<Call>,
    pub writes_log: Vec<Write>,
    /// Each processor's coverage while armed; its width is fixed at `new`.
    pub coverage: Vec<Option<Coverage>>,
    widths: Vec<u32>,
    /// Instructions by the routine they ran in, zero outside any call. A `BTreeMap`, not a `HashMap`: a second hasher
    /// instantiation changed MercuryRT's plain bus read (Mercury_Native.md §8.5.5).
    pub profile: BTreeMap<u32, i64>,
    owner: u32,
    run: i64,
    /// The budget the open frame began with, kept while the host continues it past a stop it refused.
    pub budget: i64,
    /// Entries a log holds before it stops the frame; `LOG_CAPACITY` except in tests.
    pub capacity: usize,
}

impl Hooks {
    /// Hooks for processors whose coverage spans `widths` address bits each: 16 for a 6502 or an SM83.
    pub fn new(widths: &[u32]) -> Hooks {
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
            unmatched_returns: 0,
            calls_log: Vec::new(),
            writes_log: Vec::new(),
            coverage: widths.iter().map(|_| None).collect(),
            widths: widths.to_vec(),
            profile: BTreeMap::new(),
            owner: 0,
            run: 0,
            budget: 0,
            capacity: LOG_CAPACITY,
        }
    }

    /// `emusen_native_debug_set`'s flags applied at once, the profile's run closed first.
    pub fn configure(&mut self, flags: u32, depth_target: i32, depth_guard: i32) {
        self.flush();
        let on = |bit: u32| flags & bit != 0;
        self.calls = on(flag::CALLS);
        self.writes = on(flag::WRITES);
        self.interrupts = on(flag::INTERRUPTS);
        self.each = on(flag::EACH);
        self.profiling = on(flag::PROFILING);
        for (n, slot) in self.coverage.iter_mut().enumerate() {
            let want = n < 24 && on(1 << (flag::COVERAGE + n as u32));
            if want != slot.is_some() {
                *slot = want.then(|| Coverage::new(self.widths[n]));
            }
        }
        self.depth_target = depth_target;
        self.depth_guard = depth_guard;
    }

    /// Anything a plain frame would miss: a table, a flag, or a depth armed.
    pub fn armed(&self) -> bool {
        self.calls
            || self.writes
            || self.interrupts
            || self.each
            || self.profiling
            || self.coverage.iter().any(Option::is_some)
            || !self.breakpoints.is_empty()
            || self.depth_target != i32::MIN
            || self.depth_guard >= 0
    }

    /// The registry's stack, innermost last, so a depth and an owner here are the registry's own.
    pub fn set_stack(&mut self, pairs: &[(u32, u32)]) {
        self.flush();
        self.stack.clear();
        self.stack.extend(pairs.iter().take(MAX_DEPTH));
        self.owner = self.stack.last().map_or(0, |&(_, t)| t);
    }

    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// `CallStackRegistry.NotePush`; `kind` is `kind::CALL` or an interrupt's. The push past the cap is logged, not stacked.
    #[inline(never)]
    pub fn note_call(&mut self, source: u32, target: u32, kind: u32) {
        if kind >= kind::IRQ && self.interrupts {
            self.interrupt_taken = true;
        }
        if !self.calls {
            return;
        }
        self.calls_log.push(Call { kind, source, target });
        self.flush();
        if self.stack.len() < MAX_DEPTH {
            self.stack.push((source, target));
        }
        self.owner = self.stack.last().map_or(0, |&(_, t)| t);
    }

    /// `CallStackRegistry.NotePop`: a pop with nothing open is logged and counted, and moves nothing.
    #[inline(never)]
    pub fn note_return(&mut self) {
        if !self.calls {
            return;
        }
        self.calls_log.push(Call::default());
        self.flush();
        if self.stack.pop().is_none() {
            self.unmatched_returns += 1;
        }
        self.owner = self.stack.last().map_or(0, |&(_, t)| t);
    }

    /// The profile's open run charged to the routine it ran in.
    pub fn flush(&mut self) {
        if self.run != 0 {
            *self.profile.entry(self.owner).or_insert(0) += self.run;
            self.run = 0;
        }
    }

    /// What the C# loop records before each step of processor 0: the coverage bit and the profiler's count.
    #[inline(always)]
    pub fn record(&mut self, pc: u32) {
        self.record_on(0, pc);
        if self.profiling {
            self.run += 1;
        }
    }

    /// A coverage bit for another processor's step, which the profile does not count.
    #[inline(always)]
    pub fn record_on(&mut self, processor: usize, pc: u32) {
        if let Some(Some(c)) = self.coverage.get_mut(processor) {
            let a = pc & c.mask;
            c.bits[(a >> 3) as usize] |= 1 << (a & 7);
            c.covered += 1;
        }
    }

    /// Why the loop should stop before the step at `pc`, every reason that holds; the one-shot reasons are consumed.
    #[inline(always)]
    pub fn stop_before(&mut self, pc: u32) -> u32 {
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

    /// `Breakpoint.Covers` on the address as C# passes it to `ShouldBreak`, widened to an `int`.
    #[inline(always)]
    pub fn covers(&self, pc: u32) -> bool {
        let address = pc as i32;
        self.breakpoints.iter().any(|&(start, end)| address >= start && address <= end)
    }

    /// One byte of a store, as `IWriteObserver.OnWrite` receives it: logged where a watch or a data breakpoint covers it.
    #[inline(never)]
    pub fn note_write(&mut self, space: u32, address: u32, value: u8, pc: u32) {
        let hit = |r: &Range| r.space == space && address >= r.start && address <= r.end;
        let watched = self.watch_ranges.iter().any(hit);
        let breaks = self.break_ranges.iter().any(hit);
        if watched || breaks {
            self.writes_log.push(Write { space, address, value: value as u32, pc });
        }
        if breaks {
            self.data_pending = true;
        }
    }

    /// The storing instruction's address, known once its step has set it, written into what that step logged.
    #[inline(always)]
    pub fn stamp(&mut self, from: usize, pc: u32) {
        for w in &mut self.writes_log[from..] {
            w.pc = pc;
        }
    }

    /// The breakpoints as the host sends them: pairs of first and last address.
    pub fn set_breakpoints(&mut self, pairs: &[i32]) {
        self.breakpoints = pairs.chunks_exact(2).map(|p| (p[0], p[1])).collect();
    }

    /// Store ranges as the host sends them, triples of space, first and last offset; `kind` 0 watches, 1 data breakpoints.
    pub fn set_ranges(&mut self, kind: u32, triples: &[u32]) {
        let ranges = triples.chunks_exact(3).map(|t| Range { space: t[0], start: t[1], end: t[2] }).collect();
        if kind == 0 {
            self.watch_ranges = ranges;
        } else {
            self.break_ranges = ranges;
        }
    }

    /// `emusen_native_debug_counters`: the depth, the unmatched returns, then a core's own.
    pub fn counters(&self) -> [i64; 2] {
        [self.stack.len() as i64, self.unmatched_returns]
    }
}

/// A log copied whole into `out` and then cleared, `N` values an entry; a log that does not fit is left for a larger buffer.
/// Returns how many entries there are.
pub fn drain<T, const N: usize>(log: &mut Vec<T>, out: &mut [u32], fields: impl Fn(&T) -> [u32; N]) -> usize {
    let count = log.len();
    if out.len() >= count * N {
        for (i, entry) in log.iter().enumerate() {
            out[i * N..(i + 1) * N].copy_from_slice(&fields(entry));
        }
        log.clear();
    }
    count
}

/// The profile's runs as (owner, instructions) pairs, the open run closed first, forgotten when they fit; the pair count.
pub fn drain_profile(hooks: &mut Hooks, out: &mut [i64]) -> usize {
    hooks.flush();
    let count = hooks.profile.len();
    if out.len() >= 2 * count {
        for (i, (owner, instructions)) in std::mem::take(&mut hooks.profile).into_iter().enumerate() {
            out[2 * i] = owner as i64;
            out[2 * i + 1] = instructions;
        }
    }
    count
}

/// A processor's bitmap copied and cleared when `out` holds it, with the steps it stands for; its length, zero while unarmed.
pub fn drain_coverage(hooks: &mut Hooks, processor: usize, out: &mut [u8], recorded: &mut i64) -> usize {
    let Some(Some(c)) = hooks.coverage.get_mut(processor) else { return 0 };
    *recorded = c.covered;
    if out.len() >= c.bits.len() {
        out[..c.bits.len()].copy_from_slice(&c.bits);
        c.bits.fill(0);
        c.covered = 0;
    }
    c.bits.len()
}

/// What a CPU tells its core's debugger, at no cost when unobserved: a step generic over `O: Observer` is compiled
/// once with `Unobserved` and once with `Hooks`. The Beryl CPU crates call it (Beryl-HW/README.md).
pub trait Observer {
    /// Before the instruction at `pc` on `processor`: the stop reasons, zero to run it.
    #[inline(always)]
    fn before(&mut self, _processor: usize, _pc: u32) -> u32 {
        stop::FRAME
    }

    /// One byte of a store, after it has landed.
    #[inline(always)]
    fn wrote(&mut self, _space: u32, _address: u32, _value: u8, _pc: u32) {}

    /// A call, an interrupt or an exception taken, with `kind::` kinds.
    #[inline(always)]
    fn called(&mut self, _source: u32, _target: u32, _kind: u32) {}

    #[inline(always)]
    fn returned(&mut self) {}
}

/// The plain run's observer, which observes nothing.
pub struct Unobserved;

impl Observer for Unobserved {}

/// Processor 0's stops are the hooks' own; another processor's step records its coverage, its breakpoints being its
/// core's to keep.
impl Observer for Hooks {
    #[inline(always)]
    fn before(&mut self, processor: usize, pc: u32) -> u32 {
        if processor == 0 {
            self.record(pc);
            self.stop_before(pc)
        } else {
            self.record_on(processor, pc);
            stop::FRAME
        }
    }

    #[inline(always)]
    fn wrote(&mut self, space: u32, address: u32, value: u8, pc: u32) {
        if self.writes {
            self.note_write(space, address, value, pc);
        }
    }

    #[inline(always)]
    fn called(&mut self, source: u32, target: u32, kind: u32) {
        self.note_call(source, target, kind);
    }

    #[inline(always)]
    fn returned(&mut self) {
        self.note_return();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn armed(flags: u32) -> Hooks {
        let mut h = Hooks::new(&[16]);
        h.configure(flags, i32::MIN, -1);
        h
    }

    #[test]
    fn a_return_with_nothing_open_is_logged_counted_and_the_depth_never_goes_negative() {
        let mut h = armed(flag::CALLS);
        h.note_return();
        assert_eq!((h.depth(), h.unmatched_returns), (0, 1));
        h.note_call(0x10, 0x20, kind::CALL);
        h.note_return();
        assert_eq!((h.depth(), h.unmatched_returns), (0, 1));
        assert_eq!(h.calls_log, vec![Call::default(), Call { kind: 1, source: 0x10, target: 0x20 }, Call::default()]);
        assert_eq!(h.counters(), [0, 1]);
    }

    #[test]
    fn the_stack_stops_growing_at_the_cap_and_its_returns_still_pop() {
        let mut h = armed(flag::CALLS);
        for i in 0..MAX_DEPTH as u32 + 3 {
            h.note_call(i, i, kind::CALL);
        }
        assert_eq!(h.depth(), MAX_DEPTH);
        assert_eq!(h.calls_log.len(), MAX_DEPTH + 3);
        h.note_return();
        assert_eq!(h.depth(), MAX_DEPTH - 1);
    }

    #[test]
    fn nothing_is_tracked_while_calls_are_off_but_an_interrupt_of_any_kind_still_stops() {
        for k in [kind::IRQ, kind::NMI, kind::BRK] {
            let mut h = armed(flag::INTERRUPTS);
            h.note_call(1, 0x40, k);
            h.note_return();
            assert!(h.calls_log.is_empty());
            assert_eq!(h.stop_before(0x40), stop::INTERRUPT);
        }
        let mut h = armed(flag::INTERRUPTS);
        h.note_call(1, 0x40, kind::CALL);
        assert_eq!(h.stop_before(0x40), stop::FRAME);
    }

    #[test]
    fn a_breakpoint_range_compares_as_the_int_the_registry_keeps() {
        let mut h = Hooks::new(&[16]);
        h.set_breakpoints(&[0x150, 0x152, -5, -1]);
        assert!(h.covers(0x150) && h.covers(0x151) && h.covers(0x152));
        assert!(!h.covers(0x14F) && !h.covers(0x153) && !h.covers(0xFFFF));
        assert!(h.armed());
    }

    #[test]
    fn the_profile_charges_each_run_to_the_routine_open_while_it_ran() {
        let mut h = armed(flag::CALLS | flag::PROFILING);
        h.record(0);
        h.record(0);
        h.note_call(0x100, 0x200, kind::CALL);
        h.record(0);
        h.note_return();
        h.record(0);
        let mut out = [0i64; 4];
        assert_eq!(drain_profile(&mut h, &mut out), 2);
        assert_eq!(out, [0, 3, 0x200, 1]);
        assert!(h.profile.is_empty());
    }

    #[test]
    fn a_stack_pushed_down_sets_the_depth_and_the_owner() {
        let mut h = armed(flag::CALLS | flag::PROFILING);
        h.set_stack(&[(1, 0x300), (2, 0x400)]);
        h.record(0);
        h.note_return();
        h.record(0);
        h.flush();
        assert_eq!((h.profile[&0x400], h.profile[&0x300], h.depth()), (1, 1, 1));
    }

    #[test]
    fn a_stop_consumes_its_one_shot_reasons_and_reports_every_reason_at_once() {
        let mut h = armed(flag::EACH | flag::CALLS);
        h.data_pending = true;
        h.interrupt_taken = true;
        assert_eq!(h.stop_before(0), stop::EACH | stop::DATA | stop::INTERRUPT);
        assert_eq!(h.stop_before(0), stop::EACH);
        h.configure(flag::CALLS, 0, -1);
        assert_eq!(h.stop_before(0), stop::DEPTH);
        h.note_call(1, 2, kind::CALL);
        assert_eq!(h.stop_before(0), stop::FRAME);
        h.configure(flag::CALLS, i32::MIN, 1);
        assert_eq!(h.stop_before(0), stop::FRAME);
        h.configure(flag::CALLS, i32::MIN, 0);
        assert_eq!(h.stop_before(0), stop::DEPTH);
    }

    #[test]
    fn a_write_is_logged_where_a_range_covers_it_and_only_a_break_range_stops() {
        let mut h = Hooks::new(&[16]);
        h.set_ranges(0, &[3, 0x10, 0x13]);
        h.set_ranges(1, &[5, 0x20, 0x20]);
        h.note_write(3, 0x13, 7, 0x150);
        assert_eq!(h.writes_log.len(), 1);
        assert!(!h.data_pending);
        h.note_write(3, 0x14, 7, 0);
        h.note_write(4, 0x10, 7, 0);
        assert_eq!(h.writes_log.len(), 1);
        h.note_write(5, 0x20, 9, 0);
        assert!(h.data_pending);
        h.stamp(1, 0x8123);
        assert_eq!(h.writes_log, vec![Write { space: 3, address: 0x13, value: 7, pc: 0x150 }, Write { space: 5, address: 0x20, value: 9, pc: 0x8123 }]);
        let mut out = [0u32; 7];
        assert_eq!(drain(&mut h.writes_log, &mut out, |w: &Write| [w.space, w.address, w.value, w.pc]), 2);
        assert_eq!(h.writes_log.len(), 2, "a buffer too small leaves the log");
        let mut out = [0u32; 8];
        drain(&mut h.writes_log, &mut out, |w: &Write| [w.space, w.address, w.value, w.pc]);
        assert!(h.writes_log.is_empty());
        assert_eq!(out[4..], [5, 0x20, 9, 0x8123]);
    }

    #[test]
    fn a_full_log_stops_the_frame() {
        let mut h = Hooks { capacity: 2, ..Hooks::new(&[16]) };
        h.set_ranges(0, &[3, 0, 0xFFFF]);
        h.note_write(3, 0, 1, 0);
        assert_eq!(h.stop_before(0), stop::FRAME);
        h.note_write(3, 0, 1, 0);
        assert_eq!(h.stop_before(0), stop::RING);
    }

    #[test]
    fn coverage_is_per_processor_over_its_own_width_and_drains_clear() {
        let mut h = Hooks::new(&[16, 12]);
        h.configure(1 << flag::COVERAGE | 1 << (flag::COVERAGE + 1), i32::MIN, -1);
        h.record(0x8001);
        h.record_on(1, 0x1003);
        let (mut a, mut b, mut n) = ([0u8; 0x2000], [0u8; 0x200], 0i64);
        assert_eq!(drain_coverage(&mut h, 0, &mut a, &mut n), 0x2000);
        assert_eq!((a[0x1000], n), (0b10, 1));
        assert_eq!(drain_coverage(&mut h, 1, &mut b, &mut n), 0x200);
        assert_eq!((b[0], n), (0b1000, 1));
        assert_eq!(drain_coverage(&mut h, 0, &mut a, &mut n), 0x2000);
        assert_eq!((a[0x1000], n), (0, 0));
        h.configure(0, i32::MIN, -1);
        assert_eq!(drain_coverage(&mut h, 0, &mut a, &mut n), 0);
        assert!(!h.armed());
    }

    #[test]
    fn an_observer_stops_where_the_hooks_would_and_the_unobserved_one_never() {
        let mut h = Hooks::new(&[24, 16]);
        h.set_breakpoints(&[0x200, 0x200]);
        assert_eq!(Observer::before(&mut h, 0, 0x200), stop::BREAKPOINT);
        assert_eq!(Observer::before(&mut h, 1, 0x200), stop::FRAME);
        assert_eq!(Unobserved.before(0, 0x200), stop::FRAME);
        h.configure(flag::CALLS, i32::MIN, -1);
        Observer::called(&mut h, 0x100, 0x200, kind::CALL);
        assert_eq!(h.depth(), 1);
        Observer::returned(&mut h);
        assert_eq!(h.depth(), 0);
    }
}
