//! The debugger of one session over its machine: the C# `CoreDebugTarget` (spaces, processors, registers, the
//! expression contexts, views, telemetry), `CoreDebugBridge` (the registries pushed down as tables, the frame run
//! through `debug_run_frame`, the logs drained) and `CoreEngine`'s debugger points, with the surface the runtime calls at
//! the frame's points (EmuSen_Backend.md §29.4, §29.30, §29.34).

use crate::breakpoints::{BreakpointRegistry, CallKind, Evaluate, Unprepared};
use crate::conditions;
use crate::context::{CpuContext, Extra, Live, Register, Set, Source, TargetContext, UnitContext, UnitSymbols, own_names};
use crate::coverage::{CallStackRegistry, CoverageRegistry};
use crate::expr::{self, Evaluator};
use crate::info::{Info, ProcessorInfo};
use crate::expr::Context;
use crate::labels::LabelRegistry;
use crate::machined::{self, Machined, ViewMemory};
use crate::ring::Ring;
use crate::views;
use crate::watches::{FrameLogRegistry, FreezeRegistry, WatchKind, WatchRegistry};
use emusen_cheats::registry::{Applied, CheatRegistry};
use emusen_core_host::Machine;
use emusen_dotnet::text::{equals_ignore_case, to_lower};
use emusen_native::core::caps;
use emusen_native::debug::{flag, range, run, stop};
use emusen_systems::debug::{Reference, Unit};
use std::collections::HashMap;
use std::sync::Arc;

/// What a scope's context reads besides the machine: its code space, the pack's names and its unit symbols.
#[derive(Clone, Debug, Default)]
struct ScopeData {
    code: Option<String>,
    names: Vec<String>,
    units: UnitSymbols,
}

/// How a frame ended: whole, or halted in front of `address` on processor `processor` (machine info's id).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ran {
    Completed,
    Halted { processor: u32, address: u32 },
}

/// A processor as the debugger offers it, `DebugCpu`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cpu {
    /// Lowercase, what a scope word matches.
    pub name: String,
    pub description: String,
    pub can_halt: bool,
    pub coverage: bool,
    pub call_stack: bool,
    pub code_space: Option<String>,
    /// Whether a program counter is read for it.
    pub has_pc: bool,
}

/// A space as the debugger offers it, `IDebugMemorySpace`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpaceView {
    pub name: String,
    pub size: i32,
    pub writable: bool,
    pub side_effects: bool,
}

/// One counter with the value last read, `DebugCounterValue`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Counter {
    pub name: String,
    pub label: String,
    pub unit: String,
    pub kind: String,
    pub value: i64,
}

/// One instruction a disassembly lists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instruction {
    pub address: i32,
    pub bytes: Vec<u8>,
    pub mnemonic: String,
    pub operands: String,
}

/// What a frame's view ring learnt: the spaces it copies (index, offset, size), their total, the unit registers'.
#[derive(Clone, Debug, Default)]
struct Learnt {
    copied: Vec<(usize, usize, usize)>,
    total: usize,
    registers: usize,
    generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum View {
    Palettes,
    Sprites,
    Channels,
}

/// A pack's view decoded from a copy of the spaces it reads and the units' registers, made at each refresh.
struct ViewRing {
    view: View,
    ring: Ring<u8>,
    learnt: Option<Learnt>,
    scratch: Vec<i64>,
}

/// Processor 0's registers, read once as the C# target's were when it was made.
struct CpuRing {
    id: u32,
    names: Vec<(String, i32)>,
    ring: Ring<i64>,
}

pub struct Debugger {
    pub watches: WatchRegistry,
    pub frame_log: FrameLogRegistry,
    pub freezes: FreezeRegistry,
    pub labels: LabelRegistry,
    pub call_stack: CallStackRegistry,
    pub cheats: CheatRegistry,
    applied: Applied,
    /// Processor n's breakpoints, the main processor's first; one at least.
    breakpoints: Vec<BreakpointRegistry>,
    coverage: Vec<CoverageRegistry>,
    /// The registries of processors that cannot halt, made with each machine, `new BreakpointRegistry()`.
    spare: Vec<BreakpointRegistry>,
    halting: Vec<bool>,
    coverage_bits: Vec<Vec<u8>>,
    event_frame: i64,
    events: Vec<u32>,
    profile: Vec<i64>,
    table: Vec<i32>,
    stack_pairs: Vec<u32>,
    ranges: [Vec<u32>; 4],
    info: Info,
    attached: bool,
    supported: bool,
    reads: bool,
    generation: u64,
    cpu: Option<CpuRing>,
    cpus: Vec<CpuRing>,
    video: Ring<i64>,
    apu: Ring<i64>,
    counters: Ring<i64>,
    load: Ring<f64>,
    phase_scratch: [i64; 32],
    views: [ViewRing; 3],
    mutes: u32,
    references: HashMap<i32, (Reference, i32)>,
    /// Each scope's code space, extra names and unit symbols, made once a machine info so an evaluation makes none.
    scope_main: Arc<ScopeData>,
    scope_cpus: Vec<Arc<ScopeData>>,
    big_endian: bool,
    last_frame_ms: f64,
}

impl Default for Debugger {
    fn default() -> Self {
        Debugger::new()
    }
}

fn view_ring(view: View) -> ViewRing {
    ViewRing { view, ring: Ring::new(512), learnt: None, scratch: Vec::new() }
}

/// The units a kind selects into the video or the sound rows.
fn chosen(set: Set, kind: &str) -> bool {
    match set {
        Set::Video => kind == "video",
        _ => matches!(kind, "audio" | "timer" | "cartridge"),
    }
}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger {
            watches: WatchRegistry::new(),
            frame_log: FrameLogRegistry::new(),
            freezes: FreezeRegistry::new(),
            labels: LabelRegistry::default(),
            call_stack: CallStackRegistry::new(),
            cheats: CheatRegistry::new(),
            applied: Applied::default(),
            breakpoints: vec![BreakpointRegistry::new()],
            coverage: Vec::new(),
            spare: Vec::new(),
            halting: Vec::new(),
            coverage_bits: Vec::new(),
            event_frame: 0,
            events: vec![0; 4 * 1024],
            profile: vec![0; 2 * 256],
            table: Vec::new(),
            stack_pairs: Vec::new(),
            ranges: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            info: Info::default(),
            attached: false,
            supported: false,
            reads: false,
            generation: 0,
            cpu: None,
            cpus: Vec::new(),
            video: Ring::new(64),
            apu: Ring::new(64),
            counters: Ring::new(64),
            load: Ring::new(16),
            phase_scratch: [0; 32],
            views: [view_ring(View::Palettes), view_ring(View::Sprites), view_ring(View::Channels)],
            mutes: 0,
            references: HashMap::new(),
            scope_main: Arc::default(),
            scope_cpus: Vec::new(),
            big_endian: false,
            last_frame_ms: 0.0,
        }
    }

    // ---- The runtime's surface (EmuSen_Backend.md §29.30) --------------------------------------------------------

    /// A machine made for the session: its info read, one registry set a processor (the existing kept), the call stack
    /// reset, each processor asked whether it halts; the snapshots taken, as a C# target's are when made.
    pub fn attach(&mut self, machine: &mut Machine) {
        let lib = machine.library();
        self.supported = lib.claims(caps::DEBUG);
        self.reads = self.supported && lib.claims(caps::DEBUG_READS);
        self.read_info(machine);
        let count = self.info.processors.len().max(1);
        self.breakpoints.resize_with(count, BreakpointRegistry::new);
        self.coverage.resize_with(count, CoverageRegistry::new);
        self.breakpoints.truncate(count);
        self.coverage.truncate(count);
        self.coverage_bits = (0..count)
            .map(|n| {
                let bits = self.info.processors.get(n).map_or(24, |p| p.pc_bits).clamp(1, 24);
                vec![0u8; ((1i64 << bits) / 8).min(1 << 21) as usize]
            })
            .collect();
        self.call_stack.reset();
        self.halting = (0..count).map(|n| n == 0 || (self.supported && machine.debug_set_breakpoints(n as u32, &[]).is_ok())).collect();
        if self.supported {
            self.breakpoints[0].knows_stack = true;
        }
        self.spare = (0..count).map(|_| BreakpointRegistry::new()).collect();
        let processors = self.info.processors.clone();
        if self.cpu.is_none() {
            self.cpu = processors.first().map(|p| CpuRing { id: p.id, names: p.registers.clone(), ring: Ring::new(p.registers.len()) });
        }
        self.cpus = processors.iter().skip(1).map(|p| CpuRing { id: p.id, names: p.registers.clone(), ring: Ring::new(p.registers.len()) }).collect();
        self.attached = true;
        self.refresh(machine, self.last_frame_ms);
    }

    /// A MACHINE_INFO event drained: the info read again, the views learnt again at their next refresh.
    pub fn info_changed(&mut self, machine: &mut Machine) {
        self.read_info(machine);
    }

    /// The session closed.
    pub fn detach(&mut self) {
        self.attached = false;
    }

    fn read_info(&mut self, machine: &Machine) {
        self.info = Info::read(&machine.machine_info().unwrap_or_default());
        self.generation += 1;
        self.big_endian = emusen_systems::find(&self.info.system).and_then(|s| s.byte_order.as_deref()) == Some("big");
        let pack = views::symbol_names(&self.info.system);
        let stack = self.supported;
        let mut names = pack.clone();
        if stack {
            names.push("stackdepth".to_string());
        }
        self.scope_main = Arc::new(match self.info.processors.first() {
            Some(p) => ScopeData { code: self.code_space_of(p).map(str::to_string), names, units: UnitSymbols::new(&self.info.units, &own_names(&p.registers, &pack, stack)) },
            None => ScopeData::default(),
        });
        self.scope_cpus = self
            .info
            .processors
            .iter()
            .map(|p| Arc::new(ScopeData { code: self.code_space_of(p).map(str::to_string), names: Vec::new(), units: UnitSymbols::new(&self.info.units, &own_names(&p.registers, &[], false)) }))
            .collect();
    }

    /// `_cheatVersion = -1`: the next `refresh_cheats` sends the patch table and pokes again, as after a state load.
    pub fn cheats_stale(&mut self) {
        self.applied.stale();
    }

    /// The registry handed over, applied at once, as the C# `Cheats` setter does.
    pub fn set_cheats(&mut self, cheats: CheatRegistry) {
        self.cheats = cheats;
        self.applied.stale();
    }

    /// Step 2's first (b) point: the patch table and pokes, sent only when the registry's version moved.
    pub fn refresh_cheats(&mut self, machine: &mut Machine) {
        if self.attached {
            machined::refresh_cheats(machine, &self.info, &self.cheats, &mut self.applied);
        }
    }

    /// `ApplyCheats`: every enabled poke written now, between frames, the ones the core gates too.
    pub fn apply_cheats(&mut self, machine: &mut Machine) {
        if !self.attached {
            return;
        }
        machined::refresh_cheats(machine, &self.info, &self.cheats, &mut self.applied);
        let plain = std::cell::RefCell::new(Plain { machine, info: &self.info });
        self.cheats.apply_all(&mut |s, a| plain.borrow_mut().read(s, a), &mut |s, a, v| plain.borrow_mut().write(s, a, v));
    }

    /// `Armed`: whether anything can halt, record or report, so the frame runs through `run_frame`.
    pub fn armed(&self) -> bool {
        if !self.supported || !self.attached {
            return false;
        }
        self.call_stack.is_profiling() || self.listening() || self.breakpoints.iter().any(|b| !b.is_quiet()) || self.coverage.iter().any(CoverageRegistry::is_armed)
    }

    fn listening(&self) -> bool {
        self.watches.has_watches() || self.breakpoints.iter().any(BreakpointRegistry::watches_writes)
    }

    fn listening_to_reads(&self) -> bool {
        self.reads && (self.watches.watches().iter().any(|w| w.kind != WatchKind::Write) || self.breakpoints.iter().any(|b| b.data_breakpoints().iter().any(|d| d.on_read)))
    }

    /// `CanHalt`: whether processor `n` takes breakpoints.
    pub fn can_halt(&self, n: usize) -> bool {
        self.supported && self.halting.get(n).copied().unwrap_or(false)
    }

    /// The frame through `debug_run_frame`: whole, or halted with the frame left open for the next call to resume.
    pub fn run_frame(&mut self, machine: &mut Machine, resuming: bool) -> Result<Ran, (i32, u64)> {
        self.event_frame = machine.frame_count();
        let mut flags = if resuming { run::UNCHECKED } else { 0 };
        loop {
            self.push_tables(machine);
            let answer = machine.debug_run_frame(flags);
            let noted = self.drain(machine);
            flags = run::UNCHECKED | run::CONTINUE;
            let (reasons, stopped, pc, _) = match answer {
                Ok(a) => a,
                Err(status) => return Err((status, 0)),
            };
            // An interrupt stopped for with none in the call log, as a loop that dispatches interrupts outside it reports.
            if reasons as u32 & stop::INTERRUPT != 0 && !noted {
                self.breakpoints[0].note_interrupt(CallKind::Irq);
            }
            if reasons == 0 {
                return Ok(Ran::Completed);
            }
            let n = if (stopped as usize) < self.breakpoints.len() { stopped as usize } else { 0 };
            if !self.breakpoints[n].could_break() {
                continue;
            }
            let depth = self.call_stack.depth();
            let halt = self.with_registry(machine, n, |registry, eval| registry.should_break(pc as i32, depth, eval));
            if halt {
                return Ok(Ran::Halted { processor: stopped, address: pc as u32 });
            }
        }
    }

    /// The engine's frame end in its order: the frame log, the host's pokes unless the frame holds them, the freezes,
    /// the main processor's frame.
    pub fn frame_end(&mut self, machine: &mut Machine) {
        if !self.attached {
            return;
        }
        let frame = machine.frame_count();
        let held = machine.frame_info().is_ok_and(|f| f.flags & emusen_native::core::sys::flags::FRAME_CHEATS_HELD != 0);
        let big = self.big_endian;
        // The C# reads through ReadSpace, whose cheat refresh is a version compare here: the frame began with one.
        let plain = std::cell::RefCell::new(Plain { machine, info: &self.info });
        let info = &self.info;
        self.frame_log.record_frame(frame, |space, address, width| {
            let big = match info.space_named(space).and_then(|s| s.byte_order.as_deref()) {
                Some("big") => true,
                Some("little") => false,
                _ => big,
            };
            let mut value = 0i64;
            for i in 0..width {
                let b = plain.borrow_mut().read(space, address.wrapping_add(i)) as i64;
                value = if big { value << 8 | b } else { value | b << (8 * i) };
            }
            value
        });
        let host = self.applied.host(&self.cheats);
        if host.master_enabled() && !held {
            host.apply_all(&mut |s, a| plain.borrow_mut().read(s, a), &mut |s, a, v| plain.borrow_mut().write(s, a, v));
        }
        if self.freezes.count() > 0 {
            self.freezes.enforce(|s, a| plain.borrow_mut().read(s, a), |s, a, v| plain.borrow_mut().write(s, a, v));
        }
        self.breakpoints[0].note_frame(frame);
    }

    /// `RefreshProviders`: every snapshot, the registers, the views, the counters and the load.
    pub fn refresh(&mut self, machine: &mut Machine, last_frame_ms: f64) {
        self.last_frame_ms = last_frame_ms;
        if !self.attached {
            return;
        }
        self.refresh_registers(machine);
        for v in 0..3 {
            self.refresh_view(machine, v);
        }
        self.refresh_units(machine, Set::Video);
        self.refresh_units(machine, Set::Apu);
        let counters = &self.counters;
        counters.refresh(|slot| {
            if machine.library().exports().telemetry.is_none() || !machine.library().claims(caps::TELEMETRY) {
                return 0;
            }
            machine.telemetry(slot).unwrap_or(0)
        });
        let (info, scratch, ms) = (&self.info, &mut self.phase_scratch, last_frame_ms);
        self.load.refresh(|slot| {
            if slot.len() < 2 {
                return 2;
            }
            let (n, named) = phase_milliseconds(machine, scratch, &mut slot[1..], ms);
            if n + 1 > slot.len() {
                return n + 1;
            }
            let budget = 1000.0 / info.frame_rate_hz();
            for v in slot.iter_mut().take(n + 1).skip(1) {
                *v = (*v / budget * 100.0).min(100.0);
            }
            slot[0] = if named { 1.0 } else { 0.0 };
            n + 1
        });
    }

    /// The runtime's lines the watches said, for the `log` event.
    pub fn take_said(&mut self, into: &mut Vec<String>) {
        into.append(&mut self.watches.said);
    }

    // ---- The bridge (CoreDebugBridge) ------------------------------------------------------------------------------

    fn reported_space(&self, name: &str, loads: bool) -> Option<u32> {
        self.info.spaces.iter().find(|s| (if loads { s.reports_loads } else { s.reports_stores }) && equals_ignore_case(&s.name, name)).map(|s| s.id)
    }

    /// `PushTables`: the flags, the stack, each processor's breakpoints and the four kinds of range.
    fn push_tables(&mut self, machine: &mut Machine) {
        let main = &self.breakpoints[0];
        let mut flags = flag::CALLS | flag::INTERRUPTS;
        if self.listening() {
            flags |= flag::WRITES;
        }
        if self.listening_to_reads() {
            flags |= flag::READS;
        }
        if main.single_step_armed() || main.has_pending_break() {
            flags |= flag::EACH;
        }
        if self.call_stack.is_profiling() {
            flags |= flag::PROFILING;
        }
        for (n, c) in self.coverage.iter().enumerate().take(24) {
            if c.is_armed() {
                flags |= 1 << (flag::COVERAGE + n as u32);
            }
        }
        let _ = machine.debug_set(flags, main.step_depth_target().unwrap_or(i32::MIN), main.depth_guard());
        if machine.library().exports().debug_set_stack.is_some() {
            self.stack_pairs.clear();
            for f in self.call_stack.frames() {
                self.stack_pairs.push(f.source as u32);
                self.stack_pairs.push(f.target as u32);
            }
            let _ = machine.debug_set_stack(&self.stack_pairs);
        }
        for n in 0..self.breakpoints.len() {
            if n > 0 && !self.can_halt(n) {
                continue;
            }
            let registry = &self.breakpoints[n];
            self.table.clear();
            if n > 0 && (registry.single_step_armed() || registry.has_pending_break()) {
                self.table.extend([0, i32::MAX]);
            }
            for b in registry.breakpoints().iter().filter(|b| b.enabled) {
                self.table.extend([b.address, b.end]);
            }
            let _ = machine.debug_set_breakpoints(n as u32, &self.table);
        }
        for r in &mut self.ranges {
            r.clear();
        }
        for w in self.watches.watches() {
            if w.length <= 0 {
                continue;
            }
            let last = w.address.wrapping_add(w.length).wrapping_sub(1) as u32;
            if w.kind != WatchKind::Read {
                if let Some(space) = self.reported_space(&w.space, false) {
                    self.ranges[range::WATCH as usize].extend([space, w.address as u32, last]);
                }
            }
            if w.kind != WatchKind::Write && self.reads {
                if let Some(space) = self.reported_space(&w.space, true) {
                    self.ranges[range::READ_WATCH as usize].extend([space, w.address as u32, last]);
                }
            }
        }
        for b in self.breakpoints.iter().flat_map(|r| r.data_breakpoints()) {
            if !b.enabled || (b.on_read && !self.reads) {
                continue;
            }
            let Some(space) = self.reported_space(&b.space, b.on_read) else { continue };
            let kind = if b.on_read { range::READ_BREAK } else { range::BREAK };
            self.ranges[kind as usize].extend([space, b.address as u32, b.end as u32]);
        }
        let _ = machine.debug_set_ranges(range::WATCH, &self.ranges[0]);
        let _ = machine.debug_set_ranges(range::BREAK, &self.ranges[1]);
        if !self.reads {
            return;
        }
        let _ = machine.debug_set_ranges(range::READ_WATCH, &self.ranges[2]);
        let _ = machine.debug_set_ranges(range::READ_BREAK, &self.ranges[3]);
    }

    fn grow<T: Default + Clone>(buffer: &mut Vec<T>, needed: usize) {
        if buffer.len() < needed {
            let n = needed.max(buffer.len() * 2);
            buffer.resize(n, T::default());
        }
    }

    /// `Drain`: the logs into the registries in the C# observers' order: stores, loads, calls and returns, the counts,
    /// coverage; true when an interrupt was among the calls.
    fn drain(&mut self, machine: &mut Machine) -> bool {
        let mut interrupted = false;
        let writes = machine.debug_writes(&mut []).unwrap_or(0);
        if writes > 0 {
            Self::grow(&mut self.events, writes * 4);
            let _ = machine.debug_writes(&mut self.events);
            for i in 0..writes {
                let e = [self.events[4 * i], self.events[4 * i + 1], self.events[4 * i + 2], self.events[4 * i + 3]];
                let Some(space) = self.info.spaces.iter().find(|s| s.id == e[0]).map(|s| s.name.clone()) else { continue };
                let (address, value, pc) = (e[1] as i32, e[2] as u8, e[3]);
                self.watches.record(WatchKind::Write, &space, address, value, || format!("PC=${pc:04X}"));
                for n in 0..self.breakpoints.len() {
                    self.with_registry(machine, n, |r, eval| r.note_write(&space, address, value, eval));
                }
            }
        }
        if self.reads {
            let reads = machine.debug_reads(&mut []).unwrap_or(0);
            if reads > 0 {
                Self::grow(&mut self.events, reads * 5);
                let _ = machine.debug_reads(&mut self.events);
                for i in 0..reads {
                    let e = [self.events[5 * i], self.events[5 * i + 1], self.events[5 * i + 2], self.events[5 * i + 3], self.events[5 * i + 4]];
                    let Some(space) = self.info.spaces.iter().find(|s| s.id == e[0]).map(|s| s.name.clone()) else { continue };
                    let (address, value, pc) = (e[1] as i32, e[2] as u8, e[3]);
                    if e[4] == range::READ_WATCH {
                        self.watches.record(WatchKind::Read, &space, address, value, || format!("PC=${pc:04X}"));
                    } else {
                        for n in 0..self.breakpoints.len() {
                            self.with_registry(machine, n, |r, eval| r.note_read(&space, address, value, eval));
                        }
                    }
                }
            }
        }
        let calls = machine.debug_calls(&mut []).unwrap_or(0);
        if calls > 0 {
            Self::grow(&mut self.events, calls * 3);
            let _ = machine.debug_calls(&mut self.events);
            for i in 0..calls {
                let (kind, source, target) = (self.events[3 * i], self.events[3 * i + 1], self.events[3 * i + 2]);
                if kind == 0 {
                    self.call_stack.note_pop();
                    continue;
                }
                let frame = match kind {
                    2 => CallKind::Irq,
                    3 => CallKind::Nmi,
                    4 => CallKind::Brk,
                    5 => CallKind::Cop,
                    _ => CallKind::Call,
                };
                self.call_stack.note_push(source as i32, target as i32, frame, self.event_frame, self.coverage.first_mut());
                if frame == CallKind::Call {
                    continue;
                }
                self.breakpoints[0].note_interrupt(frame);
                interrupted = true;
            }
        }
        let runs = machine.debug_profile(&mut []).unwrap_or(0);
        if runs > 0 {
            Self::grow(&mut self.profile, runs * 2);
            let _ = machine.debug_profile(&mut self.profile);
            for i in 0..runs {
                self.call_stack.note_instructions(self.profile[2 * i] as i32, self.profile[2 * i + 1]);
            }
        }
        for n in 0..self.coverage.len() {
            if !self.coverage[n].is_armed() {
                continue;
            }
            if let Ok((length, recorded)) = machine.debug_coverage(n as u32, &mut self.coverage_bits[n]) {
                if length > 0 {
                    let bits = &self.coverage_bits[n];
                    self.coverage[n].merge(&bits[..length.min(bits.len())], recorded);
                }
            }
        }
        interrupted
    }

    /// Registry `n` with the evaluator of its scope, as `BreakpointExpressions.Live` points one: the main processor's
    /// registry reads the target's context, another its processor's.
    fn with_registry<R>(&mut self, machine: &mut Machine, n: usize, f: impl FnOnce(&mut BreakpointRegistry, &mut dyn Evaluate) -> R) -> R {
        let mut registries = std::mem::take(&mut self.breakpoints);
        let registry = &mut registries[n];
        let result = if registry.prepared {
            let mut eval = Scoped { d: self, machine, processor: if n == 0 { None } else { Some(n) } };
            f(registry, &mut eval)
        } else {
            f(registry, &mut Unprepared)
        };
        self.breakpoints = registries;
        result
    }

    // ---- The target (CoreDebugTarget) ------------------------------------------------------------------------------

    pub fn info(&self) -> &Info {
        &self.info
    }

    pub fn is_attached(&self) -> bool {
        self.attached
    }

    /// Whether the machine halts: DEBUG claimed and a game loaded.
    pub fn halts(&self) -> bool {
        self.supported && self.attached
    }

    /// The main processor's registry, the session's own.
    pub fn breakpoints(&mut self) -> &mut BreakpointRegistry {
        &mut self.breakpoints[0]
    }

    /// Processor `n`'s registry, `DebugCpu.Breakpoints`: the bridge's where it halts, else one of its own.
    pub fn processor_breakpoints(&mut self, n: usize) -> Option<&mut BreakpointRegistry> {
        if n >= self.info.processors.len() {
            return None;
        }
        if n == 0 || (self.halts() && n < self.breakpoints.len()) {
            Some(&mut self.breakpoints[n])
        } else {
            self.spare.get_mut(n)
        }
    }

    /// The main processor's coverage, `Coverage`, where the machine halts.
    pub fn coverage(&mut self, n: usize) -> Option<&mut CoverageRegistry> {
        if self.halts() { self.coverage.get_mut(n) } else { None }
    }

    /// The call stack, where the machine halts.
    pub fn call_stack(&mut self) -> Option<&mut CallStackRegistry> {
        if self.halts() { Some(&mut self.call_stack) } else { None }
    }

    /// `DebugCpus`: one a processor machine info names.
    pub fn processors(&self) -> Vec<Cpu> {
        let halts = self.halts();
        self.info
            .processors
            .iter()
            .enumerate()
            .map(|(i, p)| Cpu {
                name: to_lower(&p.name),
                description: p.name.clone(),
                can_halt: halts && self.can_halt(i),
                coverage: halts && i < self.coverage.len(),
                call_stack: halts && i == 0,
                code_space: self.code_space_of(p).map(str::to_string),
                has_pc: halts,
            })
            .collect()
    }

    /// The program counter of processor `index`, live; 0 without one.
    pub fn program_counter(&self, machine: &Machine, index: usize) -> i32 {
        let Some(p) = self.info.processors.get(index) else { return 0 };
        if !self.supported || !self.attached {
            return 0;
        }
        machine.debug_pc(p.id).map_or(0, |pc| pc as i32)
    }

    /// The space a processor's code is listed from: machine info's, else `<name>BUS`, `<name>PRG`, or the first.
    fn code_space_of(&self, p: &ProcessorInfo) -> Option<&str> {
        let spaces = &self.info.spaces;
        spaces
            .iter()
            .find(|s| p.code_space == Some(s.id))
            .or_else(|| spaces.iter().find(|s| equals_ignore_case(&s.name, &format!("{}BUS", p.name))))
            .or_else(|| spaces.iter().find(|s| equals_ignore_case(&s.name, &format!("{}PRG", p.name))))
            .or_else(|| if p.id == 0 { spaces.first() } else { None })
            .map(|s| s.name.as_str())
    }

    /// `GetMemorySpaces`.
    pub fn spaces(&self, machine: &Machine) -> Vec<SpaceView> {
        if !self.attached {
            return Vec::new();
        }
        self.info
            .spaces
            .iter()
            .map(|s| SpaceView { name: s.name.clone(), size: machine.space_size(s.id).unwrap_or(0).min(i32::MAX as u64) as i32, writable: !s.read_only, side_effects: s.side_effects })
            .collect()
    }

    fn space_index(&self, name: &str) -> Option<usize> {
        self.info.spaces.iter().position(|s| equals_ignore_case(&s.name, name))
    }

    /// A byte of a listed space, by the space's rule.
    pub fn space_read(&mut self, machine: &mut Machine, name: &str, address: i32) -> u8 {
        let Some(i) = self.space_index(name).filter(|_| self.attached) else { return 0 };
        Machined { machine, info: &self.info, cheats: &self.cheats, applied: &mut self.applied }.read_listed(i, address)
    }

    /// A byte written to a listed space, unless it is read-only.
    pub fn space_write(&mut self, machine: &mut Machine, name: &str, address: i32, value: u8) {
        let Some(i) = self.space_index(name).filter(|_| self.attached) else { return };
        Machined { machine, info: &self.info, cheats: &self.cheats, applied: &mut self.applied }.write_listed(i, address, value);
    }

    fn set_names(&self, set: Set) -> Option<(&[(String, i32)], &Ring<i64>)> {
        match set {
            Set::Processor(0) => self.cpu.as_ref().map(|c| (c.names.as_slice(), &c.ring)),
            Set::Processor(i) => self.cpus.get(i - 1).map(|c| (c.names.as_slice(), &c.ring)),
            Set::Coprocessor => self.coprocessor().and_then(|i| self.set_names(Set::Processor(i))),
            Set::Video | Set::Apu => None,
        }
    }

    /// `DebugCpus.Coprocessor`: the first processor after the main one that is not the sound processor.
    pub fn coprocessor(&self) -> Option<usize> {
        self.info.processors.iter().enumerate().skip(1).find(|(_, p)| !equals_ignore_case(&p.name, "spc")).map(|(i, _)| i)
    }

    /// A set's registers as its snapshot shows them, `Current`.
    pub fn registers(&self, set: Set) -> Vec<Register> {
        match set {
            Set::Video | Set::Apu => self.unit_rows(set),
            _ => match self.set_names(set) {
                Some((names, ring)) => ring.with(|v| names.iter().enumerate().map(|(i, (n, b))| Register { name: n.clone(), value: v.get(i).copied().unwrap_or(0) as u64, bits: *b }).collect()),
                None => Vec::new(),
            },
        }
    }

    fn register(&self, set: Set, name: &str) -> Option<u64> {
        match set {
            Set::Video | Set::Apu => self.unit_rows(set).into_iter().find(|r| equals_ignore_case(&r.name, name)).map(|r| r.value),
            _ => {
                let (names, ring) = self.set_names(set)?;
                let i = names.iter().position(|(n, _)| equals_ignore_case(n, name))?;
                Some(ring.with(|v| v.get(i).copied().unwrap_or(0)) as u64)
            }
        }
    }

    /// `BuildUnits`: the chosen units' rows from the ring, a cartridge headed by a row naming its board.
    fn unit_rows(&self, set: Set) -> Vec<Register> {
        if !self.attached {
            return Vec::new();
        }
        let ring = if set == Set::Video { &self.video } else { &self.apu };
        ring.with(|values| {
            let n = values.len();
            let mut rows = Vec::new();
            let mut at = 0;
            for unit in self.info.units.iter().filter(|u| chosen(set, &u.kind)) {
                if unit.kind == "cartridge" {
                    let u = Unit {
                        name: unit.name.clone(),
                        kind: unit.kind.clone(),
                        registers: unit.registers.iter().enumerate().map(|(i, (r, _))| (r.clone(), if at + i < n { values[at + i] as u64 } else { 0 })).collect(),
                    };
                    let label = views::unit_label(&self.info.system, &u).unwrap_or_else(|| unit.name.clone());
                    rows.push(Register { name: format!("Board:{label}"), value: 0, bits: 8 });
                }
                for (name, bits) in &unit.registers {
                    let value = if at < n {
                        at += 1;
                        values[at - 1] as u64
                    } else {
                        0
                    };
                    rows.push(Register { name: name.clone(), value, bits: *bits });
                }
            }
            rows
        })
    }

    /// The counters with the values last read, `Counters.Current`.
    pub fn counters(&self) -> Vec<Counter> {
        if !self.attached {
            return Vec::new();
        }
        self.counters.with(|v| {
            self.info
                .counters
                .iter()
                .enumerate()
                .map(|(i, k)| Counter { name: k.name.clone(), label: k.label.clone(), unit: k.unit.clone(), kind: k.kind.clone(), value: v.get(i).copied().unwrap_or(0) })
                .collect()
        })
    }

    /// The phases' loads, `HardwareLoad.Current`: each name and its share of a frame.
    pub fn load(&self) -> Vec<(String, f64)> {
        self.load.with(|v| {
            if v.is_empty() {
                return Vec::new();
            }
            let names: &[String] = if self.attached { &self.info.phases } else { &[] };
            (1..v.len())
                .map(|i| {
                    let name = if v[0] == 0.0 { "frame".to_string() } else { names.get(i - 1).cloned().unwrap_or_else(|| format!("phase {}", i - 1)) };
                    (name, v[i])
                })
                .collect()
        })
    }

    fn refresh_registers(&mut self, machine: &mut Machine) {
        if let Some(c) = &self.cpu {
            c.ring.refresh(|slot| if self.attached { machined::registers(machine, c.id, slot) } else { 0 });
        }
        for c in &self.cpus {
            c.ring.refresh(|slot| machined::registers(machine, c.id, slot));
        }
    }

    fn refresh_units(&self, machine: &Machine, set: Set) {
        let ring = if set == Set::Video { &self.video } else { &self.apu };
        ring.refresh(|slot| {
            if !self.attached {
                return 0;
            }
            let mut at = 0;
            for unit in self.info.units.iter().filter(|u| chosen(set, &u.kind)) {
                let count = unit.registers.len();
                if at + count <= slot.len() {
                    machined::registers(machine, unit.id, &mut slot[at..at + count]);
                }
                at += count;
            }
            at
        });
    }

    /// The units with their registers read live, `LiveUnits`.
    fn live_units(&self, machine: &Machine) -> Vec<Unit> {
        if !self.attached {
            return Vec::new();
        }
        self.info
            .units
            .iter()
            .map(|u| {
                let values = machined::registers_all(machine, u.id);
                Unit { name: u.name.clone(), kind: u.kind.clone(), registers: u.registers.iter().enumerate().map(|(i, (n, _))| (n.clone(), values.get(i).copied().unwrap_or(0) as u64)).collect() }
            })
            .collect()
    }

    /// The units from a frame's copy of their values.
    fn copied_units(&self, copy: &[u8]) -> Vec<Unit> {
        let mut at = 0usize;
        self.info
            .units
            .iter()
            .map(|u| {
                let registers = u
                    .registers
                    .iter()
                    .enumerate()
                    .map(|(i, (n, _))| {
                        let v = if at + 8 * (i + 1) <= copy.len() { i64::from_le_bytes(copy[at + 8 * i..at + 8 * i + 8].try_into().unwrap()) } else { 0 };
                        (n.clone(), v as u64)
                    })
                    .collect();
                at += 8 * u.registers.len();
                Unit { name: u.name.clone(), kind: u.kind.clone(), registers }
            })
            .collect()
    }

    /// The view `v` over `memory`, as its ring builds it.
    fn decode(&self, v: View, memory: &mut dyn emusen_systems::debug::Memory, units: &[Unit]) -> Decoded {
        let system = self.info.system.as_str();
        match v {
            View::Palettes => Decoded::Palettes(views::palettes(system, memory, units)),
            View::Sprites => Decoded::Sprites(views::sprites(system, memory, units)),
            View::Channels => Decoded::Channels(views::audio_channels(system, units, self.mutes)),
        }
    }

    /// A view ring's refresh: learnt once a machine info, then the spaces it reads and the units' registers copied.
    fn refresh_view(&mut self, machine: &mut Machine, v: usize) {
        let has = self.attached && views::has(&self.info.system);
        if has && self.views[v].learnt.as_ref().is_none_or(|l| l.generation != self.generation) {
            let units = self.live_units(machine);
            let view = self.views[v].view;
            let mut cheats_applied = std::mem::take(&mut self.applied);
            let (copied, total) = {
                let mut at = Machined { machine: &mut *machine, info: &self.info, cheats: &self.cheats, applied: &mut cheats_applied };
                let mut memory = ViewMemory::live(&mut at);
                let _ = self.decode(view, &mut memory, &units);
                let touched = memory.touched.clone();
                let mut copied = Vec::new();
                let mut offset = 0usize;
                for (i, t) in touched.iter().enumerate() {
                    let s = &memory.spaces_ref()[i];
                    if !t || s.side_effects || s.size == 0 {
                        continue;
                    }
                    copied.push((i, offset, s.size as usize));
                    offset += s.size as usize;
                }
                (copied, offset)
            };
            self.applied = cheats_applied;
            let registers = self.info.units.iter().map(|u| u.registers.len()).sum();
            let max = self.info.units.iter().map(|u| u.registers.len()).max().unwrap_or(0);
            self.views[v].scratch = vec![0; max];
            self.views[v].learnt = Some(Learnt { copied, total, registers, generation: self.generation });
        }
        let info = &self.info;
        let vr = &mut self.views[v];
        let (learnt, scratch, ring) = (&vr.learnt, &mut vr.scratch, &vr.ring);
        ring.refresh(|slot| {
            let Some(l) = learnt.as_ref().filter(|_| has) else { return 0 };
            let total = l.total + 8 * l.registers;
            if total > slot.len() {
                return total;
            }
            for &(index, offset, size) in &l.copied {
                let _ = machine.space_read(info.spaces[index].id, 0, &mut slot[offset..offset + size]);
            }
            let mut at = l.total;
            for unit in &info.units {
                let n = machined::registers(machine, unit.id, scratch).min(unit.registers.len());
                for i in 0..unit.registers.len() {
                    let value = if i < n { scratch[i] } else { 0 };
                    slot[at..at + 8].copy_from_slice(&value.to_le_bytes());
                    at += 8;
                }
            }
            total
        });
    }

    /// A view as its ring holds it, decoded from the newest copy.
    fn current(&mut self, machine: &mut Machine, v: usize) -> Decoded {
        let none = match self.views[v].view {
            View::Palettes => Decoded::Palettes(Vec::new()),
            View::Sprites => Decoded::Sprites(Vec::new()),
            View::Channels => Decoded::Channels(Vec::new()),
        };
        if !self.attached || !views::has(&self.info.system) {
            return none;
        }
        let copy = self.views[v].ring.newest();
        let Some(l) = self.views[v].learnt.clone() else { return none };
        let units = self.copied_units(copy.get(l.total..).unwrap_or(&[]));
        let mut applied = std::mem::take(&mut self.applied);
        let decoded = {
            let mut at = Machined { machine, info: &self.info, cheats: &self.cheats, applied: &mut applied };
            let mut memory = ViewMemory::over(&mut at, &l.copied, &copy);
            self.decode(self.views[v].view, &mut memory, &units)
        };
        self.applied = applied;
        decoded
    }

    pub fn palettes(&mut self, machine: &mut Machine) -> Vec<views::Palette> {
        match self.current(machine, 0) {
            Decoded::Palettes(p) => p,
            _ => Vec::new(),
        }
    }

    pub fn sprites(&mut self, machine: &mut Machine) -> Vec<views::Sprite> {
        match self.current(machine, 1) {
            Decoded::Sprites(s) => s,
            _ => Vec::new(),
        }
    }

    pub fn audio_channels(&mut self, machine: &mut Machine) -> Vec<views::AudioChannel> {
        match self.current(machine, 2) {
            Decoded::Channels(c) => c,
            _ => Vec::new(),
        }
    }

    /// A view over the live spaces, `ViewSpaces()`, and the units read live.
    fn with_view_memory<R>(&mut self, machine: &mut Machine, f: impl FnOnce(&str, &mut ViewMemory<'_, '_>, &[Unit]) -> R) -> R {
        let units = self.live_units(machine);
        let mut applied = std::mem::take(&mut self.applied);
        let result = {
            let mut at = Machined { machine, info: &self.info, cheats: &self.cheats, applied: &mut applied };
            let mut memory = ViewMemory::live(&mut at);
            f(&self.info.system, &mut memory, &units)
        };
        self.applied = applied;
        result
    }

    pub fn tile_sheet(&mut self, machine: &mut Machine) -> views::Picture {
        if !self.attached {
            return (Vec::new(), 0, 0);
        }
        self.with_view_memory(machine, |s, m, _| views::tile_sheet(s, m))
    }

    pub fn palette_swatch(&mut self, machine: &mut Machine) -> views::Picture {
        if !self.attached {
            return (Vec::new(), 0, 0);
        }
        self.with_view_memory(machine, |s, m, u| views::palette_swatch(s, m, u))
    }

    pub fn tilemap_entry_stride(&self) -> i32 {
        if self.attached { views::tilemap_entry_stride(&self.info.system) } else { 0 }
    }

    pub fn max_sprites(&self) -> i32 {
        if self.attached { views::max_sprites(&self.info.system) } else { 0 }
    }

    /// `DecodeTilemapEntry` over a listed space, read live.
    pub fn decode_tilemap_entry(&mut self, machine: &mut Machine, space: &str, address: i32) -> String {
        if !self.attached || !views::has(&self.info.system) {
            return String::new();
        }
        let Some(i) = self.space_index(space) else { return String::new() };
        let name = self.info.spaces[i].name.clone();
        self.with_view_memory(machine, |s, m, u| {
            let mut live = LiveSpace { memory: m, index: i };
            views::decode_tilemap_entry(s, &mut live, &name, address, u)
        })
    }

    /// `DecodeTilePixels` over a listed space, read live; the refusal as `ArgumentException`'s message.
    pub fn decode_tile_pixels(&mut self, machine: &mut Machine, space: &str, address: i32, bpp: i32) -> Result<Vec<u8>, String> {
        if !self.attached || !views::has(&self.info.system) {
            return Ok(Vec::new());
        }
        let Some(i) = self.space_index(space) else { return Ok(Vec::new()) };
        let name = self.info.spaces[i].name.clone();
        self.with_view_memory(machine, |s, m, _| {
            let mut live = LiveSpace { memory: m, index: i };
            views::decode_tile_pixels(s, &mut live, &name, address, bpp).map_err(|(words, param)| match param {
                Some(p) => format!("{words} (Parameter '{p}')"),
                None => words,
            })
        })
    }

    pub fn dma_channels(&mut self, machine: &mut Machine) -> Vec<views::DmaChannel> {
        if !self.attached {
            return Vec::new();
        }
        self.with_view_memory(machine, |s, m, u| views::dma_channels(s, m, u))
    }

    pub fn name_dma_destination(&self, register: u8) -> Option<String> {
        if self.attached { views::name_dma_destination(&self.info.system, register) } else { None }
    }

    pub fn interrupt_vectors(&self) -> &'static [views::Vector] {
        if self.attached { views::vectors(&self.info.system) } else { &[] }
    }

    /// `ResolvePhysical`: the console's map for an address on the main processor's bus, after the core's translation.
    pub fn resolve_physical(&mut self, machine: &mut Machine, cpu_address: i32) -> Option<(String, i32, bool)> {
        if !self.attached {
            return None;
        }
        let mut address = cpu_address as u32;
        if machine.library().claims(caps::DEBUG_TRANSLATE) {
            match machine.translate(0, address) {
                Ok(Some(bus)) => address = bus,
                _ => return Some(("unmapped".to_string(), cpu_address, false)),
            }
        }
        self.with_view_memory(machine, |s, m, u| views::resolve_physical(s, address as i32, m, u))
    }

    /// `GetSummaryText`: the engine, system and frame, then the pack's lines from processor 0's live registers.
    pub fn summary(&mut self, machine: &mut Machine, display_name: &str, version: &str) -> String {
        if !self.attached {
            return format!("{display_name} {version}, no game");
        }
        let head = format!("{display_name} {version}, {}, frame {}", self.info.system, machine.frame_count());
        let Some(p) = self.info.processors.first().cloned() else { return head };
        let values = machined::registers_all(machine, p.id);
        let registers: Vec<(String, u64)> = p.registers.iter().enumerate().map(|(i, (n, _))| (n.clone(), values.get(i).copied().unwrap_or(0) as u64)).collect();
        match self.with_view_memory(machine, |s, m, u| views::summary(s, &registers, m, u)) {
            Some(more) => format!("{head}\n{more}"),
            None => head,
        }
    }

    /// `Disassemble`: DEBUG_DISASSEMBLE's records for processor 0 over the space named, else the pack's listing; each
    /// reference kept for `reference`.
    pub fn disassemble(&mut self, machine: &mut Machine, space: &str, address: i32, count: i32) -> Vec<Instruction> {
        if !self.attached {
            return Vec::new();
        }
        if !machine.library().claims(caps::DEBUG_DISASSEMBLE) {
            if !views::has(&self.info.system) {
                return Vec::new();
            }
            let Some(i) = self.space_index(space) else { return Vec::new() };
            let name = self.info.spaces[i].name.clone();
            let listed = self.with_view_memory(machine, |s, m, _| {
                let mut live = LiveSpace { memory: m, index: i };
                views::disassemble(s, &mut live, &name, address, count)
            });
            let mut out = Vec::with_capacity(listed.len());
            for (at, bytes, mnemonic, operands, reference) in listed {
                match reference {
                    Some(r) => self.references.insert(at, r),
                    None => self.references.remove(&at),
                };
                out.push(Instruction { address: at, bytes, mnemonic, operands });
            }
            return out;
        }
        let Some(id) = self.info.space_named(space).map(|s| s.id) else { return Vec::new() };
        let text = match machine.disassemble(0, id, address as u32, count.max(0) as u32) {
            Ok(t) if !t.is_empty() => t,
            _ => "[]".to_string(),
        };
        let doc = emusen_native::json::parse(text.as_bytes()).unwrap_or(emusen_native::json::Value::Null);
        let mut out = Vec::new();
        for r in doc.as_array().unwrap_or(&[]) {
            let at = r.get("address").and_then(emusen_native::json::Value::as_i64).unwrap_or(0) as i32;
            let bytes = r.get("bytes").and_then(emusen_native::json::Value::as_array).unwrap_or(&[]).iter().map(|b| b.as_i64().unwrap_or(0) as u8).collect();
            let text_of = |k: &str| r.get(k).and_then(emusen_native::json::Value::as_str).unwrap_or("").to_string();
            out.push(Instruction { address: at, bytes, mnemonic: text_of("mnemonic"), operands: text_of("operands") });
            match r.get("reference").filter(|v| matches!(v, emusen_native::json::Value::Object(_))) {
                Some(reference) => {
                    let kind = match reference.get("kind").and_then(emusen_native::json::Value::as_str) {
                        Some("call") => Reference::Call,
                        Some("write") => Reference::Write,
                        _ => Reference::Read,
                    };
                    let target = reference.get("target").and_then(emusen_native::json::Value::as_i64).unwrap_or(0) as i32;
                    self.references.insert(at, (kind, target));
                }
                None => {
                    self.references.remove(&at);
                }
            }
        }
        out
    }

    /// `ClassifyStaticReference`: what the last listing said the instruction at `address` names.
    pub fn reference(&self, address: i32) -> Option<(Reference, i32)> {
        self.references.get(&address).copied()
    }

    /// `SetChannelMuted`: machine info's channel n is the mutes' bit n.
    pub fn set_channel_muted(&mut self, machine: &mut Machine, index: i32, muted: bool) {
        if !(0..=31).contains(&index) || !self.attached {
            return;
        }
        self.mutes = if muted { self.mutes | 1 << index } else { self.mutes & !(1 << index) };
        if machine.library().claims(caps::MUTES) {
            let _ = machine.set_mutes(self.mutes);
        }
    }

    /// `GetAudioSamples`: the buffered samples, not drained, and the rate.
    pub fn audio_samples(&self, machine: &Machine) -> (Vec<i16>, i32) {
        if !self.attached {
            return (Vec::new(), 0);
        }
        let mut samples = Vec::new();
        if machine.library().claims(caps::AUDIO_PEEK) {
            samples = vec![0i16; machine.audio_buffered().max(0) as usize];
            let n = machine.audio_peek(&mut samples).unwrap_or(0);
            samples.truncate(n);
        }
        (samples, machine.audio_rate())
    }

    // ---- Expressions ------------------------------------------------------------------------------------------------

    /// The context of `processor` (none for the session's own scope) over the machine, `DebugExpressions.For`.
    pub fn evaluate(&mut self, machine: &mut Machine, processor: Option<usize>, expression: &str) -> Result<i64, expr::Error> {
        let mut s = Scoped { d: self, machine, processor };
        s.context(false, |c| Evaluator::new(c).evaluate(expression))
    }

    /// The symbols the context of `processor` names, `eval symbols`.
    pub fn symbol_names(&mut self, machine: &mut Machine, processor: Option<usize>) -> Vec<String> {
        let mut s = Scoped { d: self, machine, processor };
        s.context(false, |c| c.symbol_names())
    }

    /// `BreakpointExpressions.Add`: both expressions checked in the registry's scope, then the breakpoint added.
    pub fn add_breakpoint(&mut self, machine: &mut Machine, processor: Option<usize>, address: i32, end: i32, condition: Option<&str>, log: Option<&str>) -> Result<i32, String> {
        self.prepare(machine, processor, condition, log)?;
        let r = self.registry_of(processor).ok_or("no such processor")?;
        Ok(r.add_breakpoint(address, end, condition, log))
    }

    /// `BreakpointExpressions.AddData`.
    #[allow(clippy::too_many_arguments)]
    pub fn add_data_breakpoint(
        &mut self,
        machine: &mut Machine,
        processor: Option<usize>,
        space: &str,
        address: i32,
        end: i32,
        value: i32,
        on_read: bool,
        changed_only: bool,
        condition: Option<&str>,
        log: Option<&str>,
    ) -> Result<i32, String> {
        self.prepare(machine, processor, condition, log)?;
        let r = self.registry_of(processor).ok_or("no such processor")?;
        Ok(r.add_data_breakpoint(space, address, end, value, on_read, changed_only, condition, log))
    }

    /// The registry a scope names: the session's own for none, else the processor's.
    pub fn registry_of(&mut self, processor: Option<usize>) -> Option<&mut BreakpointRegistry> {
        match processor {
            None => Some(&mut self.breakpoints[0]),
            Some(n) => self.processor_breakpoints(n),
        }
    }

    fn prepare(&mut self, machine: &mut Machine, processor: Option<usize>, condition: Option<&str>, log: Option<&str>) -> Result<(), String> {
        // The main processor's registry is the session's own, which reads the target's context.
        let scope = processor.filter(|&n| n > 0);
        let prepared = {
            let mut s = Scoped { d: self, machine, processor: scope };
            s.context(false, |c| conditions::prepare(c, condition, log))?
        };
        if prepared {
            if let Some(r) = self.registry_of(processor) {
                r.prepared = true;
            }
        }
        Ok(())
    }
}

/// The views a ring holds.
enum Decoded {
    Palettes(Vec<views::Palette>),
    Sprites(Vec<views::Sprite>),
    Channels(Vec<views::AudioChannel>),
}

/// The one listed space a call names, as the C# hands a view that space alone: read through `ReadSpace`, wrapped
/// unless a view.
struct LiveSpace<'v, 'm, 'a> {
    memory: &'v mut ViewMemory<'m, 'a>,
    index: usize,
}

impl emusen_systems::debug::Memory for LiveSpace<'_, '_, '_> {
    fn spaces(&self) -> &[emusen_systems::debug::Space] {
        std::slice::from_ref(&self.memory.spaces_ref()[self.index])
    }

    fn read(&mut self, space: &str, address: i32) -> u8 {
        if self.memory.spaces_ref()[self.index].name == space { self.memory.at.read_listed(self.index, address) } else { 0 }
    }
}

/// The machine read and written by space name with no cheat refresh, `ReadSpace` once the frame's refresh is done.
struct Plain<'a> {
    machine: &'a mut Machine,
    info: &'a Info,
}

impl Plain<'_> {
    fn read(&mut self, name: &str, address: i32) -> u8 {
        let Some(id) = self.info.space_named(name).map(|s| s.id) else { return 0 };
        let mut one = [0u8; 1];
        let _ = self.machine.space_read(id, address as u32, &mut one);
        one[0]
    }

    fn write(&mut self, name: &str, address: i32, value: u8) {
        if let Some(id) = self.info.space_named(name).map(|s| s.id) {
            let _ = self.machine.space_write(id, address as u32, &[value]);
        }
    }
}

/// The debugger and its machine as one scope sees them: a `Source` for the contexts and an `Evaluate` for a registry.
struct Scoped<'d, 'm> {
    d: &'d mut Debugger,
    machine: &'m mut Machine,
    /// None for the session's own scope (the target's context); a processor's index for its own.
    processor: Option<usize>,
}

impl Scoped<'_, '_> {
    /// `DebugExpressions.For`: with no processors the zero-per-core context; else processor 0's with the pack's symbols
    /// for the session's scope, or a processor's own, each then the units'.
    fn context<R>(&mut self, live: bool, f: impl FnOnce(&mut dyn Context) -> R) -> R {
        if self.d.info.processors.is_empty() || !self.d.attached {
            let scope = TargetContext { source: self };
            return if live { f(&mut Live { scope, fresh: false }) } else { f(&mut { scope }) };
        }
        let (index, main) = match self.processor {
            None => (0, true),
            Some(n) => (n.min(self.d.info.processors.len() - 1), false),
        };
        let data = if main { Arc::clone(&self.d.scope_main) } else { self.d.scope_cpus.get(index).cloned().unwrap_or_default() };
        let scope = CpuContext { source: self, set: Set::Processor(index), default_space: data.code.as_deref(), extra: Extra { names: &data.names, main } };
        let scope = UnitContext { scope, units: &data.units };
        if live { f(&mut Live { scope, fresh: false }) } else { f(&mut { scope }) }
    }
}

impl Evaluate for Scoped<'_, '_> {
    fn condition(&mut self, condition: &str) -> Result<bool, String> {
        self.context(true, |c| conditions::condition(c, condition))
    }

    fn render(&mut self, log: &str) -> Result<String, String> {
        self.context(true, |c| conditions::render(c, log))
    }
}

impl Source for Scoped<'_, '_> {
    fn frame(&mut self) -> i64 {
        self.machine.frame_count()
    }

    fn label(&self, name: &str) -> Option<i32> {
        self.d.labels.address_of(name)
    }

    fn label_names(&self) -> Vec<String> {
        self.d.labels.all().map(|(_, n, _)| n.to_string()).collect()
    }

    fn space_count(&self) -> usize {
        if self.d.attached { self.d.info.spaces.len() } else { 0 }
    }

    fn space_name(&self, index: usize) -> &str {
        &self.d.info.spaces[index].name
    }

    fn space_size(&mut self, index: usize) -> i32 {
        self.machine.space_size(self.d.info.spaces[index].id).unwrap_or(0).min(i32::MAX as u64) as i32
    }

    fn space_read(&mut self, index: usize, address: i32) -> u8 {
        let d = &mut *self.d;
        Machined { machine: self.machine, info: &d.info, cheats: &d.cheats, applied: &mut d.applied }.read_listed(index, address)
    }

    fn registers(&mut self, set: Set) -> Vec<Register> {
        self.d.registers(set)
    }

    fn register(&mut self, set: Set, name: &str) -> Option<u64> {
        self.d.register(set, name)
    }

    fn unit_registers(&mut self, unit: u32, into: &mut [i64]) -> usize {
        machined::registers(self.machine, unit, into)
    }

    fn pack_symbol(&mut self, name: &str) -> Option<i64> {
        if !views::has(&self.d.info.system) {
            return None;
        }
        let registers: Vec<(String, u64)> = self.d.registers(Set::Processor(0)).into_iter().map(|r| (r.name, r.value)).collect();
        views::symbol(&self.d.info.system, name, &registers)
    }

    fn stack_depth(&self) -> Option<i64> {
        if self.d.halts() { Some(self.d.call_stack.depth() as i64) } else { None }
    }

    fn refresh_registers(&mut self) {
        self.d.refresh_registers(self.machine);
        self.d.refresh_units(self.machine, Set::Video);
        self.d.refresh_units(self.machine, Set::Apu);
    }
}

/// `PhaseMilliseconds`: each phase the core timed in milliseconds, named true; else the whole frame's one value.
fn phase_milliseconds(machine: &Machine, scratch: &mut [i64; 32], into: &mut [f64], last_frame_ms: f64) -> (usize, bool) {
    let n = if machine.library().claims(caps::PHASES) {
        match machine.phases(&mut []) {
            Ok(n) if n > 0 => {
                if n <= scratch.len() {
                    let _ = machine.phases(&mut scratch[..n]);
                }
                n
            }
            _ => 0,
        }
    } else {
        0
    };
    if n == 0 {
        if let Some(first) = into.first_mut() {
            *first = last_frame_ms;
        }
        return (1, false);
    }
    if n > scratch.len() || n > into.len() {
        return (n, true);
    }
    for i in 0..n {
        into[i] = scratch[i] as f64 / 1e6;
    }
    (n, true)
}
