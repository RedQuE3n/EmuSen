//! The SM83's instructions as text, a second opcode table independent of any executing one, shared by the core that
//! decodes for its debugger and the Game Boy's pack (Mercury_Debug.md §4, EmuSen_Backend.md §17.2 D10, F8).

#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};

/// What an instruction names that its bytes alone give: a jump or call's target, a store's or a load's address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reference {
    Call,
    Read,
    Write,
}

/// What an opcode with no instruction behind it prints as.
pub const ILLEGAL: &str = "???";

/// The whole CB page is two bytes: the prefix and the operation.
pub const CB_LENGTH: usize = 2;

/// Each opcode's template; a placeholder fixes its operand's form and the instruction's length.
const TEMPLATES: [&str; 256] = [
    "NOP", "LD BC,d16", "LD (BC),A", "INC BC", "INC B", "DEC B", "LD B,d8", "RLCA",
    "LD (a16),SP", "ADD HL,BC", "LD A,(BC)", "DEC BC", "INC C", "DEC C", "LD C,d8", "RRCA",
    "STOP d8", "LD DE,d16", "LD (DE),A", "INC DE", "INC D", "DEC D", "LD D,d8", "RLA",
    "JR r8", "ADD HL,DE", "LD A,(DE)", "DEC DE", "INC E", "DEC E", "LD E,d8", "RRA",
    "JR NZ,r8", "LD HL,d16", "LD (HL+),A", "INC HL", "INC H", "DEC H", "LD H,d8", "DAA",
    "JR Z,r8", "ADD HL,HL", "LD A,(HL+)", "DEC HL", "INC L", "DEC L", "LD L,d8", "CPL",
    "JR NC,r8", "LD SP,d16", "LD (HL-),A", "INC SP", "INC (HL)", "DEC (HL)", "LD (HL),d8", "SCF",
    "JR C,r8", "ADD HL,SP", "LD A,(HL-)", "DEC SP", "INC A", "DEC A", "LD A,d8", "CCF",
    "LD B,B", "LD B,C", "LD B,D", "LD B,E", "LD B,H", "LD B,L", "LD B,(HL)", "LD B,A",
    "LD C,B", "LD C,C", "LD C,D", "LD C,E", "LD C,H", "LD C,L", "LD C,(HL)", "LD C,A",
    "LD D,B", "LD D,C", "LD D,D", "LD D,E", "LD D,H", "LD D,L", "LD D,(HL)", "LD D,A",
    "LD E,B", "LD E,C", "LD E,D", "LD E,E", "LD E,H", "LD E,L", "LD E,(HL)", "LD E,A",
    "LD H,B", "LD H,C", "LD H,D", "LD H,E", "LD H,H", "LD H,L", "LD H,(HL)", "LD H,A",
    "LD L,B", "LD L,C", "LD L,D", "LD L,E", "LD L,H", "LD L,L", "LD L,(HL)", "LD L,A",
    "LD (HL),B", "LD (HL),C", "LD (HL),D", "LD (HL),E", "LD (HL),H", "LD (HL),L", "HALT", "LD (HL),A",
    "LD A,B", "LD A,C", "LD A,D", "LD A,E", "LD A,H", "LD A,L", "LD A,(HL)", "LD A,A",
    "ADD A,B", "ADD A,C", "ADD A,D", "ADD A,E", "ADD A,H", "ADD A,L", "ADD A,(HL)", "ADD A,A",
    "ADC A,B", "ADC A,C", "ADC A,D", "ADC A,E", "ADC A,H", "ADC A,L", "ADC A,(HL)", "ADC A,A",
    "SUB B", "SUB C", "SUB D", "SUB E", "SUB H", "SUB L", "SUB (HL)", "SUB A",
    "SBC A,B", "SBC A,C", "SBC A,D", "SBC A,E", "SBC A,H", "SBC A,L", "SBC A,(HL)", "SBC A,A",
    "AND B", "AND C", "AND D", "AND E", "AND H", "AND L", "AND (HL)", "AND A",
    "XOR B", "XOR C", "XOR D", "XOR E", "XOR H", "XOR L", "XOR (HL)", "XOR A",
    "OR B", "OR C", "OR D", "OR E", "OR H", "OR L", "OR (HL)", "OR A",
    "CP B", "CP C", "CP D", "CP E", "CP H", "CP L", "CP (HL)", "CP A",
    "RET NZ", "POP BC", "JP NZ,a16", "JP a16", "CALL NZ,a16", "PUSH BC", "ADD A,d8", "RST 00H",
    "RET Z", "RET", "JP Z,a16", "PREFIX CB", "CALL Z,a16", "CALL a16", "ADC A,d8", "RST 08H",
    "RET NC", "POP DE", "JP NC,a16", "???", "CALL NC,a16", "PUSH DE", "SUB d8", "RST 10H",
    "RET C", "RETI", "JP C,a16", "???", "CALL C,a16", "???", "SBC A,d8", "RST 18H",
    "LDH (a8),A", "POP HL", "LD (C),A", "???", "???", "PUSH HL", "AND d8", "RST 20H",
    "ADD SP,s8", "JP (HL)", "LD (a16),A", "???", "???", "???", "XOR d8", "RST 28H",
    "LDH A,(a8)", "POP AF", "LD A,(C)", "DI", "???", "PUSH AF", "OR d8", "RST 30H",
    "LD HL,SPs8", "LD SP,HL", "LD A,(a16)", "EI", "???", "???", "CP d8", "RST 38H",
];

const CB_OPERATIONS: [&str; 8] = ["RLC", "RRC", "RL", "RR", "SLA", "SRA", "SWAP", "SRL"];
const CB_TARGETS: [&str; 8] = ["B", "C", "D", "E", "H", "L", "(HL)", "A"];

pub fn length_of(opcode: u8) -> usize {
    if opcode == 0xCB {
        return CB_LENGTH;
    }
    let t = TEMPLATES[opcode as usize];
    if t.contains("d16") || t.contains("a16") {
        3
    } else if t.contains("d8") || t.contains("a8") || t.contains("r8") || t.contains("s8") {
        2
    } else {
        1
    }
}

pub fn mnemonic_of(opcode: u8, suffix: u8) -> String {
    if opcode == 0xCB {
        return cb_mnemonic(suffix).to_string();
    }
    let t = TEMPLATES[opcode as usize];
    t.split_once(' ').map_or(t, |(m, _)| m).to_string()
}

/// The operands as text, each placeholder replaced in the C# table's order; `address` resolves a relative branch.
pub fn format_operand(opcode: u8, address: i32, low: u8, high: u8) -> String {
    if opcode == 0xCB {
        return cb_operand(low);
    }
    let Some((_, operands)) = TEMPLATES[opcode as usize].split_once(' ') else { return String::new() };
    let word = low as u16 | (high as u16) << 8;
    let signed = low as i8;
    operands
        .replace("d16", &format!("${word:04X}"))
        .replace("a16", &format!("${word:04X}"))
        .replace("a8", &format!("$FF{low:02X}"))
        .replace("r8", &format!("${:04X}", address.wrapping_add(2).wrapping_add(signed as i32) as u16))
        .replace("s8", &if signed < 0 { format!("-{}", (signed as i32).unsigned_abs()) } else { format!("+{signed}") })
        .replace("d8", &format!("${low:02X}"))
}

fn cb_mnemonic(suffix: u8) -> &'static str {
    match suffix {
        0x00..=0x3F => CB_OPERATIONS[(suffix >> 3) as usize],
        0x40..=0x7F => "BIT",
        0x80..=0xBF => "RES",
        _ => "SET",
    }
}

fn cb_operand(suffix: u8) -> String {
    let target = CB_TARGETS[(suffix & 7) as usize];
    if suffix < 0x40 { target.to_string() } else { format!("{},{target}", (suffix >> 3) & 7) }
}

/// Only the forms whose target the bytes alone give.
pub fn classify(opcode: u8, operand: i32, address: i32) -> Option<(Reference, i32)> {
    match opcode {
        0xC4 | 0xCC | 0xCD | 0xD4 | 0xDC | 0xC2 | 0xC3 | 0xCA | 0xD2 | 0xDA => Some((Reference::Call, operand & 0xFFFF)),
        0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => Some((Reference::Call, (opcode & 0x38) as i32)),
        0x18 | 0x20 | 0x28 | 0x30 | 0x38 => Some((Reference::Call, address.wrapping_add(2).wrapping_add(operand as i8 as i32) as u16 as i32)),
        0x08 | 0xEA => Some((Reference::Write, operand & 0xFFFF)),
        0xE0 => Some((Reference::Write, 0xFF00 | (operand & 0xFF))),
        0xFA => Some((Reference::Read, operand & 0xFFFF)),
        0xF0 => Some((Reference::Read, 0xFF00 | (operand & 0xFF))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lengths_follow_the_operand_placeholders() {
        assert_eq!((length_of(0x00), length_of(0x06), length_of(0xC3), length_of(0xCB), length_of(0x18)), (1, 2, 3, 2, 2));
    }

    #[test]
    fn a_relative_branch_and_a_cb_operation_read_as_the_debugger_prints_them() {
        assert_eq!((mnemonic_of(0x18, 0xFE).as_str(), format_operand(0x18, 0x0150, 0xFE, 0).as_str()), ("JR", "$0150"));
        assert_eq!((mnemonic_of(0xCB, 0x7C).as_str(), format_operand(0xCB, 0, 0x7C, 0).as_str()), ("BIT", "7,H"));
        assert_eq!(format_operand(0xE8, 0, 0xF0, 0), "SP,-16");
        assert_eq!(mnemonic_of(0xD3, 0), ILLEGAL);
    }

    #[test]
    fn only_targets_the_bytes_give_are_classified() {
        assert_eq!(classify(0xCD, 0x1234, 0), Some((Reference::Call, 0x1234)));
        assert_eq!(classify(0xE0, 0x44, 0), Some((Reference::Write, 0xFF44)));
        assert_eq!(classify(0xE9, 0, 0), None);
    }
}
