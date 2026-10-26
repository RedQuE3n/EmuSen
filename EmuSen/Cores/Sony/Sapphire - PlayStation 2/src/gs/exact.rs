//! Exact arithmetic for texturing's coordinates (Sapphire_Native.md §31.3): the GS's single-precision S, T and Q taken as
//! the exact numbers they denote, sums of them weighted by a pixel's integer barycentric weights kept whole, the
//! quotient s × USIZE = 2^TW × ΣSw / ΣQw floored to sixteenths of a texel, and floor(16 × log2 of a ratio) for the LOD,
//! all without rounding. Numbers are fixed-size, so nothing allocates. Where the operands provably fit, the same integers
//! are computed in 128 bits; the 768-bit steps are kept for the rest (Sapphire_Native.md §46).

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

    /// log2 of the number from its top 64 bits, within about 2^-52 of the truth; the number is non-zero.
    fn log2(&self) -> f64 {
        let b = self.bits();
        let top = if b > 64 { self.shr(b - 64).0[0] } else { self.0[0] };
        log2_of_top(top, b)
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

/// `Big::log2`'s formula from a number's top 64 bits `top` and its width `b`, the one place both widths take it from.
fn log2_of_top(top: u64, b: u32) -> f64 {
    (top as f64).log2() + f64::from(b.saturating_sub(64))
}

fn bits128(v: u128) -> u32 {
    128 - v.leading_zeros()
}

/// A sum's magnitude: in 128 bits where every term fits (Sapphire_Native.md §46.3), in 768 otherwise; the same integer.
#[derive(Clone, Copy, Debug)]
enum Mag {
    Small(u128),
    Wide(Big),
}

impl Mag {
    fn wide(&self) -> Big {
        match *self {
            Mag::Small(v) => Big::from_u128(v),
            Mag::Wide(b) => b,
        }
    }

    fn is_zero(&self) -> bool {
        match self {
            Mag::Small(v) => *v == 0,
            Mag::Wide(b) => b.is_zero(),
        }
    }

    fn log2(&self) -> f64 {
        match *self {
            Mag::Small(v) => {
                let b = bits128(v);
                log2_of_top(if b > 64 { (v >> (b - 64)) as u64 } else { v as u64 }, b)
            }
            Mag::Wide(b) => b.log2(),
        }
    }
}

/// (-1)^neg × mag × 2^exp, exactly.
#[derive(Clone, Copy, Debug)]
pub struct Sum {
    neg: bool,
    mag: Mag,
    exp: i32,
}

/// A term's magnitude shifted into place is held to 125 bits, so three of them sum within 127 (§46.3).
const TERM: u32 = 125;

impl Sum {
    /// Σ f_i × w_i, exactly: in 128 bits when each term m_i × |w_i| × 2^(e_i - min e) fits in `TERM` bits, by
    /// `of_wide` otherwise.
    pub fn of(f: &[Float; 3], w: [i64; 3]) -> Sum {
        let live = |i: usize| f[i].m != 0 && w[i] != 0;
        let Some(exp) = (0..3).filter(|&i| live(i)).map(|i| f[i].e).min() else { return Sum { neg: false, mag: Mag::Small(0), exp: 0 } };
        let mut acc = 0i128;
        for i in (0..3).filter(|&i| live(i)) {
            let p = u128::from(f[i].m) * u128::from(w[i].unsigned_abs());
            let d = (f[i].e - exp) as u32;
            if bits128(p) + d > TERM {
                return Sum::of_wide(f, w);
            }
            let t = (p << d) as i128;
            acc += if f[i].neg != (w[i] < 0) { -t } else { t };
        }
        Sum { neg: acc < 0, mag: Mag::Small(acc.unsigned_abs()), exp }
    }

    /// Σ f_i × w_i in 768-bit integers, for any exponents and weights.
    fn of_wide(f: &[Float; 3], w: [i64; 3]) -> Sum {
        let live = |i: usize| f[i].m != 0 && w[i] != 0;
        let Some(exp) = (0..3).filter(|&i| live(i)).map(|i| f[i].e).min() else { return Sum { neg: false, mag: Mag::Wide(Big::ZERO), exp: 0 } };
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
            std::cmp::Ordering::Less => Sum { neg: true, mag: Mag::Wide(neg.sub(&pos)), exp },
            _ => Sum { neg: false, mag: Mag::Wide(pos.sub(&neg)), exp },
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
    if let (Mag::Small(n), Mag::Small(d)) = (num.mag, den.mag)
        && let Some((q, rest)) = divide_small(n, d, k)
    {
        return place(num.neg != den.neg, bits128(q) <= 40, q as u64, rest);
    }
    let (n, d) = (num.mag.wide(), den.mag.wide());
    let (a, b) = if k >= 0 { (n.shl(k as u32), d) } else { (n, d.shl((-k) as u32)) };
    let (q, r) = Big::div(&a, &b);
    place(num.neg != den.neg, q.bits() <= 40, q.0[0], !r.is_zero())
}

/// floor(2^k × n / d) and whether a remainder is left, for n and d of at most 127 bits, d non-zero, when 2^k × n or
/// 2^-k × d fits in 128 bits or 2^-k × d exceeds n (§46.4); `None` otherwise.
fn divide_small(n: u128, d: u128, k: i32) -> Option<(u128, bool)> {
    if k >= 0 {
        if bits128(n) + k as u32 > 128 {
            return None;
        }
        let a = n << k;
        return Some((a / d, a % d != 0));
    }
    let s = k.unsigned_abs();
    if bits128(d) + s > bits128(n) {
        // d × 2^s ≥ 2^(bits(n)) > n: the quotient is 0 and n, non-zero, remains.
        return Some((0, true));
    }
    let b = d << s;
    Some((n / b, n % b != 0))
}

/// The coordinate from the quotient's magnitude: its width within 40 bits, its low 64 bits, a remainder, and the sign.
fn place(negative: bool, small: bool, low: u64, rest: bool) -> i64 {
    if !negative {
        if small { low as i64 } else { FAR + (low & LOW) as i64 }
    } else {
        // Rounded towards minus infinity: -(q + 1) when there is a remainder.
        let v = low.wrapping_neg().wrapping_sub(u64::from(rest));
        if small && low + u64::from(rest) <= NEAR { v as i64 } else { -FAR + (v & LOW) as i64 }
    }
}

/// How far from an integer the double-precision estimate of `log2_scaled` must lie to be taken.
const CLEAR: f64 = 1.0 / (1u64 << 24) as f64;

/// floor(2^bits × log2(w / |den|)) for a positive weight total `w` and `bits` up to 7: the LOD's log2(1/|Q|), Q being
/// den / w; `None` when den is 0. The ratio's significand is taken to 256 bits and squared once a bit, each square's
/// excess over 2 giving the next bit (Sapphire_Native.md §31.3); every step truncates below 2^-250 of the value, so the
/// result is the exact floor unless the ratio's 2^bits-th power lies that near a power of two.
pub fn log2_scaled(w: u64, den: &Sum, bits: u32) -> Option<i64> {
    if den.is_zero() || w == 0 {
        return None;
    }
    // The same floor from double precision where its estimate lies clear of an integer, the estimate's error being far
    // below CLEAR; the exact steps otherwise (Sapphire_Native.md §35.7).
    let est = ((w as f64).log2() - den.mag.log2() - f64::from(den.exp)) * f64::from(1u32 << bits);
    let floor = est.floor();
    if est - floor > CLEAR && floor + 1.0 - est > CLEAR {
        return Some(floor as i64);
    }
    log2_exact(w, den, bits)
}

/// `log2_scaled` by the exact steps alone.
fn log2_exact(w: u64, den: &Sum, bits: u32) -> Option<i64> {
    let (n, d) = (Big::from_u128(u128::from(w)), den.mag.wide());
    // e = floor(log2(n / d)), then the significand m = n / d / 2^e as floor(m × 2^255), in [2^255, 2^256).
    let s = i64::from(n.bits()) - i64::from(d.bits());
    let at = |e: i64| if e >= 0 { (n, d.shl(e as u32)) } else { (n.shl((-e) as u32), d) };
    let (a, b) = at(s);
    let e = if a.cmp(&b) == std::cmp::Ordering::Less { s - 1 } else { s };
    // A ratio that is a power of two has the significand 1, whose squares give no fraction bit: the steps' own result.
    let (p, q) = at(e);
    if p.cmp(&q) == std::cmp::Ordering::Equal {
        return Some((e - i64::from(den.exp)) << bits);
    }
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

    /// The double-precision shortcut gives the exact floor: random weights and sums of one to three terms, each depth.
    #[test]
    fn the_shortcut_equals_the_exact_steps() {
        let mut x = 0x9E37_79B9_7F4A_7C15u64;
        let mut next = || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        for _ in 0..20_000 {
            let fl = |v: u64| f((v % 4096) as f32 / 64.0 + 0.001 * ((v >> 20) % 1000) as f32);
            let q = [fl(next()), fl(next()), fl(next())];
            let wts = [(next() % 4096) as i64, (next() % 4096) as i64, (next() % 4096) as i64];
            let d = Sum::of(&q, wts);
            let w = next() % 100_000 + 1;
            for bits in [4, 5, 6, 7] {
                assert_eq!(log2_scaled(w, &d, bits), log2_exact(w, &d, bits), "{w} {q:?} {wts:?} {bits}");
            }
        }
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

    /// xorshift64, so every run draws the same cases.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }

        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }
    }

    /// A single-precision value of a random class: zero, the exponent field's ends (1, 254, 255), or within `spread` of
    /// `centre`; the significand all ones, its hidden bit alone, or random; either sign.
    fn float(r: &mut Rng, centre: i64, spread: i64) -> Float {
        let m = match r.below(8) {
            0 => 0x7F_FFFF,
            1 => 0,
            _ => r.next() as u32 & 0x7F_FFFF,
        };
        let exp = match r.below(16) {
            0 => 0,
            1 => 1,
            2 => 254,
            3 => 255,
            _ => (centre + r.below(2 * spread as u64 + 1) as i64 - spread).clamp(0, 255),
        } as u32;
        Float::from_bits((r.next() as u32 & 1) << 31 | exp << 23 | m)
    }

    /// A weight of a random class: 0, ±1, a few bits, a triangle's edge function (to 2^36), a line's continued past
    /// its end (negative), anything to 2^62, or i64's ends.
    fn weight(r: &mut Rng) -> i64 {
        let sign = if r.next() & 1 == 0 { 1 } else { -1 };
        match r.below(8) {
            0 => 0,
            1 => sign,
            2 => sign * r.below(1 << 12) as i64,
            3 | 4 => r.below(1 << 36) as i64,
            5 => -(r.below(1 << 36) as i64),
            6 => sign * r.below(1 << 62) as i64,
            _ => [i64::MAX, i64::MIN, i64::MIN + 1][r.below(3) as usize],
        }
    }

    fn same_sum(a: &Sum, b: &Sum) -> bool {
        a.neg == b.neg && a.mag.wide() == b.mag.wide() && (a.mag.is_zero() || a.exp == b.exp)
    }

    /// `cases` random primitives' S and Q across every class above, the 128-bit path against the 768-bit one: the sums,
    /// the quotient at a random scale and the LOD's logarithm at each depth, bit for bit. Returns how many quotients the
    /// 128-bit path took.
    fn fast_equals_wide(seed: u64, cases: u64) -> u64 {
        let mut r = Rng(seed);
        let mut fast = 0;
        for _ in 0..cases {
            let (centre, spread) = (r.below(256) as i64, [0, 2, 8, 24, 38, 60, 128, 255][r.below(8) as usize]);
            let s: [Float; 3] = std::array::from_fn(|_| float(&mut r, centre, spread));
            let (qc, qs) = if r.below(4) == 0 { (r.below(256) as i64, spread) } else { (centre, spread) };
            let q: [Float; 3] = std::array::from_fn(|_| float(&mut r, qc, qs));
            let w: [i64; 3] = std::array::from_fn(|_| weight(&mut r));
            let scale = match r.below(8) {
                0 => r.below(600) as i32 - 300,
                _ => r.below(48) as i32 - 8,
            };
            let (sn, sq) = (Sum::of(&s, w), Sum::of(&q, w));
            let (wn, wq) = (Sum::of_wide(&s, w), Sum::of_wide(&q, w));
            assert!(same_sum(&sn, &wn) && same_sum(&sq, &wq), "{s:?} {q:?} {w:?}");
            let got = quotient(&sn, &sq, scale);
            assert_eq!(got, quotient(&wn, &wq, scale), "{s:?} {q:?} {w:?} {scale}");
            if let (Mag::Small(n), Mag::Small(d)) = (sn.mag, sq.mag)
                && n != 0
                && d != 0
                && divide_small(n, d, scale + sn.exp - sq.exp).is_some()
            {
                fast += 1;
            }
            let total = r.below(1 << 36) + 1;
            let bits = r.below(4) as u32 + 4;
            assert_eq!(log2_scaled(total, &sq, bits), log2_scaled(total, &wq, bits), "{q:?} {w:?} {total} {bits}");
        }
        fast
    }

    #[test]
    fn the_128_bit_path_equals_the_768_bit_one() {
        let fast = fast_equals_wide(0x2545_F491_4F6C_DD1D, 400_000);
        assert!(fast > 100_000 && fast < 400_000, "{fast} quotients took the 128-bit path");
    }

    /// The same over 50 million cases: `cargo test --release -- --ignored many_millions` (Sapphire_Native.md §46.5).
    #[test]
    #[ignore]
    fn many_millions_of_cases_equal() {
        let fast = fast_equals_wide(0xD1B5_4A32_D192_ED03, 50_000_000);
        println!("50,000,000 cases, {fast} quotients by the 128-bit path");
    }

    fn sums(n: u128, d: u128, neg: [bool; 2]) -> ([Sum; 2], [Sum; 2]) {
        let at = |v: u128, neg: bool, mag: fn(u128) -> Mag| Sum { neg, mag: mag(v), exp: 0 };
        ([at(n, neg[0], Mag::Small), at(d, neg[1], Mag::Small)], [at(n, neg[0], |v| Mag::Wide(Big::from_u128(v))), at(d, neg[1], |v| Mag::Wide(Big::from_u128(v)))])
    }

    /// Every numerator and denominator below 2^7, every scale from -12 to 12 and every pair of signs: the 128-bit
    /// division against the 768-bit one, exhaustively.
    #[test]
    fn small_quotients_equal_exhaustively() {
        for n in 1..128u128 {
            for d in 1..128u128 {
                for k in -12..=12 {
                    for neg in [[false, false], [false, true], [true, false], [true, true]] {
                        let ([a, b], [c, e]) = sums(n, d, neg);
                        assert_eq!(quotient(&a, &b, k), quotient(&c, &e, k), "{n} {d} {k} {neg:?}");
                    }
                }
            }
        }
    }

    /// Integers of every width to 127 bits and scales clustered on the edges of `divide_small`'s cases: 2^k × n at 128
    /// bits and one past, d × 2^-k at n's width and one past, and the quotient at 2^40 either side.
    #[test]
    fn quotients_on_the_edges_of_the_fast_cases_equal() {
        let mut r = Rng(0x9E37_79B9_7F4A_7C15);
        let wide = |r: &mut Rng, b: u32| ((u128::from(r.next()) << 64 | u128::from(r.next())) >> (128 - b)) | 1 << (b - 1);
        for _ in 0..300_000 {
            let (bn, bd) = (r.below(127) as u32 + 1, r.below(127) as u32 + 1);
            let (n, d) = (wide(&mut r, bn), wide(&mut r, bd));
            let k = match r.below(4) {
                0 => 128 - bn as i32 + r.below(3) as i32 - 1,
                1 => bn as i32 - bd as i32 - (r.below(3) as i32 - 1),
                2 => 40 + bd as i32 - bn as i32 + r.below(3) as i32 - 1,
                _ => r.below(400) as i32 - 200,
            };
            let neg = [r.next() & 1 == 0, r.next() & 1 == 0];
            let ([a, b], [c, e]) = sums(n, d, neg);
            assert_eq!(quotient(&a, &b, k), quotient(&c, &e, k), "{n} {d} {k} {neg:?}");
        }
        // Exact divisions whose quotient is 2^40 - 1, 2^40 and 2^40 + 1, without and with a remainder, either sign.
        for q in [(1u128 << 40) - 1, 1 << 40, (1 << 40) + 1] {
            for rest in [0u128, 1] {
                for neg in [[false, false], [true, false]] {
                    let d = 0xFFFF_FFFFu128;
                    let ([a, b], [c, e]) = sums(q * d + rest, d, neg);
                    assert_eq!(quotient(&a, &b, 0), quotient(&c, &e, 0), "{q} {rest} {neg:?}");
                }
            }
        }
    }

    /// A term at `TERM`'s 125 bits stays in 128 bits and one bit more goes to 768; both sums are the same number.
    #[test]
    fn a_term_of_125_bits_is_the_widest_kept_narrow() {
        let full = |exp: u32| Float::from_bits(exp << 23 | 0x7F_FFFF);
        for (gap, narrow) in [(38, true), (39, false)] {
            let f = [full(100), full(100 + gap), full(100)];
            let w = [1, i64::MAX, -1];
            let (a, b) = (Sum::of(&f, w), Sum::of_wide(&f, w));
            assert_eq!(matches!(a.mag, Mag::Small(_)), narrow, "{gap}");
            assert!(same_sum(&a, &b), "{gap}");
        }
    }
}
