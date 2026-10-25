//! The PlayStation 2's vector units as instructions: one table of the operations both units share, read in two ways.
//! In micro mode an instruction is a 64-bit pair, an upper word with five flag bits over an operation and a lower word
//! that is an operation or, with the I bit, a constant; in macro mode the EE issues the same operation as a COP2 word
//! to VU0 (Kunimatsu et al., Hot Chips 11, "Instruction opcode to support 2 modes"; VU User's Manual §4.1, §6.1). Its own
//! spelling: the micro names bare, the macro names with a `v`, the field mask after a dot. A pure decode; nothing
//! executes (EmuSen_Mips.md §9).

use crate::asm::{AsmError, err, number, ranged, split_operands, strip_comment};

/// Where an operation runs: the upper or the lower half of a micro instruction, or only as the EE's COP2 word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Upper,
    Lower,
    Macro,
}

/// How an operation's operands are written, each the VU User's Manual's mnemonic line for it (§4.2, §4.3, §6.2). A word
/// whose bits outside its form's fields are not the documented ones is written as its name and its 25 bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    /// `vfD, vfS, vfT`, the broadcast forms with their field in bits 1-0.
    Fmac,
    /// `vfD, vfS, q`
    FmacQ,
    /// `vfD, vfS, i`
    FmacI,
    /// `acc, vfS, vfT`
    Acc,
    /// `acc, vfS, q`
    AccQ,
    /// `acc, vfS, i`
    AccI,
    /// `vfT, vfS`: ABS, the conversions, MOVE, MR32.
    FtFs,
    /// `vfS, vfT`: CLIPw, whose x, y and z of fs are judged against w of ft.
    Clip,
    /// `q, vfSf, vfTf`: DIV and RSQRT, the fields fsf (bits 22-21) and ftf (bits 24-23).
    Fdiv,
    /// `q, vfTf`: SQRT.
    Sqrt,
    /// No operands: NOP, WAITQ.
    Bare,
    /// `viT, vfSf`: MTIR.
    Mtir,
    /// `vfT, viS`: MFIR.
    Mfir,
    /// `viD, viS, viT`: the integer ALU.
    Int3,
    /// `viT, viS, imm`: IADDI, a signed 5-bit immediate in bits 10-6.
    Imm5,
    /// `vfT, (viS++)`: LQI.
    LoadInc,
    /// `vfT, (--viS)`: LQD.
    LoadDec,
    /// `vfS, (viT++)`: SQI.
    StoreInc,
    /// `vfS, (--viT)`: SQD.
    StoreDec,
    /// `viT, (viS)`: ILWR and ISWR.
    IntMem,
    /// `r, vfSf`: RINIT and RXOR.
    RFs,
    /// `vfT, r`: RGET and RNEXT.
    FtR,
    /// `imm15`, the microprogram's address divided by 8 in bits 20-6: CALLMS.
    Callms,
    /// `vi27`, fs fixed at 27: CALLMSR, which starts at CMSAR0.
    Callmsr,
}

/// One operation: its micro-mode name, where it runs, and how its operands are written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Operation {
    pub name: &'static str,
    pub unit: Unit,
    pub form: Form,
}

const fn op(name: &'static str, unit: Unit, form: Form) -> Option<Operation> {
    Some(Operation { name, unit, form })
}

use Form::*;
use Unit::*;

const N: Option<Operation> = None;

/// By bits 5-0 (ps2tek, "COP2 Special1"); 3Ch-3Fh lead to `SPECIAL2`.
pub static SPECIAL1: [Option<Operation>; 64] = [
    op("addx", Upper, Fmac), op("addy", Upper, Fmac), op("addz", Upper, Fmac), op("addw", Upper, Fmac),
    op("subx", Upper, Fmac), op("suby", Upper, Fmac), op("subz", Upper, Fmac), op("subw", Upper, Fmac),
    op("maddx", Upper, Fmac), op("maddy", Upper, Fmac), op("maddz", Upper, Fmac), op("maddw", Upper, Fmac),
    op("msubx", Upper, Fmac), op("msuby", Upper, Fmac), op("msubz", Upper, Fmac), op("msubw", Upper, Fmac),
    op("maxx", Upper, Fmac), op("maxy", Upper, Fmac), op("maxz", Upper, Fmac), op("maxw", Upper, Fmac),
    op("minix", Upper, Fmac), op("miniy", Upper, Fmac), op("miniz", Upper, Fmac), op("miniw", Upper, Fmac),
    op("mulx", Upper, Fmac), op("muly", Upper, Fmac), op("mulz", Upper, Fmac), op("mulw", Upper, Fmac),
    op("mulq", Upper, FmacQ), op("maxi", Upper, FmacI), op("muli", Upper, FmacI), op("minii", Upper, FmacI),
    op("addq", Upper, FmacQ), op("maddq", Upper, FmacQ), op("addi", Upper, FmacI), op("maddi", Upper, FmacI),
    op("subq", Upper, FmacQ), op("msubq", Upper, FmacQ), op("subi", Upper, FmacI), op("msubi", Upper, FmacI),
    op("add", Upper, Fmac), op("madd", Upper, Fmac), op("mul", Upper, Fmac), op("max", Upper, Fmac),
    op("sub", Upper, Fmac), op("msub", Upper, Fmac), op("opmsub", Upper, Fmac), op("mini", Upper, Fmac),
    op("iadd", Lower, Int3), op("isub", Lower, Int3), op("iaddi", Lower, Imm5), N,
    op("iand", Lower, Int3), op("ior", Lower, Int3), N, N,
    op("callms", Macro, Callms), op("callmsr", Macro, Callmsr), N, N,
    N, N, N, N,
];

/// By bits 10-6 times four plus bits 1-0, where bits 5-2 are 1111 (ps2tek, "COP2 Special2").
pub static SPECIAL2: [Option<Operation>; 128] = {
    let mut t = [N; 128];
    let rows: [[Option<Operation>; 8]; 9] = [
        [op("addax", Upper, Acc), op("adday", Upper, Acc), op("addaz", Upper, Acc), op("addaw", Upper, Acc),
         op("subax", Upper, Acc), op("subay", Upper, Acc), op("subaz", Upper, Acc), op("subaw", Upper, Acc)],
        [op("maddax", Upper, Acc), op("madday", Upper, Acc), op("maddaz", Upper, Acc), op("maddaw", Upper, Acc),
         op("msubax", Upper, Acc), op("msubay", Upper, Acc), op("msubaz", Upper, Acc), op("msubaw", Upper, Acc)],
        [op("itof0", Upper, FtFs), op("itof4", Upper, FtFs), op("itof12", Upper, FtFs), op("itof15", Upper, FtFs),
         op("ftoi0", Upper, FtFs), op("ftoi4", Upper, FtFs), op("ftoi12", Upper, FtFs), op("ftoi15", Upper, FtFs)],
        [op("mulax", Upper, Acc), op("mulay", Upper, Acc), op("mulaz", Upper, Acc), op("mulaw", Upper, Acc),
         op("mulaq", Upper, AccQ), op("abs", Upper, FtFs), op("mulai", Upper, AccI), op("clipw", Upper, Clip)],
        [op("addaq", Upper, AccQ), op("maddaq", Upper, AccQ), op("addai", Upper, AccI), op("maddai", Upper, AccI),
         op("subaq", Upper, AccQ), op("msubaq", Upper, AccQ), op("subai", Upper, AccI), op("msubai", Upper, AccI)],
        [op("adda", Upper, Acc), op("madda", Upper, Acc), op("mula", Upper, Acc), N,
         op("suba", Upper, Acc), op("msuba", Upper, Acc), op("opmula", Upper, Acc), op("nop", Upper, Bare)],
        [op("move", Lower, FtFs), op("mr32", Lower, FtFs), N, N,
         op("lqi", Lower, LoadInc), op("sqi", Lower, StoreInc), op("lqd", Lower, LoadDec), op("sqd", Lower, StoreDec)],
        [op("div", Lower, Fdiv), op("sqrt", Lower, Sqrt), op("rsqrt", Lower, Fdiv), op("waitq", Lower, Bare),
         op("mtir", Lower, Mtir), op("mfir", Lower, Mfir), op("ilwr", Lower, IntMem), op("iswr", Lower, IntMem)],
        [op("rnext", Lower, FtR), op("rget", Lower, FtR), op("rinit", Lower, RFs), op("rxor", Lower, RFs), N, N, N, N],
    ];
    let mut r = 0;
    while r < rows.len() {
        let mut c = 0;
        while c < 8 {
            t[r * 8 + c] = rows[r][c];
            c += 1;
        }
        r += 1;
    }
    t
};

/// The 25 bits below an operation's top bits: the field mask, the three register fields and the function.
pub const BITS: u32 = 0x01FF_FFFF;

/// The fields, by the VU User's Manual's §4.1.3 and §6.1.2: dest (or ftf and fsf), ft (it), fs (is), fd (id, imm5).
pub mod field {
    pub const DEST: u32 = 0xF << 21;
    pub const FTF: u32 = 3 << 23;
    pub const FSF: u32 = 3 << 21;
    pub const FT: u32 = 31 << 16;
    pub const FS: u32 = 31 << 11;
    pub const FD: u32 = 31 << 6;
    /// CALLMS's address, bits 20-6.
    pub const IMM15: u32 = 0x7FFF << 6;
}

/// The bits a form's operands occupy; every other bit of the 25 is the operation's own or fixed.
pub const fn operand_mask(form: Form) -> u32 {
    use field::*;
    match form {
        Fmac => DEST | FT | FS | FD,
        Int3 | Imm5 => FT | FS | FD,
        FmacQ | FmacI => DEST | FS | FD,
        Acc | FtFs | Clip | Fdiv | Mfir | LoadInc | LoadDec | StoreInc | StoreDec | IntMem => DEST | FT | FS,
        AccQ | AccI => DEST | FS,
        Sqrt => FTF | FT,
        Bare | Callmsr => 0,
        Mtir => FSF | FT | FS,
        RFs => FSF | FS,
        FtR => DEST | FT,
        Callms => IMM15,
    }
}

/// The bits a form fixes beyond the operation's selector: CALLMSR's fs of 27 (§6.2).
const fn fixed(form: Form) -> u32 {
    match form {
        Callmsr => 27 << 11,
        _ => 0,
    }
}

/// The operation in the low 25 bits of a COP2 word, an upper word or a lower word.
pub fn operation(w: u32) -> Option<&'static Operation> {
    let f = w & 0x3F;
    let slot = if f >> 2 == 0xF { &SPECIAL2[((w >> 6 & 0x1F) * 4 + (w & 3)) as usize] } else { &SPECIAL1[f as usize] };
    slot.as_ref()
}

/// The operation's selecting bits with every field zero.
fn selector(name: &str) -> Option<(u32, &'static Operation)> {
    let one = SPECIAL1.iter().enumerate().take(0x3C).find_map(|(i, o)| o.as_ref().filter(|o| o.name == name).map(|o| (i as u32, o)));
    one.or_else(|| {
        SPECIAL2.iter().enumerate().find_map(|(i, o)| o.as_ref().filter(|o| o.name == name).map(|o| ((i as u32 / 4) << 6 | 0x3C | i as u32 & 3, o)))
    })
}

const DEST: [char; 4] = ['x', 'y', 'z', 'w'];

/// Bit 24 is x and bit 21 is w (VU User's Manual §4.1.3, "dest field").
fn dest(w: u32) -> String {
    let m = w >> 21 & 0xF;
    if m == 0 { String::new() } else { format!(".{}", (0..4).filter(|i| m & 8 >> i != 0).map(|i| DEST[i]).collect::<String>()) }
}

fn reg(prefix: &str, n: u32) -> String {
    format!("{prefix}{n}")
}

/// A register with one field: `vf3w`, the field 00 x to 11 w (§4.1.3, "fsf/ftf field").
fn reg_field(n: u32, f: u32) -> String {
    format!("vf{n}{}", DEST[f as usize])
}

/// Whether `w`'s bits outside the form's operands are the operation's documented ones.
fn canonical(o: &Operation, w: u32) -> bool {
    let sel = selector(o.name).map_or(0, |(s, _)| s);
    w & BITS & !operand_mask(o.form) == sel | fixed(o.form)
}

/// An operation with its operands, the name given a `v` in macro mode.
pub fn text(o: &Operation, w: u32, macro_mode: bool) -> String {
    let name = if macro_mode { format!("v{}", o.name) } else { o.name.to_string() };
    if !canonical(o, w) {
        return format!("{name} 0x{:07x}", w & BITS);
    }
    let (ft, fs, fd) = (w >> 16 & 0x1F, w >> 11 & 0x1F, w >> 6 & 0x1F);
    let (ftf, fsf) = (w >> 23 & 3, w >> 21 & 3);
    let (ops, masked) = match o.form {
        Fmac => (format!("{}, {}, {}", reg("vf", fd), reg("vf", fs), reg("vf", ft)), true),
        FmacQ => (format!("{}, {}, q", reg("vf", fd), reg("vf", fs)), true),
        FmacI => (format!("{}, {}, i", reg("vf", fd), reg("vf", fs)), true),
        Acc => (format!("acc, {}, {}", reg("vf", fs), reg("vf", ft)), true),
        AccQ => (format!("acc, {}, q", reg("vf", fs)), true),
        AccI => (format!("acc, {}, i", reg("vf", fs)), true),
        FtFs => (format!("{}, {}", reg("vf", ft), reg("vf", fs)), true),
        Clip => (format!("{}, {}", reg("vf", fs), reg("vf", ft)), true),
        Fdiv => (format!("q, {}, {}", reg_field(fs, fsf), reg_field(ft, ftf)), false),
        Sqrt => (format!("q, {}", reg_field(ft, ftf)), false),
        Bare => return name,
        Mtir => (format!("{}, {}", reg("vi", ft), reg_field(fs, fsf)), false),
        Mfir => (format!("{}, {}", reg("vf", ft), reg("vi", fs)), true),
        Int3 => (format!("{}, {}, {}", reg("vi", fd), reg("vi", fs), reg("vi", ft)), false),
        Imm5 => (format!("{}, {}, {}", reg("vi", ft), reg("vi", fs), ((fd << 27) as i32) >> 27), false),
        LoadInc => (format!("{}, ({}++)", reg("vf", ft), reg("vi", fs)), true),
        LoadDec => (format!("{}, (--{})", reg("vf", ft), reg("vi", fs)), true),
        StoreInc => (format!("{}, ({}++)", reg("vf", fs), reg("vi", ft)), true),
        StoreDec => (format!("{}, (--{})", reg("vf", fs), reg("vi", ft)), true),
        IntMem => (format!("{}, ({})", reg("vi", ft), reg("vi", fs)), true),
        RFs => (format!("r, {}", reg_field(fs, fsf)), false),
        FtR => (format!("{}, r", reg("vf", ft)), true),
        Callms => (format!("0x{:x}", w >> 6 & 0x7FFF), false),
        Callmsr => ("vi27".to_string(), false),
    };
    format!("{name}{} {ops}", if masked { dest(w) } else { String::new() })
}

/// A register number of a five-bit field, `prefix` then 0 to 31.
fn numbered(s: &str, prefix: &str) -> Result<u32, AsmError> {
    match s.strip_prefix(prefix).map(str::parse::<u32>) {
        Some(Ok(n)) if n < 32 => Ok(n),
        _ => err(format!("not a {prefix} register: {s}")),
    }
}

fn vf(s: &str) -> Result<u32, AsmError> {
    numbered(s, "vf")
}

fn vi(s: &str) -> Result<u32, AsmError> {
    numbered(s, "vi")
}

/// `vf3w` as (register, field).
fn vf_field(s: &str) -> Result<(u32, u32), AsmError> {
    let Some(c) = s.chars().last() else { return err("an empty operand") };
    let Some(f) = DEST.iter().position(|d| *d == c) else { return err(format!("not a register with a field: {s}")) };
    Ok((vf(&s[..s.len() - 1])?, f as u32))
}

/// `(viS++)`, `(--viS)` or `(viS)`: the register and the form written.
fn address(s: &str, form: Form) -> Result<u32, AsmError> {
    let inner = s.strip_prefix('(').and_then(|t| t.strip_suffix(')')).ok_or_else(|| AsmError { line: 0, message: format!("not an address: {s}") })?;
    let r = match form {
        LoadInc | StoreInc => inner.strip_suffix("++"),
        LoadDec | StoreDec => inner.strip_prefix("--"),
        _ => Some(inner),
    };
    vi(r.ok_or_else(|| AsmError { line: 0, message: format!("not this form's address: {s}") })?)
}

fn word_of(s: &str, want: &str) -> Result<(), AsmError> {
    if s == want { Ok(()) } else { err(format!("expected {want}: {s}")) }
}

/// An operation as `text` writes it, to its 25 bits; `macro_mode` asks for the `v` names.
pub fn encode(text: &str, macro_mode: bool) -> Result<(u32, &'static Operation), AsmError> {
    let text = text.trim();
    let (mnemonic, rest) = text.split_once(char::is_whitespace).unwrap_or((text, ""));
    let mnemonic = mnemonic.to_ascii_lowercase();
    let (head, mask) = mnemonic.split_once('.').unwrap_or((&mnemonic, ""));
    let name = if macro_mode {
        head.strip_prefix('v').ok_or_else(|| AsmError { line: 0, message: format!("unknown mnemonic: {mnemonic}") })?
    } else {
        head
    };
    let Some((selector, o)) = selector(name) else { return err(format!("unknown mnemonic: {mnemonic}")) };
    let operands = split_operands(rest);
    let whole = matches!(operands.as_slice(), [v] if number(v).is_some_and(|n| o.form != Callms || n > 0x7FFF));
    if whole {
        if !mask.is_empty() {
            return err(format!("{head} takes no field mask"));
        }
        let bits = ranged(operands[0], 0, BITS as i64)?;
        return if operation(bits) != Some(o) { err(format!("0x{bits:07x} is not {head}")) } else { Ok((bits, o)) };
    }
    let mut m = 0;
    let mut last = None;
    for c in mask.chars() {
        let Some(i) = DEST.iter().position(|d| *d == c) else { return err(format!("not a field mask: .{mask}")) };
        if last.is_some_and(|l| i <= l) {
            return err(format!("field mask out of order: .{mask}"));
        }
        last = Some(i);
        m |= 8 >> i;
    }
    let masked = matches!(o.form, Fmac | FmacQ | FmacI | Acc | AccQ | AccI | FtFs | Clip | Mfir | LoadInc | LoadDec | StoreInc | StoreDec | IntMem | FtR);
    if m != 0 && !masked {
        return err(format!("{head} takes no field mask"));
    }
    let want = match o.form {
        Bare => 0,
        Callms | Callmsr => 1,
        Fmac | FmacQ | FmacI | Acc | AccQ | AccI | Fdiv | Int3 | Imm5 => 3,
        _ => 2,
    };
    if operands.len() != want {
        return err(format!("{head} takes {want} operands"));
    }
    let o3 = |i: usize| operands[i];
    let fields = match o.form {
        Fmac => vf(o3(0))? << 6 | vf(o3(1))? << 11 | vf(o3(2))? << 16,
        FmacQ => vf(o3(0))? << 6 | vf(o3(1))? << 11 | word_of(o3(2), "q").map(|_| 0)?,
        FmacI => vf(o3(0))? << 6 | vf(o3(1))? << 11 | word_of(o3(2), "i").map(|_| 0)?,
        Acc => word_of(o3(0), "acc").map(|_| 0)? | vf(o3(1))? << 11 | vf(o3(2))? << 16,
        AccQ => word_of(o3(0), "acc").map(|_| 0)? | vf(o3(1))? << 11 | word_of(o3(2), "q").map(|_| 0)?,
        AccI => word_of(o3(0), "acc").map(|_| 0)? | vf(o3(1))? << 11 | word_of(o3(2), "i").map(|_| 0)?,
        FtFs => vf(o3(0))? << 16 | vf(o3(1))? << 11,
        Clip => vf(o3(0))? << 11 | vf(o3(1))? << 16,
        Fdiv => {
            word_of(o3(0), "q")?;
            let ((s, sf), (t, tf)) = (vf_field(o3(1))?, vf_field(o3(2))?);
            s << 11 | sf << 21 | t << 16 | tf << 23
        }
        Sqrt => {
            word_of(o3(0), "q")?;
            let (t, tf) = vf_field(o3(1))?;
            t << 16 | tf << 23
        }
        Bare => 0,
        Mtir => {
            let (s, sf) = vf_field(o3(1))?;
            vi(o3(0))? << 16 | s << 11 | sf << 21
        }
        Mfir => vf(o3(0))? << 16 | vi(o3(1))? << 11,
        Int3 => vi(o3(0))? << 6 | vi(o3(1))? << 11 | vi(o3(2))? << 16,
        Imm5 => vi(o3(0))? << 16 | vi(o3(1))? << 11 | (ranged(o3(2), -16, 15)? & 31) << 6,
        LoadInc | LoadDec => vf(o3(0))? << 16 | address(o3(1), o.form)? << 11,
        StoreInc | StoreDec => vf(o3(0))? << 11 | address(o3(1), o.form)? << 16,
        IntMem => vi(o3(0))? << 16 | address(o3(1), IntMem)? << 11,
        RFs => {
            word_of(o3(0), "r")?;
            let (s, sf) = vf_field(o3(1))?;
            s << 11 | sf << 21
        }
        FtR => vf(o3(0))? << 16 | word_of(o3(1), "r").map(|_| 0)?,
        Callms => ranged(o3(0), 0, 0x7FFF)? << 6,
        Callmsr => word_of(o3(0), "vi27").map(|_| fixed(Callmsr))?,
    };
    Ok((selector | m << 21 | fields, o))
}

/// The upper word's flags, bits 31-27 (ps2tek, "Upper Instructions").
pub const FLAGS: [(char, u32); 5] = [('i', 31), ('e', 30), ('m', 29), ('d', 28), ('t', 27)];

/// The I bit: the lower word is a constant for the I register, not an instruction.
pub const I_BIT: u32 = 1 << 31;

/// The lower word's class for an operation of the shared table, bits 31-25 (the Hot Chips slides' "Lower OP.").
pub const LOWER_OP: u32 = 0x40 << 25;

fn upper_text(w: u32) -> String {
    match operation(w).filter(|o| o.unit == Upper && w >> 25 & 3 == 0) {
        None => format!(".word 0x{w:08x}"),
        Some(o) => {
            let flags: String = FLAGS.iter().filter(|(_, b)| w >> b & 1 != 0).map(|(c, _)| *c).collect();
            let body = text(o, w, false);
            if flags.is_empty() { body } else { format!("[{flags}] {body}") }
        }
    }
}

fn lower_text(w: u32, constant: bool) -> String {
    if constant {
        return format!("0x{w:08x}");
    }
    match operation(w).filter(|o| o.unit == Lower && w & !BITS == LOWER_OP) {
        None => format!(".word 0x{w:08x}"),
        Some(o) => text(o, w, false),
    }
}

/// A micro instruction as text, `upper | lower`: the upper word is bits 63-32 (ps2tek, "VU Instruction Format").
pub fn disassemble(pair: u64) -> String {
    let (upper, lower) = ((pair >> 32) as u32, pair as u32);
    format!("{} | {}", upper_text(upper), lower_text(lower, upper & I_BIT != 0))
}

fn raw_word(s: &str) -> Option<Result<u32, AsmError>> {
    s.strip_prefix(".word").map(|v| ranged(v.trim(), 0, u32::MAX as i64))
}

/// One micro instruction as `disassemble` writes it.
pub fn assemble_pair(text: &str) -> Result<u64, AsmError> {
    let Some((up, low)) = text.split_once('|') else { return err("a micro instruction is upper | lower") };
    let (up, low) = (up.trim(), low.trim());
    let upper = match raw_word(up) {
        Some(w) => w?,
        None => {
            let (flags, body) = match up.strip_prefix('[') {
                Some(rest) => rest.split_once(']').map(|(f, b)| (f, b.trim())).ok_or_else(|| AsmError { line: 0, message: format!("unclosed flags: {up}") })?,
                None => ("", up),
            };
            let mut bits = 0;
            for c in flags.chars() {
                let Some((_, b)) = FLAGS.iter().find(|(f, _)| *f == c) else { return err(format!("not a flag: {c}")) };
                bits |= 1 << b;
            }
            let (w, o) = encode(body, false)?;
            if o.unit != Upper {
                return err(format!("{} is not an upper instruction", o.name));
            }
            bits | w
        }
    };
    let lower = if upper & I_BIT != 0 {
        match number(low) {
            Some(v) if (0..=u32::MAX as i64).contains(&v) => v as u32,
            _ => return err(format!("with the I bit the lower word is a constant: {low}")),
        }
    } else {
        match raw_word(low) {
            Some(w) => w?,
            None => {
                let (w, o) = encode(low, false)?;
                if o.unit != Lower {
                    return err(format!("{} is not a lower instruction", o.name));
                }
                LOWER_OP | w
            }
        }
    };
    Ok((upper as u64) << 32 | lower as u64)
}

/// A microprogram, one pair to a line, `#` and `;` beginning comments.
pub fn assemble(source: &str) -> Result<Vec<u64>, AsmError> {
    source
        .lines()
        .enumerate()
        .map(|(n, l)| (n + 1, strip_comment(l)))
        .filter(|(_, l)| !l.is_empty())
        .map(|(n, l)| assemble_pair(l).map_err(|e| AsmError { line: n, ..e }))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode::tests::mix;

    /// NOP in the upper half: Special2's row 0101, column 7.
    const NOP: u32 = 11 << 6 | 0x3C | 3;
    /// MOVE with every field zero, `move vf0, vf0`, the manual's lower no-operation (§3.2.2).
    const MOVE: u32 = 0x33C;

    /// ps2tek's "COP2 Special1" table as printed, `---` for an empty cell; the V of each name is the macro form's.
    const PS2TEK_SPECIAL1: [&str; 64] = [
        "VADDx", "VADDy", "VADDz", "VADDw", "VSUBx", "VSUBy", "VSUBz", "VSUBw", //
        "VMADDx", "VMADDy", "VMADDz", "VMADDw", "VMSUBx", "VMSUBy", "VMSUBz", "VMSUBw", //
        "VMAXx", "VMAXy", "VMAXz", "VMAXw", "VMINIx", "VMINIy", "VMINIz", "VMINIw", //
        "VMULx", "VMULy", "VMULz", "VMULw", "VMULq", "VMAXi", "VMULi", "VMINIi", //
        "VADDq", "VMADDq", "VADDi", "VMADDi", "VSUBq", "VMSUBq", "VSUbi", "VMSUBi", //
        "VADD", "VMADD", "VMUL", "VMAX", "VSUB", "VMSUB", "VOPMSUB", "VMINI", //
        "VIADD", "VISUB", "VIADDI", "---", "VIAND", "VIOR", "---", "---", //
        "VCALLMS", "CALLMSR", "---", "---", "*1", "*1", "*1", "*1",
    ];

    /// ps2tek's "COP2 Special2" table's nine filled rows; rows 1001-1111 are empty.
    const PS2TEK_SPECIAL2: [&str; 72] = [
        "VADDAx", "VADDAy", "VADDAz", "VADDAw", "VSUBAx", "VSUBAy", "VSUBAz", "VSUBAw", //
        "VMADDAx", "VMADDAy", "VMADDAz", "VMADDAw", "VMSUBAx", "VMSUBAy", "VMSUBAz", "VMSUBAw", //
        "VITOF0", "VITOF4", "VITOF12", "VITOF15", "VFTOI0", "VFTOI4", "VFTOI12", "VFTOI15", //
        "VMULAx", "VMULAy", "VMULAz", "VMULAw", "VMULAq", "VABS", "VMULAi", "VCLIPw", //
        "VADDAq", "VMADDAq", "VADDAi", "VMADDAi", "VSUBAq", "VMSUBAq", "VSUBAi", "VMSUBAi", //
        "VADDA", "VMADDA", "VMULA", "---", "VSUBA", "VMSUBA", "VOPMULA", "VNOP", //
        "VMOVE", "VMR32", "---", "---", "VLQI", "VSQI", "VLQD", "VSQD", //
        "VDIV", "VSQRT", "VRSQRT", "VWAITQ", "VMTIR", "VMFIR", "VILWR", "VISWR", //
        "VRNEXT", "VRGET", "VRINIT", "VRXOR", "---", "---", "---", "---",
    ];

    /// Two of the table's cells carry printing slips: "VSUbi" for VSUBi and "CALLMSR" without its V.
    fn name_of(cell: &str) -> String {
        let c = cell.to_ascii_lowercase();
        c.strip_prefix('v').map_or(c.clone(), str::to_string)
    }

    fn cells() -> Vec<(u32, &'static str)> {
        let mut v: Vec<(u32, &str)> = PS2TEK_SPECIAL1.iter().enumerate().filter(|(i, _)| *i < 0x3C).map(|(i, c)| (i as u32, *c)).collect();
        for (i, c) in PS2TEK_SPECIAL2.iter().enumerate() {
            v.push(((i as u32 / 4) << 6 | 0x3C | (i as u32 % 4), c));
        }
        for i in 72..128u32 {
            v.push(((i / 4) << 6 | 0x3C | (i % 4), "---"));
        }
        v
    }

    fn random_fields(o: &Operation, seed: &mut u64) -> u32 {
        mix(seed) & operand_mask(o.form)
    }

    /// Every cell of ps2tek's two tables: a named one decodes to its name in macro mode, its text reads back to the same
    /// word, and so it does in micro mode in the half the operation runs in; an empty one decodes to nothing.
    #[test]
    fn every_encoding_of_the_documented_tables_reads_back_in_both_modes() {
        let mut seed = 0x5A99_D00E_0000_0001;
        let mut named = 0;
        for (bits, cell) in cells() {
            if cell == "---" {
                assert_eq!(operation(bits), None, "{bits:07x}");
                continue;
            }
            let o = operation(bits).unwrap_or_else(|| panic!("{cell} at {bits:07x} decodes to nothing"));
            assert_eq!(o.name, name_of(cell), "{bits:07x}");
            named += 1;
            for _ in 0..64 {
                let w = bits | random_fields(o, &mut seed);
                let t = text(o, w, true);
                let (back, _) = encode(&t, true).unwrap_or_else(|e| panic!("\"{t}\": {e}"));
                assert_eq!(back, w, "\"{t}\"");
                let pair = match o.unit {
                    Upper => ((mix(&mut seed) & 0xF8 << 24 & !I_BIT | w) as u64) << 32 | (LOWER_OP | MOVE) as u64,
                    Lower => (NOP as u64) << 32 | (LOWER_OP | w) as u64,
                    Macro => continue,
                };
                let t = disassemble(pair);
                assert_eq!(assemble_pair(&t), Ok(pair), "\"{t}\"");
            }
        }
        assert_eq!(named, 55 + 65, "the named cells");
    }

    #[test]
    fn the_hot_chips_examples_and_the_flags() {
        let maddx = 0x08 | 1 << 6 | 2 << 11 | 3 << 16 | 0xF << 21;
        assert_eq!(text(operation(maddx).unwrap(), maddx, true), "vmaddx.xyzw vf1, vf2, vf3");
        let lqi = 0b01101 << 6 | 0x3C | 5 << 11 | 4 << 16 | 0x8 << 21;
        assert_eq!(text(operation(lqi).unwrap(), lqi, false), "lqi.x vf4, (vi5++)");
        assert_eq!(disassemble(0x0000_02FF_8104_2B7C), "nop | lqi.x vf4, (vi5++)", "the slides' bits: 1000000, dest, ft, is, 01101 1111 00");
        assert_eq!(assemble_pair("[e] nop | move vf0, vf0"), Ok(0x4000_02FF_8000_033C));
        let pair = ((1u64 << 30 | NOP as u64) << 32) | (LOWER_OP | MOVE) as u64;
        assert_eq!(disassemble(pair), "[e] nop | move vf0, vf0");
        let constant = ((I_BIT | 0x22) as u64) << 32 | 0x3F80_0000;
        assert_eq!(disassemble(constant), "[i] addi vf0, vf0, i | 0x3f800000");
        assert_eq!(assemble_pair("[i] addi vf0, vf0, i | 0x3f800000"), Ok(constant));
        assert_eq!(disassemble(0x0000_0030_0000_0000), ".word 0x00000030 | .word 0x00000000", "an integer add is no upper instruction");
        assert!(assemble_pair("iadd | nop").is_err());
        assert!(assemble_pair("add.yx vf1, vf2, vf3 | move vf0, vf0").is_err());
        assert_eq!(assemble("# a program\n[e] nop | move vf0, vf0\n\n nop | move vf0, vf0 ; again\n").map(|v| v.len()), Ok(2));
    }

    /// The VU User's Manual's own words for the macro instructions (§6.2's operation codes, its mnemonic lines and
    /// examples): each word's bits as the page draws them, and its text as the page writes it.
    #[test]
    #[allow(clippy::unusual_byte_groupings)] // the opcodes grouped as the pages draw them, 5, 4 and 2 bits
    fn the_manual_s_macro_pages_read_back() {
        let cop2 = |w: u32| 0x12 << 26 | 1 << 25 | w;
        let cases: [(u32, &str); 16] = [
            (0xE << 21 | 10 << 16 | 10 << 11 | 0b00111_1111_11, "vclipw.xyz vf10, vf10"),
            (1 << 23 | 20 << 16 | 10 << 11 | 0b01110_1111_00, "vdiv q, vf10x, vf20y"),
            (10 << 16 | 0b01110_1111_01, "vsqrt q, vf10x"),
            (3 << 23 | 2 << 21 | 4 << 16 | 3 << 11 | 0b01110_1111_10, "vrsqrt q, vf3z, vf4w"),
            (12 << 16 | 10 << 11 | 1 << 21 | 0b01111_1111_00, "vmtir vi12, vf10y"),
            (4 << 21 | 10 << 16 | 12 << 11 | 0b01111_1111_01, "vmfir.y vf10, vi12"),
            (4 << 16 | 5 << 11 | 31 << 6 | 0b110010, "viaddi vi4, vi5, -1"),
            (6 << 16 | 5 << 11 | 4 << 6 | 0b110000, "viadd vi4, vi5, vi6"),
            (0xF << 21 | 10 << 16 | 4 << 11 | 0b01101_1111_00, "vlqi.xyzw vf10, (vi4++)"),
            (0xF << 21 | 4 << 16 | 10 << 11 | 0b01101_1111_11, "vsqd.xyzw vf10, (--vi4)"),
            (4 << 21 | 10 << 16 | 4 << 11 | 0b01111_1111_10, "vilwr.y vi10, (vi4)"),
            (10 << 11 | 0b10000_1111_10, "vrinit r, vf10x"),
            (8 << 21 | 10 << 16 | 0b10000_1111_01, "vrget.x vf10, r"),
            (0x40 << 6 | 0b111000, "vcallms 0x40"),
            (27 << 11 | 0b111001, "vcallmsr vi27"),
            (0b01011_1111_11, "vnop"),
        ];
        for (bits, want) in cases {
            let o = operation(bits).unwrap_or_else(|| panic!("{bits:07x}"));
            assert_eq!(text(o, cop2(bits), true), want);
            assert_eq!(encode(want, true).map(|(b, _)| b), Ok(bits), "{want}");
        }
        let odd = 1 << 16 | 0b01011_1111_11;
        assert_eq!(text(operation(odd).unwrap(), odd, true), "vnop 0x00102ff", "a field the page fixes at zero, set");
        assert_eq!(encode("vnop 0x00102ff", true).map(|(b, _)| b), Ok(odd));
    }
}
