//! C#'s `Memory/Mappers/`: the five boards, an enum where C# has `IMapper`. See Mercury_Memory.md §4.

use crate::memory::cartridge::Cartridge;
use crate::state::{StateReader, StateResult, StateWriter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mapper {
    NoMbc(NoMbc),
    Mbc1(Mbc1),
    Mbc2(Mbc2),
    Mbc3(Mbc3),
    Mbc5(Mbc5),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NoMbc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mbc1 {
    pub advanced_mode: bool,
    pub bank1: i32,
    pub bank2: i32,
    pub ram_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mbc2 {
    pub ram_enabled: bool,
    pub rom_bank: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mbc3 {
    pub clock: [u8; 5],
    pub cycles_into_second: i64,
    pub last_latch_write: u8,
    pub latched: [u8; 5],
    pub ram_bank: i32,
    pub ram_enabled: bool,
    pub rom_bank: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mbc5 {
    pub ram_bank: i32,
    pub ram_enabled: bool,
    pub rom_bank: i32,
    pub rumbling: bool,
}

impl Mapper {
    /// `Cartridge.BuildMapper`'s table, with each board's constructor values.
    pub fn for_type(kind: u8) -> Option<Mapper> {
        Some(match kind {
            0x00 | 0x08 | 0x09 => Mapper::NoMbc(NoMbc),
            0x01..=0x03 => Mapper::Mbc1(Mbc1 { advanced_mode: false, bank1: 1, bank2: 0, ram_enabled: false }),
            0x05 | 0x06 => Mapper::Mbc2(Mbc2 { ram_enabled: false, rom_bank: 1 }),
            0x0F..=0x13 => Mapper::Mbc3(Mbc3 {
                clock: [0; 5],
                cycles_into_second: 0,
                last_latch_write: 0xFF,
                latched: [0; 5],
                ram_bank: 0,
                ram_enabled: false,
                rom_bank: 1,
            }),
            0x19..=0x1E => Mapper::Mbc5(Mbc5 { ram_bank: 0, ram_enabled: false, rom_bank: 1, rumbling: false }),
            _ => return None,
        })
    }

    /// `$0000-$7FFF`.
    #[inline(always)]
    pub fn read_rom(&self, cart: &Cartridge, address: u16) -> u8 {
        let rom = &cart.rom;
        let bank = match self {
            Mapper::NoMbc(_) => return rom.get(address as usize).copied().unwrap_or(0xFF),
            _ if address < 0x4000 => match self {
                Mapper::Mbc1(m) => m.low_bank(cart),
                _ => 0,
            },
            Mapper::Mbc1(m) => m.high_bank(cart),
            Mapper::Mbc2(m) => m.rom_bank & rom_mask(cart),
            Mapper::Mbc3(m) => m.rom_bank & rom_mask(cart),
            Mapper::Mbc5(m) => m.rom_bank & rom_mask(cart),
        };
        let offset = (bank as usize).wrapping_mul(crate::memory::cartridge::ROM_BANK_SIZE).wrapping_add(address as usize & 0x3FFF);
        rom.get(offset).copied().unwrap_or(0xFF)
    }

    pub fn write_rom(&mut self, cart: &Cartridge, address: u16, data: u8) {
        match self {
            Mapper::NoMbc(_) => {}
            Mapper::Mbc1(m) => match address {
                ..0x2000 => m.ram_enabled = data & 0x0F == 0x0A,
                ..0x4000 => m.bank1 = if data & 0x1F == 0 { 1 } else { (data & 0x1F) as i32 },
                ..0x6000 => m.bank2 = (data & 0x03) as i32,
                _ => m.advanced_mode = data & 0x01 != 0,
            },
            Mapper::Mbc2(m) => {
                if address >= 0x4000 {
                    return;
                }
                if address & 0x0100 != 0 {
                    m.rom_bank = if data & 0x0F == 0 { 1 } else { (data & 0x0F) as i32 };
                } else {
                    m.ram_enabled = data & 0x0F == 0x0A;
                }
            }
            Mapper::Mbc3(m) => match address {
                ..0x2000 => m.ram_enabled = data & 0x0F == 0x0A,
                ..0x4000 => m.rom_bank = if data & 0x7F == 0 { 1 } else { (data & 0x7F) as i32 },
                ..0x6000 => m.ram_bank = (data & 0x0F) as i32,
                _ => {
                    if m.last_latch_write == 0x00 && data == 0x01 {
                        m.latched = m.clock;
                    }
                    m.last_latch_write = data;
                }
            },
            Mapper::Mbc5(m) => match address {
                ..0x2000 => m.ram_enabled = data & 0x0F == 0x0A,
                ..0x3000 => m.rom_bank = (m.rom_bank & 0x100) | data as i32,
                ..0x4000 => m.rom_bank = (m.rom_bank & 0xFF) | (((data & 0x01) as i32) << 8),
                ..0x6000 => {
                    if *cart.has_rumble {
                        m.rumbling = data & 0x08 != 0;
                        m.ram_bank = (data & 0x07) as i32;
                    } else {
                        m.ram_bank = (data & 0x0F) as i32;
                    }
                }
                _ => {}
            },
        }
    }

    /// `$A000-$BFFF`; $FF when RAM is absent or not enabled, matching an open bus.
    pub fn read_ram(&self, cart: &Cartridge, address: u16) -> u8 {
        let ram = &cart.ram;
        let banked = |enabled: bool, bank: i32| -> u8 {
            if !enabled || ram.is_empty() {
                return 0xFF;
            }
            let offset = (bank as usize).wrapping_mul(crate::memory::cartridge::RAM_BANK_SIZE).wrapping_add(address as usize & 0x1FFF);
            ram.get(offset).copied().unwrap_or(0xFF)
        };
        match self {
            Mapper::NoMbc(_) => ram.get(address as usize - 0xA000).copied().unwrap_or(0xFF),
            Mapper::Mbc1(m) => banked(m.ram_enabled, m.ram_bank()),
            Mapper::Mbc2(m) => {
                if !m.ram_enabled || ram.is_empty() {
                    return 0xFF;
                }
                ram[(address as usize - 0xA000) % 512] | 0xF0
            }
            Mapper::Mbc3(m) => {
                if !m.ram_enabled {
                    return 0xFF;
                }
                if m.clock_selected() {
                    return m.latched[(m.ram_bank - 0x08) as usize];
                }
                banked(true, m.ram_bank)
            }
            Mapper::Mbc5(m) => banked(m.ram_enabled, m.ram_bank),
        }
    }

    pub fn write_ram(&mut self, cart: &mut Cartridge, address: u16, data: u8) {
        let ram = &mut cart.ram;
        let mut banked = |enabled: bool, bank: i32| {
            if !enabled || ram.is_empty() {
                return;
            }
            let offset = (bank as usize).wrapping_mul(crate::memory::cartridge::RAM_BANK_SIZE).wrapping_add(address as usize & 0x1FFF);
            if let Some(b) = ram.get_mut(offset) {
                *b = data;
            }
        };
        match self {
            Mapper::NoMbc(_) => {
                if let Some(b) = ram.get_mut(address as usize - 0xA000) {
                    *b = data;
                }
            }
            Mapper::Mbc1(m) => banked(m.ram_enabled, m.ram_bank()),
            Mapper::Mbc2(m) => {
                if m.ram_enabled && !ram.is_empty() {
                    ram[(address as usize - 0xA000) % 512] = data & 0x0F;
                }
            }
            Mapper::Mbc3(m) => {
                if !m.ram_enabled {
                    return;
                }
                if m.clock_selected() {
                    m.clock[(m.ram_bank - 0x08) as usize] = data;
                    return;
                }
                banked(true, m.ram_bank)
            }
            Mapper::Mbc5(m) => banked(m.ram_enabled, m.ram_bank),
        }
    }

    /// Clocked with base-clock cycles, for the board with a real-time clock - see Mercury_Native.md §9.2.
    #[inline(always)]
    pub fn tick(&mut self, cart: &Cartridge, cycles: i32) {
        if let Mapper::Mbc3(m) = self {
            if !*cart.has_timer || m.clock[4] & 0x40 != 0 {
                return;
            }
            m.cycles_into_second += cycles as i64;
            while m.cycles_into_second >= crate::machine::CPU_CLOCK_HZ {
                m.cycles_into_second -= crate::machine::CPU_CLOCK_HZ;
                m.advance_one_second();
            }
        }
    }

    /// The board registers `regs` shows, as C#'s `DebugState` lists them.
    pub fn debug_state(&self, cart: &Cartridge) -> Vec<(&'static str, u64, i32)> {
        match self {
            Mapper::NoMbc(_) => vec![],
            Mapper::Mbc1(m) => vec![
                ("Bank1", m.bank1 as u64, 5),
                ("Bank2", m.bank2 as u64, 2),
                ("Mode", m.advanced_mode as u64, 1),
                ("RomBank", m.high_bank(cart) as u64, 7),
                ("RamEnabled", m.ram_enabled as u64, 1),
            ],
            Mapper::Mbc2(m) => vec![("RomBank", m.rom_bank as u64, 4), ("RamEnabled", m.ram_enabled as u64, 1)],
            Mapper::Mbc3(m) => vec![
                ("RomBank", m.rom_bank as u64, 7),
                ("RamBank", m.ram_bank as u64, 4),
                ("RamEnabled", m.ram_enabled as u64, 1),
                ("RtcS", m.clock[0] as u64, 8),
                ("RtcM", m.clock[1] as u64, 8),
                ("RtcH", m.clock[2] as u64, 8),
                ("RtcDL", m.clock[3] as u64, 8),
                ("RtcDH", m.clock[4] as u64, 8),
            ],
            Mapper::Mbc5(m) => vec![
                ("RomBank", m.rom_bank as u64, 9),
                ("RamBank", m.ram_bank as u64, 4),
                ("RamEnabled", m.ram_enabled as u64, 1),
                ("Rumble", m.rumbling as u64, 1),
            ],
        }
    }

    /// The board's fields in C#'s ordinal order, its `_cart` copy among them.
    pub fn write_state(&self, w: &mut StateWriter, cart: &Cartridge) {
        match self {
            Mapper::NoMbc(_) => cart.write_as_field(w, "_cart"),
            Mapper::Mbc1(m) => m.write_state(w, cart),
            Mapper::Mbc2(m) => m.write_state(w, cart),
            Mapper::Mbc3(m) => m.write_state(w, cart),
            Mapper::Mbc5(m) => m.write_state(w, cart),
        }
    }

    /// C# reads the `_cart` copy into the one cartridge object, so the copy read last is the one that stands.
    pub fn read_state(&mut self, r: &mut StateReader, cart: &mut Cartridge) -> StateResult {
        match self {
            Mapper::NoMbc(_) => cart.read_as_field(r), // _cart
            Mapper::Mbc1(m) => m.read_state(r, cart),
            Mapper::Mbc2(m) => m.read_state(r, cart),
            Mapper::Mbc3(m) => m.read_state(r, cart),
            Mapper::Mbc5(m) => m.read_state(r, cart),
        }
    }
}

impl Mbc1 {
    fn write_state(&self, w: &mut StateWriter, cart: &Cartridge) {
        w.bool("_advancedMode", self.advanced_mode);
        w.i32("_bank1", self.bank1);
        w.i32("_bank2", self.bank2);
        cart.write_as_field(w, "_cart");
        w.bool("_ramEnabled", self.ram_enabled);
    }

    fn read_state(&mut self, r: &mut StateReader, cart: &mut Cartridge) -> StateResult {
        self.advanced_mode = r.bool()?; // _advancedMode
        self.bank1 = r.i32()?; // _bank1
        self.bank2 = r.i32()?; // _bank2
        cart.read_as_field(r)?; // _cart
        self.ram_enabled = r.bool()?; // _ramEnabled
        Ok(())
    }
}

impl Mbc2 {
    fn write_state(&self, w: &mut StateWriter, cart: &Cartridge) {
        cart.write_as_field(w, "_cart");
        w.bool("_ramEnabled", self.ram_enabled);
        w.i32("_romBank", self.rom_bank);
    }

    fn read_state(&mut self, r: &mut StateReader, cart: &mut Cartridge) -> StateResult {
        cart.read_as_field(r)?; // _cart
        self.ram_enabled = r.bool()?; // _ramEnabled
        self.rom_bank = r.i32()?; // _romBank
        Ok(())
    }
}

impl Mbc3 {
    fn write_state(&self, w: &mut StateWriter, cart: &Cartridge) {
        cart.write_as_field(w, "_cart");
        w.bytes("_clock", &self.clock);
        w.i64("_cyclesIntoSecond", self.cycles_into_second);
        w.u8("_lastLatchWrite", self.last_latch_write);
        w.bytes("_latched", &self.latched);
        w.i32("_ramBank", self.ram_bank);
        w.bool("_ramEnabled", self.ram_enabled);
        w.i32("_romBank", self.rom_bank);
    }

    fn read_state(&mut self, r: &mut StateReader, cart: &mut Cartridge) -> StateResult {
        cart.read_as_field(r)?; // _cart
        r.bytes(&mut self.clock)?; // _clock
        self.cycles_into_second = r.i64()?; // _cyclesIntoSecond
        self.last_latch_write = r.u8()?; // _lastLatchWrite
        r.bytes(&mut self.latched)?; // _latched
        self.ram_bank = r.i32()?; // _ramBank
        self.ram_enabled = r.bool()?; // _ramEnabled
        self.rom_bank = r.i32()?; // _romBank
        Ok(())
    }
}

impl Mbc5 {
    fn write_state(&self, w: &mut StateWriter, cart: &Cartridge) {
        cart.write_as_field(w, "_cart");
        w.i32("_ramBank", self.ram_bank);
        w.bool("_ramEnabled", self.ram_enabled);
        w.i32("_romBank", self.rom_bank);
        w.bool("_rumbling", self.rumbling);
    }

    fn read_state(&mut self, r: &mut StateReader, cart: &mut Cartridge) -> StateResult {
        cart.read_as_field(r)?; // _cart
        self.ram_bank = r.i32()?; // _ramBank
        self.ram_enabled = r.bool()?; // _ramEnabled
        self.rom_bank = r.i32()?; // _romBank
        self.rumbling = r.bool()?; // _rumbling
        Ok(())
    }
}

/// C#'s `Math.Max(1, RomBanks) - 1`: a mask for the power-of-two images real boards carry.
#[inline(always)]
fn rom_mask(cart: &Cartridge) -> i32 {
    cart.rom_banks().max(1) as i32 - 1
}

impl Mbc1 {
    /// Only mode 1 lets $0000-$3FFF leave bank 0.
    fn low_bank(&self, cart: &Cartridge) -> i32 {
        if self.advanced_mode { (self.bank2 << 5) & rom_mask(cart) } else { 0 }
    }

    fn high_bank(&self, cart: &Cartridge) -> i32 {
        ((self.bank2 << 5) | self.bank1) & rom_mask(cart)
    }

    fn ram_bank(&self) -> i32 {
        if self.advanced_mode { self.bank2 } else { 0 }
    }
}

impl Mbc3 {
    fn clock_selected(&self) -> bool {
        (0x08..=0x0C).contains(&self.ram_bank)
    }

    /// Day 511 wraps to 0 and latches the overflow bit, which stays set until a game clears it.
    fn advance_one_second(&mut self) {
        let c = &mut self.clock;
        c[0] = c[0].wrapping_add(1);
        if c[0] < 60 {
            return;
        }
        c[0] = 0;
        c[1] = c[1].wrapping_add(1);
        if c[1] < 60 {
            return;
        }
        c[1] = 0;
        c[2] = c[2].wrapping_add(1);
        if c[2] < 24 {
            return;
        }
        c[2] = 0;
        c[3] = c[3].wrapping_add(1);
        if c[3] != 0 {
            return;
        }
        if c[4] & 0x01 != 0 {
            c[4] = (c[4] & !0x01) | 0x80;
        } else {
            c[4] |= 0x01;
        }
    }
}
