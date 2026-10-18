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
    /// The fetch with the word decoded; a bus that keeps decoded words, as a cache's lines, gives its own, which must
    /// be `decode` of the word (BlackMoon_R3000.md §2).
    fn fetch_decoded(&mut self, address: u32) -> (u32, emusen_mips::Op) {
        let w = self.fetch(address);
        (w, emusen_mips::decode(w))
    }
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
    /// Whether `R3000::run` may take another step: nothing the core must do between two steps is due. The default
    /// says no, so a run is one step (BlackMoon_R3000.md §2).
    fn running(&mut self) -> bool {
        false
    }
    /// Whether the bus is told the processor's moves; a constant, so that the step compiled for a bus that is not holds
    /// no trace of them (BlackMoon_R3000.md §14).
    const MOVES: bool = false;
    /// A value the processor moved, for a console that follows values through the machine, when `MOVES` is true: a
    /// load once it has read, a store before it writes, so that a device the store reaches finds the bus already told
    /// (BlackMoon_R3000.md §14). A bus may only watch.
    #[inline(always)]
    fn moved(&mut self, _m: Move) {}
}

/// What `Bus::moved` is told: a general register, coprocessor 2's data register or memory given a value, and from
/// where, or a register given another's shifted; `at` is the instruction's address. R0 is given nothing, so no move names it as the register given a value,
/// and an access the isolated cache takes moves nothing (BlackMoon_R3000.md §14).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    /// LB, LBU, LH, LHU, LW, LWL or LWR: general register `register` will hold `value`, after the next instruction,
    /// from `size` bytes read at `address`.
    Load { at: u32, register: u8, address: u32, size: Size, value: u32 },
    /// SB, SH, SW, SWL or SWR: `value`, `size` bytes of general register `register`, written at `address`.
    Store { at: u32, register: u8, address: u32, size: Size, value: u32 },
    /// MFC2: general register `register` will hold coprocessor 2's data register `cop2`, `value`, after the next
    /// instruction.
    FromCop2 { at: u32, register: u8, cop2: u8, value: u32 },
    /// MTC2: coprocessor 2's data register `cop2` written with general register `register`, `value`.
    ToCop2 { at: u32, register: u8, cop2: u8, value: u32 },
    /// LWC2: coprocessor 2's data register `cop2` written with `value`, read at `address`.
    LoadCop2 { at: u32, cop2: u8, address: u32, value: u32 },
    /// SWC2: coprocessor 2's data register `cop2`, `value`, written at `address`.
    StoreCop2 { at: u32, cop2: u8, address: u32, value: u32 },
    /// An instruction whose result is a general register unchanged: `register` given `from`'s `value` by ADDU, ADD,
    /// SUBU, SUB, OR or XOR with R0, ADDIU, ADDI, ORI or XORI of 0, or a shift by 0.
    Copy { at: u32, register: u8, from: u8, value: u32 },
    /// SLL, SRL or SRA by a constant `amount` of 1 to 31: `register` given `from`'s value shifted, `value` (since the
    /// moves' version 2, BlackMoon_R3000.md §14).
    Shift { at: u32, register: u8, from: u8, kind: Shift, amount: u8, value: u32 },
}

/// A shift's direction: left, right filling with zeros, or right filling with the sign.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shift {
    Left,
    Right,
    Arithmetic,
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
    /// SR and Cause's software bits as the next boundary's interrupt check sees them, when the instruction before it
    /// was an MTC0 to SR (BlackMoon_R3000.md D-17).
    pub interrupt_view: Option<(u32, u32)>,
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

    /// The step a debugger watches: `observer.before` at the instruction boundary, stores, loads and calls as they happen.
    pub fn step_observed<B: Bus, O: Observer>(&mut self, bus: &mut B, observer: &mut O) -> Step {
        exec::step(self, bus, observer)
    }

    /// Up to `limit` steps, each as `step` makes it, ending after the first that is not an instruction or after which
    /// the bus's `running` says no; the steps taken and the last one's result.
    pub fn run<B: Bus>(&mut self, bus: &mut B, limit: u32) -> (u32, Step) {
        exec::run(self, bus, limit)
    }

    /// For a debugger in front of the instruction `word` at the PC: the first address and the count of the bytes it
    /// would load if it ran now, as `Observer::read` would then report them; nothing changed.
    pub fn load_ahead(&self, word: u32) -> Option<(u32, u32)> {
        exec::load_ahead(self, word)
    }
}

/// The R3000A's instructions as `emusen-mips` writes and reads them: coprocessor 0 by this processor's names, and
/// coprocessor 2 by the table its core gives (the GTE's on the PlayStation).
pub fn dialect(cop2: Option<&'static emusen_mips::Coprocessor>) -> emusen_mips::Dialect {
    emusen_mips::Dialect { cop: [Some(&COP0), None, cop2, None], ..emusen_mips::Dialect::MIPS_I }
}
