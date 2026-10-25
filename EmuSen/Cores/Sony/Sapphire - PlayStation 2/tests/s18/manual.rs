//! The GS User's Manual's texture mapping and fog, written apart from `src/gs/` (Sapphire_Native.md §31.3): §2.7's
//! CLUT arrangements and §3.4.7's temporary buffer, §3.4.5's wrap formulas, §3.4.6's conversions, §3.4.8's samplers,
//! §3.4.9's table, §3.4.11 and §3.4.12's levels, and §3.5's formula, on S16's model of memory. S17's model
//! (`crate::s17::manual`) rasterises and hands each pixel its weights here. The texture coordinates are decided by
//! another formulation from the source's: S, T and Q aligned to one exponent in 128-bit integers (the programs keep
//! their exponents close enough for that, which the model asserts), and log2 in double precision, asserted to be far
//! from a sixteenth's boundary unless it is exact.

#![allow(dead_code)]

use crate::s16::manual::{self as m16, CT16, CT16S, CT24, CT32, T4, T4HH, T4HL, T8, T8H, Z16, Z16S, Z24, Z32, bits};

fn bitsof(v: u64, lo: u32, n: u32) -> u64 {
    (v >> lo) & ((1u64 << n) - 1)
}

/// What a vertex carries for texturing and fog.
#[derive(Clone, Copy, Debug, Default)]
pub struct Tv {
    pub st: u64,
    pub q: u32,
    pub uv: u64,
    pub f: i128,
}

/// A pixel's place in its primitive: the vertices and their weights along U and V (equal but for a sprite), each set
/// with a positive total, the Q each vertex contributes, and the vertex of the drawing kick.
#[derive(Clone, Copy, Debug)]
pub struct Frag {
    pub v: [Tv; 3],
    pub wu: [i128; 3],
    pub wv: [i128; 3],
    pub q: [u32; 3],
    pub last: Tv,
}

/// The temporary buffer as figure 3-12 and the CSA tables draw it: 512 sixteen-bit places; a PSMCT32 entry at offset k
/// fills place k (bits 15..0) and place 256 + k (bits 31..16).
pub struct Texturing {
    places: Vec<u16>,
    cbp0: u64,
    cbp1: u64,
}

impl Default for Texturing {
    fn default() -> Texturing {
        Texturing { places: vec![0; 512], cbp0: 0, cbp1: 0 }
    }
}

/// §2.7.3's IDTEX8 arrangement in CSM1, read row by row: two rows of eight-wide halves per 32 indices.
fn csm1_index_at(x: u32, y: u32) -> u32 {
    let pair = y / 2;
    let half = x / 8;
    pair * 32 + half * 16 + (y % 2) * 8 + x % 8
}

/// The single-precision value as an exact sign, significand and power of two.
fn decode(bits: u32) -> (i128, i32) {
    let e = ((bits >> 23) & 0xFF) as i32;
    if e == 0 {
        return (0, 0);
    }
    let m = i128::from((bits & 0x7F_FFFF) | 0x80_0000);
    (if bits >> 31 == 1 { -m } else { m }, e - 150)
}

impl Texturing {
    /// The CLUT the TEX0 value `t0` asks for, loaded under CLD's conditions (§3.4.7's tables, TEX0's page).
    fn load(&mut self, t0: u64, texclut: u64, mem: &m16::Gs) {
        let psm = bitsof(t0, 20, 6) as u32;
        let count = match psm {
            T8 | T8H => 256,
            T4 | T4HL | T4HH => 16,
            _ => return,
        };
        let cbp = bitsof(t0, 37, 14);
        let go = match bitsof(t0, 61, 3) {
            1 => true,
            2 => {
                self.cbp0 = cbp;
                true
            }
            3 => {
                self.cbp1 = cbp;
                true
            }
            4 => {
                let differs = self.cbp0 != cbp;
                self.cbp0 = cbp;
                differs
            }
            5 => {
                let differs = self.cbp1 != cbp;
                self.cbp1 = cbp;
                differs
            }
            _ => false,
        };
        if !go {
            return;
        }
        let cpsm = match bitsof(t0, 51, 4) as u32 {
            0x2 => CT16,
            0xA => CT16S,
            _ => CT32,
        };
        let offset = bitsof(t0, 56, 5) as u32 * 16;
        let csm2 = bitsof(t0, 55, 1) == 1;
        let mut cells = Vec::new();
        if csm2 {
            let (cbw, cou, cov) = (bitsof(texclut, 0, 6) as u32, bitsof(texclut, 6, 6) as u32 * 16, bitsof(texclut, 12, 10) as u32);
            for i in 0..count {
                cells.push((i, cou + i, cov, cbw));
            }
        } else if count == 16 {
            for y in 0..2 {
                for x in 0..8 {
                    cells.push((y * 8 + x, x, y, 1));
                }
            }
        } else {
            for y in 0..16 {
                for x in 0..16 {
                    cells.push((csm1_index_at(x, y), x, y, 1));
                }
            }
        }
        for (i, x, y, bw) in cells {
            let v = mem.get(cpsm, cbp as u32, bw, x, y);
            if cpsm == CT32 {
                let k = ((offset + i) % 256) as usize;
                self.places[k] = (v & 0xFFFF) as u16;
                self.places[k + 256] = (v >> 16) as u16;
            } else {
                self.places[((offset + i) % 512) as usize] = v as u16;
            }
        }
    }

    /// A general register written, after S17's model has stored it: TEX0 (with MTBA's calculation) and TEX2, merged into
    /// TEX0's CLUT fields, both load the CLUT.
    pub fn write(&mut self, regs: &mut [u64], reg: u8, v: u64, mem: &m16::Gs) {
        match reg {
            0x06 | 0x07 => {
                let c = (reg - 0x06) as usize;
                if bitsof(regs[0x14 + c], 9, 1) == 1 {
                    regs[0x34 + c] = auto_levels(v);
                }
                self.load(v, regs[0x1C], mem);
            }
            0x16 | 0x17 => {
                let c = (reg - 0x16) as usize;
                let keep = (1u64 << 20) - 1 | ((1u64 << 11) - 1) << 26;
                regs[0x06 + c] = (regs[0x06 + c] & keep) | (v & !keep);
                self.load(regs[0x06 + c], regs[0x1C], mem);
            }
            _ => {}
        }
    }

    /// RGBA16 by §3.4.6: bits ED CBA to EDCBA000, alpha by the TEXA table.
    fn from16(v: u32, texa: u64) -> [i128; 4] {
        let (ta0, aem, ta1) = (bitsof(texa, 0, 8), bitsof(texa, 15, 1), bitsof(texa, 32, 8));
        let (r, g, b) = (v & 31, (v >> 5) & 31, (v >> 10) & 31);
        let a = if v >> 15 == 1 { ta1 } else if aem == 1 && r == 0 && g == 0 && b == 0 { 0 } else { ta0 };
        [r * 8, g * 8, b * 8, a as u32].map(i128::from)
    }

    fn color(&self, psm: u32, raw: u32, t0: u64, texa: u64) -> [i128; 4] {
        let ch = |v: u32| [v & 0xFF, (v >> 8) & 0xFF, (v >> 16) & 0xFF, v >> 24].map(i128::from);
        match psm {
            CT32 | Z32 => ch(raw),
            CT24 | Z24 => {
                let rgb = raw & 0xFF_FFFF;
                let a = if bitsof(texa, 15, 1) == 1 && rgb == 0 { 0 } else { bitsof(texa, 0, 8) as u32 };
                ch(rgb | a << 24)
            }
            CT16 | CT16S | Z16 | Z16S => Self::from16(raw, texa),
            _ => {
                let offset = bitsof(t0, 56, 5) as u32 * 16;
                if matches!(bitsof(t0, 51, 4), 0x2 | 0xA) {
                    Self::from16(u32::from(self.places[((offset + raw) % 512) as usize]), texa)
                } else {
                    let k = ((offset + raw) % 256) as usize;
                    ch(u32::from(self.places[k]) | u32::from(self.places[k + 256]) << 16)
                }
            }
        }
    }

    /// The pixel's colour after texture mapping (when TME) and fog (when FGE), from the fragment's colour `c`.
    pub fn shade(&self, regs: &[u64], attrs: u64, mem: &m16::Gs, frag: &Frag, c: [i128; 4]) -> [i128; 4] {
        let ctx = bitsof(attrs, 9, 1) as usize;
        let mut out = c;
        if bitsof(attrs, 4, 1) == 1 {
            let t = self.texture(regs, ctx, bitsof(attrs, 8, 1) == 1, mem, frag);
            out = function(regs[0x06 + ctx], t, c);
        }
        if bitsof(attrs, 5, 1) == 1 {
            let col = regs[0x3D];
            let f = frag_fog(frag);
            for i in 0..3 {
                let fc = bitsof(col, 8 * i as u32, 8) as i128;
                out[i] = ((f * out[i]) >> 8) + (((0xFF - f) * fc) >> 8);
            }
        }
        out
    }

    fn texture(&self, regs: &[u64], ctx: usize, uv: bool, mem: &m16::Gs, frag: &Frag) -> [i128; 4] {
        let (t0, t1, clamp) = (regs[0x06 + ctx], regs[0x14 + ctx], regs[0x08 + ctx]);
        let (tw, th) = (bitsof(t0, 26, 4).min(10) as u32, bitsof(t0, 30, 4).min(10) as u32);
        let total = |w: [i128; 3]| w[0] + w[1] + w[2];
        let (u, v) = if uv {
            let at = |c: [i128; 3], w: [i128; 3]| (c[0] * w[0] + c[1] * w[1] + c[2] * w[2]).div_euclid(total(w));
            (at(frag.v.map(|v| bitsof(v.uv, 0, 14) as i128), frag.wu), at(frag.v.map(|v| bitsof(v.uv, 16, 14) as i128), frag.wv))
        } else {
            (ratio(frag.v.map(|v| v.st as u32), frag.q, frag.wu, tw + 4), ratio(frag.v.map(|v| (v.st >> 32) as u32), frag.q, frag.wv, th + 4))
        };
        assert!(u.abs() < 1 << 40 && v.abs() < 1 << 40, "the programs keep their coordinates in range");
        // §3.4.12: LOD = (log2(1/|Q|) << L) + K, or K; Q interpolated only with STQ.
        let k = ((bitsof(t1, 32, 12) as i64) << 52 >> 52) as i128;
        let l = bitsof(t1, 19, 2) as u32;
        let lod = if bitsof(t1, 0, 1) == 1 {
            k
        } else {
            let lg = if uv { log16([frag.last.q, 0, 0], [1, 0, 0]) } else { log16(frag.q, frag.wu) };
            lg.map_or(1 << 30, |g| (g << l) + k)
        };
        let mxl = bitsof(t1, 2, 3).min(6) as u32;
        let mmin = bitsof(t1, 6, 3);
        let sample = |n: u32, bilinear: bool| self.level(regs, ctx, mem, n, u, v, bilinear, (tw, th), clamp);
        if lod <= 0 {
            return sample(0, bitsof(t1, 5, 1) == 1);
        }
        let m = (lod / 16) as u32;
        let frac = lod % 16;
        match mmin {
            2 | 4 => sample(if frac >= 8 { m + 1 } else { m }.min(mxl), mmin == 4),
            3 | 5 => {
                let a = sample(m.min(mxl), mmin == 5);
                let b = sample((m + 1).min(mxl), mmin == 5);
                [0, 1, 2, 3].map(|i| (a[i] * (16 - frac) + b[i] * frac) / 16)
            }
            _ => sample(0, mmin % 2 == 1),
        }
    }

    /// One level's colour (§3.4.8, §3.4.11): coordinates divided by 2^n, the wrap of §3.4.5 on each texel.
    #[allow(clippy::too_many_arguments)]
    fn level(&self, regs: &[u64], ctx: usize, mem: &m16::Gs, n: u32, u: i128, v: i128, bilinear: bool, (tw, th): (u32, u32), clamp: u64) -> [i128; 4] {
        let t0 = regs[0x06 + ctx];
        let psm = bitsof(t0, 20, 6) as u32;
        let (tbp, tbw) = if n == 0 {
            (bitsof(t0, 0, 14), bitsof(t0, 14, 6))
        } else {
            let r = regs[if n <= 3 { 0x34 } else { 0x36 } + ctx];
            let i = (n - 1) % 3;
            (bitsof(r, 20 * i, 14), bitsof(r, 20 * i + 14, 6))
        };
        let texa = regs[0x3B];
        let wrap = |c: i128, size_log: u32, mode: u64, lo: u64, hi: u64| -> u32 {
            let size = 1i128 << size_log.saturating_sub(n);
            let (lo, hi) = (i128::from(lo >> n), i128::from(hi >> n));
            (match mode {
                0 => c.rem_euclid(size),
                1 => c.max(0).min(size - 1),
                2 => {
                    if c < lo {
                        lo
                    } else if c > hi {
                        hi
                    } else {
                        c
                    }
                }
                _ => (c & lo) | hi,
            }) as u32
        };
        let fetch = |iu: i128, iv: i128| {
            let x = wrap(iu, tw, bitsof(clamp, 0, 2), bitsof(clamp, 4, 10), bitsof(clamp, 14, 10));
            let y = wrap(iv, th, bitsof(clamp, 2, 2), bitsof(clamp, 24, 10), bitsof(clamp, 34, 10));
            if !m16::FORMATS.contains(&psm) {
                return [0; 4];
            }
            self.color(psm, mem.get(psm, tbp as u32, tbw as u32, x, y), t0, texa)
        };
        let (un, vn) = (u.div_euclid(1 << n), v.div_euclid(1 << n));
        if !bilinear {
            return fetch(un.div_euclid(16), vn.div_euclid(16));
        }
        let (iu0, iv0) = ((un - 8).div_euclid(16), (vn - 8).div_euclid(16));
        let (a, b) = ((un - 8).rem_euclid(16), (vn - 8).rem_euclid(16));
        let (ca, cb, cc, cd) = (fetch(iu0, iv0), fetch(iu0 + 1, iv0), fetch(iu0, iv0 + 1), fetch(iu0 + 1, iv0 + 1));
        [0, 1, 2, 3].map(|i| ((16 - a) * (16 - b) * ca[i] + a * (16 - b) * cb[i] + (16 - a) * b * cc[i] + a * b * cd[i]) / 256)
    }
}

/// floor(2^scale × Σ num w / Σ q w), the floats aligned to one exponent.
fn ratio(num: [u32; 3], q: [u32; 3], w: [i128; 3], scale: u32) -> i128 {
    let all: Vec<(i128, i32)> = num.iter().chain(q.iter()).map(|&b| decode(b)).collect();
    let low = all.iter().filter(|(m, _)| *m != 0).map(|(_, e)| *e).min().unwrap_or(0);
    let term = |(m, e): (i128, i32), w: i128| m.checked_shl((e - low) as u32).and_then(|v| v.checked_mul(w)).expect("the programs keep S, T and Q close in exponent");
    let n: i128 = (0..3).map(|i| term(all[i], w[i])).sum();
    let d: i128 = (0..3).map(|i| term(all[3 + i], w[i])).sum();
    if n == 0 || d == 0 {
        return 0;
    }
    let (n, d) = if d < 0 { (-n, -d) } else { (n, d) };
    n.checked_mul(1 << scale).expect("in range").div_euclid(d)
}

/// floor(16 log2(1/|Q|)) for Q = Σ q w / Σ w, or None for Q = 0.
fn log16(q: [u32; 3], w: [i128; 3]) -> Option<i128> {
    let d: Vec<(i128, i32)> = q.iter().map(|&b| decode(b)).collect();
    let low = d.iter().filter(|(m, _)| *m != 0).map(|(_, e)| *e).min().unwrap_or(0);
    let den: i128 = (0..3).map(|i| (d[i].0 << (d[i].1 - low)) * w[i]).sum::<i128>().abs();
    let tot: i128 = w.iter().sum();
    if den == 0 {
        return None;
    }
    // An exact power of two when the odd parts agree.
    let (tz_n, tz_d) = (tot.trailing_zeros(), den.trailing_zeros());
    if tot >> tz_n == den >> tz_d {
        return Some(16 * (i128::from(tz_n) - i128::from(tz_d) - i128::from(low)));
    }
    let x = 16.0 * ((tot as f64).log2() - (den as f64).log2() - f64::from(low));
    assert!((x - x.round()).abs() > 1e-6, "log2 too near a sixteenth to decide in double precision: {x}");
    Some(x.floor() as i128)
}

/// The fog coefficient at the pixel: the vertices' F by the weights, floored, held to its range.
fn frag_fog(frag: &Frag) -> i128 {
    let t: i128 = frag.wu.iter().sum();
    ((frag.v[0].f * frag.wu[0] + frag.v[1].f * frag.wu[1] + frag.v[2].f * frag.wu[2]).div_euclid(t)).clamp(0, 255)
}

/// §3.4.9's table, A * B = (A × B) >> 7, the result held to 0 to FFh; TCC RGB keeps the fragment's alpha (D-240).
fn function(t0: u64, t: [i128; 4], f: [i128; 4]) -> [i128; 4] {
    let tcc = bitsof(t0, 34, 1) == 1;
    let star = |a: i128, b: i128| (a * b) >> 7;
    let sat = |v: i128| v.clamp(0, 255);
    let mut out = [0; 4];
    match bitsof(t0, 35, 2) {
        0 => {
            for i in 0..3 {
                out[i] = sat(star(t[i], f[i]));
            }
            out[3] = if tcc { sat(star(t[3], f[3])) } else { f[3] };
        }
        1 => {
            out[..3].copy_from_slice(&t[..3]);
            out[3] = if tcc { t[3] } else { f[3] };
        }
        tfx => {
            for i in 0..3 {
                out[i] = sat(star(t[i], f[i]) + f[3]);
            }
            out[3] = match (tcc, tfx) {
                (false, _) => f[3],
                (true, 2) => sat(t[3] + f[3]),
                _ => t[3],
            };
        }
    }
    out
}

/// §3.4.11's automatic base pointers for levels 1 to 3 (D-246): each level's bytes over 256, rounded up, after the
/// last; its width the level's texels over 64, at least 1.
fn auto_levels(t0: u64) -> u64 {
    let psm = bitsof(t0, 20, 6) as u32;
    if !m16::FORMATS.contains(&psm) {
        return 0;
    }
    let (mut w, mut h) = (1u64 << bitsof(t0, 26, 4).min(10), 1u64 << bitsof(t0, 30, 4).min(10));
    let mut tbp = bitsof(t0, 0, 14);
    let mut r = 0;
    for level in 0..3 {
        let bytes = w * h * u64::from(bits(psm)) / 8;
        tbp = (tbp + bytes.div_ceil(256)) % (1 << 14);
        w = (w / 2).max(1);
        h = (h / 2).max(1);
        r |= (tbp | std::cmp::max(1, w / 64) << 14) << (20 * level);
    }
    r
}
