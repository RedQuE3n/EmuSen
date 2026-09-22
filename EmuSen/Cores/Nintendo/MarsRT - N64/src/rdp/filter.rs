//! Four texels around a coordinate, from texture memory or through a palette, filtered or converted: C#'s `Rdp.Filter.cs`.

use super::tables::FIVE_TO_EIGHT;
use super::textures::{clamped, signed_nine};
use super::{Color, Rdp, TextureTile};

#[inline(always)]
fn converted_corners(t0: i32, t1: i32, t2: i32, t3: i32, p: Color, upper: bool, center: bool) -> i32 {
    if center {
        return p.b + ((p.r * (t2 - t3) + p.g * (t1 - t3) + ((!t3 + t0) << 6) + 0xC0) >> 8);
    }
    if upper {
        return p.b + ((p.r * (t2 - t3) + p.g * (t1 - t3) + 0x80) >> 8);
    }
    p.b + ((p.r * (t1 - t0) + p.g * (t2 - t0) + 0x80) >> 8)
}

/// Masked and mirrored as for one texel, and the step to the next: back to zero past the mask, or reversed and held at a mirror's turn.
#[inline(always)]
fn wrapped(coordinate: &mut i32, mask: i32, mirror: bool, mirror_bit: i32, wrap_bits: i32) -> i32 {
    if mask == 0 {
        return 1;
    }

    let bits = (0xFFFF >> (16 - mask)) & 0x3FF;
    if mirror {
        let wrap = (*coordinate >> mirror_bit) & 1;
        *coordinate = (*coordinate ^ -wrap) & bits;
        return if ((*coordinate - wrap) & bits) == bits { 0 } else { 1 - (wrap << 1) };
    }

    *coordinate &= bits;
    if *coordinate == bits { -(*coordinate & wrap_bits) } else { 1 }
}

/// Three texels weighted by the triangle the fraction lies in, or the rounded average of all four at the mid-texel.
#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn interpolated(t0: i32, t1: i32, t2: i32, t3: i32, s_fraction: i32, t_fraction: i32, upper: bool, center: bool) -> i32 {
    if center {
        return (t0 + t1 + t2 + t3 + 2) >> 2;
    }
    if upper {
        return t3 + (((0x20 - s_fraction) * (t2 - t3) + (0x20 - t_fraction) * (t1 - t3) + 0x10) >> 5);
    }
    t0 + ((s_fraction * (t1 - t0) + t_fraction * (t2 - t0) + 0x10) >> 5)
}

impl Rdp {
    /// The fraction picks one of the square's two triangles; YUV's chroma is half as wide, so it may pick the other.
    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    pub(super) fn four_texels(
        &self,
        s: i32,
        t: i32,
        beyond_s: bool,
        beyond_t: bool,
        tile: &TextureTile,
        bilinear: bool,
        convert: bool,
        previous: Color,
    ) -> Color {
        let clamps_s = tile.clamps_s();
        let clamps_t = tile.clamps_t();
        let s_fraction = if clamps_s && (beyond_s || (s & 0x10000) != 0) { 0 } else { s & 0x1F };
        let t_fraction = if clamps_t && (beyond_t || (t & 0x10000) != 0) { 0 } else { t & 0x1F };

        let mut s = clamped(s, clamps_s, beyond_s, tile.clamp_limit_s());
        let mut t = clamped(t, clamps_t, beyond_t, tile.clamp_limit_t());
        let s_step = wrapped(&mut s, tile.mask_s, tile.mirror_s, tile.mirror_bit_s(), -1);
        let t_step = wrapped(&mut t, tile.mask_t, tile.mirror_t, tile.mirror_bit_t(), 0xFF);

        let upper = ((s_fraction + t_fraction) & 0x20) != 0;
        let yuv = tile.format == 1;
        let s_fraction_rg = if yuv { (s_fraction >> 1) | ((s & 1) << 4) } else { s_fraction };
        let upper_rg = ((s_fraction_rg + t_fraction) & 0x20) != 0;

        if convert && !bilinear {
            let p = signed_nine(previous);
            return self.converted(p, p.b);
        }

        let m = &self.modes;
        let mut texels = [Color::default(); 4];
        if !m.sample_four {
            self.nearest_palette_texels(&mut texels, s, t, tile, upper_rg);
        } else if m.palette_enabled {
            self.palette_texels(&mut texels, s, s_step, t, t_step, tile, upper_rg);
        } else {
            self.texels(&mut texels, s, s_step, t, t_step, tile);
        }

        if upper != upper_rg && (yuv && tile.size < 2 || m.palette_enabled) {
            let (b0, b1, b2, b3) = (texels[0].b, texels[1].b, texels[2].b, texels[3].b);
            let (a0, a1, a2, a3) = (texels[0].a, texels[1].a, texels[2].a, texels[3].a);
            (texels[0].b, texels[3].b, texels[1].b, texels[2].b) = (b3, b0, b2, b1);
            (texels[0].a, texels[3].a, texels[1].a, texels[2].a) = (a3, a0, a2, a1);
        }

        if !bilinear {
            let chroma = if upper_rg { texels[3] } else { texels[0] };
            let luma = if upper { texels[3] } else { texels[0] };
            return self.converted(chroma, luma.b);
        }

        let center = m.mid_texel && s_fraction == 0x10 && t_fraction == 0x10;
        let center_rg = m.mid_texel && s_fraction_rg == 0x10 && t_fraction == 0x10;
        let [t0, t1, t2, t3] = texels;

        if convert {
            let p = signed_nine(previous);
            return Color {
                r: converted_corners(t0.r, t1.r, t2.r, t3.r, p, upper_rg, center_rg),
                g: converted_corners(t0.g, t1.g, t2.g, t3.g, p, upper_rg, center_rg),
                b: converted_corners(t0.b, t1.b, t2.b, t3.b, p, upper, center),
                a: converted_corners(t0.a, t1.a, t2.a, t3.a, p, upper, center),
            };
        }

        Color {
            r: interpolated(t0.r, t1.r, t2.r, t3.r, s_fraction_rg, t_fraction, upper_rg, center_rg),
            g: interpolated(t0.g, t1.g, t2.g, t3.g, s_fraction_rg, t_fraction, upper_rg, center_rg),
            b: interpolated(t0.b, t1.b, t2.b, t3.b, s_fraction, t_fraction, upper, center),
            a: interpolated(t0.a, t1.a, t2.a, t3.a, s_fraction, t_fraction, upper, center),
        }
    }

    /// The texel, the one to its right, the one below and the one diagonal; YUV reads chroma a further step along.
    #[inline(always)]
    fn texels(&self, texels: &mut [Color; 4], s0: i32, s_step: i32, t0: i32, t_step: i32, tile: &TextureTile) {
        let row0 = t0 & 0xFF;
        let row1 = row0 + t_step;
        let s1 = s0 + s_step;

        texels[0] = self.fetch_texel(s0, row0, tile);
        texels[2] = self.fetch_texel(s0, row1, tile);

        if tile.format != 1 {
            texels[1] = self.fetch_texel(s1, row0, tile);
            texels[3] = self.fetch_texel(s1, row1, tile);
            return;
        }

        texels[1] = self.fetch_texel(s1 + s_step, row0, tile);
        texels[3] = self.fetch_texel(s1 + s_step, row1, tile);
        if tile.size < 2 {
            return;
        }

        let luma1 = self.fetch_texel(s1, row0, tile);
        let luma3 = self.fetch_texel(s1, row1, tile);
        texels[1].b = luma1.b;
        texels[1].a = luma1.a;
        texels[3].b = luma3.b;
        texels[3].a = luma3.a;
    }

    /// Each of the four texels' palette entries, taken from its own bank of four.
    #[allow(clippy::too_many_arguments)]
    fn palette_texels(&self, texels: &mut [Color; 4], s0: i32, s_step: i32, t0: i32, t_step: i32, tile: &TextureTile, upper_rg: bool) {
        let row0 = t0 & 0xFF;
        let row1 = row0 + t_step;
        let s1 = s0 + if tile.format == 1 { s_step << 1 } else { s_step };

        texels[0] = self.palette_color(self.palette_index(s0, row0, tile) << 2, upper_rg);
        texels[1] = self.palette_color((self.palette_index(s1, row0, tile) << 2) | 1, upper_rg);
        texels[2] = self.palette_color((self.palette_index(s0, row1, tile) << 2) | 2, upper_rg);
        texels[3] = self.palette_color((self.palette_index(s1, row1, tile) << 2) | 3, upper_rg);
    }

    /// One texel's palette entry from all four banks.
    fn nearest_palette_texels(&self, texels: &mut [Color; 4], s: i32, t: i32, tile: &TextureTile, upper_rg: bool) {
        let entry = self.palette_index(s, t, tile) << 2;
        for (bank, texel) in texels.iter_mut().enumerate() {
            *texel = self.palette_color(entry | bank as i32, upper_rg);
        }
    }

    /// A nibble with the tile's palette number, a byte, or a word's high byte; YUV always reads bytes.
    #[inline(always)]
    fn palette_index(&self, s: i32, row: i32, tile: &TextureTile) -> i32 {
        let tmem = &*self.texture_memory;
        let line = tile.line.wrapping_mul(row).wrapping_add(tile.memory);
        let odd = (row & 1) != 0;
        let byte_swap = if odd { 4 } else { 0 };
        let word_swap = if odd { 2 } else { 0 };
        let yuv = tile.format == 1;

        if tile.size == 0 && !yuv {
            let value = tmem[(((((line << 4) + s) >> 1) ^ byte_swap) & 0x7FF) as usize] as i32;
            return (tile.palette << 4) | if (s & 1) != 0 { value & 0xF } else { value >> 4 };
        }

        if tile.size >= 2 && !yuv {
            return self.texture_word((((line << 2) + s) ^ word_swap) & 0x3FF) >> 8;
        }

        let bytes = tmem[((((line << 3) + s) ^ byte_swap) & 0x7FF) as usize] as i32;
        if tile.size == 0 { (tile.palette << 4) | (bytes >> 4) } else { bytes }
    }

    /// The palette's upper half of texture memory, its bank reversed in the upper chroma triangle, as RGBA16 or IA16.
    #[inline(always)]
    fn palette_color(&self, entry: i32, upper_rg: bool) -> Color {
        let value = self.texture_word(0x400 | if upper_rg { entry ^ 3 } else { entry });

        if self.modes.palette_intensity_alpha {
            Color { r: value >> 8, g: value >> 8, b: value >> 8, a: value & 0xFF }
        } else {
            Color {
                r: FIVE_TO_EIGHT[(value >> 11) as usize] as i32,
                g: FIVE_TO_EIGHT[((value >> 6) & 0x1F) as usize] as i32,
                b: FIVE_TO_EIGHT[((value >> 1) & 0x1F) as usize] as i32,
                a: if (value & 1) != 0 { 0xFF } else { 0 },
            }
        }
    }
}
