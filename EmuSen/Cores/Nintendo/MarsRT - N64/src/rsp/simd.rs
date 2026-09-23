//! The vector unit in host vectors, SSE2 to SSE4.1, exact against the element-by-element unit beside it. See Mars_Native.md §6.10.

use std::arch::x86_64::*;

use super::{DATA_MASK, Memory, Reciprocal, Rsp, rd, rt};

const HIGH: usize = 0;
const MIDDLE: usize = 1;
const LOW: usize = 2;

/// The element selector as `pshufb` keys, one per selector: lane `i` takes the lane `selected` names.
static SELECT: [[u8; 16]; 16] = keys();

const fn keys() -> [[u8; 16]; 16] {
    let mut table = [[0u8; 16]; 16];
    let mut selector = 0;
    while selector < 16 {
        let mut i = 0;
        while i < 8 {
            let j = match selector {
                0 | 1 => i,
                2 | 3 => selector - 2 + (i & 6),
                4..=7 => selector - 4 + (i & 4),
                _ => selector - 8,
            };
            table[selector][2 * i] = (2 * j) as u8;
            table[selector][2 * i + 1] = (2 * j + 1) as u8;
            i += 1;
        }
        selector += 1;
    }
    table
}

#[inline]
#[target_feature(enable = "sse2")]
fn vector(lanes: [u16; 8]) -> __m128i {
    // SAFETY: both are sixteen bytes of plain data.
    unsafe { std::mem::transmute::<[u16; 8], __m128i>(lanes) }
}

#[inline]
#[target_feature(enable = "sse2")]
fn lanes(v: __m128i) -> [u16; 8] {
    // SAFETY: both are sixteen bytes of plain data.
    unsafe { std::mem::transmute::<__m128i, [u16; 8]>(v) }
}

/// Each element's two bytes exchanged: DMEM and the register's bytes are big-endian, the host's lanes little.
const SWAP: [u8; 16] = [1, 0, 3, 2, 5, 4, 7, 6, 9, 8, 11, 10, 13, 12, 15, 14];

#[inline]
#[target_feature(enable = "sse2")]
fn bytes(block: [u8; 16]) -> __m128i {
    // SAFETY: both are sixteen bytes of plain data.
    unsafe { std::mem::transmute::<[u8; 16], __m128i>(block) }
}

#[inline]
#[target_feature(enable = "sse2")]
fn block(v: __m128i) -> [u8; 16] {
    // SAFETY: both are sixteen bytes of plain data.
    unsafe { std::mem::transmute::<__m128i, [u8; 16]>(v) }
}

#[inline]
#[target_feature(enable = "sse2")]
fn ones() -> __m128i {
    _mm_set1_epi32(-1)
}

#[inline]
#[target_feature(enable = "sse2")]
fn not(v: __m128i) -> __m128i {
    _mm_xor_si128(v, ones())
}

/// Eight flag bits as lane masks, bit `i` to lane `i`.
#[inline]
#[target_feature(enable = "sse2")]
fn masks(bits: u16) -> __m128i {
    let each = _mm_setr_epi16(1, 2, 4, 8, 16, 32, 64, 128);
    _mm_cmpeq_epi16(_mm_and_si128(_mm_set1_epi16(bits as i16), each), each)
}

/// Lane masks as eight flag bits, lane `i` to bit `i`.
#[inline]
#[target_feature(enable = "sse2")]
fn bits(mask: __m128i) -> u16 {
    (_mm_movemask_epi8(_mm_packs_epi16(mask, _mm_setzero_si128())) & 0xFF) as u16
}

/// Where the unsigned sum `a + b`, already taken as `sum`, carried out of sixteen bits.
#[inline]
#[target_feature(enable = "sse2")]
fn carried(a: __m128i, b: __m128i, sum: __m128i) -> __m128i {
    not(_mm_cmpeq_epi16(_mm_adds_epu16(a, b), sum))
}

/// `b` where the mask is set, `a` elsewhere.
#[inline]
#[target_feature(enable = "sse4.1")]
fn pick(a: __m128i, b: __m128i, mask: __m128i) -> __m128i {
    _mm_blendv_epi8(a, b, mask)
}

/// The 48-bit sum of two accumulators in thirds, wrapping: there is no fourth third.
#[inline]
#[target_feature(enable = "sse2")]
fn add48(acc: [__m128i; 3], addend: [__m128i; 3]) -> [__m128i; 3] {
    let low = _mm_add_epi16(acc[LOW], addend[LOW]);
    let carry_low = carried(acc[LOW], addend[LOW], low);
    let middle = _mm_add_epi16(acc[MIDDLE], addend[MIDDLE]);
    let carry_middle = carried(acc[MIDDLE], addend[MIDDLE], middle);
    let carry_again = _mm_and_si128(carry_low, _mm_cmpeq_epi16(middle, ones()));
    let middle = _mm_sub_epi16(middle, carry_low);
    let high = _mm_sub_epi16(_mm_sub_epi16(_mm_add_epi16(acc[HIGH], addend[HIGH]), carry_middle), carry_again);
    [high, middle, low]
}

/// `clamp_signed(value >> 16)`: bits 47:16 saturated to sixteen signed bits.
#[inline]
#[target_feature(enable = "sse2")]
fn clamp_signed(acc: &[__m128i; 3]) -> __m128i {
    _mm_packs_epi32(_mm_unpacklo_epi16(acc[MIDDLE], acc[HIGH]), _mm_unpackhi_epi16(acc[MIDDLE], acc[HIGH]))
}

/// `clamp_signed(value >> 17) & 0xFFF0`, the quarter multiplies' result.
#[inline]
#[target_feature(enable = "sse2")]
fn clamp_quarter(acc: &[__m128i; 3]) -> __m128i {
    let low = _mm_srai_epi32(_mm_unpacklo_epi16(acc[MIDDLE], acc[HIGH]), 1);
    let high = _mm_srai_epi32(_mm_unpackhi_epi16(acc[MIDDLE], acc[HIGH]), 1);
    _mm_and_si128(_mm_packs_epi32(low, high), _mm_set1_epi16(0xFFF0u16 as i16))
}

/// `clamp_unsigned`: nothing below zero, all ones past 0x7FFF_FFFF, else bits 31:16.
#[inline]
#[target_feature(enable = "sse2")]
fn clamp_unsigned(acc: &[__m128i; 3]) -> __m128i {
    let zero = _mm_setzero_si128();
    let negative = _mm_srai_epi16(acc[HIGH], 15);
    let over = _mm_or_si128(_mm_cmpgt_epi16(acc[HIGH], zero), _mm_and_si128(_mm_cmpeq_epi16(acc[HIGH], zero), _mm_srai_epi16(acc[MIDDLE], 15)));
    _mm_andnot_si128(negative, _mm_or_si128(acc[MIDDLE], over))
}

/// `clamp_low`: the low third while bits 47:16 fit sixteen signed bits, else zero below and all ones above.
#[inline]
#[target_feature(enable = "sse4.1")]
fn clamp_low(acc: &[__m128i; 3]) -> __m128i {
    let fits = _mm_cmpeq_epi16(acc[HIGH], _mm_srai_epi16(acc[MIDDLE], 15));
    pick(not(_mm_srai_epi16(acc[HIGH], 15)), acc[LOW], fits)
}

/// `s × t` signed by signed as thirds: the product sign-extended.
#[inline]
#[target_feature(enable = "sse2")]
fn signed_product(s: __m128i, t: __m128i) -> [__m128i; 3] {
    let high = _mm_mulhi_epi16(s, t);
    [_mm_srai_epi16(high, 15), high, _mm_mullo_epi16(s, t)]
}

/// `s × t` with `s` signed and `t` unsigned, which fits thirty-two signed bits; `mixed(t, s)` is the other way round.
#[inline]
#[target_feature(enable = "sse2")]
fn mixed(signed: __m128i, unsigned: __m128i) -> [__m128i; 3] {
    let high = _mm_sub_epi16(_mm_mulhi_epu16(signed, unsigned), _mm_and_si128(_mm_srai_epi16(signed, 15), unsigned));
    [_mm_srai_epi16(high, 15), high, _mm_mullo_epi16(signed, unsigned)]
}

/// `product << 1` for the fraction multiplies; the sign is the product's, taken before the doubling.
#[inline]
#[target_feature(enable = "sse2")]
fn doubled(s: __m128i, t: __m128i) -> [__m128i; 3] {
    let [sign, high, low] = signed_product(s, t);
    [sign, _mm_or_si128(_mm_slli_epi16(high, 1), _mm_srli_epi16(low, 15)), _mm_slli_epi16(low, 1)]
}

impl<M: Memory> Rsp<M> {
    #[inline]
    #[target_feature(enable = "sse2")]
    fn accumulator(&self) -> [__m128i; 3] {
        [vector(self.m.third(HIGH)), vector(self.m.third(MIDDLE)), vector(self.m.third(LOW))]
    }

    #[inline]
    #[target_feature(enable = "sse2")]
    fn set_accumulator(&mut self, acc: [__m128i; 3]) {
        self.m.set_thirds([lanes(acc[HIGH]), lanes(acc[MIDDLE]), lanes(acc[LOW])]);
    }

    #[inline]
    #[target_feature(enable = "sse2")]
    fn set_low(&mut self, low: __m128i) {
        self.m.set_third(LOW, lanes(low));
    }

    /// `vector_load`, with the byte to quad formats taken as one masked blend where they move whole elements.
    #[target_feature(enable = "sse2,ssse3,sse4.1")]
    pub(super) fn vector_load_simd(&mut self, instruction: u32) {
        let Some((format, vt, element, address)) = self.transfer_operands(instruction) else { return };
        let count = if format == 4 { 16 - (address & 0xF) as usize } else { 1 << format };
        let n = count.min(16 - element);
        if format > 4 || (element | n) & 1 != 0 {
            return self.load_format(format, vt, element, address);
        }
        let Some(base) = (address as usize).checked_sub(element).filter(|b| b + 16 <= DATA_MASK as usize + 1) else {
            return self.load_lanes(vt, element, address, n);
        };
        let loaded = _mm_shuffle_epi8(bytes(self.m.data_block(base)), bytes(SWAP));
        let index = _mm_setr_epi16(0, 1, 2, 3, 4, 5, 6, 7);
        let (first, end) = ((element / 2) as i16, ((element + n) / 2) as i16);
        let inside = _mm_and_si128(_mm_cmpgt_epi16(index, _mm_set1_epi16(first - 1)), _mm_cmpgt_epi16(_mm_set1_epi16(end), index));
        let register = pick(vector(self.m.register(vt)), loaded, inside);
        self.m.set_register(vt, lanes(register));
    }

    /// `vector_store`, with the byte to quad formats taken as one masked blend where they move whole elements and do not wrap the register.
    #[target_feature(enable = "sse2,ssse3,sse4.1")]
    pub(super) fn vector_store_simd(&mut self, instruction: u32) {
        let Some((format, vt, element, address)) = self.transfer_operands(instruction) else { return };
        let count = if format == 4 { 16 - (address & 0xF) as usize } else { 1 << format };
        if format > 4 || (element | count) & 1 != 0 || element + count > 16 {
            return self.store_format(format, vt, element, address);
        }
        let Some(base) = (address as usize).checked_sub(element).filter(|b| b + 16 <= DATA_MASK as usize + 1) else {
            return self.store_lanes(vt, element, address, count);
        };
        let stored = _mm_shuffle_epi8(vector(self.m.register(vt)), bytes(SWAP));
        let index = _mm_setr_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
        let (first, end) = (element as i8, (element + count) as i8);
        let inside = _mm_and_si128(_mm_cmpgt_epi8(index, _mm_set1_epi8(first - 1)), _mm_cmpgt_epi8(_mm_set1_epi8(end), index));
        let memory = pick(bytes(self.m.data_block(base)), stored, inside);
        self.m.set_data_block(base, block(memory));
    }

    /// `vector_op`, eight lanes at a time; the reciprocals and `VMOV` stay element by element, a table lookup on one lane.
    #[target_feature(enable = "sse2,ssse3,sse4.1")]
    pub(super) fn vector_op_simd(&mut self, instruction: u32) {
        let function = instruction & 0x3F;
        if function == 0x37 || function == 0x3F {
            return;
        }
        let vt = rt(instruction);
        let vs = rd(instruction);
        let vd = ((instruction >> 6) & 0x1F) as usize;
        let selector = ((instruction >> 21) & 0xF) as usize;

        let s = vector(self.m.register(vs));
        let mut t = vector(self.m.register(vt));
        if selector > 1 {
            // SAFETY: the table's rows are sixteen bytes.
            t = _mm_shuffle_epi8(t, unsafe { _mm_loadu_si128(SELECT[selector].as_ptr() as *const __m128i) });
        }
        let zero = _mm_setzero_si128();

        let d = match function {
            0x00 | 0x01 | 0x08 | 0x09 => {
                let product = doubled(s, t);
                let acc = if function & 0x08 != 0 {
                    add48(self.accumulator(), product)
                } else {
                    let rounding = _mm_set1_epi16(0x8000u16 as i16);
                    let carry = _mm_srai_epi16(product[LOW], 15);
                    let middle = _mm_sub_epi16(product[MIDDLE], carry);
                    let carry_again = _mm_and_si128(carry, _mm_cmpeq_epi16(product[MIDDLE], ones()));
                    [_mm_sub_epi16(product[HIGH], carry_again), middle, _mm_xor_si128(product[LOW], rounding)]
                };
                self.set_accumulator(acc);
                if function & 1 != 0 { clamp_unsigned(&acc) } else { clamp_signed(&acc) }
            }
            0x02 | 0x0A => {
                let acc = self.accumulator();
                let negative = _mm_srai_epi16(acc[HIGH], 15);
                let taken = if function == 0x02 { not(negative) } else { negative };
                let sign = _mm_srai_epi16(t, 15);
                let addend = if vs & 1 != 0 { [sign, t, zero] } else { [sign, sign, t] };
                let acc = add48(acc, addend.map(|a| _mm_and_si128(a, taken)));
                self.set_accumulator(acc);
                clamp_signed(&acc)
            }
            0x03 => {
                let [_, high, low] = signed_product(s, t);
                let bias = _mm_and_si128(_mm_srai_epi16(high, 15), _mm_set1_epi16(0x1F));
                let middle = _mm_add_epi16(low, bias);
                let acc = [_mm_sub_epi16(high, carried(low, bias, middle)), middle, zero];
                self.set_accumulator(acc);
                clamp_quarter(&acc)
            }
            0x0B => {
                let acc = self.accumulator();
                let clear = _mm_cmpeq_epi16(_mm_and_si128(acc[MIDDLE], _mm_set1_epi16(0x20)), zero);
                let negative = _mm_srai_epi16(acc[HIGH], 15);
                let above = _mm_or_si128(
                    _mm_cmpgt_epi16(acc[HIGH], zero),
                    _mm_and_si128(_mm_cmpeq_epi16(acc[HIGH], zero), not(_mm_cmpeq_epi16(_mm_and_si128(acc[MIDDLE], _mm_set1_epi16(0xFFC0u16 as i16)), zero))),
                );
                let up = _mm_and_si128(clear, negative);
                let down = _mm_and_si128(clear, above);
                let middle = _mm_or_si128(_mm_and_si128(up, _mm_set1_epi16(0x20)), _mm_and_si128(down, _mm_set1_epi16(0xFFE0u16 as i16)));
                let acc = add48(acc, [down, middle, zero]);
                self.set_accumulator(acc);
                clamp_quarter(&acc)
            }
            0x04 | 0x0C => {
                let product = [zero, zero, _mm_mulhi_epu16(s, t)];
                let acc = if function & 0x08 != 0 { add48(self.accumulator(), product) } else { product };
                self.set_accumulator(acc);
                clamp_low(&acc)
            }
            0x05 | 0x0D => {
                let product = mixed(s, t);
                let acc = if function & 0x08 != 0 { add48(self.accumulator(), product) } else { product };
                self.set_accumulator(acc);
                clamp_signed(&acc)
            }
            0x06 | 0x0E => {
                let product = mixed(t, s);
                let acc = if function & 0x08 != 0 { add48(self.accumulator(), product) } else { product };
                self.set_accumulator(acc);
                clamp_low(&acc)
            }
            0x07 | 0x0F => {
                let [_, high, low] = signed_product(s, t);
                let product = [high, low, zero];
                let acc = if function & 0x08 != 0 { add48(self.accumulator(), product) } else { product };
                self.set_accumulator(acc);
                clamp_signed(&acc)
            }
            0x10 => {
                let carry = masks(self.m.vco() & 0xFF);
                self.set_low(_mm_sub_epi16(_mm_add_epi16(s, t), carry));
                self.m.set_vco(0);
                _mm_adds_epi16(_mm_subs_epi16(_mm_min_epi16(s, t), carry), _mm_max_epi16(s, t))
            }
            0x11 => {
                let carry = masks(self.m.vco() & 0xFF);
                let wrapped = _mm_sub_epi16(t, carry);
                let saturated = _mm_subs_epi16(t, carry);
                self.set_low(_mm_sub_epi16(s, wrapped));
                self.m.set_vco(0);
                _mm_adds_epi16(_mm_subs_epi16(s, saturated), _mm_cmpgt_epi16(saturated, wrapped))
            }
            0x13 => {
                let negative = _mm_srai_epi16(s, 15);
                let kept = _mm_xor_si128(_mm_andnot_si128(_mm_cmpeq_epi16(s, zero), t), negative);
                self.set_low(_mm_sub_epi16(kept, negative));
                _mm_subs_epi16(kept, negative)
            }
            0x14 => {
                let sum = _mm_add_epi16(s, t);
                self.set_low(sum);
                self.m.set_vco(bits(carried(s, t, sum)));
                sum
            }
            0x15 => {
                let difference = _mm_sub_epi16(s, t);
                self.set_low(difference);
                let borrow = not(_mm_cmpeq_epi16(_mm_subs_epu16(t, s), zero));
                let unequal = not(_mm_cmpeq_epi16(s, t));
                self.m.set_vco(bits(borrow) | bits(unequal) << 8);
                difference
            }
            0x1D => match selector {
                8 => vector(self.m.third(HIGH)),
                9 => vector(self.m.third(MIDDLE)),
                10 => vector(self.m.third(LOW)),
                _ => zero,
            },
            0x20..=0x23 => {
                let equal = _mm_cmpeq_epi16(s, t);
                let (carry, unequal) = (masks(self.m.vco() & 0xFF), masks(self.m.vco() >> 8));
                let both = _mm_and_si128(carry, unequal);
                let (chosen, d) = match function {
                    0x20 => {
                        let chosen = _mm_or_si128(_mm_cmplt_epi16(s, t), _mm_and_si128(equal, both));
                        (chosen, pick(t, s, chosen))
                    }
                    0x21 => (_mm_andnot_si128(unequal, equal), t),
                    0x22 => (_mm_or_si128(not(equal), unequal), s),
                    _ => {
                        let chosen = _mm_or_si128(_mm_cmpgt_epi16(s, t), _mm_andnot_si128(both, equal));
                        (chosen, pick(t, s, chosen))
                    }
                };
                self.set_low(d);
                self.m.set_vcc(bits(chosen));
                self.m.set_vco(0);
                d
            }
            0x24 => {
                let (vco, vcc) = (self.m.vco(), self.m.vcc());
                let (differed, unequal, extension) = (masks(vco & 0xFF), masks(vco >> 8), masks(self.m.vce() as u16));
                let sum = _mm_add_epi16(s, t);
                let is_zero = _mm_cmpeq_epi16(sum, zero);
                let no_carry = _mm_cmpeq_epi16(_mm_adds_epu16(s, t), sum);
                let decided = _mm_or_si128(_mm_and_si128(is_zero, no_carry), _mm_and_si128(extension, _mm_or_si128(is_zero, no_carry)));
                let less_or_equal = pick(masks(vcc & 0xFF), decided, _mm_andnot_si128(unequal, differed));
                let greater_or_equal = pick(masks(vcc >> 8), _mm_cmpeq_epi16(_mm_subs_epu16(t, s), zero), not(_mm_or_si128(differed, unequal)));
                let d = pick(pick(s, t, greater_or_equal), pick(s, _mm_sub_epi16(zero, t), less_or_equal), differed);
                self.set_low(d);
                self.m.set_vcc(bits(less_or_equal) | bits(greater_or_equal) << 8);
                self.m.set_vco(0);
                self.m.set_vce(0);
                d
            }
            0x25 => {
                let differ = _mm_srai_epi16(_mm_xor_si128(s, t), 15);
                let sum = _mm_add_epi16(s, t);
                let t_negative = _mm_srai_epi16(t, 15);
                let less_or_equal = pick(t_negative, not(_mm_cmpgt_epi16(sum, zero)), differ);
                let greater_or_equal = pick(not(_mm_cmpgt_epi16(t, s)), t_negative, differ);
                let minus_one = _mm_cmpeq_epi16(sum, ones());
                let unequal = pick(not(_mm_cmpeq_epi16(s, t)), not(_mm_or_si128(_mm_cmpeq_epi16(sum, zero), minus_one)), differ);
                let d = pick(pick(s, t, greater_or_equal), pick(s, _mm_sub_epi16(zero, t), less_or_equal), differ);
                self.set_low(d);
                self.m.set_vco(bits(differ) | bits(unequal) << 8);
                self.m.set_vcc(bits(less_or_equal) | bits(greater_or_equal) << 8);
                self.m.set_vce(bits(_mm_and_si128(differ, minus_one)) as u8);
                d
            }
            0x26 => {
                let differ = _mm_srai_epi16(_mm_xor_si128(s, t), 15);
                let t_negative = _mm_srai_epi16(t, 15);
                let less_or_equal = pick(t_negative, _mm_srai_epi16(_mm_add_epi16(s, t), 15), differ);
                let greater_or_equal = pick(not(_mm_cmpgt_epi16(t, s)), t_negative, differ);
                let d = pick(pick(s, t, greater_or_equal), pick(s, not(t), less_or_equal), differ);
                self.set_low(d);
                self.m.set_vco(0);
                self.m.set_vcc(bits(less_or_equal) | bits(greater_or_equal) << 8);
                self.m.set_vce(0);
                d
            }
            0x27 => {
                let d = pick(t, s, masks(self.m.vcc() & 0xFF));
                self.set_low(d);
                self.m.set_vco(0);
                d
            }
            0x28..=0x2D => {
                let d = match function {
                    0x28 => _mm_and_si128(t, s),
                    0x29 => not(_mm_and_si128(t, s)),
                    0x2A => _mm_or_si128(t, s),
                    0x2B => not(_mm_or_si128(t, s)),
                    0x2C => _mm_xor_si128(t, s),
                    _ => not(_mm_xor_si128(t, s)),
                };
                self.set_low(d);
                d
            }
            0x30..=0x36 => {
                let (t, mut d) = (lanes(t), self.m.register(vd));
                let input = self.m.element(vt, selector & 7);
                match function {
                    0x30 => self.reciprocate(input, vs & 7, &t, &mut d, Reciprocal::Single, false),
                    0x31 => self.reciprocate(input, vs & 7, &t, &mut d, Reciprocal::Low, false),
                    0x32 => self.reciprocate(input, vs & 7, &t, &mut d, Reciprocal::High, false),
                    0x33 => self.move_element(vs & 7, &t, &mut d),
                    0x34 => self.reciprocate(input, vs & 7, &t, &mut d, Reciprocal::Single, true),
                    0x35 => self.reciprocate(input, vs & 7, &t, &mut d, Reciprocal::Low, true),
                    _ => self.reciprocate(input, vs & 7, &t, &mut d, Reciprocal::High, true),
                }
                vector(d)
            }
            _ => {
                self.set_low(_mm_add_epi16(s, t));
                zero
            }
        };
        self.m.set_register(vd, lanes(d));
    }
}
