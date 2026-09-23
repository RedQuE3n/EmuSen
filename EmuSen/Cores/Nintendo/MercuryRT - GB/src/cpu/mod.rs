//! C#'s `Cpu/Core/Cpu.cs`: the SM83's registers and flags. See Mercury_Cpu.md.

use crate::Skip;
use crate::state::{State, StateReader, StateResult, StateWriter};

pub const FLAG_Z: u8 = 0x80;
pub const FLAG_N: u8 = 0x40;
pub const FLAG_H: u8 = 0x20;
pub const FLAG_C: u8 = 0x10;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Cpu {
    pub cycles: i64,
    pub last_instruction_pc: u16,
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub halted: bool,
    pub ime: bool,
    pub l: u8,
    pub pc: u16,
    pub sp: u16,
    pub halt_bug: bool,
    pub ime_scheduled: bool,
    pub interrupt_pending: bool,
    /// Zero at every instruction boundary, so an accumulator and not machine state.
    pub ticked_this_step: Skip<i32>,
}

impl Cpu {
    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | self.f as u16
    }

    /// The flag register's low nibble is not wired, and reads back zero however it is written.
    pub fn set_af(&mut self, v: u16) {
        self.a = (v >> 8) as u8;
        self.f = (v & 0xF0) as u8;
    }

    pub fn set_bc(&mut self, v: u16) {
        (self.b, self.c) = ((v >> 8) as u8, v as u8);
    }

    pub fn set_de(&mut self, v: u16) {
        (self.d, self.e) = ((v >> 8) as u8, v as u8);
    }

    pub fn set_hl(&mut self, v: u16) {
        (self.h, self.l) = ((v >> 8) as u8, v as u8);
    }

    /// The post-boot-ROM registers, since Mercury starts with the cartridge - see Mercury_Cpu.md §5.
    pub fn reset(&mut self, cgb: bool) {
        self.set_af(if cgb { 0x1180 } else { 0x01B0 });
        self.set_bc(if cgb { 0x0000 } else { 0x0013 });
        self.set_de(if cgb { 0xFF56 } else { 0x00D8 });
        self.set_hl(if cgb { 0x000D } else { 0x014D });
        self.sp = 0xFFFE;
        self.pc = 0x0100;
        self.ime = false;
        self.halted = false;
        self.ime_scheduled = false;
        self.halt_bug = false;
        self.cycles = 0;
    }
}

impl State for Cpu {
    fn write_state(&self, w: &mut StateWriter) {
        w.i64("<Cycles>k__BackingField", self.cycles);
        w.u16("<LastInstructionPC>k__BackingField", self.last_instruction_pc);
        w.u8("A", self.a);
        w.u8("B", self.b);
        w.u8("C", self.c);
        w.u8("D", self.d);
        w.u8("E", self.e);
        w.u8("F", self.f);
        w.u8("H", self.h);
        w.bool("Halted", self.halted);
        w.bool("Ime", self.ime);
        w.u8("L", self.l);
        w.u16("PC", self.pc);
        w.u16("SP", self.sp);
        w.bool("_haltBug", self.halt_bug);
        w.bool("_imeScheduled", self.ime_scheduled);
        w.bool("_interruptPending", self.interrupt_pending);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.cycles = r.i64()?; // <Cycles>k__BackingField
        self.last_instruction_pc = r.u16()?; // <LastInstructionPC>k__BackingField
        self.a = r.u8()?; // A
        self.b = r.u8()?; // B
        self.c = r.u8()?; // C
        self.d = r.u8()?; // D
        self.e = r.u8()?; // E
        self.f = r.u8()?; // F
        self.h = r.u8()?; // H
        self.halted = r.bool()?; // Halted
        self.ime = r.bool()?; // Ime
        self.l = r.u8()?; // L
        self.pc = r.u16()?; // PC
        self.sp = r.u16()?; // SP
        self.halt_bug = r.bool()?; // _haltBug
        self.ime_scheduled = r.bool()?; // _imeScheduled
        self.interrupt_pending = r.bool()?; // _interruptPending
        Ok(())
    }
}
