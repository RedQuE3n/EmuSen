//! The SPC700: fullsnes's instruction tables for what each opcode does and its cycles, anomie's SPC700 cycle
//! document for the order of a cycle's accesses, and SingleStepTests' order where that document leaves it open
//! (VenusRT_Disputes.md D-22). See VenusRT_Native.md §21.

/// What the SPC700 drives: a read, a write, or a cycle with no access.
pub trait Bus {
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
    fn idle(&mut self);
}

pub mod flag {
    pub const C: u8 = 0x01;
    pub const Z: u8 = 0x02;
    pub const I: u8 = 0x04;
    pub const H: u8 = 0x08;
    pub const B: u8 = 0x10;
    pub const P: u8 = 0x20;
    pub const V: u8 = 0x40;
    pub const N: u8 = 0x80;
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Spc700 {
    pub pc: u16,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub psw: u8,
    /// SLEEP or STOP ran: with no interrupt sources in the APU nothing ends them (fullsnes).
    pub stopped: bool,
}

impl Spc700 {
    fn set(&mut self, f: u8, on: bool) {
        if on { self.psw |= f } else { self.psw &= !f }
    }

    fn nz(&mut self, v: u8) -> u8 {
        self.set(flag::N, v & 0x80 != 0);
        self.set(flag::Z, v == 0);
        v
    }

    fn page(&self) -> u16 {
        if self.psw & flag::P != 0 { 0x100 } else { 0 }
    }

    fn fetch<B: Bus>(&mut self, bus: &mut B) -> u8 {
        let v = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        v
    }

    fn fetch16<B: Bus>(&mut self, bus: &mut B) -> u16 {
        let lo = self.fetch(bus) as u16;
        lo | (self.fetch(bus) as u16) << 8
    }

    /// The program byte after the opcode, read and not consumed: a one-byte instruction's second cycle.
    fn dummy<B: Bus>(&mut self, bus: &mut B) {
        bus.read(self.pc);
    }

    fn push<B: Bus>(&mut self, bus: &mut B, v: u8) {
        bus.write(0x100 | self.sp as u16, v);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn pop<B: Bus>(&mut self, bus: &mut B) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        bus.read(0x100 | self.sp as u16)
    }

    fn dp(&self, offset: u8) -> u16 {
        self.page() | offset as u16
    }

    /// The direct page's word at `offset`, its high byte read from the next byte within the page.
    fn dp_word<B: Bus>(&mut self, bus: &mut B, offset: u8) -> u16 {
        let lo = bus.read(self.dp(offset)) as u16;
        lo | (bus.read(self.dp(offset.wrapping_add(1))) as u16) << 8
    }

    fn adc(&mut self, a: u8, b: u8) -> u8 {
        let r = a as u16 + b as u16 + (self.psw & flag::C) as u16;
        self.set(flag::V, !(a ^ b) & (a ^ r as u8) & 0x80 != 0);
        self.set(flag::H, (a ^ b ^ r as u8) & 0x10 != 0);
        self.set(flag::C, r > 0xFF);
        self.nz(r as u8)
    }

    fn cmp(&mut self, a: u8, b: u8) {
        self.set(flag::C, a >= b);
        self.nz(a.wrapping_sub(b));
    }

    /// The six two-operand ALU operations, by the opcode's top three bits: OR, AND, EOR, CMP, ADC, SBC. CMP gives the
    /// first operand back unchanged.
    fn alu(&mut self, op: u8, a: u8, b: u8) -> u8 {
        match op >> 5 {
            0 => self.nz(a | b),
            1 => self.nz(a & b),
            2 => self.nz(a ^ b),
            3 => {
                self.cmp(a, b);
                a
            }
            4 => self.adc(a, b),
            _ => self.adc(a, !b),
        }
    }

    /// ASL, ROL, LSR, ROR, DEC, INC, by the opcode's top three bits.
    fn modify(&mut self, op: u8, v: u8) -> u8 {
        let c = self.psw & flag::C;
        let r = match op >> 5 {
            0 => {
                self.set(flag::C, v & 0x80 != 0);
                v << 1
            }
            1 => {
                self.set(flag::C, v & 0x80 != 0);
                v << 1 | c
            }
            2 => {
                self.set(flag::C, v & 1 != 0);
                v >> 1
            }
            3 => {
                self.set(flag::C, v & 1 != 0);
                v >> 1 | c << 7
            }
            4 => v.wrapping_sub(1),
            _ => v.wrapping_add(1),
        };
        self.nz(r)
    }

    fn ya(&self) -> u16 {
        (self.y as u16) << 8 | self.a as u16
    }

    fn set_ya(&mut self, v: u16) {
        self.a = v as u8;
        self.y = (v >> 8) as u8;
        self.set(flag::N, v & 0x8000 != 0);
        self.set(flag::Z, v == 0);
    }

    /// YA plus a word with a carry in, byte by byte: H from bit 11 to 12 and V from the high byte (fullsnes, ADDW).
    fn add16(&mut self, a: u16, b: u16, carry: u16) -> u16 {
        let lo = (a & 0xFF) + (b & 0xFF) + carry;
        let (ah, bh, c) = (a >> 8, b >> 8, lo >> 8);
        let hi = ah + bh + c;
        self.set(flag::H, (ah & 0xF) + (bh & 0xF) + c > 0xF);
        self.set(flag::V, !(ah ^ bh) & (ah ^ hi) & 0x80 != 0);
        self.set(flag::C, hi > 0xFF);
        (hi & 0xFF) << 8 | (lo & 0xFF)
    }

    /// The 13-bit address and the bit number of an `aaa.b` operand.
    fn membit<B: Bus>(&mut self, bus: &mut B) -> (u16, u8) {
        let w = self.fetch16(bus);
        (w & 0x1FFF, (w >> 13) as u8)
    }

    fn branch<B: Bus>(&mut self, bus: &mut B, rel: u8, taken: bool) {
        if taken {
            bus.idle();
            bus.idle();
            self.pc = self.pc.wrapping_add(rel as i8 as u16);
        }
    }

    fn call<B: Bus>(&mut self, bus: &mut B, to: u16) {
        let pc = self.pc;
        self.push(bus, (pc >> 8) as u8);
        self.push(bus, pc as u8);
        self.pc = to;
    }

    /// One instruction; on a stopped CPU, one more of the halted cycles the suite records.
    pub fn step<B: Bus>(&mut self, bus: &mut B) {
        if self.stopped {
            self.dummy(bus);
            bus.idle();
            return;
        }
        let op = self.fetch(bus);
        match op {
            // Two-operand ALU on A: #imm, (X), dp, dp+X, !abs, !abs+X, !abs+Y, [dp]+Y, [dp+X].
            0x08 | 0x28 | 0x48 | 0x68 | 0x88 | 0xA8 => {
                let b = self.fetch(bus);
                self.a = self.alu(op, self.a, b);
            }
            0x06 | 0x26 | 0x46 | 0x66 | 0x86 | 0xA6 => {
                self.dummy(bus);
                let b = bus.read(self.dp(self.x));
                self.a = self.alu(op, self.a, b);
            }
            0x04 | 0x24 | 0x44 | 0x64 | 0x84 | 0xA4 => {
                let o = self.fetch(bus);
                let b = bus.read(self.dp(o));
                self.a = self.alu(op, self.a, b);
            }
            0x14 | 0x34 | 0x54 | 0x74 | 0x94 | 0xB4 => {
                let o = self.fetch(bus);
                bus.idle();
                let b = bus.read(self.dp(o.wrapping_add(self.x)));
                self.a = self.alu(op, self.a, b);
            }
            0x05 | 0x25 | 0x45 | 0x65 | 0x85 | 0xA5 => {
                let a = self.fetch16(bus);
                let b = bus.read(a);
                self.a = self.alu(op, self.a, b);
            }
            0x15 | 0x35 | 0x55 | 0x75 | 0x95 | 0xB5 | 0x16 | 0x36 | 0x56 | 0x76 | 0x96 | 0xB6 => {
                let a = self.fetch16(bus);
                bus.idle();
                let i = if op & 0x0F == 5 { self.x } else { self.y };
                let b = bus.read(a.wrapping_add(i as u16));
                self.a = self.alu(op, self.a, b);
            }
            0x17 | 0x37 | 0x57 | 0x77 | 0x97 | 0xB7 => {
                let o = self.fetch(bus);
                bus.idle();
                let a = self.dp_word(bus, o).wrapping_add(self.y as u16);
                let b = bus.read(a);
                self.a = self.alu(op, self.a, b);
            }
            0x07 | 0x27 | 0x47 | 0x67 | 0x87 | 0xA7 => {
                let o = self.fetch(bus);
                bus.idle();
                let a = self.dp_word(bus, o.wrapping_add(self.x));
                let b = bus.read(a);
                self.a = self.alu(op, self.a, b);
            }
            // Memory to memory: dd,ds; dp,#imm; (X),(Y). CMP reads and does not write: an IO cycle instead (anomie).
            0x09 | 0x29 | 0x49 | 0x69 | 0x89 | 0xA9 => {
                let s = self.fetch(bus);
                let b = bus.read(self.dp(s));
                let d = self.fetch(bus);
                let a = bus.read(self.dp(d));
                self.rmw_alu(bus, op, self.dp(d), a, b);
            }
            0x18 | 0x38 | 0x58 | 0x78 | 0x98 | 0xB8 => {
                let b = self.fetch(bus);
                let d = self.fetch(bus);
                let a = bus.read(self.dp(d));
                self.rmw_alu(bus, op, self.dp(d), a, b);
            }
            0x19 | 0x39 | 0x59 | 0x79 | 0x99 | 0xB9 => {
                self.dummy(bus);
                let b = bus.read(self.dp(self.y));
                let a = bus.read(self.dp(self.x));
                self.rmw_alu(bus, op, self.dp(self.x), a, b);
            }
            // Compares with X and Y.
            0xC8 | 0xAD => {
                let b = self.fetch(bus);
                let r = if op == 0xC8 { self.x } else { self.y };
                self.cmp(r, b);
            }
            0x3E | 0x7E => {
                let o = self.fetch(bus);
                let b = bus.read(self.dp(o));
                let r = if op == 0x3E { self.x } else { self.y };
                self.cmp(r, b);
            }
            0x1E | 0x5E => {
                let a = self.fetch16(bus);
                let b = bus.read(a);
                let r = if op == 0x1E { self.x } else { self.y };
                self.cmp(r, b);
            }
            // Loads.
            0xE8 => self.a = { let v = self.fetch(bus); self.nz(v) },
            0xCD => self.x = { let v = self.fetch(bus); self.nz(v) },
            0x8D => self.y = { let v = self.fetch(bus); self.nz(v) },
            0xE6 => {
                self.dummy(bus);
                let v = bus.read(self.dp(self.x));
                self.a = self.nz(v);
            }
            0xBF => {
                self.dummy(bus);
                let v = bus.read(self.dp(self.x));
                bus.idle();
                self.x = self.x.wrapping_add(1);
                self.a = self.nz(v);
            }
            0xE4 | 0xF8 | 0xEB => {
                let o = self.fetch(bus);
                let v = bus.read(self.dp(o));
                self.load(op, v);
            }
            0xF4 | 0xF9 | 0xFB => {
                let o = self.fetch(bus);
                bus.idle();
                let i = if op == 0xF9 { self.y } else { self.x };
                let v = bus.read(self.dp(o.wrapping_add(i)));
                self.load(op, v);
            }
            0xE5 | 0xE9 | 0xEC => {
                let a = self.fetch16(bus);
                let v = bus.read(a);
                self.load(op, v);
            }
            0xF5 | 0xF6 => {
                let a = self.fetch16(bus);
                bus.idle();
                let i = if op == 0xF5 { self.x } else { self.y };
                let v = bus.read(a.wrapping_add(i as u16));
                self.a = self.nz(v);
            }
            0xE7 => {
                let o = self.fetch(bus);
                bus.idle();
                let a = self.dp_word(bus, o.wrapping_add(self.x));
                let v = bus.read(a);
                self.a = self.nz(v);
            }
            0xF7 => {
                let o = self.fetch(bus);
                bus.idle();
                let a = self.dp_word(bus, o).wrapping_add(self.y as u16);
                let v = bus.read(a);
                self.a = self.nz(v);
            }
            0xBA => {
                let o = self.fetch(bus);
                let lo = bus.read(self.dp(o)) as u16;
                bus.idle();
                let hi = bus.read(self.dp(o.wrapping_add(1))) as u16;
                self.set_ya(hi << 8 | lo);
            }
            // Stores: a dummy read of the destination first, but for (X)+,A, dp,dp and MOVW's high byte (fullsnes).
            0xC4 | 0xD8 | 0xCB => {
                let o = self.fetch(bus);
                let a = self.dp(o);
                bus.read(a);
                let v = self.store_source(op);
                bus.write(a, v);
            }
            0xD4 | 0xDB | 0xD9 => {
                let o = self.fetch(bus);
                bus.idle();
                let i = if op == 0xD9 { self.y } else { self.x };
                let a = self.dp(o.wrapping_add(i));
                bus.read(a);
                let v = self.store_source(op);
                bus.write(a, v);
            }
            0xC5 | 0xC9 | 0xCC => {
                let a = self.fetch16(bus);
                bus.read(a);
                let v = self.store_source(op);
                bus.write(a, v);
            }
            0xD5 | 0xD6 => {
                let a = self.fetch16(bus);
                bus.idle();
                let i = if op == 0xD5 { self.x } else { self.y };
                let a = a.wrapping_add(i as u16);
                bus.read(a);
                bus.write(a, self.a);
            }
            0xC6 => {
                self.dummy(bus);
                let a = self.dp(self.x);
                bus.read(a);
                bus.write(a, self.a);
            }
            0xAF => {
                self.dummy(bus);
                bus.idle();
                bus.write(self.dp(self.x), self.a);
                self.x = self.x.wrapping_add(1);
            }
            0xC7 => {
                let o = self.fetch(bus);
                bus.idle();
                let a = self.dp_word(bus, o.wrapping_add(self.x));
                bus.read(a);
                bus.write(a, self.a);
            }
            0xD7 => {
                let o = self.fetch(bus);
                let a = self.dp_word(bus, o).wrapping_add(self.y as u16);
                bus.idle();
                bus.read(a);
                bus.write(a, self.a);
            }
            0x8F => {
                let v = self.fetch(bus);
                let d = self.fetch(bus);
                let a = self.dp(d);
                bus.read(a);
                bus.write(a, v);
            }
            0xFA => {
                let s = self.fetch(bus);
                let v = bus.read(self.dp(s));
                let d = self.fetch(bus);
                bus.write(self.dp(d), v);
            }
            0xDA => {
                let o = self.fetch(bus);
                bus.read(self.dp(o));
                bus.write(self.dp(o), self.a);
                bus.write(self.dp(o.wrapping_add(1)), self.y);
            }
            // Register to register.
            0x7D | 0x5D | 0xDD | 0xFD | 0x9D | 0xBD => {
                self.dummy(bus);
                match op {
                    0x7D => self.a = self.nz(self.x),
                    0x5D => self.x = self.nz(self.a),
                    0xDD => self.a = self.nz(self.y),
                    0xFD => self.y = self.nz(self.a),
                    0x9D => self.x = self.nz(self.sp),
                    _ => self.sp = self.x,
                }
            }
            // Stack.
            0x2D | 0x4D | 0x6D | 0x0D => {
                self.dummy(bus);
                let v = match op {
                    0x2D => self.a,
                    0x4D => self.x,
                    0x6D => self.y,
                    _ => self.psw,
                };
                self.push(bus, v);
                bus.idle();
            }
            0xAE | 0xCE | 0xEE | 0x8E => {
                self.dummy(bus);
                bus.idle();
                let v = self.pop(bus);
                match op {
                    0xAE => self.a = v,
                    0xCE => self.x = v,
                    0xEE => self.y = v,
                    _ => self.psw = v,
                }
            }
            // Shifts, rotates, increments and decrements.
            0x1C | 0x3C | 0x5C | 0x7C | 0x9C | 0xBC => {
                self.dummy(bus);
                self.a = self.modify(op, self.a);
            }
            0x1D | 0x3D => {
                self.dummy(bus);
                self.x = self.modify(if op == 0x1D { 0x80 } else { 0xA0 }, self.x);
            }
            0xDC | 0xFC => {
                self.dummy(bus);
                self.y = self.modify(if op == 0xDC { 0x80 } else { 0xA0 }, self.y);
            }
            0x0B | 0x2B | 0x4B | 0x6B | 0x8B | 0xAB => {
                let o = self.fetch(bus);
                let a = self.dp(o);
                let v = bus.read(a);
                let r = self.modify(op, v);
                bus.write(a, r);
            }
            0x1B | 0x3B | 0x5B | 0x7B | 0x9B | 0xBB => {
                let o = self.fetch(bus);
                bus.idle();
                let a = self.dp(o.wrapping_add(self.x));
                let v = bus.read(a);
                let r = self.modify(op, v);
                bus.write(a, r);
            }
            0x0C | 0x2C | 0x4C | 0x6C | 0x8C | 0xAC => {
                let a = self.fetch16(bus);
                let v = bus.read(a);
                let r = self.modify(op, v);
                bus.write(a, r);
            }
            // Sixteen-bit operations.
            0x7A | 0x9A => {
                let o = self.fetch(bus);
                let lo = bus.read(self.dp(o)) as u16;
                bus.idle();
                let w = lo | (bus.read(self.dp(o.wrapping_add(1))) as u16) << 8;
                let r = if op == 0x7A { self.add16(self.ya(), w, 0) } else { self.add16(self.ya(), !w, 1) };
                self.set_ya(r);
            }
            0x5A => {
                let o = self.fetch(bus);
                let w = self.dp_word(bus, o);
                let ya = self.ya();
                self.set(flag::C, ya >= w);
                let r = ya.wrapping_sub(w);
                self.set(flag::N, r & 0x8000 != 0);
                self.set(flag::Z, r == 0);
            }
            0x3A | 0x1A => {
                let o = self.fetch(bus);
                let (la, ha) = (self.dp(o), self.dp(o.wrapping_add(1)));
                let lo = bus.read(la) as u16;
                let step: u16 = if op == 0x3A { 1 } else { 0xFFFF };
                let r_lo = (lo + step) & 0xFF;
                bus.write(la, r_lo as u8);
                let hi = bus.read(ha) as u16;
                let r = (hi << 8 | lo).wrapping_add(step);
                bus.write(ha, (r >> 8) as u8);
                self.set(flag::N, r & 0x8000 != 0);
                self.set(flag::Z, r == 0);
            }
            0xCF => {
                self.dummy(bus);
                for _ in 0..7 {
                    bus.idle();
                }
                let r = self.y as u16 * self.a as u16;
                self.a = r as u8;
                self.y = (r >> 8) as u8;
                self.nz(self.y);
            }
            0x9E => {
                self.dummy(bus);
                for _ in 0..10 {
                    bus.idle();
                }
                self.div();
            }
            // One-bit operations.
            0x02 | 0x22 | 0x42 | 0x62 | 0x82 | 0xA2 | 0xC2 | 0xE2 | 0x12 | 0x32 | 0x52 | 0x72 | 0x92 | 0xB2 | 0xD2 | 0xF2 => {
                let o = self.fetch(bus);
                let a = self.dp(o);
                let v = bus.read(a);
                let bit = 1 << (op >> 5);
                bus.write(a, if op & 0x10 == 0 { v | bit } else { v & !bit });
            }
            0xEA => {
                let (a, b) = self.membit(bus);
                let v = bus.read(a);
                bus.write(a, v ^ (1 << b));
            }
            0xCA => {
                let (a, b) = self.membit(bus);
                let v = bus.read(a);
                bus.idle();
                let c = self.psw & flag::C != 0;
                bus.write(a, if c { v | 1 << b } else { v & !(1 << b) });
            }
            0xAA | 0x0A | 0x2A | 0x4A | 0x6A | 0x8A => {
                let (a, b) = self.membit(bus);
                let m = (bus.read(a) >> b) & 1 != 0;
                let c = self.psw & flag::C != 0;
                let r = match op {
                    0xAA => m,
                    0x0A => c | m,
                    0x2A => c | !m,
                    0x4A => c & m,
                    0x6A => c & !m,
                    _ => c ^ m,
                };
                if matches!(op, 0x0A | 0x2A | 0x8A) {
                    bus.idle();
                }
                self.set(flag::C, r);
            }
            0x60 | 0x80 | 0x20 | 0x40 | 0xE0 => {
                self.dummy(bus);
                match op {
                    0x60 => self.set(flag::C, false),
                    0x80 => self.set(flag::C, true),
                    0x20 => self.set(flag::P, false),
                    0x40 => self.set(flag::P, true),
                    _ => self.psw &= !(flag::V | flag::H),
                }
            }
            0xED | 0xA0 | 0xC0 => {
                self.dummy(bus);
                bus.idle();
                match op {
                    0xED => self.psw ^= flag::C,
                    0xA0 => self.set(flag::I, true),
                    _ => self.set(flag::I, false),
                }
            }
            0xDF | 0xBE => {
                self.dummy(bus);
                bus.idle();
                if op == 0xDF { self.daa() } else { self.das() }
            }
            0x9F => {
                self.dummy(bus);
                bus.idle();
                bus.idle();
                bus.idle();
                self.a = self.nz(self.a.rotate_left(4));
            }
            0x0E | 0x4E => {
                let a = self.fetch16(bus);
                let v = bus.read(a);
                bus.read(a);
                self.nz(self.a.wrapping_sub(v));
                bus.write(a, if op == 0x0E { v | self.a } else { v & !self.a });
            }
            // Branches.
            0x10 | 0x30 | 0x50 | 0x70 | 0x90 | 0xB0 | 0xD0 | 0xF0 | 0x2F => {
                let rel = self.fetch(bus);
                let f = [flag::N, flag::V, flag::C, flag::Z][(op >> 6) as usize & 3];
                let taken = op == 0x2F || ((self.psw & f != 0) == (op & 0x20 != 0));
                self.branch(bus, rel, taken);
            }
            0x03 | 0x23 | 0x43 | 0x63 | 0x83 | 0xA3 | 0xC3 | 0xE3 | 0x13 | 0x33 | 0x53 | 0x73 | 0x93 | 0xB3 | 0xD3 | 0xF3 => {
                let o = self.fetch(bus);
                let v = bus.read(self.dp(o));
                bus.idle();
                let rel = self.fetch(bus);
                let set = v & (1 << (op >> 5)) != 0;
                self.branch(bus, rel, set == (op & 0x10 == 0));
            }
            0x2E => {
                let o = self.fetch(bus);
                let v = bus.read(self.dp(o));
                bus.idle();
                let rel = self.fetch(bus);
                self.branch(bus, rel, self.a != v);
            }
            0xDE => {
                let o = self.fetch(bus);
                bus.idle();
                let v = bus.read(self.dp(o.wrapping_add(self.x)));
                bus.idle();
                let rel = self.fetch(bus);
                self.branch(bus, rel, self.a != v);
            }
            0x6E => {
                let o = self.fetch(bus);
                let a = self.dp(o);
                let v = bus.read(a).wrapping_sub(1);
                bus.write(a, v);
                let rel = self.fetch(bus);
                self.branch(bus, rel, v != 0);
            }
            0xFE => {
                self.dummy(bus);
                bus.idle();
                self.y = self.y.wrapping_sub(1);
                let rel = self.fetch(bus);
                self.branch(bus, rel, self.y != 0);
            }
            // Jumps, calls and returns.
            0x5F => self.pc = self.fetch16(bus),
            0x1F => {
                let a = self.fetch16(bus).wrapping_add(self.x as u16);
                bus.idle();
                let lo = bus.read(a) as u16;
                self.pc = lo | (bus.read(a.wrapping_add(1)) as u16) << 8;
            }
            0x3F => {
                let to = self.fetch16(bus);
                bus.idle();
                self.call(bus, to);
                bus.idle();
                bus.idle();
            }
            0x4F => {
                let u = self.fetch(bus);
                bus.idle();
                self.call(bus, 0xFF00 | u as u16);
                bus.idle();
            }
            0x01 | 0x11 | 0x21 | 0x31 | 0x41 | 0x51 | 0x61 | 0x71 | 0x81 | 0x91 | 0xA1 | 0xB1 | 0xC1 | 0xD1 | 0xE1 | 0xF1 => {
                self.dummy(bus);
                bus.idle();
                let pc = self.pc;
                self.call(bus, pc);
                bus.idle();
                let vector = 0xFFDE - 2 * (op >> 4) as u16;
                let lo = bus.read(vector) as u16;
                self.pc = lo | (bus.read(vector + 1) as u16) << 8;
            }
            0x0F => {
                self.dummy(bus);
                let pc = self.pc;
                self.call(bus, pc);
                let psw = self.psw;
                self.push(bus, psw);
                bus.idle();
                let lo = bus.read(0xFFDE) as u16;
                self.pc = lo | (bus.read(0xFFDF) as u16) << 8;
                self.set(flag::B, true);
                self.set(flag::I, false);
            }
            0x6F => {
                self.dummy(bus);
                bus.idle();
                let lo = self.pop(bus) as u16;
                self.pc = lo | (self.pop(bus) as u16) << 8;
            }
            0x7F => {
                self.dummy(bus);
                bus.idle();
                self.psw = self.pop(bus);
                let lo = self.pop(bus) as u16;
                self.pc = lo | (self.pop(bus) as u16) << 8;
            }
            0x00 => self.dummy(bus),
            0xEF | 0xFF => {
                self.stopped = true;
                self.dummy(bus);
                bus.idle();
            }
        }
    }

    fn rmw_alu<B: Bus>(&mut self, bus: &mut B, op: u8, at: u16, a: u8, b: u8) {
        let r = self.alu(op, a, b);
        if op >> 5 == 3 {
            bus.idle();
        } else {
            bus.write(at, r);
        }
    }

    fn load(&mut self, op: u8, v: u8) {
        let v = self.nz(v);
        match op {
            0xE4 | 0xF4 | 0xE5 => self.a = v,
            0xF8 | 0xF9 | 0xE9 => self.x = v,
            _ => self.y = v,
        }
    }

    fn store_source(&self, op: u8) -> u8 {
        match op {
            0xD8 | 0xC9 | 0xD9 => self.x,
            0xCB | 0xDB | 0xCC => self.y,
            _ => self.a,
        }
    }

    /// DIV YA,X, bit-serially as the SNES_MiSTer referee does it (D-23): a 17-bit register of 0, Y and A rotated left
    /// nine times, taking X shifted left by nine away where it fits; it is A=YA/X, Y=YA MOD X whenever that fits.
    fn div(&mut self) {
        let divisor = (self.x as u32) << 9;
        self.set(flag::H, self.y & 0x0F >= self.x & 0x0F);
        let mut t = (self.ya() as u32) & 0x1FFFF;
        for _ in 0..9 {
            let rotated = ((t << 1) & 0x1FFFF) | (t >> 16);
            let shifted = if rotated >= divisor { rotated ^ 1 } else { rotated };
            t = if shifted & 1 != 0 { shifted.wrapping_sub(divisor) & 0x1FFFF } else { shifted };
        }
        self.a = t as u8;
        self.y = (t >> 9) as u8;
        self.set(flag::V, t & 0x100 != 0);
        self.nz(self.a);
    }

    /// Decimal adjust after an addition, by C and H, then N and Z from the result (fullsnes, DAA).
    fn daa(&mut self) {
        if self.psw & flag::C != 0 || self.a > 0x99 {
            self.a = self.a.wrapping_add(0x60);
            self.set(flag::C, true);
        }
        if self.psw & flag::H != 0 || self.a & 0x0F > 9 {
            self.a = self.a.wrapping_add(6);
        }
        self.nz(self.a);
    }

    fn das(&mut self) {
        if self.psw & flag::C == 0 || self.a > 0x99 {
            self.a = self.a.wrapping_sub(0x60);
            self.set(flag::C, false);
        }
        if self.psw & flag::H == 0 || self.a & 0x0F > 9 {
            self.a = self.a.wrapping_sub(6);
        }
        self.nz(self.a);
    }
}
