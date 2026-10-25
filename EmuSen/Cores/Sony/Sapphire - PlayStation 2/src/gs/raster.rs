//! Which pixels a primitive covers, and the colour and Z each takes (GS User's Manual §2.4.4, §3.2.8, §3.2.9). Pixels
//! are centred where window coordinates are whole (§2.4.4); a vertex's coordinates are in sixteenths. The rules, each
//! exact in integers:
//!
//! - a **point** draws the pixel nearest its vertex, a half rounding up;
//! - a **line** draws the pixels whose diamond (§3.2.9's figure 3-5: |dx| + |dy| < 1/2 about the centre) it passes
//!   through, the start's drawn and the end's not; a crossing of the major axis's column exactly between two diamonds
//!   takes the one further down (x-major) or right (y-major), as a half rounds up;
//! - a **triangle** draws the pixels whose centre is inside it, or on a left or top side but not a right or bottom one;
//! - a **sprite** draws the centres in [x0, x1) × [y0, y1), its top and left sides in and its bottom and right out.
//!
//! Colour is the last vertex's when flat (§3.2.8) and, when Gouraud, the vertices' colours interpolated linearly at the
//! pixel's centre (along the major axis for a line, clamped to its ends) and rounded down; Z is interpolated so whatever
//! the shading, and a sprite takes its second vertex's (§3.2.8). The documents give the rule but not the DDA's fixed
//! point; the interpolation is therefore exact, a reading recorded as Sapphire_Disputes.md D-160. The loops of sprites
//! and triangles run over the primitive's bounding box within the scissoring area, whose test (`pixel`) the pixels
//! outside it would fail first. Sapphire_Native.md §27.

use super::draw::{Environment, Vertex};
use super::local::Local;
use super::pixel;

/// The whole pixels from `lo` to `hi` inclusive whose centres lie in [a, b) sixteenths, clipped: ceil(a) to ceil(b) - 1.
fn span(a: i32, b: i32, lo: i32, hi: i32) -> (i32, i32) {
    ((a + 15).div_euclid(16).max(lo), ((b + 15).div_euclid(16) - 1).min(hi))
}

/// The nearest whole pixel to `v` sixteenths, a half rounding up.
const fn nearest(v: i32) -> i32 {
    (v + 8).div_euclid(16)
}

pub fn point(env: &Environment, local: &mut Local, v: &Vertex) {
    pixel::draw(env, local, nearest(v.x), nearest(v.y), v.z, v.rgba());
}

pub fn sprite(env: &Environment, local: &mut Local, a: &Vertex, b: &Vertex) {
    let (sx0, sx1, sy0, sy1) = env.scissor;
    let (x0, x1) = span(a.x.min(b.x), a.x.max(b.x), sx0, sx1);
    let (y0, y1) = span(a.y.min(b.y), a.y.max(b.y), sy0, sy1);
    let c = b.rgba();
    for y in y0..=y1 {
        for x in x0..=x1 {
            pixel::draw(env, local, x, y, b.z, c);
        }
    }
}

/// Whether the point (`u`, `v`), in sixteenths along the line's major and minor axes, lies in the diamond of the pixel
/// nearest it: inside, or at the one corner a crossing between two diamonds is given to.
fn in_diamond(u: i32, v: i32) -> bool {
    let (cu, cv) = (16 * nearest(u), 16 * nearest(v));
    (u - cu).abs() + (v - cv).abs() < 8 || (u == cu && v == cv - 8)
}

pub fn line(env: &Environment, local: &mut Local, a: &Vertex, b: &Vertex) {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    if dx == 0 && dy == 0 {
        return;
    }
    let xmajor = dx.abs() >= dy.abs();
    let (au, av, bu, bv) = if xmajor { (a.x, a.y, b.x, b.y) } else { (a.y, a.x, b.y, b.x) };
    let s = (bu - au).signum();
    // Along the major axis with a positive denominator: t = (16c - au) / du, the minor axis v = av + t (bv - av).
    let (du, dv) = (i64::from((bu - au) * s), i64::from((bv - av) * s));
    let first = if in_diamond(au, av) {
        nearest(au)
    } else if s > 0 {
        (au + 15).div_euclid(16)
    } else {
        au.div_euclid(16)
    };
    let last = if in_diamond(bu, bv) {
        nearest(bu) - s
    } else if s > 0 {
        bu.div_euclid(16)
    } else {
        (bu + 15).div_euclid(16)
    };
    let (ca, cb) = (a.rgba(), b.rgba());
    let gouraud = env.attributes.gouraud();
    let mut c = first;
    while (last - c) * s >= 0 {
        let num = i64::from(16 * c - au) * i64::from(s);
        let r = (i64::from(av) * du + i64::from(16 * c - au) * dv + 8 * du).div_euclid(16 * du) as i32;
        let t = num.clamp(0, du);
        let z = ((i64::from(a.z) * (du - t) + i64::from(b.z) * t) / du) as u32;
        let rgba = if gouraud { std::array::from_fn(|i| ((i64::from(ca[i]) * (du - t) + i64::from(cb[i]) * t) / du) as u32) } else { cb };
        let (x, y) = if xmajor { (c, r) } else { (r, c) };
        pixel::draw(env, local, x, y, z, rgba);
        c += s;
    }
}

/// The edge function of p→q at (x, y): positive on one side, zero on the line, in sixteenths squared.
const fn edge(p: &Vertex, q: &Vertex, x: i64, y: i64) -> i64 {
    (q.x as i64 - p.x as i64) * (y - p.y as i64) - (q.y as i64 - p.y as i64) * (x - p.x as i64)
}

/// Whether a centre on edge p→q is drawn, the inside on its positive side: the inward normal (-(qy - py), qx - px)
/// points right on a left side, or down on a top side.
const fn top_left(p: &Vertex, q: &Vertex) -> i64 {
    let (dx, dy) = (q.x - p.x, q.y - p.y);
    (dy < 0 || (dy == 0 && dx > 0)) as i64
}

pub fn triangle(env: &Environment, local: &mut Local, [a, b, c]: [&Vertex; 3]) {
    let last = c;
    let area = edge(a, b, i64::from(c.x), i64::from(c.y));
    if area == 0 {
        return;
    }
    let (b, c) = if area > 0 { (b, c) } else { (c, b) };
    let area = area.abs();
    let (sx0, sx1, sy0, sy1) = env.scissor;
    let (x0, x1) = span(a.x.min(b.x).min(c.x), a.x.max(b.x).max(c.x) + 1, sx0, sx1);
    let (y0, y1) = span(a.y.min(b.y).min(c.y), a.y.max(b.y).max(c.y) + 1, sy0, sy1);
    let bias = [top_left(b, c), top_left(c, a), top_left(a, b)];
    let (ca, cb, cc) = (a.rgba(), b.rgba(), c.rgba());
    let gouraud = env.attributes.gouraud();
    for y in y0..=y1 {
        for x in x0..=x1 {
            let (px, py) = (16 * i64::from(x), 16 * i64::from(y));
            let w = [edge(b, c, px, py), edge(c, a, px, py), edge(a, b, px, py)];
            if (0..3).any(|i| w[i] + bias[i] <= 0) {
                continue;
            }
            let z = ((i128::from(a.z) * i128::from(w[0]) + i128::from(b.z) * i128::from(w[1]) + i128::from(c.z) * i128::from(w[2])) / i128::from(area)) as u32;
            let rgba = if gouraud { std::array::from_fn(|i| ((i64::from(ca[i]) * w[0] + i64::from(cb[i]) * w[1] + i64::from(cc[i]) * w[2]) / area) as u32) } else { last.rgba() };
            pixel::draw(env, local, x, y, z, rgba);
        }
    }
}
