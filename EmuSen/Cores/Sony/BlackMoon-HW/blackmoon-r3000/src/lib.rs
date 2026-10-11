//! BlackMoon's R3000A, the MIPS I processor of the PlayStation's CPU, written from the MIPS manuals (IDT's R30xx
//! Family Software Reference Manual) and psx-spx, and graded by SingleStepTests' r3000 suite. Later the PlayStation 2's
//! I/O processor. The interface is BlackMoon_R3000.md §2; the oracle §3; the build §5-§8.

pub use emusen_native::debug::{Observer, Unobserved};

#[cfg(test)]
mod boundary;
mod cop0;
mod exec;
mod state;
#[cfg(test)]
mod singlestep;

pub use cop0::{COP0, PRID_PLAYSTATION};

/// Whether the processor executes; the single-step suite's processor test skips while it is false.
pub const BUILT: bool = true;

/// The width of a data access. `Tribyte` is LWL's, LWR's, SWL's or SWR's three bytes, one access within a word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    Byte = 1,
    Half = 2,
    Tribyte = 3,
    Word = 4,
}

impl Size {
    pub fn bytes(self) -> u32 {
        self as u32
    }

    pub(crate) fn of(bytes: u32) -> Size {
        match bytes {
            1 => Size::Byte,
            2 => Size::Half,
            3 => Size::Tribyte,
            _ => Size::Word,
        }
    }
}

/// What the processor's bus meets, after whatever the processor answers itself; where the I-cache and the scratchpad
/// sit is the bus stage's decision (BlackMoon_R3000.md §2). The bus owns the clock: each access and each internal cycle
/// is reported, and the bus advances its time by it, the region's wait states included.
pub trait Bus {
    /// An instruction fetch of one word.
    fn fetch(&mut self, address: u32) -> u32;
    /// A data read of `size` bytes from `address`; the value in the low bits, anything above them ignored.
    fn read(&mut self, address: u32, size: Size) -> u32;
    /// A data write of the low `size` bytes of `value` to `address`. `value` is the register as the processor drives
    /// it, so a byte or halfword store carries the rest of the register above the datum (psx-spx, "CPU Load/Store").
    fn write(&mut self, address: u32, size: Size, value: u32);
    /// A load or store while SR's IsC isolates the cache: it reaches the cache, never memory.
    fn isolated_read(&mut self, address: u32, size: Size) -> u32;
    fn isolated_write(&mut self, address: u32, size: Size, value: u32);
    /// Cycles spent inside the processor with no access: the HI/LO interlock.
    fn idle(&mut self, cycles: u32);
    /// The bus's clock in processor cycles, which the processor only reads, to time its multiplier.
    fn cycles(&mut self) -> u64;
    /// The external interrupt request, as Cause's IP2 sees it.
    fn interrupt(&mut self) -> bool;
    /// Coprocessor 2's data registers (MFC2, MTC2, LWC2, SWC2), its control registers (CFC2, CTC2) and its commands;
    /// the console decides what answers, and when.
    fn cop2_data(&mut self, register: u8) -> u32;
    fn set_cop2_data(&mut self, register: u8, value: u32);
    fn cop2_control(&mut self, register: u8) -> u32;
    fn set_cop2_control(&mut self, register: u8, value: u32);
    fn cop2_command(&mut self, command: u32);
    /// Whether the access just made met no device (psx-spx "Memory Exceptions"): the processor takes a bus error, IBE
    /// after a fetch and DBE after a load or store. A flat bus never faults.
    fn bus_error(&mut self) -> bool {
        false
    }
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
    /// The observer stopped the step before the instruction ran: its reasons.
    Observed(u32),
}

#[derive(Clone, Debug)]
pub struct R3000 {
    pub regs: Registers,
    /// The bus cycle at which the multiplier's result can be read; MFHI and MFLO wait for it.
    pub hilo_ready: u64,
    /// PRId, a constant of the chip.
    pub prid: u32,
    /// The processor number the observer is told, and the space its stores are reported in.
    pub processor: usize,
    pub space: u32,
}

impl Default for R3000 {
    fn default() -> R3000 {
        R3000 { regs: Registers::default(), hilo_ready: 0, prid: PRID_PLAYSTATION, processor: 0, space: 0 }
    }
}

impl R3000 {
    pub fn new() -> R3000 {
        R3000::default()
    }

    /// One instruction through `bus`, or the exception or interrupt taken in its place.
    pub fn step<B: Bus>(&mut self, bus: &mut B) -> Step {
        self.step_observed(bus, &mut Unobserved)
    }

    /// The step a debugger watches: `observer.before` at the instruction boundary, stores and calls as they happen.
    pub fn step_observed<B: Bus, O: Observer>(&mut self, bus: &mut B, observer: &mut O) -> Step {
        exec::step(self, bus, observer)
    }
}

/// The R3000A's instructions as `emusen-mips` writes and reads them: coprocessor 0 by this processor's names, and
/// coprocessor 2 by the table its core gives (the GTE's on the PlayStation).
pub fn dialect(cop2: Option<&'static emusen_mips::Coprocessor>) -> emusen_mips::Dialect {
    emusen_mips::Dialect { cop: [Some(&COP0), None, cop2, None] }
}
