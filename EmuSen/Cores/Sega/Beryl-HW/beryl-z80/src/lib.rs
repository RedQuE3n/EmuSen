//! Beryl's Zilog Z80, written from Zilog's Z80 CPU User Manual and "The Undocumented Z80 Documented", and graded by
//! SingleStepTests' Z80 suite, ZEXDOC and ZEXALL. The Genesis's sound CPU, and later the Master System's and the Game
//! Gear's. The interface is Beryl_Z80.md §2; the oracle §3; the build §4-§6.

pub use emusen_native::debug::{Observer, Unobserved};

#[cfg(feature = "asm")]
pub mod asm;
#[cfg(test)]
mod boundary;
#[cfg(test)]
mod cpm;
pub mod disasm;
mod exec;
mod state;
#[cfg(test)]
mod singlestep;

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
    /// The interrupt acknowledge cycle, an M1 with IORQ and its two automatic wait states, six T-states the bus counts:
    /// the byte a device places on the data bus (mode 0's instruction, mode 2's vector), 0xFF from an open bus.
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

/// The registers a debugger lists, by name and width in bits, in the order `Registers::values` gives them: Nephrite's
/// Z80 and Endou's read the one list (Beryl_Z80.md §9).
pub const REGISTERS: [(&str, u32); 18] = [
    ("AF", 16), ("BC", 16), ("DE", 16), ("HL", 16), ("AF'", 16), ("BC'", 16), ("DE'", 16), ("HL'", 16),
    ("IX", 16), ("IY", 16), ("SP", 16), ("PC", 16), ("I", 8), ("R", 8), ("WZ", 16), ("IM", 2), ("IFF1", 1), ("IFF2", 1),
];

impl Registers {
    /// The values of `REGISTERS`, in its order.
    pub fn values(&self) -> [i64; REGISTERS.len()] {
        [
            self.af, self.bc, self.de, self.hl, self.af_, self.bc_, self.de_, self.hl_, self.ix, self.iy, self.sp, self.pc,
            self.i as u16, self.r as u16, self.wz, self.im as u16, self.iff1 as u16, self.iff2 as u16,
        ]
        .map(i64::from)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Instruction,
    /// An interrupt was accepted: its kind, as `emusen_native::debug::kind`.
    Interrupt(u32),
    /// HALT repeats its NOP until an interrupt.
    Halted,
    /// The observer stopped the step before the instruction ran: its reasons.
    Observed(u32),
}

#[derive(Clone, Debug, Default)]
pub struct Z80 {
    pub regs: Registers,
    pub halted: bool,
    /// The processor number the observer is told, and the space its stores are reported in.
    pub processor: usize,
    pub space: u32,
}

impl Z80 {
    pub fn new() -> Z80 {
        Z80::default()
    }

    /// The processor after its RESET pin: PC, I, R, IFF1, IFF2 and the interrupt mode cleared (Zilog's UM0080, "Reset"),
    /// and every other register $FFFF, AF and SP as measured on Z80s and the rest as "The Undocumented Z80 Documented"
    /// §2.4 advises an emulator to set the registers it calls undefined.
    pub fn power_on() -> Z80 {
        let f = 0xFFFF;
        let regs = Registers { af: f, bc: f, de: f, hl: f, af_: f, bc_: f, de_: f, hl_: f, ix: f, iy: f, sp: f, wz: f, ..Registers::default() };
        Z80 { regs, ..Z80::default() }
    }

    /// One instruction through `bus`.
    pub fn step<B: Bus>(&mut self, bus: &mut B) -> Step {
        self.step_observed(bus, &mut Unobserved)
    }

    /// The step a debugger watches: `observer.before` at the instruction boundary, stores and calls as they happen.
    pub fn step_observed<B: Bus, O: Observer>(&mut self, bus: &mut B, observer: &mut O) -> Step {
        exec::step(self, bus, observer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_on_clears_what_reset_clears_and_sets_the_rest() {
        let z = Z80::power_on();
        assert_eq!((z.regs.pc, z.regs.i, z.regs.r, z.regs.iff1, z.regs.iff2, z.regs.im), (0, 0, 0, false, false, 0));
        assert_eq!((z.regs.af, z.regs.sp, z.regs.hl_, z.regs.iy), (0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF));
        assert!(!z.halted);
    }

    #[test]
    fn each_register_listed_reads_its_own_field() {
        let r = Registers { af: 1, bc: 2, de: 3, hl: 4, af_: 5, bc_: 6, de_: 7, hl_: 8, ix: 9, iy: 10, sp: 11, pc: 12, i: 13, r: 14, wz: 15, im: 2, iff1: true, iff2: false, ..Registers::default() };
        let named: Vec<(&str, i64)> = REGISTERS.iter().map(|r| r.0).zip(r.values()).collect();
        assert_eq!(named[..15], [("AF", 1), ("BC", 2), ("DE", 3), ("HL", 4), ("AF'", 5), ("BC'", 6), ("DE'", 7), ("HL'", 8), ("IX", 9), ("IY", 10), ("SP", 11), ("PC", 12), ("I", 13), ("R", 14), ("WZ", 15)]);
        assert_eq!(named[15..], [("IM", 2), ("IFF1", 1), ("IFF2", 0)]);
        assert!(REGISTERS.iter().zip(r.values()).all(|(&(_, bits), v)| v < 1 << bits));
    }
}
