//! Which pixels a primitive covers and the colour and texture coordinate at each (Diamond_GPU.md §6): triangles by
//! their edge functions sampled at whole pixels with the top-left rule, their attributes from the leftmost vertex by
//! gradients in 12-bit fixed point, their rows outward from the middle vertex's; rectangles with their texture stepped, or flipped, pixel by pixel; lines stepped along the
//! major axis with both ends drawn. Each was fitted to ps1-tests' VRAM pictures.

use super::draw::Painter;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Vertex {
    pub x: i32,
    pub y: i32,
    pub rgb: [i32; 3],
    pub u: u8,
    pub v: u8,
}

/// The fractional bits of the interpolators, and the half added before the integer part is taken.
pub const FRACTION: u32 = 12;
const HALF: i64 = 1 << (FRACTION - 1);

/// The largest spans a primitive may have between its vertices (psx-spx "Vertex").
pub const MAX_DX: i32 = 1023;
pub const MAX_DY: i32 = 511;

/// The GPU's coordinates are 11 bits: a primitive reaching left of (or above) -1,024 is drawn 2,048 further on (Diamond_Disputes.md D-47).
const FOLD: i32 = 2048;

/// `v` moved 2,048 on in x or y where it reaches past -1,024 there, else as it is.
#[inline]
pub fn fold<const N: usize>(v: [Vertex; N]) -> [Vertex; N] {
    if v.iter().any(|q| q.x < -1024 || q.y < -1024) { folded(v) } else { v }
}

#[cold]
#[inline(never)]
fn folded<const N: usize>(v: [Vertex; N]) -> [Vertex; N] {
    let (fx, fy) = (if v.iter().any(|q| q.x < -1024) { FOLD } else { 0 }, if v.iter().any(|q| q.y < -1024) { FOLD } else { 0 });
    v.map(|q| Vertex { x: q.x + fx, y: q.y + fy, ..q })
}

fn trunc_div(a: i64, b: i64) -> i64 {
    a / b
}

pub fn floor_div(a: i64, b: i64) -> i64 {
    a.div_euclid(b) - if b < 0 && a.rem_euclid(b) != 0 { 1 } else { 0 }
}

fn ceil_div(a: i64, b: i64) -> i64 {
    -floor_div(-a, b)
}

/// A gradient in x and y of one attribute, in 12-bit fixed point, truncated toward zero.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Gradient {
    pub dx: i64,
    pub dy: i64,
}

fn gradient(v: &[Vertex; 3], a: [i64; 3], area: i64) -> Gradient {
    let (x0, y0) = (v[0].x as i64, v[0].y as i64);
    let (dx1, dy1, dx2, dy2) = (v[1].x as i64 - x0, v[1].y as i64 - y0, v[2].x as i64 - x0, v[2].y as i64 - y0);
    let (da1, da2) = (a[1] - a[0], a[2] - a[0]);
    Gradient { dx: trunc_div((da1 * dy2 - da2 * dy1) << FRACTION, area), dy: trunc_div((dx1 * da2 - dx2 * da1) << FRACTION, area) }
}

/// The vertex the attributes start from: the leftmost; of two in the leftmost column, the one that follows the other
/// in the command's order, 0 following 2 (the referees, Diamond_Disputes.md D-7).
pub fn base_vertex(v: &[Vertex; 3]) -> usize {
    let left = v.iter().map(|q| q.x).min().unwrap_or(0);
    let tied = |i: usize| v[i].x == left;
    (0..3).find(|&i| tied(i) && tied((i + 2) % 3)).or_else(|| (0..3).find(|&i| tied(i))).unwrap_or(0)
}

/// Whether an edge from `a` to `b` of a triangle wound with positive area owns the pixels exactly on it: a top or a left
/// edge does.
fn owns(a: (i64, i64), b: (i64, i64)) -> bool {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    dy < 0 || (dy == 0 && dx > 0)
}

/// What a triangle's pixels are found and shaded by: its bounds, its three edges wound with positive area, each from a
/// vertex to the next with whether it owns the pixels on it, and its five attributes' gradients and values at the base
/// vertex `b`, R, G, B, U and V, each in 12-bit fixed point (Diamond_GPU.md §6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Setup {
    pub min: (i32, i32),
    pub max: (i32, i32),
    pub edges: [((i64, i64), (i64, i64), bool); 3],
    pub grads: [Gradient; 5],
    pub b: usize,
    pub base: [i64; 5],
}

/// The triangle `v`'s setup, or none when it draws nothing whatever the area: too large a span, or no area. `triangle`
/// makes the same values inline, which keeps its code the base's (Diamond_Enhancements.md §18.2);
/// `setup_describes_the_pixels_triangle_draws` holds the two equal.
pub fn setup(v: &[Vertex; 3], shaded: bool, textured: bool) -> Option<Setup> {
    let xs = v.map(|q| q.x);
    let ys = v.map(|q| q.y);
    let (minx, maxx) = (*xs.iter().min().unwrap_or(&0), *xs.iter().max().unwrap_or(&0));
    let (miny, maxy) = (*ys.iter().min().unwrap_or(&0), *ys.iter().max().unwrap_or(&0));
    if maxx - minx > MAX_DX || maxy - miny > MAX_DY {
        return None;
    }
    let pt = |i: usize| (v[i].x as i64, v[i].y as i64);
    let area = (pt(1).0 - pt(0).0) * (pt(2).1 - pt(0).1) - (pt(2).0 - pt(0).0) * (pt(1).1 - pt(0).1);
    if area == 0 {
        return None;
    }
    let order = if area > 0 { [0, 1, 2] } else { [0, 2, 1] };
    let edges = [(order[0], order[1]), (order[1], order[2]), (order[2], order[0])].map(|(a, b)| (pt(a), pt(b), owns(pt(a), pt(b))));
    let grads: [Gradient; 5] = [0, 1, 2, 3, 4].map(|k| {
        let used = if k < 3 { shaded } else { textured };
        if !used {
            return Gradient::default();
        }
        let a = v.map(|q| if k < 3 { q.rgb[k] as i64 } else if k == 3 { q.u as i64 } else { q.v as i64 });
        gradient(v, a, area)
    });
    let b = base_vertex(v);
    let base: [i64; 5] = [v[b].rgb[0] as i64, v[b].rgb[1] as i64, v[b].rgb[2] as i64, v[b].u as i64, v[b].v as i64].map(|a| a << FRACTION);
    Some(Setup { min: (minx, miny), max: (maxx, maxy), edges, grads, b, base })
}

/// The triangle `v`, already folded (`fold`), its attributes from its base vertex: colour when `shaded`, the texture coordinate when `textured`;
/// its rows from the middle vertex's down, then those above it upward, each left to right (Diamond_Disputes.md D-41).
pub fn triangle(p: &mut Painter, v: [Vertex; 3], shaded: bool, textured: bool) {
    let xs = v.map(|q| q.x);
    let ys = v.map(|q| q.y);
    let (minx, maxx) = (*xs.iter().min().unwrap_or(&0), *xs.iter().max().unwrap_or(&0));
    let (miny, maxy) = (*ys.iter().min().unwrap_or(&0), *ys.iter().max().unwrap_or(&0));
    if maxx - minx > MAX_DX || maxy - miny > MAX_DY {
        return;
    }
    let pt = |i: usize| (v[i].x as i64, v[i].y as i64);
    let area = (pt(1).0 - pt(0).0) * (pt(2).1 - pt(0).1) - (pt(2).0 - pt(0).0) * (pt(1).1 - pt(0).1);
    if area == 0 {
        return;
    }
    let order = if area > 0 { [0, 1, 2] } else { [0, 2, 1] };
    let edges = [(order[0], order[1]), (order[1], order[2]), (order[2], order[0])].map(|(a, b)| (pt(a), pt(b), owns(pt(a), pt(b))));
    let grads: [Gradient; 5] = [0, 1, 2, 3, 4].map(|k| {
        let used = if k < 3 { shaded } else { textured };
        if !used {
            return Gradient::default();
        }
        let a = v.map(|q| if k < 3 { q.rgb[k] as i64 } else if k == 3 { q.u as i64 } else { q.v as i64 });
        gradient(&v, a, area)
    });
    let b = base_vertex(&v);
    let base: [i64; 5] = [v[b].rgb[0] as i64, v[b].rgb[1] as i64, v[b].rgb[2] as i64, v[b].u as i64, v[b].v as i64].map(|a| a << FRACTION);
    let area_rect = p.paint.area;
    let y_from = miny.max(area_rect.y1);
    let y_to = (maxy - 1).min(area_rect.y2);
    if y_from > y_to {
        return;
    }
    let split = (ys[0] + ys[1] + ys[2] - miny - maxy).clamp(y_from, y_to + 1);
    for y in (split..=y_to).chain((y_from..split).rev()) {
        let yy = y as i64;
        let (mut lo, mut hi) = (minx as i64, maxx as i64);
        for &((ax, ay), (bx, by), own) in &edges {
            let (dx, dy) = (bx - ax, by - ay);
            let k = dx * (yy - ay) + dy * ax;
            let t = if own { 0 } else { 1 };
            if dy > 0 {
                hi = hi.min(floor_div(k - t, dy));
            } else if dy < 0 {
                lo = lo.max(ceil_div(t - k, -dy));
            } else if k < t {
                hi = lo - 1;
            }
        }
        if lo > hi {
            continue;
        }
        p.work.rows += 1;
        let lo = lo.max(area_rect.x1 as i64);
        let hi = hi.min(area_rect.x2 as i64);
        if lo > hi {
            continue;
        }
        if !p.visible_row(y) {
            continue;
        }
        let at = |k: usize, x: i64| base[k] + grads[k].dx * (x - v[b].x as i64) + grads[k].dy * (yy - v[b].y as i64);
        let mut a = [0, 1, 2, 3, 4].map(|k| at(k, lo));
        p.row(y, lo as i32, hi as i32, |_| {
            let rgb = if shaded { [0, 1, 2].map(|k| ((a[k] + HALF) >> FRACTION) as i32) } else { v[0].rgb };
            let uv = if textured { (((a[3] + HALF) >> FRACTION) as u8, ((a[4] + HALF) >> FRACTION) as u8) } else { (0, 0) };
            for k in 0..5 {
                a[k] += grads[k].dx;
            }
            (rgb, uv)
        });
    }
}

/// A rectangle's corner, size, colour, texture coordinate and flips.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sprite {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub rgb: [i32; 3],
    pub uv: (u8, u8),
    pub flip: (bool, bool),
}

/// A rectangle; its texture coordinate steps by one a pixel, backwards along a flipped axis, an x-flipped one from
/// u with its low bit set (ps1-tests' `texture-flip` on a console and the referee, Diamond_Disputes.md D-8); unflipped,
/// a pixel pair's texels come from one texture cache entry, the left one's (amidog's expectations, D-23).
pub fn rectangle(p: &mut Painter, s: Sprite) {
    let Sprite { x, y, w, h, rgb, uv, flip } = s;
    let area = p.paint.area;
    let (x1, x2) = (x.max(area.x1), (x + w - 1).min(area.x2));
    let (y1, y2) = (y.max(area.y1), (y + h - 1).min(area.y2));
    if x1 > x2 || y1 > y2 {
        return;
    }
    let step = |f: bool| if f { -1i32 } else { 1 };
    let (su, sv) = (step(flip.0), step(flip.1));
    let u0 = (if flip.0 { uv.0 | 1 } else { uv.0 }) as i32;
    let pairs = p.paint.texture.filter(|_| !flip.0);
    for yy in y1..=y2 {
        p.work.rows += 1;
        if !p.visible_row(yy) {
            continue;
        }
        let v = (uv.1 as i32 + sv * (yy - y)) as u8;
        p.row(yy, x1, x2, |xx| {
            let mut u = (u0 + su * (xx - x)) as u8;
            if let Some(t) = pairs.filter(|_| xx & 1 == 1 && xx > x1) {
                let left = u.wrapping_sub(1);
                let entry = |u: u8| t.window(u, 0).0 >> (4 - t.depth);
                if entry(u) != entry(left) {
                    u = left;
                }
            }
            (rgb, (u, v))
        });
    }
}

/// A line from `a` to `b`, both ends drawn: the minor coordinate is the nearest to the exact line, a half rounding
/// toward the right end on an x-major line and toward less x on a y-major one (Diamond_Disputes.md D-7, D-23); the
/// colour by its 12-bit gradient.
pub fn line(p: &mut Painter, a: Vertex, b: Vertex, shaded: bool) {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    if dx.abs() > MAX_DX || dy.abs() > MAX_DY {
        return;
    }
    let n = dx.abs().max(dy.abs()) as i64;
    let x_major = dx.abs() >= dy.abs();
    let grads: [i64; 3] = [0, 1, 2].map(|k| if shaded && n > 0 { trunc_div(((b.rgb[k] - a.rgb[k]) as i64) << FRACTION, n) } else { 0 });
    let area = p.paint.area;
    let mut last_row = None;
    for k in 0..=n {
        let (x, y) = if n == 0 {
            (a.x, a.y)
        } else if x_major {
            (a.x + (dx.signum() as i64 * k) as i32, a.y + dy.signum() * floor_div(2 * k * dy.abs() as i64 + if dx < 0 { n - 1 } else { n }, 2 * n) as i32)
        } else {
            (a.x + floor_div(2 * k * dx as i64 + n - 1, 2 * n) as i32, a.y + (dy.signum() as i64 * k) as i32)
        };
        if x < area.x1 || x > area.x2 || y < area.y1 || y > area.y2 {
            continue;
        }
        if last_row != Some(y) {
            p.work.rows += 1;
            last_row = Some(y);
        }
        if !p.visible_row(y) {
            continue;
        }
        let rgb = [0, 1, 2].map(|c| ((((a.rgb[c] as i64) << FRACTION) + grads[c] * k + HALF) >> FRACTION) as i32);
        p.pixel(x, y, rgb, (0, 0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_divisions_round_as_named() {
        assert_eq!((floor_div(-7, 2), floor_div(7, 2), floor_div(7, -2), floor_div(-7, -2)), (-4, 3, -4, 3));
        assert_eq!((ceil_div(-7, 2), ceil_div(7, 2), ceil_div(7, -2)), (-3, 4, -3));
        assert_eq!((trunc_div(-7, 2), trunc_div(7, 2)), (-3, 3));
    }

    /// Random triangles drawn by `triangle`, shaded and then textured raw from a page whose texel names its coordinate,
    /// are pixel for pixel what `setup` describes, each pixel inside when every edge function reaches its owner's bound
    /// and its attributes the base's plus the gradients times the distances (the renderer's rule, Diamond_Enhancements.md
    /// §16.3, §18.2).
    #[test]
    fn setup_describes_the_pixels_triangle_draws() {
        use super::super::draw::{Area, Paint};
        use super::super::texture::{ClutCache, Texture, TextureCache};
        use super::super::vram::{Mask, Vram};
        let mut seed = 0x2545_F491_4F6C_DD1Du64;
        let mut next = |n: u64| {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((seed >> 33) % n) as i32
        };
        let mut vram = Vram::default();
        for v in 0..256 {
            for u in 0..256 {
                vram.set(512 + u, v, (u | (v & 0x7F) << 8) as u16);
            }
        }
        let page = Texture::new(8 | 2 << 7, 0, 0, false);
        let (mut cache, clut) = (TextureCache::default(), ClutCache::default());
        let mut drawn = 0;
        for trial in 0..600 {
            let textured = trial % 2 == 1;
            let v: [Vertex; 3] = std::array::from_fn(|_| Vertex { x: next(200), y: 260 + next(200), rgb: [next(256), next(256), next(256)], u: next(256) as u8, v: next(128) as u8 });
            for y in 256..512 {
                for x in 0..256 {
                    vram.set(x, y, 0);
                }
            }
            let paint = Paint { blend: None, texture: textured.then_some(page), raw: textured, dither: false, mask: Mask::default(), area: Area { x1: 0, y1: 0, x2: 1023, y2: 511 }, skip_parity: None, crop_modulation: false };
            let mut p = Painter { vram: &mut vram, cache: &mut cache, clut: &clut, paint, work: Default::default() };
            triangle(&mut p, v, !textured, textured);
            let s = setup(&v, !textured, textured);
            for y in 256..512i32 {
                for x in 0..256i32 {
                    let at = |k: usize| ((s.unwrap().base[k] + s.unwrap().grads[k].dx * (x - v[s.unwrap().b].x) as i64 + s.unwrap().grads[k].dy * (y - v[s.unwrap().b].y) as i64 + HALF) >> FRACTION) as i32;
                    let inside = s.is_some_and(|s| y >= s.min.1 && y < s.max.1 && s.edges.iter().all(|&((ax, ay), (bx, by), own)| (bx - ax) * (y as i64 - ay) - (by - ay) * (x as i64 - ax) >= if own { 0 } else { 1 }));
                    let want = if !inside {
                        0
                    } else if textured {
                        vram.get(512 + (at(3) as u8) as u32, (at(4) as u8) as u32)
                    } else {
                        let c = [0, 1, 2].map(|k| (at(k).clamp(0, 255) >> 3) as u16);
                        c[0] | c[1] << 5 | c[2] << 10
                    };
                    assert_eq!(vram.get(x as u32, y as u32), want, "trial {trial} at ({x}, {y}): {v:?}");
                    drawn += (want != 0) as u32;
                }
            }
        }
        assert!(drawn > 1_000_000, "{drawn} pixels drawn");
    }

    #[test]
    fn the_base_is_the_leftmost_vertex_and_of_two_the_follower() {
        let at = |pts: [(i32, i32); 3]| base_vertex(&pts.map(|(x, y)| Vertex { x, y, ..Vertex::default() }));
        assert_eq!(at([(40, 9), (2, 1), (13, 30)]), 1);
        assert_eq!((at([(2, 1), (40, 9), (2, 30)]), at([(2, 1), (2, 30), (40, 9)]), at([(40, 9), (2, 1), (2, 30)])), (0, 1, 2));
    }
}
