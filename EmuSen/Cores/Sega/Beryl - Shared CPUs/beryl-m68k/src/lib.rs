//! Beryl's Motorola 68000, written from the M68000 Family Programmer's Reference Manual and the M68000 User's Manual
//! and graded by SingleStepTests' 68000 suite. Shared by the Genesis's main CPU and the Sega CD's sub-CPU, and by any
//! later core with a 68000. The interface is Beryl_M68k.md §2; the oracle §3.
//!
//! Stage 0: the bus, the registers and the step's signature. No instruction executes until stage 1 (`BUILT`).

pub use emusen_native::debug::{Observer, Unobserved};

#[cfg(test)]
mod singlestep;

/// Whether the processor executes; the single-step suite skips while it is false.
pub const BUILT: bool = false;

/// The width of a bus cycle: UDS and LDS both, or one of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    Byte,
    Word,
}

/// FC2-FC0 as the processor drives them during a cycle (M68000UM, "Function Code Outputs").
pub mod function {
    pub const USER_DATA: u8 = 1;
    pub const USER_PROGRAM: u8 = 2;
    pub const SUPERVISOR_DATA: u8 = 5;
    pub const SUPERVISOR_PROGRAM: u8 = 6;
    pub const INTERRUPT_ACKNOWLEDGE: u8 = 7;
}

/// One bus cycle: a 24-bit address (A0 is the byte select of a byte cycle), its width and its function code.
/// `locked` marks TAS's read and write, between which AS stays asserted; the Genesis's bus does not complete the
/// write.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Access {
    pub address: u32,
    pub size: Size,
    pub function: u8,
    pub locked: bool,
}

/// What the processor's pins meet. The bus owns the clock: every cycle and every internal delay is reported, and the
/// bus advances its own time by it, wait states included; the processor keeps no clock of its own.
pub trait Bus {
    /// A read cycle: four clocks and any wait states the bus inserts. A byte cycle's value is in the low byte.
    fn read(&mut self, access: Access) -> u16;
    fn write(&mut self, access: Access, value: u16);
    /// Clocks the processor spends without the bus, in the order the manual places them among the cycles.
    fn idle(&mut self, clocks: u32);
    /// IPL2-IPL0 as sampled now: 0 none, 7 the non-maskable level.
    fn interrupt_level(&mut self) -> u8;
    /// The acknowledge cycle for `level`: the vector number the device places on the bus, or `None` when VPA asks
    /// for the autovector.
    fn acknowledge(&mut self, level: u8) -> Option<u8>;
    /// A word cycle at an odd address, abandoned without AS before the address error exception: its clocks pass
    /// and nothing on the bus sees it.
    fn address_error(&mut self, _access: Access, _write: bool) {
        self.idle(4);
    }
    /// The RESET instruction's pulse to the other devices.
    fn reset_devices(&mut self) {}
}

/// The programmer's model (M68000PRM §1.1-1.3), with the two prefetch words the user's manual names IRC and IRD.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Registers {
    pub d: [u32; 8],
    /// A0-A6, and A7 as the stack pointer of the current mode.
    pub a: [u32; 8],
    /// The stack pointer of the mode not in use.
    pub other_sp: u32,
    pub sr: u16,
    pub pc: u32,
    pub prefetch: [u16; 2],
}

impl Registers {
    pub fn supervisor(&self) -> bool {
        self.sr & 0x2000 != 0
    }

    pub fn usp(&self) -> u32 {
        if self.supervisor() { self.other_sp } else { self.a[7] }
    }

    pub fn ssp(&self) -> u32 {
        if self.supervisor() { self.a[7] } else { self.other_sp }
    }
}

/// What one step did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Instruction,
    /// An exception was taken: its vector number.
    Exception(u8),
    /// STOP waits for an interrupt.
    Stopped,
    /// A double bus fault halted the processor.
    Halted,
}

/// The processor cannot step yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotBuilt;

#[derive(Clone, Debug, Default)]
pub struct M68000 {
    pub regs: Registers,
    pub stopped: bool,
    pub halted: bool,
}

impl M68000 {
    pub fn new() -> M68000 {
        M68000::default()
    }

    /// One instruction, or one exception, through `bus`.
    pub fn step<B: Bus>(&mut self, bus: &mut B) -> Result<Step, NotBuilt> {
        self.step_observed(bus, &mut Unobserved)
    }

    /// The step a debugger watches: `observer.before` at the instruction boundary, stores and calls as they happen.
    pub fn step_observed<B: Bus, O: Observer>(&mut self, _bus: &mut B, _observer: &mut O) -> Result<Step, NotBuilt> {
        Err(NotBuilt)
    }
}
