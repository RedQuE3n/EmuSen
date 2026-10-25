//! The EE core (R5900) interpreter (Sapphire_Plan.md §7, S6 to S9): 128-bit GPRs, the MIPS III subset, the second
//! multiplier, branch-likely, LQ and SQ, the multimedia instructions; COP1, the FPU, on `crate::float`; COP0 in full with the TLB, the caches and the
//! scratchpad, the counters and the breakpoints; each instruction decoded by `emusen_mips::r5900` and executed as the EE Core Instruction Set
//! Manual's Operation sections write it, and charged to the timing model (`timing`). Sapphire_Native.md §16 to §19
//! are the record; the interpreter is the reference later speed work is graded against.

pub mod bus;
pub mod cache;
pub mod cop0;
pub mod fpu;
pub mod map;
mod memory;
mod mmi;
mod ops;
pub mod timing;
pub mod tlb;

pub use bus::Bus;
pub use memory::{Mode, Place};
use cop0::{cause, config, debug, event, perf};
use timing::Timing;

/// The mnemonics this stage executes, in the decoder's spelling.
pub fn names() -> impl Iterator<Item = &'static str> {
    ops::names()
}

/// Status's bits (EE Core User's Manual §3.2, "Status Register").
pub mod status {
    pub const IE: u32 = 1;
    pub const EXL: u32 = 1 << 1;
    pub const ERL: u32 = 1 << 2;
    pub const KSU: u32 = 3 << 3;
    pub const IM2: u32 = 1 << 10;
    pub const IM3: u32 = 1 << 11;
    pub const BEM: u32 = 1 << 12;
    pub const IM7: u32 = 1 << 15;
    pub const EIE: u32 = 1 << 16;
    pub const EDI: u32 = 1 << 17;
    pub const CH: u32 = 1 << 18;
    pub const BEV: u32 = 1 << 22;
    pub const DEV: u32 = 1 << 23;
    pub const CU0: u32 = 1 << 28;
    pub const CU1: u32 = 1 << 29;
}

/// A level 1 exception, with what its code in Cause.ExcCode needs (EE Core User's Manual Table 4-3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exception {
    Interrupt,
    TlbModified(u32),
    TlbRefill { vaddr: u32, store: bool },
    TlbInvalid { vaddr: u32, store: bool },
    AddressLoad(u32),
    AddressStore(u32),
    BusInstruction,
    BusData,
    Syscall,
    Break,
    Reserved,
    Unusable(u32),
    Overflow,
    Trap,
}

impl Exception {
    pub fn code(self) -> u32 {
        match self {
            Exception::Interrupt => 0,
            Exception::TlbModified(_) => 1,
            Exception::TlbRefill { store, .. } | Exception::TlbInvalid { store, .. } => 2 + u32::from(store),
            Exception::AddressLoad(_) => 4,
            Exception::AddressStore(_) => 5,
            Exception::BusInstruction => 6,
            Exception::BusData => 7,
            Exception::Syscall => 8,
            Exception::Break => 9,
            Exception::Reserved => 10,
            Exception::Unusable(_) => 11,
            Exception::Overflow => 12,
            Exception::Trap => 13,
        }
    }

    /// The virtual address BadVAddr takes, for the address error and TLB exceptions (§3.2, "BadVAddr Register").
    fn vaddr(self) -> Option<u32> {
        match self {
            Exception::AddressLoad(v) | Exception::AddressStore(v) | Exception::TlbModified(v) => Some(v),
            Exception::TlbRefill { vaddr, .. } | Exception::TlbInvalid { vaddr, .. } => Some(vaddr),
            _ => None,
        }
    }

    fn tlb(self) -> bool {
        matches!(self, Exception::TlbModified(_) | Exception::TlbRefill { .. } | Exception::TlbInvalid { .. })
    }
}

/// A level 2 exception the core raises itself (§4.1.3): its code in Cause.EXC2 and its vector's offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level2 {
    Counter,
    /// EXC2 3, as the Cause register's table and §4.2.4 give it (Sapphire_Disputes.md D-12).
    Debug,
}

/// Why the interpreter stopped: an instruction a later stage implements (COP2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Halt {
    Unimplemented { pc: u32, word: u32 },
}

/// PRId's implementation number, 2Eh; the revision is not given by the manual (§3.2, "PRId Register").
pub const PRID: u32 = 0x2E00;

pub const SPRAM: usize = 16 << 10;

pub struct Ee {
    pub gpr: [u128; 32],
    pub hi: u128,
    pub lo: u128,
    pub sa: u64,
    /// The FPU's 32 registers, its accumulator and its control and status register (EE Core User's Manual §8.2, §8.3).
    pub fpr: [u32; 32],
    pub acc: u32,
    pub fcr31: u32,
    pub pc: u32,
    next_pc: u32,
    /// Whether the instruction at `pc` is in a delay slot.
    delay: bool,
    pub cop0: [u32; 32],
    /// The debug registers behind COP0 register 24, by the moves' numbers (`cop0::debug`).
    pub debug: [u32; 8],
    pub pccr: u32,
    pub pcr: [u32; 2],
    pub tlb: tlb::Tlb,
    pub dcache: cache::DCache,
    pub icache: cache::ICache,
    pub ucab: cache::Ucab,
    pub btac: cache::Btac,
    pub spram: Box<[u8; SPRAM]>,
    /// CPCOND0, the DMAC's condition line BC0F and BC0T read (driven from S11).
    pub cpcond0: bool,
    /// The interrupt lines Int0 and Int1, Cause.IP2 and IP3 (driven from S11; Sapphire_Disputes.md D-15).
    pub int0: bool,
    pub int1: bool,
    pub timing: Timing,
    memo: Box<[(u32, u16)]>,
    /// This instruction's waits for memory, for the timing model.
    stall_fetch: u64,
    stall_data: u64,
    /// The counting mode of this instruction: the PCCR bit offset of U, S, K or EXL, none at level 2.
    count_mode: Option<u32>,
    /// A load whose address met the data breakpoint, its value to be compared once loaded (§9.2.4).
    watch_load: bool,
    /// A branch-likely nullified its slot.
    nullified: bool,
    /// This instruction wrote Wired, so Random holds 47 past its cycle.
    wired_written: bool,
    /// Instructions in the current issue cycle, and the cycles already given to Count and the counters.
    group: u8,
    cycles_seen: u64,
}

const MEMO: usize = 4096;

impl Ee {
    fn blank(pc: u32) -> Ee {
        let mut cop0 = [0; 32];
        cop0[cop0::PRID] = PRID;
        cop0[cop0::RANDOM] = 47;
        Ee {
            gpr: [0; 32],
            hi: 0,
            lo: 0,
            sa: 0,
            fpr: [0; 32],
            acc: 0,
            fcr31: fpu::FCR31_FIXED,
            pc,
            next_pc: pc.wrapping_add(4),
            delay: false,
            cop0,
            debug: [0; 8],
            pccr: 0,
            pcr: [0; 2],
            tlb: tlb::Tlb::default(),
            dcache: cache::DCache::default(),
            icache: cache::ICache::default(),
            ucab: cache::Ucab::default(),
            btac: cache::Btac::default(),
            spram: Box::new([0; SPRAM]),
            cpcond0: false,
            int0: false,
            int1: false,
            timing: Timing::default(),
            memo: vec![(1, u16::MAX); MEMO].into_boxed_slice(),
            stall_fetch: 0,
            stall_data: 0,
            count_mode: None,
            watch_load: false,
            nullified: false,
            wired_written: false,
            group: 0,
            cycles_seen: 0,
        }
    }

    /// The core after its Reset exception (§4.2.1): at BFC00000h with ERL and BEV set, the caches off and invalid,
    /// Random at 47, Wired 0, the counters and breakpoints disabled; kseg0 uncached, the manual leaving K0 undefined.
    pub fn reset() -> Ee {
        let mut e = Ee::blank(0xBFC0_0000);
        e.cop0[cop0::STATUS] = status::ERL | status::BEV;
        e.cop0[cop0::CONFIG] = config::SIZES | 2;
        e
    }

    /// The core as the project's programs are entered at `pc`: kernel mode with exceptions vectored to RAM, both
    /// caches on and empty, dual issue, kseg0 cached, and the TLB holding the four mappings ps2tek's "Memory Map"
    /// lists, as the referees' kernels leave them (Sapphire_Native.md §17.7).
    pub fn new(pc: u32) -> Ee {
        let mut e = Ee::blank(pc);
        e.cop0[cop0::CONFIG] = config::SIZES | config::DIE | config::ICE | config::DCE | config::NBE | config::BPE | 3;
        let page = |pfn: u32, c: u32| pfn << 6 | c << 3 | 7;
        const MB16: u32 = 0x01FF_E000;
        const MB1: u32 = 0x001F_E000;
        let mut n = 0;
        e.tlb.write(n, MB16, 0x0000_0000, page(0, 3), page(0x1000, 3));
        n += 1;
        e.tlb.write(n, MB16, 0x2000_0000, page(0, 2), page(0x1000, 2));
        n += 1;
        for k in 0..16u32 {
            let even = if k == 0 { 0 } else { page(k * 0x200, 7) };
            e.tlb.write(n, MB1, 0x3000_0000 + k * 0x20_0000, even | 1, page(k * 0x200 + 0x100, 7));
            n += 1;
        }
        e.tlb.write(n, 0, 0x7000_0000, 0x8000_001F, 0x0000_001F);
        n += 1;
        e.cop0[cop0::WIRED] = n as u32;
        e
    }

    pub fn status(&self) -> u32 {
        self.cop0[cop0::STATUS]
    }

    /// Kernel mode: EXL or ERL set, or KSU zero (§3.2's table of operating modes).
    pub fn kernel(&self) -> bool {
        let s = self.status();
        s & (status::EXL | status::ERL) != 0 || s & status::KSU == 0
    }

    fn supervisor(&self) -> bool {
        !self.kernel() && self.status() & status::KSU == 1 << 3
    }

    /// The breakpoint and counter mode: 0 user, 1 supervisor, 2 kernel, 3 the level 1 handler; none at level 2.
    fn mode_index(&self) -> Option<u32> {
        let s = self.status();
        if s & status::ERL != 0 {
            None
        } else if s & status::EXL != 0 {
            Some(3)
        } else {
            match s & status::KSU {
                0 => Some(2),
                0x08 => Some(1),
                0x10 => Some(0),
                _ => None,
            }
        }
    }

    pub(crate) fn set(&mut self, r: usize, v: u64) {
        if r != 0 {
            self.gpr[r] = self.gpr[r] & !(u64::MAX as u128) | v as u128;
        }
    }

    /// A 128-bit write, as the multimedia instructions and LQ make.
    pub(crate) fn set_quad(&mut self, r: usize, v: u128) {
        if r != 0 {
            self.gpr[r] = v;
        }
    }

    pub(crate) fn get(&self, r: usize) -> u64 {
        self.gpr[r] as u64
    }

    pub(crate) fn branch(&mut self, target: u32) {
        self.next_pc = target;
    }

    pub(crate) fn delay_slot(&mut self) {
        self.delay = true;
    }

    /// A branch-likely not taken: the delay slot is nullified.
    pub(crate) fn nullify(&mut self) {
        self.pc = self.next_pc;
        self.next_pc = self.pc.wrapping_add(4);
        self.nullified = true;
    }

    pub(crate) fn jump_now(&mut self, target: u32) {
        self.pc = target;
        self.next_pc = target.wrapping_add(4);
    }

    /// MTC0's write: the writable bits, and the side effects §3.2 names for Wired, Compare and EntryHi.
    pub(crate) fn write_cop0(&mut self, r: usize, v: u32) {
        let m = cop0::writable(r);
        self.cop0[r] = self.cop0[r] & !m | v & m;
        match r {
            cop0::WIRED => {
                self.cop0[cop0::RANDOM] = 47;
                self.wired_written = true;
            }
            cop0::COMPARE => self.cop0[cop0::CAUSE] &= !cause::IP7,
            cop0::ENTRYHI => self.btac.flush(),
            _ => {}
        }
    }

    /// Counts `n` of `e` on each counter selecting it, in the instruction's mode (§7.2.1).
    pub(crate) fn event(&mut self, e: cop0::Event, n: u64) {
        if self.pccr & perf::CTE == 0 || n == 0 {
            return;
        }
        let Some(m) = self.count_mode else { return };
        for (k, sel) in [(0usize, e.0), (1, e.1)] {
            let shift = if k == 0 { 5 } else { 15 };
            let mode_bit = if k == 0 { 4 - m } else { 14 - m };
            if sel == Some(self.pccr >> shift & 31) && self.pccr >> mode_bit & 1 == 1 {
                self.pcr[k] = cop0::add(self.pcr[k], n);
            }
        }
    }

    /// Takes a level 1 exception from the instruction at `pc` (EE Core User's Manual §4.1.2's pseudo-code).
    pub fn raise(&mut self, e: Exception, pc: u32, in_delay: bool) {
        let s = self.cop0[cop0::STATUS];
        let mut c = self.cop0[cop0::CAUSE] & !(0x1F << 2) | e.code() << 2;
        if let Some(v) = e.vaddr() {
            self.cop0[cop0::BADVADDR] = v;
            if e.tlb() {
                let ctx = &mut self.cop0[cop0::CONTEXT];
                *ctx = *ctx & 0xFF80_0000 | (v >> 13) << 4;
                let hi = &mut self.cop0[cop0::ENTRYHI];
                *hi = v & 0xFFFF_E000 | *hi & 0xFF;
            }
        }
        if let Exception::Unusable(n) = e {
            c = c & !(3 << 28) | n << 28;
        }
        let vector = if s & status::EXL == 0 {
            if in_delay {
                self.cop0[cop0::EPC] = pc.wrapping_sub(4);
                c |= cause::BD;
            } else {
                self.cop0[cop0::EPC] = pc;
                c &= !cause::BD;
            }
            self.cop0[cop0::STATUS] = s | status::EXL;
            match e {
                Exception::TlbRefill { .. } => 0x000,
                Exception::Interrupt => 0x200,
                _ => 0x180,
            }
        } else {
            0x180
        };
        self.cop0[cop0::CAUSE] = c;
        let base = if s & status::BEV != 0 { 0xBFC0_0200 } else { 0x8000_0000 };
        self.jump_now(base + vector);
        self.delay = false;
        self.ucab.valid = false;
    }

    /// Takes a level 2 exception (§4.1.3's pseudo-code): Cause.EXC2 and BD2, ErrorEPC, ERL, the vector by DEV.
    pub fn raise2(&mut self, kind: Level2, pc: u32, in_delay: bool) {
        let (code, vector) = match kind {
            Level2::Counter => (2, 0x80),
            Level2::Debug => (3, 0x100),
        };
        let c = self.cop0[cop0::CAUSE] & !(7 << cause::EXC2_SHIFT) & !cause::BD2;
        self.cop0[cop0::CAUSE] = c | code << cause::EXC2_SHIFT | if in_delay { cause::BD2 } else { 0 };
        self.cop0[cop0::ERROREPC] = if in_delay { pc.wrapping_sub(4) } else { pc };
        let s = self.cop0[cop0::STATUS];
        self.cop0[cop0::STATUS] = s | status::ERL;
        let base = if s & status::DEV != 0 { 0xBFC0_0200 } else { 0x8000_0000 };
        self.jump_now(base + vector);
        self.delay = false;
        self.ucab.valid = false;
    }

    /// An interrupt the core takes before its next instruction (§3.2: IM, IP, IE and EIE set, EXL and ERL clear).
    fn interrupt_pending(&mut self) -> bool {
        let c = &mut self.cop0[cop0::CAUSE];
        *c = *c & !(cause::IP2 | cause::IP3) | if self.int0 { cause::IP2 } else { 0 } | if self.int1 { cause::IP3 } else { 0 };
        let s = self.cop0[cop0::STATUS];
        let on = status::IE | status::EIE;
        s & on == on && s & (status::EXL | status::ERL) == 0 && self.cop0[cop0::CAUSE] & s & cause::IP != 0
    }

    /// The performance counter exception's condition (§7.2.4), with ERL clear (Sapphire_Disputes.md D-13).
    fn counter_pending(&self) -> bool {
        self.status() & status::ERL == 0 && self.pccr & perf::CTE != 0 && (self.pcr[0] | self.pcr[1]) & perf::OVFL != 0
    }

    /// The instruction address breakpoint at `pc` (§9.2.2, §9.2.5): sets BPC.IAB, and asks for the exception unless BED.
    fn instruction_break(&mut self, pc: u32) -> bool {
        let bpc = self.debug[debug::BPC];
        let Some(m) = self.mode_index() else { return false };
        if bpc & debug::IAE == 0 || bpc & (debug::IUE >> m) == 0 {
            return false;
        }
        let mask = self.debug[debug::IABM];
        if pc & mask != self.debug[debug::IAB] & mask {
            return false;
        }
        self.debug[debug::BPC] |= debug::IAB_FLAG;
        bpc & debug::BED == 0
    }

    /// The data breakpoint for an access at `vaddr` (§9.2.3 to §9.2.5); a load's value is compared once it is loaded.
    fn data_break(&mut self, vaddr: u32, store: bool, value: Option<u32>) -> Result<(), ops::Stop> {
        let bpc = self.debug[debug::BPC];
        if bpc & if store { debug::DWE } else { debug::DRE } == 0 {
            return Ok(());
        }
        let Some(m) = self.mode_index() else { return Ok(()) };
        let mask = self.debug[debug::DABM];
        if bpc & (debug::DUE >> m) == 0 || vaddr & mask != self.debug[debug::DAB] & mask {
            return Ok(());
        }
        if bpc & debug::DVE != 0 {
            match value {
                None => {
                    self.watch_load = true;
                    return Ok(());
                }
                Some(v) if !self.value_matches(v) => return Ok(()),
                Some(_) => {}
            }
        }
        self.debug[debug::BPC] |= if store { debug::DWB } else { debug::DRB };
        if bpc & debug::BED == 0 { Err(ops::Stop::Debug) } else { Ok(()) }
    }

    fn value_matches(&self, v: u32) -> bool {
        let m = self.debug[debug::DVBM];
        v & m == self.debug[debug::DVB] & m
    }

    /// Executes one instruction, or takes the exception that comes before it.
    pub fn step(&mut self, bus: &mut dyn Bus) -> Result<(), Halt> {
        let pc = self.pc;
        let in_delay = self.delay;
        self.count_mode = self.mode_index();
        self.stall_fetch = 0;
        self.stall_data = 0;
        self.watch_load = false;
        self.nullified = false;
        self.wired_written = false;
        if self.counter_pending() {
            self.raise2(Level2::Counter, pc, in_delay);
            return Ok(());
        }
        if self.interrupt_pending() {
            self.raise(Exception::Interrupt, pc, in_delay);
            return Ok(());
        }
        if self.instruction_break(pc) {
            self.raise2(Level2::Debug, pc, in_delay);
            return Ok(());
        }
        self.delay = false;
        if pc & 3 != 0 {
            self.raise(Exception::AddressLoad(pc), pc, in_delay);
            return Ok(());
        }
        let word = match self.fetch(bus, pc) {
            Ok(w) => w,
            Err(e) => {
                self.raise(e, pc, in_delay);
                return Ok(());
            }
        };
        let entry = self.entry(word);
        let op = ops::get(entry);
        self.pc = self.next_pc;
        self.next_pc = self.pc.wrapping_add(4);
        let mut issue = ops::issue(op, word);
        let rt = (word >> 16 & 31) as usize;
        let before = self.gpr[rt];
        let completed = match (op.run)(self, bus, word, pc) {
            Ok(()) if self.watch_load && self.value_matches(self.gpr[rt] as u32) => {
                self.gpr[rt] = before;
                self.debug[debug::BPC] |= debug::DRB;
                if self.debug[debug::BPC] & debug::BED == 0 {
                    self.raise2(Level2::Debug, pc, in_delay);
                    issue.taken = false;
                    false
                } else {
                    true
                }
            }
            Ok(()) => true,
            Err(ops::Stop::Exception(e)) => {
                self.raise(e, pc, in_delay);
                issue.taken = false;
                matches!(e, Exception::Syscall | Exception::Break | Exception::Trap)
            }
            Err(ops::Stop::Debug) => {
                self.raise2(Level2::Debug, pc, in_delay);
                issue.taken = false;
                false
            }
            Err(ops::Stop::Halt) => {
                self.pc = pc;
                self.next_pc = pc.wrapping_add(4);
                self.delay = in_delay;
                return Err(Halt::Unimplemented { pc, word });
            }
        };
        if self.delay {
            issue.taken = self.next_pc != pc.wrapping_add(8);
        }
        let c = self.cop0[cop0::CONFIG];
        issue.fetch = self.stall_fetch;
        issue.data = self.stall_data;
        issue.blocking = c & config::NBE == 0;
        issue.single = c & config::DIE == 0;
        self.timing.issue(issue);
        if self.pccr & perf::CTE != 0 {
            self.completion_events(bus, op, pc, completed);
        }
        self.advance();
        Ok(())
    }

    /// The events of Table 7-1 an instruction gives by its kind: branches issued, completions, loads and stores.
    fn completion_events(&mut self, bus: &mut dyn Bus, op: &ops::Op, pc: u32, completed: bool) {
        if ops::predicted_branch(op) {
            self.event(event::BRANCH_ISSUED, 1);
            if pc & 4 == 0 {
                self.event(event::LOW_BRANCH, 1);
            }
        }
        let done = |e: &mut Ee, o: &ops::Op| {
            e.event(event::COMPLETED, 1);
            if o.category != timing::Category::Branch {
                e.event(event::NON_BDS, 1);
            }
            if ops::is_load(o) {
                e.event(event::LOAD_COMPLETED, 1);
            } else if ops::is_store(o) {
                e.event(event::STORE_COMPLETED, 1);
            }
        };
        if completed {
            done(self, op);
        }
        if self.nullified
            && let Some(w) = self.peek_fetch(bus, pc.wrapping_add(4))
        {
            let slot = ops::get(self.entry(w));
            done(self, slot);
        }
    }

    /// Gives the cycles this instruction took to Count, Compare's match, the counters and Random (§3.2, §7.2).
    fn advance(&mut self) {
        if !self.timing.paired() {
            if self.group > 0 {
                self.event(if self.group == 1 { event::SINGLE_ISSUE } else { event::DUAL_ISSUE }, 1);
            }
            self.group = 1;
            let wired = self.cop0[cop0::WIRED] & 63;
            let r = self.cop0[cop0::RANDOM];
            if !self.wired_written {
                self.cop0[cop0::RANDOM] = if r <= wired || r > 47 { 47 } else { r - 1 };
            }
        } else {
            self.group = 2;
        }
        let now = self.timing.cycles();
        let delta = now.saturating_sub(self.cycles_seen);
        self.cycles_seen = now;
        let old = self.cop0[cop0::COUNT];
        let compare = self.cop0[cop0::COMPARE];
        if delta > 0 && u64::from(compare.wrapping_sub(old).wrapping_sub(1)) < delta {
            self.cop0[cop0::CAUSE] |= cause::IP7;
        }
        self.cop0[cop0::COUNT] = old.wrapping_add(delta as u32);
        self.event(event::CYCLE, delta);
    }

    /// The instruction table's entry for `word`, through a memo of words seen (the decode is a pure function of the word).
    fn entry(&mut self, word: u32) -> u16 {
        let slot = &mut self.memo[(word ^ word >> 12 ^ word >> 24) as usize % MEMO];
        if slot.0 != word || slot.1 == u16::MAX {
            *slot = (word, ops::index(word));
        }
        slot.1
    }

    /// Runs until `done` returns true or `limit` instructions have run; the instructions run.
    pub fn run(&mut self, bus: &mut dyn Bus, limit: u64, done: &mut dyn FnMut(&Ee, &mut dyn Bus) -> bool) -> Result<u64, Halt> {
        for n in 0..limit {
            if done(self, bus) {
                return Ok(n);
            }
            self.step(bus)?;
        }
        Ok(limit)
    }
}
