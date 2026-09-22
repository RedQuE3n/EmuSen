//! A controller port and the Controller Pak in its slot, the C# `Controller` and `ControllerPak`.

use crate::state::{State, StateReader, StateResult, StateWriter, boxed};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Controller {
    pub buttons: u16,
    pub present: bool,
    pub stick_x: i8,
    pub stick_y: i8,
    /// `Pak`: `[SkipInState]` in C#, and written by the bus's tail after the save chip.
    pub pak: Option<ControllerPak>,
}

impl State for Controller {
    fn write_state(&self, w: &mut StateWriter) {
        w.u16("Buttons", self.buttons);
        w.bool("Present", self.present);
        w.i8("StickX", self.stick_x);
        w.i8("StickY", self.stick_y);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.buttons = r.u16()?; // Buttons
        self.present = r.bool()?; // Present
        self.stick_x = r.i8()?; // StickX
        self.stick_y = r.i8()?; // StickY
        Ok(())
    }
}

/// `ControllerPak.Size`.
pub const PAK_SIZE: usize = 0x8000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControllerPak {
    pub data: Box<[u8; PAK_SIZE]>,
    pub dirty: bool,
}

impl Default for ControllerPak {
    fn default() -> Self {
        ControllerPak { data: boxed(0), dirty: false }
    }
}

impl State for ControllerPak {
    fn write_state(&self, w: &mut StateWriter) {
        w.bytes("Data", &self.data[..]);
        w.bool("Dirty", self.dirty);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.bytes(&mut self.data[..])?; // Data
        self.dirty = r.bool()?; // Dirty
        Ok(())
    }
}

/// `ControllerPak.ChunkSize`.
pub const CHUNK_SIZE: usize = 32;
const PAGE_SIZE: usize = 256;
const FIRST_FREE_PAGE: usize = 5;
const PAGES: usize = PAK_SIZE / PAGE_SIZE;

impl ControllerPak {
    /// `new ControllerPak(saved)`: an earlier run's file, or a formatted pak.
    pub fn new(saved: Option<&[u8]>) -> Self {
        let mut pak = ControllerPak::default();
        match saved {
            None => Self::format(&mut pak.data[..]),
            Some(saved) => {
                let n = saved.len().min(PAK_SIZE);
                pak.data[..n].copy_from_slice(&saved[..n]);
            }
        }
        pak
    }

    /// `Address`: the low five bits are the address's own CRC, which nothing checks.
    pub fn address(high: u8, low: u8) -> usize {
        ((high as usize) << 8) | (low as usize & 0xE0)
    }

    pub fn read(&self, address: usize, into: &mut [u8]) {
        if address < PAK_SIZE {
            into.copy_from_slice(&self.data[address..address + CHUNK_SIZE]);
        } else {
            into.fill(0);
        }
    }

    pub fn write(&mut self, address: usize, from: &[u8]) {
        if address >= PAK_SIZE {
            return;
        }
        self.data[address..address + CHUNK_SIZE].copy_from_slice(from);
        self.dirty = true;
    }

    /// `DataCrc`: CRC-8 over the chunk and one more zero byte, polynomial 0x85.
    pub fn data_crc(data: &[u8]) -> u8 {
        let mut crc: u8 = 0;
        for i in 0..=data.len() {
            let mut mask = 0x80u32;
            while mask >= 1 {
                let tap = if crc & 0x80 != 0 { 0x85 } else { 0 };
                crc <<= 1;
                if i < data.len() && (data[i] as u32 & mask) != 0 {
                    crc |= 1;
                }
                crc ^= tap;
                mask >>= 1;
            }
        }
        crc
    }

    /// `Format`: an empty pak as mupen64plus lays it out, with a fixed serial.
    pub fn format(pak: &mut [u8]) {
        pak.fill(0);
        let mut id = [0u8; CHUNK_SIZE];
        id[25] = 0x01;
        id[24] = 0x00;
        id[26] = 0x01;
        let mut sum: u16 = 0;
        for i in (0..28).step_by(2) {
            sum = sum.wrapping_add(((id[i] as u16) << 8) | id[i + 1] as u16);
        }
        let inverse = 0xFFF2u16.wrapping_sub(sum);
        id[28] = (sum >> 8) as u8;
        id[29] = sum as u8;
        id[30] = (inverse >> 8) as u8;
        id[31] = inverse as u8;
        for copy in [1usize, 3, 4, 6] {
            pak[copy * CHUNK_SIZE..(copy + 1) * CHUNK_SIZE].copy_from_slice(&id);
        }
        let mut index = [0u8; PAGE_SIZE];
        for page in FIRST_FREE_PAGE..PAGES {
            index[2 * page + 1] = 0x03;
        }
        let mut check: u8 = 0;
        for &b in &index[2 * FIRST_FREE_PAGE..PAGE_SIZE] {
            check = check.wrapping_add(b);
        }
        index[1] = check;
        pak[PAGE_SIZE..2 * PAGE_SIZE].copy_from_slice(&index);
        pak[2 * PAGE_SIZE..3 * PAGE_SIZE].copy_from_slice(&index);
    }
}
