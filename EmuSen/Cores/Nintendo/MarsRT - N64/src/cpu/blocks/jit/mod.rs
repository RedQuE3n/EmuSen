//! Steps 2 and 3 of the recompiler: blocks compiled to machine code by Cranelift on a thread of their own, published to
//! the dispatcher through one atomic word each. See Mars_Native.md §5.8.

mod emit;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering::*};
use std::thread::JoinHandle;
use std::time::Instant;

use cranelift_codegen::Context;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::FunctionBuilderContext;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module, default_libcall_names};

use crate::cpu::Cpu;
use crate::memory::bus::MemoryBus;

/// A compiled block: `(cpu, bus, context) -> exit`.
pub type Code = unsafe extern "C" fn(*mut Cpu, *mut MemoryBus, *mut Context_) -> u32;

/// The block ran to its end, or looped back until its guard failed; the machine stands at a step boundary.
pub const DONE: u32 = 0;
/// The instruction `at` raised; the counters and registers are as before it.
pub const RAISED: u32 = 1;
/// The instruction `at` ran and its tick is the dispatcher's, which is the interpreter's own.
pub const FINISH: u32 = 2;

/// What a compiled block reads and reports beyond the machine.
#[repr(C)]
pub struct Context_ {
    /// The virtual address the block was entered at.
    pub entry: u64,
    /// The earliest of the next event, the timer and the caller's cap; the guard kept every instruction short of it.
    pub stop: i64,
    pub rdram: *mut u8,
    pub rdram_len: u64,
    /// `DpInterface`'s read marks, one word a page.
    pub read_marks: *const i64,
    pub write_marks: *const i64,
    /// The instruction an exit names.
    pub at: u32,
    /// The verifier's interpreter, or null.
    pub shadow: *mut crate::cpu::blocks::verify::Shadow,
    /// The bus's write count at entry, less the block's own stores, for the variant beside a running signal processor.
    pub written: i64,
}

/// How a block is compiled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    /// Guest registers held in host registers across the block (step 3).
    pub cached: bool,
    /// A call to the verifier after every instruction.
    pub verify: bool,
    /// Loads and stores of RDRAM made by the code itself (step 3's stores; step 2 inlines the loads).
    pub inline_stores: bool,
    /// The variant for a running signal processor, stepped after every instruction as the tick steps it.
    pub beside: bool,
}

/// Where a block's code is published: zero until it is, `FAILED` if it never will be.
#[derive(Default)]
pub struct Slot {
    pub code: AtomicUsize,
}

struct Job {
    words: Box<[u32]>,
    start: u32,
    options: Options,
    slot: Arc<Slot>,
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

/// The queue between the dispatcher and the compiler's thread: a spin lock of MarsRT's own, so that ThreadSanitizer sees it.
struct Queue {
    locked: AtomicBool,
    jobs: std::cell::UnsafeCell<Vec<Job>>,
    stop: AtomicBool,
    counters: Counters,
}

// SAFETY: `jobs` is touched only while `locked` is held, taken with Acquire and let go with Release.
unsafe impl Sync for Queue {}

impl Queue {
    fn with<R>(&self, f: impl FnOnce(&mut Vec<Job>) -> R) -> R {
        while self.locked.compare_exchange_weak(false, true, Acquire, Relaxed).is_err() {
            std::hint::spin_loop();
        }
        // SAFETY: the lock is held.
        let r = f(unsafe { &mut *self.jobs.get() });
        self.locked.store(false, Release);
        r
    }
}

/// The compiler's thread, started at the first block that needs it and stopped with the machine.
pub struct Compiler {
    queue: Arc<Queue>,
    thread: Option<JoinHandle<()>>,
}

impl Default for Compiler {
    fn default() -> Compiler {
        Compiler::new()
    }
}

impl Compiler {
    pub fn new() -> Compiler {
        let queue = Arc::new(Queue {
            locked: AtomicBool::new(false),
            jobs: std::cell::UnsafeCell::new(Vec::new()),
            stop: AtomicBool::new(false),
            counters: Counters::default(),
        });
        let q = queue.clone();
        let thread = std::thread::Builder::new().name("marsrt-compiler".into()).spawn(move || run(&q)).expect("the compiler's thread");
        Compiler { queue, thread: Some(thread) }
    }

    pub fn counters(&self) -> &Counters {
        &self.queue.counters
    }

    /// Hands a block's words to the thread; its code appears in `slot` when compiled.
    pub fn enqueue(&self, words: Box<[u32]>, start: u32, options: Options, slot: Arc<Slot>) {
        self.queue.counters.queued.fetch_add(1, Relaxed);
        self.queue.with(|jobs| jobs.push(Job { words, start, options, slot }));
        if let Some(t) = &self.thread {
            t.thread().unpark();
        }
    }
}

impl Compiler {
    /// Waits until every block queued so far is published or refused.
    pub fn drain(&self) {
        let c = &self.queue.counters;
        while c.finished.load(Acquire) < c.queued.load(Relaxed) {
            std::thread::yield_now();
        }
    }
}

impl Drop for Compiler {
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

/// The thread: take every job queued, compile them, make their pages executable once, publish each.
fn run(queue: &Queue) {
    let mut module = module();
    let mut context = Context::new();
    let mut functions = FunctionBuilderContext::new();
    let perf_map = std::env::var("EMUSEN_MARSRT_PERFMAP").is_ok_and(|v| v == "1");
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
            match emit::compile(&mut module, &mut context, &mut functions, &job.words, job.start, job.options, &name) {
                Ok((id, bytes)) => done.push((job, id, bytes, name)),
                Err(e) => {
                    queue.counters.failed.fetch_add(1, Relaxed);
                    eprintln!("marsrt: block {:08X} not compiled: {e}", job.start);
                    job.slot.code.store(FAILED, Release);
                    queue.counters.finished.fetch_add(1, Release);
                }
            }
        }
        module.finalize_definitions().expect("the code made executable");
        for (job, id, bytes, name) in done {
            let code = module.get_finalized_function(id);
            if let Some(f) = map.as_mut() {
                use std::io::Write;
                let _ = writeln!(f, "{:x} {:x} {name}", code as usize, bytes);
            }
            queue.counters.bytes.fetch_add(bytes as u64, Relaxed);
            queue.counters.compiled.fetch_add(1, Relaxed);
            job.slot.code.store(code as usize, Release);
            queue.counters.finished.fetch_add(1, Release);
        }
        queue.counters.nanos.fetch_add(started.elapsed().as_nanos() as u64, Relaxed);
    }
    // SAFETY: the machine that ran this code is being dropped; nothing will call it again.
    unsafe { module.free_memory() };
}

/// What a slot holds when its block could not be compiled; it runs decoded for ever.
pub const FAILED: usize = 1;

/// A slot's code, once published.
#[inline(always)]
pub fn published(slot: &Slot) -> Option<Code> {
    let code = slot.code.load(Acquire);
    // SAFETY: a word past `FAILED` is a finalized function of this signature, published with Release after its pages became executable.
    (code > FAILED).then(|| unsafe { std::mem::transmute::<usize, Code>(code) })
}

pub(crate) fn declare(module: &mut JITModule, name: &str, context: &Context) -> Result<cranelift_module::FuncId, String> {
    module.declare_function(name, Linkage::Local, &context.func.signature).map_err(|e| e.to_string())
}
