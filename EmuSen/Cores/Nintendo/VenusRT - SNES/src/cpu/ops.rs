//! The addressing modes, cycle by cycle as Table 5-7 of the W65C816S datasheet lists them, and the instructions
//! over them. Section numbers in the comments are the table's. See VenusRT_Native.md §10.2.

use super::alu::Modify;
use super::{Bus, Cpu, Ea, Wrap, flag, pin};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    Imm,
    Dir,
    DirX,
    DirY,
    Abs,
    AbsX,
    AbsY,
    Long,
    LongX,
    Ind,
    IndX,
    IndY,
    IndLong,
    IndLongY,
    Sr,
    SrIndY,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Read {
    Lda,
    Ldx,
    Ldy,
    Ora,
    And,
    Eor,
    Adc,
    Sbc,
    Cmp,
    Cpx,
    Cpy,
    Bit,
}

impl Cpu {
    #[inline(always)]
    fn dl_cycle<B: Bus>(&mut self, bus: &mut B) {
        if self.d & 0xFF != 0 {
            self.io_operand(bus);
        }
    }

    /// Emulation mode with DL zero keeps direct-page arithmetic within the page, as the 6502 did.
    #[inline(always)]
    fn page_wrapped(&self) -> bool {
        self.e && self.d & 0xFF == 0
    }

    #[inline(always)]
    fn direct(&self, offset: u16) -> u32 {
        if self.page_wrapped() { ((self.d & 0xFF00) | (offset & 0xFF)) as u32 } else { self.d.wrapping_add(offset) as u32 }
    }

    /// A (d), (d,X) or (d),Y pointer: its second byte stays in the page as the first did, the datasheet's §7.2 (D-1).
    fn pointer16<B: Bus>(&mut self, bus: &mut B, at: u32) -> u16 {
        let lo = self.read8(bus, at, 0) as u16;
        let next = if self.page_wrapped() { (at & 0xFF00) | ((at + 1) & 0xFF) } else { (at + 1) & 0xFFFF };
        lo | (self.read8(bus, next, 0) as u16) << 8
    }

    /// Note 4's cycle: indexing across a page, a write, or a 16-bit index; at the address before the carry.
    #[inline(always)]
    fn index_cycle<B: Bus>(&mut self, bus: &mut B, base: u16, index: u16, write: bool) {
        if write || !self.x8() || (base & 0xFF) + (index & 0xFF) > 0xFF {
            let a = ((self.dbr as u32) << 16) | (base & 0xFF00) as u32 | (base.wrapping_add(index) & 0xFF) as u32;
            self.io(bus, a);
        }
    }

    #[inline(always)]
    fn data_bank(&self, address: u16) -> u32 {
        ((self.dbr as u32) << 16) | address as u32
    }

    pub(crate) fn ea<B: Bus>(&mut self, bus: &mut B, mode: Mode, write: bool) -> Ea {
        match mode {
            Mode::Imm => unreachable!("an immediate has no address"),
            // 10a
            Mode::Dir => {
                let o = self.fetch(bus) as u16;
                self.dl_cycle(bus);
                Ea { address: self.d.wrapping_add(o) as u32, wrap: Wrap::Bank0 }
            }
            // 16a, 17
            Mode::DirX | Mode::DirY => {
                let o = self.fetch(bus) as u16;
                self.dl_cycle(bus);
                self.io_operand(bus);
                let i = if mode == Mode::DirX { self.x } else { self.y };
                Ea { address: self.direct(o.wrapping_add(i)), wrap: Wrap::Bank0 }
            }
            // 1a
            Mode::Abs => {
                let a = self.fetch16(bus);
                Ea { address: self.data_bank(a), wrap: Wrap::Linear }
            }
            // 6a, 7
            Mode::AbsX | Mode::AbsY => {
                let a = self.fetch16(bus);
                let i = if mode == Mode::AbsX { self.x } else { self.y };
                self.index_cycle(bus, a, i, write);
                Ea { address: (self.data_bank(a) + i as u32) & 0xFF_FFFF, wrap: Wrap::Linear }
            }
            // 4a, 5
            Mode::Long | Mode::LongX => {
                let a = self.fetch16(bus) as u32;
                let b = self.fetch(bus) as u32;
                let i = if mode == Mode::LongX { self.x as u32 } else { 0 };
                Ea { address: ((b << 16 | a) + i) & 0xFF_FFFF, wrap: Wrap::Linear }
            }
            // 12, 13
            Mode::Ind | Mode::IndY => {
                let o = self.fetch(bus) as u16;
                self.dl_cycle(bus);
                let at = self.direct(o);
                let a = self.pointer16(bus, at);
                if mode == Mode::Ind {
                    return Ea { address: self.data_bank(a), wrap: Wrap::Linear };
                }
                self.index_cycle(bus, a, self.y, write);
                Ea { address: (self.data_bank(a) + self.y as u32) & 0xFF_FFFF, wrap: Wrap::Linear }
            }
            // 11
            Mode::IndX => {
                let o = self.fetch(bus) as u16;
                self.dl_cycle(bus);
                self.io_operand(bus);
                let at = self.direct(o.wrapping_add(self.x));
                let a = self.pointer16(bus, at);
                Ea { address: self.data_bank(a), wrap: Wrap::Linear }
            }
            // 14, 15
            Mode::IndLong | Mode::IndLongY => {
                let o = self.fetch(bus) as u16;
                self.dl_cycle(bus);
                let at = self.d.wrapping_add(o);
                let lo = self.read8(bus, at as u32, 0) as u32;
                let hi = self.read8(bus, at.wrapping_add(1) as u32, 0) as u32;
                let bank = self.read8(bus, at.wrapping_add(2) as u32, 0) as u32;
                let i = if mode == Mode::IndLongY { self.y as u32 } else { 0 };
                Ea { address: ((bank << 16 | hi << 8 | lo) + i) & 0xFF_FFFF, wrap: Wrap::Linear }
            }
            // 23
            Mode::Sr => {
                let o = self.fetch(bus) as u16;
                self.io_operand(bus);
                Ea { address: self.s.wrapping_add(o) as u32, wrap: Wrap::Bank0 }
            }
            // 24
            Mode::SrIndY => {
                let o = self.fetch(bus) as u16;
                self.io_operand(bus);
                let at = self.s.wrapping_add(o);
                let lo = self.read8(bus, at as u32, 0) as u16;
                let hi_at = at.wrapping_add(1) as u32;
                let a = lo | (self.read8(bus, hi_at, 0) as u16) << 8;
                self.io(bus, hi_at);
                Ea { address: (self.data_bank(a) + self.y as u32) & 0xFF_FFFF, wrap: Wrap::Linear }
            }
        }
    }

    fn read_op<B: Bus>(&mut self, bus: &mut B, mode: Mode, op: Read) {
        let wide = match op {
            Read::Ldx | Read::Ldy | Read::Cpx | Read::Cpy => !self.x8(),
            _ => !self.m8(),
        };
        let v = if mode == Mode::Imm {
            if wide { self.fetch16(bus) } else { self.fetch(bus) as u16 }
        } else {
            let ea = self.ea(bus, mode, false);
            self.read_data(bus, ea, wide)
        };
        match op {
            Read::Lda => {
                self.set_acc(v);
                self.set_nz(v, wide);
            }
            Read::Ldx => {
                self.x = v;
                self.set_nz(v, wide);
            }
            Read::Ldy => {
                self.y = v;
                self.set_nz(v, wide);
            }
            Read::Ora | Read::And | Read::Eor => {
                let a = self.acc();
                let r = match op {
                    Read::Ora => a | v,
                    Read::And => a & v,
                    _ => a ^ v,
                };
                self.set_acc(r);
                self.set_nz(r, wide);
            }
            Read::Adc => self.adc(v),
            Read::Sbc => self.sbc(v),
            Read::Cmp => self.compare(self.a, v, wide),
            Read::Cpx => self.compare(self.x, v, wide),
            Read::Cpy => self.compare(self.y, v, wide),
            Read::Bit => self.bit(v, mode == Mode::Imm),
        }
    }

    fn write_op<B: Bus>(&mut self, bus: &mut B, mode: Mode, value: u16, wide: bool) {
        let ea = self.ea(bus, mode, true);
        self.write_data(bus, ea, value, wide);
    }

    /// 1d, 6b, 10b, 16b: the read, an internal cycle (a write of the old value in emulation mode, note 17), then the
    /// high byte written before the low.
    fn rmw_op<B: Bus>(&mut self, bus: &mut B, mode: Mode, op: Modify) {
        let wide = !self.m8();
        let ea = self.ea(bus, mode, true);
        let lo = self.read8(bus, ea.address, pin::MLB) as u16;
        let v = if wide { lo | (self.read8(bus, ea.next(), pin::MLB) as u16) << 8 } else { lo };
        let last = if wide { ea.next() } else { ea.address };
        if self.e {
            bus.write(last, v as u8, pin::MLB | self.state_pins());
        } else {
            bus.idle(last, pin::MLB | self.state_pins());
        }
        let r = self.modify(op, v, wide);
        if wide {
            self.write8(bus, ea.next(), (r >> 8) as u8, pin::MLB);
        }
        self.write8(bus, ea.address, r as u8, pin::MLB);
    }

    fn rmw_acc<B: Bus>(&mut self, bus: &mut B, op: Modify) {
        self.io_pc(bus);
        let wide = !self.m8();
        let r = self.modify(op, self.acc(), wide);
        self.set_acc(r);
    }

    /// 19a: two cycles, the second internal.
    fn implied<B: Bus>(&mut self, bus: &mut B) {
        self.io_pc(bus);
    }

    fn index_step<B: Bus>(&mut self, bus: &mut B, y: bool, up: bool) {
        self.implied(bus);
        let mask = if self.x8() { 0xFF } else { 0xFFFF };
        let r = if y { &mut self.y } else { &mut self.x };
        *r = if up { r.wrapping_add(1) } else { r.wrapping_sub(1) } & mask;
        let v = *r;
        self.set_nz(v, !self.x8());
    }

    /// TAX, TAY, TXY, TYX, TSX: into an index, at the index width.
    fn to_index(&mut self, v: u16, y: bool) {
        let v = if self.x8() { v & 0xFF } else { v };
        if y { self.y = v } else { self.x = v }
        self.set_nz(v, !self.x8());
    }

    pub(crate) fn execute<B: Bus>(&mut self, bus: &mut B, opcode: u8) {
        use Mode::*;
        match opcode {
            // The arithmetic and logic group: one row per operation, the column is the mode.
            0x01 | 0x03 | 0x05 | 0x07 | 0x09 | 0x0D | 0x0F | 0x11 | 0x12 | 0x13 | 0x15 | 0x17 | 0x19 | 0x1D | 0x1F
            | 0x21 | 0x23 | 0x25 | 0x27 | 0x29 | 0x2D | 0x2F | 0x31 | 0x32 | 0x33 | 0x35 | 0x37 | 0x39 | 0x3D | 0x3F
            | 0x41 | 0x43 | 0x45 | 0x47 | 0x49 | 0x4D | 0x4F | 0x51 | 0x52 | 0x53 | 0x55 | 0x57 | 0x59 | 0x5D | 0x5F
            | 0x61 | 0x63 | 0x65 | 0x67 | 0x69 | 0x6D | 0x6F | 0x71 | 0x72 | 0x73 | 0x75 | 0x77 | 0x79 | 0x7D | 0x7F
            | 0xA1 | 0xA3 | 0xA5 | 0xA7 | 0xA9 | 0xAD | 0xAF | 0xB1 | 0xB2 | 0xB3 | 0xB5 | 0xB7 | 0xB9 | 0xBD | 0xBF
            | 0xC1 | 0xC3 | 0xC5 | 0xC7 | 0xC9 | 0xCD | 0xCF | 0xD1 | 0xD2 | 0xD3 | 0xD5 | 0xD7 | 0xD9 | 0xDD | 0xDF
            | 0xE1 | 0xE3 | 0xE5 | 0xE7 | 0xE9 | 0xED | 0xEF | 0xF1 | 0xF2 | 0xF3 | 0xF5 | 0xF7 | 0xF9 | 0xFD | 0xFF => {
                let op = match opcode >> 5 {
                    0 => Read::Ora,
                    1 => Read::And,
                    2 => Read::Eor,
                    3 => Read::Adc,
                    5 => Read::Lda,
                    6 => Read::Cmp,
                    _ => Read::Sbc,
                };
                self.read_op(bus, alu_mode(opcode), op);
            }
            0x81 | 0x83 | 0x85 | 0x87 | 0x8D | 0x8F | 0x91 | 0x92 | 0x93 | 0x95 | 0x97 | 0x99 | 0x9D | 0x9F => {
                let (v, wide) = (self.acc(), !self.m8());
                self.write_op(bus, alu_mode(opcode), v, wide);
            }
            0xA2 => self.read_op(bus, Imm, Read::Ldx),
            0xA6 => self.read_op(bus, Dir, Read::Ldx),
            0xB6 => self.read_op(bus, DirY, Read::Ldx),
            0xAE => self.read_op(bus, Abs, Read::Ldx),
            0xBE => self.read_op(bus, AbsY, Read::Ldx),
            0xA0 => self.read_op(bus, Imm, Read::Ldy),
            0xA4 => self.read_op(bus, Dir, Read::Ldy),
            0xB4 => self.read_op(bus, DirX, Read::Ldy),
            0xAC => self.read_op(bus, Abs, Read::Ldy),
            0xBC => self.read_op(bus, AbsX, Read::Ldy),
            0xE0 => self.read_op(bus, Imm, Read::Cpx),
            0xE4 => self.read_op(bus, Dir, Read::Cpx),
            0xEC => self.read_op(bus, Abs, Read::Cpx),
            0xC0 => self.read_op(bus, Imm, Read::Cpy),
            0xC4 => self.read_op(bus, Dir, Read::Cpy),
            0xCC => self.read_op(bus, Abs, Read::Cpy),
            0x89 => self.read_op(bus, Imm, Read::Bit),
            0x24 => self.read_op(bus, Dir, Read::Bit),
            0x34 => self.read_op(bus, DirX, Read::Bit),
            0x2C => self.read_op(bus, Abs, Read::Bit),
            0x3C => self.read_op(bus, AbsX, Read::Bit),
            0x86 | 0x96 | 0x8E => {
                let mode = match opcode { 0x86 => Dir, 0x96 => DirY, _ => Abs };
                let (v, wide) = (self.x, !self.x8());
                self.write_op(bus, mode, v, wide);
            }
            0x84 | 0x94 | 0x8C => {
                let mode = match opcode { 0x84 => Dir, 0x94 => DirX, _ => Abs };
                let (v, wide) = (self.y, !self.x8());
                self.write_op(bus, mode, v, wide);
            }
            0x64 | 0x74 | 0x9C | 0x9E => {
                let mode = match opcode { 0x64 => Dir, 0x74 => DirX, 0x9C => Abs, _ => AbsX };
                let wide = !self.m8();
                self.write_op(bus, mode, 0, wide);
            }
            // Read-modify-write: ASL, ROL, LSR, ROR, INC, DEC, TSB, TRB.
            0x0A => self.rmw_acc(bus, Modify::Asl),
            0x2A => self.rmw_acc(bus, Modify::Rol),
            0x4A => self.rmw_acc(bus, Modify::Lsr),
            0x6A => self.rmw_acc(bus, Modify::Ror),
            0x1A => self.rmw_acc(bus, Modify::Inc),
            0x3A => self.rmw_acc(bus, Modify::Dec),
            0x06 | 0x16 | 0x0E | 0x1E | 0x26 | 0x36 | 0x2E | 0x3E | 0x46 | 0x56 | 0x4E | 0x5E | 0x66 | 0x76 | 0x6E | 0x7E
            | 0xC6 | 0xD6 | 0xCE | 0xDE | 0xE6 | 0xF6 | 0xEE | 0xFE => {
                let op = match opcode & 0xE0 {
                    0x00 => Modify::Asl,
                    0x20 => Modify::Rol,
                    0x40 => Modify::Lsr,
                    0x60 => Modify::Ror,
                    0xC0 => Modify::Dec,
                    _ => Modify::Inc,
                };
                let mode = match opcode & 0x1F { 0x06 => Dir, 0x16 => DirX, 0x0E => Abs, _ => AbsX };
                self.rmw_op(bus, mode, op);
            }
            0x04 => self.rmw_op(bus, Dir, Modify::Tsb),
            0x0C => self.rmw_op(bus, Abs, Modify::Tsb),
            0x14 => self.rmw_op(bus, Dir, Modify::Trb),
            0x1C => self.rmw_op(bus, Abs, Modify::Trb),
            // Index increments and decrements.
            0xE8 => self.index_step(bus, false, true),
            0xC8 => self.index_step(bus, true, true),
            0xCA => self.index_step(bus, false, false),
            0x88 => self.index_step(bus, true, false),
            // Transfers.
            0xAA => {
                self.implied(bus);
                self.to_index(self.a, false);
            }
            0xA8 => {
                self.implied(bus);
                self.to_index(self.a, true);
            }
            0x9B => {
                self.implied(bus);
                self.to_index(self.x, true);
            }
            0xBB => {
                self.implied(bus);
                self.to_index(self.y, false);
            }
            0xBA => {
                self.implied(bus);
                self.to_index(self.s, false);
            }
            0x8A | 0x98 => {
                self.implied(bus);
                let v = if opcode == 0x8A { self.x } else { self.y };
                self.set_acc(v);
                self.set_nz(v, !self.m8());
            }
            0x9A => {
                self.implied(bus);
                self.s = if self.e { 0x0100 | (self.x & 0xFF) } else { self.x };
            }
            0x1B => {
                self.implied(bus);
                self.s = if self.e { 0x0100 | (self.a & 0xFF) } else { self.a };
            }
            0x3B => {
                self.implied(bus);
                self.a = self.s;
                self.set_nz(self.a, true);
            }
            0x5B => {
                self.implied(bus);
                self.d = self.a;
                self.set_nz(self.d, true);
            }
            0x7B => {
                self.implied(bus);
                self.a = self.d;
                self.set_nz(self.a, true);
            }
            // 19b
            0xEB => {
                self.implied(bus);
                self.io_pc(bus);
                self.a = self.a.rotate_left(8);
                self.set_nz(self.a & 0xFF, false);
            }
            0xFB => {
                self.implied(bus);
                let c = self.p & flag::C != 0;
                self.p = (self.p & !flag::C) | self.e as u8;
                self.e = c;
                self.settle();
            }
            // Flags.
            0x18 | 0x38 | 0x58 | 0x78 | 0xB8 | 0xD8 | 0xF8 => {
                self.implied(bus);
                match opcode {
                    0x18 => self.p &= !flag::C,
                    0x38 => self.p |= flag::C,
                    0x58 => self.p &= !flag::I,
                    0x78 => self.p |= flag::I,
                    0xB8 => self.p &= !flag::V,
                    0xD8 => self.p &= !flag::D,
                    _ => self.p |= flag::D,
                }
            }
            // 18, note 1: REP and SEP take three cycles, the third internal at the operand.
            0xC2 | 0xE2 => {
                let v = self.fetch(bus);
                self.io_operand(bus);
                if opcode == 0xC2 { self.p &= !v } else { self.p |= v }
                self.settle();
            }
            0xEA => self.implied(bus),
            _ => self.execute_control(bus, opcode),
        }
    }
}

/// The mode of an ORA/AND/EOR/ADC/STA/LDA/CMP/SBC opcode, from its low five bits.
fn alu_mode(opcode: u8) -> Mode {
    match opcode & 0x1F {
        0x01 => Mode::IndX,
        0x03 => Mode::Sr,
        0x05 => Mode::Dir,
        0x07 => Mode::IndLong,
        0x09 => Mode::Imm,
        0x0D => Mode::Abs,
        0x0F => Mode::Long,
        0x11 => Mode::IndY,
        0x12 => Mode::Ind,
        0x13 => Mode::SrIndY,
        0x15 => Mode::DirX,
        0x17 => Mode::IndLongY,
        0x19 => Mode::AbsY,
        0x1D => Mode::AbsX,
        _ => Mode::LongX,
    }
}
