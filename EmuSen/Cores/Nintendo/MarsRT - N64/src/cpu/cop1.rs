//! Coprocessor one: the register file, its control registers, and the arithmetic formats. C#'s Fpu.cs, Cpu.Opcodes.Cop1.cs and Cop1Math.cs. See Mars_Fpu.md.

use crate::cpu::cop0::{STATUS, STATUS_COP1_USABLE, STATUS_FPU_FULL_MODE};
use crate::cpu::{Cpu, Exec, Raised, code};
use crate::cpu::interp::{rd, rt};
use crate::cpu::softfloat::{self as sf, DOUBLE, FloatClass, FloatFormat, FloatResult, SINGLE, SoftFloat};

pub const FPU_IMPLEMENTATION: u32 = 0x0000_0A00;
pub const FCSR_WRITABLE_MASK: u32 = 0x0183_FFFF;
pub const FCSR_MASKABLE_CAUSES: u32 = 0x0001_F000;
pub const FCSR_CAUSE_UNIMPLEMENTED: u32 = 1 << 17;
pub const FCSR_ENABLES: u32 = 0x0000_0F80;
pub const FCSR_FLUSH_TO_ZERO: u32 = 1 << 24;
pub const FCSR_CONDITION: u32 = 1 << 23;
const UNDERFLOW_WATCHERS: u32 = (sf::UNDERFLOW | sf::INEXACT) << 7;
const CAUSE_SHIFT: u32 = 12;
const FLAG_SHIFT: u32 = 2;

#[inline(always)]
fn ft(i: u32) -> usize {
    ((i >> 16) & 0x1F) as usize
}
#[inline(always)]
fn fs(i: u32) -> usize {
    ((i >> 11) & 0x1F) as usize
}
#[inline(always)]
fn fd(i: u32) -> usize {
    ((i >> 6) & 0x1F) as usize
}

impl Cpu {
    #[inline(always)]
    fn fpu_full_mode(&self) -> bool {
        self.cop0[STATUS] & STATUS_FPU_FULL_MODE != 0
    }

    /// In half mode a wide access loses the index's low bit.
    #[inline(always)]
    pub(crate) fn read_fpu_wide(&self, index: usize) -> u64 {
        self.fpr[if self.fpu_full_mode() { index } else { index & !1 }]
    }

    #[inline(always)]
    pub(crate) fn write_fpu_wide(&mut self, index: usize, value: u64) {
        let at = if self.fpu_full_mode() { index } else { index & !1 };
        self.fpr[at] = value;
    }

    /// In half mode an odd index names the upper half of its pair.
    #[inline(always)]
    pub(crate) fn read_fpu_word(&self, index: usize) -> u32 {
        if self.fpu_full_mode() {
            return self.fpr[index] as u32;
        }
        let paired = self.fpr[index & !1];
        if index & 1 != 0 { (paired >> 32) as u32 } else { paired as u32 }
    }

    /// A word write leaves the other half of the register standing.
    #[inline(always)]
    pub(crate) fn write_fpu_word(&mut self, index: usize, value: u32) {
        if self.fpu_full_mode() {
            self.fpr[index] = (self.fpr[index] & 0xFFFF_FFFF_0000_0000) | value as u64;
            return;
        }
        let paired = index & !1;
        self.fpr[paired] = if index & 1 != 0 {
            (self.fpr[paired] & 0x0000_0000_FFFF_FFFF) | ((value as u64) << 32)
        } else {
            (self.fpr[paired] & 0xFFFF_FFFF_0000_0000) | value as u64
        };
    }

    fn read_fpu_control(&self, index: usize) -> u32 {
        match index {
            0 => FPU_IMPLEMENTATION,
            31 => self.fcsr,
            _ => 0,
        }
    }

    /// The write lands before the exception is considered.
    fn write_fpu_control(&mut self, index: usize, value: u32) -> Exec {
        if index != 31 {
            return Ok(());
        }
        self.fcsr = value & FCSR_WRITABLE_MASK;
        if self.enabled_cause() != 0 {
            return Err(self.raise(code::FLOATING_POINT, self.current_pc));
        }
        Ok(())
    }

    /// The unimplemented cause has no enable of its own: set by any route, it fires.
    #[inline(always)]
    fn enabled_cause(&self) -> u32 {
        ((self.fcsr >> 5) & self.fcsr & FCSR_ENABLES) | (self.fcsr & FCSR_CAUSE_UNIMPLEMENTED)
    }

    /// An operation the part does not implement clears the maskable causes and fires regardless of them.
    fn raise_unimplemented(&mut self) -> Raised {
        self.fcsr = (self.fcsr & !FCSR_MASKABLE_CAUSES) | FCSR_CAUSE_UNIMPLEMENTED;
        self.raise(code::FLOATING_POINT, self.current_pc)
    }

    /// Usability is answered before the decode.
    #[inline(always)]
    pub(crate) fn require_cop1(&mut self) -> Exec {
        if self.cop0[STATUS] & STATUS_COP1_USABLE != 0 {
            return Ok(());
        }
        Err(self.raise_with(code::COPROCESSOR_UNUSABLE, self.current_pc, false, 1))
    }

    pub(crate) fn execute_cop1(&mut self, i: u32) -> Exec {
        self.require_cop1()?;
        match (i >> 21) & 0x1F {
            0x00 => {
                let value = self.read_fpu_word(rd(i));
                self.write32(rt(i), value);
            }
            0x01 => {
                let value = self.read_fpu_wide(rd(i));
                self.write(rt(i), value);
            }
            0x02 => {
                let value = self.read_fpu_control(rd(i));
                self.write32(rt(i), value);
            }
            0x04 => self.write_fpu_word(rd(i), self.read(rt(i)) as u32),
            0x05 => self.write_fpu_wide(rd(i), self.read(rt(i))),
            0x06 => return self.write_fpu_control(rd(i), self.read(rt(i)) as u32),
            0x08 => return self.branch_on_condition(i),
            0x10 => return self.execute_format(i, false),
            0x11 => return self.execute_format(i, true),
            0x14 => return self.execute_from_integer(i, false),
            0x15 => return self.execute_from_integer(i, true),
            _ => return Err(self.raise_unimplemented()),
        }
        Ok(())
    }

    fn execute_format(&mut self, i: u32, wide: bool) -> Exec {
        let format: &FloatFormat = if wide { &DOUBLE } else { &SINGLE };
        let left = self.read_fpu_source(fs(i), wide);
        let right = self.read_fpu_operand(ft(i), wide);
        let mode = self.fcsr & 3;
        let flush = self.fcsr & FCSR_FLUSH_TO_ZERO != 0;
        let funct = i & 0x3F;

        if !wide
            && funct <= 0x02
            && mode == sf::ROUND_NEAREST
            && let Some((bits, flags)) = sf::host_single(funct, left as u32, right as u32)
        {
            return self.deliver(FloatResult::raised(bits, flags), fd(i), false);
        }

        match funct {
            0x05 => return self.deliver(sf::sign(left, format, false), fd(i), wide),
            0x07 => return self.deliver(sf::sign(left, format, true), fd(i), wide),
            0x06 => {
                let value = self.read_fpu_source(fs(i), true);
                self.write_fpu_result(fd(i), value, true);
                return Ok(());
            }
            _ => {}
        }

        let other: &FloatFormat = if wide { &SINGLE } else { &DOUBLE };
        let result = match funct {
            0x00 => sf::add(left, right, format, mode, flush),
            0x01 => sf::subtract(left, right, format, mode, flush),
            0x02 => sf::multiply(left, right, format, mode, flush),
            0x03 => sf::divide(left, right, format, mode, flush),
            0x04 => sf::square_root(left, format, mode, flush),
            0x08 => sf::to_integer(left, format, true, sf::ROUND_NEAREST),
            0x09 => sf::to_integer(left, format, true, sf::ROUND_ZERO),
            0x0A => sf::to_integer(left, format, true, sf::ROUND_POSITIVE),
            0x0B => sf::to_integer(left, format, true, sf::ROUND_NEGATIVE),
            0x0C => sf::to_integer(left, format, false, sf::ROUND_NEAREST),
            0x0D => sf::to_integer(left, format, false, sf::ROUND_ZERO),
            0x0E => sf::to_integer(left, format, false, sf::ROUND_POSITIVE),
            0x0F => sf::to_integer(left, format, false, sf::ROUND_NEGATIVE),
            0x20 | 0x21 => {
                if wide == (funct == 0x21) {
                    FloatResult::refused()
                } else {
                    sf::between(left, format, other, mode, flush)
                }
            }
            0x24 => sf::to_integer(left, format, false, mode),
            0x25 => sf::to_integer(left, format, true, mode),
            0x30.. => return self.compare(i, format, left, right),
            _ => return Err(self.raise_unimplemented()),
        };

        let integer = matches!(funct, 0x08..=0x0F | 0x24 | 0x25);
        let destination_wide = if integer { matches!(funct, 0x08..=0x0B | 0x25) } else { funct == 0x21 || (wide && funct < 0x20) };
        self.deliver(result, fd(i), destination_wide)
    }

    /// `Deliver`: a raised cause replaces the previous one; only an unraised operation adds to the flags.
    fn deliver(&mut self, result: FloatResult, destination: usize, wide: bool) -> Exec {
        if result.unimplemented {
            return Err(self.raise_unimplemented());
        }
        if result.flags & sf::UNDERFLOW != 0 && self.fcsr & UNDERFLOW_WATCHERS != 0 {
            return Err(self.raise_unimplemented());
        }
        self.fcsr = (self.fcsr & !(FCSR_MASKABLE_CAUSES | FCSR_CAUSE_UNIMPLEMENTED)) | (result.flags << CAUSE_SHIFT);
        if self.enabled_cause() != 0 {
            return Err(self.raise(code::FLOATING_POINT, self.current_pc));
        }
        self.fcsr |= result.flags << FLAG_SHIFT;
        self.write_fpu_result(destination, result.bits, wide);
        Ok(())
    }

    /// A computed 32-bit result clears the rest of the register, which a word move does not.
    #[inline(always)]
    fn write_fpu_result(&mut self, destination: usize, bits: u64, wide: bool) {
        self.fpr[destination] = if wide { bits } else { bits as u32 as u64 };
    }

    fn execute_from_integer(&mut self, i: u32, wide: bool) -> Exec {
        let funct = i & 0x3F;
        if funct != 0x20 && funct != 0x21 {
            return Err(self.raise_unimplemented());
        }
        let to_double = funct == 0x21;
        let target = if to_double { &DOUBLE } else { &SINGLE };
        let result = sf::from_integer(self.read_fpu_source(fs(i), wide), wide, target, self.fcsr & 3, self.fcsr & FCSR_FLUSH_TO_ZERO != 0);
        self.deliver(result, fd(i), to_double)
    }

    /// `Compare`: the condition bit, and which NaN raises, are the halves this part inverts.
    fn compare(&mut self, i: u32, format: &FloatFormat, left: u64, right: u64) -> Exec {
        let condition = i & 0xF;
        let a = SoftFloat::unpack(left, format);
        let b = SoftFloat::unpack(right, format);
        let unordered = a.is_nan() || b.is_nan();
        let invalid = if condition & 8 != 0 { unordered } else { a.class == FloatClass::Nan || b.class == FloatClass::Nan };
        let (mut less, mut equal) = (false, false);
        if !unordered {
            let order = SoftFloat::compare(&a, &b);
            less = order < 0;
            equal = order == 0;
        }
        let met = (condition & 4 != 0 && less) || (condition & 2 != 0 && equal) || (condition & 1 != 0 && unordered);
        let raised = if invalid { sf::INVALID } else { 0 };
        self.fcsr = (self.fcsr & !(FCSR_MASKABLE_CAUSES | FCSR_CAUSE_UNIMPLEMENTED)) | (raised << CAUSE_SHIFT);
        if self.enabled_cause() != 0 {
            return Err(self.raise(code::FLOATING_POINT, self.current_pc));
        }
        self.fcsr |= raised << FLAG_SHIFT;
        self.fcsr = if met { self.fcsr | FCSR_CONDITION } else { self.fcsr & !FCSR_CONDITION };
        Ok(())
    }

    /// Four encodings only: the rest of the sub-opcode's space is reserved.
    fn branch_on_condition(&mut self, i: u32) -> Exec {
        let selector = rt(i);
        if selector > 3 {
            return Err(self.raise_unimplemented());
        }
        let wanted = selector & 1 != 0;
        let met = (self.fcsr & FCSR_CONDITION != 0) == wanted;
        self.branch_if(met, i, selector & 2 != 0, false);
        Ok(())
    }

    /// Half mode masks the low bit of the first source index and of nothing else.
    #[inline(always)]
    fn read_fpu_source(&self, index: usize, wide: bool) -> u64 {
        self.read_fpu_operand(if self.fpu_full_mode() { index } else { index & !1 }, wide)
    }

    #[inline(always)]
    fn read_fpu_operand(&self, index: usize, wide: bool) -> u64 {
        if wide { self.fpr[index] } else { self.fpr[index] as u32 as u64 }
    }
}
