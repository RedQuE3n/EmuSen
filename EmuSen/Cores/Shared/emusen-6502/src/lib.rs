//! The 2A03's instructions as text, independent of any engine's executing table, the undocumented ones by their common
//! names: one table that MoonRT's `debug_disassemble` and the NES pack's disassembler both read (Moon_Debug.md §5,
//! Moon_CPU.md §6, EmuSen_Backend.md §21).

/// What an instruction does with a target its bytes name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reference {
    Call,
    Read,
    Write,
}

/// How an instruction addresses its operand, `Nes6502AddressMode`'s members in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Implied,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndexedIndirect,
    IndirectIndexed,
    Relative,
}

use Mode::{Absolute as Abs, AbsoluteX as Abx, AbsoluteY as Aby, Accumulator as Acc, Immediate as Imm, Implied as Imp, IndexedIndirect as Izx, IndirectIndexed as Izy, Indirect as Ind, Relative as Rel, ZeroPage as Zp, ZeroPageX as Zpx, ZeroPageY as Zpy};

/// The 256 mnemonics.
pub const NAMES: [&str; 256] = [
    "BRK", "ORA", "JAM", "SLO", "NOP", "ORA", "ASL", "SLO", "PHP", "ORA", "ASL", "ANC", "NOP", "ORA", "ASL", "SLO",
    "BPL", "ORA", "JAM", "SLO", "NOP", "ORA", "ASL", "SLO", "CLC", "ORA", "NOP", "SLO", "NOP", "ORA", "ASL", "SLO",
    "JSR", "AND", "JAM", "RLA", "BIT", "AND", "ROL", "RLA", "PLP", "AND", "ROL", "ANC", "BIT", "AND", "ROL", "RLA",
    "BMI", "AND", "JAM", "RLA", "NOP", "AND", "ROL", "RLA", "SEC", "AND", "NOP", "RLA", "NOP", "AND", "ROL", "RLA",
    "RTI", "EOR", "JAM", "SRE", "NOP", "EOR", "LSR", "SRE", "PHA", "EOR", "LSR", "ALR", "JMP", "EOR", "LSR", "SRE",
    "BVC", "EOR", "JAM", "SRE", "NOP", "EOR", "LSR", "SRE", "CLI", "EOR", "NOP", "SRE", "NOP", "EOR", "LSR", "SRE",
    "RTS", "ADC", "JAM", "RRA", "NOP", "ADC", "ROR", "RRA", "PLA", "ADC", "ROR", "ARR", "JMP", "ADC", "ROR", "RRA",
    "BVS", "ADC", "JAM", "RRA", "NOP", "ADC", "ROR", "RRA", "SEI", "ADC", "NOP", "RRA", "NOP", "ADC", "ROR", "RRA",
    "NOP", "STA", "NOP", "SAX", "STY", "STA", "STX", "SAX", "DEY", "NOP", "TXA", "ANE", "STY", "STA", "STX", "SAX",
    "BCC", "STA", "JAM", "SHA", "STY", "STA", "STX", "SAX", "TYA", "STA", "TXS", "TAS", "SHY", "STA", "SHX", "SHA",
    "LDY", "LDA", "LDX", "LAX", "LDY", "LDA", "LDX", "LAX", "TAY", "LDA", "TAX", "LXA", "LDY", "LDA", "LDX", "LAX",
    "BCS", "LDA", "JAM", "LAX", "LDY", "LDA", "LDX", "LAX", "CLV", "LDA", "TSX", "LAS", "LDY", "LDA", "LDX", "LAX",
    "CPY", "CMP", "NOP", "DCP", "CPY", "CMP", "DEC", "DCP", "INY", "CMP", "DEX", "SBX", "CPY", "CMP", "DEC", "DCP",
    "BNE", "CMP", "JAM", "DCP", "NOP", "CMP", "DEC", "DCP", "CLD", "CMP", "NOP", "DCP", "NOP", "CMP", "DEC", "DCP",
    "CPX", "SBC", "NOP", "ISC", "CPX", "SBC", "INC", "ISC", "INX", "SBC", "NOP", "USBC", "CPX", "SBC", "INC", "ISC",
    "BEQ", "SBC", "JAM", "ISC", "NOP", "SBC", "INC", "ISC", "SED", "SBC", "NOP", "ISC", "NOP", "SBC", "INC", "ISC",
];

const MODES: [Mode; 256] = [
    Imp, Izx, Imp, Izx, Zp, Zp, Zp, Zp, Imp, Imm, Acc, Imm, Abs, Abs, Abs, Abs,
    Rel, Izy, Imp, Izy, Zpx, Zpx, Zpx, Zpx, Imp, Aby, Imp, Aby, Abx, Abx, Abx, Abx,
    Abs, Izx, Imp, Izx, Zp, Zp, Zp, Zp, Imp, Imm, Acc, Imm, Abs, Abs, Abs, Abs,
    Rel, Izy, Imp, Izy, Zpx, Zpx, Zpx, Zpx, Imp, Aby, Imp, Aby, Abx, Abx, Abx, Abx,
    Imp, Izx, Imp, Izx, Zp, Zp, Zp, Zp, Imp, Imm, Acc, Imm, Abs, Abs, Abs, Abs,
    Rel, Izy, Imp, Izy, Zpx, Zpx, Zpx, Zpx, Imp, Aby, Imp, Aby, Abx, Abx, Abx, Abx,
    Imp, Izx, Imp, Izx, Zp, Zp, Zp, Zp, Imp, Imm, Acc, Imm, Ind, Abs, Abs, Abs,
    Rel, Izy, Imp, Izy, Zpx, Zpx, Zpx, Zpx, Imp, Aby, Imp, Aby, Abx, Abx, Abx, Abx,
    Imm, Izx, Imm, Izx, Zp, Zp, Zp, Zp, Imp, Imm, Imp, Imm, Abs, Abs, Abs, Abs,
    Rel, Izy, Imp, Izy, Zpx, Zpx, Zpy, Zpy, Imp, Aby, Imp, Aby, Abx, Abx, Aby, Aby,
    Imm, Izx, Imm, Izx, Zp, Zp, Zp, Zp, Imp, Imm, Imp, Imm, Abs, Abs, Abs, Abs,
    Rel, Izy, Imp, Izy, Zpx, Zpx, Zpy, Zpy, Imp, Aby, Imp, Aby, Abx, Abx, Aby, Aby,
    Imm, Izx, Imm, Izx, Zp, Zp, Zp, Zp, Imp, Imm, Imp, Imm, Abs, Abs, Abs, Abs,
    Rel, Izy, Imp, Izy, Zpx, Zpx, Zpx, Zpx, Imp, Aby, Imp, Aby, Abx, Abx, Abx, Abx,
    Imm, Izx, Imm, Izx, Zp, Zp, Zp, Zp, Imp, Imm, Imp, Imm, Abs, Abs, Abs, Abs,
    Rel, Izy, Imp, Izy, Zpx, Zpx, Zpx, Zpx, Imp, Aby, Imp, Aby, Abx, Abx, Abx, Abx,
];

pub fn mode_of(opcode: u8) -> Mode {
    MODES[opcode as usize]
}

pub fn mnemonic_of(opcode: u8) -> &'static str {
    NAMES[opcode as usize]
}

pub fn length_of(opcode: u8) -> usize {
    match mode_of(opcode) {
        Imp | Acc => 1,
        Abs | Abx | Aby | Ind => 3,
        _ => 2,
    }
}

/// The operand as text; `address` resolves a branch, which counts from the next instruction.
pub fn format_operand(opcode: u8, address: i32, low: u8, high: u8) -> String {
    let word = low as u16 | (high as u16) << 8;
    match mode_of(opcode) {
        Imp => String::new(),
        Acc => "A".into(),
        Imm => format!("#${low:02X}"),
        Zp => format!("${low:02X}"),
        Zpx => format!("${low:02X},X"),
        Zpy => format!("${low:02X},Y"),
        Abs => format!("${word:04X}"),
        Abx => format!("${word:04X},X"),
        Aby => format!("${word:04X},Y"),
        Ind => format!("(${word:04X})"),
        Izx => format!("(${low:02X},X)"),
        Izy => format!("(${low:02X}),Y"),
        Rel => format!("${:04X}", address.wrapping_add(2).wrapping_add(low as i8 as i32) as u16),
    }
}

/// Only the forms whose target the bytes alone give: an absolute or zero-page operand, by what the instruction does with it.
pub fn classify(opcode: u8, operand: i32) -> Option<(Reference, i32)> {
    if !matches!(mode_of(opcode), Abs | Zp) {
        return None;
    }
    if opcode == 0x20 || opcode == 0x4C {
        return Some((Reference::Call, operand));
    }
    match mnemonic_of(opcode) {
        "STA" | "STX" | "STY" | "SAX" | "INC" | "DEC" | "ASL" | "LSR" | "ROL" | "ROR" => Some((Reference::Write, operand)),
        "LDA" | "LDX" | "LDY" | "CMP" | "CPX" | "CPY" | "ADC" | "SBC" | "AND" | "ORA" | "EOR" | "BIT" => Some((Reference::Read, operand)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lengths_follow_the_modes() {
        assert_eq!((length_of(0xEA), length_of(0x0A), length_of(0xA9), length_of(0xAD), length_of(0x6C), length_of(0xD0)), (1, 1, 2, 3, 3, 2));
    }

    #[test]
    fn operands_are_written_as_the_debugger_shows_them() {
        assert_eq!(format_operand(0xA9, 0x8000, 0x3F, 0), "#$3F");
        assert_eq!(format_operand(0xBD, 0x8000, 0x34, 0x12), "$1234,X");
        assert_eq!(format_operand(0x6C, 0x8000, 0xFC, 0xFF), "($FFFC)");
        assert_eq!(format_operand(0xB1, 0x8000, 0x10, 0), "($10),Y");
        assert_eq!(format_operand(0xD0, 0x8000, 0xFE, 0), "$8000");
        assert_eq!(format_operand(0x0A, 0x8000, 0, 0), "A");
        assert_eq!(mnemonic_of(0xEB), "USBC");
    }

    #[test]
    fn only_absolute_and_zero_page_operands_are_references() {
        assert_eq!(classify(0x20, 0xC000), Some((Reference::Call, 0xC000)));
        assert_eq!(classify(0x8D, 0x2000), Some((Reference::Write, 0x2000)));
        assert_eq!(classify(0xA5, 0x10), Some((Reference::Read, 0x10)));
        assert_eq!(classify(0xBD, 0x1234), None);
        assert_eq!(classify(0x6C, 0xFFFC), None);
    }
}
