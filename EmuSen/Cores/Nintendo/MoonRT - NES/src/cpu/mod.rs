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
    /// `_vectored`: BRK and the interrupt sequence end without polling.
    pub vectored: crate::Skip<bool>,
    /// `_unstableHalted`: SH*'s dummy read before the write was halted by DMA.
    pub unstable_halted: crate::Skip<bool>,
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
            self.end_cycle(bus);
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
            self.service_interrupt(bus, NMI_VECTOR, false);
        } else if self.service_irq {
            self.service_interrupt(bus, IRQ_VECTOR, true);
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
        self.service_nmi = !*self.vectored && self.nmi_sampled_earlier;
        self.service_irq = !*self.vectored && !self.service_nmi && self.irq_sampled_earlier;
        *self.vectored = false;
        if self.has_delayed_i {
            let i = self.delayed_i;
            self.set_flag(FLAG_I, i);
            self.has_delayed_i = false;
        }
    }

    fn service_interrupt(&mut self, bus: &mut MemoryBus, vector: u16, hijackable: bool) {
        self.read(bus, self.pc);
        self.read(bus, self.pc);
        let from = self.pc;
        self.push(bus, (self.pc >> 8) as u8);
        self.push(bus, self.pc as u8);
        let vector = if hijackable { self.hijack_vector(vector) } else { self.no_hijack(vector) };
        self.push(bus, (self.p & !FLAG_B) | FLAG_U);
        self.set_flag(FLAG_I, true);
        self.pc = self.read_vector(bus, vector);
        if *bus.observing {
            let kind = if vector == NMI_VECTOR { emusen_native::debug::kind::NMI } else { emusen_native::debug::kind::IRQ };
            bus.hooks.note_call(from as u32, self.pc as u32, kind);
        }
    }

    /// `HijackVector`: an NMI seen by the fourth cycle of BRK or IRQ takes over its vector, and is spent.
    pub(crate) fn hijack_vector(&mut self, vector: u16) -> u16 {
        *self.vectored = true;
        if !self.nmi_pending {
            return vector;
        }
        self.nmi_pending = false;
        NMI_VECTOR
    }

    fn no_hijack(&mut self, vector: u16) -> u16 {
        *self.vectored = true;
        vector
    }

    fn read_vector(&mut self, bus: &mut MemoryBus, vector: u16) -> u16 {
        let lo = self.read(bus, vector) as u16;
        let hi = self.read(bus, vector.wrapping_add(1)) as u16;
        lo | (hi << 8)
    }

    /// `MemoryBus.Tick`, then the two lines it sets, in C#'s order (Moon_Native.md §2.5).
    #[inline(always)]
    fn tick(&mut self, bus: &mut MemoryBus) {
        self.irq_line = bus.tick();
    }

    /// `MemoryBus.EndCycle`: the third dot, then /NMI as the cycle's end sees it.
    #[inline(always)]
    fn end_cycle(&mut self, bus: &mut MemoryBus) {
        let nmi = bus.end_cycle();
        self.set_nmi_line(nmi);
    }

    #[inline(always)]
    fn read(&mut self, bus: &mut MemoryBus, address: u16) -> u8 {
        if bus.dma_pending() {
            self.run_dma(bus, address);
        }
        self.begin_cycle(bus);
        let value = bus.read(address);
        self.end_cycle(bus);
        self.sample_interrupts();
        value
    }

    #[inline(always)]
    fn write(&mut self, bus: &mut MemoryBus, address: u16, data: u8) {
        self.begin_cycle(bus);
        if let Some(irq) = bus.write(address, data, self.cycles) {
            self.irq_line = irq;
        }
        self.end_cycle(bus);
        self.sample_interrupts();
    }

    #[inline(always)]
    fn begin_cycle(&mut self, bus: &mut MemoryBus) {
        self.instruction_cycles = self.instruction_cycles.wrapping_add(1);
        self.cycles = self.cycles.wrapping_add(1);
        *bus.cpu_cycles = self.cycles;
        self.tick(bus);
    }

    /// `MemoryBus.RunDma`: every DMA that wants the bus, run while the CPU is halted on a read of `address`.
    #[cold]
    fn run_dma(&mut self, bus: &mut MemoryBus, address: u16) {
        loop {
            if *bus.oam_dma_pending {
                self.run_oam_dma(bus, address);
            } else if bus.apu.dmc.dma_requested() {
                self.run_dmc_dma(bus, address);
            } else {
                return;
            }
        }
    }

    /// `RunDmcDma`: the DMA commits after its halt; a request gone by then costs the halt alone.
    fn run_dmc_dma(&mut self, bus: &mut MemoryBus, address: u16) {
        self.halted_read(bus, address);
        if !bus.apu.dmc.dma_requested() {
            return;
        }
        self.halted_read(bus, address);
        if !bus.apu.next_cycle_is_get() {
            self.halted_read(bus, address);
        }
        self.dmc_get(bus, address);
    }

    /// `NoteDmcHalt`: the cycle about to run is a DMC request's halt, if one has just risen.
    fn note_dmc_halt(&self, bus: &MemoryBus, dmc_halt: &mut i64) {
        if *dmc_halt < 0 && bus.apu.dmc.dma_requested() {
            *dmc_halt = self.cycles.wrapping_add(1);
        }
    }

    /// `RunOamDma`: the halt, an alignment cycle before a put, then a get read and a put write per byte, with a DMC fetch alongside.
    fn run_oam_dma(&mut self, bus: &mut MemoryBus, address: u16) {
        *bus.oam_dma_pending = false;
        let source = (*bus.oam_dma_page as u16) << 8;
        let mut dmc_halt = -1i64;
        self.note_dmc_halt(bus, &mut dmc_halt);
        self.halted_read(bus, address);
        if !bus.apu.next_cycle_is_get() {
            self.note_dmc_halt(bus, &mut dmc_halt);
            self.halted_read(bus, address);
        }
        for i in 0..256u16 {
            self.note_dmc_halt(bus, &mut dmc_halt);
            while dmc_halt >= 0 && self.cycles.wrapping_add(1) >= dmc_halt + 2 {
                self.dmc_get(bus, address);
                dmc_halt = -1;
                self.note_dmc_halt(bus, &mut dmc_halt);
                self.begin_cycle(bus);
                self.end_cycle(bus);
                self.sample_interrupts();
                self.note_dmc_halt(bus, &mut dmc_halt);
            }
            self.begin_cycle(bus);
            let value = bus.dma_read(source.wrapping_add(i), address, true);
            self.end_cycle(bus);
            self.sample_interrupts();
            self.note_dmc_halt(bus, &mut dmc_halt);
            self.begin_cycle(bus);
            let slot = bus.ppu.oam_address.wrapping_add(i as u8);
            bus.ppu.oam[slot as usize] = value;
            self.end_cycle(bus);
            self.sample_interrupts();
        }
        if dmc_halt < 0 || !bus.apu.dmc.dma_requested() {
            return;
        }
        if self.cycles - dmc_halt + 1 < 2 {
            self.halted_read(bus, address);
        }
        if !bus.apu.next_cycle_is_get() {
            self.halted_read(bus, address);
        }
        self.dmc_get(bus, address);
    }

    fn halted_read(&mut self, bus: &mut MemoryBus, address: u16) {
        self.begin_cycle(bus);
        bus.read(address);
        self.end_cycle(bus);
        self.sample_interrupts();
    }

    /// `DmcGet`: the fetch reads through the DMA's view of the bus.
    fn dmc_get(&mut self, bus: &mut MemoryBus, halted: u16) {
        self.begin_cycle(bus);
        let fetch = bus.apu.dmc.dma_address();
        let value = bus.dma_read(fetch, halted, false);
        bus.apu.dmc.complete_dma(value);
        self.end_cycle(bus);
        self.sample_interrupts();
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
