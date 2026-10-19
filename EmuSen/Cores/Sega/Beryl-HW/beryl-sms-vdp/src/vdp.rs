//! The chip (Beryl_SmsVdp.md §2-§3, §5): its ports and their protocol, the registers, VRAM and CRAM, the counters, the
//! interrupts and the status flags, run a line at a time on its own clock, with the picture drawn up to the dot of
//! each write before the write is taken.

use crate::Variant;
use crate::compose::{self, Background, SpriteLine, Vram as _};
use crate::timing::{self, LATCH_DELAY, LINE_CLOCKS, LINE_DOT, NTSC_LINES, PAL_LINES, PIXEL_0_DOT};

/// The status register's bits the chip drives; the rest are the console's undriven bus [R MacDonald §4: they "return
/// garbage values"].
pub const STATUS_BITS: u8 = 0xE0;

/// The width of every picture; its height is the frame's active lines.
pub const WIDTH: usize = 256;
pub const MAX_HEIGHT: usize = 240;

/// A line's events, as the chip's clocks from the V counter's step: the line counter's step, the line's last pixel,
/// the next line's step.
const EVENTS: [u64; 3] = [LINE_DOT as u64 * 2, (PIXEL_0_DOT as u64 + WIDTH as u64) * 2, LINE_CLOCKS];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vdp {
    pub variant: Variant,
    pub pal: bool,
    /// The Game Gear's own mode, its cartridge's choice; the Master System's chips have none.
    pub gg_mode: bool,
    pub regs: [u8; 11],
    pub vram: Vec<u8>,
    /// 32 bytes of `--BBGGRR`; the Game Gear's 64 hold 32 words of `----BBBBGGGGRRRR` in GG mode.
    pub cram: [u8; 64],
    pub address: u16,
    pub code: u8,
    /// The command's first byte has been written.
    pub second: bool,
    pub buffer: u8,
    pub gg_latch: u8,
    pub frame_flag: bool,
    pub line_flag: bool,
    pub overflow: bool,
    pub collision: bool,
    pub line_counter: u8,
    /// The X scroll taken for the line, and the Y scroll for the frame.
    pub hscroll: u8,
    pub vscroll: u8,
    /// The H counter's value at TH's last rise, read at the odd ports $40-$7F.
    pub h_latch: u8,
    /// The chip's clock (the master clock over five), the line it is on and that line's first clock.
    pub clock: u64,
    pub line: u32,
    pub line_start: u64,
    /// The next of the line's `EVENTS` not yet run.
    pub event: u8,
    pub sprites: SpriteLine,
    /// The line's pixels drawn so far.
    pub drawn: u32,
    /// The active lines of the last frame whose picture is whole.
    pub height: u32,
    /// Draw nothing: the counters, flags and interrupts run as when drawing.
    pub skip: bool,
    picture: Vec<u16>,
    scratch: [u8; WIDTH],
}

impl Vdp {
    /// The chip at power-on: registers, VRAM and CRAM clear [C, Beryl_SmsVdp.md §2.4], on the frame's last line, so that a
    /// frame counted from the console's power-on holds one whole picture.
    pub fn new(variant: Variant, pal: bool) -> Vdp {
        let lines = if pal { PAL_LINES } else { NTSC_LINES };
        Vdp {
            variant,
            pal,
            gg_mode: false,
            regs: [0; 11],
            vram: vec![0; 0x4000],
            cram: [0; 64],
            address: 0,
            code: 0,
            second: false,
            buffer: 0,
            gg_latch: 0,
            frame_flag: false,
            line_flag: false,
            overflow: false,
            collision: false,
            line_counter: 0,
            hscroll: 0,
            vscroll: 0,
            h_latch: 0,
            clock: 0,
            line: lines - 1,
            line_start: 0,
            event: 0,
            sprites: SpriteLine::default(),
            drawn: 0,
            height: 192,
            skip: false,
            picture: vec![0; WIDTH * MAX_HEIGHT],
            scratch: [0; WIDTH],
        }
    }

    pub fn lines(&self) -> u32 {
        if self.pal { PAL_LINES } else { NTSC_LINES }
    }

    /// Mode 4 is on: M4 set, and not the "invalid text mode" M1 makes, which on the 315-5246 needs M2 clear [R
    /// MacDonald §6]. The TMS9918's modes are stage 4's.
    pub fn mode4(&self) -> bool {
        let (m4, m2, m1) = (self.regs[0] & 4 != 0, self.regs[0] & 2 != 0, self.regs[1] & 0x10 != 0);
        m4 && !(m1 && (self.variant.masks || !m2))
    }

    /// The display's active lines: 224 or 240 where M2 is set with M1 or M3 alone, on the chips that have them [R
    /// MacDonald §6].
    pub fn active_lines(&self) -> u32 {
        let (m2, m1, m3) = (self.regs[0] & 2 != 0, self.regs[1] & 0x10 != 0, self.regs[1] & 8 != 0);
        match (self.mode4() && m2, m1, m3) {
            (true, true, false) if self.variant.lines_224 => 224,
            (true, false, true) if self.variant.lines_240 => 240,
            _ => 192,
        }
    }

    fn display(&self) -> bool {
        self.regs[1] & 0x40 != 0
    }

    /// INT, held while a pending flag's enable is set [R MacDonald §12].
    pub fn irq(&self) -> bool {
        (self.frame_flag && self.regs[1] & 0x20 != 0) || (self.line_flag && self.regs[0] & 0x10 != 0)
    }

    /// The dot of the line the chip is at.
    fn dot(&self) -> u32 {
        ((self.clock - self.line_start) / 2) as u32
    }

    /// The chip brought to `to`, every event at or before it run.
    pub fn run(&mut self, to: u64) {
        loop {
            let at = self.line_start + EVENTS[self.event as usize];
            if at > to {
                break;
            }
            self.clock = at;
            match self.event {
                0 => self.line_point(),
                1 => self.line_end(),
                _ => self.next_line(),
            }
        }
        self.clock = self.clock.max(to);
    }

    /// The line's sprites, found for the V counter it shows as it steps; the overflow flag only on an active line,
    /// while sprites collide on any line [M FluBBa's VDP test, its "Offscreen Y, no ovr", "Offscreen Y, col" and "Spr
    /// ovr correct HC"].
    fn find_sprites(&mut self) {
        let active = self.active_lines();
        if self.mode4() {
            let v = timing::v_counter(self.line, active, self.pal);
            let over = compose::find_sprites(&self.vram[..], &self.regs, self.variant, v, active == 192, &mut self.sprites);
            self.overflow |= over && self.line < active;
        } else {
            self.sprites.count = 0;
        }
        compose::sprite_pixels(&mut self.sprites);
        self.drawn = 0;
    }

    /// The line's last pixel drawn; after the last active line, the picture is whole.
    fn line_end(&mut self) {
        self.draw(WIDTH as u32);
        let active = self.active_lines();
        if self.line + 1 == active {
            self.height = active;
        }
        self.event = 2;
    }

    /// The line counter steps as each active line and the one after begin, and is reloaded as the others do; its
    /// underflow is the line interrupt [R MacDonald §12; M FluBBa's VDP test, "Line IRQ VCount"]. The X scroll is taken
    /// for the line.
    fn line_point(&mut self) {
        if self.line <= self.active_lines() {
            if self.line_counter == 0 {
                self.line_counter = self.regs[10];
                self.line_flag = true;
            } else {
                self.line_counter -= 1;
            }
        } else {
            self.line_counter = self.regs[10];
        }
        self.hscroll = self.regs[8];
        self.event = 1;
    }

    /// The V counter steps; on the line after the last active one the frame interrupt is raised [R MacDonald §12: on
    /// line $C1 of a 192-line display], and at a frame's first line the Y scroll is taken [R MacDonald §9; Sega's Mark
    /// III manual, "Register 9 is latched in vertical blanking"].
    fn next_line(&mut self) {
        self.line = (self.line + 1) % self.lines();
        self.line_start += LINE_CLOCKS;
        if self.line == self.active_lines() + 1 {
            self.frame_flag = true;
        }
        if self.line == 0 {
            self.vscroll = self.regs[9];
        }
        self.find_sprites();
        self.event = 0;
    }

    /// The line drawn up to the dot the chip is at: what a write now changes shows from the next pixel.
    fn catch_up(&mut self) {
        let x = self.dot().saturating_sub(PIXEL_0_DOT).min(WIDTH as u32);
        self.draw(x);
    }

    /// Pixels `drawn..to` of the line, and the sprite collision among them while the display is on.
    fn draw(&mut self, to: u32) {
        let from = self.drawn;
        if to <= from {
            return;
        }
        self.drawn = to;
        let display = self.display() && self.mode4();
        if display && (from..to).contains(&(self.sprites.collision as u32)) {
            self.collision = true;
        }
        let active = self.active_lines();
        if self.skip || self.line >= active {
            return;
        }
        let row = self.line as usize * WIDTH;
        let backdrop = compose::backdrop(&self.regs);
        if !display {
            // Blanked, or a TMS9918 mode before stage 4: the backdrop (Beryl_SmsVdp.md §4.4).
            let c = self.entry(backdrop);
            self.picture[row + from as usize..row + to as usize].fill(c);
            return;
        }
        let bg = Background { line: self.line, hscroll: self.hscroll, vscroll: self.vscroll, tall_table: active != 192 };
        let span = (from as usize)..(to as usize);
        compose::background(&self.vram[..], &self.regs, self.variant, bg, from, &mut self.scratch[span.clone()]);
        let mask = self.variant.left_mask && self.regs[0] & 0x20 != 0;
        for x in span {
            let i = if mask && x < 8 { backdrop } else { compose::mix(self.scratch[x], self.sprites.pixels[x]) };
            let e = self.entry(i);
            self.picture[row + x] = e;
        }
    }

    /// CRAM's entry for an index: a `--BBGGRR` byte, or in the Game Gear's mode a 12-bit word.
    fn entry(&self, index: u8) -> u16 {
        let i = index as usize & 31;
        if self.gg_mode { u16::from_le_bytes([self.cram[2 * i], self.cram[2 * i + 1]]) & 0x0FFF } else { (self.cram[i] & 0x3F) as u16 }
    }

    /// The last whole picture: `WIDTH` entries a line, `height` lines.
    pub fn picture(&self) -> &[u16] {
        &self.picture[..WIDTH * self.height as usize]
    }

    /// The V counter, at the even ports $40-$7F.
    pub fn read_v(&self) -> u8 {
        timing::v_counter(self.line, self.active_lines(), self.pal)
    }

    /// The H counter as last latched, at the odd ports $40-$7F [R MacDonald §11].
    pub fn read_h(&self) -> u8 {
        self.h_latch
    }

    /// TH rose on either joystick port: the H counter is latched [R MacDonald §11; FluBBa's readme, "Setting the TH pin
    /// to 0 and then 1 will latch"].
    pub fn latch_h(&mut self) {
        let dot = ((self.clock + LATCH_DELAY - self.line_start) / 2) as u32 % timing::DOTS;
        self.h_latch = (timing::h_counter(dot) >> 1) as u8;
    }

    /// The control port's read: the frame, overflow and collision flags, clearing them, the line interrupt's flag and
    /// the command's first byte [R MacDonald §3-§4, §12].
    pub fn read_control(&mut self) -> u8 {
        self.catch_up();
        let v = (self.frame_flag as u8) << 7 | (self.overflow as u8) << 6 | (self.collision as u8) << 5;
        self.frame_flag = false;
        self.line_flag = false;
        self.overflow = false;
        self.collision = false;
        self.second = false;
        v
    }

    /// The data port's read: the buffer, refilled from VRAM at the address, which then steps [R MacDonald §3].
    pub fn read_data(&mut self) -> u8 {
        self.second = false;
        let v = self.buffer;
        self.buffer = self.vram.byte(self.address as u32);
        self.address = (self.address + 1) & 0x3FFF;
        v
    }

    /// The control port's write: the address's low byte at once, then its high six bits and the code; code 0 fills
    /// the buffer, code 2 writes a register, numbers 11-15 doing nothing [R MacDonald §3].
    pub fn write_control(&mut self, v: u8) {
        if !self.second {
            self.address = (self.address & 0x3F00) | v as u16;
            self.second = true;
            return;
        }
        self.second = false;
        self.address = (v as u16 & 0x3F) << 8 | (self.address & 0xFF);
        self.code = v >> 6;
        match self.code {
            0 => {
                self.buffer = self.vram.byte(self.address as u32);
                self.address = (self.address + 1) & 0x3FFF;
            }
            2 if v & 0x0F <= 10 => {
                self.catch_up();
                self.regs[(v & 0x0F) as usize] = self.address as u8;
            }
            _ => {}
        }
    }

    /// The data port's write: CRAM with code 3, else VRAM; the buffer takes the byte, and the address steps [R
    /// MacDonald §3, §5].
    pub fn write_data(&mut self, v: u8) {
        self.catch_up();
        self.second = false;
        self.buffer = v;
        let a = self.address as usize;
        if self.code == 3 {
            if !self.gg_mode {
                self.cram[a & 0x1F] = v & 0x3F;
            } else if a & 1 == 0 {
                self.gg_latch = v;
            } else {
                self.cram[a & 0x3E] = self.gg_latch;
                self.cram[a & 0x3F] = v & 0x0F;
            }
        } else {
            self.vram[a] = v;
        }
        self.address = (self.address + 1) & 0x3FFF;
    }
}
