//! The PlayStation 2's single-precision format and its arithmetic, in software on integers, so that every host and
//! every tier computes the same bits (Sapphire_Plan.md §5.3). The one owner of the format: the EE's FPU (`ee::fpu`) and
//! the vector units (`vu`) both compute with it. A number is IEEE 754's single layout read without NaN, infinity or
//! denormal: an exponent field of 255 is an ordinary exponent, and a field of 0 is zero whatever its fraction holds
//! (EE Core User's Manual §8.1.1; VU User's Manual §2.1.1). Each function returns its result with the exponent overflow
//! and underflow it met; which FCR31, MAC or status flags those become is the instruction's business, not this
//! module's. Which rules the two units share and where they part is Sapphire_Native.md §21.3.
//!
//! The rounding is the console's, as ps2autotests' `cpu/ee_fpu` console texts decide it where the manuals stop
//! (Sapphire_Native.md §19.3, Sapphire_Disputes.md D-25 to D-28): an addition aligns the smaller operand to the larger's
//! 24 bits and one more and drops what falls below, then truncates; a multiplication forms its product from a radix-4 Booth recoding
//! of the second operand with the columns below 15 dropped, then truncates; a division and a square root round to the
//! nearest; RSQRT divides by the rounded square root and truncates.
//!
//! The product is computed in closed form and DIV and SQRT on the host's arithmetic where that is proven to give the same
//! bits and flags (`host`); Sapphire_Speed.md §56 records the census, the proofs and the tests.

/// The largest magnitude, the result of an overflow and of a division by zero (EE Core User's Manual Table 8-4).
pub const MAX: u32 = 0x7FFF_FFFF;
pub const SIGN: u32 = 0x8000_0000;

/// A result and the exponent overflow or underflow that produced it, if any.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    pub bits: u32,
    pub overflow: bool,
    pub underflow: bool,
}

impl Outcome {
    const fn of(bits: u32) -> Outcome {
        Outcome { bits, overflow: false, underflow: false }
    }
}

/// A division's or reciprocal square root's result, with the invalid (0/0, a negative root) and divide-by-zero
/// conditions beside the exponent's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Quotient {
    pub value: Outcome,
    pub invalid: bool,
    pub divide: bool,
}

/// A multiply-add's result: the sum and its flags, and the product as it was added (negated for a subtraction) with
/// the product's own overflow and underflow, which the FPU's SU and the VU's sticky flags take (the FPU manual's
/// "multiply underflow occurred"; VU User's Manual §3.3.2).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Fused {
    pub value: Outcome,
    pub product: Outcome,
}

const fn sign(x: u32) -> u32 {
    x >> 31
}
const fn exponent(x: u32) -> i32 {
    (x >> 23 & 0xFF) as i32
}
/// The 24-bit significand with its hidden bit.
const fn significand(x: u32) -> u64 {
    (0x80_0000 | x & 0x7F_FFFF) as u64
}
/// Whether `x` is zero: an exponent field of 0, the would-be denormals included.
pub const fn is_zero(x: u32) -> bool {
    exponent(x) == 0
}
const fn zero(s: u32) -> u32 {
    s << 31
}
const fn largest(s: u32) -> u32 {
    s << 31 | MAX
}

/// Packs a significand of 24 bits (its top bit set) and a biased exponent, flushing below 1 and saturating above 255.
fn finish(s: u32, e: i32, m: u64) -> Outcome {
    debug_assert!(m >> 23 == 1, "{m:x}");
    if e > 255 {
        Outcome { bits: largest(s), overflow: true, underflow: false }
    } else if e < 1 {
        Outcome { bits: zero(s), overflow: false, underflow: true }
    } else {
        Outcome::of(s << 31 | (e as u32) << 23 | (m as u32 & 0x7F_FFFF))
    }
}

/// Rounds a quotient or root of 25 bits (`q`, its top bit at 24) to 24 by its last bit and the rest's sign (`sticky`):
/// the nearest, ties to even, though neither operation can produce a tie (Sapphire_Native.md §19.3).
fn nearest(s: u32, mut e: i32, q: u64, sticky: bool) -> Outcome {
    let mut m = q >> 1;
    if q & 1 == 1 && (sticky || m & 1 == 1) {
        m += 1;
        if m >> 24 == 1 {
            m >>= 1;
            e += 1;
        }
    }
    finish(s, e, m)
}

/// The bits the adder keeps below the larger operand's 24 when it aligns the smaller (Sapphire_Disputes.md D-25).
const GUARD: u32 = 1;

/// `a + b`: the smaller operand aligned to the larger's 24 bits and `GUARD` more, what falls below lost, the sum
/// truncated. A zero operand gives the other unchanged; two zeros give −0 only when both are −0 (Table 8-5).
pub fn add(a: u32, b: u32) -> Outcome {
    match (is_zero(a), is_zero(b)) {
        (true, true) => return Outcome::of(zero(sign(a) & sign(b))),
        (false, true) => return Outcome::of(a),
        (true, false) => return Outcome::of(b),
        _ => {}
    }
    let (big, small) = if a & MAX >= b & MAX { (a, b) } else { (b, a) };
    let d = exponent(big) - exponent(small);
    let aligned = if d < 25 { significand(small) << GUARD >> d } else { 0 };
    let wide = significand(big) << GUARD;
    let m = if sign(big) == sign(small) { wide + aligned } else { wide - aligned };
    if m == 0 {
        return Outcome::of(0);
    }
    let lead = 63 - m.leading_zeros() as i32;
    let e = exponent(big) + lead - 23 - GUARD as i32;
    let m = if lead >= 23 { m >> (lead - 23) } else { m << (23 - lead) };
    finish(sign(big), e, m)
}

/// `a − b`: the addition of `b` with its sign reversed.
pub fn sub(a: u32, b: u32) -> Outcome {
    add(a, b ^ SIGN)
}

/// The columns of the product below this one are not summed (Sapphire_Disputes.md D-26, D-1104).
const DROPPED_COLUMNS: u32 = 15;

/// `x × y` for two 24-bit significands as the multiplier sums them: `y` recoded as radix-4 Booth digits, each partial
/// product a multiple of `x` (a negative one as its ones' complement and a carry-in of 1), the columns below
/// `DROPPED_COLUMNS` left out. The result is the exact product or a little less. The reference `product` is proven
/// against (Sapphire_Speed.md §56.4).
#[cfg(test)]
fn booth(x: u64, y: u64) -> u64 {
    let y = y << 1;
    let mut total = 0u64;
    for i in (0..26).step_by(2) {
        let (multiple, negative) = match y >> i & 7 {
            0 | 7 => (0, false),
            1 | 2 => (x, false),
            3 => (2 * x, false),
            4 => (2 * x, true),
            _ => (x, true),
        };
        let (row, carry) = if negative { (!multiple, 1u64) } else { (multiple, 0) };
        let row = (row << i) & !((1u64 << DROPPED_COLUMNS) - 1);
        let carry = if i >= DROPPED_COLUMNS { carry << i } else { 0 };
        total = total.wrapping_add(row).wrapping_add(carry);
    }
    total & ((1 << 48) - 1)
}

/// What the columns below `DROPPED_COLUMNS` held, rows and carries (Sapphire_Speed.md §56.4).
fn loss(x: u64, y: u64) -> u64 {
    let (y, mask) = (y << 1, (1u64 << DROPPED_COLUMNS) - 1);
    let mut lost = 0;
    let mut i = 0;
    while i < DROPPED_COLUMNS {
        let d = y >> i & 7;
        let multiple = if d == 3 || d == 4 { 2 * x } else if d == 0 || d == 7 { 0 } else { x };
        let negative = d >> 2 & (d != 7) as u64;
        let row = if negative == 1 { !multiple } else { multiple };
        lost += (row << i & mask) + (negative << i);
        i += 2;
    }
    lost
}

/// The bits of `y` the digits below `DROPPED_COLUMNS` read.
const READ: u64 = (1 << ((DROPPED_COLUMNS - 1) / 2 * 2 + 2)) - 1;

/// The most `loss` can be, measured over every low part it reads (Sapphire_Speed.md §56.4).
const MOST_LOST: u64 = if DROPPED_COLUMNS == 1 { 2 } else { 8 << DROPPED_COLUMNS };

/// `booth`'s bits from column 23 up, which are all `mul` reads, without its loop (Sapphire_Speed.md §56.4).
fn product(x: u64, y: u64) -> u64 {
    let p = x * y;
    if DROPPED_COLUMNS == 1 {
        const LOSS: u64 = 0x1222_1000;
        p - (LOSS >> ((((y & 3) << 1) | (x & 1)) << 2) & 3)
    } else if y & READ == 0 || p & 0x7F_FFFF >= MOST_LOST {
        p
    } else {
        p - loss(x, y)
    }
}

/// `a × b`, `b` the operand the multiplier recodes (FPR[ft] for the FPU), so that the product is not commutative in its
/// last bit (Sapphire_Disputes.md D-26).
pub fn mul(a: u32, b: u32) -> Outcome {
    let s = sign(a) ^ sign(b);
    if is_zero(a) || is_zero(b) {
        return Outcome::of(zero(s));
    }
    let p = product(significand(a), significand(b));
    let e = exponent(a) + exponent(b) - 127;
    if p >> 47 == 1 { finish(s, e + 1, p >> 24) } else { finish(s, e, p >> 23) }
}

/// The significands' quotient `a / b` (both non-zero) as 25 bits with a sticky remainder, and its biased exponent.
fn quotient_bits(a: u32, b: u32) -> (i32, u64, bool) {
    let (mut n, d) = (significand(a), significand(b));
    let mut e = exponent(a) - exponent(b) + 127;
    if n < d {
        n <<= 1;
        e -= 1;
    }
    let n = n << 24;
    (e, n / d, n % d != 0)
}

/// `a / b`, rounded to the nearest (Sapphire_Disputes.md D-27). A zero divisor gives the largest magnitude, with
/// invalid for 0/0 and divide otherwise; the sign is the operands' signs combined.
pub fn div(a: u32, b: u32) -> Quotient {
    match host::div(a, b) {
        Some(q) => Quotient { value: Outcome::of(q), ..Quotient::default() },
        None => div_soft(a, b),
    }
}

/// `div` on integers, for every operand; the reference `host::div` is proven against.
fn div_soft(a: u32, b: u32) -> Quotient {
    let s = sign(a) ^ sign(b);
    if is_zero(b) {
        let invalid = is_zero(a);
        return Quotient { value: Outcome::of(largest(s)), invalid, divide: !invalid };
    }
    if is_zero(a) {
        return Quotient { value: Outcome::of(zero(s)), ..Quotient::default() };
    }
    let (e, q, sticky) = quotient_bits(a, b);
    Quotient { value: nearest(s, e, q, sticky), ..Quotient::default() }
}

/// The square root of `b`'s magnitude, rounded to the nearest (D-27); the root of any zero is +0 (D-29). Whether `b`
/// was negative is the instruction's flag to raise.
pub fn sqrt(b: u32) -> Outcome {
    match host::sqrt(b) {
        Some(r) => Outcome::of(r),
        None => sqrt_soft(b),
    }
}

/// `sqrt` on integers, for every operand; the reference `host::sqrt` is proven against.
fn sqrt_soft(b: u32) -> Outcome {
    if is_zero(b) {
        return Outcome::of(0);
    }
    let mut e = exponent(b) - 127;
    let mut m = significand(b);
    if e & 1 != 0 {
        m <<= 1;
        e -= 1;
    }
    let x = m << 25;
    let r = x.isqrt();
    nearest(0, e / 2 + 127, r, r * r != x)
}

/// `a / sqrt(b)`: `a` over `sqrt(b)` as SQRT gives it, the quotient truncated, the sign `a`'s alone
/// (Sapphire_Disputes.md D-28). A zero `b` gives the largest magnitude with divide, a negative one invalid beside the
/// root of its magnitude.
pub fn rsqrt(a: u32, b: u32) -> Quotient {
    let s = sign(a);
    if is_zero(b) {
        return Quotient { value: Outcome::of(largest(s)), invalid: false, divide: true };
    }
    let invalid = sign(b) == 1;
    if is_zero(a) {
        return Quotient { value: Outcome::of(zero(s)), invalid, divide: false };
    }
    let (e, q, _) = quotient_bits(a, sqrt(b).bits);
    Quotient { value: finish(s, e, q >> 1), invalid, divide: false }
}

/// `acc ± a × b` as MADD.S, MSUB.S, MADDA.S and MSUBA.S compute it (EE Core Instruction Set Manual, MADD.S's Operation
/// section, read as Sapphire_Disputes.md D-30 records): an overflowing product gives the largest magnitude of its
/// signed sense; an accumulator whose exponent field is 255 is kept, with overflow; otherwise the sum.
pub fn fused(acc: u32, a: u32, b: u32, subtract: bool) -> Fused {
    let product = mul(a, b);
    let term = if subtract { product.bits ^ SIGN } else { product.bits };
    let value = if product.overflow {
        Outcome { bits: term, overflow: true, underflow: false }
    } else if exponent(acc) == 255 {
        Outcome { bits: acc, overflow: true, underflow: false }
    } else {
        add(acc, term)
    };
    Fused { value, product: Outcome { bits: term, ..product } }
}

/// The classes of operand where the host's IEEE 754 single-precision arithmetic, rounding to the nearest as Rust
/// leaves it, gives the bits the integer functions give, and nothing else: an exponent field from 1 to 254 in, a
/// result the host keeps normal out (Sapphire_Speed.md §56.3, §56.5). `None` sends an operation to its integer form.
mod host {
    /// An exponent field of 1 to 254: a number the host reads as the console does.
    const fn normal(x: u32) -> bool {
        (x >> 23 & 0xFF).wrapping_sub(1) < 254
    }

    /// DIV.S's quotient, correctly rounded to the nearest as the console's is, where it lands at a field of 2 to 254.
    #[inline]
    pub fn div(a: u32, b: u32) -> Option<u32> {
        if !(normal(a) && normal(b)) {
            return None;
        }
        let q = (f32::from_bits(a) / f32::from_bits(b)).to_bits();
        if (q >> 23 & 0xFF).wrapping_sub(2) < 253 { Some(q) } else { None }
    }

    /// SQRT.S's root of the magnitude, correctly rounded to the nearest; every normal operand's root is normal.
    #[inline]
    pub fn sqrt(b: u32) -> Option<u32> {
        if normal(b) { Some(f32::from_bits(b & super::MAX).sqrt().to_bits()) } else { None }
    }
}

/// The order the compares use: zeros (the would-be denormals among them) equal, otherwise sign and magnitude.
pub fn order(x: u32) -> i64 {
    if is_zero(x) {
        0
    } else if sign(x) == 1 {
        -((x & MAX) as i64)
    } else {
        (x & MAX) as i64
    }
}

/// The order MAX.S and MIN.S use, on the bits as they are: −0 below +0, a would-be denormal above zero (D-31).
pub fn raw_order(x: u32) -> i64 {
    if sign(x) == 1 { -((x & MAX) as i64) - 1 } else { (x & MAX) as i64 }
}

/// The larger of two values, `a` when they are equal (MAX.S).
pub fn max(a: u32, b: u32) -> u32 {
    if raw_order(a) >= raw_order(b) { a } else { b }
}

/// The smaller of two values, `a` when they are equal (MIN.S).
pub fn min(a: u32, b: u32) -> u32 {
    if raw_order(a) <= raw_order(b) { a } else { b }
}

/// A float to a 32-bit integer, truncated towards zero; an exponent field above 9Dh clamps (CVT.W.S's Operation).
pub fn to_word(x: u32) -> u32 {
    let e = exponent(x);
    if e > 0x9D {
        return if sign(x) == 1 { 0x8000_0000 } else { 0x7FFF_FFFF };
    }
    if e < 127 {
        return 0;
    }
    let shift = e - 127 - 23;
    let v = if shift >= 0 { significand(x) << shift } else { significand(x) >> -shift } as u32;
    if sign(x) == 1 { v.wrapping_neg() } else { v }
}

/// A float to a fixed-point word with `fraction` bits below the point (0, 4, 12 or 15), truncated towards zero and
/// clamped as `to_word` clamps (VU User's Manual §2.1.2, §2.2, §2.3: FTOI0 to FTOI15, "Conversion overflow +MAX/-MAX").
pub fn to_fixed(x: u32, fraction: u32) -> u32 {
    if is_zero(x) {
        return 0;
    }
    let e = exponent(x) + fraction as i32;
    if e > 0x9D {
        return if sign(x) == 1 { 0x8000_0000 } else { 0x7FFF_FFFF };
    }
    to_word(x & !(0xFF << 23) | (e as u32) << 23)
}

/// A fixed-point word with `fraction` bits below the point to a float, truncated towards zero (ITOF0 to ITOF15, §2.2).
pub fn from_fixed(w: u32, fraction: u32) -> u32 {
    let f = from_word(w);
    if is_zero(f) { f } else { f - (fraction << 23) }
}

/// A 32-bit integer to a float, truncated towards zero (VU User's Manual §2.2, the conversion's rule).
pub fn from_word(w: u32) -> u32 {
    if w == 0 {
        return 0;
    }
    let s = w >> 31;
    let v = if s == 1 { (w as i32).unsigned_abs() } else { w } as u64;
    let lead = 63 - v.leading_zeros() as i32;
    let m = if lead >= 23 { v >> (lead - 23) } else { v << (23 - lead) };
    finish(s, lead + 127, m).bits
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn the_booth_sum_is_the_exact_product_or_just_below_it() {
        let mut seed = 0x2545_F491_4F6C_DD1Du64;
        for _ in 0..200_000 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let (x, y) = (0x80_0000 | seed & 0x7F_FFFF, 0x80_0000 | seed >> 32 & 0x7F_FFFF);
            let exact = x * y;
            let p = booth(x, y);
            assert!(p <= exact && exact - p <= MOST_LOST, "{x:x} × {y:x}: {p:x} against {exact:x}");
            assert_eq!(p, exact - loss(x, y), "{x:x} × {y:x}: the loss");
        }
    }

    #[test]
    fn a_tie_cannot_reach_the_rounding_of_a_quotient_or_a_root() {
        for (a, b) in [(0x3F80_0000u32, 0x3FFF_FFFFu32), (0x4040_0000, 0x3FC0_0000), (0x3F80_0001, 0x3F80_0003)] {
            let (_, q, sticky) = quotient_bits(a, b);
            assert!(!(q & 1 == 1 && !sticky), "{a:x} / {b:x}");
        }
    }
    /// A xorshift stream, the tests' operands.
    pub(crate) fn stream(mut seed: u64) -> impl FnMut() -> u64 {
        move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        }
    }

    /// An operand of every class the host's paths must take or refuse: zero, would-be denormals, exponent 255, and
    /// exponents at and near either end of the range and at the centre, either sign, any fraction.
    pub(crate) fn operand(r: u64) -> u32 {
        let e = match r >> 40 & 15 {
            0 => 0,
            1 => 255,
            2 => 1 + (r >> 44 & 3) as u32,
            3 => 254 - (r >> 44 & 3) as u32,
            4 => 127 + (r >> 44 & 63) as u32 - 32,
            _ => (r >> 44) as u32 % 256,
        };
        let f = match r >> 52 & 7 {
            0 => 0,
            1 => 0x7F_FFFF,
            _ => r as u32 & 0x7F_FFFF,
        };
        ((r >> 63) as u32 * SIGN) | (e << 23) | f
    }

    #[test]
    fn the_closed_form_product_is_the_booth_sum_for_every_multiplier() {
        let mut next = stream(0x9E37_79B9_7F4A_7C15);
        let xs = [0x80_0000u64, 0x80_0001, 0xFF_FFFF, 0xFF_FFFE, 0x80_0000 | next() & 0x7F_FFFF, 0x80_0001 | next() & 0x7F_FFFE];
        for x in xs {
            for y in 0x80_0000..0x100_0000u64 {
                assert_eq!(product(x, y) >> 23, booth(x, y) >> 23, "{x:x} × {y:x}");
            }
        }
        for _ in 0..2_000_000 {
            let r = next();
            let (x, y) = (0x80_0000 | r & 0x7F_FFFF, 0x80_0000 | r >> 32 & 0x7F_FFFF);
            assert_eq!(product(x, y) >> 23, booth(x, y) >> 23, "{x:x} × {y:x}");
        }
    }

    /// `mul` and `fused` as they were written on `booth`, the references for the whole outcome and its flags.
    fn mul_on_booth(a: u32, b: u32) -> Outcome {
        let s = sign(a) ^ sign(b);
        if is_zero(a) || is_zero(b) {
            return Outcome::of(zero(s));
        }
        let p = booth(significand(a), significand(b));
        let e = exponent(a) + exponent(b) - 127;
        if p >> 47 == 1 { finish(s, e + 1, p >> 24) } else { finish(s, e, p >> 23) }
    }
    fn fused_on_booth(acc: u32, a: u32, b: u32, subtract: bool) -> Fused {
        let product = mul_on_booth(a, b);
        let term = if subtract { product.bits ^ SIGN } else { product.bits };
        let value = if product.overflow {
            Outcome { bits: term, overflow: true, underflow: false }
        } else if exponent(acc) == 255 {
            Outcome { bits: acc, overflow: true, underflow: false }
        } else {
            add(acc, term)
        };
        Fused { value, product: Outcome { bits: term, ..product } }
    }

    #[test]
    fn products_and_multiply_adds_keep_their_outcomes_and_flags_over_every_class() {
        let mut next = stream(0x6A09_E667_F3BC_C908);
        let (mut over, mut under) = (0, 0);
        for i in 0..4_000_000u64 {
            let (a, b, acc) = (operand(next()), operand(next()), operand(next()));
            let m = mul(a, b);
            assert_eq!(m, mul_on_booth(a, b), "{a:08x} × {b:08x}");
            (over, under) = (over + m.overflow as u64, under + m.underflow as u64);
            assert_eq!(fused(acc, a, b, i & 1 == 1), fused_on_booth(acc, a, b, i & 1 == 1), "{acc:08x} ± {a:08x} × {b:08x}");
        }
        assert!(over > 10_000 && under > 10_000, "{over} {under}");
    }

    #[test]
    #[ignore]
    fn the_loss_never_exceeds_its_bound_over_every_low_part_it_reads() {
        for x in 0x80_0000..0x80_0000 + (1u64 << DROPPED_COLUMNS) {
            for y in 0x80_0000..0x80_0000 + READ + 1 {
                assert!(loss(x, y) <= MOST_LOST, "{x:x} × {y:x}");
            }
        }
    }

    #[test]
    #[ignore]
    fn a_billion_products_equal_the_booth_sum() {
        let mut next = stream(0xD1B5_4A32_D192_ED03);
        for _ in 0..1_000_000_000u64 {
            let r = next();
            let (x, y) = (0x80_0000 | r & 0x7F_FFFF, 0x80_0000 | r >> 32 & 0x7F_FFFF);
            assert_eq!(product(x, y) >> 23, booth(x, y) >> 23, "{x:x} × {y:x}");
        }
    }

    /// Every operand pair through `div` against `div_soft`, and a count of those the host took.
    fn divisions(n: u64, seed: u64) -> u64 {
        let mut next = stream(seed);
        let mut taken = 0;
        for _ in 0..n {
            let (a, b) = (operand(next()), operand(next()));
            taken += host::div(a, b).is_some() as u64;
            assert_eq!(div(a, b), div_soft(a, b), "{a:08x} / {b:08x}");
        }
        taken
    }

    #[test]
    fn the_host_quotient_equals_the_integer_one_wherever_it_is_taken() {
        let taken = divisions(4_000_000, 0x2545_F491_4F6C_DD1D);
        assert!(taken > 1_000_000, "{taken}");
        // Quotients whose field is 1, 2, 254 or 255 either side of rounding: the edges of the host's range.
        let mut next = stream(0x5851_F42D_4C95_7F2D);
        for _ in 0..1_000_000 {
            let r = next();
            let b = 0x3F80_0000 | r as u32 & 0x7F_FFFF;
            for e in [1u32, 2, 3, 127, 128, 253, 254, 255] {
                let a = ((r >> 63) as u32 * SIGN) | (e << 23) | ((r >> 32) as u32 & 0x7F_FFFF);
                for d in [b, b + (1 << 23), b - (1 << 23), (b + (126 << 23)) & MAX, (b & !(0xFF << 23)) | (1 << 23)] {
                    assert_eq!(div(a, d), div_soft(a, d), "{a:08x} / {d:08x}");
                }
            }
        }
    }

    #[test]
    #[ignore]
    fn a_billion_quotients_equal_the_integer_ones() {
        divisions(1_000_000_000, 0xBF58_476D_1CE4_E5B9);
    }

    #[test]
    fn the_host_root_equals_the_integer_one_for_every_operand_of_two_binades_and_every_class() {
        for e in [126u32, 127] {
            for f in 0..0x80_0000u32 {
                let x = e << 23 | f;
                assert_eq!(sqrt(x), sqrt_soft(x), "{x:08x}");
                assert_eq!(sqrt(x | SIGN), sqrt_soft(x | SIGN), "{x:08x}");
            }
        }
        let mut next = stream(0x94D0_49BB_1331_11EB);
        for _ in 0..4_000_000 {
            let x = operand(next());
            assert_eq!(sqrt(x), sqrt_soft(x), "{x:08x}");
        }
    }

    #[test]
    #[ignore]
    fn every_root_equals_the_integer_one() {
        for x in 0..=u32::MAX {
            assert_eq!(sqrt(x), sqrt_soft(x), "{x:08x}");
        }
    }

}
