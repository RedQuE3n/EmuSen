//! The chip (Beryl_SmsVdp.md §2-§3, §5): its ports and their protocol, the registers, VRAM and CRAM, the counters, the
//! interrupts and the status flags, run a line at a time on its own clock, with the picture drawn up to the dot of
//! each write before the write is taken.

use crate::Variant;
use crate::compose::{self, Background, SpriteLine};
use crate::timing::{self, LATCH_DELAY, LINE_CLOCKS, LINE_DOT, NTSC_LINES, PAL_LINES, PIXEL_0_DOT};
use crate::tms;

/// The status register's bits the chip drives; the rest are the console's undriven bus [R MacDonald §4: they "return
/// garbage values"].
pub const STATUS_BITS: u8 = 0xE0;

/// Sprites a line in the TMS9918's modes: the TMS9918A's four [R TI §2.4.5], taken for the Sega chips too
/// (Endou_Disputes.md D-10).
pub const TMS_SPRITES: usize = 4;

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
    /// The TMS9918A's status bits 4-0: the fifth sprite's number, or where the last scan ended [R TI §2.3.3].
    pub fifth: u8,
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
    /// The line's sprites are the TMS9918's, found by `tms`, rather than mode 4's.
    pub tms_sprites: bool,
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
            fifth: 0,
            line_counter: 0,
            hscroll: 0,
            vscroll: 0,
            h_latch: 0,
            clock: 0,
            line: lines - 1,
            line_start: 0,
            event: 0,
            sprites: SpriteLine::default(),
            tms_sprites: false,
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
        !self.variant.tms && m4 && !(m1 && (self.variant.masks || !m2))
    }

    /// One of the TMS9918's modes: M4 clear, or the TMS9918A, which has no M4 [R TI §2.2.1; MacDonald §6].
    pub fn tms_mode(&self) -> bool {
        self.variant.tms || self.regs[0] & 4 == 0
    }

    /// Whether the line's sprites are found and shown: the display on, in mode 4 or a TMS9918 mode with sprites.
    fn sprites_on(&self) -> bool {
        self.display() && (self.mode4() || (self.tms_mode() && tms::mode(&self.regs).sprites()))
    }

    /// The status's bits the chip drives: all eight on the TMS9918A, whose low five name the fifth sprite [R TI §2.3].
    pub fn status_bits(&self) -> u8 {
        if self.variant.tms { 0xFF } else { STATUS_BITS }
    }

    /// The VRAM byte a CPU access at `a` reaches: on the TMS9918A in its 4K mode, register 1's bit 7 clear, the cell its
    /// 4K row and column name [R MacDonald's `sc3000h` §6].
    fn cell(&self, a: u16) -> usize {
        if self.four_k() { tms::address_4k(a as u32) as usize } else { a as usize }
    }

    /// The display's active lines: 224 or 240 where M2 is set with M1 or M3 alone, on the chips that have them [R
    /// MacDonald §6]. The Game Gear's own mode is no exception; its LCD's window is fixed instead (Endou_Disputes.md D-21).
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
        (self.frame_flag && self.regs[1] & 0x20 != 0) || (!self.variant.tms && self.line_flag && self.regs[0] & 0x10 != 0)
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
        let v = timing::v_counter(self.line, active, self.pal);
        self.sprites.count = 0;
        self.tms_sprites = !self.mode4();
        if self.mode4() {
            let over = compose::find_sprites(&self.vram[..], &self.regs, self.variant, v, active == 192, &mut self.sprites);
            self.overflow |= over && self.line < active;
            compose::sprite_pixels(&mut self.sprites);
        } else {
            if self.sprites_on() {
                let scan = if self.four_k() {
                    tms::find_sprites(&tms::Vram4k(&self.vram), &self.regs, self.variant, v, TMS_SPRITES, &mut self.sprites)
                } else {
                    tms::find_sprites(&self.vram[..], &self.regs, self.variant, v, TMS_SPRITES, &mut self.sprites)
                };
                self.fifth_sprite(scan, active);
            }
            tms::sprite_pixels(&mut self.sprites);
        }
        self.drawn = 0;
    }

    /// The fifth sprite's flag on an active line, as mode 4's overflow is; on the TMS9918A only while the frame flag is
    /// clear, with the status's low bits naming the fifth sprite, or where the scan ended while none is flagged [R TI
    /// §2.3.3; Young §4.3].
    fn fifth_sprite(&mut self, scan: tms::Scan, active: u32) {
        if self.line >= active {
            return;
        }
        if !self.variant.tms {
            self.overflow |= scan.fifth.is_some();
        } else if !self.overflow {
            self.fifth = scan.fifth.unwrap_or(scan.last);
            self.overflow = scan.fifth.is_some() && !self.frame_flag;
        }
    }

    /// The TMS9918A in its 4K mode.
    fn four_k(&self) -> bool {
        self.variant.tms && self.regs[1] & 0x80 == 0
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
        if self.sprites_on() && (from..to).contains(&(self.sprites.collision as u32)) {
            self.collision = true;
        }
        let active = self.active_lines();
        if self.skip || self.line >= active {
            return;
        }
        let row = self.line as usize * WIDTH;
        let span = (from as usize)..(to as usize);
        let tms = self.tms_mode();
        if !self.display() || !(tms || self.mode4()) {
            // Blanked, or the invalid text mode, which is not modelled: the backdrop (Beryl_SmsVdp.md §4.4).
            let c = self.border();
            self.picture[row + span.start..row + span.end].fill(c);
            return;
        }
        let (four_k, line) = (self.four_k(), self.line);
        let out = &mut self.scratch[span.clone()];
        if !tms {
            let bg = Background { line: self.line, hscroll: self.hscroll, vscroll: self.vscroll, tall_table: active != 192 };
            compose::line(&self.vram[..], &self.regs, self.variant, bg, &self.sprites.pixels, from, out);
            for x in span {
                self.picture[row + x] = self.entry(self.scratch[x]);
            }
        } else {
            if four_k {
                tms::line(&tms::Vram4k(&self.vram), &self.regs, self.variant, line, &self.sprites.pixels, from, out);
            } else {
                tms::line(&self.vram[..], &self.regs, self.variant, line, &self.sprites.pixels, from, out);
            }
            for x in span {
                self.picture[row + x] = self.tms_entry(self.scratch[x]);
            }
        }
    }

    /// What the chip puts out for a TMS9918 colour code: the code itself on the TMS9918A, whose colours are its
    /// console's; on the Game Gear CRAM's entries 16-31 [R MacDonald §14]; on the Master System's chips a fixed colour of
    /// their 64 [R SMS Power!'s "Palette"].
    fn tms_entry(&self, code: u8) -> u16 {
        let code = code & 0x0F;
        if self.variant.tms {
            code as u16
        } else if self.variant.gg {
            self.entry(16 + code)
        } else {
            tms::SMS_COLOURS[code as usize] as u16
        }
    }

    /// The entry the chip shows where it draws no picture: the backdrop of register 7, in its mode's colours.
    pub fn border(&self) -> u16 {
        if self.tms_mode() { self.tms_entry(tms::backdrop(&self.regs)) } else { self.entry(compose::backdrop(&self.regs)) }
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
    /// the command's first byte [R MacDonald §3-§4, §12]; on the TMS9918A the fifth sprite's number below them [R TI §2.3].
    pub fn read_control(&mut self) -> u8 {
        self.catch_up();
        let low = if self.variant.tms { self.fifth & 0x1F } else { 0 };
        let v = (self.frame_flag as u8) << 7 | (self.overflow as u8) << 6 | (self.collision as u8) << 5 | low;
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
        self.buffer = self.vram[self.cell(self.address)];
        self.address = (self.address + 1) & 0x3FFF;
        v
    }

    /// The control port's write: the address's low byte at once, then its high six bits and the code; code 0 fills
    /// the buffer, code 2 writes a register, numbers 11-15 doing nothing [R MacDonald §3]. The TMS9918A takes bit 7 of
    /// the second byte as a register write of the number in bits 2-0, and has no CRAM [R TI Table 2-1; `sc3000h` §6]; its
    /// first byte is taken at once as the 315-5124's is [A].
    pub fn write_control(&mut self, v: u8) {
        if !self.second {
            self.address = (self.address & 0x3F00) | v as u16;
            self.second = true;
            return;
        }
        self.second = false;
        self.address = (v as u16 & 0x3F) << 8 | (self.address & 0xFF);
        self.code = if self.variant.tms { (v >> 6).min(2) } else { v >> 6 };
        let register = if self.variant.tms { v & 7 } else { v & 0x0F };
        match self.code {
            0 => {
                self.buffer = self.vram[self.cell(self.address)];
                self.address = (self.address + 1) & 0x3FFF;
            }
            2 if register <= 10 => {
                self.catch_up();
                self.regs[register as usize] = self.address as u8;
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
            let cell = self.cell(self.address);
            self.vram[cell] = v;
        }
        self.address = (self.address + 1) & 0x3FFF;
    }
}
