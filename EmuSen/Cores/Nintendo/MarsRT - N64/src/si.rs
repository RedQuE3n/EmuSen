//! The serial interface, the C# `SiInterface`, with its four ports.

use crate::bus::MemoryBus;
use crate::controller::Controller;
use crate::joybus;
use crate::mi::interrupt;
use crate::rom::Cic;
use crate::state::{State, StateReader, StateResult, StateWriter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SiInterface {
    pub controllers: [Controller; 4],
    pub dram_address: u32,
    /// `Due`: the cycle a transfer under way ends, or `i64::MAX`; carried in the bus's register tail, not here.
    pub due: i64,
    /// `PendingRead`: where a read's 64 bytes land when it ends, or -1; carried beside `due`.
    pub pending_read: i64,
}

impl Default for SiInterface {
    fn default() -> Self {
        SiInterface { controllers: Default::default(), dram_address: 0, due: i64::MAX, pending_read: -1 }
    }
}

impl State for SiInterface {
    fn write_state(&self, w: &mut StateWriter) {
        w.structures("Controllers", &self.controllers);
        w.u32("_dramAddress", self.dram_address);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.structures(&mut self.controllers)?; // Controllers
        self.dram_address = r.u32()?; // _dramAddress
        Ok(())
    }
}

/// `TransferCycles`: how long a transfer holds the interface before its interrupt.
pub const TRANSFER_CYCLES: i64 = 0x900 * 2;
const CHALLENGE_REQUEST: u8 = 0x02;
const CHALLENGE_AT: usize = 0x30;
const CHALLENGE_LENGTH: usize = 15;
const PIF_RAM_SIZE: usize = 64;

impl MemoryBus {
    /// `Catch`: the interrupt a transfer earned, raised once its time has passed, and a read's bytes with it.
    pub fn si_catch(&mut self) {
        if self.cycles < self.si.due {
            return;
        }
        self.si.due = i64::MAX;
        self.si_land();
        self.mi.raise(interrupt::SERIAL_INTERFACE);
    }

    /// `Land`: PIF RAM into memory at the end of the transfer.
    fn si_land(&mut self) {
        if self.si.pending_read < 0 {
            return;
        }
        let to = self.si.pending_read as u32;
        self.si.pending_read = -1;
        for i in 0..PIF_RAM_SIZE as u32 {
            let address = to.wrapping_add(i);
            if address as usize >= self.rdram.len() {
                break;
            }
            self.rdram[address as usize] = self.pif_ram[i as usize];
        }
        *self.written += 1;
    }

    pub fn si_read32(&self, offset: u32) -> u32 {
        match offset & 0x1C {
            0x00 => self.si.dram_address,
            0x18 => {
                (if self.si.due != i64::MAX { 0x0001 } else { 0 })
                    | if self.mi.pending & interrupt::SERIAL_INTERFACE != 0 { 0x1000 } else { 0 }
            }
            _ => 0,
        }
    }

    pub fn si_write32(&mut self, offset: u32, value: u32) {
        match offset & 0x1C {
            0x00 => self.si.dram_address = value & 0x00FF_FFF8,
            0x04 => self.si_transfer(false),
            0x10 => self.si_transfer(true),
            0x18 => self.mi.clear(interrupt::SERIAL_INTERFACE),
            _ => {}
        }
    }

    /// `Transfer`: sixty-four bytes; on the way in the PIF runs whatever block the game asked it to.
    fn si_transfer(&mut self, to_pif: bool) {
        self.si_land();
        if !to_pif {
            if self.pif_ram[PIF_RAM_SIZE - 1] & CHALLENGE_REQUEST != 0 {
                self.si_answer_challenge();
            } else {
                joybus::run(&mut self.pif_ram, &mut self.si.controllers, &mut self.save);
            }
        }
        if to_pif {
            for i in 0..PIF_RAM_SIZE as u32 {
                let address = self.si.dram_address.wrapping_add(i);
                if address as usize >= self.rdram.len() {
                    break;
                }
                self.pif_ram[i as usize] = self.rdram[address as usize];
            }
        } else {
            self.si.pending_read = self.si.dram_address as i64;
        }
        *self.written += 1;
        self.si.due = self.cycles + TRANSFER_CYCLES;
        self.sooner(self.si.due);
    }

    /// `AnswerChallenge`: only a 6105 answers; for any other chip the request is dropped.
    fn si_answer_challenge(&mut self) {
        if self.cart.as_ref().is_some_and(|c| c.cic == Cic::Nus6105) {
            self.pif_ram[CHALLENGE_AT - 2] = 0;
            self.pif_ram[CHALLENGE_AT - 1] = 0;
            let mut challenge = [0u8; CHALLENGE_LENGTH];
            challenge.copy_from_slice(&self.pif_ram[CHALLENGE_AT..CHALLENGE_AT + CHALLENGE_LENGTH]);
            crate::rom::respond(&challenge, &mut self.pif_ram[CHALLENGE_AT..CHALLENGE_AT + CHALLENGE_LENGTH]);
        }
        self.pif_ram[PIF_RAM_SIZE - 1] &= !CHALLENGE_REQUEST;
    }
}
