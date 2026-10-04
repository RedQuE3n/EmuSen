//! The 65C816's instructions by WDC's datasheet: Table 5-4, the opcode matrix, and Table 6-2's operand formats.

use super::{Instruction, Reference};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum M {
    Imp,
    Acc,
    /// Immediate as wide as the accumulator, the index registers, or always one byte.
    ImmM,
    ImmX,
    Imm8,
    Dp,
    DpX,
    DpY,
    DpInd,
    DpIndX,
    DpIndY,
    DpLong,
    DpLongY,
    Sr,
    SrIndY,
    Abs,
    AbsX,
    AbsY,
    Long,
    LongX,
    AbsInd,
    AbsIndX,
    AbsLong,
    Rel,
    RelLong,
    Move,
}

use M::*;

/// Table 5-4, row by row; 5C and DC under their alternate mnemonic JML (Table 6-1).
#[rustfmt::skip]
const OPS: [(&str, M); 256] = [
    ("BRK", Imm8), ("ORA", DpIndX), ("COP", Imm8), ("ORA", Sr), ("TSB", Dp), ("ORA", Dp), ("ASL", Dp), ("ORA", DpLong), ("PHP", Imp), ("ORA", ImmM), ("ASL", Acc), ("PHD", Imp), ("TSB", Abs), ("ORA", Abs), ("ASL", Abs), ("ORA", Long),
    ("BPL", Rel), ("ORA", DpIndY), ("ORA", DpInd), ("ORA", SrIndY), ("TRB", Dp), ("ORA", DpX), ("ASL", DpX), ("ORA", DpLongY), ("CLC", Imp), ("ORA", AbsY), ("INC", Acc), ("TCS", Imp), ("TRB", Abs), ("ORA", AbsX), ("ASL", AbsX), ("ORA", LongX),
    ("JSR", Abs), ("AND", DpIndX), ("JSL", Long), ("AND", Sr), ("BIT", Dp), ("AND", Dp), ("ROL", Dp), ("AND", DpLong), ("PLP", Imp), ("AND", ImmM), ("ROL", Acc), ("PLD", Imp), ("BIT", Abs), ("AND", Abs), ("ROL", Abs), ("AND", Long),
    ("BMI", Rel), ("AND", DpIndY), ("AND", DpInd), ("AND", SrIndY), ("BIT", DpX), ("AND", DpX), ("ROL", DpX), ("AND", DpLongY), ("SEC", Imp), ("AND", AbsY), ("DEC", Acc), ("TSC", Imp), ("BIT", AbsX), ("AND", AbsX), ("ROL", AbsX), ("AND", LongX),
    ("RTI", Imp), ("EOR", DpIndX), ("WDM", Imm8), ("EOR", Sr), ("MVP", Move), ("EOR", Dp), ("LSR", Dp), ("EOR", DpLong), ("PHA", Imp), ("EOR", ImmM), ("LSR", Acc), ("PHK", Imp), ("JMP", Abs), ("EOR", Abs), ("LSR", Abs), ("EOR", Long),
    ("BVC", Rel), ("EOR", DpIndY), ("EOR", DpInd), ("EOR", SrIndY), ("MVN", Move), ("EOR", DpX), ("LSR", DpX), ("EOR", DpLongY), ("CLI", Imp), ("EOR", AbsY), ("PHY", Imp), ("TCD", Imp), ("JML", Long), ("EOR", AbsX), ("LSR", AbsX), ("EOR", LongX),
    ("RTS", Imp), ("ADC", DpIndX), ("PER", RelLong), ("ADC", Sr), ("STZ", Dp), ("ADC", Dp), ("ROR", Dp), ("ADC", DpLong), ("PLA", Imp), ("ADC", ImmM), ("ROR", Acc), ("RTL", Imp), ("JMP", AbsInd), ("ADC", Abs), ("ROR", Abs), ("ADC", Long),
    ("BVS", Rel), ("ADC", DpIndY), ("ADC", DpInd), ("ADC", SrIndY), ("STZ", DpX), ("ADC", DpX), ("ROR", DpX), ("ADC", DpLongY), ("SEI", Imp), ("ADC", AbsY), ("PLY", Imp), ("TDC", Imp), ("JMP", AbsIndX), ("ADC", AbsX), ("ROR", AbsX), ("ADC", LongX),
    ("BRA", Rel), ("STA", DpIndX), ("BRL", RelLong), ("STA", Sr), ("STY", Dp), ("STA", Dp), ("STX", Dp), ("STA", DpLong), ("DEY", Imp), ("BIT", ImmM), ("TXA", Imp), ("PHB", Imp), ("STY", Abs), ("STA", Abs), ("STX", Abs), ("STA", Long),
    ("BCC", Rel), ("STA", DpIndY), ("STA", DpInd), ("STA", SrIndY), ("STY", DpX), ("STA", DpX), ("STX", DpY), ("STA", DpLongY), ("TYA", Imp), ("STA", AbsY), ("TXS", Imp), ("TXY", Imp), ("STZ", Abs), ("STA", AbsX), ("STZ", AbsX), ("STA", LongX),
    ("LDY", ImmX), ("LDA", DpIndX), ("LDX", ImmX), ("LDA", Sr), ("LDY", Dp), ("LDA", Dp), ("LDX", Dp), ("LDA", DpLong), ("TAY", Imp), ("LDA", ImmM), ("TAX", Imp), ("PLB", Imp), ("LDY", Abs), ("LDA", Abs), ("LDX", Abs), ("LDA", Long),
    ("BCS", Rel), ("LDA", DpIndY), ("LDA", DpInd), ("LDA", SrIndY), ("LDY", DpX), ("LDA", DpX), ("LDX", DpY), ("LDA", DpLongY), ("CLV", Imp), ("LDA", AbsY), ("TSX", Imp), ("TYX", Imp), ("LDY", AbsX), ("LDA", AbsX), ("LDX", AbsY), ("LDA", LongX),
    ("CPY", ImmX), ("CMP", DpIndX), ("REP", Imm8), ("CMP", Sr), ("CPY", Dp), ("CMP", Dp), ("DEC", Dp), ("CMP", DpLong), ("INY", Imp), ("CMP", ImmM), ("DEX", Imp), ("WAI", Imp), ("CPY", Abs), ("CMP", Abs), ("DEC", Abs), ("CMP", Long),
    ("BNE", Rel), ("CMP", DpIndY), ("CMP", DpInd), ("CMP", SrIndY), ("PEI", DpInd), ("CMP", DpX), ("DEC", DpX), ("CMP", DpLongY), ("CLD", Imp), ("CMP", AbsY), ("PHX", Imp), ("STP", Imp), ("JML", AbsLong), ("CMP", AbsX), ("DEC", AbsX), ("CMP", LongX),
    ("CPX", ImmX), ("SBC", DpIndX), ("SEP", Imm8), ("SBC", Sr), ("CPX", Dp), ("SBC", Dp), ("INC", Dp), ("SBC", DpLong), ("INX", Imp), ("SBC", ImmM), ("NOP", Imp), ("XBA", Imp), ("CPX", Abs), ("SBC", Abs), ("INC", Abs), ("SBC", Long),
    ("BEQ", Rel), ("SBC", DpIndY), ("SBC", DpInd), ("SBC", SrIndY), ("PEA", Abs), ("SBC", DpX), ("INC", DpX), ("SBC", DpLongY), ("SED", Imp), ("SBC", AbsY), ("PLX", Imp), ("XCE", Imp), ("JSR", AbsIndX), ("SBC", AbsX), ("INC", AbsX), ("SBC", LongX),
];

/// The register widths a listing is decoded with, moved by the REP and SEP it meets: true is 8 bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Widths {
    pub m: bool,
    pub x: bool,
    /// Emulation mode, where REP and SEP cannot widen anything.
    pub e: bool,
}

/// The operand's bytes after the opcode.
fn operand_length(mode: M, w: Widths) -> u32 {
    match mode {
        Imp | Acc => 0,
        ImmM => 2 - w.m as u32,
        ImmX => 2 - w.x as u32,
        Imm8 | Dp | DpX | DpY | DpInd | DpIndX | DpIndY | DpLong | DpLongY | Sr | SrIndY | Rel => 1,
        Abs | AbsX | AbsY | AbsInd | AbsIndX | AbsLong | RelLong | Move => 2,
        Long | LongX => 3,
    }
}

/// The instruction at `address`, its operand following within the bank; `bank` is DBR, for a data reference.
pub fn decode(read: &dyn Fn(u32) -> u8, address: u32, w: &mut Widths, bank: u8) -> Instruction {
    let at = |i: u32| (address & 0xFF_0000) | (address.wrapping_add(i) & 0xFFFF);
    let opcode = read(at(0));
    let (mnemonic, mode) = OPS[opcode as usize];
    let n = operand_length(mode, *w);
    let bytes = super::bytes(read, address, 1 + n, |a, i| (a & 0xFF_0000) | (a.wrapping_add(i) & 0xFFFF));
    let v = bytes[1..].iter().rev().fold(0u32, |v, &b| v << 8 | b as u32);
    let next = address.wrapping_add(1 + n) & 0xFFFF;
    let here = address & 0xFF_0000;
    let operands = match mode {
        Imp => String::new(),
        Acc => "A".into(),
        ImmM | ImmX if n == 2 => format!("#${v:04X}"),
        ImmM | ImmX | Imm8 => format!("#${v:02X}"),
        Dp => format!("${v:02X}"),
        DpX => format!("${v:02X},X"),
        DpY => format!("${v:02X},Y"),
        DpInd => format!("(${v:02X})"),
        DpIndX => format!("(${v:02X},X)"),
        DpIndY => format!("(${v:02X}),Y"),
        DpLong => format!("[${v:02X}]"),
        DpLongY => format!("[${v:02X}],Y"),
        Sr => format!("${v:02X},S"),
        SrIndY => format!("(${v:02X},S),Y"),
        Abs => format!("${v:04X}"),
        AbsX => format!("${v:04X},X"),
        AbsY => format!("${v:04X},Y"),
        Long => format!("${v:06X}"),
        LongX => format!("${v:06X},X"),
        AbsInd => format!("(${v:04X})"),
        AbsIndX => format!("(${v:04X},X)"),
        AbsLong => format!("[${v:04X}]"),
        Rel => format!("${:04X}", next.wrapping_add(v as u8 as i8 as u32) & 0xFFFF),
        RelLong => format!("${:04X}", next.wrapping_add(v) & 0xFFFF),
        // The block moves' bytes are the destination's bank, then the source's; the source is written first.
        Move => format!("${:02X},${:02X}", v >> 8, v & 0xFF),
    };
    let stores = matches!(mnemonic, "STA" | "STX" | "STY" | "STZ" | "ASL" | "LSR" | "ROL" | "ROR" | "INC" | "DEC" | "TSB" | "TRB");
    let reads = matches!(mnemonic, "LDA" | "LDX" | "LDY" | "ORA" | "AND" | "EOR" | "ADC" | "SBC" | "CMP" | "CPX" | "CPY" | "BIT");
    let reference = match (mnemonic, mode) {
        ("JSR", Abs) => Some((Reference::Call, here | v)),
        ("JSL", Long) => Some((Reference::Call, v)),
        (_, Abs) if stores => Some((Reference::Write, (bank as u32) << 16 | v)),
        (_, Long) if stores => Some((Reference::Write, v)),
        (_, Abs) if reads => Some((Reference::Read, (bank as u32) << 16 | v)),
        (_, Long) if reads => Some((Reference::Read, v)),
        _ => None,
    };
    if !w.e {
        match opcode {
            0xC2 => (w.m, w.x) = (w.m && v & 0x20 == 0, w.x && v & 0x10 == 0),
            0xE2 => (w.m, w.x) = (w.m || v & 0x20 != 0, w.x || v & 0x10 != 0),
            _ => {}
        }
    }
    Instruction { address, bytes, mnemonic: mnemonic.into(), operands, reference }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::{self, Cpu};

    fn text(bytes: &[u8], at: u32, w: &mut Widths) -> (String, usize, Option<(Reference, u32)>) {
        let read = |a: u32| bytes.get((a.wrapping_sub(at) & 0xFFFF) as usize).copied().unwrap_or(0);
        let i = decode(&read, at, w, 0x7E);
        (format!("{} {}", i.mnemonic, i.operands).trim_end().to_owned(), i.bytes.len(), i.reference)
    }

    // The datasheet's formats, an immediate as wide as its register, REP and SEP moving the widths, and the references.
    #[test]
    fn instructions_read_as_the_datasheet_writes_them() {
        let mut w = Widths { m: true, x: true, e: false };
        assert_eq!(text(&[0xA9, 0x12], 0x8000, &mut w), ("LDA #$12".into(), 2, None));
        assert_eq!(text(&[0xC2, 0x30], 0x8000, &mut w).0, "REP #$30");
        assert_eq!((w.m, w.x), (false, false));
        assert_eq!(text(&[0xA9, 0x34, 0x12], 0x8000, &mut w), ("LDA #$1234".into(), 3, None));
        assert_eq!(text(&[0xA2, 0x34, 0x12], 0x8000, &mut w).0, "LDX #$1234");
        assert_eq!(text(&[0xE2, 0x20], 0x8000, &mut w).0, "SEP #$20");
        assert_eq!((w.m, w.x), (true, false));
        assert_eq!(text(&[0x20, 0x10, 0x80], 0x01_8000, &mut w), ("JSR $8010".into(), 3, Some((Reference::Call, 0x01_8010))));
        assert_eq!(text(&[0x22, 0x56, 0x34, 0x12], 0x8000, &mut w), ("JSL $123456".into(), 4, Some((Reference::Call, 0x12_3456))));
        assert_eq!(text(&[0x8D, 0x00, 0x21], 0x8000, &mut w), ("STA $2100".into(), 3, Some((Reference::Write, 0x7E_2100))));
        assert_eq!(text(&[0xAF, 0x00, 0x00, 0x7F], 0x8000, &mut w), ("LDA $7F0000".into(), 4, Some((Reference::Read, 0x7F_0000))));
        assert_eq!(text(&[0x80, 0xFE], 0x8000, &mut w).0, "BRA $8000");
        assert_eq!(text(&[0x82, 0xFD, 0xFF], 0x8000, &mut w).0, "BRL $8000");
        assert_eq!(text(&[0x54, 0x7F, 0x7E], 0x8000, &mut w).0, "MVN $7E,$7F");
        assert_eq!(text(&[0xB7, 0x10], 0x8000, &mut w).0, "LDA [$10],Y");
        assert_eq!(text(&[0x93, 0x03], 0x8000, &mut w).0, "STA ($03,S),Y");
        assert_eq!(text(&[0xFC, 0x00, 0x90], 0x8000, &mut w).0, "JSR ($9000,X)");
        assert_eq!(text(&[0xDC, 0x00, 0x90], 0x8000, &mut w).0, "JML [$9000]");
        assert_eq!(text(&[0x0A], 0x8000, &mut w).0, "ASL A");
    }

    /// One opcode over a bus of zero operands.
    struct One(u8);

    impl cpu::Bus for One {
        fn read(&mut self, a: u32, _: u8) -> u8 {
            if a == 0x00_8000 { self.0 } else { 0 }
        }
        fn write(&mut self, _: u32, _: u8, _: u8) {}
        fn idle(&mut self, _: u32, _: u8) {}
        fn halted(&mut self) {}
    }

    // Every opcode that falls through to the next instruction is as long here as the core, which the single-step
    // suite grades, finds it, at each width of the accumulator and the index registers.
    #[test]
    fn each_instructions_length_is_the_cores() {
        let flow = [0x00, 0x02, 0x10, 0x20, 0x22, 0x30, 0x40, 0x4C, 0x50, 0x5C, 0x60, 0x6B, 0x6C, 0x70, 0x7C, 0x80, 0x82, 0x90, 0xB0, 0xD0, 0xDC, 0xF0, 0xFC, 0xCB, 0xDB, 0x44, 0x54];
        for p in [0x00u8, 0x10, 0x20, 0x30] {
            for opcode in 0..=255u8 {
                if flow.contains(&opcode) {
                    continue;
                }
                let mut c = Cpu { pc: 0x8000, p, s: 0x01F0, ..Cpu::default() };
                c.step(&mut One(opcode));
                let mut w = Widths { m: p & 0x20 != 0, x: p & 0x10 != 0, e: false };
                let read = |a: u32| if a == 0x8000 { opcode } else { 0 };
                let i = decode(&read, 0x8000, &mut w, 0);
                assert_eq!(c.pc.wrapping_sub(0x8000) as usize, i.bytes.len(), "opcode {opcode:02X} at P={p:02X}: {} {}", i.mnemonic, i.operands);
            }
        }
    }
}
