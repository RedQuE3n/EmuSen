//! The EE User's Manual's post-processing (§8.6) and data formats (§8.2), written from the manual apart from
//! `src/ipu/csc.rs`: each step in the manual's own units (the products kept in 1/128ths with their low six bits
//! cleared, the dither's coefficients as the halves the table prints), with the readings Sapphire_Native.md §34.6 gives
//! where the manual leaves one open.

#![allow(dead_code)]

/// §8.6.1: (R8, G8, B8, A8) from Y, Cb and Cr.
pub fn csc(y: u8, cb: u8, cr: u8, th0: u32, th1: u32) -> [u8; 4] {
    let cr1 = i32::from(cr) - 128;
    let cb1 = i32::from(cb) - 128;
    let cr4 = (0x0CC * cr1) & !0x3F;
    let cr5 = (0x068 * cr1) & !0x3F;
    let cb4 = (0x032 * cb1) & !0x3F;
    let cb5 = (0x102 * cb1) & !0x3F;
    let y1 = i32::from(y) - 16;
    let y3 = (0x095 * y1) & !0x3F;
    let r0 = y3 + cr4;
    let g0 = y3 - cb4 - cr5;
    let b0 = y3 + cb5;
    let r1 = (r0 + 0x40) >> 7;
    let g1 = (g0 + 0x40) >> 7;
    let b1 = (b0 + 0x40) >> 7;
    let r8 = r1.clamp(0, 255) as u32;
    let g8 = g1.clamp(0, 255) as u32;
    let b8 = b1.clamp(0, 255) as u32;
    if r8 < th0 && g8 < th0 && b8 < th0 {
        [0, 0, 0, 0]
    } else if r8 < th1 && g8 < th1 && b8 < th1 {
        [r8 as u8, g8 as u8, b8 as u8, 0x40]
    } else {
        [r8 as u8, g8 as u8, b8 as u8, 0x80]
    }
}

/// §8.6.2's table, as printed.
pub const D: [[f64; 4]; 4] = [[-4.0, 0.0, -3.0, 1.0], [2.0, -2.0, 3.0, -1.0], [-2.5, 1.5, -3.5, 0.5], [3.5, -0.5, 2.5, -1.5]];

/// §8.6.2: an RGBA pixel to (A1, R5, G5, B5) as RGB16, dithered at (x, y) of its macroblock or not.
pub fn rgb16(p: [u8; 4], x: usize, y: usize, dither: bool) -> u16 {
    let d = if dither { D[y % 4][x % 4] } else { 0.0 };
    let five = |c: u8| ((f64::from(c) + d).clamp(0.0, 255.0).floor() as u16) >> 3;
    let a1 = u16::from(p[3] == 0x40);
    a1 << 15 | five(p[2]) << 10 | five(p[1]) << 5 | five(p[0])
}

/// §8.6.3: the first CLUT entry at the least squared distance.
pub fn vq(p: u16, clut: &[u16; 16]) -> u8 {
    let parts = |v: u16| [i32::from(v & 31), i32::from(v >> 5 & 31), i32::from(v >> 10 & 31)];
    let a = parts(p);
    let mut best = 0;
    let mut least = i32::MAX;
    for i in 0..16 {
        let b = parts(clut[i]);
        let d1: i32 = (0..3).map(|k| (a[k] - b[k]).abs() * (a[k] - b[k]).abs()).sum();
        if d1 < least {
            least = d1;
            best = i;
        }
    }
    best as u8
}

/// A macroblock of RGBA pixels from RAW8 (§8.2.4): Y[16][16], Cb[8][8], Cr[8][8], each chroma sample on four pixels.
pub fn from_raw8(raw: &[u8], th0: u32, th1: u32) -> Vec<[u8; 4]> {
    (0..256).map(|i| {
        let (x, y) = (i % 16, i / 16);
        csc(raw[i], raw[256 + 8 * (y / 2) + x / 2], raw[320 + 8 * (y / 2) + x / 2], th0, th1)
    })
    .collect()
}

/// §8.2.2's 64 qwords, P00 in the lowest word of the first.
pub fn rgb32_qwords(px: &[[u8; 4]]) -> Vec<u128> {
    px.chunks(4).map(|c| c.iter().enumerate().map(|(k, p)| u128::from(u32::from_le_bytes(*p)) << (32 * k)).sum()).collect()
}

/// §8.2.3's 32 qwords.
pub fn rgb16_qwords(px: &[[u8; 4]], dither: bool) -> Vec<u128> {
    (0..32).map(|q| (0..8).map(|k| u128::from(rgb16(px[8 * q + k], (8 * q + k) % 16, (8 * q + k) / 16, dither)) << (16 * k)).sum()).collect()
}

/// §8.2.6's 8 qwords, I00 in the lowest nibble.
pub fn indx4_qwords(px: &[[u8; 4]], dither: bool, clut: &[u16; 16]) -> Vec<u128> {
    (0..8).map(|q| (0..32).map(|k| u128::from(vq(rgb16(px[32 * q + k], (32 * q + k) % 16, (32 * q + k) / 16, dither), clut)) << (4 * k)).sum()).collect()
}

/// §8.2.5's 48 qwords from the six blocks, Y in frame order: the field DCT's upper blocks hold the even lines.
pub fn raw16_qwords(blocks: &[[i16; 64]; 6], field: bool) -> Vec<u128> {
    let mut s = Vec::with_capacity(384);
    for y in 0..16 {
        for x in 0..16 {
            let b = if field { 2 * (y % 2) + x / 8 } else { 2 * (y / 8) + x / 8 };
            let row = if field { y / 2 } else { y % 8 };
            s.push(blocks[b][8 * row + x % 8]);
        }
    }
    s.extend_from_slice(&blocks[4]);
    s.extend_from_slice(&blocks[5]);
    s.chunks(8).map(|c| c.iter().enumerate().map(|(k, &v)| u128::from(v as u16) << (16 * k)).sum()).collect()
}

/// RAW8 from the six blocks of an intra macroblock, each sample saturated to [0, 255] (H.262 §7.6.8, no prediction).
pub fn raw8(blocks: &[[i16; 64]; 6], field: bool) -> Vec<u8> {
    let q = raw16_qwords(blocks, field);
    q.iter().flat_map(|w| (0..8).map(move |k| ((*w >> (16 * k)) as u16 as i16).clamp(0, 255) as u8)).collect()
}

/// IDEC's output for a macroblock: CSC, SGN, then RGB32 or RGB16 dithered or not.
pub fn idec_qwords(blocks: &[[i16; 64]; 6], field: bool, th: (u32, u32), sgn: bool, rgb16: bool, dither: bool) -> Vec<u128> {
    let mut px = from_raw8(&raw8(blocks, field), th.0, th.1);
    if sgn {
        for p in &mut px {
            p[0] ^= 0x80;
            p[1] ^= 0x80;
            p[2] ^= 0x80;
        }
    }
    if rgb16 { rgb16_qwords(&px, dither) } else { rgb32_qwords(&px) }
}
