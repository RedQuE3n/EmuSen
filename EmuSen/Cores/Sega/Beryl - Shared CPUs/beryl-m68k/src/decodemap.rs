//! The decoder and the disassembler against TomHarte's instruction map, opcode by opcode (Beryl_M68k.md §5.5): which
//! of the 65,536 words the 68000 decodes, and, for each, the instruction and its operands' modes and registers.

use std::collections::BTreeMap;
use std::path::Path;

use emusen_native::json::{self, Value};

use crate::{Access, Bus, M68000, Step};

/// The map's text for an opcode, rebuilt from the disassembler's: its names for the special forms, the size suffixes
/// it keeps, `#` for an immediate and for a register list, and the numbers it writes in place of the quick operands.
pub fn normalised(op: u16, mnemonic: &str, operands: &str) -> Option<String> {
    if mnemonic == "DC.W" || mnemonic == "ILLEGAL" {
        return None;
    }
    let (base, size) = mnemonic.split_once('.').map_or((mnemonic, ""), |(b, s)| (b, s));
    let conds = ["T", "F", "HI", "LS", "CC", "CS", "NE", "EQ", "VC", "VS", "PL", "MI", "GE", "LT", "GT", "LE"];
    let parts: Vec<&str> = split(operands);
    let mut ops: Vec<String> = parts.iter().map(|p| mode(p)).collect();
    let sized = |s: &str| format!("{}.{}", s, size.to_lowercase());
    let name = match base {
        "ADDI" | "ADDQ" => sized("ADD"),
        "SUBI" | "SUBQ" => sized("SUB"),
        "CMPI" | "CMPM" => sized("CMP"),
        "ANDI" | "ORI" | "EORI" if operands.ends_with("CCR") || operands.ends_with(",SR") => {
            let to = if operands.ends_with("CCR") { "CCR" } else { "SR" };
            return Some(format!("{base}to{to} #"));
        }
        "ANDI" => sized("AND"),
        "ORI" => sized("OR"),
        "EORI" => sized("EOR"),
        "MOVEQ" => {
            return Some(format!("MOVE.q {}, D{}", op as u8 as i8, (op >> 9) & 7));
        }
        "MOVE" if size.is_empty() => {
            return Some(match (parts[0], parts.get(1).copied()) {
                ("SR", Some(_)) => format!("MOVEfromSR {}", ops[1]),
                (_, Some("CCR")) => format!("MOVEtoCCR {}", ops[0]),
                (_, Some("SR")) => format!("MOVEtoSR {}", ops[0]),
                (_, Some("USP")) => format!("MOVEtoUSP {}", ops[0]),
                _ => format!("MOVEfromUSP {}", ops[1]),
            });
        }
        "UNLK" => "UNLINK".into(),
        "TRAP" => return Some(format!("TRAP {}", op & 15)),
        "DBRA" => "DBcc".into(),
        b if b.starts_with("DB") && conds.contains(&&b[2..]) => "DBcc".into(),
        "BSR" | "BRA" => {
            let disp = if op & 0xFF == 0 { "#".to_string() } else { (op as u8 as i8).to_string() };
            return Some(format!("{} {disp}", if base == "BSR" { "BSR" } else { "Bcc" }));
        }
        b if b.starts_with('B') && conds.contains(&&b[1..]) && op >> 12 == 6 => {
            let disp = if op & 0xFF == 0 { "#".to_string() } else { (op as u8 as i8).to_string() };
            return Some(format!("Bcc {disp}"));
        }
        b if b.starts_with('S') && conds.contains(&&b[1..]) && op >> 12 == 5 => "Scc".into(),
        "CHK" | "DIVU" | "DIVS" | "MULU" | "MULS" => base.into(),
        _ if size.is_empty() => base.into(),
        _ => sized(base),
    };
    if matches!(base, "ADDQ" | "SUBQ") || (op >> 12 == 0xE && (op >> 6) & 3 != 3 && op & 0x20 == 0) {
        ops[0] = parts[0].trim_start_matches('#').to_string();
    }
    if base.starts_with("DB") {
        ops[1] = "#".into();
    }
    if base == "MOVEM" {
        ops[if op & 0x400 != 0 { 1 } else { 0 }] = "#".into();
    }
    if ops.is_empty() { Some(name) } else { Some(format!("{name} {}", ops.join(", "))) }
}

fn split(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let (mut depth, mut start) = (0, 0);
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if !s.is_empty() {
        out.push(&s[start..]);
    }
    out
}

/// One operand in the map's words for its mode.
fn mode(p: &str) -> String {
    if p.starts_with('#') {
        return "#".into();
    }
    if let Some(i) = p.find('(') {
        if i == 0 || p.starts_with("-(") {
            return p.to_string();
        }
        let inner = &p[i + 1..p.rfind(')').unwrap()];
        let mut f = inner.split(',');
        let base = f.next().unwrap();
        return if f.next().is_some() { format!("(d8, {base}, Xn)") } else { format!("(d16, {base})") };
    }
    if p.ends_with(".W") {
        return "(xxx).w".into();
    }
    if p.ends_with(".L") {
        return "(xxx).l".into();
    }
    p.to_string()
}

/// A bus of zeros that records nothing, for decoding one opcode.
struct Zeros;

impl Bus for Zeros {
    fn read(&mut self, _: Access) -> u16 {
        0
    }
    fn write(&mut self, _: Access, _: u16) {}
    fn idle(&mut self, _: u32) {}
    fn interrupt_level(&mut self) -> u8 {
        0
    }
    fn acknowledge(&mut self, _: u8) -> Option<u8> {
        None
    }
}

/// Whether the processor decodes `op`: run in supervisor mode, it takes no illegal or line A/F exception.
pub fn decodes(op: u16) -> bool {
    let mut cpu = M68000::new();
    cpu.regs.sr = 0x2700;
    cpu.regs.a[7] = 0x1000;
    cpu.regs.pc = 0x400;
    cpu.regs.prefetch = [op, 0];
    !matches!(cpu.step(&mut Zeros), Step::Exception(4 | 10 | 11))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map() -> Option<BTreeMap<u16, Option<String>>> {
        let root = std::env::var_os(crate::singlestep::CORPUS_VARIABLE)?;
        let path = Path::new(&root).join("680x0/map/68000.official.json");
        let Ok(Value::Object(members)) = json::parse(&std::fs::read(path).ok()?) else { return None };
        Some(members.into_iter().map(|(k, v)| (u16::from_str_radix(&k, 16).unwrap(), v.as_str().filter(|s| *s != "None").map(str::to_owned))).collect())
    }

    #[test]
    fn the_quick_and_special_forms_normalise_as_the_map_writes_them() {
        let dis = |words: &[u16]| {
            let w = words.to_vec();
            let i = crate::disasm::disassemble(&move |a| w.get((a / 2) as usize).copied().unwrap_or(0), 0);
            normalised(words[0], &i.mnemonic, &i.operands)
        };
        assert_eq!(dis(&[0x5240]).as_deref(), Some("ADD.w 1, D0"));
        assert_eq!(dis(&[0xE100]).as_deref(), Some("ASL.b 8, D0"));
        assert_eq!(dis(&[0x7EFF]).as_deref(), Some("MOVE.q -1, D7"));
        assert_eq!(dis(&[0x60FE]).as_deref(), Some("Bcc -2"));
        assert_eq!(dis(&[0x48E7, 0xFFFE]).as_deref(), Some("MOVEM.l #, -(A7)"));
        assert_eq!(dis(&[0x41FA, 0x0010]).as_deref(), Some("LEA (d16, PC), A0"));
        assert_eq!(dis(&[0xC188]).as_deref(), Some("EXG D0, A0"));
        assert_eq!(dis(&[0x4AFC]), None);
    }

    /// Every opcode: the processor decodes it exactly when the map lists it, and the disassembler names it as the map
    /// does. Unrun without the corpus.
    #[test]
    fn every_opcode_against_the_map() {
        let Some(map) = map() else {
            eprintln!("{} does not hold 680x0/map: not run", crate::singlestep::CORPUS_VARIABLE);
            return;
        };
        let (mut decode_wrong, mut text_wrong) = (Vec::new(), Vec::new());
        for op in 0..=0xFFFFu16 {
            let want = map.get(&op).cloned().flatten();
            if decodes(op) != want.is_some() {
                decode_wrong.push(format!("{op:04X}: map {want:?}"));
            }
            let i = crate::disasm::disassemble(&|a| if a == 0x400 { op } else { 0 }, 0x400);
            let got = normalised(op, &i.mnemonic, &i.operands);
            if got != want {
                text_wrong.push(format!("{op:04X}: map {want:?}, disassembler {got:?} ({} {})", i.mnemonic, i.operands));
            }
        }
        eprintln!("decode differs on {}, text on {}", decode_wrong.len(), text_wrong.len());
        for l in decode_wrong.iter().take(20).chain(text_wrong.iter().take(40)) {
            eprintln!("  {l}");
        }
        assert!(decode_wrong.is_empty() && text_wrong.is_empty());
    }

    /// For every instruction that falls through, the disassembler's length equals the words the processor consumed.
    #[test]
    fn the_disassembled_length_is_what_the_processor_consumes() {
        let mut wrong = Vec::new();
        let mut checked = 0;
        for op in 0..=0xFFFFu16 {
            let i = crate::disasm::disassemble(&|a| if a == 0x400 { op } else { 0 }, 0x400);
            let flow = op >> 12 == 6 || op & 0xF0F8 == 0x50C8 || ["JMP", "JSR", "RT", "TRAP", "STOP", "CHK", "DIV", "DC.W", "ILLEGAL"].iter().any(|p| i.mnemonic.starts_with(p));
            if flow {
                continue;
            }
            let mut cpu = M68000::new();
            cpu.regs.sr = 0x2700;
            cpu.regs.a[7] = 0x1000;
            cpu.regs.pc = 0x400;
            cpu.regs.prefetch = [op, 0];
            if cpu.step(&mut Zeros) != Step::Instruction {
                continue;
            }
            checked += 1;
            if cpu.regs.pc.wrapping_sub(0x400) as usize != i.bytes.len() {
                wrong.push(format!("{op:04X} {} {}: {} bytes, the processor {}", i.mnemonic, i.operands, i.bytes.len(), cpu.regs.pc.wrapping_sub(0x400)));
            }
        }
        eprintln!("{checked} instructions checked, {} differ", wrong.len());
        for w in wrong.iter().take(20) {
            eprintln!("  {w}");
        }
        assert!(wrong.is_empty());
    }
}
