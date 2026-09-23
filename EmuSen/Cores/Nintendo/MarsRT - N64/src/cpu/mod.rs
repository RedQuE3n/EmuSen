//! The VR4300's state, the C# `Cpu` (with COP0, the FPU's registers and COP2's latch), and what C# derives beside it.

pub mod blocks;
pub mod cop0;
pub mod cop1;
pub mod hooks;
pub mod idle;
pub mod interp;
pub mod segments;
pub mod softfloat;
pub mod tlb;

use crate::Skip;
use crate::cpu::segments::Mode;
use crate::state::{State, StateReader, StateResult, StateWriter};
use crate::cpu::tlb::Tlb;

/// `ExceptionCode`, the Cause register's codes.
pub mod code {
    pub const INTERRUPT: u32 = 0;
    pub const TLB_MODIFICATION: u32 = 1;
    pub const TLB_LOAD: u32 = 2;
    pub const TLB_STORE: u32 = 3;
    pub const ADDRESS_ERROR_LOAD: u32 = 4;
    pub const ADDRESS_ERROR_STORE: u32 = 5;
    pub const SYSCALL: u32 = 8;
    pub const BREAKPOINT: u32 = 9;
    pub const RESERVED_INSTRUCTION: u32 = 10;
    pub const COPROCESSOR_UNUSABLE: u32 = 11;
    pub const OVERFLOW: u32 = 12;
    pub const TRAP: u32 = 13;
    pub const FLOATING_POINT: u32 = 15;
}

/// C#'s `CpuException`, recorded where the fault is found; the step then enters it.
#[derive(Clone, Copy, Debug, Default)]
pub struct Fault {
    pub code: u32,
    pub address: u64,
    pub in_delay_slot: bool,
    pub refill: bool,
    pub coprocessor: u32,
}

/// The marker a raising instruction returns; the fault itself is in `CpuRun::fault`.
#[derive(Clone, Copy, Debug)]
pub struct Raised;

pub type Exec<T = ()> = Result<T, Raised>;

/// The C# `Cpu`'s `[SkipInState]` fields that a load rebuilds (`Cop0Written`), and the idle loop's counters.
#[derive(Clone, Copy, Debug)]
pub struct CpuRun {
    pub mode: Mode,
    pub recheck: bool,
    pub asserted_seen: bool,
    pub timer_due: i64,
    pub scheduled_compare: u32,
    pub scheduled_bias: u32,
    pub fault: Fault,
    /// The last address a branch to itself was taken from, where the idle loop is looked for.
    pub idle_at: u64,
    pub idle_turns_passed: i64,
    pub idle_instructions: i64,
    pub rsp_steps: i64,
    /// Moves whenever a translation could have changed, so a mapped fetch remembered from before is not trusted (Mars_Native.md §5.8).
    pub tlb_generation: u32,
    /// Every exception entered: a step that raised is a step, though it is no instruction.
    pub exceptions: i64,
}

impl Default for CpuRun {
    fn default() -> Self {
        CpuRun {
            mode: Mode::Kernel,
            recheck: true,
            asserted_seen: false,
            timer_due: 0,
            scheduled_compare: 0,
            scheduled_bias: 0,
            fault: Fault::default(),
            idle_at: u64::MAX,
            idle_turns_passed: 0,
            idle_instructions: 0,
            rsp_steps: 0,
            tlb_generation: 0,
            exceptions: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Cpu {
    pub run: Skip<CpuRun>,
    pub cop0: [u64; 32],
    pub cop2_latch: u64,
    pub current_pc: u64,
    pub fcsr: u32,
    pub fpr: [u64; 32],
    pub gpr: [u64; 32],
    pub hi: u64,
    pub in_delay_slot: bool,
    pub instructions: i64,
    pub linked_flag: bool,
    pub lo: u64,
    pub next_pc: u64,
    pub pc: u64,
    pub tlb: Tlb,
    pub branch_pending: bool,
    pub cop0_latch: u64,
    pub extra_cycles: i32,
    pub last_count: u32,
    pub random_start: i64,
    /// The debugger's tables and logs, none of it in the state, boxed so the machine's layout keeps its shape (Mars_Native.md §6.5).
    pub hooks: Skip<Box<hooks::Hooks>>,
}

impl State for Cpu {
    fn write_state(&self, w: &mut StateWriter) {
        w.u64s("Cop0", &self.cop0[..]);
        w.u64("Cop2Latch", self.cop2_latch);
        w.u64("CurrentPc", self.current_pc);
        w.u32("Fcsr", self.fcsr);
        w.u64s("Fpr", &self.fpr[..]);
        w.u64s("Gpr", &self.gpr[..]);
        w.u64("Hi", self.hi);
        w.bool("InDelaySlot", self.in_delay_slot);
        w.i64("Instructions", self.instructions);
        w.bool("LinkedFlag", self.linked_flag);
        w.u64("Lo", self.lo);
        w.u64("NextPc", self.next_pc);
        w.u64("Pc", self.pc);
        w.class("Tlb", &self.tlb);
        w.bool("_branchPending", self.branch_pending);
        w.u64("_cop0Latch", self.cop0_latch);
        w.i32("_extraCycles", self.extra_cycles);
        w.u32("_lastCount", self.last_count);
        w.i64("_randomStart", self.random_start);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.u64s(&mut self.cop0[..])?; // Cop0
        self.cop2_latch = r.u64()?; // Cop2Latch
        self.current_pc = r.u64()?; // CurrentPc
        self.fcsr = r.u32()?; // Fcsr
        r.u64s(&mut self.fpr[..])?; // Fpr
        r.u64s(&mut self.gpr[..])?; // Gpr
        self.hi = r.u64()?; // Hi
        self.in_delay_slot = r.bool()?; // InDelaySlot
        self.instructions = r.i64()?; // Instructions
        self.linked_flag = r.bool()?; // LinkedFlag
        self.lo = r.u64()?; // Lo
        self.next_pc = r.u64()?; // NextPc
        self.pc = r.u64()?; // Pc
        r.class(&mut self.tlb)?; // Tlb
        self.branch_pending = r.bool()?; // _branchPending
        self.cop0_latch = r.u64()?; // _cop0Latch
        self.extra_cycles = r.i32()?; // _extraCycles
        self.last_count = r.u32()?; // _lastCount
        self.random_start = r.i64()?; // _randomStart
        Ok(())
    }
}
