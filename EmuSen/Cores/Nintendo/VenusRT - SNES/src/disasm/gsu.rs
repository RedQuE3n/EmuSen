//! The GSU's instructions by fullsnes's "SNES Cart GSU-n CPU" tables. An opcode's meaning depends on the ALT1 and
//! ALT2 prefixes and on WITH's B flag before it, so a listing carries that state from byte to byte; each prefix
//! is listed as an instruction of its own.

use super::{Instruction, Reference};

/// What the bytes before set: ALT1 (1), ALT2 (2) or both, and WITH's B flag.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Prefix {
    pub alt: u8,
    pub b: bool,
}

const BRANCH: [&str; 11] = ["BRA", "BGE", "BLT", "BNE", "BEQ", "BPL", "BMI", "BCC", "BCS", "BVC", "BVS"];

/// The instruction at `address` in its bank, and the prefix state after it.
pub fn decode(read: &dyn Fn(u32) -> u8, address: u32, prefix: &mut Prefix) -> Instruction {
    let at = |i: u32| (address & 0xFF_0000) | (address.wrapping_add(i) & 0xFFFF);
    let op = read(at(0));
    let n = op & 0x0F;
    let (alt1, alt2) = (prefix.alt & 1 != 0, prefix.alt & 2 != 0);
    // By ALT: none, ALT1, ALT2, ALT3; a form that does not exist falls back as fullsnes's "Ignored Prefixes" has it.
    let by_alt = |forms: [&'static str; 4]| forms[prefix.alt as usize & 3];
    let mut length = 1;
    let mut keep = false;
    let mut reference = None;
    let word = read(at(1)) as u32 | (read(at(2)) as u32) << 8;
    let (mnemonic, operands): (&str, String) = match op {
        0x00 => ("STOP", String::new()),
        0x01 => ("NOP", String::new()),
        0x02 => ("CACHE", String::new()),
        0x03 => ("LSR", String::new()),
        0x04 => ("ROL", String::new()),
        0x05..=0x0F => {
            length = 2;
            keep = true;
            (BRANCH[op as usize - 5], format!("${:04X}", address.wrapping_add(2).wrapping_add(read(at(1)) as i8 as u32) & 0xFFFF))
        }
        0x10..=0x1F if prefix.b => ("MOVE", format!("R{n}")),
        0x10..=0x1F => {
            keep = true;
            ("TO", format!("R{n}"))
        }
        0x20..=0x2F => {
            keep = true;
            prefix.b = true;
            ("WITH", format!("R{n}"))
        }
        0x30..=0x3B => (if alt1 { "STB" } else { "STW" }, format!("(R{n})")),
        0x3C => ("LOOP", String::new()),
        0x3D..=0x3F => {
            keep = true;
            prefix.alt |= op - 0x3C;
            (["ALT1", "ALT2", "ALT3"][op as usize - 0x3D], String::new())
        }
        0x40..=0x4B => (if alt1 { "LDB" } else { "LDW" }, format!("(R{n})")),
        0x4C => (if alt1 { "RPIX" } else { "PLOT" }, String::new()),
        0x4D => ("SWAP", String::new()),
        0x4E => (if alt1 { "CMODE" } else { "COLOR" }, String::new()),
        0x4F => ("NOT", String::new()),
        0x50..=0x5F => (by_alt(["ADD", "ADC", "ADD", "ADC"]), if alt2 { format!("#{n}") } else { format!("R{n}") }),
        0x60..=0x6F => (by_alt(["SUB", "SBC", "SUB", "CMP"]), if alt2 && !alt1 { format!("#{n}") } else { format!("R{n}") }),
        0x70 => ("MERGE", String::new()),
        0x71..=0x7F => (by_alt(["AND", "BIC", "AND", "BIC"]), if alt2 { format!("#{n}") } else { format!("R{n}") }),
        0x80..=0x8F => (by_alt(["MULT", "UMULT", "MULT", "UMULT"]), if alt2 { format!("#{n}") } else { format!("R{n}") }),
        0x90 => ("SBK", String::new()),
        0x91..=0x94 => ("LINK", format!("#{n}")),
        0x95 => ("SEX", String::new()),
        0x96 => (if alt1 { "DIV2" } else { "ASR" }, String::new()),
        0x97 => ("ROR", String::new()),
        0x98..=0x9D => (if alt1 { "LJMP" } else { "JMP" }, format!("R{n}")),
        0x9E => ("LOB", String::new()),
        0x9F => (if alt1 { "LMULT" } else { "FMULT" }, String::new()),
        0xA0..=0xAF => {
            length = 2;
            let k = read(at(1)) as u32;
            match prefix.alt {
                0 => ("IBT", format!("R{n},#${k:02X}")),
                2 => {
                    reference = Some((Reference::Write, 0x70_0000 | k << 1));
                    ("SMS", format!("(${:03X}),R{n}", k << 1))
                }
                _ => {
                    reference = Some((Reference::Read, 0x70_0000 | k << 1));
                    ("LMS", format!("R{n},(${:03X})", k << 1))
                }
            }
        }
        0xB0..=0xBF if prefix.b => ("MOVES", format!("R{n}")),
        0xB0..=0xBF => {
            keep = true;
            ("FROM", format!("R{n}"))
        }
        0xC0 => ("HIB", String::new()),
        0xC1..=0xCF => (by_alt(["OR", "XOR", "OR", "XOR"]), if alt2 { format!("#{n}") } else { format!("R{n}") }),
        0xD0..=0xDE => ("INC", format!("R{n}")),
        0xDF => (by_alt(["GETC", "GETC", "RAMB", "ROMB"]), String::new()),
        0xE0..=0xEE => ("DEC", format!("R{n}")),
        0xEF => (by_alt(["GETB", "GETBH", "GETBL", "GETBS"]), String::new()),
        0xF0..=0xFF => {
            length = 3;
            match prefix.alt {
                0 => ("IWT", format!("R{n},#${word:04X}")),
                2 => {
                    reference = Some((Reference::Write, 0x70_0000 | word));
                    ("SM", format!("(${word:04X}),R{n}"))
                }
                _ => {
                    reference = Some((Reference::Read, 0x70_0000 | word));
                    ("LM", format!("R{n},(${word:04X})"))
                }
            }
        }
    };
    if !keep {
        *prefix = Prefix::default();
    }
    let bytes = super::bytes(read, address, length, |a, i| (a & 0xFF_0000) | (a.wrapping_add(i) & 0xFFFF));
    Instruction { address, bytes, mnemonic: mnemonic.into(), operands, reference }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listing(bytes: &[u8]) -> Vec<String> {
        let read = |a: u32| bytes.get((a & 0xFFFF) as usize).copied().unwrap_or(1);
        let (mut at, mut prefix, mut out) = (0u32, Prefix::default(), Vec::new());
        while (at as usize) < bytes.len() {
            let i = decode(&read, at, &mut prefix);
            at += i.bytes.len() as u32;
            out.push(format!("{} {}", i.mnemonic, i.operands).trim_end().to_owned());
        }
        out
    }

    // fullsnes's tables: the prefixes change the opcode after them and no further, WITH turns 1n and Bn into moves,
    // and a branch leaves the prefixes standing.
    #[test]
    fn prefixes_select_the_opcode_that_follows() {
        assert_eq!(listing(&[0x51, 0x3D, 0x51, 0x3E, 0x51, 0x3F, 0x51]), ["ADD R1", "ALT1", "ADC R1", "ALT2", "ADD #1", "ALT3", "ADC #1"]);
        assert_eq!(listing(&[0x3F, 0x62, 0x3E, 0x62, 0x3D, 0x62]), ["ALT3", "CMP R2", "ALT2", "SUB #2", "ALT1", "SBC R2"]);
        assert_eq!(listing(&[0x21, 0x13, 0x21, 0xB3, 0x13, 0xB3]), ["WITH R1", "MOVE R3", "WITH R1", "MOVES R3", "TO R3", "FROM R3"]);
        assert_eq!(listing(&[0xA1, 0x80, 0xF2, 0x34, 0x12]), ["IBT R1,#$80", "IWT R2,#$1234"]);
        assert_eq!(listing(&[0x3D, 0xA1, 0x10, 0x3E, 0xF2, 0x34, 0x12]), ["ALT1", "LMS R1,($020)", "ALT2", "SM ($1234),R2"]);
        assert_eq!(listing(&[0x3D, 0x05, 0x02, 0x41, 0x41]), ["ALT1", "BRA $0005", "LDB (R1)", "LDW (R1)"]);
        assert_eq!(listing(&[0x3D, 0x4C, 0x4C, 0x3D, 0x9F, 0x9F, 0x3D, 0x98, 0x98]), ["ALT1", "RPIX", "PLOT", "ALT1", "LMULT", "FMULT", "ALT1", "LJMP R8", "JMP R8"]);
        assert_eq!(listing(&[0xDF, 0x3E, 0xDF, 0x3F, 0xDF, 0x3D, 0xEF, 0x3E, 0xEF, 0x3F, 0xEF]), ["GETC", "ALT2", "RAMB", "ALT3", "ROMB", "ALT1", "GETBH", "ALT2", "GETBL", "ALT3", "GETBS"]);
    }
}
