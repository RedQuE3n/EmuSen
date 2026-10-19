//! Mode 4's composition of a line (Beryl_SmsVdp.md §4): the sprites a line holds and their pixels, the background's
//! pixels with the scroll locks and the fine scroll's blank, and the mix of the two. It reads VRAM through `Vram`, so
//! that a console whose VRAM lies elsewhere (the Genesis, stage 4) keeps its own map, and it writes CRAM indices, never
//! colours. Nothing here allocates.

use crate::Variant;

/// VRAM as mode 4 addresses it: fourteen bits.
pub trait Vram {
    fn byte(&self, address: u32) -> u8;
}

impl Vram for [u8] {
    #[inline(always)]
    fn byte(&self, address: u32) -> u8 {
        self[(address & 0x3FFF) as usize]
    }
}

/// A background pixel: its CRAM index in bits 0-4, and in bit 7 its tile's priority over the sprites.
pub const PRIORITY: u8 = 0x80;

/// One sprite of a line: where its first pixel falls (the early clock applied), whether it is shown at twice its width,
/// and its four bitplanes for the line's row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sprite {
    pub x: i16,
    pub wide: bool,
    pub planes: [u8; 4],
}

/// A line's sprites, at most eight, and their pixels: each a CRAM index of the sprite palette, 0 where none is opaque.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpriteLine {
    pub count: u8,
    pub sprites: [Sprite; 8],
    pub pixels: [u8; 256],
    /// The first pixel at which two opaque sprite pixels meet; 256 for none.
    pub collision: u16,
}

impl Default for SpriteLine {
    fn default() -> SpriteLine {
        SpriteLine { count: 0, sprites: [Sprite::default(); 8], pixels: [0; 256], collision: 256 }
    }
}

/// The backdrop's CRAM index: register 7's colour of the sprite palette [R MacDonald §7].
#[inline]
pub fn backdrop(regs: &[u8; 11]) -> u8 {
    16 | (regs[7] & 0x0F)
}

fn colour(planes: &[u8; 4], bit: u32) -> u8 {
    (0..4).fold(0, |c, p| c | ((planes[p] >> bit) & 1) << p)
}

/// The sprites on the line whose V counter is `v`, at most eight in table order, into `out`; returns whether a ninth
/// was found [R MacDonald §10; SMS Power!'s "Sprites"]. `terminator`: a Y of $D0 ends the table, as it does in the
/// 192-line display only. On the 315-5124 register 5's bit 0 and register 6's bits 1-0 mask the fetches' addresses
/// [R SMS Power!'s "VDP Registers"].
pub fn find_sprites<V: Vram + ?Sized>(vram: &V, regs: &[u8; 11], variant: Variant, v: u8, terminator: bool, out: &mut SpriteLine) -> bool {
    let base = (regs[5] as u32 & 0x7E) << 7;
    let xn = if variant.masks && regs[5] & 1 == 0 { base } else { base | 0x80 };
    let tall = regs[1] & 2 != 0;
    let zoom = regs[1] & 1 != 0;
    let height = (if tall { 16 } else { 8 }) << zoom as u32;
    let shift: i16 = if regs[0] & 8 != 0 { 8 } else { 0 };
    let mut mask = 0x3FFF;
    if variant.masks {
        mask &= !(((regs[6] as u32 >> 1 & 1) ^ 1) << 12 | ((regs[6] as u32 & 1) ^ 1) << 11);
    }
    let mut n = 0;
    let mut overflow = false;
    for i in 0..64 {
        let y = vram.byte(base + i);
        if terminator && y == 0xD0 {
            break;
        }
        let row = v.wrapping_sub(y).wrapping_sub(1) as u32;
        if row >= height {
            continue;
        }
        if n == 8 {
            overflow = true;
            break;
        }
        let x = vram.byte(xn + 2 * i);
        let tile = vram.byte(xn + 2 * i + 1) as u32;
        let row = if zoom { row >> 1 } else { row };
        let tile = if tall { (tile & 0xFE) + (row >> 3) } else { tile };
        let at = (((regs[6] as u32 & 4) << 11) | tile << 5 | (row & 7) << 2) & mask;
        out.sprites[n] = Sprite { x: x as i16 - shift, wide: zoom, planes: [0, 1, 2, 3].map(|p| vram.byte(at | p)) };
        n += 1;
    }
    out.count = n as u8;
    // The 315-5124 widens only the first N - 4 of the N sprites on a line (Endou_Disputes.md D-6).
    if zoom && !variant.zoom_all {
        for (i, s) in out.sprites[..n].iter_mut().enumerate() {
            s.wide = i + 4 < n;
        }
    }
    overflow
}

/// The line's sprite pixels from its sprites: an earlier sprite's opaque pixel over a later one's, and the first pixel
/// where two opaque ones meet [R MacDonald §10]. Nothing past pixel 255 is drawn or collides, nor wraps to the left.
pub fn sprite_pixels(line: &mut SpriteLine) {
    line.pixels.fill(0);
    line.collision = 256;
    for s in &line.sprites[..line.count as usize] {
        let width = if s.wide { 16 } else { 8 };
        for px in 0..width {
            let x = s.x as i32 + px;
            if !(0..256).contains(&x) {
                continue;
            }
            let c = colour(&s.planes, 7 - if s.wide { px >> 1 } else { px } as u32);
            if c == 0 {
                continue;
            }
            let p = &mut line.pixels[x as usize];
            if *p != 0 {
                line.collision = line.collision.min(x as u16);
            } else {
                *p = 16 | c;
            }
        }
    }
}

/// What the background of a line is drawn with: the line's row on the display, the scrolls the chip took for it, and
/// whether the name table is 32 by 32 (the 224- and 240-line displays) rather than 32 by 28.
#[derive(Clone, Copy, Debug)]
pub struct Background {
    pub line: u32,
    pub hscroll: u8,
    pub vscroll: u8,
    pub tall_table: bool,
}

/// The background's pixels from `x0` for `out.len()` pixels [R MacDonald §9; SMS Power!'s "VDP Registers" and
/// "Tilemap"]. The first fine-scroll pixels show the backdrop, the column counter drawing nothing there.
pub fn background<V: Vram + ?Sized>(vram: &V, regs: &[u8; 11], variant: Variant, bg: Background, x0: u32, out: &mut [u8]) {
    let hs = if regs[0] & 0x40 != 0 && bg.line < 16 { 0 } else { bg.hscroll as u32 };
    let fine = hs & 7;
    let mut column = u32::MAX;
    let mut row = [0u8; 8];
    for (i, o) in out.iter_mut().enumerate() {
        let x = x0 + i as u32;
        if x < fine {
            *o = backdrop(regs);
            continue;
        }
        let k = (x - fine) >> 3;
        if k != column {
            column = k;
            row = tile_row(vram, regs, variant, bg, k, hs >> 3);
        }
        *o = row[((x - fine) & 7) as usize];
    }
}

/// The eight pixels of the `k`th column the counter draws on the line.
fn tile_row<V: Vram + ?Sized>(vram: &V, regs: &[u8; 11], variant: Variant, bg: Background, k: u32, coarse: u32) -> [u8; 8] {
    let column = (k + 32 - coarse) & 31;
    // The lock is by the column counter, not the name table's column [R MacDonald §9].
    let vs = if regs[0] & 0x80 != 0 && k >= 24 { 0 } else { bg.vscroll as u32 };
    let y = if bg.tall_table { (bg.line + vs) & 0xFF } else { (bg.line + vs) % 224 };
    let cell = (y >> 3) << 6 | column << 1;
    let at = if bg.tall_table {
        (((regs[2] as u32 & 0x0C) << 10) + 0x700 + cell) & 0x3FFF
    } else {
        let a = ((regs[2] as u32 & 0x0E) << 10) | cell;
        if variant.masks && regs[2] & 1 == 0 { a & !0x400 } else { a }
    };
    let entry = vram.byte(at) as u32 | (vram.byte(at + 1) as u32) << 8;
    let (hflip, vflip) = (entry & 0x200 != 0, entry & 0x400 != 0);
    let line = if vflip { 7 - (y & 7) } else { y & 7 };
    let pattern = (entry & 0x1FF) << 5 | line << 2;
    let planes = [0u32, 1, 2, 3].map(|p| {
        let mut a = pattern | p;
        if variant.masks {
            a &= if p < 2 { (regs[3] as u32) << 6 | 0x3F } else { (regs[4] as u32 & 7) << 11 | 0x7FF };
        }
        vram.byte(a)
    });
    let high = if entry & 0x800 != 0 { 16 } else { 0 } | if entry & 0x1000 != 0 { PRIORITY } else { 0 };
    let mut px = [0u8; 8];
    for (i, p) in px.iter_mut().enumerate() {
        let bit = if hflip { i } else { 7 - i } as u32;
        *p = high | colour(&planes, bit);
    }
    px
}

/// A pixel's CRAM index: the sprite's where it is opaque, unless the tile has priority and its pixel is not colour 0
/// [R MacDonald §9-§10; Sega's Mark III manual, "Sprite/background display priority"].
#[inline]
pub fn mix(bg: u8, sprite: u8) -> u8 {
    if sprite != 0 && (bg & PRIORITY == 0 || bg & 0x0F == 0) { sprite } else { bg & 0x1F }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn regs() -> [u8; 11] {
        [0x06, 0xC0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFB, 0x00, 0, 0, 0xFF]
    }

    /// A tile at `n` whose row `r` is colour `c` on its leftmost pixel only.
    fn mark(vram: &mut [u8], n: usize, r: usize, c: u8) {
        for p in 0..4 {
            vram[n * 32 + r * 4 + p] = if c >> p & 1 != 0 { 0x80 } else { 0 };
        }
    }

    #[test]
    fn a_sprite_s_y_is_one_less_than_its_first_line_and_d0_ends_the_table_only_in_192_lines() {
        let mut vram = vec![0u8; 0x4000];
        let r = regs();
        vram[0x3F00] = 9;
        vram[0x3F01] = 0xD0;
        vram[0x3F02] = 9;
        let mut line = SpriteLine::default();
        find_sprites(&vram[..], &r, Variant::SMS2, 9, true, &mut line);
        assert_eq!(line.count, 0, "line 9 is above a sprite at Y 9");
        find_sprites(&vram[..], &r, Variant::SMS2, 10, true, &mut line);
        assert_eq!(line.count, 1);
        find_sprites(&vram[..], &r, Variant::SMS2, 17, false, &mut line);
        assert_eq!(line.count, 2, "past $D0 in a taller display");
        find_sprites(&vram[..], &r, Variant::SMS2, 18, false, &mut line);
        assert_eq!(line.count, 0);
    }

    #[test]
    fn a_ninth_sprite_is_overflow_and_the_first_eight_keep_their_order() {
        let mut vram = vec![0u8; 0x4000];
        let r = regs();
        for i in 0..10 {
            vram[0x3F00 + i] = 50;
            vram[0x3F80 + 2 * i] = (i * 10) as u8;
        }
        vram[0x3F0A] = 0xD0;
        let mut line = SpriteLine::default();
        assert!(find_sprites(&vram[..], &r, Variant::SMS2, 51, true, &mut line));
        assert_eq!((line.count, line.sprites[7].x), (8, 70));
        vram[0x3F08] = 0xD0;
        assert!(!find_sprites(&vram[..], &r, Variant::SMS2, 51, true, &mut line), "eight exactly is no overflow");
    }

    #[test]
    fn earlier_sprites_win_and_overlapping_opaque_pixels_collide() {
        let mut line = SpriteLine { count: 3, ..SpriteLine::default() };
        line.sprites[0] = Sprite { x: 10, wide: false, planes: [0x0F, 0, 0, 0] };
        line.sprites[1] = Sprite { x: 12, wide: false, planes: [0, 0xFF, 0, 0] };
        line.sprites[2] = Sprite { x: 250, wide: false, planes: [0xFF, 0, 0, 0] };
        sprite_pixels(&mut line);
        assert_eq!(&line.pixels[10..22], [0, 0, 18, 18, 17, 17, 17, 17, 18, 18, 0, 0], "sprite 1 shows where sprite 0 is transparent");
        assert_eq!(line.collision, 14);
        assert_eq!((line.pixels[255], line.pixels[0]), (17, 0), "no wrap past the right edge");
    }

    #[test]
    fn the_first_master_system_widens_only_n_minus_four_of_n_zoomed_sprites() {
        let mut vram = vec![0u8; 0x4000];
        let mut r = regs();
        r[1] |= 1;
        for i in 0..6 {
            vram[0x3F00 + i] = 50;
        }
        vram[0x3F06] = 0xD0;
        let mut line = SpriteLine::default();
        find_sprites(&vram[..], &r, Variant::SMS1, 51, true, &mut line);
        assert_eq!(line.sprites[..6].iter().map(|s| s.wide).collect::<Vec<_>>(), [true, true, false, false, false, false]);
        find_sprites(&vram[..], &r, Variant::SMS2, 51, true, &mut line);
        assert!(line.sprites[..6].iter().all(|s| s.wide));
        find_sprites(&vram[..], &r, Variant::SMS2, 66, true, &mut line);
        assert_eq!(line.count, 6, "zoomed sprites are sixteen lines tall");
    }

    #[test]
    fn the_background_scrolls_by_its_column_counter_and_blanks_the_fine_scroll() {
        let mut vram = vec![0u8; 0x4000];
        let mut r = regs();
        r[7] = 5;
        // Column 3 of row 0 names tile 1, whose row 0 starts with colour 9.
        vram[0x3800 + 3 * 2] = 1;
        mark(&mut vram, 1, 0, 9);
        let mut out = [0u8; 256];
        background(&vram[..], &r, Variant::SMS2, Background { line: 0, hscroll: 0, vscroll: 0, tall_table: false }, 0, &mut out);
        assert_eq!(out[24], 9);
        background(&vram[..], &r, Variant::SMS2, Background { line: 0, hscroll: 3, vscroll: 0, tall_table: false }, 0, &mut out);
        assert_eq!((out[0], out[2], out[27]), (21, 21, 9), "three pixels of backdrop, then the columns shifted right");
        background(&vram[..], &r, Variant::SMS2, Background { line: 0, hscroll: 8 * 31, vscroll: 0, tall_table: false }, 0, &mut out);
        assert_eq!(out[16], 9, "a coarse scroll of 31 starts at name column 1, so column 3 is the counter's 2");
        r[0] |= 0x40;
        background(&vram[..], &r, Variant::SMS2, Background { line: 0, hscroll: 8 * 31, vscroll: 0, tall_table: false }, 0, &mut out);
        assert_eq!(out[24], 9, "the top two rows' lock");
    }

    #[test]
    fn the_vertical_scroll_wraps_at_224_or_256_and_the_right_columns_lock() {
        let mut vram = vec![0u8; 0x4000];
        let mut r = regs();
        mark(&mut vram, 1, 0, 9);
        // Row 2, columns 0 and 30, name tile 1.
        vram[0x3800 + 2 * 64] = 1;
        vram[0x3800 + 2 * 64 + 60] = 1;
        let mut out = [0u8; 256];
        let bg = Background { line: 0, hscroll: 0, vscroll: 240, tall_table: false };
        background(&vram[..], &r, Variant::SMS2, bg, 0, &mut out);
        assert_eq!((out[0], out[240]), (9, 9), "240 is 16 in a 28-row table");
        r[0] |= 0x80;
        background(&vram[..], &r, Variant::SMS2, bg, 0, &mut out);
        assert_eq!((out[0], out[240]), (9, 0), "columns 24-31 take a scroll of 0");
        // The 224-line display's table: $3700 for register 2's bits 3-2 set.
        vram[0x3700 + 30 * 64] = 1;
        background(&vram[..], &r, Variant::SMS2, Background { line: 0, hscroll: 0, vscroll: 240, tall_table: true }, 0, &mut out);
        assert_eq!(out[0], 9, "row 30 of a 32-row table at $3700");
    }

    #[test]
    fn the_first_master_system_masks_name_rows_and_pattern_fetches_with_its_old_registers() {
        let mut vram = vec![0u8; 0x4000];
        let mut r = regs();
        mark(&mut vram, 2, 0, 9);
        vram[0x3800 + 16 * 64] = 2;
        let mut out = [0u8; 8];
        let bg = Background { line: 128, hscroll: 0, vscroll: 0, tall_table: false };
        background(&vram[..], &r, Variant::SMS1, bg, 0, &mut out);
        assert_eq!(out[0], 9);
        r[2] = 0xFE;
        background(&vram[..], &r, Variant::SMS1, bg, 0, &mut out);
        assert_eq!(out[0], 0, "row 16 reads row 0's names when register 2's bit 0 is clear");
        background(&vram[..], &r, Variant::SMS2, Background { line: 0, ..bg }, 0, &mut out);
        assert_eq!(out[0], 0);
        r[2] = 0xFF;
        r[3] = 0x00;
        background(&vram[..], &r, Variant::SMS1, bg, 0, &mut out);
        assert_eq!(out[0], 8, "register 3 clear: planes 0 and 1 from tile 0, planes 2 and 3 still tile 2's");
        background(&vram[..], &r, Variant::SMS2, bg, 0, &mut out);
        assert_eq!(out[0], 9, "the 315-5246 ignores it");
    }

    #[test]
    fn a_priority_tile_covers_sprites_but_not_with_its_colour_0() {
        assert_eq!(mix(0x05, 0x13), 0x13);
        assert_eq!(mix(PRIORITY | 0x05, 0x13), 0x05);
        assert_eq!(mix(PRIORITY | 0x10, 0x13), 0x13, "colour 0 of either palette is under the sprites");
        assert_eq!(mix(0x15, 0), 0x15);
    }
}
