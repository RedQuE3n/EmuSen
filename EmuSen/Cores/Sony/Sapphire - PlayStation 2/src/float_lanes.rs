//! The console's adder and multiplier in the host's vector registers, on integers, with the VU's MAC flags:
//! `float::add`, `float::mul` and `float::fused` for a whole vector bit for bit, or for two vectors at once (`*2`, eight
//! lanes, two independent operations). A measurement kernel behind the `lanes` feature, off by default and used by
//! nothing in the machine; Sapphire_Redesign.md, R0 results and their addendum, records the argument for each step, the
//! grading and the timing.
//!
//! x86-64 with AVX2 only (`available`): the per-lane shifts are AVX2's. Each entry point is a `target_feature` function,
//! to be called only where `available` is true; the register type `Lanes` wraps one or two AVX2 instructions a method
//! and is private to that use.
//!
//! The forms: the full form (`add`, `mul`, `fused` and their pairs); `*_bits`, the same without the MAC word, for
//! operations whose flags no later pair reads; and `*_fast`, the classes §56.2 found most common (no zero operand, no
//! exponent overflow or underflow) on a short path, any other lane sending the vector to the full form.

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

/// Two operations' results in eight lanes, the second's in lanes 4 to 7; their MAC words in bits 15-0 and 31-16.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pair {
    pub bits: [u32; 8],
    pub mac: u32,
}

/// Two multiply-adds in eight lanes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FusedPair {
    pub value: Pair,
    pub product: Pair,
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

/// The MAC word's four groups as the status flag's Z, S, U, O (bits 0 to 3).
pub fn collect(mac: u32) -> u32 {
    u32::from(mac & 0xF != 0) | u32::from(mac & 0xF0 != 0) << 1 | u32::from(mac & 0xF00 != 0) << 2 | u32::from(mac & 0xF000 != 0) << 3
}

/// A field mask's MAC bits: `mask`'s bits 3-0 for the first operation, 19-16 for a second.
const fn mac_mask(mask: u32) -> u32 {
    ((mask & 0xF) * 0x1111) | (((mask >> 16 & 0xF) * 0x1111) << 16)
}

/// A register of 32-bit lanes, four or eight; each method one or two AVX2 instructions, sound only where AVX2 is present.
#[cfg(target_arch = "x86_64")]
trait Lanes: Copy {
    fn k(x: i32) -> Self;
    fn and(self, b: Self) -> Self;
    fn or(self, b: Self) -> Self;
    fn xor(self, b: Self) -> Self;
    /// `!self & b`.
    fn andnot(self, b: Self) -> Self;
    fn add(self, b: Self) -> Self;
    fn sub(self, b: Self) -> Self;
    fn eq(self, b: Self) -> Self;
    fn gt(self, b: Self) -> Self;
    /// `b` where `m`'s lanes are set, `self` elsewhere.
    fn pick(self, b: Self, m: Self) -> Self;
    fn srl<const K: i32>(self) -> Self;
    fn sll<const K: i32>(self) -> Self;
    fn sra<const K: i32>(self) -> Self;
    fn srlv(self, c: Self) -> Self;
    fn sllv(self, c: Self) -> Self;
    /// The float conversion's biased exponent field of each lane, taken as a signed integer.
    fn exponent_of(self) -> Self;
    /// The place of each lane's leading bit, for a lane above zero (below 2^31).
    #[inline(always)]
    fn lead(self) -> Self {
        // With the bit below the leading one cleared, the conversion cannot round up a binade.
        self.srl::<1>().andnot(self).exponent_of().sub(Self::k(127))
    }
    /// `self | (b & c)`.
    #[inline(always)]
    fn or_and(self, b: Self, c: Self) -> Self {
        self.or(b.and(c))
    }
    /// `float::product`'s significand: (the product less `loss` shifted to 24 bits, its top bit at 47 as 0 or 1).
    fn product(x: Self, y: Self, loss: Self) -> (Self, Self);
    /// The MAC word of masks for Z, S, U and O, per operation of four lanes.
    fn mac(z: Self, s: Self, u: Self, o: Self) -> u32;
    /// The lanes `mask` writes (`mac_mask`'s layout).
    fn on(mask: u32) -> Self;
    fn any(self) -> bool;
    /// # Safety
    /// `p` reads or writes as many u32 as the type has lanes.
    unsafe fn load(p: *const u32) -> Self;
    unsafe fn store(self, p: *mut u32);
}

#[cfg(target_arch = "x86_64")]
macro_rules! lanes {
    ($t:ty, $set1:ident, $and:ident, $or:ident, $xor:ident, $andnot:ident, $add:ident, $sub:ident, $eq:ident, $gt:ident, $blendv:ident, $srli:ident, $slli:ident, $srai:ident, $srlv:ident, $sllv:ident, $cvt:ident, $cast:ident, $mul:ident, $sub64:ident, $add64:ident, $set64:ident, $srli64:ident, $slli64:ident, $srlv64:ident, $blend32:ident, $odd:expr, $packs32:ident, $packs16:ident, $shuffle:ident, $movemask:ident, $loadu:ident, $storeu:ident, $reverse:expr, $on:expr) => {
        // SAFETY (every method): the kernel's entry points run only where AVX2 is present (`available`).
        impl Lanes for $t {
            #[inline(always)]
            fn k(x: i32) -> Self { unsafe { $set1(x) } }
            #[inline(always)]
            fn and(self, b: Self) -> Self { unsafe { $and(self, b) } }
            #[inline(always)]
            fn or(self, b: Self) -> Self { unsafe { $or(self, b) } }
            #[inline(always)]
            fn xor(self, b: Self) -> Self { unsafe { $xor(self, b) } }
            #[inline(always)]
            fn andnot(self, b: Self) -> Self { unsafe { $andnot(self, b) } }
            #[inline(always)]
            fn add(self, b: Self) -> Self { unsafe { $add(self, b) } }
            #[inline(always)]
            fn sub(self, b: Self) -> Self { unsafe { $sub(self, b) } }
            #[inline(always)]
            fn eq(self, b: Self) -> Self { unsafe { $eq(self, b) } }
            #[inline(always)]
            fn gt(self, b: Self) -> Self { unsafe { $gt(self, b) } }
            #[inline(always)]
            fn pick(self, b: Self, m: Self) -> Self { unsafe { $blendv(self, b, m) } }
            #[inline(always)]
            fn srl<const K: i32>(self) -> Self { unsafe { $srli::<K>(self) } }
            #[inline(always)]
            fn sll<const K: i32>(self) -> Self { unsafe { $slli::<K>(self) } }
            #[inline(always)]
            fn sra<const K: i32>(self) -> Self { unsafe { $srai::<K>(self) } }
            #[inline(always)]
            fn srlv(self, c: Self) -> Self { unsafe { $srlv(self, c) } }
            #[inline(always)]
            fn sllv(self, c: Self) -> Self { unsafe { $sllv(self, c) } }
            #[inline(always)]
            fn exponent_of(self) -> Self { unsafe { $srli::<23>($cast($cvt(self))) } }
            #[inline(always)]
            fn product(x: Self, y: Self, loss: Self) -> (Self, Self) {
                unsafe {
                    let low = $set64(0xFFFF_FFFF);
                    let k23 = $set64(23);
                    let even = $sub64($mul(x, y), $and(loss, low));
                    let odd = $sub64($mul($srli64::<32>(x), $srli64::<32>(y)), $srli64::<32>(loss));
                    let (te, to) = ($srli64::<47>(even), $srli64::<47>(odd));
                    let (me, mo) = ($srlv64(even, $add64(te, k23)), $srlv64(odd, $add64(to, k23)));
                    ($blend32::<{ $odd }>(me, $slli64::<32>(mo)), $blend32::<{ $odd }>(te, $slli64::<32>(to)))
                }
            }
            #[inline(always)]
            fn mac(z: Self, s: Self, u: Self, o: Self) -> u32 {
                unsafe { $movemask($shuffle($packs16($packs32(z, s), $packs32(u, o)), $reverse)) as u32 }
            }
            #[inline(always)]
            fn on(mask: u32) -> Self {
                let m = mask as i32;
                unsafe { $gt($on(m), $set1(0)) }
            }
            #[inline(always)]
            fn any(self) -> bool { unsafe { $movemask(self) != 0 } }
            #[inline(always)]
            unsafe fn load(p: *const u32) -> Self { unsafe { $loadu(p.cast()) } }
            #[inline(always)]
            unsafe fn store(self, p: *mut u32) { unsafe { $storeu(p.cast(), self) } }
        }
    };
}

#[cfg(target_arch = "x86_64")]
lanes!(__m128i, _mm_set1_epi32, _mm_and_si128, _mm_or_si128, _mm_xor_si128, _mm_andnot_si128, _mm_add_epi32, _mm_sub_epi32,
    _mm_cmpeq_epi32, _mm_cmpgt_epi32, _mm_blendv_epi8, _mm_srli_epi32, _mm_slli_epi32, _mm_srai_epi32, _mm_srlv_epi32,
    _mm_sllv_epi32, _mm_cvtepi32_ps, _mm_castps_si128, _mm_mul_epu32, _mm_sub_epi64, _mm_add_epi64, _mm_set1_epi64x,
    _mm_srli_epi64, _mm_slli_epi64, _mm_srlv_epi64, _mm_blend_epi32, 0b1010, _mm_packs_epi32, _mm_packs_epi16,
    _mm_shuffle_epi8, _mm_movemask_epi8, _mm_loadu_si128, _mm_storeu_si128,
    _mm_setr_epi8(3, 2, 1, 0, 7, 6, 5, 4, 11, 10, 9, 8, 15, 14, 13, 12),
    |m: i32| _mm_setr_epi32(m & 8, m & 4, m & 2, m & 1));
#[cfg(target_arch = "x86_64")]
lanes!(__m256i, _mm256_set1_epi32, _mm256_and_si256, _mm256_or_si256, _mm256_xor_si256, _mm256_andnot_si256, _mm256_add_epi32,
    _mm256_sub_epi32, _mm256_cmpeq_epi32, _mm256_cmpgt_epi32, _mm256_blendv_epi8, _mm256_srli_epi32, _mm256_slli_epi32,
    _mm256_srai_epi32, _mm256_srlv_epi32, _mm256_sllv_epi32, _mm256_cvtepi32_ps, _mm256_castps_si256, _mm256_mul_epu32,
    _mm256_sub_epi64, _mm256_add_epi64, _mm256_set1_epi64x, _mm256_srli_epi64, _mm256_slli_epi64, _mm256_srlv_epi64,
    _mm256_blend_epi32, 0b1010_1010, _mm256_packs_epi32, _mm256_packs_epi16, _mm256_shuffle_epi8, _mm256_movemask_epi8,
    _mm256_loadu_si256, _mm256_storeu_si256,
    _mm256_setr_epi8(3, 2, 1, 0, 7, 6, 5, 4, 11, 10, 9, 8, 15, 14, 13, 12, 3, 2, 1, 0, 7, 6, 5, 4, 11, 10, 9, 8, 15, 14, 13, 12),
    |m: i32| _mm256_setr_epi32(m & 8, m & 4, m & 2, m & 1, m >> 16 & 8, m >> 16 & 4, m >> 16 & 2, m >> 16 & 1));

/// The AVX-512VL form's registers: the AVX2 form's, with the leading bit from `vplzcntd` and `a | (b & c)` from
/// `vpternlogd`.
#[cfg(target_arch = "x86_64")]
#[derive(Clone, Copy)]
struct Z4(__m128i);
#[cfg(target_arch = "x86_64")]
#[derive(Clone, Copy)]
struct Z8(__m256i);

#[cfg(target_arch = "x86_64")]
macro_rules! wide_lanes {
    ($z:ident, $t:ty, $lzcnt:ident, $ternlog:ident) => {
        // SAFETY (every method): the AVX-512VL entry points run only where `available512` is true.
        impl Lanes for $z {
            #[inline(always)] fn k(x: i32) -> Self { $z(<$t>::k(x)) }
            #[inline(always)] fn and(self, b: Self) -> Self { $z(self.0.and(b.0)) }
            #[inline(always)] fn or(self, b: Self) -> Self { $z(self.0.or(b.0)) }
            #[inline(always)] fn xor(self, b: Self) -> Self { $z(self.0.xor(b.0)) }
            #[inline(always)] fn andnot(self, b: Self) -> Self { $z(self.0.andnot(b.0)) }
            #[inline(always)] fn add(self, b: Self) -> Self { $z(self.0.add(b.0)) }
            #[inline(always)] fn sub(self, b: Self) -> Self { $z(self.0.sub(b.0)) }
            #[inline(always)] fn eq(self, b: Self) -> Self { $z(self.0.eq(b.0)) }
            #[inline(always)] fn gt(self, b: Self) -> Self { $z(self.0.gt(b.0)) }
            #[inline(always)] fn pick(self, b: Self, m: Self) -> Self { $z(self.0.pick(b.0, m.0)) }
            #[inline(always)] fn srl<const K: i32>(self) -> Self { $z(self.0.srl::<K>()) }
            #[inline(always)] fn sll<const K: i32>(self) -> Self { $z(self.0.sll::<K>()) }
            #[inline(always)] fn sra<const K: i32>(self) -> Self { $z(self.0.sra::<K>()) }
            #[inline(always)] fn srlv(self, c: Self) -> Self { $z(self.0.srlv(c.0)) }
            #[inline(always)] fn sllv(self, c: Self) -> Self { $z(self.0.sllv(c.0)) }
            #[inline(always)] fn exponent_of(self) -> Self { $z(self.0.exponent_of()) }
            #[inline(always)] fn lead(self) -> Self { $z(<$t>::k(31).sub(unsafe { $lzcnt(self.0) })) }
            #[inline(always)] fn or_and(self, b: Self, c: Self) -> Self { $z(unsafe { $ternlog::<0xF8>(self.0, b.0, c.0) }) }
            #[inline(always)] fn product(x: Self, y: Self, loss: Self) -> (Self, Self) { let (m, t) = <$t>::product(x.0, y.0, loss.0); ($z(m), $z(t)) }
            #[inline(always)] fn mac(z: Self, s: Self, u: Self, o: Self) -> u32 { <$t>::mac(z.0, s.0, u.0, o.0) }
            #[inline(always)] fn on(mask: u32) -> Self { $z(<$t>::on(mask)) }
            #[inline(always)] fn any(self) -> bool { self.0.any() }
            #[inline(always)] unsafe fn load(p: *const u32) -> Self { $z(unsafe { <$t>::load(p) }) }
            #[inline(always)] unsafe fn store(self, p: *mut u32) { unsafe { self.0.store(p) } }
        }
    };
}
#[cfg(target_arch = "x86_64")]
wide_lanes!(Z4, __m128i, _mm_lzcnt_epi32, _mm_ternarylogic_epi32);
#[cfg(target_arch = "x86_64")]
wide_lanes!(Z8, __m256i, _mm256_lzcnt_epi32, _mm256_ternarylogic_epi32);

/// Whether the host runs the AVX-512VL form (`*_512`).
pub fn available512() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        std::arch::is_x86_feature_detected!("avx512vl") && std::arch::is_x86_feature_detected!("avx512cd") && std::arch::is_x86_feature_detected!("avx2")
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}

const SIGN: i32 = i32::MIN;

/// A result's Z and S masks: an exponent field of 0, the sign bit.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn zs<T: Lanes>(bits: T) -> (T, T) {
    (bits.and(T::k(0x7F80_0000)).eq(T::k(0)), bits.sra::<31>())
}

/// `float::finish`: the sign (bit 31 alone), the biased exponent (signed), a 24-bit significand; the bits, overflow and
/// underflow masks.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn finish<T: Lanes>(s: T, e: T, m: T) -> (T, T, T) {
    let over = e.gt(T::k(255));
    let under = T::k(1).gt(e);
    let packed = s.or(e.sll::<23>()).or_and(m, T::k(0x7F_FFFF));
    let bits = packed.pick(s.or(T::k(0x7FFF_FFFF)), over).pick(s, under);
    (bits, over, under)
}

/// The adder's general path for lanes whose operands are both non-zero: bits, overflow, underflow, and the lanes whose
/// sum cancelled to zero.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn add_core<T: Lanes>(a: T, b: T, ea: T, eb: T) -> (T, T, T, T) {
    let max = T::k(0x7FFF_FFFF);
    // The larger magnitude: `a` unless `b`'s is strictly greater (both below 2^31, so a signed compare).
    let b_big = b.and(max).gt(a.and(max));
    let (big, small) = (a.pick(b, b_big), b.pick(a, b_big));
    let e_big = ea.pick(eb, b_big);
    let d = e_big.sub(eb.pick(ea, b_big));
    let (hidden, frac) = (T::k(0x80_0000), T::k(0x7F_FFFF));
    let wide = hidden.or_and(big, frac).sll::<1>();
    // A count of 32 or more gives 0, so the shift needs no test of `d < 25`: a 25-bit value is 0 from 25 on.
    let aligned = hidden.or_and(small, frac).sll::<1>().srlv(d);
    let opposite = big.xor(small).sra::<31>();
    let m = wide.add(aligned).pick(wide.sub(aligned), opposite);
    let lead = m.lead();
    let k23 = T::k(23);
    let norm = m.srlv(lead.sub(k23)).or(m.sllv(k23.sub(lead)));
    let e = e_big.add(lead.sub(T::k(24)));
    let (bits, over, under) = finish(big.and(T::k(SIGN)), e, norm);
    let cancelled = m.eq(T::k(0));
    (cancelled.andnot(bits), over, under, cancelled)
}

/// `float::add` in lanes: the bits, overflow and underflow masks.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn add_raw<T: Lanes>(a: T, b: T) -> (T, T, T) {
    let field = T::k(0xFF);
    let (ea, eb) = (a.srl::<23>().and(field), b.srl::<23>().and(field));
    let (za, zb) = (ea.eq(T::k(0)), eb.eq(T::k(0)));
    let (bits, over, under, cancelled) = add_core(a, b, ea, eb);
    let none = za.or(zb).or(cancelled);
    // A zero operand gives the other; two zeros give the signs' conjunction as -0 or +0.
    let bits = bits.pick(b, za).pick(a, za.andnot(zb)).pick(a.and(b).and(T::k(SIGN)), za.and(zb));
    (bits, none.andnot(over), none.andnot(under))
}

/// `add_raw` where no lane has a zero operand, overflows or underflows; `None` when one does.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn add_fast_raw<T: Lanes>(a: T, b: T) -> Option<T> {
    let field = T::k(0xFF);
    let (ea, eb) = (a.srl::<23>().and(field), b.srl::<23>().and(field));
    let (bits, over, under, _) = add_core(a, b, ea, eb);
    if ea.eq(T::k(0)).or(eb.eq(T::k(0))).or(over).or(under).any() { None } else { Some(bits) }
}

/// `float::mul` in lanes, `b` the recoded operand: the bits, overflow and underflow masks.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn mul_raw<T: Lanes>(a: T, b: T) -> (T, T, T) {
    let field = T::k(0xFF);
    let (ea, eb) = (a.srl::<23>().and(field), b.srl::<23>().and(field));
    let none = ea.eq(T::k(0)).or(eb.eq(T::k(0)));
    let s = a.xor(b).and(T::k(SIGN));
    let (hidden, frac) = (T::k(0x80_0000), T::k(0x7F_FFFF));
    let (x, y) = (hidden.or_and(a, frac), hidden.or_and(b, frac));
    // The Booth sum's loss (float::product): eight two-bit answers by y's lowest two bits and x's lowest.
    let index = y.and(T::k(3)).sll::<1>().or(x.and(T::k(1))).sll::<2>();
    let loss = T::k(0x1222_1000).srlv(index).and(T::k(3));
    let (m, top) = T::product(x, y, loss);
    let (bits, over, under) = finish(s, ea.add(eb).add(top.sub(T::k(127))), m);
    (bits.pick(s, none), none.andnot(over), none.andnot(under))
}

/// `float::fused`: (sum bits, overflow, underflow, term bits, product overflow, product underflow).
#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn fused_raw<T: Lanes>(acc: T, a: T, b: T, subtract: bool) -> (T, T, T, T, T, T) {
    let (p, p_over, p_under) = mul_raw(a, b);
    let term = if subtract { p.xor(T::k(SIGN)) } else { p };
    let (sum, s_over, s_under) = add_raw(acc, term);
    let top = T::k(0x7F80_0000);
    let acc_top = acc.and(top).eq(top);
    // An overflowing product gives its term; else an accumulator at field 255 is kept; both with overflow.
    let kept = p_over.andnot(acc_top);
    let bits = sum.pick(acc, kept).pick(term, p_over);
    let special = p_over.or(kept);
    (bits, special.or(s_over), special.andnot(s_under), term, p_over, p_under)
}

/// The full form's result: the lanes `mask` writes taken over `old`, and the MAC word they set.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn flagged<T: Lanes>(bits: T, over: T, under: T, old: T, mask: u32) -> (T, u32) {
    let (z, s) = zs(bits);
    (old.pick(bits, T::on(mask)), T::mac(z, s, under, over) & mac_mask(mask))
}

macro_rules! entry {
    ($t:ty, $n:literal, $out:ident, $fused_out:ident) => {
        #[inline(always)]
        fn out(v: $t) -> [u32; $n] {
            let mut r = [0u32; $n];
            // SAFETY: `r` holds the type's lanes.
            unsafe { v.store(r.as_mut_ptr()) };
            r
        }
        #[inline(always)]
        fn get(v: &[u32; $n]) -> $t {
            // SAFETY: `v` holds the type's lanes.
            unsafe { <$t>::load(v.as_ptr()) }
        }
        #[inline(always)]
        fn arith((bits, over, under): ($t, $t, $t), old: &[u32; $n], mask: u32) -> $out {
            let (bits, mac) = flagged(bits, over, under, get(old), mask);
            $out { bits: out(bits), mac }
        }
        #[inline(always)]
        fn fused_of(acc: &[u32; $n], a: &[u32; $n], b: &[u32; $n], subtract: bool, old: &[u32; $n], mask: u32) -> $fused_out {
            let (bits, over, under, term, p_over, p_under) = fused_raw(get(acc), get(a), get(b), subtract);
            let old = get(old);
            let (v, vm) = flagged(bits, over, under, old, mask);
            let (p, pm) = flagged(term, p_over, p_under, old, mask);
            $fused_out { value: $out { bits: out(v), mac: vm }, product: $out { bits: out(p), mac: pm } }
        }
        #[inline(always)]
        fn bits_of(bits: $t, old: &[u32; $n], mask: u32) -> [u32; $n] {
            out(get(old).pick(bits, <$t>::on(mask)))
        }
        #[inline(always)]
        fn negated(b: &[u32; $n]) -> $t {
            get(b).xor(<$t>::k(SIGN))
        }
    };
}

/// Safety, for every entry point: the host has AVX2 (`available`).
#[cfg(target_arch = "x86_64")]
#[allow(clippy::missing_safety_doc)]
mod four {
    use super::*;
    entry!(__m128i, 4, Vector, FusedVector);

    /// `float::add` on four lanes, written where `mask` (lane 0 at bit 3) allows over `old`.
    #[target_feature(enable = "avx2")]
    pub fn add(a: &[u32; 4], b: &[u32; 4], old: &[u32; 4], mask: u32) -> Vector {
        arith(add_raw(get(a), get(b)), old, mask)
    }
    /// `float::sub`: the addition of `b` with its signs reversed.
    #[target_feature(enable = "avx2")]
    pub fn sub(a: &[u32; 4], b: &[u32; 4], old: &[u32; 4], mask: u32) -> Vector {
        arith(add_raw(get(a), negated(b)), old, mask)
    }
    /// `float::mul`.
    #[target_feature(enable = "avx2")]
    pub fn mul(a: &[u32; 4], b: &[u32; 4], old: &[u32; 4], mask: u32) -> Vector {
        arith(mul_raw(get(a), get(b)), old, mask)
    }
    /// `float::fused`: `acc ± a × b`, with the products' own flags for the sticky bits.
    #[target_feature(enable = "avx2")]
    pub fn fused(acc: &[u32; 4], a: &[u32; 4], b: &[u32; 4], subtract: bool, old: &[u32; 4], mask: u32) -> FusedVector {
        fused_of(acc, a, b, subtract, old, mask)
    }
    /// `add` without the MAC word.
    #[target_feature(enable = "avx2")]
    pub fn add_bits(a: &[u32; 4], b: &[u32; 4], old: &[u32; 4], mask: u32) -> [u32; 4] {
        bits_of(add_raw(get(a), get(b)).0, old, mask)
    }
    /// `mul` without the MAC word.
    #[target_feature(enable = "avx2")]
    pub fn mul_bits(a: &[u32; 4], b: &[u32; 4], old: &[u32; 4], mask: u32) -> [u32; 4] {
        bits_of(mul_raw(get(a), get(b)).0, old, mask)
    }
    /// `add` on the short path when every lane takes it, otherwise the full form.
    #[target_feature(enable = "avx2")]
    pub fn add_fast(a: &[u32; 4], b: &[u32; 4], old: &[u32; 4], mask: u32) -> Vector {
        match add_fast_raw(get(a), get(b)) {
            Some(bits) => {
                let (bits, mac) = flagged(bits, <__m128i>::k(0), <__m128i>::k(0), get(old), mask);
                Vector { bits: out(bits), mac }
            }
            None => add(a, b, old, mask),
        }
    }
}

/// Safety, for every entry point: the host has AVX2 (`available`).
#[cfg(target_arch = "x86_64")]
#[allow(clippy::missing_safety_doc)]
mod eight {
    use super::*;
    entry!(__m256i, 8, Pair, FusedPair);

    /// Two independent `float::add`s, the second in lanes 4 to 7, `mask` the first's in bits 3-0 and the second's in
    /// bits 19-16.
    #[target_feature(enable = "avx2")]
    pub fn add2(a: &[u32; 8], b: &[u32; 8], old: &[u32; 8], mask: u32) -> Pair {
        arith(add_raw(get(a), get(b)), old, mask)
    }
    #[target_feature(enable = "avx2")]
    pub fn sub2(a: &[u32; 8], b: &[u32; 8], old: &[u32; 8], mask: u32) -> Pair {
        arith(add_raw(get(a), negated(b)), old, mask)
    }
    #[target_feature(enable = "avx2")]
    pub fn mul2(a: &[u32; 8], b: &[u32; 8], old: &[u32; 8], mask: u32) -> Pair {
        arith(mul_raw(get(a), get(b)), old, mask)
    }
    #[target_feature(enable = "avx2")]
    pub fn fused2(acc: &[u32; 8], a: &[u32; 8], b: &[u32; 8], subtract: bool, old: &[u32; 8], mask: u32) -> FusedPair {
        fused_of(acc, a, b, subtract, old, mask)
    }
    #[target_feature(enable = "avx2")]
    pub fn add2_bits(a: &[u32; 8], b: &[u32; 8], old: &[u32; 8], mask: u32) -> [u32; 8] {
        bits_of(add_raw(get(a), get(b)).0, old, mask)
    }
    #[target_feature(enable = "avx2")]
    pub fn mul2_bits(a: &[u32; 8], b: &[u32; 8], old: &[u32; 8], mask: u32) -> [u32; 8] {
        bits_of(mul_raw(get(a), get(b)).0, old, mask)
    }
}

/// The AVX-512VL form of the four-lane and paired entry points, the same results bit for bit.
#[cfg(target_arch = "x86_64")]
#[allow(clippy::missing_safety_doc)]
pub mod wide {
    use super::*;
    pub mod four {
        use super::*;
        entry!(Z4, 4, Vector, FusedVector);
        #[target_feature(enable = "avx2,avx512f,avx512vl,avx512cd")]
        pub fn add(a: &[u32; 4], b: &[u32; 4], old: &[u32; 4], mask: u32) -> Vector {
            arith(add_raw(get(a), get(b)), old, mask)
        }
        #[target_feature(enable = "avx2,avx512f,avx512vl,avx512cd")]
        pub fn sub(a: &[u32; 4], b: &[u32; 4], old: &[u32; 4], mask: u32) -> Vector {
            arith(add_raw(get(a), negated(b)), old, mask)
        }
        #[target_feature(enable = "avx2,avx512f,avx512vl,avx512cd")]
        pub fn mul(a: &[u32; 4], b: &[u32; 4], old: &[u32; 4], mask: u32) -> Vector {
            arith(mul_raw(get(a), get(b)), old, mask)
        }
        #[target_feature(enable = "avx2,avx512f,avx512vl,avx512cd")]
        pub fn fused(acc: &[u32; 4], a: &[u32; 4], b: &[u32; 4], subtract: bool, old: &[u32; 4], mask: u32) -> FusedVector {
            fused_of(acc, a, b, subtract, old, mask)
        }
        #[target_feature(enable = "avx2,avx512f,avx512vl,avx512cd")]
        pub fn add_bits(a: &[u32; 4], b: &[u32; 4], old: &[u32; 4], mask: u32) -> [u32; 4] {
            bits_of(add_raw(get(a), get(b)).0, old, mask)
        }
    }
    #[allow(dead_code)]
    pub mod eight {
        use super::*;
        entry!(Z8, 8, Pair, FusedPair);
        #[target_feature(enable = "avx2,avx512f,avx512vl,avx512cd")]
        pub fn add2(a: &[u32; 8], b: &[u32; 8], old: &[u32; 8], mask: u32) -> Pair {
            arith(add_raw(get(a), get(b)), old, mask)
        }
        #[target_feature(enable = "avx2,avx512f,avx512vl,avx512cd")]
        pub fn mul2(a: &[u32; 8], b: &[u32; 8], old: &[u32; 8], mask: u32) -> Pair {
            arith(mul_raw(get(a), get(b)), old, mask)
        }
        #[target_feature(enable = "avx2,avx512f,avx512vl,avx512cd")]
        pub fn fused2(acc: &[u32; 8], a: &[u32; 8], b: &[u32; 8], subtract: bool, old: &[u32; 8], mask: u32) -> FusedPair {
            fused_of(acc, a, b, subtract, old, mask)
        }
    }
}

#[cfg(target_arch = "x86_64")]
pub use eight::{add2, add2_bits, fused2, mul2, mul2_bits, sub2};
#[cfg(target_arch = "x86_64")]
pub use four::{add, add_bits, add_fast, fused, mul, mul_bits, sub};

#[cfg(all(test, target_arch = "x86_64"))]
mod tests {
    use super::*;
    use crate::float::tests::{operand, stream};
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
            let want = reference(&old, mask, |l| float::add(a[l], b[l]));
            assert_eq!(add_fast(&a, &b, &old, mask), want, "add_fast {a:08x?} {b:08x?}");
            assert_eq!(add_bits(&a, &b, &old, mask), want.bits, "add_bits {a:08x?} {b:08x?}");
            assert_eq!(mul_bits(&a, &b, &old, mask), reference(&old, mask, |l| float::mul(a[l], b[l])).bits, "mul_bits");
            // The same operations paired with a second of other operands in lanes 4 to 7.
            let (a2, b2, c2, old2, mask2) = ([b[1], c[0], a[3], b[2]], [c[2], a[0], b[3], a[1]], [a[2], b[0], c[3], c[1]], [old[3], old[2], old[1], old[0]], !mask & 0xF);
            let join = |x: [u32; 4], y: [u32; 4]| -> [u32; 8] { [x[0], x[1], x[2], x[3], y[0], y[1], y[2], y[3]] };
            let both = |x: Vector, y: Vector| Pair { bits: join(x.bits, y.bits), mac: x.mac | y.mac << 16 };
            let (pa, pb, pc, po, pm) = (join(a, a2), join(b, b2), join(c, c2), join(old, old2), mask | mask2 << 16);
            let r = |f: fn(u32, u32) -> Outcome| both(reference(&old, mask, |l| f(a[l], b[l])), reference(&old2, mask2, |l| f(a2[l], b2[l])));
            assert_eq!(add2(&pa, &pb, &po, pm), r(float::add), "add2");
            assert_eq!(sub2(&pa, &pb, &po, pm), r(float::sub), "sub2");
            assert_eq!(mul2(&pa, &pb, &po, pm), r(float::mul), "mul2");
            assert_eq!(add2_bits(&pa, &pb, &po, pm), r(float::add).bits, "add2_bits");
            assert_eq!(mul2_bits(&pa, &pb, &po, pm), r(float::mul).bits, "mul2_bits");
            for subtract in [false, true] {
                let f = |k: bool| {
                    let one = reference(&old, mask, |l| { let x = float::fused(c[l], a[l], b[l], subtract); if k { x.product } else { x.value } });
                    let two = reference(&old2, mask2, |l| { let x = float::fused(c2[l], a2[l], b2[l], subtract); if k { x.product } else { x.value } });
                    both(one, two)
                };
                assert_eq!(fused2(&pc, &pa, &pb, subtract, &po, pm), FusedPair { value: f(false), product: f(true) }, "fused2");
                if available512() {
                    assert_eq!(wide::eight::fused2(&pc, &pa, &pb, subtract, &po, pm), FusedPair { value: f(false), product: f(true) }, "wide fused2");
                    let one = reference(&old, mask, |l| float::fused(c[l], a[l], b[l], subtract).value);
                    let p1 = reference(&old, mask, |l| float::fused(c[l], a[l], b[l], subtract).product);
                    assert_eq!(wide::four::fused(&c, &a, &b, subtract, &old, mask), FusedVector { value: one, product: p1 }, "wide fused");
                }
            }
            if available512() {
                assert_eq!(wide::four::add(&a, &b, &old, mask), reference(&old, mask, |l| float::add(a[l], b[l])), "wide add");
                assert_eq!(wide::four::sub(&a, &b, &old, mask), reference(&old, mask, |l| float::sub(a[l], b[l])), "wide sub");
                assert_eq!(wide::four::mul(&a, &b, &old, mask), reference(&old, mask, |l| float::mul(a[l], b[l])), "wide mul");
                assert_eq!(wide::four::add_bits(&a, &b, &old, mask), reference(&old, mask, |l| float::add(a[l], b[l])).bits, "wide add_bits");
                assert_eq!(wide::eight::add2(&pa, &pb, &po, pm), r(float::add), "wide add2");
                assert_eq!(wide::eight::mul2(&pa, &pb, &po, pm), r(float::mul), "wide mul2");
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
