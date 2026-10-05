//! Beryl's Zilog Z80, written from Zilog's Z80 CPU User Manual and "The Undocumented Z80 Documented", and graded by
//! SingleStepTests' Z80 suite. The Genesis's sound CPU, and later the Master System's and the Game Gear's. The
//! interface is Beryl_Z80.md §2; the oracle §3.
//!
//! Stage 0: the bus, the registers and the step's signature. No instruction executes until stage 1 (`BUILT`).

pub use emusen_native::debug::{Observer, Unobserved};

#[cfg(test)]
mod singlestep;

/// Whether the processor executes; the single-step suite skips while it is false.
pub const BUILT: bool = false;

/// What the processor's pins meet. The bus owns the clock: each machine cycle reports its T-states through the call
/// that performs it, and the bus adds the wait states WAIT asks for; the processor keeps no clock of its own.
pub trait Bus {
    /// An opcode fetch (M1): the read, then the refresh cycle with `refresh` (I and R) on the address bus.
    fn fetch(&mut self, address: u16, refresh: u16) -> u8;
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
    /// An I/O cycle; the whole 16-bit address, as the processor drives it.
    fn input(&mut self, port: u16) -> u8;
    fn output(&mut self, port: u16, value: u8);
    /// T-states the processor spends without a memory or I/O cycle.
    fn idle(&mut self, t_states: u32);
    /// INT as sampled at the end of an instruction.
    fn int_line(&mut self) -> bool;
    /// Whether NMI has fallen since it was last asked.
    fn nmi_edge(&mut self) -> bool;
    /// The interrupt acknowledge cycle: the byte a device places on the data bus (mode 0's instruction, mode 2's
    /// vector), 0xFF from an open bus.
    fn acknowledge(&mut self) -> u8;
}

/// The registers, the undocumented ones included: WZ (MEMPTR) and Q, which the flags of some instructions read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Registers {
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    pub af_: u16,
    pub bc_: u16,
    pub de_: u16,
    pub hl_: u16,
    pub ix: u16,
    pub iy: u16,
    pub sp: u16,
    pub pc: u16,
    pub i: u8,
    pub r: u8,
    pub wz: u16,
    pub q: u8,
    /// Whether the last instruction was LD A,I or LD A,R, whose P/V an interrupt accepted after it changes.
    pub p: bool,
    pub iff1: bool,
    pub iff2: bool,
    pub im: u8,
    /// EI's one-instruction delay before an interrupt is accepted.
    pub ei_pending: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Instruction,
    /// An interrupt was accepted: its kind, as `emusen_native::debug::kind`.
    Interrupt(u32),
    /// HALT repeats its NOP until an interrupt.
    Halted,
}

/// The processor cannot step yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotBuilt;

#[derive(Clone, Debug, Default)]
pub struct Z80 {
    pub regs: Registers,
    pub halted: bool,
}

impl Z80 {
    pub fn new() -> Z80 {
        Z80::default()
    }

    pub fn step<B: Bus>(&mut self, bus: &mut B) -> Result<Step, NotBuilt> {
        self.step_observed(bus, &mut Unobserved)
    }

    /// The step a debugger watches: `observer.before` at the instruction boundary, stores and calls as they happen.
    pub fn step_observed<B: Bus, O: Observer>(&mut self, _bus: &mut B, _observer: &mut O) -> Result<Step, NotBuilt> {
        Err(NotBuilt)
    }
}
