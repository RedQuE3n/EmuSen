//! Beryl's Motorola 68000, written from the M68000 Family Programmer's Reference Manual and the M68000 User's Manual,
//! with the order of bus cycles from Yacht.txt, and graded by SingleStepTests' 68000 suite and TomHarte's 680x0 tests.
//! Shared by the Genesis's main CPU and the Sega CD's sub-CPU, and by any later core with a 68000. The interface is
//! Beryl_M68k.md §2; the oracle §3; what each step built §4 on.

pub use emusen_native::debug::{Observer, Unobserved};

#[cfg(test)]
mod boundary;
pub mod disasm;
#[cfg(test)]
mod decodemap;
mod exec;
#[cfg(test)]
mod singlestep;
mod state;

/// Whether the processor executes; the single-step suite runs once it is.
pub const BUILT: bool = true;

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
    /// The acknowledge cycle for `level`, in CPU space (FC 7): the vector number the device places on the bus, or
    /// `None` when VPA asks for the autovector. The bus counts the cycle's clocks, four and any waits, or the E clock's
    /// synchronisation for an autovector.
    fn acknowledge(&mut self, level: u8) -> Option<u8>;
    /// A word cycle at an odd address, abandoned before the address error exception: AS without either data strobe,
    /// so nothing transfers, and its four clocks pass (Beryl_M68k.md §6.3).
    fn address_error(&mut self, _access: Access, _write: bool) {
        self.idle(4);
    }
    /// The RESET instruction's pulse to the other devices.
    fn reset_devices(&mut self) {}
}

/// The programmer's model (M68000PRM §1.1-1.3), with the two prefetch words the user's manual names IRD and IRC.
/// Between instructions `pc` is the address of the instruction in `prefetch[0]`, and `prefetch[1]` holds the word
/// after it.
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
    /// The observer stopped the step before the instruction ran: its reasons.
    Observed(u32),
}

#[derive(Clone, Debug, Default)]
pub struct M68000 {
    pub regs: Registers,
    pub stopped: bool,
    pub halted: bool,
    /// The processor number the observer is told, and the space its stores are reported in.
    pub processor: usize,
    pub space: u32,
    /// The interrupt level last sampled, so that a rise to level 7 interrupts even at mask 7.
    pub last_level: u8,
    /// A trace exception owed by the last instruction, and the PC it stacks.
    pub trace_pending: Option<u32>,
}

impl M68000 {
    pub fn new() -> M68000 {
        M68000::default()
    }

    /// One instruction, or one exception, through `bus`.
    pub fn step<B: Bus>(&mut self, bus: &mut B) -> Step {
        self.step_observed(bus, &mut Unobserved)
    }

    /// The step a debugger watches: `observer.before` at the instruction boundary, stores and calls as they happen.
    pub fn step_observed<B: Bus, O: Observer>(&mut self, bus: &mut B, observer: &mut O) -> Step {
        exec::step(self, bus, observer)
    }

    /// The reset exception, as when RESET and HALT are released.
    pub fn reset<B: Bus>(&mut self, bus: &mut B) {
        exec::reset(self, bus)
    }
}
