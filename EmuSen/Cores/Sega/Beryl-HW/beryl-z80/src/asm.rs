//! The Z80 assembler, a development tool for test programs and the open boot programs: Zilog's syntax as the Z80 CPU
//! User Manual writes it and as `disasm` prints it, the undocumented forms by "The Undocumented Z80 Documented"'s names,
//! with `beryl_asm`'s labels, constants, conditionals and includes (Beryl_Z80.md §8). An instruction is encoded as
//! written; where two encodings do the same, the documented and shorter one is taken.

use std::collections::{BTreeMap, HashMap};

use beryl_asm::{Ctx, Expr, Kind, Word, err, eval, fits_either, fits_signed, parse_expr, split_top, top_open};
pub use beryl_asm::{AsmError, Program};

type Stmt = beryl_asm::Stmt<Line>;

// ---------------------------------------------------------------------------------------------------------------------
// Operands

/// An index register: IX (`$DD`) or IY (`$FD`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ix {
    X,
    Y,
}

impl Ix {
    fn prefix(self) -> u8 {
        if self == Ix::X { 0xDD } else { 0xFD }
    }
}

#[derive(Clone, Debug)]
enum Op {
    /// B, C, D, E, H, L, A by their field codes (0-5, 7).
    R(u8),
    /// An index register's half: high (H's code) or low (L's).
    Half(Ix, bool),
    I,
    Rr,
    /// BC, DE, HL, SP by their field codes.
    Rp(u8),
    Af,
    AfAlt,
    Index(Ix),
    MemHl,
    MemBc,
    MemDe,
    MemSp,
    MemC,
    MemIx(Ix, Expr),
    Mem(Expr),
    Cond(u8),
    Imm(Expr),
}

const CONDS: [&str; 8] = ["NZ", "Z", "NC", "C", "PO", "PE", "P", "M"];

fn parse_operand(t: &str, sc: &str) -> Result<Op, String> {
    let u = t.to_ascii_uppercase();
    let reg = |s: &str| -> Option<Op> {
        Some(match s {
            "B" => Op::R(0),
            "C" => Op::R(1),
            "D" => Op::R(2),
            "E" => Op::R(3),
            "H" => Op::R(4),
            "L" => Op::R(5),
            "A" => Op::R(7),
            "IXH" => Op::Half(Ix::X, true),
            "IXL" => Op::Half(Ix::X, false),
            "IYH" => Op::Half(Ix::Y, true),
            "IYL" => Op::Half(Ix::Y, false),
            "I" => Op::I,
            "R" => Op::Rr,
            "BC" => Op::Rp(0),
            "DE" => Op::Rp(1),
            "HL" => Op::Rp(2),
            "SP" => Op::Rp(3),
            "AF" => Op::Af,
            "AF'" => Op::AfAlt,
            "IX" => Op::Index(Ix::X),
            "IY" => Op::Index(Ix::Y),
            _ => {
                let c = CONDS.iter().position(|c| *c == s)?;
                Op::Cond(c as u8)
            }
        })
    };
    if let Some(r) = reg(u.as_str()) {
        return Ok(r);
    }
    if u.starts_with('(') && u.ends_with(')') && top_open(&u) == Some(0) {
        let inner = u[1..u.len() - 1].trim();
        match inner {
            "HL" => return Ok(Op::MemHl),
            "BC" => return Ok(Op::MemBc),
            "DE" => return Ok(Op::MemDe),
            "SP" => return Ok(Op::MemSp),
            "C" => return Ok(Op::MemC),
            _ => {}
        }
        for (name, ix) in [("IX", Ix::X), ("IY", Ix::Y)] {
            if let Some(rest) = inner.strip_prefix(name) {
                let rest = rest.trim();
                if rest.is_empty() {
                    return Ok(Op::MemIx(ix, Expr::Num(0)));
                }
                if rest.starts_with('+') || rest.starts_with('-') {
                    let raw = t.trim();
                    let at = raw.to_ascii_uppercase().find(name).unwrap() + 2;
                    return Ok(Op::MemIx(ix, parse_expr(raw[at..raw.len() - 1].trim(), sc)?));
                }
            }
        }
        let raw = t.trim();
        return Ok(Op::Mem(parse_expr(&raw[1..raw.len() - 1], sc)?));
    }
    Ok(Op::Imm(parse_expr(t, sc)?))
}

// ---------------------------------------------------------------------------------------------------------------------
// Statements

#[derive(Clone, Debug)]
enum Data {
    Expr(Expr),
    Bytes(Vec<u8>),
}

/// A line the Z80's assembler reads itself; labels, constants, conditionals and includes are `beryl_asm`'s.
#[derive(Clone, Debug)]
enum Line {
    Org(Expr),
    Align(Expr),
    /// Bytes (1) or little-endian words (2).
    Data(u8, Vec<Data>),
    Ds(Expr, Expr),
    Inst { name: String, ops: Vec<Op> },
}

fn word(w: &Word) -> Result<Line, AsmError> {
    let line = w.line;
    let args = w.args;
    let e = |s: &str| w.expr(s);
    let data = |size: u8| -> Result<Line, AsmError> {
        let mut items = Vec::new();
        for a in split_top(args) {
            let q = a.as_bytes();
            if q.len() >= 2 && (q[0] == b'"' || (q[0] == b'\'' && q.len() > 3)) && q[q.len() - 1] == q[0] {
                if size != 1 {
                    return err(line, "a string is a list of bytes: db");
                }
                items.push(Data::Bytes(q[1..q.len() - 1].to_vec()));
            } else {
                items.push(Data::Expr(e(&a)?));
            }
        }
        Ok(Line::Data(size, items))
    };
    Ok(match w.word {
        "org" => Line::Org(e(args)?),
        "align" => Line::Align(e(args)?),
        "db" | "defb" | "defm" | "dc.b" | "byte" => data(1)?,
        "dw" | "defw" | "dc.w" | "word" => data(2)?,
        "ds" | "defs" => {
            let a = split_top(args);
            let fill = if a.len() > 1 { e(&a[1])? } else { Expr::Num(0) };
            Line::Ds(e(a.first().map(String::as_str).unwrap_or(""))?, fill)
        }
        _ => {
            let ops = if args.trim().is_empty() { Vec::new() } else { split_top(args).iter().map(|o| parse_operand(o, w.scope).map_err(|m| AsmError { line, message: m })).collect::<Result<_, _>>()? };
            Line::Inst { name: w.word.to_ascii_uppercase(), ops }
        }
    })
}

// ---------------------------------------------------------------------------------------------------------------------
// Encoding

struct Enc<'a> {
    ctx: Ctx<'a>,
    /// The last pass: every value must be known and in range.
    fin: bool,
    line: usize,
    out: Vec<u8>,
}

impl Enc<'_> {
    fn bad<T>(&self) -> Result<T, AsmError> {
        err(self.line, "bad operands")
    }

    fn value(&self, e: &Expr) -> Result<i64, AsmError> {
        match eval(e, &self.ctx) {
            Some(v) => Ok(v),
            None if self.fin => err(self.line, "undefined symbol"),
            None => Ok(0),
        }
    }

    fn check(&self, ok: bool, what: &str) -> Result<(), AsmError> {
        if self.fin && !ok { err(self.line, what) } else { Ok(()) }
    }

    fn n(&mut self, e: &Expr) -> Result<(), AsmError> {
        let v = self.value(e)?;
        self.check(fits_either(v, 8), "byte out of range")?;
        self.out.push(v as u8);
        Ok(())
    }

    fn nn(&mut self, e: &Expr) -> Result<(), AsmError> {
        let v = self.value(e)?;
        self.check(fits_either(v, 16), "word out of range")?;
        self.out.extend_from_slice(&(v as u16).to_le_bytes());
        Ok(())
    }

    fn disp(&mut self, e: &Expr) -> Result<u8, AsmError> {
        let v = self.value(e)?;
        self.check(fits_signed(v, 8), "index displacement out of range")?;
        Ok(v as u8)
    }

    /// A relative jump's displacement to `e` from the instruction at `pc` of two bytes.
    fn rel(&mut self, e: &Expr) -> Result<(), AsmError> {
        let v = self.value(e)?;
        let d = v - (self.ctx.pc + 2);
        self.check(fits_signed(d, 8), "relative jump out of range")?;
        self.out.push(d as u8);
        Ok(())
    }
}

/// What an 8-bit register operand selects once its prefix is known: its field code, the prefix, the displacement.
struct Sel {
    code: u8,
    ix: Option<Ix>,
    disp: Option<Expr>,
}

/// The operands' 8-bit registers as one instruction's fields: (HL) as 6, (IX+d) as 6 with DD and its displacement, an
/// index half as H's or L's code with its prefix. A half with H, L or a memory operand, or IX with IY, is refused.
fn sel(en: &Enc, ops: &[&Op]) -> Result<Vec<Sel>, AsmError> {
    let mut ix: Option<Ix> = None;
    let mut half = false;
    let mut mem_ix = false;
    let mut plain_hl = false;
    let mut out = Vec::new();
    for op in ops {
        let s = match op {
            Op::R(r) => {
                plain_hl |= *r == 4 || *r == 5;
                Sel { code: *r, ix: None, disp: None }
            }
            Op::MemHl => {
                plain_hl = true;
                Sel { code: 6, ix: None, disp: None }
            }
            Op::Half(x, h) => {
                if ix.is_some_and(|i| i != *x) {
                    return en.bad();
                }
                ix = Some(*x);
                half = true;
                Sel { code: if *h { 4 } else { 5 }, ix: Some(*x), disp: None }
            }
            Op::MemIx(x, d) => {
                if ix.is_some_and(|i| i != *x) {
                    return en.bad();
                }
                ix = Some(*x);
                mem_ix = true;
                Sel { code: 6, ix: Some(*x), disp: Some(d.clone()) }
            }
            _ => return en.bad(),
        };
        out.push(s);
    }
    let mem_hl = ops.iter().any(|o| matches!(o, Op::MemHl));
    if half && (plain_hl || mem_ix) || mem_ix && mem_hl {
        return en.bad();
    }
    Ok(out)
}

/// The prefix and the displacement of a selection, around `opcode`; `after` follows the displacement.
fn emit_sel(en: &mut Enc, s: &[Sel], opcode: u8, after: Option<&Expr>) -> Result<(), AsmError> {
    if let Some(x) = s.iter().find_map(|s| s.ix) {
        en.out.push(x.prefix());
    }
    en.out.push(opcode);
    if let Some(d) = s.iter().find_map(|s| s.disp.as_ref()) {
        let d = en.disp(d)?;
        en.out.push(d);
    }
    if let Some(a) = after {
        en.n(a)?;
    }
    Ok(())
}

/// HL, IX or IY as the pair code 2 with its prefix.
fn hl_like(op: &Op) -> Option<Option<Ix>> {
    match op {
        Op::Rp(2) => Some(None),
        Op::Index(x) => Some(Some(*x)),
        _ => None,
    }
}

/// A 16-bit pair of the `rp` table (BC, DE, HL, SP), HL replaced by `ix` where one is given.
fn rp_code(op: &Op, ix: Option<Ix>) -> Option<u8> {
    match (op, ix) {
        (Op::Rp(p @ (0 | 1 | 3)), _) => Some(*p),
        (Op::Rp(2), None) => Some(2),
        (Op::Index(x), Some(i)) if *x == i => Some(2),
        _ => None,
    }
}

fn prefix(en: &mut Enc, ix: Option<Ix>) {
    if let Some(x) = ix {
        en.out.push(x.prefix());
    }
}

fn bit_number(en: &Enc, e: &Expr) -> Result<u8, AsmError> {
    let b = en.value(e)?;
    en.check((0..8).contains(&b), "bit number out of range")?;
    Ok(b as u8 & 7)
}

const ALU: [&str; 8] = ["ADD", "ADC", "SUB", "SBC", "AND", "XOR", "OR", "CP"];
const ROT: [&str; 8] = ["RLC", "RRC", "RL", "RR", "SLA", "SRA", "SLL", "SRL"];

fn implied(name: &str) -> Option<&'static [u8]> {
    Some(match name {
        "NOP" => &[0x00],
        "HALT" => &[0x76],
        "DI" => &[0xF3],
        "EI" => &[0xFB],
        "EXX" => &[0xD9],
        "RLCA" => &[0x07],
        "RRCA" => &[0x0F],
        "RLA" => &[0x17],
        "RRA" => &[0x1F],
        "DAA" => &[0x27],
        "CPL" => &[0x2F],
        "SCF" => &[0x37],
        "CCF" => &[0x3F],
        "NEG" => &[0xED, 0x44],
        "RETN" => &[0xED, 0x45],
        "RETI" => &[0xED, 0x4D],
        "RRD" => &[0xED, 0x67],
        "RLD" => &[0xED, 0x6F],
        "LDI" => &[0xED, 0xA0],
        "CPI" => &[0xED, 0xA1],
        "INI" => &[0xED, 0xA2],
        "OUTI" => &[0xED, 0xA3],
        "LDD" => &[0xED, 0xA8],
        "CPD" => &[0xED, 0xA9],
        "IND" => &[0xED, 0xAA],
        "OUTD" => &[0xED, 0xAB],
        "LDIR" => &[0xED, 0xB0],
        "CPIR" => &[0xED, 0xB1],
        "INIR" => &[0xED, 0xB2],
        "OTIR" => &[0xED, 0xB3],
        "LDDR" => &[0xED, 0xB8],
        "CPDR" => &[0xED, 0xB9],
        "INDR" => &[0xED, 0xBA],
        "OTDR" => &[0xED, 0xBB],
        _ => return None,
    })
}

fn encode(en: &mut Enc, name: &str, ops: &[Op]) -> Result<(), AsmError> {
    if let Some(bytes) = implied(name) {
        if !ops.is_empty() {
            return en.bad();
        }
        en.out.extend_from_slice(bytes);
        return Ok(());
    }
    if let Some(y) = ALU.iter().position(|a| *a == name) {
        let y = y as u8;
        // ADD, ADC and SBC of pairs.
        if let [dst, src] = ops {
            if let Some(ix) = hl_like(dst) {
                let p = rp_code(src, ix).ok_or(()).or_else(|_| en.bad())?;
                return match y {
                    0 => {
                        prefix(en, ix);
                        en.out.push(0x09 | p << 4);
                        Ok(())
                    }
                    1 | 3 if ix.is_none() => {
                        en.out.extend_from_slice(&[0xED, if y == 1 { 0x4A } else { 0x42 } | p << 4]);
                        Ok(())
                    }
                    _ => en.bad(),
                };
            }
        }
        let src = match ops {
            [Op::R(7), s] => s,
            [s] => s,
            _ => return en.bad(),
        };
        if let Op::Imm(n) = src {
            en.out.push(0xC6 | y << 3);
            return en.n(n);
        }
        let s = sel(en, &[src])?;
        return emit_sel(en, &s, 0x80 | y << 3 | s[0].code, None);
    }
    if let Some(y) = ROT.iter().position(|a| *a == name) {
        return cb(en, y as u8, None, ops);
    }
    match name {
        "BIT" | "RES" | "SET" => {
            let base = match name {
                "BIT" => 1,
                "RES" => 2,
                _ => 3,
            };
            let (b, rest) = ops.split_first().ok_or(()).or_else(|_| en.bad())?;
            let Op::Imm(b) = b else { return en.bad() };
            let b = bit_number(en, b)?;
            if base == 1 && rest.len() != 1 {
                return en.bad();
            }
            cb(en, base << 3 | b, Some(base), rest)
        }
        "INC" | "DEC" => {
            let [op] = ops else { return en.bad() };
            let dec = (name == "DEC") as u8;
            if let Some(ix) = hl_like(op).or(match op {
                Op::Rp(_) => Some(None),
                _ => None,
            }) {
                let p = rp_code(op, ix).ok_or(()).or_else(|_| en.bad())?;
                prefix(en, ix);
                en.out.push(0x03 | dec << 3 | p << 4);
                return Ok(());
            }
            let s = sel(en, &[op])?;
            emit_sel(en, &s, 0x04 | dec | s[0].code << 3, None)
        }
        "PUSH" | "POP" => {
            let [op] = ops else { return en.bad() };
            let base = if name == "PUSH" { 0xC5 } else { 0xC1 };
            let (ix, p) = match op {
                Op::Af => (None, 3),
                Op::Index(x) => (Some(*x), 2),
                Op::Rp(p @ 0..=2) => (None, *p),
                _ => return en.bad(),
            };
            prefix(en, ix);
            en.out.push(base | p << 4);
            Ok(())
        }
        "EX" => match ops {
            [Op::Af, Op::AfAlt] => {
                en.out.push(0x08);
                Ok(())
            }
            [Op::Rp(1), Op::Rp(2)] => {
                en.out.push(0xEB);
                Ok(())
            }
            [Op::MemSp, r] => {
                let ix = hl_like(r).ok_or(()).or_else(|_| en.bad())?;
                prefix(en, ix);
                en.out.push(0xE3);
                Ok(())
            }
            _ => en.bad(),
        },
        "JP" => match ops {
            [Op::Imm(t)] => {
                en.out.push(0xC3);
                en.nn(t)
            }
            [Op::MemHl] => {
                en.out.push(0xE9);
                Ok(())
            }
            [Op::MemIx(x, d)] if matches!(d, Expr::Num(0)) => {
                en.out.extend_from_slice(&[x.prefix(), 0xE9]);
                Ok(())
            }
            [c, Op::Imm(t)] => {
                let c = cond(en, c, 8)?;
                en.out.push(0xC2 | c << 3);
                en.nn(t)
            }
            _ => en.bad(),
        },
        "CALL" => match ops {
            [Op::Imm(t)] => {
                en.out.push(0xCD);
                en.nn(t)
            }
            [c, Op::Imm(t)] => {
                let c = cond(en, c, 8)?;
                en.out.push(0xC4 | c << 3);
                en.nn(t)
            }
            _ => en.bad(),
        },
        "RET" => match ops {
            [] => {
                en.out.push(0xC9);
                Ok(())
            }
            [c] => {
                let c = cond(en, c, 8)?;
                en.out.push(0xC0 | c << 3);
                Ok(())
            }
            _ => en.bad(),
        },
        "JR" => match ops {
            [Op::Imm(t)] => {
                en.out.push(0x18);
                en.rel(t)
            }
            [c, Op::Imm(t)] => {
                let c = cond(en, c, 4)?;
                en.out.push(0x20 | c << 3);
                en.rel(t)
            }
            _ => en.bad(),
        },
        "DJNZ" => {
            let [Op::Imm(t)] = ops else { return en.bad() };
            en.out.push(0x10);
            en.rel(t)
        }
        "RST" => {
            let [Op::Imm(t)] = ops else { return en.bad() };
            let v = en.value(t)?;
            en.check(v & !0x38 == 0, "RST takes $00, $08 ... $38")?;
            en.out.push(0xC7 | (v as u8 & 0x38));
            Ok(())
        }
        "IM" => {
            let [Op::Imm(m)] = ops else { return en.bad() };
            let m = en.value(m)?;
            en.check((0..=2).contains(&m), "IM takes 0, 1 or 2")?;
            en.out.extend_from_slice(&[0xED, [0x46, 0x56, 0x5E][m.clamp(0, 2) as usize]]);
            Ok(())
        }
        "IN" => match ops {
            [Op::R(7), Op::Mem(n)] => {
                en.out.push(0xDB);
                en.n(n)
            }
            [Op::R(r), Op::MemC] => {
                en.out.extend_from_slice(&[0xED, 0x40 | r << 3]);
                Ok(())
            }
            [Op::MemC] => {
                en.out.extend_from_slice(&[0xED, 0x70]);
                Ok(())
            }
            _ => en.bad(),
        },
        "OUT" => match ops {
            [Op::Mem(n), Op::R(7)] => {
                en.out.push(0xD3);
                en.n(n)
            }
            [Op::MemC, Op::R(r)] => {
                en.out.extend_from_slice(&[0xED, 0x41 | r << 3]);
                Ok(())
            }
            [Op::MemC, Op::Imm(z)] => {
                let z = en.value(z)?;
                en.check(z == 0, "OUT (C) writes a register or 0")?;
                en.out.extend_from_slice(&[0xED, 0x71]);
                Ok(())
            }
            _ => en.bad(),
        },
        "LD" => ld(en, ops),
        _ => err(en.line, format!("unknown instruction `{name}`")),
    }
}

/// A condition's code, among the first `count` (four for JR); C is the register's spelling.
fn cond(en: &Enc, op: &Op, count: u8) -> Result<u8, AsmError> {
    let c = match op {
        Op::Cond(c) => *c,
        Op::R(1) => 3,
        _ => return en.bad(),
    };
    if c >= count { en.bad() } else { Ok(c) }
}

/// The CB table: a rotation (`base` none), BIT, RES or SET; an indexed one with a register also writes the result to it.
fn cb(en: &mut Enc, y: u8, base: Option<u8>, ops: &[Op]) -> Result<(), AsmError> {
    let top = base.unwrap_or(0) << 6 | (y & 7) << 3;
    match ops {
        [Op::MemIx(x, d)] | [Op::MemIx(x, d), Op::R(_)] => {
            let z = match ops.get(1) {
                Some(Op::R(r)) if base != Some(1) => *r,
                Some(_) => return en.bad(),
                None => 6,
            };
            let d = en.disp(d)?;
            en.out.extend_from_slice(&[x.prefix(), 0xCB, d, top | z]);
            Ok(())
        }
        [op] => {
            let s = sel(en, &[op])?;
            if s[0].ix.is_some() {
                return en.bad();
            }
            en.out.extend_from_slice(&[0xCB, top | s[0].code]);
            Ok(())
        }
        _ => en.bad(),
    }
}

fn ld(en: &mut Enc, ops: &[Op]) -> Result<(), AsmError> {
    let [dst, src] = ops else { return en.bad() };
    match (dst, src) {
        (Op::R(7), Op::MemBc) => en.out.push(0x0A),
        (Op::R(7), Op::MemDe) => en.out.push(0x1A),
        (Op::MemBc, Op::R(7)) => en.out.push(0x02),
        (Op::MemDe, Op::R(7)) => en.out.push(0x12),
        (Op::R(7), Op::Mem(a)) => {
            en.out.push(0x3A);
            en.nn(a)?
        }
        (Op::Mem(a), Op::R(7)) => {
            en.out.push(0x32);
            en.nn(a)?
        }
        (Op::I, Op::R(7)) => en.out.extend_from_slice(&[0xED, 0x47]),
        (Op::Rr, Op::R(7)) => en.out.extend_from_slice(&[0xED, 0x4F]),
        (Op::R(7), Op::I) => en.out.extend_from_slice(&[0xED, 0x57]),
        (Op::R(7), Op::Rr) => en.out.extend_from_slice(&[0xED, 0x5F]),
        (Op::Rp(3), r) if hl_like(r).is_some() => {
            prefix(en, hl_like(r).unwrap());
            en.out.push(0xF9);
        }
        (p @ (Op::Rp(_) | Op::Index(_)), Op::Imm(v)) => {
            let ix = hl_like(p).unwrap_or(None);
            let c = rp_code(p, ix).ok_or(()).or_else(|_| en.bad())?;
            prefix(en, ix);
            en.out.push(0x01 | c << 4);
            en.nn(v)?
        }
        (p @ (Op::Rp(_) | Op::Index(_)), Op::Mem(a)) => {
            if let Some(ix) = hl_like(p) {
                prefix(en, ix);
                en.out.push(0x2A);
            } else {
                let c = rp_code(p, None).ok_or(()).or_else(|_| en.bad())?;
                en.out.extend_from_slice(&[0xED, 0x4B | c << 4]);
            }
            en.nn(a)?
        }
        (Op::Mem(a), p @ (Op::Rp(_) | Op::Index(_))) => {
            if let Some(ix) = hl_like(p) {
                prefix(en, ix);
                en.out.push(0x22);
            } else {
                let c = rp_code(p, None).ok_or(()).or_else(|_| en.bad())?;
                en.out.extend_from_slice(&[0xED, 0x43 | c << 4]);
            }
            en.nn(a)?
        }
        (d, Op::Imm(n)) => {
            let s = sel(en, &[d])?;
            emit_sel(en, &s, 0x06 | s[0].code << 3, Some(n))?
        }
        (d, s) => {
            let sl = sel(en, &[d, s])?;
            if sl[0].code == 6 && sl[1].code == 6 {
                return en.bad();
            }
            emit_sel(en, &sl, 0x40 | sl[0].code << 3 | sl[1].code, None)?
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------------------------------
// Layout

/// One walk over the statements; with `emit`, the bytes. Every instruction's length is fixed by its operands' forms,
/// so the walks only settle the labels.
fn walk(stmts: &[Stmt], origin: u32, symbols: &mut HashMap<String, i64>, mut out: Option<&mut BTreeMap<i64, u8>>) -> Result<bool, AsmError> {
    let mut pc = origin as i64;
    let mut changed = false;
    let fin = out.is_some();
    let put = |out: &mut Option<&mut BTreeMap<i64, u8>>, at: i64, b: &[u8]| {
        if let Some(o) = out.as_mut() {
            for (k, v) in b.iter().enumerate() {
                o.insert(at + k as i64, *v);
            }
        }
    };
    for s in stmts {
        let line = s.line;
        let known = |symbols: &HashMap<String, i64>, e: &Expr, pc: i64, what: &str| eval(e, &Ctx { symbols, pc }).ok_or(AsmError { line, message: format!("{what} needs a known value") });
        match &s.kind {
            Kind::Label(n) => {
                if symbols.insert(n.clone(), pc) != Some(pc) {
                    changed = true;
                }
            }
            Kind::Equ(n, e) => {
                if let Some(v) = eval(e, &Ctx { symbols, pc }) {
                    if symbols.insert(n.clone(), v) != Some(v) {
                        changed = true;
                    }
                } else if fin {
                    return err(line, format!("constant `{n}` cannot be evaluated"));
                }
            }
            Kind::Line(Line::Org(e)) => {
                let v = known(symbols, e, pc, "`org`")?;
                if v < pc {
                    return err(line, "`org` moves backward");
                }
                pc = v;
            }
            Kind::Line(Line::Align(e)) => {
                let a = known(symbols, e, pc, "`align`")?;
                if a <= 0 {
                    return err(line, "bad alignment");
                }
                pc = (pc + a - 1) / a * a;
            }
            Kind::Line(Line::Data(size, items)) => {
                for it in items {
                    match it {
                        Data::Bytes(b) => {
                            put(&mut out, pc, b);
                            pc += b.len() as i64;
                        }
                        Data::Expr(e) => {
                            let v = match eval(e, &Ctx { symbols, pc }) {
                                Some(v) => v,
                                None if fin => return err(line, "undefined symbol in data"),
                                None => 0,
                            };
                            if fin && !fits_either(v, 8 * *size as u32) {
                                return err(line, "data out of range for its size");
                            }
                            put(&mut out, pc, &(v as u16).to_le_bytes()[..*size as usize]);
                            pc += *size as i64;
                        }
                    }
                }
            }
            Kind::Line(Line::Ds(count, fill)) => {
                let c = known(symbols, count, pc, "a count")?;
                let f = eval(fill, &Ctx { symbols, pc }).unwrap_or(0);
                for _ in 0..c.max(0) {
                    put(&mut out, pc, &[f as u8]);
                    pc += 1;
                }
            }
            Kind::Line(Line::Inst { name, ops }) => {
                let mut en = Enc { ctx: Ctx { symbols, pc }, fin, line, out: Vec::with_capacity(4) };
                encode(&mut en, name, ops)?;
                let bytes = en.out;
                put(&mut out, pc, &bytes);
                pc += bytes.len() as i64;
            }
        }
    }
    Ok(changed)
}

/// A program placed from `origin`, with `defines` known to its conditionals and `include` reading its includes.
pub fn assemble_with(source: &str, origin: u32, defines: &[(&str, i64)], include: &dyn Fn(&str) -> Option<String>) -> Result<Program, AsmError> {
    let defs: HashMap<String, i64> = defines.iter().map(|(k, v)| (k.to_string(), *v)).collect();
    let mut stmts = Vec::new();
    beryl_asm::parse(source, &defs, include, &mut stmts, 0, &["equ", "=", "defl"], &word)?;
    let mut symbols = defs;
    let mut rounds = 0;
    while walk(&stmts, origin, &mut symbols, None)? {
        rounds += 1;
        if rounds > 64 {
            return err(0, "the layout does not settle");
        }
    }
    let mut bytes = BTreeMap::new();
    walk(&stmts, origin, &mut symbols, Some(&mut bytes))?;
    beryl_asm::image(origin, bytes, symbols)
}

/// A program with no includes or defines.
pub fn assemble(source: &str, origin: u32) -> Result<Program, AsmError> {
    assemble_with(source, origin, &[], &|_| None)
}

/// One instruction, as the disassembler writes it, at `pc`: its bytes.
pub fn assemble_line(text: &str, pc: u16) -> Result<Vec<u8>, AsmError> {
    Ok(assemble(&format!(" {text}"), pc as u32)?.bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disasm::disassemble;

    fn bytes(text: &str, pc: u16) -> Vec<u8> {
        assemble_line(text, pc).unwrap_or_else(|e| panic!("{text}: {e}"))
    }

    #[test]
    fn instructions_assemble_as_the_manual_encodes_them() {
        assert_eq!(bytes("LD A,$12", 0), [0x3E, 0x12]);
        assert_eq!(bytes("ld hl,$1234", 0), [0x21, 0x34, 0x12]);
        assert_eq!(bytes("LD A,($4000)", 0), [0x3A, 0x00, 0x40]);
        assert_eq!(bytes("LD (IY+2),$99", 0), [0xFD, 0x36, 0x02, 0x99]);
        assert_eq!(bytes("LD A,(IX-5)", 0), [0xDD, 0x7E, 0xFB]);
        assert_eq!(bytes("LD H,(IX+1)", 0), [0xDD, 0x66, 0x01]);
        assert_eq!(bytes("LD IXH,IXL", 0), [0xDD, 0x65]);
        assert_eq!(bytes("LD SP,IY", 0), [0xFD, 0xF9]);
        assert_eq!(bytes("LD BC,($8000)", 0), [0xED, 0x4B, 0x00, 0x80]);
        assert_eq!(bytes("LD ($8000),HL", 0), [0x22, 0x00, 0x80]);
        assert_eq!(bytes("ADD A,B", 0), [0x80]);
        assert_eq!(bytes("SUB (HL)", 0), [0x96]);
        assert_eq!(bytes("CP $0A", 0), [0xFE, 0x0A]);
        assert_eq!(bytes("ADD IX,SP", 0), [0xDD, 0x39]);
        assert_eq!(bytes("SBC HL,DE", 0), [0xED, 0x52]);
        assert_eq!(bytes("JR $0200", 0x200), [0x18, 0xFE]);
        assert_eq!(bytes("JR NC,$0210", 0x200), [0x30, 0x0E]);
        assert_eq!(bytes("DJNZ $01FE", 0x200), [0x10, 0xFC]);
        assert_eq!(bytes("CALL C,$0100", 0), [0xDC, 0x00, 0x01]);
        assert_eq!(bytes("RET PE", 0), [0xE8]);
        assert_eq!(bytes("RST $38", 0), [0xFF]);
        assert_eq!(bytes("JP (IY)", 0), [0xFD, 0xE9]);
        assert_eq!(bytes("EX AF,AF'", 0), [0x08]);
        assert_eq!(bytes("EX (SP),IX", 0), [0xDD, 0xE3]);
        assert_eq!(bytes("IN A,($DC)", 0), [0xDB, 0xDC]);
        assert_eq!(bytes("OUT ($BF),A", 0), [0xD3, 0xBF]);
        assert_eq!(bytes("IN (C)", 0), [0xED, 0x70]);
        assert_eq!(bytes("OUT (C),0", 0), [0xED, 0x71]);
        assert_eq!(bytes("IM 1", 0), [0xED, 0x56]);
        assert_eq!(bytes("SLL A", 0), [0xCB, 0x37]);
        assert_eq!(bytes("BIT 7,(HL)", 0), [0xCB, 0x7E]);
        assert_eq!(bytes("SET 0,(IY-1),B", 0), [0xFD, 0xCB, 0xFF, 0xC0]);
        assert_eq!(bytes("RLC (IX+3)", 0), [0xDD, 0xCB, 0x03, 0x06]);
        assert_eq!(bytes("OTIR", 0), [0xED, 0xB3]);
        assert_eq!(bytes("PUSH IY", 0), [0xFD, 0xE5]);
        assert_eq!(bytes("POP AF", 0), [0xF1]);
        assert_eq!(bytes("INC IXL", 0), [0xDD, 0x2C]);
        assert_eq!(bytes("DEC (IX+$7F)", 0), [0xDD, 0x35, 0x7F]);
    }

    #[test]
    fn labels_constants_and_data() {
        let src = "
PORT    equ $BF
start:  di
        im 1
        ld sp,$DFF0
.loop:  in a,(PORT)
        jr z,.loop
        jp start
        dw start,text-start
text:   db \"AB\",0,'C'
        ds 3,$FF
end:
";
        let p = assemble(src, 0).unwrap();
        assert_eq!(&p.bytes[..11], &[0xF3, 0xED, 0x56, 0x31, 0xF0, 0xDF, 0xDB, 0xBF, 0x28, 0xFC, 0xC3]);
        assert_eq!(p.symbols["start.loop"], 6);
        let text = p.symbols["text"] as usize;
        assert_eq!(&p.bytes[text - 4..text], &[0x00, 0x00, text as u8, 0x00]);
        assert_eq!(&p.bytes[text..], b"AB\0C\xFF\xFF\xFF");
        assert_eq!(p.symbols["end"], text as i64 + 7);
    }

    #[test]
    fn errors_name_their_line() {
        assert_eq!(assemble(" nop\n bogus a\n", 0).unwrap_err().line, 2);
        assert!(assemble(" jr far\n ds 200\nfar: nop\n", 0).is_err(), "a relative jump beyond 127");
        assert!(assemble(" ld a,256\n", 0).is_err());
        assert!(assemble(" ld (ix+128),a\n", 0).is_err());
        assert!(assemble(" ld ixh,h\n", 0).is_err(), "an index half beside H");
        assert!(assemble(" ld (hl),(hl)\n", 0).is_err());
        assert!(assemble(" jr pe,0\n", 0).is_err(), "JR has four conditions");
        assert!(assemble(" ld a,undefined\n", 0).is_err());
        assert_eq!(assemble(" ld a,(2+3)*4\n", 0).unwrap().bytes, [0x3E, 20], "a parenthesised expression is a value");
    }

    /// The opcodes the disassembler prints by the text of another encoding: they read back as the same text, and the
    /// encoding taken is the documented one (Beryl_Z80.md §8.3).
    fn non_canonical(seq: &[u8]) -> Option<&'static str> {
        match *seq {
            [0xDD | 0xFD, op, ..] if !uses_index(op) => Some("a prefix that changes nothing"),
            [0xED, 0x4C | 0x54 | 0x5C | 0x64 | 0x6C | 0x74 | 0x7C] => Some("NEG"),
            [0xED, 0x55 | 0x5D | 0x65 | 0x6D | 0x75 | 0x7D] => Some("RETN"),
            [0xED, 0x4E | 0x66 | 0x6E | 0x76 | 0x7E] => Some("IM"),
            [0xED, 0x63 | 0x6B, ..] => Some("ED's LD of HL"),
            [0xDD | 0xFD, 0xCB, _, op] if op >> 6 == 1 && op & 7 != 6 => Some("an indexed BIT with a register field"),
            _ => None,
        }
    }

    /// Whether DD or FD changes the instruction after it: one that names HL, H, L or (HL), JP (HL), EX (SP),HL or the CB page.
    fn uses_index(op: u8) -> bool {
        let r = |c: u8| c == 4 || c == 5 || c == 6;
        match op {
            0x09 | 0x19 | 0x29 | 0x39 | 0x21 | 0x22 | 0x2A | 0x23 | 0x2B | 0xE1 | 0xE5 | 0xE3 | 0xE9 | 0xF9 | 0xCB => true,
            0x76 => false,
            0x40..=0x7F => r(op >> 3 & 7) || r(op & 7),
            0x80..=0xBF => r(op & 7),
            0x00..=0x3F if matches!(op & 7, 4..=6) => r(op >> 3 & 7),
            _ => false,
        }
    }

    /// Every instruction the disassembler decodes, its text assembled again at the same address: the same bytes, or for
    /// the opcodes another encoding prints, the same text (Beryl_Z80.md §8.3).
    #[test]
    fn every_instruction_form_assembles_back_to_its_bytes() {
        let mut sequences: Vec<Vec<u8>> = (0..=255u8).map(|b| vec![b]).collect();
        for prefix in [0xCB, 0xED, 0xDD, 0xFD] {
            sequences.extend((0..=255u8).map(|b| vec![prefix, b]));
        }
        for prefix in [0xDD, 0xFD] {
            for d in [0x05, 0xFB, 0x00, 0x7F, 0x80] {
                sequences.extend((0..=255u8).map(|b| vec![prefix, 0xCB, d, b]));
            }
        }
        let (mut same, mut by_text) = (0, BTreeMap::<&str, usize>::new());
        for seq in sequences {
            if matches!(seq[..], [0xDD | 0xFD] | [0xDD | 0xFD, 0xDD | 0xFD | 0xED] | [0xCB | 0xED]) {
                continue;
            }
            for tail in [[0x11u8, 0x22, 0x33], [0x80, 0xFE, 0x7F], [0x00, 0x00, 0x00]] {
                let at = 0x4000u16;
                let mut code = seq.clone();
                code.extend_from_slice(&tail);
                let read = |a: u16| code.get(a.wrapping_sub(at) as usize).copied().unwrap_or(0);
                let i = disassemble(&read, at);
                let text = format!("{} {}", i.mnemonic, i.operands);
                let got = assemble_line(&text, at).unwrap_or_else(|e| panic!("{:02X?} `{text}`: {e}", i.bytes));
                if let Some(kind) = non_canonical(&i.bytes) {
                    let back = disassemble(&|a: u16| got.get(a.wrapping_sub(at) as usize).copied().unwrap_or(0), at);
                    assert_eq!(format!("{} {}", back.mnemonic, back.operands), text, "{:02X?}", i.bytes);
                    *by_text.entry(kind).or_default() += 1;
                } else {
                    assert_eq!(got, i.bytes, "`{text}`");
                    same += 1;
                }
            }
        }
        eprintln!("{same} instructions assemble to their own bytes; by text: {by_text:?}");
        assert!(same > 4000, "{same}");
    }
}
