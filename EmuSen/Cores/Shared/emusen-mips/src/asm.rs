//! The assembler, for test programs: the disassembler's text read back, with labels, `nop` and `.word`
//! (EmuSen_Mips.md §4). It writes what Appendix A's formats name and nothing synthetic beyond `nop`.

use std::collections::HashMap;
use std::fmt;

use crate::{Dialect, GPR_NAMES, Op, Operand};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AsmError {
    /// The source line, from 1; 0 for a single line assembled alone.
    pub line: usize,
    pub message: String,
}

impl fmt::Display for AsmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for AsmError {}

fn err<T>(message: impl Into<String>) -> Result<T, AsmError> {
    Err(AsmError { line: 0, message: message.into() })
}

fn number(s: &str) -> Option<i64> {
    let (negative, digits) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s.strip_prefix('+').unwrap_or(s)),
    };
    let v = match digits.strip_prefix("0x").or_else(|| digits.strip_prefix("0X")) {
        Some(h) => i64::from_str_radix(h, 16).ok()?,
        None => digits.parse::<i64>().ok()?,
    };
    Some(if negative { -v } else { v })
}

fn gpr(s: &str) -> Result<u32, AsmError> {
    let name = s.strip_prefix('$').unwrap_or(s);
    if let Some(i) = GPR_NAMES.iter().position(|n| *n == name) {
        return Ok(i as u32);
    }
    if name == "s8" {
        return Ok(30);
    }
    match (s.starts_with('$'), name.parse::<u32>()) {
        (true, Ok(n)) if n < 32 => Ok(n),
        _ => err(format!("not a register: {s}")),
    }
}

fn cop_register(s: &str, names: Option<&[&'static str; 32]>) -> Result<u32, AsmError> {
    if let Some(i) = names.and_then(|t| t.iter().position(|n| !n.is_empty() && *n == s)) {
        return Ok(i as u32);
    }
    match s.strip_prefix('$').map(str::parse::<u32>) {
        Some(Ok(n)) if n < 32 => Ok(n),
        _ => err(format!("not a coprocessor register: {s}")),
    }
}

fn ranged(s: &str, lo: i64, hi: i64) -> Result<u32, AsmError> {
    match number(s) {
        Some(v) if (lo..=hi).contains(&v) => Ok(v as u32),
        Some(_) => err(format!("out of range: {s}")),
        None => err(format!("not a number: {s}")),
    }
}

fn split_operands(s: &str) -> Vec<&str> {
    if s.trim().is_empty() { Vec::new() } else { s.split(',').map(str::trim).collect() }
}

/// One instruction at `pc`, a branch's or jump's target given as an address or as a label `labels` knows.
fn encode(text: &str, pc: u32, dialect: &Dialect, labels: &dyn Fn(&str) -> Option<u32>) -> Result<u32, AsmError> {
    let text = text.trim();
    let (mnemonic, rest) = text.split_once(char::is_whitespace).unwrap_or((text, ""));
    let mnemonic = mnemonic.to_ascii_lowercase();
    let operands = split_operands(rest);
    if mnemonic == "nop" && operands.is_empty() {
        return Ok(0);
    }
    if mnemonic == ".word" {
        return match operands.as_slice() {
            [v] => ranged(v, i32::MIN as i64, u32::MAX as i64),
            _ => err(".word takes one value"),
        };
    }
    for (z, table) in dialect.cop.iter().enumerate() {
        if let Some(c) = table.and_then(|t| t.commands.iter().find(|c| c.name == mnemonic)) {
            let cofun = match operands.as_slice() {
                [] => c.canonical,
                [v] => ranged(v, 0, 0x01FF_FFFF)?,
                _ => return err(format!("{mnemonic} takes at most one operand")),
            };
            if cofun & c.mask != c.select {
                return err(format!("0x{cofun:07x} is not {mnemonic}"));
            }
            return Ok((0x12 + z as u32 - 2) << 26 | 1 << 25 | cofun);
        }
    }
    let Some(op) = Op::all().find(|o| o.mnemonic() == mnemonic) else {
        return err(format!("unknown mnemonic: {mnemonic}"));
    };
    let mut kinds: &[Operand] = op.operands();
    if op == Op::Jalr && operands.len() == 1 {
        kinds = &[Operand::Rs];
    }
    if matches!(op, Op::Syscall | Op::Break) && operands.is_empty() {
        kinds = &[];
    }
    if kinds.len() != operands.len() {
        return err(format!("{mnemonic} takes {} operands", kinds.len()));
    }
    let mut word = op.canonical();
    if op == Op::Jalr && kinds.len() == 1 {
        word |= 31 << 11;
    }
    let z = match op {
        Op::Mfc(z) | Op::Cfc(z) | Op::Mtc(z) | Op::Ctc(z) | Op::Lwc(z) | Op::Swc(z) => Some(z as usize),
        _ => None,
    };
    let table = z.and_then(|z| dialect.cop[z]);
    let target = |s: &str| -> Result<u32, AsmError> {
        match labels(s).or_else(|| number(s).filter(|v| (0..=u32::MAX as i64).contains(v)).map(|v| v as u32)) {
            Some(t) => Ok(t),
            None => err(format!("not an address or a known label: {s}")),
        }
    };
    let next = pc.wrapping_add(4);
    for (kind, s) in kinds.iter().zip(&operands) {
        word |= match kind {
            Operand::Rd => gpr(s)? << 11,
            Operand::Rs => gpr(s)? << 21,
            Operand::Rt => gpr(s)? << 16,
            Operand::Sa => ranged(s, 0, 31)? << 6,
            Operand::Simm => ranged(s, -0x8000, 0x7FFF)? & 0xFFFF,
            Operand::Uimm => ranged(s, 0, 0xFFFF)?,
            Operand::Mem => {
                let Some((offset, base)) = s.strip_suffix(')').and_then(|t| t.split_once('(')) else {
                    return err(format!("not offset(base): {s}"));
                };
                let offset = if offset.trim().is_empty() { 0 } else { ranged(offset.trim(), -0x8000, 0x7FFF)? & 0xFFFF };
                gpr(base.trim())? << 21 | offset
            }
            Operand::Branch => {
                let delta = target(s)?.wrapping_sub(next) as i32;
                if delta & 3 != 0 || !(-0x20000..=0x1FFFC).contains(&delta) {
                    return err(format!("branch out of reach: {s}"));
                }
                (delta >> 2) as u32 & 0xFFFF
            }
            Operand::Jump => {
                let t = target(s)?;
                if t & 3 != 0 || t & 0xF000_0000 != next & 0xF000_0000 {
                    return err(format!("jump out of reach: {s}"));
                }
                t >> 2 & 0x03FF_FFFF
            }
            Operand::Code => ranged(s, 0, 0xF_FFFF)? << 6,
            Operand::CopData => cop_register(s, table.map(|t| &t.data))? << 11,
            Operand::CopControl => cop_register(s, table.map(|t| &t.control))? << 11,
            Operand::CopRt => cop_register(s, table.map(|t| &t.data))? << 16,
            Operand::Cofun => ranged(s, 0, 0x01FF_FFFF)?,
        };
    }
    Ok(word)
}

/// One instruction, as the disassembler writes it, at `pc`.
pub fn assemble_line(text: &str, pc: u32, dialect: &Dialect) -> Result<u32, AsmError> {
    encode(text, pc, dialect, &|_| None)
}

fn strip_comment(line: &str) -> &str {
    let end = line.find(['#', ';']).unwrap_or(line.len());
    line[..end].trim()
}

/// A program, one instruction or `.word` to a line, placed from `origin`. `name:` defines a label at the next word;
/// `#` and `;` begin comments.
pub fn assemble(source: &str, origin: u32, dialect: &Dialect) -> Result<Vec<u32>, AsmError> {
    let mut labels = HashMap::new();
    let mut lines = Vec::new();
    let mut pc = origin;
    for (n, raw) in source.lines().enumerate() {
        let mut line = strip_comment(raw);
        while let Some((label, rest)) = line.split_once(':') {
            let label = label.trim();
            if label.is_empty() || !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.') {
                break;
            }
            if labels.insert(label.to_string(), pc).is_some() {
                return Err(AsmError { line: n + 1, message: format!("label defined twice: {label}") });
            }
            line = rest.trim();
        }
        if !line.is_empty() {
            lines.push((n + 1, line, pc));
            pc = pc.wrapping_add(4);
        }
    }
    let find = |s: &str| labels.get(s).copied();
    lines.into_iter().map(|(n, text, pc)| encode(text, pc, dialect, &find).map_err(|e| AsmError { line: n, ..e })).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode::tests::{mix, operand_bits};
    use crate::{Command, Coprocessor, decode, disassemble};

    static GTE_LIKE: Coprocessor = Coprocessor {
        data: {
            let mut n = [""; 32];
            n[0] = "VXY0";
            n
        },
        control: {
            let mut n = [""; 32];
            n[31] = "FLAG";
            n
        },
        commands: &[Command { name: "rtps", mask: 0x3F, select: 0x01, canonical: 0x018_0001 }],
    };

    fn dialect() -> Dialect {
        Dialect { cop: [None, None, Some(&GTE_LIKE), None], ..Dialect::MIPS_I }
    }

    /// Every operation with random operands: the text the disassembler writes assembles to the same word.
    #[test]
    fn disassembly_assembles_back_to_the_word_for_every_operation() {
        let d = dialect();
        let mut seed = 0x0123_4567_89AB_CDEF;
        for op in Op::all() {
            for _ in 0..256 {
                let pc = mix(&mut seed) & !3;
                let word = op.canonical() | mix(&mut seed) & operand_bits(op);
                let text = disassemble(word, pc, &d).to_string();
                let back = assemble_line(&text, pc, &d).unwrap_or_else(|e| panic!("{op:?} {word:08X} \"{text}\": {e}"));
                assert_eq!(back, word, "{op:?} \"{text}\"");
            }
        }
    }

    #[test]
    fn a_coprocessor_command_by_name_and_with_its_raw_bits() {
        let d = dialect();
        assert_eq!(assemble_line("rtps", 0, &d), Ok(0x4A18_0001));
        assert_eq!(disassemble(0x4A18_0001, 0, &d).to_string(), "rtps");
        assert_eq!(assemble_line("rtps 0x0080001", 0, &d), Ok(0x4A08_0001));
        assert!(assemble_line("rtps 0x0080002", 0, &d).is_err());
        assert_eq!(assemble_line("mfc2 t0, VXY0", 0, &d), Ok(0x4808_0000));
        assert_eq!(assemble_line("cfc2 t0, FLAG", 0, &d), Ok(0x4848_F800));
    }

    #[test]
    fn a_program_with_labels_comments_and_words() {
        let src = "
            start:  addiu t0, zero, 3      # count
            loop:   addiu t0, t0, -1
                    bne t0, zero, loop
                    nop
                    j start ; again
                    nop
            data:   .word 0xDEADBEEF
                    .word -1
        ";
        let words = assemble(src, 0x8001_0000, &Dialect::MIPS_I).unwrap();
        assert_eq!(words.len(), 8);
        assert_eq!(words[0], 0x2408_0003);
        assert_eq!(words[2], 0x1500_FFFE);
        assert_eq!(words[4], 0x0800_4000);
        assert_eq!(&words[6..], &[0xDEAD_BEEF, 0xFFFF_FFFF]);
        assert_eq!(decode(words[1]), Op::Addiu);
    }

    #[test]
    fn errors_name_their_line() {
        let e = assemble("nop\n  frob t0\n", 0, &Dialect::MIPS_I).unwrap_err();
        assert_eq!(e.line, 2);
        assert!(assemble_line("addiu t0, t0, 0x8000", 0, &Dialect::MIPS_I).is_err());
        assert!(assemble_line("beq t0, t0, 0x00040000", 0, &Dialect::MIPS_I).is_err());
        assert!(assemble_line("j 0x10000000", 0, &Dialect::MIPS_I).is_err());
        assert!(assemble_line("lw t0, 4(t9", 0, &Dialect::MIPS_I).is_err());
        assert_eq!(assemble_line("lw t0, (t9)", 0, &Dialect::MIPS_I), Ok(0x8F28_0000));
        assert_eq!(assemble_line("addu $8, $s8, $31", 0, &Dialect::MIPS_I), Ok(0x03DF_4021));
    }
}
