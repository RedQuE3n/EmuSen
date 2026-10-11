//! The MDEC's arithmetic on one 8x8 block (psx-spx "MDEC Decompression"): the run-length codes and the quantisation,
//! the IDCT with the scale table the program uploads, and the conversion of a block to its output words at each
//! depth. Diamond_MDEC.md §3 gives each rule's source and the fits that chose what psx-spx leaves open.

/// zigzag[position] = the coefficient's index in the stream (psx-spx "zigzag[0..63]").
const ZIGZAG: [u8; 64] = [
    0, 1, 5, 6, 14, 15, 27, 28, 2, 4, 7, 13, 16, 26, 29, 42, 3, 8, 12, 17, 25, 30, 41, 43, 9, 11, 18, 24, 31, 40, 44, 53, 10, 19, 23, 32, 39, 45, 52, 54, 20, 22, 33, 38, 46, 51, 55, 60, 21, 34, 37, 47, 50, 56, 59, 61, 35, 36, 48, 49, 57, 58, 62, 63,
];

/// zagzig[index in the stream] = the coefficient's position in the block, the inverse of `ZIGZAG`.
pub const ZAGZIG: [u8; 64] = {
    let mut t = [0u8; 64];
    let mut i = 0;
    while i < 64 {
        t[ZIGZAG[i] as usize] = i as u8;
        i += 1;
    }
    t
};

/// The padding and end-of-block code.
pub const END: u16 = 0xFE00;

fn signed10(n: u16) -> i32 {
    ((n as i32) << 22) >> 22
}

/// A block's coefficients as the run-length decoder fills them, one halfword at a time.
#[derive(Clone, Copy, Debug)]
pub struct Rle {
    pub coefficients: [i16; 64],
    /// The stream index of the last value stored, or None before the block's first halfword.
    pub k: Option<u8>,
    pub q_scale: u8,
}

impl Default for Rle {
    fn default() -> Rle {
        Rle { coefficients: [0; 64], k: None, q_scale: 0 }
    }
}

impl Rle {
    /// Whether `n` would complete the block.
    pub fn ends_with(&self, n: u16) -> bool {
        match self.k {
            None => false,
            Some(k) => n == END || k as u32 + (n as u32 >> 10) + 1 >= 63,
        }
    }

    /// Takes one halfword with the block's quantisation table; returns whether the block is complete.
    pub fn take(&mut self, n: u16, table: &[u8; 64]) -> bool {
        let Some(k) = self.k else {
            if n == END {
                return false;
            }
            self.coefficients = [0; 64];
            self.q_scale = (n >> 10) as u8;
            let v = if self.q_scale == 0 { signed10(n) * 2 } else { signed10(n) * table[0] as i32 };
            self.coefficients[0] = v.clamp(-0x400, 0x3FF) as i16;
            self.k = Some(0);
            return false;
        };
        if n == END {
            self.k = None;
            return true;
        }
        let k = k as u32 + (n as u32 >> 10) + 1;
        if k > 63 {
            self.k = None;
            return true;
        }
        let qs = self.q_scale as i32;
        let v = if qs == 0 { signed10(n) * 2 } else { (signed10(n) * table[k as usize] as i32 * qs + 4) >> 3 };
        let at = if qs == 0 { k as usize } else { ZAGZIG[k as usize] as usize };
        self.coefficients[at] = v.clamp(-0x400, 0x3FF) as i16;
        if k == 63 {
            self.k = None;
            return true;
        }
        self.k = Some(k as u8);
        false
    }
}

/// The fraction bits the IDCT's results keep, which the colour conversion and the output's rounding use.
pub const FRACTION: u32 = 4;

/// The two passes of the IDCT with the scale table's upper 13 bits (psx-spx "real_idct_core"), pass 1 down the
/// columns; each pass's sum truncated toward zero, keeping three fraction bits after pass 1 and `FRACTION` after pass 2
/// (Diamond_MDEC.md §3.3, D-30).
pub fn idct(block: &[i16; 64], scale: &[i16; 64], out: &mut [i32; 64]) {
    let mut mid = [0i64; 64];
    for y in 0..8 {
        for x in 0..8 {
            let mut s: i64 = 0;
            for z in 0..8 {
                s += block[y + z * 8] as i64 * (scale[x + z * 8] >> 3) as i64;
            }
            mid[x + y * 8] = toward_zero(s, 10);
        }
    }
    for y in 0..8 {
        for x in 0..8 {
            let mut s: i64 = 0;
            for z in 0..8 {
                s += mid[y + z * 8] * (scale[x + z * 8] >> 3) as i64;
            }
            out[x + y * 8] = toward_zero(s, 26 - 10 - FRACTION).clamp(i32::MIN as i64 >> 8, i32::MAX as i64 >> 8) as i32;
        }
    }
}

fn toward_zero(s: i64, bits: u32) -> i64 {
    if s < 0 { -((-s) >> bits) } else { s >> bits }
}

/// A value with `FRACTION` bits to a whole one, a half rounded down (Diamond_MDEC.md §3.3).
fn whole(v: i32) -> i32 {
    (v + (1 << (FRACTION - 1)) - 1) >> FRACTION
}

/// A luminance value to the signed 8-bit range: its 9-bit wrap, then saturation (psx-spx "y_to_mono").
pub fn mono(y: i32) -> i32 {
    ((whole(y) << 23) >> 23).clamp(-128, 127)
}

/// One pixel's three channels with `FRACTION` bits, from luminance and the two chroma values (psx-spx "yuv_to_rgb",
/// its factors at twelve bits), the chroma whole.
pub fn rgb(y: i32, cr: i32, cb: i32) -> [i32; 3] {
    let (cr, cb) = (whole(cr), whole(cb));
    let term = |v: i32| (v + (1 << (11 - FRACTION))) >> (12 - FRACTION);
    [y + term(cr * CR_R), y + term(cb * CB_G + cr * CR_G), y + term(cb * CB_B)]
}

/// psx-spx's factors 1.402, -0.3437, -0.7143 and 1.772 at twelve bits.
const CR_R: i32 = 5743;
const CB_G: i32 = -1408;
const CR_G: i32 = -2926;
const CB_B: i32 = 7258;

/// A channel to eight bits, saturated, and to five, each rounded from the channel's fraction (Diamond_MDEC.md §3.4).
fn eight(x: i32) -> i32 {
    whole(x).clamp(-128, 127)
}

fn five(x: i32, signed: bool) -> u32 {
    let u = if signed { x } else { x + (128 << FRACTION) };
    let v = (u + (1 << (FRACTION + 2)) - 1) >> (FRACTION + 3);
    (if signed { v.clamp(-16, 15) } else { v.clamp(0, 31) }) as u32 & 31
}

/// The output depths (psx-spx "MDEC(1)", bits 27-28).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Depth {
    Four,
    Eight,
    TwentyFour,
    Fifteen,
}

impl Depth {
    pub fn of(bits: u32) -> Depth {
        match bits & 3 {
            0 => Depth::Four,
            1 => Depth::Eight,
            2 => Depth::TwentyFour,
            _ => Depth::Fifteen,
        }
    }

    pub fn colour(self) -> bool {
        matches!(self, Depth::TwentyFour | Depth::Fifteen)
    }

    /// The words one 8x8 block comes out as.
    pub fn words(self) -> usize {
        match self {
            Depth::Four => 8,
            Depth::Eight => 16,
            Depth::Fifteen => 32,
            Depth::TwentyFour => 48,
        }
    }
}

/// How the output is formed: its depth, whether signed, and bit 15 for 15-bit pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Format {
    pub depth: Depth,
    pub signed: bool,
    pub bit15: bool,
}

/// A monochrome block's words (4- or 8-bit), in raster order.
pub fn pack_mono(y: &[i32; 64], f: Format, out: &mut [u32]) {
    let flip = if f.signed { 0 } else { 0x80 };
    match f.depth {
        Depth::Four => {
            for (w, o) in out.iter_mut().enumerate().take(8) {
                let mut v = 0;
                for i in 0..8 {
                    let p = ((mono(y[w * 8 + i]) ^ flip) & 0xFF) as u32;
                    v |= four_bit(p, f.signed) << (4 * i);
                }
                *o = v;
            }
        }
        _ => {
            for (w, o) in out.iter_mut().enumerate().take(16) {
                let mut v = 0;
                for i in 0..4 {
                    v |= (((mono(y[w * 4 + i]) ^ flip) & 0xFF) as u32) << (8 * i);
                }
                *o = v;
            }
        }
    }
}

/// An 8-bit value to four bits, rounded and saturated as the console's 4-bit output is (Diamond_MDEC.md §3.5).
fn four_bit(p: u32, signed: bool) -> u32 {
    if signed {
        (((p as u8 as i8 as i32) + 8) >> 4).clamp(-8, 7) as u32 & 15
    } else {
        ((p + 8) >> 4).min(15)
    }
}

/// One luminance block of a colour macroblock, at (`bx`, `by`) in it, with the macroblock's chroma, to its words.
pub fn pack_colour(y: &[i32; 64], cr: &[i32; 64], cb: &[i32; 64], bx: usize, by: usize, f: Format, out: &mut [u32]) {
    let flip = if f.signed { 0 } else { 0x80 };
    let pixel = |i: usize| -> [i32; 3] {
        let (x, yy) = (i % 8, i / 8);
        let c = (x + bx) / 2 + (yy + by) / 2 * 8;
        rgb(y[i], cr[c], cb[c])
    };
    match f.depth {
        Depth::Fifteen => {
            let top = if f.bit15 { 0x8000 } else { 0 };
            for (w, o) in out.iter_mut().enumerate().take(32) {
                let mut v = 0;
                for h in 0..2 {
                    let [r, g, b] = pixel(w * 2 + h);
                    v |= (five(r, f.signed) | five(g, f.signed) << 5 | five(b, f.signed) << 10 | top) << (16 * h);
                }
                *o = v;
            }
        }
        _ => {
            let mut bytes = [0u8; 192];
            for i in 0..64 {
                for (k, c) in pixel(i).into_iter().enumerate() {
                    bytes[i * 3 + k] = (eight(c) ^ flip) as u8;
                }
            }
            for (w, o) in out.iter_mut().enumerate().take(48) {
                *o = u32::from_le_bytes([bytes[w * 4], bytes[w * 4 + 1], bytes[w * 4 + 2], bytes[w * 4 + 3]]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_inverse_zigzag_undoes_the_zigzag() {
        for i in 0..64 {
            assert_eq!(ZAGZIG[ZIGZAG[i] as usize] as usize, i);
        }
        assert_eq!(&ZAGZIG[..10], &[0, 1, 8, 16, 9, 2, 3, 10, 17, 24]);
    }

    #[test]
    fn a_block_ends_at_its_end_code_or_once_its_last_coefficient_is_written() {
        let table = [8u8; 64];
        let mut r = Rle::default();
        assert!(!r.take(END, &table), "padding before a block is skipped");
        assert!(!r.take(0x0400 | 5, &table));
        assert_eq!(r.coefficients[0], 40);
        assert!(!r.take(2 << 10 | 0x3FF, &table), "two zeros, then -1 times 8 at stream index 3, rounded");
        assert_eq!(r.coefficients[ZAGZIG[3] as usize], -1);
        assert!(r.take(END, &table));
        assert!(!r.take(0x0400, &table));
        assert!(r.take(62 << 10, &table), "a run to index 63 completes the block with no end code");
        assert_eq!(r.k, None);
    }

    #[test]
    fn q_scale_zero_stores_twice_the_value_in_stream_order() {
        let table = [9u8; 64];
        let mut r = Rle::default();
        r.take(0x0003, &table);
        r.take(0x0001, &table);
        assert_eq!((r.coefficients[0], r.coefficients[1]), (6, 2));
    }
}
