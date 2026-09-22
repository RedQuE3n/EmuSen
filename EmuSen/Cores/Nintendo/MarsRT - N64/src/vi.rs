//! The video interface's serialized state, the C# `Vi`.

use crate::state::{State, StateReader, StateResult, StateWriter, boxed};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vi {
    /// `<Fields>k__BackingField`, the auto-property's backing field.
    pub fields: i64,
    pub debt: i64,
    pub field: bool,
    pub half_line: i32,
    pub held: Box<[i32; 625]>,
    pub registers: [u32; 14],
    pub was_blank: bool,
}

impl Default for Vi {
    fn default() -> Self {
        Vi {
            fields: 0,
            debt: 0,
            field: false,
            half_line: 0,
            held: boxed(0),
            registers: [0; 14],
            was_blank: false,
        }
    }
}

impl State for Vi {
    fn write_state(&self, w: &mut StateWriter) {
        w.i64("<Fields>k__BackingField", self.fields);
        w.i64("_debt", self.debt);
        w.bool("_field", self.field);
        w.i32("_halfLine", self.half_line);
        w.i32s("_held", &self.held[..]);
        w.u32s("_registers", &self.registers[..]);
        w.bool("_wasBlank", self.was_blank);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.fields = r.i64()?; // <Fields>k__BackingField
        self.debt = r.i64()?; // _debt
        self.field = r.bool()?; // _field
        self.half_line = r.i32()?; // _halfLine
        r.i32s(&mut self.held[..])?; // _held
        r.u32s(&mut self.registers[..])?; // _registers
        self.was_blank = r.bool()?; // _wasBlank
        Ok(())
    }
}
