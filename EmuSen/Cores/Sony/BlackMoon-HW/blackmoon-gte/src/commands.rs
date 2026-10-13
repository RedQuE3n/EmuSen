//! The 22 commands as fixed-width arithmetic, from psx-spx's "GTE Coordinate Calculation Commands", "GTE General
//! Purpose Calculation Commands", "GTE Color Calculation Commands" and "GTE Saturation"; where those are silent, the
//! rules BlackMoon_GTE.md §5 gives with the evidence for each.

use crate::divide::divide;
use crate::registers::{control as c, data as d, flag_word};
use crate::{Observer, Projection, Registers};

/// Each number's cycles by bits 0-5: the 22 commands' are psx-spx's ("GTE Command Summary"), the 42 numbers it lists as
/// N/A take those amidog's psxtest_gte 1.4 expects of a console (BlackMoon_GTE.md §6, D-16).
#[rustfmt::skip]
pub const CYCLES: [u8; 64] = [
    16, 15, 13, 12,  5, 18,  8,  7,  6,  5, 19, 18,  6,  5, 19, 18,
     8,  8,  8, 19, 13,  8, 44, 31, 17, 13,  8, 17, 11,  5, 14,  8,
    30, 17, 15, 14, 12,  8,  6,  5,  5,  8, 17, 13,  8,  5,  6, 18,
    23, 22, 20, 19, 17, 16, 14, 13, 11,  7,  6,  5, 18,  5,  5, 39,
];

type V3 = [i64; 3];
type M3 = [[i64; 3]; 3];

fn lo(v: u32) -> i64 {
    v as u16 as i16 as i64
}

fn hi(v: u32) -> i64 {
    (v >> 16) as u16 as i16 as i64
}

/// One command's run: the registers, the FLAG bits it raises, its shift and its lm bit.
struct Run<'a, O: Observer + ?Sized> {
    r: &'a mut Registers,
    o: &'a mut O,
    flag: u32,
    sh: u32,
    lm: bool,
}

impl<O: Observer + ?Sized> Run<'_, O> {
    // --- the inputs ---

    fn matrix(&self, base: usize) -> M3 {
        let w = &self.r.control[base..base + 5];
        [[lo(w[0]), hi(w[0]), lo(w[1])], [hi(w[1]), lo(w[2]), hi(w[2])], [lo(w[3]), hi(w[3]), lo(w[4])]]
    }

    fn vector(&self, n: usize) -> V3 {
        if n == 3 {
            return self.ir();
        }
        [lo(self.r.data[2 * n]), hi(self.r.data[2 * n]), lo(self.r.data[2 * n + 1])]
    }

    fn ir(&self) -> V3 {
        [1, 2, 3].map(|i| self.r.data[d::IR0 + i] as i32 as i64)
    }

    fn ir0(&self) -> i64 {
        self.r.data[d::IR0] as i32 as i64
    }

    fn control3(&self, base: usize) -> V3 {
        [0, 1, 2].map(|i| self.r.control[base + i] as i32 as i64)
    }

    fn rgb(&self, word: u32) -> V3 {
        [0, 8, 16].map(|s| (word >> s & 0xFF) as i64)
    }

    // --- the outputs, each with its saturation and FLAG bit (psx-spx "GTE Saturation") ---

    /// MAC1-MAC3's 44-bit overflow, FLAG 30-28 and 27-25.
    fn check(&mut self, i: usize, v: i64) -> i64 {
        if v >= 1 << 43 {
            self.flag |= 1 << (31 - i);
        } else if v < -(1 << 43) {
            self.flag |= 1 << (28 - i);
        }
        v << 20 >> 20
    }

    /// T*1000h + the row times V, checked after each addition (BlackMoon_GTE.md §5.2).
    fn sum(&mut self, i: usize, t: i64, row: [i64; 3], v: V3) -> i64 {
        let mut s = self.check(i, t << 12);
        for k in 0..3 {
            s = self.check(i, s + row[k] * v[k]);
        }
        s
    }

    /// A MAC1-MAC3 result shifted by sf, written to its register.
    fn mac(&mut self, i: usize, v: i64) -> i64 {
        let v = self.check(i, v) >> self.sh;
        self.r.data[d::MAC0 + i] = v as u32;
        v as i32 as i64
    }

    fn saturate_ir(&mut self, i: usize, v: i64, lm: bool) -> i64 {
        let low = if lm { 0 } else { -0x8000 };
        if v < low || v > 0x7FFF {
            self.flag |= 1 << (25 - i);
        }
        v.clamp(low, 0x7FFF)
    }

    fn set_ir(&mut self, i: usize, v: i64, lm: bool) {
        let v = self.saturate_ir(i, v, lm);
        self.r.data[d::IR0 + i] = v as i32 as u32;
    }

    fn set_irs(&mut self, m: V3) {
        for (i, v) in m.into_iter().enumerate() {
            self.set_ir(i + 1, v, self.lm);
        }
    }

    fn mac0(&mut self, v: i64) -> i64 {
        if v > i32::MAX as i64 {
            self.flag |= 1 << 16;
        } else if v < i32::MIN as i64 {
            self.flag |= 1 << 15;
        }
        self.r.data[d::MAC0] = v as u32;
        v
    }

    fn push_colour(&mut self, m: V3) {
        let mut word = self.r.data[d::RGBC] & 0xFF00_0000;
        for (i, &v) in m.iter().enumerate() {
            let v = v >> 4;
            if !(0..=0xFF).contains(&v) {
                self.flag |= 1 << (21 - i);
            }
            word |= (v.clamp(0, 0xFF) as u32) << (8 * i);
        }
        self.r.data.copy_within(d::RGB0 + 1..=d::RGB2, d::RGB0);
        self.r.data[d::RGB2] = word;
    }

    fn push_sz(&mut self, v: i64) -> u16 {
        if !(0..=0xFFFF).contains(&v) {
            self.flag |= 1 << 18;
        }
        let z = v.clamp(0, 0xFFFF) as u16;
        self.r.data.copy_within(d::SZ0 + 1..=d::SZ3, d::SZ0);
        self.r.data[d::SZ3] = z as u32;
        z
    }

    fn screen(&mut self, v: i64, bit: u32) -> u32 {
        let v = v >> 16;
        if !(-0x400..=0x3FF).contains(&v) {
            self.flag |= 1 << bit;
        }
        v.clamp(-0x400, 0x3FF) as u16 as u32
    }

    fn set_ir0(&mut self, v: i64) {
        let v = v >> 12;
        if !(0..=0x1000).contains(&v) {
            self.flag |= 1 << 12;
        }
        self.r.data[d::IR0] = v.clamp(0, 0x1000) as u32;
    }

    fn set_otz(&mut self, v: i64) {
        let v = v >> 12;
        if !(0..=0xFFFF).contains(&v) {
            self.flag |= 1 << 18;
        }
        self.r.data[d::OTZ] = v.clamp(0, 0xFFFF) as u32;
    }

    // --- the shared steps ---

    /// T*1000h + M*V, each row to its MAC and IR (psx-spx MVMVA).
    fn transform(&mut self, t: V3, m: M3, v: V3) -> V3 {
        let mut out = [0; 3];
        for i in 0..3 {
            let s = self.sum(i + 1, t[i], m[i], v);
            out[i] = self.mac(i + 1, s);
        }
        self.set_irs(out);
        out
    }

    /// "MAC+(FC-MAC)*IR0" on the unshifted products (psx-spx's "Details"): the difference saturated as if lm=0.
    fn cue(&mut self, m: V3) -> V3 {
        let fc = self.control3(c::FC);
        for i in 0..3 {
            let diff = (self.check(i + 1, (fc[i] << 12) - m[i]) >> self.sh) as i32 as i64;
            self.set_ir(i + 1, diff, false);
        }
        let (ir, ir0) = (self.ir(), self.ir0());
        [0, 1, 2].map(|i| ir[i] * ir0 + m[i])
    }

    /// The last step of every colour command: the MACs shifted, the colour pushed, IR from the MACs.
    fn finish_colour(&mut self, m: V3) {
        let out = [0, 1, 2].map(|i| self.mac(i + 1, m[i]));
        self.push_colour(out);
        self.set_irs(out);
    }

    /// [R*IR1, G*IR2, B*IR3] SHL 4, from RGBC.
    fn tint(&self) -> V3 {
        let (rgb, ir) = (self.rgb(self.r.data[d::RGBC]), self.ir());
        [0, 1, 2].map(|i| (rgb[i] * ir[i]) << 4)
    }

    /// The light then the colour matrix on vector n: NCS's first two lines.
    fn light(&mut self, n: usize) {
        let (llm, lcm, bk, v) = (self.matrix(c::LLM), self.matrix(c::LCM), self.control3(c::BK), self.vector(n));
        self.transform([0; 3], llm, v);
        let ir = self.ir();
        self.transform(bk, lcm, ir);
    }

    // --- the commands ---

    fn rtp(&mut self, n: usize, last: bool) {
        let (rt, tr, v) = (self.matrix(c::RT), self.control3(c::TR), self.vector(n));
        let (mut sum, mut mac) = ([0; 3], [0; 3]);
        for i in 0..3 {
            sum[i] = self.sum(i + 1, tr[i], rt[i], v);
            mac[i] = self.mac(i + 1, sum[i]);
        }
        self.set_ir(1, mac[0], self.lm);
        self.set_ir(2, mac[1], self.lm);
        let ir3 = mac[2].clamp(if self.lm { 0 } else { -0x8000 }, 0x7FFF);
        if !(-0x8000..=0x7FFF).contains(&(sum[2] >> 12)) {
            self.flag |= 1 << 22;
        }
        self.r.data[d::IR0 + 3] = ir3 as i32 as u32;
        let sz = self.push_sz(sum[2] >> 12);
        let (q, overflow) = divide(self.r.control[c::H] as u16, sz);
        if overflow {
            self.flag |= 1 << 17;
        }
        let ir = self.ir();
        let x = self.mac0(q as i64 * ir[0] + self.r.control[c::OFX] as i32 as i64);
        let y = self.mac0(q as i64 * ir[1] + self.r.control[c::OFY] as i32 as i64);
        let word = self.screen(x, 14) | self.screen(y, 13) << 16;
        self.r.data.copy_within(d::SXY0 + 1..=d::SXY2, d::SXY0);
        self.r.data[d::SXY2] = word;
        let (h, offset) = (self.r.control[c::H] as u16, [self.r.control[c::OFX] as i32, self.r.control[c::OFY] as i32]);
        self.o.projected(&Projection { vertex: n as u8, sums: sum, quotient: q, x, y, z: sz, word, h, offset });
        if last {
            let dq = self.mac0(q as i64 * self.r.control[c::DQA] as i32 as i64 + self.r.control[c::DQB] as i32 as i64);
            self.set_ir0(dq);
        }
    }

    fn nclip(&mut self) {
        let s = [0, 1, 2].map(|i| self.r.data[d::SXY0 + i]);
        let (x, y) = (s.map(lo), s.map(hi));
        self.mac0(x[0] * y[1] + x[1] * y[2] + x[2] * y[0] - x[0] * y[2] - x[1] * y[0] - x[2] * y[1]);
    }

    fn avsz(&mut self, four: bool) {
        let z: i64 = (if four { 0 } else { 1 }..4).map(|i| self.r.data[d::SZ0 + i] as i64).sum();
        let zsf = self.r.control[if four { c::ZSF4 } else { c::ZSF3 }] as i32 as i64;
        let v = self.mac0(zsf * z);
        self.set_otz(v);
    }

    fn op(&mut self) {
        let rt = self.matrix(c::RT);
        let (dd, ir) = ([rt[0][0], rt[1][1], rt[2][2]], self.ir());
        let m = [ir[2] * dd[1] - ir[1] * dd[2], ir[0] * dd[2] - ir[2] * dd[0], ir[1] * dd[0] - ir[0] * dd[1]];
        let out = [0, 1, 2].map(|i| self.mac(i + 1, m[i]));
        self.set_irs(out);
    }

    fn sqr(&mut self) {
        let ir = self.ir();
        let out = [0, 1, 2].map(|i| self.mac(i + 1, ir[i] * ir[i]));
        self.set_irs(out);
    }

    fn mvmva(&mut self, command: u32) {
        let (mx, v, cv) = ((command >> 17 & 3) as usize, (command >> 15 & 3) as usize, command >> 13 & 3);
        let m = match mx {
            0 => self.matrix(c::RT),
            1 => self.matrix(c::LLM),
            2 => self.matrix(c::LCM),
            _ => {
                let r = (self.r.data[d::RGBC] & 0xFF) as i64 * 0x10;
                let rt = self.matrix(c::RT);
                [[-r, r, self.ir0()], [rt[0][2]; 3], [rt[1][1]; 3]]
            }
        };
        let v = self.vector(v);
        match cv {
            2 => {
                let fc = self.control3(c::FC);
                let mut out = [0; 3];
                for i in 0..3 {
                    let first = self.check(i + 1, (fc[i] << 12) + m[i][0] * v[0]) >> self.sh;
                    self.saturate_ir(i + 1, first as i32 as i64, false);
                    out[i] = self.mac(i + 1, m[i][1] * v[1] + m[i][2] * v[2]);
                }
                self.set_irs(out);
            }
            _ => {
                let t = match cv {
                    0 => self.control3(c::TR),
                    1 => self.control3(c::BK),
                    _ => [0; 3],
                };
                self.transform(t, m, v);
            }
        }
    }

    fn ncs(&mut self, n: usize) {
        self.light(n);
        let m = [1, 2, 3].map(|i| self.r.data[d::MAC0 + i] as i32 as i64);
        self.push_colour(m);
    }

    fn ncc(&mut self, n: usize) {
        self.light(n);
        let m = self.tint();
        self.finish_colour(m);
    }

    fn ncd(&mut self, n: usize) {
        self.light(n);
        let m = self.tint();
        let m = self.cue(m);
        self.finish_colour(m);
    }

    fn cc(&mut self, depth: bool) {
        let (lcm, bk, ir) = (self.matrix(c::LCM), self.control3(c::BK), self.ir());
        self.transform(bk, lcm, ir);
        let mut m = self.tint();
        if depth {
            m = self.cue(m);
        }
        self.finish_colour(m);
    }

    fn dcpl(&mut self) {
        let m = self.tint();
        let m = self.cue(m);
        self.finish_colour(m);
    }

    fn dpc(&mut self, word: u32) {
        let m = self.rgb(word).map(|v| v << 16);
        let m = self.cue(m);
        self.finish_colour(m);
    }

    fn intpl(&mut self) {
        let m = self.ir().map(|v| v << 12);
        let m = self.cue(m);
        self.finish_colour(m);
    }

    fn gp(&mut self, base: bool) {
        let (ir, ir0) = (self.ir(), self.ir0());
        let mut m = [0; 3];
        for i in 0..3 {
            let start = if base { (self.r.data[d::MAC0 + 1 + i] as i32 as i64) << self.sh } else { 0 };
            m[i] = ir[i] * ir0 + start;
        }
        self.finish_colour(m);
    }
}

/// Runs `command` (the COP2 instruction's low 25 bits) on `r`; the cycles it takes.
pub fn run<O: Observer + ?Sized>(r: &mut Registers, command: u32, o: &mut O) -> u32 {
    let mut x = Run { r, o, flag: 0, sh: if command >> 19 & 1 != 0 { 12 } else { 0 }, lm: command >> 10 & 1 != 0 };
    let number = (command & 0x3F) as usize;
    match number {
        0x00 | 0x01 => x.rtp(0, true),
        0x06 => x.nclip(),
        0x0C => x.op(),
        0x10 => x.dpc(x.r.data[d::RGBC]),
        0x11 => x.intpl(),
        0x12 => x.mvmva(command),
        0x13 => x.ncd(0),
        0x14 => x.cc(true),
        0x16 => (0..3).for_each(|n| x.ncd(n)),
        0x1B => x.ncc(0),
        0x1C => x.cc(false),
        0x1E => x.ncs(0),
        0x20 => (0..3).for_each(|n| x.ncs(n)),
        0x28 => x.sqr(),
        0x29 => x.dcpl(),
        0x2A => (0..3).for_each(|_| x.dpc(x.r.data[d::RGB0])),
        0x2D => x.avsz(false),
        0x2E => x.avsz(true),
        0x30 => (0..3).for_each(|n| x.rtp(n, n == 2)),
        0x3D => x.gp(false),
        0x3E => x.gp(true),
        0x3F => (0..3).for_each(|n| x.ncc(n)),
        _ => {}
    }
    x.r.control[c::FLAG] = flag_word(x.flag);
    CYCLES[number] as u32
}
