//! Chapter 5 of the EE Core Instruction Set Manual, the COP1 pages, transcribed as plain functions on bit strings
//! apart from the interpreter (`src/ee/fpu.rs`) and the format's module (`src/float.rs`): the expected record of every
//! S9 program is computed here (Sapphire_Native.md §19.4). Numbers are taken as the EE Core User's Manual §8.1.1 writes
//! them, (−1)^s · 2^(e−127) · 1.f for 0 < e ≤ 255 and zero for e = 0, and every result is built from exact integer
//! arithmetic on those values; the readings where the pages stop are the console's, as Sapphire_Disputes.md D-24 to
//! D-33 record, each stated where it is used.

/// FCR31 as a case leaves it, the case having cleared every writable bit first.
pub const FIXED: u32 = 0x0100_0001;
pub const C: u32 = 1 << 23;
pub const I: u32 = 1 << 17;
pub const D: u32 = 1 << 16;
pub const O: u32 = 1 << 15;
pub const U: u32 = 1 << 14;
pub const SI: u32 = 1 << 6;
pub const SD: u32 = 1 << 5;
pub const SO: u32 = 1 << 4;
pub const SU: u32 = 1 << 3;
pub const FMAX: u32 = 0x7FFF_FFFF;

/// A number's sign, biased exponent and integer significand (the hidden bit included); `None` for a zero.
fn number(x: u32) -> Option<(u32, i32, u128)> {
    let e = (x >> 23 & 255) as i32;
    if e == 0 { None } else { Some((x >> 31, e, (x & 0x7F_FFFF | 0x80_0000) as u128)) }
}

/// The value s · v · 2^(k − 150) with v > 0 truncated to 24 significant bits: (bits, overflow, underflow).
fn truncate(s: u32, v: u128, k: i32) -> (u32, bool, bool) {
    let n = 128 - v.leading_zeros() as i32;
    let e = k + n - 24;
    let m = if n > 24 { v >> (n - 24) } else { v << (24 - n) };
    pack(s, e, m)
}
/// The same value rounded to the nearest 24-bit significand, `rest` saying whether anything below `v` is non-zero.
fn round(s: u32, v: u128, k: i32, rest: bool) -> (u32, bool, bool) {
    let n = 128 - v.leading_zeros() as i32;
    if n <= 24 {
        return truncate(s, v, k);
    }
    let cut = n - 24;
    let mut m = v >> cut;
    let below = v & ((1 << cut) - 1);
    let half = 1u128 << (cut - 1);
    let mut e = k + cut;
    if below > half || (below == half && (rest || m & 1 == 1)) {
        m += 1;
        if m == 1 << 24 {
            m >>= 1;
            e += 1;
        }
    }
    pack(s, e, m)
}
fn pack(s: u32, e: i32, m: u128) -> (u32, bool, bool) {
    if e > 255 {
        (s << 31 | FMAX, true, false)
    } else if e < 1 {
        (s << 31, false, true)
    } else {
        (s << 31 | (e as u32) << 23 | (m as u32 & 0x7F_FFFF), false, false)
    }
}

/// ADD.S: the smaller operand aligned to the larger's 24 bits with what falls below lost, then the sum truncated
/// (VU User's Manual §2.2, "a 24-bit calculation including hidden bits is performed, and the result is truncated"; the
/// console's `add CF_MIN, CF_ONE`, D-25). A zero operand leaves the other as it is; zeros sum by Table 8-5.
pub fn add(a: u32, b: u32) -> (u32, bool, bool) {
    let (x, y) = match (number(a), number(b)) {
        (None, None) => return ((a & b) & 0x8000_0000, false, false),
        (Some(_), None) => return (a, false, false),
        (None, Some(_)) => return (b, false, false),
        (Some(x), Some(y)) => (x, y),
    };
    let (big, small) = if (x.1, x.2) >= (y.1, y.2) { (x, y) } else { (y, x) };
    let lost = (big.1 - small.1) as u32;
    let kept = if lost >= 64 { 0 } else { small.2 >> lost };
    let total = if big.0 == small.0 { big.2 as i128 + kept as i128 } else { big.2 as i128 - kept as i128 };
    if total == 0 {
        return (0, false, false);
    }
    truncate(big.0, total.unsigned_abs(), big.1)
}
pub fn sub(a: u32, b: u32) -> (u32, bool, bool) {
    add(a, b ^ 0x8000_0000)
}

/// MUL.S: the exact product less what the multiplier's lowest column would have held (D-26): with FPR[ft]'s lowest
/// radix-4 Booth digit d (from its bits 1 and 0 and an implied 0), +x contributes bit 0 of x, +2x nothing, and a
/// negative multiple its ones' complement's bit 0 and the carry-in 1. Then truncated.
pub fn mul(a: u32, b: u32) -> (u32, bool, bool) {
    let s = (a ^ b) >> 31;
    let (Some(x), Some(y)) = (number(a), number(b)) else { return (s << 31, false, false) };
    let digit: i32 = match y.2 & 3 {
        0 => 0,
        1 => 1,
        2 => -2,
        _ => -1,
    };
    let multiple = digit.unsigned_abs() as u128 * x.2;
    let lost = if digit >= 0 { multiple & 1 } else { (1 - (multiple & 1)) + 1 };
    truncate(s, x.2 * y.2 - lost, x.1 + y.1 - 150)
}

/// DIV.S's value: the exact quotient rounded to the nearest (D-27), the largest magnitude for a zero divisor.
pub fn div(a: u32, b: u32) -> (u32, bool, bool) {
    let s = (a ^ b) >> 31;
    match (number(a), number(b)) {
        (_, None) => (s << 31 | FMAX, false, false),
        (None, _) => (s << 31, false, false),
        (Some(x), Some(y)) => {
            let n = x.2 << 64;
            round(s, n / y.2, x.1 - y.1 + 86, n % y.2 != 0)
        }
    }
}

/// SQRT.S's value: the root of the magnitude rounded to the nearest (D-27); the root of either zero is +0 (D-29).
pub fn sqrt(b: u32) -> u32 {
    let Some((_, e, m)) = number(b) else { return 0 };
    // value = m · 2^(e − 150); with an even exponent the root is sqrt(m · 2^100) · 2^((e − 150 − 100) / 2).
    let (m, t) = if (e - 150) % 2 == 0 { (m, e - 150) } else { (m << 1, e - 151) };
    let x = m << 100;
    let mut r = (x as f64).sqrt() as u128;
    while r * r > x {
        r -= 1;
    }
    while (r + 1) * (r + 1) <= x {
        r += 1;
    }
    round(0, r, (t - 100) / 2 + 150, r * r != x).0
}

/// RSQRT.S's value: FPR[fs] over SQRT.S's result for FPR[ft], truncated, signed as FPR[fs] (D-28).
pub fn rsqrt(a: u32, b: u32) -> u32 {
    let s = a >> 31;
    if number(b).is_none() {
        return s << 31 | FMAX;
    }
    let Some(x) = number(a) else { return s << 31 };
    let y = number(sqrt(b)).expect("a root of a non-zero is non-zero");
    truncate(s, (x.2 << 64) / y.2, x.1 - y.1 + 86).0
}

/// What one instruction leaves: the register it writes (an FPR, ACC or nothing) and FCR31.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub value: u32,
    pub fcr31: u32,
}

fn ou(o: bool, u: bool) -> u32 {
    (if o { O | SO } else { 0 }) | if u { U | SU } else { 0 }
}

/// The arithmetic pages from a cleared FCR31: ADD.S, SUB.S, MUL.S and their accumulator forms write O, U, SO and SU
/// from the exponent; DIV.S, SQRT.S and RSQRT.S write I and D and their sticky flags alone (their Operation sections,
/// D-32).
pub fn arith(name: &str, a: u32, b: u32) -> Outcome {
    let with = |(v, o, u): (u32, bool, bool)| Outcome { value: v, fcr31: FIXED | ou(o, u) };
    let zero = |x: u32| number(x).is_none();
    let negative = |x: u32| x >> 31 == 1 && !zero(x);
    match name {
        "add.s" | "adda.s" => with(add(a, b)),
        "sub.s" | "suba.s" => with(sub(a, b)),
        "mul.s" | "mula.s" => with(mul(a, b)),
        "div.s" => {
            let f = match (zero(a), zero(b)) {
                (true, true) => I | SI,
                (false, true) => D | SD,
                _ => 0,
            };
            Outcome { value: div(a, b).0, fcr31: FIXED | f }
        }
        "sqrt.s" => Outcome { value: sqrt(b), fcr31: FIXED | if negative(b) { I | SI } else { 0 } },
        "rsqrt.s" => {
            let f = if zero(b) { D | SD } else if negative(b) { I | SI } else { 0 };
            Outcome { value: rsqrt(a, b), fcr31: FIXED | f }
        }
        _ => panic!("{name}"),
    }
}

/// MADD.S, MSUB.S, MADDA.S and MSUBA.S as their Operation sections write them, with the console's readings (D-30): the
/// product's own overflow gives maxmin of ±TEMP's sign; an accumulator whose exponent field is 255 is kept, with O; a
/// product that underflows sets SU alone; the sum ACC ± TEMP is ADD.S's, its overflow and underflow O and U.
pub fn fused(name: &str, acc: u32, a: u32, b: u32) -> Outcome {
    let subtract = name.starts_with("msub");
    let (t, t_over, t_under) = mul(a, b);
    let t = if subtract { t ^ 0x8000_0000 } else { t };
    let mut f = if t_under { SU } else { 0 };
    let value = if t_over {
        f |= O | SO;
        t
    } else if acc >> 23 & 255 == 255 {
        f |= O | SO;
        acc
    } else {
        let (v, o, u) = add(acc, t);
        f |= ou(o, u);
        v
    };
    Outcome { value, fcr31: FIXED | f }
}

/// MAX.S and MIN.S on the bits as sign and magnitude, −0 below +0 (D-31); O and U cleared.
pub fn pick(name: &str, a: u32, b: u32) -> u32 {
    let key = |x: u32| if x >> 31 == 1 { !(x & FMAX) as i32 as i64 } else { (x & FMAX) as i64 };
    let first = if name == "max.s" { key(a) >= key(b) } else { key(a) <= key(b) };
    if first { a } else { b }
}

/// C.cond.S: the comparison on values, every zero equal (the pages, "+0 = −0"; the would-be denormals are zeros).
pub fn compare(name: &str, a: u32, b: u32) -> bool {
    let v = |x: u32| match number(x) {
        None => 0i64,
        Some((s, e, m)) => {
            let mag = (e as i64) << 23 | (m as i64 & 0x7F_FFFF);
            if s == 1 { -mag } else { mag }
        }
    };
    match name {
        "c.f.s" => false,
        "c.eq.s" => v(a) == v(b),
        "c.lt.s" => v(a) < v(b),
        "c.le.s" => v(a) <= v(b),
        _ => panic!("{name}"),
    }
}

/// CVT.W.S's Operation section: convert(FPR[fs], W) towards zero when the exponent field is at most 9Dh, else clamp.
pub fn cvt_w(x: u32) -> u32 {
    if x >> 23 & 255 > 0x9D {
        return if x >> 31 == 0 { 0x7FFF_FFFF } else { 0x8000_0000 };
    }
    let Some((s, e, m)) = number(x) else { return 0 };
    let v = if e >= 150 { m << (e - 150) } else { m >> (150 - e).min(127) } as i64;
    (if s == 1 { -v } else { v }) as u32
}

/// CVT.S.W: the integer's upper 24 significant bits, the rest truncated (VU User's Manual §2.2's conversion rule).
pub fn cvt_s(w: u32) -> u32 {
    let v = w as i32 as i64;
    if v == 0 {
        return 0;
    }
    truncate((v < 0) as u32, v.unsigned_abs() as u128, 150).0
}
