//! The MIPS instruction set as data, shared by the cores whose processors are MIPS: the fields of an instruction word,
//! the decode of MIPS I as the R3000 decodes it, the disassembler with a table per coprocessor, an assembler for test
//! programs, MIPS III as the VR4300 runs it, written in that part's own spelling, the N64's signal processor with its
//! vector unit in the same spelling, the PlayStation 2's R5900 and its vector units' micro and macro instructions. Nothing here executes; each processor crate keeps its own semantics. See EmuSen_Mips.md.

mod asm;
mod decode;
mod disasm;
pub mod r5900;
pub mod rsp;
pub mod vr4300;
pub mod vu;

pub use asm::{AsmError, assemble, assemble_line};
pub use decode::{Op, Operand, decode};
pub use disasm::{Command, Coprocessor, Dialect, Disassembly, GPR_NAMES, Spelling, disassemble};

/// An instruction word and its fields (IDT R30xx Family Software Reference Manual, Figure A.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Word(pub u32);

impl Word {
    /// Bits 31-26.
    pub fn op(self) -> u32 {
        self.0 >> 26
    }
    pub fn rs(self) -> usize {
        (self.0 >> 21 & 31) as usize
    }
    pub fn rt(self) -> usize {
        (self.0 >> 16 & 31) as usize
    }
    pub fn rd(self) -> usize {
        (self.0 >> 11 & 31) as usize
    }
    /// The shift amount, bits 10-6.
    pub fn sa(self) -> u32 {
        self.0 >> 6 & 31
    }
    pub fn funct(self) -> u32 {
        self.0 & 63
    }
    /// The 16-bit immediate, zero-extended.
    pub fn imm(self) -> u32 {
        self.0 & 0xFFFF
    }
    /// The 16-bit immediate, sign-extended.
    pub fn simm(self) -> u32 {
        self.0 as i16 as i32 as u32
    }
    /// The 26-bit jump target, in words.
    pub fn target(self) -> u32 {
        self.0 & 0x03FF_FFFF
    }
    /// SYSCALL's and BREAK's 20-bit code.
    pub fn code(self) -> u32 {
        self.0 >> 6 & 0xF_FFFF
    }
    /// The coprocessor a coprocessor instruction names, bits 27-26.
    pub fn z(self) -> u8 {
        (self.0 >> 26 & 3) as u8
    }
    /// A COPz instruction's 25-bit command.
    pub fn cofun(self) -> u32 {
        self.0 & 0x01FF_FFFF
    }
}
