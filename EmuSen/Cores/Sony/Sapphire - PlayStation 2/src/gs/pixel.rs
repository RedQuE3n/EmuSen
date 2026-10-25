//! The pixel pipeline (GS User's Manual figure 3-1, §3.6 to §3.9), in the documented order: the scissoring test and
//! SCANMSK; texture mapping and fog (`texture`, S18) and antialiasing's alpha; the alpha test with AFAIL, the
//! destination alpha test and the depth test (§3.7); alpha blending with PABE (§3.8); dithering, colour clamp and
//! alpha correction (§3.9.1 to §3.9.3); format conversion, FBMSK and ZMSK, and the writes through `local`.
//! Sapphire_Native.md §27, §31, §37.

use super::draw::Environment;
use super::local::Local;
use super::psm::Psm;
use super::texture::{self, Clut};

/// What texturing and fog take at a pixel: the texture coordinates in sixteenths of a texel, the LOD in sixteenths and
/// the fog coefficient, which `raster` interpolates only when a pixel needs them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Shade {
    pub u: i64,
    pub v: i64,
    pub lod: i32,
    pub f: u32,
}

/// A pixel's coverage when nothing is antialiased: 80h, the whole pixel (§3.6.1).
pub const FULL: u32 = 0x80;

/// RGBA32 packed into RGBA16, the upper five bits of each colour and alpha's top bit (§3.9.4); FBMSK is packed the same
/// way, its bits standing for the pixel before conversion (§3.9.5).
pub const fn pack16(c: u32) -> u32 {
    (c >> 3 & 0x1F) | (c >> 11 & 0x1F) << 5 | (c >> 19 & 0x1F) << 10 | (c >> 31) << 15
}

/// The largest Z a format holds; a pixel's Z beyond it is clamped to it (Sapphire_Disputes.md D-163).
pub const fn z_max(psm: Psm) -> u32 {
    match psm.bits() {
        32 => u32::MAX,
        b => (1 << b) - 1,
    }
}

/// A frame pixel as the blend reads it, Cd's three channels and Ad (§3.8.1): RGBA16's channels as their five bits at
/// the top of eight (Sapphire_Disputes.md D-402) and its alpha 0 or 80h; RGB24's alpha 80h. A Z format reads as its
/// colour twin.
fn destination(psm: Psm, d: u32) -> ([i32; 3], i32) {
    match psm.bits() {
        16 => ([d & 0x1F, d >> 5 & 0x1F, d >> 10 & 0x1F].map(|c| (c << 3) as i32), if d >> 15 & 1 != 0 { 0x80 } else { 0 }),
        24 => ([d & 0xFF, d >> 8 & 0xFF, d >> 16 & 0xFF].map(|c| c as i32), 0x80),
        _ => ([d & 0xFF, d >> 8 & 0xFF, d >> 16 & 0xFF].map(|c| c as i32), (d >> 24) as i32),
    }
}

/// The destination alpha test's bit (§3.7.3): bit 7 of A in RGBA32, the alpha bit in RGBA16, none in RGB24.
fn destination_alpha(psm: Psm, d: u32) -> Option<u32> {
    match psm.bits() {
        16 => Some(d >> 15 & 1),
        24 => None,
        _ => Some(d >> 31),
    }
}

/// The alpha test (§3.7.2, TEST's ATST): whether alpha `a` passes against AREF.
const fn alpha_passes(atst: u64, a: u32, aref: u32) -> bool {
    match atst {
        0 => false,
        1 => true,
        2 => a < aref,
        3 => a <= aref,
        4 => a == aref,
        5 => a >= aref,
        6 => a > aref,
        _ => a != aref,
    }
}

/// One pixel of a primitive at window coordinates (`x`, `y`), with its Z, its colour's four 8-bit channels, its
/// coverage (`FULL` unless the primitive is antialiased), and what texturing and fog take, computed only once the pixel
/// has passed scissoring.
#[allow(clippy::too_many_arguments)]
pub fn draw(env: &Environment, local: &mut Local, clut: &Clut, x: i32, y: i32, z: u32, rgba: [u32; 4], cov: u32, shade: impl FnOnce() -> Shade) {
    // The scissoring test (§3.7.1): the area's bounds pass.
    let (sx0, sx1, sy0, sy1) = env.scissor;
    if x < sx0 || x > sx1 || y < sy0 || y > sy1 {
        return;
    }
    // SCANMSK (§7.1): 10 keeps the odd rows, 11 the even; 01 is reserved and draws as 00 [A].
    if (env.scanmsk == 2 && y & 1 == 0) || (env.scanmsk == 3 && y & 1 == 1) {
        return;
    }
    let Some(fpsm) = env.frame.psm else { return };
    // Texture mapping (§3.4), then fog (§3.5), on the fragment's colour.
    let mut rgba = rgba;
    if env.texture.is_some() || env.fog.is_some() {
        let s = shade();
        if let Some(t) = &env.texture {
            rgba = t.function(t.sample(local, clut, s.u, s.v, s.lod), rgba);
        }
        if let Some(col) = env.fog {
            rgba = texture::fog(rgba, s.f, col);
        }
    }
    // Antialiasing (§3.6.1's table): the coverage replaces alpha unless ABE is set and alpha is not 80h, and the blend
    // runs whatever ABE says; Z is not written below full coverage (§3.6.2).
    let blend = &env.blend;
    let mut z_ok = true;
    if blend.antialias {
        if !blend.abe || rgba[3] == 0x80 {
            rgba[3] = cov;
        }
        z_ok = cov >= FULL;
    }
    let (x, y) = (x as u32, y as u32);
    let (bw, test) = (env.frame.bw, env.test);
    // The alpha test (§3.7.2) and AFAIL: a failing pixel goes on with only what AFAIL leaves effective; RGB_ONLY is
    // FB_ONLY in every format without RGBA32's eight bits of alpha.
    let (mut rgb_ok, mut a_ok) = (true, true);
    if test & 1 != 0 && !alpha_passes(test >> 1 & 7, rgba[3], (test >> 4) as u32 & 0xFF) {
        match test >> 12 & 3 {
            0 => return,
            1 => z_ok = false,
            2 => (rgb_ok, a_ok) = (false, false),
            _ => {
                z_ok = false;
                a_ok = fpsm.bits() != 32;
            }
        }
    }
    // The destination alpha test (§3.7.3): the frame's alpha bit must equal DATM.
    let mut old = None;
    if test >> 14 & 1 != 0 {
        let d = local.read(fpsm, env.frame.bp, bw, x, y);
        if destination_alpha(fpsm, d).is_some_and(|bit| bit != (test >> 15 & 1) as u32) {
            return;
        }
        old = Some(d);
    }
    // The depth test (§3.7.4): with ZTE 0 there is neither test nor write, as the Restriction's "Depth Test" equates it
    // with ZTE 1, ALWAYS and ZMSK 1; an undefined ZBUF format reads and writes nothing [A].
    let mut z_write = None;
    if test >> 16 & 1 != 0
        && let Some(zpsm) = env.depth.psm
    {
        let z = z.min(z_max(zpsm));
        let pass = match test >> 17 & 3 {
            0 => false,
            1 => true,
            2 => z >= local.read(zpsm, env.depth.bp, bw, x, y),
            _ => z > local.read(zpsm, env.depth.bp, bw, x, y),
        };
        if !pass {
            return;
        }
        if z_ok && !env.depth.masked {
            z_write = Some((zpsm, z));
        }
    }
    if rgb_ok || a_ok {
        let b = &env.blend;
        let blending = (b.abe || b.antialias) && (!b.pabe || rgba[3] & 0x80 != 0);
        let keep = env.frame.mask | if a_ok { 0 } else { 0xFF00_0000 };
        let old = match old {
            Some(d) => d,
            None if blending || keep != 0 => local.read(fpsm, env.frame.bp, bw, x, y),
            None => 0,
        };
        let c = output(env, fpsm, x, y, rgba, blending.then_some(old));
        let (value, keep) = if fpsm.bits() == 16 { (pack16(c), pack16(keep)) } else { (c, keep) };
        local.write(fpsm, env.frame.bp, bw, x, y, (old & keep) | (value & !keep));
    }
    if let Some((zpsm, z)) = z_write {
        local.write(zpsm, env.depth.bp, bw, x, y, z);
    }
}

/// The colour a pixel writes, RGBA32 before format conversion: alpha blending over the frame's pixel `old` when ABE
/// (or antialiasing) and PABE call for it, (A - B) × C >> 7 + D on each channel unclamped (§3.8.1); then, in the 16-bit
/// formats alone (Sapphire_Disputes.md D-403), dithering's offset (§3.9.1); then COLCLAMP's clamp to 0 to FFh or its
/// low eight bits (§3.9.2, D-404); alpha As, with FBA's bit ORed into bit 7 (§3.9.3).
fn output(env: &Environment, fpsm: Psm, x: u32, y: u32, rgba: [u32; 4], old: Option<u32>) -> u32 {
    let b = &env.blend;
    let s = [rgba[0] as i32, rgba[1] as i32, rgba[2] as i32];
    let mut c = s;
    if let Some(old) = old {
        let (d, ad) = destination(fpsm, old);
        let pick = |sel: u64, i: usize| match sel {
            0 => s[i],
            1 => d[i],
            _ => 0,
        };
        let cv = match b.alpha >> 4 & 3 {
            0 => rgba[3] as i32,
            1 => ad,
            _ => (b.alpha >> 32) as i32 & 0xFF,
        };
        c = std::array::from_fn(|i| (((pick(b.alpha & 3, i) - pick(b.alpha >> 2 & 3, i)) * cv) >> 7) + pick(b.alpha >> 6 & 3, i));
    }
    if let Some(dimx) = env.dither.filter(|_| fpsm.bits() == 16) {
        let dm = ((dimx >> (16 * (y & 3) + 4 * (x & 3))) as i32 & 7) << 29 >> 29;
        c = c.map(|v| v + dm);
    }
    let c = c.map(|v| if env.colclamp { v.clamp(0, 0xFF) as u32 } else { v as u32 & 0xFF });
    let a = rgba[3] | if env.fba { 0x80 } else { 0 };
    c[0] | c[1] << 8 | c[2] << 16 | a << 24
}
