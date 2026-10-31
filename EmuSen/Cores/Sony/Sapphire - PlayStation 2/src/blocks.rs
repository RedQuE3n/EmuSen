//! The EE's recompiled tier as the machine runs it (Sapphire_Speed.md §66): the blocks by virtual address, compiled at their
//! `threshold`-th entry on the dispatcher's thread, linked to one another, and run between the interpreter's steps. Compiled
//! code runs until a block's end reaches the deadline, the board's next event or the round's end, and the board then
//! catches up with it. A round of 0, the cycle-ordered debug build, runs the interpreter alone.

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

use emusen_jit::native::Memory;
use emusen_jit::verify::Reference;

use crate::board::System;
use crate::ee::Halt;
use crate::ee::jit::isa::{self, BOARD, Homes, IN_SLOT, NEXT_HELD, Rec, stub};
use crate::ee::jit::lower::{self, NEXT};
use crate::ee::jit::{self, Context, Enter, FRONT, Jump, Options, PAGES, Page, kind, shape};

/// Bytes of code past which every block is dropped and compiled again as it runs.
pub const BUDGET: u64 = 480 << 20;

/// A block is compiled at this entry.
pub const THRESHOLD: u32 = 64;

/// How the interpreter is stepped beside the blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verify {
    /// After the blocks' run, the interpreter stepped as many times and the two compared at that checkpoint.
    Blocks,
}

enum State {
    Counting(u32),
    /// Compiled, the body's address.
    Ready(u64),
    /// Not compiled: the first instruction is not one a block holds; looked at again at every 256th entry.
    Refused(u8),
    /// Its words were overwritten: shaped again at its next entry.
    Stale,
}

struct Block {
    pc: u32,
    paddr: u32,
    words: Box<[u32]>,
    options: Options,
    state: State,
    /// The link words that jump here, with what each held before.
    incoming: Vec<(*mut u64, u64)>,
}

/// What the dispatcher counted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    /// Times compiled code was entered and left, and the instructions it ran.
    pub entered: u64,
    pub compiled_steps: u64,
    /// Steps the interpreter took.
    pub interpreted: u64,
    /// Blocks shaped again because their page moved or their words changed.
    pub reshaped: u64,
    /// Pages the translation cache filled, and the generations it made.
    pub fills: u64,
    pub generations: u64,
    /// Slow paths by `Context::left`'s reasons.
    pub left: [u64; 5],
    /// Links made, and watched lines found written, with the blocks they killed.
    pub links: u64,
    pub written: u64,
    pub killed: u64,
    /// Every link cut for a new mapping.
    pub unlinked: u64,
    /// Bytes of hot code made, and the guest instructions they hold.
    pub hot_bytes: u64,
    pub guest_words: u64,
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

type Map32<V> = HashMap<u32, V, BuildHasherDefault<Mix>>;

pub struct Blocks {
    /// The blocks by entry address: the slab, its index, and the table compiled code and the dispatcher look in first.
    slab: Vec<Block>,
    index: Map32<u32>,
    front: Box<[Jump]>,
    /// The blocks with code from each physical page.
    by_page: Map32<Vec<u32>>,
    /// The code memory, the bytes past which it is dropped, what dropped ones spent, and the time spent compiling.
    memory: Option<Memory>,
    pub budget: u64,
    pub flushes: u64,
    compiled: (u64, u64, u64),
    pub threshold: u32,
    /// Loops the dispatcher must see: a backward jump's link waits for idle skipping to refuse its head (Sapphire_Speed.md §66).
    pub watch_loops: bool,
    refused: Map32<u8>,
    /// The longest block shaped, for measurement.
    pub longest: usize,
    verify: Option<(Verify, emusen_jit::verify::Verifier<Shadow>)>,
    pub stats: Stats,
    /// For measurement, behind the `census` feature: compiled code entered since the census was last taken.
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
    /// What a link's target depends on: the key without the user, supervisor and kernel modes.
    link_key: [u64; 3],
    /// `forget` ran: the lines' watch goes at the next entry, where the machine is at hand.
    forgot: bool,
}

// SAFETY: the pointers are into the blocks' own memory and boxes, which move with them.
unsafe impl Send for Blocks {}

impl Default for Blocks {
    fn default() -> Self {
        Blocks::new()
    }
}

impl Blocks {
    pub fn new() -> Blocks {
        let mut pages = vec![Page::default(); PAGES].into_boxed_slice();
        let mut front = vec![Jump::default(); FRONT].into_boxed_slice();
        let ctx = Box::new(Context {
            deadline: 0,
            time: 0,
            steps: 0,
            pages: pages.as_mut_ptr(),
            gen1: 1 << 32 | 1,
            front: front.as_mut_ptr(),
            vf0: std::ptr::null_mut(),
            quiet: 0,
            from: 0,
            to: 0,
            pad: 0,
            link: 0,
            owner: std::ptr::null_mut(),
            dummy: [0; 2],
            saved: [0; 7],
            left: [0; 5],
        });
        Blocks {
            slab: Vec::new(),
            index: HashMap::default(),
            front,
            by_page: HashMap::default(),
            memory: None,
            budget: BUDGET,
            flushes: 0,
            compiled: (0, 0, 0),
            threshold: THRESHOLD,
            watch_loops: false,
            refused: HashMap::default(),
            longest: shape::MAX,
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
            link_key: [0; 3],
            forgot: false,
        }
    }

    /// The settings the environment gives, for measurement: `EMUSEN_SAPPHIRE_THRESHOLD`, `EMUSEN_SAPPHIRE_BUDGET_MIB=<n>` (the code budget), `EMUSEN_SAPPHIRE_LONGEST=<n>`.
    pub fn from_env() -> Blocks {
        let mut b = Blocks::new();
        if let Some(t) = std::env::var("EMUSEN_SAPPHIRE_THRESHOLD").ok().and_then(|v| v.parse().ok()) {
            b.threshold = t;
        }
        if let Some(mib) = std::env::var("EMUSEN_SAPPHIRE_BUDGET_MIB").ok().and_then(|v| v.parse::<u64>().ok()) {
            b.budget = mib << 20;
        }
        if let Some(n) = std::env::var("EMUSEN_SAPPHIRE_LONGEST").ok().and_then(|v| v.parse::<usize>().ok()) {
            b.longest = n.clamp(1, shape::MAX);
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

    /// Every block forgotten, the table and the translation cache emptied; the lines' watch goes at the next entry.
    pub fn forget(&mut self) {
        self.slab.clear();
        self.index.clear();
        self.by_page.clear();
        let code = self.memory.as_ref().map_or(0, |m| m.stub(stub::OUT0));
        self.front.fill(Jump { code, ..Jump::default() });
        self.key = [0; 3];
        self.link_key = [0; 3];
        self.recent = [([0; 3], 0); 4];
        self.ctx.link = 0;
        self.forgot = true;
    }

    /// The translation cache made valid for the machine as it stands: a new generation when anything a page's meaning
    /// depends on has changed since the last entry; every link cut when what a link's target depends on has.
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
            let link_key = [key[0], mode & !(3 << 8) | (mode >> 8 & 3) / 3 << 8, key[2]];
            if link_key != self.link_key {
                self.link_key = link_key;
                self.unlink_all();
            }
        }
    }

    /// Every link cut, so that the next jump to each block goes through the dispatcher's checks.
    fn unlink_all(&mut self) {
        let out = self.memory.as_ref().map_or(0, |m| m.stub(stub::OUT0));
        for i in 0..self.slab.len() {
            self.unlink(i);
        }
        for j in self.front.iter_mut() {
            j.code = out;
            j.pc = 0;
            j.block = 0;
        }
        self.stats.unlinked += 1;
    }

    /// The links into block `i` cut, each word back to what it held; the table no longer reaches it.
    fn unlink(&mut self, i: usize) {
        for (slot, was) in self.slab[i].incoming.drain(..) {
            // SAFETY: a link word of a block in the code memory, alive until `forget`, which drops the `Block`s first.
            unsafe { *slot = was };
        }
    }

    /// The blocks made, the time spent compiling them, and bytes of code held.
    pub fn compiled(&self) -> (u64, u64, u64) {
        (self.compiled.0, 0, self.memory.as_ref().map_or(0, Memory::held))
    }

    /// The time spent compiling, in nanoseconds.
    pub fn compile_nanos(&self) -> u64 {
        self.compiled.1
    }

    /// The mnemonics in the blocks that hold code now.
    pub fn compiled_mnemonics(&self) -> std::collections::BTreeSet<&'static str> {
        self.slab.iter().filter(|b| matches!(b.state, State::Ready(_))).flat_map(|b| b.words.iter().map(|&w| shape::name(w))).collect()
    }

    /// The block entered at `pc`, through the table.
    #[inline]
    fn find(&mut self, pc: u32) -> Option<usize> {
        let k = (pc >> 2) as usize & (FRONT - 1);
        let f = self.front[k];
        if f.pc == pc && f.block != 0 {
            return Some(f.block as usize - 1);
        }
        let i = *self.index.get(&pc)? as usize;
        self.point(pc, i);
        Some(i)
    }

    /// The table's entry for `pc` made block `i`.
    fn point(&mut self, pc: u32, i: usize) {
        let code = match self.slab[i].state {
            State::Ready(c) => c,
            _ => self.memory().stub(stub::OUT0),
        };
        self.front[(pc >> 2) as usize & (FRONT - 1)] = Jump { pc, block: i as u32 + 1, code };
    }

    /// The block at `pc` made `b`, and whether one was there.
    fn put(&mut self, pc: u32, b: Block) -> (usize, bool) {
        if let Some(i) = self.find(pc) {
            self.unlink(i);
            self.slab[i] = b;
            self.point(pc, i);
            return (i, true);
        }
        let i = self.slab.len();
        self.slab.push(b);
        self.index.insert(pc, i as u32);
        self.point(pc, i);
        (i, false)
    }

    fn options(&self, sys: &System) -> Options {
        Options { single: sys.ee.single_issue(), blocking: sys.ee.blocking_loads() }
    }

    /// The memory, made with its stubs at the first block.
    fn memory(&mut self) -> &mut Memory {
        if self.memory.is_none() {
            let m = Memory::new(isa::stubs(slow as *const () as u64, fill as *const () as u64));
            let code = m.stub(stub::OUT0);
            self.front.iter_mut().for_each(|j| j.code = code);
            self.memory = Some(m);
        }
        self.memory.as_mut().expect("just made")
    }

    /// The block at `at` compiled, or left to the interpreter where the planner refuses it.
    fn compile(&mut self, pc: u32, at: usize) -> Option<u64> {
        let t = std::time::Instant::now();
        let (words, options) = (self.slab[at].words.clone(), self.slab[at].options);
        let code = isa::compile(&words, pc, options);
        let Some(code) = code else {
            self.slab[at].state = State::Refused(0);
            return None;
        };
        self.stats.hot_bytes += code.hot.len() as u64;
        self.stats.guest_words += words.len() as u64;
        let body = self.memory().place(code)?;
        self.slab[at].state = State::Ready(body);
        self.point(pc, at);
        self.compiled.0 += 1;
        self.compiled.1 += t.elapsed().as_nanos() as u64;
        Some(body)
    }

    /// The watched lines written since the last entry: a block that still holds the words memory holds is watched again, the others are killed.
    fn written(&mut self, sys: &mut System) {
        let lines = std::mem::take(&mut sys.board.map.written);
        self.stats.written += lines.len() as u64;
        for &line in &lines {
            let Some(list) = self.by_page.get(&(line >> 6)) else { continue };
            for i in list.clone() {
                let b = &self.slab[i as usize];
                let (from, to) = (b.paddr >> 6, (b.paddr + 4 * b.words.len() as u32 - 1) >> 6);
                if !(from..=to).contains(&line) || matches!(b.state, State::Stale) {
                    continue;
                }
                if sys.ee.code_words(&sys.board.map, b.paddr, b.words.len()) == *b.words {
                    sys.board.map.watch(b.paddr, 4 * b.words.len() as u32);
                    continue;
                }
                self.slab[i as usize].state = State::Stale;
                self.unlink(i as usize);
                self.stats.killed += 1;
                self.point(self.slab[i as usize].pc, i as usize);
            }
        }
        let mut lines = lines;
        lines.clear();
        sys.board.map.written = lines;
    }

    /// The link a return passed made, where its target has code and its loop is not one the dispatcher must watch.
    fn link(&mut self, slot: *mut u64, at: usize, to: u32) {
        let State::Ready(code) = self.slab[at].state else { return };
        if !self.linkable(self.ctx.from, to) {
            return;
        }
        // SAFETY: the link word of a block in the code memory, passed by that block's way out.
        unsafe {
            self.slab[at].incoming.push((slot, *slot));
            *slot = code;
        }
        self.stats.links += 1;
    }

    fn linkable(&self, from: u32, to: u32) -> bool {
        !self.watch_loops || to > from || from - to > crate::idle::SPAN || self.refused.get(&to).is_some_and(|&n| n >= REFUSALS)
    }

    /// Idle skipping refused the head `pc`; `REFUSALS` times, and its loop is left to run linked.
    pub fn refused(&mut self, pc: u32) {
        let n = self.refused.entry(pc).or_default();
        *n = n.saturating_add(1);
    }

    /// Idle skipping proved the head `pc`'s loop: its refusals count from none.
    pub fn proven(&mut self, pc: u32) {
        self.refused.remove(&pc);
    }

    /// The block compiled code last left, or `pc` where it ran none.
    pub fn last_from(&self, pc: u32) -> u32 {
        if self.ctx.from != 0 { self.ctx.from } else { pc }
    }
}

/// Times in a row idle skipping refuses a loop's head before its backward jump is linked.
const REFUSALS: u8 = 4;

/// One step of the machine, or compiled code to the deadline; the steps taken.
pub fn run(sys: &mut System, blocks: &mut Blocks) -> Result<u64, Halt> {
    if let Some(n) = block(sys, blocks) {
        return Ok(n);
    }
    blocks.ctx.from = 0;
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
    fills: u64,
}

/// The run's machine, during `block`.
fn sys<'a>(ctx: &Context) -> &'a mut System {
    // SAFETY: called by compiled code during `block`, whose `Run` is alive and borrowed by nothing else while its helpers run.
    unsafe { &mut *(*ctx.owner.cast::<Run>()).sys }
}

/// The time compiled code comes back by, Count's interrupt included where it can be taken.
fn deadline(sys: &System) -> u64 {
    let d = sys.board.deadline();
    if sys.ee.count_masked() { d } else { d.min(sys.ee.compare_at()) }
}

fn block(sys: &mut System, blocks: &mut Blocks) -> Option<u64> {
    if sys.board.round == 0 || !isa::supported() {
        return None;
    }
    if std::mem::take(&mut blocks.forgot) {
        sys.board.map.watch_none();
    }
    if blocks.memory.as_ref().is_some_and(|m| m.held() > blocks.budget) {
        blocks.forget();
        sys.board.map.watch_none();
        blocks.forgot = false;
        blocks.ctx.link = 0;
        blocks.memory = None;
        blocks.flushes += 1;
    }
    if !sys.board.map.written.is_empty() {
        blocks.written(sys);
    }
    let pc = sys.ee.pc;
    // The link's target: a step taken since the block returned (idle skipping's proof) has moved the core past it.
    let slot = std::mem::take(&mut blocks.ctx.link) as *mut u64;
    let slot = if blocks.ctx.to == pc { slot } else { std::ptr::null_mut() };
    let found = blocks.find(pc);
    match found.map(|i| &mut blocks.slab[i].state) {
        Some(State::Refused(n)) => {
            *n = n.wrapping_add(1);
            if *n != 0 {
                return None;
            }
        }
        Some(State::Counting(n)) if *n + 1 < blocks.threshold => {
            *n += 1;
            return None;
        }
        _ => {}
    }
    let paddr = sys.ee.block_entry()?;
    let options = blocks.options(sys);
    let fresh = found.is_some_and(|i| {
        let b = &blocks.slab[i];
        b.paddr == paddr && b.options == options && !matches!(b.state, State::Stale)
    });
    let mut at = found;
    if !fresh {
        let words = sys.ee.code_words(&sys.board.map, paddr, blocks.longest);
        let shaped = shape::shape(&words);
        let state = if shaped.is_some() { State::Counting(0) } else { State::Refused(0) };
        let words = shaped.unwrap_or_else(|| words.get(..1).map_or_else(Vec::new, <[u32]>::to_vec));
        if words.is_empty() {
            return None;
        }
        let (i, was) = blocks.put(pc, Block { pc, paddr, words: words.into_boxed_slice(), options, state, incoming: Vec::new() });
        blocks.stats.reshaped += u64::from(was);
        at = Some(i);
    }
    let at = at.expect("the block is in the map");
    let code = match blocks.slab[at].state {
        State::Ready(code) => code,
        State::Refused(_) | State::Stale => return None,
        State::Counting(n) => {
            let n = n + 1;
            if n < blocks.threshold {
                blocks.slab[at].state = State::Counting(n);
                return None;
            }
            let body = blocks.compile(pc, at)?;
            let b = &blocks.slab[at];
            let (paddr, bytes) = (b.paddr, 4 * b.words.len() as u32);
            sys.board.map.watch(paddr, bytes);
            let list = blocks.by_page.entry(paddr >> 12).or_default();
            if !list.contains(&(at as u32)) {
                list.push(at as u32);
            }
            body
        }
    };
    blocks.translation(sys);
    if !slot.is_null() {
        blocks.link(slot, at, pc);
    }
    Some(enter(sys, blocks, code))
}

/// Compiled code from `code` to the deadline; the board brought to its time.
fn enter(sys: &mut System, blocks: &mut Blocks, code: u64) -> u64 {
    let enter_at = blocks.memory.as_ref().expect("code was placed").stub(stub::ENTER);
    // SAFETY: the stub's address, of this signature.
    let enter: Enter = unsafe { std::mem::transmute::<u64, Enter>(enter_at) };
    let mut run = Run { sys, fills: 0 };
    let ctx = &mut *blocks.ctx;
    ctx.owner = (&raw mut run).cast();
    ctx.time = sys.ee.timing.cycles();
    ctx.steps = 0;
    ctx.link = 0;
    ctx.deadline = deadline(sys);
    ctx.vf0 = sys.board.vpu.units.vu0.vf_base();
    ctx.quiet = u32::from(!sys.board.vpu.running(0));
    // SAFETY: the code was compiled for this signature; the core and the context outlive the call, and nothing else
    // borrows the machine while the code and its helpers run through these pointers.
    unsafe { enter(&raw mut (*run.sys).ee, ctx, code) };
    let (t, steps) = (ctx.time, ctx.steps);
    sys.ee.blocks_ran(t, steps);
    sys.ee.count_steps(steps);
    sys.board.run_to(t, &mut sys.ee.spram[..]);
    let (i0, i1, c0) = sys.board.lines();
    (sys.ee.int0, sys.ee.int1, sys.ee.cpcond0) = (i0, i1, c0);
    blocks.stats.entered += 1;
    blocks.stats.compiled_steps += steps;
    blocks.stats.fills += run.fills;
    #[cfg(feature = "census")]
    {
        let e = blocks.census.entry(code as usize).or_default();
        e.0 += 1;
        e.1 += steps;
    }
    for (total, n) in blocks.stats.left.iter_mut().zip(ctx.left.iter_mut()) {
        *total += std::mem::take(n);
    }
    if let Some((Verify::Blocks, v)) = &mut blocks.verify {
        for _ in 0..steps {
            v.reference.step();
            v.steps += 1;
        }
        v.check(sys, "compiled code");
    }
    steps.max(1)
}

/// A slow path, called from compiled code with its pool in `Context::saved` and its frame at `frame`: the guest's dirty values stored to the core, the interpreter's operation run at the instruction's time, and what it wrote put back; nonzero where it stopped and the block leaves.
extern "C" fn slow(ctx: *mut Context, rec: *const Rec, why: u64, frame: *mut u64) -> u64 {
    // SAFETY: the context is the dispatcher's, alive for the call; the record is the block's data, 8-aligned, with the homes at the data's start.
    let (ctx, rec, homes) = unsafe { (&mut *ctx, &*rec, &*rec.cast::<u8>().sub((*rec).at as usize).cast::<Homes>()) };
    ctx.left[why as usize] += 1;
    let System { ee, board } = sys(ctx);
    let value = |ctx: &mut Context, v: usize| -> *mut u64 {
        let h = homes[v] as usize;
        debug_assert!(h != 0, "a value with no home holds nothing");
        // SAFETY: a slot of the block's frame, which is alive under this call.
        if h < 8 { &raw mut ctx.saved[h - 1] } else { unsafe { frame.add(h - 8) } }
    };
    let core = |ee: &mut crate::ee::Ee, v: usize| -> *mut u64 {
        // SAFETY: `core_off` names an 8-byte field of the core, which nothing else borrows while the block runs.
        unsafe { (&raw mut *ee).cast::<u8>().add(lower::core_off(v) as usize).cast::<u64>() }
    };
    for h in 0..2 {
        let mut m = rec.dirty[h];
        while m != 0 {
            let v = 64 * h + m.trailing_zeros() as usize;
            m &= m - 1;
            // SAFETY: as above, both 8 bytes.
            unsafe { core(ee, v).write_unaligned(*value(ctx, v)) };
        }
    }
    let t = ctx.time + u64::from(rec.time);
    let reaches = rec.flags & BOARD != 0;
    ee.begin_call(t, rec.index, rec.word);
    if reaches {
        board.run_to(t, &mut ee.spram[..]);
    }
    let op = crate::ee::jit::run_op(ee, board, rec.index, rec.word, rec.pc);
    match op {
        Ok(()) => {
            ctx.time += ee.call_stall(t);
            for h in 0..2 {
                let mut m = rec.reload[h];
                while m != 0 {
                    let v = 64 * h + m.trailing_zeros() as usize;
                    m &= m - 1;
                    // SAFETY: as above.
                    unsafe { *value(ctx, v) = core(ee, v).read_unaligned() };
                }
            }
            if reaches {
                ctx.quiet = u32::from(!board.vpu.running(0));
                let lines = board.lines();
                let same = lines == (ee.int0, ee.int1, ee.cpcond0);
                let sys = sys(ctx);
                ctx.deadline = if same && sys.board.map.written.is_empty() { deadline(sys) } else { 0 };
            }
            0
        }
        Err(stop) => {
            let slot = (rec.flags & IN_SLOT != 0).then(|| if rec.flags & NEXT_HELD != 0 {
                // SAFETY: as above; the branch wrote `NEXT` before the slot ran.
                unsafe { *value(ctx, NEXT) as u32 }
            } else {
                rec.next
            });
            ee.call_stopped(stop, rec.pc, slot);
            ctx.time += u64::from(rec.time) + 1;
            ctx.steps += u64::from(rec.step) + 1;
            ctx.link = 0;
            ctx.from = 0;
            1
        }
    }
}

/// The translation cache's fill: the page of `vaddr` as a data access in the current mode would place it, cached
/// whatever it is, so that a page the code cannot serve is not asked for again (Sapphire_Recompiler.md §48.3).
extern "C" fn fill(ctx: *mut Context, vaddr: u64, _: u64, _: u64) -> u64 {
    use crate::ee::{Mode, Place};
    // SAFETY: called by compiled code during `block`, whose `Run` and context are alive and borrowed by nothing else.
    let ctx = unsafe { &mut *ctx };
    let run = unsafe { &mut *ctx.owner.cast::<Run>() };
    let vaddr = vaddr as u32;
    let sys = sys(ctx);
    run.fills += 1;
    let (meta, host) = match sys.ee.data_page(vaddr) {
        None => (kind::NONE, 0),
        Some((place, writable)) => {
            let w = if writable { kind::WRITABLE } else { 0 };
            match place {
                Place::Scratchpad(o) => (kind::SCRATCHPAD | w, sys.ee.spram.as_ptr() as u64 + u64::from(o & !0xFFF)),
                Place::Physical(p, Mode::Cached) => (kind::CACHED | w | u64::from(p >> 12) << 32, 0),
                Place::Physical(p, Mode::Uncached | Mode::Accelerated) if (p as usize) < sys.board.map.rdram.len() => (kind::UNCACHED | w, 0),
                Place::Physical(..) => (kind::NONE, 0),
            }
        }
    };
    let i = jit::page_index(vaddr);
    // SAFETY: the pages the context points at are `Blocks::pages`, alive for the call.
    let page = unsafe { &mut *ctx.pages.add(i) };
    *page = Page { tag: ctx.gen1 | u64::from(vaddr & !0xFFF), host, meta, pad: 0 };
    1
}

/// The interpreter's machine the verifier steps (`emusen_jit::verify::Reference`).
pub struct Shadow {
    pub sys: Box<System>,
    pub quiet: bool,
}

/// The first difference between the interpreter's machine `a` and the blocks' `b` in what a program can read: the core's registers, the place, and the memories a block's loads and stores reach; the time is the byte face's own (Sapphire_Speed.md §66).
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
    same!(fpr);
    same!(acc);
    same!(fcr31);
    same!(exceptions);
    if x.in_delay_slot() != y.in_delay_slot() {
        return named("the delay slot", &x.in_delay_slot(), &y.in_delay_slot());
    }
    if let Some(i) = (0..x.spram.len()).find(|&i| x.spram[i] != y.spram[i]) {
        return named(&format!("the scratchpad's byte {i:x}"), &x.spram[i], &y.spram[i]);
    }
    None
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
        let (a, b) = (&self.sys.board.map.rdram, &s.board.map.rdram);
        a.iter().zip(b.iter()).position(|(x, y)| x != y).map(|k| format!("RDRAM's byte {k:x}"))
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
