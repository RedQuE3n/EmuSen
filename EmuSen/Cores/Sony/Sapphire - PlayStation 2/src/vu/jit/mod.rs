//! Micro mode recompiled (Sapphire_Plan.md §7, S40): a unit's microprograms compiled whole on `emusen-jit`, each entered
//! at the address the unit was started at and keyed by it, validated against the micro memory before it runs. A compiled
//! microprogram runs pair by pair exactly as `super::pipe::Pipe` runs them, and stops where the interpreter's loop
//! stops (a pair that would issue past the caller's cycle, the unit stopped, a pair with M under `release`, the bound),
//! so that the unit, its flags, Q and P, its memories, what it hands PATH1 and the cycle each is visible at are the
//! interpreter's at every point another unit can look. The pairs' operations on the format's floats are the
//! interpreter's own functions, called; the pipelines' timing, the hazards, the flow, the integer unit and the write
//! priority are compiled. Sapphire_Native.md §45 is the record.

mod emit;

use std::collections::VecDeque;
use std::sync::Arc;

use emusen_mips::vu::I_BIT;

use super::ops::{self, Ctx, Deferred};
use super::pipe::{self, FlagWrite, NONE, Pipe, State};
use super::{Landing, Link, Path1, Units, Vu};

/// A compiled microprogram: `(frame) -> exit`.
pub type Code = unsafe extern "C" fn(*mut Frame) -> u32;

/// The next pair would issue past the frame's limit, or the debugger's probe held it: nothing of it ran.
pub const STALL: u32 = 0;
/// The unit stopped: the E bit's delay slot, a D or T bit, the last pair having run.
pub const STOP: u32 = 1;
/// The next pair has no code in this microprogram.
pub const OUTSIDE: u32 = 2;
/// Under `release`, a pair with M has run.
pub const RELEASE: u32 = 3;
/// The frame's budget of pairs is spent.
pub const BOUND: u32 = 4;

/// The microprograms kept for one start, the least recently run dropped past them (Sapphire_Speed.md §55).
const VARIANTS: usize = 32;

/// Code past which the module is dropped whole (EmuSen_Jit.md §3.4), in MiB.
const BUDGET_MIB: u64 = 128;

/// How the recompiler runs: after how many entries at an address a microprogram is compiled, whether the dispatcher
/// waits for its code, the verifier, and the two compromises of Sapphire_Plan.md §5.13 the emitter carries, both off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub threshold: u32,
    pub wait: bool,
    pub verify: bool,
    /// C3: the FMAC's arithmetic on the host's floats (never on by default; Sapphire_Native.md §45.9).
    pub c3: bool,
    /// C4: the flags written at the pair's own cycle rather than its S stage (never on by default; §45.9).
    pub c4: bool,
    /// The microprograms kept for one start, and the code budget in MiB.
    pub variants: usize,
    pub budget_mib: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { threshold: 2, wait: false, verify: false, c3: false, c4: false, variants: VARIANTS, budget_mib: BUDGET_MIB }
    }
}

impl Settings {
    /// The settings the environment names for measurement and tests: `SAPPHIRE_VU_THRESHOLD`, `SAPPHIRE_VU_WAIT=1`,
    /// `SAPPHIRE_VU_VERIFY=1`, `SAPPHIRE_VU_C3=1`, `SAPPHIRE_VU_C4=1`, `SAPPHIRE_VU_VARIANTS`, `SAPPHIRE_VU_BUDGET_MIB`.
    pub fn from_environment() -> Settings {
        let on = |k: &str| std::env::var(k).is_ok_and(|v| v == "1");
        let d = Settings::default();
        let number = |k: &str| std::env::var(k).ok().and_then(|v| v.parse::<u64>().ok()).filter(|&n| n > 0);
        Settings {
            variants: number("SAPPHIRE_VU_VARIANTS").map_or(d.variants, |n| n as usize),
            budget_mib: number("SAPPHIRE_VU_BUDGET_MIB").unwrap_or(d.budget_mib),
            threshold: std::env::var("SAPPHIRE_VU_THRESHOLD").ok().and_then(|v| v.parse().ok()).unwrap_or(d.threshold),
            wait: on("SAPPHIRE_VU_WAIT"),
            verify: on("SAPPHIRE_VU_VERIFY"),
            c3: on("SAPPHIRE_VU_C3"),
            c4: on("SAPPHIRE_VU_C4"),
        }
    }
}

/// How a microprogram is compiled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    pub unit: u8,
    pub verify: bool,
    pub c3: bool,
    pub c4: bool,
}

/// The VU's lowering for `emusen-jit`: a microprogram's pairs as one function.
#[derive(Clone, Copy, Debug, Default)]
pub struct VuLowering;

impl emusen_jit::Lowering for VuLowering {
    type Options = Options;
    const NAME: &'static str = "sapphire-vu";
    const EXITS: &'static [&'static str] = &["stall", "stop", "outside", "release", "bound"];

    fn lower(&mut self, module: &cranelift_jit::JITModule, func: &mut cranelift_codegen::ir::Function, functions: &mut cranelift_frontend::FunctionBuilderContext, words: &[u32], start: u32, options: Options) -> Result<(), String> {
        emit::lower(module, func, functions, words, start, options)
    }
}

/// What the recompiler did, for the bench and the tests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    /// Microprograms handed to the compiler, and those it refused.
    pub programs: u64,
    pub refused: u64,
    /// Calls into compiled code, and the pairs they ran.
    pub calls: u64,
    pub compiled_pairs: u64,
    /// Pairs the interpreter ran in the compiled tier: before a program's code arrived, or outside every program.
    pub interpreted_pairs: u64,
    /// Times the words under a microprogram were found changed: the one running, or every one kept for an entry's start
    /// (their code kept, for the words each was compiled from).
    pub invalidated: u64,
    /// The module dropped at its budget.
    pub flushes: u64,
    /// The compiler's thread: bytes of code and nanoseconds compiling.
    pub bytes: u64,
    pub nanos: u64,
    /// Microprograms kept now, and the most kept for one start.
    pub kept: u64,
    pub widest: u64,
}

/// A microprogram: the pairs reachable from its start (`[address, upper, lower]` each), the addresses it covers, its code's
/// slot, and the micro memory's lending count when its words were last found equal to memory.
struct Program {
    words: Box<[u32]>,
    covers: Box<[u64]>,
    slot: Arc<emusen_jit::Slot>,
    checked: u64,
}

impl Program {
    fn covers(&self, pc: u16) -> bool {
        self.covers[pc as usize >> 6] >> (pc & 63) & 1 == 1
    }

    /// Whether every pair it was compiled from is still in memory; the lending count remembered when so.
    fn valid(&mut self, micro: &super::MicroMemory) -> bool {
        if micro.lent() == self.checked {
            return true;
        }
        let same = self.words.as_chunks::<3>().0.iter().all(|p| {
            let at = p[0] as usize * 8;
            let word = |i: usize| u32::from_le_bytes(micro[at + i..at + i + 4].try_into().unwrap());
            word(4) == p[1] && word(0) == p[2]
        });
        if same {
            self.checked = micro.lent();
        }
        same
    }
}

/// The pairs a microprogram started at `start` can reach: the next pair, a branch's target, the pair a BAL or JALR returns
/// to, each followed until an E bit's delay slot; a JR's target is not known and is left to the dispatcher.
fn discover(micro: &[u8], start: u16, mask: u16) -> (Vec<u32>, Box<[u64]>) {
    let pairs = mask as usize + 1;
    let mut covers = vec![0u64; pairs.div_ceil(64)].into_boxed_slice();
    let mut words = Vec::new();
    let mut work = vec![(start, false)];
    let fetch = |pc: u16| {
        let at = pc as usize * 8;
        let word = |i: usize| u32::from_le_bytes(micro[at + i..at + i + 4].try_into().unwrap());
        (word(4), word(0))
    };
    while let Some((pc, delay_of_end)) = work.pop() {
        if covers[pc as usize >> 6] >> (pc & 63) & 1 == 1 {
            continue;
        }
        covers[pc as usize >> 6] |= 1 << (pc & 63);
        let (upper, lower) = fetch(pc);
        words.extend([pc as u32, upper, lower]);
        let next = (pc + 1) & mask;
        if upper & I_BIT == 0
            && let Some((o, e)) = ops::of_lower(lower)
            && e.class == ops::Class::Branch
        {
            let rel = pc.wrapping_add(1).wrapping_add(emusen_mips::vu::imm11(lower) as u16) & mask;
            match o.name {
                "jr" => {}
                "jalr" => work.push(((pc + 2) & mask, false)),
                "bal" => {
                    work.push((rel, false));
                    work.push(((pc + 2) & mask, false));
                }
                _ => work.push((rel, false)),
            }
        }
        if !delay_of_end {
            work.push((next, upper >> 30 & 1 == 1));
        }
    }
    (words, covers)
}

/// The recompiler of one unit's micro mode, as its `Pipe` holds it.
pub struct Tier {
    unit: usize,
    settings: Settings,
    jit: emusen_jit::Jit<VuLowering>,
    /// Microprograms by their start, the address the unit was entered at: the words each was compiled from, up to
    /// `VARIANTS` for one start, so that a game that uploads its microprograms in turn finds their code again.
    programs: Vec<Vec<Program>>,
    /// The start of the microprogram that ran last, whose code a resumed run continues in.
    current: Option<(u16, usize)>,
    /// An address the unit was entered at and not yet counted: a start, a jump outside a program, a state's.
    entry: Option<u16>,
    entries: Vec<u32>,
    counts: Counts,
    reference: Option<Box<Reference>>,
    /// VU1's last run ended in compiled code at a pair that would issue past its limit: that pair, its planned T stage
    /// and the micro memory's lending count then (Sapphire_Native.md §45.3).
    stall: Option<(u16, u64, u64)>,
}

impl Tier {
    pub fn new(unit: usize, settings: Settings) -> Tier {
        let pairs = if unit == 0 { super::VU0_MEM } else { super::VU1_MEM } / 8;
        Tier {
            unit,
            settings,
            jit: emusen_jit::Jit::new(VuLowering, settings.budget_mib << 20),
            programs: (0..pairs).map(|_| Vec::new()).collect(),
            current: None,
            entry: None,
            entries: vec![0; pairs],
            counts: Counts::default(),
            reference: None,
            stall: None,
        }
    }

    /// The tier the environment asks for the unit: `SAPPHIRE_VU1=recompiler` (`SAPPHIRE_VU0` for VU0), else none.
    pub fn from_environment(unit: usize) -> Option<Box<Tier>> {
        let key = if unit == 0 { "SAPPHIRE_VU0" } else { "SAPPHIRE_VU1" };
        std::env::var(key).is_ok_and(|v| v == "recompiler").then(|| Box::new(Tier::new(unit, Settings::from_environment())))
    }

    pub fn settings(&self) -> Settings {
        self.settings
    }

    pub fn counts(&self) -> Counts {
        let mut c = self.counts;
        c.flushes = self.jit.flushes;
        c.kept = self.programs.iter().map(|v| v.len() as u64).sum();
        c.widest = self.programs.iter().map(|v| v.len() as u64).max().unwrap_or(0);
        if let Some(k) = self.jit.counters() {
            use std::sync::atomic::Ordering::Relaxed;
            c.bytes = k.bytes.load(Relaxed);
            c.nanos = k.nanos.load(Relaxed);
            c.refused = k.failed.load(Relaxed);
        }
        c
    }

    /// The unit was started at `pc`.
    pub(super) fn entered(&mut self, pc: u16) {
        self.entry = Some(pc);
        self.current = None;
        self.stall = None;
    }

    /// A state was loaded: the program that ran last is forgotten, and a running unit's address counts as an entry.
    pub(super) fn loaded(&mut self, pc: u16, running: bool) {
        self.current = None;
        self.stall = None;
        self.entry = running.then_some(pc);
    }

    fn forget_all(&mut self) {
        self.programs.iter_mut().for_each(Vec::clear);
        self.current = None;
    }

    /// The code for the pair at `pc`, if a microprogram covering it is compiled and still in memory; an entry counted,
    /// and a microprogram handed to the compiler, on the way.
    fn code_for(&mut self, pc: u16, micro: &super::MicroMemory, mask: u16) -> Option<Code> {
        if let Some((start, k)) = self.current
            && let Some(p) = self.programs[start as usize].get_mut(k)
            && p.covers(pc)
        {
            if p.valid(micro) {
                return published(&p.slot);
            }
            self.counts.invalidated += 1;
            self.current = None;
            self.entry = Some(pc);
        }
        let variants = &mut self.programs[pc as usize];
        if let Some(k) = variants.iter_mut().rposition(|p| p.valid(micro)) {
            let last = variants.len() - 1;
            variants[k..].rotate_left(1);
            self.current = Some((pc, last));
            return published(&variants[last].slot);
        }
        if !self.programs[pc as usize].is_empty() && self.entry == Some(pc) {
            self.counts.invalidated += 1;
        }
        if self.entry.take() != Some(pc) {
            return None;
        }
        self.entries[pc as usize] += 1;
        if self.entries[pc as usize] < self.settings.threshold {
            return None;
        }
        self.entries[pc as usize] = 0;
        // The budget is checked where code is added, not on every run (Sapphire_Speed.md §55.3).
        if self.jit.over_budget() {
            self.forget_all();
        }
        let (words, covers) = discover(micro, pc, mask);
        let options = Options { unit: self.unit as u8, verify: self.settings.verify, c3: self.settings.c3, c4: self.settings.c4 };
        let slot = self.jit.enqueue(words.clone().into_boxed_slice(), pc as u32, options);
        self.counts.programs += 1;
        if self.settings.wait {
            slot.wait();
        }
        let variants = &mut self.programs[pc as usize];
        if variants.len() >= self.settings.variants {
            variants.remove(0);
        }
        variants.push(Program { words: words.into_boxed_slice(), covers, slot, checked: micro.lent() });
        let k = variants.len() - 1;
        self.current = Some((pc, k));
        published(&variants[k].slot)
    }


    /// `Pipe::run_pairs` on the recompiler: compiled code where a microprogram covers the pair, the interpreter's step
    /// elsewhere, both stopping where the interpreter's loop stops.
    fn run(&mut self, pipe: &mut Pipe, units: &mut Units, limit: u64, release: bool, bound: u64) -> u64 {
        let mask = pipe.pairs(units) - 1;
        if pipe.state != State::Run || pipe.next_t.max(pipe.kick_wait) > limit {
            return 0;
        }
        let lent = if self.unit == 0 { units.vu0.micro.lent() } else { units.vu1.micro.lent() };
        if let Some((pc, planned, at)) = self.stall
            && (pc, at) == (pipe.pc, lent)
            && planned > limit
        {
            return 0;
        }
        self.stall = None;
        let mut left = bound;
        loop {
            if pipe.state != State::Run || (release && pipe.release_t != NONE) || left == 0 {
                return bound - left;
            }
            let micro = if self.unit == 0 { &units.vu0.micro } else { &units.vu1.micro };
            match self.code_for(pipe.pc, micro, mask) {
                Some(code) => {
                    if self.settings.verify {
                        self.reference.get_or_insert_with(|| Box::new(Reference::new(self.unit))).sync(pipe, units);
                    }
                    let reference = self.reference.as_deref_mut().map_or(std::ptr::null_mut(), |r| r as *mut Reference);
                    let mut frame = Frame::new(pipe, units, limit, left, release, reference);
                    // SAFETY: the code was compiled for this unit's frame; the frame's pointers are to `pipe` and
                    // `units`, which nothing else touches until the call returns.
                    let exit = unsafe { code(&mut frame) };
                    self.counts.calls += 1;
                    if let Some(text) = self.reference.as_mut().and_then(|r| r.failure.take()) {
                        panic!("{text}");
                    }
                    self.counts.compiled_pairs += left - frame.budget;
                    left = frame.budget;
                    if exit == STALL && frame.planned != 0 && self.unit == 1 {
                        self.stall = Some((pipe.pc, frame.planned, lent));
                    }
                    if exit != OUTSIDE {
                        return bound - left;
                    }
                    self.entry = Some(pipe.pc);
                }
                None => {
                    if !pipe.step(units, limit) {
                        return bound - left;
                    }
                    left -= 1;
                    self.counts.interpreted_pairs += 1;
                }
            }
        }
    }
}

fn published(slot: &emusen_jit::Slot) -> Option<Code> {
    // SAFETY: a published address is a finalized function of this signature, published with Release after its pages
    // became executable.
    slot.published().map(|code| unsafe { std::mem::transmute::<*const u8, Code>(code) })
}

/// `Pipe::run_pairs` for a unit on the recompiler.
pub(super) fn run(pipe: &mut Pipe, units: &mut Units, limit: u64, release: bool, bound: u64) -> u64 {
    let mut tier = pipe.jit.take().expect("the recompiler's tier");
    let n = tier.run(pipe, units, limit, release, bound);
    pipe.jit = Some(tier);
    n
}

/// What compiled code reads: the unit's state and its memories, the run's limits, and the scratch the helpers it calls
/// leave their results in.
#[repr(C)]
pub struct Frame {
    pub pipe: *mut Pipe,
    pub vu: *mut Vu,
    /// VU1, which VU0's operations reach through the window; null for VU1.
    pub window: *mut Vu,
    pub link: *mut Link,
    /// The unit's data memory.
    pub data: *mut u8,
    pub limit: u64,
    /// Pairs the run may still run.
    pub budget: u64,
    /// The run stops after a pair with M (`Micro::release`).
    pub release: u32,
    /// `TRACING`, `PROBING` and `HALTS`.
    pub flags: u32,
    pub reference: *mut Reference,
    /// The planned T stage of the last pair the code looked at, 0 when the probe held it.
    pub planned: u64,
    low: Deferred,
    up: Deferred,
}

pub const TRACING: u32 = 1;
pub const PROBING: u32 = 2;
pub const HALTS: u32 = 4;

impl Frame {
    fn new(pipe: &mut Pipe, units: &mut Units, limit: u64, budget: u64, release: bool, reference: *mut Reference) -> Frame {
        let flags = (u32::from(pipe.trace.is_some()) * TRACING) | (u32::from(pipe.probe.is_some()) * PROBING) | (u32::from(units.halts) * HALTS);
        let (vu, window) = if pipe.unit == 0 { (&mut units.vu0 as *mut Vu, &mut units.vu1 as *mut Vu) } else { (&mut units.vu1 as *mut Vu, std::ptr::null_mut()) };
        // SAFETY: `vu` was made from a live reference just above.
        let data = unsafe { (*vu).data.as_mut_ptr() };
        Frame {
            pipe,
            vu,
            window,
            link: &mut units.link,
            data,
            limit,
            budget,
            release: u32::from(release),
            flags,
            reference,
            planned: 0,
            low: Deferred::default(),
            up: Deferred::default(),
        }
    }
}

/// The earliest cycle a flag write, a quotient or an EFU result lands at, `NONE` with none in flight.
fn pending(pipe: &Pipe, vu: &Vu) -> u64 {
    let f = pipe.flags.front().map_or(NONE, |f| f.at);
    let q = vu.q_next.map_or(NONE, |l| l.at);
    let p = vu.p_next.map_or(NONE, |l| l.at);
    f.min(q).min(p)
}

// The helpers compiled code calls. Each borrows from the frame only what it touches, for the length of the call.

pub(super) extern "C" fn h_pending(f: *mut Frame) -> u64 {
    // SAFETY: compiled code passes the frame it was given, whose pointers are live for the run.
    unsafe { pending(&*(*f).pipe, &*(*f).vu) }
}

/// `execute`'s first step: the quotient and the EFU result due by `t`, then the flags (`Vu::settle`, `pipe::drain`).
pub(super) extern "C" fn h_land(f: *mut Frame, t: u64) -> u64 {
    // SAFETY: as `h_pending`.
    let (pipe, vu) = unsafe { (&mut *(*f).pipe, &mut *(*f).vu) };
    vu.settle(t);
    pipe::drain(&mut pipe.flags, vu, t);
    pending(pipe, vu)
}

/// The cycle WAITQ waits for (`Pipe::plan`).
pub(super) extern "C" fn h_q_at(f: *mut Frame) -> u64 {
    // SAFETY: as `h_pending`.
    unsafe { (*(*f).vu).q_next.map_or(0, |l| l.at) }
}

/// The cycle WAITP waits for (`Pipe::plan`).
pub(super) extern "C" fn h_p_at(f: *mut Frame) -> u64 {
    // SAFETY: as `h_pending`.
    unsafe { (*(*f).vu).p_last().map_or(0, |l| l.at - 1) }
}

pub const KIND_PLAIN: u32 = 0;
pub const KIND_DIV: u32 = 1 << 8;
pub const KIND_EFU: u32 = 2 << 8;

fn context<'a>(f: *mut Frame) -> Ctx<'a> {
    // SAFETY: as `h_pending`; VU0's window is VU1, a different unit from the one `vu` names.
    unsafe { Ctx::micro(&mut *(*f).vu, (*f).window.as_mut()) }
}

/// A lower operation run as `execute` runs it, its flags handed back; a divide's quotient and an EFU result sent on
/// their way at `t` with the divider or the EFU taken (`kind` with the latency in its low byte).
pub(super) extern "C" fn h_lower(f: *mut Frame, run: usize, word: u32, t: u64, kind: u32) -> u64 {
    // SAFETY: `run` is an operation's function from `ops::TABLE`, its address taken by the emitter.
    let run: fn(&mut Ctx, u32) = unsafe { std::mem::transmute::<usize, fn(&mut Ctx, u32)>(run) };
    let mut c = context(f);
    run(&mut c, word);
    let (out, quotient) = (c.out, c.q_out);
    // SAFETY: as `h_pending`.
    let (frame, vu) = unsafe { (&mut *f, &mut *(*f).vu) };
    frame.low = out;
    let latency = u64::from(kind & 0xFF);
    match kind & !0xFF {
        KIND_DIV => {
            if let Some(q) = quotient {
                let at = t + latency;
                vu.q_next = Some(Landing { at, value: q, flags: out.divide });
                vu.div_free = at;
            }
        }
        KIND_EFU => {
            if let Some(p) = out.p {
                let at = t + latency;
                vu.send_p(Landing { at, value: p, flags: None });
                vu.efu_free = at - 1;
            }
        }
        _ => {}
    }
    // SAFETY: as `h_pending`.
    unsafe { pending(&*frame.pipe, vu) }
}

/// An upper operation run as `execute` runs it, then the pair's flag write put together as `h_push` does.
pub(super) extern "C" fn h_upper(f: *mut Frame, run: usize, word: u32, t: u64, which: u32) -> u64 {
    // SAFETY: as `h_lower`.
    let run: fn(&mut Ctx, u32) = unsafe { std::mem::transmute::<usize, fn(&mut Ctx, u32)>(run) };
    let mut c = context(f);
    run(&mut c, word);
    let out = c.out;
    // SAFETY: as `h_pending`.
    unsafe { (*f).up = out };
    h_push(f, t, which)
}

pub const USE_UP: u32 = 1;
pub const USE_LOW: u32 = 2;
/// C4's flag write, made at once rather than queued for the S stage.
pub const NOW: u32 = 4;


/// The pair's flag write for its S stage, as `execute` puts it together: the upper operation's MAC, status and clipping
/// judgement, the lower's FCSET and FSSET.
pub(super) extern "C" fn h_push(f: *mut Frame, t: u64, which: u32) -> u64 {
    // SAFETY: as `h_pending`.
    let (frame, pipe, vu) = unsafe { (&mut *f, &mut *(*f).pipe, &mut *(*f).vu) };
    let up = if which & USE_UP != 0 { frame.up } else { Deferred::default() };
    let low = if which & USE_LOW != 0 { frame.low } else { Deferred::default() };
    let d = Deferred { mac: up.mac, zsuo: up.zsuo, sticky: up.sticky, clip_shift: up.clip_shift, clip_set: low.clip_set, sticky_set: low.sticky_set, ..Deferred::default() };
    if d.mac.is_some() || d.clip_shift.is_some() || d.clip_set.is_some() || d.sticky_set.is_some() {
        if which & NOW != 0 {
            // C4: written at the pair's own cycle (Sapphire_Native.md §45.9); never on by default.
            pipe::apply(vu, d);
        } else {
            pipe.flags.push_back(FlagWrite { at: t + pipe::FMAC, d });
        }
    }
    pending(pipe, vu)
}

pub const EXT_XGKICK: u32 = 0;
pub const EXT_XTOP: u32 = 1;
pub const EXT_XITOP: u32 = 2;

/// XGKICK, XTOP and XITOP as `execute` carries them out.
pub(super) extern "C" fn h_external(f: *mut Frame, word: u32, t: u64, kind: u32) {
    // SAFETY: as `h_pending`.
    let (frame, pipe, vu, link) = unsafe { (&mut *f, &mut *(*f).pipe, &mut *(*f).vu, &mut *(*f).link) };
    let unit = pipe.unit;
    match kind {
        EXT_XGKICK => {
            let addr = vu.vi((word >> 11 & 31) as usize) & 0x3FF;
            let start = t.max(pipe.path1_free);
            if pipe.path1_free > t {
                pipe.kick_wait = pipe.path1_free;
            }
            pipe.path1_free = link.path1.kick(&vu.data, addr, start);
            if let Some(r) = unsafe { frame.reference.as_mut() } {
                r.kicks.borrow_mut().push_back((addr, start, pipe.path1_free));
            }
        }
        EXT_XTOP => pipe.write_alu(vu, (word >> 16 & 31) as usize, link.top[1], t),
        _ => pipe.write_alu(vu, (word >> 16 & 31) as usize, link.itop[unit], t),
    }
}

pub(super) extern "C" fn h_trace(f: *mut Frame, pc: u32, t: u64) {
    // SAFETY: as `h_pending`.
    let pipe = unsafe { &mut *(*f).pipe };
    if let Some(tr) = &mut pipe.trace {
        tr.push((pc as u16, t));
    }
}

/// The debugger's probe asked before the pair runs (`Pipe::step`); 0 when it holds the unit there.
pub(super) extern "C" fn h_admit(f: *mut Frame, pc: u32) -> u32 {
    // SAFETY: as `h_pending`.
    let (frame, pipe) = unsafe { (&*f, &mut *(*f).pipe) };
    pipe.probe.as_mut().map_or(1, |p| u32::from(p.admit(pc * 8, frame.flags & HALTS != 0)))
}

/// The verifier: the interpreter steps the same pair on its own copy of the units, and the two are compared.
pub(super) extern "C" fn h_verify(f: *mut Frame) {
    // SAFETY: as `h_pending`; the reference is the tier's, apart from the units.
    let (frame, pipe) = unsafe { (&*f, &*(*f).pipe) };
    // SAFETY: as above; the units are borrowed only to be read.
    let units_vu = unsafe { &*frame.vu };
    let window = unsafe { frame.window.as_ref() };
    let link = unsafe { &*frame.link };
    let r = unsafe { frame.reference.as_mut() }.expect("the verifier's reference");
    if r.failure.is_some() {
        return;
    }
    // A panic cannot cross the C ABI: the first difference is kept, and the dispatcher raises it when the code returns.
    let found = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| r.follow(pipe, units_vu, window, link, frame.limit)));
    if let Err(e) = found {
        let text = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
        r.failure = Some(text);
    }
}

/// PATH1 for the verifier's interpreter: each kick the compiled code made, handed back in order.
struct Replay {
    kicks: std::rc::Rc<std::cell::RefCell<VecDeque<(u16, u64, u64)>>>,
}

impl Path1 for Replay {
    fn kick(&mut self, _: &[u8], addr: u16, at: u64) -> u64 {
        let (a, start, end) = self.kicks.borrow_mut().pop_front().expect("the interpreter kicked PATH1 where the compiled code did not");
        assert_eq!((a, start), (addr, at), "the interpreter kicked PATH1 at another address or cycle than the compiled code");
        end
    }
}

/// The verifier's interpreter: a copy of the unit's micro mode and of both units, stepped once for every pair the compiled
/// code runs and compared after each (EmuSen_Jit.md §4).
pub struct Reference {
    pipe: Pipe,
    units: Units,
    kicks: std::rc::Rc<std::cell::RefCell<VecDeque<(u16, u64, u64)>>>,
    pub steps: u64,
    failure: Option<String>,
}

impl Reference {
    fn new(unit: usize) -> Reference {
        let kicks = std::rc::Rc::new(std::cell::RefCell::new(VecDeque::new()));
        let link = Link { path1: Box::new(Replay { kicks: kicks.clone() }), ..Link::default() };
        Reference {
            pipe: Pipe::new_interpreted(unit),
            units: Units { vu0: Vu::new(super::VU0_MEM), vu1: Vu::new(super::VU1_MEM), link, halts: false },
            kicks,
            steps: 0,
            failure: None,
        }
    }

    /// The copy made level with the units the compiled code is about to run.
    fn sync(&mut self, pipe: &Pipe, units: &Units) {
        self.pipe.copy_from(pipe);
        self.units.vu0 = units.vu0.clone();
        self.units.vu1 = units.vu1.clone();
        let (l, s) = (&mut self.units.link, &units.link);
        (l.fbrst, l.irq, l.top, l.itop) = (s.fbrst, s.irq, s.top, s.itop);
        self.units.halts = units.halts;
        self.kicks.borrow_mut().clear();
    }

    fn follow(&mut self, pipe: &Pipe, vu: &Vu, window: Option<&Vu>, link: &Link, limit: u64) {
        self.steps += 1;
        assert!(self.pipe.step(&mut self.units, limit), "the interpreter did not run the pair the compiled code ran, after {} pairs", self.steps);
        let unit = pipe.unit;
        let (rv, rw) = if unit == 0 { (&self.units.vu0, Some(&self.units.vu1)) } else { (&self.units.vu1, None) };
        let difference = self.pipe.differs(pipe).or_else(|| vu_differs(rv, vu)).or_else(|| match (rw, window) {
            (Some(a), Some(b)) => vu_differs(a, b).map(|d| format!("in VU1 through the window: {d}")),
            _ => None,
        });
        let difference = difference.or_else(|| {
            let l = &self.units.link;
            ((l.irq, l.fbrst) != (link.irq, link.fbrst)).then(|| format!("the link's interrupts {:x} against {:x}", l.irq, link.irq))
        });
        let difference = difference.or_else(|| (!self.kicks.borrow().is_empty()).then(|| "a PATH1 kick the interpreter did not make".to_string()));
        if let Some(d) = difference {
            // A player's microprogram is never described: the count alone, unless the environment asks for the pair.
            if std::env::var("SAPPHIRE_VU_VERIFY_DETAIL").is_ok_and(|v| v == "1") {
                panic!("the recompiled VU{unit} parted from the interpreter after {} pairs, at pair {:#x}: {d}", self.steps, pipe.pc);
            }
            panic!("the recompiled VU{unit} parted from the interpreter after {} pairs", self.steps);
        }
    }
}

/// The first difference between two units, every register, flag, landing and memory compared.
fn vu_differs(a: &Vu, b: &Vu) -> Option<String> {
    for n in 0..32 {
        if a.vf(n) != b.vf(n) {
            return Some(format!("VF{n:02} {:08x?} against {:08x?}", a.vf(n), b.vf(n)));
        }
    }
    for n in 0..16 {
        if a.vi(n) != b.vi(n) {
            return Some(format!("VI{n:02} {:x} against {:x}", a.vi(n), b.vi(n)));
        }
    }
    let regs = |v: &Vu| (v.acc, v.i, v.q, v.p, v.r, v.status, v.mac, v.clip, v.tpc);
    if regs(a) != regs(b) {
        return Some(format!("ACC, I, Q, P, R, status, MAC, clipping, TPC {:x?} against {:x?}", regs(a), regs(b)));
    }
    let flight = |v: &Vu| (v.q_next, v.div_free, v.p_next, v.p_later, v.efu_free);
    if flight(a) != flight(b) {
        return Some(format!("the results in flight {:?} against {:?}", flight(a), flight(b)));
    }
    if a.data != b.data {
        let at = a.data.iter().zip(b.data.iter()).position(|(x, y)| x != y).unwrap_or(0);
        return Some(format!("the data memory at byte {at:#x}"));
    }
    if *a.micro != *b.micro {
        return Some("the micro memory".to_string());
    }
    None
}
