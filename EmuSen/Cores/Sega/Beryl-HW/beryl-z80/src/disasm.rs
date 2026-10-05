//! The Z80 disassembler for `DEBUG_DISASSEMBLE`: Zilog's syntax with `$` for hexadecimal, the undocumented forms by
//! the names "The Undocumented Z80 Documented" gives them, and the static reference knowable from the bytes alone
//! (Beryl_Z80.md §6).

use emusen_native::core::{Instruction, Reference};

const R: [&str; 8] = ["B", "C", "D", "E", "H", "L", "(HL)", "A"];
const CC: [&str; 8] = ["NZ", "Z", "NC", "C", "PO", "PE", "P", "M"];
const ALU: [(&str, &str); 8] = [("ADD", "A,"), ("ADC", "A,"), ("SUB", ""), ("SBC", "A,"), ("AND", ""), ("XOR", ""), ("OR", ""), ("CP", "")];
const ROT: [&str; 8] = ["RLC", "RRC", "RL", "RR", "SLA", "SRA", "SLL", "SRL"];
const BLOCK: [[&str; 4]; 4] = [["LDI", "CPI", "INI", "OUTI"], ["LDD", "CPD", "IND", "OUTD"], ["LDIR", "CPIR", "INIR", "OTIR"], ["LDDR", "CPDR", "INDR", "OTDR"]];

struct D<'a> {
    read: &'a dyn Fn(u16) -> u8,
    next: u16,
    bytes: Vec<u8>,
    reference: Option<(Reference, u32)>,
    /// "HL", "IX" or "IY": the register a prefix puts in HL's place.
    ix: &'static str,
    /// The displacement of an indexed operand, read once where it falls in the instruction.
    disp: Option<i8>,
}

impl D<'_> {
    fn byte(&mut self) -> u8 {
        let b = (self.read)(self.next);
        self.next = self.next.wrapping_add(1);
        self.bytes.push(b);
        b
    }
    fn word(&mut self) -> u16 {
        let lo = self.byte() as u16;
        lo | (self.byte() as u16) << 8
    }
    fn hex8(v: u8) -> String {
        format!("${v:02X}")
    }
    fn hex16(v: u16) -> String {
        format!("${v:04X}")
    }
    /// The memory operand: (HL), or (IX+d) with its displacement read on first use.
    fn mem(&mut self) -> String {
        if self.ix == "HL" {
            return "(HL)".into();
        }
        let d = match self.disp {
            Some(d) => d,
            None => {
                let d = self.byte() as i8;
                self.disp = Some(d);
                d
            }
        };
        if d < 0 { format!("({}-${:02X})", self.ix, -(d as i16)) } else { format!("({}+${:02X})", self.ix, d) }
    }
    /// Register `r` of a field; H and L are the index register's halves unless the other operand is the memory one.
    fn reg(&mut self, r: u8, halves: bool) -> String {
        match (r, self.ix, halves) {
            (6, _, _) => self.mem(),
            (4, "IX" | "IY", true) => format!("{}H", self.ix),
            (5, "IX" | "IY", true) => format!("{}L", self.ix),
            _ => R[r as usize].into(),
        }
    }
    fn rp(&self, p: u8) -> String {
        match p {
            0 => "BC".into(),
            1 => "DE".into(),
            2 => self.ix.into(),
            _ => "SP".into(),
        }
    }
    fn rp2(&self, p: u8) -> String {
        if p == 3 { "AF".into() } else { self.rp(p) }
    }
    fn target(&mut self) -> u16 {
        let e = self.byte() as i8 as u16;
        self.next.wrapping_add(e)
    }
    fn absolute(&mut self, kind: Reference) -> String {
        let a = self.word();
        self.reference = Some((kind, a as u32));
        Self::hex16(a)
    }
}

/// The instruction at `address`, prefixes included; an ED opcode the Z80 executes as a NOP is `DB` of its two bytes.
pub fn disassemble(read: &dyn Fn(u16) -> u8, address: u16) -> Instruction {
    let mut d = D { read, next: address, bytes: Vec::new(), reference: None, ix: "HL", disp: None };
    let mut op = d.byte();
    while op == 0xDD || op == 0xFD {
        d.ix = if op == 0xDD { "IX" } else { "IY" };
        op = d.byte();
    }
    let (mnemonic, operands) = match op {
        0xCB if d.ix == "HL" => {
            let op = d.byte();
            cb(op)
        }
        0xCB => indexed_cb(&mut d),
        0xED => {
            let op = d.byte();
            ed(&mut d, op)
        }
        _ => main(&mut d, op),
    };
    Instruction { address: address as u32, bytes: d.bytes, mnemonic: mnemonic.into(), operands, reference: d.reference }
}

fn main(d: &mut D, op: u8) -> (&'static str, String) {
    let y = (op >> 3) & 7;
    let z = op & 7;
    let p = y >> 1;
    match op >> 6 {
        0 => match z {
            0 => match y {
                0 => ("NOP", String::new()),
                1 => ("EX", "AF,AF'".into()),
                2 => ("DJNZ", D::hex16(d.target())),
                3 => ("JR", D::hex16(d.target())),
                _ => {
                    let t = d.target();
                    ("JR", format!("{},{}", CC[(y - 4) as usize], D::hex16(t)))
                }
            },
            1 if y & 1 == 0 => {
                let r = d.rp(p);
                ("LD", format!("{r},{}", D::hex16(d.word())))
            }
            1 => ("ADD", format!("{},{}", d.ix, d.rp(p))),
            2 => match y {
                0 => ("LD", "(BC),A".into()),
                1 => ("LD", "A,(BC)".into()),
                2 => ("LD", "(DE),A".into()),
                3 => ("LD", "A,(DE)".into()),
                4 => {
                    let a = d.absolute(Reference::Write);
                    ("LD", format!("({a}),{}", d.ix))
                }
                5 => {
                    let a = d.absolute(Reference::Read);
                    ("LD", format!("{},({a})", d.ix))
                }
                6 => ("LD", format!("({}),A", d.absolute(Reference::Write))),
                _ => ("LD", format!("A,({})", d.absolute(Reference::Read))),
            },
            3 => (if y & 1 == 0 { "INC" } else { "DEC" }, d.rp(p)),
            4 => ("INC", d.reg(y, true)),
            5 => ("DEC", d.reg(y, true)),
            6 => {
                let r = d.reg(y, true);
                ("LD", format!("{r},{}", D::hex8(d.byte())))
            }
            _ => (["RLCA", "RRCA", "RLA", "RRA", "DAA", "CPL", "SCF", "CCF"][y as usize], String::new()),
        },
        1 if op == 0x76 => ("HALT", String::new()),
        1 => {
            let halves = y != 6 && z != 6;
            let a = d.reg(y, halves);
            let b = d.reg(z, halves);
            ("LD", format!("{a},{b}"))
        }
        2 => {
            let r = d.reg(z, true);
            alu(y, r)
        }
        _ => match z {
            0 => ("RET", CC[y as usize].into()),
            1 => match (y & 1, p) {
                (0, _) => ("POP", d.rp2(p)),
                (_, 0) => ("RET", String::new()),
                (_, 1) => ("EXX", String::new()),
                (_, 2) => ("JP", format!("({})", d.ix)),
                _ => ("LD", format!("SP,{}", d.ix)),
            },
            2 => ("JP", format!("{},{}", CC[y as usize], D::hex16(d.word()))),
            3 => match y {
                0 => ("JP", D::hex16(d.word())),
                2 => ("OUT", format!("({}),A", D::hex8(d.byte()))),
                3 => ("IN", format!("A,({})", D::hex8(d.byte()))),
                4 => ("EX", format!("(SP),{}", d.ix)),
                5 => ("EX", "DE,HL".into()),
                6 => ("DI", String::new()),
                _ => ("EI", String::new()),
            },
            4 => ("CALL", format!("{},{}", CC[y as usize], d.absolute(Reference::Call))),
            5 if y & 1 == 0 => ("PUSH", d.rp2(p)),
            5 => ("CALL", d.absolute(Reference::Call)),
            6 => {
                let n = d.byte();
                alu(y, D::hex8(n))
            }
            _ => {
                d.reference = Some((Reference::Call, (y * 8) as u32));
                ("RST", D::hex8(y * 8))
            }
        },
    }
}

fn alu(y: u8, operand: String) -> (&'static str, String) {
    let (m, a) = ALU[y as usize];
    (m, format!("{a}{operand}"))
}

fn cb(op: u8) -> (&'static str, String) {
    let y = (op >> 3) & 7;
    let r = R[(op & 7) as usize];
    match op >> 6 {
        0 => (ROT[y as usize], r.into()),
        1 => ("BIT", format!("{y},{r}")),
        2 => ("RES", format!("{y},{r}")),
        _ => ("SET", format!("{y},{r}")),
    }
}

/// DD CB d op: the displacement comes before the opcode; a register in the low bits also receives the result.
fn indexed_cb(d: &mut D) -> (&'static str, String) {
    let m = d.mem();
    let op = d.byte();
    let y = (op >> 3) & 7;
    let z = op & 7;
    let copy = if z == 6 { String::new() } else { format!(",{}", R[z as usize]) };
    match op >> 6 {
        0 => (ROT[y as usize], format!("{m}{copy}")),
        1 => ("BIT", format!("{y},{m}")),
        2 => ("RES", format!("{y},{m}{copy}")),
        _ => ("SET", format!("{y},{m}{copy}")),
    }
}

fn ed(d: &mut D, op: u8) -> (&'static str, String) {
    let y = (op >> 3) & 7;
    let z = op & 7;
    let p = y >> 1;
    let rp = ["BC", "DE", "HL", "SP"][p as usize];
    match op {
        0x40..=0x7F => match z {
            0 if y == 6 => ("IN", "(C)".into()),
            0 => ("IN", format!("{},(C)", R[y as usize])),
            1 if y == 6 => ("OUT", "(C),0".into()),
            1 => ("OUT", format!("(C),{}", R[y as usize])),
            2 => (if y & 1 == 0 { "SBC" } else { "ADC" }, format!("HL,{rp}")),
            3 if y & 1 == 0 => ("LD", format!("({}),{rp}", d.absolute(Reference::Write))),
            3 => ("LD", format!("{rp},({})", d.absolute(Reference::Read))),
            4 => ("NEG", String::new()),
            5 => (if y == 1 { "RETI" } else { "RETN" }, String::new()),
            6 => ("IM", ["0", "0", "1", "2"][(y & 3) as usize].into()),
            _ => match y {
                0 => ("LD", "I,A".into()),
                1 => ("LD", "R,A".into()),
                2 => ("LD", "A,I".into()),
                3 => ("LD", "A,R".into()),
                4 => ("RRD", String::new()),
                5 => ("RLD", String::new()),
                _ => ("DB", format!("$ED,{}", D::hex8(op))),
            },
        },
        0xA0..=0xA3 | 0xA8..=0xAB | 0xB0..=0xB3 | 0xB8..=0xBB => (BLOCK[(y - 4) as usize][z as usize], String::new()),
        _ => ("DB", format!("$ED,{}", D::hex8(op))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bus, Z80};

    fn text(bytes: &[u8], at: u16) -> (String, usize, Option<(Reference, u32)>) {
        let b = bytes.to_vec();
        let i = disassemble(&move |a| b.get(a.wrapping_sub(at) as usize).copied().unwrap_or(0), at);
        (format!("{} {}", i.mnemonic, i.operands).trim_end().to_owned(), i.bytes.len(), i.reference)
    }

    #[test]
    fn instructions_read_as_zilog_writes_them() {
        assert_eq!(text(&[0x00], 0).0, "NOP");
        assert_eq!(text(&[0x3E, 0x12], 0).0, "LD A,$12");
        assert_eq!(text(&[0x21, 0x34, 0x12], 0).0, "LD HL,$1234");
        assert_eq!(text(&[0x3A, 0x00, 0x40], 0), ("LD A,($4000)".into(), 3, Some((Reference::Read, 0x4000))));
        assert_eq!(text(&[0x32, 0x00, 0x60], 0), ("LD ($6000),A".into(), 3, Some((Reference::Write, 0x6000))));
        assert_eq!(text(&[0xCD, 0x00, 0x01], 0), ("CALL $0100".into(), 3, Some((Reference::Call, 0x100))));
        assert_eq!(text(&[0xC4, 0x00, 0x01], 0).0, "CALL NZ,$0100");
        assert_eq!(text(&[0xFF], 0), ("RST $38".into(), 1, Some((Reference::Call, 0x38))));
        assert_eq!(text(&[0x18, 0xFE], 0x200).0, "JR $0200");
        assert_eq!(text(&[0x10, 0xFC], 0x200).0, "DJNZ $01FE");
        assert_eq!(text(&[0x80], 0).0, "ADD A,B");
        assert_eq!(text(&[0x96], 0).0, "SUB (HL)");
        assert_eq!(text(&[0xFE, 0x0A], 0).0, "CP $0A");
        assert_eq!(text(&[0xDB, 0xFE], 0).0, "IN A,($FE)");
        assert_eq!(text(&[0xDD, 0x7E, 0xFB], 0), ("LD A,(IX-$05)".into(), 3, None));
        assert_eq!(text(&[0xFD, 0x36, 0x02, 0x99], 0).0, "LD (IY+$02),$99");
        assert_eq!(text(&[0xDD, 0x66, 0x01], 0).0, "LD H,(IX+$01)");
        assert_eq!(text(&[0xDD, 0x64], 0).0, "LD IXH,IXH");
        assert_eq!(text(&[0xFD, 0x85], 0).0, "ADD A,IYL");
        assert_eq!(text(&[0xDD, 0xE9], 0).0, "JP (IX)");
        assert_eq!(text(&[0xDD, 0xEB], 0).0, "EX DE,HL");
        assert_eq!(text(&[0xDD, 0x00], 0), ("NOP".into(), 2, None));
        assert_eq!(text(&[0xCB, 0x37], 0).0, "SLL A");
        assert_eq!(text(&[0xCB, 0x7E], 0).0, "BIT 7,(HL)");
        assert_eq!(text(&[0xDD, 0xCB, 0x03, 0x06], 0), ("RLC (IX+$03)".into(), 4, None));
        assert_eq!(text(&[0xFD, 0xCB, 0xFF, 0xC0], 0).0, "SET 0,(IY-$01),B");
        assert_eq!(text(&[0xDD, 0xCB, 0x00, 0x41], 0).0, "BIT 0,(IX+$00)");
        assert_eq!(text(&[0xED, 0xB0], 0).0, "LDIR");
        assert_eq!(text(&[0xED, 0x71], 0).0, "OUT (C),0");
        assert_eq!(text(&[0xED, 0x70], 0).0, "IN (C)");
        assert_eq!(text(&[0xED, 0x4B, 0x00, 0x80], 0), ("LD BC,($8000)".into(), 4, Some((Reference::Read, 0x8000))));
        assert_eq!(text(&[0xED, 0x56], 0).0, "IM 1");
        assert_eq!(text(&[0xED, 0x00], 0), ("DB $ED,$00".into(), 2, None));
    }

    struct Flat(Vec<u8>);

    impl Bus for Flat {
        fn fetch(&mut self, a: u16, _: u16) -> u8 {
            self.0[a as usize]
        }
        fn read(&mut self, a: u16) -> u8 {
            self.0[a as usize]
        }
        fn write(&mut self, _: u16, _: u8) {}
        fn input(&mut self, _: u16) -> u8 {
            0xFF
        }
        fn output(&mut self, _: u16, _: u8) {}
        fn idle(&mut self, _: u32) {}
        fn int_line(&mut self) -> bool {
            false
        }
        fn nmi_edge(&mut self) -> bool {
            false
        }
        fn acknowledge(&mut self) -> u8 {
            0xFF
        }
    }

    /// Every opcode the processor runs without a jump: the disassembler's length is the bytes the processor took.
    #[test]
    fn the_length_is_what_the_processor_takes() {
        let jumps = ["JP", "JR", "DJNZ", "CALL", "RET", "RETI", "RETN", "RST", "LDIR", "LDDR", "CPIR", "CPDR", "INIR", "INDR", "OTIR", "OTDR"];
        let mut checked = 0;
        let mut sequences: Vec<Vec<u8>> = (0..=255u8).map(|b| vec![b]).collect();
        for prefix in [0xCB, 0xED, 0xDD, 0xFD] {
            sequences.extend((0..=255u8).map(|b| vec![prefix, b]));
        }
        for prefix in [0xDD, 0xFD] {
            sequences.extend((0..=255u8).map(|b| vec![prefix, 0xCB, 0x05, b]));
        }
        for seq in sequences {
            if matches!(seq[..], [0xDD | 0xFD] | [0xDD | 0xFD, 0xDD | 0xFD | 0xED] | [0xCB | 0xED]) {
                continue;
            }
            let mut ram = vec![0u8; 0x10000];
            ram[0x100..0x100 + seq.len()].copy_from_slice(&seq);
            ram[0x100 + seq.len()..0x104 + seq.len()].copy_from_slice(&[0x11, 0x22, 0x33, 0x44]);
            let i = disassemble(&|a| ram[a as usize], 0x100);
            if jumps.contains(&i.mnemonic.as_str()) || i.mnemonic == "HALT" {
                continue;
            }
            let mut z = Z80::new();
            z.regs.pc = 0x100;
            z.regs.bc = 0x0101;
            z.regs.sp = 0x8000;
            z.step(&mut Flat(ram));
            assert_eq!(z.regs.pc.wrapping_sub(0x100) as usize, i.bytes.len(), "{seq:02X?}: {} {}", i.mnemonic, i.operands);
            checked += 1;
        }
        eprintln!("{checked} opcodes: the disassembler's length is the processor's");
        assert!(checked > 1500, "{checked}");
    }
}
