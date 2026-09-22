//! The peripheral interface, the C# `PiInterface`.

use crate::bus::MemoryBus;
use crate::mi::interrupt;
use crate::state::{State, StateReader, StateResult, StateWriter};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PiInterface {
    pub cart_address: u32,
    pub dram_address: u32,
    pub stored: u32,
    pub stored_until: i64,
}

impl State for PiInterface {
    fn write_state(&self, w: &mut StateWriter) {
        w.u32("_cartAddress", self.cart_address);
        w.u32("_dramAddress", self.dram_address);
        w.u32("_stored", self.stored);
        w.i64("_storedUntil", self.stored_until);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.cart_address = r.u32()?; // _cartAddress
        self.dram_address = r.u32()?; // _dramAddress
        self.stored = r.u32()?; // _stored
        self.stored_until = r.i64()?; // _storedUntil
        Ok(())
    }
}

/// `StoreDecayCycles`: the FPGA core's 150 cycles at 62.5MHz, in the processor's.
pub const STORE_DECAY_CYCLES: i64 = 225;
const BLOCK_SIZE: u32 = 128;
const ROW_SIZE: u32 = 0x800;

impl MemoryBus {
    /// `IoBusy`: while a processor store is still on the cartridge bus.
    #[inline]
    pub fn pi_io_busy(&self) -> bool {
        self.cycles < self.pi.stored_until
    }

    /// `CartridgeStore`: only the first store is kept; one arriving while the bus is busy is lost.
    pub fn pi_cartridge_store(&mut self, word: u32) {
        if self.pi_io_busy() {
            return;
        }
        self.pi.stored = word;
        self.pi.stored_until = self.cycles + STORE_DECAY_CYCLES;
    }

    /// `TakeStored`: a read takes the stored word back once, and frees the bus.
    pub fn pi_take_stored(&mut self) -> Option<u32> {
        if !self.pi_io_busy() {
            return None;
        }
        self.pi.stored_until = 0;
        Some(self.pi.stored)
    }

    pub fn pi_read32(&self, offset: u32) -> u32 {
        match offset & 0x3C {
            0x00 => self.pi.dram_address,
            0x04 => self.pi.cart_address,
            0x10 => {
                (if self.pi_io_busy() { 0x02 } else { 0 })
                    | if self.mi.pending & interrupt::PERIPHERAL_INTERFACE != 0 { 0x08 } else { 0 }
            }
            _ => 0,
        }
    }

    pub fn pi_write32(&mut self, offset: u32, value: u32) {
        match offset & 0x3C {
            0x00 => self.pi.dram_address = value & 0x00FF_FFFE,
            0x04 => self.pi.cart_address = value & 0xFFFF_FFFE,
            0x08 => self.pi_transfer(value, true),
            0x0C => self.pi_transfer(value, false),
            0x10 if value & 0x02 != 0 => self.mi.clear(interrupt::PERIPHERAL_INTERFACE),
            _ => {}
        }
    }

    fn pi_transfer(&mut self, encoded: u32, to_cartridge: bool) {
        let length = (encoded & 0x00FF_FFFF) + 1;
        if to_cartridge {
            self.pi_to_cartridge(length);
        } else {
            self.pi_from_cartridge(length);
        }
        self.mi.raise(interrupt::PERIPHERAL_INTERFACE);
    }

    fn pi_to_cartridge(&mut self, length: u32) {
        for i in 0..length {
            let byte = self.read8(self.pi.dram_address.wrapping_add(i));
            self.cartridge_dma_write8(self.pi.cart_address.wrapping_add(i), byte);
        }
        self.pi.cart_address = self.pi.cart_address.wrapping_add((length + 1) & !1);
        self.pi.dram_address = self.pi.dram_address.wrapping_add(length).wrapping_add(7) & !7;
    }

    /// `FromCartridge`: blocks of at most 128 bytes, the first paying for a misaligned address twice.
    fn pi_from_cartridge(&mut self, length: u32) {
        let mut remaining = length;
        let mut largest = BLOCK_SIZE;
        let mut first = true;
        while remaining > 0 {
            let misaligned = self.pi.dram_address & 7;
            let to_row_end = ROW_SIZE - (self.pi.dram_address & (ROW_SIZE - 1));
            let block = largest.wrapping_sub(misaligned).min(to_row_end).min(remaining);
            let read = (block + 1) & !1;
            let stored = block as i32 - misaligned as i32;
            let trimmed = first && (block as i64) < (BLOCK_SIZE as i64 - 1 - misaligned as i64);
            let mut at = 0i32;
            while at < stored {
                let from = self.pi.cart_address.wrapping_add(at as u32);
                let byte = self.cartridge_dma_read8(from);
                self.write8(self.pi.dram_address, byte);
                if !trimmed || at + 1 < stored {
                    let byte = self.cartridge_dma_read8(from.wrapping_add(1));
                    self.write8(self.pi.dram_address.wrapping_add(1), byte);
                }
                self.pi.dram_address = self.pi.dram_address.wrapping_add(2);
                at += 2;
            }
            self.pi.cart_address = self.pi.cart_address.wrapping_add(read);
            remaining = remaining.saturating_sub(read);
            largest = if to_row_end < 8 { BLOCK_SIZE - misaligned } else { BLOCK_SIZE };
            self.pi.dram_address = self.pi.dram_address.wrapping_add(7) & !7;
            first = false;
        }
    }
}
