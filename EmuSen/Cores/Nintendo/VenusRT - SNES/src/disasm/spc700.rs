//! The SPC700's instructions in their native syntax, by fullsnes's "SNES APU SPC700 CPU" tables: the load and store
//! commands, the ALU groups whose opcode is a base plus an operand form, and the jump and control commands.

use super::{Instruction, Reference};

/// The six 8-bit ALU operations at bases 00h to A0h, and the six shifts and steps at the same bases.
const ALU: [&str; 6] = ["OR", "AND", "EOR", "CMP", "ADC", "SBC"];
const SHIFT: [&str; 6] = ["ASL", "ROL", "LSR", "ROR", "DEC", "INC"];
const BRANCH: [&str; 8] = ["BPL", "BMI", "BVC", "BVS", "BCC", "BCS", "BNE", "BEQ"];

/// A mnemonic and its operands as a template: `#0` an immediate from operand byte 0, `d0` a direct-page address,
/// `a` the two bytes as an address, `r0` a relative target from byte 0, `m` a 13-bit address and its bit, `u0` a page-FF
/// address. A byte's index is its place after the opcode.
fn form(op: u8) -> (&'static str, String) {
    let s = |m: &'static str, t: &str| (m, t.to_owned());
    let bit = op >> 5;
    match op {
        0x00 => s("NOP", ""),
        0xEF => s("SLEEP", ""),
        0xFF => s("STOP", ""),
        0x20 => s("CLRP", ""),
        0x40 => s("SETP", ""),
        0xA0 => s("EI", ""),
        0xC0 => s("DI", ""),
        0x60 => s("CLRC", ""),
        0x80 => s("SETC", ""),
        0xED => s("NOTC", ""),
        0xE0 => s("CLRV", ""),
        _ if op & 0x1F == 0x10 => s(BRANCH[bit as usize], "r0"),
        _ if op & 0x0F == 0x01 => ("TCALL", format!("{}", op >> 4)),
        _ if op & 0x1F == 0x02 => ("SET1", format!("d0.{bit}")),
        _ if op & 0x1F == 0x12 => ("CLR1", format!("d0.{bit}")),
        _ if op & 0x1F == 0x03 => ("BBS", format!("d0.{bit},r1")),
        _ if op & 0x1F == 0x13 => ("BBC", format!("d0.{bit},r1")),
        0x4F => s("PCALL", "u0"),
        0x6F => s("RET", ""),
        0x7F => s("RET1", ""),
        0x0F => s("BRK", ""),
        0x2F => s("BRA", "r0"),
        0x5F => s("JMP", "a"),
        0x1F => s("JMP", "[a+X]"),
        0x3F => s("CALL", "a"),
        0x2E => s("CBNE", "d0,r1"),
        0xDE => s("CBNE", "d0+X,r1"),
        0xFE => s("DBNZ", "Y,r0"),
        0x6E => s("DBNZ", "d0,r1"),
        0xE8 => s("MOV", "A,#0"),
        0xCD => s("MOV", "X,#0"),
        0x8D => s("MOV", "Y,#0"),
        0x7D => s("MOV", "A,X"),
        0x5D => s("MOV", "X,A"),
        0xDD => s("MOV", "A,Y"),
        0xFD => s("MOV", "Y,A"),
        0x9D => s("MOV", "X,SP"),
        0xBD => s("MOV", "SP,X"),
        0xE4 => s("MOV", "A,d0"),
        0xF4 => s("MOV", "A,d0+X"),
        0xE5 => s("MOV", "A,a"),
        0xF5 => s("MOV", "A,a+X"),
        0xF6 => s("MOV", "A,a+Y"),
        0xE6 => s("MOV", "A,(X)"),
        0xBF => s("MOV", "A,(X)+"),
        0xF7 => s("MOV", "A,[d0]+Y"),
        0xE7 => s("MOV", "A,[d0+X]"),
        0xF8 => s("MOV", "X,d0"),
        0xF9 => s("MOV", "X,d0+Y"),
        0xE9 => s("MOV", "X,a"),
        0xEB => s("MOV", "Y,d0"),
        0xFB => s("MOV", "Y,d0+X"),
        0xEC => s("MOV", "Y,a"),
        0xBA => s("MOVW", "YA,d0"),
        0x8F => s("MOV", "d1,#0"),
        0xFA => s("MOV", "d1,d0"),
        0xC4 => s("MOV", "d0,A"),
        0xD8 => s("MOV", "d0,X"),
        0xCB => s("MOV", "d0,Y"),
        0xD4 => s("MOV", "d0+X,A"),
        0xDB => s("MOV", "d0+X,Y"),
        0xD9 => s("MOV", "d0+Y,X"),
        0xC5 => s("MOV", "a,A"),
        0xC9 => s("MOV", "a,X"),
        0xCC => s("MOV", "a,Y"),
        0xD5 => s("MOV", "a+X,A"),
        0xD6 => s("MOV", "a+Y,A"),
        0xAF => s("MOV", "(X)+,A"),
        0xC6 => s("MOV", "(X),A"),
        0xD7 => s("MOV", "[d0]+Y,A"),
        0xC7 => s("MOV", "[d0+X],A"),
        0xDA => s("MOVW", "d0,YA"),
        0x2D => s("PUSH", "A"),
        0x4D => s("PUSH", "X"),
        0x6D => s("PUSH", "Y"),
        0x0D => s("PUSH", "PSW"),
        0xAE => s("POP", "A"),
        0xCE => s("POP", "X"),
        0xEE => s("POP", "Y"),
        0x8E => s("POP", "PSW"),
        0xC8 => s("CMP", "X,#0"),
        0x3E => s("CMP", "X,d0"),
        0x1E => s("CMP", "X,a"),
        0xAD => s("CMP", "Y,#0"),
        0x7E => s("CMP", "Y,d0"),
        0x5E => s("CMP", "Y,a"),
        0x7A => s("ADDW", "YA,d0"),
        0x9A => s("SUBW", "YA,d0"),
        0x5A => s("CMPW", "YA,d0"),
        0x3A => s("INCW", "d0"),
        0x1A => s("DECW", "d0"),
        0x9E => s("DIV", "YA,X"),
        0xCF => s("MUL", "YA"),
        0xEA => s("NOT1", "m"),
        0xCA => s("MOV1", "m,C"),
        0xAA => s("MOV1", "C,m"),
        0x0A => s("OR1", "C,m"),
        0x2A => s("OR1", "C,/m"),
        0x4A => s("AND1", "C,m"),
        0x6A => s("AND1", "C,/m"),
        0x8A => s("EOR1", "C,m"),
        0xDF => s("DAA", "A"),
        0xBE => s("DAS", "A"),
        0x9F => s("XCN", "A"),
        0x4E => s("TCLR1", "a"),
        0x0E => s("TSET1", "a"),
        0x1D => s("DEC", "X"),
        0x3D => s("INC", "X"),
        0xDC => s("DEC", "Y"),
        0xFC => s("INC", "Y"),
        _ if op < 0xC0 => {
            let n = (op >> 5) as usize;
            match op & 0x1F {
                0x08 => s(ALU[n], "A,#0"),
                0x06 => s(ALU[n], "A,(X)"),
                0x04 => s(ALU[n], "A,d0"),
                0x14 => s(ALU[n], "A,d0+X"),
                0x05 => s(ALU[n], "A,a"),
                0x15 => s(ALU[n], "A,a+X"),
                0x16 => s(ALU[n], "A,a+Y"),
                0x17 => s(ALU[n], "A,[d0]+Y"),
                0x07 => s(ALU[n], "A,[d0+X]"),
                0x09 => s(ALU[n], "d1,d0"),
                0x18 => s(ALU[n], "d1,#0"),
                0x19 => s(ALU[n], "(X),(Y)"),
                0x1C => s(SHIFT[n], "A"),
                0x0B => s(SHIFT[n], "d0"),
                0x1B => s(SHIFT[n], "d0+X"),
                0x0C => s(SHIFT[n], "a"),
                _ => s("???", ""),
            }
        }
        _ => s("???", ""),
    }
}

/// The operand bytes a template takes: two for an address or a bit's address, else one past its highest index.
fn operand_bytes(template: &str) -> u32 {
    let c: Vec<char> = template.chars().collect();
    let mut n = 0;
    for (i, &ch) in c.iter().enumerate() {
        let digit = c.get(i + 1).and_then(|d| d.to_digit(10));
        match (ch, digit) {
            ('a' | 'm', _) => n = 2,
            ('#' | 'd' | 'r' | 'u', Some(d @ 0..=1)) => n = n.max(d + 1),
            _ => {}
        }
    }
    n
}

pub fn decode(read: &dyn Fn(u32) -> u8, address: u32) -> Instruction {
    let op = read(address & 0xFFFF);
    let (mnemonic, template) = form(op);
    let n = operand_bytes(&template);
    let bytes = super::bytes(read, address, 1 + n, |a, i| a.wrapping_add(i) & 0xFFFF);
    let b = |i: usize| bytes.get(1 + i).copied().unwrap_or(0) as u32;
    let word = b(0) | b(1) << 8;
    let next = address.wrapping_add(1 + n);
    let mut out = String::new();
    let chars: Vec<char> = template.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let index = |i: usize| chars.get(i + 1).and_then(|c| c.to_digit(10)).filter(|&d| d < 2).map(|d| d as usize);
        match chars[i] {
            '#' if index(i).is_some() => {
                out += &format!("#${:02X}", b(index(i).unwrap()));
                i += 1;
            }
            'd' if index(i).is_some() => {
                out += &format!("${:02X}", b(index(i).unwrap()));
                i += 1;
            }
            'r' if index(i).is_some() => {
                out += &format!("${:04X}", next.wrapping_add(b(index(i).unwrap()) as u8 as i8 as u32) & 0xFFFF);
                i += 1;
            }
            'u' if index(i).is_some() => {
                out += &format!("$FF{:02X}", b(0));
                i += 1;
            }
            'a' => out += &format!("!${word:04X}"),
            'm' => out += &format!("${:04X}.{}", word & 0x1FFF, word >> 13),
            c => out.push(c),
        }
        i += 1;
    }
    let reference = match (mnemonic, template.as_str()) {
        ("CALL", _) => Some((Reference::Call, word)),
        ("MOV", "a,A" | "a,X" | "a,Y") | ("TSET1" | "TCLR1", _) => Some((Reference::Write, word)),
        (_, "a") if SHIFT.contains(&mnemonic) => Some((Reference::Write, word)),
        ("MOV" | "CMP", "A,a" | "X,a" | "Y,a") => Some((Reference::Read, word)),
        (_, "A,a") if ALU.contains(&mnemonic) => Some((Reference::Read, word)),
        _ => None,
    };
    Instruction { address, bytes, mnemonic: mnemonic.into(), operands: out, reference }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apu::spc700::{Bus, Spc700};

    fn text(bytes: &[u8], at: u32) -> (String, usize) {
        let read = |a: u32| bytes.get(a.wrapping_sub(at) as usize).copied().unwrap_or(0);
        let i = decode(&read, at);
        (format!("{} {}", i.mnemonic, i.operands).trim_end().to_owned(), i.bytes.len())
    }

    // fullsnes's native syntax over the boot ROM's own instructions and one of each operand form.
    #[test]
    fn instructions_read_in_the_native_syntax() {
        assert_eq!(text(&[0xCD, 0xEF], 0xFFC0), ("MOV X,#$EF".into(), 2));
        assert_eq!(text(&[0xBD], 0xFFC2), ("MOV SP,X".into(), 1));
        assert_eq!(text(&[0xC6], 0xFFC5), ("MOV (X),A".into(), 1));
        assert_eq!(text(&[0xD0, 0xFC], 0xFFC7), ("BNE $FFC5".into(), 2));
        assert_eq!(text(&[0x8F, 0xAA, 0xF4], 0xFFC9), ("MOV $F4,#$AA".into(), 3));
        assert_eq!(text(&[0x78, 0xCC, 0xF4], 0xFFCF), ("CMP $F4,#$CC".into(), 3));
        assert_eq!(text(&[0x2F, 0x19], 0xFFD4), ("BRA $FFEF".into(), 2));
        assert_eq!(text(&[0xD7, 0x00], 0xFFE2), ("MOV [$00]+Y,A".into(), 2));
        assert_eq!(text(&[0x1F, 0x00, 0x00], 0xFFFB), ("JMP [!$0000+X]".into(), 3));
        assert_eq!(text(&[0x3F, 0x34, 0x12], 0x0200), ("CALL !$1234".into(), 3));
        assert_eq!(text(&[0xFA, 0x10, 0x20], 0x0200), ("MOV $20,$10".into(), 3));
        assert_eq!(text(&[0xE3, 0x10, 0x05], 0x0200), ("BBS $10.7,$0208".into(), 3));
        assert_eq!(text(&[0x72, 0x10], 0x0200), ("CLR1 $10.3".into(), 2));
        assert_eq!(text(&[0xAA, 0x34, 0x72], 0x0200), ("MOV1 C,$1234.3".into(), 3));
        assert_eq!(text(&[0x4F, 0x80], 0x0200), ("PCALL $FF80".into(), 2));
        assert_eq!(text(&[0xF1], 0x0200), ("TCALL 15".into(), 1));
        assert_eq!(text(&[0x9C], 0x0200), ("DEC A".into(), 1));
        assert_eq!(text(&[0xB6, 0x00, 0x03], 0x0200), ("SBC A,!$0300+Y".into(), 3));
    }

    struct One(u8);

    impl Bus for One {
        fn read(&mut self, a: u16) -> u8 {
            if a == 0x0200 { self.0 } else { 0 }
        }
        fn write(&mut self, _: u16, _: u8) {}
        fn idle(&mut self) {}
    }

    // Every opcode has a name, and each that falls through is as long as the core, which the single-step suite
    // grades, finds it.
    #[test]
    fn every_opcode_is_named_and_as_long_as_the_core_finds_it() {
        let flow = |op: u8| op & 0x0F == 0x01 || op & 0x1F == 0x10 || op & 0x0F == 0x03 || matches!(op, 0x0F | 0x1F | 0x2F | 0x3F | 0x4F | 0x5F | 0x6F | 0x7F | 0x2E | 0xDE | 0xFE | 0x6E | 0xEF | 0xFF);
        for op in 0..=255u8 {
            let read = |a: u32| if a == 0x0200 { op } else { 0 };
            let i = decode(&read, 0x0200);
            assert_ne!(i.mnemonic, "???", "opcode {op:02X}");
            if flow(op) {
                continue;
            }
            let mut c = Spc700 { pc: 0x0200, sp: 0xEF, ..Spc700::default() };
            c.step(&mut One(op));
            assert_eq!(c.pc.wrapping_sub(0x0200) as usize, i.bytes.len(), "opcode {op:02X}: {} {}", i.mnemonic, i.operands);
        }
    }
}
