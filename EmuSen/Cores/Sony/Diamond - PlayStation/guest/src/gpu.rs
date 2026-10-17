//! The GPU as psx-spx's "GPU" chapter gives it: a command sent once GPUSTAT bit 26 says one may be, the display set
//! to 320x240 NTSC at 15 bits, the quick fill, and text from the font held in VRAM as a 15-bit texture, each glyph an
//! 8x8 textured rectangle. Diamond_OpenKernel_Phase2.md §16.

use crate::font::{FIRST, GLYPHS};
use crate::mem::{rd, wr};

pub const GP0: u32 = 0xBF80_1810;
pub const GP1: u32 = 0xBF80_1814;

/// The font's place in VRAM, outside the displayed 320x240: texture page 10 (x 640), 32 glyphs a row.
pub const FONT_X: u32 = 640;
pub const ADVANCE: u32 = 6;

/// A command and its parameters, sent once GPUSTAT bit 26 says the GPU is ready for a command ("GPU Status Register").
pub fn gp0(words: &[u32]) {
    let mut n = 0;
    while rd(GP1) & 1 << 26 == 0 && n < 1_000_000 {
        n += 1;
    }
    for &w in words {
        wr(GP0, w);
    }
}

pub fn gp1(word: u32) {
    wr(GP1, word);
}

/// A colour as GP0 takes it, from 8-bit red, green and blue.
pub const fn rgb(r: u32, g: u32, b: u32) -> u32 {
    r | g << 8 | b << 16
}

/// GP1's reset, 320x240 NTSC at 15 bits in the usual display range (X 600-3160, Y 10h-100h, "GP1(06h)", "GP1(07h)"),
/// the drawing area over it, the texture page of the font, then the font uploaded, the screen filled and the display on.
pub fn init(background: u32) {
    gp1(0x0000_0000);
    gp1(0x0100_0000);
    gp1(0x0800_0001);
    gp1(0x0600_0000 | 3160 << 12 | 600);
    gp1(0x0700_0000 | 0x100 << 10 | 0x10);
    gp1(0x0500_0000);
    page(FONT_X);
    gp0(&[0xE200_0000]);
    gp0(&[0xE300_0000]);
    gp0(&[0xE400_0000 | 239 << 10 | 319]);
    gp0(&[0xE500_0000]);
    gp0(&[0xE600_0000]);
    upload_font();
    fill(0, 0, 320, 240, background);
    gp1(0x0300_0000);
}

/// GP0(E1h): the texture page at VRAM column `x`, 15-bit texels, drawing to the displayed area allowed.
pub fn page(x: u32) {
    gp0(&[0xE100_0000 | x / 64 | 2 << 7 | 1 << 10]);
}

/// Each glyph's 5x8 pattern at columns 1-5 of its 8x8 cell, white where set and 0000h, transparent, elsewhere.
fn upload_font() {
    gp0(&[0xA000_0000, FONT_X, 24 << 16 | 256]);
    for y in 0..24u32 {
        for x in (0..256u32).step_by(2) {
            let mut word = 0;
            for k in 0..2 {
                let px = x + k;
                let g = (y / 8 * 32 + px / 8) as usize;
                let column = px % 8;
                let row = GLYPHS[g][(y % 8) as usize] as u32;
                if (1..=5).contains(&column) && row >> (5 - column) & 1 != 0 {
                    word |= 0x7FFF << (16 * k);
                }
            }
            wr(GP0, word);
        }
    }
}

/// GP0(02h), the quick fill.
pub fn fill(x: u32, y: u32, w: u32, h: u32, colour: u32) {
    gp0(&[0x0200_0000 | colour, y << 16 | x, h << 16 | w]);
}

/// A byte string to its first 0 or `max` characters, each glyph an 8x8 textured rectangle (GP0(74h)) blended with
/// `colour`, 80h a channel's unity, `?` for a byte the font lacks; the x after it.
pub fn text(mut x: u32, y: u32, s: &[u8], max: usize, colour: u32) -> u32 {
    for &c in s.iter().take(max) {
        if c == 0 {
            break;
        }
        let g = if (FIRST..0x80).contains(&(c as u32)) { c as u32 - FIRST } else { b'?' as u32 - FIRST };
        gp0(&[0x7400_0000 | colour, y << 16 | x, (g / 32 * 8) << 8 | g % 32 * 8]);
        x += ADVANCE;
    }
    x
}

/// A rectangle of `w` by `h` VRAM halfwords written from `data` at (x, y), GP0(A0h).
pub fn upload(x: u32, y: u32, w: u32, h: u32, data: &[u16]) {
    gp0(&[0xA000_0000, y << 16 | x, h << 16 | w]);
    let mut i = 0;
    while i < data.len() {
        let lo = data[i] as u32;
        let hi = data.get(i + 1).copied().unwrap_or(0) as u32;
        wr(GP0, lo | hi << 16);
        i += 2;
    }
}
