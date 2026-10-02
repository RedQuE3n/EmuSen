//! The S-CPU's side of the machine: the master clock and its line events, the access speeds, open bus, WRAM and its
//! port, and the I/O decode, from fullsnes ("SNES Memory Map", "SNES Timings") and anomie's open-bus document.
//! Devices later stages build are stubs here. See VenusRT_Native.md §12.

use crate::apu::smp::Smp;
use crate::cart::{Cartridge, Region, Slot};
use crate::cpu::{Bus, pin};
use crate::ppu::{Beam, Ppu};
use crate::scpu::Devices;

pub const LINE: u16 = 1364;
/// The refresh's start, H=133.5 in dots, and its length (fullsnes, "SNES Timing H/V Events").
pub const REFRESH_AT: u16 = 534;
pub const REFRESH: u16 = 40;
pub const VBLANK_LINE: u16 = 225;
/// Where line 225's NMI flag and /NMI line take effect, in the bus's end-of-cycle frame (VenusRT_Disputes.md D-8).
pub const NMI_AT: u16 = 6;

/// The master clock and where it stands in the frame.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Timing {
    pub clock: u64,
    pub line: u16,
    pub line_clock: u16,
    pub field: bool,
    pub frame: u64,
    pub pal: bool,
    pub refreshed: bool,
    pub vblank: bool,
    pub nmi_flag: bool,
    /// $4200 bits 4-5, HTIME and VTIME, and $4211's flag the comparator sets.
    pub irq_mode: u8,
    pub htime: u16,
    pub vtime: u16,
    pub irq_flag: bool,
    /// Set when line 225 reaches H=74.5, for the auto-joypad read to start (anomie's timing document).
    pub joypad_due: bool,
    /// HDMA's init on line 0 and its transfer on this line are done.
    pub hdma_init_done: bool,
    pub hdma_line_done: bool,
    /// The line clock of the next thing a cycle must stop for: an event of this line, or its end. A cycle that ends
    /// before it only moves the clock (VenusRT_Plan.md §5.2's scheduler). Zero until first scheduled; not in the state.
    pub next_event: u16,
    /// SETINI bit 0, which the line count turns on; the register is the PPU's, so this is not in the state.
    pub interlace: bool,
}

/// Where in a line the H comparator fires for an HTIME: 14 clocks past the dot, the two long dots counted, and HTIME
/// 0 at 10 clocks, which is 1374 past the previous line's dot 0 (anomie's timing document).
pub fn irq_point(htime: u16) -> u16 {
    if htime == 0 {
        return 10;
    }
    14 + 4 * htime + if htime > 323 { 2 } else { 0 } + if htime > 327 { 2 } else { 0 }
}

/// Where the flag and the CPU's line take effect in the bus's end-of-cycle frame: four clocks after the flag's point.
pub fn irq_line_point(htime: u16) -> u16 {
    irq_point(htime) + 4
}

pub const JOYPAD_AT: u16 = 298;
pub const HDMA_INIT_AT: u16 = 24;
pub const HDMA_AT: u16 = 278 * 4;

impl Timing {
    /// 1364 master clocks, but 1360 for line 240 of field 1 at 60 Hz without interlace.
    #[inline]
    pub fn line_length(&self) -> u16 {
        if !self.pal && !self.interlace && self.field && self.line == 240 { LINE - 4 } else { LINE }
    }

    pub fn lines(&self) -> u16 {
        // Interlaced frames with the field flag clear have a line more (anomie's timing document, "Clocks & Refresh").
        (if self.pal { 312 } else { 262 }) + (self.interlace && !self.field) as u16
    }

    #[inline]
    pub fn advance(&mut self, clocks: u16) {
        let mut left = clocks;
        while left > 0 {
            let len = self.line_length();
            let step = left.min(len - self.line_clock);
            let (from, to) = (self.line_clock, self.line_clock + step);
            if self.irq_mode != 0 {
                self.compare(from, to, len);
            }
            if self.line == VBLANK_LINE && from < JOYPAD_AT && JOYPAD_AT <= to {
                self.joypad_due = true;
            }
            if self.line == VBLANK_LINE && from < NMI_AT && NMI_AT <= to {
                self.nmi_flag = true;
            }
            self.clock += step as u64;
            self.line_clock = to;
            left -= step;
            if self.line_clock >= len {
                self.line_clock = 0;
                self.next_line();
            }
        }
    }

    /// The H/V comparator over the clocks (from, to] of this line; a point past the line's end falls in the next.
    fn compare(&mut self, from: u16, to: u16, len: u16) {
        let h = if self.irq_mode == 2 { 0 } else { self.htime };
        // No IRQ for dot 153 on the short line or on a frame's last line (anomie's timing document).
        if h > 339 || (h == 153 && (len != LINE || self.line == self.lines() - 1)) {
            return;
        }
        let at = irq_line_point(h);
        // A point past its line's end falls in the next line, but not over a frame's end or the short line's (D-20).
        if at >= len && (self.line == 0 || (self.line == 241 && !self.pal && !self.interlace && self.field)) {
            return;
        }
        let (at, v) = if at >= len { (at - len, self.line - 1) } else { (at, self.line) };
        let line_matches = self.irq_mode == 1 || v == self.vtime;
        if line_matches && from < at && at <= to {
            self.irq_flag = true;
        }
    }

    fn next_line(&mut self) {
        self.line += 1;
        self.refreshed = false;
        self.hdma_line_done = false;
        if self.line == VBLANK_LINE {
            self.vblank = true;
        }
        if self.line == self.lines() {
            self.line = 0;
            self.field = !self.field;
            self.frame += 1;
            self.vblank = false;
            self.nmi_flag = false;
            self.hdma_init_done = false;
        }
    }

    /// The earliest of this line's pending events and its end.
    pub fn schedule(&mut self) {
        let mut next = self.line_length();
        if !self.refreshed {
            next = next.min(REFRESH_AT);
        }
        if self.line == 0 && !self.hdma_init_done {
            next = next.min(HDMA_INIT_AT);
        }
        if self.line < VBLANK_LINE && !self.hdma_line_done {
            next = next.min(HDMA_AT);
        }
        if self.line == VBLANK_LINE && JOYPAD_AT > self.line_clock {
            next = next.min(JOYPAD_AT);
        }
        if self.line == VBLANK_LINE && NMI_AT > self.line_clock {
            next = next.min(NMI_AT);
        }
        if self.irq_mode != 0 {
            let h = if self.irq_mode == 2 { 0 } else { self.htime };
            if h <= 339 {
                let at = irq_line_point(h);
                let at = if at >= self.line_length() { at - self.line_length() } else { at };
                if at > self.line_clock {
                    next = next.min(at);
                }
            }
        }
        self.next_event = next;
    }

    pub fn hblank(&self) -> bool {
        self.line_clock >= 274 * 4 || self.line_clock < 4
    }
}

/// What the CPU's bus reaches: the cartridge, WRAM, the I/O registers and the master clock.
#[derive(Clone, Debug)]
pub struct System {
    pub cart: Cartridge,
    pub wram: Box<[u8]>,
    pub wram_address: u32,
    /// The data bus's last value, which an unmapped read returns (anomie's MDR).
    pub mdr: u8,
    /// $420D bit 0: banks $80-$FF's ROM at 6 master clocks instead of 8.
    pub fast_rom: bool,
    pub timing: Timing,
    /// $4200-$43FF as last written, for the devices later steps build.
    pub io: Box<[u8]>,
    pub ppu: Ppu,
    /// The line the PPU last finished, so that it is told of every line's end.
    pub ppu_line: u16,
    pub dev: Devices,
    /// The sound unit, run behind the S-CPU and caught up at a port access and the frame's end.
    pub apu: Smp,
    /// CPU writes to $2100-$43FF since power-on, which the header's reset-handler evidence counts; not in the state.
    pub io_writes: u32,
    /// The cartridge's IRQ line to the S-CPU (the SA-1's), as last caught up.
    pub cart_irq: bool,
}

impl System {
    /// Runs the SA-1 to the S-CPU's clock and takes its IRQ line.
    #[inline]
    pub fn catch_up_sa1(&mut self) {
        let clock = self.timing.clock;
        let c = &mut self.cart;
        if let Some(sa1) = c.sa1.as_mut() {
            sa1.run_to(clock, &c.rom, &mut c.sram);
            self.cart_irq = sa1.snes_irq();
        }
    }

    pub fn new(cart: Cartridge) -> System {
        let pal = cart.header.region() == Region::Pal;
        System {
            cart,
            wram: vec![0; 0x20000].into(),
            wram_address: 0,
            mdr: 0,
            fast_rom: false,
            timing: Timing { pal, ..Timing::default() },
            io: {
                // WRIO is all ones at power-on, which is what lets $2137 latch the counters (fullsnes).
                let mut io = vec![0u8; 0x400];
                io[0x201] = 0xFF;
                io.into()
            },
            ppu: Ppu::default(),
            ppu_line: 0,
            dev: Devices::default(),
            apu: Smp::new(None, pal),
            io_writes: 0,
            cart_irq: false,
        }
    }

    /// Master clocks a CPU access to `address` takes (fullsnes, "SNES Memory Map" and MEMSEL).
    #[inline]
    pub fn speed(&self, address: u32) -> u16 {
        let bank = (address >> 16) as u8;
        let offset = address as u16;
        if bank & 0x40 == 0 {
            match offset {
                0x0000..=0x1FFF => 8,
                0x2000..=0x3FFF => 6,
                0x4000..=0x41FF => 12,
                0x4200..=0x5FFF => 6,
                0x6000..=0x7FFF => 8,
                _ => if bank & 0x80 != 0 && self.fast_rom { 6 } else { 8 },
            }
        } else if bank & 0x80 != 0 && self.fast_rom {
            6
        } else {
            8
        }
    }

    /// A CPU cycle: first whatever pauses the CPU at its boundary (HDMA, DMA, the refresh at the first boundary past
    /// H=133.5), then the cycle's own clocks.
    #[inline]
    fn clock_cycle(&mut self, clocks: u16) {
        self.dev.nmi_at_cycle = self.dev.nmi_pending && !self.dev.nmi_hold;
        self.dev.nmi_hold = false;
        self.dev.irq_at_cycle = self.timing.irq_flag || self.cart_irq;
        let t = &mut self.timing;
        if t.line_clock + clocks < t.next_event && self.dev.dma_wait == 0 {
            t.line_clock += clocks;
            t.clock += clocks as u64;
            return;
        }
        self.before_cycle(clocks);
        self.timing.advance(clocks);
        self.after_clock();
        self.timing.schedule();
    }

    /// Where the raster stands: the line, and the dot of the line's clock with the two long dots counted.
    pub fn beam(&self) -> Beam {
        self.beam_at(self.timing.line_clock)
    }

    /// The beam at a clock of the current line; a clock before the line's start is the previous line's end.
    pub fn beam_at(&self, clock: u16) -> Beam {
        let t = &self.timing;
        if clock >= LINE {
            let previous = if t.line == 0 { t.lines() - 1 } else { t.line - 1 };
            let c = clock.wrapping_add(LINE);
            return Beam { line: previous, dot: Self::dot_of(c, LINE), vblank: previous >= VBLANK_LINE, field: t.field, pal: t.pal };
        }
        Beam { line: t.line, dot: Self::dot_of(clock, t.line_length()), vblank: t.vblank, field: t.field, pal: t.pal }
    }

    fn dot_of(c: u16, len: u16) -> u16 {
        if len != LINE || c < 1292 {
            c / 4
        } else if c < 1298 {
            323
        } else if c < 1310 {
            324 + (c - 1298) / 4
        } else if c < 1316 {
            327
        } else {
            328 + (c - 1316) / 4
        }
    }

    /// A read's value, or None where nothing drives the bus. `side_effects` false is the debugger's look.
    pub fn read_value(&mut self, address: u32, side_effects: bool) -> Option<u8> {
        let bank = (address >> 16) as u8;
        let offset = address as u16;
        if self.cart.sa1.is_some() && crate::chips::sa1::Sa1::snes_maps(address) {
            if side_effects {
                self.catch_up_sa1();
            }
            let c = &mut self.cart;
            let v = c.sa1.as_mut()?.snes_read(address, &c.rom, &c.sram, side_effects);
            return v;
        }
        if bank & 0xFE == 0x7E {
            return Some(self.wram[(address & 0x1FFFF) as usize]);
        }
        if bank & 0x40 == 0 {
            match offset {
                0x0000..=0x1FFF => return Some(self.wram[offset as usize]),
                0x2180 => {
                    let v = self.wram[self.wram_address as usize];
                    if side_effects {
                        self.wram_address = (self.wram_address + 1) & 0x1FFFF;
                    }
                    return Some(v);
                }
                0x2100..=0x213F => {
                    let beam = if offset == 0x2137 { self.beam_at(self.timing.line_clock.wrapping_sub(4)) } else { self.beam() };
                    return self.ppu.read(offset as u8, beam, self.io[0x201], side_effects);
                }
                0x2140..=0x217F => {
                    if side_effects {
                        self.apu.run_to(self.timing.clock);
                    }
                    return Some(self.apu.cpu_read((offset & 3) as usize));
                }
                0x2000..=0x3FFF => return None,
                0x4210 => {
                    let v = (if self.timing.nmi_flag { 0x80 } else { 0 }) | (self.mdr & 0x70) | 0x02;
                    // A read ending before HC=10 of line 225 leaves the flag set (D-8).
                    let held = self.timing.line == VBLANK_LINE && self.timing.line_clock < NMI_AT + 4;
                    if side_effects && !held {
                        self.timing.nmi_flag = false;
                        self.after_clock();
                    }
                    return Some(v);
                }
                0x4016 | 0x4017 | 0x4211..=0x421F => return self.read_scpu(offset, side_effects),
                0x4300..=0x437F if offset & 0x0F <= 0x0B => return Some(self.io[(offset - 0x4000) as usize]),
                0x4000..=0x5FFF => return None,
                0x6000..=0x7FFF if !matches!(self.cart.decode(address), Some(_)) => return None,
                _ => {}
            }
        }
        match self.cart.decode(address)? {
            Slot::Rom(i) => Some(self.cart.rom[i]),
            Slot::Sram(i) => Some(self.cart.sram[i]),
            Slot::Dsp(port) => {
                let clock = self.timing.clock;
                let (dsp, _) = self.cart.dsp.as_mut()?;
                if side_effects {
                    dsp.run_to(clock);
                }
                Some(dsp.host_read(port, side_effects))
            }
        }
    }

    pub(crate) fn write_value(&mut self, address: u32, value: u8) {
        let bank = (address >> 16) as u8;
        let offset = address as u16;
        if self.cart.sa1.is_some() && crate::chips::sa1::Sa1::snes_maps(address) {
            self.catch_up_sa1();
            let c = &mut self.cart;
            if let Some(sa1) = c.sa1.as_mut() {
                sa1.snes_write(address, value, &c.rom, &mut c.sram);
                self.cart_irq = sa1.snes_irq();
            }
            return;
        }
        if bank & 0xFE == 0x7E {
            self.wram[(address & 0x1FFFF) as usize] = value;
            return;
        }
        if bank & 0x40 == 0 {
            if (0x2100..0x4400).contains(&offset) {
                self.io_writes += 1;
            }
            match offset {
                0x0000..=0x1FFF => {
                    self.wram[offset as usize] = value;
                    return;
                }
                0x2100..=0x213F => {
                    let beam = self.beam();
                    self.ppu.write(offset as u8, value, beam);
                    self.timing.interlace = self.ppu.regs[0x33] & 1 != 0;
                }
                0x4201 => {
                    // Bit 7 falling latches the counters, as a read of $2137 does (fullsnes, OPHCT).
                    if self.io[0x201] & 0x80 != 0 && value & 0x80 == 0 {
                        let beam = self.beam();
                        self.ppu.latch(beam);
                    }
                    self.io[0x201] = value;
                }
                0x2180 => {
                    self.wram[self.wram_address as usize] = value;
                    self.wram_address = (self.wram_address + 1) & 0x1FFFF;
                }
                0x2140..=0x217F => {
                    self.apu.run_to(self.timing.clock);
                    self.apu.cpu_write((offset & 3) as usize, value);
                }
                0x2181 => self.wram_address = (self.wram_address & 0x1FF00) | value as u32,
                0x2182 => self.wram_address = (self.wram_address & 0x100FF) | (value as u32) << 8,
                0x2183 => self.wram_address = (self.wram_address & 0x0FFFF) | ((value as u32 & 1) << 16),
                0x420D => {
                    self.fast_rom = value & 1 != 0;
                    self.io[0x20D] = value;
                }
                0x4016 | 0x4200 | 0x4202..=0x420B => self.write_scpu(offset, value),
                0x4000..=0x43FF => self.io[(offset - 0x4000) as usize] = value,
                _ => {}
            }
            if offset < 0x8000 && !(0x6000..0x8000).contains(&offset) {
                return;
            }
        }
        match self.cart.decode(address) {
            Some(Slot::Sram(i)) => self.cart.sram[i] = value,
            Some(Slot::Dsp(port)) => {
                let clock = self.timing.clock;
                if let Some((dsp, _)) = self.cart.dsp.as_mut() {
                    dsp.run_to(clock);
                    dsp.host_write(port, value);
                }
            }
            _ => {}
        }
    }
}

impl Bus for System {
    #[inline]
    fn read(&mut self, address: u32, _pins: u8) -> u8 {
        let clocks = self.speed(address);
        self.clock_cycle(clocks);
        let v = self.read_value(address, true).unwrap_or(self.mdr);
        self.mdr = v;
        self.math_tick();
        v
    }

    /// A write with neither VDA nor VPA selects no memory: an internal cycle, which leaves the MDR alone too.
    #[inline]
    fn write(&mut self, address: u32, value: u8, pins: u8) {
        if pins & (pin::VDA | pin::VPA) == 0 {
            self.clock_cycle(6);
            self.math_tick();
            return;
        }
        let clocks = self.speed(address);
        self.clock_cycle(clocks);
        self.math_tick();
        self.mdr = value;
        self.write_value(address, value);
    }

    #[inline]
    fn idle(&mut self, _address: u32, _pins: u8) {
        self.clock_cycle(6);
        self.math_tick();
    }

    fn halted(&mut self) {
        self.clock_cycle(6);
        self.math_tick();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn system(map_mode: u8) -> System {
        let mut rom = vec![0u8; 0x10_0000];
        let at = if map_mode & 1 == 1 { 0xFFC0 } else { 0x7FC0 };
        rom[at + 0x15] = map_mode;
        rom[at + 0x16] = 0x02;
        rom[at + 0x18] = 0x03;
        rom[at + 0x1C..at + 0x20].copy_from_slice(&[0xFF, 0xFF, 0x00, 0x00]);
        System::new(Cartridge::new(&rom).unwrap())
    }

    // fullsnes's table, region by region, and MEMSEL moving only banks $80-$FF's ROM.
    #[test]
    fn each_region_takes_its_documented_master_clocks() {
        let mut s = system(0x20);
        let table = [(0x00_0000, 8), (0x00_1FFF, 8), (0x00_2100, 6), (0x00_3FFF, 6), (0x00_4016, 12), (0x00_41FF, 12), (0x00_4200, 6), (0x00_5FFF, 6),
            (0x00_6000, 8), (0x00_8000, 8), (0x80_8000, 8), (0x40_0000, 8), (0x7E_0000, 8), (0xC0_0000, 8), (0x80_0000, 8)];
        for (a, c) in table {
            assert_eq!(s.speed(a), c, "{a:06X}");
        }
        s.write(0x00_420D, 1, pin::VDA);
        for (a, c) in [(0x80_8000, 6), (0xC0_0000, 6), (0xFF_FFFF, 6), (0x00_8000, 8), (0x40_0000, 8), (0x7F_0000, 8), (0x80_1000, 8), (0x80_6000, 8)] {
            assert_eq!(s.speed(a), c, "{a:06X} with FastROM");
        }
    }

    #[test]
    fn a_line_holds_one_refresh_and_a_field_one_short_line() {
        let mut s = system(0x20);
        let mut refreshes = 0;
        let start_of = |s: &System| s.timing.clock - s.timing.line_clock as u64;
        while s.timing.frame < 1 {
            s.idle(0, 0);
        }
        let one = start_of(&s);
        while s.timing.frame < 3 {
            let before = s.timing.refreshed;
            s.idle(0, 0);
            refreshes += (!before && s.timing.refreshed) as u32;
        }
        assert_eq!(start_of(&s) - one, 2 * 262 * LINE as u64 - 4);
        assert_eq!(refreshes, 2 * 262);
    }

    #[test]
    fn open_bus_is_the_last_value_and_the_wram_port_increments() {
        let mut s = system(0x20);
        s.write(0x7E_1234, 0x5A, pin::VDA);
        assert_eq!(s.read(0x00_5000, pin::VDA), 0x5A);
        assert_eq!(s.read(0x00_1234, pin::VDA), 0x5A);
        for (r, v) in [(0x2181, 0x34), (0x2182, 0x12), (0x2183, 0x00)] {
            s.write(r, v, pin::VDA);
        }
        assert_eq!(s.read(0x00_2180, pin::VDA), 0x5A);
        s.write(0x00_2180, 0xA5, pin::VDA);
        assert_eq!(s.wram[0x1235], 0xA5);
        s.write(0x00_1000, 0x77, 0);
        assert_eq!((s.wram[0x1000], s.mdr), (0, 0xA5));
        s.write(0x70_0010, 0x42, pin::VDA);
        assert_eq!(s.read(0xF0_0010, pin::VDA), 0x42);
    }
}
