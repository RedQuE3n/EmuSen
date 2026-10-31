//! The EE's recompiled tier (Sapphire_Speed.md §66): a block of EE code as x86-64, running to its branch with a cost folded when it is compiled, one add and compare at each end, and blocks linked directly behind that compare. Events land at blocks' ends; a hardware read sees the block's start plus the steps before it. Nothing is undone: a faulting access cancels its instruction and leaves the block with the work before it kept. `lower.rs` writes the code on `emusen-assembler`, from `emusen-mips-planner`'s plan; `crate::blocks` dispatches.

pub(crate) mod isa;
pub(crate) mod lower;
pub mod shape;

use super::Ee;
use super::map::Map;

/// The block run at the third argument: `(core, context, body)`.
pub type Enter = unsafe extern "C" fn(*mut Ee, *mut Context, u64);

/// The entries of the translation cache, by virtual page.
pub const PAGES: usize = 4096;

/// The entries of the table of blocks, by the entry's word address.
pub const FRONT: usize = 1 << 16;

/// The entry a virtual page takes: its number's low twelve bits mixed with its top eight, so that the windows onto the
/// same memory (kuseg, kseg0, kseg1, the TLB's uncached and accelerated pages) do not take one entry in turn.
pub fn page_index(vaddr: u32) -> usize {
    ((vaddr >> 12 ^ vaddr >> 24) as usize) & (PAGES - 1)
}
/// What a page of the translation cache reaches: nothing fast (the interpreter's path), the scratchpad, the data cache
/// (`Page::meta`'s bits 51..32 the physical page), or RDRAM uncached, which the interpreter's path takes.
pub mod kind {
    pub const NONE: u64 = 0;
    pub const SCRATCHPAD: u64 = 1;
    pub const CACHED: u64 = 2;
    pub const UNCACHED: u64 = 3;
    /// A store may be made (the TLB's D bit).
    pub const WRITABLE: u64 = 4;
}

/// One entry of the translation cache: valid while `tag` is the context's generation with the page's address
/// (`Context::gen1 | vaddr & !0xFFF`); `host` the page's first byte for the scratchpad.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Page {
    pub tag: u64,
    pub host: u64,
    pub meta: u64,
    pub pad: u64,
}

/// One entry of the table of blocks: the block at `pc`, `code` the way to it, or the stub that returns to the dispatcher while it has no code.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Jump {
    pub pc: u32,
    /// One past the block's index, 0 for an empty entry.
    pub block: u32,
    pub code: u64,
}

/// What a compiled block reads and reports beyond the core.
#[repr(C)]
pub struct Context {
    /// A block's end at or past this time returns to the dispatcher.
    pub deadline: u64,
    /// The time: the EE's cycles at the block's entry, moved on at each end, here while a helper runs.
    pub time: u64,
    /// Instructions run since the dispatcher entered a block.
    pub steps: u64,
    /// The translation cache (`PAGES` entries), and its generation shifted to bits 63..32 with bit 0 set.
    pub pages: *mut Page,
    pub gen1: u64,
    /// The table of blocks (`FRONT` entries).
    pub front: *mut Jump,
    /// VU0's VF registers, for LQC2 and SQC2; and whether VU0 rests, so that they may run in place.
    pub vf0: *mut u8,
    pub quiet: u32,
    /// The block a return came from.
    pub from: u32,
    /// The place a return left the core at, and the link's word it passed, 0 for none.
    pub to: u32,
    pub pad: u32,
    pub link: u64,
    /// The dispatcher's own state, for the helpers.
    pub owner: *mut std::ffi::c_void,
    /// A tag no line has, for the stores that reach no cache line.
    pub dummy: [u64; 2],
    /// The pool's registers while a slow path runs.
    pub saved: [u64; 7],
    /// Slow paths taken: a call, misaligned, a page the code does not serve, a data cache miss, uncached.
    pub left: [u64; 5],
}

/// The interpreter's operation `index` on the word at `pc`.
pub(crate) fn run_op(ee: &mut Ee, bus: &mut dyn super::Bus, index: u16, word: u32, pc: u32) -> Result<(), super::ops::Stop> {
    (super::ops::get(index).run)(ee, bus, word, pc)
}

/// How a block's cost is folded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Options {
    /// Config.DIE clear: one instruction an issue cycle.
    pub single: bool,
    /// Config.NBE clear: a data wait stalls the pipe as well as the result.
    pub blocking: bool,
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

    /// The words of memory from `paddr` on, as a refill would read them, to the end of its 4 KiB page or `n` words; none outside RDRAM and the boot ROM.
    pub(crate) fn code_words(&self, map: &Map, paddr: u32, n: usize) -> Vec<u32> {
        let room = ((0x1000 - (paddr & 0xFFF)) / 4) as usize;
        (0..room.min(n)).map_while(|k| map.code_word(paddr + 4 * k as u32)).collect()
    }

    /// The cycles Count last saw.
    pub(crate) fn cycles_seen(&self) -> u64 {
        self.cycles_seen
    }

    /// The instruction a helper is about to run stands at time `t`: the issue model at rest there, its stalls clear.
    pub(crate) fn begin_call(&mut self, t: u64, index: u16, word: u32) {
        self.timing.rest_at(t);
        (self.current, self.current_word, self.stall_until, self.stall_data, self.stall_fetch) = (index, word, 0, 0, 0);
    }

    /// The cycles the call's instruction stalled past time `t`.
    pub(crate) fn call_stall(&self, t: u64) -> u64 {
        self.stall_until.saturating_sub(t) + u64::from(self.stall_data)
    }

    /// A call's instruction stopped: its exception raised as a step raises it, or the core left before it where it halts.
    pub(crate) fn call_stopped(&mut self, stop: super::ops::Stop, pc: u32, slot: Option<u32>) {
        use super::ops::Stop;
        use super::Level2;
        let in_delay = slot.is_some();
        match stop {
            Stop::Exception(e) => self.raise(e, pc, in_delay),
            Stop::Debug => self.raise2(Level2::Debug, pc, in_delay),
            Stop::Halt => {
                (self.pc, self.next_pc, self.delay) = (pc, slot.unwrap_or(pc.wrapping_add(4)), in_delay);
            }
        }
    }

    /// Blocks ran to time `t`, `steps` instructions: the issue model at rest there, and Count, Compare's match, Random and the counts given what they took.
    pub(crate) fn blocks_ran(&mut self, t: u64, steps: u64) {
        use super::cop0::{CAUSE, COMPARE, COUNT, RANDOM, WIRED, cause};
        let (count0, compare, seen0) = (u64::from(self.cop0[COUNT]), u64::from(self.cop0[COMPARE]), self.cycles_seen);
        let delta = t.wrapping_sub(seen0);
        self.cop0[COUNT] = count0.wrapping_add(delta) as u32;
        self.cycles_seen = t;
        if compare.wrapping_sub(count0).wrapping_sub(1) & 0xFFFF_FFFF < delta {
            self.cop0[CAUSE] |= cause::IP7;
        }
        self.timing.rest_at(t);
        self.timing.instructions += steps;
        let wired = u64::from(self.cop0[WIRED] & 63).min(47);
        let span = 48 - wired;
        let r = u64::from(self.cop0[RANDOM]);
        let from = if r < wired || r > 47 { 0 } else { 47 - r };
        self.cop0[RANDOM] = (47 - (from + steps) % span) as u32;
        self.group = 1;
    }

    /// The time Count next reaches Compare, after the cycles it has seen.
    pub(crate) fn compare_at(&self) -> u64 {
        use super::cop0::{COMPARE, COUNT};
        self.cycles_seen + u64::from(self.cop0[COMPARE].wrapping_sub(self.cop0[COUNT]).wrapping_sub(1)) + 1
    }

    /// Whether Count's interrupt can be taken as Status stands.
    pub(crate) fn count_masked(&self) -> bool {
        self.cop0[super::cop0::STATUS] & super::cop0::cause::IP7 == 0
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
