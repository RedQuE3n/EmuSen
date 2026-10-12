//! The ROM's character sets, which an open kernel does not carry: for charset 2 and 3's documented codes a blank glyph
//! for the ideographic space and one box, the missing-character glyph, for every other. Diamond_OpenKernel_Phase2.md
//! §25.1.

/// Two 16x15 glyphs at one bit a pixel, each row two bytes with the left pixel in bit 7 of the first, the form a game's
/// own glyphs take (§25.1): the space, then a box over columns 2-13 and rows 1-13.
static GLYPHS: [u8; 60] = {
    let mut g = [0u8; 60];
    let mut row = 1;
    while row <= 13 {
        let bits: u16 = if row == 1 || row == 13 { 0x3FFC } else { 0x2004 };
        g[30 + 2 * row] = (bits >> 8) as u8;
        g[30 + 2 * row + 1] = bits as u8;
        row += 1;
    }
    g
};

const SPACE: u32 = 0x8140;
const GLYPH: u32 = 30;

/// psx-spx's ranges for B(51h): 8140h-84BEh (charset 2) and 889Fh-9872h (charset 3).
fn documented(code: u32) -> bool {
    matches!(code, 0x8140..=0x84BE | 0x889F..=0x9872)
}

/// B(53h): the glyph's offset within the kernel's two, or -1.
pub extern "C" fn krom2_offset(code: u32) -> u32 {
    match code {
        SPACE => 0,
        c if documented(c) => GLYPH,
        _ => u32::MAX,
    }
}

/// B(51h): the glyph's address in the BIOS ROM as KSEG1 gives it, or -1.
pub extern "C" fn krom2_raw_add(code: u32) -> u32 {
    match krom2_offset(code) {
        u32::MAX => u32::MAX,
        offset => (&raw const GLYPHS as u32 | 0x2000_0000) + offset,
    }
}
