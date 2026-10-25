//! The IPU's post-processing (EE User's Manual §8.6): colour space conversion from YCbCr to RGBA with the alpha the
//! thresholds of SETTH give, the 4 × 4 ordered dither and truncation to RGB16, and vector quantisation to a 4-bit index
//! against the CLUT of SETVQ; and the macroblock layouts of RAW8, RGB32, RGB16 and INDX4 (§8.2). Each step is the
//! manual's in its order and precision; the readings it leaves open are Sapphire_Native.md §34.6.

/// The dither's coefficients D in halves, by row y mod 4 and column x mod 4 (§8.6.2's table).
pub const DITHER: [[i32; 4]; 4] = [[-8, 0, -6, 2], [4, -4, 6, -2], [-5, 3, -7, 1], [7, -1, 5, -3]];

/// An RGBA pixel as RGB32 holds it: R in bits 7..0, G 15..8, B 23..16, A 31..24 (§8.2.2).
pub fn rgb32(r: u8, g: u8, b: u8, a: u8) -> u32 {
    u32::from(r) | u32::from(g) << 8 | u32::from(b) << 16 | u32::from(a) << 24
}

/// §8.6.1, steps 1 to 10: Y, Cb and Cr to RGBA with the alpha of thresholds `th0` and `th1`.
pub fn convert(y: u8, cb: u8, cr: u8, th0: u32, th1: u32) -> u32 {
    let cr1 = i32::from(cr) - 128;
    let cb1 = i32::from(cb) - 128;
    let cr4 = (0x0CC * cr1) >> 6;
    let cr5 = (0x068 * cr1) >> 6;
    let cb4 = (0x032 * cb1) >> 6;
    let cb5 = (0x102 * cb1) >> 6;
    let y3 = (0x095 * (i32::from(y) - 16)) >> 6;
    let round = |v: i32| ((v + 1) >> 1).clamp(0, 255) as u32;
    let (r, g, b) = (round(y3 + cr4), round(y3 - cb4 - cr5), round(y3 + cb5));
    if r < th0 && g < th0 && b < th0 {
        0
    } else {
        let a = if r < th1 && g < th1 && b < th1 { 0x40 } else { 0x80 };
        r | g << 8 | b << 16 | a << 24
    }
}

/// §8.6.1, step 11: IDEC's SGN, each colour's highest bit reversed.
pub fn signed(rgba: u32) -> u32 {
    rgba ^ 0x0080_8080
}

/// §8.6.2: RGB32 to RGB16 at pixel (x, y) of its macroblock, dithered or truncated; A1 is A8 = 40h.
pub fn rgb16(rgba: u32, x: usize, y: usize, dither: bool) -> u16 {
    let d = if dither { DITHER[y & 3][x & 3] } else { 0 };
    let five = |c: u32| -> u16 { (((2 * (c & 0xFF) as i32 + d) >> 1).clamp(0, 255) >> 3) as u16 };
    let a = u16::from(rgba >> 24 == 0x40);
    five(rgba) | five(rgba >> 8) << 5 | five(rgba >> 16) << 10 | a << 15
}

/// §8.6.3: the index of the CLUT entry nearest an RGB16 pixel by the sum of squared differences, the first such.
pub fn vq(p: u16, clut: &[u16; 16]) -> u8 {
    let c = |v: u16, s: u32| i32::from(v >> s & 0x1F);
    let mut best = (i32::MAX, 0u8);
    for (i, &e) in clut.iter().enumerate() {
        let d = (0..3).map(|k| (c(p, 5 * k) - c(e, 5 * k)).pow(2)).sum::<i32>();
        if d < best.0 {
            best = (d, i as u8);
        }
    }
    best.1
}

/// A macroblock of 256 RGBA pixels, row by row.
pub type Pixels = [u32; 256];

/// RAW8's macroblock (§8.2.4, 384 bytes: Y's 16 rows, then Cb and Cr 8 × 8) converted pixel by pixel, each chroma
/// sample serving the 2 × 2 luminance samples it lies on.
pub fn convert_raw8(raw: &[u8; 384], th0: u32, th1: u32, out: &mut Pixels) {
    for y in 0..16 {
        for x in 0..16 {
            let c = 8 * (y / 2) + x / 2;
            out[16 * y + x] = convert(raw[16 * y + x], raw[256 + c], raw[320 + c], th0, th1);
        }
    }
}

/// The qwords of a macroblock in RGB32 (64) or RGB16 (32, dithered or not), in the order §8.2.2 and §8.2.3 give.
pub fn pack(px: &Pixels, rgb16_out: bool, dither: bool, out: &mut [u128; 64]) -> usize {
    if !rgb16_out {
        for (q, p) in out.iter_mut().zip(px.chunks_exact(4)) {
            *q = p.iter().rev().fold(0u128, |a, &w| a << 32 | u128::from(w));
        }
        return 64;
    }
    for (i, q) in out.iter_mut().take(32).enumerate() {
        *q = (0..8).rev().fold(0u128, |a, k| {
            let n = 8 * i + k;
            a << 16 | u128::from(rgb16(px[n], n % 16, n / 16, dither))
        });
    }
    32
}

/// A macroblock in INDX4 (§8.2.6, 8 qwords, the first pixel in the low nibble), each pixel taken to RGB16 and
/// quantised.
pub fn pack_indx4(px: &Pixels, dither: bool, clut: &[u16; 16], out: &mut [u128; 64]) -> usize {
    for (i, q) in out.iter_mut().take(8).enumerate() {
        *q = (0..32).rev().fold(0u128, |a, k| {
            let n = 32 * i + k;
            a << 4 | u128::from(vq(rgb16(px[n], n % 16, n / 16, dither), clut))
        });
    }
    8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grey_and_the_primaries_follow_the_manuals_steps() {
        assert_eq!(convert(16, 128, 128, 0, 0), rgb32(0, 0, 0, 0x80));
        assert_eq!(convert(235, 128, 128, 0, 0), rgb32(255, 255, 255, 0x80), "(219 × 149) >> 6 = 509 halves, 255 rounded up");
        assert_eq!(convert(128, 128, 128, 0, 0), rgb32(130, 130, 130, 0x80));
        assert_eq!(convert(81, 90, 240, 0, 0) & 0xFF, 254, "(149 × 65 >> 6) + (204 × 112 >> 6) = 508 halves");
        assert_eq!(convert(20, 128, 128, 10, 0), 0, "below TH0: every channel and alpha 0");
        assert_eq!(convert(20, 128, 128, 0, 10) >> 24, 0x40, "below TH1: translucent");
    }

    #[test]
    fn the_dither_adds_halves_and_truncates() {
        assert_eq!(rgb16(rgb32(8, 8, 8, 0x80), 0, 0, true), 0, "8 − 4 = 4, five bits 0");
        assert_eq!(rgb16(rgb32(8, 8, 8, 0x80), 2, 3, true), 0x0421, "8 + 2.5, floored to 10, five bits 1");
        assert_eq!(rgb16(rgb32(255, 0, 0, 0x40), 0, 3, true), 0x801F, "clipped at 255; A1 for alpha 40h");
    }
}
