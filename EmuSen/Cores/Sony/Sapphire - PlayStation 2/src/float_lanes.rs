//! The console's adder and multiplier four lanes at a time in the host's vector registers, on integers, with the VU's MAC
//! flags: `float::add`, `float::mul` and `float::fused` for a whole vector, bit for bit. A measurement kernel behind the
//! `lanes` feature, off by default and used by nothing in the machine; Sapphire_Redesign.md, R0 results, R0.4 records the
//! argument for each step, the grading and the timing.
//!
//! x86-64 with AVX2 only (`available`): the per-lane shifts are AVX2's. Every function here is `unsafe` for that reason
//! alone and touches no memory but its arguments.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// Four lanes' results and their MAC flags in the VU's layout (lane 0, x, at bit 3 of each group: Z, S, U, O).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Vector {
    pub bits: [u32; 4],
    pub mac: u32,
}

/// A multiply-add's four lanes: the sum with its MAC flags, and the products' as added (negated for a subtraction).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FusedVector {
    pub value: Vector,
    pub product: Vector,
}

/// Whether the host runs this kernel.
pub fn available() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        std::arch::is_x86_feature_detected!("avx2")
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}

/// The four flag groups' answers as the MAC word; each mask an all-ones or zero lane.
#[cfg(target_arch = "x86_64")]
#[inline]
#[target_feature(enable = "avx2")]
fn mac_of(z: __m128i, s: __m128i, u: __m128i, o: __m128i) -> u32 {
    let bytes = _mm_packs_epi16(_mm_packs_epi32(z, s), _mm_packs_epi32(u, o));
    let reversed = _mm_shuffle_epi8(bytes, _mm_setr_epi8(3, 2, 1, 0, 7, 6, 5, 4, 11, 10, 9, 8, 15, 14, 13, 12));
    _mm_movemask_epi8(reversed) as u32
}

/// A result's Z and S masks: an exponent field of 0, the sign bit.
#[cfg(target_arch = "x86_64")]
#[inline]
#[target_feature(enable = "avx2")]
fn zs(bits: __m128i) -> (__m128i, __m128i) {
    let z = _mm_cmpeq_epi32(_mm_and_si128(bits, _mm_set1_epi32(0x7F80_0000)), _mm_setzero_si128());
    (z, _mm_srai_epi32(bits, 31))
}

/// `float::finish` for four lanes: the sign (bit 31 alone), the biased exponent (signed), a 24-bit significand.
/// Returns the bits and the overflow and underflow masks.
#[cfg(target_arch = "x86_64")]
#[inline]
#[target_feature(enable = "avx2")]
fn finish(s: __m128i, e: __m128i, m: __m128i) -> (__m128i, __m128i, __m128i) {
    let over = _mm_cmpgt_epi32(e, _mm_set1_epi32(255));
    let under = _mm_cmpgt_epi32(_mm_set1_epi32(1), e);
    let packed = _mm_or_si128(_mm_or_si128(s, _mm_slli_epi32(e, 23)), _mm_and_si128(m, _mm_set1_epi32(0x7F_FFFF)));
    let largest = _mm_or_si128(s, _mm_set1_epi32(0x7FFF_FFFF));
    let bits = _mm_blendv_epi8(_mm_blendv_epi8(packed, largest, over), s, under);
    (bits, over, under)
}

/// The lanes of `old` the field mask (`mask`, lane 0 at bit 3) leaves alone, and the MAC bits it clears.
#[cfg(target_arch = "x86_64")]
#[inline]
#[target_feature(enable = "avx2")]
fn masked(bits: __m128i, mac: u32, old: __m128i, mask: u32) -> Vector {
    let on = _mm_cmpgt_epi32(_mm_and_si128(_mm_set1_epi32(mask as i32), _mm_setr_epi32(8, 4, 2, 1)), _mm_setzero_si128());
    let out = _mm_blendv_epi8(old, bits, on);
    let mut v = [0u32; 4];
    // SAFETY: four u32 are 16 bytes, written unaligned.
    unsafe { _mm_storeu_si128(v.as_mut_ptr().cast(), out) };
    Vector { bits: v, mac: mac & (mask & 0xF) * 0x1111 }
}

#[cfg(target_arch = "x86_64")]
#[inline]
#[target_feature(enable = "avx2")]
fn load(v: &[u32; 4]) -> __m128i {
    // SAFETY: four u32 are 16 bytes, read unaligned.
    unsafe { _mm_loadu_si128(v.as_ptr().cast()) }
}

/// `float::add` in four lanes: the bits, overflow and underflow masks.
#[cfg(target_arch = "x86_64")]
#[inline]
#[target_feature(enable = "avx2")]
fn add_raw(a: __m128i, b: __m128i) -> (__m128i, __m128i, __m128i) {
    let field = _mm_set1_epi32(0xFF);
    let ea = _mm_and_si128(_mm_srli_epi32(a, 23), field);
    let eb = _mm_and_si128(_mm_srli_epi32(b, 23), field);
    let zero = _mm_setzero_si128();
    let za = _mm_cmpeq_epi32(ea, zero);
    let zb = _mm_cmpeq_epi32(eb, zero);
    let max = _mm_set1_epi32(0x7FFF_FFFF);
    // The larger magnitude: `a` unless `b`'s is strictly greater (both below 2^31, so a signed compare).
    let b_big = _mm_cmpgt_epi32(_mm_and_si128(b, max), _mm_and_si128(a, max));
    let big = _mm_blendv_epi8(a, b, b_big);
    let small = _mm_blendv_epi8(b, a, b_big);
    let e_big = _mm_blendv_epi8(ea, eb, b_big);
    let d = _mm_sub_epi32(e_big, _mm_blendv_epi8(eb, ea, b_big));
    let hidden = _mm_set1_epi32(0x80_0000);
    let frac = _mm_set1_epi32(0x7F_FFFF);
    let wide = _mm_slli_epi32(_mm_or_si128(hidden, _mm_and_si128(big, frac)), 1);
    // A count of 32 or more gives 0, so the shift needs no test of `d < 25`: a 25-bit value is 0 from 25 on.
    let aligned = _mm_srlv_epi32(_mm_slli_epi32(_mm_or_si128(hidden, _mm_and_si128(small, frac)), 1), d);
    let opposite = _mm_srai_epi32(_mm_xor_si128(big, small), 31);
    let m = _mm_blendv_epi8(_mm_add_epi32(wide, aligned), _mm_sub_epi32(wide, aligned), opposite);
    // The leading bit's place, exactly: with the bit below the leading one cleared, the conversion cannot round up a binade.
    let lone = _mm_andnot_si128(_mm_srli_epi32(m, 1), m);
    let lead = _mm_sub_epi32(_mm_srli_epi32(_mm_castps_si128(_mm_cvtepi32_ps(lone)), 23), _mm_set1_epi32(127));
    let k23 = _mm_set1_epi32(23);
    let norm = _mm_or_si128(_mm_srlv_epi32(m, _mm_sub_epi32(lead, k23)), _mm_sllv_epi32(m, _mm_sub_epi32(k23, lead)));
    let e = _mm_add_epi32(e_big, _mm_sub_epi32(lead, _mm_set1_epi32(24)));
    let s = _mm_and_si128(big, _mm_set1_epi32(i32::MIN));
    let (bits, over, under) = finish(s, e, norm);
    let cancelled = _mm_cmpeq_epi32(m, zero);
    let none = _mm_or_si128(_mm_or_si128(za, zb), cancelled);
    let bits = _mm_andnot_si128(cancelled, bits);
    // A zero operand gives the other; two zeros give the signs' conjunction as -0 or +0.
    let both = _mm_and_si128(za, zb);
    let bits = _mm_blendv_epi8(bits, b, za);
    let bits = _mm_blendv_epi8(bits, a, _mm_andnot_si128(za, zb));
    let bits = _mm_blendv_epi8(bits, _mm_and_si128(_mm_and_si128(a, b), _mm_set1_epi32(i32::MIN)), both);
    (bits, _mm_andnot_si128(none, over), _mm_andnot_si128(none, under))
}

/// `float::mul` in four lanes, `b` the recoded operand: the bits, overflow and underflow masks.
#[cfg(target_arch = "x86_64")]
#[inline]
#[target_feature(enable = "avx2")]
fn mul_raw(a: __m128i, b: __m128i) -> (__m128i, __m128i, __m128i) {
    let field = _mm_set1_epi32(0xFF);
    let ea = _mm_and_si128(_mm_srli_epi32(a, 23), field);
    let eb = _mm_and_si128(_mm_srli_epi32(b, 23), field);
    let zero = _mm_setzero_si128();
    let none = _mm_or_si128(_mm_cmpeq_epi32(ea, zero), _mm_cmpeq_epi32(eb, zero));
    let s = _mm_and_si128(_mm_xor_si128(a, b), _mm_set1_epi32(i32::MIN));
    let hidden = _mm_set1_epi32(0x80_0000);
    let frac = _mm_set1_epi32(0x7F_FFFF);
    let x = _mm_or_si128(hidden, _mm_and_si128(a, frac));
    let y = _mm_or_si128(hidden, _mm_and_si128(b, frac));
    // The Booth sum's loss (float::product): eight two-bit answers by y's lowest two bits and x's lowest.
    let index = _mm_slli_epi32(_mm_or_si128(_mm_slli_epi32(_mm_and_si128(y, _mm_set1_epi32(3)), 1), _mm_and_si128(x, _mm_set1_epi32(1))), 2);
    let loss = _mm_and_si128(_mm_srlv_epi32(_mm_set1_epi32(0x1222_1000), index), _mm_set1_epi32(3));
    let p = _mm256_sub_epi64(_mm256_mul_epu32(_mm256_cvtepu32_epi64(x), _mm256_cvtepu32_epi64(y)), _mm256_cvtepu32_epi64(loss));
    let top = _mm256_srli_epi64(p, 47);
    let m = _mm256_srlv_epi64(p, _mm256_add_epi64(top, _mm256_set1_epi64x(23)));
    let pick = _mm256_setr_epi32(0, 2, 4, 6, 0, 2, 4, 6);
    let m = _mm256_castsi256_si128(_mm256_permutevar8x32_epi32(m, pick));
    let top = _mm256_castsi256_si128(_mm256_permutevar8x32_epi32(top, pick));
    let e = _mm_add_epi32(_mm_add_epi32(ea, eb), _mm_sub_epi32(top, _mm_set1_epi32(127)));
    let (bits, over, under) = finish(s, e, m);
    (_mm_blendv_epi8(bits, s, none), _mm_andnot_si128(none, over), _mm_andnot_si128(none, under))
}

/// `float::add` on every lane, the results written where `mask` (lane 0 at bit 3) allows over `old`.
///
/// # Safety
/// The host has AVX2 (`available`).
#[cfg(target_arch = "x86_64")]
#[inline]
#[target_feature(enable = "avx2")]
pub fn add(a: &[u32; 4], b: &[u32; 4], old: &[u32; 4], mask: u32) -> Vector {
    let (bits, over, under) = add_raw(load(a), load(b));
    let (z, s) = zs(bits);
    masked(bits, mac_of(z, s, under, over), load(old), mask)
}

/// `float::sub` on every lane: the addition of `b` with its signs reversed.
///
/// # Safety
/// The host has AVX2 (`available`).
#[cfg(target_arch = "x86_64")]
#[inline]
#[target_feature(enable = "avx2")]
pub fn sub(a: &[u32; 4], b: &[u32; 4], old: &[u32; 4], mask: u32) -> Vector {
    let (bits, over, under) = add_raw(load(a), _mm_xor_si128(load(b), _mm_set1_epi32(i32::MIN)));
    let (z, s) = zs(bits);
    masked(bits, mac_of(z, s, under, over), load(old), mask)
}

/// `float::mul` on every lane.
///
/// # Safety
/// The host has AVX2 (`available`).
#[cfg(target_arch = "x86_64")]
#[inline]
#[target_feature(enable = "avx2")]
pub fn mul(a: &[u32; 4], b: &[u32; 4], old: &[u32; 4], mask: u32) -> Vector {
    let (bits, over, under) = mul_raw(load(a), load(b));
    let (z, s) = zs(bits);
    masked(bits, mac_of(z, s, under, over), load(old), mask)
}

/// `float::fused` on every lane: `acc ± a × b`, with the products' own flags for the sticky bits.
///
/// # Safety
/// The host has AVX2 (`available`).
#[cfg(target_arch = "x86_64")]
#[inline]
#[target_feature(enable = "avx2")]
pub fn fused(acc: &[u32; 4], a: &[u32; 4], b: &[u32; 4], subtract: bool, old: &[u32; 4], mask: u32) -> FusedVector {
    let acc = load(acc);
    let (p, p_over, p_under) = mul_raw(load(a), load(b));
    let term = if subtract { _mm_xor_si128(p, _mm_set1_epi32(i32::MIN)) } else { p };
    let (sum, s_over, s_under) = add_raw(acc, term);
    let acc_top = _mm_cmpeq_epi32(_mm_and_si128(acc, _mm_set1_epi32(0x7F80_0000)), _mm_set1_epi32(0x7F80_0000));
    // An overflowing product gives its term; else an accumulator at field 255 is kept; both with overflow.
    let kept = _mm_andnot_si128(p_over, acc_top);
    let bits = _mm_blendv_epi8(_mm_blendv_epi8(sum, acc, kept), term, p_over);
    let special = _mm_or_si128(p_over, kept);
    let over = _mm_or_si128(special, s_over);
    let under = _mm_andnot_si128(special, s_under);
    let (z, s) = zs(bits);
    let (pz, ps) = zs(term);
    let old = load(old);
    FusedVector { value: masked(bits, mac_of(z, s, under, over), old, mask), product: masked(term, mac_of(pz, ps, p_under, p_over), old, mask) }
}

/// The MAC word's four groups as the status flag's Z, S, U, O (bits 0 to 3).
pub fn collect(mac: u32) -> u32 {
    u32::from(mac & 0xF != 0) | u32::from(mac & 0xF0 != 0) << 1 | u32::from(mac & 0xF00 != 0) << 2 | u32::from(mac & 0xF000 != 0) << 3
}

#[cfg(all(test, target_arch = "x86_64"))]
mod tests {
    use super::*;
    use crate::float::{self, Outcome};

    /// The MAC bits of one lane, as `vu::ops` sets them.
    fn mac_bits(o: Outcome, lane: usize) -> u32 {
        let at = 3 - lane as u32;
        u32::from(float::is_zero(o.bits)) << at | (o.bits >> 31) << (4 + at) | u32::from(o.underflow) << (8 + at) | u32::from(o.overflow) << (12 + at)
    }

    /// The reference: each lane through `float`, the lanes off the mask keeping `old` and no flag.
    fn reference(old: &[u32; 4], mask: u32, f: impl Fn(usize) -> Outcome) -> Vector {
        let mut v = Vector { bits: *old, mac: 0 };
        for l in 0..4 {
            if mask >> (3 - l) & 1 == 1 {
                let o = f(l);
                v.bits[l] = o.bits;
                v.mac |= mac_bits(o, l);
            }
        }
        v
    }

    fn stream(mut seed: u64) -> impl FnMut() -> u64 {
        move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        }
    }

    /// `float`'s tests' classes (Sapphire_Speed.md §56.5): zero, would-be denormals, field 255, fields 1 to 4 and 251 to
    /// 254, within 32 of 127 and anything, with a zero, all-ones or random fraction, either sign.
    fn operand(r: u64) -> u32 {
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
        ((r >> 63) as u32) << 31 | e << 23 | f
    }

    /// The reduced set graded exhaustively in pairs: every exponent field at either end and a run of 55 about the
    /// centre (every alignment distance from 0 to 54), sixteen fractions at the edges of the significand, both signs.
    fn reduced() -> Vec<u32> {
        let fields: Vec<u32> = (0..=5).chain(100..=154).chain(250..=255).collect();
        let fractions = [0, 1, 2, 3, 0x1FFF, 0x2000, 0x3F_FFFF, 0x40_0000, 0x40_0001, 0x55_5555, 0x2A_AAAA, 0x7F_FFFC, 0x7F_FFFD, 0x7F_FFFE, 0x7F_FFFF, 0x60_0000];
        let mut v = Vec::new();
        for s in 0..2u32 {
            for &e in &fields {
                for &f in &fractions {
                    v.push(s << 31 | e << 23 | f);
                }
            }
        }
        v
    }

    fn check(a: [u32; 4], b: [u32; 4], c: [u32; 4], old: [u32; 4], mask: u32, flags: &mut [u64; 2]) {
        // SAFETY: the tests run only where `available` is true.
        unsafe {
            let got = add(&a, &b, &old, mask);
            let want = reference(&old, mask, |l| float::add(a[l], b[l]));
            assert_eq!(got, want, "add {a:08x?} {b:08x?} mask {mask:x}");
            let got = sub(&a, &b, &old, mask);
            assert_eq!(got, reference(&old, mask, |l| float::sub(a[l], b[l])), "sub {a:08x?} {b:08x?}");
            let got = mul(&a, &b, &old, mask);
            let want = reference(&old, mask, |l| float::mul(a[l], b[l]));
            flags[0] += u64::from((want.mac >> 8 & 0xF).count_ones() + (want.mac >> 12).count_ones());
            assert_eq!(got, want, "mul {a:08x?} {b:08x?}");
            for subtract in [false, true] {
                let got = fused(&c, &a, &b, subtract, &old, mask);
                let value = reference(&old, mask, |l| float::fused(c[l], a[l], b[l], subtract).value);
                let product = reference(&old, mask, |l| float::fused(c[l], a[l], b[l], subtract).product);
                flags[1] += u64::from((value.mac >> 8).count_ones());
                assert_eq!(got, FusedVector { value, product }, "fused {c:08x?} {a:08x?} {b:08x?} subtract {subtract}");
            }
        }
    }

    #[test]
    fn every_pair_of_the_reduced_set_equals_float_in_every_lane() {
        if !available() {
            return;
        }
        let v = reduced();
        let mut next = stream(0x5EED_1A9E_5000_0001);
        let mut flags = [0; 2];
        // Each pair in lane `i % 4`, beside three other pairs of the set, against an accumulator drawn from it.
        for (i, &x) in v.iter().enumerate() {
            for (j, &y) in v.iter().enumerate().step_by(1) {
                let k = (i + j) % 4;
                let r = next();
                let mut a = [v[r as usize % v.len()], v[(r >> 16) as usize % v.len()], v[(r >> 32) as usize % v.len()], v[(r >> 48) as usize % v.len()]];
                let mut b = [a[3], a[2], a[1], a[0]];
                a[k] = x;
                b[k] = y;
                let c = [v[(r >> 8) as usize % v.len()], y, x, v[(r >> 24) as usize % v.len()]];
                if j % 4 == 0 {
                    check(a, b, c, [0xDEAD_BEEF; 4], 0xF, &mut flags);
                } else {
                    check(a, b, c, [r as u32, (r >> 32) as u32, 7, 9], (r >> 60) as u32, &mut flags);
                }
            }
        }
        assert!(flags[0] > 10_000 && flags[1] > 10_000, "overflows and underflows met: {flags:?}");
    }

    #[test]
    fn four_million_vectors_of_every_class_equal_float() {
        if !available() {
            return;
        }
        let mut next = stream(0x0DDB_A11C_0FFE_E123);
        let mut flags = [0; 2];
        for _ in 0..4_000_000 {
            let mut v = [0u32; 12];
            for x in &mut v {
                *x = operand(next());
            }
            let m = next();
            check(v[0..4].try_into().unwrap(), v[4..8].try_into().unwrap(), v[8..12].try_into().unwrap(), [m as u32; 4], (m >> 32) as u32 & 0xF | 0x8, &mut flags);
        }
        assert!(flags[0] > 10_000 && flags[1] > 10_000, "overflows and underflows met: {flags:?}");
    }

    #[test]
    #[ignore = "a billion vectors, a minute or more in release"]
    fn a_billion_vectors_equal_float() {
        if !available() {
            return;
        }
        let mut next = stream(0x1234_5678_9ABC_DEF1);
        let mut flags = [0; 2];
        for _ in 0..1_000_000_000u64 / 4 {
            let r = [next(), next(), next()];
            let v = |i: usize| (r[i] as u32, (r[i] >> 32) as u32);
            let ((a0, a1), (b0, b1), (c0, c1)) = (v(0), v(1), v(2));
            let a = [operand(r[0]), a0, a1 ^ b0, operand(r[1])];
            let b = [operand(r[2]), b1, c0, a0 ^ c1];
            let c = [c0, operand(r[0] ^ r[2]), c1, b0];
            check(a, b, c, [0; 4], 0xF, &mut flags);
        }
    }
}
