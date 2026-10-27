//! The instructions, from the IDT R30xx manual's Appendix A and Chapters 3-4 and psx-spx's "CPU Specifications"; the
//! delay slots, exceptions and the multiplier's interlock as BlackMoon_R3000.md §5 records each reading.

use emusen_mips::{Op, Word, decode};
use emusen_native::debug::kind;

use crate::cop0::{self, cause, code, sr};
use crate::{BranchDelay, Bus, LoadDelay, Move, Observer, R3000, Registers, Shift, Size, Step};

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

/// Whether the boundary after a step that completed an instruction would do nothing but fetch, while SR, Cause, DCIC
/// and the interrupt line hold: the debug unit off, no MTC0 view pending, and no interrupt it would take with Cause as
/// it stands; then the line as Cause's IP2 has it, which the boundary before wrote.
fn quiet(cpu: &R3000) -> Option<bool> {
    let r = &cpu.regs;
    let checked = r.dcic & cop0::dcic::DE != 0 || r.interrupt_view.is_some();
    let taken = r.sr & sr::IEC != 0 && r.sr & r.cause & 0xFF00 != 0;
    (!checked && !taken).then_some(r.cause & cause::IP2 != 0)
}

/// `R3000::run`: steps while the bus says the run may go on. After a full step that leaves the boundary quiet, the
/// steps that follow make only the fetchable check, the fetch and the instruction, until the line moves, the PC cannot
/// be fetched from, or an instruction that can change SR, Cause or DCIC has run; then a full step again
/// (BlackMoon_R3000.md §13).
pub(crate) fn run<B: Bus>(cpu: &mut R3000, b: &mut B, limit: u32) -> (u32, Step) {
    let mut n = 0;
    'full: loop {
        let s = step(cpu, b, &mut crate::Unobserved);
        n += 1;
        if s != Step::Instruction || n >= limit || !b.running() || b.stop_before(cpu.regs.pc) {
            return (n, s);
        }
        let Some(line) = quiet(cpu) else { continue };
        loop {
            let at = cpu.regs.pc;
            if b.interrupt() != line || !fetchable(cpu, at) {
                continue 'full;
            }
            let slot = cpu.regs.branch;
            let pending = cpu.regs.load.register.map(|r| (r as usize, cpu.regs.load.value));
            let next = if slot.slot && slot.taken { slot.target } else { at.wrapping_add(4) };
            let (s, moved) = instruction(cpu, b, &mut crate::Unobserved, at, slot, pending, next);
            n += 1;
            if s != Step::Instruction || n >= limit || !b.running() || b.stop_before(cpu.regs.pc) {
                return (n, s);
            }
            if moved {
                continue 'full;
            }
        }
    }
}

/// One step at the boundary: an interrupt, then the observer's stop, then the instruction (BlackMoon_R3000.md §5).
#[inline(always)]
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
    instruction(cpu, b, o, at, slot, pending, next).0
}

/// The fetch and the instruction, once the boundary has let it run: the common instructions by `common`, the rest
/// by `Run::execute`; and whether the instruction may have changed SR, Cause or DCIC, as only MTC0 and RFE can.
#[inline(always)]
fn instruction<B: Bus, O: Observer>(cpu: &mut R3000, b: &mut B, o: &mut O, at: u32, slot: BranchDelay, pending: Option<(usize, u32)>, next: u32) -> (Step, bool) {
    let (word, op) = b.fetch_decoded(at);
    if b.bus_error() {
        return (raise(cpu, b, o, at, 0, next, slot, pending, code::IBE, GENERAL), true);
    }
    cpu.regs.branch = BranchDelay::default();
    cpu.regs.load = LoadDelay::default();
    if let Some(s) = common(cpu, b, o, at, word, op, slot, pending, next) {
        return (s, false);
    }
    let mut x = Run { cpu, b, o, at, word, next, slot, pending, result: None, load: None, faulted: false };
    (x.execute(op), matches!(op, Op::Mtc(0) | Op::Cop(0)))
}

/// An exception the instruction at `at` raises, as `Run::exception` enters it.
#[cold]
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn raise<B: Bus, O: Observer>(cpu: &mut R3000, b: &mut B, o: &mut O, at: u32, word: u32, next: u32, slot: BranchDelay, pending: Option<(usize, u32)>, code: u32, vectors: [u32; 2]) -> Step {
    let mut x = Run { cpu, b, o, at, word, next, slot, pending, result: None, load: None, faulted: false };
    x.exception(code, vectors)
}

/// The address error or break a data access at `address`, whose bus address is `bus`, raises: the debug unit's
/// comparison, then alignment and kernel space in user mode, with BadVaddr, then the break (BlackMoon_R3000.md D-20).
#[inline(always)]
fn access_fault(r: &mut Registers, address: u32, bus: u32, align: u32, write: bool) -> Option<(u32, [u32; 2])> {
    let trap = cop0::data_break(r, bus, write);
    if address & align != 0 || (!cop0::kernel(r) && address >= 0x8000_0000) {
        r.bad_vaddr = address;
        return Some((if write { code::ADES } else { code::ADEL }, GENERAL));
    }
    trap.then_some((code::BP, DEBUG))
}

/// Whether a data access reaches memory, the cache not isolated: only then is a load's or a store's move told.
#[inline(always)]
fn reaches(cpu: &R3000) -> bool {
    cpu.regs.sr & sr::ISC == 0
}

/// A data read through the bus, or from the cache while it is isolated, its bytes told to the observer unless it met
/// no device; the value and whether it met none.
#[inline(always)]
fn bus_read<B: Bus, O: Observer>(cpu: &R3000, b: &mut B, o: &mut O, at: u32, address: u32, size: Size) -> (u32, bool) {
    if cpu.regs.sr & sr::ISC != 0 {
        return (b.isolated_read(address, size) & mask(size), false);
    }
    let v = b.read(address, size);
    let faulted = b.bus_error();
    let v = v & mask(size);
    if !faulted {
        for i in 0..size.bytes() {
            o.read(cpu.space, address.wrapping_add(i), (v >> (8 * i)) as u8, at);
        }
    }
    (v, faulted)
}

/// A data write through the bus, or to the cache while it is isolated, its bytes told to the observer unless it met no
/// device; whether it met none.
#[inline(always)]
fn bus_write<B: Bus, O: Observer>(cpu: &R3000, b: &mut B, o: &mut O, at: u32, address: u32, size: Size, value: u32) -> bool {
    if cpu.regs.sr & sr::ISC != 0 {
        b.isolated_write(address, size, value);
        return false;
    }
    b.write(address, size, value);
    if b.bus_error() {
        return true;
    }
    for i in 0..size.bytes() {
        o.wrote(cpu.space, address.wrapping_add(i), (value >> (8 * i)) as u8, at);
    }
    false
}

/// The previous load lands, unless the instruction loads the same register `loads`.
#[inline(always)]
fn land(r: &mut Registers, pending: Option<(usize, u32)>, loads: usize) {
    if let Some((p, y)) = pending
        && p != loads
    {
        r.r[p] = y;
    }
}

/// A branch or jump completed: the delay slot's branch recorded, the previous load landed, the link register (0 for
/// none) given the address after the delay slot.
#[inline(always)]
fn jumped(cpu: &mut R3000, pending: Option<(usize, u32)>, next: u32, taken: bool, target: u32, link: usize) -> Step {
    let r = &mut cpu.regs;
    r.branch = BranchDelay { slot: true, taken, target };
    land(r, pending, 32);
    r.r[link] = next.wrapping_add(4);
    r.r[0] = 0;
    r.pc = next;
    Step::Instruction
}

/// LB, LBU, LH, LHU and LW: the checks, the read, the value sign-extended or not, landing after the next instruction.
#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn load<B: Bus, O: Observer>(cpu: &mut R3000, b: &mut B, o: &mut O, at: u32, word: u32, next: u32, slot: BranchDelay, pending: Option<(usize, u32)>, size: Size, signed: bool) -> Step {
    let w = Word(word);
    let address = cpu.regs.r[w.rs()].wrapping_add(w.simm());
    if let Some((c, vectors)) = access_fault(&mut cpu.regs, address, address, size.bytes() - 1, false) {
        return raise(cpu, b, o, at, word, next, slot, pending, c, vectors);
    }
    let (x, faulted) = bus_read(cpu, b, o, at, address, size);
    if faulted {
        return raise(cpu, b, o, at, word, next, slot, pending, code::DBE, GENERAL);
    }
    let x = match (size, signed) {
        (Size::Byte, true) => x as i8 as u32,
        (Size::Half, true) => x as i16 as u32,
        _ => x,
    };
    let rt = w.rt();
    if B::MOVES && reaches(cpu) && rt != 0 {
        b.moved(Move::Load { at, register: rt as u8, address, size, value: x });
    }
    let r = &mut cpu.regs;
    land(r, pending, rt);
    r.r[0] = 0;
    r.load = LoadDelay { register: Some(rt as u8), value: x };
    r.pc = next;
    Step::Instruction
}

/// SB, SH and SW: the checks and the write of rt as it was before the previous load lands.
#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn store<B: Bus, O: Observer>(cpu: &mut R3000, b: &mut B, o: &mut O, at: u32, word: u32, next: u32, slot: BranchDelay, pending: Option<(usize, u32)>, size: Size) -> Step {
    let w = Word(word);
    let address = cpu.regs.r[w.rs()].wrapping_add(w.simm());
    if let Some((c, vectors)) = access_fault(&mut cpu.regs, address, address, size.bytes() - 1, true) {
        return raise(cpu, b, o, at, word, next, slot, pending, c, vectors);
    }
    let v = cpu.regs.r[w.rt()];
    if B::MOVES && reaches(cpu) {
        b.moved(Move::Store { at, register: w.rt() as u8, address, size, value: v & mask(size) });
    }
    if bus_write(cpu, b, o, at, address, size, v) {
        return raise(cpu, b, o, at, word, next, slot, pending, code::DBE, GENERAL);
    }
    let r = &mut cpu.regs;
    land(r, pending, 32);
    r.r[0] = 0;
    r.pc = next;
    Step::Instruction
}

/// The register an instruction that only computes writes and its value; `None` for any other instruction, and for
/// ADD, ADDI and SUB when they overflow (IDT R30xx manual, Appendix A).
#[inline(always)]
fn computed(op: Op, w: Word, a: u32, b: u32) -> Option<(usize, u32)> {
    let (rt, rd) = (w.rt(), w.rd());
    Some(match op {
        Op::Sll => (rd, b << w.sa()),
        Op::Srl => (rd, b >> w.sa()),
        Op::Sra => (rd, ((b as i32) >> w.sa()) as u32),
        Op::Sllv => (rd, b << (a & 31)),
        Op::Srlv => (rd, b >> (a & 31)),
        Op::Srav => (rd, ((b as i32) >> (a & 31)) as u32),
        Op::Add => (rd, (a as i32).checked_add(b as i32)? as u32),
        Op::Addu => (rd, a.wrapping_add(b)),
        Op::Sub => (rd, (a as i32).checked_sub(b as i32)? as u32),
        Op::Subu => (rd, a.wrapping_sub(b)),
        Op::And => (rd, a & b),
        Op::Or => (rd, a | b),
        Op::Xor => (rd, a ^ b),
        Op::Nor => (rd, !(a | b)),
        Op::Slt => (rd, ((a as i32) < (b as i32)) as u32),
        Op::Sltu => (rd, (a < b) as u32),
        Op::Addi => (rt, (a as i32).checked_add(w.simm() as i32)? as u32),
        Op::Addiu => (rt, a.wrapping_add(w.simm())),
        Op::Slti => (rt, ((a as i32) < (w.simm() as i32)) as u32),
        Op::Sltiu => (rt, (a < w.simm()) as u32),
        Op::Andi => (rt, a & w.imm()),
        Op::Ori => (rt, a | w.imm()),
        Op::Xori => (rt, a ^ w.imm()),
        Op::Lui => (rt, w.imm() << 16),
        _ => return None,
    })
}

/// The register an instruction that only computes gives unchanged, as `Move::Copy` names the forms.
#[inline(always)]
fn copied(op: Op, w: Word) -> Option<usize> {
    let (rs, rt) = (w.rs(), w.rt());
    match op {
        Op::Addu | Op::Add | Op::Subu | Op::Sub | Op::Or | Op::Xor if rt == 0 => Some(rs),
        Op::Addu | Op::Add | Op::Or | Op::Xor if rs == 0 => Some(rt),
        Op::Addiu | Op::Addi | Op::Ori | Op::Xori if w.imm() == 0 => Some(rs),
        Op::Sll | Op::Srl | Op::Sra if w.sa() == 0 => Some(rt),
        _ => None,
    }
}

/// A shift by a constant other than 0, as `Move::Shift` names it.
#[inline(always)]
fn shifted(op: Op, w: Word) -> Option<Shift> {
    match op {
        _ if w.sa() == 0 => None,
        Op::Sll => Some(Shift::Left),
        Op::Srl => Some(Shift::Right),
        Op::Sra => Some(Shift::Arithmetic),
        _ => None,
    }
}

/// A branch or jump: whether it is taken, its target (a conditional branch's counted from the next instruction's
/// address) and the register it links, 0 for none.
#[inline(always)]
fn transfer(op: Op, w: Word, a: u32, b: u32, next: u32) -> Option<(bool, u32, usize)> {
    let near = next.wrapping_add(w.simm() << 2);
    let far = next & 0xF000_0000 | w.target() << 2;
    Some(match op {
        Op::Beq => (a == b, near, 0),
        Op::Bne => (a != b, near, 0),
        Op::Blez => (a as i32 <= 0, near, 0),
        Op::Bgtz => (a as i32 > 0, near, 0),
        Op::Bltz => ((a as i32) < 0, near, 0),
        Op::Bgez => ((a as i32) >= 0, near, 0),
        Op::Bltzal => ((a as i32) < 0, near, 31),
        Op::Bgezal => ((a as i32) >= 0, near, 31),
        Op::J => (true, far, 0),
        Op::Jal => (true, far, 31),
        Op::Jr => (true, a, 0),
        Op::Jalr => (true, a, w.rd()),
        _ => return None,
    })
}

/// LB, LBU, LH, LHU and LW: the size and whether the value is sign-extended.
#[inline(always)]
fn loaded(op: Op) -> Option<(Size, bool)> {
    Some(match op {
        Op::Lb => (Size::Byte, true),
        Op::Lbu => (Size::Byte, false),
        Op::Lh => (Size::Half, true),
        Op::Lhu => (Size::Half, false),
        Op::Lw => (Size::Word, false),
        _ => return None,
    })
}

/// SB, SH and SW: the size.
#[inline(always)]
fn stored(op: Op) -> Option<Size> {
    Some(match op {
        Op::Sb => Size::Byte,
        Op::Sh => Size::Half,
        Op::Sw => Size::Word,
        _ => return None,
    })
}

/// The common instructions, those that compute, branch, jump, load or store a byte, halfword or word, each completed
/// as `Run::finish` completes one: the previous load lands unless this one loads the same register, then the result,
/// R0 kept zero, a load left for the next instruction. `None` for any other instruction, before it has done anything.
#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn common<B: Bus, O: Observer>(cpu: &mut R3000, b: &mut B, o: &mut O, at: u32, word: u32, op: Op, slot: BranchDelay, pending: Option<(usize, u32)>, next: u32) -> Option<Step> {
    let w = Word(word);
    let (a, v) = (cpu.regs.r[w.rs()], cpu.regs.r[w.rt()]);
    match computed(op, w, a, v) {
        Some((i, x)) => {
            if B::MOVES && i != 0 {
                if let Some(from) = copied(op, w) {
                    b.moved(Move::Copy { at, register: i as u8, from: from as u8, value: x });
                } else if let Some(kind) = shifted(op, w) {
                    b.moved(Move::Shift { at, register: i as u8, from: w.rt() as u8, kind, amount: w.sa() as u8, value: x });
                }
            }
            let r = &mut cpu.regs;
            land(r, pending, 32);
            r.r[i] = x;
            r.r[0] = 0;
            r.pc = next;
            return Some(Step::Instruction);
        }
        None if matches!(op, Op::Add | Op::Addi | Op::Sub) => return Some(raise(cpu, b, o, at, word, next, slot, pending, code::OV, GENERAL)),
        None => {}
    }
    if let Some((taken, target, link)) = transfer(op, w, a, v, next) {
        match op {
            Op::Jr if w.rs() == 31 => o.returned(),
            Op::Jalr if link != 0 => o.called(at, target, kind::CALL),
            Op::Jal => o.called(at, target, kind::CALL),
            Op::Bltzal | Op::Bgezal if taken => o.called(at, target, kind::CALL),
            _ => {}
        }
        return Some(jumped(cpu, pending, next, taken, target, link));
    }
    if let Some((size, signed)) = loaded(op) {
        return Some(load(cpu, b, o, at, word, next, slot, pending, size, signed));
    }
    if let Some(size) = stored(op) {
        return Some(store(cpu, b, o, at, word, next, slot, pending, size));
    }
    None
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

    /// BC0F, BC0T and their kin: the target counted from the next instruction's address.
    fn branch_if(&mut self, taken: bool) {
        let target = self.next.wrapping_add(Word(self.word).simm() << 2);
        self.cpu.regs.branch = BranchDelay { slot: true, taken, target };
    }

    /// The checks of a data access whose bus address is `bus`, as `access_fault` makes them.
    fn check(&mut self, address: u32, bus: u32, align: u32, write: bool) -> Option<Step> {
        let (c, vectors) = access_fault(&mut self.cpu.regs, address, bus, align, write)?;
        Some(self.exception(c, vectors))
    }

    fn read(&mut self, address: u32, size: Size) -> u32 {
        let (v, faulted) = bus_read(self.cpu, self.b, self.o, self.at, address, size);
        self.faulted |= faulted;
        v
    }

    fn write(&mut self, address: u32, size: Size, value: u32) {
        self.faulted |= bus_write(self.cpu, self.b, self.o, self.at, address, size, value);
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
                if B::MOVES && !self.faulted && reaches(self.cpu) && rt != 0 {
                    let size = Size::of(if op == Op::Lwl { o + 1 } else { 4 - o });
                    self.b.moved(Move::Load { at: self.at, register: rt as u8, address: bus, size, value: v });
                }
                self.load = Some((rt, v));
            }
            Op::Swl | Op::Swr => {
                let address = self.address();
                let bus = if op == Op::Swl { address & !3 } else { address };
                if let Some(e) = self.check(address, bus, 0, true) {
                    return e;
                }
                let o = address & 3;
                if B::MOVES && reaches(self.cpu) {
                    let (size, value) = if op == Op::Swl { (Size::of(o + 1), b >> (24 - 8 * o)) } else { (Size::of(4 - o), b) };
                    self.b.moved(Move::Store { at: self.at, register: rt as u8, address: bus, size, value: value & mask(size) });
                }
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
                        if B::MOVES && !self.faulted && reaches(self.cpu) {
                            self.b.moved(Move::LoadCop2 { at: self.at, cop2: rt as u8, address, value: v });
                        }
                    }
                    (2, true) => {
                        let v = self.b.cop2_data(rt as u8);
                        if B::MOVES && reaches(self.cpu) {
                            self.b.moved(Move::StoreCop2 { at: self.at, cop2: rt as u8, address, value: v });
                        }
                        self.write(address, Size::Word, v);
                    }
                    (_, false) => {
                        self.read(address, Size::Word);
                    }
                    (_, true) => self.write(address, Size::Word, 0),
                }
            }
            Op::Reserved => return self.exception(code::RI, GENERAL),
            _ => unreachable!("{op:?} is one of the common instructions, which `common` runs"),
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
            (2, Op::Mfc(_)) => {
                let v = self.b.cop2_data(rd as u8);
                if B::MOVES && rt != 0 {
                    self.b.moved(Move::FromCop2 { at: self.at, register: rt as u8, cop2: rd as u8, value: v });
                }
                self.load = Some((rt, v));
            }
            (2, Op::Cfc(_)) => self.load = Some((rt, self.b.cop2_control(rd as u8))),
            (2, Op::Mtc(_)) => {
                let v = self.r(rt);
                self.b.set_cop2_data(rd as u8, v);
                if B::MOVES {
                    self.b.moved(Move::ToCop2 { at: self.at, register: rt as u8, cop2: rd as u8, value: v });
                }
            }
            (2, Op::Ctc(_)) => self.b.set_cop2_control(rd as u8, self.r(rt)),
            (2, _) => self.b.cop2_command(w.cofun()),
            (_, Op::Mfc(_) | Op::Cfc(_)) => self.load = Some((rt, 0)),
            _ => {}
        }
        self.finish()
    }
}
