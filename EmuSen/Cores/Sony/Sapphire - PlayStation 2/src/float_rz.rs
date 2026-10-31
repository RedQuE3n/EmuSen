//! `float::add`, `mul` and `fused` on the host's floats with MXCSR set to `MODE` (rounding towards zero, denormals read
//! and written as zero), bit for bit with the MAC flags, eight lanes a register; a kernel for the VU recompiler and the
//! `lanes` bench. The adder cuts the smaller operand at the larger's guard bit with one magic constant, the multiplier
//! takes the host's product and sends the lanes the Booth loss can reach to `float::mul`; a field of 255 or an overflow
//! sends the vector to `float_lanes`. The argument, the proofs and the timing: Sapphire_Redesign.md, "The VU's floats
//! under rounding towards zero".

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::float;
use crate::float_lanes::{FusedPair, FusedVector, Pair, Vector};

const SIGN: u32 = 0x8000_0000;
const MAX: u32 = 0x7FFF_FFFF;
const EXP: u32 = 0x7F80_0000;
const FRAC: u32 = 0x007F_FFFF;
const HIDDEN: u32 = 0x0080_0000;
/// Four lanes' bits reversed: lane 0 at bit 0 to the VU's lane 0 at bit 3, and back.
const REV: [u32; 16] = [0, 8, 4, 12, 2, 10, 6, 14, 1, 9, 5, 13, 3, 11, 7, 15];

/// MXCSR as every function here requires it: exceptions masked, rounding towards zero, denormals as zero in and out.
pub const MODE: u32 = 0x1F80 | 0x6000 | 0x0040 | 0x8000;

/// Whether the host runs this kernel.
pub fn available() -> bool {
    crate::float_lanes::available()
}

/// Sets MXCSR to `MODE` and returns the old word. Unsafe: the thread's other float arithmetic reads the mode too.
#[cfg(target_arch = "x86_64")]
pub unsafe fn enter() -> u32 {
    let (mut old, mode) = (0u32, MODE);
    // SAFETY: both pointers are to locals; the caller owns the thread's float environment until `leave`.
    unsafe { std::arch::asm!("stmxcsr [{0}]", "ldmxcsr [{1}]", in(reg) &mut old, in(reg) &mode, options(nostack)) };
    old
}

/// Restores the word `enter` returned.
#[cfg(target_arch = "x86_64")]
pub unsafe fn leave(old: u32) {
    // SAFETY: a pointer to a local.
    unsafe { std::arch::asm!("ldmxcsr [{0}]", in(reg) &old, options(nostack)) };
}

/// The MAC groups' mask for the field mask's lanes (first operation's bits 3-0, a second's 19-16).
const fn mac_mask(mask: u32) -> u32 {
    ((mask & 0xF) * 0x1111) | (((mask >> 16 & 0xF) * 0x1111) << 16)
}

/// The kernel on eight lanes, each function a straight line of instructions a recompiler can emit as it stands.
#[cfg(target_arch = "x86_64")]
pub mod raw {
    use super::*;

    #[target_feature(enable = "avx2")]
    #[inline]
    pub fn k(x: u32) -> __m256i {
        _mm256_set1_epi32(x as i32)
    }
    #[target_feature(enable = "avx2")]
    #[inline]
    fn f(v: __m256i) -> __m256 {
        _mm256_castsi256_ps(v)
    }
    #[target_feature(enable = "avx2")]
    #[inline]
    fn i(v: __m256) -> __m256i {
        _mm256_castps_si256(v)
    }
    #[target_feature(enable = "avx2")]
    #[inline]
    fn and(a: __m256i, b: __m256i) -> __m256i {
        _mm256_and_si256(a, b)
    }
    #[target_feature(enable = "avx2")]
    #[inline]
    fn pick(a: __m256i, b: __m256i, m: __m256i) -> __m256i {
        i(_mm256_blendv_ps(f(a), f(b), f(m)))
    }

    /// A lane whose field is 255 or whose result the host saturated: left to the integer form.
    #[target_feature(enable = "avx2")]
    #[inline]
    fn trouble(r: __m256i) -> __m256i {
        _mm256_cmpgt_epi32(and(r, k(MAX)), k(0x7F7F_FFFE))
    }

    /// `float::add`: the bits, and the lanes left to the integer form.
    #[target_feature(enable = "avx2")]
    #[inline]
    pub fn add(a: __m256i, b: __m256i) -> (__m256i, __m256i) {
        let big = i(_mm256_cmp_ps::<_CMP_GT_OQ>(f(and(b, k(MAX))), f(and(a, k(MAX)))));
        let (l, s) = (pick(a, b, big), pick(b, a, big));
        let c = _mm256_sub_epi32(and(l, k(EXP)), k(HIDDEN));
        let cut = i(_mm256_cmp_ps::<_CMP_LT_OQ>(f(and(s, k(MAX))), f(c)));
        let c = f(_mm256_or_si256(c, and(s, k(SIGN))));
        let cut_s = _mm256_sub_ps(_mm256_add_ps(f(s), c), c);
        let r = i(_mm256_add_ps(f(l), _mm256_blendv_ps(f(s), cut_s, f(cut))));
        (r, trouble(r))
    }

    /// `float::mul`, `b` recoded: the bits, the lanes `fix` must recompute, and the lanes left to the integer form.
    #[target_feature(enable = "avx2")]
    #[inline]
    pub fn mul(a: __m256i, b: __m256i) -> (__m256i, __m256i, __m256i) {
        let p = i(_mm256_mul_ps(f(a), f(b)));
        let x = _mm256_or_si256(and(a, k(FRAC)), k(HIDDEN));
        let y = _mm256_or_si256(and(b, k(FRAC)), k(HIDDEN));
        let low = _mm256_mullo_epi32(x, y);
        let ep = and(p, k(EXP));
        let top = _mm256_cmpeq_epi32(ep, _mm256_sub_epi32(_mm256_add_epi32(and(a, k(EXP)), and(b, k(EXP))), k(126 << 23)));
        let kept = and(low, pick(k(FRAC), k(FRAC | HIDDEN), top));
        let near = _mm256_cmpgt_epi32(k(float::MOST_LOST as u32), kept);
        let quiet = _mm256_cmpeq_epi32(and(b, k(float::READ as u32)), _mm256_setzero_si256());
        let zero = _mm256_cmpeq_epi32(ep, _mm256_setzero_si256());
        // A short multiplicand loses 2^k a negative digit: one unit low where those digits outnumber the kept bits' top (§59.2).
        let short = _mm256_cmpeq_epi32(and(a, k((1 << float::K) - 1)), _mm256_setzero_si256());
        let low_one = _mm256_cmpgt_epi32(crate::float_lanes::negatives(b), _mm256_srli_epi32::<{ float::K as i32 }>(kept));
        let fix = _mm256_andnot_si256(_mm256_or_si256(_mm256_or_si256(quiet, zero), short), near);
        let trouble = trouble(p);
        (_mm256_add_epi32(p, _mm256_andnot_si256(zero, and(short, low_one))), fix, trouble)
    }

    /// The lanes of `fix` recomputed by `float::mul`; a branch the stream takes rarely.
    #[target_feature(enable = "avx2")]
    #[inline]
    pub fn fixed(p: __m256i, a: __m256i, b: __m256i, fix: __m256i) -> __m256i {
        let mut m = _mm256_movemask_ps(f(fix)) as u32;
        if m == 0 {
            return p;
        }
        let (mut pa, mut aa, mut ba) = ([0u32; 8], [0u32; 8], [0u32; 8]);
        // SAFETY: each array holds eight lanes.
        unsafe {
            _mm256_storeu_si256(pa.as_mut_ptr().cast(), p);
            _mm256_storeu_si256(aa.as_mut_ptr().cast(), a);
            _mm256_storeu_si256(ba.as_mut_ptr().cast(), b);
        }
        while m != 0 {
            let l = m.trailing_zeros() as usize;
            pa[l] = float::mul(aa[l], ba[l]).bits;
            m &= m - 1;
        }
        // SAFETY: as above.
        unsafe { _mm256_loadu_si256(pa.as_ptr().cast()) }
    }

    /// `float::fused` on the lanes `live` (the field mask's): the sum, the term as added, and the lanes left to the integer form.
    #[target_feature(enable = "avx2")]
    #[inline]
    pub fn fused(acc: __m256i, a: __m256i, b: __m256i, subtract: bool, live: __m256i) -> (__m256i, __m256i, __m256i) {
        let (p, fix, pt) = mul(a, b);
        let p = fixed(p, a, b, and(fix, live));
        let term = if subtract { _mm256_xor_si256(p, k(SIGN)) } else { p };
        let (r, st) = add(acc, term);
        (r, term, and(_mm256_or_si256(pt, st), live))
    }

    /// The MAC word's Z, S and U groups of the four lanes from lane `at`, for a result `r` of operands `a` and `b`: U is
    /// a zero result of two non-zero operands, and for a sum (`sum`) not an exact cancellation.
    #[target_feature(enable = "avx2")]
    #[inline]
    pub fn mac(r: __m256i, a: __m256i, b: __m256i, at: u32, sum: bool) -> u32 {
        let zero = _mm256_cmpeq_epi32(and(r, k(EXP)), _mm256_setzero_si256());
        let z = _mm256_movemask_ps(f(zero)) as u32;
        let s = _mm256_movemask_ps(f(r)) as u32;
        let live = and(_mm256_cmpgt_epi32(and(a, k(EXP)), _mm256_setzero_si256()), _mm256_cmpgt_epi32(and(b, k(EXP)), _mm256_setzero_si256()));
        let live = if sum { _mm256_andnot_si256(_mm256_cmpeq_epi32(a, _mm256_xor_si256(b, k(SIGN))), live) } else { live };
        let u = _mm256_movemask_ps(f(and(zero, live))) as u32;
        word(z >> at, s >> at, u >> at)
    }

    fn word(z: u32, s: u32, u: u32) -> u32 {
        REV[(z & 0xF) as usize] | REV[(s & 0xF) as usize] << 4 | REV[(u & 0xF) as usize] << 8
    }

    /// The register's lanes `mask` writes, as a blend mask: the first operation's bits 3-0 to lanes 0-3, a second's 19-16 to 4-7.
    #[target_feature(enable = "avx2")]
    #[inline]
    pub fn on(mask: u32) -> __m256i {
        let bits = _mm256_setr_epi32(8, 4, 2, 1, 8 << 16, 4 << 16, 2 << 16, 1 << 16);
        _mm256_cmpgt_epi32(and(k(mask), bits), _mm256_setzero_si256())
    }

    /// The lanes of a mask as bits, lane 0 at bit 0.
    #[target_feature(enable = "avx2")]
    #[inline]
    pub fn bits(m: __m256i) -> u32 {
        _mm256_movemask_ps(f(m)) as u32
    }

    /// Whether any lane of `m` is set.
    #[target_feature(enable = "avx2")]
    #[inline]
    pub fn any(m: __m256i) -> bool {
        bits(m) != 0
    }
}

macro_rules! entry {
    ($n:literal, $out:ident, $fused_out:ident, $add:path, $mul:path, $fused:path, $load:ident, $store:ident) => {
        #[target_feature(enable = "avx2")]
        #[inline]
        pub(super) fn get(v: &[u32; $n]) -> __m256i {
            // SAFETY: `v` holds the register's lanes.
            unsafe { $load(v.as_ptr().cast()) }
        }
        #[target_feature(enable = "avx2")]
        #[inline]
        fn finish(bits: __m256i, mac: u32, old: &[u32; $n], mask: u32) -> $out {
            let mut r = [0u32; $n];
            // SAFETY: `r` holds the register's lanes.
            unsafe { $store(r.as_mut_ptr().cast(), _mm256_blendv_ps(_mm256_castsi256_ps(get(old)), _mm256_castsi256_ps(bits), _mm256_castsi256_ps(raw::on(mask)))) };
            $out { bits: r, mac: mac & mac_mask(mask) }
        }
        /// `float::add`, `mask`'s lanes written over `old`; MXCSR must be `MODE`. A lane the mask leaves is never a reason to
        /// take the integer form: its flags are not read (`vu::ops::lanes`).
        #[target_feature(enable = "avx2")]
        pub fn add(a: &[u32; $n], b: &[u32; $n], old: &[u32; $n], mask: u32) -> $out {
            let (x, y) = (get(a), get(b));
            let (r, trouble) = raw::add(x, y);
            if raw::any(_mm256_and_si256(trouble, raw::on(mask))) {
                return $add(a, b, old, mask);
            }
            finish(r, raw::mac(r, x, y, 0, true) | if $n == 8 { raw::mac(r, x, y, 4, true) << 16 } else { 0 }, old, mask)
        }
        /// `float::sub`.
        #[target_feature(enable = "avx2")]
        pub fn sub(a: &[u32; $n], b: &[u32; $n], old: &[u32; $n], mask: u32) -> $out {
            add(a, &b.map(|x| x ^ SIGN), old, mask)
        }
        /// `float::mul`.
        #[target_feature(enable = "avx2")]
        pub fn mul(a: &[u32; $n], b: &[u32; $n], old: &[u32; $n], mask: u32) -> $out {
            let (x, y) = (get(a), get(b));
            let (p, fix, trouble) = raw::mul(x, y);
            let live = raw::on(mask);
            if raw::any(_mm256_and_si256(trouble, live)) {
                return $mul(a, b, old, mask);
            }
            let p = raw::fixed(p, x, y, _mm256_and_si256(fix, live));
            finish(p, raw::mac(p, x, y, 0, false) | if $n == 8 { raw::mac(p, x, y, 4, false) << 16 } else { 0 }, old, mask)
        }
        /// `float::fused`, with the term's own flags.
        #[target_feature(enable = "avx2")]
        pub fn fused(acc: &[u32; $n], a: &[u32; $n], b: &[u32; $n], subtract: bool, old: &[u32; $n], mask: u32) -> $fused_out {
            let (c, x, y) = (get(acc), get(a), get(b));
            let (r, term, trouble) = raw::fused(c, x, y, subtract, raw::on(mask));
            if raw::any(trouble) {
                return $fused(acc, a, b, subtract, old, mask);
            }
            let (mut vm, mut pm) = (raw::mac(r, c, term, 0, true), raw::mac(term, x, y, 0, false));
            if $n == 8 {
                vm |= raw::mac(r, c, term, 4, true) << 16;
                pm |= raw::mac(term, x, y, 4, false) << 16;
            }
            $fused_out { value: finish(r, vm, old, mask), product: finish(term, pm, old, mask) }
        }
    };
}

/// One vector of four lanes in the register's lower half. Safety, for every entry point: `available`, and MXCSR at `MODE`.
#[cfg(target_arch = "x86_64")]
#[allow(clippy::missing_safety_doc)]
pub mod four {
    use super::*;
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn load(p: *const __m128i) -> __m256i {
        // SAFETY: the caller's pointer holds four lanes.
        _mm256_zextsi128_si256(unsafe { _mm_loadu_si128(p) })
    }
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn store(p: *mut __m128i, v: __m256) {
        // SAFETY: the caller's pointer holds four lanes.
        unsafe { _mm_storeu_si128(p, _mm256_castsi256_si128(_mm256_castps_si256(v))) }
    }
    entry!(4, Vector, FusedVector, crate::float_lanes::add, crate::float_lanes::mul, crate::float_lanes::fused, load, store);
}

/// Two operations in eight lanes, `mask`'s second operation in bits 19-16. Safety: as `four`.
#[cfg(target_arch = "x86_64")]
#[allow(clippy::missing_safety_doc)]
pub mod eight {
    use super::*;
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn load(p: *const __m256i) -> __m256i {
        // SAFETY: the caller's pointer holds eight lanes.
        unsafe { _mm256_loadu_si256(p) }
    }
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn store(p: *mut __m256i, v: __m256) {
        // SAFETY: the caller's pointer holds eight lanes.
        unsafe { _mm256_storeu_si256(p, _mm256_castps_si256(v)) }
    }
    entry!(8, Pair, FusedPair, crate::float_lanes::add2, crate::float_lanes::mul2, crate::float_lanes::fused2, load, store);
}

/// The lanes each operation keeps on the host path, lane 0 at bit 0: whether no lane is left to the integer form, and
/// which `fixed` recomputes (`mul`'s and the multiply-add's product).
#[cfg(target_arch = "x86_64")]
pub struct Taken {
    pub whole: bool,
    pub fixed: u32,
}

/// `Taken` for each operation on four lanes: 0 add, 1 mul, 2 multiply-add. Safety: as `four`.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub fn taken(op: u8, c: &[u32; 4], a: &[u32; 4], b: &[u32; 4]) -> Taken {
    let (c, a, b) = (four::get(c), four::get(a), four::get(b));
    let bits = |m: __m256i| raw::bits(m) & 0xF;
    match op {
        0 => Taken { whole: bits(raw::add(a, b).1) == 0, fixed: 0 },
        1 => {
            let (_, fix, t) = raw::mul(a, b);
            Taken { whole: bits(t) == 0, fixed: bits(fix) }
        }
        _ => {
            let (_, fix, _) = raw::mul(a, b);
            Taken { whole: bits(raw::fused(c, a, b, false, raw::k(!0)).2) == 0, fixed: bits(fix) }
        }
    }
}

#[cfg(all(test, target_arch = "x86_64"))]
mod tests {
    use super::*;
    use crate::float::Outcome;
    use crate::float::tests::stream;
    use crate::float_lanes::tests::{Paired, class_vectors, expected, random_lanes, reduced_pairs, three};

    /// Both widths against `float` lane by lane; counts the vectors kept whole and the lanes `fixed` recomputed.
    fn check(a: [u32; 4], b: [u32; 4], c: [u32; 4], old: [u32; 4], mask: u32, counts: &mut [u64; 4]) {
        let e = expected(a, b, c, old, mask);
        let Paired { pa, pb, pc, po, pm, .. } = e.paired;
        // SAFETY: the tests run where AVX2 is present, with MXCSR set by the caller.
        unsafe {
            for (op, n) in [(0u8, 0usize), (1, 1), (2, 2)] {
                let t = taken(op, &c, &a, &b);
                counts[n] += u64::from(t.whole);
                counts[3] += u64::from(t.fixed.count_ones());
            }
            assert_eq!(four::add(&a, &b, &old, mask), e.add, "add {a:08x?} {b:08x?}");
            assert_eq!(four::sub(&a, &b, &old, mask), e.sub, "sub {a:08x?} {b:08x?}");
            assert_eq!(four::mul(&a, &b, &old, mask), e.mul, "mul {a:08x?} {b:08x?}");
            assert_eq!(eight::add(&pa, &pb, &po, pm), e.add2, "eight add");
            assert_eq!(eight::sub(&pa, &pb, &po, pm), e.sub2, "eight sub");
            assert_eq!(eight::mul(&pa, &pb, &po, pm), e.mul2, "eight mul");
            for (i, subtract) in [false, true].into_iter().enumerate() {
                assert_eq!(four::fused(&c, &a, &b, subtract, &old, mask), e.fused[i], "fused {c:08x?} {a:08x?} {b:08x?} {subtract}");
                assert_eq!(eight::fused(&pc, &pa, &pb, subtract, &po, pm), e.fused2[i], "eight fused");
            }
        }
    }

    /// Runs `f` with MXCSR at `MODE`, restoring it after.
    fn in_mode(f: impl FnOnce()) {
        if !available() {
            return;
        }
        // SAFETY: the tests' own arithmetic is on integers; the mode is restored before returning.
        let old = unsafe { enter() };
        f();
        unsafe { leave(old) };
    }

    #[test]
    fn every_pair_of_the_reduced_set_equals_float_under_rounding_towards_zero() {
        in_mode(|| {
            let mut counts = [0; 4];
            reduced_pairs(true, |a, b, c, o, m| check(a, b, c, o, m, &mut counts));
            assert!(counts[..3].iter().all(|&t| t > 100_000) && counts[3] > 10_000, "kept whole and fixed: {counts:?}");
        });
    }

    #[test]
    fn four_million_vectors_of_every_class_equal_float_under_rounding_towards_zero() {
        in_mode(|| {
            let mut counts = [0; 4];
            class_vectors(true, |a, b, c, o, m| check(a, b, c, o, m, &mut counts));
            assert!(counts[..3].iter().all(|&t| t > 100_000) && counts[3] > 10_000, "kept whole and fixed: {counts:?}");
        });
    }

    #[test]
    #[ignore = "a billion lanes, minutes in release"]
    fn a_billion_lanes_equal_float_under_rounding_towards_zero() {
        in_mode(|| {
            let mut counts = [0; 4];
            random_lanes(|a, b, c, o, m| check(a, b, c, o, m, &mut counts));
            println!("kept whole and fixed: {counts:?}");
        });
    }

    /// Eight lanes of `raw::add` against `float::add`, the fallback lanes through `float_lanes` as the entry points send them.
    #[target_feature(enable = "avx2")]
    fn add_lanes(a: [u32; 8], b: [u32; 8]) {
        let (r, trouble) = raw::add(eight::get(&a), eight::get(&b));
        let mut got = [0u32; 8];
        // SAFETY: eight lanes.
        unsafe { _mm256_storeu_si256(got.as_mut_ptr().cast(), r) };
        let t = raw::bits(trouble);
        for l in 0..8 {
            let want = float::add(a[l], b[l]).bits;
            if t >> l & 1 == 0 {
                assert_eq!(got[l], want, "{:08x} + {:08x}", a[l], b[l]);
            }
        }
    }

    /// The adder over every fraction of the smaller operand (`step` apart) at every distance 0 to 26 from the larger, both
    /// signs and both orders, the larger's fraction random, zero or all ones: the cut and the guard bit at every alignment.
    fn cut_sweep(step: usize) {
        let mut next = stream(0xC0DE_D00D_0000_0007);
        for d in 0..=26u32 {
            for big_f in [0, FRAC, next() as u32 & FRAC, next() as u32 & FRAC] {
                let big = 0x3F80_0000 | big_f;
                let small_e = (127 - d) << 23;
                for f in (0..0x80_0000u32).step_by(step) {
                    let (s, n) = (small_e | f, SIGN | small_e | f);
                    // SAFETY: `in_mode` checked AVX2.
                    unsafe { add_lanes([big, s, big, n, big ^ SIGN, s, big ^ SIGN, n], [s, big, n, big, s, big ^ SIGN, n, big ^ SIGN]) };
                }
            }
        }
    }

    #[test]
    fn the_cut_is_exact_at_every_distance_over_a_sample_of_fractions() {
        in_mode(|| cut_sweep(13));
    }

    #[test]
    #[ignore = "every fraction at every distance, a minute in release"]
    fn the_cut_is_exact_at_every_distance_over_every_fraction() {
        in_mode(|| cut_sweep(1));
    }

    /// Every multiplier against six multiplicands (as `float`'s own proof of its closed form), eight lanes a step.
    #[test]
    fn every_multiplier_against_six_multiplicands_equals_float() {
        in_mode(|| {
            let mut next = stream(0x9E37_79B9_7F4A_7C15);
            let xs = [0x3F80_0000u32, 0x3F80_0001, 0x3FFF_FFFF, 0x3FFF_FFFE, 0x3F80_0000 | next() as u32 & FRAC, 0x3F80_0001 | next() as u32 & 0x7F_FFFE];
            let mut fixed = 0u64;
            for x in xs {
                for y0 in (0..0x80_0000u32).step_by(8) {
                    let a = [x; 8];
                    let b: [u32; 8] = std::array::from_fn(|l| 0x3F80_0000 | (y0 + l as u32));
                    // SAFETY: `in_mode` checked AVX2.
                    unsafe {
                        let (p, fix, trouble) = raw::mul(eight::get(&a), eight::get(&b));
                        assert_eq!(raw::bits(trouble), 0);
                        fixed += u64::from(raw::bits(fix).count_ones());
                        let mut got = [0u32; 8];
                        _mm256_storeu_si256(got.as_mut_ptr().cast(), raw::fixed(p, eight::get(&a), eight::get(&b), fix));
                        for l in 0..8 {
                            assert_eq!(got[l], float::mul(a[l], b[l]).bits, "{:08x} × {:08x}", a[l], b[l]);
                        }
                    }
                }
            }
            assert!(fixed > 100_000, "{fixed}");
        });
    }

    /// The vectors kept whole against the scalar rule: no field of 255, no overflow, no result at the host's largest finite.
    #[test]
    fn the_vectors_kept_whole_are_the_scalar_rule() {
        in_mode(|| {
            let mut next = stream(0xA11C_E5ED_0000_0060);
            let field = |x: u32| x >> 23 & 0xFF;
            let edge = |bits: u32| (bits & MAX) >= 0x7F7F_FFFE;
            for _ in 0..1_000_000 {
                let (a, b, c) = three(&mut next);
                let rule = |o: fn(u32, u32) -> Outcome| (0..4).all(|l| field(a[l]) != 255 && field(b[l]) != 255 && !o(a[l], b[l]).overflow && !edge(o(a[l], b[l]).bits));
                let near = |o: fn(u32, u32) -> Outcome| (0..4).any(|l| edge(o(a[l], b[l]).bits));
                // SAFETY: AVX2 present, the mode set.
                let (add, mul) = unsafe { (taken(0, &c, &a, &b), taken(1, &c, &a, &b)) };
                if !near(float::add) {
                    assert_eq!(add.whole, rule(float::add), "add {a:08x?} {b:08x?}");
                }
                if !near(float::mul) {
                    assert_eq!(mul.whole, rule(float::mul), "mul {a:08x?} {b:08x?}");
                }
            }
        });
    }
}
