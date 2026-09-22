//! The depth buffer's encoding, the four compare modes, the store, and shade and depth correction: C#'s `Rdp.Depth.cs`.

use super::{Rdp, RdpMemory, TWO_CYCLE};

/// Eighteen bits of depth kept as a three-bit exponent of leading ones and an eleven-bit mantissa.
#[inline(always)]
pub(super) fn compress_depth(z: i32) -> i32 {
    let leading = (!(((z & 0x3FFFF) as u32) << 14)).leading_zeros().min(7) as i32;
    let mantissa = if leading <= 4 { z >> (4 - leading) } else { z << (leading - 4).min(2) };
    (mantissa & 0x1FFC) | (leading << 13)
}

#[inline(always)]
pub(super) fn decompress_depth(stored: i32) -> i32 {
    let packed = (stored >> 2) & 0x3FFF;
    let exponent = (packed >> 11) & 7;
    (((packed & 0x7FF) << (6 - exponent).max(0)) + (0x40000 - (0x40000 >> exponent))) & 0x3FFFF
}

/// Four bits of a power of two's exponent.
#[inline(always)]
pub(super) fn delta_z_encoding(value: i32) -> i32 {
    (if (value & 0xFF00) != 0 { 8 } else { 0 })
        | (if (value & 0xF0F0) != 0 { 4 } else { 0 })
        | (if (value & 0xCCCC) != 0 { 2 } else { 0 })
        | (if (value & 0xAAAA) != 0 { 1 } else { 0 })
}

/// The depth slope as the power of two above its magnitude.
pub(super) fn normalize_delta_z(sum: i32) -> i32 {
    if (sum & 0xC000) != 0 {
        return 0x8000;
    }
    if (sum & 0xFFFF) == 0 {
        return 1;
    }
    highest_bit(sum) << 1
}

#[inline(always)]
fn highest_bit(value: i32) -> i32 {
    if value == 0 { 0 } else { 1i32.wrapping_shl(31 - (value as u32).leading_zeros()) }
}

impl Rdp {
    /// Whether the pixel survives; sets the blend decision and overflow, and may scale coverage.
    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    pub(super) fn compare_depth(
        &mut self,
        index: u32,
        z: i32,
        delta_z: i32,
        delta_z_encoded: i32,
        memory_coverage: i32,
        coverage: &mut i32,
        blend: &mut bool,
        overflow: &mut bool,
        mem: &RdpMemory,
    ) -> bool {
        let z = z & 0x3FFFF;
        *overflow = ((memory_coverage + *coverage) & 8) != 0;
        let m = &self.modes;

        let two_cycle = m.cycle_type == TWO_CYCLE;
        let shifts = (if two_cycle { m.second_blend_cycle.second_alpha } else { m.first_blend_cycle.second_alpha }) == 1;
        let past_shifts = two_cycle && m.first_blend_cycle.second_alpha == 1;

        if !m.depth_compare {
            *blend = m.force_blend || (!*overflow && m.antialias);
            let far = if delta_z_encoded < 0xB { 4 } else { 0xF - delta_z_encoded };
            if shifts {
                self.blend_shift_a = 0;
                self.blend_shift_b = far;
                self.split.shift = self.split.row_stamp;
            }
            if past_shifts {
                self.past_shift_a = 0;
                self.past_shift_b = far;
                self.split.past_shift = self.split.row_stamp;
            }
            self.past_stored_encoded = 0xF;
            return true;
        }

        let (stored, stored_hidden) = read_depth_word(index, mem);

        let old = decompress_depth(stored);
        let stored_encoded = ((stored & 3) << 2) | stored_hidden;
        let mut stored_slope = 1i32.wrapping_shl(stored_encoded as u32);

        if shifts {
            self.blend_shift_a = (delta_z_encoded - stored_encoded).clamp(0, 4);
            self.blend_shift_b = (stored_encoded - delta_z_encoded).clamp(0, 4);
            self.split.shift = self.split.row_stamp;
        }

        if past_shifts {
            self.past_shift_a = (delta_z_encoded - self.past_stored_encoded).clamp(0, 4);
            self.past_shift_b = (self.past_stored_encoded - delta_z_encoded).clamp(0, 4);
            self.split.past_shift = self.split.row_stamp;
        }

        self.past_stored_encoded = stored_encoded;

        let precision = (stored >> 13) & 0xF;
        if precision < 3 {
            stored_slope = if stored_slope != 0x8000 { (stored_slope.wrapping_shl(1)).max(16 >> precision) } else { 0xFFFF };
        }

        let slope = highest_bit((delta_z | stored_slope) & 0xFFFF);
        let margin = slope << 3;

        let m = &self.modes;
        let farther = z + margin >= old;
        *blend = m.force_blend || (!*overflow && m.antialias && farther);

        let in_front = z < old;
        let nearer = z - margin <= old;
        let maximum = old == 0x3FFFF;

        match m.depth_mode {
            0 => maximum || (if *overflow { in_front } else { nearer }),
            1 => {
                if !in_front || !farther || !*overflow {
                    return maximum || (if *overflow { in_front } else { nearer });
                }
                let shift = delta_z_encoding(slope & 0xFFFF);
                *coverage = (((((old >> shift) - (z >> shift)) & 0xF) * *coverage) >> 3) & 0xF;
                true
            }
            2 => in_front || maximum,
            _ => farther && nearer && !maximum,
        }
    }

    /// Shade scaled back from its extra precision, or moved to the first covered sample of a partial pixel.
    #[inline(always)]
    pub(super) fn correct_shade(value: i32, correct_dx: i32, correct_dy: i32, offset: (u8, u8), coverage: i32) -> i32 {
        let corrected =
            if coverage == 8 { value >> 2 } else { ((value << 2) + offset.0 as i32 * correct_dx + offset.1 as i32 * correct_dy) >> 4 };
        super::one_cycle::clamp9(corrected & 0x1FF)
    }

    #[inline(always)]
    pub(super) fn correct_depth(&self, z: i32, offset: (u8, u8), coverage: i32) -> i32 {
        let corrected = if coverage == 8 {
            z >> 3
        } else {
            ((z << 2) + offset.0 as i32 * self.depth_correct_dx + offset.1 as i32 * self.depth_correct_dy) >> 5
        };

        match (corrected & 0x60000) >> 17 {
            2 => 0x3FFFF,
            3 => 0,
            _ => corrected & 0x3FFFF,
        }
    }
}

/// The word and its hidden bits at a depth index, or zero past the memory's end.
#[inline(always)]
pub(super) fn read_depth_word(index: u32, mem: &RdpMemory) -> (i32, i32) {
    let at = index.wrapping_mul(2);
    if (at.wrapping_add(1) as usize) < mem.len() {
        ((mem.get(at as usize) as i32) << 8 | mem.get(at as usize + 1) as i32, mem.get_hidden(index as usize) as i32)
    } else {
        (0, 0)
    }
}

#[inline(always)]
pub(super) fn store_depth(index: u32, z: i32, delta_z_encoded: i32, mem: &mut RdpMemory) {
    let at = index.wrapping_mul(2);
    if (at.wrapping_add(1) as usize) >= mem.len() {
        return;
    }
    let stored = compress_depth(z & 0x3FFFF) | (delta_z_encoded >> 2);
    mem.set(at as usize, (stored >> 8) as u8);
    mem.set(at as usize + 1, stored as u8);
    mem.set_hidden(index as usize, (delta_z_encoded & 3) as u8);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_compresses_and_decompresses_at_every_exponent() {
        assert_eq!(compress_depth(0x3FFFF) >> 13, 7);
        assert_eq!(decompress_depth(0xFFFC), 0x3FFFF);
        assert_eq!(decompress_depth(0), 0);
        assert_eq!(normalize_delta_z(0), 1);
        assert_eq!(normalize_delta_z(0x4000), 0x8000);
        assert_eq!(normalize_delta_z(5), 8);
        assert_eq!(delta_z_encoding(0x8000), 0xF);
    }
}
