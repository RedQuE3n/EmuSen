//! The 68000 disassembler for `DEBUG_DISASSEMBLE`: Motorola's syntax as the Programmer's Reference Manual writes it,
//! with the same decode the processor uses and the static reference knowable from the words alone. Checked against
//! TomHarte's instruction map opcode by opcode (Beryl_M68k.md §5.5).

use emusen_native::core::{Instruction, Reference};

const CONDITIONS: [&str; 16] = ["T", "F", "HI", "LS", "CC", "CS", "NE", "EQ", "VC", "VS", "PL", "MI", "GE", "LT", "GT", "LE"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Sz {
    B,
    W,
    L,
}

impl Sz {
    fn suffix(self) -> &'static str {
        match self {
            Sz::B => ".B",
            Sz::W => ".W",
            Sz::L => ".L",
        }
    }

    fn bits(b: u16) -> Option<Sz> {
        match b & 3 {
            0 => Some(Sz::B),
            1 => Some(Sz::W),
            2 => Some(Sz::L),
            _ => None,
        }
    }
}

/// Which effective addresses an operand allows (PRM Table 2-4's categories, as the 68000 decodes them).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Cat {
    /// Every mode.
    All,
    /// Every mode but An.
    Data,
    /// Dn and the alterable memory modes.
    DataAlterable,
    /// The alterable memory modes.
    MemoryAlterable,
    /// (An), (d16,An), (d8,An,Xn), absolute and the PC-relative modes.
    Control,
    /// The control modes that can be written.
    ControlAlterable,
    /// Every alterable mode, An included.
    Alterable,
}

fn allowed(cat: Cat, mode: u16, reg: u16) -> bool {
    let pc_or_imm = mode == 7 && reg >= 2;
    match cat {
        Cat::All => !(mode == 7 && reg > 4),
        Cat::Data => mode != 1 && !(mode == 7 && reg > 4),
        Cat::DataAlterable => mode != 1 && !(mode == 7 && reg > 1),
        Cat::MemoryAlterable => mode >= 2 && !(mode == 7 && reg > 1),
        Cat::Control => matches!(mode, 2 | 5 | 6) || (mode == 7 && reg <= 3),
        Cat::ControlAlterable => matches!(mode, 2 | 5 | 6) || (mode == 7 && reg <= 1),
        Cat::Alterable => !pc_or_imm && !(mode == 7 && reg > 1),
    }
}

fn hex(v: u32) -> String {
    format!("${v:X}")
}

fn signed(v: i32) -> String {
    if v < 0 { format!("-${:X}", v.unsigned_abs()) } else { format!("${v:X}") }
}

struct D<'a> {
    read: &'a dyn Fn(u32) -> u16,
    next: u32,
    words: Vec<u16>,
    reference: Option<(Reference, u32)>,
}

impl D<'_> {
    fn word(&mut self) -> u16 {
        let w = (self.read)(self.next);
        self.next = self.next.wrapping_add(2);
        self.words.push(w);
        w
    }

    fn long(&mut self) -> u32 {
        (self.word() as u32) << 16 | self.word() as u32
    }

    fn brief(&mut self, base: &str) -> String {
        let ext = self.word();
        let r = (ext >> 12) & 7;
        let kind = if ext & 0x8000 != 0 { 'A' } else { 'D' };
        let size = if ext & 0x800 != 0 { 'L' } else { 'W' };
        format!("{}({base},{kind}{r}.{size})", signed(ext as u8 as i8 as i32))
    }

    /// An effective address's text, reading its extension words; an absolute or PC-relative address is noted as the
    /// instruction's reference of kind `kind` when it has none yet.
    fn ea(&mut self, mode: u16, reg: u16, size: Sz, kind: Reference) -> String {
        let r = reg & 7;
        match mode & 7 {
            0 => format!("D{r}"),
            1 => format!("A{r}"),
            2 => format!("(A{r})"),
            3 => format!("(A{r})+"),
            4 => format!("-(A{r})"),
            5 => format!("{}(A{r})", signed(self.word() as i16 as i32)),
            6 => self.brief(&format!("A{r}")),
            _ => match r {
                0 => {
                    let a = self.word() as i16 as i32 as u32;
                    self.note(kind, a & 0xFF_FFFF);
                    format!("${:04X}.W", a as u16)
                }
                1 => {
                    let a = self.long();
                    self.note(kind, a & 0xFF_FFFF);
                    format!("${a:08X}.L")
                }
                2 => {
                    let base = self.next;
                    let a = base.wrapping_add(self.word() as i16 as i32 as u32);
                    self.note(kind, a & 0xFF_FFFF);
                    format!("${:06X}(PC)", a & 0xFF_FFFF)
                }
                3 => self.brief("PC"),
                _ => match size {
                    Sz::B => format!("#${:02X}", self.word() & 0xFF),
                    Sz::W => format!("#${:04X}", self.word()),
                    Sz::L => format!("#${:08X}", self.long()),
                },
            },
        }
    }

    fn note(&mut self, kind: Reference, address: u32) {
        if self.reference.is_none() {
            self.reference = Some((kind, address));
        }
    }
}

/// A MOVEM register list: bit 0 is D0 and bit 15 A7, or reversed for the predecrement form.
fn reglist(mask: u16, reversed: bool) -> String {
    let mask = if reversed { mask.reverse_bits() } else { mask };
    let mut parts = Vec::new();
    for (bank, base) in [('D', 0), ('A', 8)] {
        let mut i = 0;
        while i < 8 {
            if mask & (1 << (base + i)) != 0 {
                let start = i;
                while i + 1 < 8 && mask & (1 << (base + i + 1)) != 0 {
                    i += 1;
                }
                parts.push(if i == start { format!("{bank}{start}") } else { format!("{bank}{start}-{bank}{i}") });
            }
            i += 1;
        }
    }
    if parts.is_empty() { "#0".into() } else { parts.join("/") }
}

/// The instruction at `address`; an opcode the 68000 does not decode is `DC.W` of its word.
pub fn disassemble(read: &dyn Fn(u32) -> u16, address: u32) -> Instruction {
    let mut d = D { read, next: address, words: Vec::new(), reference: None };
    let op = d.word();
    let (mnemonic, operands) = decode(&mut d, op, address).unwrap_or_else(|| ("DC.W".into(), format!("${op:04X}")));
    let words = if mnemonic == "DC.W" { vec![op] } else { d.words };
    let reference = if mnemonic == "DC.W" { None } else { d.reference };
    Instruction { address, bytes: words.iter().flat_map(|w| w.to_be_bytes()).collect(), mnemonic, operands, reference }
}

fn decode(d: &mut D, op: u16, address: u32) -> Option<(String, String)> {
    let mode = (op >> 3) & 7;
    let reg = op & 7;
    let rx = (op >> 9) & 7;
    let m = |s: &str| s.to_string();
    let read = Reference::Read;
    let write = Reference::Write;
    Some(match op >> 12 {
        0x0 => {
            if op & 0x100 != 0 {
                if mode == 1 {
                    let sz = if op & 0x40 != 0 { Sz::L } else { Sz::W };
                    let disp = signed(d.word() as i16 as i32);
                    return Some(if op & 0x80 != 0 {
                        (format!("MOVEP{}", sz.suffix()), format!("D{rx},{disp}(A{reg})"))
                    } else {
                        (format!("MOVEP{}", sz.suffix()), format!("{disp}(A{reg}),D{rx}"))
                    });
                }
                let kind = (op >> 6) & 3;
                let cat = if kind == 0 { Cat::Data } else { Cat::DataAlterable };
                if !allowed(cat, mode, reg) {
                    return None;
                }
                let sz = if mode == 0 { Sz::L } else { Sz::B };
                let name = ["BTST", "BCHG", "BCLR", "BSET"][kind as usize];
                (m(name), format!("D{rx},{}", d.ea(mode, reg, sz, if kind == 0 { read } else { write })))
            } else if rx == 4 {
                let kind = (op >> 6) & 3;
                let cat = if kind == 0 { Cat::Data } else { Cat::DataAlterable };
                if !allowed(cat, mode, reg) || (mode == 7 && reg == 4) {
                    return None;
                }
                let n = d.word() & 0xFF;
                let name = ["BTST", "BCHG", "BCLR", "BSET"][kind as usize];
                (m(name), format!("#{n},{}", d.ea(mode, reg, if mode == 0 { Sz::L } else { Sz::B }, if kind == 0 { read } else { write })))
            } else if op & 0xFF == 0x3C || op & 0xFF == 0x7C {
                let name = match rx {
                    0 => "ORI",
                    1 => "ANDI",
                    5 => "EORI",
                    _ => return None,
                };
                if op & 0x40 != 0 { (m(name), format!("#${:04X},SR", d.word())) } else { (m(name), format!("#${:02X},CCR", d.word() & 0xFF)) }
            } else {
                let sz = Sz::bits(op >> 6)?;
                let name = match rx {
                    0 => "ORI",
                    1 => "ANDI",
                    2 => "SUBI",
                    3 => "ADDI",
                    5 => "EORI",
                    6 => "CMPI",
                    _ => return None,
                };
                if !allowed(Cat::DataAlterable, mode, reg) {
                    return None;
                }
                let imm = d.ea(7, 4, sz, read);
                (format!("{name}{}", sz.suffix()), format!("{imm},{}", d.ea(mode, reg, sz, if rx == 6 { read } else { write })))
            }
        }
        0x1..=0x3 => {
            let sz = match op >> 12 {
                1 => Sz::B,
                2 => Sz::L,
                _ => Sz::W,
            };
            let (dm, dr) = ((op >> 6) & 7, rx);
            if !allowed(if sz == Sz::B { Cat::Data } else { Cat::All }, mode, reg) {
                return None;
            }
            if dm == 1 {
                if sz == Sz::B {
                    return None;
                }
                let src = d.ea(mode, reg, sz, read);
                return Some((format!("MOVEA{}", sz.suffix()), format!("{src},A{dr}")));
            }
            if !allowed(Cat::DataAlterable, dm, dr) {
                return None;
            }
            let src = d.ea(mode, reg, sz, read);
            d.reference = d.reference.filter(|_| !(dm == 7));
            let dst = d.ea(dm, dr, sz, write);
            (format!("MOVE{}", sz.suffix()), format!("{src},{dst}"))
        }
        0x4 => decode4(d, op)?,
        0x5 => {
            if (op >> 6) & 3 == 3 {
                let c = CONDITIONS[((op >> 8) & 15) as usize];
                if mode == 1 {
                    let base = d.next;
                    let t = base.wrapping_add(d.word() as i16 as i32 as u32) & 0xFF_FFFF;
                    let name = if c == "F" { "DBRA".to_string() } else { format!("DB{c}") };
                    return Some((name, format!("D{reg},${t:06X}")));
                }
                if !allowed(Cat::DataAlterable, mode, reg) {
                    return None;
                }
                (format!("S{c}"), d.ea(mode, reg, Sz::B, write))
            } else {
                let sz = Sz::bits(op >> 6)?;
                if !allowed(Cat::Alterable, mode, reg) || (mode == 1 && sz == Sz::B) {
                    return None;
                }
                let n = if rx == 0 { 8 } else { rx };
                let name = if op & 0x100 != 0 { "SUBQ" } else { "ADDQ" };
                (format!("{name}{}", sz.suffix()), format!("#{n},{}", d.ea(mode, reg, sz, write)))
            }
        }
        0x6 => {
            let c = (op >> 8) & 15;
            let base = address.wrapping_add(2);
            let (t, sz) = if op & 0xFF == 0 {
                (base.wrapping_add(d.word() as i16 as i32 as u32), ".W")
            } else {
                (base.wrapping_add(op as u8 as i8 as i32 as u32), ".S")
            };
            let t = t & 0xFF_FFFF;
            let name = match c {
                0 => "BRA".to_string(),
                1 => {
                    d.note(Reference::Call, t);
                    "BSR".to_string()
                }
                _ => format!("B{}", CONDITIONS[c as usize]),
            };
            (format!("{name}{sz}"), format!("${t:06X}"))
        }
        0x7 => {
            if op & 0x100 != 0 {
                return None;
            }
            (m("MOVEQ"), format!("#{},D{rx}", signed(op as u8 as i8 as i32)))
        }
        0x8 | 0xC => {
            let opmode = (op >> 6) & 7;
            let and = op >> 12 == 0xC;
            match opmode {
                3 | 7 => {
                    if !allowed(Cat::Data, mode, reg) {
                        return None;
                    }
                    let name = match (and, opmode) {
                        (false, 3) => "DIVU.W",
                        (false, _) => "DIVS.W",
                        (true, 3) => "MULU.W",
                        _ => "MULS.W",
                    };
                    (m(name), format!("{},D{rx}", d.ea(mode, reg, Sz::W, read)))
                }
                4 if mode <= 1 => {
                    let name = if and { "ABCD" } else { "SBCD" };
                    if mode == 0 { (m(name), format!("D{reg},D{rx}")) } else { (m(name), format!("-(A{reg}),-(A{rx})")) }
                }
                5 if and && mode == 0 => (m("EXG"), format!("D{rx},D{reg}")),
                5 if and && mode == 1 => (m("EXG"), format!("A{rx},A{reg}")),
                6 if and && mode == 1 => (m("EXG"), format!("D{rx},A{reg}")),
                5 | 6 if mode <= 1 => return None,
                _ => {
                    let sz = Sz::bits(opmode)?;
                    let name = if and { "AND" } else { "OR" };
                    if opmode & 4 == 0 {
                        if !allowed(Cat::Data, mode, reg) {
                            return None;
                        }
                        (format!("{name}{}", sz.suffix()), format!("{},D{rx}", d.ea(mode, reg, sz, read)))
                    } else {
                        if !allowed(Cat::MemoryAlterable, mode, reg) {
                            return None;
                        }
                        (format!("{name}{}", sz.suffix()), format!("D{rx},{}", d.ea(mode, reg, sz, write)))
                    }
                }
            }
        }
        0x9 | 0xD => {
            let add = op >> 12 == 0xD;
            let base = if add { "ADD" } else { "SUB" };
            let opmode = (op >> 6) & 7;
            if opmode == 3 || opmode == 7 {
                let sz = if opmode == 7 { Sz::L } else { Sz::W };
                if !allowed(Cat::All, mode, reg) {
                    return None;
                }
                return Some((format!("{base}A{}", sz.suffix()), format!("{},A{rx}", d.ea(mode, reg, sz, read))));
            }
            let sz = Sz::bits(opmode)?;
            if opmode & 4 != 0 && mode <= 1 {
                let name = format!("{base}X{}", sz.suffix());
                return Some(if mode == 0 { (name, format!("D{reg},D{rx}")) } else { (name, format!("-(A{reg}),-(A{rx})")) });
            }
            if opmode & 4 == 0 {
                if !allowed(Cat::All, mode, reg) || (mode == 1 && sz == Sz::B) {
                    return None;
                }
                (format!("{base}{}", sz.suffix()), format!("{},D{rx}", d.ea(mode, reg, sz, read)))
            } else {
                if !allowed(Cat::MemoryAlterable, mode, reg) {
                    return None;
                }
                (format!("{base}{}", sz.suffix()), format!("D{rx},{}", d.ea(mode, reg, sz, write)))
            }
        }
        0xB => {
            let opmode = (op >> 6) & 7;
            if opmode == 3 || opmode == 7 {
                let sz = if opmode == 7 { Sz::L } else { Sz::W };
                if !allowed(Cat::All, mode, reg) {
                    return None;
                }
                return Some((format!("CMPA{}", sz.suffix()), format!("{},A{rx}", d.ea(mode, reg, sz, read))));
            }
            let sz = Sz::bits(opmode)?;
            if opmode & 4 == 0 {
                if !allowed(Cat::All, mode, reg) || (mode == 1 && sz == Sz::B) {
                    return None;
                }
                (format!("CMP{}", sz.suffix()), format!("{},D{rx}", d.ea(mode, reg, sz, read)))
            } else if mode == 1 {
                (format!("CMPM{}", sz.suffix()), format!("(A{reg})+,(A{rx})+"))
            } else {
                if !allowed(Cat::DataAlterable, mode, reg) {
                    return None;
                }
                (format!("EOR{}", sz.suffix()), format!("D{rx},{}", d.ea(mode, reg, sz, write)))
            }
        }
        0xE => {
            const NAMES: [&str; 4] = ["AS", "LS", "ROX", "RO"];
            let dir = if op & 0x100 != 0 { "L" } else { "R" };
            if (op >> 6) & 3 == 3 {
                if op & 0x800 != 0 || !allowed(Cat::MemoryAlterable, mode, reg) {
                    return None;
                }
                let name = NAMES[((op >> 9) & 3) as usize];
                return Some((format!("{name}{dir}.W"), d.ea(mode, reg, Sz::W, write)));
            }
            let sz = Sz::bits(op >> 6)?;
            let name = NAMES[((op >> 3) & 3) as usize];
            let count = if op & 0x20 != 0 { format!("D{rx}") } else { format!("#{}", if rx == 0 { 8 } else { rx }) };
            (format!("{name}{dir}{}", sz.suffix()), format!("{count},D{reg}"))
        }
        _ => return None,
    })
}

fn decode4(d: &mut D, op: u16) -> Option<(String, String)> {
    let mode = (op >> 3) & 7;
    let reg = op & 7;
    let rx = (op >> 9) & 7;
    let m = |s: &str| s.to_string();
    let read = Reference::Read;
    let write = Reference::Write;
    if op & 0x100 != 0 {
        return match (op >> 6) & 3 {
            2 if allowed(Cat::Data, mode, reg) => Some((m("CHK.W"), format!("{},D{rx}", d.ea(mode, reg, Sz::W, read)))),
            3 if allowed(Cat::Control, mode, reg) => Some((m("LEA"), format!("{},A{rx}", d.ea(mode, reg, Sz::L, read)))),
            _ => None,
        };
    }
    let size = (op >> 6) & 3;
    Some(match (op >> 8) & 0xF {
        0x0 | 0x2 | 0x4 | 0x6 if size == 3 => match rx {
            0 if allowed(Cat::DataAlterable, mode, reg) => (m("MOVE"), format!("SR,{}", d.ea(mode, reg, Sz::W, write))),
            2 if allowed(Cat::Data, mode, reg) => (m("MOVE"), format!("{},CCR", d.ea(mode, reg, Sz::W, read))),
            3 if allowed(Cat::Data, mode, reg) => (m("MOVE"), format!("{},SR", d.ea(mode, reg, Sz::W, read))),
            _ => return None,
        },
        0x0 | 0x2 | 0x4 | 0x6 => {
            if !allowed(Cat::DataAlterable, mode, reg) {
                return None;
            }
            let sz = Sz::bits(size)?;
            let name = ["NEGX", "CLR", "NEG", "NOT"][(rx & 3) as usize];
            (format!("{name}{}", sz.suffix()), d.ea(mode, reg, sz, write))
        }
        0x8 => match size {
            0 if allowed(Cat::DataAlterable, mode, reg) => (m("NBCD"), d.ea(mode, reg, Sz::B, write)),
            1 if mode == 0 => (m("SWAP"), format!("D{reg}")),
            1 if allowed(Cat::Control, mode, reg) => (m("PEA"), d.ea(mode, reg, Sz::L, read)),
            2 | 3 if mode == 0 => (if size == 2 { m("EXT.W") } else { m("EXT.L") }, format!("D{reg}")),
            2 | 3 if allowed(Cat::ControlAlterable, mode, reg) || mode == 4 => {
                let mask = d.word();
                let sz = if size == 3 { Sz::L } else { Sz::W };
                (format!("MOVEM{}", sz.suffix()), format!("{},{}", reglist(mask, mode == 4), d.ea(mode, reg, sz, write)))
            }
            _ => return None,
        },
        0xA => match size {
            _ if op == 0x4AFC => (m("ILLEGAL"), String::new()),
            3 if allowed(Cat::DataAlterable, mode, reg) => (m("TAS"), d.ea(mode, reg, Sz::B, write)),
            3 => return None,
            s if allowed(Cat::DataAlterable, mode, reg) => {
                let sz = Sz::bits(s)?;
                (format!("TST{}", sz.suffix()), d.ea(mode, reg, sz, read))
            }
            _ => return None,
        },
        0xC => match size {
            2 | 3 if allowed(Cat::Control, mode, reg) || mode == 3 => {
                let mask = d.word();
                let sz = if size == 3 { Sz::L } else { Sz::W };
                (format!("MOVEM{}", sz.suffix()), format!("{},{}", d.ea(mode, reg, sz, read), reglist(mask, false)))
            }
            _ => return None,
        },
        0xE => match size {
            1 => match mode {
                0 | 1 => (m("TRAP"), format!("#{}", op & 15)),
                2 => (m("LINK"), format!("A{reg},#{}", signed(d.word() as i16 as i32))),
                3 => (m("UNLK"), format!("A{reg}")),
                4 => (m("MOVE"), format!("A{reg},USP")),
                5 => (m("MOVE"), format!("USP,A{reg}")),
                7 => return None,
                _ => match reg {
                    0 => (m("RESET"), String::new()),
                    1 => (m("NOP"), String::new()),
                    2 => (m("STOP"), format!("#{}", hex(d.word() as u32))),
                    3 => (m("RTE"), String::new()),
                    5 => (m("RTS"), String::new()),
                    6 => (m("TRAPV"), String::new()),
                    7 => (m("RTR"), String::new()),
                    _ => return None,
                },
            },
            2 | 3 if allowed(Cat::Control, mode, reg) => {
                let jsr = size == 2;
                let text = d.ea(mode, reg, Sz::L, Reference::Call);
                if !jsr {
                    d.reference = None;
                }
                (if jsr { m("JSR") } else { m("JMP") }, text)
            }
            _ => return None,
        },
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(words: &[u16], at: u32) -> (String, usize, Option<(Reference, u32)>) {
        let w = words.to_vec();
        let i = disassemble(&move |a| w.get((a.wrapping_sub(at) / 2) as usize).copied().unwrap_or(0), at);
        (format!("{} {}", i.mnemonic, i.operands).trim_end().to_owned(), i.bytes.len(), i.reference)
    }

    #[test]
    fn instructions_read_as_motorola_writes_them() {
        assert_eq!(text(&[0x4E71], 0).0, "NOP");
        assert_eq!(text(&[0x22D8], 0).0, "MOVE.L (A0)+,(A1)+");
        assert_eq!(text(&[0x3B7C, 0x1234, 0xFFF0], 0).0, "MOVE.W #$1234,-$10(A5)");
        assert_eq!(text(&[0x41F9, 0x00FF, 0x0000], 0), ("LEA $00FF0000.L,A0".into(), 6, Some((Reference::Read, 0xFF_0000))));
        assert_eq!(text(&[0x4EB9, 0x0000, 0x0500], 0x400), ("JSR $00000500.L".into(), 6, Some((Reference::Call, 0x500))));
        assert_eq!(text(&[0x6100, 0x0010], 0x400), ("BSR.W $000412".into(), 4, Some((Reference::Call, 0x412))));
        assert_eq!(text(&[0x51CF, 0xFFEA], 0x424).0, "DBRA D7,$000410");
        assert_eq!(text(&[0x48E7, 0xC0C0], 0).0, "MOVEM.L D0-D1/A0-A1,-(A7)");
        assert_eq!(text(&[0x4CDF, 0x0303], 0).0, "MOVEM.L (A7)+,D0-D1/A0-A1");
        assert_eq!(text(&[0x33C0, 0x00C0, 0x0000], 0), ("MOVE.W D0,$00C00000.L".into(), 6, Some((Reference::Write, 0xC0_0000))));
        assert_eq!(text(&[0x1039, 0x00A1, 0x0001], 0).2, Some((Reference::Read, 0xA1_0001)));
        assert_eq!(text(&[0x0839, 0x0003, 0x00A1, 0x0003], 0).0, "BTST #3,$00A10003.L");
        assert_eq!(text(&[0x303B, 0x1006], 0).0, "MOVE.W $6(PC,D1.W),D0");
        assert_eq!(text(&[0xE74A], 0).0, "LSL.W #3,D2");
        assert_eq!(text(&[0x46FC, 0x2700], 0).0, "MOVE #$2700,SR");
        assert_eq!(text(&[0x027C, 0xF8FF], 0).0, "ANDI #$F8FF,SR");
        assert_eq!(text(&[0xA123], 0), ("DC.W $A123".into(), 2, None));
    }
}
