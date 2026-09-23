//! The texture image, the eight tiles, and loads from RDRAM into texture memory: C#'s `Rdp.TextureMemory.cs`.

use super::fill::quarters;
use super::{Rdp, RdpMemory, TextureTile};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum LoadKind {
    Tile,
    Block,
    Palette,
}

impl TextureTile {
    /// A tile with no mask always clamps.
    #[inline(always)]
    pub fn clamps_s(&self) -> bool {
        self.clamp_s || self.mask_s == 0
    }

    #[inline(always)]
    pub fn clamps_t(&self) -> bool {
        self.clamp_t || self.mask_t == 0
    }

    /// A mirror wraps at no more than ten bits.
    #[inline(always)]
    pub fn mirror_bit_s(&self) -> i32 {
        if self.mask_s <= 10 { self.mask_s } else { 10 }
    }

    #[inline(always)]
    pub fn mirror_bit_t(&self) -> i32 {
        if self.mask_t <= 10 { self.mask_t } else { 10 }
    }

    #[inline(always)]
    pub fn clamp_limit_s(&self) -> i32 {
        ((self.sh >> 2) - (self.sl >> 2)) & 0x3FF
    }

    #[inline(always)]
    pub fn clamp_limit_t(&self) -> i32 {
        ((self.th >> 2) - (self.tl >> 2)) & 0x3FF
    }
}

/// The 32-bit word at an index of RDRAM, big-endian, or zero past its end.
#[inline(always)]
fn image_word(index: i32, mem: &RdpMemory) -> u32 {
    let at = (index & 0x3F_FFFF) as usize * 4;
    if at + 3 < mem.texture_len() { mem.texture_be32(at) } else { 0 }
}

/// The eight bytes from the image pointer, read through the two 32-bit words that hold them and a following pair.
#[inline(always)]
fn image_window(pointer: i32, mem: &RdpMemory) -> u64 {
    let word = (pointer >> 2) & !1;
    let first = ((image_word(word, mem) as u64) << 32) | image_word(word + 1, mem) as u64;
    let second = ((image_word(word + 2, mem) as u64) << 32) | image_word(word + 3, mem) as u64;

    let offset = (pointer & 7) * 8;
    if offset == 0 { first } else { (first << offset) | (second >> (64 - offset)) }
}

impl Rdp {
    pub(super) fn set_texture_image(&mut self, word: u64) {
        self.texture_image_size = ((word >> 51) & 3) as i32;
        self.texture_image_width = ((word >> 32) & 0x3FF) as i32 + 1;
        self.texture_image = (word as u32) & 0x00FF_FFFF;
    }

    pub(super) fn set_tile(&mut self, word: u64) {
        self.multiple.tiles_changed = true;
        let tile = &mut self.tiles[((word >> 24) & 7) as usize];
        tile.format = ((word >> 53) & 7) as i32;
        tile.size = ((word >> 51) & 3) as i32;
        tile.line = ((word >> 41) & 0x1FF) as i32;
        tile.memory = ((word >> 32) & 0x1FF) as i32;
        tile.palette = ((word >> 20) & 0xF) as i32;
        tile.clamp_t = ((word >> 19) & 1) != 0;
        tile.mirror_t = ((word >> 18) & 1) != 0;
        tile.mask_t = ((word >> 14) & 0xF) as i32;
        tile.shift_t = ((word >> 10) & 0xF) as i32;
        tile.clamp_s = ((word >> 9) & 1) != 0;
        tile.mirror_s = ((word >> 8) & 1) != 0;
        tile.mask_s = ((word >> 4) & 0xF) as i32;
        tile.shift_s = (word & 0xF) as i32;
    }

    pub(super) fn set_tile_size(&mut self, word: u64) -> usize {
        self.multiple.tiles_changed = true;
        let index = ((word >> 24) & 7) as usize;
        let tile = &mut self.tiles[index];
        tile.sl = quarters(word >> 44);
        tile.tl = quarters(word >> 32);
        tile.sh = quarters(word >> 12);
        tile.th = quarters(word);
        index
    }

    /// A tile or palette load walks rows of image texels four quarters apart; a block load walks one row whose t steps by a slope.
    pub(super) fn load(&mut self, word: u64, kind: LoadKind, mem: &RdpMemory) {
        let index = self.set_tile_size(word);
        let tile = self.tiles[index];
        let block = kind == LoadKind::Block;

        let (top, bottom, first, last, right) = if block {
            let row = tile.tl & 0x3FF;
            (row << 2, (row << 2) | 3, row, row, tile.sh)
        } else {
            (tile.tl, tile.th | 3, tile.tl >> 2, tile.th >> 2, tile.sh >> 2)
        };

        if kind == LoadKind::Palette && last > first {
            return;
        }

        let left = if block { (tile.sl << 20) >> 20 } else { tile.sl >> 2 };
        let s_step = (0x200 >> (if block { self.texture_image_size + 2 } else { self.texture_image_size })) << 16;
        let t_step = if block { (tile.th << 8) & !0x1F } else { 0 };

        for row in first..=last {
            let valid = (row << 2).max(top) < ((row << 2) + 4).min(bottom);
            let t = ((tile.tl << 3) << 16).wrapping_add(if block { 0 } else { (row - first).wrapping_mul(0x20 << 16) });
            self.load_row(&tile, row, if valid { right } else { 0 }, left, (tile.sl << 3) << 16, t, s_step, t_step, kind, mem);
        }
    }

    /// An eight-byte window of the image goes to four words of texture memory per step; a four-bit image loads nothing.
    #[allow(clippy::too_many_arguments)]
    fn load_row(&mut self, tile: &TextureTile, row: i32, right: i32, left: i32, mut s: i32, mut t: i32, s_step: i32, t_step: i32, kind: LoadKind, mem: &RdpMemory) {
        let (image_advance, texel_advance) = match self.texture_image_size {
            1 => (8, 8),
            2 => {
                if kind == LoadKind::Palette {
                    (2, 1)
                } else {
                    (8, 4)
                }
            }
            3 => (8, 2),
            _ => (0, 0),
        };
        if texel_advance == 0 {
            return;
        }

        let layout = if tile.format == 1 {
            0
        } else if tile.format == 0 && tile.size == 3 {
            1
        } else {
            2
        };
        let mut pointer = (self.texture_image as i32)
            .wrapping_add((self.texture_image_width.wrapping_mul(row).wrapping_add(left) << self.texture_image_size) >> 1);
        let length = (right - left + 1) & 0xFFF;
        let shift = if kind == LoadKind::Tile { 5 } else { 3 };

        let mut n = 0;
        while n < length {
            let ts = (((s >> 16) & 0xFFFF) as i16 as i32 - (tile.sl << 3)) >> shift;
            let tt = (((t >> 16) & 0xFFFF) as i16 as i32 - (tile.tl << 3)) >> shift;

            let mut window = image_window(pointer, mem);
            if kind == LoadKind::Palette && (pointer & 1) == 0 {
                window = (window >> 48) * 0x0001_0001_0001_0001;
            }

            self.store_texels(tile, ts, tt, window, layout);

            s = s.wrapping_add(s_step) & !0x1F;
            t = t.wrapping_add(t_step) & !0x1F;
            pointer = pointer.wrapping_add(image_advance);
            n += texel_advance;
        }
    }

    /// Four consecutive words sorted into four banks by their low two bits, odd texel rows swapping pairs of banks.
    fn store_texels(&mut self, tile: &TextureTile, s: i32, t: i32, window: u64, layout: i32) {
        let words = (if tile.size == 1 || tile.format == 1 {
            s >> 1
        } else if tile.size >= 2 {
            s
        } else {
            s >> 2
        }) & 0x7FF;

        let row = tile.line.wrapping_mul(t).wrapping_add(tile.memory);
        let first = ((row << 2).wrapping_add(words)) & 0x7FD;
        let high = (first & 0x400) != 0;
        let odd_row = (t & 1) != 0;

        let mut bank = [0i32; 4];
        for i in 0..4 {
            let index = ((first + i) & 0x7FF) ^ if odd_row { 2 } else { 0 };
            bank[(index & 3) as usize] = index & 0x3FF;
        }

        if layout == 2 {
            let half = if high { 0x400 } else { 0 };
            let swapped = odd_row;
            self.write_texture_word(bank[0] | half, (window >> if swapped { 16 } else { 48 }) as i32);
            self.write_texture_word(bank[1] | half, (window >> if swapped { 0 } else { 32 }) as i32);
            self.write_texture_word(bank[2] | half, (window >> if swapped { 48 } else { 16 }) as i32);
            self.write_texture_word(bank[3] | half, (window >> if swapped { 32 } else { 0 }) as i32);
            return;
        }

        let (low, upper) = if layout == 0 {
            (
                ((((window >> 56) & 0xFF) << 24) | (((window >> 40) & 0xFF) << 16) | (((window >> 24) & 0xFF) << 8) | ((window >> 8) & 0xFF)) as u32,
                ((((window >> 48) & 0xFF) << 24) | (((window >> 32) & 0xFF) << 16) | (((window >> 16) & 0xFF) << 8) | (window & 0xFF)) as u32,
            )
        } else {
            ((((window >> 48) << 16) | ((window >> 16) & 0xFFFF)) as u32, ((((window >> 32) & 0xFFFF) << 16) | (window & 0xFFFF)) as u32)
        };

        let alternate = ((words & 2) != 0) ^ odd_row;
        let one = if alternate { bank[2] } else { bank[0] };
        let two = if alternate { bank[3] } else { bank[1] };
        self.write_texture_word(one, (low >> 16) as i32);
        self.write_texture_word(two, low as i32);
        self.write_texture_word(one | 0x400, (upper >> 16) as i32);
        self.write_texture_word(two | 0x400, upper as i32);
    }

    #[inline(always)]
    fn write_texture_word(&mut self, index: i32, value: i32) {
        self.multiple.texture_memory_changed = true;
        let at = index as usize * 2;
        self.texture_memory[at] = (value >> 8) as u8;
        self.texture_memory[at + 1] = value as u8;
    }
}
