//! The baseline's lowering of one planned block to x86-64 (EmuSen_Recompiler.md §14.2 to §14.4): the entry's test, the
//! guest registers in fixed homes, the integer ALU, the loads and stores through the translation cache, the data cache's
//! hit and the scratchpad with page and line reuse, the branches, and every exit through the shared stubs.

use std::mem::offset_of;

use emusen_assembler::x64::{RAX, RBP, RBX, RCX, RDI, RDX, RSI, R8, R9, R10, R11, R13, R14, R15, RSP, X64};
use emusen_assembler::{Assembler, By, Code, Cond, Label, Mem, Op, Reg, Section, ShiftOp, Src, Width, at};
use emusen_mips_planner::{Class, Plan, Step};

use super::super::super::cache::{DCache, DTag};
use super::super::super::timing::layout;
use super::super::{Context, Page, SHORT, kind, shape};
use super::{BRANCHED, Rec, SLOTS, SYNC, Static, mode, stub};
use crate::ee::Ee;

const GPR: i32 = offset_of!(Ee, gpr) as i32;
const T: i32 = offset_of!(Ee, timing) as i32;
const D_TAGS: i32 = (offset_of!(Ee, dcache) + offset_of!(DCache, tags)) as i32;
const D_DATA: i32 = (offset_of!(Ee, dcache) + offset_of!(DCache, data)) as i32;
const TAG_PFN: i32 = offset_of!(DTag, pfn) as i32;
const TAG_VALID: i32 = offset_of!(DTag, valid) as i32;
const TAG_DIRTY: i32 = offset_of!(DTag, dirty) as i32;
const TAG_LOCK: i32 = offset_of!(DTag, lock) as i32;
const P_TAG: i32 = offset_of!(Page, tag) as i32;
const P_HOST: i32 = offset_of!(Page, host) as i32;
const P_META: i32 = offset_of!(Page, meta) as i32;
const X_GEN1: i32 = offset_of!(Context, gen1) as i32;
const X_LIMIT: i32 = offset_of!(Context, limit) as i32;
const X_LOG: i32 = offset_of!(Context, log) as i32;
const X_LEFT: i32 = offset_of!(Context, left) as i32;
const X_DUMMY: i32 = offset_of!(Context, dummy) as i32;

/// The purposes bytes are counted under: `forms::PURPOSES`' twelve, then the entry's test and the undo log.
pub(crate) const P_ENTRY: usize = 1;
const P_REGS: usize = 3;
const P_CHECKS: usize = 4;
const P_ADDR: usize = 5;
const P_TRANSLATE: usize = 6;
const P_CACHE: usize = 7;
const P_DATA: usize = 8;
const P_WORK: usize = 9;
const P_EXIT: usize = 10;
const P_FAULT: usize = 11;
pub(crate) const P_TEST: usize = 12;
pub(crate) const P_LOG: usize = 13;

/// Why a load or store left, `Context::left`'s index (as `emit.rs` counts them).
const LEFT_ALIGN: i64 = 1;
const LEFT_PAGE: i64 = 2;
const LEFT_MISS: i64 = 3;
const LEFT_UNCACHED: i64 = 4;
/// As `emit.rs`: a load or store leaving at most this many steps after the last one shares that one's state.
const REWIND_MOST: u32 = 2;

/// The context, the core, the translation cache and the entry's last issue, fixed for the whole tier (§3.4).
const CTX: Reg = R15;
const EE: Reg = R14;
const PAGES: Reg = R13;
/// The registers a block's values take, in first-use order; the rest are the scratch registers.
const POOL: [Reg; 7] = [RBX, RBP, RCX, R8, R9, R10, R11];
const X: Reg = RAX;
const TG: Reg = RDX;
const A: Reg = RSI;
const P: Reg = RDI;

/// A value's home: a pool register or a frame slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Home {
    None,
    Reg(Reg),
    Slot(i32),
}

/// Value ids: guest registers' low halves 0..32, high halves 32..64, the branch's outcome, then each access's record.
const NEXT: usize = 64;
const F_A: usize = 0;
const F_HOST: usize = 1;
const F_META: usize = 2;
const F_PTR: usize = 3;
const F_TAG: usize = 4;
fn rec_id(j: usize, f: usize) -> usize {
    65 + 5 * j + f
}
const VALUES: usize = 65 + 5 * shape::MAX;

/// The state after the last load or store, which a rollback past it writes back.
#[derive(Clone, Copy)]
struct SnapSt {
    base: u32,
    dirty: u64,
    pc: u32,
    logged: u32,
    sync: bool,
    /// Slots holding a value as it stood here, saved before a later write.
    saved: [Option<i32>; 64],
}

/// What the lowering knows at a point of the straight line.
#[derive(Clone, Copy)]
struct St {
    held: [bool; VALUES],
    /// Guest values written (lows in bits 0..32, highs 32..64).
    dirty: u64,
    steps: u32,
    logged: u32,
    sync: bool,
    branched: bool,
    snap: Option<SnapSt>,
}

/// An exit's place.
#[derive(Clone, Copy)]
enum At {
    Line(u32),
    /// After the slot: the value `NEXT` holds.
    Jumped,
    /// At the slot, the outcome `NEXT` holds, or known.
    Slot(u32, Option<u32>),
}

struct L<'a> {
    a: X64,
    plan: &'a Plan,
    times: &'a Static,
    home: Vec<Home>,
    st: St,
    next_slot: i32,
    after: u32,
    prototype: bool,
    purpose: usize,
}

fn rs(w: u32) -> usize {
    (w >> 21 & 31) as usize
}
fn rt(w: u32) -> usize {
    (w >> 16 & 31) as usize
}
fn rd(w: u32) -> usize {
    (w >> 11 & 31) as usize
}
fn simm(w: u32) -> i32 {
    i32::from(w as u16 as i16)
}
fn slot_mem(k: i32) -> Mem {
    at(RSP, 8 * k)
}
fn core_at(v: usize) -> Mem {
    at(EE, GPR + 16 * (v % 32) as i32 + if v >= 32 { 8 } else { 0 })
}

/// The block's last use of each record field, by the later accesses that reuse it.
fn record_uses(plan: &Plan) -> Vec<[Option<usize>; 5]> {
    let mut uses = vec![[None; 5]; plan.steps.len()];
    for (k, s) in plan.steps.iter().enumerate() {
        let store = s.info.access.is_some_and(|a| a.store);
        if let Some(j) = s.reuse.line {
            uses[j][F_A] = Some(k);
            uses[j][F_PTR] = Some(k);
            if store {
                uses[j][F_TAG] = Some(k);
            }
            // The line's reuse joins the earlier access's page for its own record.
            uses[j][F_HOST] = uses[j][F_HOST].max(Some(k));
            uses[j][F_META] = uses[j][F_META].max(Some(k));
        }
        if let Some((j, same)) = s.reuse.page {
            if !same {
                uses[j][F_A] = uses[j][F_A].max(Some(k));
            }
            uses[j][F_HOST] = uses[j][F_HOST].max(Some(k));
            uses[j][F_META] = uses[j][F_META].max(Some(k));
        }
    }
    uses
}

impl L<'_> {
    /// Homes by a linear scan in first-use order: guest values live to the block's end, records to their last reuse.
    fn allocate(&mut self) {
        let uses = record_uses(self.plan);
        let mut free: Vec<Reg> = POOL.iter().rev().copied().collect();
        let mut busy: Vec<(usize, Reg)> = Vec::new();
        let mut seen = [false; VALUES];
        for (i, s) in self.plan.steps.iter().enumerate() {
            busy.retain(|&(end, r)| {
                if end < i {
                    free.push(r);
                    false
                } else {
                    true
                }
            });
            let mut vals: Vec<usize> = Vec::new();
            for r in 1..32 {
                if (s.info.reads | s.info.writes) >> r & 1 == 1 {
                    vals.push(r);
                }
            }
            if let Some(acc) = s.info.access
                && acc.bytes == 16
            {
                vals.push(32 + rt(s.word));
            }
            if s.info.class == Class::Branch {
                vals.push(NEXT);
            }
            for v in vals {
                if v == 32 || seen[v] {
                    continue;
                }
                seen[v] = true;
                self.home[v] = match free.pop() {
                    Some(r) => Home::Reg(r),
                    None => self.new_slot().map_or(Home::None, Home::Slot),
                };
            }
            for (f, end) in uses[i].iter().enumerate() {
                if let Some(end) = *end {
                    let v = rec_id(i, f);
                    self.home[v] = match free.pop() {
                        Some(r) => {
                            busy.push((end, r));
                            Home::Reg(r)
                        }
                        None if self.next_slot < SLOTS - 64 => self.new_slot().map_or(Home::None, Home::Slot),
                        None => Home::None,
                    };
                }
            }
        }
    }

    fn new_slot(&mut self) -> Option<i32> {
        (self.next_slot < SLOTS).then(|| {
            self.next_slot += 1;
            self.next_slot - 1
        })
    }

    /// A value in a register: its home, loaded from the core the first time, or `tmp`.
    fn get(&mut self, v: usize, tmp: Reg) -> Reg {
        if v == 0 || v == 32 {
            self.a.mov_imm(tmp, 0);
            return tmp;
        }
        let p = self.push_purpose(P_REGS);
        let r = match self.home[v] {
            Home::Reg(h) => {
                if !self.st.held[v] {
                    self.a.load(8, false, h, core_at(v));
                    self.st.held[v] = true;
                }
                h
            }
            Home::Slot(k) => {
                if (v < 64 && self.st.dirty >> v & 1 == 1) || (v >= 64 && self.st.held[v]) {
                    self.a.load(8, false, tmp, slot_mem(k));
                } else {
                    self.a.load(8, false, tmp, core_at(v));
                }
                tmp
            }
            Home::None => {
                assert!(v < 64, "a value with no home is a guest register");
                self.a.load(8, false, tmp, core_at(v));
                tmp
            }
        };
        self.pur(p);
        r
    }

    fn push_purpose(&mut self, p: usize) -> usize {
        let was = self.purpose;
        self.pur(p);
        was
    }

    fn pur(&mut self, p: usize) {
        self.purpose = p;
        self.a.purpose(p);
    }

    /// The register a new value of `v` is computed in: its home, or `tmp`.
    fn dest(&self, v: usize, tmp: Reg) -> Reg {
        match self.home[v] {
            Home::Reg(h) => h,
            _ => tmp,
        }
    }

    /// `v` written with the value in `r`: stored to its slot when it has one, dirty and held from here.
    fn put(&mut self, v: usize, r: Reg) {
        match self.home[v] {
            Home::Slot(k) => self.a.store(8, r, slot_mem(k)),
            Home::Reg(h) => assert_eq!(h, r, "a value computed in its home"),
            Home::None => panic!("a written value has a home"),
        }
        self.st.held[v] = true;
        if v < 64 {
            self.st.dirty |= 1 << v;
        }
    }

    /// Before `v` is written: its value as the last load or store left it saved, where a rollback could write it back.
    fn before_write(&mut self, v: usize, tmp: Reg) {
        let Some(mut s) = self.st.snap else { return };
        if v >= 64 || s.dirty >> v & 1 == 0 || s.saved[v].is_some() {
            return;
        }
        let k = self.new_slot().expect("a slot for a saved value");
        let p = self.push_purpose(P_REGS);
        let r = self.get(v, tmp);
        self.a.store(8, r, slot_mem(k));
        self.pur(p);
        s.saved[v] = Some(k);
        self.st.snap = Some(s);
    }
}

// The rest of `L`: purposes kept beside the assembler.
impl L<'_> {
    fn rec(&mut self, st: &St, steps: u32, md: u8, pc: u32, next: u32) -> u32 {
        let base = st.snap.map_or(0, |s| s.base);
        let flags = if st.sync { SYNC } else { 0 } | if st.branched { BRANCHED } else { 0 };
        let r = Rec {
            steps: steps as u8,
            base: base as u8,
            logged: st.logged as u8,
            flags,
            mode: md,
            pad: [0; 3],
            pc,
            next,
            after: self.after,
            pad2: 0,
            times: self.times.times,
            snap: self.times.snaps[steps as usize],
        };
        self.a.data(r.bytes())
    }

    /// Every dirty guest value stored to the core.
    fn store_all(&mut self, st: &St) {
        for v in 1..64 {
            if v != 32 && st.dirty >> v & 1 == 1 {
                let r = match self.home[v] {
                    Home::Reg(h) => h,
                    Home::Slot(k) => {
                        self.a.load(8, false, X, slot_mem(k));
                        X
                    }
                    Home::None => unreachable!("a dirty value has a home"),
                };
                self.a.store(8, r, core_at(v));
            }
        }
    }

    /// The state after the last load or store written back, its commit made, and the rollback's return.
    fn partial(&mut self, st: &St, s: &SnapSt) {
        for v in 1..64 {
            if v != 32 && s.dirty >> v & 1 == 1 {
                let r = match (s.saved[v], self.home[v]) {
                    (Some(k), _) | (None, Home::Slot(k)) => {
                        self.a.load(8, false, X, slot_mem(k));
                        X
                    }
                    (None, Home::Reg(h)) => h,
                    (None, Home::None) => unreachable!("a dirty value has a home"),
                };
                self.a.store(8, r, core_at(v));
            }
        }
        let snap_st = St { steps: s.base, logged: s.logged, sync: s.sync, branched: false, snap: None, ..*st };
        let k = self.rec(&snap_st, s.base, mode::LINE, s.pc, 0);
        self.a.lea_data(RDI, k);
        self.a.call_stub(stub::COMMIT);
        self.a.jump_stub(stub::ROLLBACK);
    }

    /// The rollback after an exit's replay kept fewer steps than it ran: the state after the last load or store where the
    /// replay kept it, nothing written otherwise.
    fn rollback(&mut self, st: &St, steps: u32) {
        match st.snap {
            Some(s) if s.base < steps => {
                let undo = self.a.label();
                self.a.cmp(Width::W32, X, Src::Imm(s.base as i32));
                self.a.branch(Cond::Ltu, undo);
                self.partial(st, &s);
                self.a.bind(undo);
                self.a.jump_stub(stub::ROLLBACK);
            }
            _ => self.a.jump_stub(stub::ROLLBACK),
        }
    }

    /// The exit at this point, with the state `st`; laid in the current section, its rollback cold.
    fn exit(&mut self, st: &St, place: At, why: Option<i64>) {
        let p = self.push_purpose(P_EXIT);
        if st.steps == 0 {
            if let Some(w) = why {
                self.a.alu_mem(Op::Add, 8, at(CTX, X_LEFT + 8 * w as i32), Src::Imm(1));
            }
            self.a.jump_stub(stub::DONE0);
            self.pur(p);
            return;
        }
        let (md, pc, next) = match place {
            At::Line(pc) => (mode::LINE, pc, 0),
            At::Jumped => (mode::JUMPED, 0, 0),
            At::Slot(pc, None) => (mode::SLOT, pc, 0),
            At::Slot(pc, Some(n)) => (mode::SLOT_KNOWN, pc, n),
        };
        let k = self.rec(st, st.steps, md, pc, next);
        self.a.lea_data(RDI, k);
        if matches!(md, mode::JUMPED | mode::SLOT) {
            let saved = self.st;
            self.st = *st;
            let r = self.get(NEXT, RSI);
            self.a.mov(Width::W64, RSI, r);
            self.st = saved;
        }
        if let Some(w) = why {
            self.a.mov_imm(RDX, w);
        } else {
            self.a.mov_imm(RDX, -1);
        }
        self.a.call_stub(stub::EXIT);
        let back = self.a.label();
        self.a.cmp(Width::W32, X, Src::Imm(st.steps as i32));
        self.a.branch(Cond::Ne, back);
        self.store_all(st);
        self.a.jump_stub(stub::DONE);
        let here = self.a.current();
        self.a.section(Section::Cold);
        self.a.bind(back);
        self.rollback(st, st.steps);
        self.a.section(here);
        self.pur(p);
    }

    /// A load or store's exit after some steps of its block (`emit.rs`'s `leave`): the board replayed through them, then the
    /// state after the last load or store, or nothing written.
    fn leave(&mut self, st: &St, why: i64) {
        let k = self.rec(st, st.steps, mode::LINE, 0, 0);
        self.a.lea_data(RDI, k);
        self.a.mov_imm(RSI, why);
        self.a.call_stub(stub::LEAVE);
        match st.snap {
            Some(s) => {
                let undo = self.a.label();
                self.a.cmp(Width::W32, X, Src::Imm(s.base as i32));
                self.a.branch(Cond::Ltu, undo);
                self.partial(st, &s);
                self.a.bind(undo);
                self.a.jump_stub(stub::ROLLBACK);
            }
            None => self.a.jump_stub(stub::ROLLBACK),
        }
    }

    /// An exit taken when `c` holds, laid cold; the state is `st`.
    fn exit_if(&mut self, c: Cond, st: &St, place: At) {
        let out = self.a.label();
        self.a.branch(c, out);
        let here = self.a.current();
        self.a.section(Section::Cold);
        self.a.bind(out);
        self.exit(st, place, None);
        self.a.section(here);
    }
}

impl L<'_> {
    /// `d = s op t`, at 32 bits sign-extended where `sext`.
    fn binary(&mut self, op: Op, w: Width, sext: bool, d: usize, s: usize, t: Result<usize, i32>) {
        if d == 0 {
            return;
        }
        self.before_write(d, RDI);
        let a = self.get(s, X);
        let b = match t {
            Ok(t) => Src::Reg(self.get(t, TG)),
            Err(k) => Src::Imm(k),
        };
        let h = self.dest(d, X);
        self.a.alu(op, w, h, a, b);
        if sext {
            self.a.sext32(h, h);
        }
        self.put(d, h);
    }

    /// An operation that traps on overflow: computed aside, the exit before it where it overflows, then written.
    fn trapping(&mut self, op: Op, w: Width, d: usize, s: usize, t: Result<usize, i32>, pc: At) {
        let st = self.st;
        let a = self.get(s, X);
        let b = match t {
            Ok(t) => Src::Reg(self.get(t, TG)),
            Err(k) => Src::Imm(k),
        };
        self.pur(P_FAULT);
        self.a.alu(op, w, X, a, b);
        self.exit_if(Cond::Overflow, &st, pc);
        self.pur(P_WORK);
        if d == 0 {
            return;
        }
        self.before_write(d, RDI);
        let h = self.dest(d, X);
        match w {
            Width::W32 => self.a.sext32(h, X),
            Width::W64 => self.a.mov(Width::W64, h, X),
        }
        self.put(d, h);
    }

    fn shift(&mut self, op: ShiftOp, w: Width, d: usize, t: usize, by: Result<usize, u8>) {
        if d == 0 {
            return;
        }
        self.before_write(d, RDI);
        let a = self.get(t, X);
        let by = match by {
            Ok(s) => By::Reg(self.get(s, TG)),
            Err(k) => By::Imm(k),
        };
        let h = self.dest(d, X);
        self.a.shift(op, w, h, a, by);
        if w == Width::W32 {
            self.a.sext32(h, h);
        }
        self.put(d, h);
    }

    fn set_if(&mut self, c: Cond, d: usize, s: usize, t: Result<usize, i32>) {
        if d == 0 {
            return;
        }
        self.before_write(d, RDI);
        let a = self.get(s, X);
        let b = match t {
            Ok(t) => Src::Reg(self.get(t, TG)),
            Err(k) => Src::Imm(k),
        };
        self.a.cmp(Width::W64, a, b);
        let h = self.dest(d, X);
        self.a.set(c, h);
        self.put(d, h);
    }

    fn plain(&mut self, s: &Step, place: At) {
        let w = s.word;
        let (s_, t, d) = (rs(w), rt(w), rd(w));
        let sa = (w >> 6 & 31) as u8;
        self.pur(P_WORK);
        use Width::{W32, W64};
        match shape::name(w) {
            "add" => self.trapping(Op::Add, W32, d, s_, Ok(t), place),
            "sub" => self.trapping(Op::Sub, W32, d, s_, Ok(t), place),
            "addi" => self.trapping(Op::Add, W32, t, s_, Err(simm(w)), place),
            "dadd" => self.trapping(Op::Add, W64, d, s_, Ok(t), place),
            "dsub" => self.trapping(Op::Sub, W64, d, s_, Ok(t), place),
            "daddi" => self.trapping(Op::Add, W64, t, s_, Err(simm(w)), place),
            "addu" => self.binary(Op::Add, W32, true, d, s_, Ok(t)),
            "subu" => self.binary(Op::Sub, W32, true, d, s_, Ok(t)),
            "addiu" => self.binary(Op::Add, W32, true, t, s_, Err(simm(w))),
            "daddu" => self.binary(Op::Add, W64, false, d, s_, Ok(t)),
            "dsubu" => self.binary(Op::Sub, W64, false, d, s_, Ok(t)),
            "daddiu" => self.binary(Op::Add, W64, false, t, s_, Err(simm(w))),
            "and" => self.binary(Op::And, W64, false, d, s_, Ok(t)),
            "or" => self.binary(Op::Or, W64, false, d, s_, Ok(t)),
            "xor" => self.binary(Op::Xor, W64, false, d, s_, Ok(t)),
            "andi" => self.binary(Op::And, W64, false, t, s_, Err((w & 0xFFFF) as i32)),
            "ori" => self.binary(Op::Or, W64, false, t, s_, Err((w & 0xFFFF) as i32)),
            "xori" => self.binary(Op::Xor, W64, false, t, s_, Err((w & 0xFFFF) as i32)),
            "nor" => {
                if d != 0 {
                    self.binary(Op::Or, W64, false, d, s_, Ok(t));
                    let h = self.dest(d, X);
                    self.a.not(W64, h, h);
                    self.put(d, h);
                }
            }
            "lui" => {
                if t != 0 {
                    self.before_write(t, RDI);
                    let h = self.dest(t, X);
                    self.a.mov_imm(h, i64::from((w << 16) as i32));
                    self.put(t, h);
                }
            }
            "slt" => self.set_if(Cond::Lt, d, s_, Ok(t)),
            "sltu" => self.set_if(Cond::Ltu, d, s_, Ok(t)),
            "slti" => self.set_if(Cond::Lt, t, s_, Err(simm(w))),
            "sltiu" => self.set_if(Cond::Ltu, t, s_, Err(simm(w))),
            n @ ("movz" | "movn") => {
                if d != 0 {
                    self.before_write(d, RDI);
                    let old = self.get(d, X);
                    let c = self.get(t, TG);
                    let a = self.get(s_, RSI);
                    self.a.test(W64, c, Src::Reg(c));
                    self.a.select(if n == "movz" { Cond::Eq } else { Cond::Ne }, old, a);
                    self.put(d, old);
                }
            }
            "sll" => self.shift(ShiftOp::Shl, W32, d, t, Err(sa)),
            "srl" => self.shift(ShiftOp::Shr, W32, d, t, Err(sa)),
            "sra" => self.shift(ShiftOp::Sar, W32, d, t, Err(sa)),
            "sllv" => self.shift(ShiftOp::Shl, W32, d, t, Ok(s_)),
            "srlv" => self.shift(ShiftOp::Shr, W32, d, t, Ok(s_)),
            "srav" => self.shift(ShiftOp::Sar, W32, d, t, Ok(s_)),
            "dsll" => self.shift(ShiftOp::Shl, W64, d, t, Err(sa)),
            "dsrl" => self.shift(ShiftOp::Shr, W64, d, t, Err(sa)),
            "dsra" => self.shift(ShiftOp::Sar, W64, d, t, Err(sa)),
            "dsll32" => self.shift(ShiftOp::Shl, W64, d, t, Err(sa + 32)),
            "dsrl32" => self.shift(ShiftOp::Shr, W64, d, t, Err(sa + 32)),
            "dsra32" => self.shift(ShiftOp::Sar, W64, d, t, Err(sa + 32)),
            "dsllv" => self.shift(ShiftOp::Shl, W64, d, t, Ok(s_)),
            "dsrlv" => self.shift(ShiftOp::Shr, W64, d, t, Ok(s_)),
            "dsrav" => self.shift(ShiftOp::Sar, W64, d, t, Ok(s_)),
            "sync" => self.st.sync = true,
            "sync.p" => {}
            other => unreachable!("{other} is not in E0's slice"),
        }
        self.st.steps += 1;
    }
}

impl L<'_> {
    /// A record field's value in a register, or none where it has no home.
    fn field(&mut self, j: usize, f: usize, tmp: Reg) -> Option<Reg> {
        match self.home[rec_id(j, f)] {
            Home::Reg(h) => Some(h),
            Home::Slot(k) => {
                self.a.load(8, false, tmp, slot_mem(k));
                Some(tmp)
            }
            Home::None => None,
        }
    }

    fn keep(&mut self, j: usize, f: usize, r: Reg) {
        match self.home[rec_id(j, f)] {
            Home::Reg(h) => self.a.mov(Width::W64, h, r),
            Home::Slot(k) => self.a.store(8, r, slot_mem(k)),
            Home::None => {}
        }
    }

    fn has(&self, j: usize, fs: &[usize]) -> bool {
        fs.iter().all(|&f| self.home[rec_id(j, f)] != Home::None)
    }

    /// The translation cache's entry of `A`'s page in X.
    fn page_entry(&mut self) {
        self.a.mov(Width::W32, X, A);
        self.a.shift(ShiftOp::Shr, Width::W32, X, X, By::Imm(12));
        self.a.mov(Width::W32, TG, A);
        self.a.shift(ShiftOp::Shr, Width::W32, TG, TG, By::Imm(24));
        self.a.alu(Op::Xor, Width::W32, X, X, Src::Reg(TG));
        self.a.alu(Op::And, Width::W32, X, X, Src::Imm(super::super::PAGES as i32 - 1));
        self.a.shift(ShiftOp::Shl, Width::W32, X, X, By::Imm(5));
        self.a.alu(Op::Add, Width::W64, X, X, Src::Reg(PAGES));
    }

    /// The full translation: the page's host address in P and its meaning in TG, the fill called cold on a miss.
    fn translate(&mut self, slow: &mut Slow) {
        let p = self.push_purpose(P_TRANSLATE);
        self.page_entry();
        self.a.mov(Width::W32, TG, A);
        self.a.alu(Op::And, Width::W32, TG, TG, Src::Imm(-4096));
        self.a.load(8, false, P, at(CTX, X_GEN1));
        self.a.alu(Op::Or, Width::W64, TG, TG, Src::Reg(P));
        self.a.cmp_mem(8, at(X, P_TAG), Src::Reg(TG));
        let (miss, known) = (self.a.label(), self.a.label());
        self.a.branch(Cond::Ne, miss);
        self.a.bind(known);
        self.a.load(8, false, P, at(X, P_HOST));
        self.a.load(8, false, TG, at(X, P_META));
        let here = self.a.current();
        self.a.section(Section::Cold);
        self.a.bind(miss);
        self.a.mov(Width::W32, RDI, A);
        self.a.call_stub(stub::FILL);
        self.a.test(Width::W32, X, Src::Reg(X));
        let l = slow.label(&mut self.a, LEFT_PAGE);
        self.a.branch(Cond::Eq, l);
        self.page_entry();
        self.a.jump(known);
        self.a.section(here);
        self.pur(p);
    }
}

/// The ways a load or store leaves before it, each a cold label laid once the access is done.
struct Slow {
    used: Vec<(i64, Label)>,
}

impl Slow {
    fn label(&mut self, a: &mut X64, why: i64) -> Label {
        if let Some(&(_, l)) = self.used.iter().find(|(w, _)| *w == why) {
            return l;
        }
        let l = a.label();
        self.used.push((why, l));
        l
    }
}

impl L<'_> {
    /// A load or store (`emit.rs`'s `memory` under the cache, page, line and stub levers, with E0's paths).
    fn memory(&mut self, i: usize, s: &Step, place: At) {
        let acc = s.info.access.expect("a load or store");
        let w = s.word;
        let pre = self.st;
        let mut slow = Slow { used: Vec::new() };
        self.pur(P_ADDR);
        let b = self.get(acc.base as usize, A);
        self.a.lea(Width::W32, A, at(b, simm(w)));
        if (2..=8).contains(&acc.bytes) {
            self.a.test(Width::W32, A, Src::Imm(i32::from(acc.bytes) - 1));
            let l = slow.label(&mut self.a, LEFT_ALIGN);
        self.a.branch(Cond::Ne, l);
        } else if acc.bytes == 16 {
            self.a.alu(Op::And, Width::W32, A, A, Src::Imm(-16));
        }
        let join = self.a.label();
        let mut fallback: Option<Section> = None;
        // One tag check a line: an earlier access's line on the same base value, one compare.
        if let Some(j) = s.reuse.line
            && self.has(j, &[F_A, F_PTR, F_HOST, F_META])
            && (!acc.store || self.has(j, &[F_TAG]))
        {
            self.pur(P_CACHE);
            let ra = self.field(j, F_A, TG).expect("kept");
            self.a.mov(Width::W64, X, A);
            self.a.alu(Op::Xor, Width::W64, X, X, Src::Reg(ra));
            self.a.cmp(Width::W64, X, Src::Imm(64));
            let full = self.a.label();
            self.a.branch(Cond::Geu, full);
            let ra = self.field(j, F_A, TG).expect("kept");
            self.a.mov(Width::W64, P, A);
            self.a.alu(Op::Sub, Width::W64, P, P, Src::Reg(ra));
            let rp = self.field(j, F_PTR, X).expect("kept");
            self.a.alu(Op::Add, Width::W64, P, P, Src::Reg(rp));
            if self.home[rec_id(i, F_HOST)] != Home::None {
                let h = self.field(j, F_HOST, X).expect("kept");
                self.keep(i, F_HOST, h);
                let m = self.field(j, F_META, X).expect("kept");
                self.keep(i, F_META, m);
            }
            if acc.store {
                let t = self.field(j, F_TAG, TG).expect("kept");
                self.a.mov(Width::W64, TG, t);
            }
            self.a.jump(join);
            fallback = Some(self.a.current());
            self.a.section(Section::Cold);
            self.a.bind(full);
        }
        // One translation a page.
        match s.reuse.page {
            Some((j, same)) if self.has(j, &[F_HOST, F_META]) && (same || self.has(j, &[F_A])) => {
                self.pur(P_TRANSLATE);
                let pjoin = self.a.label();
                if !same {
                    let ra = self.field(j, F_A, TG).expect("kept");
                    self.a.mov(Width::W64, X, A);
                    self.a.alu(Op::Xor, Width::W64, X, X, Src::Reg(ra));
                    self.a.cmp(Width::W64, X, Src::Imm(4096));
                    let fullp = self.a.label();
                    let here = self.a.current();
                    self.a.branch(Cond::Geu, fullp);
                    let h = self.field(j, F_HOST, X).expect("kept");
                    self.a.mov(Width::W64, P, h);
                    let m = self.field(j, F_META, X).expect("kept");
                    self.a.mov(Width::W64, TG, m);
                    self.a.jump(pjoin);
                    self.a.section(Section::Cold);
                    self.a.bind(fullp);
                    self.translate(&mut slow);
                    self.a.jump(pjoin);
                    self.a.section(here);
                } else {
                    let h = self.field(j, F_HOST, X).expect("kept");
                    self.a.mov(Width::W64, P, h);
                    let m = self.field(j, F_META, X).expect("kept");
                    self.a.mov(Width::W64, TG, m);
                }
                self.a.bind(pjoin);
            }
            _ => self.translate(&mut slow),
        }
        self.keep(i, F_HOST, P);
        self.keep(i, F_META, TG);
        // The kind's dispatch: a cached hit in either way, then the scratchpad; everything else leaves.
        self.pur(P_CACHE);
        let (mask, cached, spr) = if acc.store {
            ((3 | kind::WRITABLE) as i32, (kind::CACHED | kind::WRITABLE) as i32, (kind::SCRATCHPAD | kind::WRITABLE) as i32)
        } else {
            (3, kind::CACHED as i32, kind::SCRATCHPAD as i32)
        };
        let (other, w1, tail, flat) = (self.a.label(), self.a.label(), self.a.label(), self.a.label());
        self.a.mov(Width::W32, X, TG);
        self.a.alu(Op::And, Width::W32, X, X, Src::Imm(mask));
        self.a.cmp(Width::W32, X, Src::Imm(cached));
        self.a.branch(Cond::Ne, other);
        self.a.mov(Width::W32, X, A);
        self.a.alu(Op::And, Width::W32, X, X, Src::Imm(0xFC0));
        self.a.mov(Width::W32, P, X);
        self.a.shift(ShiftOp::Shr, Width::W32, P, P, By::Imm(2));
        self.a.alu(Op::Add, Width::W64, P, P, Src::Reg(EE));
        self.a.shift(ShiftOp::Shr, Width::W64, TG, TG, By::Imm(32));
        self.a.cmp_mem(4, at(P, D_TAGS + TAG_PFN), Src::Reg(TG));
        self.a.branch(Cond::Ne, w1);
        self.a.cmp_mem(1, at(P, D_TAGS + TAG_VALID), Src::Imm(0));
        self.a.branch(Cond::Eq, w1);
        self.a.bind(tail);
        self.a.lea(Width::W64, TG, at(P, D_TAGS));
        self.a.alu(Op::Add, Width::W64, X, X, Src::Reg(X));
        self.a.load(8, false, P, at(EE, D_DATA));
        self.a.alu(Op::Add, Width::W64, P, P, Src::Reg(X));
        self.a.mov(Width::W32, X, A);
        self.a.alu(Op::And, Width::W32, X, X, Src::Imm(63));
        self.a.alu(Op::Add, Width::W64, P, P, Src::Reg(X));
        self.a.jump(join);
        self.a.bind(w1);
        self.a.cmp_mem(4, at(P, D_TAGS + 8 + TAG_PFN), Src::Reg(TG));
        let l = slow.label(&mut self.a, LEFT_MISS);
        self.a.branch(Cond::Ne, l);
        self.a.cmp_mem(1, at(P, D_TAGS + 8 + TAG_VALID), Src::Imm(0));
        let l = slow.label(&mut self.a, LEFT_MISS);
        self.a.branch(Cond::Eq, l);
        self.a.alu(Op::Add, Width::W64, P, P, Src::Imm(8));
        self.a.alu(Op::Add, Width::W64, X, X, Src::Imm(32));
        self.a.jump(tail);
        self.a.bind(other);
        self.a.mov(Width::W32, X, TG);
        self.a.alu(Op::And, Width::W32, X, X, Src::Imm(mask));
        self.a.cmp(Width::W32, X, Src::Imm(spr));
        self.a.branch(Cond::Ne, flat);
        self.a.mov(Width::W32, X, A);
        self.a.alu(Op::And, Width::W32, X, X, Src::Imm(0xFFF));
        self.a.alu(Op::Add, Width::W64, P, P, Src::Reg(X));
        self.a.lea(Width::W64, TG, at(CTX, X_DUMMY));
        self.a.jump(join);
        let here = self.a.current();
        self.a.section(Section::Cold);
        self.a.bind(flat);
        if !acc.store {
            self.a.mov(Width::W32, X, TG);
            self.a.alu(Op::And, Width::W32, X, X, Src::Imm(3));
            self.a.cmp(Width::W32, X, Src::Imm(kind::UNCACHED as i32));
            let l = slow.label(&mut self.a, LEFT_UNCACHED);
        self.a.branch(Cond::Eq, l);
        }
        let l = slow.label(&mut self.a, LEFT_PAGE);
        self.a.jump(l);
        self.a.section(fallback.unwrap_or(here));
        self.a.bind(join);
        self.keep(i, F_A, A);
        self.keep(i, F_PTR, P);
        self.keep(i, F_TAG, TG);

        // The data access.
        let t = rt(w);
        if acc.store {
            if !self.prototype {
                self.log();
            }
            self.pur(P_DATA);
            if acc.bytes == 16 {
                let lo = self.get(t, X);
                self.a.store(8, lo, at(P, 0));
                let hi = self.get(32 + t, X);
                self.a.store(8, hi, at(P, 8));
            } else {
                let v = self.get(t, X);
                self.a.store(acc.bytes, v, at(P, 0));
            }
            self.pur(P_CACHE);
            self.a.cmp_mem(1, at(TG, TAG_LOCK), Src::Imm(0));
            self.a.set(Cond::Eq, X);
            self.a.alu_mem(Op::Or, 1, at(TG, TAG_DIRTY), Src::Reg(X));
            self.st.sync = true;
        } else if t != 0 {
            self.pur(P_DATA);
            let signed = matches!(shape::name(w), "lb" | "lh" | "lw");
            self.before_write(t, X);
            let h = self.dest(t, X);
            self.a.load(acc.bytes.min(8), signed, h, at(P, 0));
            self.put(t, h);
            if acc.bytes == 16 {
                self.before_write(32 + t, X);
                let h = self.dest(32 + t, X);
                self.a.load(8, false, h, at(P, 8));
                self.put(32 + t, h);
            }
        }
        self.st.steps += 1;
        self.st.snap = Some(SnapSt { base: self.st.steps, dirty: self.st.dirty, pc: s.pc.wrapping_add(4), logged: self.st.logged, sync: self.st.sync, saved: [None; 64] });

        // The ways it leaves, before it.
        let here = self.a.current();
        self.a.section(Section::Cold);
        for (why, l) in slow.used {
            self.a.bind(l);
            let base = pre.snap.map_or(0, |s| s.base);
            if pre.steps == 0 || pre.steps - base > REWIND_MOST {
                self.exit(&pre, place, Some(why));
            } else {
                let p = self.push_purpose(P_EXIT);
                self.leave(&pre, why);
                self.pur(p);
            }
        }
        self.a.section(here);
    }

    /// The store's undo: the 16 bytes of its quadword and its line's tag, two entries of the log.
    fn log(&mut self) {
        let p = self.push_purpose(P_LOG);
        let k = self.st.logged as i32;
        assert!(k + 2 <= 2 * shape::MAX as i32, "two undo entries a store");
        let e = X_LOG + 24 * k;
        self.a.mov(Width::W32, X, A);
        self.a.alu(Op::And, Width::W32, X, X, Src::Imm(15));
        self.a.neg(Width::W64, X, X);
        self.a.alu(Op::Add, Width::W64, X, X, Src::Reg(P));
        self.a.store(8, X, at(CTX, e));
        self.a.load(8, false, A, at(X, 0));
        self.a.store(8, A, at(CTX, e + 8));
        self.a.load(8, false, A, at(X, 8));
        self.a.store(8, A, at(CTX, e + 16));
        self.a.mov_imm(X, SHORT as i64);
        self.a.alu(Op::Or, Width::W64, X, X, Src::Reg(TG));
        self.a.store(8, X, at(CTX, e + 24));
        self.a.load(8, false, A, at(TG, 0));
        self.a.store(8, A, at(CTX, e + 32));
        self.st.logged += 2;
        self.pur(p);
    }

    /// A branch or jump with its slot, and the block's final exit.
    fn branch(&mut self, i: usize) {
        let s = self.plan.steps[i];
        let slot_step = self.plan.steps[i + 1];
        let w = s.word;
        let name = shape::name(w);
        let slot = s.pc.wrapping_add(4);
        let after = s.pc.wrapping_add(8);
        let target = slot.wrapping_add((simm(w) as u32) << 2);
        self.pur(P_WORK);
        let likely = s.info.likely;
        // The condition in the flags, or none for a jump.
        let cond = match name {
            "beq" | "bne" | "beql" | "bnel" => {
                let a = self.get(rs(w), X);
                if rt(w) == 0 {
                    self.a.cmp(Width::W64, a, Src::Imm(0));
                } else {
                    let b = self.get(rt(w), TG);
                    self.a.cmp(Width::W64, a, Src::Reg(b));
                }
                Some(if name.starts_with("beq") { Cond::Eq } else { Cond::Ne })
            }
            "j" | "jal" | "jr" | "jalr" => None,
            _ => {
                let a = self.get(rs(w), X);
                self.a.cmp(Width::W64, a, Src::Imm(0));
                Some(match name {
                    "blez" | "blezl" => Cond::Le,
                    "bgtz" | "bgtzl" => Cond::Gt,
                    "bltz" | "bltzl" | "bltzal" | "bltzall" => Cond::Lt,
                    _ => Cond::Ge,
                })
            }
        };
        // Where the block goes: known, or computed into NEXT.
        let known: Option<u32> = match (name, cond) {
            ("j" | "jal", _) => Some(slot & 0xF000_0000 | (w & 0x03FF_FFFF) << 2),
            ("jr" | "jalr", _) => {
                let a = self.get(rs(w), X);
                let h = self.dest(NEXT, RDI);
                self.a.mov(Width::W32, h, a);
                self.put(NEXT, h);
                None
            }
            (_, Some(_)) if likely => Some(target),
            (_, Some(c)) => {
                let h = self.dest(NEXT, RDI);
                self.a.mov_imm(h, i64::from(after));
                self.a.mov_imm(RSI, i64::from(target));
                self.a.select(c, h, RSI);
                self.put(NEXT, h);
                None
            }
            _ => unreachable!("a branch has a condition or is a jump"),
        };
        let link = match name {
            "bltzal" | "bgezal" | "bltzall" | "bgezall" | "jal" => Some(31),
            "jalr" => Some(rd(w)),
            _ => None,
        };
        if let Some(r) = link.filter(|&r| r != 0) {
            self.before_write(r, RDI);
            let h = self.dest(r, X);
            self.a.mov_imm(h, i64::from(after));
            self.put(r, h);
        }
        self.st.steps += 1;
        self.st.branched = true;
        if likely {
            let c = cond.expect("a likely branch has a condition");
            let on = self.a.label();
            self.a.branch(c, on);
            let st = self.st;
            self.exit(&st, At::Line(after), None);
            self.a.bind(on);
        }
        let place = At::Slot(slot, known);
        self.straight(i + 1, &slot_step, place);
        let st = self.st;
        match known {
            Some(n) => self.exit(&st, At::Line(n), None),
            None => self.exit(&st, At::Jumped, None),
        }
    }

    fn straight(&mut self, i: usize, s: &Step, place: At) {
        match s.info.class {
            Class::Memory => self.memory(i, s, place),
            _ => self.plain(s, place),
        }
    }
}

/// The block's code and the offset of its fallback word; `prototype` as `compile` says.
pub(crate) fn lower(plan: &Plan, times: &Static, prototype: bool) -> Option<(Code, u32)> {
    let after = plan.steps.iter().find(|s| s.info.class == Class::Branch).map_or(0, |s| s.pc.wrapping_add(8));
    let mut l = L { a: X64::new(), plan, times, home: vec![Home::None; VALUES], st: St { held: [false; VALUES], dirty: 0, steps: 0, logged: 0, sync: false, branched: false, snap: None }, next_slot: 0, after, prototype, purpose: 0 };
    l.allocate();
    let fallback = l.a.data(&[0; 8]);
    let entry = l.a.label();
    let body = l.a.label();
    if !prototype {
        // The entry's test: at rest for what the block reads, or the Cranelift function (§14.3).
        let dynamic = l.a.label();
        l.pur(P_TEST);
        l.a.bind(dynamic);
        l.a.jump_via_data(fallback);
        l.a.bind(entry);
        l.a.load(8, false, X, at(RDI, T + layout::LAST));
        l.a.alu(Op::Add, Width::W64, X, X, Src::Imm(1));
        l.a.cmp_mem(1, at(RDI, T + layout::OPEN), Src::Imm(0));
        l.a.branch(Cond::Ne, dynamic);
        l.a.cmp_mem(1, at(RDI, T + layout::GROUP_ENDS), Src::Imm(0));
        l.a.branch(Cond::Ne, dynamic);
        l.a.cmp_mem(8, at(RDI, T + layout::FLOOR), Src::Reg(X));
        l.a.branch(Cond::Gtu, dynamic);
        for r in (0..64).filter(|r| times.exposed >> r & 1 == 1) {
            l.a.cmp_mem(8, at(RDI, T + layout::READY + 8 * r), Src::Reg(X));
            l.a.branch(Cond::Gtu, dynamic);
        }
    } else {
        l.a.bind(entry);
    }
    l.pur(P_ENTRY);
    l.a.lea_label(R11, body);
    l.a.jump_stub(stub::ENTER);
    l.a.bind(body);
    let n = plan.steps.len();
    let mut k = 0;
    while k < n {
        let s = plan.steps[k];
        if k > 0 && s.pc & 63 == 0 {
            // A new line of the instruction cache: the block leaves here when the cache does not hold it.
            l.pur(P_CHECKS);
            l.a.cmp_mem(4, at(CTX, X_LIMIT), Src::Imm(k as i32));
            let st = l.st;
            l.exit_if(Cond::Leu, &st, At::Line(s.pc));
        }
        match s.info.class {
            Class::Branch => {
                l.branch(k);
                break;
            }
            _ => {
                l.straight(k, &s, At::Line(s.pc));
                k += 1;
                if k == n {
                    let st = l.st;
                    l.exit(&st, At::Line(s.pc.wrapping_add(4)), None);
                }
            }
        }
    }
    Some((l.a.finish(entry), fallback))
}
