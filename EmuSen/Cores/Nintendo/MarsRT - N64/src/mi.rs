//! The MIPS interface, the C# `MiInterface`.

use crate::state::{State, StateReader, StateResult, StateWriter};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct MiInterface {
    pub ebus: bool,
    /// `Mask`, a `MiInterrupt` written as its int32.
    pub mask: i32,
    /// `Pending`, a `MiInterrupt` written as its int32.
    pub pending: i32,
    pub repeat_count: u32,
    pub repeating: bool,
    pub upper: bool,
}

impl State for MiInterface {
    fn write_state(&self, w: &mut StateWriter) {
        w.bool("Ebus", self.ebus);
        w.i32("Mask", self.mask);
        w.i32("Pending", self.pending);
        w.u32("RepeatCount", self.repeat_count);
        w.bool("Repeating", self.repeating);
        w.bool("Upper", self.upper);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.ebus = r.bool()?; // Ebus
        self.mask = r.i32()?; // Mask
        self.pending = r.i32()?; // Pending
        self.repeat_count = r.u32()?; // RepeatCount
        self.repeating = r.bool()?; // Repeating
        self.upper = r.bool()?; // Upper
        Ok(())
    }
}
