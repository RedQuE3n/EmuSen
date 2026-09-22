//! The VR4300's floating point, ported from C#'s SoftFloat, SoftFloatMath, SoftFloatConvert and HostSingle. See Mars_FpuMath.md.

pub const INEXACT: u32 = 1 << 0;
pub const UNDERFLOW: u32 = 1 << 1;
pub const OVERFLOW: u32 = 1 << 2;
pub const DIVIDE_BY_ZERO: u32 = 1 << 3;
pub const INVALID: u32 = 1 << 4;

pub const ROUND_NEAREST: u32 = 0;
pub const ROUND_ZERO: u32 = 1;
pub const ROUND_POSITIVE: u32 = 2;
pub const ROUND_NEGATIVE: u32 = 3;

const WORKING_BIT: i32 = 127;
const WIDEST_SOURCE: i64 = 1 << 55;
const WIDEST_WHOLE_EXPONENT: i32 = 53;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FloatClass {
    Zero,
    Subnormal,
    Normal,
    Infinity,
    Nan,
    NanUnsupported,
}

/// `FloatFormat`: the handful of numbers every operation needs.
#[derive(Clone, Copy, Debug)]
pub struct FloatFormat {
    pub mantissa_bits: i32,
    pub exponent_bits: i32,
    pub bias: i32,
    pub default_nan: u64,
}

pub const SINGLE: FloatFormat = FloatFormat { mantissa_bits: 23, exponent_bits: 8, bias: 127, default_nan: 0x7FBF_FFFF };
pub const DOUBLE: FloatFormat = FloatFormat { mantissa_bits: 52, exponent_bits: 11, bias: 1023, default_nan: 0x7FF7_FFFF_FFFF_FFFF };

impl FloatFormat {
    #[inline]
    fn max_biased_exponent(&self) -> i32 {
        (1 << self.exponent_bits) - 1
    }
    #[inline]
    fn mantissa_mask(&self) -> u64 {
        (1u64 << self.mantissa_bits) - 1
    }
    #[inline]
    fn nan_select_bit(&self) -> u64 {
        1u64 << (self.mantissa_bits - 1)
    }
    #[inline]
    pub fn sign_bit(&self) -> u64 {
        1u64 << (self.mantissa_bits + self.exponent_bits)
    }
    #[inline]
    fn largest_finite(&self) -> u64 {
        (((self.max_biased_exponent() - 1) as u64) << self.mantissa_bits) | self.mantissa_mask()
    }
    #[inline]
    fn infinity(&self) -> u64 {
        (self.max_biased_exponent() as u64) << self.mantissa_bits
    }
}

/// `SoftFloat`: the leading one at bit 63 of the significand, the exponent unbiased.
#[derive(Clone, Copy, Debug)]
pub struct SoftFloat {
    pub sign: bool,
    pub exponent: i32,
    pub significand: u64,
    pub class: FloatClass,
}

impl SoftFloat {
    #[inline]
    pub fn is_nan(&self) -> bool {
        matches!(self.class, FloatClass::Nan | FloatClass::NanUnsupported)
    }

    pub fn unpack(bits: u64, f: &FloatFormat) -> SoftFloat {
        let mut v = SoftFloat { sign: bits & f.sign_bit() != 0, exponent: 0, significand: 0, class: FloatClass::Zero };
        let biased = ((bits >> f.mantissa_bits) & f.max_biased_exponent() as u64) as i32;
        let mantissa = bits & f.mantissa_mask();
        if biased == 0 {
            v.class = if mantissa == 0 { FloatClass::Zero } else { FloatClass::Subnormal };
            if v.class == FloatClass::Subnormal {
                let highest = 63 - mantissa.leading_zeros() as i32;
                v.exponent = 1 - f.bias - f.mantissa_bits + highest;
                v.significand = mantissa << (63 - highest);
            }
            return v;
        }
        if biased == f.max_biased_exponent() {
            v.class = if mantissa == 0 {
                FloatClass::Infinity
            } else if mantissa & f.nan_select_bit() != 0 {
                FloatClass::Nan
            } else {
                FloatClass::NanUnsupported
            };
            return v;
        }
        v.class = FloatClass::Normal;
        v.exponent = biased - f.bias;
        v.significand = (mantissa | (1u64 << f.mantissa_bits)) << (63 - f.mantissa_bits);
        v
    }

    pub fn compare(a: &SoftFloat, b: &SoftFloat) -> i32 {
        if a.class == FloatClass::Zero && b.class == FloatClass::Zero {
            return 0;
        }
        if a.sign != b.sign {
            return if a.sign { -1 } else { 1 };
        }
        let magnitude = compare_magnitude(a, b);
        if a.sign { -magnitude } else { magnitude }
    }
}

fn compare_magnitude(a: &SoftFloat, b: &SoftFloat) -> i32 {
    if a.class == FloatClass::Infinity {
        return if b.class == FloatClass::Infinity { 0 } else { 1 };
    }
    if b.class == FloatClass::Infinity {
        return -1;
    }
    if a.class == FloatClass::Zero {
        return if b.class == FloatClass::Zero { 0 } else { -1 };
    }
    if b.class == FloatClass::Zero {
        return 1;
    }
    if a.exponent != b.exponent {
        return if a.exponent < b.exponent { -1 } else { 1 };
    }
    if a.significand == b.significand {
        return 0;
    }
    if a.significand < b.significand { -1 } else { 1 }
}

#[inline]
fn zero(sign: bool, f: &FloatFormat) -> u64 {
    if sign { f.sign_bit() } else { 0 }
}
#[inline]
fn infinite(sign: bool, f: &FloatFormat) -> u64 {
    f.infinity() | if sign { f.sign_bit() } else { 0 }
}
#[inline]
fn smallest(sign: bool, f: &FloatFormat) -> u64 {
    (1u64 << f.mantissa_bits) | if sign { f.sign_bit() } else { 0 }
}
#[inline]
fn largest(sign: bool, f: &FloatFormat) -> u64 {
    f.largest_finite() | if sign { f.sign_bit() } else { 0 }
}

/// `FloatResult`: what an operation produced, and what it wants recorded.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FloatResult {
    pub bits: u64,
    pub flags: u32,
    pub unimplemented: bool,
}

impl FloatResult {
    #[inline]
    pub fn refused() -> Self {
        FloatResult { bits: 0, flags: 0, unimplemented: true }
    }
    #[inline]
    pub fn exact(bits: u64) -> Self {
        FloatResult { bits, flags: 0, unimplemented: false }
    }
    #[inline]
    pub fn raised(bits: u64, flags: u32) -> Self {
        FloatResult { bits, flags, unimplemented: false }
    }
}

#[inline]
fn refuses(v: &SoftFloat) -> bool {
    matches!(v.class, FloatClass::Subnormal | FloatClass::NanUnsupported)
}

#[inline]
fn invalidated(f: &FloatFormat) -> FloatResult {
    FloatResult::raised(f.default_nan, INVALID)
}

fn repack(v: &SoftFloat, f: &FloatFormat) -> u64 {
    let mantissa = (v.significand >> (63 - f.mantissa_bits)) & f.mantissa_mask();
    (if v.sign { f.sign_bit() } else { 0 }) | (((v.exponent + f.bias) as u64) << f.mantissa_bits) | mantissa
}

#[inline]
fn highest_bit(value: u128) -> i32 {
    127 - value.leading_zeros() as i32
}

pub fn add(left: u64, right: u64, f: &FloatFormat, mode: u32, flush: bool) -> FloatResult {
    add_signed(left, right, f, mode, flush, false)
}

pub fn subtract(left: u64, right: u64, f: &FloatFormat, mode: u32, flush: bool) -> FloatResult {
    add_signed(left, right, f, mode, flush, true)
}

pub fn multiply(left: u64, right: u64, f: &FloatFormat, mode: u32, flush: bool) -> FloatResult {
    let a = SoftFloat::unpack(left, f);
    let b = SoftFloat::unpack(right, f);
    if refuses(&a) || refuses(&b) {
        return FloatResult::refused();
    }
    if a.is_nan() || b.is_nan() {
        return invalidated(f);
    }
    let sign = a.sign ^ b.sign;
    let is_zero = a.class == FloatClass::Zero || b.class == FloatClass::Zero;
    let is_infinite = a.class == FloatClass::Infinity || b.class == FloatClass::Infinity;
    if is_zero && is_infinite {
        return invalidated(f);
    }
    if is_infinite {
        return FloatResult::exact(infinite(sign, f));
    }
    if is_zero {
        return FloatResult::exact(zero(sign, f));
    }
    let mut product = a.significand as u128 * b.significand as u128;
    let mut exponent = a.exponent + b.exponent;
    if (product >> WORKING_BIT) != 0 {
        exponent += 1;
    } else {
        product <<= 1;
    }
    round(sign, exponent, product, false, f, mode, flush)
}

pub fn divide(left: u64, right: u64, f: &FloatFormat, mode: u32, flush: bool) -> FloatResult {
    let a = SoftFloat::unpack(left, f);
    let b = SoftFloat::unpack(right, f);
    if refuses(&a) || refuses(&b) {
        return FloatResult::refused();
    }
    if a.is_nan() || b.is_nan() {
        return invalidated(f);
    }
    let sign = a.sign ^ b.sign;
    if a.class == FloatClass::Infinity {
        return if b.class == FloatClass::Infinity { invalidated(f) } else { FloatResult::exact(infinite(sign, f)) };
    }
    if b.class == FloatClass::Infinity {
        return FloatResult::exact(zero(sign, f));
    }
    if b.class == FloatClass::Zero {
        return if a.class == FloatClass::Zero {
            invalidated(f)
        } else {
            FloatResult::raised(infinite(sign, f), DIVIDE_BY_ZERO)
        };
    }
    if a.class == FloatClass::Zero {
        return FloatResult::exact(zero(sign, f));
    }
    let numerator = (a.significand as u128) << 64;
    let quotient = numerator / b.significand as u128;
    let sticky = numerator.wrapping_sub(quotient.wrapping_mul(b.significand as u128)) != 0;
    let leading = highest_bit(quotient);
    let shift = WORKING_BIT - leading;
    round(sign, a.exponent - b.exponent - 64 - shift + WORKING_BIT, quotient.wrapping_shl(shift as u32), sticky, f, mode, flush)
}

pub fn sign(bits: u64, f: &FloatFormat, negate: bool) -> FloatResult {
    let v = SoftFloat::unpack(bits, f);
    if refuses(&v) {
        return FloatResult::refused();
    }
    if v.is_nan() {
        return invalidated(f);
    }
    FloatResult::exact(if negate { bits ^ f.sign_bit() } else { bits & !f.sign_bit() })
}

pub fn square_root(bits: u64, f: &FloatFormat, mode: u32, flush: bool) -> FloatResult {
    let v = SoftFloat::unpack(bits, f);
    if refuses(&v) {
        return FloatResult::refused();
    }
    if v.is_nan() {
        return invalidated(f);
    }
    if v.class == FloatClass::Zero {
        return FloatResult::exact(zero(v.sign, f));
    }
    if v.sign {
        return invalidated(f);
    }
    if v.class == FloatClass::Infinity {
        return FloatResult::exact(infinite(false, f));
    }
    let mut power = v.exponent - 63;
    let mut radicand = v.significand as u128;
    if power & 1 != 0 {
        radicand <<= 1;
        power -= 1;
    }
    radicand <<= 62;
    let (root, exact) = integer_square_root(radicand);
    let leading = highest_bit(root);
    let shift = WORKING_BIT - leading;
    round(false, (power / 2) - 31 - shift + WORKING_BIT, root.wrapping_shl(shift as u32), !exact, f, mode, flush)
}

fn integer_square_root(value: u128) -> (u128, bool) {
    let mut root: u128 = 0;
    let mut remainder: u128 = 0;
    let mut shift = 126i32;
    while shift >= 0 {
        remainder = (remainder << 2) | ((value >> shift) & 3);
        let candidate = (root << 2) | 1;
        root <<= 1;
        if remainder >= candidate {
            remainder -= candidate;
            root |= 1;
        }
        shift -= 2;
    }
    (root, remainder == 0)
}

fn add_signed(left: u64, right: u64, f: &FloatFormat, mode: u32, flush: bool, negate_right: bool) -> FloatResult {
    let mut a = SoftFloat::unpack(left, f);
    let mut b = SoftFloat::unpack(right, f);
    if refuses(&a) || refuses(&b) {
        return FloatResult::refused();
    }
    if a.is_nan() || b.is_nan() {
        return invalidated(f);
    }
    if negate_right {
        b.sign = !b.sign;
    }
    if a.class == FloatClass::Infinity || b.class == FloatClass::Infinity {
        if a.class == FloatClass::Infinity && b.class == FloatClass::Infinity && a.sign != b.sign {
            return invalidated(f);
        }
        let s = if a.class == FloatClass::Infinity { a.sign } else { b.sign };
        return FloatResult::exact(infinite(s, f));
    }
    if a.class == FloatClass::Zero && b.class == FloatClass::Zero {
        let s = if a.sign == b.sign { a.sign } else { mode == ROUND_NEGATIVE };
        return FloatResult::exact(zero(s, f));
    }
    if a.class == FloatClass::Zero {
        return FloatResult::exact(repack(&b, f));
    }
    if b.class == FloatClass::Zero {
        return FloatResult::exact(repack(&a, f));
    }
    if b.exponent > a.exponent || (b.exponent == a.exponent && b.significand > a.significand) {
        std::mem::swap(&mut a, &mut b);
    }
    let distance = a.exponent - b.exponent;
    let wide = (a.significand as u128) << 63;
    let mut addend = (b.significand as u128) << 63;
    let mut sticky = false;
    if distance >= 128 {
        sticky = true;
        addend = 0;
    } else if distance > 0 {
        sticky = (addend & ((1u128 << distance) - 1)) != 0;
        addend >>= distance;
    }
    let sum = if a.sign == b.sign {
        wide.wrapping_add(addend)
    } else {
        let mut sum = wide.wrapping_sub(addend);
        if sticky {
            sum = sum.wrapping_sub(1);
        }
        if sum == 0 {
            return FloatResult::exact(zero(mode == ROUND_NEGATIVE, f));
        }
        sum
    };
    let leading = highest_bit(sum);
    round(a.sign, a.exponent + 1 - (WORKING_BIT - leading), sum.wrapping_shl((WORKING_BIT - leading) as u32), sticky, f, mode, flush)
}

/// `RoundInto`: conversions arrive with a significand already normalised.
#[inline]
pub fn round_into(sign: bool, exponent: i32, significand: u128, sticky: bool, f: &FloatFormat, mode: u32, flush: bool) -> FloatResult {
    round(sign, exponent, significand, sticky, f, mode, flush)
}

fn round(sign: bool, mut exponent: i32, significand: u128, sticky: bool, f: &FloatFormat, mode: u32, flush: bool) -> FloatResult {
    let keep = f.mantissa_bits + 1;
    let shift = 128 - keep;
    let mut head = significand >> shift;
    let tail = significand - (head << shift);
    let half = 1u128 << (shift - 1);
    let round_bit = tail >= half;
    let below = (tail & (half - 1)) != 0 || sticky;
    let inexact = round_bit || below;
    let increment = match mode {
        ROUND_NEAREST => round_bit && (below || (head & 1) != 0),
        ROUND_ZERO => false,
        ROUND_POSITIVE => inexact && !sign,
        _ => inexact && sign,
    };
    if increment {
        head += 1;
    }
    if (head >> keep) != 0 {
        head >>= 1;
        exponent += 1;
    }
    let biased = exponent + f.bias;
    if biased >= f.max_biased_exponent() {
        return overflowed(sign, f, mode);
    }
    if biased <= 0 {
        if !flush {
            return FloatResult::refused();
        }
        let away = if mode == ROUND_POSITIVE { !sign } else { mode == ROUND_NEGATIVE && sign };
        let flushed = if away { smallest(sign, f) } else { zero(sign, f) };
        return FloatResult::raised(flushed, UNDERFLOW | INEXACT);
    }
    let bits = (if sign { f.sign_bit() } else { 0 }) | ((biased as u64) << f.mantissa_bits) | ((head as u64) & f.mantissa_mask());
    FloatResult::raised(bits, if inexact { INEXACT } else { 0 })
}

fn overflowed(sign: bool, f: &FloatFormat, mode: u32) -> FloatResult {
    let is_infinite = match mode {
        ROUND_NEAREST => true,
        ROUND_ZERO => false,
        ROUND_POSITIVE => !sign,
        _ => sign,
    };
    let bits = if is_infinite { infinite(sign, f) } else { largest(sign, f) };
    FloatResult::raised(bits, OVERFLOW | INEXACT)
}

/// `SoftFloatConvert.Between`: one float format to the other.
pub fn between(bits: u64, from: &FloatFormat, to: &FloatFormat, mode: u32, flush: bool) -> FloatResult {
    let v = SoftFloat::unpack(bits, from);
    match v.class {
        FloatClass::Subnormal | FloatClass::NanUnsupported => FloatResult::refused(),
        FloatClass::Nan => FloatResult::raised(to.default_nan, INVALID),
        FloatClass::Infinity => FloatResult::exact(infinite(v.sign, to)),
        FloatClass::Zero => FloatResult::exact(zero(v.sign, to)),
        FloatClass::Normal => round_into(v.sign, v.exponent, (v.significand as u128) << 64, false, to, mode, flush),
    }
}

/// `SoftFloatConvert.ToInteger`: out of range, infinite or not a number is refused rather than saturated.
pub fn to_integer(bits: u64, from: &FloatFormat, wide: bool, mode: u32) -> FloatResult {
    let v = SoftFloat::unpack(bits, from);
    match v.class {
        FloatClass::Subnormal | FloatClass::NanUnsupported | FloatClass::Nan | FloatClass::Infinity => {
            return FloatResult::refused();
        }
        FloatClass::Zero => return FloatResult::exact(0),
        FloatClass::Normal => {}
    }
    if v.exponent >= if wide { WIDEST_WHOLE_EXPONENT } else { 32 } {
        return FloatResult::refused();
    }
    let shift = 63 - v.exponent;
    let (mut whole, round_bit, below): (u128, bool, bool) = if shift <= 0 {
        ((v.significand as u128).wrapping_shl((-shift) as u32), false, false)
    } else if shift >= 128 {
        (0, false, true)
    } else {
        let whole = (v.significand as u128) >> shift;
        let tail = (v.significand as u128) - (whole << shift);
        let half = 1u128 << (shift - 1);
        (whole, tail >= half, (tail & (half - 1)) != 0)
    };
    let inexact = round_bit || below;
    let increment = match mode {
        ROUND_NEAREST => round_bit && (below || (whole & 1) != 0),
        ROUND_ZERO => false,
        ROUND_POSITIVE => inexact && !v.sign,
        _ => inexact && v.sign,
    };
    if increment {
        whole += 1;
    }
    let top = 1u128 << if wide { 63 } else { 31 };
    let limit = if v.sign { top } else { top - 1 };
    if whole > limit {
        return FloatResult::refused();
    }
    let magnitude = whole as u64;
    let mut result = if v.sign { 0u64.wrapping_sub(magnitude) } else { magnitude };
    if !wide {
        result &= 0xFFFF_FFFF;
    }
    FloatResult::raised(result, if inexact { INEXACT } else { 0 })
}

/// `SoftFloatConvert.FromInteger`.
pub fn from_integer(bits: u64, wide: bool, to: &FloatFormat, mode: u32, flush: bool) -> FloatResult {
    let value: i64 = if wide { bits as i64 } else { bits as u32 as i32 as i64 };
    if wide && !(-WIDEST_SOURCE..WIDEST_SOURCE).contains(&value) {
        return FloatResult::refused();
    }
    if value == 0 {
        return FloatResult::exact(0);
    }
    let sign = value < 0;
    let magnitude = if sign { value.wrapping_neg() as u64 } else { value as u64 };
    let leading = 63 - magnitude.leading_zeros() as i32;
    round_into(sign, leading, ((magnitude << (63 - leading)) as u128) << 64, false, to, mode, flush)
}

/// `SmallestNormal`: 1.17549435082228750797e-38, which is 2^-126 exactly as a double.
const SMALLEST_NORMAL: f64 = f32::MIN_POSITIVE as f64;

/// `HostSingle.TryCompute`: single add, subtract and multiply on the host, only where its answer is provably the software unit's.
#[inline]
pub fn host_single(function: u32, left: u32, right: u32) -> Option<(u64, u32)> {
    let left_exponent = (left >> 23) & 0xFF;
    let right_exponent = (right >> 23) & 0xFF;
    if left_exponent.wrapping_sub(1) >= 254 || right_exponent.wrapping_sub(1) >= 254 {
        return None;
    }
    let a = f32::from_bits(left) as f64;
    let b = f32::from_bits(right) as f64;
    let exact = if function == 2 {
        a * b
    } else {
        if (left_exponent as i32 - right_exponent as i32).abs() > 28 {
            return None;
        }
        if function == 0 { a + b } else { a - b }
    };
    let size = exact.abs();
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if !(size >= SMALLEST_NORMAL) {
        return None;
    }
    let rounded = exact as f32;
    if rounded.is_infinite() {
        return None;
    }
    Some((rounded.to_bits() as u64, if rounded as f64 != exact { INEXACT } else { 0 }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The host path and the software unit agree wherever the host answers, as `MarsHostSingleTests` asks of C#.
    #[test]
    fn the_host_single_agrees_with_the_software_unit() {
        let mut seed = 0x1234_5678_9ABC_DEF0u64;
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let mut answered = 0;
        for _ in 0..200_000 {
            let r = next();
            let left = r as u32;
            let right = if r & (1 << 40) != 0 { (r >> 32) as u32 } else { left ^ ((r >> 33) as u32 & 0x807F_FFFF) };
            for function in 0..3 {
                if let Some((bits, flags)) = host_single(function, left, right) {
                    answered += 1;
                    let reference = match function {
                        0 => add(left as u64, right as u64, &SINGLE, ROUND_NEAREST, false),
                        1 => subtract(left as u64, right as u64, &SINGLE, ROUND_NEAREST, false),
                        _ => multiply(left as u64, right as u64, &SINGLE, ROUND_NEAREST, false),
                    };
                    assert_eq!((bits, flags, false), (reference.bits, reference.flags, reference.unimplemented), "{function} {left:08X} {right:08X}");
                }
            }
        }
        assert!(answered > 100_000);
    }

    #[test]
    fn known_values_round_as_the_csharp_unit_does() {
        assert_eq!(add(0x3F80_0000, 0x3F80_0000, &SINGLE, ROUND_NEAREST, false), FloatResult::exact(0x4000_0000));
        assert_eq!(divide(0x3F80_0000, 0x4040_0000, &SINGLE, ROUND_NEAREST, false), FloatResult::raised(0x3EAA_AAAB, INEXACT));
        assert_eq!(divide(0x3F80_0000, 0x4040_0000, &SINGLE, ROUND_ZERO, false), FloatResult::raised(0x3EAA_AAAA, INEXACT));
        assert_eq!(square_root(0x4000_0000_0000_0000, &DOUBLE, ROUND_NEAREST, false), FloatResult::raised(0x3FF6_A09E_667F_3BCD, INEXACT));
        assert_eq!(divide(0x3F80_0000, 0, &SINGLE, ROUND_NEAREST, false), FloatResult::raised(0x7F80_0000, DIVIDE_BY_ZERO));
        assert_eq!(to_integer(0x3FC0_0000, &SINGLE, false, ROUND_NEAREST), FloatResult::raised(2, INEXACT));
        assert_eq!(to_integer(0x4020_0000, &SINGLE, false, ROUND_NEAREST), FloatResult::raised(2, INEXACT));
        assert_eq!(to_integer(0xC020_0000, &SINGLE, false, ROUND_NEGATIVE), FloatResult::raised(0xFFFF_FFFD, INEXACT));
        assert_eq!(from_integer(3, false, &SINGLE, ROUND_NEAREST, false), FloatResult::exact(0x4040_0000));
        assert!(multiply(0x0080_0000, 0x0080_0000, &SINGLE, ROUND_NEAREST, false).unimplemented);
        assert_eq!(multiply(0x0080_0000, 0x0080_0000, &SINGLE, ROUND_POSITIVE, true), FloatResult::raised(0x0080_0000, UNDERFLOW | INEXACT));
        assert_eq!(between(0x3FF0_0000_0000_0000, &DOUBLE, &SINGLE, ROUND_NEAREST, false), FloatResult::exact(0x3F80_0000));
    }
}
