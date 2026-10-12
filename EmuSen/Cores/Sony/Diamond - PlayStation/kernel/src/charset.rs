//! The ROM's character sets, which an open kernel does not carry: for charset 2 and 3's documented codes, the
//! full-width Latin letters, digits and common punctuation drawn from the kernel's own 5x8 font, a blank glyph for the
//! ideographic space, and one box, the missing-character glyph, for every other. Diamond_OpenKernel_Phase2.md §25.1.

use crate::font;

/// The full-width code's ASCII form among the letters, digits and punctuation of JIS X 0208's first rows ("Shift-JIS
/// Character Set"); the card manager's titles use the same.
pub const fn ascii(code: u32) -> Option<u8> {
    let (a, b) = (code >> 8, code & 0xFF);
    Some(match (a, b) {
        (0x82, 0x4F..=0x58) => (b - 0x4F) as u8 + b'0',
        (0x82, 0x60..=0x79) => (b - 0x60) as u8 + b'A',
        (0x82, 0x81..=0x9A) => (b - 0x81) as u8 + b'a',
        (0x81, 0x40) => b' ',
        (0x81, 0x43) => b',',
        (0x81, 0x44) => b'.',
        (0x81, 0x46) => b':',
        (0x81, 0x47) => b';',
        (0x81, 0x48) => b'?',
        (0x81, 0x49) => b'!',
        (0x81, 0x5B..=0x5D | 0x7C) => b'-',
        (0x81, 0x5E) => b'/',
        (0x81, 0x60) => b'~',
        (0x81, 0x62) => b'|',
        (0x81, 0x65 | 0x66) => b'\'',
        (0x81, 0x67 | 0x68) => b'"',
        (0x81, 0x69) => b'(',
        (0x81, 0x6A) => b')',
        (0x81, 0x6D) => b'[',
        (0x81, 0x6E) => b']',
        (0x81, 0x7B) => b'+',
        (0x81, 0x81) => b'=',
        (0x81, 0x83) => b'<',
        (0x81, 0x84) => b'>',
        (0x81, 0x90) => b'$',
        (0x81, 0x93) => b'%',
        (0x81, 0x94) => b'#',
        (0x81, 0x95) => b'&',
        (0x81, 0x96) => b'*',
        (0x81, 0x97) => b'@',
        _ => return None,
    })
}

const SPACE: u32 = 0x8140;
const GLYPH: usize = 30;
/// The codes drawn from the font all lie in 8140h-829Ah.
const DRAWN_FIRST: u32 = 0x8140;
const DRAWN_LAST: u32 = 0x829A;
const SPAN: usize = (DRAWN_LAST - DRAWN_FIRST + 1) as usize;

/// Whether a code of the span is drawn from the font: mapped, and not the space, which stays blank.
const fn drawn(code: u32) -> Option<u8> {
    match ascii(code) {
        Some(b' ') | None => None,
        c => c,
    }
}

/// Each code of the span's glyph number, 0 for none; glyph 0 is the space and 1 the box.
static INDEX: [u8; SPAN] = {
    let mut t = [0u8; SPAN];
    let (mut i, mut n) = (0, 2u8);
    while i < SPAN {
        if drawn(DRAWN_FIRST + i as u32).is_some() {
            t[i] = n;
            n += 1;
        }
        i += 1;
    }
    t
};

const COUNT: usize = {
    let (mut i, mut n) = (0, 2);
    while i < SPAN {
        if drawn(DRAWN_FIRST + i as u32).is_some() {
            n += 1;
        }
        i += 1;
    }
    n
};

/// A 16x15 glyph at one bit a pixel, each row two bytes with the left pixel in bit 7 of the first, the form a game's
/// own glyphs take (§25.1), from 15 rows of 16-bit patterns, bit 15 the left pixel.
const fn store(g: &mut [u8; COUNT * GLYPH], n: usize, rows: [u16; 15]) {
    let mut r = 0;
    while r < 15 {
        g[n * GLYPH + 2 * r] = (rows[r] >> 8) as u8;
        g[n * GLYPH + 2 * r + 1] = rows[r] as u8;
        r += 1;
    }
}

/// The font's 5x8 glyph doubled across into columns 3-12, its rows 0-6 doubled down and row 7 once, into rows 0-14.
const fn scaled(c: u8) -> [u16; 15] {
    let cell = font::GLYPHS[(c as u32 - font::FIRST) as usize];
    let mut rows = [0u16; 15];
    let mut y = 0;
    while y < 15 {
        let src = cell[y * 8 / 15];
        let mut x = 0;
        while x < 5 {
            if src >> (4 - x) & 1 != 0 {
                rows[y] |= 0b11 << (11 - 2 * x);
            }
            x += 1;
        }
        y += 1;
    }
    rows
}

/// The space, the box over columns 2-13 and rows 1-13, then each drawn code in order.
static GLYPHS: [u8; COUNT * GLYPH] = {
    let mut g = [0u8; COUNT * GLYPH];
    let mut boxed = [0u16; 15];
    let mut row = 1;
    while row <= 13 {
        boxed[row] = if row == 1 || row == 13 { 0x3FFC } else { 0x2004 };
        row += 1;
    }
    store(&mut g, 1, boxed);
    let mut i = 0;
    while i < SPAN {
        if let Some(c) = drawn(DRAWN_FIRST + i as u32) {
            store(&mut g, INDEX[i] as usize, scaled(c));
        }
        i += 1;
    }
    g
};

/// psx-spx's ranges for B(51h): 8140h-84BEh (charset 2) and 889Fh-9872h (charset 3).
fn documented(code: u32) -> bool {
    matches!(code, 0x8140..=0x84BE | 0x889F..=0x9872)
}

/// B(53h): the glyph's offset within the kernel's glyphs, or -1.
pub extern "C" fn krom2_offset(code: u32) -> u32 {
    if code == SPACE {
        return 0;
    }
    if !documented(code) {
        return u32::MAX;
    }
    let n = match code {
        DRAWN_FIRST..=DRAWN_LAST => INDEX[(code - DRAWN_FIRST) as usize],
        _ => 0,
    };
    GLYPH as u32 * if n == 0 { 1 } else { n as u32 }
}

/// B(51h): the glyph's address in the BIOS ROM as KSEG1 gives it, or -1.
pub extern "C" fn krom2_raw_add(code: u32) -> u32 {
    match krom2_offset(code) {
        u32::MAX => u32::MAX,
        offset => (&raw const GLYPHS as u32 | 0x2000_0000) + offset,
    }
}
