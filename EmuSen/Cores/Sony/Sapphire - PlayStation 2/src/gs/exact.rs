//! Exact arithmetic for texturing's coordinates (Sapphire_Native.md §31.3): the GS's single-precision S, T and Q taken as
//! the exact numbers they denote, sums of them weighted by a pixel's integer barycentric weights kept whole, the
//! quotient s × USIZE = 2^TW × ΣSw / ΣQw floored to sixteenths of a texel, and floor(16 × log2 of a ratio) for the LOD,
//! all without rounding. Numbers are fixed-size, so nothing allocates.

/// Limbs of 64 bits: 768 bits hold every product and shift the coordinates need (a weighted sum of at most 342 bits
/// shifted by at most 268).
const LIMBS: usize = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Big([u64; LIMBS]);

impl Big {
    const ZERO: Big = Big([0; LIMBS]);

    fn from_u128(v: u128) -> Big {
        let mut b = Big::ZERO;
        b.0[0] = v as u64;
        b.0[1] = (v >> 64) as u64;
        b
    }

    fn is_zero(&self) -> bool {
        self.0.iter().all(|&l| l == 0)
    }

    fn bits(&self) -> u32 {
        for i in (0..LIMBS).rev() {
            if self.0[i] != 0 {
                return 64 * i as u32 + 64 - self.0[i].leading_zeros();
            }
        }
        0
    }

    fn shl(&self, n: u32) -> Big {
        debug_assert!(self.is_zero() || self.bits() + n <= 64 * LIMBS as u32);
        let (limbs, bits) = ((n / 64) as usize, n % 64);
        let mut r = Big::ZERO;
        for i in (limbs..LIMBS).rev() {
            let lo = self.0[i - limbs];
            let below = if bits != 0 && i > limbs { self.0[i - limbs - 1] >> (64 - bits) } else { 0 };
            r.0[i] = if bits == 0 { lo } else { lo << bits | below };
        }
        r
    }

    fn shr(&self, n: u32) -> Big {
        let (limbs, bits) = ((n / 64) as usize, n % 64);
        let mut r = Big::ZERO;
        for i in 0..LIMBS - limbs {
            let lo = self.0[i + limbs] >> bits;
            let above = if bits != 0 && i + limbs + 1 < LIMBS { self.0[i + limbs + 1] << (64 - bits) } else { 0 };
            r.0[i] = lo | above;
        }
        r
    }

    /// The square of a number of at most 384 bits.
    fn square(&self) -> Big {
        let n = LIMBS / 2;
        let mut r = Big::ZERO;
        for i in 0..n {
            let mut carry = 0u128;
            for j in 0..n {
                let t = u128::from(self.0[i]) * u128::from(self.0[j]) + u128::from(r.0[i + j]) + carry;
                r.0[i + j] = t as u64;
                carry = t >> 64;
            }
            r.0[i + n] = carry as u64;
        }
        r
    }

    fn shr1(&mut self) {
        for i in 0..LIMBS {
            let above = if i + 1 < LIMBS { self.0[i + 1] << 63 } else { 0 };
            self.0[i] = self.0[i] >> 1 | above;
        }
    }

    fn add(&self, o: &Big) -> Big {
        let mut r = Big::ZERO;
        let mut carry = false;
        for i in 0..LIMBS {
            let (s, c1) = self.0[i].overflowing_add(o.0[i]);
            let (s, c2) = s.overflowing_add(u64::from(carry));
            r.0[i] = s;
            carry = c1 || c2;
        }
        r
    }

    /// `self - o`, `self` being the larger.
    fn sub(&self, o: &Big) -> Big {
        let mut r = Big::ZERO;
        let mut borrow = false;
        for i in 0..LIMBS {
            let (s, b1) = self.0[i].overflowing_sub(o.0[i]);
            let (s, b2) = s.overflowing_sub(u64::from(borrow));
            r.0[i] = s;
            borrow = b1 || b2;
        }
        r
    }

    fn cmp(&self, o: &Big) -> std::cmp::Ordering {
        for i in (0..LIMBS).rev() {
            match self.0[i].cmp(&o.0[i]) {
                std::cmp::Ordering::Equal => {}
                other => return other,
            }
        }
        std::cmp::Ordering::Equal
    }

    /// The quotient and remainder of `a / b`, `b` non-zero, by shift and subtract.
    fn div(a: &Big, b: &Big) -> (Big, Big) {
        let (ab, bb) = (a.bits(), b.bits());
        let mut q = Big::ZERO;
        if ab < bb {
            return (q, *a);
        }
        let mut r = *a;
        let mut s = ab - bb;
        let mut d = b.shl(s);
        loop {
            if r.cmp(&d) != std::cmp::Ordering::Less {
                r = r.sub(&d);
                q.0[(s / 64) as usize] |= 1 << (s % 64);
            }
            if s == 0 {
                return (q, r);
            }
            d.shr1();
            s -= 1;
        }
    }
}

/// A single-precision value as the GS takes it [A, Sapphire_Disputes.md D-247]: sign, the 24-bit significand with its
/// hidden bit, and the exponent of its last bit; a zero exponent field is zero, and the largest is an ordinary exponent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Float {
    pub neg: bool,
    pub m: u32,
    pub e: i32,
}

impl Float {
    pub const fn from_bits(b: u32) -> Float {
        let exp = (b >> 23 & 0xFF) as i32;
        if exp == 0 {
            return Float { neg: false, m: 0, e: 0 };
        }
        Float { neg: b >> 31 != 0, m: b & 0x7F_FFFF | 0x80_0000, e: exp - 150 }
    }
}

/// (-1)^neg × mag × 2^exp, exactly.
#[derive(Clone, Copy, Debug)]
pub struct Sum {
    neg: bool,
    mag: Big,
    exp: i32,
}

impl Sum {
    /// Σ f_i × w_i, exactly.
    pub fn of(f: &[Float; 3], w: [i64; 3]) -> Sum {
        let live = |i: usize| f[i].m != 0 && w[i] != 0;
        let Some(exp) = (0..3).filter(|&i| live(i)).map(|i| f[i].e).min() else { return Sum { neg: false, mag: Big::ZERO, exp: 0 } };
        let (mut pos, mut neg) = (Big::ZERO, Big::ZERO);
        for i in (0..3).filter(|&i| live(i)) {
            let t = Big::from_u128(u128::from(f[i].m) * u128::from(w[i].unsigned_abs())).shl((f[i].e - exp) as u32);
            if f[i].neg != (w[i] < 0) {
                neg = neg.add(&t);
            } else {
                pos = pos.add(&t);
            }
        }
        match pos.cmp(&neg) {
            std::cmp::Ordering::Less => Sum { neg: true, mag: neg.sub(&pos), exp },
            _ => Sum { neg: false, mag: pos.sub(&neg), exp },
        }
    }

    pub fn is_zero(&self) -> bool {
        self.mag.is_zero()
    }
}

/// A coordinate beyond ±2^40 sixteenths is kept as a stand-in that lies beyond every clamp's bounds on the same side and
/// agrees with it in its low 24 bits, which is all that wrapping by any texture size or region mask reads.
const NEAR: u64 = 1 << 40;
const FAR: i64 = 1 << 41;
const LOW: u64 = (1 << 24) - 1;

/// floor(2^scale × num / den) as a coordinate (`NEAR`'s stand-in beyond ±2^40); 0 when either is 0 [A, D-247].
pub fn quotient(num: &Sum, den: &Sum, scale: i32) -> i64 {
    if num.is_zero() || den.is_zero() {
        return 0;
    }
    let k = scale + num.exp - den.exp;
    let (a, b) = if k >= 0 { (num.mag.shl(k as u32), den.mag) } else { (num.mag, den.mag.shl((-k) as u32)) };
    let (q, r) = Big::div(&a, &b);
    let small = q.bits() <= 40;
    let low = q.0[0];
    if num.neg == den.neg {
        if small { low as i64 } else { FAR + (low & LOW) as i64 }
    } else {
        // Rounded towards minus infinity: -(q + 1) when there is a remainder.
        let v = low.wrapping_neg().wrapping_sub(u64::from(!r.is_zero()));
        if small && low + u64::from(!r.is_zero()) <= NEAR { v as i64 } else { -FAR + (v & LOW) as i64 }
    }
}

/// floor(2^bits × log2(w / |den|)) for a positive weight total `w` and `bits` up to 7: the LOD's log2(1/|Q|), Q being
/// den / w; `None` when den is 0. The ratio's significand is taken to 256 bits and squared once a bit, each square's
/// excess over 2 giving the next bit (Sapphire_Native.md §31.3); every step truncates below 2^-250 of the value, so the
/// result is the exact floor unless the ratio's 2^bits-th power lies that near a power of two.
pub fn log2_scaled(w: u64, den: &Sum, bits: u32) -> Option<i64> {
    if den.is_zero() || w == 0 {
        return None;
    }
    let (n, d) = (Big::from_u128(u128::from(w)), den.mag);
    // e = floor(log2(n / d)), then the significand m = n / d / 2^e as floor(m × 2^255), in [2^255, 2^256).
    let s = i64::from(n.bits()) - i64::from(d.bits());
    let at = |e: i64| if e >= 0 { (n, d.shl(e as u32)) } else { (n.shl((-e) as u32), d) };
    let (a, b) = at(s);
    let e = if a.cmp(&b) == std::cmp::Ordering::Less { s - 1 } else { s };
    let (a, b) = at(e - 255);
    let (mut m, _) = Big::div(&a, &b);
    let mut frac = 0i64;
    for _ in 0..bits {
        m = m.square().shr(255);
        frac <<= 1;
        if m.bits() > 256 {
            frac |= 1;
            m.shr1();
        }
    }
    Some(((e - i64::from(den.exp)) << bits) + frac)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(v: f32) -> Float {
        Float::from_bits(v.to_bits())
    }

    #[test]
    fn quotients_floor_exactly_and_keep_the_low_bits_far_out() {
        let one = [f(1.0), f(1.0), f(1.0)];
        let n = Sum::of(&[f(0.5), f(0.25), f(-0.75)], [3, 5, 1]);
        let d = Sum::of(&one, [3, 5, 1]);
        // (1.5 + 1.25 - 0.75) / 9 × 2^10 = 227.55...
        assert_eq!(quotient(&n, &d, 10), 227);
        let neg = Sum::of(&[f(-0.5), f(0.0), f(0.0)], [1, 0, 0]);
        assert_eq!(quotient(&neg, &Sum::of(&[f(3.0), f(0.0), f(0.0)], [1, 0, 0]), 4), -3);
        let big = Sum::of(&[f(3.0e20), f(0.0), f(0.0)], [1, 0, 0]);
        let q = quotient(&big, &Sum::of(&one, [1, 0, 0]), 4);
        assert!(q > 1 << 40);
        let exact = (3.0e20f32 as f64 as u128) * 16;
        assert_eq!(q as u64 & LOW, exact as u64 & LOW);
    }

    #[test]
    fn log2_is_floored_at_every_scale() {
        // The single-precision neighbours of 1/sqrt(2) either side, 3F3504F4h above and 3F3504F3h below.
        for (q, want) in [(1.0f32, 0), (0.5, 16), (2.0, -16), (f32::from_bits(0x3F35_04F4), 7), (f32::from_bits(0x3F35_04F3), 8), (0.25, 32), (3.0, -26)] {
            let d = Sum::of(&[f(q), f(0.0), f(0.0)], [1, 0, 0]);
            assert_eq!(log2_scaled(1, &d, 4), Some(want), "{q}");
        }
        for bits in 0..8 {
            for (w, q) in [(4u64, [0.5f32, 0.25, 1.0]), (7, [0.3, 0.3, 0.3]), (1000, [0.001, 2.0, 0.01]), (3, [100.0, 0.5, 7.0])] {
                let d = Sum::of(&q.map(f), [1, 1, 2]);
                let exact = (q[0] as f64 + q[1] as f64 + 2.0 * q[2] as f64) / w as f64;
                let want = (-(exact.log2()) * f64::from(1 << bits)).floor() as i64;
                assert_eq!(log2_scaled(w, &d, bits), Some(want), "{w} {q:?} {bits}");
            }
        }
    }
}
