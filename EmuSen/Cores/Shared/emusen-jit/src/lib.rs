//! A recompiler's shared machinery, generic over a processor's lowering: the compiler's thread and its queue, the slots code is
//! published in, Cranelift's module with its W^X batches and its code budget, the counters, and the verifier's shape. The
//! lowering (a block's words as Cranelift IR), the block shapes and the dispatcher are the processor's. See EmuSen_Jit.md.

pub mod verify;
#[cfg(feature = "native")]
pub mod native;

#[cfg(test)]
mod tests;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering::*};
use std::thread::JoinHandle;
use std::time::Instant;

use cranelift_codegen::settings::{self, Configurable};
use cranelift_codegen::{Context, ir};
use cranelift_frontend::FunctionBuilderContext;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module, default_libcall_names};

/// What a processor gives the crate: a block's words turned into a Cranelift function, and the names of the exits it returns.
pub trait Lowering: Clone + Send + 'static {
    /// How a block is compiled, as the processor chooses.
    type Options: Copy + Send + 'static;
    /// The compiler's thread is `<NAME>-compiler`, and what it prints starts `<NAME>:`.
    const NAME: &'static str;
    /// The exits a compiled block returns, by the value it returns.
    const EXITS: &'static [&'static str];
    /// One block as a function: its signature and its body, written into `func`, which the crate has cleared.
    fn lower(&mut self, module: &JITModule, func: &mut ir::Function, functions: &mut FunctionBuilderContext, words: &[u32], start: u32, options: Self::Options) -> Result<(), String>;
    /// Version 1.1, `native`: the block's tailored code, published in place of its function; from 1.2 the function is compiled only when `Native` asks for it. None by default.
    #[cfg(feature = "native")]
    fn native(&mut self, words: &[u32], start: u32, options: Self::Options) -> Option<native::Native> {
        let _ = (words, start, options);
        None
    }
    /// Version 1.1, `native`: the shared stubs the tailored code reaches by id, placed once a code memory.
    #[cfg(feature = "native")]
    fn stubs(&self) -> Vec<emusen_assembler::Code> {
        Vec::new()
    }
}

/// An exit's name, or none for a value the lowering does not name.
pub fn exit_name<L: Lowering>(exit: u32) -> Option<&'static str> {
    L::EXITS.get(exit as usize).copied()
}

/// What a slot holds when its block could not be compiled; the dispatcher runs it another way for ever.
pub const FAILED: usize = 1;

/// Where a block's code is published: zero until it is, `FAILED` if it never will be, else the code's address.
#[derive(Default)]
pub struct Slot {
    code: AtomicUsize,
}

impl Slot {
    /// The code's address, once published: stored with Release after its pages became executable, read here with Acquire.
    #[inline(always)]
    pub fn published(&self) -> Option<*const u8> {
        let code = self.code.load(Acquire);
        (code > FAILED).then_some(code as *const u8)
    }

    /// Whether the compiler is done with it, published or refused.
    #[inline(always)]
    pub fn settled(&self) -> bool {
        self.code.load(Acquire) != 0
    }

    /// Waits until the compiler is done with it; the code, if it was published.
    pub fn wait(&self) -> Option<*const u8> {
        while !self.settled() {
            std::thread::yield_now();
        }
        self.published()
    }

    fn publish(&self, code: usize) {
        self.code.store(code, Release);
    }
}

/// What the compiler's thread did, shared.
#[derive(Default)]
pub struct Counters {
    pub queued: AtomicU64,
    pub finished: AtomicU64,
    pub compiled: AtomicU64,
    pub nanos: AtomicU64,
    pub bytes: AtomicU64,
    pub failed: AtomicU64,
    /// `native`: blocks published as tailored code, and the nanoseconds their lowering took.
    #[cfg(feature = "native")]
    pub native: AtomicU64,
    #[cfg(feature = "native")]
    pub native_nanos: AtomicU64,
    /// `native`, 1.2: tailored blocks with no function that the code memory could not place, refused; the budget then drops the module.
    #[cfg(feature = "native")]
    pub unplaced: AtomicU64,
}

struct Job<O> {
    words: Box<[u32]>,
    start: u32,
    options: O,
    slot: Arc<Slot>,
}

/// The queue between the dispatcher and the compiler's thread: a spin lock of the crate's own, so that ThreadSanitizer sees it.
struct Queue<O> {
    locked: AtomicBool,
    jobs: std::cell::UnsafeCell<Vec<Job<O>>>,
    stop: AtomicBool,
    counters: Counters,
}

// SAFETY: `jobs` is touched only while `locked` is held, taken with Acquire and let go with Release; a job's parts are Send.
unsafe impl<O: Send> Sync for Queue<O> {}

impl<O> Queue<O> {
    fn with<R>(&self, f: impl FnOnce(&mut Vec<Job<O>>) -> R) -> R {
        while self.locked.compare_exchange_weak(false, true, Acquire, Relaxed).is_err() {
            std::hint::spin_loop();
        }
        // SAFETY: the lock is held.
        let r = f(unsafe { &mut *self.jobs.get() });
        self.locked.store(false, Release);
        r
    }
}

/// The compiler's thread and the module it owns; dropping it stops the thread and frees every byte of code it made.
pub struct Compiler<L: Lowering> {
    queue: Arc<Queue<L::Options>>,
    thread: Option<JoinHandle<()>>,
}

impl<L: Lowering> Compiler<L> {
    /// Starts the thread; with `perf_map`, each function's address, size and name go to `/tmp/perf-<pid>.map` for `perf`.
    pub fn new(lowering: L, perf_map: bool) -> Compiler<L> {
        let queue = Arc::new(Queue { locked: AtomicBool::new(false), jobs: std::cell::UnsafeCell::new(Vec::new()), stop: AtomicBool::new(false), counters: Counters::default() });
        let q = queue.clone();
        let thread = std::thread::Builder::new().name(format!("{}-compiler", L::NAME)).spawn(move || run(lowering, &q, perf_map)).expect("the compiler's thread");
        Compiler { queue, thread: Some(thread) }
    }

    pub fn counters(&self) -> &Counters {
        &self.queue.counters
    }

    /// Hands a block's words to the thread; its code appears in `slot` when compiled.
    pub fn enqueue(&self, words: Box<[u32]>, start: u32, options: L::Options, slot: Arc<Slot>) {
        self.queue.counters.queued.fetch_add(1, Relaxed);
        self.queue.with(|jobs| jobs.push(Job { words, start, options, slot }));
        if let Some(t) = &self.thread {
            t.thread().unpark();
        }
    }

    /// Waits until every block queued so far is published or refused.
    pub fn drain(&self) {
        let c = &self.queue.counters;
        while c.finished.load(Acquire) < c.queued.load(Relaxed) {
            std::thread::yield_now();
        }
    }
}

impl<L: Lowering> Compiler<L> {
    /// Stops the thread without waiting for it: it refuses what is queued, finishes the block in hand and frees its code
    /// itself. Only for an owner that has forgotten every address the module published (EmuSen_Jit.md §3.4).
    fn retire(mut self) {
        self.queue.stop.store(true, Release);
        if let Some(t) = self.thread.take() {
            t.thread().unpark();
        }
    }
}

impl<L: Lowering> Drop for Compiler<L> {
    fn drop(&mut self) {
        self.queue.stop.store(true, Release);
        if let Some(t) = self.thread.take() {
            t.thread().unpark();
            let _ = t.join();
        }
    }
}

fn module() -> JITModule {
    let mut flags = settings::builder();
    flags.set("opt_level", "speed").unwrap();
    let isa = cranelift_native::builder().expect("a host Cranelift targets").finish(settings::Flags::new(flags)).expect("the host's flags");
    JITModule::new(JITBuilder::with_isa(isa, default_libcall_names()))
}

/// One block lowered, declared and defined; its function and the bytes of its code.
fn compile<L: Lowering>(lowering: &mut L, module: &mut JITModule, context: &mut Context, functions: &mut FunctionBuilderContext, job: &Job<L::Options>, name: &str) -> Result<(cranelift_module::FuncId, usize), String> {
    module.clear_context(context);
    lowering.lower(module, &mut context.func, functions, &job.words, job.start, job.options)?;
    let id = module.declare_function(name, Linkage::Local, &context.func.signature).map_err(|e| e.to_string())?;
    module.define_function(id, context).map_err(|e| format!("{e:?}"))?;
    Ok((id, context.compiled_code().map_or(0, |c| c.code_info().total_size as usize)))
}

/// The perf map every compiler thread of this process appends to, one whole line a write (Sapphire_Disputes.md D-1102).
fn perf_map_path() -> String {
    format!("/tmp/perf-{}.map", std::process::id())
}

/// The thread: take every job queued, compile them, make their pages executable once, publish each.
fn run<L: Lowering>(mut lowering: L, queue: &Queue<L::Options>, perf_map: bool) {
    let mut module = module();
    let mut context = Context::new();
    let mut functions = FunctionBuilderContext::new();
    let mut map = perf_map.then(|| std::fs::OpenOptions::new().create(true).append(true).open(perf_map_path()).ok()).flatten();
    let mut serial = 0u64;
    #[cfg(feature = "native")]
    let mut memory: Option<native::Memory> = None;
    loop {
        let jobs = queue.with(std::mem::take);
        if queue.stop.load(Acquire) {
            refuse(queue, jobs);
            break;
        }
        if jobs.is_empty() {
            std::thread::park_timeout(std::time::Duration::from_millis(1));
            continue;
        }
        let started = Instant::now();
        let mut done = Vec::with_capacity(jobs.len());
        let mut jobs = jobs.into_iter();
        while let Some(job) = jobs.next() {
            if queue.stop.load(Acquire) {
                refuse(queue, std::iter::once(job).chain(jobs).collect());
                break;
            }
            serial += 1;
            let name = format!("b{:08X}_{serial}", job.start);
            #[cfg(feature = "native")]
            let tailored = {
                let t = Instant::now();
                let n = lowering.native(&job.words, job.start, job.options);
                if n.is_some() {
                    queue.counters.native_nanos.fetch_add(t.elapsed().as_nanos() as u64, Relaxed);
                }
                n
            };
            #[cfg(feature = "native")]
            let function = tailored.as_ref().is_none_or(native::Native::needs_function);
            #[cfg(not(feature = "native"))]
            let function = true;
            if !function {
                done.push(Done { job, function: None, name, #[cfg(feature = "native")] tailored });
                continue;
            }
            match compile(&mut lowering, &mut module, &mut context, &mut functions, &job, &name) {
                Ok(function) => done.push(Done { job, function: Some(function), name, #[cfg(feature = "native")] tailored }),
                Err(e) => {
                    queue.counters.failed.fetch_add(1, Relaxed);
                    eprintln!("{}: block {:08X} not compiled: {e}", L::NAME, job.start);
                    job.slot.publish(FAILED);
                    queue.counters.finished.fetch_add(1, Release);
                }
            }
        }
        // W^X: the batch was written into fresh pages, made readable and executable here in one call; they are never writable again.
        module.finalize_definitions().expect("the code made executable");
        #[cfg(feature = "native")]
        let mut placed = {
            let had: Vec<Option<(usize, u64)>> = done.iter().map(|d| d.tailored.as_ref().map(|n| (n.code.hot.len(), n.bytes()))).collect();
            let natives: Vec<(native::Native, u64)> = done.iter_mut().filter_map(|d| d.tailored.take().map(|n| (n, d.function.map_or(0, |f| module.get_finalized_function(f.0) as u64)))).collect();
            let entries = if natives.is_empty() { Vec::new() } else { memory.get_or_insert_with(native::Memory::new).place(&|| lowering.stubs(), natives) };
            let mut it = entries.into_iter();
            had.into_iter().map(|h| h.and_then(|(hot, bytes)| it.next().flatten().map(|e| (e, hot, bytes)))).collect::<Vec<_>>().into_iter()
        };
        for Done { job, function, name, .. } in done {
            #[cfg(feature = "native")]
            let placed = placed.next().flatten();
            #[cfg(not(feature = "native"))]
            let placed: Option<(u64, usize, u64)> = None;
            let (code, size, bytes) = match (placed, function) {
                (Some((entry, hot, bytes)), f) => {
                    #[cfg(feature = "native")]
                    queue.counters.native.fetch_add(1, Relaxed);
                    (entry as *const u8, hot, bytes + f.map_or(0, |f| f.1 as u64))
                }
                (None, Some((id, size))) => (module.get_finalized_function(id), size, size as u64),
                (None, None) => {
                    #[cfg(feature = "native")]
                    queue.counters.unplaced.fetch_add(1, Relaxed);
                    queue.counters.failed.fetch_add(1, Relaxed);
                    job.slot.publish(FAILED);
                    queue.counters.finished.fetch_add(1, Release);
                    continue;
                }
            };
            if let Some(f) = map.as_mut() {
                use std::io::Write;
                let _ = f.write_all(format!("{:x} {:x} {name}\n", code as usize, size).as_bytes());
            }
            queue.counters.bytes.fetch_add(bytes, Relaxed);
            queue.counters.compiled.fetch_add(1, Relaxed);
            job.slot.publish(code as usize);
            queue.counters.finished.fetch_add(1, Release);
        }
        queue.counters.nanos.fetch_add(started.elapsed().as_nanos() as u64, Relaxed);
    }
    // SAFETY: the owner that ran this code is dropping it; nothing will call it again.
    unsafe { module.free_memory() };
}

/// A job lowered in a batch: its Cranelift function and that code's size, if it has one, and its tailored code.
struct Done<O> {
    job: Job<O>,
    function: Option<(cranelift_module::FuncId, usize)>,
    name: String,
    #[cfg(feature = "native")]
    tailored: Option<native::Native>,
}

/// Jobs a stopped thread will not compile: settled as refused, so that no `Slot::wait` waits on them for ever.
fn refuse<O>(queue: &Queue<O>, jobs: Vec<Job<O>>) {
    for job in jobs {
        job.slot.publish(FAILED);
        queue.counters.finished.fetch_add(1, Release);
    }
}

/// 1.2: what compiling has cost: blocks published, nanoseconds spent compiling, and bytes of code made.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Spent {
    pub compiled: u64,
    pub nanos: u64,
    pub bytes: u64,
}

impl Spent {
    fn add(&mut self, o: Spent) {
        (self.compiled, self.nanos, self.bytes) = (self.compiled + o.compiled, self.nanos + o.nanos, self.bytes + o.bytes);
    }

    fn of(c: &Counters) -> Spent {
        Spent { compiled: c.compiled.load(Relaxed), nanos: c.nanos.load(Relaxed), bytes: c.bytes.load(Relaxed) }
    }
}

/// A processor's compiler as its dispatcher holds it: started at the first block it is given, dropped past its code budget.
pub struct Jit<L: Lowering> {
    lowering: L,
    compiler: Option<Compiler<L>>,
    /// Bytes of code past which `over_budget` drops the module and every byte of code with it.
    pub budget: u64,
    /// Each compiled function written to the perf map, as `Compiler::new` says.
    pub perf_map: bool,
    /// Times the budget dropped the module.
    pub flushes: u64,
    /// 1.2: of those, the drops for a full code memory under the budget.
    pub full: u64,
    /// 1.2: what the compiler's threads already dropped spent, then what `compile_now` spent on the caller's thread, before the last drop and since.
    dropped: [Spent; 2],
    now: Spent,
    /// 1.2, `native`: the code memory `compile_now` places in, dropped with the module; and whether it could not place a block.
    #[cfg(feature = "native")]
    memory: Option<native::Memory>,
    #[cfg(feature = "native")]
    unplaced: bool,
}

/// A copy keeps the settings and starts with no code.
impl<L: Lowering> Clone for Jit<L> {
    fn clone(&self) -> Self {
        Jit::with(self.lowering.clone(), self.budget, self.perf_map)
    }
}

impl<L: Lowering> Jit<L> {
    pub fn new(lowering: L, budget: u64) -> Jit<L> {
        Jit::with(lowering, budget, false)
    }

    fn with(lowering: L, budget: u64, perf_map: bool) -> Jit<L> {
        Jit {
            lowering,
            compiler: None,
            budget,
            perf_map,
            flushes: 0,
            full: 0,
            dropped: [Spent::default(); 2],
            now: Spent::default(),
            #[cfg(feature = "native")]
            memory: None,
            #[cfg(feature = "native")]
            unplaced: false,
        }
    }

    /// 1.2: what compiling has cost since this `Jit` was made, across its drops: on the compiler's threads, and on the caller's by `compile_now`.
    pub fn spent(&self) -> [Spent; 2] {
        let mut threads = self.dropped[0];
        if let Some(c) = self.counters() {
            threads.add(Spent::of(c));
        }
        let mut here = self.dropped[1];
        here.add(self.now);
        [threads, here]
    }

    /// Bytes of code held now: the compiler's and `compile_now`'s.
    pub fn held(&self) -> u64 {
        self.counters().map_or(0, |c| c.bytes.load(Relaxed)) + self.now.bytes
    }

    /// 1.2, `native`: a block's tailored code made and published on the caller's thread, in the `Jit`'s own code memory,
    /// with no Cranelift function; none, and nothing done, when the lowering has no tailored code for it or the code
    /// needs its function, and the caller then enqueues it. A slot refused where the memory could not place the code.
    #[cfg(feature = "native")]
    pub fn compile_now(&mut self, words: &[u32], start: u32, options: L::Options) -> Option<Arc<Slot>> {
        let t = Instant::now();
        let n = self.lowering.native(words, start, options).filter(|n| !n.needs_function())?;
        let bytes = n.bytes();
        let Jit { lowering, memory, .. } = self;
        let entry = memory.get_or_insert_with(native::Memory::new).place(&|| lowering.stubs(), vec![(n, 0)]).pop().flatten();
        let slot = Arc::new(Slot::default());
        match entry {
            Some(e) => {
                self.now.compiled += 1;
                self.now.bytes += bytes;
                slot.publish(e as usize);
            }
            None => {
                self.unplaced = true;
                slot.publish(FAILED);
            }
        }
        self.now.nanos += t.elapsed().as_nanos() as u64;
        Some(slot)
    }

    /// The module dropped, its thread retired without a join, and what it spent kept.
    fn drop_module(&mut self) {
        if let Some(c) = self.compiler.take() {
            self.dropped[0].add(Spent::of(c.counters()));
            c.retire();
        }
        let now = std::mem::take(&mut self.now);
        self.dropped[1].add(now);
        #[cfg(feature = "native")]
        {
            self.memory = None;
            self.unplaced = false;
        }
    }

    /// Hands a block to the compiler, starting its thread if none runs; the slot its code will appear in.
    pub fn enqueue(&mut self, words: Box<[u32]>, start: u32, options: L::Options) -> Arc<Slot> {
        let slot = Arc::new(Slot::default());
        let (lowering, perf_map) = (&self.lowering, self.perf_map);
        self.compiler.get_or_insert_with(|| Compiler::new(lowering.clone(), perf_map)).enqueue(words, start, options, slot.clone());
        slot
    }

    /// The running compiler's counters, or none before the first block and after a drop.
    pub fn counters(&self) -> Option<&Counters> {
        self.compiler.as_ref().map(Compiler::counters)
    }

    /// Waits for every block queued to be published or refused.
    pub fn drain(&self) {
        if let Some(c) = &self.compiler {
            c.drain();
        }
    }

    /// The thread stopped and every byte of code freed: each address published so far is dead, and its holder must forget it.
    pub fn reset(&mut self) {
        if let Some(c) = self.compiler.take() {
            self.dropped[0].add(Spent::of(c.counters()));
        }
        self.drop_module();
    }

    /// Code is never freed a block at a time: past the budget the module goes whole, its thread retired without a join (it
    /// frees the code once it has finished the block in hand). True when it went, and the caller must then forget every
    /// address published before running again.
    /// From 1.2, `native`: also when a tailored block with no function could not be placed, its code memory full.
    pub fn over_budget(&mut self) -> bool {
        #[cfg(feature = "native")]
        let full = self.unplaced || self.counters().is_some_and(|c| c.unplaced.load(Relaxed) > 0);
        #[cfg(not(feature = "native"))]
        let full = false;
        if self.held() > self.budget || full {
            self.full += u64::from(self.held() <= self.budget);
            self.drop_module();
            self.flushes += 1;
            return true;
        }
        false
    }
}
