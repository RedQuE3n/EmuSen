//! The µPD77C25's and µPD96050's 24-bit instructions by fullsnes's "NEC uPD77C25" tables: the ALU instruction with
//! its move, pointer adjustments and return, the load of an immediate, and the jumps. An address here is the
//! opcode's byte offset in the program, three bytes each, low byte first.

use super::{Instruction, Reference};

const ALU: [&str; 16] = ["NOP", "OR", "AND", "XOR", "SUB", "ADD", "SBB", "ADC", "DEC", "INC", "NOT", "SAR1", "RCL1", "SLL2", "SLL4", "XCHG"];
const SRC: [&str; 16] = ["TRB", "A", "B", "TR", "DP", "RP", "RO", "SGN", "DR", "DRNF", "SR", "SIM", "SIL", "K", "L", "MEM"];
const DST: [&str; 16] = ["@NON", "@A", "@B", "@TR", "@DP", "@RP", "@DR", "@SR", "@SOL", "@SOM", "@K", "@KLR", "@KLM", "@L", "@TRB", "@MEM"];
/// The ALU's P input: RAM[DP], the internal data bus (SRC), and the product's high and low halves.
const P: [&str; 4] = ["RAM", "IDB", "M", "N"];

/// The jump's name by its nine-bit BRCH field, or None for a pattern the table does not list.
fn jump(brch: u32) -> Option<&'static str> {
    Some(match brch {
        0x000 => "JMPSO",
        0x100 | 0x101 => "JMP",
        0x140 | 0x141 => "CALL",
        0x080 => "JNCA",
        0x082 => "JCA",
        0x084 => "JNCB",
        0x086 => "JCB",
        0x088 => "JNZA",
        0x08A => "JZA",
        0x08C => "JNZB",
        0x08E => "JZB",
        0x090 => "JNOVA0",
        0x092 => "JOVA0",
        0x094 => "JNOVB0",
        0x096 => "JOVB0",
        0x098 => "JNOVA1",
        0x09A => "JOVA1",
        0x09C => "JNOVB1",
        0x09E => "JOVB1",
        0x0A0 => "JNSA0",
        0x0A2 => "JSA0",
        0x0A4 => "JNSB0",
        0x0A6 => "JSB0",
        0x0A8 => "JNSA1",
        0x0AA => "JSA1",
        0x0AC => "JNSB1",
        0x0AE => "JSB1",
        0x0B0 => "JDPL0",
        0x0B1 => "JDPLN0",
        0x0B2 => "JDPLF",
        0x0B3 => "JDPLNF",
        0x0B4 => "JNSIAK",
        0x0B6 => "JSIAK",
        0x0B8 => "JNSOAK",
        0x0BA => "JSOAK",
        0x0BC => "JNRQM",
        0x0BE => "JRQM",
        _ => return None,
    })
}

/// The instruction whose three bytes start at `address` (taken down to a multiple of three); `st` is the µPD96050,
/// whose jumps carry two more address bits.
pub fn decode(read: &dyn Fn(u32) -> u8, address: u32, st: bool) -> Instruction {
    let address = address - address % 3;
    let bytes = super::bytes(read, address, 3, |a, i| a + i);
    let op = bytes[0] as u32 | (bytes[1] as u32) << 8 | (bytes[2] as u32) << 16;
    let mut reference = None;
    let (mnemonic, operands) = match op >> 22 {
        0 | 1 => {
            let (rt, p, alu, acc) = (op >> 22 & 1 != 0, (op >> 20 & 3) as usize, (op >> 16 & 15) as usize, if op >> 15 & 1 != 0 { "B" } else { "A" });
            let (dpl, dph, rp, src, dst) = (op >> 13 & 3, op >> 9 & 15, op >> 8 & 1, (op >> 4 & 15) as usize, (op & 15) as usize);
            let mut parts = Vec::new();
            match alu {
                0 => {}
                1..=7 => parts.push(format!("{acc},{}", P[p])),
                _ => parts.push(acc.to_owned()),
            }
            if dst != 0 {
                parts.push(format!("{}={}", DST[dst], SRC[src]));
            }
            if dpl != 0 {
                parts.push(["", "DPINC", "DPDEC", "DPCLR"][dpl as usize].to_owned());
            }
            if dph != 0 {
                parts.push(format!("M{dph:X}"));
            }
            if rp != 0 {
                parts.push("RPDEC".to_owned());
            }
            if rt {
                parts.push("RET".to_owned());
            }
            (if alu == 0 && dst != 0 { "MOV" } else { ALU[alu] }, parts.join(" "))
        }
        2 => {
            let brch = op >> 13 & 0x1FF;
            let target = (op >> 2 & 0x7FF) | if st { (op & 3) << 11 } else { 0 } | if st && brch & 1 != 0 && brch & 0x100 != 0 { 0x2000 } else { 0 };
            match jump(brch) {
                Some("JMPSO") => ("JMPSO", String::new()),
                Some(name) => {
                    if name == "CALL" {
                        reference = Some((Reference::Call, target * 3));
                    }
                    (name, format!("${target:04X}"))
                }
                None => ("JP", format!("#${brch:03X},${target:04X}")),
            }
        }
        _ => ("LD", format!("{},#${:04X}", DST[(op & 15) as usize], op >> 6 & 0xFFFF)),
    };
    Instruction { address, bytes, mnemonic: mnemonic.into(), operands, reference }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(op: u32, st: bool) -> (String, Option<(Reference, u32)>) {
        let bytes = op.to_le_bytes();
        let i = decode(&|a| bytes[a as usize % 3], 0, st);
        (format!("{} {}", i.mnemonic, i.operands).trim_end().to_owned(), i.reference)
    }

    // fullsnes's fields: an ALU instruction with its move, pointer steps and return; a load; the jumps with their
    // word addresses, a call's as a reference in bytes.
    #[test]
    fn the_three_instruction_forms_read_by_their_fields() {
        assert_eq!(text(0x000000, false).0, "NOP");
        assert_eq!(text(0x400000, false).0, "NOP RET");
        assert_eq!(text(0x05_80_F1, false).0, "ADD B,RAM @A=MEM");
        assert_eq!(text(0x00_23_14, false).0, "MOV @DP=A DPINC M1 RPDEC");
        assert_eq!(text(0x19_00_00, false).0, "INC A");
        assert_eq!(text(0xC0_00_00 | 0x1234 << 6 | 0x0A, false).0, "LD @K,#$1234");
        assert_eq!(text(0x80_00_00 | 0x100 << 13 | 0x123 << 2, false), ("JMP $0123".into(), None));
        assert_eq!(text(0x80_00_00 | 0x140 << 13 | 0x7FF << 2, false), ("CALL $07FF".into(), Some((Reference::Call, 0x7FF * 3))));
        assert_eq!(text(0x80_00_00 | 0x08A << 13 | 0x010 << 2, false).0, "JZA $0010");
        assert_eq!(text(0x80_00_00 | 0x0BE << 13 | 0x010 << 2 | 3, true).0, "JRQM $1810");
        assert_eq!(text(0x80_00_00 | 0x141 << 13 | 0x001 << 2, true).0, "CALL $2001");
        assert_eq!(text(0x80_00_00, true).0, "JMPSO");
    }
}
