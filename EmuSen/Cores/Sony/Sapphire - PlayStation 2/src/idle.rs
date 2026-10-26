//! Idle skipping (stage S33, Sapphire_Plan.md §5.12 lever 1, Sapphire_Native.md §47): a loop of the EE whose turn is
//! seen, instruction by instruction, to read only what nothing changes before the board's next event and to come back
//! to its head with every register as it was and the timing shifted by the turn's cycles has its further turns up to
//! that event passed at once: the issue model, Count, Random and the counts moved as those turns would move them, and
//! the board brought through each of their instructions' times as `System::step` brings it, the IOP run at each and the
//! skip ended at the first time the IOP's side wakes the DMAC. Detection is the core's, at run time, from what it
//! executes; nothing names an address.

use crate::blocks::Blocks;
use crate::board::System;
use crate::clock::IOP;
use crate::dmac::Watch;
use crate::ee::Place;
use crate::ee::idle::{Access, Mark, Period};
use crate::ee::observe::Reach;

/// The farthest a backward jump may reach to make a loop's head, and the longest turn proven, in instructions.
pub const SPAN: u32 = 0x400;
pub const MAX_TURN: usize = 256;
/// The most distinct stores a proven turn makes.
const MAX_STORES: usize = 16;
/// Heads tracked, and the arrivals at one head before its turn is first proven.
const HEADS: usize = 1024;
const ARRIVALS: u8 = 3;
/// A head whose proof fails waits twice as many arrivals as before its next, up to 2^12 - 1.
const MAX_BACKOFF: u8 = 12;
/// The most turns one skip passes, far beyond a field's.
const MAX_TURNS: u64 = 1 << 32;

/// Why a turn was not passed, counted by kind (`Counts::refused`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Refusal {
    /// A store that changed the memory it reached.
    Store,
    /// A load from a device or a memory another unit writes between events: the units' registers at 10000000h, the VUs'
    /// memories, the GS's registers, IOP RAM through the EE's window, or anywhere else but memory.
    Device,
    Vu,
    Gs,
    Window,
    Elsewhere,
    /// A read of a timer, which counts with time; of the VIFs', the GIF's or the IPU's registers or FIFOs; of the SIF's
    /// registers, which the IOP writes at any time.
    Timer,
    Unit,
    Sif,
    /// A COP2 instruction: VU0's macro mode, or a read of a VU's state.
    Coprocessor,
    /// A COP0 move, ERET, a system call, a trap, CACHE, an instruction not implemented.
    Control,
    /// A register different at the head a turn later.
    Changed,
    /// A line refilled into a cache during the turn.
    Cache,
    /// The issue model, Count or Random not the turn before's shifted in time.
    Timing,
    /// A turn longer than `MAX_TURN`.
    Long,
    /// The counters or a breakpoint on, or the core in a delay slot.
    Counters,
    /// The debugger's probe fitted beside the IOP.
    Probe,
    /// The lines or the field changed, or the DMAC wrote what the turn had read, during the turn.
    Event,
    /// An exception or interrupt taken during the turn.
    Exception,
    /// The frame's batch ended during the turn.
    Budget,
}

pub const REFUSALS: usize = 20;

impl Refusal {
    /// Whether the head waits longer before its next proof: not for what passes with time.
    fn penalised(self) -> bool {
        !matches!(self, Refusal::Event | Refusal::Budget | Refusal::Probe)
    }
}

/// What idle skipping did: the core's own counts, cycles and counts only, never an address (Sapphire_Native.md §47.6).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    /// Skips of the EE's waits, and the EE cycles and steps they passed.
    pub ee_skips: u64,
    pub ee_cycles: u64,
    pub ee_steps: u64,
    /// Skips of the IOP's waits, and the IOP cycles and instructions they passed.
    pub iop_skips: u64,
    pub iop_cycles: u64,
    pub iop_instructions: u64,
    /// Turns stepped to prove them, the proofs that held, and the skips the IOP side's wake ended early.
    pub attempts: u64,
    pub proofs: u64,
    pub woken: u64,
    /// Turns passed by bringing the board through each instruction's time with `run_to`, across an event or while a
    /// unit runs.
    pub replayed: u64,
    /// Skips ended because the DMAC wrote memory the turn reaches.
    pub written: u64,
    /// The refusals, by `Refusal`'s order, and the arrivals at heads waiting after each kind, about a turn run each.
    pub refused: [u64; REFUSALS],
    pub waited: [u64; REFUSALS],
}

#[derive(Clone, Copy, Default)]
struct Head {
    pc: u32,
    seen: u8,
    wait: u16,
    backoff: u8,
    why: u8,
}

/// The skipper: whether it is on, the heads it has met, and its counts. None of it is the machine's state.
pub struct Idle {
    pub on: bool,
    heads: Box<[Head]>,
    /// The DMAC's watch, kept between skips so that its lists keep their room.
    watch: Box<Watch>,
    pub counts: Counts,
}

impl Default for Idle {
    fn default() -> Self {
        Idle::new(false)
    }
}

impl Idle {
    pub fn new(on: bool) -> Idle {
        Idle { on, heads: vec![Head::default(); HEADS].into_boxed_slice(), watch: Box::default(), counts: Counts::default() }
    }

    /// Whether a step from `from` to `to`, leaving the core outside a delay slot, was a backward jump to a head.
    #[inline]
    pub fn backward(&self, from: u32, to: u32, in_delay: bool) -> bool {
        self.on && to <= from && from - to <= SPAN && !in_delay
    }

    /// The heads forgotten, as after a state is loaded.
    pub fn forget(&mut self) {
        self.heads.fill(Head::default());
    }
}

/// One step of the machine, through the dispatcher's interpreter when the recompiled tier runs, so that its verifier
/// follows.
fn step(sys: &mut System, blocks: &mut Option<&mut Blocks>) {
    let _ = match blocks {
        Some(b) => crate::blocks::interpret(sys, b).map(|_| ()),
        None => sys.step(),
    };
}

/// At an arrival at a head (`Idle::backward`): after `ARRIVALS` in a row, its turn proven by stepping it, and if it
/// repeats, the turns before the board's next event passed. The steps taken: at most `left` stepped, so that a frame's
/// batch ends where the plain run's does, and any number passed, none of which crosses an event.
pub fn arrive(sys: &mut System, idle: &mut Idle, mut blocks: Option<&mut Blocks>, left: u64) -> u64 {
    let head = sys.ee.pc;
    let at = (head as usize >> 2) & (HEADS - 1);
    let slot = &mut idle.heads[at];
    if slot.pc != head {
        *slot = Head { pc: head, seen: 0, wait: 0, backoff: 0, why: 0 };
    }
    if slot.seen < ARRIVALS {
        slot.seen += 1;
        return 0;
    }
    if slot.wait > 0 {
        slot.wait -= 1;
        idle.counts.waited[usize::from(slot.why)] += 1;
        return 0;
    }
    idle.counts.attempts += 1;
    let (steps, r) = attempt(sys, &mut blocks, left, &mut idle.counts, &mut idle.watch);
    let slot = &mut idle.heads[at];
    match r {
        Ok(()) => slot.backoff = 0,
        Err(e) => {
            idle.counts.refused[e as usize] += 1;
            slot.why = e as u8;
            if e.penalised() {
                slot.backoff = (slot.backoff + 1).min(MAX_BACKOFF);
                slot.wait = (1u16 << slot.backoff) - 1;
            }
        }
    }
    steps
}

/// The proof's turn stepped, then the skip; the steps taken and why nothing was passed. The DMAC's writes to what the
/// turn reaches are watched from the instruction that first reaches it, so that a turn that read memory the DMAC then
/// changed is not taken for one that repeats.
fn attempt(sys: &mut System, blocks: &mut Option<&mut Blocks>, left: u64, counts: &mut Counts, spare: &mut Box<Watch>) -> (u64, Result<(), Refusal>) {
    if let Err(e) = sys.ee.idle_eligible() {
        return (0, Err(e));
    }
    if sys.board.iop.probe.is_some() {
        return (0, Err(Refusal::Probe));
    }
    let a = sys.ee.mark();
    let mut watch = std::mem::take(spare);
    watch.ranges.clear();
    watch.hits.clear();
    sys.board.dmac.watch = Some(watch);
    let mut offsets = [0u64; MAX_TURN];
    let turn = prove(sys, blocks, left, &mut offsets);
    let watch = sys.board.dmac.watch.take().expect("the watch stays while the turn is stepped");
    let (steps, unpaired, traffic, registers) = match turn {
        Ok(t) => t,
        Err((steps, e)) => {
            *spare = watch;
            return (steps, Err(e));
        }
    };
    let offsets = &offsets[..steps as usize];
    let p = match sys.ee.period_from(&a, unpaired) {
        Ok(p) => p,
        Err(e) => {
            *spare = watch;
            return (steps, Err(e));
        }
    };
    counts.proofs += 1;
    let passed = skip(sys, &a, &p, offsets, traffic, !registers, watch, spare, counts);
    if passed > 0
        && let Some(b) = blocks
    {
        b.follow(sys, passed, "a skipped wait");
    }
    (steps + passed, Ok(()))
}

/// One turn from the head stepped and checked: every instruction one a turn may hold, the lines, the field and the
/// watched memory and the registers read unchanged after each step, no exception, and every store leaving its bytes as
/// it found them. The steps, each step's cycles from the head, the issue cycles that did not pair, the turn's traffic past
/// the caches and whether it reads a register; or the steps taken and the refusal.
fn prove(sys: &mut System, blocks: &mut Option<&mut Blocks>, left: u64, offsets: &mut [u64; MAX_TURN]) -> Result<(u64, u64, [u64; 3], bool), (u64, Refusal)> {
    let head = sys.ee.pc;
    let b = &sys.board;
    let (traffic, lines, fields) = (b.traffic, b.lines(), b.pcrtc.fields);
    let t1 = sys.ee.timing.cycles();
    let mut stores = [(Reach { vaddr: 0, bytes: 0, store: true }, [0u8; 16]); MAX_STORES];
    let mut registers = [(0u32, 0u32); MAX_STORES];
    let (mut n, mut stored, mut read, mut unpaired) = (0usize, 0usize, 0usize, 0u64);
    loop {
        let steps = n as u64;
        if steps == left {
            return Err((steps, Refusal::Budget));
        }
        if n == MAX_TURN {
            return Err((steps, Refusal::Long));
        }
        let pc = sys.ee.pc;
        let Some(word) = sys.ee.peek_fetch(&mut sys.board, pc) else { return Err((steps, Refusal::Device)) };
        let System { ee, board } = &mut *sys;
        match ee.idle_admits(word, &|p| board.pure_register(p).is_some()) {
            Err(e) => return Err((steps, e)),
            Ok(Some(Access::Register(p))) => {
                if !registers[..read].iter().any(|r| r.0 == p) {
                    if read == MAX_STORES {
                        return Err((steps, Refusal::Long));
                    }
                    registers[read] = (p, board.pure_register(p).expect("a pure register"));
                    read += 1;
                }
            }
            Ok(Some(Access::Memory(r))) => {
                if r.store && !stores[..stored].iter().any(|s| s.0 == r) {
                    if stored == MAX_STORES {
                        return Err((steps, Refusal::Long));
                    }
                    stores[stored] = (r, sys.ee.seen(&sys.board.map.rdram, r));
                    stored += 1;
                }
                if let Some(range) = range_of(&sys.ee, r) {
                    let w = sys.board.dmac.watch.as_mut().expect("the watch is set");
                    if !w.ranges.contains(&range) {
                        w.ranges.push(range);
                    }
                }
            }
            Ok(None) => {}
        }
        let taken = sys.ee.exceptions.iter().chain(&sys.ee.exceptions2).sum::<u64>();
        step(sys, blocks);
        offsets[n] = sys.ee.timing.cycles() - t1;
        n += 1;
        let steps = n as u64;
        unpaired += u64::from(!sys.ee.timing.paired());
        if sys.ee.exceptions.iter().chain(&sys.ee.exceptions2).sum::<u64>() != taken {
            return Err((steps, Refusal::Exception));
        }
        let b = &sys.board;
        if b.lines() != lines
            || b.pcrtc.fields != fields
            || b.dmac.watch.as_ref().is_some_and(|w| !w.hits.is_empty())
            || registers[..read].iter().any(|&(p, v)| b.pure_register(p) != Some(v))
        {
            return Err((steps, Refusal::Event));
        }
        if sys.ee.pc == head && !sys.ee.in_delay_slot() {
            break;
        }
    }
    let steps = n as u64;
    if stores[..stored].iter().any(|&(r, was)| sys.ee.seen(&sys.board.map.rdram, r) != was) {
        return Err((steps, Refusal::Store));
    }
    let traffic = std::array::from_fn(|i| sys.board.traffic[i] - traffic[i]);
    Ok((steps, unpaired, traffic, read > 0))
}

/// Where an access lands as the DMAC addresses it: physical memory, or the scratchpad.
fn range_of(ee: &crate::ee::Ee, r: Reach) -> Option<(bool, u32, u32)> {
    Some(match ee.place(r.vaddr)? {
        Place::Scratchpad(o) => (true, o, o + r.bytes),
        Place::Physical(p, _) => (false, p, p + r.bytes),
    })
}

/// The turns of period `p` passed from the head the core stands at, one turn after mark `a`, up to Compare's turn, the
/// board brought through each of their instructions' times as `System::step` brings it. While `run_to` would take its
/// quick path, the IOP alone is brought along, up to the board's next event (`drive_iop`); a turn across an event, or
/// while a unit runs, is `run_to` at each time. The skip ends at the first time the lines or the field change, the DMAC
/// writes what the turn reaches, or (in the quick path, before it is taken) the IOP side wakes the DMAC: the turn's
/// instructions up to that time then run on the core alone, shown the watched memory as it was before that time's write,
/// and the wake is taken as `run_to` takes it. A turn that reads a register is passed only while the quick path holds,
/// since a register's change cannot be shown to the turn's instructions as it was (`replay` false).
#[allow(clippy::too_many_arguments)]
fn skip(sys: &mut System, a: &Mark, p: &Period, offsets: &[u64], traffic: [u64; 3], replay: bool, mut watch: Box<Watch>, spare: &mut Box<Watch>, counts: &mut Counts) -> u64 {
    let t2 = sys.ee.timing.cycles();
    let turns = sys.ee.turns_before_compare(p.span).min(MAX_TURNS);
    if turns == 0 {
        *spare = watch;
        return 0;
    }
    let (lines, fields) = (sys.board.lines(), sys.board.pcrtc.fields);
    watch.hits.clear();
    sys.board.dmac.watch = Some(watch);
    let (mut k, mut stop, mut quick) = (0u64, None, false);
    while k < turns {
        let t = t2 + k * p.span;
        let next = sys.board.next_event();
        if sys.board.quiet() && next > t + p.span {
            let n = ((next - 1 - t) / p.span).min(turns - k);
            let (whole, woken) = drive_iop(sys, t, p.span, offsets, n, counts);
            k += whole;
            quick = true;
            if let Some((i, at)) = woken {
                stop = Some((i, at, true));
                break;
            }
            continue;
        }
        if !replay {
            break;
        }
        quick = false;
        counts.replayed += 1;
        let mut ended = None;
        for (i, &o) in offsets.iter().enumerate() {
            let at = t + o;
            let System { ee, board } = &mut *sys;
            board.run_to(at, &mut ee.spram[..]);
            if board.lines() != lines || board.pcrtc.fields != fields || board.dmac.watch.as_ref().is_some_and(|w| !w.hits.is_empty()) {
                ended = Some((i + 1, at));
                break;
            }
        }
        if let Some((i, at)) = ended {
            stop = Some((i, at, false));
            break;
        }
        k += 1;
    }
    let watch = sys.board.dmac.watch.take().expect("the watch stays while the turns are passed");
    sys.ee.pass(a, p, k);
    for (n, d) in sys.board.traffic.iter_mut().zip(traffic) {
        *n += k * d;
    }
    let mut passed = k * p.steps;
    let end = match stop {
        Some((i, at, woke)) => {
            let after = undo(sys, &watch.hits);
            for _ in 0..i {
                sys.ee.step_alone(&mut sys.board);
            }
            redo(sys, &after);
            counts.written += u64::from(!watch.hits.is_empty());
            debug_assert_eq!(sys.ee.timing.cycles(), at, "the stop's time is the step's");
            *spare = watch;
            passed += i as u64;
            counts.woken += u64::from(woke);
            at
        }
        None => {
            *spare = watch;
            t2 + k * p.span
        }
    };
    let System { ee, board } = sys;
    if quick {
        board.now = board.now.max(end);
        board.idle_through(end);
        if matches!(stop, Some((_, _, true))) {
            board.quick_wake(&mut ee.spram[..]);
        }
    }
    let (i0, i1, c0) = board.lines();
    (ee.int0, ee.int1, ee.cpcond0) = (i0, i1, c0);
    if passed > 0 {
        counts.ee_skips += 1;
        counts.ee_cycles += end - t2;
        counts.ee_steps += passed;
    }
    passed
}

/// The watched quadwords the DMAC wrote put back as they were before, the earliest value of each; their values after.
fn undo(sys: &mut System, hits: &[(bool, u32, u128)]) -> Vec<(bool, u32, u128)> {
    use crate::dmac::Memory;
    let read = |sys: &mut System, spr: bool, at: u32| {
        if spr { u128::from_le_bytes(sys.ee.spram[at as usize..at as usize + 16].try_into().unwrap()) } else { sys.board.map.read_q(at).unwrap_or(0) }
    };
    let mut after: Vec<(bool, u32, u128)> = Vec::new();
    for &(spr, at, _) in hits {
        if !after.iter().any(|x| (x.0, x.1) == (spr, at)) {
            let v = read(sys, spr, at);
            after.push((spr, at, v));
        }
    }
    for &(spr, at, old) in hits.iter().rev() {
        put(sys, spr, at, old);
    }
    after
}

fn redo(sys: &mut System, after: &[(bool, u32, u128)]) {
    for &(spr, at, v) in after {
        put(sys, spr, at, v);
    }
}

fn put(sys: &mut System, spr: bool, at: u32, v: u128) {
    use crate::dmac::Memory;
    if spr {
        sys.ee.spram[at as usize..at as usize + 16].copy_from_slice(&v.to_le_bytes());
    } else {
        sys.board.map.write_q(at, v);
    }
}

/// The IOP brought through the times of `turns` turns from `t2`, instruction `i` of turn `k` ending at `t2 + k·span +
/// offsets[i]`, as the quick path of `run_to` brings it at each: a call at a time the IOP has already reached does
/// nothing, so only the others are made (every one while the IOP is held, which moves its units at each). The whole
/// turns run, and the instructions and time of the turn in which its side woke the DMAC, if it did. Where the IOP is
/// itself seen to wait in a pure loop between two calls, its turns are passed too (`pass_iop`).
fn drive_iop(sys: &mut System, t2: u64, span: u64, offsets: &[u64], turns: u64, counts: &mut Counts) -> (u64, Option<(usize, u64)>) {
    let iop = &mut sys.board.iop;
    let woke = |iop: &crate::iop::Iop| iop.bus.sif.wake_ee_pending();
    if iop.held {
        for k in 0..turns {
            for (i, &o) in offsets.iter().enumerate() {
                let t = t2 + k * span + o;
                iop.catch_up(t);
                if woke(iop) {
                    return (k, Some((i + 1, t)));
                }
            }
        }
        return (turns, None);
    }
    let end = t2 + turns * span;
    let mut mark: Option<IopMark> = None;
    let mut prev = iop.cpu.regs.pc;
    loop {
        let x = iop.bus.now * IOP;
        let (k, i) = if x < t2 + offsets[0] {
            (0, 0)
        } else {
            let d = x - t2;
            let r = d % span;
            (d / span, offsets.iter().position(|&o| o > r).expect("the turn's last time is its span"))
        };
        if k >= turns {
            return (turns, None);
        }
        let t = t2 + k * span + offsets[i];
        iop.catch_up(t);
        assert!(iop.bus.now * IOP >= t, "the IOP reaches each time it is brought to, no probe being fitted");
        if woke(iop) {
            return (k, Some((i + 1, t)));
        }
        let pc = iop.cpu.regs.pc;
        // A head is an address the IOP comes back to between two calls, outside a delay slot.
        if !iop.cpu.regs.branch.slot && !iop.tty.pending() {
            if let Some(m) = mark.as_ref().filter(|m| m.regs.pc == pc) {
                if let Some((cycles, steps)) = m.period(iop) {
                    pass_iop(iop, m, cycles, steps, end, counts);
                }
                mark = Some(IopMark::take(iop));
            } else if pc <= prev && prev - pc <= SPAN {
                mark = Some(IopMark::take(iop));
            }
        }
        prev = pc;
    }
}

/// Everything a turn of the IOP's pure loop may change, taken at its head between two of the board's calls.
struct IopMark {
    regs: blackmoon_r3000::Registers,
    hilo_ready: u64,
    now: u64,
    instructions: u64,
    /// The bus's writes, accesses past memory and refills, the exceptions taken, the calls the console's hook caught
    /// and the bytes it kept, and the bus's next event.
    counts: [u64; 6],
    next: u64,
}

impl IopMark {
    fn take(iop: &crate::iop::Iop) -> IopMark {
        let b = &iop.bus;
        IopMark {
            regs: iop.cpu.regs,
            hilo_ready: iop.cpu.hilo_ready,
            now: b.now,
            instructions: iop.instructions,
            counts: [b.writes, b.io, b.fills, iop.exceptions.iter().sum(), iop.tty.calls.iter().sum(), iop.tty.len as u64],
            next: b.next_event(),
        }
    }

    /// The turn from this mark to the IOP now, if it repeats: every register as it was, nothing written, nothing but
    /// memory read, no line refilled, no exception and no event of the bus's units, the multiplier's ready cycle
    /// moved by the turn or already past. Its cycles and instructions.
    fn period(&self, iop: &crate::iop::Iop) -> Option<(u64, u64)> {
        let now = IopMark::take(iop);
        let cycles = now.now.checked_sub(self.now).filter(|&c| c > 0)?;
        let steps = now.instructions - self.instructions;
        let hilo = now.hilo_ready == self.hilo_ready + cycles || (now.hilo_ready == self.hilo_ready && self.hilo_ready <= self.now);
        (now.regs == self.regs && now.counts == self.counts && now.next == self.next && steps > 0 && hilo).then_some((cycles, steps))
    }
}

/// The IOP's turns from mark `m` passed at once: as many whole turns as begin before its bus's next event and leave a
/// call of the board before `end`, the EE's last time, to bring it the rest of the way as the plain run does, so that
/// what each call keeps of its own (the units' sync time, the run's horizon) is that call's.
fn pass_iop(iop: &mut crate::iop::Iop, m: &IopMark, cycles: u64, steps: u64, end: u64, counts: &mut Counts) {
    let x = iop.bus.now;
    let limit = (m.next / IOP).min(end.div_ceil(IOP) - 1);
    let k = limit.saturating_sub(x) / cycles;
    if k == 0 {
        return;
    }
    iop.bus.now += k * cycles;
    if iop.cpu.hilo_ready != m.hilo_ready {
        iop.cpu.hilo_ready += k * cycles;
    }
    iop.instructions += k * steps;
    counts.iop_skips += 1;
    counts.iop_cycles += k * cycles;
    counts.iop_instructions += k * steps;
}
