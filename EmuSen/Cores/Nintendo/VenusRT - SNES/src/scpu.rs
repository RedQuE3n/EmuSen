//! The S-CPU's devices: DMA and HDMA, NMI and the H/V IRQ, the multiply and divide unit, and the joypad ports, from
//! fullsnes ("SNES DMA Transfers", "SNES Maths Multiply/Divide", "SNES Controllers I/O Ports") and anomie's timing
//! document for their timing. See VenusRT_Native.md §13.

use crate::bus::{HDMA_AT, HDMA_INIT_AT, REFRESH, REFRESH_AT, System};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Math {
    #[default]
    Idle,
    Multiply,
    Divide,
}

#[derive(Clone, Debug, Default)]
pub struct Devices {
    pub nmitimen: u8,
    /// The CPU's NMI input as last seen, and the edge it latched; the machine takes the NMI between instructions.
    pub nmi_seen: bool,
    pub nmi_pending: bool,
    pub wrmpya: u8,
    pub wrmpyb: u8,
    pub wrdiv: u16,
    pub wrdivb: u8,
    pub rddiv: u16,
    pub rdmpy: u16,
    pub math: Math,
    /// The unit's internal shifter: WRMPYB moving left for a product, WRDIVB moving right for a quotient.
    pub shifter: u32,
    pub math_step: u8,
    /// The step the write that started the unit does not take.
    pub math_fresh: bool,
    /// $420B's channels waiting for their pause, and the CPU cycles until it.
    pub dma_pending: u8,
    pub dma_wait: u8,
    pub hdma_active: u8,
    pub hdma_transfer: u8,
    /// $4218-$421F, the auto-read's busy end, the manual strobe and the two ports' shift registers.
    pub joy: [u16; 4],
    pub joy_busy_until: u64,
    pub strobe: bool,
    pub shift: [u16; 2],
    pub pads: [u16; 2],
    /// The interrupt lines as the final cycle of the instruction found them, which is what the CPU's check sees.
    pub nmi_at_cycle: bool,
    pub irq_at_cycle: bool,
    /// A stand-in for the sound CPU's boot ROM until stage 4: the CPU's writes to $2140-$2143, and whether its kick came.
    pub apu_stub: [u8; 4],
    pub apu_written: bool,
}

/// The B-bus addresses of a DMA unit's bytes, by its mode.
const UNIT: [&[u8]; 8] = [&[0], &[0, 1], &[0, 0], &[0, 0, 1, 1], &[0, 1, 2, 3], &[0, 1, 0, 1], &[0, 0], &[0, 0, 1, 1]];

/// A pad's buttons in `PadButton` order, as the serial order puts them: B first into bit 15 (fullsnes's table).
pub fn serial(buttons: u16) -> u16 {
    (0..12).fold(0, |w, i| if buttons & (1 << i) != 0 { w | (0x8000 >> i) } else { w })
}

impl System {
    fn reg(&self, channel: usize, r: usize) -> u8 {
        self.io[0x300 + channel * 16 + r]
    }

    fn set_reg(&mut self, channel: usize, r: usize, v: u8) {
        self.io[0x300 + channel * 16 + r] = v;
    }

    fn reg16(&self, channel: usize, r: usize) -> u16 {
        self.reg(channel, r) as u16 | (self.reg(channel, r + 1) as u16) << 8
    }

    fn set_reg16(&mut self, channel: usize, r: usize, v: u16) {
        self.set_reg(channel, r, v as u8);
        self.set_reg(channel, r + 1, (v >> 8) as u8);
    }

    /// One byte between the buses, with DMA's own decode: the B-bus at $21xx, the A-bus never at $2100-$21FF,
    /// $4300-$437F or $420B/$420C, which read open bus and drop writes (fullsnes's DMA notes).
    fn move_byte(&mut self, a_address: u32, b: u8, to_b: bool) {
        let b_address = 0x2100 | b as u32;
        let a_blocked = a_address & 0x40_0000 == 0 && matches!(a_address as u16, 0x2100..=0x21FF | 0x4300..=0x437F | 0x420B | 0x420C);
        if to_b {
            let v = if a_blocked { self.mdr } else { self.read_value(a_address, true).unwrap_or(self.mdr) };
            self.mdr = v;
            self.write_value(b_address, v);
        } else {
            let v = self.read_value(b_address, true).unwrap_or(self.mdr);
            self.mdr = v;
            if !a_blocked {
                self.write_value(a_address, v);
            }
        }
    }

    /// The pause around a transfer: to a multiple of 8 master clocks first, then to a whole cycle of the speed the CPU
    /// resumes at, counted from the pause's start (anomie's DMA timing).
    fn align_before(&mut self) -> u64 {
        let start = self.timing.clock;
        let to8 = 8 - (start % 8) as u16;
        self.advance_paused(to8);
        start
    }

    fn align_after(&mut self, start: u64, resume: u16) {
        let elapsed = (self.timing.clock - start) as u16;
        let r = resume.max(1);
        let pad = r - (elapsed % r);
        self.advance_paused(pad);
    }

    /// Time with the CPU paused: the clock and its line events, but no math step and no CPU-side events.
    fn advance_paused(&mut self, clocks: u16) {
        self.timing.advance(clocks);
        self.after_clock();
    }

    fn run_dma(&mut self, resume: u16) {
        let start = self.align_before();
        self.advance_paused(8);
        for channel in 0..8 {
            if self.dev.dma_pending & (1 << channel) == 0 {
                continue;
            }
            self.advance_paused(8);
            let params = self.reg(channel, 0);
            let b = self.reg(channel, 1);
            let bank = (self.reg(channel, 4) as u32) << 16;
            let mut address = self.reg16(channel, 2);
            let mut count = self.reg16(channel, 5);
            let unit = UNIT[(params & 7) as usize];
            let step: i32 = match (params >> 3) & 3 {
                0 => 1,
                2 => -1,
                _ => 0,
            };
            let mut i = 0usize;
            loop {
                self.hdma_if_due(resume);
                if !self.timing.refreshed && self.timing.line_clock >= REFRESH_AT {
                    self.timing.refreshed = true;
                    self.advance_paused(REFRESH);
                }
                self.move_byte(bank | address as u32, b.wrapping_add(unit[i % unit.len()]), params & 0x80 == 0);
                self.advance_paused(8);
                address = (address as i32 + step) as u16;
                count = count.wrapping_sub(1);
                i += 1;
                if count == 0 {
                    break;
                }
            }
            self.set_reg16(channel, 2, address);
            self.set_reg16(channel, 5, 0);
        }
        self.dev.dma_pending = 0;
        self.io[0x20B] = 0;
        self.align_after(start, resume);
    }

    fn hdma_if_due(&mut self, resume: u16) {
        let t = &self.timing;
        if t.line == 0 && !t.hdma_init_done && t.line_clock >= HDMA_INIT_AT {
            self.hdma_init(resume);
        }
        let t = &self.timing;
        if t.line < 225 && !t.hdma_line_done && t.line_clock >= HDMA_AT {
            self.hdma_line(resume);
        }
    }

    /// The next entry of a channel's table: the line counter, and for indirect HDMA the data pointer.
    fn hdma_load(&mut self, channel: usize) -> u16 {
        let bank = (self.reg(channel, 4) as u32) << 16;
        let mut table = self.reg16(channel, 8);
        let ntrl = self.read_value(bank | table as u32, true).unwrap_or(self.mdr);
        table = table.wrapping_add(1);
        self.set_reg(channel, 0xA, ntrl);
        let mut cost = 0;
        if self.reg(channel, 0) & 0x40 != 0 && ntrl != 0 {
            let lo = self.read_value(bank | table as u32, true).unwrap_or(self.mdr);
            let hi = self.read_value(bank | table.wrapping_add(1) as u32, true).unwrap_or(self.mdr);
            table = table.wrapping_add(2);
            self.set_reg(channel, 5, lo);
            self.set_reg(channel, 6, hi);
            cost = 16;
        }
        self.set_reg16(channel, 8, table);
        if ntrl == 0 {
            self.dev.hdma_active &= !(1 << channel);
        }
        self.dev.hdma_transfer |= 1 << channel;
        cost
    }

    /// V=0, H=6: every enabled channel reloads its table and first entry (anomie: ~18 clocks, 8 a direct channel,
    /// 24 an indirect one).
    fn hdma_init(&mut self, resume: u16) {
        self.timing.hdma_init_done = true;
        let enabled = self.io[0x20C];
        // No channel has ended yet; one enabled later in the frame transfers at once only if an init ran (fullsnes's
        // two cases of HDMA started mid-frame).
        self.dev.hdma_active = 0xFF;
        self.dev.hdma_transfer = if enabled == 0 { 0 } else { 0xFF };
        if enabled == 0 {
            return;
        }
        let start = self.align_before();
        self.advance_paused(18);
        for channel in 0..8 {
            if enabled & (1 << channel) == 0 {
                continue;
            }
            let reload = self.reg16(channel, 2);
            self.set_reg16(channel, 8, reload);
            let cost = self.hdma_load(channel);
            self.advance_paused(8 + cost);
        }
        self.align_after(start, resume);
    }

    /// H=278 of a visible line: each active channel's unit when its entry says so, then its counter (anomie: ~18
    /// clocks a line, 8 a channel, 16 for a new indirect pointer, 8 a byte).
    fn hdma_line(&mut self, resume: u16) {
        self.timing.hdma_line_done = true;
        let active = self.dev.hdma_active & self.io[0x20C];
        if active == 0 {
            return;
        }
        let start = self.align_before();
        self.advance_paused(18);
        for channel in 0..8 {
            if active & (1 << channel) == 0 {
                continue;
            }
            self.dev.dma_pending &= !(1 << channel);
            self.advance_paused(8);
            if self.dev.hdma_transfer & (1 << channel) != 0 {
                let params = self.reg(channel, 0);
                let b = self.reg(channel, 1);
                let indirect = params & 0x40 != 0;
                for &offset in UNIT[(params & 7) as usize] {
                    let a = if indirect {
                        let at = self.reg16(channel, 5);
                        self.set_reg16(channel, 5, at.wrapping_add(1));
                        ((self.reg(channel, 7) as u32) << 16) | at as u32
                    } else {
                        let at = self.reg16(channel, 8);
                        self.set_reg16(channel, 8, at.wrapping_add(1));
                        ((self.reg(channel, 4) as u32) << 16) | at as u32
                    };
                    self.move_byte(a, b.wrapping_add(offset), params & 0x80 == 0);
                    self.advance_paused(8);
                }
            }
            let ntrl = self.reg(channel, 0xA);
            let left = (ntrl & 0x7F).wrapping_sub(1) & 0x7F;
            self.set_reg(channel, 0xA, (ntrl & 0x80) | left);
            if ntrl & 0x80 != 0 { self.dev.hdma_transfer |= 1 << channel } else { self.dev.hdma_transfer &= !(1 << channel) }
            if left == 0 {
                let cost = self.hdma_load(channel);
                self.advance_paused(cost);
            }
        }
        self.align_after(start, resume);
    }

    /// Before a CPU cycle of `clocks`: HDMA, the refresh, and a DMA whose pause has come.
    pub(crate) fn before_cycle(&mut self, clocks: u16) {
        self.hdma_if_due(clocks);
        if self.dev.dma_pending != 0 && self.dev.dma_wait > 0 {
            self.dev.dma_wait -= 1;
            if self.dev.dma_wait == 0 {
                self.run_dma(clocks);
                self.hdma_if_due(clocks);
            }
        }
        if !self.timing.refreshed && self.timing.line_clock >= REFRESH_AT {
            self.timing.refreshed = true;
            self.advance_paused(REFRESH);
        }
    }

    /// After any advance of the clock: the NMI edge, the joypad read's start, the vblank end of HDMA.
    pub(crate) fn after_clock(&mut self) {
        while self.ppu_line != self.timing.line {
            let next = if self.ppu_line + 1 >= self.timing.lines() { 0 } else { self.ppu_line + 1 };
            self.ppu.end_line(self.ppu_line, next);
            self.ppu_line = next;
        }
        let nmi = self.timing.nmi_flag && self.dev.nmitimen & 0x80 != 0;
        if nmi && !self.dev.nmi_seen {
            self.dev.nmi_pending = true;
        }
        self.dev.nmi_seen = nmi;
        if self.timing.vblank {
            self.dev.hdma_active = 0;
        }
        if self.timing.joypad_due {
            self.timing.joypad_due = false;
            if self.dev.nmitimen & 1 != 0 {
                for p in 0..2 {
                    self.dev.joy[p] = serial(self.dev.pads[p]);
                }
                self.dev.joy[2] = 0;
                self.dev.joy[3] = 0;
                self.dev.shift = [0xFFFF; 2];
                self.dev.joy_busy_until = self.timing.clock + 4224;
            }
        }
    }

    /// One CPU cycle's step of the unit, which the CPU clock drives (fullsnes), between a read's sampling and a write's
    /// latch in the same cycle, as jonasquinn's `muldiv_tests` notes and timing tests describe it: a product adds the shifter to RDMPY when RDDIV's low bit is set and shifts RDDIV right; a quotient
    /// shifts the divisor right, takes it from RDMPY when it fits and shifts the bit into RDDIV.
    pub(crate) fn math_tick(&mut self) {
        if self.dev.math_fresh {
            self.dev.math_fresh = false;
            return;
        }
        let d = &mut self.dev;
        match d.math {
            Math::Idle => return,
            Math::Multiply => {
                if d.rddiv & 1 != 0 {
                    d.rdmpy = d.rdmpy.wrapping_add(d.shifter as u16);
                }
                d.shifter <<= 1;
                d.rddiv >>= 1;
            }
            Math::Divide => {
                d.shifter >>= 1;
                let fits = d.rdmpy as u32 >= d.shifter;
                if fits {
                    d.rdmpy = (d.rdmpy as u32 - d.shifter) as u16;
                }
                d.rddiv = (d.rddiv << 1) | fits as u16;
            }
        }
        d.math_step += 1;
        if d.math_step == if d.math == Math::Multiply { 8 } else { 16 } {
            d.math = Math::Idle;
        }
    }

    /// The sound CPU's ports as its boot ROM presents them (fullsnes, "Uploader"): $BBAA until the CPU's $CC kick on
    /// port 0, then each port as last written, which is the boot ROM's acknowledge on port 0. Stage 4 replaces this.
    pub(crate) fn read_apu_stub(&self, port: usize) -> u8 {
        if self.dev.apu_written { self.dev.apu_stub[port] } else { [0xAA, 0xBB, 0, 0][port] }
    }

    /// $4016-$4017 and $4200-$421F as the CPU reads them, or None for open bus.
    pub(crate) fn read_scpu(&mut self, offset: u16, side_effects: bool) -> Option<u8> {
        Some(match offset {
            0x4016 | 0x4017 => {
                let p = (offset & 1) as usize;
                let bit = (self.dev.shift[p] >> 15) as u8;
                if side_effects && !self.dev.strobe {
                    self.dev.shift[p] = (self.dev.shift[p] << 1) | 1;
                }
                let fixed = if p == 1 { 0x1C } else { 0 };
                (self.mdr & 0xE0) | fixed | bit
            }
            0x4211 => {
                let v = (if self.timing.irq_flag { 0x80 } else { 0 }) | (self.mdr & 0x7F);
                if side_effects {
                    self.timing.irq_flag = false;
                }
                v
            }
            0x4212 => {
                let busy = self.timing.clock < self.dev.joy_busy_until;
                (if self.timing.vblank { 0x80 } else { 0 }) | (if self.timing.hblank() { 0x40 } else { 0 }) | (self.mdr & 0x3E) | busy as u8
            }
            0x4214 => self.dev.rddiv as u8,
            0x4215 => (self.dev.rddiv >> 8) as u8,
            0x4216 => self.dev.rdmpy as u8,
            0x4217 => (self.dev.rdmpy >> 8) as u8,
            0x4218..=0x421F => {
                let w = self.dev.joy[((offset - 0x4218) / 2) as usize];
                if offset & 1 == 0 { w as u8 } else { (w >> 8) as u8 }
            }
            _ => return None,
        })
    }

    pub(crate) fn write_scpu(&mut self, offset: u16, value: u8) {
        match offset {
            0x4016 => {
                let strobe = value & 1 != 0;
                if strobe || self.dev.strobe {
                    self.dev.shift = [serial(self.dev.pads[0]), serial(self.dev.pads[1])];
                }
                self.dev.strobe = strobe;
            }
            0x4200 => {
                self.dev.nmitimen = value;
                self.timing.irq_mode = (value >> 4) & 3;
                if self.timing.irq_mode == 0 {
                    self.timing.irq_flag = false;
                }
                self.after_clock();
            }
            0x4202 => self.dev.wrmpya = value,
            0x4203 => {
                self.dev.wrmpyb = value;
                self.dev.rdmpy = 0;
                if self.dev.math != Math::Multiply {
                    self.dev.rddiv = (value as u16) << 8 | self.dev.wrmpya as u16;
                    self.dev.shifter = value as u32;
                    self.dev.math = Math::Multiply;
                    self.dev.math_step = 0;
                }
            }
            0x4204 => self.dev.wrdiv = (self.dev.wrdiv & 0xFF00) | value as u16,
            0x4205 => self.dev.wrdiv = (self.dev.wrdiv & 0x00FF) | (value as u16) << 8,
            0x4206 => {
                self.dev.wrdivb = value;
                self.dev.rdmpy = self.dev.wrdiv;
                if self.dev.math != Math::Divide {
                    self.dev.shifter = (value as u32) << 16;
                    self.dev.math = Math::Divide;
                    self.dev.math_step = 0;
                }
            }
            0x4207 => self.timing.htime = (self.timing.htime & 0x100) | value as u16,
            0x4208 => self.timing.htime = (self.timing.htime & 0xFF) | ((value as u16 & 1) << 8),
            0x4209 => self.timing.vtime = (self.timing.vtime & 0x100) | value as u16,
            0x420A => self.timing.vtime = (self.timing.vtime & 0xFF) | ((value as u16 & 1) << 8),
            0x420B => {
                if value != 0 {
                    self.dev.dma_pending = value;
                    self.dev.dma_wait = 2;
                }
            }
            _ => {}
        }
        if (0x4200..=0x421F).contains(&offset) {
            self.io[(offset - 0x4000) as usize] = value;
        }
        self.timing.schedule();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cart::Cartridge;
    use crate::cpu::{Bus, pin};

    fn system() -> System {
        let mut rom = vec![0u8; 0x8000];
        rom[0x7FD5] = 0x20;
        System::new(Cartridge::new(&rom).unwrap())
    }

    // fullsnes: eight cycles a product, sixteen a quotient, division by zero giving $FFFF and the dividend.
    #[test]
    fn the_unit_finishes_in_its_documented_cycles() {
        let mut s = system();
        for (r, v) in [(0x4202, 200u8), (0x4203, 201)] {
            s.write(r, v, pin::VDA);
        }
        for _ in 0..7 {
            s.idle(0, 0);
        }
        assert_ne!(s.read_scpu(0x4216, false).unwrap() as u16 | (s.read_scpu(0x4217, false).unwrap() as u16) << 8, 40200);
        s.idle(0, 0);
        assert_eq!(s.read_scpu(0x4216, false).unwrap() as u16 | (s.read_scpu(0x4217, false).unwrap() as u16) << 8, 40200);
        for (r, v) in [(0x4204, 0x39u8), (0x4205, 0x30), (0x4206, 0)] {
            s.write(r, v, pin::VDA);
        }
        for _ in 0..16 {
            s.idle(0, 0);
        }
        assert_eq!((s.dev.rddiv, s.dev.rdmpy), (0xFFFF, 0x3039));
        for (r, v) in [(0x4204, 0x39u8), (0x4205, 0x30), (0x4206, 7)] {
            s.write(r, v, pin::VDA);
        }
        for _ in 0..16 {
            s.idle(0, 0);
        }
        assert_eq!((s.dev.rddiv, s.dev.rdmpy), (12345 / 7, 12345 % 7));
    }

    #[test]
    fn a_dma_moves_its_bytes_one_cycle_after_the_write_and_costs_eight_a_byte() {
        let mut s = system();
        for i in 0..4 {
            s.wram[0x100 + i] = 0x10 + i as u8;
        }
        for (r, v) in [(0x4300u32, 0x00u8), (0x4301, 0x80), (0x4302, 0x00), (0x4303, 0x01), (0x4304, 0x7E), (0x4305, 4), (0x4306, 0), (0x2181, 0x00), (0x2182, 0x02), (0x2183, 0)] {
            s.write(r, v, pin::VDA);
        }
        s.write(0x420B, 1, pin::VDA);
        let before = s.timing.clock;
        s.idle(0, 0);
        assert_eq!(s.wram[0x200], 0);
        s.idle(0, 0);
        assert_eq!(&s.wram[0x200..0x204], &[0x10, 0x11, 0x12, 0x13]);
        let paused = s.timing.clock - before - 12;
        assert!((48..=48 + 14).contains(&paused), "{paused}");
        assert_eq!((s.io[0x305], s.io[0x306], s.io[0x302], s.io[0x303]), (0, 0, 0x04, 0x01));
    }

    #[test]
    fn the_comparator_raises_the_irq_flag_at_htime_and_vtime() {
        let mut s = system();
        for (r, v) in [(0x4207u32, 100u8), (0x4208, 0), (0x4209, 3), (0x420A, 0), (0x4200, 0x30)] {
            s.write(r, v, pin::VDA);
        }
        while !s.timing.irq_flag {
            s.idle(0, 0);
        }
        assert_eq!(s.timing.line, 3);
        assert!((14 + 400..14 + 400 + 6).contains(&s.timing.line_clock), "{}", s.timing.line_clock);
        assert_eq!(s.read_scpu(0x4211, true).unwrap() & 0x80, 0x80);
        assert!(!s.timing.irq_flag);
    }

    #[test]
    fn the_joypad_is_read_at_vblank_and_shifted_by_hand() {
        let mut s = system();
        s.dev.pads[0] = 1 | 1 << 8;
        s.write(0x4200, 1, pin::VDA);
        while s.timing.line != 226 {
            s.idle(0, 0);
        }
        assert_eq!(s.dev.joy[0], 0x8080);
        s.write(0x4016, 1, pin::VDA);
        s.write(0x4016, 0, pin::VDA);
        let bits: Vec<u8> = (0..17).map(|_| s.read(0x4016, pin::VDA) & 1).collect();
        assert_eq!(&bits[..9], &[1, 0, 0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(bits[16], 1);
    }
}
