//! The 65C816, written from WDC's W65C816S datasheet (Table 5-7 for every bus cycle) and graded by SingleStepTests.
//! One call on the bus per cycle; the bus decides what a cycle costs. See VenusRT_Native.md §10.

mod alu;
mod control;
mod ops;

pub use control::Interrupt;

/// The pins of a cycle, besides RWB, which the call's kind gives.
pub mod pin {
    pub const VDA: u8 = 1;
    pub const VPA: u8 = 2;
    pub const VPB: u8 = 4;
    pub const MLB: u8 = 8;
    pub const E: u8 = 16;
    pub const M: u8 = 32;
    pub const X: u8 = 64;
}

/// The processor status bits.
pub mod flag {
    pub const C: u8 = 0x01;
    pub const Z: u8 = 0x02;
    pub const I: u8 = 0x04;
    pub const D: u8 = 0x08;
    pub const X: u8 = 0x10;
    pub const M: u8 = 0x20;
    pub const V: u8 = 0x40;
    pub const N: u8 = 0x80;
}

/// What the CPU drives: one call per cycle. A read with neither VDA nor VPA is an internal cycle the bus must still
/// clock; `idle` is that cycle by name.
pub trait Bus {
    fn read(&mut self, address: u32, pins: u8) -> u8;
    fn write(&mut self, address: u32, value: u8, pins: u8);
    fn idle(&mut self, address: u32, pins: u8);
    /// A cycle in which WAI or STP holds the processor; the bus decides its length.
    fn halted(&mut self);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cpu {
    /// The accumulator C; A is its low byte, B its high.
    pub a: u16,
    pub x: u16,
    pub y: u16,
    pub s: u16,
    pub d: u16,
    pub dbr: u8,
    pub pbr: u8,
    pub pc: u16,
    pub p: u8,
    pub e: bool,
    /// WAI: halted until an interrupt line is taken.
    pub waiting: bool,
    /// STP: halted until reset.
    pub stopped: bool,
    /// The opcode is not one the core implements.
    pub unimplemented: bool,
}

/// How the byte after an effective address is found: within bank 0's 16 bits, or carried through all 24.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Wrap {
    Bank0,
    Linear,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Ea {
    pub address: u32,
    pub wrap: Wrap,
}

impl Ea {
    #[inline(always)]
    pub fn next(self) -> u32 {
        match self.wrap {
            Wrap::Bank0 => (self.address & 0xFF_0000) | (self.address.wrapping_add(1) & 0xFFFF),
            Wrap::Linear => self.address.wrapping_add(1) & 0xFF_FFFF,
        }
    }
}

/// The opcodes stage 1's second step added: branches, jumps, calls, returns, the stack, interrupts, moves, halts, WDM.
pub fn is_control(opcode: u8) -> bool {
    matches!(opcode, 0x10 | 0x30 | 0x50 | 0x70 | 0x90 | 0xB0 | 0xD0 | 0xF0 | 0x80 | 0x82 | 0x4C | 0x5C | 0x6C | 0x7C | 0xDC | 0x20
        | 0xFC | 0x22 | 0x60 | 0x6B | 0x40 | 0x00 | 0x02 | 0x48 | 0xDA | 0x5A | 0x8B | 0x4B | 0x08 | 0x0B | 0x68 | 0xFA | 0x7A
        | 0xAB | 0x28 | 0x2B | 0xF4 | 0xD4 | 0x62 | 0x44 | 0x54 | 0xCB | 0xDB | 0x42)
}

impl Cpu {
    #[inline(always)]
    pub fn m8(&self) -> bool {
        self.p & flag::M != 0
    }

    #[inline(always)]
    pub fn x8(&self) -> bool {
        self.p & flag::X != 0
    }

    /// E, M and X as pins, for every cycle.
    #[inline(always)]
    pub(crate) fn state_pins(&self) -> u8 {
        (if self.e { pin::E } else { 0 }) | (if self.m8() { pin::M } else { 0 }) | (if self.x8() { pin::X } else { 0 })
    }

    /// The width rules that P and E impose: emulation forces M and X and the stack's page; X set clears the indexes' high bytes.
    pub fn settle(&mut self) {
        if self.e {
            self.p |= flag::M | flag::X;
            self.s = 0x0100 | (self.s & 0xFF);
        }
        if self.x8() {
            self.x &= 0xFF;
            self.y &= 0xFF;
        }
    }

    #[inline(always)]
    pub(crate) fn pc_address(&self) -> u32 {
        ((self.pbr as u32) << 16) | self.pc as u32
    }

    #[inline(always)]
    pub(crate) fn fetch<B: Bus>(&mut self, bus: &mut B) -> u8 {
        let v = bus.read(self.pc_address(), pin::VPA | self.state_pins());
        self.pc = self.pc.wrapping_add(1);
        v
    }

    #[inline(always)]
    pub(crate) fn fetch16<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let lo = self.fetch(bus) as u16;
        lo | (self.fetch(bus) as u16) << 8
    }

    #[inline(always)]
    pub(crate) fn io<B: Bus>(&mut self, bus: &mut B, address: u32) {
        bus.idle(address, self.state_pins());
    }

    /// An internal cycle at the program counter's current address.
    #[inline(always)]
    pub(crate) fn io_pc<B: Bus>(&mut self, bus: &mut B) {
        let a = self.pc_address();
        self.io(bus, a);
    }

    /// An internal cycle at the operand byte just fetched (the datasheet's PBR,PC+1 of a two-byte instruction).
    #[inline(always)]
    pub(crate) fn io_operand<B: Bus>(&mut self, bus: &mut B) {
        let a = ((self.pbr as u32) << 16) | self.pc.wrapping_sub(1) as u32;
        self.io(bus, a);
    }

    #[inline(always)]
    pub(crate) fn read8<B: Bus>(&mut self, bus: &mut B, address: u32, extra: u8) -> u8 {
        bus.read(address, pin::VDA | extra | self.state_pins())
    }

    #[inline(always)]
    pub(crate) fn write8<B: Bus>(&mut self, bus: &mut B, address: u32, value: u8, extra: u8) {
        bus.write(address, value, pin::VDA | extra | self.state_pins());
    }

    pub(crate) fn read_data<B: Bus>(&mut self, bus: &mut B, ea: Ea, wide: bool) -> u16 {
        let lo = self.read8(bus, ea.address, 0) as u16;
        if wide { lo | (self.read8(bus, ea.next(), 0) as u16) << 8 } else { lo }
    }

    pub(crate) fn write_data<B: Bus>(&mut self, bus: &mut B, ea: Ea, value: u16, wide: bool) {
        self.write8(bus, ea.address, value as u8, 0);
        if wide {
            self.write8(bus, ea.next(), (value >> 8) as u8, 0);
        }
    }

    /// One instruction: the opcode fetch and everything it does; a halted cycle while WAI or STP holds.
    pub fn step<B: Bus>(&mut self, bus: &mut B) {
        if self.waiting || self.stopped {
            bus.halted();
            return;
        }
        let opcode = bus.read(self.pc_address(), pin::VDA | pin::VPA | self.state_pins());
        self.pc = self.pc.wrapping_add(1);
        self.execute(bus, opcode);
    }
}
