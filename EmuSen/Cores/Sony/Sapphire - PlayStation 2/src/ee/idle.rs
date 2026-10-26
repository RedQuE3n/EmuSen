//! What idle skipping needs of the EE core (stage S33, Sapphire_Native.md §47): which instructions a wait may hold, a
//! mark of everything a turn of a loop can change, the comparison of two marks a turn apart, and further turns passed at
//! once. The machine's side, the proof's steps and the skip, is `crate::idle`.

use super::cop0::{self, debug, perf};
use super::map::{self, Region};
use super::timing::Timing;
use super::observe::Reach;
use super::{Bus, Ee, Place, ops};
use crate::idle::Refusal;

/// Everything a turn of a pure loop may change, taken at the loop's head.
#[derive(Clone, Debug)]
pub(crate) struct Mark {
    gpr: [u128; 32],
    hi: u128,
    lo: u128,
    sa: u64,
    fpr: [u32; 32],
    acc: u32,
    fcr31: u32,
    pc: u32,
    next_pc: u32,
    cop0: [u32; 32],
    debug: [u32; 8],
    pccr: u32,
    pcr: [u32; 2],
    lines: (bool, bool, bool),
    group: u8,
    cycles_seen: u64,
    timing: Timing,
    exceptions: u64,
    pub(crate) modes: [u64; 5],
    fills: u64,
}

/// What a turn's load or store reaches: memory, or a register whose reads change nothing (`Board::pure_register`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Access {
    Memory(Reach),
    Register(u32),
}

/// A turn proven to repeat: its cycles, its instructions, and the issue cycles that did not pair, which move Random.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Period {
    pub span: u64,
    pub steps: u64,
    pub unpaired: u64,
}

/// Random after `n` issue cycles that do not pair, from `r` with Wired at `wired`, as each step's advance moves it.
pub(crate) fn random_after(r: u32, wired: u32, n: u64) -> u32 {
    if n == 0 {
        return r;
    }
    let (r, n) = if r <= wired || r > 47 { (47, n - 1) } else { (r, n) };
    if wired >= 47 {
        return 47;
    }
    let cycle = u64::from(48 - wired);
    47 - ((u64::from(47 - r) + n) % cycle) as u32
}

impl Ee {
    /// Whether a turn may hold the instruction `word` at the PC: no coprocessor 0 or 2 move, no exception of its own, a
    /// load only from RDRAM, the scratchpad or the boot ROM, which nothing but the DMAC changes between the board's
    /// events, or from a register `pure` names, and a store only to RDRAM or the scratchpad. A memory access's reach is
    /// given, so that the DMAC's writes there can be watched and a store seen to leave the bytes as it found them, and a
    /// register's address, so that its value can be seen not to change. Read before the step, with nothing changed but
    /// the decode memo.
    pub(crate) fn idle_admits(&mut self, word: u32, pure: &dyn Fn(u32) -> bool) -> Result<Option<Access>, Refusal> {
        let op = ops::get(self.entry(word));
        match op.kind {
            ops::Kind::Cop2 => return Err(Refusal::Coprocessor),
            ops::Kind::Fpu if !ops::is_load(op) && !ops::is_store(op) => return Ok(None),
            _ => {}
        }
        if op.name == "pref" {
            return Ok(None);
        }
        if ops::is_load(op) || ops::is_store(op) {
            let reach = self.reach(word).ok_or(Refusal::Control)?;
            let memory = match self.place(reach.vaddr) {
                Some(Place::Scratchpad(_)) => true,
                Some(Place::Physical(p, _)) => map::region(p) == Region::Rdram || (!reach.store && map::region(p) == Region::BootRom),
                None => false,
            };
            return match self.place(reach.vaddr) {
                _ if memory => Ok(Some(Access::Memory(reach))),
                Some(Place::Physical(p, _)) if !reach.store && reach.bytes <= 4 && pure(p) => Ok(Some(Access::Register(p))),
                Some(Place::Physical(p, _)) => Err(match p {
                    0x1000_0000..0x1000_2000 => Refusal::Timer,
                    0x1000_2000..0x1000_8000 => Refusal::Unit,
                    0x1000_F200..0x1000_F300 => Refusal::Sif,
                    0x1000_0000..0x1100_0000 => Refusal::Device,
                    0x1100_0000..0x1200_0000 => Refusal::Vu,
                    0x1200_0000..0x1400_0000 => Refusal::Gs,
                    0x1C00_0000..0x1C20_0000 => Refusal::Window,
                    _ => Refusal::Elsewhere,
                }),
                _ => Err(Refusal::Elsewhere),
            };
        }
        let control = matches!(op.category, super::timing::Category::Cop0 | super::timing::Category::Eret)
            || matches!(op.name, "syscall" | "break" | "cache" | "reserved" | "later")
            || (op.name.starts_with('t') && op.category == super::timing::Category::Alu);
        if control { Err(Refusal::Control) } else { Ok(None) }
    }

    /// The bytes a store's reach covers as the core sees them, a valid data cache line over RDRAM, the scratchpad.
    pub(crate) fn seen(&self, rdram: &[u8], reach: Reach) -> [u8; 16] {
        let mut v = [0; 16];
        for (i, b) in v.iter_mut().enumerate().take(reach.bytes as usize) {
            *b = self.place(reach.vaddr.wrapping_add(i as u32)).and_then(|p| self.peek_byte(rdram, &[], p)).unwrap_or(0);
        }
        v
    }

    /// Whether the core may be at a loop's head whose turns are passed: outside a delay slot, the counters and the
    /// breakpoints off.
    pub(crate) fn idle_eligible(&self) -> Result<(), Refusal> {
        let bpc = self.debug[debug::BPC];
        if self.delay || self.pccr & perf::CTE != 0 || bpc & (debug::IAE | debug::DRE | debug::DWE) != 0 || self.timing.instructions == 0 {
            return Err(Refusal::Counters);
        }
        Ok(())
    }

    pub(crate) fn mark(&self) -> Mark {
        Mark {
            gpr: self.gpr,
            hi: self.hi,
            lo: self.lo,
            sa: self.sa,
            fpr: self.fpr,
            acc: self.acc,
            fcr31: self.fcr31,
            pc: self.pc,
            next_pc: self.next_pc,
            cop0: self.cop0,
            debug: self.debug,
            pccr: self.pccr,
            pcr: self.pcr,
            lines: (self.int0, self.int1, self.cpcond0),
            group: self.group,
            cycles_seen: self.cycles_seen,
            timing: self.timing.clone(),
            exceptions: self.exceptions.iter().chain(&self.exceptions2).sum(),
            modes: self.modes,
            fills: self.fills,
        }
    }

    /// The turn from mark `a` to now, if it repeats: every register as it was, nothing refilled into a cache, the issue
    /// model `a`'s a turn later shifted in time, Count and the cycles it saw moved by the turn's cycles and Random by its
    /// issue cycles; Cause unchanged, so that no interrupt was asked for on the way.
    pub(crate) fn period_from(&self, a: &Mark, unpaired: u64) -> Result<Period, Refusal> {
        let b = self.mark();
        if (b.gpr, b.hi, b.lo, b.sa, b.fpr, b.acc, b.fcr31) != (a.gpr, a.hi, a.lo, a.sa, a.fpr, a.acc, a.fcr31) {
            return Err(Refusal::Changed);
        }
        if (b.pc, b.next_pc, b.debug, b.pccr, b.pcr, b.lines, b.group, b.exceptions) != (a.pc, a.next_pc, a.debug, a.pccr, a.pcr, a.lines, a.group, a.exceptions) {
            return Err(Refusal::Changed);
        }
        if b.fills != a.fills {
            return Err(Refusal::Cache);
        }
        let span = b.timing.period_from(&a.timing).ok_or(Refusal::Timing)?;
        let wired = a.cop0[cop0::WIRED] & 63;
        let moved = |r: usize| matches!(r, cop0::COUNT | cop0::RANDOM);
        if (0..32).any(|r| !moved(r) && a.cop0[r] != b.cop0[r])
            || b.cop0[cop0::COUNT] != a.cop0[cop0::COUNT].wrapping_add(span as u32)
            || b.cycles_seen != a.cycles_seen + span
            || b.cop0[cop0::RANDOM] != random_after(a.cop0[cop0::RANDOM], wired, unpaired)
        {
            return Err(Refusal::Timing);
        }
        Ok(Period { span, steps: b.timing.instructions - a.timing.instructions, unpaired })
    }

    /// The most turns that may be passed before Count would reach Compare and ask for IP7, which Cause already holding
    /// IP7 does not limit.
    pub(crate) fn turns_before_compare(&self, span: u64) -> u64 {
        if self.cop0[cop0::CAUSE] & cop0::cause::IP7 != 0 {
            return u64::MAX;
        }
        u64::from(self.cop0[cop0::COMPARE].wrapping_sub(self.cop0[cop0::COUNT]).wrapping_sub(1)) / span
    }

    /// `k` more turns of the period proven from mark `a` to now: the issue model, Count, the cycles it saw, Random and
    /// the steps by mode as `k` turns would leave them; nothing else a turn changes.
    pub(crate) fn pass(&mut self, a: &Mark, p: &Period, k: u64) {
        self.timing.pass(&a.timing, k);
        self.cycles_seen += k * p.span;
        self.cop0[cop0::COUNT] = self.cop0[cop0::COUNT].wrapping_add((k * p.span) as u32);
        let wired = self.cop0[cop0::WIRED] & 63;
        self.cop0[cop0::RANDOM] = random_after(self.cop0[cop0::RANDOM], wired, k * p.unpaired);
        for (m, &was) in self.modes.iter_mut().zip(&a.modes) {
            *m += k * (*m - was);
        }
    }

    /// One step of the core alone, the board not brought along: a turn's instruction before the board's time the skip
    /// has already reached.
    pub(crate) fn step_alone(&mut self, bus: &mut dyn Bus) {
        let _ = self.step(bus);
    }
}

#[cfg(test)]
mod tests {
    use super::random_after;

    fn step(r: u32, wired: u32) -> u32 {
        if r <= wired || r > 47 { 47 } else { r - 1 }
    }

    #[test]
    fn random_after_is_the_advances_step_repeated() {
        for wired in [0, 1, 5, 30, 46, 47, 50] {
            for r in [0, 1, 4, 5, 6, 20, 46, 47, 48, 63] {
                let mut x = r;
                for n in 0..200u64 {
                    assert_eq!(random_after(r, wired, n), x, "r {r} wired {wired} n {n}");
                    x = step(x, wired);
                }
            }
        }
    }
}
