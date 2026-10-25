//! The EE Core Instruction Set Manual's Operation sections, transcribed as plain functions on bit strings, apart from
//! the interpreter (`src/ee/ops.rs`): the expected record of every S6 program is computed here (Sapphire_Native.md
//! §16.5). Written from the pseudocode's own vocabulary (bit slices, sign_extend, zero_extend) with i128 arithmetic.

/// X(hi..lo), as the pseudocode slices a bit string.
pub fn bits(x: u128, hi: u32, lo: u32) -> u128 {
    (x >> lo) & ((1u128 << (hi - lo + 1)) - 1)
}

/// sign_extend of an n-bit string to 64 bits.
pub fn sign_extend(x: u128, n: u32) -> u64 {
    let v = bits(x, n - 1, 0);
    if bits(v, n - 1, n - 1) == 1 { (v | (!0u128 << n)) as u64 } else { v as u64 }
}

pub fn zero_extend(x: u128, n: u32) -> u64 {
    bits(x, n - 1, 0) as u64
}

/// NotWordValue: bits 63..31 not all equal.
pub fn not_word(x: u64) -> bool {
    let top = bits(x as u128, 63, 31);
    top != 0 && top != (1u128 << 33) - 1
}

fn signed(x: u64) -> i128 {
    x as i64 as i128
}
fn unsigned(x: u64) -> i128 {
    x as i128
}

/// The outcome of one instruction: the register it writes (or none), the exception it raises (Cause.ExcCode), and HI
/// and LO as 128-bit registers afterwards.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub dest: Option<u64>,
    pub exception: Option<u32>,
    pub hi: u128,
    pub lo: u128,
}

pub const OVERFLOW: u32 = 12;
pub const TRAP: u32 = 13;

/// A three-register or immediate integer operation: `a` is GPR[rs], `b` GPR[rt] or sign_extend(immediate) as each page
/// says, `imm` the raw 16 bits, `sa` the shift field.
#[allow(clippy::too_many_arguments)]
pub fn integer(name: &str, a: u64, b: u64, imm: u16, sa: u32, old: u64, hi: u128, lo: u128) -> Outcome {
    let simm = sign_extend(imm as u128, 16);
    let uimm = zero_extend(imm as u128, 16);
    let mut out = Outcome { dest: None, exception: None, hi, lo };
    let word = |t: i128| sign_extend(t as u128, 32);
    let overflow32 = |t: i128| t > i32::MAX as i128 || t < i32::MIN as i128;
    let overflow64 = |t: i128| t > i64::MAX as i128 || t < i64::MIN as i128;
    let w = |x: u64| signed(sign_extend(x as u128, 32));
    out.dest = match name {
        "add" | "addi" => {
            let t = w(a) + w(if name == "add" { b } else { simm });
            if overflow32(t) {
                out.exception = Some(OVERFLOW);
                None
            } else {
                Some(word(t))
            }
        }
        "addu" => Some(word(w(a) + w(b))),
        "addiu" => Some(word(w(a) + w(simm))),
        "sub" => {
            let t = w(a) - w(b);
            if overflow32(t) {
                out.exception = Some(OVERFLOW);
                None
            } else {
                Some(word(t))
            }
        }
        "subu" => Some(word(w(a) - w(b))),
        "dadd" | "daddi" | "dsub" => {
            let t = match name {
                "dadd" => signed(a) + signed(b),
                "daddi" => signed(a) + signed(simm),
                _ => signed(a) - signed(b),
            };
            if overflow64(t) {
                out.exception = Some(OVERFLOW);
                None
            } else {
                Some(t as u64)
            }
        }
        "daddu" => Some((unsigned(a) + unsigned(b)) as u64),
        "daddiu" => Some((unsigned(a) + unsigned(simm)) as u64),
        "dsubu" => Some((unsigned(a) - unsigned(b)) as u64),
        "and" => Some(a & b),
        "or" => Some(a | b),
        "xor" => Some(a ^ b),
        "nor" => Some(!(a | b)),
        "andi" => Some(a & uimm),
        "ori" => Some(a | uimm),
        "xori" => Some(a ^ uimm),
        "lui" => Some(sign_extend((imm as u128) << 16, 32)),
        "slt" => Some((signed(a) < signed(b)) as u64),
        "sltu" => Some((unsigned(a) < unsigned(b)) as u64),
        "slti" => Some((signed(a) < signed(simm)) as u64),
        "sltiu" => Some((unsigned(a) < unsigned(simm)) as u64),
        "movz" => Some(if b == 0 { a } else { old }),
        "movn" => Some(if b != 0 { a } else { old }),
        "sll" => Some(sign_extend(bits(b as u128, 31 - sa, 0) << sa, 32)),
        "srl" => Some(sign_extend(bits(b as u128, 31, sa), 32)),
        "sra" => Some(sign_extend(bits(b as u128, 31, sa), 32 - sa)),
        "sllv" | "srlv" | "srav" => {
            let s = bits(a as u128, 4, 0) as u32;
            return integer(&name[..3], 0, b, 0, s, old, hi, lo);
        }
        "dsll" | "dsll32" => {
            let s = sa + if name.ends_with("32") { 32 } else { 0 };
            Some(((b as u128) << s) as u64)
        }
        "dsrl" | "dsrl32" => {
            let s = sa + if name.ends_with("32") { 32 } else { 0 };
            Some(bits(b as u128, 63, s) as u64)
        }
        "dsra" | "dsra32" => {
            let s = sa + if name.ends_with("32") { 32 } else { 0 };
            Some(sign_extend(bits(b as u128, 63, s), 64 - s))
        }
        "dsllv" | "dsrlv" | "dsrav" => {
            let s = bits(a as u128, 5, 0) as u32;
            let base = &name[..4];
            return if s >= 32 { integer(&format!("{base}32"), 0, b, 0, s - 32, old, hi, lo) } else { integer(base, 0, b, 0, s, old, hi, lo) };
        }
        _ => panic!("not an integer operation: {name}"),
    };
    out
}

/// LO and HI halves: pipeline 0 is bits 63..0, pipeline 1 bits 127..64.
fn with_half(reg: u128, one: bool, v: u64) -> u128 {
    if one { bits(reg, 63, 0) | (v as u128) << 64 } else { bits(reg, 127, 64) << 64 | v as u128 }
}
fn half_of(reg: u128, one: bool) -> u64 {
    if one { bits(reg, 127, 64) as u64 } else { bits(reg, 63, 0) as u64 }
}

/// The multiply, multiply-add and divide forms; GPR[rd] is `old` where the form writes none.
pub fn muldiv(name: &str, a: u64, b: u64, old: u64, hi: u128, lo: u128) -> Outcome {
    let one = name.ends_with('1');
    let base = name.trim_end_matches('1');
    let (sa, sb) = (signed(sign_extend(a as u128, 32)), signed(sign_extend(b as u128, 32)));
    let (ua, ub) = (zero_extend(a as u128, 32) as i128, zero_extend(b as u128, 32) as i128);
    let acc = (bits(half_of(hi, one) as u128, 31, 0) << 32 | bits(half_of(lo, one) as u128, 31, 0)) as i128;
    let mut out = Outcome { dest: Some(old), exception: None, hi, lo };
    let prod = match base {
        "mult" => Some(sa * sb),
        "multu" => Some(ua * ub),
        "madd" => Some(acc + sa * sb),
        "maddu" => Some(acc + ua * ub),
        _ => None,
    };
    if let Some(p) = prod {
        let p = p as u128;
        out.lo = with_half(lo, one, sign_extend(bits(p, 31, 0), 32));
        out.hi = with_half(hi, one, sign_extend(bits(p, 63, 32), 32));
        out.dest = Some(sign_extend(bits(p, 31, 0), 32));
        return out;
    }
    let (q, r) = match base {
        "div" => {
            assert!(sb != 0, "a zero divisor is undefined");
            let q = if sa == i32::MIN as i128 && sb == -1 { sa } else { sa / sb };
            (q, sa - q * sb)
        }
        "divu" => {
            assert!(ub != 0, "a zero divisor is undefined");
            (ua / ub, ua % ub)
        }
        _ => panic!("not a multiply or divide: {name}"),
    };
    out.lo = with_half(lo, one, sign_extend(q as u128, 32));
    out.hi = with_half(hi, one, sign_extend(r as u128, 32));
    out
}

/// A trap's condition (the trap pages' Operation sections), `b` GPR[rt] or sign_extend(immediate).
pub fn trap(name: &str, a: u64, b: u64) -> bool {
    match name {
        "tge" | "tgei" => signed(a) >= signed(b),
        "tgeu" | "tgeiu" => unsigned(a) >= unsigned(b),
        "tlt" | "tlti" => signed(a) < signed(b),
        "tltu" | "tltiu" => unsigned(a) < unsigned(b),
        "teq" | "teqi" => a == b,
        "tne" | "tnei" => a != b,
        _ => panic!("not a trap: {name}"),
    }
}

/// A branch's condition on GPR[rs] (and GPR[rt]) 63..0.
pub fn branch(name: &str, a: u64, b: u64) -> bool {
    match name {
        "beq" | "beql" => a == b,
        "bne" | "bnel" => a != b,
        "blez" | "blezl" => signed(a) <= 0,
        "bgtz" | "bgtzl" => signed(a) > 0,
        "bltz" | "bltzl" | "bltzal" | "bltzall" => signed(a) < 0,
        "bgez" | "bgezl" | "bgezal" | "bgezall" => signed(a) >= 0,
        _ => panic!("not a branch: {name}"),
    }
}

/// A load from the 16 bytes `mem` (an aligned quadword) at byte `k`, into a register holding `old`.
pub fn load(name: &str, mem: &[u8; 16], k: usize, old: u64) -> u64 {
    let le = |at: usize, n: usize| -> u128 { (0..n).fold(0u128, |v, i| v | (mem[at + i] as u128) << (8 * i)) };
    match name {
        "lb" => sign_extend(le(k, 1), 8),
        "lbu" => zero_extend(le(k, 1), 8),
        "lh" => sign_extend(le(k, 2), 16),
        "lhu" => zero_extend(le(k, 2), 16),
        "lw" => sign_extend(le(k, 4), 32),
        "lwu" => zero_extend(le(k, 4), 32),
        "ld" => le(k, 8) as u64,
        "lwl" => {
            let (word, byte) = (k / 4 * 4, k % 4);
            let memword = le(word, 4);
            let temp = bits(memword, 8 * byte as u32 + 7, 0) << (24 - 8 * byte) | if byte == 3 { 0 } else { bits(old as u128, 23 - 8 * byte as u32, 0) };
            sign_extend(temp, 32)
        }
        "lwr" => {
            let (word, byte) = (k / 4 * 4, k % 4);
            let memword = le(word, 4);
            let loaded = bits(memword, 31, 8 * byte as u32);
            let kept = if byte == 0 { 0 } else { bits(old as u128, 31, 32 - 8 * byte as u32) << (32 - 8 * byte) };
            let temp = kept | loaded;
            if byte == 0 { sign_extend(temp, 32) } else { (bits(old as u128, 63, 32) << 32 | temp) as u64 }
        }
        "ldl" => {
            let (dw, byte) = (k / 8 * 8, k % 8);
            let m = le(dw, 8);
            let loaded = bits(m, 8 * byte as u32 + 7, 0) << (56 - 8 * byte);
            let kept = if byte == 7 { 0 } else { bits(old as u128, 55 - 8 * byte as u32, 0) };
            (loaded | kept) as u64
        }
        "ldr" => {
            let (dw, byte) = (k / 8 * 8, k % 8);
            let m = le(dw, 8);
            let loaded = bits(m, 63, 8 * byte as u32);
            let kept = if byte == 0 { 0 } else { bits(old as u128, 63, 64 - 8 * byte as u32) << (64 - 8 * byte) };
            (loaded | kept) as u64
        }
        _ => panic!("not a load: {name}"),
    }
}

/// A store of `reg` into the 16 bytes `mem` at byte `k`.
pub fn store(name: &str, mem: &mut [u8; 16], k: usize, reg: u64) {
    let r = reg.to_le_bytes();
    match name {
        "sb" => mem[k] = r[0],
        "sh" => mem[k..k + 2].copy_from_slice(&r[..2]),
        "sw" => mem[k..k + 4].copy_from_slice(&r[..4]),
        "sd" => mem[k..k + 8].copy_from_slice(&r),
        "swl" => {
            let (word, byte) = (k / 4 * 4, k % 4);
            for i in 0..=byte {
                mem[word + i] = r[3 - byte + i];
            }
        }
        "swr" => {
            let (word, byte) = (k / 4 * 4, k % 4);
            mem[word + byte..word + 4].copy_from_slice(&r[..4 - byte]);
        }
        "sdl" => {
            let (dw, byte) = (k / 8 * 8, k % 8);
            for i in 0..=byte {
                mem[dw + i] = r[7 - byte + i];
            }
        }
        "sdr" => {
            let (dw, byte) = (k / 8 * 8, k % 8);
            mem[dw + byte..dw + 8].copy_from_slice(&r[..8 - byte]);
        }
        _ => panic!("not a store: {name}"),
    }
}

/// The natural alignment a load or store requires, or 1 for the unaligned forms.
pub fn alignment(name: &str) -> usize {
    match name {
        "lh" | "lhu" | "sh" => 2,
        "lw" | "lwu" | "sw" => 4,
        "ld" | "sd" => 8,
        _ => 1,
    }
}
