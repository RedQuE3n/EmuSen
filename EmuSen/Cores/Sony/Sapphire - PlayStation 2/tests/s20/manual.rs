//! The PCRTC as the GS User's Manual's chapter 5 and §7.2 give it, transcribed apart from `src/pcrtc/` for the S20
//! tests: the timing's figures (§5.1.2's table read at 13.5 MHz, Sapphire_Native.md §29.3), the two read circuits and
//! the merge circuit, over S16's model of the memory (`s16::manual::Gs`).

use crate::s16::manual::{CT16, CT16S, CT24, CT32, Gs};
use crate::s16::ga;

/// EE cycles in `vck` ticks of 54 MHz, counted from a field's start at EE cycle `base` (294.912 / 54 = 2048 / 375).
pub fn cycles(base: u64, vck: u64) -> u64 {
    base + (vck * 2048).div_ceil(375)
}

/// NTSC: a line of 858 × 4 VCK; 263 lines a non-interlaced field, 262.5 an interlaced one; V-blank 22 lines from the
/// field's start; the V-sync 4 lines in; H-blank the line less BT.601's 720 pixels.
pub const LINE: u64 = 858 * 4;
pub const NI_LINES: u64 = 263;
pub const VBLANK: u64 = 22;
pub const VSYNC: u64 = 4;
pub const HBLANK: u64 = (858 - 720) * 4;

pub fn field_vck(interlaced: bool) -> u64 {
    if interlaced { LINE * 525 / 2 } else { LINE * NI_LINES }
}

fn reg(gs: &Gs, g: u32) -> u64 {
    gs.privileged.get(&g).copied().unwrap_or(0)
}

fn bits(v: u64, lo: u32, n: u32) -> u64 {
    v >> lo & ((1 << n) - 1)
}

/// One circuit's colour and alpha at raster `r` and VCK `v`, if it shows there (§5.2.2).
fn circuit(gs: &Gs, n: usize, r: u64, v: u64, interlaced: bool) -> Option<([u8; 3], u8)> {
    let pm = reg(gs, ga::PMODE);
    if pm >> n & 1 == 0 {
        return None;
    }
    let fb = reg(gs, if n == 0 { ga::DISPFB1 } else { ga::DISPFB2 });
    let d = reg(gs, if n == 0 { ga::DISPLAY1 } else { ga::DISPLAY2 });
    let (dx, dy, magh, magv, dw, dh) = (bits(d, 0, 12), bits(d, 12, 11), bits(d, 23, 4) + 1, bits(d, 27, 2) + 1, bits(d, 32, 12), bits(d, 44, 11));
    if v < dx || v > dx + dw || r < dy || r > dy + dh {
        return None;
    }
    let frame = interlaced && reg(gs, ga::SMODE2) & 2 != 0;
    let scan = if frame { (r - dy) / 2 } else { r - dy };
    let x = (bits(fb, 32, 11) + (v - dx) / magh) as u32;
    let y = (bits(fb, 43, 11) + scan / magv) as u32;
    let (fbp, fbw, psm) = (bits(fb, 0, 9) as u32, bits(fb, 9, 6) as u32, bits(fb, 15, 5) as u32);
    let p = |psm| gs.get(psm, fbp * 32, fbw, x, y);
    Some(match psm {
        CT32 => {
            let c = p(CT32);
            ([c as u8, (c >> 8) as u8, (c >> 16) as u8], (c >> 24) as u8)
        }
        CT24 => {
            let c = p(CT24);
            ([c as u8, (c >> 8) as u8, (c >> 16) as u8], 0x80)
        }
        CT16 | CT16S => {
            let c = p(psm);
            let five = |k: u32| ((c >> (5 * k) & 31) << 3) as u8;
            ([five(0), five(1), five(2)], if c & 0x8000 != 0 { 0x80 } else { 0 })
        }
        _ => panic!("the programs show only the colour formats"),
    })
}

/// OUT1 at raster `r`, VCK `v` (§5.2.3).
pub fn pixel(gs: &Gs, r: u64, v: u64, interlaced: bool) -> [u8; 3] {
    let pm = reg(gs, ga::PMODE);
    let bgv = reg(gs, ga::BGCOLOR);
    let bg = [bgv as u8, (bgv >> 8) as u8, (bgv >> 16) as u8];
    let out2 = circuit(gs, 1, r, v, interlaced).map_or(bg, |c| c.0);
    let rd = if pm & 0x80 != 0 { bg } else { out2 };
    match circuit(gs, 0, r, v, interlaced) {
        None => rd,
        Some((rs, a1)) => {
            let a = if pm & 0x20 != 0 { bits(pm, 8, 8) as u32 } else { (2 * a1 as u32).min(255) };
            std::array::from_fn(|i| ((rs[i] as u32 * a + rd[i] as u32 * (255 - a) + 127) / 255) as u8)
        }
    }
}

/// The picture's rectangle (Sapphire_Native.md §29.5): the union of the circuits switched on, a pixel the finest
/// circuit's VCK; left edge, pitch, width, first raster, height.
pub fn rectangle(gs: &Gs) -> (u64, u64, u32, u64, u32) {
    let pm = reg(gs, ga::PMODE);
    let (mut x0, mut x1, mut y0, mut y1, mut pitch) = (u64::MAX, 0, u64::MAX, 0, u64::MAX);
    for n in 0..2 {
        if pm >> n & 1 == 1 {
            let d = reg(gs, if n == 0 { ga::DISPLAY1 } else { ga::DISPLAY2 });
            x0 = x0.min(bits(d, 0, 12));
            x1 = x1.max(bits(d, 0, 12) + bits(d, 32, 12) + 1);
            y0 = y0.min(bits(d, 12, 11));
            y1 = y1.max(bits(d, 12, 11) + bits(d, 44, 11) + 1);
            pitch = pitch.min(bits(d, 23, 4) + 1);
        }
    }
    (x0, pitch, (x1 - x0).div_ceil(pitch) as u32, y0, (y1 - y0) as u32)
}

/// The expected picture, RGBA rows, every raster read with the registers as they stand.
pub fn picture(gs: &Gs, interlaced: bool) -> Vec<u8> {
    let (x0, pitch, w, y0, h) = rectangle(gs);
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for row in 0..h as u64 {
        for x in 0..w as u64 {
            let c = pixel(gs, y0 + row, x0 + x * pitch, interlaced);
            out.extend([c[0], c[1], c[2], 255]);
        }
    }
    out
}
