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
