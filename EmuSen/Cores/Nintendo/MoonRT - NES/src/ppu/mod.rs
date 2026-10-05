//! C#'s `Ppu`: the 2C02's registers, its VRAM bus and its state. See Moon_PPU.md.

mod render;
mod timing;

use crate::Skip;
use crate::memory::Board;
use crate::memory::cartridge::mirroring as m;
use crate::state::{StateReader, StateResult, StateWriter};

pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 240;
pub const VISIBLE_SCANLINES: i32 = 240;
pub const VBLANK_SCANLINE: i32 = 241;
pub const PRE_RENDER_SCANLINE: i32 = 261;
pub const TOTAL_SCANLINES: i32 = 262;
pub const OAM_SIZE: usize = 256;
pub const SPRITES_PER_LINE: usize = 8;
pub const OPEN_BUS_DECAY_FRAMES: u8 = 36;
pub const FRAME_BYTES: usize = SCREEN_WIDTH * SCREEN_HEIGHT * 4;

#[derive(Clone, Debug, PartialEq)]
pub struct Ppu {
    pub bus_address: u16,
    pub ciram: Vec<u8>,
    pub control: u8,
    pub cycle: i32,
    pub fine_x: u8,
    pub frame_complete: bool,
    pub frame_count: i64,
    pub mask: u8,
    pub oam: [u8; OAM_SIZE],
    pub oam_address: u8,
    pub open_bus: u8,
    pub open_bus_decay: [u8; 8],
    pub palette_ram: [u8; 32],
    pub ppu_clock: i64,
    pub read_buffer: u8,
    pub render_v: u16,
    pub scanline: i32,
    pub status: u8,
    pub suppress_v_blank: bool,
    pub t: u16,
    pub v: u16,
    pub write_toggle: bool,

    pub frame_rgba: Skip<Vec<u8>>,
    pub skip_rendering: Skip<bool>,
    pub bg_line: Skip<[u8; SCREEN_WIDTH]>,
    pub sprite_indices: Skip<[i32; SPRITES_PER_LINE]>,
    pub sprite_count: Skip<i32>,
    pub skip_last_dot: Skip<bool>,
    /// `RenderingSince`, the dot rendering was last switched on; written in the state's tail (Moon_PPU.md §3.4).
    pub rendering_since: Skip<i64>,
}

impl Default for Ppu {
    fn default() -> Self {
        Ppu {
            bus_address: 0,
            ciram: vec![0; 0x1000],
            control: 0,
            cycle: 0,
            fine_x: 0,
            frame_complete: false,
            frame_count: 0,
            mask: 0,
            oam: [0; OAM_SIZE],
            oam_address: 0,
            open_bus: 0,
            open_bus_decay: [0; 8],
            palette_ram: [0; 32],
            ppu_clock: 0,
            read_buffer: 0,
            render_v: 0,
            scanline: 0,
            status: 0,
            suppress_v_blank: false,
            t: 0,
            v: 0,
            write_toggle: false,
            frame_rgba: Skip(vec![0; FRAME_BYTES]),
            skip_rendering: Skip(false),
            bg_line: Skip([0; SCREEN_WIDTH]),
            sprite_indices: Skip([0; SPRITES_PER_LINE]),
            sprite_count: Skip(0),
            skip_last_dot: Skip(false),
            rendering_since: Skip(i64::MIN),
        }
    }
}

impl Ppu {
    #[inline(always)]
    pub fn nmi_enabled(&self) -> bool {
        (self.control & 0x80) != 0
    }
    #[inline(always)]
    pub fn sprites_are_8x16(&self) -> bool {
        (self.control & 0x20) != 0
    }
    #[inline(always)]
    pub fn background_pattern_base(&self) -> i32 {
        if (self.control & 0x10) != 0 { 0x1000 } else { 0 }
    }
    #[inline(always)]
    pub fn sprite_pattern_base(&self) -> i32 {
        if (self.control & 0x08) != 0 { 0x1000 } else { 0 }
    }
    #[inline(always)]
    fn address_increment(&self) -> u16 {
        if (self.control & 0x04) != 0 { 32 } else { 1 }
    }
    #[inline(always)]
    pub fn grayscale(&self) -> bool {
        (self.mask & 0x01) != 0
    }
    #[inline(always)]
    fn show_background_left(&self) -> bool {
        (self.mask & 0x02) != 0
    }
    #[inline(always)]
    fn show_sprites_left(&self) -> bool {
        (self.mask & 0x04) != 0
    }
    #[inline(always)]
    pub fn show_background(&self) -> bool {
        (self.mask & 0x08) != 0
    }
    #[inline(always)]
    pub fn show_sprites(&self) -> bool {
        (self.mask & 0x10) != 0
    }
    #[inline(always)]
    pub fn rendering_enabled(&self) -> bool {
        (self.mask & 0x18) != 0
    }
    #[inline(always)]
    pub fn vblank_flag(&self) -> bool {
        (self.status & 0x80) != 0
    }
    #[inline(always)]
    fn set_vblank_flag(&mut self, value: bool) {
        self.status = if value { self.status | 0x80 } else { self.status & !0x80 };
    }
    #[inline(always)]
    fn set_sprite0_hit(&mut self, value: bool) {
        self.status = if value { self.status | 0x40 } else { self.status & !0x40 };
    }
    #[inline(always)]
    fn set_sprite_overflow(&mut self, value: bool) {
        self.status = if value { self.status | 0x20 } else { self.status & !0x20 };
    }

    /// `NmiOutput`: the level the PPU drives on the CPU's NMI line.
    #[inline(always)]
    pub fn nmi_output(&self) -> bool {
        self.nmi_enabled() && self.vblank_flag()
    }

    /// Power-on: the soft reset, then everything the RESET line leaves alone.
    pub fn reset(&mut self) {
        self.soft_reset();
        self.status = 0;
        self.oam_address = 0;
        self.v = 0;
        self.t = 0;
        self.fine_x = 0;
        self.frame_count = 0;
        self.cycle = 0;
        self.scanline = 0;
        self.ppu_clock = 0;
        self.frame_complete = false;
        self.bus_address = 0;
        *self.rendering_since = i64::MIN;
        self.ciram.fill(0);
        self.palette_ram.fill(0);
        self.oam.fill(0);
        self.frame_rgba.fill(0);
    }

    /// What the RESET line clears - see Moon_Core.md §6.
    pub fn soft_reset(&mut self) {
        self.control = 0;
        self.mask = 0;
        self.write_toggle = false;
        self.read_buffer = 0;
        self.open_bus = 0;
        self.suppress_v_blank = false;
        self.open_bus_decay.fill(0);
    }

    fn advance_vram_address(&mut self, board: &mut Board) {
        self.v = (self.v.wrapping_add(self.address_increment())) & 0x7FFF;
        self.set_bus_address(self.v & 0x3FFF, board);
    }

    fn refresh_open_bus(&mut self, value: u8, mask: u8) {
        self.open_bus = (self.open_bus & !mask) | (value & mask);
        for bit in 0..8 {
            if (mask & (1 << bit)) != 0 {
                self.open_bus_decay[bit] = OPEN_BUS_DECAY_FRAMES;
            }
        }
    }

    /// Once a frame: the latch leaks a bit at a time - see Moon_PPU.md §2.5.
    pub fn decay_open_bus(&mut self) {
        for bit in 0..8 {
            if self.open_bus_decay[bit] == 0 {
                continue;
            }
            self.open_bus_decay[bit] -= 1;
            if self.open_bus_decay[bit] == 0 {
                self.open_bus &= !(1u8 << bit);
            }
        }
    }

    /// `$2000-$2007` reads, already mirrored down by the bus.
    pub fn read_register(&mut self, register: i32, board: &mut Board) -> u8 {
        match register & 0x07 {
            2 => {
                let value = (self.status & 0xE0) | (self.open_bus & 0x1F);
                if self.scanline == VBLANK_SCANLINE && self.cycle == 0 {
                    self.suppress_v_blank = true;
                }
                self.set_vblank_flag(false);
                self.write_toggle = false;
                self.refresh_open_bus(value, 0xE0);
                value
            }
            4 => {
                let clearing = self.rendering_enabled()
                    && (self.scanline < VISIBLE_SCANLINES || self.scanline == PRE_RENDER_SCANLINE)
                    && (1..=64).contains(&self.cycle);
                let mut value = if clearing { 0xFF } else { self.oam[self.oam_address as usize] };
                if !clearing && (self.oam_address & 0x03) == 0x02 {
                    value &= 0xE3;
                }
                self.refresh_open_bus(value, 0xFF);
                value
            }
            7 => {
                let address = self.v & 0x3FFF;
                if address >= 0x3F00 {
                    let value = (self.open_bus & 0xC0) | (self.read_palette(address) & 0x3F);
                    self.read_buffer = self.read_ciram(address, board);
                    self.advance_vram_address(board);
                    self.refresh_open_bus(value, 0x3F);
                    return value;
                }
                let value = self.read_buffer;
                self.read_buffer = self.read_vram(address, board);
                self.advance_vram_address(board);
                self.refresh_open_bus(value, 0xFF);
                value
            }
            _ => self.open_bus,
        }
    }

    pub fn write_register(&mut self, register: i32, value: u8, board: &mut Board) {
        self.refresh_open_bus(value, 0xFF);
        match register & 0x07 {
            0 => {
                self.control = value;
                self.t = (self.t & 0xF3FF) | (((value & 0x03) as u16) << 10);
            }
            1 => {
                if !self.rendering_enabled() && (value & 0x18) != 0 {
                    *self.rendering_since = self.ppu_clock;
                }
                self.mask = value;
            }
            3 => self.oam_address = value,
            4 => {
                self.oam[self.oam_address as usize] = value;
                self.oam_address = self.oam_address.wrapping_add(1);
            }
            5 => {
                if !self.write_toggle {
                    self.fine_x = value & 0x07;
                    self.t = (self.t & 0xFFE0) | (value >> 3) as u16;
                } else {
                    self.t = (self.t & 0x8FFF) | (((value & 0x07) as u16) << 12);
                    self.t = (self.t & 0xFC1F) | (((value & 0xF8) as u16) << 2);
                }
                self.write_toggle = !self.write_toggle;
            }
            6 => {
                if !self.write_toggle {
                    self.t = (self.t & 0x00FF) | (((value & 0x3F) as u16) << 8);
                } else {
                    self.t = (self.t & 0xFF00) | value as u16;
                    self.v = self.t;
                    self.set_bus_address(self.v & 0x3FFF, board);
                }
                self.write_toggle = !self.write_toggle;
            }
            7 => {
                self.write_vram(self.v & 0x3FFF, value, board);
                self.advance_vram_address(board);
            }
            _ => {}
        }
    }

    pub fn read_vram(&mut self, address: u16, board: &mut Board) -> u8 {
        let address = address & 0x3FFF;
        if address < 0x2000 {
            return board.mapper.read_chr(&board.cart, address);
        }
        if address < 0x3F00 {
            return self.read_ciram(address, board);
        }
        self.read_palette(address)
    }

    pub fn write_vram(&mut self, address: u16, value: u8, board: &mut Board) {
        let address = address & 0x3FFF;
        if address < 0x2000 {
            board.mapper.write_chr(&mut board.cart, address, value);
        } else if address < 0x3F00 {
            if !board.mapper.supplies_nametables() {
                let i = self.nametable_offset(address, board);
                self.ciram[i] = value;
            }
        } else {
            self.palette_ram[palette_offset(address)] = value;
        }
    }

    #[inline(always)]
    fn read_ciram(&self, address: u16, board: &Board) -> u8 {
        if board.mapper.supplies_nametables() {
            board.mapper.read_nametable(&board.cart, address)
        } else {
            self.ciram[self.nametable_offset(address, board)]
        }
    }

    #[inline(always)]
    fn read_palette(&self, address: u16) -> u8 {
        let value = self.palette_ram[palette_offset(address)];
        if self.grayscale() { value & 0x30 } else { value }
    }

    /// `$3000-$3EFF` mirrors `$2000-$2EFF` before the board's own mirroring applies.
    #[inline(always)]
    pub fn nametable_offset(&self, address: u16, board: &Board) -> usize {
        let index = (address as i32 - 0x2000) & 0x0FFF;
        let table = index / 0x0400;
        let offset = index % 0x0400;
        let page = match board.mapper.mirroring(&board.cart) {
            m::HORIZONTAL => table >> 1,
            m::VERTICAL => table & 1,
            m::SINGLE_SCREEN_LOWER => 0,
            m::SINGLE_SCREEN_UPPER => 1,
            _ => table,
        };
        (page * 0x0400 + offset) as usize
    }

    pub fn write_state(&self, w: &mut StateWriter) {
        w.u16("BusAddress", self.bus_address);
        w.bytes("Ciram", &self.ciram);
        w.u8("Control", self.control);
        w.i32("Cycle", self.cycle);
        w.u8("FineX", self.fine_x);
        w.bool("FrameComplete", self.frame_complete);
        w.i64("FrameCount", self.frame_count);
        w.u8("Mask", self.mask);
        w.bytes("Oam", &self.oam);
        w.u8("OamAddress", self.oam_address);
        w.u8("OpenBus", self.open_bus);
        w.bytes("OpenBusDecay", &self.open_bus_decay);
        w.bytes("PaletteRam", &self.palette_ram);
        w.i64("PpuClock", self.ppu_clock);
        w.u8("ReadBuffer", self.read_buffer);
        w.u16("RenderV", self.render_v);
        w.i32("Scanline", self.scanline);
        w.u8("Status", self.status);
        w.bool("SuppressVBlank", self.suppress_v_blank);
        w.u16("T", self.t);
        w.u16("V", self.v);
        w.bool("WriteToggle", self.write_toggle);
    }

    pub fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.bus_address = r.u16()?; // BusAddress
        r.bytes(&mut self.ciram)?; // Ciram
        self.control = r.u8()?; // Control
        self.cycle = r.i32()?; // Cycle
        self.fine_x = r.u8()?; // FineX
        self.frame_complete = r.bool()?; // FrameComplete
        self.frame_count = r.i64()?; // FrameCount
        self.mask = r.u8()?; // Mask
        r.bytes(&mut self.oam)?; // Oam
        self.oam_address = r.u8()?; // OamAddress
        self.open_bus = r.u8()?; // OpenBus
        r.bytes(&mut self.open_bus_decay)?; // OpenBusDecay
        r.bytes(&mut self.palette_ram)?; // PaletteRam
        self.ppu_clock = r.i64()?; // PpuClock
        self.read_buffer = r.u8()?; // ReadBuffer
        self.render_v = r.u16()?; // RenderV
        self.scanline = r.i32()?; // Scanline
        self.status = r.u8()?; // Status
        self.suppress_v_blank = r.bool()?; // SuppressVBlank
        self.t = r.u16()?; // T
        self.v = r.u16()?; // V
        self.write_toggle = r.bool()?; // WriteToggle
        Ok(())
    }
}

/// The four sprite backdrop entries are holes that mirror the background's - see Moon_PPU.md §2.4.
#[inline(always)]
pub fn palette_offset(address: u16) -> usize {
    let mut index = (address & 0x1F) as usize;
    if (index & 0x13) == 0x10 {
        index &= 0x0F;
    }
    index
}
