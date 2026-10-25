//! The GS User's Manual's drawing, written apart from `src/gs/`: the vertex queue of §3.2.3 to §3.2.6, the drawing rules
//! of §3.2.9 tested pixel by pixel against each primitive's geometry, the interpolation of §3.2.8 as barycentric
//! weights, and the pixel test, format conversion and masks of §3.7 and §3.9, on S16's model of local memory (whose
//! layout is the transcription of chapter 8). Where the documents leave a choice the model takes the reading
//! Sapphire_Native.md §27.3 records; it decides each by a different formulation from the source's, so that the two
//! agreeing says the reading is implemented, not merely repeated.

#![allow(dead_code)]

use crate::s16::manual::{self as m16, bits};

/// A vertex in primitive coordinates (sixteenths) while queued, in window coordinates when drawn, with Z and colour.
#[derive(Clone, Copy, Debug)]
struct V {
    x: i128,
    y: i128,
    z: i128,
    c: [i128; 4],
}

pub struct Model {
    pub mem: m16::Gs,
    regs: Vec<u64>,
    queue: Vec<V>,
}

impl Default for Model {
    fn default() -> Model {
        Model { mem: m16::Gs::default(), regs: vec![0; 0x63], queue: Vec::new() }
    }
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
        self.queue.push(V {
            x: bitsof(v, 0, 16) as i128,
            y: bitsof(v, 16, 16) as i128,
            z: z as i128,
            c: [0, 8, 16, 24].map(|s| bitsof(rgba, s, 8) as i128),
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

    /// Point: the pixel whose centre is within half a pixel, the lower bound in (§3.2.9, figure 3-3).
    fn point(&mut self, a: V) {
        for (x, y) in self.candidates(&[a.x], &[a.y]) {
            if 16 * x - 8 <= a.x && a.x < 16 * x + 8 && 16 * y - 8 <= a.y && a.y < 16 * y + 8 {
                self.pixel(x, y, a.z, a.c);
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
                self.pixel(x, y, b.z, b.c);
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
            self.pixel(x, y, z, c);
        }
    }

    /// Triangle: centres inside, or on a left or top side (figure 3-6). Each side is classed by the third vertex: a
    /// slanted side is a left side when the third vertex lies to the right of the side's line at its own height, a
    /// level side a top side when the third vertex lies below it.
    fn triangle(&mut self, a: V, b: V, c: V) {
        let last = c;
        let cross = |p: V, q: V, x: i128, y: i128| (q.x - p.x) * (y - p.y) - (q.y - p.y) * (x - p.x);
        let area = cross(a, b, c.x, c.y);
        if area == 0 {
            return;
        }
        let sides = [(b, c, a), (c, a, b), (a, b, c)];
        let drawn_side = |p: V, q: V, r: V| {
            if p.y == q.y {
                r.y > p.y
            } else {
                ((r.x - p.x) * (q.y - p.y) - (q.x - p.x) * (r.y - p.y)) * (q.y - p.y).signum() > 0
            }
        };
        for (x, y) in self.candidates(&[a.x, b.x, c.x], &[a.y, b.y, c.y]) {
            let (px, py) = (16 * x, 16 * y);
            let mut weights = [0i128; 3];
            let mut inside = true;
            for (i, &(p, q, r)) in sides.iter().enumerate() {
                let e = cross(p, q, px, py);
                let towards = cross(p, q, r.x, r.y);
                weights[i] = e * towards.signum();
                if weights[i] < 0 || (weights[i] == 0 && !drawn_side(p, q, r)) {
                    inside = false;
                }
            }
            if !inside {
                continue;
            }
            let total = area.abs();
            let mix = |va: i128, vb: i128, vc: i128| (va * weights[0] + vb * weights[1] + vc * weights[2]).div_euclid(total);
            let z = mix(a.z, b.z, c.z);
            let col = if self.gouraud() { [0, 1, 2, 3].map(|i| mix(a.c[i], b.c[i], c.c[i])) } else { last.c };
            self.pixel(x, y, z, col);
        }
    }

    /// §3.7's tests in order (scissoring, then the depth test; the alpha tests are S19's and off in every program) and
    /// §3.9's format conversion and masks.
    fn pixel(&mut self, x: i128, y: i128, z: i128, c: [i128; 4]) {
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
        let zbuf = self.regs[0x4E + ctx];
        let test = self.regs[0x47 + ctx];
        let Some(fpsm) = frame_format(bitsof(frame, 24, 6) as u32) else { return };
        let (fbp, fbw, fbmsk) = (bitsof(frame, 0, 9) as u32 * 32, bitsof(frame, 16, 6) as u32, (frame >> 32) as u32);
        let (x, y) = (x as u32, y as u32);
        let zpsm = 0x30 | bitsof(zbuf, 24, 4) as u32;
        let zbp = bitsof(zbuf, 0, 9) as u32 * 32;
        let mut z_out = None;
        if bitsof(test, 16, 1) == 1 && [m16::Z32, m16::Z24, m16::Z16, m16::Z16S].contains(&zpsm) {
            let top = (1i128 << bits(zpsm)) - 1;
            let zv = z.min(top) as u32;
            let old = self.mem.get(zpsm, zbp, fbw, x, y);
            let ok = match bitsof(test, 17, 2) {
                0 => false,
                1 => true,
                2 => zv >= old,
                _ => zv > old,
            };
            if !ok {
                return;
            }
            if bitsof(zbuf, 32, 1) == 0 {
                z_out = Some(zv);
            }
        }
        let rgba = (c[0] | c[1] << 8 | c[2] << 16 | c[3] << 24) as u32;
        let (v, m) = if bits(fpsm) == 16 { (to16(rgba), to16(fbmsk)) } else { (rgba, fbmsk) };
        let old = self.mem.get(fpsm, fbp, fbw, x, y);
        self.mem.put(fpsm, fbp, fbw, x, y, (old & m) | (v & !m));
        if let Some(zv) = z_out {
            self.mem.put(zpsm, zbp, fbw, x, y, zv);
        }
    }
}
