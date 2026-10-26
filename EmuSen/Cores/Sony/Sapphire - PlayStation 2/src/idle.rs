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
use crate::ee::idle::{Mark, Period};

/// The farthest a backward jump may reach to make a loop's head, and the longest turn proven, in instructions.
pub const SPAN: u32 = 0x400;
pub const MAX_TURN: usize = 64;
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
    /// A store.
    Store,
    /// A load from a device or a memory another unit writes between events (IOP RAM, the registers).
    Device,
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
    /// The GIF busy (a path's transfer, a kick, its FIFO), a VIF or a VU running, the IOP side's wake pending, or the
    /// debugger's probe fitted beside the IOP: `run_to` would not take its quick path.
    Gif,
    Vif,
    Vu,
    Wake,
    Probe,
    /// The board's next event, or the IOP side's wake, reached during the turn.
    Event,
    /// An exception or interrupt taken during the turn.
    Exception,
    /// The frame's batch ended during the turn.
    Budget,
}

pub const REFUSALS: usize = 17;

impl Refusal {
    /// Whether the head waits longer before its next proof: not for what passes with time.
    fn penalised(self) -> bool {
        !matches!(self, Refusal::Event | Refusal::Budget | Refusal::Gif | Refusal::Vif | Refusal::Vu | Refusal::Wake | Refusal::Probe)
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
    /// The refusals, by `Refusal`'s order.
    pub refused: [u64; REFUSALS],
}

#[derive(Clone, Copy, Default)]
struct Head {
    pc: u32,
    seen: u8,
    wait: u16,
    backoff: u8,
}

/// The skipper: whether it is on, the heads it has met, and its counts. None of it is the machine's state.
pub struct Idle {
    pub on: bool,
    heads: Box<[Head]>,
    pub counts: Counts,
}

impl Default for Idle {
    fn default() -> Self {
        Idle::new(false)
    }
}

impl Idle {
    pub fn new(on: bool) -> Idle {
        Idle { on, heads: vec![Head::default(); HEADS].into_boxed_slice(), counts: Counts::default() }
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
        *slot = Head { pc: head, seen: 0, wait: 0, backoff: 0 };
    }
    if slot.seen < ARRIVALS {
        slot.seen += 1;
        return 0;
    }
    if slot.wait > 0 {
        slot.wait -= 1;
        return 0;
    }
    idle.counts.attempts += 1;
    let (steps, r) = attempt(sys, &mut blocks, left, &mut idle.counts);
    let slot = &mut idle.heads[at];
    match r {
        Ok(()) => slot.backoff = 0,
        Err(e) => {
            idle.counts.refused[e as usize] += 1;
            if e.penalised() {
                slot.backoff = (slot.backoff + 1).min(MAX_BACKOFF);
                slot.wait = (1u16 << slot.backoff) - 1;
            }
        }
    }
    steps
}

/// The proof's turn stepped, then the skip; the steps taken and why nothing was passed.
fn attempt(sys: &mut System, blocks: &mut Option<&mut Blocks>, left: u64, counts: &mut Counts) -> (u64, Result<(), Refusal>) {
    if let Err(e) = sys.ee.idle_eligible() {
        return (0, Err(e));
    }
    let b = &sys.board;
    let busy = [
        (b.iop.probe.is_some(), Refusal::Probe),
        (!b.gif.quiet(), Refusal::Gif),
        (!(b.vif[0].idle() && b.vif[1].idle()), Refusal::Vif),
        (!b.vpu.idle(), Refusal::Vu),
        (b.iop.bus.sif.wake_ee_pending(), Refusal::Wake),
    ];
    if let Some(&(_, r)) = busy.iter().find(|x| x.0) {
        return (0, Err(r));
    }
    debug_assert!(b.quiet());
    let head = sys.ee.pc;
    let a = sys.ee.mark();
    let (wakes, traffic, next) = (b.wakes, b.traffic, b.next_event());
    let t1 = sys.ee.timing.cycles();
    let mut offsets = [0u64; MAX_TURN];
    let (mut steps, mut unpaired) = (0usize, 0u64);
    loop {
        if steps as u64 == left {
            return (steps as u64, Err(Refusal::Budget));
        }
        if steps == MAX_TURN {
            return (steps as u64, Err(Refusal::Long));
        }
        let pc = sys.ee.pc;
        let Some(word) = sys.ee.peek_fetch(&mut sys.board, pc) else { return (steps as u64, Err(Refusal::Device)) };
        if let Err(e) = sys.ee.idle_admits(word) {
            return (steps as u64, Err(e));
        }
        let taken = sys.ee.exceptions.iter().chain(&sys.ee.exceptions2).sum::<u64>();
        step(sys, blocks);
        let t = sys.ee.timing.cycles();
        offsets[steps] = t - t1;
        steps += 1;
        unpaired += u64::from(!sys.ee.timing.paired());
        if sys.ee.exceptions.iter().chain(&sys.ee.exceptions2).sum::<u64>() != taken {
            return (steps as u64, Err(Refusal::Exception));
        }
        if t >= next || sys.board.wakes != wakes {
            return (steps as u64, Err(Refusal::Event));
        }
        if sys.ee.pc == head && !sys.ee.in_delay_slot() {
            break;
        }
    }
    let p = match sys.ee.period_from(&a, unpaired) {
        Ok(p) => p,
        Err(e) => return (steps as u64, Err(e)),
    };
    counts.proofs += 1;
    let per_turn = std::array::from_fn::<u64, 3, _>(|i| sys.board.traffic[i] - traffic[i]);
    let passed = skip(sys, &a, &p, &offsets[..steps], per_turn, counts);
    if passed > 0
        && let Some(b) = blocks
    {
        b.follow(sys, passed, "a skipped wait");
    }
    (steps as u64 + passed, Ok(()))
}

/// The turns of period `p` before the board's next event passed from the head the core stands at, one turn after mark
/// `a`: the IOP brought through each of their instructions' times, the skip ending at the first time its side wakes the
/// DMAC, where the turn's instructions up to that time run on the core alone and the wake is taken as `run_to` takes it.
/// The steps passed.
fn skip(sys: &mut System, a: &Mark, p: &Period, offsets: &[u64], traffic: [u64; 3], counts: &mut Counts) -> u64 {
    let t2 = sys.ee.timing.cycles();
    let next = sys.board.next_event();
    if next <= t2 {
        return 0;
    }
    let turns = ((next - 1 - t2) / p.span).min(sys.ee.turns_before_compare(p.span)).min(MAX_TURNS);
    if turns == 0 {
        return 0;
    }
    let (whole, woken) = drive_iop(sys, t2, p.span, offsets, turns, counts);
    sys.ee.pass(a, p, whole);
    for (n, d) in sys.board.traffic.iter_mut().zip(traffic) {
        *n += whole * d;
    }
    let mut passed = whole * p.steps;
    let end = match woken {
        Some((i, t)) => {
            for _ in 0..i {
                sys.ee.step_alone(&mut sys.board);
            }
            debug_assert_eq!(sys.ee.timing.cycles(), t, "the wake's time is the step's");
            passed += i as u64;
            counts.woken += 1;
            t
        }
        None => t2 + whole * p.span,
    };
    let System { ee, board } = sys;
    board.now = board.now.max(end);
    board.idle_through(end);
    if woken.is_some() {
        board.quick_wake(&mut ee.spram[..]);
    }
    let (i0, i1, c0) = board.lines();
    (ee.int0, ee.int1, ee.cpcond0) = (i0, i1, c0);
    counts.ee_skips += 1;
    counts.ee_cycles += end - t2;
    counts.ee_steps += passed;
    passed
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
