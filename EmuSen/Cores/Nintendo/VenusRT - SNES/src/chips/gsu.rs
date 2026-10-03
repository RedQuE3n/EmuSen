//! The GSU (Mario Chip, Super FX 1 and 2), from fullsnes ("SNES Cart GSU-n" and its chapters): sixteen registers with
//! R15 as a pipelined program counter, the prefixes, the ALU, multiply, ROM and RAM opcodes, PLOT and RPIX on the
//! three screen heights and OBJ mode, the 512-byte code cache, the one-byte ROM buffer and the SNES-side map with its
//! vector substitution while the GSU owns the ROM. It runs behind the S-CPU, caught up after each S-CPU instruction
//! and before each S-CPU access to the board. See VenusRT_Native.md §26.

use crate::cart::mirror;

mod sfr {
    pub const Z: u16 = 0x0002;
    pub const CY: u16 = 0x0004;
    pub const S: u16 = 0x0008;
    pub const OV: u16 = 0x0010;
    pub const GO: u16 = 0x0020;
    pub const IRQ: u16 = 0x8000;
}

#[derive(Clone, Debug)]
pub struct Gsu {
    pub r: [u16; 16],
    pub sfr: u16,
    pub pbr: u8,
    pub rombr: u8,
    pub rambr: u8,
    pub cbr: u16,
    pub scbr: u8,
    pub scmr: u8,
    pub colr: u8,
    pub por: u8,
    pub bramr: u8,
    pub cfgr: u8,
    pub clsr: u8,
    pub vcr: u8,
    pub cache: Box<[u8]>,
    /// One bit per 16-byte cache line: loaded.
    pub lines: u32,
    /// The prefixes: ALT1 and ALT2, the WITH flag, and the source and destination registers.
    pub alt: u8,
    pub b: bool,
    pub sreg: u8,
    pub dreg: u8,
    /// The SNES's even-byte write, held for the odd byte (fullsnes: "LATCH").
    pub latch: u8,
    /// The opcode byte fetched ahead, and whether R15 was written since (a jump), so the next fetch is not a step on.
    pub pipe: u8,
    pub jumped: bool,
    pub rom_buffer: u8,
    /// The master clock the ROM buffer's load ends at, and whether this opcode wrote R14 (D-35).
    pub rom_ready: u64,
    pub r14_written: bool,
    pub ram_address: u16,
    /// Master clocks the GSU has run.
    pub clock: u64,
}

/// fullsnes, "GSU Interrupt Vectors": what the S-CPU reads from ROM while the GSU owns it, by the address's low bits.
const RUNNING_VECTORS: [u16; 8] = [0x0100, 0x0100, 0x0104, 0x0100, 0x0100, 0x0108, 0x0100, 0x010C];

impl Gsu {
    pub fn new(gsu2: bool) -> Gsu {
        Gsu {
            r: [0; 16],
            sfr: 0,
            pbr: 0,
            rombr: 0,
            rambr: 0,
            cbr: 0,
            scbr: 0,
            scmr: 0,
            colr: 0,
            por: 0,
            bramr: 0,
            cfgr: 0,
            clsr: 0,
            vcr: if gsu2 { 4 } else { 1 },
            cache: vec![0; 512].into(),
            lines: 0,
            alt: 0,
            b: false,
            sreg: 0,
            dreg: 0,
            latch: 0,
            pipe: 0x01,
            jumped: false,
            rom_buffer: 0,
            rom_ready: 0,
            r14_written: false,
            ram_address: 0,
            clock: 0,
        }
    }

    fn running(&self) -> bool {
        self.sfr & sfr::GO != 0
    }

    /// The S-CPU's IRQ line from the board: STOP's interrupt unless CFGR masks it.
    pub fn snes_irq(&self) -> bool {
        self.sfr & sfr::IRQ != 0 && self.cfgr & 0x80 == 0
    }

    /// A ROM offset for a GSU or S-CPU address: LoROM in banks 00-3F, HiROM in 40-5F (fullsnes's GSU2 map).
    fn rom_at(address: u32, size: usize) -> Option<usize> {
        let bank = (address >> 16) as usize & 0x7F;
        let offset = address as usize & 0xFFFF;
        let at = match bank {
            0x00..=0x3F => bank << 15 | (offset & 0x7FFF),
            0x40..=0x5F => (bank & 0x1F) << 16 | offset,
            _ => return None,
        };
        Some(mirror(at, size))
    }

    fn ram_index(ram: &[u8], at: usize) -> Option<usize> {
        if ram.is_empty() { None } else { Some(at % ram.len()) }
    }

    /// Whether an S-CPU address is the board's.
    pub fn snes_maps(address: u32) -> bool {
        let bank = (address >> 16) as u8 & 0x7F;
        let offset = address as u16;
        match bank {
            0x00..=0x3F => matches!(offset, 0x3000..=0x34FF | 0x6000..=0xFFFF),
            0x40..=0x5F | 0x70 | 0x71 => true,
            _ => false,
        }
    }

    pub fn snes_read(&mut self, address: u32, rom: &[u8], ram: &[u8]) -> Option<u8> {
        let bank = (address >> 16) as u8 & 0x7F;
        let offset = address as u16;
        let rom_owned = self.running() && self.scmr & 0x10 != 0;
        let ram_owned = self.running() && self.scmr & 0x08 != 0;
        match bank {
            0x00..=0x3F => match offset {
                0x3000..=0x34FF => self.io_read(offset),
                0x6000..=0x7FFF => {
                    if ram_owned {
                        None
                    } else {
                        Self::ram_index(ram, offset as usize - 0x6000).map(|i| ram[i])
                    }
                }
                _ => self.rom_byte(address, rom, rom_owned),
            },
            0x40..=0x5F => self.rom_byte(address, rom, rom_owned),
            _ => {
                if ram_owned {
                    None
                } else {
                    Self::ram_index(ram, ((bank as usize & 1) << 16) | offset as usize).map(|i| ram[i])
                }
            }
        }
    }

    fn rom_byte(&self, address: u32, rom: &[u8], owned: bool) -> Option<u8> {
        if owned {
            let v = RUNNING_VECTORS[((address >> 1) & 7) as usize];
            return Some(if address & 1 == 0 { v as u8 } else { (v >> 8) as u8 });
        }
        Some(rom[Self::rom_at(address, rom.len())?])
    }

    pub fn snes_write(&mut self, address: u32, value: u8, rom: &[u8], ram: &mut [u8]) {
        let bank = (address >> 16) as u8 & 0x7F;
        let offset = address as u16;
        let ram_owned = self.running() && self.scmr & 0x08 != 0;
        match bank {
            0x00..=0x3F => match offset {
                0x3000..=0x34FF => self.io_write(offset, value, rom, ram),
                0x6000..=0x7FFF if !ram_owned => {
                    if let Some(i) = Self::ram_index(ram, offset as usize - 0x6000) {
                        ram[i] = value;
                    }
                }
                _ => {}
            },
            0x70 | 0x71 if !ram_owned => {
                if let Some(i) = Self::ram_index(ram, ((bank as usize & 1) << 16) | offset as usize) {
                    ram[i] = value;
                }
            }
            _ => {}
        }
    }

    /// The I/O page's register for an address, through the GSU2's mirrors (fullsnes's "Full I/O Map").
    fn io_reg(offset: u16) -> Option<u16> {
        match offset {
            0x3100..=0x32FF => Some(offset),
            0x3000..=0x30FF | 0x3300..=0x34FF => {
                let r = offset & 0x3F;
                Some(0x3000 | if (0x20..0x30).contains(&r) { r + 0x10 } else { r })
            }
            _ => None,
        }
    }

    fn io_read(&mut self, offset: u16) -> Option<u8> {
        let reg = Self::io_reg(offset)?;
        Some(match reg {
            0x3100..=0x32FF => self.cache[(reg - 0x3100) as usize],
            0x3000..=0x301F => {
                let v = self.r[((reg - 0x3000) >> 1) as usize];
                if reg & 1 == 0 { v as u8 } else { (v >> 8) as u8 }
            }
            0x3030 => self.sfr as u8,
            0x3031 => {
                let v = (self.sfr >> 8) as u8;
                self.sfr &= !sfr::IRQ;
                v
            }
            0x3034 => self.pbr,
            0x3036 => self.rombr,
            0x303B => self.vcr,
            0x303C => self.rambr,
            0x303E => self.cbr as u8,
            0x303F => (self.cbr >> 8) as u8,
            _ => 0,
        })
    }

    fn io_write(&mut self, offset: u16, v: u8, rom: &[u8], ram: &[u8]) {
        let Some(reg) = Self::io_reg(offset) else { return };
        match reg {
            0x3100..=0x32FF => {
                let k = (reg - 0x3100) as usize;
                self.cache[k] = v;
                if k & 15 == 15 {
                    self.lines |= 1 << (k >> 4);
                }
            }
            0x3000..=0x301F if reg & 1 == 0 => self.latch = v,
            0x3000..=0x301F => {
                let n = ((reg - 0x3000) >> 1) as usize;
                self.r[n] = (v as u16) << 8 | self.latch as u16;
                if n == 14 {
                    self.refill_rom_buffer(rom);
                    self.rom_ready = self.clock + self.rom_load() * self.cycle();
                }
                if n == 15 {
                    self.start(rom, ram);
                }
            }
            0x3030 => {
                let was = self.running();
                self.sfr = (self.sfr & !0x3E) | (v as u16 & 0x3E);
                if was && !self.running() {
                    self.cbr = 0;
                    self.lines = 0;
                } else if !was && self.running() {
                    self.start(rom, ram);
                }
            }
            0x3031 => self.sfr = (self.sfr & 0x00FF) | ((v as u16) << 8 & !sfr::IRQ) | (self.sfr & sfr::IRQ),
            0x3033 => self.bramr = v,
            0x3034 => {
                self.pbr = v;
                self.lines = 0;
            }
            0x3037 => self.cfgr = v,
            0x3038 => self.scbr = v,
            0x3039 => self.clsr = v,
            0x303A => self.scmr = v,
            _ => {}
        }
    }

    /// GO set from the S-CPU: the first opcode is fetched at R15.
    fn start(&mut self, rom: &[u8], ram: &[u8]) {
        self.sfr |= sfr::GO;
        self.pipe = self.code(self.r[15], rom, ram).0;
        self.jumped = false;
    }

    fn cycle(&self) -> u64 {
        if self.clsr & 1 != 0 { 1 } else { 2 }
    }

    /// A memory access's cycles, uncached: 3 at 10.74 MHz and 5 at 21.48 (fullsnes, "CPU Misc").
    fn slow(&self) -> u64 {
        if self.clsr & 1 != 0 { 5 } else { 3 }
    }

    /// An opcode byte at `pc` in PBR's bank: from the code cache when its line is loaded, else from ROM or RAM,
    /// loading the line when `pc` falls inside the 512 bytes CBR starts.
    fn code(&mut self, pc: u16, rom: &[u8], ram: &[u8]) -> (u8, u64) {
        let rel = pc.wrapping_sub(self.cbr);
        if rel < 512 {
            let line = (pc as usize & 0x1FF) >> 4;
            if self.lines & (1 << line) != 0 {
                return (self.cache[pc as usize & 0x1FF], 1);
            }
            let start = pc & !15;
            for k in 0..16u16 {
                let at = start.wrapping_add(k);
                self.cache[at as usize & 0x1FF] = self.fetch_memory(at, rom, ram);
            }
            self.lines |= 1 << line;
            // D-35: the whole line at slow() a byte, a state to begin and one to end, the CPU waiting for all of it.
            return (self.cache[pc as usize & 0x1FF], 16 * self.slow() + 2);
        }
        (self.fetch_memory(pc, rom, ram), self.slow())
    }

    fn fetch_memory(&self, pc: u16, rom: &[u8], ram: &[u8]) -> u8 {
        if self.pbr & 0x7F >= 0x70 {
            Self::ram_index(ram, ((self.pbr as usize & 1) << 16) | pc as usize).map_or(0, |i| ram[i])
        } else {
            Self::rom_at((self.pbr as u32) << 16 | pc as u32, rom.len()).map_or(0, |o| rom[o])
        }
    }

    /// D-35: a ROM-buffer load ends ROM_CYCLES + 4 cycles after the opcode that wrote R14.
    fn rom_load(&self) -> u64 {
        if self.clsr & 1 != 0 { 7 } else { 5 }
    }

    /// The cycles an opcode reading the ROM buffer waits for a load still running.
    fn rom_wait(&self) -> u64 {
        self.rom_ready.saturating_sub(self.clock).div_ceil(self.cycle())
    }

    fn refill_rom_buffer(&mut self, rom: &[u8]) {
        self.rom_buffer = Self::rom_at((self.rombr as u32) << 16 | self.r[14] as u32, rom.len()).map_or(0, |o| rom[o]);
    }

    fn ram_at(&self, a: u16) -> usize {
        (self.rambr as usize & 1) << 16 | a as usize
    }

    fn read_byte(&mut self, ram: &[u8], a: u16) -> u8 {
        self.ram_address = a;
        Self::ram_index(ram, self.ram_at(a)).map_or(0, |i| ram[i])
    }

    /// fullsnes: a word at an odd address is the even word with its bytes swapped.
    fn read_word(&mut self, ram: &[u8], a: u16) -> u16 {
        self.ram_address = a;
        let lo = Self::ram_index(ram, self.ram_at(a)).map_or(0, |i| ram[i]);
        let hi = Self::ram_index(ram, self.ram_at(a ^ 1)).map_or(0, |i| ram[i]);
        u16::from_le_bytes([lo, hi])
    }

    fn write_byte(&mut self, ram: &mut [u8], a: u16, v: u8) {
        self.ram_address = a;
        if let Some(i) = Self::ram_index(ram, self.ram_at(a)) {
            ram[i] = v;
        }
    }

    fn write_word(&mut self, ram: &mut [u8], a: u16, v: u16) {
        self.ram_address = a;
        let [lo, hi] = v.to_le_bytes();
        if let Some(i) = Self::ram_index(ram, self.ram_at(a)) {
            ram[i] = lo;
        }
        if let Some(i) = Self::ram_index(ram, self.ram_at(a ^ 1)) {
            ram[i] = hi;
        }
    }

    /// A register write; R15 jumps (after the byte already fetched), R14 refills the ROM buffer.
    fn set(&mut self, n: usize, v: u16, rom: &[u8]) {
        self.r[n] = v;
        if n == 15 {
            self.jumped = true;
        } else if n == 14 {
            self.refill_rom_buffer(rom);
            self.r14_written = true;
        }
    }

    fn set_sz(&mut self, v: u16) {
        self.sfr &= !(sfr::S | sfr::Z);
        if v & 0x8000 != 0 {
            self.sfr |= sfr::S;
        }
        if v == 0 {
            self.sfr |= sfr::Z;
        }
    }

    fn flag(&mut self, f: u16, on: bool) {
        if on { self.sfr |= f } else { self.sfr &= !f }
    }

    /// The operand byte after the opcode, taken from the pipeline, which is refilled from the next address.
    fn operand(&mut self, rom: &[u8], ram: &[u8]) -> (u8, u64) {
        let v = self.pipe;
        self.r[15] = self.r[15].wrapping_add(1);
        let (next, cost) = self.code(self.r[15], rom, ram);
        self.pipe = next;
        (v, cost)
    }

    /// Runs until the GSU's clock reaches `target`; stopped, it only lets the time pass.
    pub fn run_to(&mut self, target: u64, rom: &[u8], ram: &mut [u8]) {
        while self.clock < target {
            if !self.running() {
                self.clock = target;
                return;
            }
            let cycles = self.step(rom, ram);
            self.clock += cycles * self.cycle();
        }
    }

    /// One opcode: the pipelined byte executes while the next is fetched (fullsnes, "Jump Notes"). Returns cycles.
    fn step(&mut self, rom: &[u8], ram: &mut [u8]) -> u64 {
        let op = self.pipe;
        if self.jumped {
            self.jumped = false;
        } else {
            self.r[15] = self.r[15].wrapping_add(1);
        }
        let (next, mut cost) = self.code(self.r[15], rom, ram);
        self.pipe = next;
        cost += self.execute(op, rom, ram);
        if self.r14_written {
            self.r14_written = false;
            self.rom_ready = self.clock + (cost + self.rom_load()) * self.cycle();
        }
        cost
    }

    fn reset_prefix(&mut self) {
        self.alt = 0;
        self.b = false;
        self.sreg = 0;
        self.dreg = 0;
        self.sfr &= !0x1300;
    }

    fn alt_flags(&mut self) {
        self.sfr = (self.sfr & !0x0300) | (self.alt as u16) << 8;
    }

    /// One opcode by fullsnes's MOV, ALU, JMP and prefix tables; returns any cycles beyond the fetch.
    fn execute(&mut self, op: u8, rom: &[u8], ram: &mut [u8]) -> u64 {
        let n = (op & 15) as usize;
        let s = self.r[self.sreg as usize];
        let d = self.dreg as usize;
        let alt = self.alt;
        let mut extra = 0;
        match op {
            0x00 => {
                self.sfr &= !sfr::GO;
                self.sfr |= sfr::IRQ;
                self.r[15] = self.r[15].wrapping_add(1);
            }
            0x01 => {}
            0x02 => {
                if self.cbr != self.r[15] & 0xFFF0 {
                    self.cbr = self.r[15] & 0xFFF0;
                    self.lines = 0;
                }
            }
            0x03 => {
                let r = s >> 1;
                self.flag(sfr::CY, s & 1 != 0);
                self.set(d, r, rom);
                self.set_sz(r);
            }
            0x04 => {
                let r = (s << 1) | (self.sfr & sfr::CY != 0) as u16;
                self.flag(sfr::CY, s & 0x8000 != 0);
                self.set(d, r, rom);
                self.set_sz(r);
            }
            0x05..=0x0F => {
                let (off, c) = self.operand(rom, ram);
                extra += c;
                let f = self.sfr;
                let (sf, ov, z, cy) = (f & sfr::S != 0, f & sfr::OV != 0, f & sfr::Z != 0, f & sfr::CY != 0);
                let take = match op {
                    0x05 => true,
                    0x06 => sf == ov,
                    0x07 => sf != ov,
                    0x08 => !z,
                    0x09 => z,
                    0x0A => !sf,
                    0x0B => sf,
                    0x0C => !cy,
                    0x0D => cy,
                    0x0E => !ov,
                    _ => ov,
                };
                if take {
                    let target = self.r[15].wrapping_add(off as i8 as u16);
                    self.set(15, target, rom);
                }
                // Branches keep the prefixes (fullsnes, "GSU Prefix Opcodes").
                return extra;
            }
            0x10..=0x1F => {
                if self.b {
                    self.set(n, s, rom);
                } else {
                    self.dreg = n as u8;
                    return 0;
                }
            }
            0x20..=0x2F => {
                self.sreg = n as u8;
                self.dreg = n as u8;
                self.b = true;
                self.sfr |= 0x1000;
                return 0;
            }
            0x30..=0x3B => {
                let a = self.r[n];
                if alt & 1 != 0 {
                    self.write_byte(ram, a, s as u8);
                } else {
                    self.write_word(ram, a, s);
                }
                extra += self.slow();
            }
            0x3C => {
                let r = self.r[12].wrapping_sub(1);
                self.r[12] = r;
                self.set_sz(r);
                if r != 0 {
                    let to = self.r[13];
                    self.set(15, to, rom);
                }
            }
            0x3D..=0x3F => {
                self.alt = op - 0x3C;
                self.alt_flags();
                return 0;
            }
            0x40..=0x4B => {
                let a = self.r[n];
                let v = if alt & 1 != 0 { self.read_byte(ram, a) as u16 } else { self.read_word(ram, a) };
                self.set(d, v, rom);
                extra += self.slow() * if alt & 1 != 0 { 1 } else { 2 };
            }
            0x4C => {
                if alt & 1 != 0 {
                    let v = self.pixel(ram) as u16;
                    self.set(d, v, rom);
                    self.set_sz(v);
                    extra += 20;
                } else {
                    self.plot(ram);
                }
            }
            0x4D => {
                let r = s.rotate_right(8);
                self.set(d, r, rom);
                self.set_sz(r);
            }
            0x4E => {
                if alt & 1 != 0 {
                    self.por = s as u8 & 0x1F;
                } else {
                    self.colr = self.color_in(s as u8);
                }
            }
            0x4F => {
                let r = !s;
                self.set(d, r, rom);
                self.set_sz(r);
            }
            0x50..=0x6F => {
                let sub = op >= 0x60;
                let imm = if sub { alt == 2 } else { alt & 2 != 0 };
                let b = if imm { n as u16 } else { self.r[n] };
                let carry = self.sfr & sfr::CY != 0;
                let (r, cy, ov) = if !sub {
                    let c = (alt & 1 != 0 && carry) as u32;
                    let sum = s as u32 + b as u32 + c;
                    let r = sum as u16;
                    (r, sum > 0xFFFF, !(s ^ b) & (s ^ r) & 0x8000 != 0)
                } else {
                    let borrow = (alt == 1 && !carry) as i32;
                    let diff = s as i32 - b as i32 - borrow;
                    let r = diff as u16;
                    (r, diff >= 0, (s ^ b) & (s ^ r) & 0x8000 != 0)
                };
                self.flag(sfr::CY, cy);
                self.flag(sfr::OV, ov);
                self.set_sz(r);
                // ALT3 with SUB is CMP: the flags only.
                if !(sub && alt == 3) {
                    self.set(d, r, rom);
                }
            }
            0x70 => {
                let r = (self.r[7] & 0xFF00) | (self.r[8] >> 8);
                self.set(d, r, rom);
                self.flag(sfr::S, r & 0x8080 != 0);
                self.flag(sfr::OV, r & 0xC0C0 != 0);
                self.flag(sfr::CY, r & 0xE0E0 != 0);
                self.flag(sfr::Z, r & 0xF0F0 != 0);
            }
            0x71..=0x7F | 0xC1..=0xCF => {
                let b = if alt & 2 != 0 { n as u16 } else { self.r[n] };
                let r = match (op >= 0xC1, alt & 1 != 0) {
                    (false, false) => s & b,
                    (false, true) => s & !b,
                    (true, false) => s | b,
                    (true, true) => s ^ b,
                };
                self.set(d, r, rom);
                self.set_sz(r);
            }
            0x80..=0x8F => {
                let b = if alt & 2 != 0 { n as u16 } else { self.r[n] };
                let r = if alt & 1 != 0 { (s & 0xFF) * (b & 0xFF) } else { ((s as i8 as i16) * (b as i8 as i16)) as u16 };
                self.set(d, r, rom);
                self.set_sz(r);
                extra += if self.cfgr & 0x20 != 0 { 0 } else { 1 };
            }
            0x90 => {
                let a = self.ram_address;
                self.write_word(ram, a, s);
                extra += self.slow();
            }
            0x91..=0x94 => {
                let r = self.r[15].wrapping_add(n as u16);
                self.r[11] = r;
            }
            0x95 => {
                let r = s as u8 as i8 as i16 as u16;
                self.set(d, r, rom);
                self.set_sz(r);
            }
            0x96 => {
                let r = if alt & 1 != 0 && s == 0xFFFF { 0 } else { ((s as i16) >> 1) as u16 };
                self.flag(sfr::CY, s & 1 != 0);
                self.set(d, r, rom);
                self.set_sz(r);
            }
            0x97 => {
                let r = (s >> 1) | ((self.sfr & sfr::CY != 0) as u16) << 15;
                self.flag(sfr::CY, s & 1 != 0);
                self.set(d, r, rom);
                self.set_sz(r);
            }
            0x98..=0x9D => {
                if alt & 1 != 0 {
                    self.pbr = self.r[n] as u8;
                    self.set(15, s, rom);
                    self.cbr = s & 0xFFF0;
                    self.lines = 0;
                } else {
                    let to = self.r[n];
                    self.set(15, to, rom);
                }
            }
            0x9E => {
                let r = s & 0xFF;
                self.set(d, r, rom);
                self.flag(sfr::S, r & 0x80 != 0);
                self.flag(sfr::Z, r == 0);
            }
            0x9F => {
                let p = (s as i16 as i32) * (self.r[6] as i16 as i32);
                if alt & 1 != 0 {
                    self.r[4] = p as u16;
                }
                let r = (p >> 16) as u16;
                self.set(d, r, rom);
                self.flag(sfr::CY, p & 0x8000 != 0);
                self.set_sz(r);
                // D-35: three microcode cycles, then the multiplier's hold of 5 at MS0=0 and 1 at MS0=1, LMULT as FMULT.
                extra += 2 + if self.cfgr & 0x20 != 0 { 1 } else { 5 };
            }
            0xA0..=0xAF => {
                let (k, c) = self.operand(rom, ram);
                extra += c;
                match alt {
                    0 => self.set(n, k as i8 as i16 as u16, rom),
                    2 => {
                        let v = self.r[n];
                        self.write_word(ram, (k as u16) << 1, v);
                        extra += self.slow() * 2;
                    }
                    _ => {
                        let v = self.read_word(ram, (k as u16) << 1);
                        self.set(n, v, rom);
                        extra += self.slow() * 2;
                    }
                }
            }
            0xB0..=0xBF => {
                if self.b {
                    let r = self.r[n];
                    self.set(d, r, rom);
                    self.flag(sfr::OV, r & 0x80 != 0);
                    self.set_sz(r);
                } else {
                    self.sreg = n as u8;
                    return 0;
                }
            }
            0xC0 => {
                let r = s >> 8;
                self.set(d, r, rom);
                self.flag(sfr::S, r & 0x80 != 0);
                self.flag(sfr::Z, r == 0);
            }
            0xD0..=0xDE => {
                let r = self.r[n].wrapping_add(1);
                self.set(n, r, rom);
                self.set_sz(r);
            }
            0xDF => match alt {
                2 => self.rambr = s as u8 & 1,
                3 => self.rombr = s as u8 & 0x7F,
                _ => {
                    self.colr = self.color_in(self.rom_buffer);
                    extra += self.rom_wait();
                }
            },
            0xE0..=0xEE => {
                let r = self.r[n].wrapping_sub(1);
                self.set(n, r, rom);
                self.set_sz(r);
            }
            0xEF => {
                let v = self.rom_buffer as u16;
                let r = match alt {
                    0 => v,
                    1 => (v << 8) | (s & 0xFF),
                    2 => (s & 0xFF00) | v,
                    _ => v as u8 as i8 as i16 as u16,
                };
                self.set(d, r, rom);
                extra += self.rom_wait();
            }
            _ => {
                let (lo, c1) = self.operand(rom, ram);
                let (hi, c2) = self.operand(rom, ram);
                extra += c1 + c2;
                let k = u16::from_le_bytes([lo, hi]);
                match alt {
                    0 => self.set(n, k, rom),
                    2 => {
                        let v = self.r[n];
                        self.write_word(ram, k, v);
                        extra += self.slow() * 2;
                    }
                    _ => {
                        let v = self.read_word(ram, k);
                        self.set(n, v, rom);
                        extra += self.slow() * 2;
                    }
                }
            }
        }
        self.reset_prefix();
        extra
    }

    /// COLOR and GETC through POR's high-nibble and freeze-high bits.
    fn color_in(&self, v: u8) -> u8 {
        let v = if self.por & 0x04 != 0 { (v & 0xF0) | (v >> 4) } else { v };
        if self.por & 0x08 != 0 { (self.colr & 0xF0) | (v & 0x0F) } else { v }
    }

    fn bpp(&self) -> usize {
        match self.scmr & 3 {
            0 => 2,
            3 => 8,
            _ => 4,
        }
    }

    /// The RAM offset of the row holding pixel (x, y), by the screen height or OBJ mode (fullsnes, "Bitmap I/O").
    fn row_address(&self, x: u16, y: u16) -> usize {
        let height = if self.por & 0x10 != 0 { 3 } else { ((self.scmr >> 2) & 1) | ((self.scmr >> 4) & 2) };
        let (x, y) = (x as usize, y as usize);
        let tile = match height {
            0 => (x / 8) * 0x10 + y / 8,
            1 => (x / 8) * 0x14 + y / 8,
            2 => (x / 8) * 0x18 + y / 8,
            _ => (y / 0x80) * 0x200 + (x / 0x80) * 0x100 + ((y / 8) & 0x0F) * 0x10 + ((x / 8) & 0x0F),
        };
        tile * 8 * self.bpp() + (self.scbr as usize) * 0x400 + (y & 7) * 2
    }

    fn plot(&mut self, ram: &mut [u8]) {
        let (x, y) = (self.r[1] & 0xFF, self.r[2] & 0xFF);
        let bpp = self.bpp();
        let mut c = self.colr;
        if self.por & 0x02 != 0 && bpp != 8 && (x ^ y) & 1 != 0 {
            c >>= 4;
        }
        let mask = match bpp {
            2 => 0x03,
            4 => 0x0F,
            _ => if self.por & 0x08 != 0 { 0x0F } else { 0xFF },
        };
        if self.por & 0x01 != 0 || c & mask != 0 {
            let base = self.row_address(x, y);
            let bit = 7 - (x & 7);
            for p in 0..bpp {
                let at = base + (p >> 1) * 16 + (p & 1);
                if let Some(i) = Self::ram_index(ram, at) {
                    ram[i] = (ram[i] & !(1 << bit)) | (((c >> p) & 1) << bit);
                }
            }
        }
        self.r[1] = self.r[1].wrapping_add(1);
    }

    fn pixel(&self, ram: &[u8]) -> u8 {
        let (x, y) = (self.r[1] & 0xFF, self.r[2] & 0xFF);
        let base = self.row_address(x, y);
        let bit = 7 - (x & 7);
        (0..self.bpp()).fold(0, |c, p| {
            let at = base + (p >> 1) * 16 + (p & 1);
            c | ((Self::ram_index(ram, at).map_or(0, |i| ram[i]) >> bit) & 1) << p
        })
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut o = Vec::new();
        for v in self.r {
            o.extend(v.to_le_bytes());
        }
        for v in [self.sfr, self.cbr, self.ram_address] {
            o.extend(v.to_le_bytes());
        }
        o.extend([self.pbr, self.rombr, self.rambr, self.scbr, self.scmr, self.colr, self.por, self.bramr, self.cfgr, self.clsr, self.vcr]);
        o.extend([self.alt, self.b as u8, self.sreg, self.dreg, self.latch, self.pipe, self.jumped as u8, self.rom_buffer]);
        o.extend(self.lines.to_le_bytes());
        o.extend(self.clock.to_le_bytes());
        o.extend_from_slice(&self.cache);
        o.extend(self.rom_ready.to_le_bytes());
        o
    }

    pub fn unpack(&mut self, d: &[u8]) {
        let w = |i: usize| u16::from_le_bytes([d[i * 2], d[i * 2 + 1]]);
        for i in 0..16 {
            self.r[i] = w(i);
        }
        (self.sfr, self.cbr, self.ram_address) = (w(16), w(17), w(18));
        let b = &d[38..];
        (self.pbr, self.rombr, self.rambr, self.scbr, self.scmr, self.colr) = (b[0], b[1], b[2], b[3], b[4], b[5]);
        (self.por, self.bramr, self.cfgr, self.clsr, self.vcr) = (b[6], b[7], b[8], b[9], b[10]);
        (self.alt, self.b, self.sreg, self.dreg, self.latch) = (b[11] & 3, b[12] != 0, b[13] & 15, b[14] & 15, b[15]);
        (self.pipe, self.jumped, self.rom_buffer) = (b[16], b[17] != 0, b[18]);
        self.lines = u32::from_le_bytes(b[19..23].try_into().expect("four bytes"));
        self.clock = u64::from_le_bytes(b[23..31].try_into().expect("eight bytes"));
        self.cache.copy_from_slice(&b[31..31 + 512]);
        self.rom_ready = u64::from_le_bytes(b[543..551].try_into().expect("eight bytes"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A ROM whose bank 0 at $8000 holds `code`, and a GSU started there as the S-CPU starts it.
    fn run(code: &[u8], ram: &mut [u8]) -> Gsu {
        let mut rom = vec![0u8; 0x10_0000];
        rom[..code.len()].copy_from_slice(code);
        let mut g = Gsu::new(true);
        g.io_write(0x303A, 0x18, &rom, ram);
        g.io_write(0x301E, 0x00, &rom, ram);
        g.io_write(0x301F, 0x80, &rom, ram);
        g.run_to(100_000, &rom, ram);
        g
    }

    // fullsnes's ALU table: IBT loads a sign-extended byte, ADD and SUB set the flags, and TO/FROM pick the registers.
    #[test]
    fn the_alu_adds_and_subtracts_with_the_prefixes() {
        let mut ram = vec![0u8; 0x8000];
        // IBT R1,#7F; IBT R2,#01; FROM R1; TO R3; ADD R2; STOP; NOP
        let g = run(&[0xA1, 0x7F, 0xA2, 0x01, 0xB1, 0x13, 0x52, 0x00, 0x01], &mut ram);
        assert_eq!(g.r[3], 0x80);
        assert_eq!(g.sfr & (sfr::OV | sfr::S | sfr::Z | sfr::GO), 0);
        assert!(g.sfr & sfr::IRQ != 0);
    }

    // fullsnes, "Jump Notes": the byte after a branch is executed before its target.
    #[test]
    fn a_branch_executes_the_byte_after_it() {
        let mut ram = vec![0u8; 0x8000];
        // BRA +2 (from the byte after it); INC R1 (the delay byte); INC R2 (skipped); INC R3; STOP; NOP
        let g = run(&[0x05, 0x02, 0xD1, 0xD2, 0xD3, 0x00, 0x01], &mut ram);
        assert_eq!((g.r[1], g.r[2], g.r[3]), (1, 0, 1));
    }

    // PLOT at 2 bits in the 128-high screen: the pixel's two bitplanes at its tile row (fullsnes's address formula).
    #[test]
    fn plot_writes_a_pixels_bitplanes() {
        let mut ram = vec![0u8; 0x8000];
        // IBT R0,#03; COLOR; IBT R1,#09; IBT R2,#0A; PLOT; STOP; NOP
        let g = run(&[0xA0, 0x03, 0x4E, 0xA1, 0x09, 0xA2, 0x0A, 0x4C, 0x00, 0x01], &mut ram);
        assert_eq!(g.r[1], 10);
        let row = (9 / 8) * 0x10 * 16 + (10 / 8) * 16 + (10 & 7) * 2;
        assert_eq!((ram[row], ram[row + 1]), (0x40, 0x40));
    }
}
