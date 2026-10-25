//! The pixel pipeline (GS User's Manual figure 3-1, §3.7, §3.9), in the documented order: the scissoring test and
//! SCANMSK; texture mapping, fog and antialiasing (S18, S19); the alpha test and the
//! destination alpha test (S19); the depth test; alpha blending, dithering, colour clamp and alpha correction (S19);
//! format conversion, FBMSK and ZMSK, and the writes through `local`. Sapphire_Native.md §27.

use super::draw::Environment;
use super::local::Local;
use super::psm::Psm;

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

/// One pixel of a primitive at window coordinates (`x`, `y`), with its Z and its colour's four 8-bit channels.
pub fn draw(env: &Environment, local: &mut Local, x: i32, y: i32, z: u32, rgba: [u32; 4]) {
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
    let (x, y) = (x as u32, y as u32);
    let (bw, test) = (env.frame.bw, env.test);
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
        if !env.depth.masked {
            z_write = Some((zpsm, z));
        }
    }
    let c = rgba[0] | rgba[1] << 8 | rgba[2] << 16 | rgba[3] << 24;
    let (mut value, keep) = if fpsm.bits() == 16 { (pack16(c), pack16(env.frame.mask)) } else { (c, env.frame.mask) };
    if keep != 0 {
        value = (local.read(fpsm, env.frame.bp, bw, x, y) & keep) | (value & !keep);
    }
    local.write(fpsm, env.frame.bp, bw, x, y, value);
    if let Some((zpsm, z)) = z_write {
        local.write(zpsm, env.depth.bp, bw, x, y, z);
    }
}
