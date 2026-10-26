//! The EE's recompiled tier as the machine runs it (Sapphire_Plan.md §7, S36): the blocks by virtual address, each
//! checked against the instruction cache on entry, handed to `emusen-jit`'s compiler at its `threshold`-th entry, and run
//! between the interpreter's steps. A compiled block leaves the board behind it; its exit brings the board through each
//! of its instructions' cycles in turn, as `System::step` would have, and keeps the instructions before the first
//! interrupt the interpreter would take. Sapphire_Native.md §44 is the record.

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::Arc;

use crate::board::System;
use crate::ee::Halt;
use crate::ee::jit::{self, Code, Context, FINAL, Jit, Options, Slot, shape};
use emusen_jit::verify::Reference;

/// Bytes of code past which every block is dropped and compiled again as it runs (`EmuSen_Jit.md` §3.4).
pub const BUDGET: u64 = 256 << 20;

/// A block is handed to the compiler at this entry (MarsRT's sixty-fourth, `Mars_Native.md` §5.8.4).
pub const THRESHOLD: u32 = 64;

/// How the interpreter is stepped beside the blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verify {
    /// After each block's exit, the interpreter stepped as many times and the two compared.
    Blocks,
    /// After every compiled instruction, from the code itself, everything written back first.
    Steps,
}

enum State {
    Counting(u32),
    Queued(Arc<Slot>),
    Ready(Code),
    /// Not compiled: the first instruction is not one a block holds, or the compiler refused it.
    Refused,
}

struct Block {
    paddr: u32,
    words: Box<[u32]>,
    options: Options,
    state: State,
}

/// What the dispatcher counted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    /// Compiled blocks entered, and the instructions they ran.
    pub entered: u64,
    pub compiled_steps: u64,
    /// Exits whose replay found an interrupt first, and the instructions the interpreter then ran again.
    pub rollbacks: u64,
    pub rerun_steps: u64,
    /// Steps the interpreter took.
    pub interpreted: u64,
    /// Blocks shaped again because the cache no longer held their words or their page moved.
    pub reshaped: u64,
}

/// A hash of a word address, for the map of blocks.
#[derive(Default)]
pub struct Mix(u64);

impl Hasher for Mix {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0 ^ u64::from(b)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        }
    }
    fn write_u32(&mut self, v: u32) {
        self.0 = u64::from(v).wrapping_mul(0x9E37_79B9_7F4A_7C15).rotate_right(29);
    }
}

pub struct Blocks {
    map: HashMap<u32, Block, BuildHasherDefault<Mix>>,
    jit: Jit,
    pub threshold: u32,
    /// Wait for each block's code when it is handed over, as the tests do.
    pub wait: bool,
    /// Compromise C1 in the code compiled from now on (off by default; `effect: accuracy`).
    pub fixed_cost: bool,
    verify: Option<(Verify, emusen_jit::verify::Verifier<Shadow>)>,
    pub stats: Stats,
    ctx: Box<Context>,
}

impl Default for Blocks {
    fn default() -> Self {
        Blocks::new()
    }
}

impl Blocks {
    pub fn new() -> Blocks {
        let ctx = Box::new(Context { times: [0; shape::MAX], lines: [0; shape::MAX], steps: 0, replay, hook, owner: std::ptr::null_mut() });
        let mut jit = Jit::new(jit::R5900, BUDGET);
        jit.perf_map = std::env::var("EMUSEN_SAPPHIRE_PERFMAP").is_ok_and(|v| v == "1");
        Blocks { map: HashMap::default(), jit, threshold: THRESHOLD, wait: false, fixed_cost: false, verify: None, stats: Stats::default(), ctx }
    }

    /// The settings the environment gives, for measurement: `EMUSEN_SAPPHIRE_THRESHOLD`, `EMUSEN_SAPPHIRE_C1=1`.
    pub fn from_env() -> Blocks {
        let mut b = Blocks::new();
        if let Some(t) = std::env::var("EMUSEN_SAPPHIRE_THRESHOLD").ok().and_then(|v| v.parse().ok()) {
            b.threshold = t;
        }
        b.fixed_cost = std::env::var("EMUSEN_SAPPHIRE_C1").is_ok_and(|v| v == "1");
        b
    }

    /// The interpreter's machine stepped beside the blocks from now on, `reference` equal to the machine they run;
    /// `quiet` names a difference without its values (a run of the player's BIOS, Sapphire_Plan.md §1.4).
    pub fn verify(&mut self, how: Verify, reference: Box<System>, quiet: bool) {
        self.verify = Some((how, emusen_jit::verify::Verifier::new(Shadow { sys: reference, quiet })));
        self.forget();
    }

    /// Steps the reference took, when a verifier is attached.
    pub fn verified(&self) -> Option<u64> {
        self.verify.as_ref().map(|(_, v)| v.steps)
    }

    /// The whole of both machines compared, at a run's end.
    pub fn check_all(&self, sys: &System) {
        if let Some((_, v)) = &self.verify {
            v.check_all(sys);
        }
    }

    /// Every block forgotten, its code kept until the budget drops the module.
    pub fn forget(&mut self) {
        self.map.clear();
    }

    /// The compiler's counters: blocks compiled, refused, and bytes of code.
    pub fn compiled(&self) -> (u64, u64, u64) {
        use std::sync::atomic::Ordering::Relaxed;
        self.jit.counters().map_or((0, 0, 0), |c| (c.compiled.load(Relaxed), c.failed.load(Relaxed), c.bytes.load(Relaxed)))
    }

    /// Waits until every block handed over is compiled or refused.
    pub fn drain(&self) {
        self.jit.drain();
    }

    /// The code budget, and the times it dropped every block.
    pub fn set_budget(&mut self, bytes: u64) {
        self.jit.budget = bytes;
    }
    pub fn flushes(&self) -> u64 {
        self.jit.flushes
    }

    /// The mnemonics in the blocks that hold code now.
    pub fn compiled_mnemonics(&self) -> std::collections::BTreeSet<&'static str> {
        self.map.values().filter(|b| matches!(b.state, State::Ready(_))).flat_map(|b| b.words.iter().map(|&w| shape::name(w))).collect()
    }

    fn options(&self, sys: &System) -> Options {
        Options { single: sys.ee.single_issue(), verify: matches!(self.verify, Some((Verify::Steps, _))), fixed_cost: self.fixed_cost }
    }
}

/// One step of the machine, or a compiled block of at most `left` steps; the steps taken.
pub fn run(sys: &mut System, blocks: &mut Blocks, left: u64) -> Result<u64, Halt> {
    if let Some(n) = block(sys, blocks, left) {
        return Ok(n);
    }
    interpret(sys, blocks)
}

/// One interpreter step, followed by the verifier's.
pub fn interpret(sys: &mut System, blocks: &mut Blocks) -> Result<u64, Halt> {
    let r = sys.step();
    blocks.stats.interpreted += 1;
    if let Some((_, v)) = &mut blocks.verify {
        v.follow(sys, "an interpreter step");
    }
    r.map(|()| 1)
}

/// What the helpers reach through the context's owner.
struct Run {
    sys: *mut System,
    verify: *mut Option<(Verify, emusen_jit::verify::Verifier<Shadow>)>,
    /// The verifier's finding inside a helper, raised once the code has returned (a panic cannot cross compiled code).
    failed: Option<String>,
}

fn block(sys: &mut System, blocks: &mut Blocks, left: u64) -> Option<u64> {
    if blocks.jit.over_budget() {
        blocks.forget();
    }
    let paddr = sys.ee.block_entry()?;
    let pc = sys.ee.pc;
    let options = blocks.options(sys);
    let valid = blocks.map.get(&pc).is_some_and(|b| b.paddr == paddr && b.options == options && sys.ee.holds(pc, paddr, &b.words));
    if !valid {
        let words = sys.ee.cached_words(pc, paddr, shape::MAX);
        let shaped = shape::shape(&words);
        let state = if shaped.is_some() { State::Counting(0) } else { State::Refused };
        let words = shaped.unwrap_or_else(|| words.get(..1).map_or_else(Vec::new, <[u32]>::to_vec));
        if words.is_empty() {
            return None;
        }
        if blocks.map.insert(pc, Block { paddr, words: words.into_boxed_slice(), options, state }).is_some() {
            blocks.stats.reshaped += 1;
        }
    }
    let threshold = blocks.threshold;
    let wait = blocks.wait;
    let b = blocks.map.get_mut(&pc).expect("the block is in the map");
    let code = match &b.state {
        State::Ready(code) => *code,
        State::Refused => return None,
        State::Counting(n) => {
            let n = n + 1;
            if n < threshold {
                b.state = State::Counting(n);
                return None;
            }
            let slot = blocks.jit.enqueue(b.words.clone(), pc, b.options);
            if wait {
                slot.wait();
            }
            b.state = settle(slot)?;
            match b.state {
                State::Ready(code) => code,
                _ => return None,
            }
        }
        State::Queued(slot) => {
            b.state = settle(slot.clone())?;
            match b.state {
                State::Ready(code) => code,
                _ => return None,
            }
        }
    };
    if b.words.len() as u64 > left {
        return None;
    }
    let mut run = Run { sys, verify: &mut blocks.verify, failed: None };
    let ctx = &mut *blocks.ctx;
    ctx.owner = (&mut run as *mut Run).cast();
    // SAFETY: the code was compiled for this signature; the core and the context outlive the call, and nothing else
    // borrows the machine while the code and its helpers run through these pointers.
    let exit = unsafe { code(&raw mut (*run.sys).ee, ctx) };
    if let Some(f) = run.failed.take() {
        panic!("{f}");
    }
    let steps = ctx.steps;
    match exit {
        jit::DONE => {
            if steps == 0 {
                return None;
            }
            blocks.stats.entered += 1;
            blocks.stats.compiled_steps += u64::from(steps);
            if let Some((Verify::Blocks, v)) = &mut blocks.verify {
                for _ in 0..steps {
                    v.reference.step();
                    v.steps += 1;
                }
                v.check(sys, "a compiled block");
            }
            Some(u64::from(steps))
        }
        _ => {
            // The replay brought the board through `steps` instructions' cycles and found an interrupt due after them:
            // the interpreter runs those instructions on the untouched core, with the lines each step would have seen.
            blocks.stats.rollbacks += 1;
            blocks.stats.rerun_steps += u64::from(steps);
            for j in 0..steps as usize {
                if j > 0 {
                    set_lines(sys, ctx.lines[j - 1]);
                }
                sys.ee.step(&mut sys.board).expect("a block's instruction is the interpreter's");
                if let Some((_, v)) = &mut blocks.verify {
                    v.reference.step();
                    v.steps += 1;
                }
            }
            set_lines(sys, ctx.lines[steps as usize - 1]);
            if let Some((_, v)) = &blocks.verify {
                v.check(sys, "a block's instructions run again before an interrupt");
            }
            Some(u64::from(steps))
        }
    }
}

fn settle(slot: Arc<Slot>) -> Option<State> {
    if let Some(code) = jit::published(&slot) {
        Some(State::Ready(code))
    } else if slot.settled() {
        Some(State::Refused)
    } else {
        Some(State::Queued(slot))
    }
}

fn pack((i0, i1, c0): (bool, bool, bool)) -> u8 {
    u8::from(i0) | u8::from(i1) << 1 | u8::from(c0) << 2
}

fn set_lines(sys: &mut System, l: u8) {
    sys.ee.int0 = l & 1 != 0;
    sys.ee.int1 = l & 2 != 0;
    sys.ee.cpcond0 = l & 4 != 0;
}

/// The replay: the board brought to each instruction's cycles in turn, as `System::step` brings it after each, until the
/// lines or Count would have the interpreter take an interrupt before the next instruction. The steps that stand; when
/// all do, the core's lines and Cause.IP2 and IP3 are set as the last step leaves them.
extern "C" fn replay(ctx: *mut Context, steps: u32) -> u32 {
    use crate::ee::cop0::{CAUSE, COMPARE, COUNT, STATUS, cause};
    // SAFETY: called by compiled code during `block`, whose `Run` and context are alive and borrowed by nothing else.
    let ctx = unsafe { &mut *ctx };
    let run = unsafe { &mut *ctx.owner.cast::<Run>() };
    let sys = unsafe { &mut *run.sys };
    let (count0, compare, seen0) = (sys.ee.cop0[COUNT], sys.ee.cop0[COMPARE], sys.ee.cycles_seen());
    let (status, cause_now) = (sys.ee.cop0[STATUS], sys.ee.cop0[CAUSE]);
    let ip7 = cause_now & cause::IP7 != 0;
    let m = steps as usize;
    let System { ee, board } = sys;
    let times = &ctx.times[..m];
    let lines_out = &mut ctx.lines;
    let ran = board.run_through(times, &mut ee.spram[..], &mut |j, lines| {
        lines_out[j] = pack(lines);
        let ip7 = ip7 || u64::from(compare.wrapping_sub(count0).wrapping_sub(1)) < times[j].wrapping_sub(seen0);
        !(j + 1 < m && crate::ee::Ee::interrupt_due(status, cause_now, lines.0, lines.1, ip7))
    });
    if ran < m {
        ctx.steps = ran as u32;
        return ctx.steps;
    }
    set_lines(sys, ctx.lines[m - 1]);
    if m >= 2 {
        let l = ctx.lines[m - 2];
        let c = &mut sys.ee.cop0[CAUSE];
        *c = *c & !(cause::IP2 | cause::IP3) | if l & 1 != 0 { cause::IP2 } else { 0 } | if l & 2 != 0 { cause::IP3 } else { 0 };
    }
    steps
}

/// The verifier's call after each compiled instruction, everything written back: the board brought to its cycles, the
/// interpreter stepped beside it and the two compared; then, unless it was the block's last, the next step's interrupt
/// check made, its answer whether the block leaves here.
extern "C" fn hook(ctx: *mut Context, steps: u32) -> u32 {
    // SAFETY: as `replay`.
    let ctx = unsafe { &mut *ctx };
    let run = unsafe { &mut *ctx.owner.cast::<Run>() };
    let sys = unsafe { &mut *run.sys };
    let last = steps & FINAL != 0;
    let j = (steps & !FINAL) as usize;
    let t = sys.ee.timing.cycles();
    assert_eq!(t, ctx.times[j - 1], "the cycles written back are the cycles the code stored");
    let System { ee, board } = sys;
    board.run_to(t, &mut ee.spram[..]);
    let lines = board.lines();
    ctx.lines[j - 1] = pack(lines);
    set_lines(sys, ctx.lines[j - 1]);
    // SAFETY: as above.
    if let Some((_, v)) = unsafe { &mut *run.verify } {
        let sys = &*sys;
        if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| v.follow(sys, "a compiled instruction"))) {
            run.failed = Some(e.downcast_ref::<String>().cloned().unwrap_or_else(|| "the verifier failed".into()));
            return 1;
        }
    }
    u32::from(!last && sys.ee.check_interrupt())
}

/// The interpreter's machine the verifier steps (`emusen_jit::verify::Reference`).
pub struct Shadow {
    pub sys: Box<System>,
    pub quiet: bool,
}

/// The first difference between the interpreter's machine `a` and the blocks' `b` in everything a step can change: the
/// core's registers, COP0, the issue model, the lines, and the board's time and INTC; `quiet` names it without values.
pub fn compare(a: &System, b: &System, quiet: bool) -> Option<String> {
    let named = |what: &str, x: &dyn std::fmt::Debug, y: &dyn std::fmt::Debug| -> Option<String> {
        Some(if quiet { what.to_string() } else { format!("{what}: interpreter {x:x?}, blocks {y:x?}") })
    };
    let (x, y) = (&a.ee, &b.ee);
    for r in 0..32 {
        if x.gpr[r] != y.gpr[r] {
            return named(&format!("GPR {r}"), &x.gpr[r], &y.gpr[r]);
        }
    }
    macro_rules! same {
        ($($f:ident).+) => {
            if x.$($f).+ != y.$($f).+ {
                return named(stringify!($($f).+), &x.$($f).+, &y.$($f).+);
            }
        };
    }
    same!(hi);
    same!(lo);
    same!(sa);
    same!(pc);
    same!(cop0);
    same!(fpr);
    same!(pcr);
    same!(int0);
    same!(int1);
    same!(cpcond0);
    same!(ucab.valid);
    same!(exceptions);
    if x.timing != y.timing {
        return named("the issue model", &x.timing, &y.timing);
    }
    if (x.in_delay_slot(), x.cycles_seen()) != (y.in_delay_slot(), y.cycles_seen()) {
        return named("the delay slot or the cycles Count saw", &(x.in_delay_slot(), x.cycles_seen()), &(y.in_delay_slot(), y.cycles_seen()));
    }
    let (p, q) = (&a.board, &b.board);
    if p.now != q.now || p.lines() != q.lines() || (p.intc.stat, p.intc.mask) != (q.intc.stat, q.intc.mask) {
        return named("the board's time, lines or INTC", &(p.now, p.lines(), p.intc.stat), &(q.now, q.lines(), q.intc.stat));
    }
    None
}

/// The mnemonics a block compiles at S36, in the decoder's spelling.
pub fn mnemonics() -> impl Iterator<Item = &'static str> {
    shape::compiled()
}

impl emusen_jit::verify::Reference for Shadow {
    type Subject<'a> = &'a System;

    fn step(&mut self) {
        let _ = self.sys.step();
    }

    fn differs(&self, s: &System) -> Option<String> {
        compare(&self.sys, s, self.quiet)
    }

    fn differs_whole(&self, s: &System) -> Option<String> {
        let (a, b) = (state_of(&self.sys), state_of(s));
        (a != b).then(|| format!("in the saved state ({} against {} bytes, first difference at byte {})", a.len(), b.len(), a.iter().zip(&b).position(|(x, y)| x != y).unwrap_or(a.len().min(b.len()))))
    }
}

/// A system's whole state, `SPH2`'s blocks as `System::write_state` writes them.
pub fn state_of(sys: &System) -> Vec<u8> {
    let mut w = emusen_native::StateWriter::counter();
    sys.write_state(&mut w);
    let mut out = vec![0; w.len()];
    let mut w = emusen_native::StateWriter::new(&mut out);
    sys.write_state(&mut w);
    out
}
