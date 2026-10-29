//! The EE recompiler's lowering (Sapphire_Plan.md §7, S36 and S37): a block of integer instructions, branches, loads and
//! stores as Cranelift IR, for `emusen-jit`, which owns the compiler's thread, publication, the module and the verifier's
//! shape. A compiled block keeps the guest registers and the issue model's fields in host registers from its entry to its
//! exit, computes each instruction's issue cycle as `timing` does, reaches RDRAM, the scratchpad and the data cache's lines
//! directly through a translation cache, and at its exit hands the cycles to the dispatcher's replay of the board, which
//! decides how many of its instructions stand. Sapphire_Native.md §44 and Sapphire_Recompiler.md §48 are the record.

#[cfg(feature = "baseline")]
pub(crate) mod baseline;
mod emit;
#[cfg_attr(not(feature = "forms"), path = "forms_off.rs")]
pub(crate) mod forms;
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
/// The replay found an interrupt the interpreter would take before the block's end, or the DMAC due between its loads
/// and stores: the interpreter runs the `Context::steps` instructions that stand. When they reach past the last load or
/// store (`Context::base`), the core holds the state after it; otherwise nothing was written but the stores
/// `Context::log` undoes (Sapphire_Recompiler.md §48.5). A load or store's exit returns it too, when every step stands
/// (§48.15).
pub const ROLLBACK: u32 = 1;

/// The entries of the translation cache, by virtual page.
pub const PAGES: usize = 4096;

/// The entry a virtual page takes: its number's low twelve bits mixed with its top eight, so that the windows onto the
/// same memory (kuseg, kseg0, kseg1, the TLB's uncached and accelerated pages) do not take one entry in turn.
pub fn page_index(vaddr: u32) -> usize {
    ((vaddr >> 12 ^ vaddr >> 24) as usize) & (PAGES - 1)
}
/// What a page of the translation cache reaches: nothing fast (the interpreter's path), the scratchpad, the data cache
/// (`Page::meta`'s bits 51..32 the physical page), or RDRAM uncached, for stores.
pub mod kind {
    pub const NONE: u64 = 0;
    pub const SCRATCHPAD: u64 = 1;
    pub const CACHED: u64 = 2;
    pub const UNCACHED: u64 = 3;
    /// With `UNCACHED`: the accelerated pages, whose loads go through the UCAB.
    pub const ACCELERATED: u64 = 8;
    /// A store may be made (the TLB's D bit).
    pub const WRITABLE: u64 = 4;
}
/// An undo entry of 8 bytes, not 16, has this bit set in its address.
pub const SHORT: u64 = 1 << 63;

/// One entry of the translation cache: valid while `tag` is the context's generation with the page's address
/// (`Context::gen1 | vaddr & !0xFFF`); `host` the page's first byte for the scratchpad and RDRAM.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Page {
    pub tag: u64,
    pub host: u64,
    pub meta: u64,
    pub pad: u64,
}

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
    /// The block's instructions that may run: all of them, or those before the first line the cache does not hold; the
    /// block leaves at that line's first instruction.
    pub limit: u32,
    /// The board brought to each instruction's cycles in turn: the steps that stand, all of them unless an interrupt
    /// would be taken first.
    pub replay: Helper,
    /// The verifier's call after each instruction, with everything written back: nonzero when an interrupt is due.
    pub hook: Helper,
    /// A load or store's exit once its block has run a step (`leave_word`): the reason counted, the guard set and the
    /// board replayed through the steps run, which stand unless it stops; the steps that stand, in `steps` too
    /// (Sapphire_Recompiler.md §48.15).
    pub leave: Helper,
    /// The dispatcher's own state, for the helpers.
    pub owner: *mut std::ffi::c_void,
    /// Where the measurement of held registers (`Options::spill`) writes them after each instruction.
    pub spill: [u64; 36],
    /// The translation cache's fill for a page the code did not find: `(context, vaddr) -> 1`.
    pub fill: Helper,
    /// The translation cache (`PAGES` entries), and its generation shifted to bits 63..32 with bit 0 set.
    pub pages: *mut Page,
    pub gen1: u64,
    /// The board's next event at the entry: no load or store runs after an instruction whose cycles reach it.
    pub horizon: u64,
    /// Set by an exit before the replay: the times within which the DMAC may not run (the last load or store's
    /// index), the steps after it (`base`), and the undo entries written.
    pub guard: u32,
    pub base: u32,
    pub logged: u32,
    /// Stores and loads made to RDRAM past the caches, for the board's traffic counts.
    pub uncached: u64,
    pub uncached_loads: u64,
    /// A tag no line has, for the stores that reach no cache line.
    pub dummy: [u64; 2],
    /// Each store's undo: the address of the 16 bytes it changes (or 8, with `SHORT`) and their bytes before it.
    pub log: [[u64; 3]; 2 * shape::MAX],
    /// The loads and stores that left their blocks: at the board's next event, misaligned, on a page the code does not
    /// serve (a register, the boot ROM, a fault), missing the data cache, and a load of an accelerated page (the UCAB).
    pub left: [u64; 5],
    /// A helper's answer when the code calls it through `forms::PRESERVE`'s trampolines, which return nothing.
    pub answer: u32,
    /// `forms::STUB`'s shared exit: the replay, and with every step standing the exit's commit (`commit_word`).
    #[cfg(feature = "forms")]
    pub commit: Commit,
    /// The baseline's entry was not at rest, set by its code for the shared exit's prefix (EmuSen_Recompiler.md §15.2).
    #[cfg(feature = "baseline")]
    pub moved: u32,
    /// The steps whose cycles the prefix computed, for the static share.
    #[cfg(feature = "baseline")]
    pub prefix: u32,
}

/// The shared exit `forms::STUB` calls: `(context, word, place, end) -> steps standing`.
#[cfg(feature = "forms")]
pub type Commit = extern "C" fn(*mut Context, u64, u64, u64) -> u32;

/// The shared exit's word: the steps, the base and log count of the last load or store, the UCAB's flag, the delay slot, and `last` moved on by `k` when relative.
#[cfg(feature = "forms")]
pub const fn commit_word(steps: u32, base: u32, logged: u32, sync: bool, delay: bool, k: Option<u32>) -> u64 {
    assert!(steps < 128 && base < 128 && logged < 256, "a block of at most 64 instructions");
    let relative = match k {
        Some(k) => 1 << 24 | (k as u64) << 32,
        None => 0,
    };
    steps as u64 | (base as u64) << 7 | (logged as u64) << 14 | (sync as u64) << 22 | (delay as u64) << 23 | relative
}

#[cfg(feature = "forms")]
impl Ee {
    /// The shared exit's commit once every step stands: the cycles to `end` (or `last` moved on by the word's `k`), Count, the cycles seen, Cause's IP7, the instructions issued, the UCAB, the place.
    pub(crate) fn commit_exit(&mut self, word: u64, place: u64, end: u64) {
        use super::cop0::{CAUSE, COMPARE, COUNT, cause};
        let steps = word & 0x7F;
        let end = if word >> 24 & 1 != 0 { self.timing.advance_last(word >> 32) } else { end };
        let (count0, compare, seen0) = (u64::from(self.cop0[COUNT]), u64::from(self.cop0[COMPARE]), self.cycles_seen);
        let delta = end.wrapping_sub(seen0);
        self.cop0[COUNT] = count0.wrapping_add(delta) as u32;
        self.cycles_seen = end;
        if compare.wrapping_sub(count0).wrapping_sub(1) & 0xFFFF_FFFF < delta {
            self.cop0[CAUSE] |= cause::IP7;
        }
        self.timing.instructions += steps;
        if word >> 22 & 1 != 0 {
            self.ucab.valid = false;
        }
        (self.pc, self.next_pc, self.delay) = (place as u32, (place >> 32) as u32, word >> 23 & 1 != 0);
    }
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
    /// Loads and stores in blocks (S37); without, every one ends its block as at S36, for measurement.
    pub memory: bool,
    /// Config.NBE clear: a data wait stalls the pipe as well as the result (an uncached load's).
    pub blocking: bool,
}

/// The R5900's lowering, `emit.rs`, for `emusen-jit`.
#[derive(Clone, Copy, Debug, Default)]
pub struct R5900;

impl emusen_jit::Lowering for R5900 {
    type Options = Options;
    const NAME: &'static str = "sapphire-ee";
    const EXITS: &'static [&'static str] = &["done", "rollback"];

    fn lower(&mut self, module: &JITModule, func: &mut ir::Function, functions: &mut FunctionBuilderContext, words: &[u32], start: u32, options: Options) -> Result<(), String> {
        #[cfg(feature = "forms")]
        {
            if std::env::var_os("EMUSEN_SAPPHIRE_FORMS_BYTES").is_some() {
                forms::shadow(module, functions, words, start, options);
            }
            forms::set(forms::chosen() & !forms::PINNED);
        }
        emit::lower(module, func, functions, words, start, options)
    }

    #[cfg(feature = "baseline")]
    fn native(&mut self, words: &[u32], start: u32, options: Options) -> Option<emusen_jit::native::Native> {
        if !baseline::dispatched() {
            return None;
        }
        let code = baseline::compile(words, start, options, false)?;
        // No fallback word: the function only for the forms' byte count, or E1's form under BASELINE_FUNCTION=1 (EmuSen_Recompiler.md §16).
        let function = std::env::var_os("EMUSEN_SAPPHIRE_FORMS_BYTES").is_some() || std::env::var("EMUSEN_SAPPHIRE_BASELINE_FUNCTION").is_ok_and(|v| v == "1");
        Some(emusen_jit::native::Native { code, fallback: None, function })
    }

    #[cfg(feature = "baseline")]
    fn stubs(&self) -> Vec<emusen_assembler::Code> {
        baseline::stubs(crate::blocks::baseline_helpers())
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
/// What a load or store's exit hands `Context::leave`: the steps run, the steps after the last load or store before it
/// (`Context::base`) and the undo entries written, with why it left (`Context::left`'s index) at `WHY_SHIFT`.
pub const fn leave_word(steps: u32, base: u32, logged: u32) -> u32 {
    assert!(steps < 128 && base < 128 && logged < 256, "a block of at most 64 instructions");
    steps | base << 7 | logged << 14
}
pub const WHY_SHIFT: u32 = 24;
/// `leave_word`'s fields and the reason: (steps, base, logged, why).
pub const fn left_by(word: u32) -> (u32, u32, u32, usize) {
    (word & 0x7F, word >> 7 & 0x7F, word >> 14 & 0xFF, (word >> WHY_SHIFT) as usize)
}

impl Ee {
    /// Where a block may be entered now: the physical address of `pc`, when no step boundary check would act there and
    /// the instruction cache serves the fetch; none when the interpreter must take the step (Sapphire_Native.md §44.3).
    /// Makes the interrupt check's write of Cause.IP2 and IP3, as the step would.
    pub(crate) fn block_entry(&mut self) -> Option<u32> {
        use super::cop0::{config, debug, perf};
        use super::memory::{Mode, Place};
        if self.delay || self.pccr & perf::CTE != 0 || self.debug[debug::BPC] & (debug::IAE | debug::DRE | debug::DWE) != 0 || self.timing.instructions == 0 {
            return None;
        }
        let pc = self.pc;
        if pc & 3 != 0 || self.cop0[super::cop0::CONFIG] & config::ICE == 0 || self.interrupt_pending() || self.interrupt_after != 0 {
            return None;
        }
        match self.fetch_place(pc).ok()? {
            Place::Physical(paddr, Mode::Cached | Mode::Through) => Some(paddr),
            _ => None,
        }
    }

    /// Config.NBE clear: blocking loads.
    pub(crate) fn blocking_loads(&self) -> bool {
        self.cop0[super::cop0::CONFIG] & super::cop0::config::NBE == 0
    }

    /// Config.DIE clear: one instruction an issue cycle.
    pub(crate) fn single_issue(&self) -> bool {
        self.cop0[super::cop0::CONFIG] & super::cop0::config::DIE == 0
    }

    /// The words the instruction cache holds from `vaddr` (physical `paddr`) on, to the end of its 4 KiB page or `n`
    /// words, a line it does not hold read from RDRAM (`rdram`), as its refill would read it now; stopping at the first
    /// line it neither holds nor can read so, and whether it stopped there. A block shaped past a line the cache does not
    /// hold runs only to that line until the cache holds it (`held_words`, Sapphire_Recompiler.md §48.4).
    pub(crate) fn cached_words(&self, vaddr: u32, paddr: u32, n: usize, rdram: &[u8]) -> (Vec<u32>, bool) {
        let mut out = Vec::with_capacity(n);
        let mut at = 0u32;
        while out.len() < n {
            let (v, p) = (vaddr + at, paddr + at);
            let set = super::cache::ICache::set(v);
            match self.icache.hit(set, p) {
                Some(way) => out.push(self.icache.data[set][way][(p >> 2) as usize & 15]),
                None => match rdram.get(p as usize..p as usize + 4) {
                    Some(b) => out.push(u32::from_le_bytes(b.try_into().expect("four bytes"))),
                    None => return (out, true),
                },
            }
            at += 4;
            if (vaddr + at) & 0xFFF == 0 {
                break;
            }
        }
        (out, false)
    }

    /// How many of `words`, from `vaddr` (physical `paddr`) on, the instruction cache holds exactly, what the fetches of
    /// the block's instructions would read, each a hit: all of them, or those before the first line it does not hold,
    /// which the interpreter's fetch refills; none when a line it holds differs from the block.
    pub(crate) fn held_words(&self, vaddr: u32, paddr: u32, words: &[u32]) -> Option<usize> {
        let mut k = 0;
        while k < words.len() {
            let (v, p) = (vaddr + 4 * k as u32, paddr + 4 * k as u32);
            let set = super::cache::ICache::set(v);
            let Some(way) = self.icache.hit(set, p) else { return Some(k) };
            let first = (p >> 2) as usize & 15;
            let take = (16 - first).min(words.len() - k);
            if self.icache.data[set][way][first..first + take] != words[k..k + take] {
                return None;
            }
            k += take;
        }
        Some(k)
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

    /// What the translation cache keys its pages by besides the TLB's stamp: the ASID, the operating mode as a data
    /// access sees it (user, supervisor, kernel, or kernel at level 2, where kuseg is unmapped), and kseg0's and the data
    /// cache's configuration.
    pub(crate) fn translation_key(&self) -> [u64; 2] {
        use super::cop0::{CONFIG, ENTRYHI, config};
        use super::status;
        let mode = if self.status() & status::ERL != 0 {
            3
        } else if self.kernel() {
            2
        } else {
            u32::from(self.supervisor())
        };
        let cfg = self.cop0[CONFIG] & (config::K0 | config::DCE);
        [self.tlb_stamp, u64::from(self.cop0[ENTRYHI] & 0xFF) | u64::from(mode) << 8 | u64::from(cfg) << 32]
    }

    /// `n` steps counted in the current mode's count (`modes`), as each step counts itself.
    pub(crate) fn count_steps(&mut self, n: u64) {
        let m = self.mode_index().map_or(4, |m| m as usize);
        self.modes[m] += n;
    }
}
