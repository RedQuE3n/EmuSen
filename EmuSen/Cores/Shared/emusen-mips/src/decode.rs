//! The decode of MIPS I as the R3000 decodes it: the opcode map of the IDT R30xx manual's Appendix A, with the
//! R3000's partial decode of REGIMM and of the coprocessor formats (EmuSen_Mips.md §2).

use crate::Word;

/// An instruction's operation. A coprocessor instruction carries its coprocessor's number; what a coprocessor's
/// command means is its own crate's (EmuSen_Mips.md §2.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Op {
    Sll,
    Srl,
    Sra,
    Sllv,
    Srlv,
    Srav,
    Jr,
    Jalr,
    Syscall,
    Break,
    Mfhi,
    Mthi,
    Mflo,
    Mtlo,
    Mult,
    Multu,
    Div,
    Divu,
    Add,
    Addu,
    Sub,
    Subu,
    And,
    Or,
    Xor,
    Nor,
    Slt,
    Sltu,
    Bltz,
    Bgez,
    Bltzal,
    Bgezal,
    J,
    Jal,
    Beq,
    Bne,
    Blez,
    Bgtz,
    Addi,
    Addiu,
    Slti,
    Sltiu,
    Andi,
    Ori,
    Xori,
    Lui,
    Lb,
    Lh,
    Lwl,
    Lw,
    Lbu,
    Lhu,
    Lwr,
    Sb,
    Sh,
    Swl,
    Sw,
    Swr,
    Mfc(u8),
    Cfc(u8),
    Mtc(u8),
    Ctc(u8),
    Bcf(u8),
    Bct(u8),
    Cop(u8),
    Lwc(u8),
    Swc(u8),
    /// An encoding MIPS I leaves unassigned: the reserved instruction exception.
    Reserved,
}

/// What an operand is, in the order Appendix A's formats write them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operand {
    Rd,
    Rs,
    Rt,
    Sa,
    /// The immediate, sign-extended.
    Simm,
    /// The immediate, zero-extended.
    Uimm,
    /// `offset(base)`: the sign-extended immediate and rs.
    Mem,
    /// A branch's target, written as an address.
    Branch,
    /// A jump's target, written as an address.
    Jump,
    /// SYSCALL's and BREAK's code, written only when it is not zero.
    Code,
    /// A coprocessor data register in the rd field (MFCz, MTCz).
    CopData,
    /// A coprocessor control register in the rd field (CFCz, CTCz).
    CopControl,
    /// A coprocessor data register in the rt field (LWCz, SWCz).
    CopRt,
    /// COPz's 25-bit command.
    Cofun,
}

/// The operation of `word`.
pub fn decode(word: u32) -> Op {
    let w = Word(word);
    match w.op() {
        0x00 => special(w.funct()),
        0x01 => {
            let link = w.rt() & 0x1E == 0x10;
            match (w.rt() & 1 == 1, link) {
                (false, false) => Op::Bltz,
                (true, false) => Op::Bgez,
                (false, true) => Op::Bltzal,
                (true, true) => Op::Bgezal,
            }
        }
        0x02 => Op::J,
        0x03 => Op::Jal,
        0x04 => Op::Beq,
        0x05 => Op::Bne,
        0x06 => Op::Blez,
        0x07 => Op::Bgtz,
        0x08 => Op::Addi,
        0x09 => Op::Addiu,
        0x0A => Op::Slti,
        0x0B => Op::Sltiu,
        0x0C => Op::Andi,
        0x0D => Op::Ori,
        0x0E => Op::Xori,
        0x0F => Op::Lui,
        0x10..=0x13 => coprocessor(w),
        0x20 => Op::Lb,
        0x21 => Op::Lh,
        0x22 => Op::Lwl,
        0x23 => Op::Lw,
        0x24 => Op::Lbu,
        0x25 => Op::Lhu,
        0x26 => Op::Lwr,
        0x28 => Op::Sb,
        0x29 => Op::Sh,
        0x2A => Op::Swl,
        0x2B => Op::Sw,
        0x2E => Op::Swr,
        0x30..=0x33 => Op::Lwc(w.z()),
        0x38..=0x3B => Op::Swc(w.z()),
        _ => Op::Reserved,
    }
}

fn special(funct: u32) -> Op {
    match funct {
        0x00 => Op::Sll,
        0x02 => Op::Srl,
        0x03 => Op::Sra,
        0x04 => Op::Sllv,
        0x06 => Op::Srlv,
        0x07 => Op::Srav,
        0x08 => Op::Jr,
        0x09 => Op::Jalr,
        0x0C => Op::Syscall,
        0x0D => Op::Break,
        0x10 => Op::Mfhi,
        0x11 => Op::Mthi,
        0x12 => Op::Mflo,
        0x13 => Op::Mtlo,
        0x18 => Op::Mult,
        0x19 => Op::Multu,
        0x1A => Op::Div,
        0x1B => Op::Divu,
        0x20 => Op::Add,
        0x21 => Op::Addu,
        0x22 => Op::Sub,
        0x23 => Op::Subu,
        0x24 => Op::And,
        0x25 => Op::Or,
        0x26 => Op::Xor,
        0x27 => Op::Nor,
        0x2A => Op::Slt,
        0x2B => Op::Sltu,
        _ => Op::Reserved,
    }
}

fn coprocessor(w: Word) -> Op {
    let z = w.z();
    if w.0 & 1 << 25 != 0 {
        return Op::Cop(z);
    }
    match w.rs() {
        0x00 => Op::Mfc(z),
        0x02 => Op::Cfc(z),
        0x04 => Op::Mtc(z),
        0x06 => Op::Ctc(z),
        0x08 if w.rt() & 1 == 0 => Op::Bcf(z),
        0x08 => Op::Bct(z),
        _ => Op::Reserved,
    }
}

impl Op {
    /// The mnemonic, lower case, as Appendix A names it with the coprocessor's number in place of z.
    pub fn mnemonic(self) -> &'static str {
        fn z(names: [&'static str; 4], n: u8) -> &'static str {
            names[n as usize & 3]
        }
        match self {
            Op::Sll => "sll",
            Op::Srl => "srl",
            Op::Sra => "sra",
            Op::Sllv => "sllv",
            Op::Srlv => "srlv",
            Op::Srav => "srav",
            Op::Jr => "jr",
            Op::Jalr => "jalr",
            Op::Syscall => "syscall",
            Op::Break => "break",
            Op::Mfhi => "mfhi",
            Op::Mthi => "mthi",
            Op::Mflo => "mflo",
            Op::Mtlo => "mtlo",
            Op::Mult => "mult",
            Op::Multu => "multu",
            Op::Div => "div",
            Op::Divu => "divu",
            Op::Add => "add",
            Op::Addu => "addu",
            Op::Sub => "sub",
            Op::Subu => "subu",
            Op::And => "and",
            Op::Or => "or",
            Op::Xor => "xor",
            Op::Nor => "nor",
            Op::Slt => "slt",
            Op::Sltu => "sltu",
            Op::Bltz => "bltz",
            Op::Bgez => "bgez",
            Op::Bltzal => "bltzal",
            Op::Bgezal => "bgezal",
            Op::J => "j",
            Op::Jal => "jal",
            Op::Beq => "beq",
            Op::Bne => "bne",
            Op::Blez => "blez",
            Op::Bgtz => "bgtz",
            Op::Addi => "addi",
            Op::Addiu => "addiu",
            Op::Slti => "slti",
            Op::Sltiu => "sltiu",
            Op::Andi => "andi",
            Op::Ori => "ori",
            Op::Xori => "xori",
            Op::Lui => "lui",
            Op::Lb => "lb",
            Op::Lh => "lh",
            Op::Lwl => "lwl",
            Op::Lw => "lw",
            Op::Lbu => "lbu",
            Op::Lhu => "lhu",
            Op::Lwr => "lwr",
            Op::Sb => "sb",
            Op::Sh => "sh",
            Op::Swl => "swl",
            Op::Sw => "sw",
            Op::Swr => "swr",
            Op::Mfc(n) => z(["mfc0", "mfc1", "mfc2", "mfc3"], n),
            Op::Cfc(n) => z(["cfc0", "cfc1", "cfc2", "cfc3"], n),
            Op::Mtc(n) => z(["mtc0", "mtc1", "mtc2", "mtc3"], n),
            Op::Ctc(n) => z(["ctc0", "ctc1", "ctc2", "ctc3"], n),
            Op::Bcf(n) => z(["bc0f", "bc1f", "bc2f", "bc3f"], n),
            Op::Bct(n) => z(["bc0t", "bc1t", "bc2t", "bc3t"], n),
            Op::Cop(n) => z(["cop0", "cop1", "cop2", "cop3"], n),
            Op::Lwc(n) => z(["lwc0", "lwc1", "lwc2", "lwc3"], n),
            Op::Swc(n) => z(["swc0", "swc1", "swc2", "swc3"], n),
            Op::Reserved => ".word",
        }
    }

    /// The operands in Appendix A's order.
    pub fn operands(self) -> &'static [Operand] {
        use Operand::*;
        match self {
            Op::Add | Op::Addu | Op::Sub | Op::Subu | Op::And | Op::Or | Op::Xor | Op::Nor | Op::Slt | Op::Sltu => &[Rd, Rs, Rt],
            Op::Sll | Op::Srl | Op::Sra => &[Rd, Rt, Sa],
            Op::Sllv | Op::Srlv | Op::Srav => &[Rd, Rt, Rs],
            Op::Jr | Op::Mthi | Op::Mtlo => &[Rs],
            Op::Jalr => &[Rd, Rs],
            Op::Syscall | Op::Break => &[Code],
            Op::Mfhi | Op::Mflo => &[Rd],
            Op::Mult | Op::Multu | Op::Div | Op::Divu => &[Rs, Rt],
            Op::Bltz | Op::Bgez | Op::Bltzal | Op::Bgezal | Op::Blez | Op::Bgtz => &[Rs, Branch],
            Op::Beq | Op::Bne => &[Rs, Rt, Branch],
            Op::J | Op::Jal => &[Jump],
            Op::Addi | Op::Addiu | Op::Slti | Op::Sltiu => &[Rt, Rs, Simm],
            Op::Andi | Op::Ori | Op::Xori => &[Rt, Rs, Uimm],
            Op::Lui => &[Rt, Uimm],
            Op::Lb | Op::Lh | Op::Lwl | Op::Lw | Op::Lbu | Op::Lhu | Op::Lwr | Op::Sb | Op::Sh | Op::Swl | Op::Sw | Op::Swr => &[Rt, Mem],
            Op::Mfc(_) | Op::Mtc(_) => &[Rt, CopData],
            Op::Cfc(_) | Op::Ctc(_) => &[Rt, CopControl],
            Op::Bcf(_) | Op::Bct(_) => &[Branch],
            Op::Cop(_) => &[Cofun],
            Op::Lwc(_) | Op::Swc(_) => &[CopRt, Mem],
            Op::Reserved => &[],
        }
    }

    /// The word with every operand field zero, in the encoding the manual gives (the canonical one where the R3000's
    /// decode ignores some bits).
    pub fn canonical(self) -> u32 {
        let special = |f: u32| f;
        let primary = |o: u32| o << 26;
        let regimm = |rt: u32| 0x01 << 26 | rt << 16;
        let cop = |z: u8, rs: u32| (0x10 | z as u32) << 26 | rs << 21;
        match self {
            Op::Sll => special(0x00),
            Op::Srl => special(0x02),
            Op::Sra => special(0x03),
            Op::Sllv => special(0x04),
            Op::Srlv => special(0x06),
            Op::Srav => special(0x07),
            Op::Jr => special(0x08),
            Op::Jalr => special(0x09),
            Op::Syscall => special(0x0C),
            Op::Break => special(0x0D),
            Op::Mfhi => special(0x10),
            Op::Mthi => special(0x11),
            Op::Mflo => special(0x12),
            Op::Mtlo => special(0x13),
            Op::Mult => special(0x18),
            Op::Multu => special(0x19),
            Op::Div => special(0x1A),
            Op::Divu => special(0x1B),
            Op::Add => special(0x20),
            Op::Addu => special(0x21),
            Op::Sub => special(0x22),
            Op::Subu => special(0x23),
            Op::And => special(0x24),
            Op::Or => special(0x25),
            Op::Xor => special(0x26),
            Op::Nor => special(0x27),
            Op::Slt => special(0x2A),
            Op::Sltu => special(0x2B),
            Op::Bltz => regimm(0x00),
            Op::Bgez => regimm(0x01),
            Op::Bltzal => regimm(0x10),
            Op::Bgezal => regimm(0x11),
            Op::J => primary(0x02),
            Op::Jal => primary(0x03),
            Op::Beq => primary(0x04),
            Op::Bne => primary(0x05),
            Op::Blez => primary(0x06),
            Op::Bgtz => primary(0x07),
            Op::Addi => primary(0x08),
            Op::Addiu => primary(0x09),
            Op::Slti => primary(0x0A),
            Op::Sltiu => primary(0x0B),
            Op::Andi => primary(0x0C),
            Op::Ori => primary(0x0D),
            Op::Xori => primary(0x0E),
            Op::Lui => primary(0x0F),
            Op::Lb => primary(0x20),
            Op::Lh => primary(0x21),
            Op::Lwl => primary(0x22),
            Op::Lw => primary(0x23),
            Op::Lbu => primary(0x24),
            Op::Lhu => primary(0x25),
            Op::Lwr => primary(0x26),
            Op::Sb => primary(0x28),
            Op::Sh => primary(0x29),
            Op::Swl => primary(0x2A),
            Op::Sw => primary(0x2B),
            Op::Swr => primary(0x2E),
            Op::Mfc(z) => cop(z, 0x00),
            Op::Cfc(z) => cop(z, 0x02),
            Op::Mtc(z) => cop(z, 0x04),
            Op::Ctc(z) => cop(z, 0x06),
            Op::Bcf(z) => cop(z, 0x08),
            Op::Bct(z) => cop(z, 0x08) | 1 << 16,
            Op::Cop(z) => cop(z, 0x10),
            Op::Lwc(z) => primary(0x30 | z as u32),
            Op::Swc(z) => primary(0x38 | z as u32),
            Op::Reserved => primary(0x3F),
        }
    }

    /// Every operation, each coprocessor's four times.
    pub fn all() -> impl Iterator<Item = Op> {
        const PLAIN: [Op; 58] = [
            Op::Sll, Op::Srl, Op::Sra, Op::Sllv, Op::Srlv, Op::Srav, Op::Jr, Op::Jalr, Op::Syscall, Op::Break, Op::Mfhi, Op::Mthi, Op::Mflo,
            Op::Mtlo, Op::Mult, Op::Multu, Op::Div, Op::Divu, Op::Add, Op::Addu, Op::Sub, Op::Subu, Op::And, Op::Or, Op::Xor, Op::Nor,
            Op::Slt, Op::Sltu, Op::Bltz, Op::Bgez, Op::Bltzal, Op::Bgezal, Op::J, Op::Jal, Op::Beq, Op::Bne, Op::Blez, Op::Bgtz, Op::Addi,
            Op::Addiu, Op::Slti, Op::Sltiu, Op::Andi, Op::Ori, Op::Xori, Op::Lui, Op::Lb, Op::Lh, Op::Lwl, Op::Lw, Op::Lbu, Op::Lhu,
            Op::Lwr, Op::Sb, Op::Sh, Op::Swl, Op::Sw, Op::Swr,
        ];
        const COP: [fn(u8) -> Op; 9] = [Op::Mfc, Op::Cfc, Op::Mtc, Op::Ctc, Op::Bcf, Op::Bct, Op::Cop, Op::Lwc, Op::Swc];
        PLAIN.into_iter().chain(COP.into_iter().flat_map(|f| (0..4).map(f)))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A small deterministic generator, so the operand fields of a sweep differ case by case.
    pub(crate) fn mix(seed: &mut u64) -> u32 {
        *seed ^= *seed << 13;
        *seed ^= *seed >> 7;
        *seed ^= *seed << 17;
        (*seed >> 16) as u32
    }

    /// The bits each operand occupies.
    pub(crate) fn operand_bits(op: Op) -> u32 {
        op.operands().iter().fold(0, |m, o| {
            m | match o {
                Operand::Rd | Operand::CopData | Operand::CopControl => 31 << 11,
                Operand::Rs => 31 << 21,
                Operand::Rt | Operand::CopRt => 31 << 16,
                Operand::Sa => 31 << 6,
                Operand::Simm | Operand::Uimm | Operand::Branch => 0xFFFF,
                Operand::Mem => 31 << 21 | 0xFFFF,
                Operand::Jump => 0x03FF_FFFF,
                Operand::Code => 0xF_FFFF << 6,
                Operand::Cofun => 0x01FF_FFFF,
            }
        })
    }

    #[test]
    fn every_operation_decodes_from_its_canonical_word_whatever_its_operands() {
        let mut seed = 0x9E37_79B9_7F4A_7C15;
        for op in Op::all() {
            let bits = operand_bits(op);
            assert_eq!(op.canonical() & bits, 0, "{op:?}: an operand overlaps the fixed bits");
            for _ in 0..64 {
                let w = op.canonical() | mix(&mut seed) & bits;
                assert_eq!(decode(w), op, "{w:08X}");
            }
        }
    }

    #[test]
    fn the_primary_and_secondary_maps_reserve_what_psx_spx_marks_n_a() {
        let primary_na = [0x14..=0x1F, 0x27..=0x27, 0x2C..=0x2D, 0x2F..=0x2F, 0x34..=0x37, 0x3C..=0x3F];
        for r in primary_na {
            for o in r {
                assert_eq!(decode(o << 26), Op::Reserved, "primary {o:02X}");
            }
        }
        let assigned = [0x00, 0x02, 0x03, 0x04, 0x06, 0x07, 0x08, 0x09, 0x0C, 0x0D, 0x10, 0x11, 0x12, 0x13, 0x18, 0x19, 0x1A, 0x1B];
        for f in 0..64u32 {
            let reserved = !(assigned.contains(&f) || (0x20..=0x27).contains(&f) || f == 0x2A || f == 0x2B);
            assert_eq!(decode(f) == Op::Reserved, reserved, "secondary {f:02X}");
        }
    }

    #[test]
    fn regimm_links_only_with_rt_bits_20_to_17_clear_and_bit_16_selects_the_condition() {
        for rt in 0..32u32 {
            let want = match (rt & 1, rt & 0x1E == 0x10) {
                (0, false) => Op::Bltz,
                (1, false) => Op::Bgez,
                (0, true) => Op::Bltzal,
                _ => Op::Bgezal,
            };
            assert_eq!(decode(0x0400_0000 | rt << 16), want, "rt {rt}");
        }
    }

    #[test]
    fn the_coprocessor_formats() {
        for z in 0..4u8 {
            let base = (0x10 | z as u32) << 26;
            assert_eq!(decode(base | 1 << 25 | 0x1E0_0000), Op::Cop(z));
            for (rs, want) in [(0, Op::Mfc(z)), (2, Op::Cfc(z)), (4, Op::Mtc(z)), (6, Op::Ctc(z))] {
                assert_eq!(decode(base | rs << 21), want);
            }
            assert_eq!(decode(base | 8 << 21), Op::Bcf(z));
            assert_eq!(decode(base | 8 << 21 | 1 << 16), Op::Bct(z));
            for rs in [1, 3, 5, 7, 9, 10, 15] {
                assert_eq!(decode(base | rs << 21), Op::Reserved, "rs {rs}");
            }
        }
    }
}
