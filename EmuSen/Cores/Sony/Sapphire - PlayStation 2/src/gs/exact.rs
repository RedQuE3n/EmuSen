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

/// ceil(2^(255 + j/16)) for j = 1 to 15, little-endian limbs: the thresholds of floor(16 log2) within an octave,
/// irrational, so a significand of 256 bits is never equal to one (the table's derivation is in Sapphire_Native.md §31.3).
const OCTAVE: [[u64; 4]; 15] = [
    [0x5D42B362AF1EE85A, 0x148A0459E7585151, 0xC5C95B8C2154C1B2, 0x85AAC367CC487B14],
    [0x91E135EE84A3F734, 0x1AA84FFBEBAC349F, 0xFBE4628758A53C90, 0x8B95C1E3EA8BD6E6],
    [0xF1203CAF65BFB9BA, 0x1942B34816FB4F26, 0x0FD6D8E0AE5AC9D8, 0x91C3D373AB11C336],
    [0xD78B65CBEFA7BB70, 0x5E139A1B14FA8178, 0x46AD23182E42F6F6, 0x9837F0518DB8A96F],
    [0x21F977FE7C7FA118, 0x65C15C122133E2A2, 0xA0911F09EBB9FDD1, 0x9EF5326091A111AD],
    [0x2589C98A8290D3F1, 0x1DD170ACE2BCFC17, 0x1CBD7F621710701B, 0xA5FED6A9B15138EA],
    [0xB165F141833A67DB, 0x6BE409407034FDED, 0x4980A8C8F59A2EC4, 0xAD583EEA42A14AC6],
    [0xED17AC8583339916, 0x1D6F60BA893BA84C, 0x597D89B3754ABE9F, 0xB504F333F9DE6484],
    [0x0D9A4BE023ECE032, 0x15B34BBCB0298F41, 0xA8811FB66D0FAF7A, 0xBD08A39F580C36BE],
    [0xC7686006E4E6C093, 0x6B0F939998251A36, 0x3E2AD0C964DD9F37, 0xC5672A115506DADD],
    [0x2BBD398AF35C07A0, 0x6F28610B8C36485A, 0xE235838F95F2C6ED, 0xCE248C151F8480E3],
    [0xB5C13ADA0E77829A, 0x1D733AF522058B16, 0x39A68BB9902D3FDE, 0xD744FCCAD69D6AF4],
    [0x1CB99D3F1FF298A3, 0x224B251B33092002, 0x065895048DD333CA, 0xE0CCDEEC2A94E111],
    [0x17D8D1E8CA31880B, 0xC4FAACE043B7F91C, 0xD02D75B3706E54FA, 0xEAC0C6E7DD24392E],
    [0xBDD80329364AA2A0, 0x6F510308677709F5, 0x7B9D0C7AED980FC3, 0xF5257D152486CC2C],
];

/// floor(16 × log2(w / |den|)) for a positive weight total `w`: the LOD's log2(1/|Q|) in K's fixed point, Q being
/// den / w; `None` when den is 0.
pub fn log2_16(w: u64, den: &Sum) -> Option<i64> {
    if den.is_zero() || w == 0 {
        return None;
    }
    let (n, d) = (Big::from_u128(u128::from(w)), den.mag);
    // e = floor(log2(n / d)), then the significand of n / d to 256 bits, in [2^255, 2^256).
    let s = i64::from(n.bits()) - i64::from(d.bits());
    let at = |e: i64| if e >= 0 { (n, d.shl(e as u32)) } else { (n.shl((-e) as u32), d) };
    let (a, b) = at(s);
    let e = if a.cmp(&b) == std::cmp::Ordering::Less { s - 1 } else { s };
    let (a, b) = at(e - 255);
    let (m, _) = Big::div(&a, &b);
    let top = [m.0[0], m.0[1], m.0[2], m.0[3]];
    let ge = |t: &[u64; 4]| (0..4).rev().map(|i| top[i].cmp(&t[i])).find(|o| o.is_ne()).is_none_or(|o| o.is_gt());
    let j = OCTAVE.iter().take_while(|t| ge(t)).count() as i64;
    Some(16 * (e - i64::from(den.exp)) + j)
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
    fn log2_in_sixteenths_is_floored() {
        for (q, want) in [(1.0f32, 0), (0.5, 16), (2.0, -16), (0.7071068, 7), (0.70710677, 8), (0.25, 32), (3.0, -26)] {
            let d = Sum::of(&[f(q), f(0.0), f(0.0)], [1, 0, 0]);
            assert_eq!(log2_16(1, &d), Some(want), "{q}");
        }
        let d = Sum::of(&[f(0.5), f(0.25), f(1.0)], [1, 1, 2]);
        assert_eq!(log2_16(4, &d), Some((16.0 * (4.0f64 / 2.75).log2()).floor() as i64));
    }
}
