//! The 68000's execution: instruction semantics from the Programmer's Reference Manual, each instruction's bus cycles
//! and internal delays in the order Yacht.txt gives them, corrected where the single-step suites show otherwise
//! (Beryl_M68k.md §4).

use crate::{Access, Bus, M68000, Observer, Size, Step};
use emusen_native::debug::kind;

const C: u16 = 1;
const V: u16 = 2;
const Z: u16 = 4;
const N: u16 = 8;
const X: u16 = 16;

/// An operand width.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum W {
    B,
    W,
    L,
}

impl W {
    fn mask(self) -> u32 {
        match self {
            W::B => 0xFF,
            W::W => 0xFFFF,
            W::L => 0xFFFF_FFFF,
        }
    }

    fn msb(self) -> u32 {
        match self {
            W::B => 0x80,
            W::W => 0x8000,
            W::L => 0x8000_0000,
        }
    }

    fn bytes(self) -> u32 {
        match self {
            W::B => 1,
            W::W => 2,
            W::L => 4,
        }
    }

    /// The size field of most instructions, bits 7-6.
    fn from_bits(b: u16) -> Option<W> {
        match b & 3 {
            0 => Some(W::B),
            1 => Some(W::W),
            2 => Some(W::L),
            _ => None,
        }
    }
}

fn sext(v: u32, w: W) -> u32 {
    match w {
        W::B => v as u8 as i8 as i32 as u32,
        W::W => v as u16 as i16 as i32 as u32,
        W::L => v,
    }
}

/// A word access at an odd address: the cycle is abandoned and group 0 processing follows.
#[derive(Clone, Copy, Debug)]
pub(crate) struct AddressFault {
    pub address: u32,
    pub write: bool,
    pub function: u8,
}

type R<T> = Result<T, AddressFault>;

/// A resolved effective address.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Ea {
    D(usize),
    A(usize),
    /// A memory operand; `program` for the PC-relative modes, read in program space.
    M(u32, bool),
    I(u32),
}

/// What an effective-address calculation is for, which decides the predecrement's idle cycle.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Use {
    Read,
    /// MOVE's destination, whose predecrement costs no idle cycle.
    MoveDest,
}

struct Run<'a, B: Bus, O: Observer> {
    cpu: &'a mut M68000,
    b: &'a mut B,
    o: &'a mut O,
    /// The address the next prefetch reads: the opcode's plus four at an instruction's start.
    au: u32,
    /// The opcode's address.
    pc: u32,
    /// The PC a group 0 exception stacks, which advances at points of its own (Beryl_M68k.md §4.2).
    pcv: u32,
    /// The instruction register a group 0 frame stacks: the opcode, or the next one once a final store has begun.
    ir: u16,
    /// An address register's update deferred until its operand has been transferred: the register and its value.
    post: Option<(usize, u32)>,
}

/// One instruction or exception: a pending trace first, then a pending interrupt, then the instruction, which leaves
/// its trace pending (Beryl_M68k.md §5).
pub(crate) fn step<B: Bus, O: Observer>(cpu: &mut M68000, b: &mut B, o: &mut O) -> Step {
    if cpu.halted {
        b.idle(4);
        return Step::Halted;
    }
    if let Some(resume) = cpu.trace_pending.take() {
        cpu.stopped = false;
        return run(cpu, b, o, |x| {
            x.idle(4);
            x.exception(9, resume)
        });
    }
    let level = b.interrupt_level() & 7;
    let edge = level == 7 && cpu.last_level < 7;
    cpu.last_level = level;
    let mask = ((cpu.regs.sr >> 8) & 7) as u8;
    if edge || level > mask {
        let resume = if cpu.stopped { cpu.regs.pc.wrapping_add(4) } else { cpu.regs.pc };
        cpu.stopped = false;
        return run(cpu, b, o, |x| x.interrupt(level, resume));
    }
    if cpu.stopped {
        b.idle(4);
        return Step::Stopped;
    }
    let pc = cpu.regs.pc;
    let why = o.before(cpu.processor, pc);
    if why != 0 {
        return Step::Observed(why);
    }
    let traced = cpu.regs.sr & 0x8000 != 0;
    let step = run(cpu, b, o, |x| x.execute());
    if traced && matches!(step, Step::Instruction | Step::Stopped | Step::Exception(5..=7 | 32..=47)) {
        cpu.trace_pending = Some(if step == Step::Stopped { cpu.regs.pc.wrapping_add(4) } else { cpu.regs.pc });
    }
    step
}

/// One piece of work over a fresh run at the instruction boundary, an address error turned into group 0 processing.
fn run<B: Bus, O: Observer>(cpu: &mut M68000, b: &mut B, o: &mut O, work: impl FnOnce(&mut Run<'_, B, O>) -> R<Step>) -> Step {
    let pc = cpu.regs.pc;
    let mut x = Run { au: pc.wrapping_add(4), pc, pcv: pc.wrapping_add(2), post: None, ir: cpu.regs.prefetch[0], cpu, b, o };
    let step = match work(&mut x) {
        Ok(s) => s,
        Err(f) => x.address_error(f),
    };
    x.cpu.regs.pc = x.au.wrapping_sub(4);
    step
}

/// The reset exception: supervisor mode at mask 7, the stack pointer and PC from vectors 0 and 1 in supervisor program
/// space, and the queue filled from the PC (Beryl_M68k.md §5.2).
pub(crate) fn reset<B: Bus>(cpu: &mut M68000, b: &mut B) {
    cpu.halted = false;
    cpu.stopped = false;
    let sr = cpu.regs.sr;
    let mut o = crate::Unobserved;
    let pc = cpu.regs.pc;
    let mut x = Run { au: pc.wrapping_add(4), pc, pcv: pc.wrapping_add(2), post: None, ir: 0, cpu, b, o: &mut o };
    x.set_sr((sr & 0x00FF) | 0x2700);
    x.idle(14);
    let r = (|| -> R<()> {
        let ssp = x.read16(0, Size::Word, 6)? as u32;
        let ssp = ssp << 16 | x.read16(2, Size::Word, 6)? as u32;
        let hi = x.read16(4, Size::Word, 6)? as u32;
        let pc = hi << 16 | x.read16(6, Size::Word, 6)? as u32;
        x.set_a(7, ssp);
        x.au = pc;
        x.prefetch()?;
        x.idle(2);
        x.next()
    })();
    if r.is_err() {
        x.cpu.halted = true;
    }
    x.cpu.regs.pc = x.au.wrapping_sub(4);
}

impl<B: Bus, O: Observer> Run<'_, B, O> {
    // ---- registers and flags

    fn sr(&self) -> u16 {
        self.cpu.regs.sr
    }

    fn s(&self) -> bool {
        self.cpu.regs.sr & 0x2000 != 0
    }

    fn set_sr(&mut self, v: u16) {
        let v = v & 0xA71F;
        let r = &mut self.cpu.regs;
        if (r.sr ^ v) & 0x2000 != 0 {
            std::mem::swap(&mut r.a[7], &mut r.other_sp);
        }
        r.sr = v;
    }

    fn set_ccr(&mut self, v: u16) {
        self.cpu.regs.sr = (self.cpu.regs.sr & 0xFF00) | (v & 0x1F);
    }

    fn flag(&self, f: u16) -> bool {
        self.cpu.regs.sr & f != 0
    }

    fn put(&mut self, f: u16, on: bool) {
        if on {
            self.cpu.regs.sr |= f;
        } else {
            self.cpu.regs.sr &= !f;
        }
    }

    fn d(&self, n: usize) -> u32 {
        self.cpu.regs.d[n]
    }

    fn a(&self, n: usize) -> u32 {
        self.cpu.regs.a[n]
    }

    fn set_d(&mut self, n: usize, w: W, v: u32) {
        let m = w.mask();
        let r = &mut self.cpu.regs.d[n];
        *r = (*r & !m) | (v & m);
    }

    fn set_a(&mut self, n: usize, v: u32) {
        self.cpu.regs.a[n] = v;
    }

    fn nz(&mut self, v: u32, w: W) {
        let v = v & w.mask();
        self.put(N, v & w.msb() != 0);
        self.put(Z, v == 0);
    }

    /// N and Z from the result, V and C cleared: the logical instructions and MOVE.
    fn logic(&mut self, v: u32, w: W) {
        self.nz(v, w);
        self.put(V | C, false);
    }

    fn cond(&self, c: u16) -> bool {
        let (n, z, v, cy) = (self.flag(N), self.flag(Z), self.flag(V), self.flag(C));
        match c & 15 {
            0 => true,
            1 => false,
            2 => !cy && !z,
            3 => cy || z,
            4 => !cy,
            5 => cy,
            6 => !z,
            7 => z,
            8 => !v,
            9 => v,
            10 => !n,
            11 => n,
            12 => n == v,
            13 => n != v,
            14 => !z && n == v,
            _ => z || n != v,
        }
    }

    fn add(&mut self, s: u32, d: u32, x: bool, w: W, extend: bool) -> u32 {
        let m = w.mask();
        let (s, d) = (s & m, d & m);
        let r64 = s as u64 + d as u64 + x as u64;
        let r = (r64 as u32) & m;
        let carry = r64 > m as u64;
        self.put(C, carry);
        self.put(X, carry);
        self.put(V, (s ^ r) & (d ^ r) & w.msb() != 0);
        self.put(N, r & w.msb() != 0);
        if extend {
            if r != 0 {
                self.put(Z, false);
            }
        } else {
            self.put(Z, r == 0);
        }
        r
    }

    /// `d - s - x`; `compare` leaves X alone.
    fn sub(&mut self, s: u32, d: u32, x: bool, w: W, extend: bool, compare: bool) -> u32 {
        let m = w.mask();
        let (s, d) = (s & m, d & m);
        let r = d.wrapping_sub(s).wrapping_sub(x as u32) & m;
        let borrow = (s as u64 + x as u64) > d as u64;
        self.put(C, borrow);
        if !compare {
            self.put(X, borrow);
        }
        self.put(V, (s ^ d) & (r ^ d) & w.msb() != 0);
        self.put(N, r & w.msb() != 0);
        if extend {
            if r != 0 {
                self.put(Z, false);
            }
        } else {
            self.put(Z, r == 0);
        }
        r
    }

    // ---- the bus

    fn fc_data(&self) -> u8 {
        if self.s() { 5 } else { 1 }
    }

    fn fc_program(&self) -> u8 {
        if self.s() { 6 } else { 2 }
    }

    fn read16(&mut self, raw: u32, size: Size, function: u8) -> R<u16> {
        let address = raw & 0xFF_FFFF;
        let access = Access { address, size, function, locked: false };
        if size == Size::Word && address & 1 != 0 {
            self.b.address_error(access, false);
            return Err(AddressFault { address: raw, write: false, function });
        }
        Ok(self.b.read(access))
    }

    fn write16(&mut self, raw: u32, size: Size, value: u16, function: u8) -> R<()> {
        let address = raw & 0xFF_FFFF;
        let access = Access { address, size, function, locked: false };
        if size == Size::Word && address & 1 != 0 {
            self.b.address_error(access, true);
            return Err(AddressFault { address: raw, write: true, function });
        }
        self.b.write(access, value);
        match size {
            Size::Byte => self.o.wrote(self.cpu.space, address, value as u8, self.pc),
            Size::Word => {
                self.o.wrote(self.cpu.space, address, (value >> 8) as u8, self.pc);
                self.o.wrote(self.cpu.space, address | 1, value as u8, self.pc);
            }
        }
        Ok(())
    }

    /// An operand read in data space (or program space for `program`); a long is the high word, then the low.
    fn read(&mut self, address: u32, w: W, program: bool) -> R<u32> {
        let fc = if program { self.fc_program() } else { self.fc_data() };
        Ok(match w {
            W::B => self.read16(address, Size::Byte, fc)? as u32 & 0xFF,
            W::W => self.read16(address, Size::Word, fc)? as u32,
            W::L => {
                let hi = self.read16(address, Size::Word, fc)? as u32;
                let lo = self.read16(address.wrapping_add(2), Size::Word, fc)? as u32;
                hi << 16 | lo
            }
        })
    }

    /// An operand write; a long is the high word, then the low.
    fn write(&mut self, address: u32, w: W, v: u32) -> R<()> {
        let fc = self.fc_data();
        match w {
            W::B => self.write16(address, Size::Byte, v as u16 & 0xFF, fc),
            W::W => self.write16(address, Size::Word, v as u16, fc),
            W::L => {
                self.write16(address, Size::Word, (v >> 16) as u16, fc)?;
                self.write16(address.wrapping_add(2), Size::Word, v as u16, fc)
            }
        }
    }

    /// A long written low word first, as the read-modify-write instructions and the predecrement store it.
    fn write_low_first(&mut self, address: u32, w: W, v: u32) -> R<()> {
        if w != W::L {
            return self.write(address, w, v);
        }
        let fc = self.fc_data();
        self.write16(address.wrapping_add(2), Size::Word, v as u16, fc)?;
        self.write16(address, Size::Word, (v >> 16) as u16, fc)
    }

    fn idle(&mut self, clocks: u32) {
        self.b.idle(clocks);
    }

    /// The next word into IRC.
    fn prefetch(&mut self) -> R<()> {
        let fc = self.fc_program();
        let w = self.read16(self.au, Size::Word, fc)?;
        self.cpu.regs.prefetch[1] = w;
        self.au = self.au.wrapping_add(2);
        Ok(())
    }

    fn irc(&self) -> u16 {
        self.cpu.regs.prefetch[1]
    }

    /// An extension word: IRC, refilled.
    fn ext16(&mut self) -> R<u16> {
        let v = self.irc();
        self.prefetch()?;
        Ok(v)
    }

    fn ext32(&mut self) -> R<u32> {
        let hi = self.ext16()? as u32;
        let lo = self.ext16()? as u32;
        Ok(hi << 16 | lo)
    }

    /// The instruction's last prefetch: IRC into IRD, the next word into IRC.
    fn next(&mut self) -> R<()> {
        self.cpu.regs.prefetch[0] = self.cpu.regs.prefetch[1];
        self.prefetch()
    }

    /// A jump: the queue refilled from `target`.
    fn fill(&mut self, target: u32) -> R<()> {
        self.au = target;
        self.prefetch()?;
        self.next()
    }

    /// After SR or CCR changes, the queue is refilled from the word after the instruction.
    fn refill(&mut self) -> R<()> {
        self.au = self.au.wrapping_sub(2);
        self.prefetch()?;
        self.next()
    }

    fn push32(&mut self, v: u32) -> R<()> {
        let sp = self.a(7).wrapping_sub(4);
        self.set_a(7, sp);
        self.write(sp, W::L, v)
    }

    fn pop32(&mut self) -> R<u32> {
        let sp = self.a(7);
        let v = self.read(sp, W::L, false)?;
        self.set_a(7, sp.wrapping_add(4));
        Ok(v)
    }

    // ---- effective addresses

    /// The brief extension word of the indexed modes: the index register, its width and the 8-bit displacement.
    fn index(&mut self, base: u32) -> R<u32> {
        let ext = self.ext16()?;
        let r = (ext >> 12) as usize & 7;
        let xn = if ext & 0x8000 != 0 { self.a(r) } else { self.d(r) };
        let xn = if ext & 0x800 != 0 { xn } else { xn as u16 as i16 as i32 as u32 };
        Ok(base.wrapping_add(xn).wrapping_add(ext as u8 as i8 as i32 as u32))
    }

    /// An effective address resolved, with the extension words it fetches and the delays it spends.
    fn ea(&mut self, mode: u16, reg: u16, w: W, why: Use) -> R<Ea> {
        let r = reg as usize & 7;
        let step = if w == W::B && r == 7 { 2 } else { w.bytes() };
        Ok(match mode & 7 {
            0 => Ea::D(r),
            1 => Ea::A(r),
            2 => Ea::M(self.a(r), false),
            3 => {
                let a = self.a(r);
                if w == W::L || why == Use::MoveDest {
                    self.post = Some((r, a.wrapping_add(step)));
                } else {
                    self.set_a(r, a.wrapping_add(step));
                }
                Ea::M(a, false)
            }
            4 => {
                if why == Use::Read {
                    self.idle(2);
                    if w == W::W {
                        self.pcv = self.pcv.wrapping_add(2);
                    }
                }
                let a = self.a(r).wrapping_sub(step);
                if why == Use::MoveDest && w == W::L {
                    self.post = Some((r, a));
                } else {
                    self.set_a(r, a);
                }
                Ea::M(a, false)
            }
            5 => {
                let d = self.ext16()? as i16 as i32 as u32;
                Ea::M(self.a(r).wrapping_add(d), false)
            }
            6 => {
                self.idle(2);
                let base = self.a(r);
                Ea::M(self.index(base)?, false)
            }
            _ => match reg & 7 {
                0 => {
                    if why == Use::Read {
                        self.pcv = self.pcv.wrapping_add(2);
                    }
                    Ea::M(self.ext16()? as i16 as i32 as u32, false)
                }
                1 => {
                    self.pcv = self.pcv.wrapping_add(if why == Use::Read { 4 } else { 2 });
                    Ea::M(self.ext32()?, false)
                }
                2 => {
                    let base = self.au.wrapping_sub(2);
                    let d = self.ext16()? as i16 as i32 as u32;
                    Ea::M(base.wrapping_add(d), true)
                }
                3 => {
                    self.idle(2);
                    let base = self.au.wrapping_sub(2);
                    Ea::M(self.index(base)?, true)
                }
                _ => {
                    self.pcv = self.pcv.wrapping_add(if w == W::L { 4 } else { 2 });
                    match w {
                        W::L => Ea::I(self.ext32()?),
                        _ => Ea::I(self.ext16()? as u32 & w.mask()),
                    }
                }
            },
        })
    }

    fn get(&mut self, ea: Ea, w: W) -> R<u32> {
        Ok(match ea {
            Ea::D(n) => self.d(n) & w.mask(),
            Ea::A(n) => self.a(n) & w.mask(),
            Ea::M(a, p) => {
                let v = self.read(a, w, p)?;
                self.apply_post();
                v
            }
            Ea::I(v) => v & w.mask(),
        })
    }

    fn apply_post(&mut self) {
        if let Some((r, v)) = self.post.take() {
            self.set_a(r, v);
        }
    }

    /// A register destination written now; a memory one written with its long low word first, after `next`.
    fn put_rmw(&mut self, ea: Ea, w: W, v: u32) -> R<()> {
        match ea {
            Ea::D(n) => self.set_d(n, w, v),
            Ea::A(n) => self.set_a(n, v),
            Ea::M(a, _) => self.write_low_first(a, w, v)?,
            Ea::I(_) => {}
        }
        Ok(())
    }

    fn is_mem(ea: Ea) -> bool {
        matches!(ea, Ea::M(..))
    }

    // ---- exceptions

    /// Group 1 and 2 processing: SR saved, supervisor mode, the PC and SR stacked low word of the PC first, the
    /// vector read and the queue filled from the handler.
    fn exception(&mut self, vector: u8, pc: u32) -> R<Step> {
        let old = self.sr();
        self.set_sr((old | 0x2000) & !0x8000);
        self.exception_from(old, vector, pc)
    }

    /// The stacking and the vector, supervisor mode entered already with `old` the SR to save.
    fn exception_from(&mut self, old: u16, vector: u8, pc: u32) -> R<Step> {
        let sp = self.a(7).wrapping_sub(6);
        self.set_a(7, sp);
        let fc = self.fc_data();
        self.write16(sp.wrapping_add(4), Size::Word, pc as u16, fc)?;
        self.write16(sp, Size::Word, old, fc)?;
        self.write16(sp.wrapping_add(2), Size::Word, (pc >> 16) as u16, fc)?;
        let handler = self.read(vector as u32 * 4, W::L, false)?;
        self.o.called(pc, handler, kind::IRQ);
        self.au = handler;
        self.prefetch()?;
        self.idle(2);
        self.next()?;
        Ok(Step::Exception(vector))
    }

    /// An interrupt at `level`: six idle clocks, the PC's low word stacked, the acknowledge cycle (the bus counts its
    /// clocks) or the autovector, four more idle clocks, the SR and the PC's high word, then the vector.
    fn interrupt(&mut self, level: u8, pc: u32) -> R<Step> {
        self.idle(6);
        let old = self.sr();
        self.set_sr((old | 0x2000) & !0x8700 | (level as u16) << 8);
        let sp = self.a(7).wrapping_sub(6);
        self.set_a(7, sp);
        let fc = self.fc_data();
        self.write16(sp.wrapping_add(4), Size::Word, pc as u16, fc)?;
        let vector = self.b.acknowledge(level).unwrap_or(24 + level);
        self.idle(4);
        self.write16(sp, Size::Word, old, fc)?;
        self.write16(sp.wrapping_add(2), Size::Word, (pc >> 16) as u16, fc)?;
        let handler = self.read(vector as u32 * 4, W::L, false)?;
        self.o.called(pc, handler, if level == 7 { kind::NMI } else { kind::IRQ });
        self.au = handler;
        self.prefetch()?;
        self.idle(2);
        self.next()?;
        Ok(Step::Exception(vector))
    }

    /// Group 0: the access that faulted, stacked with the instruction register and the PC (Beryl_M68k.md §4).
    fn address_error(&mut self, f: AddressFault) -> Step {
        self.idle(8);
        let old = self.sr();
        self.set_sr((old | 0x2000) & !0x8000);
        let ir = self.ir;
        let status = (ir & !0x1F) | if f.write { 0 } else { 0x10 } | f.function as u16;
        let pc = self.pcv;
        let sp = self.a(7).wrapping_sub(14);
        self.set_a(7, sp);
        let fc = 5;
        let writes = [
            (sp.wrapping_add(12), pc as u16),
            (sp.wrapping_add(8), old),
            (sp.wrapping_add(10), (pc >> 16) as u16),
            (sp.wrapping_add(6), ir),
            (sp.wrapping_add(4), f.address as u16),
            (sp, status),
            (sp.wrapping_add(2), (f.address >> 16) as u16),
        ];
        for (a, v) in writes {
            if self.write16(a, Size::Word, v, fc).is_err() {
                self.cpu.halted = true;
                return Step::Halted;
            }
        }
        let r = (|| -> R<()> {
            let handler = self.read(12, W::L, false)?;
            self.au = handler;
            self.prefetch()?;
            self.idle(2);
            self.next()
        })();
        if r.is_err() {
            self.cpu.halted = true;
            return Step::Halted;
        }
        Step::Exception(3)
    }

    fn illegal(&mut self, vector: u8) -> R<Step> {
        self.idle(4);
        let pc = self.pc;
        self.exception(vector, pc)
    }

    fn privileged(&mut self) -> Option<R<Step>> {
        if self.s() { None } else { Some(self.illegal(8)) }
    }

    // ---- decode

    fn execute(&mut self) -> R<Step> {
        let op = self.cpu.regs.prefetch[0];
        match op >> 12 {
            0x0 => self.line0(op),
            0x1 => self.mov(op, W::B),
            0x2 => self.mov(op, W::L),
            0x3 => self.mov(op, W::W),
            0x4 => self.line4(op),
            0x5 => self.line5(op),
            0x6 => self.branch(op),
            0x7 => self.moveq(op),
            0x8 => self.line8(op),
            0x9 => self.addsub(op, false),
            0xA => self.illegal(10),
            0xB => self.lineb(op),
            0xC => self.linec(op),
            0xD => self.addsub(op, true),
            0xE => self.shift(op),
            _ => self.illegal(11),
        }
    }

    fn ok(&mut self) -> R<Step> {
        Ok(Step::Instruction)
    }

    // ---- line 0: immediates, bit operations, MOVEP

    fn line0(&mut self, op: u16) -> R<Step> {
        let mode = (op >> 3) & 7;
        let reg = op & 7;
        if op & 0x100 != 0 {
            if mode == 1 {
                return self.movep(op);
            }
            return self.bit(op, false);
        }
        let kind = (op >> 9) & 7;
        if kind == 4 {
            return self.bit(op, true);
        }
        if op & 0xFF == 0x3C || op & 0xFF == 0x7C {
            return self.imm_sr(op);
        }
        let Some(w) = W::from_bits(op >> 6) else { return self.illegal(4) };
        if kind == 7 || mode == 1 || (mode == 7 && reg > 1) {
            return self.illegal(4);
        }
        self.pcv = self.pcv.wrapping_add(if w == W::L { 4 } else { 2 });
        let imm = match w {
            W::L => self.ext32()?,
            _ => self.ext16()? as u32 & w.mask(),
        };
        let ea = self.ea(mode, reg, w, Use::Read)?;
        let d = self.get(ea, w)?;
        let r = match kind {
            0 => d | imm,
            1 => d & imm,
            2 => self.sub(imm, d, false, w, false, false),
            3 => self.add(imm, d, false, w, false),
            5 => d ^ imm,
            _ => {
                self.sub(imm, d, false, w, false, true);
                self.next()?;
                if w == W::L && !Self::is_mem(ea) {
                    self.idle(2);
                }
                return self.ok();
            }
        };
        if matches!(kind, 0 | 1 | 5) {
            self.logic(r, w);
        }
        self.next()?;
        if w == W::L && !Self::is_mem(ea) {
            self.idle(4);
        }
        self.put_rmw(ea, w, r)?;
        self.ok()
    }

    fn imm_sr(&mut self, op: u16) -> R<Step> {
        let word = op & 0x40 != 0;
        if word && let Some(r) = self.privileged() {
            return r;
        }
        let imm = self.ext16()?;
        let kind = (op >> 9) & 7;
        let cur = self.sr();
        let v = match kind {
            0 => cur | imm,
            1 => cur & imm,
            5 => cur ^ imm,
            _ => return self.illegal(4),
        };
        if word {
            self.set_sr(v);
        } else {
            self.set_ccr(v);
        }
        self.idle(8);
        self.refill()?;
        self.ok()
    }

    fn bit(&mut self, op: u16, immediate: bool) -> R<Step> {
        let mode = (op >> 3) & 7;
        let reg = op & 7;
        let kind = (op >> 6) & 3;
        if mode == 1 || (mode == 7 && reg > if kind == 0 { if immediate { 3 } else { 4 } } else { 1 }) {
            return self.illegal(4);
        }
        let number = if immediate { self.ext16()? as u32 } else { self.d(((op >> 9) & 7) as usize) };
        if mode == 0 {
            let r = reg as usize;
            let bit = number & 31;
            let v = self.d(r);
            self.put(Z, v & (1 << bit) == 0);
            let nv = match kind {
                1 => v ^ (1 << bit),
                2 => v & !(1 << bit),
                3 => v | (1 << bit),
                _ => v,
            };
            self.cpu.regs.d[r] = nv;
            self.next()?;
            match kind {
                0 => self.idle(2),
                2 => self.idle(if bit < 16 { 4 } else { 6 }),
                _ => self.idle(if bit < 16 { 2 } else { 4 }),
            }
            return self.ok();
        }
        let ea = self.ea(mode, reg, W::B, Use::Read)?;
        let v = self.get(ea, W::B)?;
        let bit = number & 7;
        self.put(Z, v & (1 << bit) == 0);
        self.next()?;
        if matches!(ea, Ea::I(_)) {
            self.idle(2);
        }
        if kind != 0 {
            let nv = match kind {
                1 => v ^ (1 << bit),
                2 => v & !(1 << bit),
                _ => v | (1 << bit),
            };
            self.put_rmw(ea, W::B, nv)?;
        }
        self.ok()
    }

    fn movep(&mut self, op: u16) -> R<Step> {
        let dn = ((op >> 9) & 7) as usize;
        let an = (op & 7) as usize;
        let disp = self.ext16()? as i16 as i32 as u32;
        let a = self.a(an).wrapping_add(disp);
        let long = op & 0x40 != 0;
        let n = if long { 4 } else { 2 };
        let fc = self.fc_data();
        if op & 0x80 != 0 {
            let v = self.d(dn);
            for i in 0..n {
                let byte = (v >> (8 * (n - 1 - i))) as u16 & 0xFF;
                self.write16(a.wrapping_add(2 * i), Size::Byte, byte, fc)?;
            }
        } else {
            let mut v = 0u32;
            for i in 0..n {
                v = v << 8 | (self.read16(a.wrapping_add(2 * i), Size::Byte, fc)? as u32 & 0xFF);
            }
            self.set_d(dn, if long { W::L } else { W::W }, v);
        }
        self.next()?;
        self.ok()
    }

    // ---- lines 1-3: MOVE and MOVEA

    fn mov(&mut self, op: u16, w: W) -> R<Step> {
        let (sm, sr) = ((op >> 3) & 7, op & 7);
        let (dm, dr) = ((op >> 6) & 7, (op >> 9) & 7);
        if (sm == 7 && sr > 4) || (sm == 1 && w == W::B) || (dm == 7 && dr > 1) || (dm == 1 && w == W::B) {
            return self.illegal(4);
        }
        let src = self.ea(sm, sr, w, Use::Read)?;
        let v = self.get(src, w)?;
        let src_mem = Self::is_mem(src);
        self.pcv = self.au;
        match dm {
            0 => {
                self.logic(v, w);
                self.set_d(dr as usize, w, v);
                self.next()?;
            }
            1 => {
                self.set_a(dr as usize, sext(v, w));
                self.next()?;
            }
            4 => {
                let ea = self.ea(dm, dr, w, Use::MoveDest)?;
                self.logic(v, w);
                self.next()?;
                if w != W::L {
                    self.ir = self.cpu.regs.prefetch[0];
                }
                if let Ea::M(a, _) = ea {
                    self.write_low_first(a, w, v)?;
                }
                self.apply_post();
            }
            7 if dr == 1 && src_mem => {
                let hi = self.ext16()? as u32;
                let a = hi << 16 | self.irc() as u32;
                self.store_flags(v, w, dm, dr, src_mem);
                self.write(a, w, v)?;
                self.logic(v, w);
                self.prefetch()?;
                self.next()?;
            }
            _ => {
                let ea = self.ea(dm, dr, w, Use::MoveDest)?;
                self.store_flags(v, w, dm, dr, src_mem);
                if let Ea::M(a, _) = ea {
                    self.write(a, w, v)?;
                }
                self.logic(v, w);
                self.apply_post();
                self.next()?;
            }
        }
        self.ok()
    }

    /// The flags MOVE has set when its first store faults: all of them for a byte or word; for a long, what the
    /// suite records by destination and source (Beryl_M68k.md §4.2).
    fn store_flags(&mut self, v: u32, w: W, dm: u16, dr: u16, src_mem: bool) {
        if w != W::L {
            return self.logic(v, w);
        }
        match (dm, dr, src_mem) {
            (2 | 3, _, false) => {}
            (2 | 3, _, true) | (7, 1, true) => self.logic(v & 0xFFFF, W::W),
            (5 | 6, _, false) => self.nz(v, W::L),
            _ => self.logic(v, W::L),
        }
    }

    fn moveq(&mut self, op: u16) -> R<Step> {
        if op & 0x100 != 0 {
            return self.illegal(4);
        }
        let v = op as u8 as i8 as i32 as u32;
        self.cpu.regs.d[((op >> 9) & 7) as usize] = v;
        self.logic(v, W::L);
        self.next()?;
        self.ok()
    }

    // ---- line 4

    fn line4(&mut self, op: u16) -> R<Step> {
        let mode = (op >> 3) & 7;
        let reg = op & 7;
        if op & 0x100 != 0 {
            return match (op >> 6) & 3 {
                2 => self.chk(op),
                3 => self.lea(op),
                _ => self.illegal(4),
            };
        }
        match (op >> 8) & 0xF {
            0x0 | 0x2 | 0x4 | 0x6 => {
                let size = (op >> 6) & 3;
                let kind = (op >> 9) & 3;
                if size == 3 {
                    return match kind {
                        0 => self.move_from_sr(mode, reg),
                        1 => self.illegal(4),
                        2 => self.move_to_sr(mode, reg, false),
                        _ => self.move_to_sr(mode, reg, true),
                    };
                }
                self.unary(op, kind)
            }
            0x8 => match (op >> 6) & 3 {
                0 => self.nbcd(mode, reg),
                1 if mode == 0 => {
                    let r = reg as usize;
                    let v = self.d(r).rotate_left(16);
                    self.cpu.regs.d[r] = v;
                    self.logic(v, W::L);
                    self.next()?;
                    self.ok()
                }
                1 => self.pea(mode, reg),
                s if mode == 0 => {
                    let r = reg as usize;
                    let v = if s == 2 { sext(self.d(r), W::B) & 0xFFFF } else { sext(self.d(r), W::W) };
                    let w = if s == 2 { W::W } else { W::L };
                    self.set_d(r, w, v);
                    self.logic(v, w);
                    self.next()?;
                    self.ok()
                }
                s => self.movem(op, s == 3, false),
            },
            0xA => {
                if op == 0x4AFC {
                    return self.illegal(4);
                }
                match (op >> 6) & 3 {
                    3 => self.tas(mode, reg),
                    s => self.tst(W::from_bits(s).unwrap(), mode, reg),
                }
            }
            0xC => match (op >> 6) & 3 {
                2 | 3 => self.movem(op, op & 0x40 != 0, true),
                _ => self.illegal(4),
            },
            0xE => self.line4e(op),
            _ => self.illegal(4),
        }
    }

    fn unary(&mut self, op: u16, kind: u16) -> R<Step> {
        let mode = (op >> 3) & 7;
        let reg = op & 7;
        let w = W::from_bits(op >> 6).unwrap();
        if mode == 1 || (mode == 7 && reg > 1) {
            return self.illegal(4);
        }
        let ea = self.ea(mode, reg, w, Use::Read)?;
        let d = self.get(ea, w)?;
        let r = match kind {
            0 => {
                let x = self.flag(X);
                self.sub(d, 0, x, w, true, false)
            }
            1 => {
                self.put(N | V | C, false);
                self.put(Z, true);
                0
            }
            2 => self.sub(d, 0, false, w, false, false),
            _ => {
                let r = !d & w.mask();
                self.logic(r, w);
                r
            }
        };
        self.next()?;
        if w == W::L && !Self::is_mem(ea) {
            self.idle(2);
        }
        self.put_rmw(ea, w, r)?;
        self.ok()
    }

    fn move_from_sr(&mut self, mode: u16, reg: u16) -> R<Step> {
        if mode == 1 || (mode == 7 && reg > 1) {
            return self.illegal(4);
        }
        let ea = self.ea(mode, reg, W::W, Use::Read)?;
        if Self::is_mem(ea) {
            self.get(ea, W::W)?;
        }
        let v = self.sr() as u32;
        self.next()?;
        if !Self::is_mem(ea) {
            self.idle(2);
        }
        self.put_rmw(ea, W::W, v)?;
        self.ok()
    }

    fn move_to_sr(&mut self, mode: u16, reg: u16, sr: bool) -> R<Step> {
        if mode == 1 || (mode == 7 && reg > 4) {
            return self.illegal(4);
        }
        if sr && let Some(r) = self.privileged() {
            return r;
        }
        let ea = self.ea(mode, reg, W::W, Use::Read)?;
        let v = self.get(ea, W::W)? as u16;
        if sr {
            self.set_sr(v);
        } else {
            self.set_ccr(v);
        }
        self.idle(4);
        self.refill()?;
        self.ok()
    }

    fn nbcd(&mut self, mode: u16, reg: u16) -> R<Step> {
        if mode == 1 || (mode == 7 && reg > 1) {
            return self.illegal(4);
        }
        let ea = self.ea(mode, reg, W::B, Use::Read)?;
        let d = self.get(ea, W::B)?;
        let x = self.flag(X);
        let r = self.bcd_sub(d, 0, x);
        self.next()?;
        if !Self::is_mem(ea) {
            self.idle(2);
        }
        self.put_rmw(ea, W::B, r)?;
        self.ok()
    }

    fn pea(&mut self, mode: u16, reg: u16) -> R<Step> {
        if !matches!(mode, 2 | 5 | 6) && !(mode == 7 && reg <= 3) {
            return self.illegal(4);
        }
        let ea = self.ea(mode, reg, W::L, Use::Read)?;
        let Ea::M(a, _) = ea else { unreachable!() };
        let absolute = mode == 7 && reg < 2;
        if mode == 6 || (mode == 7 && reg == 3) {
            self.idle(2);
        }
        if !absolute {
            self.next()?;
        }
        self.push32(a)?;
        if absolute {
            self.next()?;
        }
        self.ok()
    }

    fn tst(&mut self, w: W, mode: u16, reg: u16) -> R<Step> {
        if mode == 1 || (mode == 7 && reg > 1) {
            return self.illegal(4);
        }
        let ea = self.ea(mode, reg, w, Use::Read)?;
        let v = self.get(ea, w)?;
        self.logic(v, w);
        self.next()?;
        self.ok()
    }

    fn tas(&mut self, mode: u16, reg: u16) -> R<Step> {
        if mode == 1 || (mode == 7 && reg > 1) {
            return self.illegal(4);
        }
        let ea = self.ea(mode, reg, W::B, Use::Read)?;
        match ea {
            Ea::D(n) => {
                let v = self.d(n) & 0xFF;
                self.logic(v, W::B);
                self.set_d(n, W::B, v | 0x80);
            }
            Ea::M(a, _) => {
                let fc = self.fc_data();
                let a = a & 0xFF_FFFF;
                let v = self.b.read(Access { address: a, size: Size::Byte, function: fc, locked: true }) as u32 & 0xFF;
                self.logic(v, W::B);
                self.idle(2);
                self.b.write(Access { address: a, size: Size::Byte, function: fc, locked: true }, (v | 0x80) as u16);
                self.o.wrote(self.cpu.space, a, (v | 0x80) as u8, self.pc);
            }
            _ => {}
        }
        self.next()?;
        self.ok()
    }

    fn chk(&mut self, op: u16) -> R<Step> {
        let mode = (op >> 3) & 7;
        let reg = op & 7;
        if mode == 1 || (mode == 7 && reg > 4) {
            return self.illegal(4);
        }
        let ea = self.ea(mode, reg, W::W, Use::Read)?;
        let bound = self.get(ea, W::W)? as u16 as i16;
        let dn = self.d(((op >> 9) & 7) as usize) as u16 as i16;
        self.put(Z, dn == 0);
        self.put(V | C, false);
        self.put(N, dn < 0);
        if dn < 0 || dn > bound {
            let wrapped = (bound as u16).wrapping_sub(dn as u16) & 0x8000 != 0;
            self.idle(if dn < 0 && !wrapped { 10 } else { 8 });
            let pc = self.au.wrapping_sub(2);
            return self.exception(6, pc);
        }
        self.idle(6);
        self.next()?;
        self.ok()
    }

    fn lea(&mut self, op: u16) -> R<Step> {
        let mode = (op >> 3) & 7;
        let reg = op & 7;
        if !matches!(mode, 2 | 5 | 6) && !(mode == 7 && reg <= 3) {
            return self.illegal(4);
        }
        let ea = self.ea(mode, reg, W::L, Use::Read)?;
        let Ea::M(a, _) = ea else { unreachable!() };
        if mode == 6 || (mode == 7 && reg == 3) {
            self.idle(2);
        }
        self.set_a(((op >> 9) & 7) as usize, a);
        self.next()?;
        self.ok()
    }

    /// MOVEM: the mask word, then the address, then each register in order (Beryl_M68k.md §4).
    fn movem(&mut self, op: u16, long: bool, to_regs: bool) -> R<Step> {
        let mode = (op >> 3) & 7;
        let reg = op & 7;
        let w = if long { W::L } else { W::W };
        let ok = if to_regs { matches!(mode, 2 | 3 | 5 | 6) || (mode == 7 && reg <= 3) } else { matches!(mode, 2 | 4 | 5 | 6) || (mode == 7 && reg <= 1) };
        if !ok {
            return self.illegal(4);
        }
        let mask = self.ext16()?;
        let an = reg as usize;
        let mut a = match mode {
            3 | 4 => self.a(an),
            _ => match self.ea(mode, reg, w, Use::MoveDest)? {
                Ea::M(a, _) => a,
                _ => unreachable!(),
            },
        };
        let program = mode == 7 && reg >= 2;
        self.pcv = self.au;
        if to_regs {
            for i in 0..16 {
                if mask & (1 << i) != 0 {
                    let v = sext(self.read(a, w, program)?, w);
                    if i < 8 {
                        self.cpu.regs.d[i] = v;
                    } else {
                        self.cpu.regs.a[i - 8] = v;
                    }
                    a = a.wrapping_add(w.bytes());
                }
            }
            let fc = if program { self.fc_program() } else { self.fc_data() };
            self.read16(a, Size::Word, fc)?;
            if mode == 3 {
                self.set_a(an, a);
            }
        } else if mode == 4 {
            let start = self.a(an);
            for i in 0..16 {
                if mask & (1 << i) != 0 {
                    let r = 15 - i;
                    let v = if r < 8 { self.d(r) } else if r - 8 == an { start } else { self.a(r - 8) };
                    a = a.wrapping_sub(w.bytes());
                    self.write_low_first(a, w, v)?;
                }
            }
            self.set_a(an, a);
        } else {
            for i in 0..16 {
                if mask & (1 << i) != 0 {
                    let v = if i < 8 { self.d(i) } else { self.a(i - 8) };
                    self.write(a, w, v)?;
                    a = a.wrapping_add(w.bytes());
                }
            }
        }
        self.next()?;
        self.ok()
    }

    fn line4e(&mut self, op: u16) -> R<Step> {
        let mode = (op >> 3) & 7;
        let reg = op & 7;
        match (op >> 6) & 3 {
            1 => match (op >> 3) & 7 {
                0 | 1 => {
                    self.idle(4);
                    let pc = self.au.wrapping_sub(2);
                    self.exception(32 + (op & 15) as u8, pc)
                }
                2 => {
                    let r = reg as usize;
                    let d = self.ext16()? as i16 as i32 as u32;
                    let v = self.a(r);
                    let sp = self.a(7).wrapping_sub(4);
                    self.set_a(7, sp);
                    self.write(sp, W::L, v)?;
                    self.set_a(r, sp);
                    self.set_a(7, self.a(7).wrapping_add(d));
                    self.next()?;
                    self.ok()
                }
                3 => {
                    self.pcv = self.au;
                    let r = reg as usize;
                    let sp = self.a(r);
                    let v = self.read(sp, W::L, false)?;
                    self.set_a(7, sp.wrapping_add(4));
                    self.set_a(r, v);
                    self.next()?;
                    self.ok()
                }
                4 | 5 => {
                    if let Some(r) = self.privileged() {
                        return r;
                    }
                    let r = reg as usize;
                    if mode == 4 {
                        self.cpu.regs.other_sp = self.a(r);
                    } else {
                        let usp = self.cpu.regs.other_sp;
                        self.set_a(r, usp);
                    }
                    self.next()?;
                    self.ok()
                }
                6 => self.line4e_misc(op),
                _ => self.illegal(4),
            },
            2 | 3 => self.jump(op, op & 0x40 == 0),
            _ => self.illegal(4),
        }
    }

    fn line4e_misc(&mut self, op: u16) -> R<Step> {
        match op & 7 {
            0 => {
                if let Some(r) = self.privileged() {
                    return r;
                }
                self.idle(128);
                self.b.reset_devices();
                self.next()?;
                self.ok()
            }
            1 => {
                self.next()?;
                self.ok()
            }
            2 => {
                if let Some(r) = self.privileged() {
                    return r;
                }
                let v = self.irc();
                self.set_sr(v);
                self.idle(4);
                self.cpu.stopped = true;
                Ok(Step::Stopped)
            }
            3 => {
                if let Some(r) = self.privileged() {
                    return r;
                }
                let sp = self.a(7);
                let sr = self.read(sp, W::W, false)? as u16;
                let pc = self.read(sp.wrapping_add(2), W::L, false)?;
                self.set_a(7, sp.wrapping_add(6));
                self.set_sr(sr);
                self.o.returned();
                self.fill(pc)?;
                self.ok()
            }
            5 => {
                let pc = self.pop32()?;
                self.o.returned();
                self.fill(pc)?;
                self.ok()
            }
            6 => {
                if self.flag(V) {
                    let old = self.sr();
                    self.set_sr((old | 0x2000) & !0x8000);
                    self.next()?;
                    let pc = self.au.wrapping_sub(4);
                    return self.exception_from(old, 7, pc);
                }
                self.next()?;
                self.ok()
            }
            7 => {
                let sp = self.a(7);
                let ccr = self.read(sp, W::W, false)? as u16;
                let pc = self.read(sp.wrapping_add(2), W::L, false)?;
                self.set_a(7, sp.wrapping_add(6));
                self.set_ccr(ccr);
                self.o.returned();
                self.fill(pc)?;
                self.ok()
            }
            _ => self.illegal(4),
        }
    }

    /// JMP and JSR: the target's displacement taken from IRC without a fetch, the queue filled from the target, and
    /// JSR's return address stacked between the two fetches.
    fn jump(&mut self, op: u16, jsr: bool) -> R<Step> {
        let mode = (op >> 3) & 7;
        let reg = op & 7;
        let r = reg as usize;
        let (target, ret) = match (mode, reg) {
            (2, _) => (self.a(r), self.au.wrapping_sub(2)),
            (5, _) => {
                self.idle(2);
                (self.a(r).wrapping_add(self.irc() as i16 as i32 as u32), self.au)
            }
            (6, _) => {
                self.idle(6);
                let t = self.brief(self.a(r));
                (t, self.au)
            }
            (7, 0) => {
                self.idle(2);
                (self.irc() as i16 as i32 as u32, self.au)
            }
            (7, 1) => {
                let hi = self.ext16()? as u32;
                (hi << 16 | self.irc() as u32, self.au)
            }
            (7, 2) => {
                self.idle(2);
                (self.au.wrapping_sub(2).wrapping_add(self.irc() as i16 as i32 as u32), self.au)
            }
            (7, 3) => {
                self.idle(6);
                let t = self.brief(self.au.wrapping_sub(2));
                (t, self.au)
            }
            _ => return self.illegal(4),
        };
        if jsr {
            self.pcv = ret;
        }
        self.au = target;
        self.prefetch()?;
        if jsr {
            self.push32(ret)?;
            self.o.called(self.pc, target, kind::CALL);
        }
        self.next()?;
        self.ok()
    }

    /// An indexed address from the brief word in IRC, which is not fetched past.
    fn brief(&self, base: u32) -> u32 {
        let ext = self.irc();
        let r = (ext >> 12) as usize & 7;
        let xn = if ext & 0x8000 != 0 { self.a(r) } else { self.d(r) };
        let xn = if ext & 0x800 != 0 { xn } else { xn as u16 as i16 as i32 as u32 };
        base.wrapping_add(xn).wrapping_add(ext as u8 as i8 as i32 as u32)
    }

    // ---- line 5

    fn line5(&mut self, op: u16) -> R<Step> {
        let mode = (op >> 3) & 7;
        let reg = op & 7;
        if (op >> 6) & 3 == 3 {
            let c = (op >> 8) & 15;
            if mode == 1 {
                return self.dbcc(c, reg as usize);
            }
            if mode == 7 && reg > 1 {
                return self.illegal(4);
            }
            let ea = self.ea(mode, reg, W::B, Use::Read)?;
            let t = self.cond(c);
            if Self::is_mem(ea) {
                self.get(ea, W::B)?;
            }
            self.next()?;
            if t && !Self::is_mem(ea) {
                self.idle(2);
            }
            self.put_rmw(ea, W::B, if t { 0xFF } else { 0 })?;
            return self.ok();
        }
        let w = W::from_bits(op >> 6).unwrap();
        if (mode == 1 && w == W::B) || (mode == 7 && reg > 1) {
            return self.illegal(4);
        }
        let q = match (op >> 9) & 7 {
            0 => 8,
            n => n as u32,
        };
        let sub = op & 0x100 != 0;
        let ea = self.ea(mode, reg, w, Use::Read)?;
        if let Ea::A(n) = ea {
            let a = self.a(n);
            self.set_a(n, if sub { a.wrapping_sub(q) } else { a.wrapping_add(q) });
            self.next()?;
            self.idle(4);
            return self.ok();
        }
        let d = self.get(ea, w)?;
        let r = if sub { self.sub(q, d, false, w, false, false) } else { self.add(q, d, false, w, false) };
        self.next()?;
        if w == W::L && !Self::is_mem(ea) {
            self.idle(4);
        }
        self.put_rmw(ea, w, r)?;
        self.ok()
    }

    fn dbcc(&mut self, c: u16, r: usize) -> R<Step> {
        if self.cond(c) {
            self.idle(4);
            self.prefetch()?;
            self.next()?;
            return self.ok();
        }
        let count = (self.d(r) as u16).wrapping_sub(1);
        self.idle(2);
        let target = self.au.wrapping_sub(2).wrapping_add(self.irc() as i16 as i32 as u32);
        if count != 0xFFFF {
            self.pcv = self.au;
            self.fill(target)?;
            self.set_d(r, W::W, count as u32);
            return self.ok();
        }
        self.set_d(r, W::W, count as u32);
        let fc = self.fc_program();
        self.read16(target, Size::Word, fc)?;
        self.prefetch()?;
        self.next()?;
        self.ok()
    }

    // ---- line 6

    fn branch(&mut self, op: u16) -> R<Step> {
        let c = (op >> 8) & 15;
        let d8 = op as u8;
        let base = self.au.wrapping_sub(2);
        let word = d8 == 0;
        let disp = if word { self.irc() as i16 as i32 as u32 } else { d8 as i8 as i32 as u32 };
        let target = base.wrapping_add(disp);
        if c == 1 {
            let ret = if word { self.au } else { self.au.wrapping_sub(2) };
            self.idle(2);
            self.push32(ret)?;
            self.o.called(self.pc, target, kind::CALL);
            self.pcv = target;
            self.fill(target)?;
            return self.ok();
        }
        if self.cond(c) {
            self.idle(2);
            self.fill(target)?;
        } else {
            self.idle(4);
            if word {
                self.prefetch()?;
            }
            self.next()?;
        }
        self.ok()
    }

    // ---- lines 8 and C: OR, AND, DIV, MUL, SBCD, ABCD, EXG

    fn line8(&mut self, op: u16) -> R<Step> {
        match (op >> 6) & 7 {
            3 => self.div(op, false),
            7 => self.div(op, true),
            4 if (op >> 3) & 6 == 0 => self.bcd(op, false),
            5 | 6 if (op >> 3) & 6 == 0 => self.illegal(4),
            _ => self.logical(op, true),
        }
    }

    fn linec(&mut self, op: u16) -> R<Step> {
        match (op >> 6) & 7 {
            3 => self.mul(op, false),
            7 => self.mul(op, true),
            4 if (op >> 3) & 6 == 0 => self.bcd(op, true),
            5 if (op >> 3) & 7 <= 1 => self.exg(op),
            6 if (op >> 3) & 7 == 1 => self.exg(op),
            5 | 6 if (op >> 3) & 6 == 0 => self.illegal(4),
            _ => self.logical(op, false),
        }
    }

    fn logical(&mut self, op: u16, or: bool) -> R<Step> {
        let mode = (op >> 3) & 7;
        let reg = op & 7;
        let dn = ((op >> 9) & 7) as usize;
        let w = W::from_bits(op >> 6).unwrap();
        let to_ea = op & 0x100 != 0;
        if mode == 1 || (mode == 7 && reg > if to_ea { 1 } else { 4 }) || (to_ea && mode == 0) {
            return self.illegal(4);
        }
        let ea = self.ea(mode, reg, w, Use::Read)?;
        let e = self.get(ea, w)?;
        let r = if or { e | self.d(dn) } else { e & self.d(dn) } & w.mask();
        self.logic(r, w);
        self.next()?;
        if to_ea {
            self.put_rmw(ea, w, r)?;
        } else {
            if w == W::L {
                self.idle(if Self::is_mem(ea) { 2 } else { 4 });
            }
            self.set_d(dn, w, r);
        }
        self.ok()
    }

    /// ABCD's sum with its decimal correction and flags, as the BCD verifier's reference model states them
    /// (Beryl_M68k.md §4.3): the binary carries of each nibble and the decimal carries select the correction.
    fn bcd_add(&mut self, s: u32, d: u32, x: bool) -> u32 {
        let ss = (s + d + x as u32) & 0xFF;
        let bc = ((s & d) | (!ss & s) | (!ss & d)) & 0x88;
        let dc = (((ss + 0x66) ^ ss) & 0x110) >> 1;
        let corf = (bc | dc) - ((bc | dc) >> 2);
        let rr = (ss + corf) & 0xFF;
        let carry = (bc | (ss & !rr)) & 0x80 != 0;
        self.put(C | X, carry);
        self.put(V, !ss & rr & 0x80 != 0);
        self.put(N, rr & 0x80 != 0);
        if rr != 0 {
            self.put(Z, false);
        }
        rr
    }

    /// SBCD's difference `d - s - x` and NBCD's (`d` zero), by the same model.
    fn bcd_sub(&mut self, s: u32, d: u32, x: bool) -> u32 {
        let dd = d.wrapping_sub(s).wrapping_sub(x as u32) & 0xFF;
        let bc = ((!d & s) | (dd & !d) | (dd & s)) & 0x88;
        let corf = bc - (bc >> 2);
        let rr = dd.wrapping_sub(corf) & 0xFF;
        let carry = (bc | (!dd & rr)) & 0x80 != 0;
        self.put(C | X, carry);
        self.put(V, dd & !rr & 0x80 != 0);
        self.put(N, rr & 0x80 != 0);
        if rr != 0 {
            self.put(Z, false);
        }
        rr
    }

    fn bcd(&mut self, op: u16, add: bool) -> R<Step> {
        let rx = ((op >> 9) & 7) as usize;
        let ry = (op & 7) as usize;
        if op & 8 == 0 {
            let (s, d) = (self.d(ry) & 0xFF, self.d(rx) & 0xFF);
            let x = self.flag(X);
            let r = if add { self.bcd_add(s, d, x) } else { self.bcd_sub(s, d, x) };
            self.set_d(rx, W::B, r);
            self.next()?;
            self.idle(2);
            return self.ok();
        }
        self.idle(2);
        let sa = self.a(ry).wrapping_sub(if ry == 7 { 2 } else { 1 });
        self.set_a(ry, sa);
        let s = self.read(sa, W::B, false)?;
        let da = self.a(rx).wrapping_sub(if rx == 7 { 2 } else { 1 });
        self.set_a(rx, da);
        let d = self.read(da, W::B, false)?;
        let x = self.flag(X);
        let r = if add { self.bcd_add(s, d, x) } else { self.bcd_sub(s, d, x) };
        self.next()?;
        self.write(da, W::B, r)?;
        self.ok()
    }

    fn exg(&mut self, op: u16) -> R<Step> {
        let rx = ((op >> 9) & 7) as usize;
        let ry = (op & 7) as usize;
        let regs = &mut self.cpu.regs;
        match (op >> 3) & 0x1F {
            0x08 => regs.d.swap(rx, ry),
            0x09 => regs.a.swap(rx, ry),
            _ => std::mem::swap(&mut regs.d[rx], &mut regs.a[ry]),
        }
        self.next()?;
        self.idle(2);
        self.ok()
    }

    fn mul(&mut self, op: u16, signed: bool) -> R<Step> {
        let mode = (op >> 3) & 7;
        let reg = op & 7;
        if mode == 1 || (mode == 7 && reg > 4) {
            return self.illegal(4);
        }
        let dn = ((op >> 9) & 7) as usize;
        let ea = self.ea(mode, reg, W::W, Use::Read)?;
        let s = self.get(ea, W::W)? as u16;
        let d = self.d(dn) as u16;
        let (r, m) = if signed {
            let r = (s as i16 as i32).wrapping_mul(d as i16 as i32) as u32;
            let pattern = (s as u32) << 1;
            let m = (0..16).filter(|i| ((pattern >> i) & 3) == 1 || ((pattern >> i) & 3) == 2).count() as u32;
            (r, m)
        } else {
            (s as u32 * d as u32, s.count_ones())
        };
        self.cpu.regs.d[dn] = r;
        self.logic(r, W::L);
        self.next()?;
        self.idle(34 + 2 * m);
        self.ok()
    }

    fn div(&mut self, op: u16, signed: bool) -> R<Step> {
        let mode = (op >> 3) & 7;
        let reg = op & 7;
        if mode == 1 || (mode == 7 && reg > 4) {
            return self.illegal(4);
        }
        let dn = ((op >> 9) & 7) as usize;
        let ea = self.ea(mode, reg, W::W, Use::Read)?;
        let s = self.get(ea, W::W)? as u16;
        let d = self.d(dn);
        if s == 0 {
            self.put(C, false);
            self.idle(8);
            let pc = self.au.wrapping_sub(2);
            return self.exception(5, pc);
        }
        if !signed {
            let divisor = s as u32;
            if (d >> 16) >= divisor {
                self.overflow();
                self.idle(6);
                self.next()?;
                return self.ok();
            }
            let q = d / divisor;
            let rem = d % divisor;
            self.cpu.regs.d[dn] = rem << 16 | q;
            self.nz(q, W::W);
            self.put(V | C, false);
            self.idle(Self::divu_clocks(d, divisor) - 4);
            self.next()?;
            return self.ok();
        }
        let divisor = s as i16 as i32;
        let dividend = d as i32;
        let (ad, av) = (dividend.unsigned_abs(), divisor.unsigned_abs());
        let mut units = 6 + (dividend < 0) as u32;
        if (ad >> 16) >= av {
            self.overflow();
            self.idle((units + 2) * 2 - 4);
            self.next()?;
            return self.ok();
        }
        units += 55;
        if divisor >= 0 {
            if dividend >= 0 { units -= 1 } else { units += 1 }
        }
        let mut aq = ad / av;
        for _ in 0..15 {
            if aq & 0x8000 == 0 {
                units += 1;
            }
            aq <<= 1;
        }
        let q = dividend.wrapping_div(divisor);
        let rem = dividend.wrapping_rem(divisor);
        if q > 32767 || q < -32768 {
            self.overflow();
        } else {
            self.cpu.regs.d[dn] = (rem as u32) << 16 | (q as u32 & 0xFFFF);
            self.nz(q as u32, W::W);
            self.put(V | C, false);
        }
        self.idle(units * 2 - 4);
        self.next()?;
        self.ok()
    }

    /// A quotient that does not fit: V and N set, Z and C clear, the register unchanged.
    fn overflow(&mut self) {
        self.put(V | N, true);
        self.put(Z | C, false);
    }

    /// DIVU's clocks, the prefetch included, from Yacht.txt's flowchart: 76 at best, two more for each quotient bit
    /// found with the dividend's top bit clear, four more for each such bit not found.
    fn divu_clocks(dividend: u32, divisor: u32) -> u32 {
        let mut clocks = 76;
        let hd = divisor << 16;
        let mut dv = dividend;
        for _ in 0..15 {
            let top = dv & 0x8000_0000 != 0;
            dv <<= 1;
            if top {
                dv = dv.wrapping_sub(hd);
            } else if dv >= hd {
                dv = dv.wrapping_sub(hd);
                clocks += 2;
            } else {
                clocks += 4;
            }
        }
        clocks
    }

    // ---- lines 9 and D: SUB, ADD, SUBA, ADDA, SUBX, ADDX

    fn addsub(&mut self, op: u16, add: bool) -> R<Step> {
        let mode = (op >> 3) & 7;
        let reg = op & 7;
        let rn = ((op >> 9) & 7) as usize;
        let opmode = (op >> 6) & 7;
        if opmode == 3 || opmode == 7 {
            let w = if opmode == 7 { W::L } else { W::W };
            if mode == 7 && reg > 4 {
                return self.illegal(4);
            }
            let ea = self.ea(mode, reg, w, Use::Read)?;
            let s = sext(self.get(ea, w)?, w);
            let a = self.a(rn);
            self.set_a(rn, if add { a.wrapping_add(s) } else { a.wrapping_sub(s) });
            self.next()?;
            self.idle(if w == W::L && Self::is_mem(ea) { 2 } else { 4 });
            return self.ok();
        }
        let w = W::from_bits(opmode).unwrap();
        let to_ea = op & 0x100 != 0;
        if to_ea && mode <= 1 {
            return self.addsubx(op, add, w);
        }
        if (mode == 1 && w == W::B) || (mode == 7 && reg > if to_ea { 1 } else { 4 }) {
            return self.illegal(4);
        }
        let ea = self.ea(mode, reg, w, Use::Read)?;
        let e = self.get(ea, w)?;
        let dn = self.d(rn);
        if to_ea {
            let r = if add { self.add(dn, e, false, w, false) } else { self.sub(dn, e, false, w, false, false) };
            self.next()?;
            self.put_rmw(ea, w, r)?;
        } else {
            let r = if add { self.add(e, dn, false, w, false) } else { self.sub(e, dn, false, w, false, false) };
            self.next()?;
            if w == W::L {
                self.idle(if Self::is_mem(ea) { 2 } else { 4 });
            }
            self.set_d(rn, w, r);
        }
        self.ok()
    }

    fn addsubx(&mut self, op: u16, add: bool, w: W) -> R<Step> {
        let rx = ((op >> 9) & 7) as usize;
        let ry = (op & 7) as usize;
        let x = self.flag(X);
        if op & 8 == 0 {
            let (s, d) = (self.d(ry), self.d(rx));
            let r = if add { self.add(s, d, x, w, true) } else { self.sub(s, d, x, w, true, false) };
            self.set_d(rx, w, r);
            self.next()?;
            if w == W::L {
                self.idle(4);
            }
            return self.ok();
        }
        self.idle(2);
        self.pcv = self.au;
        let step = |r: usize| if w == W::B && r == 7 { 2 } else { w.bytes() };
        let sa = self.a(ry).wrapping_sub(step(ry));
        let fc = self.fc_data();
        let s = if w == W::L {
            let lo = self.read16(sa.wrapping_add(2), Size::Word, fc)? as u32;
            let hi = self.read16(sa, Size::Word, fc)? as u32;
            self.set_a(ry, sa);
            hi << 16 | lo
        } else {
            self.set_a(ry, sa);
            self.read(sa, w, false)?
        };
        let da = self.a(rx).wrapping_sub(step(rx));
        let d = if w == W::L {
            let lo = self.read16(da.wrapping_add(2), Size::Word, fc)? as u32;
            let hi = self.read16(da, Size::Word, fc)? as u32;
            self.set_a(rx, da);
            hi << 16 | lo
        } else {
            self.set_a(rx, da);
            self.read(da, w, false)?
        };
        let r = if add { self.add(s, d, x, w, true) } else { self.sub(s, d, x, w, true, false) };
        if w == W::L {
            self.write16(da.wrapping_add(2), Size::Word, r as u16, fc)?;
            self.next()?;
            self.write16(da, Size::Word, (r >> 16) as u16, fc)?;
        } else {
            self.next()?;
            self.write(da, w, r)?;
        }
        self.ok()
    }

    // ---- line B: CMP, CMPA, CMPM, EOR

    fn lineb(&mut self, op: u16) -> R<Step> {
        let mode = (op >> 3) & 7;
        let reg = op & 7;
        let rn = ((op >> 9) & 7) as usize;
        let opmode = (op >> 6) & 7;
        if opmode == 3 || opmode == 7 {
            let w = if opmode == 7 { W::L } else { W::W };
            if mode == 7 && reg > 4 {
                return self.illegal(4);
            }
            let ea = self.ea(mode, reg, w, Use::Read)?;
            let s = sext(self.get(ea, w)?, w);
            let a = self.a(rn);
            self.sub(s, a, false, W::L, false, true);
            self.next()?;
            self.idle(2);
            return self.ok();
        }
        let w = W::from_bits(opmode).unwrap();
        if op & 0x100 == 0 {
            if (mode == 1 && w == W::B) || (mode == 7 && reg > 4) {
                return self.illegal(4);
            }
            let ea = self.ea(mode, reg, w, Use::Read)?;
            let s = self.get(ea, w)?;
            let d = self.d(rn);
            self.sub(s, d, false, w, false, true);
            self.next()?;
            if w == W::L {
                self.idle(2);
            }
            return self.ok();
        }
        if mode == 1 {
            self.pcv = self.au;
            let ay = (op & 7) as usize;
            let s = if w == W::L {
                let a = self.a(ay);
                self.set_a(ay, a.wrapping_add(2));
                let v = self.read(a, W::L, false)?;
                self.set_a(ay, a.wrapping_add(4));
                v
            } else {
                self.postinc_read(ay, w, false)?
            };
            let d = self.postinc_read(rn, w, true)?;
            self.sub(s, d, false, w, false, true);
            self.next()?;
            return self.ok();
        }
        if mode == 7 && reg > 1 {
            return self.illegal(4);
        }
        let ea = self.ea(mode, reg, w, Use::Read)?;
        let e = self.get(ea, w)?;
        let r = (e ^ self.d(rn)) & w.mask();
        self.logic(r, w);
        self.next()?;
        if w == W::L && !Self::is_mem(ea) {
            self.idle(4);
        }
        self.put_rmw(ea, w, r)?;
        self.ok()
    }

    /// A postincrement read, the register advanced before the read or, `late`, after it.
    fn postinc_read(&mut self, r: usize, w: W, late: bool) -> R<u32> {
        let a = self.a(r);
        let step = if w == W::B && r == 7 { 2 } else { w.bytes() };
        if !late {
            self.set_a(r, a.wrapping_add(step));
        }
        let v = self.read(a, w, false)?;
        if late {
            self.set_a(r, a.wrapping_add(step));
        }
        Ok(v)
    }

    // ---- line E: shifts and rotates

    fn shift(&mut self, op: u16) -> R<Step> {
        if (op >> 6) & 3 == 3 {
            let mode = (op >> 3) & 7;
            let reg = op & 7;
            if op & 0x800 != 0 || mode <= 1 || (mode == 7 && reg > 1) {
                return self.illegal(4);
            }
            let ea = self.ea(mode, reg, W::W, Use::Read)?;
            let v = self.get(ea, W::W)?;
            let r = self.shift_op((op >> 9) & 3, op & 0x100 != 0, v, 1, W::W);
            self.next()?;
            self.put_rmw(ea, W::W, r)?;
            return self.ok();
        }
        let w = W::from_bits(op >> 6).unwrap();
        let r = (op & 7) as usize;
        let field = ((op >> 9) & 7) as u32;
        let count = if op & 0x20 != 0 { self.d(field as usize) & 63 } else if field == 0 { 8 } else { field };
        let v = self.d(r) & w.mask();
        let res = self.shift_op((op >> 3) & 3, op & 0x100 != 0, v, count, w);
        self.set_d(r, w, res);
        self.next()?;
        self.idle(2 * count + if w == W::L { 4 } else { 2 });
        self.ok()
    }

    /// One shift or rotate by `count` (PRM §4): kind 0 arithmetic, 1 logical, 2 rotate through X, 3 rotate.
    fn shift_op(&mut self, kind: u16, left: bool, v: u32, count: u32, w: W) -> u32 {
        let m = w.mask();
        let msb = w.msb();
        let bits = w.bytes() * 8;
        let mut r = v & m;
        let mut c = false;
        let mut overflow = false;
        match kind {
            0 | 1 => {
                for _ in 0..count {
                    if left {
                        c = r & msb != 0;
                        let nr = (r << 1) & m;
                        if (nr ^ r) & msb != 0 {
                            overflow = true;
                        }
                        r = nr;
                    } else {
                        c = r & 1 != 0;
                        r = if kind == 0 { (r >> 1) | (r & msb) } else { r >> 1 };
                    }
                }
                if count > 0 {
                    self.put(X, c);
                }
                self.put(C, c && count > 0);
                self.put(V, kind == 0 && left && overflow);
            }
            2 => {
                let mut x = self.flag(X);
                for _ in 0..count {
                    if left {
                        let out = r & msb != 0;
                        r = ((r << 1) | x as u32) & m;
                        x = out;
                    } else {
                        let out = r & 1 != 0;
                        r = (r >> 1) | if x { msb } else { 0 };
                        x = out;
                    }
                }
                self.put(X, x);
                self.put(C, x);
                self.put(V, false);
            }
            _ => {
                let n = count % bits;
                if count > 0 {
                    r = if left { ((r << n) | (r >> ((bits - n) % bits))) & m } else { ((r >> n) | (r << ((bits - n) % bits))) & m };
                    if n == 0 {
                        r = v & m;
                    }
                    c = if left { r & 1 != 0 } else { r & msb != 0 };
                }
                self.put(C, count > 0 && c);
                self.put(V, false);
            }
        }
        self.nz(r, w);
        r
    }
}
