//! C#'s `Cpu.Opcodes.cs` and `Cpu.Opcodes.Cb.cs`: the unprefixed and `$CB` tables. See Mercury_Cpu.md §6.

use super::{Cpu, CpuBus, FLAG_C, FLAG_H, FLAG_N, FLAG_Z, IllegalOpcode};

impl Cpu {
    /// Operand order for the register-shaped blocks; index 6 is the byte at HL.
    #[inline(always)]
    fn get_operand<B: CpuBus>(&mut self, bus: &mut B, index: u8) -> u8 {
        match index {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => {
                let hl = self.hl();
                self.read_cycle(bus, hl)
            }
            _ => self.a,
        }
    }

    #[inline(always)]
    fn set_operand<B: CpuBus>(&mut self, bus: &mut B, index: u8, value: u8) {
        match index {
            0 => self.b = value,
            1 => self.c = value,
            2 => self.d = value,
            3 => self.e = value,
            4 => self.h = value,
            5 => self.l = value,
            6 => {
                let hl = self.hl();
                self.write_cycle(bus, hl, value)
            }
            _ => self.a = value,
        }
    }

    pub(super) fn execute<B: CpuBus>(&mut self, bus: &mut B, op: u8) -> Result<i32, IllegalOpcode> {
        if (0x40..=0x7F).contains(&op) && op != 0x76 {
            let (dst, src) = ((op >> 3) & 7, op & 7);
            let v = self.get_operand(bus, src);
            self.set_operand(bus, dst, v);
            return Ok(if dst == 6 || src == 6 { 8 } else { 4 });
        }
        if (0x80..=0xBF).contains(&op) {
            let src = op & 7;
            let v = self.get_operand(bus, src);
            match (op >> 3) & 7 {
                0 => self.add8(v, false),
                1 => self.add8(v, true),
                2 => self.sub8(v, false),
                3 => self.sub8(v, true),
                4 => self.and8(v),
                5 => self.xor8(v),
                6 => self.or8(v),
                _ => self.cp8(v),
            }
            return Ok(if src == 6 { 8 } else { 4 });
        }

        Ok(match op {
            0x00 => 4,
            0x10 => {
                self.fetch(bus);
                bus.stop();
                4
            }
            0x76 => {
                if !self.ime && self.interrupt_pending {
                    self.halt_bug = true;
                } else {
                    self.halted = true;
                }
                4
            }

            0x01 => {
                let v = self.fetch16(bus);
                self.set_bc(v);
                12
            }
            0x11 => {
                let v = self.fetch16(bus);
                self.set_de(v);
                12
            }
            0x21 => {
                let v = self.fetch16(bus);
                self.set_hl(v);
                12
            }
            0x31 => {
                self.sp = self.fetch16(bus);
                12
            }

            0x02 => {
                self.write_cycle(bus, self.bc(), self.a);
                8
            }
            0x12 => {
                self.write_cycle(bus, self.de(), self.a);
                8
            }
            0x22 => {
                let hl = self.hl();
                self.write_cycle(bus, hl, self.a);
                self.set_hl(hl.wrapping_add(1));
                8
            }
            0x32 => {
                let hl = self.hl();
                self.write_cycle(bus, hl, self.a);
                self.set_hl(hl.wrapping_sub(1));
                8
            }

            0x0A => {
                self.a = self.read_cycle(bus, self.bc());
                8
            }
            0x1A => {
                self.a = self.read_cycle(bus, self.de());
                8
            }
            0x2A => {
                let hl = self.hl();
                self.a = self.read_cycle(bus, hl);
                self.set_hl(hl.wrapping_add(1));
                8
            }
            0x3A => {
                let hl = self.hl();
                self.a = self.read_cycle(bus, hl);
                self.set_hl(hl.wrapping_sub(1));
                8
            }

            0x03 => {
                self.set_bc(self.bc().wrapping_add(1));
                8
            }
            0x13 => {
                self.set_de(self.de().wrapping_add(1));
                8
            }
            0x23 => {
                self.set_hl(self.hl().wrapping_add(1));
                8
            }
            0x33 => {
                self.sp = self.sp.wrapping_add(1);
                8
            }
            0x0B => {
                self.set_bc(self.bc().wrapping_sub(1));
                8
            }
            0x1B => {
                self.set_de(self.de().wrapping_sub(1));
                8
            }
            0x2B => {
                self.set_hl(self.hl().wrapping_sub(1));
                8
            }
            0x3B => {
                self.sp = self.sp.wrapping_sub(1);
                8
            }

            0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x3C => {
                let r = (op >> 3) & 7;
                let v = self.get_operand(bus, r);
                let v = self.inc8(v);
                self.set_operand(bus, r, v);
                4
            }
            0x34 => {
                let hl = self.hl();
                let v = self.read_cycle(bus, hl);
                let v = self.inc8(v);
                self.write_cycle(bus, hl, v);
                12
            }
            0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x3D => {
                let r = (op >> 3) & 7;
                let v = self.get_operand(bus, r);
                let v = self.dec8(v);
                self.set_operand(bus, r, v);
                4
            }
            0x35 => {
                let hl = self.hl();
                let v = self.read_cycle(bus, hl);
                let v = self.dec8(v);
                self.write_cycle(bus, hl, v);
                12
            }

            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x3E => {
                let v = self.fetch(bus);
                self.set_operand(bus, (op >> 3) & 7, v);
                8
            }
            0x36 => {
                let v = self.fetch(bus);
                let hl = self.hl();
                self.write_cycle(bus, hl, v);
                12
            }

            0x09 => {
                self.add_hl(self.bc());
                8
            }
            0x19 => {
                self.add_hl(self.de());
                8
            }
            0x29 => {
                self.add_hl(self.hl());
                8
            }
            0x39 => {
                self.add_hl(self.sp);
                8
            }

            0x07 => {
                self.a = self.rlc(self.a, false);
                4
            }
            0x0F => {
                self.a = self.rrc(self.a, false);
                4
            }
            0x17 => {
                self.a = self.rl(self.a, false);
                4
            }
            0x1F => {
                self.a = self.rr(self.a, false);
                4
            }

            0x27 => {
                self.daa();
                4
            }
            0x2F => {
                self.a = !self.a;
                self.set_flag(FLAG_N, true);
                self.set_flag(FLAG_H, true);
                4
            }
            0x37 => {
                self.set_flag(FLAG_C, true);
                self.set_flag(FLAG_N, false);
                self.set_flag(FLAG_H, false);
                4
            }
            0x3F => {
                let c = self.flag(FLAG_C);
                self.set_flag(FLAG_C, !c);
                self.set_flag(FLAG_N, false);
                self.set_flag(FLAG_H, false);
                4
            }

            0x08 => {
                let target = self.fetch16(bus);
                self.write_cycle(bus, target, self.sp as u8);
                self.write_cycle(bus, target.wrapping_add(1), (self.sp >> 8) as u8);
                20
            }

            0x18 => self.jump_relative(bus, true),
            0x20 => self.jump_relative(bus, !self.flag(FLAG_Z)),
            0x28 => self.jump_relative(bus, self.flag(FLAG_Z)),
            0x30 => self.jump_relative(bus, !self.flag(FLAG_C)),
            0x38 => self.jump_relative(bus, self.flag(FLAG_C)),

            0xC3 => self.jump(bus, true),
            0xC2 => self.jump(bus, !self.flag(FLAG_Z)),
            0xCA => self.jump(bus, self.flag(FLAG_Z)),
            0xD2 => self.jump(bus, !self.flag(FLAG_C)),
            0xDA => self.jump(bus, self.flag(FLAG_C)),
            0xE9 => {
                self.pc = self.hl();
                4
            }

            0xCD => self.call(bus, true),
            0xC4 => self.call(bus, !self.flag(FLAG_Z)),
            0xCC => self.call(bus, self.flag(FLAG_Z)),
            0xD4 => self.call(bus, !self.flag(FLAG_C)),
            0xDC => self.call(bus, self.flag(FLAG_C)),

            0xC9 => {
                self.pc = self.pop(bus);
                bus.note_return();
                16
            }
            0xC0 => self.return_if(bus, !self.flag(FLAG_Z)),
            0xC8 => self.return_if(bus, self.flag(FLAG_Z)),
            0xD0 => self.return_if(bus, !self.flag(FLAG_C)),
            0xD8 => self.return_if(bus, self.flag(FLAG_C)),
            0xD9 => {
                self.pc = self.pop(bus);
                self.ime = true;
                bus.note_return();
                16
            }

            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
                self.push(bus, self.pc);
                self.pc = (op & 0x38) as u16;
                bus.note_call(self.last_instruction_pc, self.pc, false);
                16
            }

            0xC1 => {
                let v = self.pop(bus);
                self.set_bc(v);
                12
            }
            0xD1 => {
                let v = self.pop(bus);
                self.set_de(v);
                12
            }
            0xE1 => {
                let v = self.pop(bus);
                self.set_hl(v);
                12
            }
            0xF1 => {
                let v = self.pop(bus);
                self.set_af(v);
                12
            }

            0xC5 => {
                self.push(bus, self.bc());
                16
            }
            0xD5 => {
                self.push(bus, self.de());
                16
            }
            0xE5 => {
                self.push(bus, self.hl());
                16
            }
            0xF5 => {
                self.push(bus, self.af());
                16
            }

            0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => {
                let v = self.fetch(bus);
                match (op >> 3) & 7 {
                    0 => self.add8(v, false),
                    1 => self.add8(v, true),
                    2 => self.sub8(v, false),
                    3 => self.sub8(v, true),
                    4 => self.and8(v),
                    5 => self.xor8(v),
                    6 => self.or8(v),
                    _ => self.cp8(v),
                }
                8
            }

            0xE0 => {
                let address = 0xFF00 + self.fetch(bus) as u16;
                self.write_cycle(bus, address, self.a);
                12
            }
            0xF0 => {
                let address = 0xFF00 + self.fetch(bus) as u16;
                self.a = self.read_cycle(bus, address);
                12
            }
            0xE2 => {
                self.write_cycle(bus, 0xFF00 + self.c as u16, self.a);
                8
            }
            0xF2 => {
                self.a = self.read_cycle(bus, 0xFF00 + self.c as u16);
                8
            }

            0xEA => {
                let address = self.fetch16(bus);
                self.write_cycle(bus, address, self.a);
                16
            }
            0xFA => {
                let address = self.fetch16(bus);
                self.a = self.read_cycle(bus, address);
                16
            }

            0xE8 => {
                self.sp = self.add_sp_offset(bus);
                16
            }
            0xF8 => {
                let v = self.add_sp_offset(bus);
                self.set_hl(v);
                12
            }
            0xF9 => {
                self.sp = self.hl();
                8
            }

            0xF3 => {
                self.ime = false;
                self.ime_scheduled = false;
                4
            }
            0xFB => {
                self.ime_scheduled = true;
                4
            }

            0xCB => {
                let cb = self.fetch(bus);
                self.execute_cb(bus, cb)
            }

            _ => return Err(IllegalOpcode { opcode: op, pc: self.last_instruction_pc }),
        })
    }

    fn execute_cb<B: CpuBus>(&mut self, bus: &mut B, op: u8) -> i32 {
        let operand = op & 7;
        let index = ((op >> 3) & 7) as u32;
        let memory = operand == 6;

        if (0x40..=0x7F).contains(&op) {
            let v = self.get_operand(bus, operand);
            self.bit(v, index);
            return if memory { 12 } else { 8 };
        }

        let v = self.get_operand(bus, operand);
        let result = match op {
            0x00..=0x07 => self.rlc(v, true),
            0x08..=0x0F => self.rrc(v, true),
            0x10..=0x17 => self.rl(v, true),
            0x18..=0x1F => self.rr(v, true),
            0x20..=0x27 => self.sla(v),
            0x28..=0x2F => self.sra(v),
            0x30..=0x37 => self.swap(v),
            0x38..=0x3F => self.srl(v),
            0x80..=0xBF => v & !(1 << index),
            _ => v | (1 << index),
        };
        self.set_operand(bus, operand, result);
        if memory { 16 } else { 8 }
    }

    fn jump_relative<B: CpuBus>(&mut self, bus: &mut B, taken: bool) -> i32 {
        let offset = self.fetch(bus) as i8;
        if !taken {
            return 8;
        }
        self.pc = self.pc.wrapping_add(offset as u16);
        12
    }

    fn jump<B: CpuBus>(&mut self, bus: &mut B, taken: bool) -> i32 {
        let target = self.fetch16(bus);
        if !taken {
            return 12;
        }
        self.pc = target;
        16
    }

    fn call<B: CpuBus>(&mut self, bus: &mut B, taken: bool) -> i32 {
        let target = self.fetch16(bus);
        if !taken {
            return 12;
        }
        self.push(bus, self.pc);
        self.pc = target;
        bus.note_call(self.last_instruction_pc, target, false);
        24
    }

    fn return_if<B: CpuBus>(&mut self, bus: &mut B, taken: bool) -> i32 {
        if !taken {
            return 8;
        }
        self.pc = self.pop(bus);
        bus.note_return();
        20
    }
}
