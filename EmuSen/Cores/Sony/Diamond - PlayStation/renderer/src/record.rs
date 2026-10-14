//! A primitive as the walk's shader reads it: one record of `WORDS` words, made from what the GPU's own decoding and
//! setup give (`diamond::gpu::decode`, `raster::setup`), with the VRAM it writes and the VRAM it reads as texels. The
//! layout is `shaders/walk.comp`'s (Diamond_Enhancements.md §16.3).

use diamond::gpu::draw::Paint;
use diamond::gpu::raster::{self, FRACTION, MAX_DX, MAX_DY, Sprite, Vertex};
use precision::Fraction;
use diamond::gpu::texture::Texture;
use diamond::gpu::vram::{Mask, Rect};

pub const WORDS: usize = 48;

pub const TRIANGLE: u32 = 1;
pub const RECTANGLE: u32 = 2;
pub const LINE: u32 = 3;
pub const FILL: u32 = 4;
pub const TRANSFER: u32 = 5;
pub const PRECISE: u32 = 6;

pub const SHADED: u32 = 1 << 8;
pub const TEXTURED: u32 = 1 << 9;
pub const RAW: u32 = 1 << 10;
pub const DITHER: u32 = 1 << 11;
pub const BLEND: u32 = 1 << 12;
pub const CHECK: u32 = 1 << 15;
pub const SET: u32 = 1 << 16;
pub const CROP: u32 = 1 << 17;
pub const SKIP: u32 = 1 << 18;
pub const PARITY: u32 = 1 << 19;
pub const PAIRS: u32 = 1 << 20;
pub const PERSPECTIVE: u32 = 1 << 21;
/// A precise triangle's: the bit of its first edge's bound, the next two edges' after it; that it is filled to the
/// console's own triangle; and the first of that triangle's edges' bounds, with whether the console winds it the other
/// way (Diamond_Enhancements.md §33).
pub const BOUNDS: u32 = 22;
pub const FILLED: u32 = 1 << 25;
pub const OWN_BOUNDS: u32 = 26;
pub const TURNED: u32 = 1 << 29;
/// A textured primitive drawn with its texels mixed bilinearly (Diamond_Enhancements.md §41).
pub const FILTERED: u32 = 1 << 30;

/// Columns `x0..=x1` and rows `y0..=y1` of VRAM, either running past the edge and wrapping round; at most 1,024 columns
/// and 512 rows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub x0: u32,
    pub x1: u32,
    pub y0: u32,
    pub y1: u32,
}

impl Span {
    /// Drawing coordinates `x0..=x1` and `y0..=y1` as VRAM holds them, rows past 511 on the row modulo 512.
    pub fn new(x0: i32, x1: i32, y0: i32, y1: i32) -> Span {
        let (bx, by) = (x0.rem_euclid(1024), y0.rem_euclid(512));
        Span { x0: bx as u32, x1: (bx + (x1 - x0).clamp(0, 1023)) as u32, y0: by as u32, y1: (by + (y1 - y0).clamp(0, 511)) as u32 }
    }

    /// `w` columns from `x` and `h` rows from `y`, wrapping.
    pub fn sized(x: u32, y: u32, w: u32, h: u32) -> Span {
        let (x, y) = (x & 1023, y & 511);
        Span { x0: x, x1: x + w.clamp(1, 1024) - 1, y0: y, y1: y + h.clamp(1, 512) - 1 }
    }
}

/// A record and what it touches: the VRAM it writes, and the VRAM it reads as texels and as a palette.
#[derive(Clone, Copy, Debug)]
pub struct Record {
    pub words: [i32; WORDS],
    pub writes: Span,
    pub texels: Option<Span>,
}

/// The flags and the texture's words every pixel-drawing record shares: its kind, how it paints, and its texture's page,
/// depth, CLUT slot and window.
fn painted(kind: u32, paint: &Paint, slot: u32, words: &mut [i32; WORDS]) {
    let mut flags = kind;
    if let Some(mode) = paint.blend {
        flags |= BLEND | mode << 13;
    }
    if paint.raw {
        flags |= RAW;
    }
    if paint.dither {
        flags |= DITHER;
    }
    flags |= mask_flags(paint.mask);
    if paint.crop_modulation {
        flags |= CROP;
    }
    if let Some(parity) = paint.skip_parity {
        flags |= SKIP | (parity as u32 & 1) << 19;
    }
    if let Some(t) = paint.texture {
        flags |= TEXTURED;
        words[5] = (t.page_x | t.page_y << 16) as i32;
        words[6] = (t.depth | slot << 8) as i32;
        words[7] = (t.mask_u as u32 | (t.mask_v as u32) << 8 | (t.off_u as u32) << 16 | (t.off_v as u32) << 24) as i32;
    }
    words[0] = flags as i32;
}

fn mask_flags(mask: Mask) -> u32 {
    (if mask.check { CHECK } else { 0 }) | if mask.set != 0 { SET } else { 0 }
}

fn bounds(words: &mut [i32; WORDS], x: (i32, i32), y: (i32, i32)) {
    [words[1], words[2], words[3], words[4]] = [x.0, x.1, y.0, y.1];
}

fn colour(rgb: [i32; 3]) -> i32 {
    rgb[0] | rgb[1] << 8 | rgb[2] << 16
}

/// The halfwords a texture coordinate range `lo..=hi` reads, per axis, through the window (`mask`, `offset`): the whole
/// axis when the window moves it or the range wraps past 255.
fn texel_range(lo: i32, hi: i32, mask: u8, offset: u8) -> (i32, i32) {
    if mask != 0 {
        let fixed = (offset & mask) as i32;
        return (fixed, fixed | (!mask) as i32);
    }
    if lo < 0 || hi > 255 { (0, 255) } else { (lo, hi) }
}

/// The VRAM a texture's coordinates `u` and `v` (each a range, before the window) read: the page's halfwords at its
/// depth, rows at or past 512 reading nothing (`texture.rs`'s `texel`).
fn texture_span(t: &Texture, u: (i32, i32), v: (i32, i32)) -> Option<Span> {
    let (u0, u1) = texel_range(u.0, u.1, t.mask_u, t.off_u);
    let (v0, v1) = texel_range(v.0, v.1, t.mask_v, t.off_v);
    let shift = [2, 1, 0][t.depth as usize];
    let (y0, y1) = (t.page_y as i32 + v0, (t.page_y as i32 + v1).min(511));
    if y0 > 511 {
        return None;
    }
    let x0 = t.page_x as i32 + (u0 >> shift);
    let x1 = t.page_x as i32 + (u1 >> shift);
    Some(Span { x0: x0 as u32 & 1023, x1: (x0 as u32 & 1023) + (x1 - x0) as u32, y0: y0 as u32, y1: y1 as u32 })
}

/// The palette a texture reads: its CLUT's row, 16 or 256 entries wrapping in x.
pub fn clut_span(t: &Texture) -> Span {
    Span::sized(t.clut_x, t.clut_y, t.entries(), 1)
}

/// Whether a textured triangle's texture is drawn a texel a pixel and unrotated: its gradients in the console's 12-bit
/// units one whole texel along one axis and nothing along the other, the sprites and menus a game builds of polygons,
/// which the filtering of 3D polygons leaves crisp (Diamond_Enhancements.md §41).
pub fn texel_a_pixel(r: &Record) -> bool {
    let w = &r.words;
    let one = 1 << FRACTION;
    let unit = |a: i32, b: i32| (a.abs() == one && b == 0) || (a == 0 && b.abs() == one);
    unit(w[38], w[39]) && unit(w[40], w[41]) && (w[38] == 0) == (w[41] == 0)
}

/// A triangle, folded, under `paint`, its palette in `slot`; none when it draws nothing (`raster::triangle`).
pub fn triangle(v: [Vertex; 3], shaded: bool, textured: bool, paint: &Paint, slot: u32) -> Option<Record> {
    let s = raster::setup(&v, shaded, textured)?;
    let a = paint.area;
    let (y_lo, y_hi) = (s.min.1.max(a.y1), (s.max.1 - 1).min(a.y2));
    let (x_lo, x_hi) = (s.min.0.max(a.x1), s.max.0.min(a.x2));
    if y_lo > y_hi || x_lo > x_hi {
        return None;
    }
    let mut words = [0i32; WORDS];
    painted(TRIANGLE, paint, slot, &mut words);
    if shaded {
        words[0] |= SHADED as i32;
    }
    bounds(&mut words, (x_lo, x_hi), (y_lo, y_hi));
    words[8] = colour(v[0].rgb);
    for (e, &((ax, ay), (bx, by), own)) in s.edges.iter().enumerate() {
        words[10 + 5 * e..15 + 5 * e].copy_from_slice(&[ax as i32, ay as i32, (bx - ax) as i32, (by - ay) as i32, if own { 0 } else { 1 }]);
    }
    [words[25], words[26]] = [v[s.b].x, v[s.b].y];
    for k in 0..5 {
        words[27 + k] = s.base[k] as i32;
        words[32 + 2 * k] = i32::try_from(s.grads[k].dx).expect("a gradient within 32 bits");
        words[33 + 2 * k] = i32::try_from(s.grads[k].dy).expect("a gradient within 32 bits");
    }
    // The vertices' coordinates' least and greatest, u's then v's, which hold a sample's coordinate at N x (§39.1).
    let (lo, hi) = (|f: fn(&Vertex) -> u8| v.iter().map(f).min().unwrap() as i32, |f: fn(&Vertex) -> u8| v.iter().map(f).max().unwrap() as i32);
    words[9] = lo(|q| q.u) | hi(|q| q.u) << 8 | lo(|q| q.v) << 16 | hi(|q| q.v) << 24;
    let texels = paint.texture.and_then(|t| {
        let range = |f: fn(&Vertex) -> u8| (v.iter().map(|q| f(q) as i32).min().unwrap_or(0) - 1, v.iter().map(|q| f(q) as i32).max().unwrap_or(0) + 1);
        texture_span(&t, range(|q| q.u), range(|q| q.v))
    });
    Some(Record { words, writes: Span::new(x_lo, x_hi, y_lo, y_hi), texels })
}

/// A rectangle under `paint`, its palette in `slot`; none when the area leaves nothing of it (`raster::rectangle`).
pub fn rectangle(s: Sprite, paint: &Paint, slot: u32) -> Option<Record> {
    let a = paint.area;
    let (x1, x2) = (s.x.max(a.x1), (s.x + s.w - 1).min(a.x2));
    let (y1, y2) = (s.y.max(a.y1), (s.y + s.h - 1).min(a.y2));
    if x1 > x2 || y1 > y2 {
        return None;
    }
    let mut words = [0i32; WORDS];
    painted(RECTANGLE, paint, slot, &mut words);
    if paint.texture.is_some() && !s.flip.0 {
        words[0] |= PAIRS as i32;
    }
    bounds(&mut words, (x1, x2), (y1, y2));
    words[8] = colour(s.rgb);
    let (su, sv) = (if s.flip.0 { -1 } else { 1 }, if s.flip.1 { -1 } else { 1 });
    let u0 = (if s.flip.0 { s.uv.0 | 1 } else { s.uv.0 }) as i32;
    words[10..16].copy_from_slice(&[s.x, s.y, u0, s.uv.1 as i32, su, sv]);
    // The texels the whole sprite reads, per axis, which a filtered sprite's are held within: all of them where the
    // coordinates wrap (§41).
    let reach = |start: i32, step: i32, n: i32| {
        let end = start + step * (n - 1);
        if (0..256).contains(&end) { (start.min(end), start.max(end)) } else { (0, 255) }
    };
    let (ru, rv) = (reach(u0, su, s.w), reach(s.uv.1 as i32, sv, s.h));
    words[9] = ru.0 | ru.1 << 8 | rv.0 << 16 | rv.1 << 24;
    let texels = paint.texture.and_then(|t| {
        let ends = |start: i32, step: i32, from: i32, to: i32| {
            let (a, b) = (start + step * from, start + step * to);
            (a.min(b) - 1, a.max(b))
        };
        texture_span(&t, ends(u0, su, x1 - s.x, x2 - s.x), ends(s.uv.1 as i32, sv, y1 - s.y, y2 - s.y))
    });
    Some(Record { words, writes: Span::new(x1, x2, y1, y2), texels })
}

/// A line from `a` to `b`, folded, under `paint`; none when it draws nothing (`raster::line`).
pub fn line(a: Vertex, b: Vertex, shaded: bool, paint: &Paint) -> Option<Record> {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    if dx.abs() > MAX_DX || dy.abs() > MAX_DY {
        return None;
    }
    let ar = paint.area;
    let (x_lo, x_hi) = (a.x.min(b.x).max(ar.x1), a.x.max(b.x).min(ar.x2));
    let (y_lo, y_hi) = (a.y.min(b.y).max(ar.y1), a.y.max(b.y).min(ar.y2));
    if x_lo > x_hi || y_lo > y_hi {
        return None;
    }
    let n = dx.abs().max(dy.abs());
    let mut words = [0i32; WORDS];
    painted(LINE, paint, 0, &mut words);
    bounds(&mut words, (x_lo, x_hi), (y_lo, y_hi));
    words[10..16].copy_from_slice(&[a.x, a.y, dx, dy, n, (dx.abs() >= dy.abs()) as i32]);
    for c in 0..3 {
        words[16 + c] = a.rgb[c];
        words[19 + c] = if shaded && n > 0 { (((b.rgb[c] - a.rgb[c]) as i64) << FRACTION) as i32 / n } else { 0 };
    }
    Some(Record { words, writes: Span::new(x_lo, x_hi, y_lo, y_hi), texels: None })
}

/// GP0(02h): `r` filled with the colour word's top five bits a channel, the lines of parity `skip` left alone; none for an
/// empty fill (`vram.rs`'s `fill`).
pub fn fill(r: Rect, colour_word: u32, skip: Option<i32>) -> Option<Record> {
    if r.w == 0 || r.h == 0 {
        return None;
    }
    let mut words = [0i32; WORDS];
    words[0] = (FILL | skip.map_or(0, |p| SKIP | (p as u32 & 1) << 19)) as i32;
    let c = (colour_word >> 3 & 0x1F) | (colour_word >> 11 & 0x1F) << 5 | (colour_word >> 19 & 0x1F) << 10;
    // The colour word's three lowest bits a channel, which true colour keeps (Diamond_Enhancements.md §40).
    let low = (colour_word & 7) | (colour_word >> 8 & 7) << 3 | (colour_word >> 16 & 7) << 6;
    words[10..16].copy_from_slice(&[r.x as i32, r.y as i32, r.w as i32, r.h as i32, c as i32, low as i32]);
    Some(Record { words, writes: Span::sized(r.x, r.y, r.w, r.h), texels: None })
}

/// Pixels `i0..i1` of a CPU-to-VRAM transfer into `r`, under `mask`, from the submission's data at pixel `offset`.
pub fn transfer(r: Rect, i0: u32, i1: u32, offset: u32, mask: Mask) -> Record {
    let mut words = [0i32; WORDS];
    words[0] = (TRANSFER | mask_flags(mask)) as i32;
    words[10..17].copy_from_slice(&[r.x as i32, r.y as i32, r.w as i32, r.h as i32, i0 as i32, i1 as i32, offset as i32]);
    let (j0, j1) = (i0 / r.w, (i1.max(i0 + 1) - 1) / r.w);
    Record { words, writes: Span::sized(r.x, r.y + j0, r.w, j1 - j0 + 1), texels: None }
}

/// What a precise triangle makes: its record, nothing when it covers no pixel, or none at all when a gradient does not fit
/// the record's 32 bits, so that its polygon is drawn on the integer path (Diamond_Enhancements.md §22.2).
#[derive(Clone, Copy, Debug)]
pub enum Made {
    Drawn(Record),
    Nothing,
    Integer,
}

/// The sub-pixel grid's bits, and the shift that brings a fine gradient's product back to the integer rule's 12 bits.
const FINE: i64 = 8;

/// The range a precise triangle's texture coordinates are held within, packed as u's least and greatest then v's: the
/// vertices' own, and with sub-pixel vertices short of the end the coordinate runs toward across the screen, since that
/// texel lies past the right or bottom edge the console's integer edges leave undrawn (Diamond_Enhancements.md §26.3).
/// At zero fraction nothing is held, so that the record is the integer rule's.
fn held(v: &[Vertex; 3], f: &[Fraction; 3], gu: [i32; 2], gv: [i32; 2]) -> u32 {
    if f.iter().all(|q| q.dx == 0 && q.dy == 0) {
        return 0xFF00_FF00;
    }
    let range = |c: [u8; 3], g: [i32; 2]| {
        let (mut lo, mut hi) = (*c.iter().min().unwrap() as u32, *c.iter().max().unwrap() as u32);
        if hi > lo && g[0] >= 0 && g[1] >= 0 {
            hi -= 1;
        } else if hi > lo && g[0] <= 0 && g[1] <= 0 {
            lo += 1;
        }
        lo | hi << 8
    };
    range(v.map(|q| q.u), gu) | range(v.map(|q| q.v), gv) << 16
}

/// A triangle with precise vertices, already folded, under `paint`: the integer vertices `v` the GPU decoded moved by
/// `f`'s offsets in 256ths of a pixel, its texture in perspective by `f`'s depths when they differ (§22.2, §22.3). Its
/// record carries each vertex's offset, so that the walk can fill it to the console's own triangle (§33): its bounds
/// are then the two triangles' together, and one of no precise area whose own has some is the integer path's.
pub fn precise_triangle(v: [Vertex; 3], f: [Fraction; 3], shaded: bool, textured: bool, paint: &Paint, slot: u32) -> Made {
    let (xs, ys) = (v.map(|q| q.x), v.map(|q| q.y));
    if xs.iter().max().unwrap() - xs.iter().min().unwrap() > MAX_DX || ys.iter().max().unwrap() - ys.iter().min().unwrap() > MAX_DY {
        return Made::Nothing;
    }
    let p: [(i64, i64); 3] = std::array::from_fn(|i| (((v[i].x as i64) << FINE) + f[i].dx as i64, ((v[i].y as i64) << FINE) + f[i].dy as i64));
    let area = (p[1].0 - p[0].0) * (p[2].1 - p[0].1) - (p[2].0 - p[0].0) * (p[1].1 - p[0].1);
    // The console's own triangle, which fills this one when there is some fraction, since at none the two are one.
    let console = raster::setup(&v, false, false);
    let own = console.filter(|_| f.iter().any(|q| q.dx != 0 || q.dy != 0));
    if area == 0 {
        return if own.is_some() { Made::Integer } else { Made::Nothing };
    }
    let ceil = |a: i64| -((-a) >> FINE);
    let (min, max) = (|c: fn(&(i64, i64)) -> i64| p.iter().map(c).min().unwrap(), |c: fn(&(i64, i64)) -> i64| p.iter().map(c).max().unwrap());
    let a = paint.area;
    let (mut x, mut y) = ((ceil(min(|q| q.0)) as i32, (max(|q| q.0) >> FINE) as i32), (ceil(min(|q| q.1)) as i32, ceil(max(|q| q.1)) as i32 - 1));
    if let Some(s) = &own {
        (x, y) = ((x.0.min(s.min.0), x.1.max(s.max.0)), (y.0.min(s.min.1), y.1.max(s.max.1 - 1)));
    }
    let ((x_lo, x_hi), (y_lo, y_hi)) = ((x.0.max(a.x1), x.1.min(a.x2)), (y.0.max(a.y1), y.1.min(a.y2)));
    if y_lo > y_hi || x_lo > x_hi {
        return Made::Nothing;
    }
    let mut words = [0i32; WORDS];
    let gradient = |k: usize| -> Option<(i32, i32)> {
        let at = v.map(|q| if k < 3 { q.rgb[k] as i64 } else if k == 3 { q.u as i64 } else { q.v as i64 });
        let (dx1, dy1, dx2, dy2) = (p[1].0 - p[0].0, p[1].1 - p[0].1, p[2].0 - p[0].0, p[2].1 - p[0].1);
        let (da1, da2) = (at[1] - at[0], at[2] - at[0]);
        let shift = FRACTION as i64 + FINE;
        Some((i32::try_from(((da1 * dy2 - da2 * dy1) << shift) / area).ok()?, i32::try_from(((dx1 * da2 - dx2 * da1) << shift) / area).ok()?))
    };
    for k in 0..5 {
        if if k < 3 { shaded } else { textured } {
            let Some((gx, gy)) = gradient(k) else { return Made::Integer };
            [words[32 + 2 * k], words[33 + 2 * k]] = [gx, gy];
        }
    }
    painted(PRECISE, paint, slot, &mut words);
    if shaded {
        words[0] |= SHADED as i32;
    }
    bounds(&mut words, (x_lo, x_hi), (y_lo, y_hi));
    words[8] = colour(v[0].rgb);
    let order = if area > 0 { [0, 1, 2] } else { [0, 2, 1] };
    // The console winds its triangle as this one or the other way; its edges are then these, reversed.
    let c = order.map(|k| (v[k].x as i64, v[k].y as i64));
    let turned = (c[1].0 - c[0].0) * (c[2].1 - c[0].1) - (c[2].0 - c[0].0) * (c[1].1 - c[0].1) < 0;
    let owns = |dx: i64, dy: i64| dy < 0 || (dy == 0 && dx > 0);
    for e in 0..3 {
        let (s, t) = (order[e], order[(e + 1) % 3]);
        let (dx, dy) = (p[t].0 - p[s].0, p[t].1 - p[s].1);
        words[10 + 5 * e..15 + 5 * e].copy_from_slice(&[p[s].0 as i32, p[s].1 as i32, dx as i32, dy as i32, f[s].dx & 0xFFFF | f[s].dy << 16]);
        words[0] |= (!owns(dx, dy) as i32) << (BOUNDS + e as u32);
        let (dx, dy) = ((v[t].x - v[s].x) as i64, (v[t].y - v[s].y) as i64);
        words[0] |= (!(if turned { owns(-dx, -dy) } else { owns(dx, dy) }) as i32) << (OWN_BOUNDS + e as u32);
        let o = order[(e + 2) % 3];
        words[42 + e] = v[o].u as i32 | (v[o].v as i32) << 8;
    }
    if own.is_some() {
        words[0] |= (FILLED | if turned { TURNED } else { 0 }) as i32;
    }
    // The integer rule's base vertex; a triangle of no integer area, which the console does not draw, takes its first.
    let b = console.map_or(0, |s| s.b);
    [words[25], words[26]] = [p[b].0 as i32, p[b].1 as i32];
    let base = [v[b].rgb[0], v[b].rgb[1], v[b].rgb[2], v[b].u as i32, v[b].v as i32];
    for k in 0..5 {
        words[27 + k] = base[k] << FRACTION;
    }
    words[9] = held(&v, &f, [words[38], words[39]], [words[40], words[41]]) as i32;
    let z = f.map(|q| q.z.max(1));
    if textured && (z[0] != z[1] || z[1] != z[2]) {
        let near = *z.iter().min().unwrap() as f32;
        words[0] |= PERSPECTIVE as i32;
        for e in 0..3 {
            words[45 + e] = (near / z[order[(e + 2) % 3]] as f32).to_bits() as i32;
        }
    }
    let texels = paint.texture.and_then(|t| {
        let range = |f: fn(&Vertex) -> u8| (v.iter().map(|q| f(q) as i32).min().unwrap_or(0) - 1, v.iter().map(|q| f(q) as i32).max().unwrap_or(0) + 1);
        texture_span(&t, range(|q| q.u), range(|q| q.v))
    });
    Made::Drawn(Record { words, writes: Span::new(x_lo, x_hi, y_lo, y_hi), texels })
}
