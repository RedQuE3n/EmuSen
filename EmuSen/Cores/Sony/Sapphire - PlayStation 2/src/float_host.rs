//! `float::add`, `mul` and `fused` on the host's floats, bit for bit with the MAC flags; a measurement kernel behind `lanes` (Sapphire_Speed.md §59).

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::float;
use crate::float_lanes::{FusedPair, FusedVector, Pair, Vector};

const SIGN: u32 = 0x8000_0000;
const MAX: u32 = 0x7FFF_FFFF;
/// Rounding towards zero with exceptions suppressed, the `{rz-sae}` of every rounded operation here.
#[cfg(target_arch = "x86_64")]
const RZ: i32 = _MM_FROUND_TO_ZERO | _MM_FROUND_NO_EXC;
/// Four lanes' bits reversed: lane 0 at bit 0 to the VU's lane 0 at bit 3, and back.
const REV: [u32; 16] = [0, 8, 4, 12, 2, 10, 6, 14, 1, 9, 5, 13, 3, 11, 7, 15];

/// Whether the host runs the `{rz-sae}` form (`four`, `eight`).
pub fn available512() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        std::arch::is_x86_feature_detected!("avx512f") && std::arch::is_x86_feature_detected!("avx512dq") && std::arch::is_x86_feature_detected!("avx2")
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}

/// Whether the host runs the AVX2 form (`avx2`).
pub fn available() -> bool {
    crate::float_lanes::available()
}

const fn field(x: u32) -> u32 {
    x >> 23 & 0xFF
}

/// Whether `float::mul(a, b)` is the host's product of the two rounded towards zero, or a signed zero (§59.2).
pub fn mul_taken(a: u32, b: u32) -> bool {
    let (ea, eb) = (field(a), field(b));
    if ea == 0 || eb == 0 {
        return true;
    }
    let (x, y) = ((a | 0x80_0000) as u64 & 0xFF_FFFF, (b | 0x80_0000) as u64 & 0xFF_FFFF);
    let short = x & ((1 << float::K) - 1) == 0;
    ea <= 254 && eb <= 254 && (129..=380).contains(&(ea + eb)) && (y & float::READ == 0 || x * y & 0x7F_FFFF >= float::MOST_LOST || short)
}

/// Whether `float::add(a, b)` is the host's sum of the larger and the smaller cut at its guard bit, rounded towards zero (§59.3).
pub fn add_taken(a: u32, b: u32) -> bool {
    let (ea, eb) = (field(a), field(b));
    let (big, small) = if (ea != 0) as u32 * (a & MAX) >= (eb != 0) as u32 * (b & MAX) { (ea, eb) } else { (eb, ea) };
    big <= 253 && (big >= 25 || small == 0)
}

/// Whether `float::fused(acc, a, b, subtract)` is taken: its product and then its sum.
pub fn fused_taken(acc: u32, a: u32, b: u32, subtract: bool) -> bool {
    mul_taken(a, b) && add_taken(acc, float::mul(a, b).bits ^ if subtract { SIGN } else { 0 })
}

/// The MAC groups' mask for the field mask's lanes (first operation's bits 3-0, a second's 19-16).
const fn mac_mask(mask: u32) -> u32 {
    ((mask & 0xF) * 0x1111) | (((mask >> 16 & 0xF) * 0x1111) << 16)
}

/// The `{rz-sae}` form's operations on up to sixteen lanes; each returns the bits and the lanes it takes exactly.
#[cfg(target_arch = "x86_64")]
mod z {
    use super::*;

    #[target_feature(enable = "avx2,avx512f,avx512dq")]
    #[inline]
    fn k(x: u32) -> __m512i {
        _mm512_set1_epi32(x as i32)
    }
    #[target_feature(enable = "avx2,avx512f,avx512dq")]
    #[inline]
    fn fields(v: __m512i) -> __m512i {
        _mm512_and_si512(_mm512_srli_epi32::<23>(v), k(0xFF))
    }
    /// The console's zeros, every field-0 lane, as the host's signed zeros.
    #[target_feature(enable = "avx2,avx512f,avx512dq")]
    #[inline]
    fn zeroed(v: __m512i) -> __m512i {
        _mm512_mask_mov_epi32(v, _mm512_cmpeq_epi32_mask(fields(v), k(0)), _mm512_and_si512(v, k(SIGN)))
    }

    /// `float::mul`, `b` recoded (§59.2).
    #[target_feature(enable = "avx2,avx512f,avx512dq")]
    #[inline]
    pub fn mul(a: __m512i, b: __m512i) -> (__m512i, __mmask16) {
        let (ea, eb) = (fields(a), fields(b));
        let zero = _mm512_cmpeq_epi32_mask(ea, k(0)) | _mm512_cmpeq_epi32_mask(eb, k(0));
        let below = _mm512_cmplt_epu32_mask(_mm512_sub_epi32(ea, k(1)), k(254)) & _mm512_cmplt_epu32_mask(_mm512_sub_epi32(eb, k(1)), k(254));
        let sum = _mm512_add_epi32(ea, eb);
        let range = _mm512_cmpge_epi32_mask(sum, k(129)) & _mm512_cmple_epi32_mask(sum, k(380));
        let (hidden, frac) = (k(0x80_0000), k(0x7F_FFFF));
        let x = _mm512_or_si512(_mm512_and_si512(a, frac), hidden);
        let y = _mm512_or_si512(_mm512_and_si512(b, frac), hidden);
        let low = _mm512_mullo_epi32(x, y);
        let quiet = !_mm512_test_epi32_mask(y, k(float::READ as u32));
        let far = _mm512_cmpge_epu32_mask(_mm512_and_si512(low, frac), k(float::MOST_LOST as u32));
        let short = !_mm512_test_epi32_mask(x, k((1 << float::K) - 1));
        let taken = zero | (below & range & (quiet | far | short));
        let p = _mm512_castps_si512(_mm512_mul_round_ps::<RZ>(_mm512_castsi512_ps(a), _mm512_castsi512_ps(b)));
        // A short multiplicand's loss is 2^k a negative digit; the product is one unit low where it passes D (§59.2).
        let top = _mm512_sub_epi32(fields(p), _mm512_sub_epi32(sum, k(127)));
        let d = _mm512_srli_epi32::<{ float::K }>(_mm512_and_si512(low, _mm512_or_si512(frac, _mm512_slli_epi32::<23>(top))));
        let negatives = _mm512_zextsi256_si512(crate::float_lanes::negatives(_mm512_castsi512_si256(y)));
        let p = _mm512_mask_sub_epi32(p, short & _mm512_cmpgt_epu32_mask(negatives, d), p, k(1));
        (_mm512_mask_mov_epi32(p, zero, _mm512_and_si512(_mm512_xor_si512(a, b), k(SIGN))), taken)
    }

    /// `float::add`: S' = RZ(S + C) − C cuts the smaller at the larger's guard bit, C = ±2^(e−1); then RZ(L + S') (§59.3).
    #[target_feature(enable = "avx2,avx512f,avx512dq")]
    #[inline]
    pub fn add(a: __m512i, b: __m512i) -> (__m512i, __mmask16) {
        let (a, b) = (zeroed(a), zeroed(b));
        let b_big = _mm512_cmpgt_epi32_mask(_mm512_and_si512(b, k(MAX)), _mm512_and_si512(a, k(MAX)));
        let (big, small) = (_mm512_mask_blend_epi32(b_big, a, b), _mm512_mask_blend_epi32(b_big, b, a));
        let (fb, fs) = (fields(big), fields(small));
        let taken = _mm512_cmple_epi32_mask(fb, k(253)) & (_mm512_cmpge_epi32_mask(fb, k(25)) | _mm512_cmpeq_epi32_mask(fs, k(0)));
        let cut = _mm512_cmpge_epi32_mask(_mm512_sub_epi32(fb, fs), k(2));
        let c = _mm512_castsi512_ps(_mm512_or_si512(_mm512_and_si512(small, k(SIGN)), _mm512_slli_epi32::<23>(_mm512_sub_epi32(fb, k(1)))));
        let s = _mm512_castsi512_ps(small);
        let s = _mm512_mask_sub_round_ps::<RZ>(s, cut, _mm512_mask_add_round_ps::<RZ>(s, cut, s, c), c);
        (_mm512_castps_si512(_mm512_add_round_ps::<RZ>(_mm512_castsi512_ps(big), s)), taken)
    }

    /// `float::fused`'s sum and term: two truncated steps, never one fused one.
    #[target_feature(enable = "avx2,avx512f,avx512dq")]
    #[inline]
    pub fn fused(acc: __m512i, a: __m512i, b: __m512i, subtract: bool) -> (__m512i, __m512i, __mmask16) {
        let (p, pt) = mul(a, b);
        let term = if subtract { _mm512_xor_si512(p, k(SIGN)) } else { p };
        let (sum, st) = add(acc, term);
        (sum, term, pt & st)
    }

    /// The MAC word's Z and S groups of the four lanes from lane `at` on.
    #[target_feature(enable = "avx2,avx512f,avx512dq")]
    #[inline]
    pub fn mac(bits: __m512i, at: u32) -> u32 {
        let z = (_mm512_cmpeq_epi32_mask(_mm512_and_si512(bits, k(0x7F80_0000)), k(0)) as u32) >> at & 0xF;
        let s = (_mm512_movepi32_mask(bits) as u32) >> at & 0xF;
        REV[z as usize] | REV[s as usize] << 4
    }

    /// The register's lanes `mask` writes: the first operation's bits 3-0 to lanes 0-3, a second's bits 19-16 to 4-7.
    pub fn on(mask: u32) -> __mmask16 {
        (REV[(mask & 0xF) as usize] | REV[(mask >> 16 & 0xF) as usize] << 4) as __mmask16
    }

    /// # Safety
    /// `p` holds the lanes `m` reads.
    #[cfg(test)]
    #[target_feature(enable = "avx2,avx512f,avx512dq")]
    #[inline]
    unsafe fn load(p: *const u32, m: __mmask16) -> __m512i {
        unsafe { _mm512_maskz_loadu_epi32(m, p.cast()) }
    }

    /// The lanes each operation takes, lane 0 at bit 0, as `avx2::taken`.
    #[cfg(test)]
    #[target_feature(enable = "avx2,avx512f,avx512dq")]
    pub(super) fn taken(op: u8, c: &[u32; 4], a: &[u32; 4], b: &[u32; 4]) -> u32 {
        // SAFETY: four lanes each.
        let (c, a, b) = unsafe { (load(c.as_ptr(), 0xF), load(a.as_ptr(), 0xF), load(b.as_ptr(), 0xF)) };
        u32::from(match op {
            0 => add(a, b).1,
            1 => mul(a, b).1,
            _ => fused(c, a, b, false).2,
        }) & 0xF
    }
}

macro_rules! host_entry {
    ($n:literal, $all:expr, $out:ident, $fused_out:ident, $add:path, $mul:path, $fused:path, $load:ident, $zext:ident, $store:ident, $narrow:ident) => {
        #[target_feature(enable = "avx2,avx512f,avx512dq")]
        #[inline]
        fn finish(bits: __m512i, old: &[u32; $n], mask: u32) -> $out {
            let mut r = [0u32; $n];
            // SAFETY: `r` holds the register's lanes; a whole store, which a later load can take from (§59.4).
            unsafe { $store(r.as_mut_ptr().cast(), $narrow(_mm512_mask_mov_epi32(get(old), z::on(mask), bits))) };
            let mut mac = 0;
            for h in 0..$n / 4 {
                mac |= z::mac(bits, 4 * h) << (16 * h);
            }
            $out { bits: r, mac: mac & mac_mask(mask) }
        }
        #[target_feature(enable = "avx2,avx512f,avx512dq")]
        #[inline]
        fn get(v: &[u32; $n]) -> __m512i {
            // SAFETY: `v` holds the register's lanes.
            $zext(unsafe { $load(v.as_ptr().cast()) })
        }
        /// `float::add`, `mask`'s lanes written over `old`.
        #[target_feature(enable = "avx2,avx512f,avx512dq")]
        pub fn add(a: &[u32; $n], b: &[u32; $n], old: &[u32; $n], mask: u32) -> $out {
            let (bits, taken) = z::add(get(a), get(b));
            if taken & $all != $all {
                return $add(a, b, old, mask);
            }
            finish(bits, old, mask)
        }
        /// `float::sub`.
        #[target_feature(enable = "avx2,avx512f,avx512dq")]
        pub fn sub(a: &[u32; $n], b: &[u32; $n], old: &[u32; $n], mask: u32) -> $out {
            add(a, &b.map(|x| x ^ SIGN), old, mask)
        }
        /// `float::mul`.
        #[target_feature(enable = "avx2,avx512f,avx512dq")]
        pub fn mul(a: &[u32; $n], b: &[u32; $n], old: &[u32; $n], mask: u32) -> $out {
            let (bits, taken) = z::mul(get(a), get(b));
            if taken & $all != $all {
                return $mul(a, b, old, mask);
            }
            finish(bits, old, mask)
        }
        /// `float::fused`, with the terms' own flags.
        #[target_feature(enable = "avx2,avx512f,avx512dq")]
        pub fn fused(acc: &[u32; $n], a: &[u32; $n], b: &[u32; $n], subtract: bool, old: &[u32; $n], mask: u32) -> $fused_out {
            let (sum, term, taken) = z::fused(get(acc), get(a), get(b), subtract);
            if taken & $all != $all {
                return $fused(acc, a, b, subtract, old, mask);
            }
            $fused_out { value: finish(sum, old, mask), product: finish(term, old, mask) }
        }
    };
}

/// The `{rz-sae}` form on one vector of four lanes; a lane it does not take sends the vector to `float_lanes`.
/// Safety, for every entry point: `available512`.
#[cfg(target_arch = "x86_64")]
#[allow(clippy::missing_safety_doc)]
pub mod four {
    use super::*;
    host_entry!(4, 0xF, Vector, FusedVector, crate::float_lanes::add, crate::float_lanes::mul, crate::float_lanes::fused, _mm_loadu_si128, _mm512_zextsi128_si512, _mm_storeu_si128, _mm512_castsi512_si128);
}

/// The same on two vectors in eight lanes, `mask`'s second operation in bits 19-16.
#[cfg(target_arch = "x86_64")]
#[allow(clippy::missing_safety_doc)]
pub mod eight {
    use super::*;
    host_entry!(8, 0xFF, Pair, FusedPair, crate::float_lanes::add2, crate::float_lanes::mul2, crate::float_lanes::fused2, _mm256_loadu_si256, _mm512_zextsi256_si512, _mm256_storeu_si256, _mm512_castsi512_si256);
}

/// The AVX2 form: the same results with no rounding mode, the exact sums and products in doubles cut to 24 bits.
/// Safety, for every entry point: `available`.
#[cfg(target_arch = "x86_64")]
#[allow(clippy::missing_safety_doc)]
pub mod avx2 {
    use super::*;

    #[target_feature(enable = "avx2")]
    #[inline]
    fn k(x: u32) -> __m128i {
        _mm_set1_epi32(x as i32)
    }
    #[target_feature(enable = "avx2")]
    #[inline]
    fn fields(v: __m128i) -> __m128i {
        _mm_and_si128(_mm_srli_epi32::<23>(v), k(0xFF))
    }
    #[target_feature(enable = "avx2")]
    #[inline]
    fn bits(m: __m128i) -> u32 {
        _mm_movemask_ps(_mm_castsi128_ps(m)) as u32
    }
    /// An exact double rounded towards zero to 24 bits, as single precision (exact: the double then fits).
    #[target_feature(enable = "avx2")]
    #[inline]
    fn truncated(d: __m256d) -> __m128i {
        let cut = _mm256_and_si256(_mm256_castpd_si256(d), _mm256_set1_epi64x(!0x1FFF_FFFF));
        _mm_castps_si128(_mm256_cvtpd_ps(_mm256_castsi256_pd(cut)))
    }
    #[target_feature(enable = "avx2")]
    #[inline]
    fn wide(v: __m128i) -> __m256d {
        _mm256_cvtps_pd(_mm_castsi128_ps(v))
    }
    #[target_feature(enable = "avx2")]
    #[inline]
    fn zeroed(v: __m128i) -> __m128i {
        _mm_blendv_epi8(v, _mm_and_si128(v, k(SIGN)), _mm_cmpeq_epi32(fields(v), k(0)))
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    fn mul_raw(a: __m128i, b: __m128i) -> (__m128i, u32) {
        let (ea, eb) = (fields(a), fields(b));
        let zero = _mm_or_si128(_mm_cmpeq_epi32(ea, k(0)), _mm_cmpeq_epi32(eb, k(0)));
        let below = _mm_and_si128(_mm_cmpgt_epi32(k(255), ea), _mm_cmpgt_epi32(k(255), eb));
        let sum = _mm_add_epi32(ea, eb);
        let range = _mm_and_si128(_mm_cmpgt_epi32(sum, k(128)), _mm_cmpgt_epi32(k(381), sum));
        let (hidden, frac) = (k(0x80_0000), k(0x7F_FFFF));
        let x = _mm_or_si128(_mm_and_si128(a, frac), hidden);
        let y = _mm_or_si128(_mm_and_si128(b, frac), hidden);
        let low = _mm_mullo_epi32(x, y);
        let quiet = _mm_cmpeq_epi32(_mm_and_si128(y, k(float::READ as u32)), k(0));
        let far = _mm_cmpgt_epi32(_mm_and_si128(low, frac), k(float::MOST_LOST as u32 - 1));
        let short = _mm_cmpeq_epi32(_mm_and_si128(x, k((1 << float::K) - 1)), k(0));
        let taken = _mm_or_si128(zero, _mm_and_si128(_mm_and_si128(below, range), _mm_or_si128(_mm_or_si128(quiet, far), short)));
        let p = truncated(_mm256_mul_pd(wide(a), wide(b)));
        let top = _mm_sub_epi32(fields(p), _mm_sub_epi32(sum, k(127)));
        let d = _mm_srli_epi32::<{ float::K as i32 }>(_mm_and_si128(low, _mm_or_si128(frac, _mm_slli_epi32::<23>(top))));
        let one_low = _mm_and_si128(short, _mm_cmpgt_epi32(crate::float_lanes::negatives(y), d));
        let p = _mm_add_epi32(p, one_low);
        (_mm_blendv_epi8(p, _mm_and_si128(_mm_xor_si128(a, b), k(SIGN)), zero), bits(taken))
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    fn add_raw(a: __m128i, b: __m128i) -> (__m128i, u32) {
        let (a, b) = (zeroed(a), zeroed(b));
        let b_big = _mm_cmpgt_epi32(_mm_and_si128(b, k(MAX)), _mm_and_si128(a, k(MAX)));
        let (big, small) = (_mm_blendv_epi8(a, b, b_big), _mm_blendv_epi8(b, a, b_big));
        let (fb, fs) = (fields(big), fields(small));
        let taken = _mm_and_si128(_mm_cmpgt_epi32(k(254), fb), _mm_or_si128(_mm_cmpgt_epi32(fb, k(24)), _mm_cmpeq_epi32(fs, k(0))));
        // The smaller cut at the larger's guard bit: d − 1 fraction bits cleared, nothing left from d = 25.
        let d = _mm_sub_epi32(fb, fs);
        let keep = _mm_sllv_epi32(k(u32::MAX), _mm_max_epi32(_mm_sub_epi32(d, k(1)), k(0)));
        let cut = _mm_blendv_epi8(_mm_and_si128(small, keep), _mm_and_si128(small, k(SIGN)), _mm_cmpgt_epi32(d, k(24)));
        (truncated(_mm256_add_pd(wide(big), wide(cut))), bits(taken))
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    fn get(v: &[u32; 4]) -> __m128i {
        // SAFETY: `v` holds four lanes.
        unsafe { _mm_loadu_si128(v.as_ptr().cast()) }
    }
    #[target_feature(enable = "avx2")]
    #[inline]
    fn finish(r: __m128i, old: &[u32; 4], mask: u32) -> Vector {
        let on = _mm_cmpgt_epi32(_mm_setr_epi32((mask & 8) as i32, (mask & 4) as i32, (mask & 2) as i32, (mask & 1) as i32), k(0));
        let mut out = [0u32; 4];
        // SAFETY: `out` holds four lanes.
        unsafe { _mm_storeu_si128(out.as_mut_ptr().cast(), _mm_blendv_epi8(get(old), r, on)) };
        let z = bits(_mm_cmpeq_epi32(_mm_and_si128(r, k(0x7F80_0000)), k(0)));
        let s = bits(r);
        Vector { bits: out, mac: (REV[z as usize] | REV[s as usize] << 4) & mac_mask(mask) }
    }

    #[target_feature(enable = "avx2")]
    pub fn add(a: &[u32; 4], b: &[u32; 4], old: &[u32; 4], mask: u32) -> Vector {
        let (r, taken) = add_raw(get(a), get(b));
        if taken != 0xF {
            return crate::float_lanes::add(a, b, old, mask);
        }
        finish(r, old, mask)
    }
    #[target_feature(enable = "avx2")]
    pub fn sub(a: &[u32; 4], b: &[u32; 4], old: &[u32; 4], mask: u32) -> Vector {
        add(a, &b.map(|x| x ^ SIGN), old, mask)
    }
    #[target_feature(enable = "avx2")]
    pub fn mul(a: &[u32; 4], b: &[u32; 4], old: &[u32; 4], mask: u32) -> Vector {
        let (r, taken) = mul_raw(get(a), get(b));
        if taken != 0xF {
            return crate::float_lanes::mul(a, b, old, mask);
        }
        finish(r, old, mask)
    }
    #[target_feature(enable = "avx2")]
    pub fn fused(acc: &[u32; 4], a: &[u32; 4], b: &[u32; 4], subtract: bool, old: &[u32; 4], mask: u32) -> FusedVector {
        let (p, pt) = mul_raw(get(a), get(b));
        let term = if subtract { _mm_xor_si128(p, k(SIGN)) } else { p };
        let (sum, st) = add_raw(get(acc), term);
        if pt & st != 0xF {
            return crate::float_lanes::fused(acc, a, b, subtract, old, mask);
        }
        FusedVector { value: finish(sum, old, mask), product: finish(term, old, mask) }
    }

    fn halves(v: &[u32; 8]) -> [[u32; 4]; 2] {
        [[v[0], v[1], v[2], v[3]], [v[4], v[5], v[6], v[7]]]
    }
    fn join(x: Vector, y: Vector) -> Pair {
        let b = [x.bits[0], x.bits[1], x.bits[2], x.bits[3], y.bits[0], y.bits[1], y.bits[2], y.bits[3]];
        Pair { bits: b, mac: x.mac | y.mac << 16 }
    }
    /// Two operations of four lanes, as `eight` takes them.
    #[target_feature(enable = "avx2")]
    pub fn add2(a: &[u32; 8], b: &[u32; 8], old: &[u32; 8], mask: u32) -> Pair {
        let (a, b, o) = (halves(a), halves(b), halves(old));
        join(add(&a[0], &b[0], &o[0], mask & 0xF), add(&a[1], &b[1], &o[1], mask >> 16 & 0xF))
    }
    #[target_feature(enable = "avx2")]
    pub fn mul2(a: &[u32; 8], b: &[u32; 8], old: &[u32; 8], mask: u32) -> Pair {
        let (a, b, o) = (halves(a), halves(b), halves(old));
        join(mul(&a[0], &b[0], &o[0], mask & 0xF), mul(&a[1], &b[1], &o[1], mask >> 16 & 0xF))
    }
    #[target_feature(enable = "avx2")]
    pub fn fused2(acc: &[u32; 8], a: &[u32; 8], b: &[u32; 8], subtract: bool, old: &[u32; 8], mask: u32) -> FusedPair {
        let (c, a, b, o) = (halves(acc), halves(a), halves(b), halves(old));
        let (f, g) = (fused(&c[0], &a[0], &b[0], subtract, &o[0], mask & 0xF), fused(&c[1], &a[1], &b[1], subtract, &o[1], mask >> 16 & 0xF));
        FusedPair { value: join(f.value, g.value), product: join(f.product, g.product) }
    }

    /// The lanes each operation takes on the host path, lane 0 at bit 0: 0 add, 1 mul, 2 multiply-add.
    #[cfg(test)]
    #[target_feature(enable = "avx2")]
    pub(super) fn taken(op: u8, c: &[u32; 4], a: &[u32; 4], b: &[u32; 4]) -> u32 {
        match op {
            0 => add_raw(get(a), get(b)).1,
            1 => mul_raw(get(a), get(b)).1,
            _ => {
                let (p, pt) = mul_raw(get(a), get(b));
                pt & add_raw(get(c), p).1
            }
        }
    }
}

#[cfg(all(test, target_arch = "x86_64"))]
mod tests {
    use super::*;
    use crate::float::Outcome;
    use crate::float::tests::{operand, stream};

    fn mac_bits(o: Outcome, lane: usize) -> u32 {
        let at = 3 - lane as u32;
        u32::from(float::is_zero(o.bits)) << at | (o.bits >> 31) << (4 + at) | u32::from(o.underflow) << (8 + at) | u32::from(o.overflow) << (12 + at)
    }
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
    fn join(x: [u32; 4], y: [u32; 4]) -> [u32; 8] {
        [x[0], x[1], x[2], x[3], y[0], y[1], y[2], y[3]]
    }
    fn both(x: Vector, y: Vector) -> Pair {
        Pair { bits: join(x.bits, y.bits), mac: x.mac | y.mac << 16 }
    }

    /// Both forms against `float` lane by lane, four lanes and paired; counts the vectors taken whole as add, mul, multiply-add.
    fn check(a: [u32; 4], b: [u32; 4], c: [u32; 4], old: [u32; 4], mask: u32, taken: &mut [u64; 3]) {
        let wide = available512();
        let r = |f: fn(u32, u32) -> Outcome| reference(&old, mask, |l| f(a[l], b[l]));
        taken[0] += u64::from((0..4).all(|l| add_taken(a[l], b[l])));
        taken[1] += u64::from((0..4).all(|l| mul_taken(a[l], b[l])));
        // The paired forms' second operation: other operands, the other lanes written.
        let (a2, b2, c2, old2, mask2) = ([b[1], c[0], a[3], b[2]], [c[2], a[0], b[3], a[1]], [a[2], b[0], c[3], c[1]], [old[3], old[2], old[1], old[0]], !mask & 0xF);
        let (pa, pb, pc, po, pm) = (join(a, a2), join(b, b2), join(c, c2), join(old, old2), mask | mask2 << 16);
        let r2 = |f: fn(u32, u32) -> Outcome| both(r(f), reference(&old2, mask2, |l| f(a2[l], b2[l])));
        // SAFETY: the tests run where AVX2 is present, and the wide form only where `available512` is.
        unsafe {
            assert_eq!(avx2::add(&a, &b, &old, mask), r(float::add), "avx2 add {a:08x?} {b:08x?}");
            assert_eq!(avx2::sub(&a, &b, &old, mask), r(float::sub), "avx2 sub {a:08x?} {b:08x?}");
            assert_eq!(avx2::mul(&a, &b, &old, mask), r(float::mul), "avx2 mul {a:08x?} {b:08x?}");
            assert_eq!(avx2::add2(&pa, &pb, &po, pm), r2(float::add), "avx2 add2");
            assert_eq!(avx2::mul2(&pa, &pb, &po, pm), r2(float::mul), "avx2 mul2");
            if wide {
                assert_eq!(four::add(&a, &b, &old, mask), r(float::add), "add {a:08x?} {b:08x?}");
                assert_eq!(four::sub(&a, &b, &old, mask), r(float::sub), "sub {a:08x?} {b:08x?}");
                assert_eq!(four::mul(&a, &b, &old, mask), r(float::mul), "mul {a:08x?} {b:08x?}");
                assert_eq!(eight::add(&pa, &pb, &po, pm), r2(float::add), "eight add");
                assert_eq!(eight::sub(&pa, &pb, &po, pm), r2(float::sub), "eight sub");
                assert_eq!(eight::mul(&pa, &pb, &po, pm), r2(float::mul), "eight mul");
            }
            for subtract in [false, true] {
                taken[2] += u64::from((0..4).all(|l| fused_taken(c[l], a[l], b[l], subtract)));
                let part = |acc: [u32; 4], x: [u32; 4], y: [u32; 4], o: [u32; 4], m: u32, k: bool| {
                    reference(&o, m, |l| {
                        let f = float::fused(acc[l], x[l], y[l], subtract);
                        if k { f.product } else { f.value }
                    })
                };
                let want = FusedVector { value: part(c, a, b, old, mask, false), product: part(c, a, b, old, mask, true) };
                let want2 = FusedPair {
                    value: both(want.value, part(c2, a2, b2, old2, mask2, false)),
                    product: both(want.product, part(c2, a2, b2, old2, mask2, true)),
                };
                assert_eq!(avx2::fused(&c, &a, &b, subtract, &old, mask), want, "avx2 fused {c:08x?} {a:08x?} {b:08x?} {subtract}");
                assert_eq!(avx2::fused2(&pc, &pa, &pb, subtract, &po, pm), want2, "avx2 fused2");
                if wide {
                    assert_eq!(four::fused(&c, &a, &b, subtract, &old, mask), want, "fused {c:08x?} {a:08x?} {b:08x?} {subtract}");
                    assert_eq!(eight::fused(&pc, &pa, &pb, subtract, &po, pm), want2, "eight fused");
                }
            }
        }
    }

    /// R0.4's reduced set: fields 0-5, 100-154 and 250-255, sixteen edge fractions, both signs.
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

    #[test]
    fn every_pair_of_the_reduced_set_equals_float_on_the_host_path() {
        if !available() {
            return;
        }
        let v = reduced();
        let mut next = stream(0x5EED_1A9E_5000_0001);
        let mut taken = [0; 3];
        for (i, &x) in v.iter().enumerate() {
            for (j, &y) in v.iter().enumerate() {
                let k = (i + j) % 4;
                let r = next();
                let mut a = [v[r as usize % v.len()], v[(r >> 16) as usize % v.len()], v[(r >> 32) as usize % v.len()], v[(r >> 48) as usize % v.len()]];
                let mut b = [a[3], a[2], a[1], a[0]];
                a[k] = x;
                b[k] = y;
                let mut c = [v[(r >> 8) as usize % v.len()], y, x, v[(r >> 24) as usize % v.len()]];
                // Every fourth pair fills the vector, so that vectors the host takes whole are met as often.
                if j % 4 == 1 {
                    (a, b, c) = ([x; 4], [y; 4], [c[0]; 4]);
                }
                check(a, b, c, [r as u32, (r >> 32) as u32, 7, 9], if j % 4 == 0 { 0xF } else { (r >> 60) as u32 }, &mut taken);
            }
        }
        assert!(taken.iter().all(|&t| t > 100_000), "vectors taken whole: {taken:?}");
    }

    #[test]
    fn four_million_vectors_of_every_class_equal_float_on_the_host_path() {
        if !available() {
            return;
        }
        let mut next = stream(0x0DDB_A11C_0FFE_E123);
        let mut taken = [0; 3];
        for i in 0..4_000_000 {
            let mut v = [0u32; 12];
            for x in &mut v {
                *x = operand(next());
            }
            // Half the vectors repeat one lane's operands, so that vectors the host takes whole are met.
            if i % 2 == 0 {
                for l in 1..4 {
                    v[l] = v[0];
                    v[4 + l] = v[4];
                    v[8 + l] = v[8];
                }
            }
            let m = next();
            check(v[0..4].try_into().unwrap(), v[4..8].try_into().unwrap(), v[8..12].try_into().unwrap(), [m as u32; 4], (m >> 32) as u32 & 0xF | 0x8, &mut taken);
        }
        assert!(taken.iter().all(|&t| t > 100_000), "vectors taken whole: {taken:?}");
    }

    #[test]
    #[ignore = "a billion lanes, minutes in release"]
    fn a_billion_lanes_equal_float_on_the_host_path() {
        if !available() {
            return;
        }
        let mut next = stream(0x1234_5678_9ABC_DEF1);
        let mut taken = [0; 3];
        for _ in 0..1_000_000_000u64 / 4 {
            let r = [next(), next(), next()];
            let v = |i: usize| (r[i] as u32, (r[i] >> 32) as u32);
            let ((a0, a1), (b0, b1), (c0, c1)) = (v(0), v(1), v(2));
            let a = [operand(r[0]), a0, a1 ^ b0, operand(r[1])];
            let b = [operand(r[2]), b1, c0, a0 ^ c1];
            let c = [c0, operand(r[0] ^ r[2]), c1, b0];
            check(a, b, c, [0; 4], 0xF, &mut taken);
        }
        println!("vectors taken whole: {taken:?}");
    }

    /// The vector a billion-lane run failed on when it ran a stale build of seeded fault F7 (Sapphire_Speed.md §59.5).
    #[test]
    fn the_vector_a_stale_mutant_failed_on_equals_float() {
        if !available() {
            return;
        }
        let (a, b) = std::hint::black_box(([0x07F3_A3B1, 0x15F3_A3B1, 0x858A_71D6, 0x92AA_88B6], [0x807F_FFFF, 0x9272_58A4, 0xBE48_3027, 0xD460_131E]));
        check(a, b, [b[2], a[1], b[0], a[3]], [0; 4], 0xF, &mut [0; 3]);
    }

    /// The scalar rules and the vector masks agree lane by lane, the rules being what the speed's shares count.
    #[test]
    fn the_vector_masks_are_the_scalar_rules() {
        if !available() {
            return;
        }
        let mut next = stream(0xA11C_E5ED_0000_0059);
        for _ in 0..1_000_000 {
            let (a, b, c) = ([0; 4].map(|_: u32| operand(next())), [0; 4].map(|_: u32| operand(next())), [0; 4].map(|_: u32| operand(next())));
            let lanes = |f: &dyn Fn(usize) -> bool| (0..4).fold(0u32, |m, l| m | u32::from(f(l)) << l);
            let want = [lanes(&|l| add_taken(a[l], b[l])), lanes(&|l| mul_taken(a[l], b[l])), lanes(&|l| fused_taken(c[l], a[l], b[l], false))];
            for op in 0..3u8 {
                // SAFETY: AVX2 is present, and the wide form runs only where `available512` is.
                unsafe {
                    assert_eq!(avx2::taken(op, &c, &a, &b), want[op as usize], "avx2 op {op} {c:08x?} {a:08x?} {b:08x?}");
                    if available512() {
                        assert_eq!(z::taken(op, &c, &a, &b), want[op as usize], "op {op} {c:08x?} {a:08x?} {b:08x?}");
                    }
                }
            }
        }
    }
}
