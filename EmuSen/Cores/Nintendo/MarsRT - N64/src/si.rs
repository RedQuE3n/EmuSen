//! The serial interface, the C# `SiInterface`, with its four ports.

use crate::controller::Controller;
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
