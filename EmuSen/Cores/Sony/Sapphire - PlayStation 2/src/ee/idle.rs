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

    /// The turn from mark `a` to now, if it repeats: every register as it was but the inductions `moved`, nothing refilled
    /// into a cache, the issue model `a`'s a turn later shifted in time, Count and the cycles it saw moved by the turn's
    /// cycles and Random by its issue cycles; Cause unchanged, so that no interrupt was asked for on the way.
    pub(crate) fn period_from(&self, a: &Mark, unpaired: u64, moved: u32) -> Result<Period, Refusal> {
        let b = self.mark();
        let kept = |m: &Mark| std::array::from_fn::<u128, 32, _>(|r| if moved >> r & 1 == 1 { 0 } else { m.gpr[r] });
        if (kept(&b), b.hi, b.lo, b.sa, b.fpr, b.acc, b.fcr31) != (kept(a), a.hi, a.lo, a.sa, a.fpr, a.acc, a.fcr31) {
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

/// The inductions of a turn being checked (Sapphire_Native.md §47.5): the registers that move from turn to turn, the
/// words of memory that do, and for each register holding a value taken from one of them at this point of the turn, the
/// induction it was taken from and its width. A value of width 32 is a sign extension of its low word, moved by 32-bit
/// additions; one of width 64 by 64-bit additions; each is its induction plus a constant, so that a turn moves it by its
/// induction's step. The comparisons the turn makes of such values with values that do not move are kept, to bound the
/// turns that may be passed by the first turn at which one would change its answer.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Taint {
    /// Per register: none, or its induction (a register's number, or 32 plus a word's index) and its width.
    regs: [Option<(u8, u8)>; 32],
    /// The width each register induction was first read at, zero while unread.
    read_as: [u8; 32],
    /// Each word as (virtual address, physical address, width in bits).
    words: [(u32, u32, u8); MAX_WORDS],
    n: usize,
    tests: [Test; MAX_TESTS],
    t: usize,
}

/// A comparison: the induction, the width (zero for its induction's width at the turn's end), whether signed, the value
/// now, the value compared with (plus one where the induction stands on the right of a less-than), and whether it was an
/// equality.
#[derive(Clone, Copy, Debug, Default)]
struct Test {
    source: u8,
    width: u8,
    signed: bool,
    value: u64,
    with: u64,
    after: bool,
    equal: bool,
}

/// The most words of memory a turn's inductions may be, and the most comparisons of them it may make.
pub(crate) const MAX_WORDS: usize = 4;
const MAX_TESTS: usize = 8;

impl Taint {
    /// The registers `moved` as inductions of their own, of a width not yet known.
    pub(crate) fn new(moved: u32) -> Taint {
        let mut regs = [None; 32];
        for (r, t) in regs.iter_mut().enumerate().skip(1) {
            if moved >> r & 1 == 1 {
                *t = Some((r as u8, 0));
            }
        }
        Taint { regs, read_as: [0; 32], words: [(0, 0, 0); MAX_WORDS], n: 0, tests: [Test::default(); MAX_TESTS], t: 0 }
    }

    pub(crate) fn add_word(&mut self, vaddr: u32, paddr: u32, bits: u8) -> bool {
        if self.n == MAX_WORDS {
            return false;
        }
        self.words[self.n] = (vaddr, paddr, bits);
        self.n += 1;
        true
    }

    fn word(&self, paddr: u32) -> Option<usize> {
        self.words[..self.n].iter().position(|w| w.1 == paddr)
    }

    fn overlaps(&self, paddr: u32, bytes: u32) -> bool {
        self.words[..self.n].iter().any(|w| paddr < w.1 + u32::from(w.2 / 8) && paddr + bytes > w.1)
    }

    fn get(&self, r: usize) -> Option<(u8, u8)> {
        if r == 0 { None } else { self.regs[r] }
    }

    /// Register `r` read as a value of width `w`: its induction, the width fixed for an induction of its own not yet
    /// read; none for a value that does not move, an error for a width other than its own.
    fn read(&mut self, r: usize, w: u8) -> Result<Option<u8>, Refusal> {
        let Some((src, width)) = self.get(r) else { return Ok(None) };
        let width = if width == 0 {
            let s = usize::from(src);
            if self.read_as[s] == 0 {
                self.read_as[s] = w;
            }
            self.read_as[s]
        } else {
            width
        };
        if width == w { Ok(Some(src)) } else { Err(Refusal::InductionOp) }
    }

    fn set(&mut self, r: usize, v: Option<(u8, u8)>) {
        if r != 0 {
            self.regs[r] = v;
        }
    }

    fn test(&mut self, t: Test) -> Result<(), Refusal> {
        if self.t == MAX_TESTS {
            return Err(Refusal::InductionBranch);
        }
        self.tests[self.t] = t;
        self.t += 1;
        Ok(())
    }

    /// Each induction register's width at the turn's end: the width of the value it holds, which must be its own
    /// induction's or another's, the width it was read at; none where a moved register holds no induction's value.
    pub(crate) fn widths(&self, moved: u32) -> Option<[u8; 32]> {
        let mut w = [0u8; 32];
        for r in (1..32).filter(|r| moved >> r & 1 == 1) {
            let (_, width) = self.regs[r]?;
            let width = match (width, self.read_as[r]) {
                (0, 0) => 64,
                (0, w) => w,
                (w, _) => w,
            };
            if self.read_as[r] != 0 && self.read_as[r] != width {
                return None;
            }
            w[r] = width;
        }
        Some(w)
    }

    /// The most turns after this one through which every comparison keeps its answer, each induction moving by its step
    /// (`steps`, by register, then by word) and none passing its width's range, where its order would wrap.
    pub(crate) fn turns(&self, steps: &[u64; 32 + MAX_WORDS], widths: &[u8; 32]) -> u64 {
        let mut most = u64::MAX;
        for t in &self.tests[..self.t] {
            let source = usize::from(t.source);
            let w = u32::from(if t.width != 0 { t.width } else if source < 32 { widths[source] } else { self.words[source - 32].2 });
            let d = steps[source];
            let signed_step = if w == 32 { i128::from(d as u32 as i32) } else { i128::from(d as i64) };
            let as_domain = |v: u64| -> i128 {
                match (w, t.signed) {
                    (32, true) => i128::from(v as u32 as i32),
                    (32, false) => i128::from(v as u32),
                    (_, true) => i128::from(v as i64),
                    _ => i128::from(v),
                }
            };
            let (lo, hi): (i128, i128) = match (w, t.signed) {
                (32, true) => (i128::from(i32::MIN), i128::from(i32::MAX)),
                (32, false) => (0, i128::from(u32::MAX)),
                (_, true) => (i128::from(i64::MIN), i128::from(i64::MAX)),
                _ => (0, i128::from(u64::MAX)),
            };
            let (s, x) = (as_domain(t.value), as_domain(t.with) + i128::from(t.after));
            if w == 32 && (t.value != t.value as u32 as i32 as i64 as u64 || t.with != t.with as u32 as i32 as i64 as u64) {
                return 0;
            }
            if signed_step == 0 {
                continue;
            }
            let wrap = if signed_step > 0 { (hi - s) / signed_step + 1 } else { (s - lo) / -signed_step + 1 };
            let flip = if t.equal {
                if s == x {
                    1
                } else if (x - s) % signed_step == 0 && (x - s) / signed_step > 0 {
                    (x - s) / signed_step
                } else {
                    i128::MAX
                }
            } else if signed_step > 0 {
                if s < x { (x - s + signed_step - 1) / signed_step } else { i128::MAX }
            } else if s < x {
                i128::MAX
            } else {
                (s - x) / -signed_step + 1
            };
            let first = flip.min(wrap).max(1) - 1;
            most = most.min(u64::try_from(first).unwrap_or(u64::MAX));
        }
        most
    }
}

impl Ee {
    /// The physical address of a word a store reaches when a loop's count may be kept there: four or eight aligned bytes
    /// of RDRAM in a valid line of the data cache, written back, so that no unit sees the word while the line stays.
    pub(crate) fn idle_word(&self, reach: Reach) -> Option<u32> {
        use super::memory::Mode;
        let Some(Place::Physical(p, m)) = self.place(reach.vaddr) else { return None };
        let cached = matches!(reach.bytes, 4 | 8) && reach.vaddr & (reach.bytes - 1) == 0 && map::region(p) == Region::Rdram && self.data_mode(m) == Mode::Cached;
        (cached && self.dcache.hit(super::cache::DCache::set(reach.vaddr), p).is_some()).then_some(p)
    }

    /// Before the step of `word` in a second turn, the inductions' values kept out of the turn's path, its addresses and
    /// every effect but the inductions themselves: they may only be loaded whole from their word, moved, added to or less
    /// a value that does not move at their own width, compared, and stored whole back into an induction word; the taint
    /// moved as the instruction will move it and each comparison noted with the values it makes now.
    pub(crate) fn idle_taint(&mut self, word: u32, t: &mut Taint) -> Result<(), Refusal> {
        use super::timing::Category;
        let op = ops::get(self.entry(word));
        let (rs, rt, rd) = (ops::rs(word), ops::rt(word), ops::rd(word));
        let tainted = |r: usize| t.get(r).is_some();
        if op.kind != ops::Kind::Cpu {
            if tainted(rs) || tainted(rt) {
                return Err(Refusal::InductionOp);
            }
            if matches!(op.name, "mfc1" | "cfc1") {
                t.set(rt, None);
            }
            return Ok(());
        }
        if ops::is_load(op) || ops::is_store(op) {
            if tainted(rs) {
                return Err(Refusal::InductionAddress);
            }
            let reach = self.reach(word).ok_or(Refusal::Control)?;
            let paddr = match self.place(reach.vaddr) {
                Some(Place::Physical(p, _)) => Some(p),
                _ => None,
            };
            let at = paddr.and_then(|p| t.word(p));
            let width = at.map(|i| t.words[i].2);
            let whole = |name64: &str, name32: &str| match width {
                Some(32) => op.name == name32,
                Some(64) => op.name == name64,
                _ => false,
            };
            let touches = paddr.is_some_and(|p| t.overlaps(p, reach.bytes));
            if reach.store {
                let whole = whole("sd", "sw");
                match (whole, tainted(rt)) {
                    (true, true) => {
                        t.read(rt, width.expect("a word's width"))?;
                    }
                    (false, false) if !touches => {}
                    _ => return Err(Refusal::InductionStore),
                }
            } else {
                let whole = whole("ld", "lw");
                if (touches && !whole) || (tainted(rt) && matches!(op.name, "lwl" | "lwr" | "ldl" | "ldr")) {
                    return Err(Refusal::InductionOp);
                }
                t.set(rt, if whole { Some((32 + at.expect("a word") as u8, width.expect("a word's width"))) } else { None });
            }
            return Ok(());
        }
        let v = |r: usize| self.gpr[r] as u64;
        let imm = word as u16 as i16 as i64 as u64;
        let out: Option<(u8, u8)> = match op.name {
            "addiu" | "daddiu" => {
                let w = if op.name == "addiu" { 32 } else { 64 };
                t.read(rs, w)?.map(|s| (s, w))
            }
            "addu" | "daddu" | "or" if rt == 0 || rs == 0 => {
                let r = if rt == 0 { rs } else { rt };
                match (op.name, t.get(r)) {
                    (_, None) => None,
                    ("addu", _) => t.read(r, 32)?.map(|s| (s, 32)),
                    (_, Some((s, w))) => Some((s, w)),
                }
            }
            "addu" | "daddu" | "subu" | "dsubu" => {
                let w = if matches!(op.name, "addu" | "subu") { 32 } else { 64 };
                let sub = op.name.ends_with("subu");
                match (tainted(rs), tainted(rt)) {
                    (true, true) => return Err(Refusal::InductionOp),
                    (false, true) if sub => return Err(Refusal::InductionOp),
                    (true, false) => t.read(rs, w)?.map(|s| (s, w)),
                    (false, true) => t.read(rt, w)?.map(|s| (s, w)),
                    (false, false) => None,
                }
            }
            "slt" | "sltu" | "slti" | "sltiu" => {
                let signed = matches!(op.name, "slt" | "slti");
                let immediate = op.name.ends_with('i') || op.name.ends_with("iu");
                let (a, b) = if immediate { (tainted(rs), false) } else { (tainted(rs), tainted(rt)) };
                if a && b {
                    return Err(Refusal::InductionOp);
                }
                if a || b {
                    // x < c, or c < x, whose answer is that of x < c + 1 reversed: each changes where x crosses its bound.
                    let (r, other, after) = if a { (rs, if immediate { imm } else { v(rt) }, false) } else { (rt, v(rs), true) };
                    let (source, width) = t.get(r).expect("a tainted register");
                    t.test(Test { source, width, signed, value: v(r), with: other, after, equal: false })?;
                }
                None
            }
            _ if op.category == Category::Branch => {
                let reads_rs = op.reads & ops::uses::RS != 0 && tainted(rs);
                let reads_rt = op.reads & ops::uses::RT != 0 && tainted(rt);
                if reads_rs && reads_rt {
                    return Err(Refusal::InductionBranch);
                }
                if reads_rs || reads_rt {
                    let (r, other) = if reads_rs { (rs, if op.reads & ops::uses::RT != 0 { v(rt) } else { 0 }) } else { (rt, v(rs)) };
                    let (source, width) = t.get(r).expect("a tainted register");
                    let test = |signed, with, equal| Test { source, width, signed, value: v(r), with, after: false, equal };
                    match op.name {
                        "beq" | "bne" | "beql" | "bnel" => t.test(test(false, other, true))?,
                        "bltz" | "bgez" | "bltzl" | "bgezl" | "bltzal" | "bgezal" | "bltzall" | "bgezall" => t.test(test(true, 0, false))?,
                        "blez" | "bgtz" | "blezl" | "bgtzl" => t.test(test(true, 1, false))?,
                        _ => return Err(Refusal::InductionBranch),
                    }
                }
                if op.writes & ops::uses::RA != 0 {
                    t.set(31, None);
                }
                if op.writes & ops::uses::RD != 0 {
                    t.set(rd, None);
                }
                return Ok(());
            }
            _ => {
                let reads = (op.reads & ops::uses::RS != 0 && tainted(rs)) || (op.reads & ops::uses::RT != 0 && tainted(rt)) || (op.reads & ops::uses::RD != 0 && tainted(rd));
                if reads {
                    return Err(Refusal::InductionOp);
                }
                None
            }
        };
        if op.writes & ops::uses::RD != 0 {
            t.set(rd, out);
        }
        if op.writes & ops::uses::RT != 0 {
            t.set(rt, out);
        }
        if op.writes & ops::uses::RA != 0 {
            t.set(31, None);
        }
        Ok(())
    }

    /// The registers whose values differ from mark `a` to now, the GPRs' low halves only; none if a high half differs.
    pub(crate) fn moved_from(&self, a: &Mark) -> Option<u32> {
        let mut m = 0;
        for r in 1..32 {
            if self.gpr[r] >> 64 != a.gpr[r] >> 64 {
                return None;
            }
            m |= u32::from(self.gpr[r] != a.gpr[r]) << r;
        }
        Some(m)
    }

    /// Each register of `moved`'s step from `a` to now at its width, and that it is the step from `before` to `a`: a
    /// width-32 register a sign extension of its low word throughout, moved by the same amount of it each turn.
    pub(crate) fn steps_from(&self, before: &Mark, a: &Mark, moved: u32, widths: &[u8; 32]) -> Option<[u64; 32]> {
        let mut d = [0u64; 32];
        for r in (1..32).filter(|r| moved >> r & 1 == 1) {
            let (x, y, z) = (before.gpr[r] as u64, a.gpr[r] as u64, self.gpr[r] as u64);
            if widths[r] == 32 {
                let sext = |v: u64| v == v as u32 as i32 as i64 as u64;
                if !(sext(x) && sext(y) && sext(z)) || (z as u32).wrapping_sub(y as u32) != (y as u32).wrapping_sub(x as u32) {
                    return None;
                }
                d[r] = u64::from((z as u32).wrapping_sub(y as u32));
            } else {
                if z.wrapping_sub(y) != y.wrapping_sub(x) {
                    return None;
                }
                d[r] = z.wrapping_sub(y);
            }
        }
        Some(d)
    }

    /// A word of an induction as the core sees it now, of `bits` bits.
    pub(crate) fn idle_word_value(&self, vaddr: u32, paddr: u32, bits: u8) -> u64 {
        let set = super::cache::DCache::set(vaddr);
        let way = self.dcache.hit(set, paddr).expect("an induction word stays in its line");
        let at = (paddr & 63) as usize;
        let line = &self.dcache.data[set][way];
        if bits == 32 { u64::from(u32::from_le_bytes(line[at..at + 4].try_into().unwrap())) } else { u64::from_le_bytes(line[at..at + 8].try_into().unwrap()) }
    }

    /// The registers and induction words moved on by `k` turns of their steps, each at its width.
    pub(crate) fn pass_inductions(&mut self, moved: u32, widths: &[u8; 32], d: &[u64; 32], words: &[(u32, u32, u8, u64)], k: u64) {
        for r in (1..32).filter(|r| moved >> r & 1 == 1) {
            let v = self.gpr[r] as u64;
            let v = if widths[r] == 32 { (v as u32).wrapping_add((d[r] as u32).wrapping_mul(k as u32)) as i32 as i64 as u64 } else { v.wrapping_add(d[r].wrapping_mul(k)) };
            self.gpr[r] = self.gpr[r] & !(u64::MAX as u128) | v as u128;
        }
        for &(vaddr, paddr, bits, step) in words {
            let set = super::cache::DCache::set(vaddr);
            let way = self.dcache.hit(set, paddr).expect("an induction word stays in its line");
            let at = (paddr & 63) as usize;
            let line = &mut self.dcache.data[set][way];
            if bits == 32 {
                let v = u32::from_le_bytes(line[at..at + 4].try_into().unwrap()).wrapping_add((step as u32).wrapping_mul(k as u32));
                line[at..at + 4].copy_from_slice(&v.to_le_bytes());
            } else {
                let v = u64::from_le_bytes(line[at..at + 8].try_into().unwrap()).wrapping_add(step.wrapping_mul(k));
                line[at..at + 8].copy_from_slice(&v.to_le_bytes());
            }
        }
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
