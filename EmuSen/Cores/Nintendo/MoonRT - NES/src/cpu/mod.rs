//! C#'s `Cpu`: the 2A03's 6502, one bus access per cycle, with its interrupt sampling. See Moon_CPU.md.

mod opcodes;

use crate::memory::bus::MemoryBus;
use crate::state::{StateReader, StateResult, StateWriter};

pub const NMI_VECTOR: u16 = 0xFFFA;
pub const RESET_VECTOR: u16 = 0xFFFC;
pub const IRQ_VECTOR: u16 = 0xFFFE;
pub const UNSTABLE_MAGIC: u8 = 0xEE;
pub const RESET_CYCLES: i32 = 8;

pub const FLAG_C: u8 = 1 << 0;
pub const FLAG_Z: u8 = 1 << 1;
pub const FLAG_I: u8 = 1 << 2;
pub const FLAG_D: u8 = 1 << 3;
pub const FLAG_B: u8 = 1 << 4;
pub const FLAG_U: u8 = 1 << 5;
pub const FLAG_V: u8 = 1 << 6;
pub const FLAG_N: u8 = 1 << 7;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Cpu {
    pub a: u8,
    pub cycles: i64,
    pub jammed: bool,
    pub last_instruction_pc: u16,
    pub p: u8,
    pub pc: u16,
    pub s: u8,
    pub x: u8,
    pub y: u8,
    pub delayed_i: bool,
    pub has_delayed_i: bool,
    pub instruction_cycles: i32,
    pub irq_line: bool,
    pub irq_sampled_earlier: bool,
    pub irq_sampled_last: bool,
    pub nmi_line: bool,
    pub nmi_pending: bool,
    pub nmi_sampled_earlier: bool,
    pub nmi_sampled_last: bool,
    pub service_irq: bool,
    pub service_nmi: bool,
}

impl Cpu {
    #[inline(always)]
    pub fn flag(&self, flag: u8) -> bool {
        (self.p & flag) != 0
    }

    #[inline(always)]
    pub fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.p |= flag;
        } else {
            self.p &= !flag;
        }
    }

    /// Power-on: A/X/Y cleared, S at 0, then the soft reset's three phantom pushes.
    pub fn reset(&mut self, bus: &mut MemoryBus) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.s = 0;
        self.p = FLAG_I | FLAG_U;
        self.cycles = 0;
        self.soft_reset(bus);
    }

    /// RESET only sets I and subtracts 3 from S - see Moon_CPU.md §5.1.
    pub fn soft_reset(&mut self, bus: &mut MemoryBus) {
        self.s = self.s.wrapping_sub(3);
        self.p |= FLAG_I | FLAG_U;
        self.jammed = false;
        self.nmi_line = false;
        self.nmi_pending = false;
        self.irq_line = false;
        self.service_nmi = false;
        self.service_irq = false;
        self.nmi_sampled_last = false;
        self.nmi_sampled_earlier = false;
        self.irq_sampled_last = false;
        self.irq_sampled_earlier = false;
        self.has_delayed_i = false;
        self.delayed_i = false;
        self.instruction_cycles = 0;
        self.pc = bus.read(RESET_VECTOR) as u16 | ((bus.read(RESET_VECTOR + 1) as u16) << 8);
        self.last_instruction_pc = self.pc;
        for _ in 0..RESET_CYCLES {
            self.cycles = self.cycles.wrapping_add(1);
            self.tick(bus);
            self.sample_interrupts();
        }
    }

    /// Edge-triggered: armed on a rise, held until serviced.
    #[inline(always)]
    pub fn set_nmi_line(&mut self, level: bool) {
        if level && !self.nmi_line {
            self.nmi_pending = true;
        }
        self.nmi_line = level;
    }

    /// One instruction or one interrupt entry; the cycles it cost.
    pub fn step(&mut self, bus: &mut MemoryBus) -> i32 {
        self.instruction_cycles = 0;
        self.last_instruction_pc = self.pc;
        if self.jammed {
            self.read(bus, self.pc);
            return self.instruction_cycles;
        }
        if self.service_nmi {
            self.nmi_pending = false;
            self.service_interrupt(bus, NMI_VECTOR);
        } else if self.service_irq {
            self.service_interrupt(bus, IRQ_VECTOR);
        } else {
            let pc = self.pc;
            self.pc = pc.wrapping_add(1);
            let opcode = self.read(bus, pc);
            self.dispatch(bus, opcode);
        }
        self.poll_interrupts();
        self.instruction_cycles
    }

    #[inline(always)]
    fn poll_interrupts(&mut self) {
        self.service_nmi = self.nmi_sampled_earlier;
        self.service_irq = !self.service_nmi && self.irq_sampled_earlier;
        if self.has_delayed_i {
            let i = self.delayed_i;
            self.set_flag(FLAG_I, i);
            self.has_delayed_i = false;
        }
    }

    fn service_interrupt(&mut self, bus: &mut MemoryBus, vector: u16) {
        self.read(bus, self.pc);
        self.read(bus, self.pc);
        self.push(bus, (self.pc >> 8) as u8);
        self.push(bus, self.pc as u8);
        self.push(bus, (self.p & !FLAG_B) | FLAG_U);
        self.set_flag(FLAG_I, true);
        self.pc = self.read_vector(bus, vector);
    }

    fn read_vector(&mut self, bus: &mut MemoryBus, vector: u16) -> u16 {
        let lo = self.read(bus, vector) as u16;
        let hi = self.read(bus, vector.wrapping_add(1)) as u16;
        lo | (hi << 8)
    }

    /// `MemoryBus.Tick`, then the two lines it sets, in C#'s order (Moon_Native.md §2.5).
    #[inline(always)]
    fn tick(&mut self, bus: &mut MemoryBus) {
        let (nmi, irq) = bus.tick();
        self.set_nmi_line(nmi);
        self.irq_line = irq;
    }

    #[inline(always)]
    fn read(&mut self, bus: &mut MemoryBus, address: u16) -> u8 {
        self.begin_cycle(bus);
        let value = bus.read(address);
        self.sample_interrupts();
        value
    }

    #[inline(always)]
    fn write(&mut self, bus: &mut MemoryBus, address: u16, data: u8) {
        self.begin_cycle(bus);
        if let Some(irq) = bus.write(address, data, self.cycles) {
            self.irq_line = irq;
        }
        self.sample_interrupts();
    }

    #[inline(always)]
    fn begin_cycle(&mut self, bus: &mut MemoryBus) {
        self.instruction_cycles = self.instruction_cycles.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(1);
        self.tick(bus);
    }

    #[inline(always)]
    fn sample_interrupts(&mut self) {
        self.nmi_sampled_earlier = self.nmi_sampled_last;
        self.irq_sampled_earlier = self.irq_sampled_last;
        self.nmi_sampled_last = self.nmi_pending;
        self.irq_sampled_last = self.irq_line && !self.flag(FLAG_I);
    }

    /// A taken branch drops an IRQ that arrived only this cycle - see Moon_CPU.md §5.5.
    #[inline(always)]
    fn suppress_just_arrived_irq(&mut self) {
        if self.irq_sampled_last && !self.irq_sampled_earlier {
            self.irq_sampled_last = false;
        }
    }

    #[inline(always)]
    fn push(&mut self, bus: &mut MemoryBus, value: u8) {
        let s = self.s;
        self.s = s.wrapping_sub(1);
        self.write(bus, 0x0100 | s as u16, value);
    }

    #[inline(always)]
    fn pull(&mut self, bus: &mut MemoryBus) -> u8 {
        self.s = self.s.wrapping_add(1);
        self.read(bus, 0x0100 | self.s as u16)
    }

    fn pull_with_dummy(&mut self, bus: &mut MemoryBus) -> u8 {
        self.read(bus, 0x0100 | self.s as u16);
        self.pull(bus)
    }

    #[inline(always)]
    fn set_zero_negative(&mut self, value: u8) -> u8 {
        self.set_flag(FLAG_Z, value == 0);
        self.set_flag(FLAG_N, (value & 0x80) != 0);
        value
    }

    /// `Cpu`'s fields as C# walks them; `_bus` is the interface-typed field that writes the whole bus (Moon_Native.md §3.1).
    pub fn write_state(&self, w: &mut StateWriter, bus: &MemoryBus) {
        w.u8("A", self.a);
        w.i64("Cycles", self.cycles);
        w.bool("Jammed", self.jammed);
        w.u16("LastInstructionPC", self.last_instruction_pc);
        w.u8("P", self.p);
        w.u16("PC", self.pc);
        w.u8("S", self.s);
        w.u8("X", self.x);
        w.u8("Y", self.y);
        w.group_class("_bus", |w| bus.write_state(w));
        w.bool("_delayedI", self.delayed_i);
        w.bool("_hasDelayedI", self.has_delayed_i);
        w.i32("_instructionCycles", self.instruction_cycles);
        w.bool("_irqLine", self.irq_line);
        w.bool("_irqSampledEarlier", self.irq_sampled_earlier);
        w.bool("_irqSampledLast", self.irq_sampled_last);
        w.bool("_nmiLine", self.nmi_line);
        w.bool("_nmiPending", self.nmi_pending);
        w.bool("_nmiSampledEarlier", self.nmi_sampled_earlier);
        w.bool("_nmiSampledLast", self.nmi_sampled_last);
        w.bool("_serviceIrq", self.service_irq);
        w.bool("_serviceNmi", self.service_nmi);
    }

    pub fn read_state(&mut self, r: &mut StateReader, bus: &mut MemoryBus) -> StateResult {
        self.a = r.u8()?; // A
        self.cycles = r.i64()?; // Cycles
        self.jammed = r.bool()?; // Jammed
        self.last_instruction_pc = r.u16()?; // LastInstructionPC
        self.p = r.u8()?; // P
        self.pc = r.u16()?; // PC
        self.s = r.u8()?; // S
        self.x = r.u8()?; // X
        self.y = r.u8()?; // Y
        if r.present()? {
            bus.read_state(r)?;
        }
        self.delayed_i = r.bool()?; // _delayedI
        self.has_delayed_i = r.bool()?; // _hasDelayedI
        self.instruction_cycles = r.i32()?; // _instructionCycles
        self.irq_line = r.bool()?; // _irqLine
        self.irq_sampled_earlier = r.bool()?; // _irqSampledEarlier
        self.irq_sampled_last = r.bool()?; // _irqSampledLast
        self.nmi_line = r.bool()?; // _nmiLine
        self.nmi_pending = r.bool()?; // _nmiPending
        self.nmi_sampled_earlier = r.bool()?; // _nmiSampledEarlier
        self.nmi_sampled_last = r.bool()?; // _nmiSampledLast
        self.service_irq = r.bool()?; // _serviceIrq
        self.service_nmi = r.bool()?; // _serviceNmi
        Ok(())
    }
}
