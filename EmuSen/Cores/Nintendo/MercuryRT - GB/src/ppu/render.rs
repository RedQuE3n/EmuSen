//! C#'s `Ppu/Ppu.Render.cs` and `Ppu.Cgb.cs`: line composition and the colour palettes. See Mercury_Ppu.md §4-§6 and Mercury_Cgb.md §2-§3.

use super::{PALETTE_RAM_SIZE, Ppu, SCREEN_WIDTH};

const SPRITES_PER_LINE: usize = 10;
const SPRITE_COUNT: usize = 40;
const VRAM_BANK_STRIDE: usize = 0x2000;

/// Four neutral greys, not the panel's green cast - see Mercury_Ppu.md §6.
pub const DMG_SHADES: [u8; 4] = [0xFF, 0xAA, 0x55, 0x00];

impl Ppu {
    /// The window, sprites and the window's own line counter, which advances even when rendering is skipped.
    pub(super) fn render_scanline(&mut self, line: usize, vram: &[u8], oam: &[u8], cgb: bool) {
        let window_on_this_line = self.window_enabled() && self.window_triggered && self.wx <= 166;
        if !*self.skip_rendering {
            let mut bg_color = [0u8; SCREEN_WIDTH];
            let mut bg_priority = [false; SCREEN_WIDTH];
            self.render_background(line, window_on_this_line, vram, cgb, &mut bg_color, &mut bg_priority);
            if self.lcdc & 0x02 != 0 {
                self.render_sprites(line, vram, oam, cgb, &bg_color, &bg_priority);
            }
        }
        if window_on_this_line {
            self.window_line = self.window_line.wrapping_add(1);
        }
    }

    fn tile_row_address(&self, tile: u8, row: usize) -> usize {
        let base = if self.lcdc & 0x10 != 0 { tile as isize * 16 } else { 0x1000 + (tile as i8 as isize) * 16 };
        (base + row as isize * 2) as usize
    }

    fn render_background(&mut self, line: usize, window: bool, vram: &[u8], cgb: bool, bg_color: &mut [u8; SCREEN_WIDTH], bg_priority: &mut [bool; SCREEN_WIDTH]) {
        if !cgb && self.lcdc & 0x01 == 0 {
            for x in 0..SCREEN_WIDTH {
                self.write_shade(line, x, false, 0, 0);
            }
            return;
        }
        let window_start_x = self.wx as i32 - 7;
        let background_y = (line + self.scy as usize) & 0xFF;
        let window_map = if self.lcdc & 0x40 != 0 { 0x1C00 } else { 0x1800 };
        let bg_map = if self.lcdc & 0x08 != 0 { 0x1C00 } else { 0x1800 };

        for x in 0..SCREEN_WIDTH {
            let in_window = window && x as i32 >= window_start_x;
            let map_base = if in_window { window_map } else { bg_map };
            let map_x = if in_window { (x as i32 - window_start_x) as usize } else { (x + self.scx as usize) & 0xFF };
            let map_y = if in_window { self.window_line as usize } else { background_y };

            let map_entry = map_base + ((map_y >> 3) << 5) + ((map_x >> 3) & 0x1F);
            let tile = vram[map_entry];
            let attributes = if cgb { vram[map_entry + VRAM_BANK_STRIDE] } else { 0 };

            let mut row = map_y & 7;
            if attributes & 0x40 != 0 {
                row = 7 - row;
            }
            let address = self.tile_row_address(tile, row) + if attributes & 0x08 != 0 { VRAM_BANK_STRIDE } else { 0 };
            let column = map_x & 7;
            let bit = if attributes & 0x20 != 0 { column } else { 7 - column };
            let color = ((vram[address] >> bit) & 1) | (((vram[address + 1] >> bit) & 1) << 1);

            bg_color[x] = color;
            bg_priority[x] = attributes & 0x80 != 0;
            if cgb {
                color_pixel(&mut self.frame_rgba, line, x, &self.bg_palette_ram, (attributes & 7) as usize, color as usize);
            } else {
                self.write_shade(line, x, false, 0, shade(self.bgp, color));
            }
        }
    }

    fn render_sprites(&mut self, line: usize, vram: &[u8], oam: &[u8], cgb: bool, bg_color: &[u8; SCREEN_WIDTH], bg_priority: &[bool; SCREEN_WIDTH]) {
        let height: i32 = if self.lcdc & 0x04 != 0 { 16 } else { 8 };
        let mut indices = [0usize; SPRITES_PER_LINE];
        let mut count = 0;
        for i in 0..SPRITE_COUNT {
            if count >= SPRITES_PER_LINE {
                break;
            }
            let y = oam[i * 4] as i32 - 16;
            if (line as i32) < y || line as i32 >= y + height {
                continue;
            }
            indices[count] = i;
            count += 1;
        }
        if count == 0 {
            return;
        }
        // A stable insertion sort on X, so equal X keeps the lower OAM index in front; a CGB keeps OAM order.
        if !cgb {
            for i in 1..count {
                let index = indices[i];
                let x = oam[index * 4 + 1];
                let mut j = i;
                while j > 0 && oam[indices[j - 1] * 4 + 1] > x {
                    indices[j] = indices[j - 1];
                    j -= 1;
                }
                indices[j] = index;
            }
        }

        let mut claimed = [false; SCREEN_WIDTH];
        for &index in &indices[..count] {
            let entry = index * 4;
            let sprite_y = oam[entry] as i32 - 16;
            let sprite_x = oam[entry + 1] as i32 - 8;
            let mut tile = oam[entry + 2];
            let attributes = oam[entry + 3];
            let mut row = line as i32 - sprite_y;
            if attributes & 0x40 != 0 {
                row = height - 1 - row;
            }
            if height == 16 {
                tile &= 0xFE;
            }
            let mut address = tile as usize * 16 + row as usize * 2;
            if cgb && attributes & 0x08 != 0 {
                address += VRAM_BANK_STRIDE;
            }
            let (low, high) = (vram[address], vram[address + 1]);
            let palette = if attributes & 0x10 != 0 { self.obp1 } else { self.obp0 };
            let behind = attributes & 0x80 != 0;

            for column in 0..8 {
                let x = sprite_x + column;
                if x < 0 || x as usize >= SCREEN_WIDTH || claimed[x as usize] {
                    continue;
                }
                let x = x as usize;
                let bit = if attributes & 0x20 != 0 { column } else { 7 - column };
                let color = ((low >> bit) & 1) | (((high >> bit) & 1) << 1);
                if color == 0 {
                    continue;
                }
                claimed[x] = true;
                let background_wins = bg_color[x] != 0 && if cgb { self.lcdc & 0x01 != 0 && (behind || bg_priority[x]) } else { behind };
                if background_wins {
                    continue;
                }
                if cgb {
                    color_pixel(&mut self.frame_rgba, line, x, &self.obj_palette_ram, (attributes & 7) as usize, color as usize);
                } else {
                    self.write_shade(line, x, true, ((attributes & 0x10) >> 4) as usize, shade(palette, color));
                }
            }
        }
    }

    /// A DMG shade: grey on a Game Boy, and on a Game Boy Color the colour at that index of the palette the boot ROM chose (Mercury_Model.md §4.1).
    #[inline(always)]
    fn write_shade(&mut self, line: usize, x: usize, object: bool, palette: usize, shade: u8) {
        if *self.compat {
            let ram = if object { &self.obj_palette_ram } else { &self.bg_palette_ram };
            color_pixel(&mut self.frame_rgba, line, x, ram, palette, shade as usize);
        } else {
            self.write_pixel(line, x, shade);
        }
    }

    #[inline(always)]
    fn write_pixel(&mut self, line: usize, x: usize, shade: u8) {
        let offset = (line * SCREEN_WIDTH + x) * 4;
        let v = DMG_SHADES[shade as usize];
        self.frame_rgba[offset..offset + 4].copy_from_slice(&[v, v, v, 0xFF]);
    }

    pub fn read_bg_palette_index(&self) -> u8 {
        self.bg_palette_index | 0x40
    }

    pub fn read_obj_palette_index(&self) -> u8 {
        self.obj_palette_index | 0x40
    }

    pub fn write_bg_palette_index(&mut self, data: u8) {
        self.bg_palette_index = data & 0xBF;
    }

    pub fn write_obj_palette_index(&mut self, data: u8) {
        self.obj_palette_index = data & 0xBF;
    }

    pub fn read_bg_palette_data(&self) -> u8 {
        self.bg_palette_ram[(self.bg_palette_index & 0x3F) as usize]
    }

    pub fn read_obj_palette_data(&self) -> u8 {
        self.obj_palette_ram[(self.obj_palette_index & 0x3F) as usize]
    }

    pub fn write_bg_palette_data(&mut self, data: u8) {
        self.bg_palette_ram[(self.bg_palette_index & 0x3F) as usize] = data;
        self.bg_palette_index = advance(self.bg_palette_index);
    }

    pub fn write_obj_palette_data(&mut self, data: u8) {
        self.obj_palette_ram[(self.obj_palette_index & 0x3F) as usize] = data;
        self.obj_palette_index = advance(self.obj_palette_index);
    }
}

#[inline(always)]
fn color_pixel(frame: &mut [u8], line: usize, x: usize, palette_ram: &[u8; PALETTE_RAM_SIZE], palette: usize, color: usize) {
    let entry = palette * 8 + color * 2;
    let rgb555 = palette_ram[entry] as u32 | ((palette_ram[entry + 1] as u32) << 8);
    let offset = (line * SCREEN_WIDTH + x) * 4;
    frame[offset..offset + 4].copy_from_slice(&[expand(rgb555 & 0x1F), expand((rgb555 >> 5) & 0x1F), expand((rgb555 >> 10) & 0x1F), 0xFF]);
}

fn shade(palette: u8, color: u8) -> u8 {
    (palette >> (color * 2)) & 0x03
}

/// Bit 7 makes a write step the index.
fn advance(index: u8) -> u8 {
    if index & 0x80 != 0 { 0x80 | (index.wrapping_add(1) & 0x3F) } else { index }
}

/// Five bits to eight, replicating the top three so full scale stays full.
fn expand(channel: u32) -> u8 {
    ((channel << 3) | (channel >> 2)) as u8
}
