//! The VU User's Manual's pages for the macro instructions (§6.2, which refers to §4.2 and §4.3) and its flag rules
//! (§2.3, §3.3), transcribed as plain functions on bit strings apart from the interpreter (`src/vu`, `src/ee/cop2.rs`)
//! and the format's module (`src/float.rs`). A lane's arithmetic is the FPU's chapter-5 transcription (`s9::manual`):
//! the VU manual's §2.2 states the same rounding as the EE Core User's Manual's §8.7, and its multiply pages' Remarks the
//! same uneven multiplier ("by using VF[fs] as a multiplicand, the results of multiplication with 1 are guaranteed to be
//! accurate"), so the shared readings are D-25 to D-30's (Sapphire_Disputes.md D-43). The VU's own rules are written
//! here: the MAC and status flags, the multiply-adds' sticky flags, the divider's flags, the conversions, CLIP and the
//! random unit.

use crate::s9::manual as fpu;

pub const SIGN: u32 = 0x8000_0000;
pub const MAX: u32 = 0x7FFF_FFFF;
pub type Lanes = [u32; 4];
/// A lane's result: its bits, and whether the exponent overflowed or underflowed.
pub type R = (u32, bool, bool);

pub fn zero(x: u32) -> bool {
    x >> 23 & 255 == 0
}

/// One lane's MAC bits (§3.3.1): Z when the result is zero, S its sign bit (a clamped −0 or −MAX included, "set
/// according to the clamped results", D-44), U and O; x is the high bit of each group of four.
pub fn mac_bits(r: R, lane: usize) -> u32 {
    let at = 3 - lane as u32;
    (zero(r.0) as u32) << at | (r.0 >> 31) << (4 + at) | (r.2 as u32) << (8 + at) | (r.1 as u32) << (12 + at)
}

/// The status flag's Z, S, U and O: any lane's (§3.3.2).
pub fn zsuo(mac: u32) -> u32 {
    (0..4).map(|g| ((mac >> (4 * g) & 0xF != 0) as u32) << g).sum()
}

/// The FMAC's flags after an instruction over the lanes of `dest` (bit 3 x … bit 0 w).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flags {
    pub mac: u32,
    pub status: u32,
}

impl Flags {
    /// A flag-setting FMAC instruction: the MAC flags from the results, Z, S, U, O collected, the sticky flags taking
    /// them and the products' (a multiply-add's, §3.3.2: "multiplication results are reflected only in the
    /// ZS/SS/US/OS flags"); I and D kept.
    pub fn fmac(&mut self, results: &[(usize, R, Option<R>)]) {
        let mac = results.iter().fold(0, |a, &(l, r, _)| a | mac_bits(r, l));
        let extra = results.iter().filter_map(|&(l, _, p)| p.map(|p| zsuo(mac_bits(p, l)))).fold(0, |a, b| a | b);
        let f = zsuo(mac);
        self.mac = mac;
        self.status = self.status & 0xFF0 | f | (f | extra) << 6;
    }
    /// CTC2 to the status flag: the sticky flags written, the rest kept (§5.1.3).
    pub fn write_status(&mut self, v: u32) {
        self.status = self.status & 0x3F | v & 0xFC0;
    }
    /// The divider's I and D written, IS and DS collecting them (§3.3.5's DIV, RSQRT and SQRT rows).
    pub fn divide(&mut self, invalid: bool, divide: bool) {
        let f = (invalid as u32) << 4 | (divide as u32) << 5;
        self.status = self.status & !0x30 | f | f << 6;
    }
}

pub fn lanes_of(dest: u32) -> impl Iterator<Item = usize> {
    (0..4).filter(move |l| dest & 8 >> l != 0)
}

pub fn add(a: u32, b: u32) -> R {
    fpu::add(a, b)
}
pub fn sub(a: u32, b: u32) -> R {
    fpu::sub(a, b)
}
pub fn mul(a: u32, b: u32) -> R {
    fpu::mul(a, b)
}

/// MADD's and MSUB's lane (§3.3.6's tables): a product that overflows gives ±MAX of the term added, an accumulator with
/// exponent field 255 is kept with O, otherwise ACC ± the product; with the term as added, its own flags.
pub fn fused(acc: u32, a: u32, b: u32, subtract: bool) -> (R, R) {
    let (t, over, under) = fpu::mul(a, b);
    let t = if subtract { t ^ SIGN } else { t };
    let value = if over {
        (t, true, false)
    } else if acc >> 23 & 255 == 255 {
        (acc, true, false)
    } else {
        fpu::add(acc, t)
    };
    (value, (t, over, under))
}

/// DIV (§4.3, §2.3): the quotient as DIV.S's; I for 0/0, D for any other zero divisor.
pub fn div(a: u32, b: u32) -> (u32, bool, bool) {
    (fpu::div(a, b).0, zero(a) && zero(b), !zero(a) && zero(b))
}
/// SQRT: the root of the magnitude, I for a negative value, D cleared.
pub fn sqrt(b: u32) -> (u32, bool, bool) {
    (fpu::sqrt(b), b >> 31 == 1 && !zero(b), false)
}
/// RSQRT: the value as RSQRT.S's; D for a zero root under a non-zero dividend ("0 division (except 0/0)", §3.3.2), I
/// for a negative value (D-46).
pub fn rsqrt(a: u32, b: u32) -> (u32, bool, bool) {
    (fpu::rsqrt(a, b), b >> 31 == 1 && !zero(b), zero(b) && !zero(a))
}

/// MAX and MINI: as MAX.S and MIN.S compare (D-31, D-47).
pub fn max(a: u32, b: u32) -> u32 {
    fpu::pick("max.s", a, b)
}
pub fn mini(a: u32, b: u32) -> u32 {
    fpu::pick("min.s", a, b)
}

/// FTOIn: x · 2ⁿ truncated towards zero, ±MAX past the word's range (§2.2, §2.3).
pub fn ftoi(x: u32, n: u32) -> u32 {
    if zero(x) {
        return 0;
    }
    let (s, e, m) = (x >> 31, (x >> 23 & 255) as i32, (x & 0x7F_FFFF | 0x80_0000) as i128);
    let shift = e - 150 + n as i32;
    let v = if shift >= 0 { if shift > 40 { i128::MAX / 4 } else { m << shift } } else { m >> (-shift).min(64) };
    let v = if s == 1 { -v } else { v };
    if v > i32::MAX as i128 {
        MAX
    } else if v < i32::MIN as i128 {
        SIGN
    } else {
        v as i32 as u32
    }
}
/// ITOFn: the word's top 24 significant bits as a float (§2.2's conversion rule), times 2⁻ⁿ.
pub fn itof(w: u32, n: u32) -> u32 {
    let f = fpu::cvt_s(w);
    if zero(f) { f } else { f - (n << 23) }
}

/// A value as a number for the compares: zero for every exponent field of 0, sign and magnitude otherwise.
fn value(x: u32) -> i64 {
    if zero(x) {
        0
    } else if x >> 31 == 1 {
        -((x & MAX) as i64)
    } else {
        (x & MAX) as i64
    }
}
/// CLIPw's judgement of x, y and z against ±|w| (§3.3.3): +x in bit 0, −x in bit 1, then y and z (D-49).
pub fn clip(v: Lanes, w: u32) -> u32 {
    let limit = value(w & MAX);
    (0..3).map(|l| ((value(v[l]) > limit) as u32) << (2 * l) | ((value(v[l]) < -limit) as u32) << (2 * l + 1)).sum()
}

/// RNEXT's M series, as `vu/lower/random`'s console text decides it (D-48): R shifted left, bit 22 XOR bit 4 entering.
pub fn next_r(r: u32) -> u32 {
    let feed = (r >> 22 ^ r >> 4) & 1;
    (r << 1 | feed) & 0x7F_FFFF
}
