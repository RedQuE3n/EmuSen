//! The copy mode: four texels a step written straight to the colour image as bytes: C#'s `Rdp.Copy.cs`.

use super::textures::{masked, shifted};
use super::{ATTRIBUTE_S, ATTRIBUTE_T, ATTRIBUTE_W, ATTRIBUTES, Rdp, RdpMemory, Rows, TextureTile};

/// A texel narrower than sixteen bits becomes one byte: a nibble doubled, a palette number and a nibble, or two nibbles.
#[inline(always)]
fn replicated(word: i32, nibble: i32, size: i32, format: i32, palette: i32) -> i32 {
    if size == 0 {
        let value = (word >> ((nibble ^ 3) << 2)) & 0xF;
        if format == 2 {
            return (palette << 4) | value;
        }
        if format != 3 {
            return (value << 4) | value;
        }
        let doubled = (value << 4) | value;
        return (doubled & 0xE0) | ((doubled & 0xE0) >> 3) | ((doubled & 0xC0) >> 6);
    }

    if size != 1 {
        return (word >> 8) & 0xFF;
    }

    let at = ((nibble ^ 3) | 1) << 2;
    if format != 3 {
        return (((word >> at) & 0xF) << 4) | ((word >> (at & !4)) & 0xF);
    }
    let half = (word >> at) & 0xF;
    (half << 4) | half
}

/// A four-bit texel's index carries the tile's palette number; a wider one takes both nibbles of its own byte.
#[inline(always)]
fn copy_palette_index(word: i32, nibble: i32, tile: &TextureTile) -> i32 {
    if tile.size == 0 {
        return (tile.palette << 4) | ((word >> ((nibble ^ 3) << 2)) & 0xF);
    }
    let at = ((nibble & 2) ^ 2) << 2;
    ((if at != 0 { (word >> 12) & 0xF } else { (word >> 4) & 0xF }) << 4) | ((word >> at) & 0xF)
}

impl Rdp {
    /// Eight bytes a step, cut short at the span's end, mirrored when the span runs right to left.
    pub(super) fn draw_copy(&mut self, rows: Rows, major_on_left: bool, tile: i32, max_level: i32, mem: &mut RdpMemory) {
        let size = self.color_image_size;
        if size == 3 {
            return;
        }

        let direction: i32 = if major_on_left { 1 } else { -1 };
        let pixel_bytes = if size == 2 { 2 } else { 1 };
        let per_step = if size == 0 { 8 } else { 16 >> size };

        let ds = direction.wrapping_mul(self.texture_step[0]);
        let dt = direction.wrapping_mul(self.texture_step[1]);
        let dw = direction.wrapping_mul(self.texture_step[2]);

        for y in rows.0..=rows.1 {
            let yu = y as usize;
            if !self.span_drawn[yu] || self.span_right[yu] < self.span_left[yu] {
                continue;
            }

            let at = yu * ATTRIBUTES;
            let mut s = self.span_attributes[at + ATTRIBUTE_S];
            let mut t = self.span_attributes[at + ATTRIBUTE_T];
            let mut w = self.span_attributes[at + ATTRIBUTE_W];

            let row = y.wrapping_mul(self.color_image_width);
            let (near, far) = if major_on_left { (self.span_left[yu], self.span_right[yu]) } else { (self.span_right[yu], self.span_left[yu]) };
            let mut pointer = self.color_image.wrapping_add(row.wrapping_add(near).wrapping_mul(pixel_bytes) as u32);
            let last = self.color_image.wrapping_add(row.wrapping_add(far).wrapping_mul(pixel_bytes) as u32);

            let mut left = self.span_right[yu] - self.span_left[yu];
            while left >= 0 {
                let (cs, ct) = self.texture_coordinates(s, t, w);
                let from = self
                    .level_of_detail(
                        s.wrapping_add(ds),
                        t.wrapping_add(dt),
                        w.wrapping_add(dw),
                        s.wrapping_add(ds << 1),
                        t.wrapping_add(dt << 1),
                        w.wrapping_add(dw << 1),
                        tile,
                        max_level,
                    )
                    .0;

                let texels = if size == 0 { 0 } else { self.copy_texels(cs, ct, from) };
                let mask = self.copy_alpha_mask(texels);

                let remaining = (if major_on_left { last.wrapping_sub(pointer) } else { pointer.wrapping_sub(last) }) as i32;
                let mut bytes = 8.min(remaining.wrapping_add(pixel_bytes));
                let mut byte_at = pointer;
                let mut k = 7;
                while bytes > 0 {
                    if (mask & (1 << k)) != 0 {
                        write_copy_byte(byte_at, ((texels >> (k << 3)) & 0xFF) as u8, mem);
                    }
                    k -= 1;
                    bytes -= 1;
                    byte_at = byte_at.wrapping_add(direction as u32);
                }

                s = s.wrapping_add(ds);
                t = t.wrapping_add(dt);
                w = w.wrapping_add(dw);
                pointer = pointer.wrapping_add(direction.wrapping_mul(8) as u32);
                left -= per_step;
            }
        }
    }

    /// The four texels at s to s + 3, each a sixteen-bit word, and the eight bytes they make.
    fn copy_texels(&self, s: i32, t: i32, tile_index: i32) -> u64 {
        let tile = &self.tiles[tile_index as usize];

        let s = (shifted(s, tile.shift_s) - (tile.sl << 3)) >> 5;
        let mut t = (shifted(t, tile.shift_t) - (tile.tl << 3)) >> 5;
        if tile.mask_t != 0 {
            t = masked(t, tile.mask_t, tile.mirror_t, tile.mirror_bit_t());
        }

        let yuv = tile.format == 1;
        let stride = if yuv || tile.size == 1 {
            2
        } else if tile.size >= 2 {
            4
        } else {
            1
        };
        let start = ((((tile.line.wrapping_mul(t)) & 0x1FF) + tile.memory) << 4) & 0x1FFF;

        let mut along = [0i32; 4];
        for (k, a) in along.iter_mut().enumerate() {
            let column = if tile.mask_s != 0 { masked(s + k as i32, tile.mask_s, tile.mirror_s, tile.mirror_bit_s()) } else { s + k as i32 };
            *a = column.wrapping_mul(stride) & 0x1FFF;
        }

        let mut upper = [0i32; 4];
        for k in 0..4 {
            upper[k] = (start + along[k]) & 0x1FFF;
        }

        let mut lower = [0i32; 4];
        lower[0] = upper[0];
        lower[2] = upper[2];
        lower[1] = if yuv { (upper[0] + ((along[1] - along[0]) << 1)) & 0x1FFF } else { upper[1] };
        lower[3] = if yuv { (lower[1] + along[3] - along[0]) & 0x1FFF } else { upper[3] };

        if (t & 1) != 0 {
            for k in 0..4 {
                upper[k] ^= 8;
                lower[k] ^= 8;
            }
        }

        let palette = self.modes.palette_enabled;
        let size = if palette { 2 } else { tile.size };
        let wide = !palette && (tile.format == 1 || tile.format == 0 && tile.size == 3);

        let mut words = [0i32; 4];
        for k in 0..4 {
            let word = self.copy_bank_word(&lower, k, 0);
            words[k] = if palette {
                self.texture_word(0x400 | (copy_palette_index(word, lower[k] & 3, tile) << 2) | k as i32)
            } else if wide || (lower[k] & 0x1000) == 0 {
                word
            } else {
                self.copy_bank_word(&upper, k, 0x400)
            };
        }

        let low = ((words[2] as u64) << 16) | words[3] as u32 as u64;
        if size == 2 {
            return ((((words[0] as u64) << 16) | words[1] as u32 as u64) << 32) | low;
        }

        let mut high: u64 = 0;
        for k in 0..4 {
            high = (high << 8) | replicated(words[k], lower[k] & 3, size, tile.format, tile.palette) as u32 as u64;
        }
        (high << 32) | low
    }

    /// The four addresses reach four banks at once, so a texel whose bank an earlier one claimed reads that one's word.
    #[inline(always)]
    fn copy_bank_word(&self, addresses: &[i32; 4], texel: usize, half: i32) -> i32 {
        let bank = (addresses[texel] >> 2) & 3;
        let mut first = 0;
        while ((addresses[first] >> 2) & 3) != bank {
            first += 1;
        }
        self.texture_word(half | ((addresses[first] >> 2) & 0x3FF))
    }

    /// Alpha compare keeps a sixteen-bit image's pixels by their alpha bit and an eight-bit one's by its byte.
    fn copy_alpha_mask(&self, texels: u64) -> i32 {
        if !self.modes.alpha_compare {
            return 0xFF;
        }
        let size = self.color_image_size;
        if size != 1 && size != 2 {
            return 0;
        }

        let threshold = self.alpha_threshold();
        let mut mask = 0;
        for k in 0..4 {
            let keep = if size == 2 { ((texels >> (48 - (k << 4))) & 1) != 0 } else { ((texels >> (24 - (k << 3))) & 0xFF) as i32 >= threshold };
            if keep {
                mask |= 0xC0 >> (k << 1);
            }
        }
        mask
    }
}

/// One byte, and the hidden bits its pair shares, which only an odd address writes.
#[inline(always)]
fn write_copy_byte(address: u32, value: u8, mem: &mut RdpMemory) {
    if address as usize >= mem.rdram.len() {
        return;
    }
    mem.rdram[address as usize] = value;
    if (address & 1) != 0 {
        mem.hidden[(address >> 1) as usize] = (value & 1) * 3;
    }
}
