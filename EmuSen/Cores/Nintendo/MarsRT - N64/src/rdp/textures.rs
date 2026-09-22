//! Texture rectangles, coordinates and perspective division, and a texel from texture memory: C#'s `Rdp.Textures.cs`.

use super::tables::{DIVIDE_TABLE, FIVE_TO_EIGHT};
use super::{ATTRIBUTE_S, ATTRIBUTE_T, Color, Rdp, RdpMemory, TextureTile};

/// The product scaled by the reciprocal's shift, flagged over or under when bits beyond the range differ.
#[inline(always)]
fn divided(mut product: i32, shift: i32) -> i32 {
    let range = ((1 << 30) - 1) & -((1 << 29) >> shift);
    let outside = product & range;

    let scaled = if shift != 0xE {
        product >>= 13 - shift;
        product
    } else {
        product << 1
    };

    let flags = if outside == range || outside == 0 {
        0
    } else if (product & (1 << 29)) == 0 {
        2 << 17
    } else {
        1 << 17
    };
    (scaled & 0x1FFFF) | flags
}

#[inline(always)]
fn clamp_coordinate(value: i32) -> i32 {
    if (value & 0x40000) != 0 {
        return 0x7FFF;
    }
    if (value & 0x20000) != 0 {
        return 0x8000;
    }
    match value & 0x18000 {
        0x8000 => 0x7FFF,
        0x10000 => 0x8000,
        _ => value & 0xFFFF,
    }
}

#[inline(always)]
pub(super) fn shifted(coordinate: i32, shift: i32) -> i32 {
    if shift < 11 { (coordinate as i16 as i32) >> shift } else { (coordinate << (16 - shift)) as i16 as i32 }
}

#[inline(always)]
pub(super) fn clamped(coordinate: i32, clamps: bool, beyond: bool, limit: i32) -> i32 {
    if !clamps {
        return coordinate >> 5;
    }
    if beyond {
        return limit;
    }
    if (coordinate & 0x10000) == 0 { coordinate >> 5 } else { 0 }
}

#[inline(always)]
pub(super) fn masked(coordinate: i32, mask: i32, mirror: bool, mirror_bit: i32) -> i32 {
    let coordinate = if mirror && ((coordinate >> mirror_bit) & 1) != 0 { !coordinate } else { coordinate };
    coordinate & (0xFFFF >> (16 - mask)) & 0x3FF
}

#[inline(always)]
pub(super) fn signed_nine(color: Color) -> Color {
    Color { r: (color.r << 23) >> 23, g: (color.g << 23) >> 23, b: (color.b << 23) >> 23, a: color.a }
}

#[inline(always)]
fn gray(value: i32) -> Color {
    let v = value & 0xFF;
    Color { r: v, g: v, b: v, a: v }
}

impl Rdp {
    /// A rectangle carrying s and t and their steps, the steps swapped between x and y when flipped.
    pub(super) fn textured_rectangle(&mut self, flipped: bool, mem: &mut RdpMemory) {
        let word = self.command[0];
        let coordinates = self.command[1];
        self.clear_attributes();

        let dsdx = ((coordinates >> 16) as i16 as i32) << 11;
        let dtdy = (coordinates as i16 as i32) << 11;
        self.attribute_value[ATTRIBUTE_S] = ((coordinates >> 48) as i32) << 16;
        self.attribute_value[ATTRIBUTE_T] = (((coordinates >> 32) & 0xFFFF) as i32) << 16;

        if flipped {
            self.attribute_dx[ATTRIBUTE_T] = dtdy;
            self.attribute_de[ATTRIBUTE_S] = dsdx;
            self.attribute_dy[ATTRIBUTE_S] = dsdx;
        } else {
            self.attribute_dx[ATTRIBUTE_S] = dsdx;
            self.attribute_de[ATTRIBUTE_T] = dtdy;
            self.attribute_dy[ATTRIBUTE_T] = dtdy;
        }

        let rows = self.walk_rectangle(word);
        self.draw(rows, true, ((word >> 24) & 7) as i32, 0, mem);
    }

    #[inline(always)]
    pub(super) fn texture_coordinates(&self, s: i32, t: i32, w: i32) -> (i32, i32) {
        let (s, t) = self.divided_coordinates(s, t, w);
        (clamp_coordinate(s), clamp_coordinate(t))
    }

    /// s and t over w, or s and t alone, to seventeen bits and two overflow flags.
    #[inline(always)]
    pub(super) fn divided_coordinates(&self, s: i32, t: i32, w: i32) -> (i32, i32) {
        let (s, t, w) = (s >> 16, t >> 16, w >> 16);

        if !self.modes.perspective {
            return ((s as i16 as i32) & 0x1FFFF, (t as i16 as i32) & 0x1FFFF);
        }

        let flags = if (w as i16) <= 0 { 2 << 17 } else { 0 };
        let entry = DIVIDE_TABLE[(w & 0x7FFF) as usize];
        let reciprocal = entry >> 4;
        let shift = entry & 0xF;

        (divided((s as i16 as i32) * reciprocal, shift) | flags, divided((t as i16 as i32) * reciprocal, shift) | flags)
    }

    /// Shifted and made relative to the tile's corner, then sampled at one texel, or at four when filtering or a palette asks.
    #[inline(always)]
    pub(super) fn texel(&self, s: i32, t: i32, tile_index: i32, bilinear: bool, convert: bool, previous: Color) -> Color {
        let tile = &self.tiles[tile_index as usize];

        let s = shifted(s, tile.shift_s);
        let t = shifted(t, tile.shift_t);
        let beyond_s = (s >> 3) >= tile.sh;
        let beyond_t = (t >> 3) >= tile.th;

        let s = s - (tile.sl << 3);
        let t = t - (tile.tl << 3);

        if self.modes.sample_four || self.modes.palette_enabled {
            self.four_texels(s, t, beyond_s, beyond_t, tile, bilinear, convert, previous)
        } else {
            self.point_texel(s, t, beyond_s, beyond_t, tile, bilinear, convert, previous)
        }
    }

    /// Clamped, masked and mirrored, fetched, and converted from YUV unless filtered.
    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    fn point_texel(&self, s: i32, t: i32, beyond_s: bool, beyond_t: bool, tile: &TextureTile, bilinear: bool, convert: bool, previous: Color) -> Color {
        if convert {
            return if bilinear {
                Color { r: previous.b, g: previous.b, b: previous.b, a: previous.b }
            } else {
                let p = signed_nine(previous);
                self.converted(p, p.b)
            };
        }

        let mut s = clamped(s, tile.clamps_s(), beyond_s, tile.clamp_limit_s());
        let mut t = clamped(t, tile.clamps_t(), beyond_t, tile.clamp_limit_t());

        if tile.mask_s != 0 {
            s = masked(s, tile.mask_s, tile.mirror_s, tile.mirror_bit_s());
        }
        if tile.mask_t != 0 {
            t = masked(t, tile.mask_t, tile.mirror_t, tile.mirror_bit_t());
        }

        let texel = self.fetch_texel(s, t & 0xFF, tile);
        if bilinear { texel } else { self.converted(texel, texel.b) }
    }

    /// The luma plus the chroma through the four conversion constants.
    #[inline(always)]
    pub(super) fn converted(&self, chroma: Color, luma: i32) -> Color {
        Color {
            r: (luma + ((self.k0 * chroma.g + 0x80) >> 8)) & 0x1FF,
            g: (luma + ((self.k1 * chroma.r + self.k2 * chroma.g + 0x80) >> 8)) & 0x1FF,
            b: (luma + ((self.k3 * chroma.r + 0x80) >> 8)) & 0x1FF,
            a: luma & 0x1FF,
        }
    }

    /// Each format and size from its place in texture memory; formats past four read as intensity.
    #[inline(always)]
    pub(super) fn fetch_texel(&self, s: i32, row: i32, tile: &TextureTile) -> Color {
        let tmem = &*self.texture_memory;
        let line = tile.line.wrapping_mul(row).wrapping_add(tile.memory);
        let odd = (row & 1) != 0;
        let byte_swap = if odd { 4 } else { 0 };
        let word_swap = if odd { 2 } else { 0 };

        let format = if tile.format < 5 { tile.format } else { 4 };
        match (format << 2) | tile.size {
            0x0 | 0x10 => {
                let value = self.nibble(((line << 4) + s) >> 1, s, byte_swap);
                gray(value | (value << 4))
            }
            0x1 | 0x9 | 0x11 => gray(tmem[((((line << 3) + s) ^ byte_swap) & 0xFFF) as usize] as i32),
            0x2 => {
                let value = self.texture_word((((line << 2) + s) ^ word_swap) & 0x7FF);
                Color {
                    r: FIVE_TO_EIGHT[(value >> 11) as usize] as i32,
                    g: FIVE_TO_EIGHT[((value >> 6) & 0x1F) as usize] as i32,
                    b: FIVE_TO_EIGHT[((value >> 1) & 0x1F) as usize] as i32,
                    a: if (value & 1) != 0 { 0xFF } else { 0 },
                }
            }
            0x3 => {
                let index = (((line << 2) + s) ^ word_swap) & 0x3FF;
                let first = self.texture_word(index);
                let second = self.texture_word(index | 0x400);
                Color { r: first >> 8, g: first & 0xFF, b: second >> 8, a: second & 0xFF }
            }
            0x4 | 0x5 => {
                let mut value = tmem[((((line << 3) + s) ^ byte_swap) & 0x7FF) as usize] as i32;
                if tile.size == 0 {
                    value = (value & 0xF0) | ((value & 0xF0) >> 4);
                }
                Color { r: value - 0x80, g: value - 0x80, b: value, a: value }
            }
            0x6 | 0x7 => {
                let bytes = (line << 3) + s;
                let chroma = self.texture_word(((bytes >> 1) ^ word_swap) & 0x3FF);
                let mut texel = Color { r: (chroma >> 8) - 0x80, g: (chroma & 0xFF) - 0x80, b: 0, a: 0 };

                if tile.size == 2 || (s & 1) != 0 {
                    let v = tmem[(((bytes ^ byte_swap) & 0x7FF) | 0x800) as usize] as i32;
                    texel.b = v;
                    texel.a = v;
                } else {
                    let luma = self.texture_word((((bytes >> 1) ^ word_swap) & 0x3FF) | 0x400);
                    texel.b = luma >> 8;
                    texel.a = ((luma >> 8) & 0xF) | (luma & 0xF0);
                }
                texel
            }
            0x8 => {
                let value = self.nibble(((line << 4) + s) >> 1, s, byte_swap);
                gray((tile.palette << 4) | value)
            }
            0xC => {
                let value = self.nibble(((line << 4) + s) >> 1, s, byte_swap);
                let intensity = value & 0xE;
                let i = ((intensity << 4) | (intensity << 1) | (intensity >> 2)) & 0xFF;
                Color { r: i, g: i, b: i, a: if (value & 1) != 0 { 0xFF } else { 0 } }
            }
            0xD => {
                let value = tmem[((((line << 3) + s) ^ byte_swap) & 0xFFF) as usize] as i32;
                let intensity = (value & 0xF0) | ((value & 0xF0) >> 4);
                Color { r: intensity, g: intensity, b: intensity, a: ((value & 0xF) << 4) | (value & 0xF) }
            }
            0xE => {
                let value = self.texture_word((((line << 2) + s) ^ word_swap) & 0x7FF);
                Color { r: value >> 8, g: value >> 8, b: value >> 8, a: value & 0xFF }
            }
            _ => {
                let value = self.texture_word((((line << 2) + s) ^ word_swap) & 0x7FF);
                Color { r: value >> 8, g: value & 0xFF, b: value >> 8, a: value & 0xFF }
            }
        }
    }

    /// The high nibble for an even s and the low for an odd one.
    #[inline(always)]
    fn nibble(&self, index: i32, s: i32, byte_swap: i32) -> i32 {
        let value = self.texture_memory[((index ^ byte_swap) & 0xFFF) as usize] as i32;
        if (s & 1) != 0 { value & 0xF } else { value >> 4 }
    }

    #[inline(always)]
    pub(super) fn texture_word(&self, index: i32) -> i32 {
        let at = index as usize * 2;
        ((self.texture_memory[at] as i32) << 8) | self.texture_memory[at + 1] as i32
    }
}
