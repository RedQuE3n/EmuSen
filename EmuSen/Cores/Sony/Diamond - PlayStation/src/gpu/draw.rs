//! The pixel pipeline every rendering command shares (psx-spx "GPU (MISC)", "GP0(E6h)"): the texel and its
//! transparency, modulation, the dither matrix, the four semi-transparency modes, the mask bit, the drawing area and
//! the interlaced field. Colours enter at eight bits a channel and leave at five (Diamond_GPU.md §5).

use super::texture::{ClutCache, Texture, TextureCache, texel};
use super::vram::{Mask, Vram};

/// psx-spx's dither offsets, by y and x modulo 4.
pub const DITHER: [[i32; 4]; 4] = [[-4, 0, -3, 1], [2, -2, 3, -1], [-3, 1, -4, 0], [3, -1, 2, -2]];

/// The drawing area, inclusive at both corners.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Area {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

/// What a primitive does with each pixel it covers.
#[derive(Clone, Copy, Debug)]
pub struct Paint {
    /// The semi-transparency mode when the primitive is semi-transparent.
    pub blend: Option<u32>,
    pub texture: Option<Texture>,
    /// A texture used as it is, not modulated by the colour.
    pub raw: bool,
    pub dither: bool,
    pub mask: Mask,
    pub area: Area,
    /// In 480-line interlaced mode with drawing to the displayed field prohibited, the parity of the lines left alone.
    pub skip_parity: Option<i32>,
    /// The 160-pin GPU crops the colour to five bits before modulating (psx-spx "Shaded Textures").
    pub crop_modulation: bool,
}

/// What a primitive did, for its time: texels fetched, pixels written and pixels read before they were written (or
/// left), the texture cache's misses, and the rows it spanned.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Work {
    pub texels: u32,
    pub writes: u32,
    pub read_writes: u32,
    pub misses: u32,
    pub rows: u32,
}

pub struct Painter<'a> {
    pub vram: &'a mut Vram,
    pub cache: &'a mut TextureCache,
    pub clut: &'a ClutCache,
    pub paint: Paint,
    pub work: Work,
}

impl Painter<'_> {
    #[inline]
    pub fn visible_row(&self, y: i32) -> bool {
        self.paint.skip_parity != Some(y & 1)
    }

    /// A pixel already inside the area, at eight bits a channel and its texture coordinate.
    #[inline]
    pub fn pixel(&mut self, x: i32, y: i32, rgb: [i32; 3], uv: (u8, u8)) {
        let p = self.paint;
        let (xu, yu) = (x as u32, y as u32);
        let (mut c, flag) = match p.texture {
            Some(t) => {
                self.work.texels += 1;
                let tx = texel(self.vram, self.cache, self.clut, &t, uv.0, uv.1, &mut self.work.misses);
                if tx == 0 {
                    return;
                }
                let t5 = [(tx & 31) as i32, (tx >> 5 & 31) as i32, (tx >> 10 & 31) as i32];
                let c = if p.raw {
                    [t5[0] << 3, t5[1] << 3, t5[2] << 3]
                } else if p.crop_modulation {
                    [0, 1, 2].map(|i| ((rgb[i] >> 3) * t5[i]) >> 1)
                } else {
                    [0, 1, 2].map(|i| (rgb[i] * t5[i]) >> 4)
                };
                (c, tx & 0x8000)
            }
            None => (rgb, 0),
        };
        if p.dither {
            let d = DITHER[(y & 3) as usize][(x & 3) as usize];
            c = c.map(|v| v + d);
        }
        let c = c.map(|v| (v.clamp(0, 255) >> 3) as u16);
        let blended = p.blend.filter(|_| p.texture.is_none() || flag != 0);
        if blended.is_some() || p.mask.check {
            self.work.read_writes += 1;
        } else {
            self.work.writes += 1;
        }
        let old = self.vram.get(xu, yu);
        if p.mask.check && old & 0x8000 != 0 {
            return;
        }
        let mut out = c[0] | c[1] << 5 | c[2] << 10;
        if let Some(mode) = blended {
            out = blend(old, out, mode);
        }
        self.vram.set(xu, yu, out | flag | p.mask.set);
    }
}

/// B the pixel in VRAM, F the new one, each channel at five bits, saturated: B/2+F/2, B+F, B-F, B+F/4.
#[inline]
pub fn blend(back: u16, front: u16, mode: u32) -> u16 {
    let mut out = 0;
    for shift in [0, 5, 10] {
        let b = (back >> shift & 31) as i32;
        let f = (front >> shift & 31) as i32;
        let v = match mode {
            0 => (b + f) >> 1,
            1 => (b + f).min(31),
            2 => (b - f).max(0),
            _ => (b + (f >> 2)).min(31),
        };
        out |= (v as u16) << shift;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_four_modes_saturate() {
        assert_eq!(blend(0x7FFF, 0x7FFF, 0), 0x7FFF);
        assert_eq!(blend(31, 31, 1), 31);
        assert_eq!(blend(4, 31, 2), 0);
        assert_eq!(blend(30, 31, 3), 31);
        assert_eq!(blend(10, 8, 3), 12);
    }
}
