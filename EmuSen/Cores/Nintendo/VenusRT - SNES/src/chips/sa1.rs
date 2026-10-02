//! The SA-1, from fullsnes ("SNES Cart SA-1" and its chapters): a second 65816 on the cartridge's own bus, with the
//! 2 KiB I-RAM, BW-RAM (banks 40-4F for the S-CPU, 40-5F for the SA-1, absindx's tests 155-162) and its bitmap view, the MMC's ROM banks, the two sides' interrupts and vectors, the H/V timer,
//! normal and character-conversion DMA, the arithmetic unit and the variable-length bit reader. It runs behind the
//! S-CPU and is caught up after each S-CPU instruction and before each S-CPU access to it. See VenusRT_Native.md §25.

use crate::cart::mirror;
use crate::cpu::{Bus, Cpu, Interrupt, flag};

/// The registers' writers, by fullsnes's "Side" column.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Snes,
    Sa1,
}

/// SA-1 bus cycles in master clocks: 10.74 MHz, BW-RAM at half that (fullsnes's DMA rates).
const FAST: u64 = 2;
const BWRAM: u64 = 4;

#[derive(Clone, Debug)]
pub struct Sa1 {
    pub cpu: Cpu,
    pub iram: Box<[u8]>,
    /// Master clocks the SA-1 has run, against the S-CPU's.
    pub clock: u64,
    /// The SA-1's own data bus, which unmapped reads return (absindx's RAM-protection notes).
    pub mdr: u8,
    pub ccnt: u8,
    pub sie: u8,
    /// SFR: the message, the vector selects and the two interrupts to the S-CPU.
    pub sfr: u8,
    pub scnt: u8,
    /// CFR: the message from the S-CPU and the four interrupts to the SA-1.
    pub cfr: u8,
    pub cie: u8,
    pub nmi_pending: bool,
    pub reset_pending: bool,
    pub crv: u16,
    pub cnv: u16,
    pub civ: u16,
    pub snv: u16,
    pub siv: u16,
    pub tmc: u8,
    pub hcmp: u16,
    pub vcmp: u16,
    /// The timer's zero, in master clocks, and the dots counted to the last check.
    pub timer_zero: u64,
    pub timer_seen: u64,
    pub hlatch: u16,
    pub vlatch: u16,
    pub pal: bool,
    pub mmc: [u8; 4],
    pub bmaps: u8,
    pub bmap: u8,
    pub sbwe: u8,
    pub cbwe: u8,
    pub bwpa: u8,
    pub siwp: u8,
    pub ciwp: u8,
    pub bbf: u8,
    pub dcnt: u8,
    pub cdma: u8,
    pub sda: u32,
    pub dda: u32,
    pub dtc: u16,
    pub brf: [u8; 16],
    /// Character conversion 2's next row, and conversion 1's next byte of the stream the S-CPU reads.
    pub row: u8,
    pub char1: Option<u32>,
    pub mcnt: u8,
    pub ma: u16,
    pub mb: u16,
    pub mr: u64,
    pub overflow: bool,
    pub vbd: u8,
    pub vda: u32,
    pub vbit: u32,
}

impl Sa1 {
    pub fn new(pal: bool) -> Sa1 {
        Sa1 {
            cpu: Cpu::default(),
            iram: vec![0; 0x800].into(),
            clock: 0,
            mdr: 0,
            ccnt: 0x20,
            sie: 0,
            sfr: 0,
            scnt: 0,
            cfr: 0,
            cie: 0,
            nmi_pending: false,
            reset_pending: false,
            crv: 0,
            cnv: 0,
            civ: 0,
            snv: 0,
            siv: 0,
            tmc: 0,
            hcmp: 0,
            vcmp: 0,
            timer_zero: 0,
            timer_seen: 0,
            hlatch: 0,
            vlatch: 0,
            pal,
            mmc: [0, 1, 2, 3],
            bmaps: 0,
            bmap: 0,
            sbwe: 0,
            cbwe: 0,
            bwpa: 0xFF,
            siwp: 0,
            ciwp: 0,
            bbf: 0,
            dcnt: 0,
            cdma: 0,
            sda: 0,
            dda: 0,
            dtc: 0,
            brf: [0; 16],
            row: 0,
            char1: None,
            mcnt: 0,
            ma: 0,
            mb: 0,
            mr: 0,
            overflow: false,
            vbd: 0,
            vda: 0,
            vbit: 0,
        }
    }

    /// The S-CPU's IRQ line from the cartridge: the SA-1's and the character-conversion interrupt, where enabled.
    pub fn snes_irq(&self) -> bool {
        self.sfr & self.sie & 0xA0 != 0
    }

    fn sa1_irq(&self) -> bool {
        self.cfr & self.cie & 0xE0 != 0
    }

    /// A ROM offset for a CPU address, by the MMC's banks (fullsnes, "Memory Control"): LoROM 00-3F/80-BF and HiROM
    /// C0-FF in four 1 MiB regions.
    pub fn rom_offset(&self, address: u32, size: usize) -> Option<usize> {
        let bank = (address >> 16) as usize;
        let offset = address as usize & 0xFFFF;
        let at = if bank & 0x40 == 0 && offset >= 0x8000 {
            let region = ((bank >> 5) & 1) | ((bank >> 6) & 2);
            let reg = self.mmc[region];
            let block = if reg & 0x80 != 0 { (reg & 7) as usize } else { region };
            block << 20 | (bank & 0x1F) << 15 | (offset & 0x7FFF)
        } else if bank >= 0xC0 {
            let block = (self.mmc[(bank >> 4) & 3] & 7) as usize;
            block << 20 | (bank & 0x0F) << 16 | offset
        } else {
            return None;
        };
        Some(mirror(at, size))
    }

    fn bw_size_mask(bw: &[u8]) -> Option<usize> {
        if bw.is_empty() { None } else { Some(bw.len().next_power_of_two() - 1) }
    }

    /// Whether a write to BW-RAM offset `at` is allowed: outside BWPA's protected start it always is; inside, while
    /// either side's write enable is set (absindx's tests 35 and 38), BWPA's size capped at the 256 KiB space.
    fn bw_writable(&self, at: usize) -> bool {
        let size = (256usize << (self.bwpa & 0x0F)).min(0x4_0000);
        at >= size || (self.sbwe | self.cbwe) & 0x80 != 0
    }

    /// The bitmap view of BW-RAM (fullsnes, BBF): 2- or 4-bit pixels, the leftmost in the low bits.
    fn bitmap_read(&self, bw: &[u8], i: usize) -> u8 {
        let Some(mask) = Self::bw_size_mask(bw) else { return self.mdr };
        if self.bbf & 0x80 != 0 {
            (bw[(i >> 2) & mask & (bw.len() - 1)] >> ((i & 3) * 2)) & 3
        } else {
            (bw[(i >> 1) & mask & (bw.len() - 1)] >> ((i & 1) * 4)) & 15
        }
    }

    fn bitmap_write(&mut self, bw: &mut [u8], i: usize, v: u8) {
        let Some(mask) = Self::bw_size_mask(bw) else { return };
        let n = bw.len() - 1;
        if self.bbf & 0x80 != 0 {
            let s = (i & 3) * 2;
            let b = &mut bw[(i >> 2) & mask & n];
            *b = (*b & !(3 << s)) | ((v & 3) << s);
        } else {
            let s = (i & 1) * 4;
            let b = &mut bw[(i >> 1) & mask & n];
            *b = (*b & !(15 << s)) | ((v & 15) << s);
        }
    }

    fn bw_index(bw: &[u8], at: usize) -> Option<usize> {
        Self::bw_size_mask(bw).map(|m| (at & m) % bw.len())
    }

    /// Whether an S-CPU address is the cartridge's under the SA-1: its I/O, I-RAM, BW-RAM and ROM.
    pub fn snes_maps(address: u32) -> bool {
        let bank = (address >> 16) as u8;
        let offset = address as u16;
        if bank & 0x40 == 0 {
            matches!(offset, 0x2200..=0x23FF | 0x3000..=0x37FF | 0x6000..=0xFFFF)
        } else {
            matches!(bank, 0x40..=0x4F) || bank >= 0xC0
        }
    }

    /// The S-CPU's read of a cartridge address, or None where nothing drives the bus.
    pub fn snes_read(&mut self, address: u32, rom: &[u8], bw: &[u8], side_effects: bool) -> Option<u8> {
        let bank = (address >> 16) as u8;
        let offset = address as u16;
        if bank & 0x40 == 0 {
            match offset {
                0x2200..=0x23FF => match offset {
                    0x2300 => Some(self.sfr),
                    0x230E => Some(VERSION),
                    _ => None,
                },
                0x3000..=0x37FF => Some(self.iram[(offset & 0x7FF) as usize]),
                0x6000..=0x7FFF => {
                    let at = ((self.bmaps & 0x1F) as usize) << 13 | (offset as usize - 0x6000);
                    Self::bw_index(bw, at).map(|i| bw[i])
                }
                _ => {
                    let v = match (bank & 0x7F, offset) {
                        (0x00, 0xFFEA) if self.scnt & 0x10 != 0 => self.snv as u8,
                        (0x00, 0xFFEB) if self.scnt & 0x10 != 0 => (self.snv >> 8) as u8,
                        (0x00, 0xFFEE) if self.scnt & 0x40 != 0 => self.siv as u8,
                        (0x00, 0xFFEF) if self.scnt & 0x40 != 0 => (self.siv >> 8) as u8,
                        _ => rom[self.rom_offset(address, rom.len())?],
                    };
                    Some(v)
                }
            }
        } else if bank <= 0x4F {
            if let Some(n) = self.char1 {
                if side_effects {
                    self.char1 = Some(n + 1);
                }
                return Some(self.char1_byte(bw, n));
            }
            Self::bw_index(bw, address as usize & 0x3_FFFF).map(|i| bw[i])
        } else {
            Some(rom[self.rom_offset(address, rom.len())?])
        }
    }

    pub fn snes_write(&mut self, address: u32, value: u8, rom: &[u8], bw: &mut [u8]) {
        let bank = (address >> 16) as u8;
        let offset = address as u16;
        if bank & 0x40 == 0 {
            match offset {
                0x2200..=0x23FF => self.io_write(Side::Snes, offset, value, rom, bw),
                0x3000..=0x37FF => {
                    if self.siwp & (1 << ((offset >> 8) & 7)) != 0 {
                        self.iram[(offset & 0x7FF) as usize] = value;
                    }
                }
                0x6000..=0x7FFF => {
                    let at = ((self.bmaps & 0x1F) as usize) << 13 | (offset as usize - 0x6000);
                    self.bw_store(bw, at, value);
                }
                _ => {}
            }
        } else if bank <= 0x4F {
            self.bw_store(bw, address as usize & 0x3_FFFF, value);
        }
    }

    fn bw_store(&mut self, bw: &mut [u8], at: usize, value: u8) {
        if let Some(i) = Self::bw_index(bw, at) {
            if self.bw_writable(at) {
                bw[i] = value;
            }
        }
    }

    /// One byte of character conversion 1's output: the bitmap at SDA, CDMA's width in characters and depth,
    /// read out as bitplaned tiles in order (fullsnes, "Character Conversion 1").
    fn char1_byte(&self, bw: &[u8], n: u32) -> u8 {
        let bpp = [8u32, 4, 2, 8][(self.cdma & 3) as usize];
        let width = 1u32 << ((self.cdma >> 2) & 7).min(5);
        let per_tile = 8 * bpp;
        let (tile, b) = (n / per_tile, n % per_tile);
        let (pair, y, odd) = (b / 16, (b % 16) / 2, b % 2);
        let plane = pair * 2 + odd;
        let row = (tile / width) * 8 + y;
        let stride = width * 8 * bpp;
        let mut out = 0u8;
        for px in 0..8 {
            let x = (tile % width) * 8 + px;
            let bit = row * stride + x * bpp;
            let at = self.sda as usize + (bit / 8) as usize;
            let byte = Self::bw_index(bw, at).map_or(0, |i| bw[i]);
            let value = (byte >> (bit % 8)) as u32 & ((1 << bpp) - 1).min(0xFF);
            out |= (((value >> plane) & 1) as u8) << (7 - px);
        }
        out
    }

    /// The S-CPU's or the SA-1's write to $2200-$23FF, each register taking only its side's writes.
    fn io_write(&mut self, side: Side, reg: u16, v: u8, rom: &[u8], bw: &mut [u8]) {
        let snes = matches!(reg, 0x2200..=0x2208 | 0x2220..=0x2224 | 0x2226 | 0x2228 | 0x2229);
        let both = matches!(reg, 0x2231..=0x2237);
        if !both && (snes != (side == Side::Snes)) {
            return;
        }
        match reg {
            0x2200 => {
                self.cfr = (self.cfr & 0xF0) | (v & 0x0F);
                if v & 0x80 != 0 {
                    self.cfr |= 0x80;
                }
                if v & 0x10 != 0 {
                    self.cfr |= 0x10;
                    if self.cie & 0x10 != 0 {
                        self.nmi_pending = true;
                    }
                }
                if self.ccnt & 0x20 != 0 && v & 0x20 == 0 {
                    self.reset_pending = true;
                }
                if v & 0x20 != 0 {
                    // D-34: holding the SA-1 in reset clears its I-RAM write protection (absindx's test 221).
                    self.ciwp = 0;
                }
                self.ccnt = v;
            }
            0x2201 => self.sie = v,
            0x2202 => self.sfr &= !(v & 0xA0),
            0x2203 => self.crv = (self.crv & 0xFF00) | v as u16,
            0x2204 => self.crv = (self.crv & 0x00FF) | (v as u16) << 8,
            0x2205 => self.cnv = (self.cnv & 0xFF00) | v as u16,
            0x2206 => self.cnv = (self.cnv & 0x00FF) | (v as u16) << 8,
            0x2207 => self.civ = (self.civ & 0xFF00) | v as u16,
            0x2208 => self.civ = (self.civ & 0x00FF) | (v as u16) << 8,
            0x2209 => {
                self.scnt = v;
                self.sfr = (self.sfr & 0xA0) | (v & 0x5F);
                if v & 0x80 != 0 {
                    self.sfr |= 0x80;
                }
            }
            0x220A => self.cie = v,
            0x220B => {
                self.cfr &= !(v & 0xF0);
                if v & 0x10 != 0 {
                    self.nmi_pending = false;
                }
            }
            0x220C => self.snv = (self.snv & 0xFF00) | v as u16,
            0x220D => self.snv = (self.snv & 0x00FF) | (v as u16) << 8,
            0x220E => self.siv = (self.siv & 0xFF00) | v as u16,
            0x220F => self.siv = (self.siv & 0x00FF) | (v as u16) << 8,
            0x2210 => self.tmc = v,
            0x2211 => {
                self.timer_zero = self.clock;
                self.timer_seen = 0;
            }
            0x2212 => self.hcmp = (self.hcmp & 0x100) | v as u16,
            0x2213 => self.hcmp = (self.hcmp & 0xFF) | ((v as u16 & 1) << 8),
            0x2214 => self.vcmp = (self.vcmp & 0x100) | v as u16,
            0x2215 => self.vcmp = (self.vcmp & 0xFF) | ((v as u16 & 1) << 8),
            0x2220..=0x2223 => self.mmc[(reg - 0x2220) as usize] = v,
            0x2224 => self.bmaps = v,
            0x2225 => self.bmap = v,
            0x2226 => self.sbwe = v,
            0x2227 => self.cbwe = v,
            0x2228 => self.bwpa = v,
            0x2229 => self.siwp = v,
            0x222A => self.ciwp = v,
            0x2230 => {
                self.dcnt = v;
                if v & 0x20 == 0 || v & 0x80 == 0 {
                    self.row = 0;
                }
            }
            0x2231 => {
                self.cdma = v & 0x1F;
                if v & 0x80 != 0 {
                    self.char1 = None;
                }
            }
            0x2232 => self.sda = (self.sda & 0xFFFF00) | v as u32,
            0x2233 => self.sda = (self.sda & 0xFF00FF) | (v as u32) << 8,
            0x2234 => self.sda = (self.sda & 0x00FFFF) | (v as u32) << 16,
            0x2235 => self.dda = (self.dda & 0xFFFF00) | v as u32,
            0x2236 => {
                self.dda = (self.dda & 0xFF00FF) | (v as u32) << 8;
                if self.dcnt & 0x20 != 0 && self.dcnt & 0x10 != 0 {
                    // Character conversion 1 starts on DDA; its first character is ready for the S-CPU at once.
                    self.char1 = Some(0);
                    self.sfr |= 0x20;
                } else if self.dcnt & 0xA4 == 0x80 {
                    self.dma(rom, bw);
                }
            }
            0x2237 => {
                self.dda = (self.dda & 0x00FFFF) | (v as u32) << 16;
                if self.dcnt & 0xA4 == 0x84 {
                    self.dma(rom, bw);
                }
            }
            0x2238 => self.dtc = (self.dtc & 0xFF00) | v as u16,
            0x2239 => self.dtc = (self.dtc & 0x00FF) | (v as u16) << 8,
            0x223F => self.bbf = v,
            0x2240..=0x224F => {
                self.brf[(reg - 0x2240) as usize] = v;
                if self.dcnt & 0xB0 == 0xA0 && (reg == 0x2247 || reg == 0x224F) {
                    self.char2_row(if reg == 0x2247 { 0 } else { 8 });
                }
            }
            0x2250 => {
                self.mcnt = v;
                if v & 2 != 0 {
                    self.mr = 0;
                    self.overflow = false;
                }
            }
            0x2251 => self.ma = (self.ma & 0xFF00) | v as u16,
            0x2252 => self.ma = (self.ma & 0x00FF) | (v as u16) << 8,
            0x2253 => self.mb = (self.mb & 0xFF00) | v as u16,
            0x2254 => {
                self.mb = (self.mb & 0x00FF) | (v as u16) << 8;
                self.arithmetic();
            }
            0x2258 => {
                self.vbd = v;
                if v & 0x80 == 0 {
                    self.vbit += self.vlen();
                }
            }
            0x2259 => self.vda = (self.vda & 0xFFFF00) | v as u32,
            0x225A => self.vda = (self.vda & 0xFF00FF) | (v as u32) << 8,
            0x225B => {
                self.vda = (self.vda & 0x00FFFF) | (v as u32) << 16;
                self.vbit = 0;
            }
            _ => {}
        }
    }

    fn vlen(&self) -> u32 {
        match self.vbd & 0x0F {
            0 => 16,
            n => n as u32,
        }
    }

    /// The 16 bits at the variable-length reader's position, ROM read through the SA-1's map.
    fn vdp(&self, rom: &[u8]) -> u16 {
        let byte = |k: u32| self.rom_offset(self.vda.wrapping_add(k) & 0xFF_FFFF, rom.len()).map_or(0, |o| rom[o]) as u32;
        let at = self.vbit / 8;
        let word = byte(at) | byte(at + 1) << 8 | byte(at + 2) << 16;
        (word >> (self.vbit % 8)) as u16
    }

    /// fullsnes, "Arithmetic Maths": a signed product, a signed quotient with an unsigned remainder, or a 40-bit
    /// running sum of products; division by zero gives zeros.
    fn arithmetic(&mut self) {
        let a = self.ma as i16 as i64;
        match self.mcnt & 3 {
            0 => self.mr = (a * self.mb as i16 as i64) as u64 & 0xFF_FFFF_FFFF,
            1 => {
                let b = self.mb as i64;
                if b == 0 {
                    self.mr = 0;
                } else {
                    let (q, r) = (a.div_euclid(b), a.rem_euclid(b));
                    self.mr = ((r as u64 & 0xFFFF) << 16) | (q as u64 & 0xFFFF);
                }
            }
            _ => {
                let sum = ((self.mr << 24) as i64 >> 24) + a * self.mb as i16 as i64;
                self.overflow |= !(-(1i64 << 39)..(1i64 << 39)).contains(&sum);
                self.mr = sum as u64 & 0xFF_FFFF_FFFF;
            }
        }
    }

    /// Character conversion 2: the eight pixels just written, bitplaned into row `row` of the I-RAM buffer's tile.
    fn char2_row(&mut self, half: usize) {
        let bpp = [8usize, 4, 2, 8][(self.cdma & 3) as usize];
        let per_tile = 8 * bpp;
        let tile = (self.row as usize / 8) & 1;
        let y = self.row as usize % 8;
        let base = (self.dda as usize & 0x7FF) + tile * per_tile;
        for plane in 0..bpp {
            let mut byte = 0u8;
            for x in 0..8 {
                byte |= ((self.brf[half + x] >> plane) & 1) << (7 - x);
            }
            self.iram[(base + (plane / 2) * 16 + y * 2 + (plane % 2)) & 0x7FF] = byte;
        }
        self.row = (self.row + 1) % 16;
    }

    /// Normal DMA (fullsnes, "DMA Transfers"): DTC bytes from ROM, BW-RAM or I-RAM to I-RAM or BW-RAM, the SA-1's
    /// clock charged at the transfer's rate, and the DMA interrupt raised at its end.
    fn dma(&mut self, rom: &[u8], bw: &mut [u8]) {
        let source = self.dcnt & 3;
        let to_bw = self.dcnt & 4 != 0;
        for k in 0..self.dtc as u32 {
            let v = match source {
                0 => self.rom_offset((self.sda + k) & 0xFF_FFFF, rom.len()).map_or(0, |o| rom[o]),
                1 => Self::bw_index(bw, ((self.sda + k) & 0x3_FFFF) as usize).map_or(0, |i| bw[i]),
                _ => self.iram[((self.sda + k) & 0x7FF) as usize],
            };
            if to_bw {
                if let Some(i) = Self::bw_index(bw, ((self.dda + k) & 0x3_FFFF) as usize) {
                    bw[i] = v;
                }
            } else {
                self.iram[((self.dda + k) & 0x7FF) as usize] = v;
            }
        }
        self.clock += self.dtc as u64 * if source == 0 && !to_bw { FAST } else { BWRAM };
        self.cfr |= 0x20;
    }

    /// The H/V timer's interrupt: whether a match was passed since the last check (fullsnes, "Timer"; HCNT and VCNT
    /// read as compare values, as fullsnes argues they must be).
    fn timer_check(&mut self) {
        if self.tmc & 3 == 0 {
            return;
        }
        let dots = (self.clock - self.timer_zero) / 4;
        let seen = self.timer_seen;
        self.timer_seen = dots;
        let (width, lines) = if self.tmc & 0x80 != 0 { (512u64, 512u64) } else { (341, if self.pal { 312 } else { 262 }) };
        let (period, phase) = match self.tmc & 3 {
            1 => (width, self.hcmp as u64),
            2 => (width * lines, self.vcmp as u64 * width),
            _ => (width * lines, self.vcmp as u64 * width + self.hcmp as u64),
        };
        let passed = |d: u64| (d + period - phase % period) / period;
        if dots > seen && passed(dots) > passed(seen) {
            self.cfr |= 0x40;
        }
    }

    fn counters(&self) -> (u16, u16) {
        let dots = (self.clock - self.timer_zero) / 4;
        let (width, lines) = if self.tmc & 0x80 != 0 { (512u64, 512u64) } else { (341, if self.pal { 312 } else { 262 }) };
        ((dots % width) as u16, ((dots / width) % lines) as u16)
    }

    /// Runs the SA-1 until its clock reaches `target`; held in reset or by CCNT's wait it only lets time pass.
    pub fn run_to(&mut self, target: u64, rom: &[u8], bw: &mut [u8]) {
        while self.clock < target {
            if self.ccnt & 0x60 != 0 {
                self.clock = target;
                self.timer_check();
                return;
            }
            let mut cpu = self.cpu;
            {
                let mut bus = Sa1Bus { s: self, rom, bw };
                if bus.s.reset_pending {
                    bus.s.reset_pending = false;
                    cpu.interrupt(&mut bus, Interrupt::Reset);
                } else if bus.s.nmi_pending {
                    bus.s.nmi_pending = false;
                    cpu.interrupt(&mut bus, Interrupt::Nmi);
                } else if bus.s.sa1_irq() && (cpu.p & flag::I == 0 || cpu.waiting) {
                    if cpu.p & flag::I == 0 {
                        cpu.interrupt(&mut bus, Interrupt::Irq);
                    } else {
                        cpu.waiting = false;
                    }
                } else {
                    cpu.step(&mut bus);
                }
            }
            self.cpu = cpu;
            self.timer_check();
        }
    }

    /// The state beyond the I-RAM, which travels beside it.
    pub fn pack(&self) -> Vec<u8> {
        let c = &self.cpu;
        let mut o = Vec::new();
        for v in [c.a, c.x, c.y, c.s, c.d, c.pc, self.crv, self.cnv, self.civ, self.snv, self.siv, self.hcmp, self.vcmp, self.hlatch, self.vlatch, self.dtc, self.ma, self.mb] {
            o.extend(v.to_le_bytes());
        }
        o.extend([c.dbr, c.pbr, c.p, c.e as u8, c.waiting as u8, c.stopped as u8, self.mdr, self.ccnt, self.sie, self.sfr, self.scnt, self.cfr, self.cie]);
        o.extend([self.nmi_pending as u8, self.reset_pending as u8, self.tmc, self.bmaps, self.bmap, self.sbwe, self.cbwe, self.bwpa, self.siwp, self.ciwp, self.bbf]);
        o.extend([self.dcnt, self.cdma, self.row, self.mcnt, self.overflow as u8, self.vbd, self.char1.is_some() as u8]);
        o.extend(self.mmc);
        o.extend(self.brf);
        for v in [self.clock, self.timer_zero, self.timer_seen, self.mr] {
            o.extend(v.to_le_bytes());
        }
        for v in [self.sda, self.dda, self.vda, self.vbit, self.char1.unwrap_or(0)] {
            o.extend(v.to_le_bytes());
        }
        o.extend_from_slice(&self.iram);
        o
    }

    pub fn unpack(&mut self, d: &[u8]) {
        let w = |i: usize| u16::from_le_bytes([d[i * 2], d[i * 2 + 1]]);
        let c = &mut self.cpu;
        (c.a, c.x, c.y, c.s, c.d, c.pc) = (w(0), w(1), w(2), w(3), w(4), w(5));
        (self.crv, self.cnv, self.civ, self.snv, self.siv, self.hcmp, self.vcmp) = (w(6), w(7), w(8), w(9), w(10), w(11), w(12));
        (self.hlatch, self.vlatch, self.dtc, self.ma, self.mb) = (w(13), w(14), w(15), w(16), w(17));
        let b = &d[36..];
        (c.dbr, c.pbr, c.p, c.e, c.waiting, c.stopped) = (b[0], b[1], b[2], b[3] != 0, b[4] != 0, b[5] != 0);
        (self.mdr, self.ccnt, self.sie, self.sfr, self.scnt, self.cfr, self.cie) = (b[6], b[7], b[8], b[9], b[10], b[11], b[12]);
        (self.nmi_pending, self.reset_pending, self.tmc, self.bmaps, self.bmap, self.sbwe) = (b[13] != 0, b[14] != 0, b[15], b[16], b[17], b[18]);
        (self.cbwe, self.bwpa, self.siwp, self.ciwp, self.bbf) = (b[19], b[20], b[21], b[22], b[23]);
        (self.dcnt, self.cdma, self.row, self.mcnt, self.overflow, self.vbd) = (b[24], b[25], b[26] % 16, b[27], b[28] != 0, b[29]);
        let char1 = b[30] != 0;
        self.mmc.copy_from_slice(&b[31..35]);
        self.brf.copy_from_slice(&b[35..51]);
        let q = |i: usize| u64::from_le_bytes(b[51 + i * 8..59 + i * 8].try_into().expect("eight bytes"));
        (self.clock, self.timer_zero, self.timer_seen, self.mr) = (q(0), q(1), q(2), q(3));
        let l = |i: usize| u32::from_le_bytes(b[83 + i * 4..87 + i * 4].try_into().expect("four bytes"));
        (self.sda, self.dda, self.vda, self.vbit) = (l(0), l(1), l(2), l(3));
        self.char1 = char1.then_some(l(4));
        self.iram.copy_from_slice(&b[103..103 + 0x800]);
    }
}

/// $230E's version code. fullsnes knows none; the chip is the RF5A123, and the value is VenusRT's choice until a
/// test ROM says otherwise (D-33).
const VERSION: u8 = 0x23;

/// The SA-1's view of the cartridge (fullsnes, "Memory Map (SA-1 Side)").
struct Sa1Bus<'a> {
    s: &'a mut Sa1,
    rom: &'a [u8],
    bw: &'a mut [u8],
}

impl Sa1Bus<'_> {
    fn load(&mut self, address: u32) -> (u8, u64) {
        let s = &mut *self.s;
        let bank = (address >> 16) as u8;
        let offset = address as u16;
        let open = s.mdr;
        let (v, cost) = if bank & 0x40 == 0 {
            match offset {
                0x0000..=0x07FF => (s.iram[offset as usize], FAST),
                0x3000..=0x37FF => (s.iram[(offset & 0x7FF) as usize], FAST),
                0x2200..=0x23FF => {
                    let v = match offset {
                        0x2301 => s.cfr,
                        0x2302 => {
                            let (h, v) = s.counters();
                            (s.hlatch, s.vlatch) = (h, v);
                            h as u8
                        }
                        0x2303 => (s.hlatch >> 8) as u8,
                        0x2304 => s.vlatch as u8,
                        0x2305 => (s.vlatch >> 8) as u8,
                        0x2306..=0x230A => (s.mr >> ((offset - 0x2306) * 8)) as u8,
                        0x230B => (s.overflow as u8) << 7 | (open & 0x7F),
                        0x230C => s.vdp(self.rom) as u8,
                        0x230D => {
                            let v = (s.vdp(self.rom) >> 8) as u8;
                            if s.vbd & 0x80 != 0 {
                                s.vbit += s.vlen();
                            }
                            v
                        }
                        _ => open,
                    };
                    (v, FAST)
                }
                0x6000..=0x7FFF => {
                    if s.bmap & 0x80 != 0 {
                        let i = ((s.bmap & 0x7F) as usize) << 13 | (offset as usize - 0x6000);
                        (s.bitmap_read(self.bw, i), BWRAM)
                    } else {
                        let at = ((s.bmap & 0x1F) as usize) << 13 | (offset as usize - 0x6000);
                        (Sa1::bw_index(self.bw, at).map_or(open, |i| self.bw[i]), BWRAM)
                    }
                }
                0x8000..=0xFFFF => {
                    let v = match (bank & 0x7F, offset) {
                        (0x00, 0xFFEA) => s.cnv as u8,
                        (0x00, 0xFFEB) => (s.cnv >> 8) as u8,
                        (0x00, 0xFFEE) => s.civ as u8,
                        (0x00, 0xFFEF) => (s.civ >> 8) as u8,
                        (0x00, 0xFFFC) => s.crv as u8,
                        (0x00, 0xFFFD) => (s.crv >> 8) as u8,
                        _ => s.rom_offset(address, self.rom.len()).map_or(open, |o| self.rom[o]),
                    };
                    (v, FAST)
                }
                _ => (open, FAST),
            }
        } else if bank <= 0x5F {
            (Sa1::bw_index(self.bw, address as usize & 0x3_FFFF).map_or(open, |i| self.bw[i]), BWRAM)
        } else if (0x60..=0x6F).contains(&bank) {
            (s.bitmap_read(self.bw, address as usize & 0xF_FFFF), BWRAM)
        } else if bank >= 0xC0 {
            (s.rom_offset(address, self.rom.len()).map_or(open, |o| self.rom[o]), FAST)
        } else {
            (open, FAST)
        };
        (v, cost)
    }

    fn store(&mut self, address: u32, value: u8) -> u64 {
        let s = &mut *self.s;
        let bank = (address >> 16) as u8;
        let offset = address as u16;
        if bank & 0x40 == 0 {
            match offset {
                0x0000..=0x07FF | 0x3000..=0x37FF => {
                    if s.ciwp & (1 << ((offset >> 8) & 7)) != 0 {
                        s.iram[(offset & 0x7FF) as usize] = value;
                    }
                    FAST
                }
                0x2200..=0x23FF => {
                    s.io_write(Side::Sa1, offset, value, self.rom, self.bw);
                    FAST
                }
                0x6000..=0x7FFF => {
                    if s.bmap & 0x80 != 0 {
                        let i = ((s.bmap & 0x7F) as usize) << 13 | (offset as usize - 0x6000);
                        s.bitmap_write(self.bw, i, value);
                    } else {
                        let at = ((s.bmap & 0x1F) as usize) << 13 | (offset as usize - 0x6000);
                        s.bw_store(self.bw, at, value);
                    }
                    BWRAM
                }
                _ => FAST,
            }
        } else if bank <= 0x5F {
            s.bw_store(self.bw, address as usize & 0x3_FFFF, value);
            BWRAM
        } else if (0x60..=0x6F).contains(&bank) {
            s.bitmap_write(self.bw, address as usize & 0xF_FFFF, value);
            BWRAM
        } else {
            FAST
        }
    }
}

impl Bus for Sa1Bus<'_> {
    fn read(&mut self, address: u32, _pins: u8) -> u8 {
        let (v, cost) = self.load(address);
        self.s.mdr = v;
        self.s.clock += cost;
        v
    }

    fn write(&mut self, address: u32, value: u8, pins: u8) {
        if pins & (crate::cpu::pin::VDA | crate::cpu::pin::VPA) == 0 {
            self.s.clock += FAST;
            return;
        }
        self.s.mdr = value;
        self.s.clock += self.store(address, value);
    }

    fn idle(&mut self, _address: u32, _pins: u8) {
        self.s.clock += FAST;
    }

    fn halted(&mut self) {
        self.s.clock += FAST;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // fullsnes, "Memory Control": LoROM banks use their region's 1 MiB block until bit 7 maps another; HiROM always
    // follows the register.
    #[test]
    fn the_mmc_maps_rom_by_region() {
        let mut s = Sa1::new(false);
        let size = 0x40_0000;
        assert_eq!(s.rom_offset(0x00_8000, size), Some(0));
        assert_eq!(s.rom_offset(0x20_8000, size), Some(0x10_0000));
        assert_eq!(s.rom_offset(0xC1_0000, size), Some(0x1_0000));
        s.mmc[0] = 0x83;
        assert_eq!(s.rom_offset(0x01_8000, size), Some(0x30_8000));
        s.mmc[2] = 0x01;
        assert_eq!(s.rom_offset(0xE0_1234, size), Some(0x10_1234));
    }

    // fullsnes, "Arithmetic Maths": a signed product, a signed quotient and an unsigned remainder, and the running sum.
    #[test]
    fn the_arithmetic_unit_multiplies_divides_and_sums() {
        let mut s = Sa1::new(false);
        let (rom, mut bw) = (vec![0u8; 0x8000], vec![0u8; 0x2000]);
        for (r, v) in [(0x2250, 0), (0x2251, 0xFE), (0x2252, 0xFF), (0x2253, 3), (0x2254, 0)] {
            s.io_write(Side::Sa1, r, v, &rom, &mut bw);
        }
        assert_eq!(s.mr, 0xFF_FFFF_FFFA);
        for (r, v) in [(0x2250, 1), (0x2251, 0xF9), (0x2252, 0xFF), (0x2253, 2), (0x2254, 0)] {
            s.io_write(Side::Sa1, r, v, &rom, &mut bw);
        }
        assert_eq!(s.mr & 0xFFFF_FFFF, 1 << 16 | 0xFFFC);
        for (r, v) in [(0x2250, 2), (0x2251, 10), (0x2252, 0), (0x2253, 10), (0x2254, 0), (0x2254, 0)] {
            s.io_write(Side::Sa1, r, v, &rom, &mut bw);
        }
        assert_eq!(s.mr, 200);
    }

    // Character conversion 2 at 2 bits: a row of pixels becomes the tile row's two bitplanes in I-RAM.
    #[test]
    fn character_conversion_two_bitplanes_a_row() {
        let mut s = Sa1::new(false);
        let (rom, mut bw) = (vec![0u8; 0x8000], vec![0u8; 0x2000]);
        for (r, v) in [(0x2230, 0xA0), (0x2231, 2), (0x2235, 0x00), (0x2236, 0x01)] {
            s.io_write(Side::Sa1, r, v, &rom, &mut bw);
        }
        for x in 0..8u16 {
            s.io_write(Side::Sa1, 0x2240 + x, [3, 0, 1, 2, 0, 0, 0, 1][x as usize], &rom, &mut bw);
        }
        assert_eq!((s.iram[0x100], s.iram[0x101]), (0b1010_0001, 0b1001_0000));
    }
}
