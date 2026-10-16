//! C#'s `Ppu/Ppu.Timing.cs`: the mode machine, a dot at a time. See Mercury_Ppu.md §2 and §3.

use super::{BASE_DRAWING_CYCLES, OAM_SCAN_CYCLES, Ppu, PpuMode, SCREEN_HEIGHT};

pub const TOTAL_SCANLINES: u8 = 154;
pub const CYCLES_PER_SCANLINE: i32 = 456;
const VBLANK_BIT: u8 = 1 << 0;
const STAT_BIT: u8 = 1 << 1;

impl Ppu {
    pub fn lcd_enabled(&self) -> bool {
        self.lcdc & 0x80 != 0
    }

    pub fn window_enabled(&self) -> bool {
        self.lcdc & 0x20 != 0
    }

    /// Bit 7 is unwired and reads back set; bits 2-0 are the hardware's own state.
    pub fn read_stat(&self) -> u8 {
        (0x80 | self.stat_enables as i32 | if self.lyc_matches() { 0x04 } else { 0 } | self.mode_i32()) as u8
    }

    /// The comparison runs only while the LCD is on; off, it holds what it was (Mercury_Native.md §13.11).
    pub fn lyc_matches(&self) -> bool {
        if self.lcd_enabled() { self.ly == self.lyc } else { self.lyc_while_off }
    }

    /// Version 11's part of the tail after the mixer.
    pub fn write_tail(&self, w: &mut crate::state::StateWriter) {
        w.bool("LycWhileOff", self.lyc_while_off);
    }

    pub fn read_tail(&mut self, r: &mut crate::state::StateReader, version: i32) -> crate::state::StateResult {
        self.lyc_while_off = version >= 11 && r.bool()?; // LycWhileOff
        Ok(())
    }

    pub fn write_stat(&mut self, data: u8, iflags: &mut u8) {
        self.stat_enables = data & 0x78;
        self.update_stat_line(iflags);
    }

    pub fn write_lyc(&mut self, data: u8, iflags: &mut u8) {
        self.lyc = data;
        self.update_stat_line(iflags);
    }

    pub fn write_lcdc(&mut self, data: u8, iflags: &mut u8) {
        let was = self.lcd_enabled();
        self.lcdc = data;
        if was && !self.lcd_enabled() {
            self.disable_lcd();
        } else if !was && self.lcd_enabled() {
            self.enable_lcd(iflags);
        }
    }

    /// One T-cycle; true when a visible line has just entered hblank, which is when an hblank HDMA moves a block.
    #[inline(always)]
    pub fn tick(&mut self, iflags: &mut u8, vram: &[u8], oam: &[u8], cgb: bool) -> bool {
        if !self.lcd_enabled() {
            return false;
        }
        self.dot += 1;
        if self.dot < CYCLES_PER_SCANLINE - 4 && self.dot != OAM_SCAN_CYCLES && self.dot != self.drawing_end {
            return false;
        }
        self.dot_event(iflags, vram, oam, cgb)
    }

    /// The dots where something happens, a few a line: kept out of the copies of the dot loop the CPU's bus inlines (Mercury_Native.md §14.1).
    #[inline(never)]
    fn dot_event(&mut self, iflags: &mut u8, vram: &[u8], oam: &[u8], cgb: bool) -> bool {
        // The line's last machine cycle, where a CGB's line-144 mode-2 interrupt comes early, shares the test for the line's end.
        if self.dot >= CYCLES_PER_SCANLINE - 4 {
            if self.dot >= CYCLES_PER_SCANLINE {
                self.begin_scanline(iflags);
            } else if self.dot == CYCLES_PER_SCANLINE - 4 && self.ly as usize == SCREEN_HEIGHT - 1 && *self.cgb_hardware {
                self.update_stat_line_with(iflags, true);
            }
            return false;
        }
        if self.ly as usize >= SCREEN_HEIGHT {
            return false;
        }
        if self.dot == OAM_SCAN_CYCLES {
            if self.window_enabled() && self.ly == self.wy {
                self.window_triggered = true;
            }
            self.drawing_end = OAM_SCAN_CYCLES + BASE_DRAWING_CYCLES + (self.scx & 0x07) as i32;
            self.enter_mode(PpuMode::Drawing, iflags);
        } else if self.dot == self.drawing_end && self.is_mode(PpuMode::Drawing) {
            let line = self.ly as usize;
            self.render_scanline(line, vram, oam, cgb);
            self.enter_mode(PpuMode::HBlank, iflags);
            return true;
        }
        false
    }

    pub fn is_mode(&self, mode: PpuMode) -> bool {
        self.mode_raw.is_none() && self.mode == mode
    }

    /// Once a line, so kept out of the dot loop (Mercury_Native.md §13.11).
    #[cold]
    #[inline(never)]
    fn begin_scanline(&mut self, iflags: &mut u8) {
        self.dot = 0;
        self.ly = self.ly.wrapping_add(1);
        if self.ly >= TOTAL_SCANLINES {
            self.ly = 0;
            self.window_line = 0;
            self.window_triggered = false;
            self.frame_count += 1;
            self.frame_complete = true;
        }
        if self.ly as usize == SCREEN_HEIGHT {
            // On a DMG the mode-2 source also fires as line 144 begins, with vblank.
            self.mode = PpuMode::VBlank;
            self.mode_raw = None;
            self.update_stat_line_with(iflags, !*self.cgb_hardware);
            *iflags |= VBLANK_BIT;
        } else if (self.ly as usize) < SCREEN_HEIGHT {
            self.enter_mode(PpuMode::OamScan, iflags);
        } else {
            self.update_stat_line(iflags);
        }
    }

    fn enter_mode(&mut self, mode: PpuMode, iflags: &mut u8) {
        self.mode = mode;
        self.mode_raw = None;
        self.update_stat_line(iflags);
    }

    /// Four sources feed one line into the interrupt controller; only its rising edge requests - see Mercury_Ppu.md §3.2.
    fn update_stat_line(&mut self, iflags: &mut u8) {
        self.update_stat_line_with(iflags, false);
    }

    /// With the LCD off the line does not move, so nothing written then raises the interrupt.
    #[inline(never)]
    fn update_stat_line_with(&mut self, iflags: &mut u8, oam_at_vblank: bool) {
        if !self.lcd_enabled() {
            return;
        }
        let line = (self.stat_enables & 0x40 != 0 && self.lyc_matches())
            || (self.stat_enables & 0x20 != 0 && (self.is_mode(PpuMode::OamScan) || oam_at_vblank))
            || (self.stat_enables & 0x10 != 0 && self.is_mode(PpuMode::VBlank))
            || (self.stat_enables & 0x08 != 0 && self.is_mode(PpuMode::HBlank));
        if line && !self.stat_line {
            *iflags |= STAT_BIT;
        }
        self.stat_line = line;
    }

    fn disable_lcd(&mut self) {
        self.lyc_while_off = self.ly == self.lyc;
        self.ly = 0;
        self.dot = 0;
        self.mode = PpuMode::HBlank;
        self.mode_raw = None;
        self.window_line = 0;
        self.window_triggered = false;
        self.clear_screen();
    }

    fn enable_lcd(&mut self, iflags: &mut u8) {
        self.ly = 0;
        self.dot = 0;
        self.window_line = 0;
        self.window_triggered = false;
        self.drawing_end = OAM_SCAN_CYCLES + BASE_DRAWING_CYCLES + (self.scx & 0x07) as i32;
        self.enter_mode(PpuMode::OamScan, iflags);
    }
}
