//! Texture mapping and fog (GS User's Manual §2.6, §2.7, §3.4, §3.5; TEX0, TEX1, TEX2, CLAMP, TEXA, TEXCLUT, MIPTBP1,
//! MIPTBP2, FOG and FOGCOL's pages): the CLUT's temporary buffer as the GS's own state, loaded from local memory when
//! TEX0 or TEX2 is written as CLD says; the context's texture registers read at the drawing kick (`Sampler`); a
//! primitive's texture coordinates interpolated at each pixel exactly and taken to sixteenths of a texel (`Setup`,
//! through `exact`); the LOD in K's fixed point; wrapping and clamping on whole texels; point, bilinear and the
//! mipmapping filters; format conversion with TEXA; the texture function; and fog. Sapphire_Native.md §31.

use super::draw::Vertex;
use super::exact::{self, Float, Sum};
use super::local::{self, Local, Memory, Units};
use super::psm::Psm;

const fn field(v: u64, lo: u32, n: u32) -> u32 {
    ((v >> lo) & ((1u64 << n) - 1)) as u32
}

/// The bits of TEX0 that TEX2 writes (TEX2's page: PSM, CBP, CPSM, CSM, CSA and CLD).
pub const TEX2_BITS: u64 = 0x3F << 20 | !((1u64 << 37) - 1);

/// CPSM's formats (TEX0's page); a code the page does not list reads as PSMCT32 [A, Sapphire_Disputes.md D-244].
const fn clut_format(code: u32) -> Psm {
    match code {
        0x2 => Psm::Ct16,
        0xA => Psm::Ct16s,
        _ => Psm::Ct32,
    }
}

/// The CLUT's temporary buffer (§3.4.7): 512 entries of 16 bits; a 32-bit CLUT entry keeps its low half in the first
/// 256 and its high half in the second, so 16-bit CLUTs loaded at CSA 16 to 31 overwrite the upper halves of 32-bit
/// ones. CBP0 and CBP1 are the GS's internal registers CLD compares.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clut {
    pub entries: [u16; 512],
    pub cbp: [u32; 2],
}

impl Default for Clut {
    fn default() -> Clut {
        Clut { entries: [0; 512], cbp: [0; 2] }
    }
}

impl Clut {
    /// TEX0, as written or as TEX2 has changed it, takes effect (§3.4.7's "Load Operation"): CLD decides whether the
    /// CLUT is loaded from CBP, in CSM1's arrangement at the buffer's start or CSM2's row at TEXCLUT's offset. Only an
    /// indexed PSM has a CLUT of a known size to load [A, D-243]. Whether it was loaded.
    /// The rectangle a load by `tex0` and `texclut` would read, as (format, base, width, xs, ys): what the device path
    /// reads back first (Sapphire_Gs.md §4.5); none for a format with no CLUT.
    pub fn source(tex0: u64, texclut: u64) -> Option<(Psm, u32, u32, (u32, u32), (u32, u32))> {
        let n = match Psm::from_code(field(tex0, 20, 6)) {
            Some(Psm::T8 | Psm::T8h) => 256,
            Some(Psm::T4 | Psm::T4hl | Psm::T4hh) => 16,
            _ => return None,
        };
        let (cpsm, cbp) = (clut_format(field(tex0, 51, 4)), field(tex0, 37, 14));
        if tex0 >> 55 & 1 != 0 {
            let (cbw, cou, cov) = (field(texclut, 0, 6), field(texclut, 6, 6) * 16, field(texclut, 12, 10));
            return Some((cpsm, cbp, cbw, ((cou) & 0x7FF, (cou + n - 1).min(0x7FF)), (cov, cov)));
        }
        Some((cpsm, cbp, 1, (0, if n == 16 { 7 } else { 15 }), (0, if n == 16 { 1 } else { 15 })))
    }

    pub fn load(&mut self, tex0: u64, texclut: u64, local: &mut impl Memory) -> bool {
        self.load_by(tex0, texclut, |psm, bp, bw, x, y| local.read(psm, bp, bw, x, y))
    }

    /// `load` from the memory itself, a CSM1 CLUT's entries found in its first page's table (Sapphire_Gs.md §6.1).
    pub fn load_local(&mut self, tex0: u64, texclut: u64, local: &Local) -> bool {
        self.load_by(tex0, texclut, |psm, bp, bw, x, y| {
            if bw != 1 || x >= 16 || y >= 16 {
                return local.read(psm, bp, bw, x, y);
            }
            let e = u32::from(local::page_table(psm)[(y * 64 + x) as usize]);
            let word = local.word(((bp * 64 + (e & 0x7FF)) as usize) & (local::WORDS - 1));
            if psm == Psm::Ct32 { word } else { word >> (16 * (e >> 11)) & 0xFFFF }
        })
    }

    fn load_by(&mut self, tex0: u64, texclut: u64, mut read: impl FnMut(Psm, u32, u32, u32, u32) -> u32) -> bool {
        let n = match Psm::from_code(field(tex0, 20, 6)) {
            Some(Psm::T8 | Psm::T8h) => 256,
            Some(Psm::T4 | Psm::T4hl | Psm::T4hh) => 16,
            _ => return false,
        };
        let cbp = field(tex0, 37, 14);
        let slot = match field(tex0, 61, 3) {
            1 => None,
            2 => Some(0),
            3 => Some(1),
            4 if self.cbp[0] != cbp => Some(0),
            5 if self.cbp[1] != cbp => Some(1),
            _ => return false,
        };
        if let Some(s) = slot {
            self.cbp[s] = cbp;
        }
        let cpsm = clut_format(field(tex0, 51, 4));
        let csa = field(tex0, 56, 5) * 16;
        let csm2 = tex0 >> 55 & 1 != 0;
        let (cbw, cou, cov) = (field(texclut, 0, 6), field(texclut, 6, 6) * 16, field(texclut, 12, 10));
        for i in 0..n {
            // §2.7.3's figures: CSM1 as 8×2 or 16×16 (pairs of rows of eight, the second eight of each 16 indices to
            // the right), CSM2 one row; CSA's offset applies in both (TEX0's page asks for 0 in CSM2).
            let (x, y, bw) = if csm2 {
                (cou + i, cov, cbw)
            } else if n == 16 {
                (i & 7, i >> 3, 1)
            } else {
                (i & 7 | (i >> 4 & 1) << 3, (i >> 3 & 1) | (i >> 5) << 1, 1)
            };
            let v = read(cpsm, cbp, bw, x, y);
            if cpsm == Psm::Ct32 {
                let k = ((csa + i) & 255) as usize;
                self.entries[k] = v as u16;
                self.entries[256 + k] = (v >> 16) as u16;
            } else {
                self.entries[((csa + i) & 511) as usize] = v as u16;
            }
        }
        true
    }

    /// The entry an index selects, in the CPSM in force at the drawing.
    fn entry(&self, cpsm32: bool, csa: u32, index: u32) -> u32 {
        if cpsm32 {
            let k = ((csa + index) & 255) as usize;
            u32::from(self.entries[k]) | u32::from(self.entries[256 + k]) << 16
        } else {
            u32::from(self.entries[((csa + index) & 511) as usize])
        }
    }
}

/// MIPTBP1 as the automatic calculation sets it when TEX0 is written with TEX1's MTBA set (§3.4.11): levels 1 to 3 each
/// after the one before, a level taking its texels' bits rounded up to whole blocks, its width the texture's halved
/// in units of 64, at least 1 [A, D-246].
pub fn auto_miptbp1(tex0: u64) -> u64 {
    let Some(psm) = Psm::from_code(field(tex0, 20, 6)) else { return 0 };
    let (mut w, mut h) = (1u64 << field(tex0, 26, 4).min(10), 1u64 << field(tex0, 30, 4).min(10));
    let mut tbp = u64::from(field(tex0, 0, 14));
    let mut out = 0;
    for n in 0..3 {
        tbp = (tbp + (w * h * u64::from(psm.bits())).div_ceil(2048)) & 0x3FFF;
        (w, h) = ((w / 2).max(1), (h / 2).max(1));
        out |= (tbp | (w / 64).max(1) << 14) << (20 * n);
    }
    out
}

/// A wrap mode and its parameters for one axis (CLAMP's page).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Wrap {
    mode: u32,
    min: u32,
    max: u32,
    log2: u32,
}

impl Wrap {
    /// §3.4.5 on a whole texel at level `n`: REPEAT modulo the level's size, CLAMP to it, REGION_CLAMP to MIN and then
    /// to MAX, the formulas in their order (so MAX wins when MIN exceeds it), REGION_REPEAT by MSK and FIX; the region's
    /// bounds and masks shifted right by the level (Sapphire_Disputes.md D-245).
    fn apply(&self, i: i64, n: u32) -> u32 {
        let size = (1i64 << self.log2.saturating_sub(n)).max(1);
        let (min, max) = (i64::from(self.min >> n), i64::from(self.max >> n));
        (match self.mode {
            0 => i & (size - 1),
            1 => i.clamp(0, size - 1),
            2 => i.max(min).min(max),
            _ => (i & min) | max,
        }) as u32
    }
}

/// The context's texture registers at a drawing kick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sampler {
    psm: Option<Psm>,
    /// Each level's base pointer in blocks and width in units of 64 texels.
    base: [(u32, u32); 7],
    wrap: [Wrap; 2],
    rgba: bool,
    tfx: u32,
    cpsm32: bool,
    csa: u32,
    ta: [u32; 2],
    aem: bool,
    lcm: bool,
    mxl: u32,
    mmag: bool,
    mmin: u32,
    l: u32,
    /// K in sixteenths, signed (int1:7:4).
    k: i32,
    uv: bool,
}

impl Sampler {
    pub fn new([tex0, tex1, clamp, mip1, mip2, texa]: [u64; 6], uv: bool) -> Sampler {
        let tw = field(tex0, 26, 4).min(10);
        let th = field(tex0, 30, 4).min(10);
        let mut base = [(field(tex0, 0, 14), field(tex0, 14, 6)); 7];
        for n in 0..3 {
            base[n + 1] = (field(mip1, 20 * n as u32, 14), field(mip1, 20 * n as u32 + 14, 6));
            base[n + 4] = (field(mip2, 20 * n as u32, 14), field(mip2, 20 * n as u32 + 14, 6));
        }
        Sampler {
            psm: Psm::from_code(field(tex0, 20, 6)),
            base,
            wrap: [
                Wrap { mode: field(clamp, 0, 2), min: field(clamp, 4, 10), max: field(clamp, 14, 10), log2: tw },
                Wrap { mode: field(clamp, 2, 2), min: field(clamp, 24, 10), max: field(clamp, 34, 10), log2: th },
            ],
            rgba: tex0 >> 34 & 1 != 0,
            tfx: field(tex0, 35, 2),
            cpsm32: clut_format(field(tex0, 51, 4)) == Psm::Ct32,
            csa: field(tex0, 56, 5) * 16,
            ta: [field(texa, 0, 8), field(texa, 32, 8)],
            aem: texa >> 15 & 1 != 0,
            lcm: tex1 & 1 != 0,
            mxl: field(tex1, 2, 3).min(6),
            mmag: tex1 >> 5 & 1 != 0,
            mmin: field(tex1, 6, 3),
            l: field(tex1, 19, 2),
            k: ((field(tex1, 32, 12) << 20) as i32) >> 20,
            uv,
        }
    }

    /// RGBA16 to RGBA32 (§3.4.6): each 5-bit colour shifted up by three, alpha from TEXA by the A bit and AEM.
    fn expand16(&self, v: u32) -> [u32; 4] {
        let a = if v >> 15 != 0 {
            self.ta[1]
        } else if self.aem && v & 0x7FFF == 0 {
            0
        } else {
            self.ta[0]
        };
        [(v & 0x1F) << 3, (v >> 5 & 0x1F) << 3, (v >> 10 & 0x1F) << 3, a]
    }

    /// The texel at whole coordinates of level `n` (already wrapped), converted to RGBA32 (§3.4.6). A Z format reads as
    /// its colour twin in its own layout [A].
    fn texel(&self, local: &Local, clut: &Clut, n: usize, x: u32, y: u32) -> [u32; 4] {
        let Some(psm) = self.psm else { return [0; 4] };
        let (bp, bw) = self.base[n];
        let raw = local.read(psm, bp, bw, x, y);
        let c = match psm {
            Psm::T8 | Psm::T8h | Psm::T4 | Psm::T4hl | Psm::T4hh => {
                let e = clut.entry(self.cpsm32, self.csa, raw);
                if self.cpsm32 {
                    return [e & 0xFF, e >> 8 & 0xFF, e >> 16 & 0xFF, e >> 24];
                }
                return self.expand16(e);
            }
            Psm::Ct16 | Psm::Ct16s | Psm::Z16 | Psm::Z16s => return self.expand16(raw),
            _ => raw,
        };
        let a = match psm {
            Psm::Ct24 | Psm::Z24 if self.aem && c & 0xFF_FFFF == 0 => 0,
            Psm::Ct24 | Psm::Z24 => self.ta[0],
            _ => c >> 24,
        };
        [c & 0xFF, c >> 8 & 0xFF, c >> 16 & 0xFF, a]
    }

    fn at(&self, local: &Local, clut: &Clut, n: u32, iu: i64, iv: i64) -> [u32; 4] {
        self.texel(local, clut, n as usize, self.wrap[0].apply(iu, n), self.wrap[1].apply(iv, n))
    }

    /// One level's colour at (`u`, `v`) in sixteenths of a level-0 texel (§3.4.8): point sampling takes the texel the
    /// coordinates fall in, bilinear the four about (u - 0.5, v - 0.5) weighted by its fractions in sixteenths, the sum
    /// rounded down once (Sapphire_Disputes.md D-249).
    fn level(&self, local: &Local, clut: &Clut, n: u32, u: i64, v: i64, linear: bool) -> [u32; 4] {
        let (u, v) = (u >> n, v >> n);
        if !linear {
            return self.at(local, clut, n, u >> 4, v >> 4);
        }
        let (u, v) = (u - 8, v - 8);
        let (iu, iv, a, b) = (u >> 4, v >> 4, (u & 15) as u32, (v & 15) as u32);
        let t = [self.at(local, clut, n, iu, iv), self.at(local, clut, n, iu + 1, iv), self.at(local, clut, n, iu, iv + 1), self.at(local, clut, n, iu + 1, iv + 1)];
        let w = [(16 - a) * (16 - b), a * (16 - b), (16 - a) * b, a * b];
        std::array::from_fn(|c| (0..4).map(|i| t[i][c] * w[i]).sum::<u32>() >> 8)
    }

    /// The texture's colour for a pixel (§3.4.8, §3.4.12): the LOD, in sixteenths, chooses MMAG at level 0 when it is
    /// 0 or less (D-241) and MMIN otherwise; the mipmapping modes take level m or m + 1 by rounding, or both blended by
    /// the LOD's fraction in sixteenths, rounded down once (D-250), every level held to MXL.
    pub fn sample(&self, local: &Local, clut: &Clut, u: i64, v: i64, lod: i32) -> [u32; 4] {
        if lod <= 0 {
            return self.level(local, clut, 0, u, v, self.mmag);
        }
        let linear = self.mmin & 1 == 1;
        match self.mmin {
            2 | 4 => {
                let n = (((lod + 8) >> 4) as u32).min(self.mxl);
                self.level(local, clut, n, u, v, self.mmin == 4)
            }
            3 | 5 => {
                let (m, f) = (((lod >> 4) as u32).min(self.mxl), (lod & 15) as u32);
                let lo = self.level(local, clut, m, u, v, self.mmin == 5);
                let hi = self.level(local, clut, (m + 1).min(self.mxl), u, v, self.mmin == 5);
                std::array::from_fn(|c| (lo[c] * (16 - f) + hi[c] * f) >> 4)
            }
            // 0 and 1, and 6 and 7, which TEX1's page leaves undefined, as level 0 [A].
            _ => self.level(local, clut, 0, u, v, linear),
        }
    }

    /// The texture function (§3.4.9): A * B is (A × B) >> 7 and every result is held to 0 to FFh. With TCC RGB the
    /// fragment's alpha is kept in every function, DECAL included (D-240).
    pub fn function(&self, t: [u32; 4], f: [u32; 4]) -> [u32; 4] {
        let mul = |a: u32, b: u32| (a * b) >> 7;
        let rgb: [u32; 3] = match self.tfx {
            0 => std::array::from_fn(|c| mul(t[c], f[c]).min(255)),
            1 => [t[0], t[1], t[2]],
            _ => std::array::from_fn(|c| (mul(t[c], f[c]) + f[3]).min(255)),
        };
        let a = match (self.rgba, self.tfx) {
            (false, _) => f[3],
            (true, 0) => mul(t[3], f[3]).min(255),
            (true, 2) => (t[3] + f[3]).min(255),
            (true, _) => t[3],
        };
        [rgb[0], rgb[1], rgb[2], a]
    }

    pub fn uv(&self) -> bool {
        self.uv
    }

    /// The page lanes every texel the sampler can read lies in (Sapphire_Native.md §50): each level it can choose, 0 to
    /// MXL under a mipmapping MMIN and 0 otherwise, over the coordinates its wrap modes can give, [0, size) for REPEAT
    /// and CLAMP, [0, MAX] for REGION_CLAMP and [0, MIN | MAX] for REGION_REPEAT, each shifted right by the level.
    pub fn units(&self) -> Units {
        self.units_within(None)
    }

    /// The page lanes the texels a primitive can read lie in (Sapphire_Gs.md §4.3), from its vertices each with the Q it
    /// takes, for a primitive whose weights are never negative (not a line): its coordinates lie between its vertices'
    /// (a perspective quotient with every Q of one sign is a convex combination of theirs), widened by a texel for the
    /// bilinear filter and two for the exact floor; `units` where that cannot be said.
    pub fn reach(&self, v: &[(&Vertex, u32)]) -> Units {
        self.units_within(self.range(v))
    }

    /// The coordinates a primitive's texels lie within, as `reach` takes them; none where that cannot be said.
    pub fn range(&self, v: &[(&Vertex, u32)]) -> Option<((i64, i64), (i64, i64))> {
        let (tw, th) = (self.wrap[0].log2, self.wrap[1].log2);
        let span = |c: &dyn Fn(&(&Vertex, u32)) -> Option<f64>| -> Option<(i64, i64)> {
            let (mut lo, mut hi) = (f64::MAX, f64::MIN);
            for x in v {
                let t = c(x)?;
                lo = lo.min(t);
                hi = hi.max(t);
            }
            (lo.is_finite() && hi.is_finite() && lo > -1e12 && hi < 1e12).then(|| (lo.floor() as i64 - 48, hi.ceil() as i64 + 48))
        };
        let range = if self.uv {
            span(&|x| Some((x.0.uv & 0x3FFF) as f64)).zip(span(&|x| Some((x.0.uv >> 16 & 0x3FFF) as f64)))
        } else {
            let q = |x: &(&Vertex, u32)| f32::from_bits(x.1) as f64;
            let first = q(&v[0]);
            let same = first != 0.0 && v.iter().all(|x| q(x).is_finite() && q(x) * first > 0.0);
            let st = |x: &(&Vertex, u32), hi: bool, log2: u32| {
                let s = f32::from_bits(if hi { (x.0.st >> 32) as u32 } else { x.0.st as u32 }) as f64;
                Some(s / q(x) * f64::from(1u32 << (log2 + 4)))
            };
            if same { span(&|x| st(x, false, tw)).zip(span(&|x| st(x, true, th))) } else { None }
        };
        range
    }

    fn units_within(&self, range: Option<((i64, i64), (i64, i64))>) -> Units {
        let mut s = Units::default();
        self.rects(range, |psm, bp, bw, xs, ys| s.or(&local::units(psm, bp, bw, xs, ys)));
        s
    }

    /// Each level's rectangle of texels the sampler can read, as `f(psm, bp, bw, xs, ys)`: within `range` (sixteenths of
    /// a level-0 texel along U and V) where it is known, else over every coordinate its wrap modes can give.
    pub fn rects(&self, range: Option<((i64, i64), (i64, i64))>, mut f: impl FnMut(Psm, u32, u32, (u32, u32), (u32, u32))) {
        let Some(psm) = self.psm else { return };
        let top = if (2..=5).contains(&self.mmin) { self.mxl } else { 0 };
        for n in 0..=top {
            let (xs, ys) = match range {
                Some((us, vs)) => {
                    let axis = |w: &Wrap, (lo, hi): (i64, i64)| {
                        let size = (1i64 << w.log2.saturating_sub(n)).max(1);
                        let (a, b) = ((lo >> n) >> 4, (hi >> n) >> 4);
                        let (a, b) = match w.mode {
                            0 if a >= 0 && b < size => (a as u32, b as u32),
                            0 => (0, size as u32 - 1),
                            1 => (a.clamp(0, size - 1) as u32, b.clamp(0, size - 1) as u32),
                            2 => (w.apply(a, n), w.apply(b, n)),
                            _ => (0, (w.min >> n) | (w.max >> n)),
                        };
                        (a.min(b), a.max(b))
                    };
                    (axis(&self.wrap[0], us), axis(&self.wrap[1], vs))
                }
                None => {
                    let extent = |w: &Wrap| match w.mode {
                        0 | 1 => (1u32 << w.log2.saturating_sub(n)).max(1) - 1,
                        2 => w.max >> n,
                        _ => (w.min >> n) | (w.max >> n),
                    };
                    ((0, extent(&self.wrap[0])), (0, extent(&self.wrap[1])))
                }
            };
            let (bp, bw) = self.base[n as usize];
            f(psm, bp, bw, xs, ys);
        }
    }
}

/// Fog (§3.5, FOG's page): each colour (F × C + (100h - F) × FOGCOL) >> 8, so that F 0 gives FOGCOL itself, as FOG's
/// page says, which §3.5's two products shifted apart do not (Sapphire_Disputes.md D-248); alpha kept.
pub fn fog(c: [u32; 4], f: u32, col: [u32; 3]) -> [u32; 4] {
    let mix = |v: u32, k: u32| (f * v + (256 - f) * k) >> 8;
    [mix(c[0], col[0]), mix(c[1], col[1]), mix(c[2], col[2]), c[3]]
}

/// What the LOD is computed from (§3.4.12, §3.4.4's table).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Lod {
    /// LCM 1, or LCM 0 with UV, where Q is not interpolated: one value for the primitive.
    Fixed(i32),
    /// LCM 0 with STQ: from the pixel's interpolated Q.
    PerPixel,
}

/// A primitive's texture coordinates, ready to be interpolated at each pixel: S, T and Q as exact values, or U and V
/// in sixteenths (UV's int 0:10:4), and the LOD's source.
#[derive(Clone, Copy, Debug)]
pub struct Setup {
    uv: bool,
    s: [Float; 3],
    t: [Float; 3],
    q: [Float; 3],
    u: [i64; 3],
    v: [i64; 3],
    scale: (i32, i32),
    lod: Lod,
    l: u32,
    k: i32,
}

/// The LOD in K's fixed point (int1:7:4) from floor(2^(L + 4) × log2(1/|Q|)): (log2(1/|Q|) << L) + K (TEX1's page),
/// exact and then floored to sixteenths (Sapphire_Disputes.md D-242); a Q of 0 is taken as the farthest level [A].
fn lod_of(log: Option<i64>, k: i32) -> i32 {
    log.map_or(i32::MAX >> 1, |g| (g + i64::from(k)).clamp(i64::from(i32::MIN >> 1), i64::from(i32::MAX >> 1)) as i32)
}

impl Setup {
    /// From the primitive's vertices (unused ones repeated), `q` being the Q each takes (a sprite's every vertex takes
    /// the second's, [A]), and `last` the vertex of the drawing kick, whose Q gives a UV primitive's LOD [A, D-242].
    pub fn new(sampler: &Sampler, v: [&Vertex; 3], q: [u32; 3], last: &Vertex) -> Setup {
        let tw = sampler.wrap[0].log2 as i32;
        let th = sampler.wrap[1].log2 as i32;
        let q = q.map(Float::from_bits);
        let lod = if sampler.lcm {
            Lod::Fixed(sampler.k)
        } else if sampler.uv {
            let fq = Float::from_bits((last.rgbaq >> 32) as u32);
            Lod::Fixed(lod_of(exact::log2_scaled(1, &Sum::of(&[fq; 3], [1, 0, 0]), sampler.l + 4), sampler.k))
        } else {
            Lod::PerPixel
        };
        Setup {
            uv: sampler.uv,
            s: v.map(|v| Float::from_bits(v.st as u32)),
            t: v.map(|v| Float::from_bits((v.st >> 32) as u32)),
            q,
            u: v.map(|v| (v.uv & 0x3FFF) as i64),
            v: v.map(|v| (v.uv >> 16 & 0x3FFF) as i64),
            scale: (tw + 4, th + 4),
            lod,
            l: sampler.l,
            k: sampler.k,
        }
    }

    /// The coordinates in sixteenths of a texel and the LOD at a pixel whose weights for the vertices are `wu` along U
    /// and `wv` along V (the same but for a sprite), each with a positive total; every quantity exact, then floored.
    pub fn at(&self, wu: [i64; 3], wv: [i64; 3]) -> (i64, i64, i32) {
        let total = |w: [i64; 3]| w[0] + w[1] + w[2];
        let lod = |q: &Sum| match self.lod {
            Lod::Fixed(l) => l,
            Lod::PerPixel => lod_of(exact::log2_scaled(total(wu) as u64, q, self.l + 4), self.k),
        };
        if self.uv {
            let lerp = |c: &[i64; 3], w: [i64; 3]| (i128::from(c[0]) * i128::from(w[0]) + i128::from(c[1]) * i128::from(w[1]) + i128::from(c[2]) * i128::from(w[2])).div_euclid(i128::from(total(w))) as i64;
            let l = match self.lod {
                Lod::Fixed(l) => l,
                Lod::PerPixel => lod(&Sum::of(&self.q, wu)),
            };
            return (lerp(&self.u, wu), lerp(&self.v, wv), l);
        }
        // ΣQw is taken once along U and reused along V and for the LOD when the weights agree, as they do but for a sprite.
        let qu = Sum::of(&self.q, wu);
        let qv = if wv == wu { qu } else { Sum::of(&self.q, wv) };
        (exact::quotient(&Sum::of(&self.s, wu), &qu, self.scale.0), exact::quotient(&Sum::of(&self.t, wv), &qv, self.scale.1), lod(&qu))
    }
}
