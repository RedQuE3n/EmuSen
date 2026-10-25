//! The GS User's Manual's drawing, written apart from `src/gs/`: the vertex queue of §3.2.3 to §3.2.6, the drawing rules
//! of §3.2.9 tested pixel by pixel against each primitive's geometry, the interpolation of §3.2.8 as barycentric
//! weights, and the pixel test, format conversion and masks of §3.7 and §3.9, on S16's model of local memory (whose
//! layout is the transcription of chapter 8). Where the documents leave a choice the model takes the reading
//! Sapphire_Native.md §27.3 records; it decides each by a different formulation from the source's, so that the two
//! agreeing says the reading is implemented, not merely repeated. Texture mapping and fog (S18) are `texture`'s, to
//! which each pixel's weights are handed (Sapphire_Native.md §31).

#![allow(dead_code)]

#[path = "../s18/manual.rs"]
pub mod texture;

use crate::s16::manual::{self as m16, bits};
use texture::{Frag, Tv};

/// A vertex in primitive coordinates (sixteenths) while queued, in window coordinates when drawn, with Z and colour.
#[derive(Clone, Copy, Debug)]
struct V {
    x: i128,
    y: i128,
    z: i128,
    c: [i128; 4],
    t: Tv,
}

pub struct Model {
    pub mem: m16::Gs,
    regs: Vec<u64>,
    queue: Vec<V>,
    tex: texture::Texturing,
}

impl Default for Model {
    fn default() -> Model {
        Model { mem: m16::Gs::default(), regs: vec![0; 0x63], queue: Vec::new(), tex: texture::Texturing::default() }
    }
}

/// The fragment of a pixel whose vertices `v` have weights `wu` along U and `wv` along V, Q from `q`.
fn frag(v: [V; 3], wu: [i128; 3], wv: [i128; 3], q: [u32; 3], last: V) -> Frag {
    Frag { v: v.map(|v| v.t), wu, wv, q, last: last.t }
}

fn bitsof(v: u64, lo: u32, n: u32) -> u64 {
    (v >> lo) & ((1u64 << n) - 1)
}

/// The colour formats FRAME may name, and the Z formats it may name too (§2.4.5, FRAME's page).
fn frame_format(code: u32) -> Option<u32> {
    [m16::CT32, m16::CT24, m16::CT16, m16::CT16S, m16::Z32, m16::Z24, m16::Z16, m16::Z16S].contains(&code).then_some(code)
}

/// The RGBA16 packing of figure 3-17's format conversion: bits 7..3 of R, G and B, bit 7 of A.
fn to16(v: u32) -> u32 {
    let r = (v & 0xFF) >> 3;
    let g = ((v >> 8) & 0xFF) >> 3;
    let b = ((v >> 16) & 0xFF) >> 3;
    let a = (v >> 31) & 1;
    r | g << 5 | b << 10 | a << 15
}

/// Figure 3-6's rule for the point (px, py) against triangle abc: its weights when it is inside, or on a left or top
/// side but not a right or bottom one. Each side is classed by the third vertex: a slanted side is a left side when the
/// third vertex lies to the right of the side's line at its own height, a level side a top side when the third vertex
/// lies below it.
fn holds(a: V, b: V, c: V, px: i128, py: i128) -> Option<[i128; 3]> {
    let cross = |p: V, q: V, x: i128, y: i128| (q.x - p.x) * (y - p.y) - (q.y - p.y) * (x - p.x);
    let drawn_side = |p: V, q: V, r: V| {
        if p.y == q.y {
            r.y > p.y
        } else {
            ((r.x - p.x) * (q.y - p.y) - (q.x - p.x) * (r.y - p.y)) * (q.y - p.y).signum() > 0
        }
    };
    let mut weights = [0i128; 3];
    for (i, &(p, q, r)) in [(b, c, a), (c, a, b), (a, b, c)].iter().enumerate() {
        weights[i] = cross(p, q, px, py) * cross(p, q, r.x, r.y).signum();
        if weights[i] < 0 || (weights[i] == 0 && !drawn_side(p, q, r)) {
            return None;
        }
    }
    Some(weights)
}

impl Model {
    fn attrs(&self) -> u64 {
        if self.regs[0x1A] & 1 == 1 { self.regs[0x00] } else { self.regs[0x1B] }
    }

    fn ctxt(&self) -> usize {
        bitsof(self.attrs(), 9, 1) as usize
    }

    /// A general register written through the host interface.
    pub fn write(&mut self, reg: u8, v: u64) {
        self.mem.write(reg, v);
        if self.mem.busdir & 1 == 1 {
            return;
        }
        assert!(self.mem.pending_signal.is_none(), "S17's programs never stall the GS");
        if (reg as usize) < self.regs.len() {
            self.regs[reg as usize] = v;
        }
        self.tex.write(&mut self.regs, reg, v, &self.mem);
        match reg {
            0x00 => self.queue.clear(),
            0x04 | 0x05 | 0x0C | 0x0D => self.vertex(reg, v),
            _ => {}
        }
    }

    pub fn write_privileged(&mut self, ga: u32, v: u64) {
        self.mem.write_privileged(ga, v);
    }

    pub fn read(&mut self) -> u64 {
        self.mem.read()
    }

    /// §3.2.3's vertex kick and §3.2.4's drawing kick, by the tables of register orders.
    fn vertex(&mut self, reg: u8, v: u64) {
        let with_fog = reg == 0x04 || reg == 0x0C;
        let rgba = self.regs[0x01];
        let z = if with_fog { bitsof(v, 32, 24) } else { bitsof(v, 32, 32) };
        if with_fog {
            // XYZF2 and XYZF3 carry F into FOG (ps2tek "GS Vertex Attributes").
            self.regs[0x0A] = v & 0xFF00_0000_0000_0000;
        }
        self.queue.push(V {
            x: bitsof(v, 0, 16) as i128,
            y: bitsof(v, 16, 16) as i128,
            z: z as i128,
            c: [0, 8, 16, 24].map(|s| bitsof(rgba, s, 8) as i128),
            t: Tv { st: self.regs[0x02], q: (rgba >> 32) as u32, uv: self.regs[0x03], f: bitsof(self.regs[0x0A], 56, 8) as i128 },
        });
        let kind = bitsof(self.regs[0x00], 0, 3);
        let needed = match kind {
            0 | 7 => 1,
            1 | 2 | 6 => 2,
            _ => 3,
        };
        if self.queue.len() < needed {
            return;
        }
        let q = self.queue.clone();
        if reg == 0x04 || reg == 0x05 {
            // §2.4.3: window = primitive - offset, the offset the drawing's context names.
            let o = self.regs[0x18 + self.ctxt()];
            let w: Vec<V> = q.iter().map(|v| V { x: v.x - bitsof(o, 0, 16) as i128, y: v.y - bitsof(o, 32, 16) as i128, ..*v }).collect();
            let q = w;
            match kind {
                0 => self.point(q[0]),
                1 | 2 => self.line(q[0], q[1]),
                3..=5 => self.triangle(q[0], q[1], q[2]),
                6 => self.sprite(q[0], q[1]),
                _ => {}
            }
        }
        self.queue = match kind {
            2 => vec![q[1]],
            4 => vec![q[1], q[2]],
            5 => vec![q[0], q[2]],
            _ => vec![],
        };
    }

    fn scissor(&self) -> (i128, i128, i128, i128) {
        let s = self.regs[0x40 + self.ctxt()];
        (bitsof(s, 0, 11) as i128, bitsof(s, 16, 11) as i128, bitsof(s, 32, 11) as i128, bitsof(s, 48, 11) as i128)
    }

    /// The pixels of a box of sixteenths, widened by one each way, within the scissoring area.
    fn candidates(&self, xs: &[i128], ys: &[i128]) -> Vec<(i128, i128)> {
        let (sx0, sx1, sy0, sy1) = self.scissor();
        let lo = |v: &[i128]| v.iter().min().unwrap().div_euclid(16) - 1;
        let hi = |v: &[i128]| v.iter().max().unwrap().div_euclid(16) + 1;
        let mut out = Vec::new();
        for y in lo(ys).max(sy0)..=hi(ys).min(sy1) {
            for x in lo(xs).max(sx0)..=hi(xs).min(sx1) {
                out.push((x, y));
            }
        }
        out
    }

    fn gouraud(&self) -> bool {
        bitsof(self.attrs(), 3, 1) == 1
    }

    fn antialiased(&self) -> bool {
        bitsof(self.attrs(), 7, 1) == 1
    }

    /// §3.6's coverage of a line's pixel (Sapphire_Disputes.md D-406): 80h less 80h × the line's offset from the
    /// pixel's centre along the minor axis, at the centre's column, below 80h as §3.6.2 says.
    fn cov_line(&self, a: V, b: V, x: i128, y: i128) -> Option<i128> {
        if !self.antialiased() {
            return None;
        }
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let (off, den) = if dx.abs() >= dy.abs() { (a.y * dx + (16 * x - a.x) * dy - 16 * y * dx, dx) } else { (a.x * dy + (16 * y - a.y) * dx - 16 * x * dy, dy) };
        let (off, den) = (off.abs(), den.abs());
        Some(((128 * (16 * den - off)).div_euclid(16 * den)).clamp(0, 127))
    }

    /// The coverage of a triangle's pixel (D-406): half the number of the 256 points at the centres of its sixteenths
    /// that the triangle's drawing rule holds.
    fn cov_triangle(&self, a: V, b: V, c: V, x: i128, y: i128) -> Option<i128> {
        if !self.antialiased() {
            return None;
        }
        let d = |v: V| V { x: 2 * v.x, y: 2 * v.y, ..v };
        let mut n = 0;
        for j in 0..16 {
            for k in 0..16 {
                if holds(d(a), d(b), d(c), 2 * (16 * x - 8 + k) + 1, 2 * (16 * y - 8 + j) + 1).is_some() {
                    n += 1;
                }
            }
        }
        Some(n / 2)
    }

    /// Point: the pixel whose centre is within half a pixel, the lower bound in (§3.2.9, figure 3-3).
    fn point(&mut self, a: V) {
        for (x, y) in self.candidates(&[a.x], &[a.y]) {
            if 16 * x - 8 <= a.x && a.x < 16 * x + 8 && 16 * y - 8 <= a.y && a.y < 16 * y + 8 {
                self.pixel(x, y, a.z, a.c, &frag([a; 3], [1, 0, 0], [1, 0, 0], [a.t.q; 3], a), None);
            }
        }
    }

    /// Sprite: centres on or inside the top and left sides and inside the bottom and right (figure 3-8); colour and Z
    /// from the second vertex (§3.2.8).
    fn sprite(&mut self, a: V, b: V) {
        let (x0, x1) = (a.x.min(b.x), a.x.max(b.x));
        let (y0, y1) = (a.y.min(b.y), a.y.max(b.y));
        for (x, y) in self.candidates(&[a.x, b.x], &[a.y, b.y]) {
            if x0 <= 16 * x && 16 * x < x1 && y0 <= 16 * y && 16 * y < y1 {
                // S and U run from a to b along x, T and V along y; Q and F are b's (§3.2.8; Native §31.3).
                let lin = |p: i128, pa: i128, pb: i128| if pb > pa { [pb - p, p - pa, 0] } else { [p - pb, pa - p, 0] };
                let bf = V { t: Tv { f: b.t.f, ..a.t }, ..a };
                self.pixel(x, y, b.z, b.c, &frag([bf, b, b], lin(16 * x, a.x, b.x), lin(16 * y, a.y, b.y), [b.t.q; 3], b), None);
            }
        }
    }

    /// Line: a pixel is drawn when the segment meets its diamond (figure 3-5), the open diamond or the one corner a
    /// crossing between two diamonds belongs to (the upper for an x-major line, the left for a y-major one), and the
    /// end point is not in it; the minimum of |x - cx| + |y - cy| along the segment is found at its ends or where it
    /// crosses the centre's row or column, each tried exactly.
    fn line(&mut self, a: V, b: V) {
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        if dx == 0 && dy == 0 {
            return;
        }
        let xmajor = dx.abs() >= dy.abs();
        let corner = |x: i128, y: i128| if xmajor { (16 * x, 16 * y - 8) } else { (16 * x - 8, 16 * y) };
        let holds = |px: i128, py: i128, x: i128, y: i128| (px - 16 * x).abs() + (py - 16 * y).abs() < 8 || (px, py) == corner(x, y);
        for (x, y) in self.candidates(&[a.x, b.x], &[a.y, b.y]) {
            let (cx, cy) = (16 * x, 16 * y);
            let mut ts: Vec<(i128, i128)> = vec![(0, 1), (1, 1)];
            if dx != 0 {
                ts.push(if dx > 0 { (cx - a.x, dx) } else { (a.x - cx, -dx) });
            }
            if dy != 0 {
                ts.push(if dy > 0 { (cy - a.y, dy) } else { (a.y - cy, -dy) });
            }
            let meets = ts.iter().filter(|(n, d)| 0 <= *n && n <= d).any(|&(n, d)| (a.x * d + n * dx - cx * d).abs() + (a.y * d + n * dy - cy * d).abs() < 8 * d);
            let (kx, ky) = corner(x, y);
            let on = (kx - a.x) * dy == (ky - a.y) * dx && a.x.min(b.x) <= kx && kx <= a.x.max(b.x) && a.y.min(b.y) <= ky && ky <= a.y.max(b.y);
            if !(meets || on) || holds(b.x, b.y, x, y) {
                continue;
            }
            // Along the major axis: t = (centre - a) / (b - a), continued past the ends, the result held to its range.
            let (mut t, mut d) = if xmajor { (cx - a.x, dx) } else { (cy - a.y, dy) };
            if d < 0 {
                t = -t;
                d = -d;
            }
            let lerp = |p: i128, q: i128| (p * (d - t) + q * t).div_euclid(d);
            let z = lerp(a.z, b.z).clamp(0, u32::MAX as i128);
            let c = if self.gouraud() { [0, 1, 2, 3].map(|i| lerp(a.c[i], b.c[i]).clamp(0, 255)) } else { b.c };
            let w = [d - t, t, 0];
            self.pixel(x, y, z, c, &frag([a, b, b], w, w, [a.t.q, b.t.q, b.t.q], b), self.cov_line(a, b, x, y));
        }
    }

    /// Triangle: the centres `holds` takes (figure 3-6), colour and Z by the weights it gives.
    fn triangle(&mut self, a: V, b: V, c: V) {
        let last = c;
        let area = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
        if area == 0 {
            return;
        }
        for (x, y) in self.candidates(&[a.x, b.x, c.x], &[a.y, b.y, c.y]) {
            let Some(weights) = holds(a, b, c, 16 * x, 16 * y) else { continue };
            let total = area.abs();
            let mix = |va: i128, vb: i128, vc: i128| (va * weights[0] + vb * weights[1] + vc * weights[2]).div_euclid(total);
            let z = mix(a.z, b.z, c.z);
            let col = if self.gouraud() { [0, 1, 2, 3].map(|i| mix(a.c[i], b.c[i], c.c[i])) } else { last.c };
            self.pixel(x, y, z, col, &frag([a, b, c], weights, weights, [a.t.q, b.t.q, c.t.q], last), self.cov_triangle(a, b, c, x, y));
        }
    }

    /// §3.7's tests in order (scissoring, the alpha test with AFAIL, the destination alpha test, the depth test),
    /// §3.8's blending, and §3.9's dithering, clamp, alpha correction, format conversion and masks (`pixel_ops`).
    /// `cov` is the pixel's coverage when the primitive is antialiased (§3.6.1).
    fn pixel(&mut self, x: i128, y: i128, z: i128, c: [i128; 4], frag: &Frag, cov: Option<i128>) {
        let ctx = self.ctxt();
        let (sx0, sx1, sy0, sy1) = self.scissor();
        if x < sx0 || x > sx1 || y < sy0 || y > sy1 {
            return;
        }
        let msk = self.regs[0x22] & 3;
        if (msk == 2 && y % 2 == 0) || (msk == 3 && y % 2 == 1) {
            return;
        }
        let frame = self.regs[0x4C + ctx];
        let Some(fpsm) = frame_format(bitsof(frame, 24, 6) as u32) else { return };
        // §3.4 and §3.5: texture mapping and fog before the tests (figure 3-1).
        let c = self.tex.shade(&self.regs, self.attrs(), &self.mem, frag, c);
        let zbuf = self.regs[0x4E + ctx];
        let (fbp, fbw, fbmsk) = (bitsof(frame, 0, 9) as u32 * 32, bitsof(frame, 16, 6) as u32, (frame >> 32) as u32);
        let zpsm = 0x30 | bitsof(zbuf, 24, 4) as u32;
        let zbp = bitsof(zbuf, 0, 9) as u32 * 32;
        let regs = pixel_ops::Regs {
            test: self.regs[0x47 + ctx],
            alpha: self.regs[0x42 + ctx],
            fba: self.regs[0x4A + ctx],
            pabe: self.regs[0x49],
            dthe: self.regs[0x45],
            dimx: self.regs[0x44],
            colclamp: self.regs[0x46],
            abe: bitsof(self.attrs(), 6, 1) == 1,
        };
        let (xu, yu) = (x as u32, y as u32);
        let z_format = [m16::Z32, m16::Z24, m16::Z16, m16::Z16S].contains(&zpsm);
        let fb = self.mem.get(fpsm, fbp, fbw, xu, yu);
        let zb = if z_format { self.mem.get(zpsm, zbp, fbw, xu, yu) } else { 0 };
        let p = pixel_ops::Pixel { x, y, z, c, cov };
        let out = pixel_ops::run(&regs, &p, fpsm, fbmsk, fb, z_format.then_some((zpsm, zb, bitsof(zbuf, 32, 1) == 1)));
        if let Some(v) = out.frame {
            self.mem.put(fpsm, fbp, fbw, xu, yu, v);
        }
        if let Some(zv) = out.z {
            self.mem.put(zpsm, zbp, fbw, xu, yu, zv);
        }
    }
}

/// §3.6 to §3.9 for one pixel that has passed scissoring and been textured and fogged, as the manual's tables state
/// them, each table transcribed as a table: what is written to the frame and the Z buffer, given the pixel and both
/// buffers' present values.
pub mod pixel_ops {
    use super::{bits, to16};

    pub struct Regs {
        pub test: u64,
        pub alpha: u64,
        pub fba: u64,
        pub pabe: u64,
        pub dthe: u64,
        pub dimx: u64,
        pub colclamp: u64,
        pub abe: bool,
    }

    pub struct Pixel {
        pub x: i128,
        pub y: i128,
        pub z: i128,
        pub c: [i128; 4],
        pub cov: Option<i128>,
    }

    pub struct Out {
        pub frame: Option<u32>,
        pub z: Option<u32>,
    }

    fn field(v: u64, lo: u32, n: u32) -> u64 {
        (v >> lo) & ((1 << n) - 1)
    }

    /// A frame value as RGBA in eight bits a channel, the way §3.8.1's last paragraph treats the formats.
    fn unpack(fpsm: u32, v: u32) -> [i128; 4] {
        if bits(fpsm) == 16 {
            let ch = |s: u32| i128::from((v >> s) & 31) * 8;
            [ch(0), ch(5), ch(10), if v & 0x8000 != 0 { 0x80 } else { 0 }]
        } else {
            let ch = |s: u32| i128::from((v >> s) & 255);
            [ch(0), ch(8), ch(16), if bits(fpsm) == 24 { 0x80 } else { ch(24) }]
        }
    }

    pub fn run(r: &Regs, p: &Pixel, fpsm: u32, fbmsk: u32, fb: u32, zbuf: Option<(u32, u32, bool)>) -> Out {
        let none = Out { frame: None, z: None };
        let mut c = p.c;
        // §3.6.1's table, row by row: (AA1, ABE, A is 80h) -> (alpha output is Cov, blending runs).
        let blending_asked = match (p.cov, r.abe) {
            (None, abe) => abe,
            (Some(cov), false) => {
                c[3] = cov;
                true
            }
            (Some(cov), true) => {
                if c[3] == 0x80 {
                    c[3] = cov;
                }
                true
            }
        };
        // §3.7.2: the comparison as a list indexed by ATST, then AFAIL's table of what stays effective.
        let (mut eff_rgb, mut eff_a, mut eff_z) = (true, true, true);
        if field(r.test, 0, 1) == 1 {
            let (a, aref) = (c[3], i128::from(field(r.test, 4, 8)));
            let passes = [false, true, a < aref, a <= aref, a == aref, a >= aref, a > aref, a != aref];
            if !passes[field(r.test, 1, 3) as usize] {
                let rgba32 = bits(fpsm) == 32;
                let table = [(false, false, false), (true, true, false), (false, false, true), if rgba32 { (true, false, false) } else { (true, true, false) }];
                (eff_rgb, eff_a, eff_z) = table[field(r.test, 12, 2) as usize];
                if !eff_rgb && !eff_a && !eff_z {
                    return none;
                }
            }
        }
        // §3.7.3's three tables.
        if field(r.test, 14, 1) == 1 {
            let datm = field(r.test, 15, 1) as u32;
            let pass = match bits(fpsm) {
                32 => (fb >> 31) == datm,
                16 => ((fb >> 15) & 1) == datm,
                _ => true,
            };
            if !pass {
                return none;
            }
        }
        // §3.7.4, with D-163's clamp; the Z write needs ZMSK 0, an effective Z, and full coverage (§3.6.2).
        let mut z_out = None;
        if field(r.test, 16, 1) == 1
            && let Some((zpsm, old, zmsk)) = zbuf
        {
            let zv = p.z.min((1i128 << bits(zpsm)) - 1) as u32;
            let pass = [false, true, zv >= old, zv > old][field(r.test, 17, 2) as usize];
            if !pass {
                return none;
            }
            if !zmsk && eff_z && p.cov.is_none_or(|v| v >= 0x80) {
                z_out = Some(zv);
            }
        }
        if !eff_rgb && !eff_a {
            return Out { frame: None, z: z_out };
        }
        // §3.8.1: (A - B) * C + D with X * Y = (X x Y) >> 7, unclamped; PABE (§3.8.2) by the source alpha's MSB.
        let d = unpack(fpsm, fb);
        let mut rgb = [c[0], c[1], c[2]];
        if blending_asked && !(field(r.pabe, 0, 1) == 1 && c[3] < 0x80) {
            let colour = |sel: u64, i: usize| [c[i], d[i], 0][sel.min(2) as usize];
            let coef = [c[3], d[3], i128::from(field(r.alpha, 32, 8))][field(r.alpha, 4, 2).min(2) as usize];
            for (i, v) in rgb.iter_mut().enumerate() {
                let diff = colour(field(r.alpha, 0, 2), i) - colour(field(r.alpha, 2, 2), i);
                *v = (diff * coef).div_euclid(128) + colour(field(r.alpha, 6, 2), i);
            }
        }
        // §3.9.1: the 4 x 4 matrix of DIMX, DMyx at bits 16y + 4x, each int 1:2:0, added at [Y % 4][X % 4].
        if field(r.dthe, 0, 1) == 1 && bits(fpsm) == 16 {
            let mut m = [[0i128; 4]; 4];
            for (row, line) in m.iter_mut().enumerate() {
                for (col, e) in line.iter_mut().enumerate() {
                    let raw = field(r.dimx, (16 * row + 4 * col) as u32, 3) as i128;
                    *e = if raw >= 4 { raw - 8 } else { raw };
                }
            }
            let off = m[p.y.rem_euclid(4) as usize][p.x.rem_euclid(4) as usize];
            rgb = rgb.map(|v| v + off);
        }
        // §3.9.2: clamped to [0, 255], or the lower 8 bits.
        let rgb = rgb.map(|v| if field(r.colclamp, 0, 1) == 1 { v.max(0).min(255) } else { v.rem_euclid(256) } as u32);
        // §3.9.3: As with FBA ORed in as its MSB.
        let a = c[3] as u32 | if field(r.fba, 0, 1) == 1 { 0x80 } else { 0 };
        let mut v = rgb[0] | rgb[1] << 8 | rgb[2] << 16 | a << 24;
        let mut keep = fbmsk;
        if !eff_a {
            keep |= 0xFF00_0000;
        }
        if bits(fpsm) == 16 {
            v = to16(v);
            keep = to16(keep);
        }
        Out { frame: Some((fb & keep) | (v & !keep)), z: z_out }
    }
}
