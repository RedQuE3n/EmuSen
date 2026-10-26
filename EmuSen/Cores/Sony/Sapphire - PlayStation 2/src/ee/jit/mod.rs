//! The EE recompiler's lowering (Sapphire_Plan.md §7, S36): a block of integer instructions and branches as Cranelift IR,
//! for `emusen-jit`, which owns the compiler's thread, publication, the module and the verifier's shape. A compiled block
//! keeps the guest registers and the issue model's fields in host registers from its entry to its exit, computes each
//! instruction's issue cycle as `timing` does, and at its exit hands the cycles to the dispatcher's replay of the board,
//! which decides how many of its instructions stand. Sapphire_Native.md §44 is the record.

mod emit;
pub mod shape;

use cranelift_codegen::ir;
use cranelift_frontend::FunctionBuilderContext;
use cranelift_jit::JITModule;

use super::Ee;

pub use emusen_jit::Slot;

/// A compiled block: `(core, context) -> exit`.
pub type Code = unsafe extern "C" fn(*mut Ee, *mut Context) -> u32;

/// The block ran `Context::steps` instructions and left the core as the interpreter would after them.
pub const DONE: u32 = 0;
/// The replay found an interrupt the interpreter would take before the block's end: nothing was written, and the
/// interpreter runs the `Context::steps` instructions that stand.
pub const ROLLBACK: u32 = 1;

/// A helper compiled code calls: `(context, steps) -> answer`.
pub type Helper = extern "C" fn(*mut Context, u32) -> u32;

/// What a compiled block reads and reports beyond the core.
#[repr(C)]
pub struct Context {
    /// The cycle count after each instruction, `Timing::cycles` as each step leaves it.
    pub times: [u64; shape::MAX],
    /// The board's lines after each instruction's cycles, as the replay found them (INT0, INT1, CPCOND0 in bits 0 to 2).
    pub lines: [u8; shape::MAX],
    /// The steps that stand, on either exit.
    pub steps: u32,
    /// The board brought to each instruction's cycles in turn: the steps that stand, all of them unless an interrupt
    /// would be taken first.
    pub replay: Helper,
    /// The verifier's call after each instruction, with everything written back: nonzero when an interrupt is due.
    pub hook: Helper,
    /// The dispatcher's own state, for the helpers.
    pub owner: *mut std::ffi::c_void,
    /// Where the measurement of held registers (`Options::spill`) writes them after each instruction.
    pub spill: [u64; 36],
}

/// How a block is compiled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Options {
    /// Config.DIE clear: one instruction an issue cycle.
    pub single: bool,
    /// Everything written back and the verifier called after every instruction, the board replayed by its calls.
    pub verify: bool,
    /// Compromise C1 (Sapphire_Plan.md §5.13): a fixed cycle an instruction in place of the issue model. Off by default;
    /// `effect: accuracy`.
    pub fixed_cost: bool,
    /// For measurement only: every register the block has written stored to `Context::spill` after each instruction,
    /// as a block that kept them in memory would (Sapphire_Native.md §44.8). Exact; it changes no value.
    pub spill: bool,
}

/// The R5900's lowering, `emit.rs`, for `emusen-jit`.
#[derive(Clone, Copy, Debug, Default)]
pub struct R5900;

impl emusen_jit::Lowering for R5900 {
    type Options = Options;
    const NAME: &'static str = "sapphire-ee";
    const EXITS: &'static [&'static str] = &["done", "rollback"];

    fn lower(&mut self, module: &JITModule, func: &mut ir::Function, functions: &mut FunctionBuilderContext, words: &[u32], start: u32, options: Options) -> Result<(), String> {
        emit::lower(module, func, functions, words, start, options)
    }
}

/// The dispatcher's compiler.
pub type Jit = emusen_jit::Jit<R5900>;

/// A slot's code, once published.
#[inline(always)]
pub fn published(slot: &Slot) -> Option<Code> {
    // SAFETY: a published address is a finalized function of this signature, published with Release after its pages became executable.
    slot.published().map(|code| unsafe { std::mem::transmute::<*const u8, Code>(code) })
}

/// The flag a hook's step count carries when the step is the block's last.
pub const FINAL: u32 = 1 << 31;

impl Ee {
    /// Where a block may be entered now: the physical address of `pc`, when no step boundary check would act there and
    /// the instruction cache serves the fetch; none when the interpreter must take the step (Sapphire_Native.md §44.3).
    /// Makes the interrupt check's write of Cause.IP2 and IP3, as the step would.
    pub(crate) fn block_entry(&mut self) -> Option<u32> {
        use super::cop0::{config, debug, perf};
        use super::memory::{Access, Mode, Place};
        if self.delay || self.pccr & perf::CTE != 0 || self.debug[debug::BPC] & debug::IAE != 0 || self.timing.instructions == 0 {
            return None;
        }
        let pc = self.pc;
        if pc & 3 != 0 || self.cop0[super::cop0::CONFIG] & config::ICE == 0 || self.interrupt_pending() || self.interrupt_after != 0 {
            return None;
        }
        match self.translate(pc, Access::Fetch).ok()? {
            Place::Physical(paddr, Mode::Cached | Mode::Through) => Some(paddr),
            _ => None,
        }
    }

    /// Config.DIE clear: one instruction an issue cycle.
    pub(crate) fn single_issue(&self) -> bool {
        self.cop0[super::cop0::CONFIG] & super::cop0::config::DIE == 0
    }

    /// The words the instruction cache holds from `vaddr` (physical `paddr`) on, to the end of its 4 KiB page or `n`
    /// words, stopping at the first line it does not hold; and whether it stopped there.
    pub(crate) fn cached_words(&self, vaddr: u32, paddr: u32, n: usize) -> (Vec<u32>, bool) {
        let mut out = Vec::with_capacity(n);
        let mut at = 0u32;
        while out.len() < n {
            let (v, p) = (vaddr + at, paddr + at);
            let set = super::cache::ICache::set(v);
            let Some(way) = self.icache.hit(set, p) else { return (out, true) };
            out.push(self.icache.data[set][way][(p >> 2) as usize & 15]);
            at += 4;
            if (vaddr + at) & 0xFFF == 0 {
                break;
            }
        }
        (out, false)
    }

    /// Whether the instruction cache holds exactly `words` from `vaddr` (physical `paddr`) on, what the fetches of the
    /// block's instructions would read, each a hit; none when a line is missing, which the interpreter's fetch refills.
    pub(crate) fn holds(&self, vaddr: u32, paddr: u32, words: &[u32]) -> Option<bool> {
        let mut k = 0;
        while k < words.len() {
            let (v, p) = (vaddr + 4 * k as u32, paddr + 4 * k as u32);
            let set = super::cache::ICache::set(v);
            let way = self.icache.hit(set, p)?;
            let first = (p >> 2) as usize & 15;
            let take = (16 - first).min(words.len() - k);
            if self.icache.data[set][way][first..first + take] != words[k..k + take] {
                return Some(false);
            }
            k += take;
        }
        Some(true)
    }

    /// The cycles Count last saw, which each step's advance moves to the issue model's count.
    pub(crate) fn cycles_seen(&self) -> u64 {
        self.cycles_seen
    }

    /// Whether the interrupt check before the next step would take an interrupt with Status and Cause as given, the
    /// lines and IP7 replacing Cause's.
    pub(crate) fn interrupt_due(s: u32, cause_now: u32, int0: bool, int1: bool, ip7: bool) -> bool {
        use super::cop0::cause;
        use super::status;
        let on = status::IE | status::EIE;
        if s & on != on || s & (status::EXL | status::ERL) != 0 {
            return false;
        }
        let c = cause_now & !(cause::IP2 | cause::IP3 | cause::IP7)
            | if int0 { cause::IP2 } else { 0 }
            | if int1 { cause::IP3 } else { 0 }
            | if ip7 { cause::IP7 } else { 0 };
        c & s & cause::IP != 0
    }

    /// The interrupt check's write of Cause.IP2 and IP3 and its answer, as a step makes them first.
    pub(crate) fn check_interrupt(&mut self) -> bool {
        self.interrupt_pending()
    }
}
