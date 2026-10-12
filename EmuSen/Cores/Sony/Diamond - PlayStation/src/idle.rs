//! Idle skipping (Diamond_Speed.md §4): a loop whose turn is proven to repeat until the clock's next event, and seen
//! to take the same time each turn, has its further turns up to that event passed at once, leaving the machine as the
//! interpreter would. The proof is `prove`, the observation and the skip `Idle::arrive`.

use blackmoon_r3000::{R3000, Registers, Size};
use emusen_mips::{Op, Word, decode};

use crate::bus::{Board, Target};
use crate::icache::Lookup;
use crate::kernel;

/// The longest turn proven, in instructions, and the farthest a backward branch may reach to make a loop's head.
const MAX_PATH: usize = 64;
const SPAN: u32 = 0x400;
/// No head pending: an address no instruction is fetched from.
const NONE: u32 = 1;
/// The steps the plain loop runs unwatched between watches, and the steps of a watch, which each window and each skip
/// begins again: a wait is met within `FREE` steps of its start, and code that never waits pays for one step in nine.
pub const FREE: u32 = 4096;
pub const TRACK: u32 = 512;
/// The turns compared before a skip: the write queue's four entries are each written once in four turns of one store.
pub const PERIOD: u64 = 4;

/// A value in the proof: a constant, or the induction's value at the turn's start plus a constant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum V {
    C(u32),
    L(u32),
}

/// A memory word the loop reads or writes, where it is in the board's memories.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Loc {
    Ram(usize),
    Scratch(usize),
}

/// The one value that changes from turn to turn: a register or a memory word, and its step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Induction {
    Reg(usize),
    Mem(Loc),
}

/// What a proof gives: the induction and its step, the turns from the proof's state before the path would change,
/// whether a turn queues stores for RAM, and the registers that move with the induction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Proof {
    pub induction: Option<(Induction, u32)>,
    pub turns: u64,
    pub queued: bool,
    /// The registers that move by the step each turn: a register induction, and those a turn writes from the induction
    /// before reading.
    pub movers: u32,
}

/// What the machine's seams need of a path: the kernel's side-load and putchar tap.
#[derive(Clone, Copy)]
pub struct Seams {
    pub exe_pending: bool,
    pub taps_putchar: bool,
}

struct Walk {
    /// The registers and the pending load at the turn's end.
    regs: [V; 32],
    load: Option<(usize, V)>,
    /// The stores of the turn, last value per word, and the comparisons on the induction: (offset, constant, equal now).
    stores: Vec<(Loc, V)>,
    tests: Vec<(u32, u32, bool)>,
    queued: bool,
    /// The registers the turn reads before it writes them.
    live: u32,
}

fn word_at(board: &Board, loc: Loc) -> u32 {
    match loc {
        Loc::Ram(a) => board.ram_word(a as u32),
        Loc::Scratch(a) => u32::from_le_bytes(board.scratch[a..a + 4].try_into().unwrap()),
    }
}

pub(crate) fn set_word_at(board: &mut Board, loc: Loc, v: u32) {
    match loc {
        Loc::Ram(a) => board.set_ram_word(a as u32, v),
        Loc::Scratch(a) => board.scratch[a..a + 4].copy_from_slice(&v.to_le_bytes()),
    }
}

/// The value of the induction at the turn's start, now.
pub fn induction_value(cpu: &R3000, board: &Board, i: Induction) -> u32 {
    match i {
        Induction::Reg(r) => cpu.regs.r[r],
        Induction::Mem(loc) => word_at(board, loc),
    }
}

/// The I/O registers a loop may read: pure reads that only the clock's events or the processor's stores change
/// (I_STAT, I_MASK and the DMA registers); their value as `Board::io_read` gives it.
fn io_peek(board: &Board, offset: u32) -> Option<u32> {
    let o = offset & !3;
    let v = match o {
        0x070 => board.irq.stat,
        0x074 => board.irq.mask,
        0x080..=0x0FF => board.dma.read(o - 0x80),
        _ => return None,
    };
    Some(v >> (8 * (offset & 3)))
}

/// The proof at the loop's head `head`: one turn walked over the code the I-cache holds, from the processor's state,
/// with a constant for every value but the induction. `None` when anything in the turn could differ from the next
/// turn but by the induction, or could reach a device (Diamond_Speed.md §4.2).
pub fn prove(cpu: &R3000, board: &Board, head: u32, seams: Seams) -> Option<Proof> {
    let r = &cpu.regs;
    let line = board.irq.line();
    let cause = if line { r.cause | 1 << 10 } else { r.cause & !(1 << 10) };
    let enabled = r.sr & 1 != 0 && r.sr & cause & 0xFF00 != 0;
    if r.pc != head || r.branch.slot || r.interrupt_view.is_some() || r.dcic & 1 << 23 != 0 || r.sr & 1 << 16 != 0 || enabled || !board.cache.enabled() {
        return None;
    }
    let first = walk(cpu, board, head, None, seams)?;
    let start: Vec<V> = r.r.iter().map(|&v| V::C(v)).collect();
    let changed_regs: Vec<usize> = (1..32).filter(|&i| first.live & 1 << i != 0 && first.regs[i] != start[i]).collect();
    let changed_mem: Vec<(Loc, V)> = first.stores.iter().copied().filter(|s| s.1 != V::C(word_at(board, s.0))).collect();
    let induction = match (changed_regs.as_slice(), changed_mem.as_slice()) {
        ([], []) => {
            return (first.load == pending(cpu)).then_some(Proof { induction: None, turns: u64::MAX, queued: first.queued, movers: 0 });
        }
        ([reg], []) => Induction::Reg(*reg),
        ([], [s]) => Induction::Mem(s.0),
        _ => return None,
    };
    let w = walk(cpu, board, head, Some(induction), seams)?;
    let step = match induction {
        Induction::Reg(i) => w.regs[i],
        Induction::Mem(loc) => w.stores.iter().find(|s| s.0 == loc).map(|s| s.1)?,
    };
    let V::L(step) = step else { return None };
    let mut movers = 0;
    for (i, v) in w.regs.iter().enumerate().skip(1) {
        match (*v, w.live & 1 << i != 0) {
            (V::L(o), _) if Induction::Reg(i) == induction && o == step => movers |= 1 << i,
            (V::L(_), true) => return None,
            (V::L(_), false) => movers |= 1 << i,
            (c, true) if c != start[i] => return None,
            _ => {}
        }
    }
    if w.load != pending(cpu) {
        return None;
    }
    for s in &w.stores {
        if Induction::Mem(s.0) != induction && s.1 != V::C(word_at(board, s.0)) {
            return None;
        }
    }
    let x = induction_value(cpu, board, induction);
    let mut turns = u64::MAX;
    for &(a, c, equal) in &w.tests {
        let k = if equal {
            1
        } else {
            match step {
                1 => c.wrapping_sub(x).wrapping_sub(a) as u64,
                u32::MAX => x.wrapping_add(a).wrapping_sub(c) as u64,
                _ => return None,
            }
        };
        turns = turns.min(k);
    }
    Some(Proof { induction: Some((induction, step)), turns, queued: w.queued, movers })
}

fn pending(cpu: &R3000) -> Option<(usize, V)> {
    cpu.regs.load.register.map(|r| (r as usize, V::C(cpu.regs.load.value)))
}

/// One turn from `head`, as `blackmoon_r3000`'s step runs each instruction: the pending load lands after the
/// operands are read, unless the instruction loads the same register; a branch takes effect after its slot.
fn walk(cpu: &R3000, board: &Board, head: u32, induction: Option<Induction>, seams: Seams) -> Option<Walk> {
    let r: &Registers = &cpu.regs;
    let kernel_mode = r.sr & 2 == 0;
    let mut regs = [V::C(0); 32];
    for (i, v) in regs.iter_mut().enumerate() {
        *v = V::C(r.r[i]);
    }
    if let Some(Induction::Reg(i)) = induction {
        regs[i] = V::L(0);
    }
    let mut load = pending(cpu);
    let mut branch: Option<u32> = None;
    let mut w = Walk { regs, load: None, stores: Vec::new(), tests: Vec::new(), queued: false, live: 0 };
    let mut written = 0u32;
    let mut at = head;
    let mut closed = false;
    for n in 0..=MAX_PATH {
        if n > 0 && at == head && branch.is_none() {
            if !closed {
                return None;
            }
            w.regs = regs;
            w.load = load;
            return Some(w);
        }
        if n == MAX_PATH || at & 3 != 0 || at >= 0xA000_0000 || (!kernel_mode && at >= 0x8000_0000) {
            return None;
        }
        let physical = at & 0x1FFF_FFFF;
        if (seams.exe_pending && physical == kernel::SHELL_ENTRY) || (seams.taps_putchar && matches!(physical, kernel::A_CALLS | kernel::B_CALLS)) {
            return None;
        }
        if !matches!(board.decode(at), Target::Ram(_) | Target::Bios(_)) {
            return None;
        }
        let Lookup::Hit(word) = board.cache.lookup(physical) else { return None };
        let slot = branch.take();
        let next = slot.unwrap_or(at.wrapping_add(4));
        let landing = load.take();
        let i = Word(word);
        let (rs, rt, rd) = (i.rs(), i.rt(), i.rd());
        let (a, b) = (regs[rs], regs[rt]);
        let op = decode(word);
        let reads = match op {
            Op::Sll | Op::Srl | Op::Sra => 1 << rt,
            Op::Lui | Op::J | Op::Jal => 0,
            Op::Addi | Op::Addiu | Op::Slti | Op::Sltiu | Op::Andi | Op::Ori | Op::Xori | Op::Blez | Op::Bgtz | Op::Bltz | Op::Bgez | Op::Bltzal | Op::Bgezal | Op::Jr | Op::Jalr => 1 << rs,
            Op::Lb | Op::Lbu | Op::Lh | Op::Lhu | Op::Lw => 1 << rs,
            _ => 1 << rs | 1 << rt,
        };
        w.live |= reads & !written;
        let mut result: Option<(usize, V)> = None;
        let mut loaded: Option<(usize, V)> = None;
        let mut taken: Option<bool> = None;
        let mut target = 0;
        let c = |v: V| match v {
            V::C(x) => Some(x),
            V::L(_) => None,
        };
        match op {
            Op::Sll | Op::Srl | Op::Sra if c(b).is_some() || rd == 0 => {
                let x = c(b).unwrap_or(0);
                let v = match op {
                    Op::Sll => x << i.sa(),
                    Op::Srl => x >> i.sa(),
                    _ => ((x as i32) >> i.sa()) as u32,
                };
                result = Some((rd, V::C(v)));
            }
            Op::Sllv | Op::Srlv | Op::Srav | Op::And | Op::Nor | Op::Slt | Op::Sltu => {
                let (x, y) = (c(a)?, c(b)?);
                let v = match op {
                    Op::Sllv => y << (x & 31),
                    Op::Srlv => y >> (x & 31),
                    Op::Srav => ((y as i32) >> (x & 31)) as u32,
                    Op::And => x & y,
                    Op::Nor => !(x | y),
                    Op::Slt => ((x as i32) < (y as i32)) as u32,
                    _ => (x < y) as u32,
                };
                result = Some((rd, V::C(v)));
            }
            Op::Addu => result = Some((rd, add(a, b)?)),
            Op::Subu => result = Some((rd, sub(a, b)?)),
            Op::Or | Op::Xor => {
                let v = match (a, b) {
                    (V::C(x), V::C(y)) => V::C(if op == Op::Or { x | y } else { x ^ y }),
                    (V::L(o), V::C(0)) | (V::C(0), V::L(o)) => V::L(o),
                    _ => return None,
                };
                result = Some((rd, v));
            }
            Op::Add | Op::Sub => {
                let (x, y) = (c(a)? as i32, c(b)? as i32);
                let v = if op == Op::Add { x.checked_add(y)? } else { x.checked_sub(y)? };
                result = Some((rd, V::C(v as u32)));
            }
            Op::Addi => result = Some((rt, V::C((c(a)? as i32).checked_add(i.simm() as i32)? as u32))),
            Op::Addiu => result = Some((rt, add(a, V::C(i.simm()))?)),
            Op::Slti => result = Some((rt, V::C(((c(a)? as i32) < (i.simm() as i32)) as u32))),
            Op::Sltiu => result = Some((rt, V::C((c(a)? < i.simm()) as u32))),
            Op::Andi => result = Some((rt, V::C(c(a)? & i.imm()))),
            Op::Ori | Op::Xori => {
                let v = match a {
                    V::C(x) => V::C(if op == Op::Ori { x | i.imm() } else { x ^ i.imm() }),
                    V::L(o) if i.imm() == 0 => V::L(o),
                    _ => return None,
                };
                result = Some((rt, v));
            }
            Op::Lui => result = Some((rt, V::C(i.imm() << 16))),
            Op::Beq | Op::Bne => {
                let equal = match (a, b) {
                    (V::C(x), V::C(y)) => x == y,
                    (V::L(x), V::L(y)) => x == y,
                    (V::L(o), V::C(k)) | (V::C(k), V::L(o)) => {
                        let x = induction.map(|ind| induction_value(cpu, board, ind))?;
                        let now = x.wrapping_add(o) == k;
                        w.tests.push((o, k, now));
                        now
                    }
                };
                taken = Some(equal == (op == Op::Beq));
                target = next.wrapping_add(i.simm() << 2);
            }
            Op::Blez | Op::Bgtz | Op::Bltz | Op::Bgez | Op::Bltzal | Op::Bgezal => {
                let x = c(a)? as i32;
                let t = match op {
                    Op::Blez => x <= 0,
                    Op::Bgtz => x > 0,
                    Op::Bltz | Op::Bltzal => x < 0,
                    _ => x >= 0,
                };
                taken = Some(t);
                target = next.wrapping_add(i.simm() << 2);
                if matches!(op, Op::Bltzal | Op::Bgezal) {
                    result = Some((31, V::C(next.wrapping_add(4))));
                }
            }
            Op::J | Op::Jal => {
                taken = Some(true);
                target = next & 0xF000_0000 | i.target() << 2;
                if op == Op::Jal {
                    result = Some((31, V::C(next.wrapping_add(4))));
                }
            }
            Op::Jr | Op::Jalr => {
                taken = Some(true);
                target = c(a)?;
                if op == Op::Jalr {
                    result = Some((rd, V::C(next.wrapping_add(4))));
                }
            }
            Op::Lb | Op::Lbu | Op::Lh | Op::Lhu | Op::Lw => {
                let address = c(a)?.wrapping_add(i.simm());
                let size = match op {
                    Op::Lb | Op::Lbu => 1,
                    Op::Lh | Op::Lhu => 2,
                    _ => 4,
                };
                if address & (size - 1) != 0 || (!kernel_mode && address >= 0x8000_0000) {
                    return None;
                }
                let word = match board.decode(address) {
                    Target::Ram(m) => stored(board, &w.stores, Loc::Ram(m & !3), induction),
                    Target::Scratch(m) => stored(board, &w.stores, Loc::Scratch(m & !3), induction),
                    Target::Bios(m) => V::C(Board::bytes_read(&board.bios, m, [Size::Byte, Size::Half, Size::Tribyte, Size::Word][size as usize - 1])),
                    Target::Io(o) => V::C(io_peek(board, o)?),
                    _ => return None,
                };
                let shift = if matches!(board.decode(address), Target::Ram(_) | Target::Scratch(_)) { 8 * (address & 3) } else { 0 };
                let v = match word {
                    V::C(x) => V::C(narrow(x >> shift, size, op)),
                    l if size == 4 => l,
                    _ => return None,
                };
                loaded = Some((rt, v));
            }
            Op::Sb | Op::Sh | Op::Sw => {
                let address = c(a)?.wrapping_add(i.simm());
                let size = match op {
                    Op::Sb => 1,
                    Op::Sh => 2,
                    _ => 4,
                };
                if address & (size - 1) != 0 || (!kernel_mode && address >= 0x8000_0000) {
                    return None;
                }
                let loc = match board.decode(address) {
                    Target::Ram(m) => {
                        w.queued |= address < 0xA000_0000;
                        Loc::Ram(m & !3)
                    }
                    Target::Scratch(m) => Loc::Scratch(m & !3),
                    _ => return None,
                };
                let v = match b {
                    V::L(_) if size == 4 && induction == Some(Induction::Mem(loc)) => b,
                    V::L(_) => return None,
                    V::C(x) if size == 4 => V::C(x),
                    V::C(x) => {
                        let V::C(old) = stored(board, &w.stores, loc, induction) else { return None };
                        let shift = 8 * (address & 3);
                        let mask = ((1u64 << (8 * size)) - 1) as u32;
                        V::C(old & !(mask << shift) | (x & mask) << shift)
                    }
                };
                w.stores.retain(|s| s.0 != loc);
                w.stores.push((loc, v));
            }
            _ => return None,
        }
        if let Some(t) = taken {
            if slot.is_some() {
                return None;
            }
            branch = Some(if t { target } else { next.wrapping_add(4) });
            let slot_at = at.wrapping_add(4);
            closed = t && target == head && target < slot_at && slot_at - target <= SPAN;
        }
        if let Some((j, v)) = landing
            && loaded.map(|l| l.0) != Some(j)
        {
            regs[j] = v;
            written |= 1 << j;
        }
        if let Some((j, v)) = result {
            regs[j] = v;
            written |= 1 << j;
        }
        regs[0] = V::C(0);
        load = loaded;
        at = next;
    }
    None
}

fn add(a: V, b: V) -> Option<V> {
    Some(match (a, b) {
        (V::C(x), V::C(y)) => V::C(x.wrapping_add(y)),
        (V::L(o), V::C(k)) | (V::C(k), V::L(o)) => V::L(o.wrapping_add(k)),
        _ => return None,
    })
}

fn sub(a: V, b: V) -> Option<V> {
    Some(match (a, b) {
        (V::C(x), V::C(y)) => V::C(x.wrapping_sub(y)),
        (V::L(o), V::C(k)) => V::L(o.wrapping_sub(k)),
        (V::L(x), V::L(y)) => V::C(x.wrapping_sub(y)),
        _ => return None,
    })
}

/// A word of RAM or the scratchpad as the turn sees it: what the turn stored there, the induction, or memory.
fn stored(board: &Board, stores: &[(Loc, V)], loc: Loc, induction: Option<Induction>) -> V {
    match stores.iter().find(|s| s.0 == loc) {
        Some(&(_, v)) => v,
        None if induction == Some(Induction::Mem(loc)) => V::L(0),
        None => V::C(word_at(board, loc)),
    }
}

/// A loaded value at its width, sign-extended as the instruction asks.
fn narrow(x: u32, size: u32, op: Op) -> u32 {
    match (size, op) {
        (1, Op::Lb) => x as i8 as u32,
        (1, _) => x & 0xFF,
        (2, Op::Lh) => x as i16 as u32,
        (2, _) => x & 0xFFFF,
        _ => x,
    }
}

/// The parts of the bus's timing a turn moves: the clock, the end of the processor's last bus access and the write
/// queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Timing {
    now: u64,
    cpu_busy: u64,
    queue: [u64; 4],
}

impl Timing {
    fn of(board: &Board) -> Timing {
        Timing { now: board.now, cpu_busy: board.cpu_busy, queue: board.queue }
    }
}

/// A loop seen from its head: the proof, the state at the first arrival, and the arrivals since.
#[derive(Clone)]
struct Window {
    head: u32,
    proof: Proof,
    regs: Registers,
    hilo: u64,
    timing: Timing,
    x: u32,
    events: u64,
    arrivals: u64,
}

fn slot(head: u32) -> usize {
    (head >> 2) as usize & 1023
}

/// What the skipper keeps between arrivals; none of it is the machine's state, so a state saved or loaded never
/// meets it.
#[derive(Clone)]
pub struct Idle {
    pub on: bool,
    /// Every how many skips one is checked against the interpreter, which runs the same turns on a copy (0: none).
    pub verify: u64,
    /// The steps left of the watch; at zero the machine runs `FREE` steps unwatched.
    pub tracking: u32,
    /// The head a backward branch just led to, met once its delay slot has run, or `NONE`.
    pub pending: u32,
    window: Option<Window>,
    /// Heads whose proof failed, and how many arrivals each waits before the next attempt.
    wait: Box<[(u32, u16, u16); 1024]>,
    pub skips: u64,
    pub turns: u64,
    pub cycles: u64,
    pub proofs: u64,
    pub refused: u64,
    pub dropped: u64,
    pub unequal: u64,
}

impl Default for Idle {
    fn default() -> Idle {
        Idle { on: true, verify: 0, tracking: TRACK, pending: NONE, window: None, wait: Box::new([(0, 0, 0); 1024]), skips: 0, turns: 0, cycles: 0, proofs: 0, refused: 0, dropped: 0, unequal: 0 }
    }
}

impl Idle {
    /// A watch begins after the unwatched steps.
    pub fn watch(&mut self) {
        self.tracking = TRACK;
        self.reset();
    }

    /// Forgets the loop being observed, as an exception, a new frame or a state loaded must.
    pub fn reset(&mut self) {
        self.pending = NONE;
        self.window = None;
    }

    /// After a step that completed an instruction: a taken backward branch names its target as the next head, which
    /// arrives once the delay slot has run; written without a branch on the step's kind, which the host cannot predict.
    #[inline(always)]
    pub fn note(&mut self, cpu: &R3000) -> Option<u32> {
        let b = &cpu.regs.branch;
        let arrived = !b.slot & (self.pending == cpu.regs.pc);
        let back = b.taken & (cpu.regs.pc.wrapping_sub(b.target).wrapping_sub(1) < SPAN);
        self.pending = if back { b.target } else { NONE };
        if arrived { Some(cpu.regs.pc) } else { None }
    }

    /// At a loop's head, its delay slot run: begins a window with a proof, or at the window's fourth arrival checks
    /// that the turns repeated and plans whole periods of them up to the next event.
    #[cold]
    #[inline(never)]
    pub fn arrive(&mut self, head: u32, cpu: &R3000, board: &Board, seams: Seams) -> Option<Skip> {
        if let Some(w) = &mut self.window {
            if w.events != board.events_run {
                let stale = w.head;
                self.window = None;
                self.dropped += 1;
                self.wait_longer(stale);
            } else if w.head != head {
                return None;
            } else {
                w.arrivals += 1;
                if w.arrivals < PERIOD {
                    return None;
                }
                let w = self.window.take().unwrap();
                if let Some(skip) = self.plan(&w, cpu, board) {
                    self.tracking = TRACK;
                    self.wait[slot(head)] = (head, 0, 0);
                    self.skips += 1;
                    self.turns += skip.turns;
                    self.cycles += skip.moved;
                    return Some(skip);
                }
                self.unequal += 1;
                self.wait_longer(head);
            }
        }
        let (h, left, _) = self.wait[slot(head)];
        if h == head && left > 0 {
            self.wait[slot(head)].1 -= 1;
            return None;
        }
        self.proofs += 1;
        match prove(cpu, board, head, seams) {
            Some(proof) if proof.turns > 2 * PERIOD => {
                let x = proof.induction.map_or(0, |(i, _)| induction_value(cpu, board, i));
                self.tracking = TRACK;
                self.window = Some(Window { head, proof, regs: cpu.regs, hilo: cpu.hilo_ready, timing: Timing::of(board), x, events: board.events_run, arrivals: 0 });
            }
            _ => {
                self.refused += 1;
                self.wait_longer(head);
            }
        }
        None
    }

    /// A head whose proof failed, or whose window met an event or did not repeat, waits twice as many arrivals as
    /// last time before its next proof, up to 4,095.
    fn wait_longer(&mut self, head: u32) {
        let (h, _, level) = self.wait[slot(head)];
        let level = if h == head { (level * 2 + 1).min(4095) } else { 1 };
        self.wait[slot(head)] = (head, level, level);
    }

    /// The fourth arrival: the registers as they were, the induction moved by its step four times, the clock and the
    /// bus's timing moved by one span; then as many whole periods as end before the next event and before the path
    /// would change.
    fn plan(&self, w: &Window, cpu: &R3000, board: &Board) -> Option<Skip> {
        let step = w.proof.induction.map_or(0, |(_, s)| s);
        let mut regs = cpu.regs;
        for r in (1..32).filter(|r| w.proof.movers & 1 << r != 0) {
            if regs.r[r] != w.regs.r[r].wrapping_add(step.wrapping_mul(PERIOD as u32)) {
                return None;
            }
            regs.r[r] = w.regs.r[r];
        }
        if regs != w.regs || cpu.hilo_ready != w.hilo || board.bus_free > w.timing.now {
            return None;
        }
        let t = Timing::of(board);
        let span = t.now - w.timing.now;
        let busy_moves = t.cpu_busy == w.timing.cpu_busy + span;
        if !busy_moves && t.cpu_busy != w.timing.cpu_busy {
            return None;
        }
        let queue_ok = if w.proof.queued { (0..4).all(|j| t.queue[j] == w.timing.queue[j] + span) } else { t.queue == w.timing.queue };
        if !queue_ok || span == 0 {
            return None;
        }
        if let Some((i, step)) = w.proof.induction
            && induction_value(cpu, board, i) != w.x.wrapping_add(step.wrapping_mul(PERIOD as u32))
        {
            return None;
        }
        if board.next_event <= t.now {
            return None;
        }
        let by_event = (board.next_event - t.now - 1) / span;
        let by_path = (w.proof.turns - PERIOD) / PERIOD;
        let n = by_event.min(by_path);
        if n == 0 {
            return None;
        }
        let by = step.wrapping_mul((n * PERIOD) as u32);
        let induction = match w.proof.induction {
            Some((Induction::Mem(loc), _)) => Some((loc, word_at(board, loc).wrapping_add(by))),
            _ => None,
        };
        Some(Skip { head: w.head, turns: n * PERIOD, moved: n * span, busy_moves, queued: w.proof.queued, induction, movers: w.proof.movers, by })
    }
}

/// A skip planned: the turns passed and the cycles they take, what of the bus's timing moves with the clock, and what
/// the induction moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Skip {
    pub head: u32,
    pub turns: u64,
    pub moved: u64,
    busy_moves: bool,
    queued: bool,
    /// A memory induction's word and its value after the turns, the registers that move, and by how much.
    induction: Option<(Loc, u32)>,
    movers: u32,
    by: u32,
}

impl Skip {
    /// The machine as `turns` more turns of the loop leave it.
    pub fn apply(&self, cpu: &mut R3000, board: &mut Board) {
        board.now += self.moved;
        if self.busy_moves {
            board.cpu_busy += self.moved;
        }
        if self.queued {
            for q in &mut board.queue {
                *q += self.moved;
            }
        }
        if let Some((loc, v)) = self.induction {
            set_word_at(board, loc, v);
        }
        for r in (1..32).filter(|r| self.movers & 1 << r != 0) {
            cpu.regs.r[r] = cpu.regs.r[r].wrapping_add(self.by);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::Machine;
    use crate::machine::tests::{machine_for, program};
    use crate::media::Market;

    const SEAMS: Seams = Seams { exe_pending: false, taps_putchar: false };

    /// A machine at the second arrival at its program's first loop head, with 11h, 22h, 33h and 44h at 80020000h.
    fn at_head(source: &str) -> (Machine, u32) {
        let mut m = machine_for(&program(source), Market::Americas);
        for (i, v) in [0x11u32, 0x22, 0x33, 0x44].into_iter().enumerate() {
            m.board.set_ram_word(0x2_0000 + 4 * i as u32, v);
        }
        let mut t = Idle::default();
        let mut seen = None;
        for _ in 0..100_000 {
            m.step();
            if let Some(h) = t.note(&m.cpu) {
                if seen == Some(h) {
                    return (m, h);
                }
                seen = Some(h);
            }
        }
        panic!("no loop");
    }

    /// The turns the interpreter runs from here before the path leaves the loop's head for good.
    fn turns_left(m: &mut Machine, head: u32) -> u64 {
        let mut t = Idle::default();
        let mut n = 0;
        for _ in 0..1_000_000 {
            m.step();
            match t.note(&m.cpu) {
                Some(h) if h == head => n += 1,
                _ if m.cpu.regs.pc == 0x8001_1000 => return n,
                _ => {}
            }
        }
        panic!("the loop never left");
    }

    #[test]
    fn the_walk_lands_each_load_as_the_processor_does() {
        let src = "lui t1, 0x8002\nori s0, zero, 50\nori a0, zero, 0x22\nloop: lw v0, 0(t1)\nlw v0, 4(t1)\nor t0, v0, zero\nlw t2, 8(t1)\nbne t0, a0, 0x80011000\naddiu t3, t2, 1\naddiu s0, s0, -1\nbne s0, zero, loop\nlw t4, 12(t1)\nj 0x80011000\nnop";
        let (mut m, head) = at_head(src);
        let p = prove(&m.cpu, &m.board, head, SEAMS).expect("the loop is proven");
        assert_eq!(p.induction, Some((Induction::Reg(16), u32::MAX)));
        assert_eq!((m.cpu.regs.r[8], m.cpu.regs.r[11], m.cpu.regs.load.register), (0x22, 0x34, Some(12)), "t0 the value before both loads, t3 before the load; t4 pending");
        assert_eq!(p.turns, turns_left(&mut m, head), "the path changes at the turn the interpreter leaves");
    }

    #[test]
    fn a_timeout_is_proven_to_the_turn_it_expires_counting_either_way_in_a_register_or_memory() {
        for (src, movers) in [
            ("lui t1, 0x8002\nori s0, zero, 40\nloop: lw v0, 16(t1)\nnop\nbne v0, zero, out\naddiu s0, s0, -1\nbne s0, zero, loop\nnop\nout: j 0x80011000\nnop", 1 << 16),
            ("lui t1, 0x8002\naddu s0, zero, zero\nloop: lw v0, 16(t1)\naddiu s0, s0, 1\nori a0, zero, 33\nbeq s0, a0, out\nnop\nbeq v0, zero, loop\nnop\nout: j 0x80011000\nnop", 1 << 16),
            ("lui t1, 0x8002\nori a0, zero, 37\nsw a0, 20(t1)\nloop: lw v0, 16(t1)\nnop\nbne v0, zero, out\nnop\nlw v0, 20(t1)\nnop\naddiu v0, v0, -1\nsw v0, 20(t1)\nbne v0, zero, loop\nnop\nout: j 0x80011000\nnop", 1 << 2),
        ] {
            let (mut m, head) = at_head(src);
            let p = prove(&m.cpu, &m.board, head, SEAMS).unwrap_or_else(|| panic!("proven: {src}"));
            assert!(p.induction.is_some(), "{src}");
            assert_eq!(p.movers, movers, "the registers that move with the induction, v0 a copy of the word: {src}");
            assert_eq!(p.turns, turns_left(&mut m, head), "{src}");
        }
    }

    #[test]
    fn loops_that_could_meet_a_device_or_differ_from_turn_to_turn_are_refused() {
        for (why, src) in [
            ("a timer read", "lui t1, 0x1F80\nloop: lhu v0, 0x1120(t1)\nnop\nbne v0, zero, loop\nnop"),
            ("GPUSTAT", "lui t1, 0x1F80\nloop: lw v0, 0x1814(t1)\nnop\nbne v0, zero, loop\nnop"),
            ("the multiplier", "ori t0, zero, 3\nloop: mult t0, t0\nmflo v0\nbne v0, zero, loop\nnop"),
            ("two words changing", "lui t1, 0x8002\nloop: lw v0, 0(t1)\nlw v1, 4(t1)\naddiu v0, v0, 1\naddiu v1, v1, 1\nsw v0, 0(t1)\nbne v0, zero, loop\nsw v1, 4(t1)"),
            ("an induction in an inequality", "ori s0, zero, 100\nloop: addiu s0, s0, -1\nbgtz s0, loop\nnop"),
            ("a store to I/O", "lui t1, 0x1F80\nloop: sw zero, 0x1070(t1)\nbeq zero, zero, loop\nnop"),
            ("coprocessor 0", "loop: mfc0 v0, SR\nbeq zero, zero, loop\nnop"),
        ] {
            let (m, head) = at_head(src);
            assert_eq!(prove(&m.cpu, &m.board, head, SEAMS), None, "{why}");
        }
        let (m, head) = at_head("lui t1, 0x8002\nloop: lw v0, 16(t1)\nnop\nbeq v0, zero, loop\nnop");
        assert_eq!(prove(&m.cpu, &m.board, head, SEAMS), Some(Proof { induction: None, turns: u64::MAX, queued: false, movers: 0 }), "a pure poll");
    }
}
