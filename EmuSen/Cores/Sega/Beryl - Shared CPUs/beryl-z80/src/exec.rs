//! The instructions: Zilog's Z80 CPU User Manual for the operations and their machine cycles, "The Undocumented Z80
//! Documented" for the undocumented opcodes and flags, and the published MEMPTR and Q notes for WZ and Q.
//! Beryl_Z80.md §4 records each reading.

use emusen_native::debug::kind;

use crate::{Bus, Observer, Step, Z80};

pub(crate) const C: u8 = 0x01;
pub(crate) const N: u8 = 0x02;
pub(crate) const PV: u8 = 0x04;
pub(crate) const X: u8 = 0x08;
pub(crate) const H: u8 = 0x10;
pub(crate) const Y: u8 = 0x20;
pub(crate) const Z: u8 = 0x40;
pub(crate) const S: u8 = 0x80;

/// HL, or the index register a DD or FD prefix puts in its place.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Idx {
    Hl,
    Ix,
    Iy,
}

fn szxy(v: u8) -> u8 {
    (v & (S | Y | X)) | if v == 0 { Z } else { 0 }
}

fn parity(v: u8) -> u8 {
    if v.count_ones() & 1 == 0 { PV } else { 0 }
}

fn szxyp(v: u8) -> u8 {
    szxy(v) | parity(v)
}

struct Run<'a, B: Bus, O: Observer> {
    cpu: &'a mut Z80,
    b: &'a mut B,
    o: &'a mut O,
    /// The instruction's address, for the observer.
    at: u16,
    /// Whether the instruction wrote F, which Q records.
    flags: bool,
    /// The Q the previous instruction left, which SCF and CCF read.
    q: u8,
}

/// One step at the boundary: an NMI, then an INT the flip-flops and EI's delay allow, then HALT's NOP, then the
/// observer's stop and the instruction (Beryl_Z80.md §5).
pub(crate) fn step<B: Bus, O: Observer>(cpu: &mut Z80, b: &mut B, o: &mut O) -> Step {
    let at = cpu.regs.pc;
    let q = cpu.regs.q;
    if b.nmi_edge() {
        let mut x = Run { cpu, b, o, at, flags: false, q };
        x.accept(true);
        return Step::Interrupt(kind::NMI);
    }
    if cpu.regs.iff1 && !cpu.regs.ei_pending && b.int_line() {
        let mut x = Run { cpu, b, o, at, flags: false, q };
        x.accept(false);
        return Step::Interrupt(kind::IRQ);
    }
    if cpu.halted {
        let mut x = Run { cpu, b, o, at, flags: false, q };
        x.m1();
        x.cpu.regs.pc = at;
        x.cpu.regs.ei_pending = false;
        x.cpu.regs.p = false;
        x.cpu.regs.q = 0;
        return Step::Halted;
    }
    let why = o.before(cpu.processor, at as u32);
    if why != 0 {
        return Step::Observed(why);
    }
    let mut x = Run { cpu, b, o, at, flags: false, q };
    x.cpu.regs.ei_pending = false;
    x.cpu.regs.p = false;
    x.instruction();
    x.cpu.regs.q = if x.flags { x.f() } else { 0 };
    if x.cpu.halted { Step::Halted } else { Step::Instruction }
}

impl<B: Bus, O: Observer> Run<'_, B, O> {
    // ---- registers

    fn a(&self) -> u8 {
        (self.cpu.regs.af >> 8) as u8
    }
    fn f(&self) -> u8 {
        self.cpu.regs.af as u8
    }
    fn set_a(&mut self, v: u8) {
        self.cpu.regs.af = (self.cpu.regs.af & 0xFF) | (v as u16) << 8;
    }
    fn set_f(&mut self, v: u8) {
        self.cpu.regs.af = (self.cpu.regs.af & 0xFF00) | v as u16;
        self.flags = true;
    }
    fn hl(&self, ix: Idx) -> u16 {
        match ix {
            Idx::Hl => self.cpu.regs.hl,
            Idx::Ix => self.cpu.regs.ix,
            Idx::Iy => self.cpu.regs.iy,
        }
    }
    fn set_hl(&mut self, ix: Idx, v: u16) {
        match ix {
            Idx::Hl => self.cpu.regs.hl = v,
            Idx::Ix => self.cpu.regs.ix = v,
            Idx::Iy => self.cpu.regs.iy = v,
        }
    }

    /// Register `r` of an opcode's field (not 6, the memory operand); H and L are the index register's halves under
    /// a prefix.
    fn get8(&self, r: u8, ix: Idx) -> u8 {
        let g = &self.cpu.regs;
        match r {
            0 => (g.bc >> 8) as u8,
            1 => g.bc as u8,
            2 => (g.de >> 8) as u8,
            3 => g.de as u8,
            4 => (self.hl(ix) >> 8) as u8,
            5 => self.hl(ix) as u8,
            7 => self.a(),
            _ => unreachable!(),
        }
    }
    fn set8(&mut self, r: u8, ix: Idx, v: u8) {
        let hi = |p: u16| (p & 0xFF) | (v as u16) << 8;
        let lo = |p: u16| (p & 0xFF00) | v as u16;
        match r {
            0 => self.cpu.regs.bc = hi(self.cpu.regs.bc),
            1 => self.cpu.regs.bc = lo(self.cpu.regs.bc),
            2 => self.cpu.regs.de = hi(self.cpu.regs.de),
            3 => self.cpu.regs.de = lo(self.cpu.regs.de),
            4 => {
                let p = self.hl(ix);
                self.set_hl(ix, hi(p));
            }
            5 => {
                let p = self.hl(ix);
                self.set_hl(ix, lo(p));
            }
            7 => self.set_a(v),
            _ => unreachable!(),
        }
    }
    fn rp(&self, p: u8, ix: Idx) -> u16 {
        match p {
            0 => self.cpu.regs.bc,
            1 => self.cpu.regs.de,
            2 => self.hl(ix),
            _ => self.cpu.regs.sp,
        }
    }
    fn set_rp(&mut self, p: u8, ix: Idx, v: u16) {
        match p {
            0 => self.cpu.regs.bc = v,
            1 => self.cpu.regs.de = v,
            2 => self.set_hl(ix, v),
            _ => self.cpu.regs.sp = v,
        }
    }
    fn rp2(&self, p: u8, ix: Idx) -> u16 {
        if p == 3 { self.cpu.regs.af } else { self.rp(p, ix) }
    }
    fn set_rp2(&mut self, p: u8, ix: Idx, v: u16) {
        if p == 3 { self.cpu.regs.af = v } else { self.set_rp(p, ix, v) }
    }

    // ---- machine cycles

    /// An M1 cycle: the opcode, then the refresh with I and R, R's low seven bits counting.
    fn m1(&mut self) -> u8 {
        let g = &mut self.cpu.regs;
        let refresh = (g.i as u16) << 8 | g.r as u16;
        let pc = g.pc;
        g.pc = pc.wrapping_add(1);
        g.r = (g.r & 0x80) | (g.r.wrapping_add(1) & 0x7F);
        self.b.fetch(pc, refresh)
    }
    fn imm(&mut self) -> u8 {
        let pc = self.cpu.regs.pc;
        self.cpu.regs.pc = pc.wrapping_add(1);
        self.b.read(pc)
    }
    fn imm16(&mut self) -> u16 {
        let lo = self.imm() as u16;
        lo | (self.imm() as u16) << 8
    }
    fn read(&mut self, a: u16) -> u8 {
        self.b.read(a)
    }
    fn write(&mut self, a: u16, v: u8) {
        self.b.write(a, v);
        self.o.wrote(self.cpu.space, a as u32, v, self.at as u32);
    }
    fn idle(&mut self, t: u32) {
        self.b.idle(t);
    }
    fn push(&mut self, v: u16) {
        let sp = self.cpu.regs.sp.wrapping_sub(1);
        self.write(sp, (v >> 8) as u8);
        let sp = sp.wrapping_sub(1);
        self.write(sp, v as u8);
        self.cpu.regs.sp = sp;
    }
    fn pop(&mut self) -> u16 {
        let sp = self.cpu.regs.sp;
        let lo = self.read(sp) as u16;
        let hi = self.read(sp.wrapping_add(1)) as u16;
        self.cpu.regs.sp = sp.wrapping_add(2);
        lo | hi << 8
    }
    /// The memory operand's address: HL, or the index plus the displacement after five internal T-states.
    fn operand(&mut self, ix: Idx) -> u16 {
        if ix == Idx::Hl {
            return self.cpu.regs.hl;
        }
        let d = self.imm() as i8 as u16;
        self.idle(5);
        let a = self.hl(ix).wrapping_add(d);
        self.cpu.regs.wz = a;
        a
    }

    fn cond(&self, c: u8) -> bool {
        let f = self.f();
        match c {
            0 => f & Z == 0,
            1 => f & Z != 0,
            2 => f & C == 0,
            3 => f & C != 0,
            4 => f & PV == 0,
            5 => f & PV != 0,
            6 => f & S == 0,
            _ => f & S != 0,
        }
    }

    // ---- the arithmetic

    fn alu(&mut self, op: u8, v: u8) {
        let a = self.a();
        let cf = self.f() & C;
        match op {
            0 | 1 => {
                let c = if op == 1 { cf } else { 0 };
                let r16 = a as u16 + v as u16 + c as u16;
                let r = r16 as u8;
                let ov = if (a ^ r) & (v ^ r) & 0x80 != 0 { PV } else { 0 };
                self.set_f(szxy(r) | ((a ^ v ^ r) & H) | ov | if r16 > 0xFF { C } else { 0 });
                self.set_a(r);
            }
            2 | 3 | 7 => {
                let c = if op == 3 { cf } else { 0 };
                let r16 = (a as u16).wrapping_sub(v as u16).wrapping_sub(c as u16);
                let r = r16 as u8;
                let ov = if (a ^ v) & (a ^ r) & 0x80 != 0 { PV } else { 0 };
                let base = (r & S) | if r == 0 { Z } else { 0 } | ((a ^ v ^ r) & H) | ov | N | if r16 > 0xFF { C } else { 0 };
                if op == 7 {
                    self.set_f(base | (v & (X | Y)));
                } else {
                    self.set_f(base | (r & (X | Y)));
                    self.set_a(r);
                }
            }
            4 => {
                let r = a & v;
                self.set_f(szxyp(r) | H);
                self.set_a(r);
            }
            5 => {
                let r = a ^ v;
                self.set_f(szxyp(r));
                self.set_a(r);
            }
            _ => {
                let r = a | v;
                self.set_f(szxyp(r));
                self.set_a(r);
            }
        }
    }
    fn inc8(&mut self, v: u8) -> u8 {
        let r = v.wrapping_add(1);
        let f = (self.f() & C) | szxy(r) | if r & 0xF == 0 { H } else { 0 } | if r == 0x80 { PV } else { 0 };
        self.set_f(f);
        r
    }
    fn dec8(&mut self, v: u8) -> u8 {
        let r = v.wrapping_sub(1);
        let f = (self.f() & C) | szxy(r) | N | if r & 0xF == 0xF { H } else { 0 } | if r == 0x7F { PV } else { 0 };
        self.set_f(f);
        r
    }
    fn rot(&mut self, op: u8, v: u8) -> u8 {
        let cf = self.f() & C;
        let (r, c) = match op {
            0 => (v.rotate_left(1), v >> 7),
            1 => (v.rotate_right(1), v & 1),
            2 => (v << 1 | cf, v >> 7),
            3 => (v >> 1 | cf << 7, v & 1),
            4 => (v << 1, v >> 7),
            5 => (v >> 1 | (v & 0x80), v & 1),
            6 => (v << 1 | 1, v >> 7),
            _ => (v >> 1, v & 1),
        };
        self.set_f(szxyp(r) | c);
        r
    }
    /// BIT: X and Y from `xy`, the register for a register operand, WZ's high byte for a memory one.
    fn bit(&mut self, n: u8, v: u8, xy: u8) {
        let set = v & (1 << n);
        let mut f = (self.f() & C) | H | (xy & (X | Y));
        if set == 0 {
            f |= Z | PV;
        } else if n == 7 {
            f |= S;
        }
        self.set_f(f);
    }
    fn add16(&mut self, a: u16, v: u16) -> u16 {
        let r = a as u32 + v as u32;
        let f = (self.f() & (S | Z | PV)) | ((r >> 8) as u8 & (X | Y)) | if (a ^ v ^ r as u16) & 0x1000 != 0 { H } else { 0 } | if r > 0xFFFF { C } else { 0 };
        self.set_f(f);
        self.cpu.regs.wz = a.wrapping_add(1);
        r as u16
    }
    fn adc16(&mut self, a: u16, v: u16, sub: bool) -> u16 {
        let c = (self.f() & C) as u32;
        let r = if sub { (a as u32).wrapping_sub(v as u32).wrapping_sub(c) } else { a as u32 + v as u32 + c };
        let r16 = r as u16;
        let ov = if sub { (a ^ v) & (a ^ r16) & 0x8000 != 0 } else { (a ^ r16) & (v ^ r16) & 0x8000 != 0 };
        let f = ((r16 >> 8) as u8 & (S | X | Y))
            | if r16 == 0 { Z } else { 0 }
            | if (a ^ v ^ r16) & 0x1000 != 0 { H } else { 0 }
            | if ov { PV } else { 0 }
            | if sub { N } else { 0 }
            | if r > 0xFFFF { C } else { 0 };
        self.set_f(f);
        self.cpu.regs.wz = a.wrapping_add(1);
        r16
    }
    fn daa(&mut self) {
        let a = self.a();
        let f = self.f();
        let mut diff = 0u8;
        let mut c = f & C;
        if f & H != 0 || a & 0xF > 9 {
            diff = 6;
        }
        if c != 0 || a > 0x99 {
            diff |= 0x60;
            c = C;
        }
        let (r, h) = if f & N != 0 {
            (a.wrapping_sub(diff), if f & H != 0 && a & 0xF < 6 { H } else { 0 })
        } else {
            (a.wrapping_add(diff), if a & 0xF > 9 { H } else { 0 })
        };
        self.set_f(szxyp(r) | h | (f & N) | c);
        self.set_a(r);
    }

    // ---- interrupts

    /// An accepted interrupt: NMI's M1 (its opcode ignored) or INT's acknowledge, then the push and the handler's
    /// address by the mode. An INT accepted after LD A,I or LD A,R clears the P/V it set.
    fn accept(&mut self, nmi: bool) {
        self.cpu.halted = false;
        self.cpu.regs.ei_pending = false;
        if !nmi && self.cpu.regs.p {
            let f = self.f() & !PV;
            self.cpu.regs.af = (self.cpu.regs.af & 0xFF00) | f as u16;
        }
        self.cpu.regs.p = false;
        self.cpu.regs.q = 0;
        let pc = self.cpu.regs.pc;
        if nmi {
            self.cpu.regs.iff1 = false;
            self.m1();
            self.cpu.regs.pc = pc;
            self.idle(1);
            self.push(pc);
            self.handler(pc, 0x66, kind::NMI);
            return;
        }
        self.cpu.regs.iff1 = false;
        self.cpu.regs.iff2 = false;
        let g = &mut self.cpu.regs;
        g.r = (g.r & 0x80) | (g.r.wrapping_add(1) & 0x7F);
        let byte = self.b.acknowledge();
        match self.cpu.regs.im {
            2 => {
                self.idle(1);
                self.push(pc);
                let table = (self.cpu.regs.i as u16) << 8 | byte as u16;
                let lo = self.read(table) as u16;
                let hi = self.read(table.wrapping_add(1)) as u16;
                self.handler(pc, lo | hi << 8, kind::IRQ);
            }
            1 => {
                self.idle(1);
                self.push(pc);
                self.handler(pc, 0x38, kind::IRQ);
            }
            _ => {
                if byte & 0xC7 == 0xC7 {
                    self.idle(1);
                    self.push(pc);
                    self.handler(pc, (byte & 0x38) as u16, kind::IRQ);
                } else {
                    self.main(byte, Idx::Hl);
                }
            }
        }
    }

    fn handler(&mut self, from: u16, t: u16, why: u32) {
        self.o.called(from as u32, t as u32, why);
        self.cpu.regs.pc = t;
        self.cpu.regs.wz = t;
    }

    // ---- decoding

    fn instruction(&mut self) {
        let mut op = self.m1();
        let mut ix = Idx::Hl;
        while op == 0xDD || op == 0xFD {
            ix = if op == 0xDD { Idx::Ix } else { Idx::Iy };
            self.q = 0;
            op = self.m1();
        }
        match op {
            0xCB if ix == Idx::Hl => {
                let op = self.m1();
                self.cb(op);
            }
            0xCB => self.indexed_cb(ix),
            0xED => {
                let op = self.m1();
                self.ed(op);
            }
            _ => self.main(op, ix),
        }
    }

    fn main(&mut self, op: u8, ix: Idx) {
        let y = (op >> 3) & 7;
        let z = op & 7;
        let p = y >> 1;
        match op >> 6 {
            0 => match z {
                0 => match y {
                    0 => {}
                    1 => {
                        let g = &mut self.cpu.regs;
                        std::mem::swap(&mut g.af, &mut g.af_);
                    }
                    2 => {
                        self.idle(1);
                        let b = ((self.cpu.regs.bc >> 8) as u8).wrapping_sub(1);
                        self.cpu.regs.bc = (self.cpu.regs.bc & 0xFF) | (b as u16) << 8;
                        let e = self.imm() as i8 as u16;
                        if b != 0 {
                            self.idle(5);
                            let t = self.cpu.regs.pc.wrapping_add(e);
                            self.cpu.regs.pc = t;
                            self.cpu.regs.wz = t;
                        }
                    }
                    _ => {
                        let e = self.imm() as i8 as u16;
                        if y == 3 || self.cond(y - 4) {
                            self.idle(5);
                            let t = self.cpu.regs.pc.wrapping_add(e);
                            self.cpu.regs.pc = t;
                            self.cpu.regs.wz = t;
                        }
                    }
                },
                1 => {
                    if y & 1 == 0 {
                        let v = self.imm16();
                        self.set_rp(p, ix, v);
                    } else {
                        self.idle(7);
                        let r = self.add16(self.hl(ix), self.rp(p, ix));
                        self.set_hl(ix, r);
                    }
                }
                2 => match y {
                    0 | 2 => {
                        let a = if y == 0 { self.cpu.regs.bc } else { self.cpu.regs.de };
                        let v = self.a();
                        self.write(a, v);
                        self.cpu.regs.wz = (v as u16) << 8 | (a.wrapping_add(1) & 0xFF);
                    }
                    1 | 3 => {
                        let a = if y == 1 { self.cpu.regs.bc } else { self.cpu.regs.de };
                        let v = self.read(a);
                        self.set_a(v);
                        self.cpu.regs.wz = a.wrapping_add(1);
                    }
                    4 => {
                        let a = self.imm16();
                        let v = self.hl(ix);
                        self.write(a, v as u8);
                        self.write(a.wrapping_add(1), (v >> 8) as u8);
                        self.cpu.regs.wz = a.wrapping_add(1);
                    }
                    5 => {
                        let a = self.imm16();
                        let lo = self.read(a) as u16;
                        let hi = self.read(a.wrapping_add(1)) as u16;
                        self.set_hl(ix, lo | hi << 8);
                        self.cpu.regs.wz = a.wrapping_add(1);
                    }
                    6 => {
                        let a = self.imm16();
                        let v = self.a();
                        self.write(a, v);
                        self.cpu.regs.wz = (v as u16) << 8 | (a.wrapping_add(1) & 0xFF);
                    }
                    _ => {
                        let a = self.imm16();
                        let v = self.read(a);
                        self.set_a(v);
                        self.cpu.regs.wz = a.wrapping_add(1);
                    }
                },
                3 => {
                    self.idle(2);
                    let v = self.rp(p, ix);
                    let v = if y & 1 == 0 { v.wrapping_add(1) } else { v.wrapping_sub(1) };
                    self.set_rp(p, ix, v);
                }
                4 | 5 => {
                    if y == 6 {
                        let a = self.operand(ix);
                        let v = self.read(a);
                        self.idle(1);
                        let r = if z == 4 { self.inc8(v) } else { self.dec8(v) };
                        self.write(a, r);
                    } else {
                        let v = self.get8(y, ix);
                        let r = if z == 4 { self.inc8(v) } else { self.dec8(v) };
                        self.set8(y, ix, r);
                    }
                }
                6 => {
                    if y == 6 {
                        let a = if ix == Idx::Hl {
                            self.cpu.regs.hl
                        } else {
                            let d = self.imm() as i8 as u16;
                            let a = self.hl(ix).wrapping_add(d);
                            self.cpu.regs.wz = a;
                            a
                        };
                        let n = self.imm();
                        if ix != Idx::Hl {
                            self.idle(2);
                        }
                        self.write(a, n);
                    } else {
                        let n = self.imm();
                        self.set8(y, ix, n);
                    }
                }
                _ => self.accumulator(y),
            },
            1 => {
                if op == 0x76 {
                    self.cpu.halted = true;
                } else if y == 6 {
                    let a = self.operand(ix);
                    let v = self.get8(z, Idx::Hl);
                    self.write(a, v);
                } else if z == 6 {
                    let a = self.operand(ix);
                    let v = self.read(a);
                    self.set8(y, Idx::Hl, v);
                } else {
                    let v = self.get8(z, ix);
                    self.set8(y, ix, v);
                }
            }
            2 => {
                let v = if z == 6 {
                    let a = self.operand(ix);
                    self.read(a)
                } else {
                    self.get8(z, ix)
                };
                self.alu(y, v);
            }
            _ => match z {
                0 => {
                    self.idle(1);
                    if self.cond(y) {
                        self.ret();
                    }
                }
                1 => {
                    if y & 1 == 0 {
                        let v = self.pop();
                        self.set_rp2(p, ix, v);
                    } else {
                        match p {
                            0 => self.ret(),
                            1 => {
                                let g = &mut self.cpu.regs;
                                std::mem::swap(&mut g.bc, &mut g.bc_);
                                std::mem::swap(&mut g.de, &mut g.de_);
                                std::mem::swap(&mut g.hl, &mut g.hl_);
                            }
                            2 => self.cpu.regs.pc = self.hl(ix),
                            _ => {
                                self.idle(2);
                                self.cpu.regs.sp = self.hl(ix);
                            }
                        }
                    }
                }
                2 => {
                    let t = self.imm16();
                    self.cpu.regs.wz = t;
                    if self.cond(y) {
                        self.cpu.regs.pc = t;
                    }
                }
                3 => match y {
                    0 => {
                        let t = self.imm16();
                        self.cpu.regs.wz = t;
                        self.cpu.regs.pc = t;
                    }
                    2 => {
                        let n = self.imm();
                        let a = self.a();
                        let port = (a as u16) << 8 | n as u16;
                        self.b.output(port, a);
                        self.cpu.regs.wz = (a as u16) << 8 | (n.wrapping_add(1) as u16);
                    }
                    3 => {
                        let n = self.imm();
                        let port = (self.a() as u16) << 8 | n as u16;
                        let v = self.b.input(port);
                        self.set_a(v);
                        self.cpu.regs.wz = port.wrapping_add(1);
                    }
                    4 => {
                        let sp = self.cpu.regs.sp;
                        let lo = self.read(sp) as u16;
                        let hi = self.read(sp.wrapping_add(1)) as u16;
                        self.idle(1);
                        let v = self.hl(ix);
                        self.write(sp.wrapping_add(1), (v >> 8) as u8);
                        self.write(sp, v as u8);
                        self.idle(2);
                        let n = lo | hi << 8;
                        self.set_hl(ix, n);
                        self.cpu.regs.wz = n;
                    }
                    5 => {
                        let g = &mut self.cpu.regs;
                        std::mem::swap(&mut g.de, &mut g.hl);
                    }
                    6 => {
                        self.cpu.regs.iff1 = false;
                        self.cpu.regs.iff2 = false;
                    }
                    _ => {
                        self.cpu.regs.iff1 = true;
                        self.cpu.regs.iff2 = true;
                        self.cpu.regs.ei_pending = true;
                    }
                },
                4 => {
                    let t = self.imm16();
                    self.cpu.regs.wz = t;
                    if self.cond(y) {
                        self.call(t);
                    }
                }
                5 => {
                    if y & 1 == 0 {
                        self.idle(1);
                        let v = self.rp2(p, ix);
                        self.push(v);
                    } else {
                        let t = self.imm16();
                        self.cpu.regs.wz = t;
                        self.call(t);
                    }
                }
                6 => {
                    let n = self.imm();
                    self.alu(y, n);
                }
                _ => {
                    self.idle(1);
                    let pc = self.cpu.regs.pc;
                    self.push(pc);
                    let t = (y as u16) * 8;
                    self.o.called(self.at as u32, t as u32, kind::CALL);
                    self.cpu.regs.pc = t;
                    self.cpu.regs.wz = t;
                }
            },
        }
    }

    /// CALL's push, after the extra T-state of its second operand read.
    fn call(&mut self, t: u16) {
        self.idle(1);
        let pc = self.cpu.regs.pc;
        self.push(pc);
        self.o.called(self.at as u32, t as u32, kind::CALL);
        self.cpu.regs.pc = t;
    }

    fn ret(&mut self) {
        let t = self.pop();
        self.o.returned();
        self.cpu.regs.pc = t;
        self.cpu.regs.wz = t;
    }

    fn accumulator(&mut self, y: u8) {
        let a = self.a();
        let f = self.f();
        let keep = f & (S | Z | PV);
        match y {
            0 => {
                let r = a.rotate_left(1);
                self.set_f(keep | (r & (X | Y)) | (a >> 7));
                self.set_a(r);
            }
            1 => {
                let r = a.rotate_right(1);
                self.set_f(keep | (r & (X | Y)) | (a & 1));
                self.set_a(r);
            }
            2 => {
                let r = a << 1 | (f & C);
                self.set_f(keep | (r & (X | Y)) | (a >> 7));
                self.set_a(r);
            }
            3 => {
                let r = a >> 1 | (f & C) << 7;
                self.set_f(keep | (r & (X | Y)) | (a & 1));
                self.set_a(r);
            }
            4 => self.daa(),
            5 => {
                let r = !a;
                self.set_f((f & (S | Z | PV | C)) | H | N | (r & (X | Y)));
                self.set_a(r);
            }
            6 => {
                let xy = ((self.q ^ f) | a) & (X | Y);
                self.set_f(keep | xy | C);
            }
            _ => {
                let xy = ((self.q ^ f) | a) & (X | Y);
                self.set_f(keep | xy | if f & C != 0 { H } else { C });
            }
        }
    }

    fn cb(&mut self, op: u8) {
        let y = (op >> 3) & 7;
        let z = op & 7;
        if z == 6 {
            let a = self.cpu.regs.hl;
            let v = self.read(a);
            self.idle(1);
            match op >> 6 {
                0 => {
                    let r = self.rot(y, v);
                    self.write(a, r);
                }
                1 => {
                    let xy = (self.cpu.regs.wz >> 8) as u8;
                    self.bit(y, v, xy);
                }
                2 => self.write(a, v & !(1 << y)),
                _ => self.write(a, v | 1 << y),
            }
            return;
        }
        let v = self.get8(z, Idx::Hl);
        match op >> 6 {
            0 => {
                let r = self.rot(y, v);
                self.set8(z, Idx::Hl, r);
            }
            1 => self.bit(y, v, v),
            2 => self.set8(z, Idx::Hl, v & !(1 << y)),
            _ => self.set8(z, Idx::Hl, v | 1 << y),
        }
    }

    /// DD CB d op and FD CB d op: the displacement, then the opcode read as data, two internal T-states, and the
    /// operand; a result is also written to the register the opcode's low bits name, except for (HL)'s field.
    fn indexed_cb(&mut self, ix: Idx) {
        let d = self.imm() as i8 as u16;
        let op = self.imm();
        self.idle(2);
        let a = self.hl(ix).wrapping_add(d);
        self.cpu.regs.wz = a;
        let y = (op >> 3) & 7;
        let z = op & 7;
        let v = self.read(a);
        self.idle(1);
        let r = match op >> 6 {
            0 => self.rot(y, v),
            1 => {
                self.bit(y, v, (a >> 8) as u8);
                return;
            }
            2 => v & !(1 << y),
            _ => v | 1 << y,
        };
        self.write(a, r);
        if z != 6 {
            self.set8(z, Idx::Hl, r);
        }
    }

    fn ed(&mut self, op: u8) {
        let y = (op >> 3) & 7;
        let z = op & 7;
        let p = y >> 1;
        match op {
            0x40..=0x7F => match z {
                0 => {
                    let bc = self.cpu.regs.bc;
                    let v = self.b.input(bc);
                    self.cpu.regs.wz = bc.wrapping_add(1);
                    let f = (self.f() & C) | szxyp(v);
                    self.set_f(f);
                    if y != 6 {
                        self.set8(y, Idx::Hl, v);
                    }
                }
                1 => {
                    let bc = self.cpu.regs.bc;
                    let v = if y == 6 { 0 } else { self.get8(y, Idx::Hl) };
                    self.b.output(bc, v);
                    self.cpu.regs.wz = bc.wrapping_add(1);
                }
                2 => {
                    self.idle(7);
                    let hl = self.cpu.regs.hl;
                    let v = self.rp(p, Idx::Hl);
                    let r = self.adc16(hl, v, y & 1 == 0);
                    self.cpu.regs.hl = r;
                }
                3 => {
                    let a = self.imm16();
                    if y & 1 == 0 {
                        let v = self.rp(p, Idx::Hl);
                        self.write(a, v as u8);
                        self.write(a.wrapping_add(1), (v >> 8) as u8);
                    } else {
                        let lo = self.read(a) as u16;
                        let hi = self.read(a.wrapping_add(1)) as u16;
                        self.set_rp(p, Idx::Hl, lo | hi << 8);
                    }
                    self.cpu.regs.wz = a.wrapping_add(1);
                }
                4 => {
                    let a = self.a();
                    self.set_a(0);
                    self.alu(2, a);
                }
                5 => {
                    self.cpu.regs.iff1 = self.cpu.regs.iff2;
                    self.ret();
                }
                6 => self.cpu.regs.im = [0, 0, 1, 2][(y & 3) as usize],
                _ => match y {
                    0 => {
                        self.idle(1);
                        self.cpu.regs.i = self.a();
                    }
                    1 => {
                        self.idle(1);
                        self.cpu.regs.r = self.a();
                    }
                    2 | 3 => {
                        self.idle(1);
                        let v = if y == 2 { self.cpu.regs.i } else { self.cpu.regs.r };
                        self.set_a(v);
                        let f = (self.f() & C) | szxy(v) | if self.cpu.regs.iff2 { PV } else { 0 };
                        self.set_f(f);
                        self.cpu.regs.p = true;
                    }
                    4 | 5 => {
                        let hl = self.cpu.regs.hl;
                        let v = self.read(hl);
                        self.idle(4);
                        let a = self.a();
                        let (m, na) = if y == 4 {
                            (a << 4 | v >> 4, (a & 0xF0) | (v & 0xF))
                        } else {
                            (v << 4 | (a & 0xF), (a & 0xF0) | v >> 4)
                        };
                        self.write(hl, m);
                        self.set_a(na);
                        let f = (self.f() & C) | szxyp(na);
                        self.set_f(f);
                        self.cpu.regs.wz = hl.wrapping_add(1);
                    }
                    _ => {}
                },
            },
            0xA0..=0xA3 | 0xA8..=0xAB | 0xB0..=0xB3 | 0xB8..=0xBB => self.block(y, z),
            _ => {}
        }
    }

    /// LDI, CPI, INI, OUTI and their decrementing and repeating forms (Beryl_Z80.md §4).
    fn block(&mut self, y: u8, z: u8) {
        let down = y & 1 != 0;
        let repeat = y >= 6;
        let step = |v: u16| if down { v.wrapping_sub(1) } else { v.wrapping_add(1) };
        let hl = self.cpu.regs.hl;
        match z {
            0 => {
                let v = self.read(hl);
                let de = self.cpu.regs.de;
                self.write(de, v);
                self.idle(2);
                self.cpu.regs.hl = step(hl);
                self.cpu.regs.de = step(de);
                let bc = self.cpu.regs.bc.wrapping_sub(1);
                self.cpu.regs.bc = bc;
                let n = self.a().wrapping_add(v);
                let mut f = (self.f() & (S | Z | C)) | (n & X) | ((n << 4) & Y) | if bc != 0 { PV } else { 0 };
                if repeat && bc != 0 {
                    self.idle(5);
                    f = self.repeat(f);
                }
                self.set_f(f);
            }
            1 => {
                let v = self.read(hl);
                self.idle(5);
                self.cpu.regs.hl = step(hl);
                let bc = self.cpu.regs.bc.wrapping_sub(1);
                self.cpu.regs.bc = bc;
                self.cpu.regs.wz = step(self.cpu.regs.wz);
                let a = self.a();
                let r = a.wrapping_sub(v);
                let h = (a ^ v ^ r) & H;
                let n = r.wrapping_sub(if h != 0 { 1 } else { 0 });
                let mut f = (self.f() & C) | N | (r & S) | if r == 0 { Z } else { 0 } | h | (n & X) | ((n << 4) & Y) | if bc != 0 { PV } else { 0 };
                if repeat && bc != 0 && r != 0 {
                    self.idle(5);
                    f = self.repeat(f);
                }
                self.set_f(f);
            }
            _ => {
                self.idle(1);
                let (v, k) = if z == 2 {
                    let bc = self.cpu.regs.bc;
                    let v = self.b.input(bc);
                    self.cpu.regs.wz = step(bc);
                    self.write(hl, v);
                    let c = step(bc) as u8;
                    (v, v as u16 + c as u16)
                } else {
                    let v = self.read(hl);
                    let b = ((self.cpu.regs.bc >> 8) as u8).wrapping_sub(1);
                    let bc = (self.cpu.regs.bc & 0xFF) | (b as u16) << 8;
                    self.cpu.regs.bc = bc;
                    self.cpu.regs.wz = step(bc);
                    self.b.output(bc, v);
                    (v, v as u16 + step(hl) as u8 as u16)
                };
                if z == 2 {
                    let b = ((self.cpu.regs.bc >> 8) as u8).wrapping_sub(1);
                    self.cpu.regs.bc = (self.cpu.regs.bc & 0xFF) | (b as u16) << 8;
                }
                self.cpu.regs.hl = step(hl);
                let b = (self.cpu.regs.bc >> 8) as u8;
                let carry = k > 0xFF;
                let mut f = szxy(b) | if v & 0x80 != 0 { N } else { 0 } | if carry { H | C } else { 0 } | parity((k as u8 & 7) ^ b);
                if repeat && b != 0 {
                    self.idle(5);
                    f = self.repeat(f);
                    let mut pv = f & PV;
                    f &= !(PV | H);
                    if carry {
                        if v & 0x80 != 0 {
                            pv ^= parity(b.wrapping_sub(1) & 7) ^ PV;
                            f |= if b & 0xF == 0 { H } else { 0 };
                        } else {
                            pv ^= parity(b.wrapping_add(1) & 7) ^ PV;
                            f |= if b & 0xF == 0xF { H } else { 0 };
                        }
                    } else {
                        pv ^= parity(b & 7) ^ PV;
                    }
                    f |= pv;
                }
                self.set_f(f);
            }
        }
    }

    /// A repeating block instruction going round again: the PC back on it, WZ one past it, and X and Y from the PC's
    /// high byte.
    fn repeat(&mut self, f: u8) -> u8 {
        let pc = self.cpu.regs.pc.wrapping_sub(2);
        self.cpu.regs.pc = pc;
        self.cpu.regs.wz = pc.wrapping_add(1);
        (f & !(X | Y)) | ((pc >> 8) as u8 & (X | Y))
    }
}
