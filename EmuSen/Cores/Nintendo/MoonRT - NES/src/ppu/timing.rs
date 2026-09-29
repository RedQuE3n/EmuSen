//! C#'s `Ppu.Timing.cs`: the dot-by-dot clock, modelled on Mesen's `NesPpu::Exec`. See Moon_PPU.md §1.

use super::{PRE_RENDER_SCANLINE, Ppu, VBLANK_SCANLINE, VISIBLE_SCANLINES};
use crate::memory::Board;

pub const LAST_DOT: i32 = 340;

impl Ppu {
    #[inline(always)]
    pub fn step(&mut self, dots: i32, board: &mut Board) {
        for _ in 0..dots {
            self.tick(board);
        }
    }

    #[inline(always)]
    fn tick(&mut self, board: &mut Board) {
        self.ppu_clock = self.ppu_clock.wrapping_add(1);
        if self.cycle < LAST_DOT {
            self.cycle += 1;
            self.run_dot(board);
        } else {
            self.begin_scanline();
        }
    }

    fn begin_scanline(&mut self) {
        self.cycle = 0;
        self.scanline = self.scanline.wrapping_add(1);
        if self.scanline > PRE_RENDER_SCANLINE {
            self.scanline = 0;
            self.frame_count = self.frame_count.wrapping_add(1);
            self.frame_complete = true;
            self.decay_open_bus();
        }
    }

    #[inline(always)]
    fn run_dot(&mut self, board: &mut Board) {
        if self.scanline < VISIBLE_SCANLINES {
            self.run_visible_dot(board);
        } else if self.scanline == VBLANK_SCANLINE && self.cycle == 1 {
            if !self.suppress_v_blank {
                self.status |= 0x80;
            }
            self.suppress_v_blank = false;
        } else if self.scanline == PRE_RENDER_SCANLINE {
            self.run_pre_render_dot(board);
        }
    }

    #[inline(always)]
    fn run_visible_dot(&mut self, board: &mut Board) {
        self.fetch_for_dot(board);
        if self.cycle == 256 {
            self.render_scanline(self.scanline, board);
            if self.rendering_enabled() {
                self.increment_y();
            }
        } else if self.cycle == 257 {
            if self.rendering_enabled() {
                self.copy_horizontal();
            }
            self.render_v = self.v;
        }
    }

    fn run_pre_render_dot(&mut self, board: &mut Board) {
        if self.cycle == 1 {
            self.status &= !0xE0;
        }
        self.fetch_for_dot(board);
        if self.cycle == 256 && self.rendering_enabled() {
            self.increment_y();
        } else if self.cycle == 257 {
            if self.rendering_enabled() {
                self.copy_horizontal();
            }
            self.render_v = self.v;
        } else if self.cycle >= 280 && self.cycle <= 304 && self.rendering_enabled() {
            self.copy_vertical();
            self.render_v = self.v;
        } else if self.cycle == 338 {
            *self.skip_last_dot = self.rendering_enabled() && (self.frame_count & 1) != 0;
        } else if self.cycle == 339 && *self.skip_last_dot {
            self.cycle = LAST_DOT;
        }
    }

    /// The addresses hardware would put on the bus, which is what a board watching A12 sees.
    #[inline(always)]
    fn fetch_for_dot(&mut self, board: &mut Board) {
        if !self.rendering_enabled() {
            return;
        }
        let c = self.cycle;
        if (1..=256).contains(&c) || (321..=336).contains(&c) {
            self.background_fetch(board);
        } else if (257..=320).contains(&c) {
            self.sprite_fetch(board);
            self.oam_address = 0;
        } else if c == 337 || c == 339 {
            self.set_bus_address(0x2000 | (self.v & 0x0FFF), board);
        }
    }

    #[inline(always)]
    fn background_fetch(&mut self, board: &mut Board) {
        let v = self.v;
        match self.cycle & 0x07 {
            1 => self.set_bus_address(0x2000 | (v & 0x0FFF), board),
            3 => self.set_bus_address(0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07), board),
            5 | 7 => self.set_bus_address((self.background_pattern_base() as u16) | ((v >> 12) & 0x07), board),
            _ => {}
        }
        if (self.cycle & 0x07) == 0 && self.cycle != 0 {
            let mut v = self.v;
            increment_coarse_x(&mut v);
            self.v = v;
        }
    }

    #[inline(always)]
    fn sprite_fetch(&mut self, board: &mut Board) {
        let v = self.v;
        match (self.cycle - 257) & 0x07 {
            0 => self.set_bus_address(0x2000 | (v & 0x0FFF), board),
            2 => self.set_bus_address(0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07), board),
            4 | 6 => {
                let base = self.sprite_pattern_base_for_line();
                self.set_bus_address((base | 0x08) as u16, board);
            }
            _ => {}
        }
    }

    /// 8x16 sprites take their table from the tile's low bit; the empty slots fetch tile $FF - see Moon_Memory.md §4.6b.
    fn sprite_pattern_base_for_line(&self) -> i32 {
        if !self.sprites_are_8x16() {
            return self.sprite_pattern_base();
        }
        if *self.sprite_count < 8 {
            return 0x1000;
        }
        for s in 0..*self.sprite_count as usize {
            if (self.oam[(self.sprite_indices[s] * 4 + 1) as usize] & 0x01) != 0 {
                return 0x1000;
            }
        }
        0
    }

    #[inline(always)]
    pub(super) fn set_bus_address(&mut self, address: u16, board: &mut Board) {
        self.bus_address = address;
        board.mapper.on_ppu_address(address, self.ppu_clock);
    }
}

#[inline(always)]
pub(super) fn increment_coarse_x(v: &mut u16) {
    if (*v & 0x001F) == 0x001F {
        *v = (*v & !0x001F) ^ 0x0400;
    } else {
        *v = v.wrapping_add(1);
    }
}
