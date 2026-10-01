//! The arithmetic and logic of the data operations, at 8 or 16 bits. Decimal mode follows Bruce Clark's
//! "Decimal Mode" tutorial (Appendix A) and is graded by SingleStepTests; see VenusRT_Native.md §10.3.

use super::{Cpu, flag};

impl Cpu {
    #[inline(always)]
    pub(crate) fn set_nz(&mut self, v: u16, wide: bool) {
        let (zero, negative) = if wide { (v == 0, v & 0x8000 != 0) } else { (v & 0xFF == 0, v & 0x80 != 0) };
        self.p = (self.p & !(flag::N | flag::Z)) | if zero { flag::Z } else { 0 } | if negative { flag::N } else { 0 };
    }

    #[inline(always)]
    fn set_flag(&mut self, f: u8, on: bool) {
        if on { self.p |= f } else { self.p &= !f }
    }

    /// The accumulator's operand width: C at 16 bits, A at 8 with B left alone.
    #[inline(always)]
    pub(crate) fn acc(&self) -> u16 {
        if self.m8() { self.a & 0xFF } else { self.a }
    }

    #[inline(always)]
    pub(crate) fn set_acc(&mut self, v: u16) {
        if self.m8() { self.a = (self.a & 0xFF00) | (v & 0xFF) } else { self.a = v }
    }

    pub(crate) fn adc(&mut self, m: u16) {
        let wide = !self.m8();
        let a = self.acc() as u32;
        let m = m as u32;
        let c = (self.p & flag::C) as u32;
        let (top, mask) = if wide { (0x8000u32, 0xFFFFu32) } else { (0x80, 0xFF) };
        let r = if self.p & flag::D == 0 {
            let r = a + m + c;
            self.set_flag(flag::V, !(a ^ m) & (a ^ r) & top != 0);
            self.set_flag(flag::C, r > mask);
            r
        } else {
            let nibbles = if wide { 4 } else { 2 };
            let (mut r, mut carry) = (0u32, c);
            let mut unadjusted = 0u32;
            for i in 0..nibbles {
                let sum = ((a >> (4 * i)) & 0xF) + ((m >> (4 * i)) & 0xF) + carry;
                if i == nibbles - 1 {
                    unadjusted = r | (sum << (4 * i));
                }
                let (digit, out) = if sum > 9 { ((sum + 6) & 0xF, 1) } else { (sum, 0) };
                r |= digit << (4 * i);
                carry = out;
            }
            self.set_flag(flag::V, !(a ^ m) & (a ^ unadjusted) & top != 0);
            self.set_flag(flag::C, carry != 0);
            r
        };
        let r = (r & mask) as u16;
        self.set_nz(r, wide);
        self.set_acc(r);
    }

    pub(crate) fn sbc(&mut self, m: u16) {
        let wide = !self.m8();
        let a = self.acc() as u32;
        let m = m as u32;
        let c = (self.p & flag::C) as u32;
        let (top, mask) = if wide { (0x8000u32, 0xFFFFu32) } else { (0x80, 0xFF) };
        let binary = a.wrapping_sub(m).wrapping_sub(1 - c);
        self.set_flag(flag::V, (a ^ m) & (a ^ binary) & top != 0);
        let r = if self.p & flag::D == 0 {
            self.set_flag(flag::C, binary <= mask);
            binary
        } else {
            let nibbles = if wide { 4 } else { 2 };
            let (mut r, mut borrow) = (0u32, 1 - c as i32);
            for i in 0..nibbles {
                let mut digit = ((a >> (4 * i)) & 0xF) as i32 - ((m >> (4 * i)) & 0xF) as i32 - borrow;
                borrow = 0;
                if digit < 0 {
                    digit -= 6;
                    borrow = 1;
                }
                r |= ((digit & 0xF) as u32) << (4 * i);
            }
            self.set_flag(flag::C, borrow == 0);
            r
        };
        let r = (r & mask) as u16;
        self.set_nz(r, wide);
        self.set_acc(r);
    }

    /// CMP, CPX and CPY: the register less the operand, flags only.
    pub(crate) fn compare(&mut self, register: u16, m: u16, wide: bool) {
        let mask = if wide { 0xFFFF } else { 0xFF };
        let (r, m) = (register & mask, m & mask);
        self.set_flag(flag::C, r >= m);
        self.set_nz(r.wrapping_sub(m), wide);
    }

    /// BIT: Z from A AND the operand; N and V from the operand's top bits, except for an immediate.
    pub(crate) fn bit(&mut self, m: u16, immediate: bool) {
        let wide = !self.m8();
        self.set_flag(flag::Z, self.acc() & m == 0);
        if !immediate {
            let (n, v) = if wide { (0x8000, 0x4000) } else { (0x80, 0x40) };
            self.set_flag(flag::N, m & n != 0);
            self.set_flag(flag::V, m & v != 0);
        }
    }

    /// A read-modify-write operation on a value of the accumulator's width or the memory's: the new value.
    pub(crate) fn modify(&mut self, op: Modify, v: u16, wide: bool) -> u16 {
        let top = if wide { 0x8000 } else { 0x80 };
        let mask = if wide { 0xFFFF } else { 0xFF };
        let r = match op {
            Modify::Asl => {
                self.set_flag(flag::C, v & top != 0);
                (v << 1) & mask
            }
            Modify::Lsr => {
                self.set_flag(flag::C, v & 1 != 0);
                (v & mask) >> 1
            }
            Modify::Rol => {
                let c = (self.p & flag::C) as u16;
                self.set_flag(flag::C, v & top != 0);
                ((v << 1) | c) & mask
            }
            Modify::Ror => {
                let c = if self.p & flag::C != 0 { top } else { 0 };
                self.set_flag(flag::C, v & 1 != 0);
                ((v & mask) >> 1) | c
            }
            Modify::Inc => v.wrapping_add(1) & mask,
            Modify::Dec => v.wrapping_sub(1) & mask,
            Modify::Tsb | Modify::Trb => {
                let a = self.acc();
                self.set_flag(flag::Z, a & v & mask == 0);
                return if op == Modify::Tsb { (v | a) & mask } else { v & !a & mask };
            }
        };
        self.set_nz(r, wide);
        r
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Modify {
    Asl,
    Lsr,
    Rol,
    Ror,
    Inc,
    Dec,
    Tsb,
    Trb,
}
