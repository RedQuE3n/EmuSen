//! A rendering command's words as the GPU reads them (Diamond_GPU.md §2, §4, §6): the vertices under the drawing offset,
//! a polygon's, a rectangle's and a line's parts, the drawing area and the page attribute; and `Drawing`, the state a
//! command is drawn under. The software rasteriser draws by these, and a renderer told the command stream decodes it by
//! the same functions rather than a copy of them (Diamond_Enhancements.md §15.2).

use super::draw::{Area, Paint};
use super::raster::{Sprite, Vertex};
use super::texture::{ClutCache, Texture};
use super::vram::Mask;

fn sext11(v: u32) -> i32 {
    ((v << 21) as i32) >> 21
}

fn rgb(word: u32) -> [i32; 3] {
    [(word & 0xFF) as i32, (word >> 8 & 0xFF) as i32, (word >> 16 & 0xFF) as i32]
}

/// A vertex with the drawing offset GP0(E5h) added in 11 bits, wrapping (Diamond_Disputes.md D-23).
#[inline]
pub fn vertex(offset: u32, xy: u32, colour: u32, uv: u32) -> Vertex {
    let x = sext11((sext11(xy) + sext11(offset)) as u32);
    let y = sext11((sext11(xy >> 16) + sext11(offset >> 11)) as u32);
    Vertex { x, y, rgb: rgb(colour), u: uv as u8, v: (uv >> 8) as u8 }
}

/// `v` placed from `first` by their distance as the command gave them, the offset moving the primitive whole (D-23).
#[inline]
pub fn place(offset: u32, first: Vertex, v: Vertex) -> Vertex {
    let (ox, oy) = (sext11(offset), sext11(offset >> 11));
    let raw = |p: i32, o: i32| sext11((p - o) as u32);
    Vertex { x: first.x + raw(v.x, ox) - raw(first.x, ox), y: first.y + raw(v.y, oy) - raw(first.y, oy), ..v }
}

/// The drawing area from GP0(E3h) and (E4h): Y ten bits on the 208-pin chip, stopping at 511 while GP1(09h) gives it a
/// second megabyte (Diamond_GPU.md §5).
#[inline]
pub fn area(e3: u32, e4: u32, v2: bool, two_megabytes: bool) -> Area {
    let y = |v: u32| (v >> 10 & if v2 { 0x3FF } else { 0x1FF }) as i32;
    let y2 = if two_megabytes { y(e4).min(511) } else { y(e4) };
    Area { x1: (e3 & 0x3FF) as i32, y1: y(e3), x2: (e4 & 0x3FF) as i32, y2 }
}

/// GP0(E1h) with a polygon's page attribute taken into bits 0-8, and bit 11 while GP1(09h) allows it.
#[inline]
pub fn with_page(e1: u32, page: u32, two_megabytes: bool) -> u32 {
    let bits = 0x1FF | if two_megabytes { 0x800 } else { 0 };
    e1 & !bits | page & bits
}

/// A polygon's words: its three or four vertices, each placed from the first, and its flags and attributes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Polygon {
    pub v: [Vertex; 4],
    pub quad: bool,
    pub shaded: bool,
    pub textured: bool,
    pub blended: bool,
    pub raw: bool,
    /// The first vertex's CLUT attribute and the second's page attribute.
    pub clut: u32,
    pub page: u32,
}

#[inline]
pub fn polygon(w: &[u32], offset: u32) -> Polygon {
    let c = w[0] >> 24;
    let (quad, shaded, textured, blended, raw) = (c & 8 != 0, c & 0x10 != 0, c & 4 != 0, c & 2 != 0, c & 1 != 0);
    let n = if quad { 4 } else { 3 };
    let mut v = [Vertex::default(); 4];
    let mut at = 1;
    let (mut clut, mut page) = (0, 0);
    for (i, vx) in v.iter_mut().enumerate().take(n) {
        let colour = if i > 0 && shaded {
            at += 1;
            w[at - 1]
        } else {
            w[0]
        };
        let xy = w[at];
        at += 1;
        let uv = if textured {
            at += 1;
            w[at - 1]
        } else {
            0
        };
        if i == 0 {
            clut = uv >> 16;
        } else if i == 1 {
            page = uv >> 16;
        }
        *vx = vertex(offset, xy, colour, uv);
    }
    for i in 1..n {
        v[i] = place(offset, v[0], v[i]);
    }
    Polygon { v, quad, shaded, textured, blended, raw, clut, page }
}

/// A rectangle's words: its corner under the offset, its size, colour and texture coordinate, with GP0(E1h)'s `flip`;
/// whether it is textured, semi-transparent and raw; and its CLUT attribute.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rectangle {
    pub sprite: Sprite,
    pub textured: bool,
    pub blended: bool,
    pub raw: bool,
    pub clut: u32,
}

#[inline]
pub fn rectangle(w: &[u32], offset: u32, flip: (bool, bool)) -> Rectangle {
    let c = w[0] >> 24;
    let (textured, blended, raw) = (c & 4 != 0, c & 2 != 0, c & 1 != 0);
    let mut at = 2;
    let uv = if textured {
        at += 1;
        w[2]
    } else {
        0
    };
    let (sw, sh) = match c >> 3 & 3 {
        0 => ((w[at] & 0x3FF) as i32, (w[at] >> 16 & 0x1FF) as i32),
        1 => (1, 1),
        2 => (8, 8),
        _ => (16, 16),
    };
    let v = vertex(offset, w[1], w[0], uv);
    let sprite = Sprite { x: v.x, y: v.y, w: sw, h: sh, rgb: v.rgb, uv: (v.u, v.v), flip };
    Rectangle { sprite, textured, blended, raw, clut: uv >> 16 }
}

/// A line's first segment: its two ends, the second placed from the first, and whether it is shaded.
#[inline]
pub fn line(w: &[u32], offset: u32) -> (Vertex, Vertex, bool) {
    let shaded = w[0] >> 24 & 0x10 != 0;
    let a = vertex(offset, w[1], w[0], 0);
    let b = place(offset, a, if shaded { vertex(offset, w[3], w[2], 0) } else { vertex(offset, w[2], w[0], 0) });
    (a, b, shaded)
}

/// The drawing state a command is drawn under, as the GPU holds it when the command starts: GP0(E1h) to (E6h), the chip's
/// version, GP1(09h), the parity of the interlaced field's lines left alone, and the CLUT cache's tag (Diamond_GPU.md §13,
/// Diamond_Enhancements.md §15.2).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Drawing {
    pub env: [u32; 6],
    pub version: u32,
    pub second_megabyte: bool,
    pub skip_parity: Option<i32>,
    /// The CLUT cache's position and how many entries it holds.
    pub clut: (u32, u32, u32),
}

impl Drawing {
    pub fn v2(&self) -> bool {
        self.version != 0
    }

    pub fn two_megabytes(&self) -> bool {
        self.v2() && self.second_megabyte
    }

    pub fn area(&self) -> Area {
        area(self.env[2], self.env[3], self.v2(), self.two_megabytes())
    }

    pub fn mask(&self) -> Mask {
        Mask::from_e6(self.env[5])
    }

    /// GP0(E1h)'s flips, the 208-pin chip's alone.
    pub fn flips(&self) -> (bool, bool) {
        if self.v2() { (self.env[0] & 1 << 12 != 0, self.env[0] & 1 << 13 != 0) } else { (false, false) }
    }

    /// The state with a polygon's page attribute taken into GP0(E1h), as the GPU takes it before drawing the polygon.
    pub fn with_page(self, page: u32) -> Drawing {
        Drawing { env: [with_page(self.env[0], page, self.two_megabytes()), self.env[1], self.env[2], self.env[3], self.env[4], self.env[5]], ..self }
    }

    pub fn texture(&self, clut: u32) -> Texture {
        Texture::new(self.env[0], clut, self.env[1], self.two_megabytes())
    }

    /// Whether the GPU's CLUT cache loads `t`'s palette before drawing with it.
    pub fn loads_clut(&self, t: &Texture) -> bool {
        let (x, y, loaded) = self.clut;
        !ClutCache::holds(x, y, loaded, t)
    }

    /// What a primitive does with each pixel, as `Gpu::paint` makes it.
    pub fn paint(&self, blended: bool, texture: Option<Texture>, raw: bool, dither: bool) -> Paint {
        let e1 = self.env[0];
        Paint {
            blend: blended.then_some(e1 >> 5 & 3),
            texture,
            raw,
            dither: dither && e1 & 1 << 9 != 0,
            mask: self.mask(),
            area: self.area(),
            skip_parity: self.skip_parity,
            crop_modulation: !self.v2(),
        }
    }
}
