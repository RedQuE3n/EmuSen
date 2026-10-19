//! The TMS9918's modes (Beryl_SmsVdp.md §9): Graphics I and II, Text, Multicolor and the undocumented combinations,
//! and their sprites, composed a line at a time as TMS9918 colour codes, 0 transparent. The 315-5124, the 315-5246 and
//! the TMS9918A share it; what the chip does with a code is the chip's (`Vdp`). Nothing here allocates.

use crate::Variant;
use crate::compose::{Sprite, SpriteLine, Vram};

/// The mode M1, M2 and M3 select, named as MacDonald's §6 and Young's §3 name them: M1 is register 1's bit 4, M2
/// register 0's bit 1, M3 register 1's bit 3 (TI's manual calls the last two M3 and M2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Graphic1,
    Graphic2,
    Text,
    Multicolor,
    /// Mode 1+2: Text with Graphics II's three pattern tables [R Young §3.5].
    Text2,
    /// Mode 2+3: Multicolor with Graphics II's three pattern tables [R Young §3.6].
    Multicolor2,
    /// Modes 1+3 and 1+2+3: forty columns of four pixels of the text colour and two of the backdrop [R Young §3.7].
    Bars,
}

pub fn mode(regs: &[u8; 11]) -> Mode {
    match (regs[1] & 0x10 != 0, regs[0] & 2 != 0, regs[1] & 8 != 0) {
        (false, false, false) => Mode::Graphic1,
        (true, false, false) => Mode::Text,
        (false, true, false) => Mode::Graphic2,
        (true, true, false) => Mode::Text2,
        (false, false, true) => Mode::Multicolor,
        (false, true, true) => Mode::Multicolor2,
        (true, _, true) => Mode::Bars,
    }
}

impl Mode {
    /// Sprites are shown in every mode but those M1 selects [R TI §2.4.5; Young §4].
    pub fn sprites(self) -> bool {
        !matches!(self, Mode::Text | Mode::Text2 | Mode::Bars)
    }
}

/// The colour code a backdrop pixel shows: register 7's low nibble [R TI §2.2.8].
#[inline]
pub fn backdrop(regs: &[u8; 11]) -> u8 {
    regs[7] & 0x0F
}

/// A Graphics II table's address for the byte at `index` (the third's 2 KiB block, the name and the row): register 4's
/// bit 2 or register 3's bit 7 chooses the table's half, and on the chips with `masks` the register's lower bits are an
/// AND mask over the index's high bits [R TI §2.4.2: "The LSBs must be set to all 1s"; Young §3.3; MacDonald §15].
fn split(index: u32, base_bit: u8, mask: u8, mask_shift: u32, masks: bool) -> u32 {
    let base = if base_bit != 0 { 0x2000 } else { 0 };
    base | if masks { index & ((mask as u32) << mask_shift | ((1 << mask_shift) - 1)) } else { index }
}

fn pattern_split(regs: &[u8; 11], variant: Variant, index: u32) -> u32 {
    split(index, regs[4] & 4, regs[4] & 3, 11, variant.masks)
}

/// The background's colour codes for pixels `x0..x0 + out.len()` of `line`, 0 where the pattern plane is transparent.
pub fn background<V: Vram + ?Sized>(vram: &V, regs: &[u8; 11], variant: Variant, line: u32, x0: u32, out: &mut [u8]) {
    let mode = mode(regs);
    let names = (regs[2] as u32 & 0x0F) << 10;
    let (row, y) = (line >> 3, line & 7);
    let text = (regs[7] >> 4, regs[7] & 0x0F);
    for (i, o) in out.iter_mut().enumerate() {
        let x = x0 + i as u32;
        *o = match mode {
            Mode::Text | Mode::Text2 | Mode::Bars => {
                // Forty columns of six pixels between borders of eight [R Young §3.2; TI §2.4.4].
                if !(8..248).contains(&x) {
                    0
                } else {
                    let (column, bit) = ((x - 8) / 6, (x - 8) % 6);
                    let pattern = if mode == Mode::Bars {
                        0xF0
                    } else {
                        let name = vram.byte(names + row * 40 + column) as u32;
                        let index = name << 3 | y;
                        let at = if mode == Mode::Text { (regs[4] as u32 & 7) << 11 | index } else { pattern_split(regs, variant, (row >> 3) << 11 | index) };
                        vram.byte(at)
                    };
                    // The two low bits of a text pattern are not shown [R TI §2.4.4].
                    if pattern << bit & 0x80 != 0 { text.0 } else { text.1 }
                }
            }
            Mode::Graphic1 | Mode::Graphic2 => {
                let name = vram.byte(names + row * 32 + (x >> 3)) as u32;
                let (pattern, colours) = if mode == Mode::Graphic1 {
                    (vram.byte((regs[4] as u32 & 7) << 11 | name << 3 | y), vram.byte((regs[3] as u32) << 6 | name >> 3))
                } else {
                    let index = (row >> 3) << 11 | name << 3 | y;
                    (vram.byte(pattern_split(regs, variant, index)), vram.byte(split(index, regs[3] & 0x80, regs[3] & 0x7F, 6, variant.masks)))
                };
                if pattern << (x & 7) & 0x80 != 0 { colours >> 4 } else { colours & 0x0F }
            }
            Mode::Multicolor | Mode::Multicolor2 => {
                // Two bytes a name, chosen by the row; each a 4 by 4 block left (its high nibble) and right [R TI §2.4.3].
                let name = vram.byte(names + row * 32 + (x >> 3)) as u32;
                let index = name << 3 | (row & 3) << 1 | y >> 2;
                let at = if mode == Mode::Multicolor { (regs[4] as u32 & 7) << 11 | index } else { pattern_split(regs, variant, (row >> 3) << 11 | index) };
                let b = vram.byte(at);
                if x & 4 == 0 { b >> 4 } else { b & 0x0F }
            }
        };
    }
}

/// What the scan of a line's sprites found: the fifth sprite on it, if any, and the number at which the scan ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scan {
    pub fifth: Option<u8>,
    /// The terminator's number, or 31 when the table ran out [R Young §4.3].
    pub last: u8,
}

/// The sprites on the line whose V counter is `v`, at most `limit` of the 32 in table order, into `out` [R TI §2.4.5;
/// Young §4]: a sprite shows from the line after its Y, a Y of $D0 ends the table, 8 or 16 pixels square by register 1's
/// bit 1 and doubled by its bit 0. A sprite is held as `compose::Sprite` with its X after the early clocks, `wide` its
/// magnification, and in `planes` its pattern's left and right bytes for the row, its colour and its size.
pub fn find_sprites<V: Vram + ?Sized>(vram: &V, regs: &[u8; 11], variant: Variant, v: u8, limit: usize, out: &mut SpriteLine) -> Scan {
    let base = (regs[5] as u32 & 0x7F) << 7;
    let (big, mag) = (regs[1] & 2 != 0, regs[1] & 1 != 0);
    let height = (if big { 16 } else { 8 }) << mag as u32;
    // The Sega chips shift every sprite eight pixels left by register 0's bit 3 in these modes too [R MacDonald §15].
    let shift: i16 = if !variant.tms && regs[0] & 8 != 0 { 8 } else { 0 };
    let mut n = 0;
    let mut scan = Scan { fifth: None, last: 31 };
    for i in 0..32 {
        let a = base + 4 * i;
        let y = vram.byte(a);
        if y == 0xD0 {
            scan.last = i as u8;
            break;
        }
        let row = v.wrapping_sub(y).wrapping_sub(1) as u32;
        if row >= height {
            continue;
        }
        if n == limit {
            scan.fifth = Some(i as u8);
            break;
        }
        let (x, name, colour) = (vram.byte(a + 1), vram.byte(a + 2) as u32, vram.byte(a + 3));
        let row = if mag { row >> 1 } else { row };
        let generator = (regs[6] as u32 & 7) << 11;
        let at = if big { generator | (name & 0xFC) << 3 | row } else { generator | name << 3 | row };
        let right = if big { vram.byte(at + 16) } else { 0 };
        // The early clock bit moves a sprite 32 pixels left [R TI §2.4.5].
        let x = x as i16 - if colour & 0x80 != 0 { 32 } else { 0 } - shift;
        out.sprites[n] = Sprite { x, wide: mag, planes: [vram.byte(at), right, colour & 0x0F, big as u8] };
        n += 1;
    }
    out.count = n as u8;
    scan
}

/// The line's sprite pixels as colour codes: the first sprite whose pattern bit is set and whose colour is not 0, and the
/// first pixel where two sprites' set bits meet, whatever their colours [R TI §2.3.2, §2.4]. Nothing past pixel 255 is
/// drawn or collides (Beryl_SmsVdp.md §9.3).
pub fn sprite_pixels(line: &mut SpriteLine) {
    line.pixels.fill(0);
    line.collision = 256;
    let mut set = [false; 256];
    for s in &line.sprites[..line.count as usize] {
        let bits = (s.planes[0] as u16) << 8 | s.planes[1] as u16;
        let size = if s.planes[3] != 0 { 16 } else { 8 };
        let width = size << s.wide as i32;
        for px in 0..width {
            let x = s.x as i32 + px;
            if !(0..256).contains(&x) {
                continue;
            }
            let bit = if s.wide { px >> 1 } else { px };
            if bits << bit & 0x8000 == 0 {
                continue;
            }
            let x = x as usize;
            if set[x] {
                line.collision = line.collision.min(x as u16);
            }
            set[x] = true;
            if line.pixels[x] == 0 {
                line.pixels[x] = s.planes[2];
            }
        }
    }
}

/// Pixels `x0..x0 + out.len()` of `line` as colour codes: a sprite's over the pattern plane's, the backdrop's where both
/// are transparent [R TI §2.4].
pub fn line<V: Vram + ?Sized>(vram: &V, regs: &[u8; 11], variant: Variant, line: u32, sprites: &[u8; 256], x0: u32, out: &mut [u8]) {
    background(vram, regs, variant, line, x0, out);
    let back = backdrop(regs);
    for (i, o) in out.iter_mut().enumerate() {
        let s = sprites[x0 as usize + i];
        *o = if s != 0 { s } else if *o != 0 { *o } else { back };
    }
}

/// The colour the 315-5124 and 315-5246 show for each code, as a `--BBGGRR` entry of their 64 colours [R SMS Power!'s
/// "Palette", its "Master System/Mark III" table].
pub const SMS_COLOURS: [u8; 16] = [0x00, 0x00, 0x08, 0x0C, 0x10, 0x30, 0x01, 0x3C, 0x02, 0x03, 0x05, 0x0F, 0x04, 0x33, 0x15, 0x3F];

/// The TMS9918A's VRAM address in its 4K mode, register 1's bit 7 clear, as the 16K mode's address of the same cell:
/// the row and column the chip puts out for a 4K DRAM [R MacDonald's `sc3000h` §6; Young §2.1].
#[inline]
pub fn address_4k(a: u32) -> u32 {
    (a & 0x3F) | (a >> 12 & 1) << 6 | (a >> 6 & 0x3F) << 7 | (a & 0x2000)
}

/// VRAM as the TMS9918A addresses it in its 4K mode.
pub struct Vram4k<'a>(pub &'a [u8]);

impl Vram for Vram4k<'_> {
    #[inline(always)]
    fn byte(&self, address: u32) -> u8 {
        self.0[address_4k(address & 0x3FFF) as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn regs(r0: u8, r1: u8) -> [u8; 11] {
        // Names at $1800, colours at $2000, patterns at $0000, sprites at $1B00 and $3800.
        [r0, r1, 0x06, 0x80, 0x00, 0x36, 0x07, 0xF4, 0, 0, 0]
    }

    #[test]
    fn m1_m2_and_m3_select_the_modes_as_the_documents_list_them() {
        let m = |r0, r1| mode(&regs(r0, r1));
        assert_eq!([m(0, 0), m(0, 0x10), m(2, 0), m(2, 0x10), m(0, 8), m(2, 8), m(0, 0x18), m(2, 0x18)], [
            Mode::Graphic1,
            Mode::Text,
            Mode::Graphic2,
            Mode::Text2,
            Mode::Multicolor,
            Mode::Multicolor2,
            Mode::Bars,
            Mode::Bars
        ]);
        assert!(Mode::Multicolor.sprites() && !Mode::Text2.sprites());
    }

    #[test]
    fn graphics_i_colours_each_group_of_eight_names_from_its_byte() {
        let mut vram = vec![0u8; 0x4000];
        let r = regs(0, 0x40);
        vram[0x1800 + 32 + 2] = 17;
        vram[17 * 8 + 3] = 0b1010_0000;
        vram[0x2000 + 2] = 0x5C;
        let mut out = [0u8; 8];
        background(&vram[..], &r, Variant::TMS9918A, 11, 16, &mut out);
        assert_eq!(out, [5, 12, 5, 12, 12, 12, 12, 12], "row 1, line 3 of name 17: set bits colour 5, clear ones colour 12");
    }

    #[test]
    fn graphics_ii_masks_its_thirds_on_the_tms9918a_and_the_315_5124_and_not_on_the_315_5246() {
        let mut vram = vec![0u8; 0x4000];
        let mut r = regs(2, 0x40);
        (r[3], r[4]) = (0xFF, 0x03);
        vram[0x1800 + 8 * 32] = 1;
        // Name 1 of the second third, and of the first.
        vram[0x800 + 8] = 0x80;
        vram[0x2000 + 0x800 + 8] = 0x90;
        vram[8] = 0x00;
        vram[0x2000 + 8] = 0xAB;
        let mut out = [0u8; 1];
        background(&vram[..], &r, Variant::TMS9918A, 64, 0, &mut out);
        assert_eq!(out[0], 9, "register 4's bit 0 and register 3's bits set: the second third's tables");
        r[4] = 0x02;
        r[3] = 0x9F;
        background(&vram[..], &r, Variant::TMS9918A, 64, 0, &mut out);
        assert_eq!(out[0], 11, "the TMS9918A's masks clear the third's bit in both tables: the first third's clear pixel");
        background(&vram[..], &r, Variant::SMS1, 64, 0, &mut out);
        assert_eq!(out[0], 11, "both masks clear the third's bit: the first third's tables");
        background(&vram[..], &r, Variant::SMS2, 64, 0, &mut out);
        assert_eq!(out[0], 9, "the 315-5246 masks nothing");
    }

    #[test]
    fn text_draws_forty_columns_of_six_pixels_from_the_patterns_high_bits() {
        let mut vram = vec![0u8; 0x4000];
        let r = regs(0, 0x50);
        vram[0x1800 + 41] = 3;
        vram[3 * 8 + 2] = 0b1000_0111;
        let mut out = [0u8; 20];
        background(&vram[..], &r, Variant::TMS9918A, 10, 0, &mut out);
        assert_eq!(&out[..8], [0; 8], "the left border");
        assert_eq!(&out[14..20], [15, 4, 4, 4, 4, 15], "column 1 from pixel 14: the two low bits are not shown");
        let mut bars = [0u8; 8];
        background(&vram[..], &regs(0, 0x58), Variant::TMS9918A, 0, 8, &mut bars);
        assert_eq!(bars, [15, 15, 15, 15, 4, 4, 15, 15]);
    }

    #[test]
    fn multicolor_shows_two_nibbles_a_byte_and_two_bytes_a_name_row() {
        let mut vram = vec![0u8; 0x4000];
        let r = regs(0, 0x48);
        vram[0x1800 + 33] = 2;
        // Row 1 of the name table reads bytes 2 and 3 of name 2.
        vram[2 * 8 + 2] = 0x6A;
        vram[2 * 8 + 3] = 0x1B;
        let mut out = [0u8; 8];
        background(&vram[..], &r, Variant::TMS9918A, 8, 8, &mut out);
        assert_eq!(out, [6, 6, 6, 6, 10, 10, 10, 10]);
        background(&vram[..], &r, Variant::TMS9918A, 12, 8, &mut out);
        assert_eq!(out, [1, 1, 1, 1, 11, 11, 11, 11]);
    }

    #[test]
    fn four_sprites_a_line_the_fifth_named_and_208_ends_the_table() {
        let mut vram = vec![0u8; 0x4000];
        let r = regs(0, 0x40);
        for i in 0..6 {
            vram[0x1B00 + 4 * i] = 49;
            vram[0x1B00 + 4 * i + 1] = 40 * i as u8;
        }
        let mut line = SpriteLine::default();
        let scan = find_sprites(&vram[..], &r, Variant::TMS9918A, 50, 4, &mut line);
        assert_eq!((line.count, scan.fifth), (4, Some(4)));
        vram[0x1B00 + 16] = 0xD0;
        let scan = find_sprites(&vram[..], &r, Variant::TMS9918A, 50, 4, &mut line);
        assert_eq!((line.count, scan), (4, Scan { fifth: None, last: 4 }));
        let scan = find_sprites(&vram[..], &r, Variant::TMS9918A, 60, 4, &mut line);
        assert_eq!((line.count, scan), (0, Scan { fifth: None, last: 4 }), "line 60 is past an 8-line sprite at 50");
    }

    #[test]
    fn sprites_take_the_early_clock_size_and_magnification_and_collide_whatever_their_colour() {
        let mut vram = vec![0u8; 0x4000];
        let mut r = regs(0, 0x43);
        // Sprite 0: 16 by 16 doubled, name 4 (its left column $3820 + row, its right $3830 + row), colour 0.
        vram[0x1B00..0x1B04].copy_from_slice(&[9, 100, 6, 0x80]);
        vram[0x3800 + 4 * 8] = 0x80;
        vram[0x3800 + 4 * 8 + 16] = 0x01;
        // Sprite 1: over sprite 0's first pixels, colour 7.
        vram[0x1B04..0x1B08].copy_from_slice(&[9, 68, 4, 7]);
        vram[0x1B08] = 0xD0;
        let mut line = SpriteLine::default();
        find_sprites(&vram[..], &r, Variant::TMS9918A, 10, 4, &mut line);
        sprite_pixels(&mut line);
        assert_eq!((line.sprites[0].x, line.count), (68, 2), "the early clock: 100 - 32");
        assert_eq!(line.collision, 68, "a colour 0 sprite still collides");
        assert_eq!((line.pixels[68], line.pixels[70], line.pixels[99]), (7, 0, 7), "sprite 0 is colour 0: sprite 1, of the same pattern, shows");
        r[0] = 8;
        find_sprites(&vram[..], &r, Variant::SMS1, 10, 4, &mut line);
        assert_eq!(line.sprites[0].x, 60, "the Sega chips' register 0 bit 3 moves sprites a further 8");
    }

    #[test]
    fn the_4k_mode_puts_a_row_and_column_of_six_bits_with_the_high_bits_above_them() {
        assert_eq!([address_4k(0x003F), address_4k(0x0040), address_4k(0x1000), address_4k(0x2000), address_4k(0x0FFF)], [0x3F, 0x80, 0x40, 0x2000, 0x1FBF]);
        let mut seen = vec![false; 0x4000];
        for a in 0..0x4000 {
            assert!(!std::mem::replace(&mut seen[address_4k(a) as usize], true), "the map is one to one");
        }
    }
}
