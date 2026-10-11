//! BlackMoon's R3000A, the MIPS I processor of the PlayStation's CPU, written from the MIPS manuals (IDT's R30xx
//! Family Software Reference Manual) and psx-spx, and graded by SingleStepTests' r3000 suite. Later the PlayStation 2's
//! I/O processor. The interface is BlackMoon_R3000.md §2; the oracle §3.
//!
//! Stage 1: the bus, the registers with both delay slots, the state block and the step's signature. No instruction
//! executes until Diamond's stage 2 (`BUILT`).

pub use emusen_native::debug::{Observer, Unobserved};

mod state;
#[cfg(test)]
mod singlestep;

/// Whether the processor executes; the single-step suite's processor test skips while it is false.
pub const BUILT: bool = false;

/// The width of a data access.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    Byte = 1,
    Half = 2,
    Word = 4,
}

impl Size {
    pub fn bytes(self) -> u32 {
        self as u32
    }
}

/// What the processor's bus meets, after whatever the processor answers itself; where the I-cache and the scratchpad
/// sit is the bus stage's decision (BlackMoon_R3000.md §2). The bus owns the clock: each access and each internal cycle
/// is reported, and the bus advances its time by it, the region's wait states included.
pub trait Bus {
    /// An instruction fetch of one word.
    fn fetch(&mut self, address: u32) -> u32;
    /// A data read; the value in the low bits.
    fn read(&mut self, address: u32, size: Size) -> u32;
    fn write(&mut self, address: u32, size: Size, value: u32);
    /// Cycles spent inside the processor with no access: interlocks on HI/LO and on coprocessor 2.
    fn idle(&mut self, cycles: u32);
    /// The external interrupt request, as Cause's IP2 sees it.
    fn interrupt(&mut self) -> bool;
    /// Coprocessor 2's data registers (MFC2, MTC2, LWC2, SWC2), its control registers (CFC2, CTC2) and its commands;
    /// the console decides what answers, and when.
    fn cop2_data(&mut self, register: u8) -> u32;
    fn set_cop2_data(&mut self, register: u8, value: u32);
    fn cop2_control(&mut self, register: u8) -> u32;
    fn set_cop2_control(&mut self, register: u8, value: u32);
    fn cop2_command(&mut self, command: u32);
}

/// The branch delay: whether the next instruction is a branch's delay slot, whether the branch is taken, and where to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BranchDelay {
    pub slot: bool,
    pub taken: bool,
    pub target: u32,
}

/// The load delay: the register a load will write after the next instruction, and the value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LoadDelay {
    pub register: Option<u8>,
    pub value: u32,
}

/// The general registers, HI and LO, the PC, coprocessor 0's registers (psx-spx "COP0 Register Summary") and the
/// two delay slots, which are architectural on the R3000A.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Registers {
    pub r: [u32; 32],
    pub hi: u32,
    pub lo: u32,
    pub pc: u32,
    pub sr: u32,
    pub cause: u32,
    pub epc: u32,
    pub bad_vaddr: u32,
    pub tar: u32,
    pub dcic: u32,
    pub bpc: u32,
    pub bda: u32,
    pub bpcm: u32,
    pub bdam: u32,
    pub branch: BranchDelay,
    pub load: LoadDelay,
}

/// The registers a debugger lists, in `debug_registers`' order, each 32 bits: the general registers by their
/// conventional names (psx-spx "CPU Registers"), HI, LO and the PC, then coprocessor 0's.
pub const DEBUG_REGISTERS: [&str; 45] = [
    "zero", "at", "v0", "v1", "a0", "a1", "a2", "a3", "t0", "t1", "t2", "t3", "t4", "t5", "t6", "t7", "s0", "s1", "s2", "s3", "s4", "s5",
    "s6", "s7", "t8", "t9", "k0", "k1", "gp", "sp", "fp", "ra", "hi", "lo", "pc", "SR", "CAUSE", "EPC", "BadVaddr", "TAR", "DCIC", "BPC",
    "BDA", "BPCM", "BDAM",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Instruction,
    /// An exception or interrupt was taken: its Cause code.
    Exception(u8),
}

/// The processor cannot step yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotBuilt;

#[derive(Clone, Debug, Default)]
pub struct R3000 {
    pub regs: Registers,
}

impl R3000 {
    pub fn new() -> R3000 {
        R3000::default()
    }

    pub fn step<B: Bus>(&mut self, bus: &mut B) -> Result<Step, NotBuilt> {
        self.step_observed(bus, &mut Unobserved)
    }

    /// The step a debugger watches: `observer.before` at the instruction boundary, stores and calls as they happen.
    pub fn step_observed<B: Bus, O: Observer>(&mut self, _bus: &mut B, _observer: &mut O) -> Result<Step, NotBuilt> {
        Err(NotBuilt)
    }
}
