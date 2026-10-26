//! Steps 2 and 3 of the recompiler: the VR4300's lowering, the context a compiled block reads and the exits it returns. The
//! thread, publication, module and code budget are `emusen-jit`'s. See Mars_Native.md §5.8 and EmuSen_Jit.md.

mod emit;

use cranelift_codegen::ir;
use cranelift_frontend::FunctionBuilderContext;
use cranelift_jit::JITModule;

use crate::cpu::Cpu;
use crate::memory::bus::MemoryBus;

pub use emusen_jit::Slot;

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

/// The VR4300's lowering, `emit.rs`, for `emusen-jit`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Vr4300;

impl emusen_jit::Lowering for Vr4300 {
    type Options = Options;
    const NAME: &'static str = "marsrt";
    const EXITS: &'static [&'static str] = &["done", "raised", "finish"];

    fn lower(&mut self, module: &JITModule, func: &mut ir::Function, functions: &mut FunctionBuilderContext, words: &[u32], start: u32, options: Options) -> Result<(), String> {
        emit::lower(module, func, functions, words, start, options)
    }
}

/// The dispatcher's compiler: started at the first block that needs it, dropped with the machine or past the code's bound.
pub type Jit = emusen_jit::Jit<Vr4300>;

/// A compiler with the code's bound, writing the perf map with `EMUSEN_MARSRT_PERFMAP=1`.
pub fn jit(code_bound: u64) -> Jit {
    let mut jit = Jit::new(Vr4300, code_bound);
    jit.perf_map = std::env::var("EMUSEN_MARSRT_PERFMAP").is_ok_and(|v| v == "1");
    jit
}

/// A slot's code, once published.
#[inline(always)]
pub fn published(slot: &Slot) -> Option<Code> {
    // SAFETY: a published address is a finalized function of this signature, published with Release after its pages became executable.
    slot.published().map(|code| unsafe { std::mem::transmute::<*const u8, Code>(code) })
}
