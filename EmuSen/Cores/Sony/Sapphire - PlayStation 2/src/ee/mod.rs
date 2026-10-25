//! The EE core (R5900) interpreter, integer part (Sapphire_Plan.md §7, S6): 128-bit GPRs, the MIPS III subset, the
//! second multiplier, branch-likely, LQ and SQ, and the exceptions those raise, each instruction decoded by
//! `emusen_mips::r5900` and executed as the EE Core Instruction Set Manual's Operation sections write it, and charged
//! to the timing model (`timing`). Sapphire_Native.md §16 is the record; the interpreter is the reference later speed
//! work is graded against.

pub mod bus;
mod ops;
pub mod timing;

pub use bus::Bus;
use timing::Timing;

/// The mnemonics this stage executes, in the decoder's spelling.
pub fn names() -> impl Iterator<Item = &'static str> {
    ops::names()
}

/// COP0's register numbers this stage uses (EE Core User's Manual §3.2).
pub mod cop0 {
    pub const BADVADDR: usize = 8;
    pub const STATUS: usize = 12;
    pub const CAUSE: usize = 13;
    pub const EPC: usize = 14;
    pub const PRID: usize = 15;
    pub const ERROREPC: usize = 30;
}

/// Status's bits (EE Core User's Manual §3.2, "Status Register").
pub mod status {
    pub const EXL: u32 = 1 << 1;
    pub const ERL: u32 = 1 << 2;
    pub const KSU: u32 = 3 << 3;
    pub const EIE: u32 = 1 << 16;
    pub const EDI: u32 = 1 << 17;
    pub const BEV: u32 = 1 << 22;
    pub const CU0: u32 = 1 << 28;
}

/// A level 1 exception's code in Cause.ExcCode (EE Core User's Manual Table 4-3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exception {
    AddressLoad(u32),
    AddressStore(u32),
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
            Exception::AddressLoad(_) => 4,
            Exception::AddressStore(_) => 5,
            Exception::Syscall => 8,
            Exception::Break => 9,
            Exception::Reserved => 10,
            Exception::Unusable(_) => 11,
            Exception::Overflow => 12,
            Exception::Trap => 13,
        }
    }
}

/// Why the interpreter stopped: an instruction a later stage implements (COP1, COP2, MMI, CACHE, the TLB).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Halt {
    Unimplemented { pc: u32, word: u32 },
}

/// PRId's implementation number, 2Eh; the revision is not given by the manual (§3.2, "PRId Register").
pub const PRID: u32 = 0x2E00;

pub struct Ee {
    pub gpr: [u128; 32],
    pub hi: u128,
    pub lo: u128,
    pub sa: u64,
    pub pc: u32,
    next_pc: u32,
    /// Whether the instruction at `pc` is in a delay slot.
    delay: bool,
    pub cop0: [u32; 32],
    pub timing: Timing,
    memo: Box<[(u32, u16)]>,
}

const MEMO: usize = 4096;

impl Ee {
    /// A core entering `pc` in kernel mode with exceptions vectored to RAM (BEV clear), as a program is entered.
    pub fn new(pc: u32) -> Ee {
        let mut cop0 = [0; 32];
        cop0[cop0::PRID] = PRID;
        Ee { gpr: [0; 32], hi: 0, lo: 0, sa: 0, pc, next_pc: pc.wrapping_add(4), delay: false, cop0, timing: Timing::default(), memo: vec![(1, u16::MAX); MEMO].into_boxed_slice() }
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

    /// The physical address of `vaddr`, or the address error a mode's access raises (§5.1; §4.2.9).
    pub(crate) fn translate(&self, vaddr: u32, store: bool) -> Result<u32, Exception> {
        let allowed = self.kernel() || vaddr < 0x8000_0000 || (self.supervisor() && (0xC000_0000..0xE000_0000).contains(&vaddr));
        if !allowed {
            return Err(if store { Exception::AddressStore(vaddr) } else { Exception::AddressLoad(vaddr) });
        }
        Ok(bus::physical(vaddr))
    }

    pub(crate) fn set(&mut self, r: usize, v: u64) {
        if r != 0 {
            self.gpr[r] = self.gpr[r] & !(u64::MAX as u128) | v as u128;
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
    }

    pub(crate) fn jump_now(&mut self, target: u32) {
        self.pc = target;
        self.next_pc = target.wrapping_add(4);
    }

    /// Takes a level 1 exception from the instruction at `pc` (EE Core User's Manual §4.1.2's pseudo-code).
    pub fn raise(&mut self, e: Exception, pc: u32, in_delay: bool) {
        let s = self.cop0[cop0::STATUS];
        let mut cause = self.cop0[cop0::CAUSE] & !(0x1F << 2) | e.code() << 2;
        match e {
            Exception::AddressLoad(v) | Exception::AddressStore(v) => self.cop0[cop0::BADVADDR] = v,
            Exception::Unusable(n) => cause = cause & !(3 << 28) | n << 28,
            _ => {}
        }
        if s & status::EXL == 0 {
            if in_delay {
                self.cop0[cop0::EPC] = pc.wrapping_sub(4);
                cause |= 1 << 31;
            } else {
                self.cop0[cop0::EPC] = pc;
                cause &= !(1 << 31);
            }
            self.cop0[cop0::STATUS] = s | status::EXL;
        }
        self.cop0[cop0::CAUSE] = cause;
        let base = if s & status::BEV != 0 { 0xBFC0_0200 } else { 0x8000_0000 };
        self.jump_now(base + 0x180);
        self.delay = false;
    }

    /// Executes one instruction.
    pub fn step(&mut self, bus: &mut dyn Bus) -> Result<(), Halt> {
        let pc = self.pc;
        let in_delay = self.delay;
        self.delay = false;
        if pc & 3 != 0 {
            self.raise(Exception::AddressLoad(pc), pc, in_delay);
            return Ok(());
        }
        let word = match self.translate(pc, false) {
            Ok(p) => bus.read(p, 4) as u32,
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
        match (op.run)(self, bus, word, pc) {
            Ok(()) => {}
            Err(ops::Stop::Exception(e)) => {
                self.raise(e, pc, in_delay);
                issue.taken = false;
            }
            Err(ops::Stop::Halt) => {
                self.pc = pc;
                self.next_pc = pc.wrapping_add(4);
                self.delay = in_delay;
                return Err(Halt::Unimplemented { pc, word });
            }
        }
        if self.delay {
            issue.taken = self.next_pc != pc.wrapping_add(8);
        }
        self.timing.issue(issue);
        Ok(())
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
    pub fn run(&mut self, bus: &mut dyn Bus, limit: u64, done: &mut dyn FnMut(&mut dyn Bus) -> bool) -> Result<u64, Halt> {
        for n in 0..limit {
            if done(bus) {
                return Ok(n);
            }
            self.step(bus)?;
        }
        Ok(limit)
    }
}
