//! Which pixels a primitive covers, and the colour and Z each takes (GS User's Manual §2.4.4, §3.2.8, §3.2.9). Pixels
//! are centred where window coordinates are whole (§2.4.4); a vertex's coordinates are in sixteenths. The rules, each
//! exact in integers:
//!
//! - a **point** draws the pixel nearest its vertex, a half rounding up;
//! - a **line** draws the pixels whose diamond (§3.2.9's figure 3-5: |dx| + |dy| < 1/2 about the centre) it passes
//!   through, the start's drawn and the end's not: one pixel in each column of its major axis it crosses, the one whose
//!   diamond holds the crossing (a crossing exactly between two diamonds taking the one further down for an x-major
//!   line, further right for a y-major one, as a half rounds up), from the start's pixel when the start lies in a
//!   diamond to the pixel before the end's when the end does, an end on a diamond's edge lying in none;
//! - a **triangle** draws the pixels whose centre is inside it, or on a left or top side but not a right or bottom one;
//! - a **sprite** draws the centres in [x0, x1) × [y0, y1), its top and left sides in and its bottom and right out.
//!
//! Colour is the last vertex's when flat (§3.2.8) and, when Gouraud, the vertices' colours interpolated linearly at the
//! pixel's centre and rounded down; along a line that is by the major axis, continued past an end whose pixel's centre
//! lies beyond it and held to the channel's range (D-165). Z is interpolated so whatever the shading, and a sprite takes
//! its second vertex's (§3.2.8). The documents give the rule but not the DDA's fixed point; the interpolation is
//! therefore exact, a reading recorded as Sapphire_Disputes.md D-160 and D-164. The loops of sprites and triangles run
//! over the primitive's bounding box within the scissoring area, whose test (`pixel`) the pixels outside it would fail
//! first. Sapphire_Native.md §27.
//!
//! Texture coordinates and the fog coefficient (S18, §31) are interpolated by the same weights whatever the shading: a
//! triangle's barycentric ones, a line's along its major axis (continued past an end, the fog held to 0 to 255), a
//! sprite's along x for S and U and along y for T and V with the second vertex's Q and fog, a point's its vertex's.

use super::draw::{Environment, Vertex};
use super::local::Local;
use super::pixel::{self, Shade};
use super::texture::{Clut, Setup};

/// The whole pixels from `lo` to `hi` inclusive whose centres lie in [a, b) sixteenths, clipped: ceil(a) to ceil(b) - 1.
fn span(a: i32, b: i32, lo: i32, hi: i32) -> (i32, i32) {
    ((a + 15).div_euclid(16).max(lo), ((b + 15).div_euclid(16) - 1).min(hi))
}

/// The nearest whole pixel to `v` sixteenths, a half rounding up.
const fn nearest(v: i32) -> i32 {
    (v + 8).div_euclid(16)
}

/// The primitive's texture setup, when it is textured; `q` the Q each vertex contributes.
fn setup(env: &Environment, v: [&Vertex; 3], q: [u32; 3], last: &Vertex) -> Option<Setup> {
    env.texture.as_ref().map(|t| Setup::new(t, v, q, last))
}

fn q(v: &Vertex) -> u32 {
    (v.rgbaq >> 32) as u32
}

/// The shade at weights `wu` (and `wv` along V) with fog coefficient `f`.
fn shade(tex: &Option<Setup>, wu: [i64; 3], wv: [i64; 3], f: u32) -> Shade {
    let (u, v, lod) = tex.as_ref().map_or((0, 0, 0), |t| t.at(wu, wv));
    Shade { u, v, lod, f }
}

pub fn point(env: &Environment, local: &mut Local, clut: &Clut, v: &Vertex) {
    let tex = setup(env, [v; 3], [q(v); 3], v);
    pixel::draw(env, local, clut, nearest(v.x), nearest(v.y), v.z, v.rgba(), pixel::FULL, || shade(&tex, [1, 0, 0], [1, 0, 0], u32::from(v.f)));
}

pub fn sprite(env: &Environment, local: &mut Local, clut: &Clut, a: &Vertex, b: &Vertex) {
    let (sx0, sx1, sy0, sy1) = env.scissor;
    let (x0, x1) = span(a.x.min(b.x), a.x.max(b.x), sx0, sx1);
    let (y0, y1) = span(a.y.min(b.y), a.y.max(b.y), sy0, sy1);
    let c = b.rgba();
    let tex = setup(env, [a, b, b], [q(b); 3], b);
    // Weights of a and b at a centre along one axis, their total made positive whichever corner comes first.
    let along = |p: i32, pa: i32, pb: i32| {
        let s = (pb - pa).signum() as i64;
        [i64::from(pb - 16 * p) * s, i64::from(16 * p - pa) * s, 0]
    };
    for y in y0..=y1 {
        for x in x0..=x1 {
            pixel::draw(env, local, clut, x, y, b.z, c, pixel::FULL, || shade(&tex, along(x, a.x, b.x), along(y, a.y, b.y), u32::from(b.f)));
        }
    }
}

/// Whether an end of a line, (`u`, `v`) in sixteenths along its major and minor axes, lies in the diamond of the pixel
/// nearest it: inside, or at the corner a crossing between two diamonds is given to; on an edge it lies in none
/// (Sapphire_Disputes.md D-161).
fn in_diamond(u: i32, v: i32) -> bool {
    let (cu, cv) = (16 * nearest(u), 16 * nearest(v));
    (u - cu).abs() + (v - cv).abs() < 8 || (u == cu && v == cv - 8)
}

pub fn line(env: &Environment, local: &mut Local, clut: &Clut, a: &Vertex, b: &Vertex) {
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
    let tex = setup(env, [a, b, b], [q(a), q(b), q(b)], b);
    let antialias = env.blend.antialias;
    let mut c = first;
    while (last - c) * s >= 0 {
        let t = i64::from(16 * c - au) * i64::from(s);
        let r = (i64::from(av) * du + i64::from(16 * c - au) * dv + 8 * du).div_euclid(16 * du) as i32;
        let lerp = |p: u32, q: u32| (i64::from(p) * (du - t) + i64::from(q) * t).div_euclid(du);
        let z = lerp(a.z, b.z).clamp(0, i64::from(u32::MAX)) as u32;
        let rgba = if gouraud { std::array::from_fn(|i| lerp(ca[i], cb[i]).clamp(0, 255) as u32) } else { cb };
        let (x, y) = if xmajor { (c, r) } else { (r, c) };
        let w = [du - t, t, 0];
        let cov = if antialias { line_coverage(i64::from(av) * du + i64::from(16 * c - au) * dv - 16 * i64::from(r) * du, du) } else { pixel::FULL };
        pixel::draw(env, local, clut, x, y, z, rgba, cov, || shade(&tex, w, w, lerp(u32::from(a.f), u32::from(b.f)).clamp(0, 255) as u32));
        c += s;
    }
}

/// A line pixel's coverage under AA1 (Sapphire_Disputes.md D-406): 80h × (1 - |e|), e the line's distance from the pixel's
/// centre along the minor axis at the centre's column, `e_du` being 16 e × du; held below 80h, as §3.6.2 says a line's
/// always is.
fn line_coverage(e_du: i64, du: i64) -> u32 {
    ((16 * du - e_du.abs()) * 8).div_euclid(du).clamp(0, 0x7F) as u32
}

/// A triangle pixel's coverage under AA1 (D-406): the points of a 16 × 16 grid at the centres of the pixel's sixteenths
/// that the triangle holds, by its drawing rule, halved, so 80h for the whole pixel; the counter-clockwise edges b→c,
/// c→a, a→b with `bias` their top-left ties.
fn coverage([a, b, c]: [&Vertex; 3], bias: [i64; 3], x: i32, y: i32) -> u32 {
    // In doubled sixteenths, so that the grid's points are whole: 2 (16x - 8 + k) + 1 for k from 0 to 15.
    let edges = [(b, c), (c, a), (a, b)];
    let at = |i: usize, px: i64, py: i64| {
        let (p, q) = edges[i];
        let (qx, qy, ox, oy) = (i64::from(q.x - p.x), i64::from(q.y - p.y), i64::from(p.x), i64::from(p.y));
        qx * (py - 2 * oy) - qy * (px - 2 * ox)
    };
    let (x0, y0) = (2 * (16 * i64::from(x) - 8), 2 * (16 * i64::from(y) - 8));
    let corners = [(x0, y0), (x0 + 32, y0), (x0, y0 + 32), (x0 + 32, y0 + 32)];
    if corners.iter().all(|&(px, py)| (0..3).all(|i| at(i, px, py) > 0)) {
        return pixel::FULL;
    }
    let mut n = 0;
    for j in 0..16 {
        for k in 0..16 {
            let (px, py) = (x0 + 2 * k + 1, y0 + 2 * j + 1);
            n += u32::from((0..3).all(|i| at(i, px, py) + bias[i] > 0));
        }
    }
    n / 2
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

pub fn triangle(env: &Environment, local: &mut Local, clut: &Clut, [a, b, c]: [&Vertex; 3]) {
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
    let tex = setup(env, [a, b, c], [q(a), q(b), q(c)], last);
    let fog = [a.f, b.f, c.f].map(i64::from);
    for y in y0..=y1 {
        for x in x0..=x1 {
            let (px, py) = (16 * i64::from(x), 16 * i64::from(y));
            let w = [edge(b, c, px, py), edge(c, a, px, py), edge(a, b, px, py)];
            if (0..3).any(|i| w[i] + bias[i] <= 0) {
                continue;
            }
            let z = ((i128::from(a.z) * i128::from(w[0]) + i128::from(b.z) * i128::from(w[1]) + i128::from(c.z) * i128::from(w[2])) / i128::from(area)) as u32;
            let rgba = if gouraud { std::array::from_fn(|i| ((i64::from(ca[i]) * w[0] + i64::from(cb[i]) * w[1] + i64::from(cc[i]) * w[2]) / area) as u32) } else { last.rgba() };
            let cov = if env.blend.antialias { coverage([a, b, c], bias, x, y) } else { pixel::FULL };
            pixel::draw(env, local, clut, x, y, z, rgba, cov, || shade(&tex, w, w, ((fog[0] * w[0] + fog[1] * w[1] + fog[2] * w[2]) / area) as u32));
        }
    }
}
