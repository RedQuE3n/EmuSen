//! A recompiler's shared machinery, generic over a processor's lowering: the compiler's thread and its queue, the slots code is
//! published in, Cranelift's module with its W^X batches and its code budget, the counters, and the verifier's shape. The
//! lowering (a block's words as Cranelift IR), the block shapes and the dispatcher are the processor's. See EmuSen_Jit.md.

pub mod verify;

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

/// The thread: take every job queued, compile them, make their pages executable once, publish each.
fn run<L: Lowering>(mut lowering: L, queue: &Queue<L::Options>, perf_map: bool) {
    let mut module = module();
    let mut context = Context::new();
    let mut functions = FunctionBuilderContext::new();
    let mut map = perf_map.then(|| std::fs::File::create(format!("/tmp/perf-{}.map", std::process::id())).ok()).flatten();
    let mut serial = 0u64;
    loop {
        let jobs = queue.with(std::mem::take);
        if jobs.is_empty() {
            if queue.stop.load(Acquire) {
                break;
            }
            std::thread::park_timeout(std::time::Duration::from_millis(1));
            continue;
        }
        let started = Instant::now();
        let mut done = Vec::with_capacity(jobs.len());
        for job in jobs {
            serial += 1;
            let name = format!("b{:08X}_{serial}", job.start);
            match compile(&mut lowering, &mut module, &mut context, &mut functions, &job, &name) {
                Ok((id, bytes)) => done.push((job, id, bytes, name)),
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
        for (job, id, bytes, name) in done {
            let code = module.get_finalized_function(id);
            if let Some(f) = map.as_mut() {
                use std::io::Write;
                let _ = writeln!(f, "{:x} {:x} {name}", code as usize, bytes);
            }
            queue.counters.bytes.fetch_add(bytes as u64, Relaxed);
            queue.counters.compiled.fetch_add(1, Relaxed);
            job.slot.publish(code as usize);
            queue.counters.finished.fetch_add(1, Release);
        }
        queue.counters.nanos.fetch_add(started.elapsed().as_nanos() as u64, Relaxed);
    }
    // SAFETY: the owner that ran this code is dropping it; nothing will call it again.
    unsafe { module.free_memory() };
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
}

/// A copy keeps the settings and starts with no code.
impl<L: Lowering> Clone for Jit<L> {
    fn clone(&self) -> Self {
        Jit { lowering: self.lowering.clone(), compiler: None, budget: self.budget, perf_map: self.perf_map, flushes: 0 }
    }
}

impl<L: Lowering> Jit<L> {
    pub fn new(lowering: L, budget: u64) -> Jit<L> {
        Jit { lowering, compiler: None, budget, perf_map: false, flushes: 0 }
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
        self.compiler = None;
    }

    /// Code is never freed a block at a time: past the budget the module goes whole. True when it went, and the caller must then
    /// forget every address published before running again.
    pub fn over_budget(&mut self) -> bool {
        if self.counters().is_some_and(|c| c.bytes.load(Relaxed) > self.budget) {
            self.reset();
            self.flushes += 1;
            return true;
        }
        false
    }
}
