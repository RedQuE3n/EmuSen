//! The cartridge's save chip, the C# `SaveChip`, and its three devices.

use std::sync::Arc;

use crate::Skip;
use crate::joybus;
use crate::state::{State, StateReader, StateResult, StateWriter, boxed};

/// `N64SaveType`, as the int32 a state stores.
pub mod save_type {
    pub const UNKNOWN: i32 = 0;
    pub const NONE: i32 = 1;
    pub const EEPROM_4K: i32 = 2;
    pub const EEPROM_16K: i32 = 3;
    pub const SRAM_256K: i32 = 4;
    pub const SRAM_BANKED_768K: i32 = 5;
    pub const FLASH_RAM: i32 = 6;
    pub const SRAM_1M: i32 = 7;
}

/// `Sram.BankSize`.
pub const SRAM_BANK: usize = 0x8000;

/// `SaveChip`: the type, then the device the type builds (`Become`); any other type has none.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SaveChip {
    /// `Type`, an `N64SaveType` written as its int32.
    pub kind: i32,
    pub device: SaveDevice,
    /// `_saved`: an earlier run's file, held until the chip is known.
    pub saved: Skip<Option<Arc<Vec<u8>>>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum SaveDevice {
    #[default]
    None,
    Eeprom(Eeprom),
    Sram(Sram),
    Flash(FlashRam),
}

impl SaveChip {
    /// `Become`: the device a type builds, erased to 0xFF as the C# constructors leave it.
    pub fn new(kind: i32) -> Self {
        let mut chip = SaveChip { kind: save_type::UNKNOWN, device: SaveDevice::None, saved: Skip(None) };
        chip.become_type(kind);
        chip
    }

    /// `new SaveChip(type, saved)`: an unknown type waits for the game's first move.
    pub fn with_saved(kind: i32, saved: Option<Arc<Vec<u8>>>) -> Self {
        let mut chip = SaveChip { kind: save_type::UNKNOWN, device: SaveDevice::None, saved: Skip(saved) };
        if kind != save_type::UNKNOWN {
            chip.become_type(kind);
        }
        chip
    }

    fn become_type(&mut self, kind: i32) {
        use save_type::*;
        self.kind = kind;
        let saved = self.saved.as_deref().map(|v| &v[..]);
        match kind {
            EEPROM_4K | EEPROM_16K => self.device = SaveDevice::Eeprom(Eeprom::new(kind == EEPROM_16K).loaded(saved)),
            SRAM_256K => self.device = SaveDevice::Sram(Sram::new(1).loaded(saved)),
            SRAM_BANKED_768K => self.device = SaveDevice::Sram(Sram::new(3).loaded(saved)),
            SRAM_1M => self.device = SaveDevice::Sram(Sram::new(4).loaded(saved)),
            FLASH_RAM => self.device = SaveDevice::Flash(FlashRam::new().loaded(saved)),
            _ => {}
        }
    }

    /// `SaveChip.WriteState`.
    pub fn write_state(&self, w: &mut StateWriter) {
        w.group("Save", |w| {
            w.i32("Type", self.kind);
            match &self.device {
                SaveDevice::None => {}
                SaveDevice::Eeprom(d) => w.structure("Eeprom", d),
                SaveDevice::Sram(d) => w.structure("Sram", d),
                SaveDevice::Flash(d) => w.structure("Flash", d),
            }
        });
    }

    /// `SaveChip.ReadState`: the type rebuilds the device, which is then read whole; `_saved` stays.
    pub fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        *self = SaveChip::with_saved(r.i32()?, self.saved.take());
        match &mut self.device {
            SaveDevice::None => Ok(()),
            SaveDevice::Eeprom(d) => d.read_state(r),
            SaveDevice::Sram(d) => d.read_state(r),
            SaveDevice::Flash(d) => d.read_state(r),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Eeprom {
    pub data: Box<[u8; 0x800]>,
    pub dirty: bool,
    pub large: bool,
}

impl Eeprom {
    pub fn new(large: bool) -> Self {
        Eeprom { data: boxed(0xFF), dirty: false, large }
    }
}

impl State for Eeprom {
    fn write_state(&self, w: &mut StateWriter) {
        w.bytes("Data", &self.data[..]);
        w.bool("Dirty", self.dirty);
        w.bool("Large", self.large);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.bytes(&mut self.data[..])?; // Data
        self.dirty = r.bool()?; // Dirty
        self.large = r.bool()?; // Large
        Ok(())
    }
}

/// `Sram`: its data's length is fixed by the type that built it; `_banks` is read back as stored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sram {
    pub data: Vec<u8>,
    pub dirty: bool,
    pub banks: i32,
}

impl Sram {
    pub fn new(banks: i32) -> Self {
        Sram { data: vec![0xFF; banks as usize * SRAM_BANK], dirty: false, banks }
    }
}

impl State for Sram {
    fn write_state(&self, w: &mut StateWriter) {
        w.bytes("Data", &self.data);
        w.bool("Dirty", self.dirty);
        w.i32("_banks", self.banks);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.bytes(&mut self.data)?; // Data
        self.dirty = r.bool()?; // Dirty
        self.banks = r.i32()?; // _banks
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlashRam {
    pub data: Box<[u8; 0x2_0000]>,
    pub dirty: bool,
    /// `_mode`, a `FlashRam.Mode` written as its int32.
    pub mode: i32,
    pub page: [u8; 128],
    pub page_number: u32,
    pub status: u64,
}

impl FlashRam {
    pub fn new() -> Self {
        FlashRam { data: boxed(0xFF), dirty: false, mode: 0, page: [0; 128], page_number: 0, status: 0 }
    }
}

impl Default for FlashRam {
    fn default() -> Self {
        Self::new()
    }
}

impl State for FlashRam {
    fn write_state(&self, w: &mut StateWriter) {
        w.bytes("Data", &self.data[..]);
        w.bool("Dirty", self.dirty);
        w.i32("_mode", self.mode);
        w.bytes("_page", &self.page[..]);
        w.u32("_pageNumber", self.page_number);
        w.u64("_status", self.status);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.bytes(&mut self.data[..])?; // Data
        self.dirty = r.bool()?; // Dirty
        self.mode = r.i32()?; // _mode
        r.bytes(&mut self.page[..])?; // _page
        self.page_number = r.u32()?; // _pageNumber
        self.status = r.u64()?; // _status
        Ok(())
    }
}

fn load_into(data: &mut [u8], saved: Option<&[u8]>) {
    if let Some(saved) = saved {
        let n = saved.len().min(data.len());
        data[..n].copy_from_slice(&saved[..n]);
    }
}

/// `Eeprom` commands and kinds.
pub const EEPROM_INFO: u8 = 0x00;
pub const EEPROM_RESET: u8 = 0xFF;
pub const EEPROM_READ: u8 = 0x04;
pub const EEPROM_WRITE: u8 = 0x05;
pub const KIND_4KBIT: u8 = 0x80;
pub const KIND_16KBIT: u8 = 0xC0;
const EEPROM_BLOCK: usize = 8;

/// `FlashRam` status words and modes.
pub const STATUS_IDENTIFY: u64 = 0x1111_8001_00C2_001D;
pub const STATUS_ERASE: u64 = 0x1111_8008_00C2_001D;
pub const STATUS_PROGRAM: u64 = 0x1111_8004_00C2_001D;
pub const STATUS_READ: u64 = 0x1111_8004_F000_001D;
const MODE_READ: i32 = 1;
const MODE_STATUS: i32 = 2;
const MODE_ERASE: i32 = 3;
const MODE_PROGRAM: i32 = 4;
const FLASH_SIZE: usize = 0x2_0000;
const FLASH_PAGE: usize = 128;

impl SaveChip {
    pub fn dirty(&self) -> bool {
        match &self.device {
            SaveDevice::None => false,
            SaveDevice::Eeprom(d) => d.dirty,
            SaveDevice::Sram(d) => d.dirty,
            SaveDevice::Flash(d) => d.dirty,
        }
    }

    pub fn contents(&self) -> Option<&[u8]> {
        match &self.device {
            SaveDevice::None => None,
            SaveDevice::Eeprom(d) => Some(&d.data[..]),
            SaveDevice::Sram(d) => Some(&d.data[..]),
            SaveDevice::Flash(d) => Some(&d.data[..]),
        }
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        match &mut self.device {
            SaveDevice::None => {}
            SaveDevice::Eeprom(d) => d.dirty = dirty,
            SaveDevice::Sram(d) => d.dirty = dirty,
            SaveDevice::Flash(d) => d.dirty = dirty,
        }
    }

    /// `FromSaveLength`: what an earlier run's save says the chip was, by its length alone.
    pub fn from_save_length(length: usize) -> i32 {
        match length {
            0x200 | 0x800 => save_type::EEPROM_4K,
            0x8000 => save_type::SRAM_256K,
            0x18000 => save_type::SRAM_BANKED_768K,
            0x20000 => save_type::FLASH_RAM,
            _ => save_type::UNKNOWN,
        }
    }

    /// `AnswerJoybus`: the fifth channel; an EEPROM answers, an undecided chip answers as the smaller one.
    pub fn answer_joybus(&mut self, ram: &mut [u8; 64], command: usize, send: usize, receive: usize, wrote: &mut usize) -> bool {
        *wrote = 0;
        if send == 0 {
            return false;
        }
        if self.kind == save_type::UNKNOWN && matches!(ram[command], EEPROM_READ | EEPROM_WRITE) {
            self.become_type(save_type::EEPROM_4K);
        }
        if let SaveDevice::Eeprom(e) = &mut self.device {
            return e.answer(ram, command, send, receive, wrote);
        }
        if self.kind != save_type::UNKNOWN || !matches!(ram[command], EEPROM_INFO | EEPROM_RESET) {
            return false;
        }
        *wrote = 3;
        joybus::reply(ram, command + send, receive, &[0x00, KIND_4KBIT, 0x00]);
        true
    }

    /// The processor on the second domain: a first touch here names FlashRAM.
    pub fn read32(&mut self, offset: u32) -> u32 {
        if self.kind == save_type::UNKNOWN {
            self.become_type(save_type::FLASH_RAM);
        }
        match &self.device {
            SaveDevice::Flash(f) => f.read32(offset),
            SaveDevice::Sram(s) => {
                ((s.read8(offset) as u32) << 24)
                    | ((s.read8(offset.wrapping_add(1)) as u32) << 16)
                    | ((s.read8(offset.wrapping_add(2)) as u32) << 8)
                    | s.read8(offset.wrapping_add(3)) as u32
            }
            _ => Self::open_bus(offset),
        }
    }

    pub fn write32(&mut self, offset: u32, value: u32) {
        if self.kind == save_type::UNKNOWN {
            self.become_type(save_type::FLASH_RAM);
        }
        match &mut self.device {
            SaveDevice::Flash(f) => f.write32(offset, value),
            SaveDevice::Sram(s) => {
                for i in 0..4u32 {
                    s.write8(offset.wrapping_add(i), (value >> (24 - 8 * i)) as u8);
                }
            }
            _ => {}
        }
    }

    /// A transfer on the second domain: a first touch here names SRAM.
    pub fn dma_read8(&mut self, offset: u32) -> u8 {
        if self.kind == save_type::UNKNOWN {
            self.become_type(save_type::SRAM_256K);
        }
        match &self.device {
            SaveDevice::Sram(s) => s.read8(offset),
            SaveDevice::Flash(f) => f.dma_read8(offset),
            _ => 0,
        }
    }

    pub fn dma_write8(&mut self, offset: u32, value: u8) {
        if self.kind == save_type::UNKNOWN {
            self.become_type(save_type::SRAM_256K);
        }
        match &mut self.device {
            SaveDevice::Sram(s) => s.write8(offset, value),
            SaveDevice::Flash(f) => f.dma_write8(offset, value),
            _ => {}
        }
    }

    /// `OpenBus`: nothing answers, so the bus keeps the low half of the address twice.
    pub fn open_bus(offset: u32) -> u32 {
        (offset & 0xFFFF).wrapping_mul(0x0001_0001)
    }
}

impl Eeprom {
    fn loaded(mut self, saved: Option<&[u8]>) -> Self {
        load_into(&mut self.data[..], saved);
        self
    }

    pub fn answer(&mut self, ram: &mut [u8; 64], command: usize, send: usize, receive: usize, wrote: &mut usize) -> bool {
        *wrote = 0;
        let reply = command + send;
        match ram[command] {
            EEPROM_INFO | EEPROM_RESET => {
                *wrote = 3;
                joybus::reply(ram, reply, receive, &[0x00, if self.large { KIND_16KBIT } else { KIND_4KBIT }, 0x00]);
                true
            }
            EEPROM_READ => {
                if send < 2 {
                    return false;
                }
                *wrote = EEPROM_BLOCK;
                let at = ram[command + 1] as usize * EEPROM_BLOCK;
                let mut block = [0u8; EEPROM_BLOCK];
                block.copy_from_slice(&self.data[at..at + EEPROM_BLOCK]);
                joybus::reply(ram, reply, receive, &block);
                true
            }
            EEPROM_WRITE => {
                if send < 2 || receive < 1 {
                    return false;
                }
                let block = ram[command + 1] as usize * EEPROM_BLOCK;
                for i in 0..send - 2 {
                    self.data[block + (i & (EEPROM_BLOCK - 1))] = ram[command + 2 + i];
                }
                self.dirty = true;
                *wrote = 1;
                joybus::reply(ram, reply, receive, &[0x00]);
                true
            }
            _ => false,
        }
    }
}

impl Sram {
    fn loaded(mut self, saved: Option<&[u8]>) -> Self {
        load_into(&mut self.data, saved);
        self
    }

    /// `Locate`: one bank repeats across the domain; banked SRAM has nothing past its last bank.
    fn locate(&self, offset: u32) -> Option<usize> {
        if self.banks == 1 {
            return Some((offset & (SRAM_BANK as u32 - 1)) as usize);
        }
        let bank = ((offset >> 18) & 3) as i32;
        if bank < self.banks { Some(bank as usize * SRAM_BANK + (offset & (SRAM_BANK as u32 - 1)) as usize) } else { None }
    }

    pub fn read8(&self, offset: u32) -> u8 {
        self.locate(offset).and_then(|at| self.data.get(at).copied()).unwrap_or(0)
    }

    pub fn write8(&mut self, offset: u32, value: u8) {
        if let Some(at) = self.locate(offset) {
            self.data[at] = value;
            self.dirty = true;
        }
    }
}

impl FlashRam {
    fn loaded(mut self, saved: Option<&[u8]>) -> Self {
        load_into(&mut self.data[..], saved);
        self
    }

    pub fn read32(&self, offset: u32) -> u32 {
        if offset & 4 == 0 { (self.status >> 32) as u32 } else { self.status as u32 }
    }

    /// A write anywhere but the domain's first word is a command.
    pub fn write32(&mut self, offset: u32, value: u32) {
        if offset == 0 {
            return;
        }
        match (value >> 24) as u8 {
            0x4B => self.page_number = value & 0x3FF,
            0x78 => {
                self.mode = MODE_ERASE;
                self.status = STATUS_ERASE;
            }
            0xA5 => {
                self.page_number = value & 0x3FF;
                self.status = STATUS_PROGRAM;
            }
            0xB4 => self.mode = MODE_PROGRAM,
            0xD2 => self.execute(),
            0xE1 => {
                self.mode = MODE_STATUS;
                self.status = STATUS_IDENTIFY;
            }
            0xF0 => {
                self.mode = MODE_READ;
                self.status = STATUS_READ;
            }
            _ => {}
        }
    }

    pub fn dma_read8(&self, offset: u32) -> u8 {
        match self.mode {
            MODE_STATUS => (self.status >> (8 * (7 - (offset & 7)))) as u8,
            MODE_READ => self.data[offset as usize & (FLASH_SIZE - 1)],
            _ => 0,
        }
    }

    pub fn dma_write8(&mut self, offset: u32, value: u8) {
        self.page[offset as usize & (FLASH_PAGE - 1)] = value;
    }

    fn execute(&mut self) {
        let at = self.page_number as usize * FLASH_PAGE;
        match self.mode {
            MODE_ERASE => self.data[at..at + FLASH_PAGE].fill(0xFF),
            MODE_PROGRAM => self.data[at..at + FLASH_PAGE].copy_from_slice(&self.page),
            _ => return,
        }
        self.dirty = true;
    }
}
