//! The peripheral interface, the C# `PiInterface`.

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
