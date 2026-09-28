//! C#'s `Ppu.Render.cs`: scroll advance and the line composition, whose CHR reads, sprite 0 and overflow are machine state. See Moon_PPU.md §3, Moon_Native.md §2.5.

use super::timing::increment_coarse_x;
use super::{Ppu, SCREEN_WIDTH, SPRITES_PER_LINE, palette_offset};
use crate::memory::Board;

/// The 64 NTSC colours as RGB triples; emphasis bits are not applied.
pub static NES_PALETTE: [u8; 192] = [
    84, 84, 84, 0, 30, 116, 8, 16, 144, 48, 0, 136, 68, 0, 100, 92, 0, 48, 84, 4, 0, 60, 24, 0, //
    32, 42, 0, 8, 58, 0, 0, 64, 0, 0, 60, 0, 0, 50, 60, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    152, 150, 152, 8, 76, 196, 48, 50, 236, 92, 30, 228, 136, 20, 176, 160, 20, 100, 152, 34, 32, 120, 60, 0, //
    84, 90, 0, 40, 114, 0, 8, 124, 0, 0, 118, 40, 0, 102, 120, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    236, 238, 236, 76, 154, 236, 120, 124, 236, 176, 98, 236, 228, 84, 236, 236, 88, 180, 236, 106, 100, 212, 136, 32, //
    160, 170, 0, 116, 196, 0, 76, 208, 32, 56, 204, 108, 56, 180, 204, 60, 60, 60, 0, 0, 0, 0, 0, 0, //
    236, 238, 236, 168, 204, 236, 188, 188, 236, 212, 178, 236, 236, 174, 236, 236, 174, 212, 236, 180, 176, 228, 196, 144, //
    204, 210, 120, 180, 222, 120, 168, 226, 144, 152, 226, 180, 160, 214, 228, 160, 162, 160, 0, 0, 0, 0, 0, 0,
];

impl Ppu {
    /// Coarse Y wraps at 29, not 31: rows 30 and 31 are the attribute table.
    pub(super) fn increment_y(&mut self) {
        if (self.v & 0x7000) != 0x7000 {
            self.v = self.v.wrapping_add(0x1000);
            return;
        }
        self.v &= !0x7000;
        let mut y = ((self.v & 0x03E0) >> 5) as i32;
        if y == 29 {
            y = 0;
            self.v ^= 0x0800;
        } else if y == 31 {
            y = 0;
        } else {
            y += 1;
        }
        self.v = (self.v & !0x03E0) | ((y << 5) as u16);
    }

    pub(super) fn copy_horizontal(&mut self) {
        self.v = (self.v & !0x041F) | (self.t & 0x041F);
    }

    pub(super) fn copy_vertical(&mut self) {
        self.v = (self.v & !0x7BE0) | (self.t & 0x7BE0);
    }

    pub(super) fn render_scanline(&mut self, line: i32, board: &mut Board) {
        self.bg_line.fill(0);
        if self.show_background() {
            self.render_background(board);
        }
        self.evaluate_sprites(line);
        self.composite(line, board);
    }

    /// 33 tiles, so fine X can shift the first partly off the left edge.
    fn render_background(&mut self, board: &mut Board) {
        let mut v = self.render_v;
        let fine_y = ((v >> 12) & 0x07) as i32;
        let mut screen_x = -(self.fine_x as i32);
        let base = self.background_pattern_base();
        while screen_x < SCREEN_WIDTH as i32 {
            let tile = self.read_ciram(0x2000 | (v & 0x0FFF), board) as i32;
            let attribute_address = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
            let attribute = self.read_ciram(attribute_address, board) as i32;
            let quadrant = (((v >> 4) & 0x04) | (v & 0x02)) as i32;
            let palette_number = (attribute >> quadrant) & 0x03;
            let pattern_address = base + tile * 16 + fine_y;
            let low = board.mapper.read_chr(&board.cart, pattern_address as u16) as i32;
            let high = board.mapper.read_chr(&board.cart, (pattern_address + 8) as u16) as i32;
            for pixel in 0..8 {
                let x = screen_x + pixel;
                if (x as u32) >= SCREEN_WIDTH as u32 {
                    continue;
                }
                let bit = 7 - pixel;
                let color = ((low >> bit) & 0x01) | (((high >> bit) & 0x01) << 1);
                self.bg_line[x as usize] = if color == 0 { 0 } else { (palette_number * 4 + color) as u8 };
            }
            screen_x += 8;
            increment_coarse_x(&mut v);
        }
    }

    /// Eight per line, in OAM order; the ninth only sets the overflow flag.
    fn evaluate_sprites(&mut self, line: i32) {
        *self.sprite_count = 0;
        let height = if self.sprites_are_8x16() { 16 } else { 8 };
        for i in 0..64 {
            let row = line - self.oam[i * 4] as i32 - 1;
            if row < 0 || row >= height {
                continue;
            }
            if *self.sprite_count as usize == SPRITES_PER_LINE {
                self.set_sprite_overflow(true);
                break;
            }
            let n = *self.sprite_count as usize;
            self.sprite_indices[n] = i as i32;
            *self.sprite_count += 1;
        }
    }

    fn composite(&mut self, line: i32, board: &mut Board) {
        let height = if self.sprites_are_8x16() { 16 } else { 8 };
        let frame_offset = line * SCREEN_WIDTH as i32 * 4;
        let show_sprites = self.show_sprites();
        let show_sprites_left = self.show_sprites_left();
        let show_background = self.show_background();
        let show_background_left = self.show_background_left();
        for x in 0..SCREEN_WIDTH as i32 {
            let bg_entry = self.bg_line[x as usize] as i32;
            let bg_opaque = bg_entry != 0 && (x >= 8 || show_background_left);

            let mut sprite_entry = 0;
            let mut sprite_in_front = false;
            let mut sprite_is_zero = false;

            if show_sprites && (x >= 8 || show_sprites_left) {
                for s in 0..*self.sprite_count as usize {
                    let index = self.sprite_indices[s];
                    let sprite_x = self.oam[(index * 4 + 3) as usize] as i32;
                    let column = x - sprite_x;
                    if !(0..8).contains(&column) {
                        continue;
                    }
                    let color = self.sprite_pixel(index, line, column, height, board);
                    if color == 0 {
                        continue;
                    }
                    let attributes = self.oam[(index * 4 + 2) as usize] as i32;
                    sprite_entry = 0x10 + (attributes & 0x03) * 4 + color;
                    sprite_in_front = (attributes & 0x20) == 0;
                    sprite_is_zero = index == 0;
                    break;
                }
            }

            if sprite_is_zero && bg_opaque && x != SCREEN_WIDTH as i32 - 1 && show_background {
                self.set_sprite0_hit(true);
            }

            let entry = if sprite_entry != 0 && (sprite_in_front || !bg_opaque) {
                sprite_entry
            } else if bg_opaque {
                bg_entry
            } else {
                0
            };

            if *self.skip_rendering {
                continue;
            }

            let mut nes_color = self.palette_ram[palette_offset((0x3F00 + entry) as u16)];
            if self.grayscale() {
                nes_color &= 0x30;
            }
            let palette = (nes_color & 0x3F) as usize * 3;
            let pixel = (frame_offset + x * 4) as usize;
            if let Some(out) = self.frame_rgba.get_mut(pixel..pixel + 4) {
                out.copy_from_slice(&[NES_PALETTE[palette], NES_PALETTE[palette + 1], NES_PALETTE[palette + 2], 0xFF]);
            } else {
                crate::fault(crate::Fault::IndexOutOfRange);
            }
        }
    }

    fn sprite_pixel(&mut self, index: i32, line: i32, column: i32, height: i32, board: &mut Board) -> i32 {
        let tile = self.oam[(index * 4 + 1) as usize] as i32;
        let attributes = self.oam[(index * 4 + 2) as usize] as i32;
        let mut row = line - self.oam[(index * 4) as usize] as i32 - 1;
        let mut column = column;
        if (attributes & 0x80) != 0 {
            row = height - 1 - row;
        }
        if (attributes & 0x40) != 0 {
            column = 7 - column;
        }
        let pattern_address = if self.sprites_are_8x16() {
            let table = (tile & 0x01) * 0x1000;
            let mut tile_index = tile & 0xFE;
            if row >= 8 {
                tile_index += 1;
                row -= 8;
            }
            table + tile_index * 16 + row
        } else {
            self.sprite_pattern_base() + tile * 16 + row
        };
        let low = board.mapper.read_chr(&board.cart, pattern_address as u16) as i32;
        let high = board.mapper.read_chr(&board.cart, (pattern_address + 8) as u16) as i32;
        let bit = 7 - column;
        ((low >> bit) & 0x01) | (((high >> bit) & 0x01) << 1)
    }
}
