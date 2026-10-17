//! Save icons from psx-spx's "Title Frame" and "Icon Frame(s)": byte 02h of the title frame 11h, 12h or 13h for one,
//! two or three 16x16 frames of 4 bits, the 16-colour CLUT at 60h-7Fh with 0000h fully transparent, two frames changing
//! every 16 frames and three every 11. Each held in VRAM beside the font and drawn as a raw textured rectangle at 1:1
//! (Diamond_Shell.md §3.1).

use diamond_guest::gpu::{self, gp0, upload, FONT_X};

/// Texture page 14 (x 896) for the icons, 4 bits a texel: frame f of row k at texels (16f, 16k).
const PAGE_X: u32 = 896;
/// The CLUTs, one row each, at x 960 from y 256.
const CLUT_X: u32 = 960;
const CLUT_Y: u32 = 256;

/// Frames an icon has, or None for a title frame no icon follows (a damaged save).
pub fn frames(title: &[u8; 512]) -> Option<u32> {
    if &title[..2] != b"SC" {
        return None;
    }
    match title[2] {
        0x11 => Some(1),
        0x12 => Some(2),
        0x13 => Some(3),
        _ => None,
    }
}

/// Row `k`'s icon: its CLUT and its frames written to VRAM.
pub fn load(k: usize, title: &[u8; 512]) {
    let Some(n) = frames(title) else { return };
    let mut clut = [0u16; 16];
    for (i, c) in clut.iter_mut().enumerate() {
        *c = u16::from_le_bytes([title[0x60 + 2 * i], title[0x61 + 2 * i]]);
    }
    upload(CLUT_X, CLUT_Y + k as u32, 16, 1, &clut);
    for f in 0..n {
        let mut pixels = [0u16; 64];
        for (i, p) in pixels.iter_mut().enumerate() {
            let at = 0x80 * (f as usize + 1) + 2 * i;
            *p = u16::from_le_bytes([title[at], title[at + 1]]);
        }
        upload(PAGE_X + 4 * f, 16 * k as u32, 4, 16, &pixels);
    }
}

/// Row `k`'s icon at (x, y), the frame the console's frame count `now` shows.
pub fn draw(k: usize, title: &[u8; 512], x: u32, y: u32, now: u32) {
    let Some(n) = frames(title) else { return };
    let f = match n {
        2 => now / 16 % 2,
        3 => now / 11 % 3,
        _ => 0,
    };
    let clut = CLUT_X / 16 | (CLUT_Y + k as u32) << 6;
    gp0(&[0xE100_0000 | PAGE_X / 64 | 1 << 10]);
    gp0(&[0x6500_0000, y << 16 | x, clut << 16 | (16 * k as u32) << 8 | 16 * f, 16 << 16 | 16]);
    gpu::page(FONT_X);
}
