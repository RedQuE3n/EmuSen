//! Branches, jumps, calls and returns, the stack, BRK, COP, RTI and the hardware interrupt sequence, the block moves,
//! WAI, STP and WDM, cycle by cycle from the datasheet's Table 5-7. See VenusRT_Native.md §11.

use super::{Bus, Cpu, flag, pin};

/// The hardware interrupt lines; the machine decides when one is taken, between instructions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interrupt {
    Reset,
    Abort,
    Nmi,
    Irq,
}

impl Interrupt {
    /// The vector's address in bank 0, by mode (the datasheet's Table 6-1).
    pub fn vector(self, emulation: bool) -> u16 {
        match (self, emulation) {
            (Interrupt::Reset, _) => 0xFFFC,
            (Interrupt::Abort, true) => 0xFFF8,
            (Interrupt::Abort, false) => 0xFFE8,
            (Interrupt::Nmi, true) => 0xFFFA,
            (Interrupt::Nmi, false) => 0xFFEA,
            (Interrupt::Irq, true) => 0xFFFE,
            (Interrupt::Irq, false) => 0xFFEE,
        }
    }
}

const COP_VECTOR: (u16, u16) = (0xFFF4, 0xFFE4);
const BRK_VECTOR: (u16, u16) = (0xFFFE, 0xFFE6);

impl Cpu {
    // The 6502's stack: in emulation mode it stays in page 1.
    fn push<B: Bus>(&mut self, bus: &mut B, v: u8) {
        let s = self.s as u32;
        self.write8(bus, s, v, 0);
        self.s = if self.e { 0x0100 | (self.s.wrapping_sub(1) & 0xFF) } else { self.s.wrapping_sub(1) };
    }

    fn pull<B: Bus>(&mut self, bus: &mut B) -> u8 {
        self.s = if self.e { 0x0100 | (self.s.wrapping_add(1) & 0xFF) } else { self.s.wrapping_add(1) };
        let s = self.s as u32;
        self.read8(bus, s, 0)
    }

    // The 65816's own stack opcodes address S in 16 bits even in emulation mode; S is put back in page 1 after.
    fn push_wide<B: Bus>(&mut self, bus: &mut B, v: u8) {
        let s = self.s as u32;
        self.write8(bus, s, v, 0);
        self.s = self.s.wrapping_sub(1);
    }

    fn pull_wide<B: Bus>(&mut self, bus: &mut B) -> u8 {
        self.s = self.s.wrapping_add(1);
        let s = self.s as u32;
        self.read8(bus, s, 0)
    }

    fn stack_back_in_page(&mut self) {
        if self.e {
            self.s = 0x0100 | (self.s & 0xFF);
        }
    }

    fn push16<B: Bus>(&mut self, bus: &mut B, v: u16, wide_rules: bool) {
        if wide_rules {
            self.push_wide(bus, (v >> 8) as u8);
            self.push_wide(bus, v as u8);
        } else {
            self.push(bus, (v >> 8) as u8);
            self.push(bus, v as u8);
        }
    }

    fn program_read<B: Bus>(&mut self, bus: &mut B, offset: u16) -> u8 {
        bus.read(((self.pbr as u32) << 16) | offset as u32, pin::VPA | self.state_pins())
    }

    fn vector<B: Bus>(&mut self, bus: &mut B, at: u16) -> u16 {
        let lo = self.read8(bus, at as u32, pin::VPB) as u16;
        lo | (self.read8(bus, at.wrapping_add(1) as u32, pin::VPB) as u16) << 8
    }

    /// 22j: BRK and COP, with the signature byte fetched; emulation mode pushes no bank (note 7).
    fn software_interrupt<B: Bus>(&mut self, bus: &mut B, vectors: (u16, u16)) {
        self.fetch(bus);
        if !self.e {
            let b = self.pbr;
            self.push(bus, b);
        }
        let pc = self.pc;
        self.push16(bus, pc, false);
        let p = self.p;
        self.push(bus, p);
        self.enter_handler(bus, if self.e { vectors.0 } else { vectors.1 });
    }

    fn enter_handler<B: Bus>(&mut self, bus: &mut B, at: u16) {
        self.p = (self.p | flag::I) & !flag::D;
        self.pbr = 0;
        self.pc = self.vector(bus, at);
    }

    /// 22a: a hardware interrupt between instructions. The opcode at PC is fetched and dropped, an internal cycle
    /// follows, and PC itself is pushed; in emulation mode P goes with its B bit clear (note 11). A reset reads the
    /// stack instead of writing it (note 10) and conditions the registers first (§2.25).
    pub fn interrupt<B: Bus>(&mut self, bus: &mut B, kind: Interrupt) {
        self.waiting = false;
        if kind == Interrupt::Reset {
            self.stopped = false;
            self.e = true;
            self.d = 0;
            self.dbr = 0;
            self.pbr = 0;
            self.p = (self.p | flag::M | flag::X | flag::I) & !flag::D;
            self.settle();
        }
        let at = self.pc_address();
        bus.read(at, pin::VDA | pin::VPA | self.state_pins());
        self.io(bus, at);
        let pushes = [(!self.e).then_some(self.pbr), Some((self.pc >> 8) as u8), Some(self.pc as u8), Some(if self.e { self.p & !0x10 } else { self.p })];
        for v in pushes.into_iter().flatten() {
            if kind == Interrupt::Reset {
                let s = self.s as u32;
                self.read8(bus, s, 0);
                self.s = 0x0100 | (self.s.wrapping_sub(1) & 0xFF);
            } else {
                self.push(bus, v);
            }
        }
        let at = kind.vector(self.e);
        self.enter_handler(bus, at);
    }

    /// 20: a taken branch adds a cycle, and in emulation mode a second when it crosses a page (notes 5 and 6).
    fn branch<B: Bus>(&mut self, bus: &mut B, taken: bool) {
        let offset = self.fetch(bus) as i8 as u16;
        if !taken {
            return;
        }
        self.io_operand(bus);
        let target = self.pc.wrapping_add(offset);
        if self.e && target & 0xFF00 != self.pc & 0xFF00 {
            self.io_operand(bus);
        }
        self.pc = target;
    }

    fn pull_into(&mut self, v: u16, wide: bool) -> u16 {
        self.set_nz(v, wide);
        v
    }

    pub(crate) fn execute_control<B: Bus>(&mut self, bus: &mut B, opcode: u8) {
        match opcode {
            0x10 | 0x30 | 0x50 | 0x70 | 0x90 | 0xB0 | 0xD0 | 0xF0 => {
                let bit = [flag::N, flag::V, flag::C, flag::Z][(opcode >> 6) as usize];
                let set = self.p & bit != 0;
                self.branch(bus, set == (opcode & 0x20 != 0));
            }
            0x80 => self.branch(bus, true),
            // 21
            0x82 => {
                let offset = self.fetch16(bus);
                self.io_operand(bus);
                self.pc = self.pc.wrapping_add(offset);
            }
            // 1b, 4b
            0x4C => self.pc = self.fetch16(bus),
            0x5C => {
                let target = self.fetch16(bus);
                self.pbr = self.fetch(bus);
                self.pc = target;
            }
            // 3b, 3a
            0x6C | 0xDC => {
                let a = self.fetch16(bus);
                let lo = self.read8(bus, a as u32, 0) as u16;
                let hi = self.read8(bus, a.wrapping_add(1) as u32, 0) as u16;
                if opcode == 0xDC {
                    self.pbr = self.read8(bus, a.wrapping_add(2) as u32, 0);
                }
                self.pc = lo | hi << 8;
            }
            // 2a
            0x7C => {
                let a = self.fetch16(bus).wrapping_add(self.x);
                self.io_operand(bus);
                let lo = self.program_read(bus, a) as u16;
                self.pc = lo | (self.program_read(bus, a.wrapping_add(1)) as u16) << 8;
            }
            // 1c
            0x20 => {
                let target = self.fetch16(bus);
                self.io_operand(bus);
                let back = self.pc.wrapping_sub(1);
                self.push16(bus, back, false);
                self.pc = target;
            }
            // 2b
            0xFC => {
                let lo = self.fetch(bus) as u16;
                let back = self.pc;
                self.push16(bus, back, true);
                let a = (lo | (self.fetch(bus) as u16) << 8).wrapping_add(self.x);
                self.io_operand(bus);
                let lo = self.program_read(bus, a) as u16;
                self.pc = lo | (self.program_read(bus, a.wrapping_add(1)) as u16) << 8;
                self.stack_back_in_page();
            }
            // 4c
            0x22 => {
                let target = self.fetch16(bus);
                let b = self.pbr;
                let at = self.s as u32;
                self.push_wide(bus, b);
                self.io(bus, at);
                let bank = self.fetch(bus);
                let back = self.pc.wrapping_sub(1);
                self.push16(bus, back, true);
                self.pbr = bank;
                self.pc = target;
                self.stack_back_in_page();
            }
            // 22h
            0x60 => {
                self.io_pc(bus);
                self.io_pc(bus);
                let lo = self.pull(bus) as u16;
                let hi = self.pull(bus) as u16;
                let s = self.s as u32;
                self.io(bus, s);
                self.pc = (lo | hi << 8).wrapping_add(1);
            }
            // 22i
            0x6B => {
                self.io_pc(bus);
                self.io_pc(bus);
                let lo = self.pull_wide(bus) as u16;
                let hi = self.pull_wide(bus) as u16;
                self.pbr = self.pull_wide(bus);
                self.pc = (lo | hi << 8).wrapping_add(1);
                self.stack_back_in_page();
            }
            // 22g
            0x40 => {
                self.io_pc(bus);
                self.io_pc(bus);
                let p = self.pull(bus);
                let lo = self.pull(bus) as u16;
                let hi = self.pull(bus) as u16;
                self.pc = lo | hi << 8;
                if !self.e {
                    self.pbr = self.pull(bus);
                }
                self.p = p;
                self.settle();
            }
            0x00 => self.software_interrupt(bus, BRK_VECTOR),
            0x02 => self.software_interrupt(bus, COP_VECTOR),
            // 22c: PHA, PHX, PHY, PHB, PHK, PHP, and PHD by the 65816's own rule.
            0x48 | 0xDA | 0x5A | 0x8B | 0x4B | 0x08 | 0x0B => {
                self.io_pc(bus);
                let (v, wide) = match opcode {
                    0x48 => (self.a, !self.m8()),
                    0xDA => (self.x, !self.x8()),
                    0x5A => (self.y, !self.x8()),
                    0x8B => (self.dbr as u16, false),
                    0x4B => (self.pbr as u16, false),
                    0x08 => (self.p as u16, false),
                    _ => (self.d, true),
                };
                if opcode == 0x0B {
                    self.push16(bus, v, true);
                    self.stack_back_in_page();
                } else if wide {
                    self.push16(bus, v, false);
                } else {
                    self.push(bus, v as u8);
                }
            }
            // 22b: PLA, PLX, PLY, PLB and PLP, and PLD and PLB by the 65816's own rule.
            0x68 | 0xFA | 0x7A | 0xAB | 0x28 | 0x2B => {
                self.io_pc(bus);
                self.io_pc(bus);
                match opcode {
                    0x2B => {
                        let lo = self.pull_wide(bus) as u16;
                        let v = lo | (self.pull_wide(bus) as u16) << 8;
                        self.d = self.pull_into(v, true);
                        self.stack_back_in_page();
                    }
                    0x28 => {
                        self.p = self.pull(bus);
                        self.settle();
                    }
                    // PLB addresses S in 16 bits, as gilyon's cputest finds and the datasheet's list omits (D-3).
                    0xAB => {
                        let v = self.pull_wide(bus) as u16;
                        self.dbr = self.pull_into(v, false) as u8;
                        self.stack_back_in_page();
                    }
                    _ => {
                        let wide = if opcode == 0x68 { !self.m8() } else { !self.x8() };
                        let lo = self.pull(bus) as u16;
                        let v = if wide { lo | (self.pull(bus) as u16) << 8 } else { lo };
                        let v = self.pull_into(v, wide);
                        match opcode {
                            0x68 => self.set_acc(v),
                            0xFA => self.x = v,
                            _ => self.y = v,
                        }
                    }
                }
            }
            // 22d
            0xF4 => {
                let v = self.fetch16(bus);
                self.push16(bus, v, true);
                self.stack_back_in_page();
            }
            // 22e: the pointer is read as (d)'s is.
            0xD4 => {
                let o = self.fetch(bus) as u16;
                if self.d & 0xFF != 0 {
                    self.io_operand(bus);
                }
                let at = self.d.wrapping_add(o) as u32;
                let lo = self.read8(bus, at, 0) as u16;
                let v = lo | (self.read8(bus, (at + 1) & 0xFFFF, 0) as u16) << 8;
                self.push16(bus, v, true);
                self.stack_back_in_page();
            }
            // 22f
            0x62 => {
                let offset = self.fetch16(bus);
                self.io_operand(bus);
                let v = self.pc.wrapping_add(offset);
                self.push16(bus, v, true);
                self.stack_back_in_page();
            }
            // 9a, 9b: one byte a repetition; the opcode is fetched again until the count runs out.
            0x44 | 0x54 => {
                let dst = self.fetch(bus);
                let src = self.fetch(bus);
                let from = ((src as u32) << 16) | self.x as u32;
                let v = self.read8(bus, from, 0);
                let to = ((dst as u32) << 16) | self.y as u32;
                self.write8(bus, to, v, 0);
                self.io(bus, to);
                self.io(bus, to);
                self.dbr = dst;
                let mask = if self.x8() { 0xFF } else { 0xFFFF };
                let (x, y) = if opcode == 0x54 { (self.x.wrapping_add(1), self.y.wrapping_add(1)) } else { (self.x.wrapping_sub(1), self.y.wrapping_sub(1)) };
                self.x = x & mask;
                self.y = y & mask;
                self.a = self.a.wrapping_sub(1);
                if self.a != 0xFFFF {
                    self.pc = self.pc.wrapping_sub(3);
                }
            }
            // 19d, 19c
            0xCB | 0xDB => {
                self.io_pc(bus);
                self.io_pc(bus);
                if opcode == 0xCB { self.waiting = true } else { self.stopped = true }
            }
            // WDM: two bytes, the second fetched and dropped (D-41).
            0x42 => {
                self.fetch(bus);
            }
            _ => self.unimplemented = true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Log {
        memory: std::collections::HashMap<u32, u8>,
        cycles: Vec<(char, u32, u8, u8)>,
    }

    impl Bus for Log {
        fn read(&mut self, address: u32, pins: u8) -> u8 {
            let v = *self.memory.get(&address).unwrap_or(&0);
            self.cycles.push(('r', address, v, pins));
            v
        }
        fn write(&mut self, address: u32, value: u8, pins: u8) {
            self.memory.insert(address, value);
            self.cycles.push(('w', address, value, pins));
        }
        fn idle(&mut self, address: u32, pins: u8) {
            self.cycles.push(('i', address, 0, pins));
        }
        fn halted(&mut self) {
            self.cycles.push(('h', 0, 0, 0));
        }
    }

    fn vectored(at: u16) -> Log {
        let mut log = Log::default();
        log.memory.insert(at as u32, 0x34);
        log.memory.insert(at as u32 + 1, 0x12);
        log
    }

    // Table 5-7, 22a, native: the dropped fetch, an internal cycle, PBR, PCH, PCL and P pushed, the vector with VPB.
    #[test]
    fn a_native_nmi_pushes_four_bytes_and_takes_its_vector() {
        let mut cpu = Cpu { pc: 0x8000, pbr: 0x7E, s: 0x01FF, p: flag::D, ..Cpu::default() };
        let mut bus = vectored(0xFFEA);
        cpu.interrupt(&mut bus, Interrupt::Nmi);
        let kinds: String = bus.cycles.iter().map(|c| c.0).collect();
        assert_eq!(kinds, "riwwwwrr");
        assert_eq!(bus.cycles[0].3 & (pin::VDA | pin::VPA), pin::VDA | pin::VPA);
        assert_eq!(bus.cycles[2..6].iter().map(|c| (c.1, c.2)).collect::<Vec<_>>(), vec![(0x1FF, 0x7E), (0x1FE, 0x80), (0x1FD, 0x00), (0x1FC, flag::D)]);
        assert!(bus.cycles[6].3 & pin::VPB != 0 && bus.cycles[6].1 == 0xFFEA);
        assert_eq!((cpu.pc, cpu.pbr, cpu.s, cpu.p & (flag::I | flag::D)), (0x1234, 0, 0x01FB, flag::I));
    }

    // Note 7 and note 11: emulation mode pushes no bank and clears B in the pushed P.
    #[test]
    fn an_emulation_irq_pushes_three_bytes_with_b_clear() {
        let mut cpu = Cpu { pc: 0x8000, e: true, s: 0x0100, p: 0x30, ..Cpu::default() };
        let mut bus = vectored(0xFFFE);
        cpu.interrupt(&mut bus, Interrupt::Irq);
        let writes: Vec<_> = bus.cycles.iter().filter(|c| c.0 == 'w').map(|c| (c.1, c.2)).collect();
        assert_eq!(writes, vec![(0x100, 0x80), (0x1FF, 0x00), (0x1FE, 0x20)]);
        assert_eq!((cpu.pc, cpu.s), (0x1234, 0x01FD));
    }

    // §2.25: reset conditions the registers, reads the stack instead of writing it, and leaves WAI and STP.
    #[test]
    fn reset_conditions_the_registers_and_writes_nothing() {
        let mut cpu = Cpu { pc: 0x1234, pbr: 5, dbr: 6, d: 0x7777, x: 0xABCD, s: 0x2345, stopped: true, waiting: true, ..Cpu::default() };
        let mut bus = vectored(0xFFFC);
        cpu.interrupt(&mut bus, Interrupt::Reset);
        assert!(bus.cycles.iter().all(|c| c.0 != 'w'));
        assert_eq!((cpu.e, cpu.d, cpu.dbr, cpu.pbr, cpu.x, cpu.pc), (true, 0, 0, 0, 0xCD, 0x1234));
        assert_eq!(cpu.p & (flag::M | flag::X | flag::I | flag::D), flag::M | flag::X | flag::I);
        assert!(!cpu.stopped && !cpu.waiting);
        assert_eq!(Interrupt::Abort.vector(false), 0xFFE8);
    }

    // A waiting or stopped CPU does nothing but a halted cycle a step.
    #[test]
    fn wai_halts_until_an_interrupt() {
        let mut cpu = Cpu { pc: 0x8000, ..Cpu::default() };
        let mut bus = vectored(0xFFEE);
        bus.memory.insert(0x8000, 0xCB);
        cpu.step(&mut bus);
        cpu.step(&mut bus);
        assert_eq!(bus.cycles.iter().map(|c| c.0).collect::<String>(), "riih");
        cpu.interrupt(&mut bus, Interrupt::Irq);
        assert_eq!(cpu.pc, 0x1234);
    }

    // D-41: WDM's second cycle is a program fetch of the byte after the opcode, not an internal cycle.
    #[test]
    fn wdm_fetches_its_second_byte() {
        let mut cpu = Cpu { pc: 0x8000, pbr: 0x80, ..Cpu::default() };
        let mut bus = Log::default();
        bus.memory.insert(0x80_8000, 0x42);
        bus.memory.insert(0x80_8001, 0x5A);
        cpu.step(&mut bus);
        assert_eq!(bus.cycles.iter().map(|c| (c.0, c.1, c.2)).collect::<Vec<_>>(), vec![('r', 0x80_8000, 0x42), ('r', 0x80_8001, 0x5A)]);
        assert_eq!(bus.cycles[1].3 & (pin::VDA | pin::VPA), pin::VPA);
        assert_eq!(cpu.pc, 0x8002);
    }
}
