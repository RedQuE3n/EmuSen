//! C#'s `Cpu/Core/Cpu.cs`: the SM83's registers, its step and its bus cycles. See Mercury_Cpu.md.

mod alu;
mod opcodes;

use crate::Skip;
use crate::state::{State, StateReader, StateResult, StateWriter};

pub const FLAG_Z: u8 = 0x80;
pub const FLAG_N: u8 = 0x40;
pub const FLAG_H: u8 = 0x20;
pub const FLAG_C: u8 = 0x10;

const INTERRUPT_VECTORS: [u16; 5] = [0x0040, 0x0048, 0x0050, 0x0058, 0x0060];

/// All the SM83 can see - C#'s `ICpuBus`, here a generic so the step is monomorphised.
pub trait CpuBus {
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, data: u8);
    /// Runs the rest of the machine forward while the CPU is mid-instruction - see Mercury_Cpu.md §3.
    fn tick(&mut self, cycles: i32);
    fn stop(&mut self);
}

/// An opcode no SM83 has; C# throws `NotSupportedException` with its address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IllegalOpcode {
    pub opcode: u8,
    pub pc: u16,
}

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

    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | self.c as u16
    }

    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | self.e as u16
    }

    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | self.l as u16
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

    /// What a Game Boy Color's boot ROM hands a Game Boy cartridge: B the title checksum, HL where the logo map was drawn - see Mercury_Model.md §3.2.
    pub fn reset_for_compatibility(&mut self, title_checksum: u8) {
        self.reset(true);
        self.set_af(0x1180);
        self.set_bc((title_checksum as u16) << 8);
        self.set_de(0x0008);
        self.set_hl(if matches!(title_checksum, 0x43 | 0x58) { 0x991A } else { 0x007C });
    }

    /// `Cpu.Step`: one instruction or one interrupt dispatch, ticking the bus as it goes; returns the T-cycles and the bit serviced.
    #[inline(always)]
    pub fn step<B: CpuBus>(&mut self, bus: &mut B, interrupt_enable: u8, interrupt_flags: u8) -> Result<(i32, i32), IllegalOpcode> {
        *self.ticked_this_step = 0;
        let pending = (interrupt_enable & interrupt_flags & 0x1F) != 0;
        self.interrupt_pending = pending;

        if self.halted {
            if !pending {
                return Ok((self.advance(bus, 4), -1));
            }
            self.halted = false;
        }

        if self.ime && pending {
            let bit = lowest_set_bit(interrupt_enable & interrupt_flags);
            let cycles = self.service_interrupt(bus, bit);
            return Ok((self.advance(bus, cycles), bit));
        }

        if self.ime_scheduled {
            self.ime_scheduled = false;
            self.ime = true;
        }

        self.last_instruction_pc = self.pc;
        let opcode = self.fetch(bus);
        if self.halt_bug {
            self.pc = self.pc.wrapping_sub(1);
            self.halt_bug = false;
        }

        let cycles = self.execute(bus, opcode)?;
        Ok((self.advance(bus, cycles), -1))
    }

    /// The cycles an instruction spends thinking rather than on the bus, settled at its end - see Mercury_Cpu.md §3.1.
    #[inline(always)]
    fn advance<B: CpuBus>(&mut self, bus: &mut B, cycles: i32) -> i32 {
        self.cycles += cycles as i64;
        if cycles > *self.ticked_this_step {
            bus.tick(cycles - *self.ticked_this_step);
        }
        cycles
    }

    /// One machine cycle: the rest of the machine runs, then the transfer lands at its end.
    #[inline(always)]
    fn read_cycle<B: CpuBus>(&mut self, bus: &mut B, address: u16) -> u8 {
        bus.tick(4);
        *self.ticked_this_step += 4;
        bus.read(address)
    }

    #[inline(always)]
    fn write_cycle<B: CpuBus>(&mut self, bus: &mut B, address: u16, data: u8) {
        bus.tick(4);
        *self.ticked_this_step += 4;
        bus.write(address, data);
    }

    fn service_interrupt<B: CpuBus>(&mut self, bus: &mut B, bit: i32) -> i32 {
        self.ime = false;
        self.push(bus, self.pc);
        self.pc = INTERRUPT_VECTORS[bit as usize];
        20
    }

    #[inline(always)]
    fn fetch<B: CpuBus>(&mut self, bus: &mut B) -> u8 {
        let pc = self.pc;
        self.pc = pc.wrapping_add(1);
        self.read_cycle(bus, pc)
    }

    #[inline(always)]
    fn fetch16<B: CpuBus>(&mut self, bus: &mut B) -> u16 {
        let low = self.fetch(bus) as u16;
        let high = self.fetch(bus) as u16;
        low | (high << 8)
    }

    fn push<B: CpuBus>(&mut self, bus: &mut B, value: u16) {
        self.sp = self.sp.wrapping_sub(1);
        self.write_cycle(bus, self.sp, (value >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        self.write_cycle(bus, self.sp, value as u8);
    }

    fn pop<B: CpuBus>(&mut self, bus: &mut B) -> u16 {
        let low = self.read_cycle(bus, self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        let high = self.read_cycle(bus, self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        low | (high << 8)
    }

    #[inline(always)]
    fn flag(&self, mask: u8) -> bool {
        self.f & mask != 0
    }

    #[inline(always)]
    fn set_flag(&mut self, mask: u8, on: bool) {
        if on {
            self.f |= mask;
        } else {
            self.f &= !mask;
        }
    }
}

fn lowest_set_bit(value: u8) -> i32 {
    (0..5).find(|bit| value & (1 << bit) != 0).unwrap_or(-1)
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
