//! The cartridge's save chip, the C# `SaveChip`, and its three devices.

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
        use save_type::*;
        let device = match kind {
            EEPROM_4K | EEPROM_16K => SaveDevice::Eeprom(Eeprom::new(kind == EEPROM_16K)),
            SRAM_256K => SaveDevice::Sram(Sram::new(1)),
            SRAM_BANKED_768K => SaveDevice::Sram(Sram::new(3)),
            SRAM_1M => SaveDevice::Sram(Sram::new(4)),
            FLASH_RAM => SaveDevice::Flash(FlashRam::new()),
            _ => SaveDevice::None,
        };
        SaveChip { kind, device }
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

    /// `SaveChip.ReadState`: the type rebuilds the device, which is then read whole.
    pub fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        *self = SaveChip::new(r.i32()?);
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
