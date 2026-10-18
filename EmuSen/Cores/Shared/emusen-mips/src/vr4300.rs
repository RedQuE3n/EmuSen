//! MIPS III as the VR4300 runs it, written in the VR4300's spelling (`Mars_Disassembler.md`): register 30 as `s8`,
//! a register form folded to one operand where the other is the zero register (`beqz`, `move`, `b`), hexadecimal in
//! upper case after `0x`, and `.word` for what the part does not decode. A pure decode of one word, naming what the
//! VR4300 executes and no more, with the address the word names where the word alone gives it (EmuSen_Mips.md §6).

/// What an instruction does with the address it names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reference {
    Call,
    Read,
    Write,
}

/// One decoded word: its mnemonic, its operands as text, and the address it names where the word gives it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instruction {
    pub mnemonic: String,
    pub operands: String,
    pub reference: Option<(Reference, u32)>,
}

/// The general registers as the VR4300's documents name them.
pub const GPR: [&str; 32] = [
    "zero", "at", "v0", "v1", "a0", "a1", "a2", "a3", "t0", "t1", "t2", "t3", "t4", "t5", "t6", "t7", "s0", "s1", "s2", "s3", "s4", "s5",
    "s6", "s7", "t8", "t9", "k0", "k1", "gp", "sp", "s8", "ra",
];

/// Coprocessor 0's registers, an unnamed one as `$n`.
pub const COP0: [&str; 32] = [
    "Index", "Random", "EntryLo0", "EntryLo1", "Context", "PageMask", "Wired", "$7", "BadVAddr", "Count", "EntryHi", "Compare", "Status", "Cause",
    "EPC", "PRId", "Config", "LLAddr", "WatchLo", "WatchHi", "XContext", "$21", "$22", "$23", "$24", "$25", "PErr", "CacheErr", "TagLo", "TagHi",
    "ErrorEPC", "$31",
];

const CONDITIONS: [&str; 16] = ["f", "un", "eq", "ueq", "olt", "ult", "ole", "ule", "sf", "ngle", "seq", "ngl", "lt", "nge", "le", "ngt"];

/// A value as the VR4300's spelling writes it: `0x` and upper-case digits, a negative one with its sign.
pub fn hex(value: i64) -> String {
    if value < 0 { format!("-0x{:X}", value.unsigned_abs()) } else { format!("0x{value:X}") }
}

fn rs(w: u32) -> usize {
    ((w >> 21) & 0x1F) as usize
}
fn rt(w: u32) -> usize {
    ((w >> 16) & 0x1F) as usize
}
fn rd(w: u32) -> usize {
    ((w >> 11) & 0x1F) as usize
}
fn sa(w: u32) -> usize {
    ((w >> 6) & 0x1F) as usize
}
fn gpr(i: usize) -> &'static str {
    GPR[i]
}
fn fpr(i: usize) -> String {
    format!("f{i}")
}
fn simm(w: u32) -> i64 {
    w as u16 as i16 as i64
}

fn plain(mnemonic: &str, operands: impl Into<String>) -> Instruction {
    Instruction { mnemonic: mnemonic.to_string(), operands: operands.into(), reference: None }
}

/// What the part does not decode.
pub fn word(w: u32) -> Instruction {
    plain(".word", format!("0x{w:08X}"))
}

/// A software code field is shown only when something was put in it.
fn code(c: u32) -> String {
    if c == 0 { String::new() } else { hex(c as i64) }
}

fn branch(mnemonic: &str, registers: &str, w: u32, address: u32) -> Instruction {
    let target = address.wrapping_add(4).wrapping_add(((w as u16 as i16 as i32) << 2) as u32);
    Instruction { mnemonic: mnemonic.to_string(), operands: format!("{registers}0x{target:08X}"), reference: Some((Reference::Call, target)) }
}

fn jump(mnemonic: &str, w: u32, address: u32) -> Instruction {
    let target = (address.wrapping_add(4) & 0xF000_0000) | ((w & 0x03FF_FFFF) << 2);
    Instruction { mnemonic: mnemonic.to_string(), operands: format!("0x{target:08X}"), reference: Some((Reference::Call, target)) }
}

fn offset(w: u32) -> String {
    format!("{}({})", hex(simm(w)), gpr(rs(w)))
}

/// Only a base of zero makes the address a constant of the word.
fn access(mnemonic: &str, register: &str, w: u32, kind: Reference) -> Instruction {
    let operands = format!("{register}, {}", offset(w));
    let reference = (rs(w) == 0).then_some((kind, simm(w) as i32 as u32));
    Instruction { mnemonic: mnemonic.to_string(), operands, reference }
}

fn load(m: &str, r: &str, w: u32) -> Instruction {
    access(m, r, w, Reference::Read)
}
fn store(m: &str, r: &str, w: u32) -> Instruction {
    access(m, r, w, Reference::Write)
}

fn signed_immediate(m: &str, w: u32) -> Instruction {
    plain(m, format!("{}, {}, {}", gpr(rt(w)), gpr(rs(w)), hex(simm(w))))
}

fn unsigned_immediate(m: &str, w: u32) -> Instruction {
    plain(m, format!("{}, {}, {}", gpr(rt(w)), gpr(rs(w)), hex((w & 0xFFFF) as i64)))
}

/// One VR4300 word fetched from `address`.
pub fn decode(w: u32, address: u32) -> Instruction {
    let (s, t) = (rs(w), rt(w));
    match w >> 26 {
        0x00 => special(w),
        0x01 => regimm(w, address),
        0x02 => jump("j", w, address),
        0x03 => jump("jal", w, address),
        0x04 if s == 0 && t == 0 => branch("b", "", w, address),
        0x04 if t == 0 => branch("beqz", &format!("{}, ", gpr(s)), w, address),
        0x04 => branch("beq", &format!("{}, {}, ", gpr(s), gpr(t)), w, address),
        0x05 if t == 0 => branch("bnez", &format!("{}, ", gpr(s)), w, address),
        0x05 => branch("bne", &format!("{}, {}, ", gpr(s), gpr(t)), w, address),
        0x06 => branch("blez", &format!("{}, ", gpr(s)), w, address),
        0x07 => branch("bgtz", &format!("{}, ", gpr(s)), w, address),
        0x08 => signed_immediate("addi", w),
        0x09 => signed_immediate("addiu", w),
        0x0A => signed_immediate("slti", w),
        0x0B => signed_immediate("sltiu", w),
        0x0C => unsigned_immediate("andi", w),
        0x0D => unsigned_immediate("ori", w),
        0x0E => unsigned_immediate("xori", w),
        0x0F => plain("lui", format!("{}, {}", gpr(t), hex((w & 0xFFFF) as i64))),
        0x10 => cop0(w, address),
        0x11 => cop1(w, address),
        0x12 => cop2(w),
        0x14 => branch("beql", &format!("{}, {}, ", gpr(s), gpr(t)), w, address),
        0x15 => branch("bnel", &format!("{}, {}, ", gpr(s), gpr(t)), w, address),
        0x16 => branch("blezl", &format!("{}, ", gpr(s)), w, address),
        0x17 => branch("bgtzl", &format!("{}, ", gpr(s)), w, address),
        0x18 => signed_immediate("daddi", w),
        0x19 => signed_immediate("daddiu", w),
        0x1A => load("ldl", gpr(t), w),
        0x1B => load("ldr", gpr(t), w),
        0x20 => load("lb", gpr(t), w),
        0x21 => load("lh", gpr(t), w),
        0x22 => load("lwl", gpr(t), w),
        0x23 => load("lw", gpr(t), w),
        0x24 => load("lbu", gpr(t), w),
        0x25 => load("lhu", gpr(t), w),
        0x26 => load("lwr", gpr(t), w),
        0x27 => load("lwu", gpr(t), w),
        0x28 => store("sb", gpr(t), w),
        0x29 => store("sh", gpr(t), w),
        0x2A => store("swl", gpr(t), w),
        0x2B => store("sw", gpr(t), w),
        0x2C => store("sdl", gpr(t), w),
        0x2D => store("sdr", gpr(t), w),
        0x2E => store("swr", gpr(t), w),
        0x2F => plain("cache", format!("{}, {}", hex(t as i64), offset(w))),
        0x30 => load("ll", gpr(t), w),
        0x31 => load("lwc1", &fpr(t), w),
        0x34 => load("lld", gpr(t), w),
        0x35 => load("ldc1", &fpr(t), w),
        0x37 => load("ld", gpr(t), w),
        0x38 => store("sc", gpr(t), w),
        0x39 => store("swc1", &fpr(t), w),
        0x3C => store("scd", gpr(t), w),
        0x3D => store("sdc1", &fpr(t), w),
        0x3F => store("sd", gpr(t), w),
        _ => word(w),
    }
}

fn special(w: u32) -> Instruction {
    let (s, t, d) = (rs(w), rt(w), rd(w));
    let shift = format!("{}, {}, {}", gpr(d), gpr(t), hex(sa(w) as i64));
    let variable = format!("{}, {}, {}", gpr(d), gpr(t), gpr(s));
    let three = format!("{}, {}, {}", gpr(d), gpr(s), gpr(t));
    let pair = format!("{}, {}", gpr(s), gpr(t));
    let mv = || plain("move", format!("{}, {}", gpr(d), gpr(s)));
    match w & 0x3F {
        0x00 if w == 0 => plain("nop", ""),
        0x00 => plain("sll", shift),
        0x02 => plain("srl", shift),
        0x03 => plain("sra", shift),
        0x04 => plain("sllv", variable),
        0x06 => plain("srlv", variable),
        0x07 => plain("srav", variable),
        0x08 => plain("jr", gpr(s)),
        0x09 => plain("jalr", if d == 31 { gpr(s).to_string() } else { format!("{}, {}", gpr(d), gpr(s)) }),
        0x0C => plain("syscall", code((w >> 6) & 0xFFFFF)),
        0x0D => plain("break", code((w >> 6) & 0xFFFFF)),
        0x0F => plain("sync", ""),
        0x10 => plain("mfhi", gpr(d)),
        0x11 => plain("mthi", gpr(s)),
        0x12 => plain("mflo", gpr(d)),
        0x13 => plain("mtlo", gpr(s)),
        0x14 => plain("dsllv", variable),
        0x16 => plain("dsrlv", variable),
        0x17 => plain("dsrav", variable),
        0x18 => plain("mult", pair),
        0x19 => plain("multu", pair),
        0x1A => plain("div", pair),
        0x1B => plain("divu", pair),
        0x1C => plain("dmult", pair),
        0x1D => plain("dmultu", pair),
        0x1E => plain("ddiv", pair),
        0x1F => plain("ddivu", pair),
        0x20 => plain("add", three),
        0x21 if t == 0 => mv(),
        0x21 => plain("addu", three),
        0x22 if s == 0 => plain("neg", format!("{}, {}", gpr(d), gpr(t))),
        0x22 => plain("sub", three),
        0x23 if s == 0 => plain("negu", format!("{}, {}", gpr(d), gpr(t))),
        0x23 => plain("subu", three),
        0x24 => plain("and", three),
        0x25 if t == 0 => mv(),
        0x25 => plain("or", three),
        0x26 => plain("xor", three),
        0x27 if t == 0 => plain("not", format!("{}, {}", gpr(d), gpr(s))),
        0x27 => plain("nor", three),
        0x2A => plain("slt", three),
        0x2B => plain("sltu", three),
        0x2C => plain("dadd", three),
        0x2D if t == 0 => mv(),
        0x2D => plain("daddu", three),
        0x2E => plain("dsub", three),
        0x2F => plain("dsubu", three),
        0x30 => trap("tge", &pair, w),
        0x31 => trap("tgeu", &pair, w),
        0x32 => trap("tlt", &pair, w),
        0x33 => trap("tltu", &pair, w),
        0x34 => trap("teq", &pair, w),
        0x36 => trap("tne", &pair, w),
        0x38 => plain("dsll", shift),
        0x3A => plain("dsrl", shift),
        0x3B => plain("dsra", shift),
        0x3C => plain("dsll32", shift),
        0x3E => plain("dsrl32", shift),
        0x3F => plain("dsra32", shift),
        _ => word(w),
    }
}

fn trap(m: &str, pair: &str, w: u32) -> Instruction {
    let c = code((w >> 6) & 0x3FF);
    plain(m, if c.is_empty() { pair.to_string() } else { format!("{pair}, {c}") })
}

fn regimm(w: u32, address: u32) -> Instruction {
    let r = gpr(rs(w));
    let trap = format!("{r}, {}", hex(simm(w)));
    let reg = format!("{r}, ");
    match rt(w) {
        0x00 => branch("bltz", &reg, w, address),
        0x01 => branch("bgez", &reg, w, address),
        0x02 => branch("bltzl", &reg, w, address),
        0x03 => branch("bgezl", &reg, w, address),
        0x08 => plain("tgei", trap),
        0x09 => plain("tgeiu", trap),
        0x0A => plain("tlti", trap),
        0x0B => plain("tltiu", trap),
        0x0C => plain("teqi", trap),
        0x0E => plain("tnei", trap),
        0x10 => branch("bltzal", &reg, w, address),
        0x11 if rs(w) == 0 => branch("bal", "", w, address),
        0x11 => branch("bgezal", &reg, w, address),
        0x12 => branch("bltzall", &reg, w, address),
        0x13 => branch("bgezall", &reg, w, address),
        _ => word(w),
    }
}

fn condition_branch(coprocessor: u32, selector: usize) -> String {
    format!("bc{coprocessor}{}{}", if selector & 1 != 0 { 't' } else { 'f' }, if selector & 2 != 0 { "l" } else { "" })
}

/// The CO half ignores every bit between the sub-opcode and the function.
fn cop0(w: u32, address: u32) -> Instruction {
    let (s, t, d) = (rs(w), rt(w), rd(w));
    if s & 0x10 != 0 {
        return match w & 0x3F {
            0x01 => plain("tlbr", ""),
            0x02 => plain("tlbwi", ""),
            0x06 => plain("tlbwr", ""),
            0x08 => plain("tlbp", ""),
            0x18 => plain("eret", ""),
            _ => word(w),
        };
    }
    match s {
        0x00 => plain("mfc0", format!("{}, {}", gpr(t), COP0[d])),
        0x01 => plain("dmfc0", format!("{}, {}", gpr(t), COP0[d])),
        0x04 => plain("mtc0", format!("{}, {}", gpr(t), COP0[d])),
        0x05 => plain("dmtc0", format!("{}, {}", gpr(t), COP0[d])),
        0x02 => plain("cfc0", format!("{}, ${d}", gpr(t))),
        0x06 => plain("ctc0", format!("{}, ${d}", gpr(t))),
        0x08 if t <= 3 => branch(&condition_branch(0, t), "", w, address),
        _ => word(w),
    }
}

fn cop1(w: u32, address: u32) -> Instruction {
    let (s, t, d) = (rs(w), rt(w), rd(w));
    match s {
        0x00 => plain("mfc1", format!("{}, {}", gpr(t), fpr(d))),
        0x01 => plain("dmfc1", format!("{}, {}", gpr(t), fpr(d))),
        0x02 => plain("cfc1", format!("{}, fcr{d}", gpr(t))),
        0x04 => plain("mtc1", format!("{}, {}", gpr(t), fpr(d))),
        0x05 => plain("dmtc1", format!("{}, {}", gpr(t), fpr(d))),
        0x06 => plain("ctc1", format!("{}, fcr{d}", gpr(t))),
        0x08 if t <= 3 => branch(&condition_branch(1, t), "", w, address),
        0x10 => float_format(w, 's'),
        0x11 => float_format(w, 'd'),
        0x14 => integer_format(w, 'w'),
        0x15 => integer_format(w, 'l'),
        _ => word(w),
    }
}

/// Converting a format to itself is refused, so it has no mnemonic.
fn float_format(w: u32, format: char) -> Instruction {
    let (fd, fs, ft) = (fpr(sa(w)), fpr(rd(w)), fpr(rt(w)));
    let funct = w & 0x3F;
    let name: Option<String> = match funct {
        0x00 => Some("add".into()),
        0x01 => Some("sub".into()),
        0x02 => Some("mul".into()),
        0x03 => Some("div".into()),
        0x04 => Some("sqrt".into()),
        0x05 => Some("abs".into()),
        0x06 => Some("mov".into()),
        0x07 => Some("neg".into()),
        0x08 => Some("round.l".into()),
        0x09 => Some("trunc.l".into()),
        0x0A => Some("ceil.l".into()),
        0x0B => Some("floor.l".into()),
        0x0C => Some("round.w".into()),
        0x0D => Some("trunc.w".into()),
        0x0E => Some("ceil.w".into()),
        0x0F => Some("floor.w".into()),
        0x20 if format == 'd' => Some("cvt.s".into()),
        0x21 if format == 's' => Some("cvt.d".into()),
        0x24 => Some("cvt.w".into()),
        0x25 => Some("cvt.l".into()),
        0x30.. => Some(format!("c.{}", CONDITIONS[(funct & 0xF) as usize])),
        _ => None,
    };
    let Some(name) = name else { return word(w) };
    let operands = match funct {
        ..=0x03 => format!("{fd}, {fs}, {ft}"),
        0x30.. => format!("{fs}, {ft}"),
        _ => format!("{fd}, {fs}"),
    };
    plain(&format!("{name}.{format}"), operands)
}

/// The integer formats convert to a float and do nothing else.
fn integer_format(w: u32, format: char) -> Instruction {
    match w & 0x3F {
        0x20 => plain(&format!("cvt.s.{format}"), format!("{}, {}", fpr(sa(w)), fpr(rd(w)))),
        0x21 => plain(&format!("cvt.d.{format}"), format!("{}, {}", fpr(sa(w)), fpr(rd(w)))),
        _ => word(w),
    }
}

/// One latch behind every index, but the index is still what the word names.
fn cop2(w: u32) -> Instruction {
    let operands = format!("{}, ${}", gpr(rt(w)), rd(w));
    match rs(w) {
        0x00 => plain("mfc2", operands),
        0x01 => plain("dmfc2", operands),
        0x02 => plain("cfc2", operands),
        0x04 => plain("mtc2", operands),
        0x05 => plain("dmtc2", operands),
        0x06 => plain("ctc2", operands),
        _ => word(w),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(w: u32, address: u32) -> String {
        let i = decode(w, address);
        if i.operands.is_empty() { i.mnemonic } else { format!("{} {}", i.mnemonic, i.operands) }
    }

    #[test]
    fn the_vr4300_spelling_folds_and_names_as_mars_did() {
        assert_eq!(text(0, 0), "nop");
        assert_eq!(text(0x1000_0001, 0x8000_0000), "b 0x80000008");
        assert_eq!(text(0x1080_0001, 0x8000_0000), "beqz a0, 0x80000008");
        assert_eq!(text(0x0080_1021, 0), "move v0, a0");
        assert_eq!(text(0x27BD_FFF0, 0), "addiu sp, sp, -0x10");
        assert_eq!(text(0x3C08_8000, 0), "lui t0, 0x8000");
        assert_eq!(text(0x8C08_1234, 0), "lw t0, 0x1234(zero)");
        assert_eq!(decode(0x8C08_1234, 0).reference, Some((Reference::Read, 0x1234)));
        assert_eq!(text(0x4008_6000, 0), "mfc0 t0, Status");
        assert_eq!(text(0x4600_1080, 0), "add.s f2, f2, f0");
        assert_eq!(text(0xDFBF_0018, 0), "ld ra, 0x18(sp)");
        assert_eq!(text(0x03C0_0008, 0), "jr s8");
        assert_eq!(text(0x4C00_0000, 0), ".word 0x4C000000");
    }
}
