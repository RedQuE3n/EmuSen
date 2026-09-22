//! The recompiler: blocks of the VR4300's code validated against memory on entry and run between the interpreter's checks.
//! C#'s `Cpu.Blocks.cs` and `Cpu/Blocks/`; see Mars_Recompiler.md and Mars_Native.md §5.8.

pub mod cache;
pub mod decoded;
pub mod ops;
pub mod shape;
pub mod verify;

use crate::cpu::Cpu;
use crate::cpu::cop0::ENTRY_HI;
use crate::cpu::interp::{KERNEL_DIRECT_BASE, KERNEL_DIRECT_SIZE};
use crate::cpu::segments::{self, Mode, Segment};
use crate::cpu::tlb::TlbResult;
use crate::memory::bus::MemoryBus;
use crate::memory::dp_threads::site;
use crate::memory::ram::Ram;
use cache::Cache;
use decoded::Op;
use shape::MAX_WORDS;
use verify::Shadow;

/// How far the recompiler goes: each tier runs everything the one before could not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    /// Step 1: every block a loop over its decoded instructions.
    Decoded,
}

impl Tier {
    /// The furthest tier built.
    pub const BEST: Tier = Tier::Decoded;

    /// 1 is step 1 and so on; 0, or a number past the last, is the furthest built.
    pub fn from_number(n: u32) -> Tier {
        match n {
            1 => Tier::Decoded,
            _ => Tier::BEST,
        }
    }

    fn from_env() -> Tier {
        match std::env::var("EMUSEN_MARSRT_TIER").as_deref() {
            Ok("decoded") => Tier::Decoded,
            _ => Tier::BEST,
        }
    }
}

/// A run of consecutive words, the bytes it was shaped from, and what it runs as.
pub struct Block {
    pub start: u32,
    pub words: u32,
    pub refused: bool,
    pub image: Box<[u8]>,
    pub ops: Box<[Op]>,
    /// The most cycles its instructions tick, stalls included.
    pub max_cycles: i64,
    pub runs: u32,
}

impl Block {
    #[inline(always)]
    pub fn end(&self) -> u32 {
        self.start + self.words * 4
    }

    /// The comparison on entry: the words in memory are the words the block was made from (Mars_Recompiler.md §2.3).
    #[inline(always)]
    fn matches(&self, rdram: &Ram) -> bool {
        rdram[self.start as usize..self.end() as usize] == self.image[..]
    }

    fn shaped(rdram: &Ram, start: u32, limit: u32) -> Block {
        let word = |n: u32| rdram.be32(start + 4 * n);
        let s = shape::shape(word, limit);
        let ops: Box<[Op]> = (0..s.words).map(|n| Op { word: word(n), run: ops::handler(word(n)) }).collect();
        Block {
            start,
            words: s.words,
            refused: s.refused,
            image: rdram[start as usize..(start + 4 * s.words) as usize].into(),
            max_cycles: ops.iter().map(|op| shape::cycles(op.word)).sum(),
            ops,
            runs: 0,
        }
    }
}

/// What the recompiler did, for the harness and the page.
#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    /// Blocks shaped, and those shaped again because the words under them changed.
    pub shaped: u64,
    pub discarded: u64,
    /// Dispatches that ran a block, and the instructions they ran.
    pub entries: u64,
    pub instructions: u64,
    /// Dispatches left to one interpreter step: a delay slot, another mode, a refused or page-crossing block, code outside RDRAM.
    pub stepped: u64,
    /// Entries through the TLB.
    pub mapped: u64,
}

/// The last page a mapped fetch was translated through, trusted while the translation's generation stands (Mars_Recompiler.md §17).
#[derive(Clone, Copy, Default)]
struct Fetch {
    generation: u32,
    page: u64,
    frame: u32,
    valid: bool,
}

/// The recompiler's state beside a machine: none of it in the save state, all of it rebuilt from memory on demand.
pub struct Blocks {
    pub on: bool,
    pub tier: Tier,
    /// Blocks reached through the TLB; off only to measure them.
    pub mapped: bool,
    /// The interpreter run beside the blocks and compared after every instruction; on in debug builds with `EMUSEN_MARSRT_VERIFY_BLOCKS=1`.
    pub verify: bool,
    cache: Cache,
    fetch: Fetch,
    shadow: Option<Box<Shadow>>,
    pub stats: Stats,
}

impl Default for Blocks {
    fn default() -> Self {
        Blocks {
            on: false,
            tier: Tier::from_env(),
            mapped: std::env::var("EMUSEN_MARSRT_NOMAPPEDBLOCKS").map_or(true, |v| v != "1"),
            verify: Blocks::verify_by_default(),
            cache: Cache::default(),
            fetch: Fetch::default(),
            shadow: None,
            stats: Stats::default(),
        }
    }
}

/// A copy of a machine starts with no blocks, and blocks are shaped again from its memory; the settings go with it.
impl Clone for Blocks {
    fn clone(&self) -> Self {
        Blocks { on: self.on, tier: self.tier, mapped: self.mapped, verify: self.verify, ..Blocks::default() }
    }
}

impl std::fmt::Debug for Blocks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Blocks {{ on: {}, tier: {:?}, live: {} }}", self.on, self.tier, self.cache.live)
    }
}

impl Blocks {
    /// The verifier's default: `EMUSEN_MARSRT_VERIFY_BLOCKS=1`.
    pub fn verify_by_default() -> bool {
        std::env::var("EMUSEN_MARSRT_VERIFY_BLOCKS").is_ok_and(|v| v == "1")
    }

    /// Live blocks, shaped, discarded, entries, instructions in blocks, interpreter steps, mapped entries, and four the
    /// compiled tiers fill: blocks compiled, nanoseconds compiling, bytes of code, and entries that ran compiled code.
    pub fn counters(&self) -> Vec<i64> {
        let s = &self.stats;
        vec![self.cache.live as i64, s.shaped as i64, s.discarded as i64, s.entries as i64, s.instructions as i64, s.stepped as i64, s.mapped as i64, 0, 0, 0, 0]
    }

    /// Every block forgotten, as a machine of another RDRAM size needs.
    pub fn clear(&mut self) {
        self.cache.clear();
        self.fetch.valid = false;
    }

    pub fn live(&self) -> usize {
        self.cache.live
    }

    /// Kernel mode's view of a mapped program counter, through the page last translated when nothing has moved since.
    fn mapped(&mut self, cpu: &Cpu, pc: u64) -> Option<u32> {
        if !self.mapped || cpu.wide_addressing() {
            return None;
        }
        let page = pc & !0xFFF;
        let f = self.fetch;
        if f.valid && f.page == page && f.generation == cpu.run.tlb_generation {
            return Some(f.frame | (pc as u32 & 0xFFF));
        }
        if segments::decode(pc, Mode::Kernel, false) != Segment::Mapped {
            return None;
        }
        let TlbResult::Mapped(physical) = cpu.tlb.try_translate(pc, cpu.cop0[ENTRY_HI], false) else { return None };
        self.fetch = Fetch { generation: cpu.run.tlb_generation, page, frame: physical & !0xFFF, valid: true };
        Some(physical)
    }

    /// One block where one can run, or one interpreter step where none can (Mars_Recompiler.md §3.1).
    pub fn step(&mut self, cpu: &mut Cpu, bus: &mut MemoryBus, cap_at: i64, fields: i64) {
        let pc = cpu.pc;
        if cpu.branch_pending || cpu.run.mode != Mode::Kernel || pc & 3 != 0 || cpu.extra_cycles != 0 {
            return self.interpret(cpu, bus);
        }
        let direct = pc.wrapping_sub(KERNEL_DIRECT_BASE) < KERNEL_DIRECT_SIZE;
        let physical = if direct {
            (pc as u32) & 0x1FFF_FFFF
        } else {
            match self.mapped(cpu, pc) {
                Some(p) => p,
                None => return self.interpret(cpu, bus),
            }
        };
        let size = bus.rdram.len() as u32;
        if physical >= size {
            return self.interpret(cpu, bus);
        }

        // A mapped block keeps to its page, since the next virtual page may be any frame (Mars_Recompiler.md §17).
        let limit = if direct { (size - physical) / 4 } else { (0x1000 - (physical & 0xFFF)) / 4 };
        let reach = limit.min(MAX_WORDS) * 4;
        if bus.dp.read_marked(physical) || bus.dp.read_marked(physical + reach - 1) {
            bus.dp.wait_read_range(physical, reach, site::BLOCK);
        }

        let Blocks { cache, stats, shadow, .. } = self;
        let block = match cache.get_mut(physical) {
            Some(b) if b.matches(&bus.rdram) => b,
            found => {
                stats.discarded += found.is_some() as u64;
                stats.shaped += 1;
                cache.place(Block::shaped(&bus.rdram, physical, limit))
            }
        };
        if block.refused || (!direct && (physical & 0xFFF) + block.words * 4 > 0x1000) {
            return self.interpret(cpu, bus);
        }
        block.runs = block.runs.saturating_add(1);

        // The step's own opening: the address, the slot flag, and the interrupt check, whose raise is entered as the step enters it.
        cpu.current_pc = pc;
        cpu.in_delay_slot = false;
        let asserted = bus.mi.asserted();
        if (cpu.run.recheck || asserted != cpu.run.asserted_seen) && cpu.check_interrupts(asserted).is_err() {
            cpu.enter_exception();
            bus.tick(1);
            if let Some(s) = shadow.as_deref_mut() {
                s.follow(cpu, bus, "an interrupt at a block's entry");
            }
            return;
        }

        stats.entries += 1;
        stats.mapped += !direct as u64;
        let before = cpu.instructions;
        decoded::run(cpu, bus, block, pc, cap_at, fields, shadow.as_deref_mut());
        stats.instructions += (cpu.instructions - before) as u64;
    }

    #[inline(always)]
    fn interpret(&mut self, cpu: &mut Cpu, bus: &mut MemoryBus) {
        self.stats.stepped += 1;
        cpu.step(bus);
        if let Some(s) = self.shadow.as_deref_mut() {
            s.follow(cpu, bus, "an interpreter step");
        }
    }

    /// The frame's loop, `RunQuietly` with the idle skip, through the blocks; with the verifier on, an interpreter beside it.
    pub fn run_frame(&mut self, cpu: &mut Cpu, bus: &mut MemoryBus, cap_at: i64, idle_skip: bool, rsp_whole: bool) {
        let fields = bus.vi.fields;
        self.begin(cpu, bus);
        while bus.vi.fields == fields && bus.cycles < cap_at {
            if idle_skip && cpu.pc == cpu.run.idle_at && cpu.try_idle(bus, cap_at, rsp_whole) {
                if let Some(s) = self.shadow.as_deref_mut() {
                    assert!(s.idle(cap_at, rsp_whole), "the interpreter's machine was not at the idle loop the blocks' was");
                    s.check(cpu, bus, "the idle loop");
                }
                continue;
            }
            self.step(cpu, bus, cap_at, fields);
        }
        self.end(cpu, bus);
    }

    /// Exactly `steps` of the interpreter's steps, a raise counting as one: a block may run only as many cycles as steps remain, and each costs one at least (Mars_Recompiler.md §6).
    pub fn run_steps(&mut self, cpu: &mut Cpu, bus: &mut MemoryBus, steps: u64) {
        self.begin(cpu, bus);
        let taken = |c: &Cpu| (c.instructions + c.run.exceptions) as u64;
        let end = taken(cpu) + steps;
        while taken(cpu) < end {
            let cap_at = bus.cycles + (end - taken(cpu)) as i64;
            self.step(cpu, bus, cap_at, bus.vi.fields);
        }
        self.end(cpu, bus);
    }

    fn begin(&mut self, cpu: &Cpu, bus: &MemoryBus) {
        self.shadow = (self.verify && bus.dp.threads.is_none()).then(|| Shadow::of(cpu, bus));
    }

    fn end(&mut self, cpu: &Cpu, bus: &MemoryBus) {
        if let Some(s) = self.shadow.take() {
            s.check_all(cpu, bus);
        }
    }
}
