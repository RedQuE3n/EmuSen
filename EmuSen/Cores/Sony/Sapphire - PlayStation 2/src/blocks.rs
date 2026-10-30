//! The EE's recompiled tier as the machine runs it (Sapphire_Plan.md §7, S36 and S37): the blocks by virtual address, each
//! checked against the instruction cache on entry, handed to `emusen-jit`'s compiler at its `threshold`-th entry, and run
//! between the interpreter's steps. A compiled block leaves the board behind it; its exit brings the board through each
//! of its instructions' cycles in turn, as `System::step` would have, and keeps the instructions before the first
//! interrupt the interpreter would take, or before the DMAC would run between two of its accesses to memory.
//! Sapphire_Native.md §44 and Sapphire_Recompiler.md §48 are the record.

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::Arc;

use crate::board::{System, Tail};
use crate::ee::Halt;
use crate::ee::jit::{self, Code, Context, FINAL, Jit, Options, PAGES, Page, SHORT, Slot, WORD, kind, shape};
use emusen_jit::verify::Reference;

#[cfg(feature = "baseline")]
mod baseline;
#[cfg(feature = "baseline")]
pub use baseline::counts as baseline_counts;
#[cfg(feature = "baseline")]
pub use baseline::prefix_counts as baseline_prefix_counts;
#[cfg(feature = "baseline")]
pub use crate::ee::jit::baseline::set_dispatched as baseline_dispatched;
/// The baseline's helpers, for its stubs (EmuSen_Recompiler.md §14.4).
#[cfg(feature = "baseline")]
pub(crate) fn baseline_helpers() -> [u64; 5] {
    baseline::helpers()
}

/// Bytes of code past which every block is dropped and compiled again as it runs (`EmuSen_Jit.md` §3.4).
pub const BUDGET: u64 = 256 << 20;
/// The budget when the baseline is dispatched, its blocks' data being larger (EmuSen_Recompiler.md §16.5).
#[cfg(feature = "baseline")]
pub const BASELINE_BUDGET: u64 = 480 << 20;
#[cfg(feature = "baseline")]
/// The most instructions a block is shaped with on the tailored tier: past this, code outgrows the budget (Sapphire_Speed.md §65.5).
const BASELINE_LONGEST: usize = 16;

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
    /// Not compiled: the first instruction is not one a block holds, or the compiler refused it; checked against the
    /// cache again at every 256th entry, and left to the interpreter at the others.
    Refused(u8),
}

struct Block {
    paddr: u32,
    words: Box<[u32]>,
    options: Options,
    state: State,
    /// Entries in a row whose first instruction, a load or store, left at once; and the entries left to the
    /// interpreter after enough of them, as a refused block's are (Sapphire_Recompiler.md §48.4).
    idle: u8,
    rest: u8,
    /// The cycles assumed before its last load or store, `NO_SPAN` when none follows the first instruction (`forms::HORIZON`).
    span: u8,
    /// The baseline compiles it (`EMUSEN_SAPPHIRE_BASELINE=1`): its entries are counted for the static share.
    #[cfg(feature = "baseline")]
    covered: bool,
    /// It holds COP2 instructions: entered only with VU0 quiet (Sapphire_Speed.md §65).
    #[cfg(feature = "baseline")]
    cop2: bool,
}

const NO_SPAN: u8 = u8::MAX;

/// The index of the block's last load or store, one cycle an instruction (Sapphire_Redesign.md, "One horizon check a block").
fn span_of(words: &[u32]) -> u8 {
    match words.iter().rposition(|&w| shape::touches_memory(w)) {
        Some(g) if g > 0 => g as u8,
        _ => NO_SPAN,
    }
}

/// Entries in a row that run nothing before a block is rested, and the entries it rests for.
const IDLE: u8 = 4;
const REST: u8 = 64;

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
    /// Entries left to the interpreter because a line of the block was not in the cache, or a new block ran into one.
    pub refills: u64,
    pub waits: u64,
    /// Rollbacks for the DMAC due between a block's loads and stores, and rollbacks that kept the state after its last
    /// one (Sapphire_Recompiler.md §48.5).
    pub dmac_stops: u64,
    pub kept_past: u64,
    /// Pages the translation cache filled, and the generations it made.
    pub fills: u64,
    pub generations: u64,
    /// Loads and stores that left their blocks, by `Context::left`'s reasons.
    pub left: [u64; 5],
    /// Entries that ran only to the first line the instruction cache did not hold.
    pub to_line: u64,
    /// Exits a load or store took through the block's shared exit with every step standing, and the steps the interpreter
    /// then ran from the last load or store before it (Sapphire_Recompiler.md §48.15).
    pub rewinds: u64,
    pub rewound_steps: u64,
    /// Calls run inside blocks, and entries refused a block holding COP2 while VU0 was not quiet (Sapphire_Speed.md §65).
    pub calls: u64,
    pub vu0_busy: u64,
    /// Entries that found their block's code still with the compiler, the interpreter taking the step.
    pub queued: u64,
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

/// Entries of the direct-mapped front of the block map, by the entry's word address (Sapphire_Speed.md §64.3).
const FRONT: usize = 1 << 16;

pub struct Blocks {
    /// The blocks by entry address: the slab, its index, and the front the dispatcher looks in first.
    slab: Vec<Block>,
    index: HashMap<u32, u32, BuildHasherDefault<Mix>>,
    front: Box<[(u32, u32)]>,
    jit: Jit,
    pub threshold: u32,
    /// Wait for each block's code when it is handed over, as the tests do.
    pub wait: bool,
    /// A new block that would run on into a line neither the cache nor RDRAM holds waits for the interpreter's fetch, and
    /// one past a line the cache does not hold is shaped from RDRAM (true, the default); false shapes it from the lines
    /// the cache holds, so that code run once is compiled in short blocks, as a test of each instruction wants
    /// (Sapphire_Recompiler.md §48.4, §48.7).
    pub wait_lines: bool,
    /// Compromise C1 in the code compiled from now on (off by default; `effect: accuracy`).
    pub fixed_cost: bool,
    /// The registers written out after each instruction, for measuring what holding them buys.
    pub spill: bool,
    /// Loads and stores compiled into blocks (S37); without, each ends its block as at S36.
    pub memory: bool,
    /// A block's COP2 instructions run when VU0 is quiet; off with `EMUSEN_SAPPHIRE_COP2=0`, the block leaving before the first.
    #[cfg(feature = "baseline")]
    pub cop2: bool,
    /// The most instructions a block is shaped with, at most `shape::MAX`.
    pub longest: usize,
    /// A block the baseline covers compiled on the dispatcher's thread when it reaches the threshold, not queued (EmuSen_Recompiler.md §16).
    #[cfg(feature = "baseline")]
    pub now: bool,
    verify: Option<(Verify, emusen_jit::verify::Verifier<Shadow>)>,
    pub stats: Stats,
    /// For measurement, behind the `census` feature: each compiled block entered since the census was last taken, by
    /// its code's address, with its entries, the steps it ran and its guest instructions (Sapphire_Redesign.md, R0
    /// results, the working set).
    #[cfg(feature = "census")]
    pub census: HashMap<usize, (u64, u64, u64)>,
    ctx: Box<Context>,
    /// The translation cache the context points at, and what its generation was made for: the TLB's stamp, the mode,
    /// ASID and configuration, and RDRAM's place.
    pages: Box<[Page]>,
    key: [u64; 3],
    generation: u64,
    /// The last keys seen with their generations: a page filled under one stays valid while its key does, so a return
    /// to a key (the kernel's handler and back to the program) finds its pages again.
    recent: [([u64; 3], u64); 4],
    turn: usize,
}

impl Default for Blocks {
    fn default() -> Self {
        Blocks::new()
    }
}

impl Blocks {
    pub fn new() -> Blocks {
        let mut pages = vec![Page::default(); PAGES].into_boxed_slice();
        let ctx = Box::new(Context {
            times: [0; shape::MAX],
            lines: [0; shape::MAX],
            steps: 0,
            limit: 0,
            replay,
            hook,
            leave,
            owner: std::ptr::null_mut(),
            spill: [0; 36],
            fill,
            pages: pages.as_mut_ptr(),
            gen1: 1 << 32 | 1,
            horizon: 0,
            guard: 0,
            base: 0,
            logged: 0,
            uncached: 0,
            uncached_loads: 0,
            dummy: [0; 2],
            log: [[0; 3]; 4 * shape::MAX],
            left: [0; 5],
            answer: 0,
            calls: 0,
            #[cfg(feature = "baseline")]
            values: [0; 72],
            #[cfg(feature = "baseline")]
            vf0: std::ptr::null_mut(),
            #[cfg(feature = "baseline")]
            room: 0,
            #[cfg(feature = "baseline")]
            quiet: 0,
            #[cfg(feature = "baseline")]
            macro_step: 0,
            #[cfg(feature = "baseline")]
            vu0_saved: None,
            #[cfg(feature = "baseline")]
            handed: [0; 2],
            #[cfg(feature = "forms")]
            commit: jit::forms::commit_trampoline(commit),
            #[cfg(feature = "baseline")]
            moved: 0,
            #[cfg(feature = "baseline")]
            prefix: 0,
        });
        let mut ctx = ctx;
        [ctx.replay, ctx.leave, ctx.fill] = jit::forms::trampolines([ctx.replay, ctx.leave, ctx.fill]);
        #[cfg(feature = "baseline")]
        let budget = if jit::baseline::dispatched() { BASELINE_BUDGET } else { BUDGET };
        #[cfg(not(feature = "baseline"))]
        let budget = BUDGET;
        let mut jit = Jit::new(jit::R5900, budget);
        jit.perf_map = std::env::var("EMUSEN_SAPPHIRE_PERFMAP").is_ok_and(|v| v == "1");
        Blocks {
            slab: Vec::new(),
            index: HashMap::default(),
            front: vec![(0, 0); FRONT].into_boxed_slice(),
            jit,
            threshold: THRESHOLD,
            wait: false,
            wait_lines: true,
            fixed_cost: false,
            spill: false,
            memory: true,
            #[cfg(feature = "baseline")]
            cop2: std::env::var("EMUSEN_SAPPHIRE_COP2").map_or(true, |v| v != "0"),
            longest: shape::MAX,
            #[cfg(feature = "baseline")]
            now: false,
            verify: None,
            stats: Stats::default(),
            #[cfg(feature = "census")]
            census: HashMap::default(),
            ctx,
            pages,
            key: [0; 3],
            generation: 1,
            recent: [([0; 3], 0); 4],
            turn: 0,
        }
    }

    /// The settings the environment gives, for measurement: `EMUSEN_SAPPHIRE_THRESHOLD`, `EMUSEN_SAPPHIRE_C1=1`,
    /// `EMUSEN_SAPPHIRE_SPILL=1`, `EMUSEN_SAPPHIRE_MEMORY=0` (S36's blocks, every load and store left to the interpreter),
    /// `EMUSEN_SAPPHIRE_LONGEST=<n>` (blocks of at most n instructions), `EMUSEN_SAPPHIRE_WAIT=1` (each block's code waited for
    /// when it is handed over, so that nothing runs interpreted behind the compiler's queue), `EMUSEN_SAPPHIRE_BASELINE_NOW=1`
    /// (with `baseline`, each covered block compiled on the dispatcher's thread and the rest left to the interpreter, the default
    /// while the baseline is dispatched; `=0` queues them), `EMUSEN_SAPPHIRE_BUDGET_MIB=<n>` (the code budget).
    pub fn from_env() -> Blocks {
        let mut b = Blocks::new();
        if let Some(t) = std::env::var("EMUSEN_SAPPHIRE_THRESHOLD").ok().and_then(|v| v.parse().ok()) {
            b.threshold = t;
        }
        b.fixed_cost = std::env::var("EMUSEN_SAPPHIRE_C1").is_ok_and(|v| v == "1");
        b.spill = std::env::var("EMUSEN_SAPPHIRE_SPILL").is_ok_and(|v| v == "1");
        b.memory = !std::env::var("EMUSEN_SAPPHIRE_MEMORY").is_ok_and(|v| v == "0");
        b.wait = std::env::var("EMUSEN_SAPPHIRE_WAIT").is_ok_and(|v| v == "1");
        #[cfg(feature = "baseline")]
        {
            b.now = jit::baseline::dispatched() && std::env::var("EMUSEN_SAPPHIRE_BASELINE_NOW").map_or(true, |v| v != "0");
            if jit::baseline::dispatched() {
                b.longest = BASELINE_LONGEST;
            }
        }
        if let Some(mib) = std::env::var("EMUSEN_SAPPHIRE_BUDGET_MIB").ok().and_then(|v| v.parse::<u64>().ok()) {
            b.set_budget(mib << 20);
        }
        if let Some(n) = std::env::var("EMUSEN_SAPPHIRE_LONGEST").ok().and_then(|v| v.parse::<usize>().ok()) {
            b.longest = n.clamp(2, shape::MAX);
        }
        b
    }

    /// The interpreter's machine stepped beside the blocks from now on, `reference` equal to the machine they run;
    /// `quiet` names a difference without its values (a run of the player's BIOS, Sapphire_Plan.md §1.4).
    pub fn verify(&mut self, how: Verify, reference: Box<System>, quiet: bool) {
        self.verify = Some((how, emusen_jit::verify::Verifier::new(Shadow { sys: reference, quiet })));
        self.forget();
    }

    /// The verifier's machine, for what a frame does beside the steps (the IOP's console hook resolved at its end).
    pub fn reference_mut(&mut self) -> Option<&mut System> {
        self.verify.as_mut().map(|(_, v)| &mut *v.reference.sys)
    }

    /// The verifier's machine stepped `n` times beside steps the machine passed at once (idle skipping,
    /// Sapphire_Native.md §47), and the two compared.
    pub fn follow(&mut self, sys: &System, n: u64, what: &str) {
        if let Some((_, v)) = &mut self.verify {
            for _ in 0..n {
                v.reference.step();
                v.steps += 1;
            }
            v.check(sys, what);
        }
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

    /// Every block forgotten, its code kept until the budget drops the module; the translation cache emptied.
    pub fn forget(&mut self) {
        self.slab.clear();
        self.index.clear();
        self.front.fill((0, 0));
        self.key = [0; 3];
        self.recent = [([0; 3], 0); 4];
    }

    /// The translation cache made valid for the machine as it stands: a new generation when anything a page's meaning
    /// depends on has changed since the last entry.
    fn translation(&mut self, sys: &System) {
        let [stamp, mode] = sys.ee.translation_key();
        let key = [stamp, mode, sys.board.map.rdram.as_ptr() as u64];
        if key != self.key {
            self.key = key;
            let generation = match self.recent.iter().find(|(k, _)| *k == key) {
                Some(&(_, g)) => g,
                None => {
                    self.generation += 1;
                    if self.generation > u64::from(u32::MAX) {
                        self.pages.fill(Page::default());
                        self.recent = [([0; 3], 0); 4];
                        self.generation = 1;
                    }
                    self.recent[self.turn] = (key, self.generation);
                    self.turn = (self.turn + 1) % self.recent.len();
                    self.stats.generations += 1;
                    self.generation
                }
            };
            self.ctx.gen1 = generation << 32 | 1;
        }
    }

    /// The compiler's counters: blocks compiled, refused, and bytes of code.
    pub fn compiled(&self) -> (u64, u64, u64) {
        use std::sync::atomic::Ordering::Relaxed;
        self.jit.counters().map_or((0, 0, 0), |c| (c.compiled.load(Relaxed), c.failed.load(Relaxed), c.bytes.load(Relaxed)))
    }

    /// The compiler's time spent compiling, in nanoseconds.
    pub fn compile_nanos(&self) -> u64 {
        self.jit.counters().map_or(0, |c| c.nanos.load(std::sync::atomic::Ordering::Relaxed))
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
    pub fn budget(&self) -> u64 {
        self.jit.budget
    }
    /// Of the flushes, those for a full code memory under the budget.
    pub fn full_flushes(&self) -> u64 {
        self.jit.full
    }

    /// What compiling has cost across the budget's drops, on the compiler's thread and on the dispatcher's, and the bytes held now.
    pub fn spent(&self) -> ([emusen_jit::Spent; 2], u64) {
        (self.jit.spent(), self.jit.held())
    }

    /// The mnemonics in the blocks that hold code now.
    pub fn compiled_mnemonics(&self) -> std::collections::BTreeSet<&'static str> {
        self.slab.iter().filter(|b| matches!(b.state, State::Ready(_))).flat_map(|b| b.words.iter().map(|&w| shape::name(w))).collect()
    }

    /// The block entered at `pc`, through the front.
    #[inline]
    fn find(&mut self, pc: u32) -> Option<usize> {
        let f = &mut self.front[(pc >> 2) as usize & (FRONT - 1)];
        if f.0 == pc && f.1 != 0 {
            return Some(f.1 as usize - 1);
        }
        let i = *self.index.get(&pc)?;
        *f = (pc, i + 1);
        Some(i as usize)
    }

    /// The block at `pc` made `b`, and whether one was there.
    fn put(&mut self, pc: u32, b: Block) -> (usize, bool) {
        if let Some(i) = self.find(pc) {
            self.slab[i] = b;
            return (i, true);
        }
        let i = self.slab.len();
        self.slab.push(b);
        self.index.insert(pc, i as u32);
        self.front[(pc >> 2) as usize & (FRONT - 1)] = (pc, i as u32 + 1);
        (i, false)
    }

    fn options(&self, sys: &System) -> Options {
        Options { single: sys.ee.single_issue(), verify: matches!(self.verify, Some((Verify::Steps, _))), fixed_cost: self.fixed_cost, spill: self.spill, memory: self.memory, blocking: sys.ee.blocking_loads() }
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
    /// What is left of the time the replay stopped inside, when it stopped before a wake of the DMAC.
    tail: Option<Tail>,
    /// The replay stopped for the DMAC, not for an interrupt.
    guarded: bool,
    /// The shared exit's replay, every step standing.
    rewound: bool,
    fills: u64,
}

fn block(sys: &mut System, blocks: &mut Blocks, left: u64) -> Option<u64> {
    if blocks.jit.over_budget() {
        blocks.forget();
    }
    // A block not compiled yet is counted, and one refused passed over, without the entry's checks: neither runs code.
    let threshold = blocks.threshold;
    let pc = sys.ee.pc;
    let found = blocks.find(pc);
    if let Some(i) = found
        && blocks.slab[i].rest > 0
    {
        blocks.slab[i].rest -= 1;
        return None;
    }
    match found.map(|i| &mut blocks.slab[i].state) {
        Some(State::Refused(n)) => {
            *n = n.wrapping_add(1);
            if *n != 0 {
                return None;
            }
        }
        Some(State::Counting(n)) if *n + 1 < threshold => {
            *n += 1;
            return None;
        }
        _ => {}
    }
    let paddr = sys.ee.block_entry()?;
    let options = blocks.options(sys);
    let held = found.and_then(|i| {
        let b = &blocks.slab[i];
        if b.paddr == paddr && b.options == options { sys.ee.held_words(pc, paddr, &b.words) } else { None }
    });
    let mut at = found;
    if held.is_none() {
        // Past a line the cache does not hold, the words RDRAM holds; only the lines held when the dispatcher does not
        // wait for lines (a test's code run once).
        let rdram: &[u8] = if blocks.wait_lines { &sys.board.map.rdram } else { &[] };
        let (words, missed) = sys.ee.cached_words(pc, paddr, blocks.longest, rdram);
        #[cfg(feature = "baseline")]
        let calls = jit::baseline::calls(options);
        #[cfg(not(feature = "baseline"))]
        let calls = false;
        let (mut shaped, mut short) = shape::shape(&words, options.memory, calls);
        #[cfg(feature = "baseline")]
        if calls && shaped.as_ref().is_some_and(|w| w.iter().any(|&x| shape::class(x) == shape::Class::Call) && !jit::baseline::covers(w, options)) {
            // Calls only in blocks the baseline compiles; this one is shaped again without them.
            (shaped, short) = shape::shape(&words, options.memory, false);
        }
        if short && missed && blocks.wait_lines {
            // The block may run on into a line the cache does not hold yet: shaped once the interpreter has fetched it.
            blocks.stats.waits += 1;
            return None;
        }
        let state = if shaped.is_some() { State::Counting(0) } else { State::Refused(0) };
        let words = shaped.unwrap_or_else(|| words.get(..1).map_or_else(Vec::new, <[u32]>::to_vec));
        if words.is_empty() {
            return None;
        }
        let span = span_of(&words);
        let (i, was) = blocks.put(pc, Block {
            paddr,
            #[cfg(feature = "baseline")]
            covered: jit::baseline::dispatched() && jit::baseline::covers(&words, options),
            #[cfg(feature = "baseline")]
            cop2: words.iter().any(|&w| shape::class(w) == shape::Class::Call && jit::baseline::cop2(w)),
            words: words.into_boxed_slice(),
            options,
            state,
            idle: 0,
            rest: 0,
            span,
        });
        blocks.stats.reshaped += u64::from(was);
        at = Some(i);
    }
    let wait = blocks.wait;
    let at = at.expect("the block is in the map");
    let b = &mut blocks.slab[at];
    // The instructions before the first line the cache does not hold, to the last line the block can leave at.
    let held = held.unwrap_or_else(|| sys.ee.held_words(pc, paddr, &b.words).unwrap_or(0));
    let b_len = b.words.len();
    let limit = if held == b_len { held } else { leaving_point(pc, &b.words, held) };
    if limit == 0 {
        // The block's first line is not in the cache: the interpreter's fetch refills it, and the block stands.
        blocks.stats.refills += 1;
        return None;
    }
    let code = match &b.state {
        State::Ready(code) => *code,
        State::Refused(_) => return None,
        State::Counting(n) => {
            let n = n + 1;
            if n < threshold {
                b.state = State::Counting(n);
                return None;
            }
            #[cfg(feature = "baseline")]
            let now = if blocks.now && b.covered { blocks.jit.compile_now(&b.words, pc, b.options) } else { None };
            #[cfg(not(feature = "baseline"))]
            let now = None;
            #[cfg(feature = "baseline")]
            if blocks.now && now.is_none() {
                // Compiled on this thread or not at all, so that which steps run compiled never depends on another thread's time.
                b.state = State::Refused(0);
                return None;
            }
            let slot = now.unwrap_or_else(|| blocks.jit.enqueue(b.words.clone(), pc, b.options));
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
                _ => {
                    blocks.stats.queued += 1;
                    return None;
                }
            }
        }
    };
    if b.words.len() as u64 > left {
        return None;
    }
    #[cfg(feature = "baseline")]
    let one_check = b.covered || jit::forms::chosen() & jit::forms::HORIZON != 0;
    #[cfg(not(feature = "baseline"))]
    let one_check = jit::forms::chosen() & jit::forms::HORIZON != 0;
    #[cfg(feature = "baseline")]
    let at_each = b.covered;
    #[cfg(not(feature = "baseline"))]
    let at_each = false;
    if !at_each && b.span != NO_SPAN && !b.options.verify && one_check && sys.ee.timing.cycles() + u64::from(b.span) >= sys.board.next_event() {
        // The block's last access would reach the board's next event at one cycle an instruction: counted as left at the event.
        blocks.stats.left[0] += 1;
        return None;
    }
    #[cfg(feature = "baseline")]
    let quiet = b.cop2 && blocks.cop2 && sys.board.vu0_quiet(sys.ee.timing.last() + 1, jit::baseline::QUIET);
    #[cfg(feature = "baseline")]
    {
        blocks.stats.vu0_busy += u64::from(b.cop2 && !quiet);
    }
    blocks.translation(sys);
    #[cfg(feature = "baseline")]
    {
        (blocks.ctx.vu0_saved, blocks.ctx.macro_step, blocks.ctx.quiet) = (None, 0, u32::from(quiet));
        blocks.ctx.room = sys.board.next_event().saturating_sub(sys.ee.timing.cycles()).min(u64::from(u32::MAX)) as u32;
        if quiet {
            blocks.ctx.vf0 = sys.board.vpu.units.vu0.vf_base();
        }
    }
    let mut run = Run { sys, verify: &mut blocks.verify, failed: None, tail: None, guarded: false, rewound: false, fills: 0 };
    blocks.stats.to_line += u64::from(limit < b_len);
    let ctx = &mut *blocks.ctx;
    ctx.owner = (&mut run as *mut Run).cast();
    ctx.limit = limit as u32;
    ctx.horizon = sys.board.next_event();
    (ctx.uncached, ctx.uncached_loads, ctx.guard, ctx.base, ctx.logged) = (0, 0, 0, 0, 0);
    // SAFETY: the code was compiled for this signature; the core and the context outlive the call, and nothing else
    // borrows the machine while the code and its helpers run through these pointers.
    #[cfg(feature = "baseline")]
    {
        (ctx.moved, ctx.prefix) = (0, 0);
    }
    let exit = unsafe { code(&raw mut (*run.sys).ee, ctx) };
    if let Some(f) = run.failed.take() {
        panic!("{f}");
    }
    let steps = ctx.steps;
    #[cfg(feature = "baseline")]
    if blocks.slab[at].covered {
        baseline::count(ctx.moved != 0, ctx.prefix, steps);
    }
    #[cfg(feature = "census")]
    {
        let e = blocks.census.entry(code as usize).or_default();
        e.0 += 1;
        e.1 += u64::from(steps);
        e.2 = b_len as u64;
    }
    blocks.stats.fills += run.fills;
    blocks.stats.calls += u64::from(std::mem::take(&mut ctx.calls));
    for (total, n) in blocks.stats.left.iter_mut().zip(ctx.left.iter_mut()) {
        *total += std::mem::take(n);
    }
    match exit {
        jit::DONE => {
            let b = &mut blocks.slab[at];
            if steps == 0 {
                // Its first instruction's access left at once (a register polled, a miss): after a few in a row the
                // interpreter takes the next entries without the entry's checks.
                b.idle += 1;
                if b.idle >= IDLE {
                    (b.idle, b.rest) = (0, REST);
                }
                return None;
            }
            b.idle = 0;
            #[cfg(feature = "baseline")]
            last_macro(sys, ctx);
            sys.board.traffic[0] += ctx.uncached_loads;
            sys.board.traffic[1] += ctx.uncached;
            sys.ee.count_steps(u64::from(steps));
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
            // The replay brought the board through `steps` instructions' cycles and found an interrupt due after them, or
            // the DMAC due before the block's last access: the interpreter runs those instructions with the lines each
            // step would have seen, then the wake the replay held. Past the last access the core holds the state after it
            // and the instructions from there run again; short of it the stores are undone and every instruction runs
            // again (Sapphire_Recompiler.md §48.5).
            // From the shared exit with every step standing, the same: the steps from the last load or store to the
            // access that left run here (§48.15).
            let (steps, base) = (steps as usize, ctx.base as usize);
            let kept_past = base > 0 && steps >= base;
            let start = if kept_past { base } else { 0 };
            if kept_past {
                #[cfg(feature = "baseline")]
                last_macro(sys, ctx);
                sys.board.traffic[0] += ctx.uncached_loads;
                sys.board.traffic[1] += ctx.uncached;
                sys.ee.count_steps(base as u64);
            } else {
                #[cfg(feature = "baseline")]
                if let Some((regs, last)) = &ctx.vu0_saved {
                    sys.board.vpu.units.vu0.set_macro_regs(regs);
                    sys.board.vpu.last_macro = *last;
                }
                undo(ctx);
            }
            if run.rewound {
                let b = &mut blocks.slab[at];
                b.idle = 0;
                blocks.stats.entered += 1;
                blocks.stats.compiled_steps += start as u64;
                blocks.stats.rewinds += 1;
                blocks.stats.rewound_steps += (steps - start) as u64;
            } else {
                blocks.stats.kept_past += u64::from(kept_past);
                blocks.stats.dmac_stops += u64::from(run.guarded);
                blocks.stats.rollbacks += 1;
                blocks.stats.rerun_steps += (steps - start) as u64;
            }
            for j in start..steps {
                if j > 0 {
                    set_lines(sys, ctx.lines[j - 1]);
                }
                sys.ee.step(&mut sys.board).expect("a block's instruction is the interpreter's");
            }
            match run.tail {
                Some(tail) => {
                    let System { ee, board } = &mut *sys;
                    board.finish(tail, &mut ee.spram[..]);
                    set_lines(sys, pack(sys.board.lines()));
                }
                None if steps > 0 => set_lines(sys, ctx.lines[steps - 1]),
                None => {}
            }
            if kept_past && steps == base && base >= 2 {
                // No step ran again to make the interrupt check's write of IP2 and IP3 that the last kept one made.
                set_cause_lines(sys, ctx.lines[base - 2]);
            }
            if let Some((_, v)) = &mut blocks.verify {
                for _ in 0..steps {
                    v.reference.step();
                    v.steps += 1;
                }
                v.check(sys, "a block's instructions run again before an interrupt or the DMAC");
            }
            (steps > 0).then_some(steps as u64)
        }
    }
}

/// The last instruction at most `held` into the block where it can leave for a line the cache does not hold: a line's
/// first instruction that is not a delay slot (`emit::lower`'s check); 0 when there is none.
fn leaving_point(pc: u32, words: &[u32], held: usize) -> usize {
    (1..=held.min(words.len().saturating_sub(1)))
        .rev()
        .find(|&i| pc.wrapping_add(4 * i as u32) & 63 == 0 && shape::class(words[i - 1]) != shape::Class::Branch)
        .unwrap_or(0)
}

/// The block's stores undone, last first, from the context's log.
fn undo(ctx: &Context) {
    for &[addr, lo, hi] in ctx.log[..ctx.logged as usize].iter().rev() {
        let p = (addr & !(SHORT | WORD)) as *mut u8;
        // SAFETY: each address is one the block's code read 4, 8 or 16 bytes at and stored into during this call, inside
        // the core, the scratchpad, RDRAM or the context, all alive and borrowed by nothing else now.
        unsafe {
            if addr & WORD != 0 {
                std::ptr::write_unaligned(p.cast::<u32>(), lo as u32);
                continue;
            }
            std::ptr::write_unaligned(p.cast::<u64>(), lo);
            if addr & SHORT == 0 {
                std::ptr::write_unaligned(p.add(8).cast::<u64>(), hi);
            }
        }
    }
}

/// VU0's last macro operation's issue cycle, once the block's steps stand and their cycles are known.
#[cfg(feature = "baseline")]
fn last_macro(sys: &mut System, ctx: &Context) {
    if ctx.macro_step != 0 {
        sys.board.vpu.last_macro = Some(ctx.times[ctx.macro_step as usize - 1] - 1);
    }
}

fn set_cause_lines(sys: &mut System, l: u8) {
    use crate::ee::cop0::{CAUSE, cause};
    let c = &mut sys.ee.cop0[CAUSE];
    *c = *c & !(cause::IP2 | cause::IP3) | if l & 1 != 0 { cause::IP2 } else { 0 } | if l & 2 != 0 { cause::IP3 } else { 0 };
}

/// The translation cache's fill: the page of `vaddr` as a data access in the current mode would place it, cached
/// whatever it is, so that a page the code cannot serve is not asked for again (Sapphire_Recompiler.md §48.3).
extern "C" fn fill(ctx: *mut Context, vaddr: u32) -> u32 {
    use crate::ee::{Mode, Place};
    // SAFETY: called by compiled code during `block`, whose `Run` and context are alive and borrowed by nothing else.
    let ctx = unsafe { &mut *ctx };
    let run = unsafe { &mut *ctx.owner.cast::<Run>() };
    let sys = unsafe { &mut *run.sys };
    run.fills += 1;
    let seen = jit::forms::watch(&sys.ee, ctx);
    let (meta, host) = match sys.ee.data_page(vaddr) {
        None => (kind::NONE, 0),
        Some((place, writable)) => {
            let w = if writable { kind::WRITABLE } else { 0 };
            match place {
                Place::Scratchpad(o) => (kind::SCRATCHPAD | w, sys.ee.spram.as_ptr() as u64 + u64::from(o & !0xFFF)),
                Place::Physical(p, Mode::Cached) => (kind::CACHED | w | u64::from(p >> 12) << 32, 0),
                Place::Physical(p, m @ (Mode::Uncached | Mode::Accelerated)) if (p as usize) < sys.board.map.rdram.len() => {
                    let accel = if m == Mode::Accelerated { kind::ACCELERATED } else { 0 };
                    (kind::UNCACHED | accel | w, sys.board.map.rdram.as_ptr() as u64 + u64::from(p & !0xFFF))
                }
                Place::Physical(..) => (kind::NONE, 0),
            }
        }
    };
    let i = jit::page_index(vaddr);
    // SAFETY: the pages the context points at are `Blocks::pages`, alive for the call.
    let page = unsafe { &mut *ctx.pages.add(i) };
    *page = Page { tag: ctx.gen1 | u64::from(vaddr & !0xFFF), host, meta, pad: 0 };
    jit::forms::watched(seen, &sys.ee, ctx, 2);
    1
}

fn settle(slot: Arc<Slot>) -> Option<State> {
    if let Some(code) = jit::published(&slot) {
        Some(State::Ready(code))
    } else if slot.settled() {
        Some(State::Refused(0))
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
    // SAFETY: called by compiled code during `block`, whose context is alive and borrowed by nothing else.
    let ctx = unsafe { &mut *ctx };
    let seen = watch(ctx);
    let kept = replay_steps(ctx, steps, false);
    watched(seen, ctx, 0);
    kept
}

/// `forms::STUB`'s shared exit (`jit::commit_word`): the guard set, the board replayed, and with every step standing the exit's commit.
#[cfg(feature = "forms")]
extern "C" fn commit(ctx: *mut Context, word: u64, place: u64, end: u64) -> u32 {
    // SAFETY: as `replay`.
    let ctx = unsafe { &mut *ctx };
    let (steps, base, logged) = ((word & 0x7F) as u32, (word >> 7 & 0x7F) as u32, (word >> 14 & 0xFF) as u32);
    (ctx.guard, ctx.base, ctx.logged) = (base.saturating_sub(1), base, logged);
    let seen = watch(ctx);
    let kept = replay_steps(ctx, steps, false);
    watched(seen, ctx, 0);
    if kept == steps {
        // SAFETY: as `replay`.
        let sys = unsafe { &mut *(*ctx.owner.cast::<Run>()).sys };
        sys.ee.commit_exit(word, place, end);
        ctx.steps = steps;
    }
    kept
}

/// `forms::watch` and `forms::watched` around a helper's body.
fn watch(ctx: &Context) -> Option<jit::forms::Watched> {
    // SAFETY: as `replay`.
    let sys = unsafe { &*(*ctx.owner.cast::<Run>()).sys };
    jit::forms::watch(&sys.ee, ctx)
}

fn watched(seen: Option<jit::forms::Watched>, ctx: &Context, which: usize) {
    // SAFETY: as `replay`.
    let sys = unsafe { &*(*ctx.owner.cast::<Run>()).sys };
    jit::forms::watched(seen, &sys.ee, ctx, which);
}

/// A load or store's exit once its block has run a step (`jit::leave_word`): the reason counted, the guard set and the
/// board replayed; with every step standing, the lines are left to the dispatcher, which runs the steps after the last
/// load or store as after a rollback (Sapphire_Recompiler.md §48.15).
extern "C" fn leave(ctx: *mut Context, word: u32) -> u32 {
    // SAFETY: as `replay`.
    let ctx = unsafe { &mut *ctx };
    let (steps, base, logged, why) = jit::left_by(word);
    ctx.left[why] += 1;
    (ctx.guard, ctx.base, ctx.logged) = (base.saturating_sub(1), base, logged);
    let seen = watch(ctx);
    let kept = replay_steps(ctx, steps, true);
    watched(seen, ctx, 1);
    kept
}

fn replay_steps(ctx: &mut Context, steps: u32, rewind: bool) -> u32 {
    use crate::ee::cop0::{CAUSE, COMPARE, COUNT, STATUS, cause};
    // SAFETY: during `block`, whose `Run` is alive and borrowed by nothing else while the code and its helpers run.
    let run = unsafe { &mut *ctx.owner.cast::<Run>() };
    let sys = unsafe { &mut *run.sys };
    let (count0, compare, seen0) = (sys.ee.cop0[COUNT], sys.ee.cop0[COMPARE], sys.ee.cycles_seen());
    let (status, cause_now) = (sys.ee.cop0[STATUS], sys.ee.cop0[CAUSE]);
    let ip7 = cause_now & cause::IP7 != 0;
    let m = steps as usize;
    let System { ee, board } = sys;
    let mut seen = Seen { times: &ctx.times[..m], lines: &mut ctx.lines, ip7, compare, count0, seen0, status, cause_now, interrupted: false };
    let (ran, tail) = board.run_through(&ctx.times[..m], &mut ee.spram[..], ctx.guard as usize, &mut seen);
    let interrupted = seen.interrupted;
    run.tail = tail;
    run.guarded = ran < m && !interrupted;
    if ran < m || rewind {
        run.rewound = ran == m;
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

/// What the replay sees at each time: the lines kept, and whether the interpreter would take an interrupt before the
/// next instruction, Count's against Compare at that time.
struct Seen<'a> {
    times: &'a [u64],
    lines: &'a mut [u8; shape::MAX],
    ip7: bool,
    compare: u32,
    count0: u32,
    seen0: u64,
    status: u32,
    cause_now: u32,
    interrupted: bool,
}

impl Seen<'_> {
    fn ip7(&self, j: usize) -> bool {
        self.ip7 || u64::from(self.compare.wrapping_sub(self.count0).wrapping_sub(1)) < self.times[j].wrapping_sub(self.seen0)
    }
}

impl crate::board::Times for Seen<'_> {
    fn each(&mut self, j: usize, lines: (bool, bool, bool)) -> bool {
        self.lines[j] = pack(lines);
        let ip7 = self.ip7(j);
        self.interrupted = j + 1 < self.times.len() && crate::ee::Ee::interrupt_due(self.status, self.cause_now, lines.0, lines.1, ip7);
        !self.interrupted
    }

    /// The lines standing, an interrupt is due before the next instruction where it is due with Count's request as it
    /// stands at that time; the last time is never one.
    fn quiet(&mut self, from: usize, to: usize, lines: (bool, bool, bool)) -> usize {
        self.lines[from..to].fill(pack(lines));
        let due = |ip7| crate::ee::Ee::interrupt_due(self.status, self.cause_now, lines.0, lines.1, ip7);
        let (without, with) = (due(false), due(true));
        let last = self.times.len() - 1;
        if !without && !with {
            return to;
        }
        (from..to.min(last)).find(|&j| if self.ip7(j) { with } else { without }).unwrap_or(to)
    }
}

/// The verifier's call after each compiled instruction, everything written back: the board brought to its cycles, the
/// interpreter stepped beside it and the two compared; then, unless it was the block's last, the next step's interrupt
/// check made, its answer whether the block leaves here.
extern "C" fn hook(ctx: *mut Context, steps: u32) -> u32 {
    // SAFETY: as `replay`.
    let ctx = unsafe { &mut *ctx };
    let run = unsafe { &mut *ctx.owner.cast::<Run>() };
    let sys = unsafe { &mut *run.sys };
    let seen = jit::forms::watch(&sys.ee, ctx);
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
    let due = u32::from(!last && sys.ee.check_interrupt());
    jit::forms::watched(seen, &sys.ee, ctx, 3);
    due
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
    same!(interrupt_after);
    same!(cpcond0);
    same!(ucab.valid);
    same!(exceptions);
    // What a block's loads and stores reach besides RDRAM, which the whole state compares.
    if let Some(i) = (0..x.dcache.tags.len()).find(|&i| x.dcache.tags[i] != y.dcache.tags[i] || x.dcache.data[i] != y.dcache.data[i]) {
        return named(&format!("the data cache's set {i}"), &x.dcache.tags[i], &y.dcache.tags[i]);
    }
    if let Some(i) = (0..x.spram.len()).find(|&i| x.spram[i] != y.spram[i]) {
        return named(&format!("the scratchpad's byte {i:x}"), &x.spram[i], &y.spram[i]);
    }
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
