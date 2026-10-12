//! The instructions, from the IDT R30xx manual's Appendix A and Chapters 3-4 and psx-spx's "CPU Specifications"; the
//! delay slots, exceptions and the multiplier's interlock as BlackMoon_R3000.md §5 records each reading.

use emusen_mips::{Op, Word, decode};
use emusen_native::debug::kind;

use crate::cop0::{self, cause, code, sr};
use crate::{BranchDelay, Bus, LoadDelay, Observer, R3000, Size, Step};

/// The general exception vector, and the debug unit's, with BEV clear and set (psx-spx "Exception Vectors").
const GENERAL: [u32; 2] = [0x8000_0080, 0xBFC0_0180];
const DEBUG: [u32; 2] = [0x8000_0040, 0xBFC0_0140];

/// The multiplier's latency by rs (psx-spx "Multiply/divide"): 6, 9 or 13 cycles as rs's significant bits grow.
fn multiply_cycles(rs: u32, signed: bool) -> u32 {
    let magnitude = if signed && (rs as i32) < 0 { !rs } else { rs };
    match magnitude {
        0..0x800 => 6,
        0x800..0x10_0000 => 9,
        _ => 13,
    }
}

const DIVIDE_CYCLES: u32 = 36;

fn mask(size: Size) -> u32 {
    match size {
        Size::Word => u32::MAX,
        s => (1 << (8 * s.bytes())) - 1,
    }
}

struct Run<'a, B: Bus, O: Observer> {
    cpu: &'a mut R3000,
    b: &'a mut B,
    o: &'a mut O,
    /// The instruction's address and word.
    at: u32,
    word: u32,
    /// The address of the next instruction: the delay slot's successor, or a taken branch's target.
    next: u32,
    /// The branch delay as the instruction began, which an exception records.
    slot: BranchDelay,
    /// The previous instruction's load, written after this instruction reads its operands.
    pending: Option<(usize, u32)>,
    /// What this instruction writes to a general register at once, and what it loads for the next instruction.
    result: Option<(usize, u32)>,
    load: Option<(usize, u32)>,
    /// Whether a data access met no device, so the instruction ends in DBE instead of completing.
    faulted: bool,
}

fn fetchable(cpu: &R3000, address: u32) -> bool {
    address & 3 == 0 && (cop0::kernel(&cpu.regs) || address < 0x8000_0000)
}

/// The bytes the instruction `word` at the PC would read from the bus if it ran now, as (first address, count): none for
/// an instruction that loads nothing, or whose fetch, debug unit, coprocessor check or address check would end it first,
/// or with the cache isolated (BlackMoon_R3000.md §12).
pub(crate) fn load_ahead(cpu: &R3000, word: u32) -> Option<(u32, u32)> {
    let r = &cpu.regs;
    let mut probe = *r;
    if !fetchable(cpu, r.pc) || cop0::pc_break(&mut probe, r.pc) || r.sr & sr::ISC != 0 {
        return None;
    }
    let w = Word(word);
    let address = r.r[w.rs()].wrapping_add(w.simm());
    let (bus, first, count, align) = match decode(word) {
        Op::Lb | Op::Lbu => (address, address, 1, 0),
        Op::Lh | Op::Lhu => (address, address, 2, 1),
        Op::Lw => (address, address, 4, 3),
        Op::Lwl => (address & !3, address & !3, (address & 3) + 1, 0),
        Op::Lwr => (address, address, 4 - (address & 3), 0),
        Op::Lwc(z) if r.sr & sr::CU0 << z != 0 => (address, address, 4, 3),
        _ => return None,
    };
    let trap = cop0::data_break(&mut probe, bus, false);
    if address & align != 0 || (!cop0::kernel(r) && address >= 0x8000_0000) || trap {
        return None;
    }
    Some((first, count))
}

/// One step at the boundary: an interrupt, then the observer's stop, then the instruction (BlackMoon_R3000.md §5).
pub(crate) fn step<B: Bus, O: Observer>(cpu: &mut R3000, b: &mut B, o: &mut O) -> Step {
    let at = cpu.regs.pc;
    if b.interrupt() {
        cpu.regs.cause |= cause::IP2;
    } else {
        cpu.regs.cause &= !cause::IP2;
    }
    let slot = cpu.regs.branch;
    let pending = cpu.regs.load.register.map(|r| (r as usize, cpu.regs.load.value));
    let r = &cpu.regs;
    let (status, software) = r.interrupt_view.unwrap_or((r.sr, r.cause & cause::SW));
    if status & sr::IEC != 0 && status & (r.cause & !cause::SW | software) & 0xFF00 != 0 {
        // The interrupted instruction is fetched, and a GTE command runs before the exception (psx-spx).
        let mut word = 0;
        if fetchable(cpu, at) {
            word = b.fetch(at);
            if word >> 25 == 0x25 && cpu.regs.sr & sr::CU2 != 0 {
                b.cop2_command(word & 0x01FF_FFFF);
            }
        }
        let mut x = Run { cpu, b, o, at, word, next: 0, slot, pending, result: None, load: None, faulted: false };
        return x.exception(code::INT, GENERAL);
    }
    let why = o.before(cpu.processor, at);
    if why != 0 {
        return Step::Observed(why);
    }
    let next = if slot.slot && slot.taken { slot.target } else { at.wrapping_add(4) };
    let trap = cop0::pc_break(&mut cpu.regs, at);
    if !fetchable(cpu, at) {
        cpu.regs.bad_vaddr = at;
        let mut x = Run { cpu, b, o, at, word: 0, next, slot, pending, result: None, load: None, faulted: false };
        return x.exception(code::ADEL, GENERAL);
    }
    if trap {
        let mut x = Run { cpu, b, o, at, word: 0, next, slot, pending, result: None, load: None, faulted: false };
        return x.exception(code::BP, DEBUG);
    }
    cpu.regs.interrupt_view = None;
    let (word, op) = b.fetch_decoded(at);
    if b.bus_error() {
        let mut x = Run { cpu, b, o, at, word: 0, next, slot, pending, result: None, load: None, faulted: false };
        return x.exception(code::IBE, GENERAL);
    }
    cpu.regs.branch = BranchDelay::default();
    cpu.regs.load = LoadDelay::default();
    let mut x = Run { cpu, b, o, at, word, next, slot, pending, result: None, load: None, faulted: false };
    x.execute(op)
}

impl<B: Bus, O: Observer> Run<'_, B, O> {
    fn r(&self, i: usize) -> u32 {
        self.cpu.regs.r[i]
    }

    fn set(&mut self, i: usize, v: u32) {
        self.result = Some((i, v));
    }

    /// The instruction completes: the previous load lands unless this one loads the same register, then this
    /// instruction's own result, and its load waits for the next.
    #[inline(always)]
    fn finish(&mut self) -> Step {
        if self.faulted {
            return self.exception(code::DBE, GENERAL);
        }
        let r = &mut self.cpu.regs;
        if let Some((i, v)) = self.pending
            && self.load.map(|l| l.0) != Some(i)
        {
            r.r[i] = v;
        }
        if let Some((i, v)) = self.result {
            r.r[i] = v;
        }
        r.r[0] = 0;
        if let Some((i, v)) = self.load {
            r.load = LoadDelay { register: Some(i as u8), value: v };
        }
        r.pc = self.next;
        Step::Instruction
    }

    /// Exception entry (IDT R30xx manual, Chapter 4; psx-spx "COP0 - Exception Handling"): the previous load lands,
    /// Cause and EPC record where and why, the mode stack is pushed, and execution continues at the vector.
    fn exception(&mut self, code: u32, vectors: [u32; 2]) -> Step {
        let r = &mut self.cpu.regs;
        if let Some((i, v)) = self.pending {
            r.r[i] = v;
            r.r[0] = 0;
        }
        let bd = self.slot.slot;
        let bt = bd && self.slot.taken;
        let ce = if code == code::DBE || vectors == DEBUG { 0 } else { self.word >> 26 & 3 };
        r.cause = r.cause & !cause::EXCEPTION | code << 2 | ce << 28 | (bt as u32) << 30 | (bd as u32) << 31;
        r.epc = if bd { self.at.wrapping_sub(4) } else { self.at };
        if bd {
            r.tar = if bt { self.slot.target } else { self.at.wrapping_add(4) };
        }
        r.interrupt_view = None;
        r.sr = r.sr & !0x3F | r.sr << 2 & 0x3C;
        let vector = vectors[(r.sr & sr::BEV != 0) as usize];
        r.pc = vector;
        r.branch = BranchDelay::default();
        r.load = LoadDelay::default();
        self.o.called(self.at, vector, if code == code::INT { kind::IRQ } else { kind::BRK });
        Step::Exception(code as u8)
    }

    fn branch(&mut self, taken: bool, target: u32) {
        self.cpu.regs.branch = BranchDelay { slot: true, taken, target };
    }

    /// A conditional branch: its target is counted from the next instruction's address.
    fn branch_if(&mut self, taken: bool) {
        let target = self.next.wrapping_add(Word(self.word).simm() << 2);
        self.branch(taken, target);
    }

    fn call(&mut self, target: u32) {
        self.o.called(self.at, target, kind::CALL);
    }

    /// The checks of a data access whose bus address is `bus`: the debug unit's comparison, then alignment and kernel
    /// space in user mode (address error), then the break (BlackMoon_R3000.md D-20).
    fn check(&mut self, address: u32, bus: u32, align: u32, write: bool) -> Option<Step> {
        let trap = cop0::data_break(&mut self.cpu.regs, bus, write);
        if address & align != 0 || (!cop0::kernel(&self.cpu.regs) && address >= 0x8000_0000) {
            self.cpu.regs.bad_vaddr = address;
            return Some(self.exception(if write { code::ADES } else { code::ADEL }, GENERAL));
        }
        if trap {
            return Some(self.exception(code::BP, DEBUG));
        }
        None
    }

    fn read(&mut self, address: u32, size: Size) -> u32 {
        if self.cpu.regs.sr & sr::ISC != 0 {
            return self.b.isolated_read(address, size) & mask(size);
        }
        let v = self.b.read(address, size);
        self.faulted |= self.b.bus_error();
        let v = v & mask(size);
        if !self.faulted {
            for i in 0..size.bytes() {
                self.o.read(self.cpu.space, address.wrapping_add(i), (v >> (8 * i)) as u8, self.at);
            }
        }
        v
    }

    fn write(&mut self, address: u32, size: Size, value: u32) {
        if self.cpu.regs.sr & sr::ISC != 0 {
            self.b.isolated_write(address, size, value);
            return;
        }
        self.b.write(address, size, value);
        if self.b.bus_error() {
            self.faulted = true;
            return;
        }
        for i in 0..size.bytes() {
            self.o.wrote(self.cpu.space, address.wrapping_add(i), (value >> (8 * i)) as u8, self.at);
        }
    }

    fn address(&self) -> u32 {
        let w = Word(self.word);
        self.r(w.rs()).wrapping_add(w.simm())
    }

    fn multiplied(&mut self, product: u64, cycles: u32) {
        self.cpu.regs.hi = (product >> 32) as u32;
        self.cpu.regs.lo = product as u32;
        self.cpu.hilo_ready = self.b.cycles() + cycles as u64;
    }

    fn divided(&mut self, quotient: u32, remainder: u32) {
        self.cpu.regs.lo = quotient;
        self.cpu.regs.hi = remainder;
        self.cpu.hilo_ready = self.b.cycles() + DIVIDE_CYCLES as u64;
    }

    /// MFHI and MFLO hold until the multiplier is done.
    fn interlock(&mut self) {
        let now = self.b.cycles();
        if now < self.cpu.hilo_ready {
            self.b.idle((self.cpu.hilo_ready - now) as u32);
        }
    }

    /// Whether coprocessor `z`'s instructions may run: its CU bit, or coprocessor 0 in kernel mode.
    fn usable(&self, z: u8) -> bool {
        self.cpu.regs.sr & sr::CU0 << z != 0 || (z == 0 && cop0::kernel(&self.cpu.regs))
    }

    /// The load from `address` of `size` bytes, sign-extended or not, into rt after the next instruction.
    fn load(&mut self, size: Size, signed: bool) -> Step {
        let address = self.address();
        if let Some(e) = self.check(address, address, size.bytes() - 1, false) {
            return e;
        }
        let v = self.read(address, size);
        let v = match (size, signed) {
            (Size::Byte, true) => v as i8 as u32,
            (Size::Half, true) => v as i16 as u32,
            _ => v,
        };
        self.load = Some((Word(self.word).rt(), v));
        self.finish()
    }

    fn store(&mut self, size: Size) -> Step {
        let address = self.address();
        if let Some(e) = self.check(address, address, size.bytes() - 1, true) {
            return e;
        }
        let v = self.r(Word(self.word).rt());
        self.write(address, size, v);
        self.finish()
    }

    /// LWL and LWR merge into rt as it will be after the pending load, so the pair needs no delay between them.
    fn merge_base(&self, rt: usize) -> u32 {
        match self.pending {
            Some((i, v)) if i == rt => v,
            _ => self.r(rt),
        }
    }

    fn execute(&mut self, op: Op) -> Step {
        let w = Word(self.word);
        let (rs, rt, rd) = (w.rs(), w.rt(), w.rd());
        let (a, b) = (self.r(rs), self.r(rt));
        match op {
            Op::Sll => self.set(rd, b << w.sa()),
            Op::Srl => self.set(rd, b >> w.sa()),
            Op::Sra => self.set(rd, ((b as i32) >> w.sa()) as u32),
            Op::Sllv => self.set(rd, b << (a & 31)),
            Op::Srlv => self.set(rd, b >> (a & 31)),
            Op::Srav => self.set(rd, ((b as i32) >> (a & 31)) as u32),
            Op::Jr => {
                if rs == 31 {
                    self.o.returned();
                }
                self.branch(true, a);
            }
            Op::Jalr => {
                self.set(rd, self.next.wrapping_add(4));
                self.branch(true, a);
                if rd != 0 {
                    self.call(a);
                }
            }
            Op::Syscall => return self.exception(code::SYS, GENERAL),
            Op::Break => return self.exception(code::BP, GENERAL),
            Op::Mfhi => {
                self.interlock();
                self.set(rd, self.cpu.regs.hi);
            }
            Op::Mflo => {
                self.interlock();
                self.set(rd, self.cpu.regs.lo);
            }
            Op::Mthi => self.cpu.regs.hi = a,
            Op::Mtlo => self.cpu.regs.lo = a,
            Op::Mult => self.multiplied((a as i32 as i64 * b as i32 as i64) as u64, multiply_cycles(a, true)),
            Op::Multu => self.multiplied(a as u64 * b as u64, multiply_cycles(a, false)),
            Op::Div => match (a as i32, b as i32) {
                (n, 0) => self.divided(if n < 0 { 1 } else { u32::MAX }, a),
                (i32::MIN, -1) => self.divided(0x8000_0000, 0),
                (n, d) => self.divided((n / d) as u32, (n % d) as u32),
            },
            Op::Divu => match b {
                0 => self.divided(u32::MAX, a),
                d => self.divided(a / d, a % d),
            },
            Op::Add => match (a as i32).checked_add(b as i32) {
                Some(v) => self.set(rd, v as u32),
                None => return self.exception(code::OV, GENERAL),
            },
            Op::Addu => self.set(rd, a.wrapping_add(b)),
            Op::Sub => match (a as i32).checked_sub(b as i32) {
                Some(v) => self.set(rd, v as u32),
                None => return self.exception(code::OV, GENERAL),
            },
            Op::Subu => self.set(rd, a.wrapping_sub(b)),
            Op::And => self.set(rd, a & b),
            Op::Or => self.set(rd, a | b),
            Op::Xor => self.set(rd, a ^ b),
            Op::Nor => self.set(rd, !(a | b)),
            Op::Slt => self.set(rd, ((a as i32) < (b as i32)) as u32),
            Op::Sltu => self.set(rd, (a < b) as u32),
            Op::Bltz | Op::Bgez | Op::Bltzal | Op::Bgezal => {
                let taken = ((a as i32) < 0) != matches!(op, Op::Bgez | Op::Bgezal);
                self.branch_if(taken);
                if matches!(op, Op::Bltzal | Op::Bgezal) {
                    self.set(31, self.next.wrapping_add(4));
                    if taken {
                        let target = self.cpu.regs.branch.target;
                        self.call(target);
                    }
                }
            }
            Op::J | Op::Jal => {
                let target = self.next & 0xF000_0000 | w.target() << 2;
                self.branch(true, target);
                if op == Op::Jal {
                    self.set(31, self.next.wrapping_add(4));
                    self.call(target);
                }
            }
            Op::Beq => self.branch_if(a == b),
            Op::Bne => self.branch_if(a != b),
            Op::Blez => self.branch_if(a as i32 <= 0),
            Op::Bgtz => self.branch_if(a as i32 > 0),
            Op::Addi => match (a as i32).checked_add(w.simm() as i32) {
                Some(v) => self.set(rt, v as u32),
                None => return self.exception(code::OV, GENERAL),
            },
            Op::Addiu => self.set(rt, a.wrapping_add(w.simm())),
            Op::Slti => self.set(rt, ((a as i32) < (w.simm() as i32)) as u32),
            Op::Sltiu => self.set(rt, (a < w.simm()) as u32),
            Op::Andi => self.set(rt, a & w.imm()),
            Op::Ori => self.set(rt, a | w.imm()),
            Op::Xori => self.set(rt, a ^ w.imm()),
            Op::Lui => self.set(rt, w.imm() << 16),
            Op::Lb => return self.load(Size::Byte, true),
            Op::Lbu => return self.load(Size::Byte, false),
            Op::Lh => return self.load(Size::Half, true),
            Op::Lhu => return self.load(Size::Half, false),
            Op::Lw => return self.load(Size::Word, false),
            Op::Lwl | Op::Lwr => {
                let address = self.address();
                let bus = if op == Op::Lwl { address & !3 } else { address };
                if let Some(e) = self.check(address, bus, 0, false) {
                    return e;
                }
                let o = address & 3;
                let base = self.merge_base(rt);
                let v = if op == Op::Lwl {
                    let m = self.read(address & !3, Size::of(o + 1));
                    base & (0x00FF_FFFF >> (8 * o)) | m << (24 - 8 * o)
                } else {
                    let m = self.read(address, Size::of(4 - o));
                    base & !(u32::MAX >> (8 * o)) | m
                };
                self.load = Some((rt, v));
            }
            Op::Sb => return self.store(Size::Byte),
            Op::Sh => return self.store(Size::Half),
            Op::Sw => return self.store(Size::Word),
            Op::Swl | Op::Swr => {
                let address = self.address();
                let bus = if op == Op::Swl { address & !3 } else { address };
                if let Some(e) = self.check(address, bus, 0, true) {
                    return e;
                }
                let o = address & 3;
                if op == Op::Swl {
                    self.write(address & !3, Size::of(o + 1), b >> (24 - 8 * o));
                } else {
                    self.write(address, Size::of(4 - o), b);
                }
            }
            Op::Mfc(z) | Op::Cfc(z) | Op::Mtc(z) | Op::Ctc(z) | Op::Cop(z) => return self.coprocessor(z, op),
            Op::Bcf(z) | Op::Bct(z) => {
                if self.cpu.regs.sr & sr::CU0 << z == 0 {
                    return self.exception(code::CPU, GENERAL);
                }
                let taken = matches!(op, Op::Bcf(_));
                self.branch_if(taken);
            }
            Op::Lwc(z) | Op::Swc(z) => {
                if self.cpu.regs.sr & sr::CU0 << z == 0 {
                    return self.exception(code::CPU, GENERAL);
                }
                let write = matches!(op, Op::Swc(_));
                let address = self.address();
                if let Some(e) = self.check(address, address, 3, write) {
                    return e;
                }
                match (z, write) {
                    (2, false) => {
                        let v = self.read(address, Size::Word);
                        self.b.set_cop2_data(rt as u8, v);
                    }
                    (2, true) => {
                        let v = self.b.cop2_data(rt as u8);
                        self.write(address, Size::Word, v);
                    }
                    (_, false) => {
                        self.read(address, Size::Word);
                    }
                    (_, true) => self.write(address, Size::Word, 0),
                }
            }
            Op::Reserved => return self.exception(code::RI, GENERAL),
        }
        self.finish()
    }

    /// MFCz, CFCz, MTCz, CTCz and COPz (BlackMoon_R3000.md §6).
    fn coprocessor(&mut self, z: u8, op: Op) -> Step {
        let w = Word(self.word);
        let garbage = z == 0 && matches!(op, Op::Mfc(_)) && w.rd() >= 16;
        if !self.usable(z) && !garbage {
            return self.exception(code::CPU, GENERAL);
        }
        let (rt, rd) = (w.rt(), w.rd());
        match (z, op) {
            (0, Op::Mfc(_)) => match cop0::read(&self.cpu.regs, self.cpu.prid, rd) {
                Some(v) => self.load = Some((rt, v)),
                None => return self.exception(code::RI, GENERAL),
            },
            (0, Op::Mtc(_)) => {
                let v = self.r(rt);
                let r = &mut self.cpu.regs;
                if rd == 12 {
                    r.interrupt_view = Some((r.sr, r.cause & cause::SW));
                }
                cop0::write(r, rd, v)
            }
            (0, Op::Cfc(_) | Op::Ctc(_)) => return self.exception(code::RI, GENERAL),
            (0, _) => match w.cofun() & 0x3F {
                0x10 => {
                    cop0::rfe(&mut self.cpu.regs);
                    self.o.returned();
                }
                0x01 | 0x02 | 0x06 | 0x08 => return self.exception(code::RI, GENERAL),
                _ => {}
            },
            (2, Op::Mfc(_)) => self.load = Some((rt, self.b.cop2_data(rd as u8))),
            (2, Op::Cfc(_)) => self.load = Some((rt, self.b.cop2_control(rd as u8))),
            (2, Op::Mtc(_)) => self.b.set_cop2_data(rd as u8, self.r(rt)),
            (2, Op::Ctc(_)) => self.b.set_cop2_control(rd as u8, self.r(rt)),
            (2, _) => self.b.cop2_command(w.cofun()),
            (_, Op::Mfc(_) | Op::Cfc(_)) => self.load = Some((rt, 0)),
            _ => {}
        }
        self.finish()
    }
}
