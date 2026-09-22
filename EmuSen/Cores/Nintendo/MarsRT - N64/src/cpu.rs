//! The VR4300's serialized state, the C# `Cpu` (with COP0, the FPU's registers and COP2's latch).

use crate::state::{State, StateReader, StateResult, StateWriter};
use crate::tlb::Tlb;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Cpu {
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
