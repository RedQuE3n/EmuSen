//! The disassembler: Appendix A's mnemonics and operand order, the general registers by their conventional names, and
//! a table per coprocessor for its registers' and commands' names (EmuSen_Mips.md §3).

use std::fmt;

use crate::{Op, Operand, Word, decode};

/// The general registers' conventional names (IDT R30xx manual, Table 2.1).
pub const GPR_NAMES: [&str; 32] = [
    "zero", "at", "v0", "v1", "a0", "a1", "a2", "a3", "t0", "t1", "t2", "t3", "t4", "t5", "t6", "t7", "s0", "s1", "s2", "s3", "s4", "s5",
    "s6", "s7", "t8", "t9", "k0", "k1", "gp", "sp", "fp", "ra",
];

/// A coprocessor command: its name, the bits of the command that select it, and the command as its documents give it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Command {
    pub name: &'static str,
    /// The bits of the 25-bit command the coprocessor decodes the command from.
    pub mask: u32,
    pub select: u32,
    /// The whole 25 bits as the documents write the command; written by its name alone.
    pub canonical: u32,
}

/// What one coprocessor's instructions are called: its registers' names (empty for an unnamed register, written
/// `$n`) and its commands. The table belongs to the crate that owns the coprocessor.
#[derive(Clone, Copy, Debug)]
pub struct Coprocessor {
    pub data: [&'static str; 32],
    pub control: [&'static str; 32],
    pub commands: &'static [Command],
}

impl Coprocessor {
    pub fn command(&self, cofun: u32) -> Option<&'static Command> {
        self.commands.iter().find(|c| cofun & c.mask == c.select)
    }
}

/// How a processor's documents write its instructions: Appendix A's MIPS I as the R3000's do, or the VR4300's MIPS III
/// with its folded forms and `s8` (EmuSen_Mips.md §6, decided by the tester on 2026-10-17 so that neither console's
/// debugger output changes).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Spelling {
    #[default]
    AppendixA,
    Vr4300,
}

/// The coprocessors a processor has, by number, an absent one written with numbers only; and its spelling. The VR4300's
/// spelling names its own coprocessors' registers, so its tables here go unread.
#[derive(Clone, Copy, Debug, Default)]
pub struct Dialect {
    pub cop: [Option<&'static Coprocessor>; 4],
    pub spelling: Spelling,
}

impl Dialect {
    pub const MIPS_I: Dialect = Dialect { cop: [None; 4], spelling: Spelling::AppendixA };
    pub const VR4300: Dialect = Dialect { cop: [None; 4], spelling: Spelling::Vr4300 };
}

/// An instruction ready to be written as text: `word`, fetched from `pc`.
pub struct Disassembly<'a> {
    pub word: u32,
    pub pc: u32,
    pub dialect: &'a Dialect,
}

pub fn disassemble(word: u32, pc: u32, dialect: &Dialect) -> Disassembly<'_> {
    Disassembly { word, pc, dialect }
}

fn hex(f: &mut fmt::Formatter, v: u32) -> fmt::Result {
    write!(f, "0x{v:x}")
}

fn signed(f: &mut fmt::Formatter, v: u32) -> fmt::Result {
    let v = v as i32;
    if v < 0 { write!(f, "-0x{:x}", v.unsigned_abs()) } else { write!(f, "0x{v:x}") }
}

fn cop_register(f: &mut fmt::Formatter, names: Option<&[&'static str; 32]>, n: usize) -> fmt::Result {
    match names.map(|t| t[n]) {
        Some(name) if !name.is_empty() => f.write_str(name),
        _ => write!(f, "${n}"),
    }
}

impl fmt::Display for Disassembly<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.dialect.spelling == Spelling::Vr4300 {
            let i = crate::vr4300::decode(self.word, self.pc);
            f.write_str(&i.mnemonic)?;
            return if i.operands.is_empty() { Ok(()) } else { write!(f, " {}", i.operands) };
        }
        let w = Word(self.word);
        let op = decode(self.word);
        if self.word == 0 {
            return f.write_str("nop");
        }
        if op == Op::Reserved {
            f.write_str(".word ")?;
            return write!(f, "0x{:08x}", self.word);
        }
        let table = |z: u8| self.dialect.cop[z as usize & 3];
        if let Op::Cop(z) = op
            && let Some(c) = table(z).and_then(|t| t.command(w.cofun()))
        {
            f.write_str(c.name)?;
            return if w.cofun() == c.canonical { Ok(()) } else { write!(f, " 0x{:07x}", w.cofun()) };
        }
        f.write_str(op.mnemonic())?;
        let mut operands: &[Operand] = op.operands();
        if op == Op::Jalr && w.rd() == 31 {
            operands = &[Operand::Rs];
        }
        if matches!(op, Op::Syscall | Op::Break) && w.code() == 0 {
            operands = &[];
        }
        let z = w.z();
        let next = self.pc.wrapping_add(4);
        for (i, operand) in operands.iter().enumerate() {
            f.write_str(if i == 0 { " " } else { ", " })?;
            match operand {
                Operand::Rd => f.write_str(GPR_NAMES[w.rd()])?,
                Operand::Rs => f.write_str(GPR_NAMES[w.rs()])?,
                Operand::Rt => f.write_str(GPR_NAMES[w.rt()])?,
                Operand::Sa => write!(f, "{}", w.sa())?,
                Operand::Simm => signed(f, w.simm())?,
                Operand::Uimm => hex(f, w.imm())?,
                Operand::Mem => {
                    signed(f, w.simm())?;
                    write!(f, "({})", GPR_NAMES[w.rs()])?
                }
                Operand::Branch => write!(f, "0x{:08x}", next.wrapping_add(w.simm() << 2))?,
                Operand::Jump => write!(f, "0x{:08x}", next & 0xF000_0000 | w.target() << 2)?,
                Operand::Code => hex(f, w.code())?,
                Operand::CopData => cop_register(f, table(z).map(|t| &t.data), w.rd())?,
                Operand::CopControl => cop_register(f, table(z).map(|t| &t.control), w.rd())?,
                Operand::CopRt => cop_register(f, table(z).map(|t| &t.data), w.rt())?,
                Operand::Cofun => write!(f, "0x{:07x}", w.cofun())?,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_COP: Coprocessor = Coprocessor {
        data: {
            let mut n = [""; 32];
            n[12] = "SR";
            n
        },
        control: [""; 32],
        commands: &[Command { name: "rfe", mask: 0x3F, select: 0x10, canonical: 0x10 }],
    };

    fn text(word: u32, pc: u32) -> String {
        let d = Dialect { cop: [Some(&TEST_COP), None, None, None], ..Dialect::MIPS_I };
        disassemble(word, pc, &d).to_string()
    }

    /// Each format line of the IDT R30xx manual's Appendix A, its fields given values that tell them apart: rd = v0,
    /// rs = a0 (base), rt = a1, sa = 3, an immediate of 0x10 (offset), a branch of one word forward, a coprocessor's
    /// register 12. The disassembly must write the format's mnemonic and its operands in its order.
    #[test]
    fn every_appendix_a_format_is_written_in_its_order() {
        let rd = 2u32 << 11;
        let rs = 4u32 << 21;
        let rt = 5u32 << 16;
        let pc = 0x8001_0000u32;
        let cases: [(Op, u32, &str); 61] = [
            (Op::Add, rd | rs | rt, "add v0, a0, a1"),
            (Op::Addi, rs | rt | 0x10, "addi a1, a0, 0x10"),
            (Op::Addiu, rs | rt | 0xFFF0, "addiu a1, a0, -0x10"),
            (Op::Addu, rd | rs | rt, "addu v0, a0, a1"),
            (Op::And, rd | rs | rt, "and v0, a0, a1"),
            (Op::Andi, rs | rt | 0x10, "andi a1, a0, 0x10"),
            (Op::Beq, rs | rt | 1, "beq a0, a1, 0x80010008"),
            (Op::Bgez, rs | 1, "bgez a0, 0x80010008"),
            (Op::Bgezal, rs | 1, "bgezal a0, 0x80010008"),
            (Op::Bgtz, rs | 1, "bgtz a0, 0x80010008"),
            (Op::Blez, rs | 1, "blez a0, 0x80010008"),
            (Op::Bltz, rs | 1, "bltz a0, 0x80010008"),
            (Op::Bltzal, rs | 1, "bltzal a0, 0x80010008"),
            (Op::Bne, rs | rt | 0xFFFF, "bne a0, a1, 0x80010000"),
            (Op::Break, 0, "break"),
            (Op::Cfc(0), rt | 12 << 11, "cfc0 a1, $12"),
            (Op::Cop(1), 0x10, "cop1 0x0000010"),
            (Op::Ctc(1), rt | 12 << 11, "ctc1 a1, $12"),
            (Op::Div, rs | rt, "div a0, a1"),
            (Op::Divu, rs | rt, "divu a0, a1"),
            (Op::J, 0x0004_0010, "j 0x80100040"),
            (Op::Jal, 0x0004_0010, "jal 0x80100040"),
            (Op::Jalr, rs | 31 << 11, "jalr a0"),
            (Op::Jalr, rd | rs, "jalr v0, a0"),
            (Op::Jr, rs, "jr a0"),
            (Op::Lb, rs | rt | 0x10, "lb a1, 0x10(a0)"),
            (Op::Lbu, rs | rt | 0x10, "lbu a1, 0x10(a0)"),
            (Op::Lh, rs | rt | 0x10, "lh a1, 0x10(a0)"),
            (Op::Lhu, rs | rt | 0x10, "lhu a1, 0x10(a0)"),
            (Op::Lui, rt | 0x1F80, "lui a1, 0x1f80"),
            (Op::Lw, rs | rt | 0xFFFC, "lw a1, -0x4(a0)"),
            (Op::Lwc(2), rs | 12 << 16 | 0x10, "lwc2 $12, 0x10(a0)"),
            (Op::Lwl, rs | rt | 0x10, "lwl a1, 0x10(a0)"),
            (Op::Lwr, rs | rt | 0x10, "lwr a1, 0x10(a0)"),
            (Op::Mfc(0), rt | 12 << 11, "mfc0 a1, SR"),
            (Op::Mfhi, rd, "mfhi v0"),
            (Op::Mflo, rd, "mflo v0"),
            (Op::Mtc(0), rt | 12 << 11, "mtc0 a1, SR"),
            (Op::Mthi, rs, "mthi a0"),
            (Op::Mtlo, rs, "mtlo a0"),
            (Op::Mult, rs | rt, "mult a0, a1"),
            (Op::Multu, rs | rt, "multu a0, a1"),
            (Op::Nor, rd | rs | rt, "nor v0, a0, a1"),
            (Op::Or, rd | rs | rt, "or v0, a0, a1"),
            (Op::Ori, rs | rt | 0x10, "ori a1, a0, 0x10"),
            (Op::Sb, rs | rt | 0x10, "sb a1, 0x10(a0)"),
            (Op::Sh, rs | rt | 0x10, "sh a1, 0x10(a0)"),
            (Op::Sll, rd | rt | 3 << 6, "sll v0, a1, 3"),
            (Op::Sllv, rd | rs | rt, "sllv v0, a1, a0"),
            (Op::Slt, rd | rs | rt, "slt v0, a0, a1"),
            (Op::Slti, rs | rt | 0x10, "slti a1, a0, 0x10"),
            (Op::Sltiu, rs | rt | 0x10, "sltiu a1, a0, 0x10"),
            (Op::Sltu, rd | rs | rt, "sltu v0, a0, a1"),
            (Op::Sra, rd | rt | 3 << 6, "sra v0, a1, 3"),
            (Op::Srav, rd | rs | rt, "srav v0, a1, a0"),
            (Op::Srl, rd | rt | 3 << 6, "srl v0, a1, 3"),
            (Op::Srlv, rd | rs | rt, "srlv v0, a1, a0"),
            (Op::Sub, rd | rs | rt, "sub v0, a0, a1"),
            (Op::Subu, rd | rs | rt, "subu v0, a0, a1"),
            (Op::Sw, rs | rt | 0x10, "sw a1, 0x10(a0)"),
            (Op::Syscall, 0, "syscall"),
        ];
        for (op, fields, want) in cases {
            assert_eq!(text(op.canonical() | fields, pc), want, "{op:?}");
        }
        let more: [(Op, u32, &str); 9] = [
            (Op::Swc(2), rs | 12 << 16 | 0x10, "swc2 $12, 0x10(a0)"),
            (Op::Swl, rs | rt | 0x10, "swl a1, 0x10(a0)"),
            (Op::Swr, rs | rt | 0x10, "swr a1, 0x10(a0)"),
            (Op::Xor, rd | rs | rt, "xor v0, a0, a1"),
            (Op::Xori, rs | rt | 0x10, "xori a1, a0, 0x10"),
            (Op::Syscall, 0x1234 << 6, "syscall 0x1234"),
            (Op::Bcf(2), 0xFFFF, "bc2f 0x80010000"),
            (Op::Bct(2), 1, "bc2t 0x80010008"),
            (Op::Cop(0), 0x10, "rfe"),
        ];
        for (op, fields, want) in more {
            assert_eq!(text(op.canonical() | fields, pc), want, "{op:?}");
        }
    }

    #[test]
    fn nop_a_reserved_word_and_a_command_with_bits_beyond_its_canonical_form() {
        assert_eq!(text(0, 0), "nop");
        assert_eq!(text(0xFC00_0000, 0), ".word 0xfc000000");
        assert_eq!(text(0x4200_0410, 0), "rfe 0x0000410");
        assert_eq!(text(0x4200_0001, 0), "cop0 0x0000001");
    }
}
