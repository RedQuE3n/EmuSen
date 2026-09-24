//! C#'s `Cpu/Opcodes/Cpu.Alu.cs`: the arithmetic, logic and shift primitives. See Mercury_Cpu.md §6.

use super::{Cpu, FLAG_C, FLAG_H, FLAG_N, FLAG_Z};

impl Cpu {
    pub(super) fn add8(&mut self, value: u8, with_carry: bool) {
        let carry = (with_carry && self.flag(FLAG_C)) as i32;
        let result = self.a as i32 + value as i32 + carry;
        self.set_flag(FLAG_H, (self.a & 0x0F) as i32 + (value & 0x0F) as i32 + carry > 0x0F);
        self.set_flag(FLAG_C, result > 0xFF);
        self.a = result as u8;
        self.set_flag(FLAG_Z, self.a == 0);
        self.set_flag(FLAG_N, false);
    }

    pub(super) fn sub8(&mut self, value: u8, with_carry: bool) {
        let carry = (with_carry && self.flag(FLAG_C)) as i32;
        let result = self.a as i32 - value as i32 - carry;
        self.set_flag(FLAG_H, (self.a & 0x0F) as i32 - (value & 0x0F) as i32 - carry < 0);
        self.set_flag(FLAG_C, result < 0);
        self.a = result as u8;
        self.set_flag(FLAG_Z, self.a == 0);
        self.set_flag(FLAG_N, true);
    }

    pub(super) fn and8(&mut self, value: u8) {
        self.a &= value;
        self.f = 0;
        self.set_flag(FLAG_H, true);
        self.set_flag(FLAG_Z, self.a == 0);
    }

    pub(super) fn or8(&mut self, value: u8) {
        self.a |= value;
        self.f = 0;
        self.set_flag(FLAG_Z, self.a == 0);
    }

    pub(super) fn xor8(&mut self, value: u8) {
        self.a ^= value;
        self.f = 0;
        self.set_flag(FLAG_Z, self.a == 0);
    }

    pub(super) fn cp8(&mut self, value: u8) {
        let saved = self.a;
        self.sub8(value, false);
        self.a = saved;
    }

    pub(super) fn inc8(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        self.set_flag(FLAG_H, value & 0x0F == 0x0F);
        self.set_flag(FLAG_Z, result == 0);
        self.set_flag(FLAG_N, false);
        result
    }

    pub(super) fn dec8(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        self.set_flag(FLAG_H, value & 0x0F == 0);
        self.set_flag(FLAG_Z, result == 0);
        self.set_flag(FLAG_N, true);
        result
    }

    pub(super) fn add_hl(&mut self, value: u16) {
        let hl = self.hl();
        let result = hl as i32 + value as i32;
        self.set_flag(FLAG_H, (hl & 0x0FFF) as i32 + (value & 0x0FFF) as i32 > 0x0FFF);
        self.set_flag(FLAG_C, result > 0xFFFF);
        self.set_flag(FLAG_N, false);
        self.set_hl(result as u16);
    }

    /// Both carries come from the low byte, even though the operand is signed.
    pub(super) fn add_sp_offset<B: super::CpuBus>(&mut self, bus: &mut B) -> u16 {
        let offset = self.fetch(bus) as i8 as i32;
        let sp = self.sp as i32;
        self.f = 0;
        self.set_flag(FLAG_H, (sp & 0x0F) + (offset & 0x0F) > 0x0F);
        self.set_flag(FLAG_C, (sp & 0xFF) + (offset & 0xFF) > 0xFF);
        (sp + offset) as u16
    }

    pub(super) fn daa(&mut self) {
        let mut a = self.a as i32;
        if !self.flag(FLAG_N) {
            if self.flag(FLAG_C) || a > 0x99 {
                a += 0x60;
                self.set_flag(FLAG_C, true);
            }
            if self.flag(FLAG_H) || (a & 0x0F) > 0x09 {
                a += 0x06;
            }
        } else {
            if self.flag(FLAG_C) {
                a -= 0x60;
            }
            if self.flag(FLAG_H) {
                a -= 0x06;
            }
        }
        self.a = a as u8;
        self.set_flag(FLAG_Z, self.a == 0);
        self.set_flag(FLAG_H, false);
    }

    fn shifted(&mut self, result: u8, carry: bool, set_zero: bool) -> u8 {
        self.f = 0;
        self.set_flag(FLAG_C, carry);
        self.set_flag(FLAG_Z, set_zero && result == 0);
        result
    }

    pub(super) fn rlc(&mut self, v: u8, set_zero: bool) -> u8 {
        self.shifted(v.rotate_left(1), v & 0x80 != 0, set_zero)
    }

    pub(super) fn rrc(&mut self, v: u8, set_zero: bool) -> u8 {
        self.shifted(v.rotate_right(1), v & 0x01 != 0, set_zero)
    }

    pub(super) fn rl(&mut self, v: u8, set_zero: bool) -> u8 {
        let c = self.flag(FLAG_C) as u8;
        self.shifted((v << 1) | c, v & 0x80 != 0, set_zero)
    }

    pub(super) fn rr(&mut self, v: u8, set_zero: bool) -> u8 {
        let c = if self.flag(FLAG_C) { 0x80 } else { 0 };
        self.shifted((v >> 1) | c, v & 0x01 != 0, set_zero)
    }

    pub(super) fn sla(&mut self, v: u8) -> u8 {
        self.shifted(v << 1, v & 0x80 != 0, true)
    }

    pub(super) fn sra(&mut self, v: u8) -> u8 {
        self.shifted((v >> 1) | (v & 0x80), v & 0x01 != 0, true)
    }

    pub(super) fn srl(&mut self, v: u8) -> u8 {
        self.shifted(v >> 1, v & 0x01 != 0, true)
    }

    pub(super) fn swap(&mut self, v: u8) -> u8 {
        self.shifted(v.rotate_left(4), false, true)
    }

    pub(super) fn bit(&mut self, v: u8, index: u32) {
        self.set_flag(FLAG_Z, v & (1 << index) == 0);
        self.set_flag(FLAG_N, false);
        self.set_flag(FLAG_H, true);
    }
}
