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
