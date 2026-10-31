//! One planned EE block lowered to x86-64: the guest's values in registers from its first use to the block's end, a load's or store's slow path a call that returns into the block, and one add and compare at each end (Sapphire_Speed.md §66).

use std::mem::offset_of;

use emusen_assembler::x64::{RAX, RBP, RBX, RCX, RDI, RDX, RSI, R8, R9, R10, R11, R12, R13, R14, R15, RSP, X64};
use emusen_assembler::{Assembler, By, Code, Cond, Label, Mem, Op, Reg, Section, ShiftOp, Src, Width, at};
use emusen_mips_planner::{Class, Plan, Step};

use super::super::cache::{DCache, DTag};
use super::super::ops;
use super::isa::{BOARD, Cost, IN_SLOT, LIGHT, NEXT_HELD, Rec, SLOTS, stub};
use super::{Context, Jump, Page, kind, shape};
use crate::ee::Ee;

const GPR: i32 = offset_of!(Ee, gpr) as i32;
const HI: i32 = offset_of!(Ee, hi) as i32;
const LO: i32 = offset_of!(Ee, lo) as i32;
const SA: i32 = offset_of!(Ee, sa) as i32;
const FPR: i32 = offset_of!(Ee, fpr) as i32;
const ACC: i32 = offset_of!(Ee, acc) as i32;
const FCR31: i32 = offset_of!(Ee, fcr31) as i32;
const STATUS: i32 = (offset_of!(Ee, cop0) + 4 * crate::ee::cop0::STATUS) as i32;
const UCAB_VALID: i32 = offset_of!(Ee, ucab) as i32 + offset_of!(super::super::cache::Ucab, valid) as i32;
const X_VF0: i32 = offset_of!(Context, vf0) as i32;
const X_QUIET: i32 = offset_of!(Context, quiet) as i32;
const X_DEADLINE: i32 = offset_of!(Context, deadline) as i32;
const X_WAIT: i32 = offset_of!(Context, wait) as i32;
const X_STEPS: i32 = offset_of!(Context, steps) as i32;
const X_FRONT: i32 = offset_of!(Context, front) as i32;
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
const X_DUMMY: i32 = offset_of!(Context, dummy) as i32;

/// The purposes bytes are counted under.
const P_REGS: usize = 3;
const P_ADDR: usize = 5;
const P_TRANSLATE: usize = 6;
const P_CACHE: usize = 7;
const P_DATA: usize = 8;
const P_WORK: usize = 9;
const P_EXIT: usize = 10;
const P_FAULT: usize = 11;

/// The loads and stores of a unit's part, which never leave for alignment.
const UNALIGNED: [&str; 8] = ["lwl", "lwr", "ldl", "ldr", "swl", "swr", "sdl", "sdr"];
/// Why a slow path was taken, `Context::left`'s index.
const LEFT_CALL: i64 = 0;
const LEFT_ALIGN: i64 = 1;
const LEFT_PAGE: i64 = 2;
const LEFT_MISS: i64 = 3;
const LEFT_UNCACHED: i64 = 4;

/// The context, the core, the translation cache and the time, fixed for the whole tier.
const CTX: Reg = R15;
const EE: Reg = R14;
const PAGES: Reg = R13;
const TIME: Reg = R12;
/// The registers a block's values take, in first-use order; the rest are the scratch registers.
pub(crate) const POOL: [Reg; 7] = [RBX, RBP, RCX, R8, R9, R10, R11];
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

/// Value ids: guest registers' low halves 0..32, high halves 32..64, LO, HI, LO1, HI1 and SA, the branch's outcome, then each access's record.
const LO0: usize = 64;
const HI0: usize = 65;
const LO1: usize = 66;
const HI1: usize = 67;
const SAV: usize = 68;
/// The guest's values, held and written back alike.
pub(crate) const GUEST: usize = 69;
pub(crate) const NEXT: usize = 69;
const F_A: usize = 0;
const F_HOST: usize = 1;
const F_META: usize = 2;
const F_PTR: usize = 3;
const F_TAG: usize = 4;
fn rec_id(j: usize, f: usize) -> usize {
    70 + 5 * j + f
}
const fn values(steps: usize) -> usize {
    70 + 5 * steps
}

/// The calls the lowering makes in place: MFC1 and MTC1, and the multimedia instructions that are 64-bit operations on each half.
fn native(w: u32) -> bool {
    matches!(shape::name(w), "mfc1" | "mtc1") || vf_move(w) || wide(w)
}

/// QMFC2 and QMTC2, in place while VU0 rests.
fn vf_move(w: u32) -> bool {
    matches!(shape::name(w), "qmfc2" | "qmtc2")
}

/// LQC2 and SQC2, whose quadword is VF[rt], not a GPR.
fn vf_memory(w: u32) -> bool {
    matches!(shape::name(w), "lqc2" | "sqc2")
}

/// LWC1 and SWC1, whose word is FPR[rt].
fn fp_memory(w: u32) -> bool {
    matches!(shape::name(w), "lwc1" | "swc1")
}

/// The multimedia instructions lowered in place (Sapphire_Speed.md §65).
fn wide(w: u32) -> bool {
    matches!(shape::name(w), "por" | "pand" | "pxor" | "pnor" | "pcpyld" | "pcpyud" | "pextlw" | "pextuw" | "pextlh" | "pextuh" | "pextlb" | "pextub")
}

/// The block's values a call writes.
fn call_writes(w: u32, op: &ops::Op) -> Vec<usize> {
    let mut out = Vec::new();
    if op.kind == ops::Kind::Cop2 {
        if matches!(op.name, "qmfc2" | "cfc2") && rt(w) != 0 {
            out.push(rt(w));
            if op.name == "qmfc2" {
                out.push(32 + rt(w));
            }
        }
        return out;
    }
    if op.kind == ops::Kind::Fpu {
        if op.writes & crate::ee::fpu::uses::GRT != 0 && rt(w) != 0 {
            out.push(rt(w));
        }
        return out;
    }
    use ops::uses::{HILO0, HILO1, RD};
    if op.writes & RD != 0 && rd(w) != 0 {
        out.extend([rd(w), 32 + rd(w)]);
    }
    if op.writes & (HILO0 | HILO1) != 0 {
        out.extend([LO0, HI0, LO1, HI1]);
    }
    out
}

/// What the lowering knows at a point of the straight line.
struct St {
    held: Vec<bool>,
    /// Guest values written, a bit a value id below `GUEST`.
    dirty: u128,
    steps: u32,
    sync: bool,
}

struct L<'a> {
    a: X64,
    plan: &'a Plan,
    cost: &'a Cost,
    home: Vec<Home>,
    uses: Vec<[Option<usize>; 5]>,
    st: St,
    next_slot: i32,
    purpose: usize,
    /// The block's data so far, mirrored so that a record knows its own offset.
    data_len: u32,
    /// The step in hand, and the dirty values before it.
    now: usize,
    pre: u128,
    /// In a delay slot: the place after it, none where `NEXT` holds it.
    slot: Option<Option<u32>>,
    /// The block only waits (`waits`).
    waits: bool,
}

/// Whether a block only waits: it jumps back to its own start, stores nothing, calls nothing and carries no value from turn to turn, so that a turn repeats until something outside changes what it loads (after PCSX2's idle loops found at compile time, `iR5900.cpp`).
fn waits(plan: &Plan) -> bool {
    let n = plan.steps.len();
    if n < 2 || plan.exposed & plan.written != 0 || plan.steps[n - 2].info.class != Class::Branch {
        return false;
    }
    let b = plan.steps[n - 2];
    let target = b.pc.wrapping_add(4).wrapping_add((simm(b.word) as u32) << 2);
    let known = match shape::name(b.word) {
        "beq" | "bne" | "blez" | "bgtz" | "bltz" | "bgez" | "beql" | "bnel" | "blezl" | "bgtzl" | "bltzl" | "bgezl" => target == plan.start,
        "j" => b.pc.wrapping_add(4) & 0xF000_0000 | (b.word & 0x03FF_FFFF) << 2 == plan.start,
        _ => false,
    };
    known && plan.steps.iter().enumerate().all(|(k, s)| {
        k == n - 2
            || match s.info.class {
                Class::Memory => s.info.access.is_some_and(|a| !a.store) && !vf_memory(s.word) && !fp_memory(s.word),
                Class::Plain => shape::class(s.word) == shape::Class::Plain && specials(s.word).is_empty() && !matches!(shape::name(s.word), "sync"),
                _ => false,
            }
    })
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
pub(crate) fn core_off(v: usize) -> i32 {
    match v {
        LO0 => LO,
        HI0 => HI,
        LO1 => LO + 8,
        HI1 => HI + 8,
        SAV => SA,
        _ => GPR + 16 * (v % 32) as i32 + if v >= 32 { 8 } else { 0 },
    }
}
fn core_at(v: usize) -> Mem {
    at(EE, core_off(v))
}

/// The values beyond the general registers an instruction reads or writes: LO, HI, LO1, HI1, SA.
fn specials(w: u32) -> &'static [usize] {
    match shape::name(w) {
        "mfhi" | "mthi" => &[HI0],
        "mflo" | "mtlo" => &[LO0],
        "mfhi1" | "mthi1" => &[HI1],
        "mflo1" | "mtlo1" => &[LO1],
        "mult" | "multu" | "madd" | "maddu" | "div" | "divu" => &[LO0, HI0],
        "mult1" | "multu1" | "madd1" | "maddu1" | "div1" | "divu1" => &[LO1, HI1],
        "mfsa" | "mtsa" | "mtsab" | "mtsah" => &[SAV],
        _ => &[],
    }
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
        self.uses = uses.clone();
        let mut free: Vec<Reg> = POOL.iter().rev().copied().collect();
        let mut busy: Vec<(usize, Reg)> = Vec::new();
        let mut seen = vec![false; values(self.plan.steps.len())];
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
            if shape::class(s.word) == shape::Class::Call && !native(s.word) && s.info.class != Class::Memory {
                continue;
            }
            if wide(s.word) {
                vals.extend([32 + rs(s.word), 32 + rt(s.word), 32 + rd(s.word)]);
            }
            if vf_move(s.word) {
                vals.push(32 + rt(s.word));
            }
            for r in 1..32 {
                if (s.info.reads | s.info.writes) >> r & 1 == 1 {
                    vals.push(r);
                }
            }
            if let Some(acc) = s.info.access
                && acc.bytes == 16
                && !vf_memory(s.word)
            {
                vals.push(32 + rt(s.word));
            }
            vals.extend_from_slice(specials(s.word));
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
                if (v < GUEST && self.st.dirty >> v & 1 == 1) || (v >= GUEST && self.st.held[v]) {
                    self.a.load(8, false, tmp, slot_mem(k));
                } else {
                    self.a.load(8, false, tmp, core_at(v));
                }
                tmp
            }
            Home::None => {
                assert!(v < GUEST, "a value with no home is a guest register");
                self.a.load(8, false, tmp, core_at(v));
                tmp
            }
        };
        self.pur(p);
        r
    }

    /// Out-of-line code begins: laid cold, and jumped over where the flow is already cold, so that it is never fallen into.
    fn cold_begin(&mut self) -> (Section, Option<Label>) {
        let here = self.a.current();
        let over = (here == Section::Cold).then(|| {
            let l = self.a.label();
            self.a.jump(l);
            l
        });
        self.a.section(Section::Cold);
        (here, over)
    }

    fn cold_end(&mut self, (here, over): (Section, Option<Label>)) {
        self.a.section(here);
        if let Some(l) = over {
            self.a.bind(l);
        }
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
        if v < GUEST {
            self.st.dirty |= 1 << v;
        }
    }

}

// The rest of `L`: purposes kept beside the assembler.
impl L<'_> {
    /// The record of the step in hand, to its slow path: `reload` the values the operation writes.
    fn rec(&mut self, s: &Step, reload: u128, board: bool) -> u32 {
        let time = if self.now == 0 { 0 } else { self.cost.times[self.now - 1] };
        let (next, held) = match self.slot {
            Some(Some(n)) => (n, 0),
            Some(None) => (0, NEXT_HELD),
            None => (0, 0),
        };
        let flags = if self.slot.is_some() { IN_SLOT } else { 0 } | held | if board { BOARD } else { 0 } | if crate::ee::cop2_light(s.word) { LIGHT } else { 0 };
        let halves = |m: u128| [m as u64, (m >> 64) as u64];
        let at = self.data_len.next_multiple_of(8);
        let r = Rec { dirty: halves(self.pre), reload: halves(reload), word: s.word, pc: s.pc, time, step: self.now as u16, index: ops::index(s.word), at, next, flags, pad: 0 };
        self.data(r.bytes())
    }

    /// Bytes appended to the block's data, as `Assembler::data` lays them; their offset.
    fn data(&mut self, bytes: &[u8]) -> u32 {
        let at = self.a.data(bytes);
        assert_eq!(at, self.data_len.next_multiple_of(8), "the data laid as mirrored");
        self.data_len = at + bytes.len() as u32;
        at
    }

    /// The slow path's call: the record's address, why, and the stub, which returns here or leaves the block.
    fn slow_call(&mut self, k: u32, why: i64) {
        self.a.lea_data(RDI, k);
        self.a.mov_imm(RSI, why);
        self.a.call_stub(stub::SLOW);
    }

    /// Where `c` holds the interpreter's operation runs in place of the step in hand, which raises and leaves the block.
    fn fault_if(&mut self, c: Cond, s: &Step) {
        let to = self.a.label();
        let back = self.a.label();
        self.a.branch(c, to);
        let out = self.cold_begin();
        self.a.bind(to);
        let k = self.rec(s, 0, false);
        self.slow_call(k, LEFT_CALL);
        self.a.jump(back);
        self.cold_end(out);
        self.a.bind(back);
    }

    /// Every dirty guest value stored to the core.
    fn store_all(&mut self) {
        for v in 1..GUEST {
            if v != 32 && self.st.dirty >> v & 1 == 1 {
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

    /// The core as the block's last instruction leaves it, and its instructions counted.
    fn write_back(&mut self, steps: u32) {
        let p = self.push_purpose(P_EXIT);
        self.store_all();
        if self.st.sync {
            self.a.store_imm(1, 0, at(EE, UCAB_VALID));
        }
        self.a.alu_mem(Op::Add, 8, at(CTX, X_STEPS), Src::Imm(steps as i32));
        self.pur(p);
    }

    /// The end of a path to `target`, its cycles added: on through the link's word unless the deadline has come. A wait's jump back to its own start runs the time on to where what it reads can change.
    fn end_at(&mut self, target: u32, cycles: u32) {
        let p = self.push_purpose(P_EXIT);
        if self.waits && target == self.plan.start {
            self.a.alu(Op::Add, Width::W64, TIME, TIME, Src::Imm(cycles as i32));
            self.a.load(8, false, X, at(CTX, X_WAIT));
            self.a.cmp(Width::W64, X, Src::Reg(TIME));
            self.a.select(Cond::Gtu, TIME, X);
            self.a.mov_imm(RDI, i64::from(target));
            self.a.mov_imm(RSI, 0);
            self.a.mov_imm(RDX, i64::from(target));
            self.a.jump_stub(stub::OUT);
            self.pur(p);
            return;
        }
        let out = self.a.label();
        let slot = self.a.data_address(out);
        self.data_len = self.data_len.next_multiple_of(8) + 8;
        self.a.alu(Op::Add, Width::W64, TIME, TIME, Src::Imm(cycles as i32));
        self.a.cmp_mem(8, at(CTX, X_DEADLINE), Src::Reg(TIME));
        let cold = self.a.label();
        self.a.branch(Cond::Leu, cold);
        self.a.jump_via_data(slot);
        let here = self.a.current();
        self.a.section(Section::Cold);
        self.a.bind(cold);
        self.a.bind(out);
        self.a.mov_imm(RDI, i64::from(target));
        self.a.lea_data(RSI, slot);
        self.a.mov_imm(RDX, i64::from(self.plan.start));
        self.a.jump_stub(stub::OUT);
        self.a.section(here);
        self.pur(p);
    }

    /// The end of a path to the address in X, through the table of blocks.
    fn end_through(&mut self, cycles: u32) {
        let p = self.push_purpose(P_EXIT);
        self.a.mov(Width::W32, RDI, X);
        self.a.alu(Op::Add, Width::W64, TIME, TIME, Src::Imm(cycles as i32));
        self.a.cmp_mem(8, at(CTX, X_DEADLINE), Src::Reg(TIME));
        let out = self.a.label();
        self.a.branch(Cond::Leu, out);
        if self.plan.start >> 31 == 0 {
            // From kuseg a jump to a kernel segment goes through the dispatcher, which checks the mode.
            self.a.test(Width::W32, RDI, Src::Reg(RDI));
            self.a.branch(Cond::Lt, out);
        }
        self.a.mov(Width::W32, TG, RDI);
        self.a.shift(ShiftOp::Shl, Width::W32, TG, TG, By::Imm(2));
        self.a.alu(Op::And, Width::W32, TG, TG, Src::Imm(((super::FRONT - 1) * size_of::<Jump>()) as i32));
        self.a.load(8, false, A, at(CTX, X_FRONT));
        self.a.alu(Op::Add, Width::W64, TG, TG, Src::Reg(A));
        self.a.cmp_mem(4, at(TG, offset_of!(Jump, pc) as i32), Src::Reg(RDI));
        self.a.branch(Cond::Ne, out);
        self.a.jump_mem(at(TG, offset_of!(Jump, code) as i32));
        let here = self.a.current();
        self.a.section(Section::Cold);
        self.a.bind(out);
        self.a.jump_stub(stub::OUT0);
        self.a.section(here);
        self.pur(p);
    }
}

impl L<'_> {
    /// `d = s op t`, at 32 bits sign-extended where `sext`.
    fn binary(&mut self, op: Op, w: Width, sext: bool, d: usize, s: usize, t: Result<usize, i32>) {
        if d == 0 {
            return;
        }
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
    fn trapping(&mut self, op: Op, w: Width, d: usize, s: usize, t: Result<usize, i32>, step: &Step) {
        let a = self.get(s, X);
        let b = match t {
            Ok(t) => Src::Reg(self.get(t, TG)),
            Err(k) => Src::Imm(k),
        };
        self.pur(P_FAULT);
        self.a.alu(op, w, X, a, b);
        self.fault_if(Cond::Overflow, step);
        self.pur(P_WORK);
        if d == 0 {
            return;
        }
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

    /// `d = s` for two values of any kind.
    fn copy(&mut self, d: usize, s: usize) {
        if d == 0 {
            return;
        }
        let a = self.get(s, X);
        let h = self.dest(d, X);
        self.a.mov(Width::W64, h, a);
        self.put(d, h);
    }

    /// LO, HI and rd from the 64-bit result in X: its low word and its high word, each sign-extended.
    fn product_out(&mut self, upper: bool, d: usize) {
        let (lo, hi) = if upper { (LO1, HI1) } else { (LO0, HI0) };
        self.a.mov(Width::W64, TG, X);
        self.a.shift(ShiftOp::Shr, Width::W64, TG, TG, By::Imm(32));
        for (v, from) in [(lo, X), (hi, TG)] {
            let h = self.dest(v, RDI);
            self.a.sext32(h, from);
            self.put(v, h);
        }
        if d != 0 {
            let h = self.dest(d, RDI);
            self.a.sext32(h, X);
            self.put(d, h);
        }
    }

    /// MULT, MULTU, MADD and MADDU of either pipeline (`ops.rs`'s `product`).
    fn multiply(&mut self, unsigned: bool, add: bool, upper: bool, s: usize, t: usize, d: usize) {
        let a = self.get(s, X);
        if unsigned { self.a.mov(Width::W32, X, a) } else { self.a.sext32(X, a) }
        let b = self.get(t, TG);
        if unsigned { self.a.mov(Width::W32, TG, b) } else { self.a.sext32(TG, b) }
        self.a.mul(X, X, TG);
        if add {
            let (lo, hi) = if upper { (LO1, HI1) } else { (LO0, HI0) };
            let h = self.get(hi, A);
            self.a.mov(Width::W64, A, h);
            self.a.shift(ShiftOp::Shl, Width::W64, A, A, By::Imm(32));
            let l = self.get(lo, TG);
            self.a.mov(Width::W32, TG, l);
            self.a.alu(Op::Or, Width::W64, A, A, Src::Reg(TG));
            self.a.alu(Op::Add, Width::W64, X, X, Src::Reg(A));
        }
        self.product_out(upper, d);
    }

    /// DIV and DIVU of either pipeline (`ops.rs`'s `quotient_signed` and `quotient_unsigned`): a zero divisor and, signed, -1 out of line.
    fn divide(&mut self, signed: bool, upper: bool, s: usize, t: usize) {
        let n = self.get(s, P);
        self.a.mov(Width::W64, P, n);
        let dv = self.get(t, A);
        self.a.mov(Width::W64, A, dv);
        let (zero, minus, join) = (self.a.label(), self.a.label(), self.a.label());
        self.a.test(Width::W32, A, Src::Reg(A));
        self.a.branch(Cond::Eq, zero);
        if signed {
            self.a.cmp(Width::W32, A, Src::Imm(-1));
            self.a.branch(Cond::Eq, minus);
        }
        self.a.divrem(signed, Width::W32, X, TG, P, A);
        self.a.bind(join);
        let out = self.cold_begin();
        self.a.bind(zero);
        // q = all ones (signed: 1 where the dividend is negative), r = the dividend.
        self.a.mov_imm(X, 0xFFFF_FFFF);
        if signed {
            self.a.mov_imm(TG, 1);
            self.a.test(Width::W32, P, Src::Reg(P));
            self.a.select(Cond::Lt, X, TG);
        }
        self.a.mov(Width::W32, TG, P);
        self.a.jump(join);
        if signed {
            self.a.bind(minus);
            self.a.neg(Width::W32, X, P);
            self.a.mov_imm(TG, 0);
            self.a.jump(join);
        }
        self.cold_end(out);
        let (lo, hi) = if upper { (LO1, HI1) } else { (LO0, HI0) };
        for (v, from) in [(lo, X), (hi, TG)] {
            let h = self.dest(v, RDI);
            self.a.sext32(h, from);
            self.put(v, h);
        }
    }

    fn plain(&mut self, s: &Step) {
        let w = s.word;
        let (s_, t, d) = (rs(w), rt(w), rd(w));
        let sa = (w >> 6 & 31) as u8;
        self.pur(P_WORK);
        use Width::{W32, W64};
        match shape::name(w) {
            "add" => self.trapping(Op::Add, W32, d, s_, Ok(t), s),
            "sub" => self.trapping(Op::Sub, W32, d, s_, Ok(t), s),
            "addi" => self.trapping(Op::Add, W32, t, s_, Err(simm(w)), s),
            "dadd" => self.trapping(Op::Add, W64, d, s_, Ok(t), s),
            "dsub" => self.trapping(Op::Sub, W64, d, s_, Ok(t), s),
            "daddi" => self.trapping(Op::Add, W64, t, s_, Err(simm(w)), s),
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
            "mfhi" => self.copy(d, HI0),
            "mflo" => self.copy(d, LO0),
            "mfhi1" => self.copy(d, HI1),
            "mflo1" => self.copy(d, LO1),
            "mfsa" => self.copy(d, SAV),
            "mthi" => self.copy(HI0, s_),
            "mtlo" => self.copy(LO0, s_),
            "mthi1" => self.copy(HI1, s_),
            "mtlo1" => self.copy(LO1, s_),
            n @ ("mult" | "multu" | "mult1" | "multu1" | "madd" | "maddu" | "madd1" | "maddu1") => {
                let upper = n.ends_with('1');
                self.multiply(n.contains("multu") || n.contains("maddu"), n.starts_with("madd"), upper, s_, t, d);
            }
            n @ ("div" | "divu" | "div1" | "divu1") => self.divide(!n.starts_with("divu"), n.ends_with('1'), s_, t),
            n @ ("mtsa" | "mtsab" | "mtsah") => {
                let a = self.get(s_, X);
                self.a.mov(W64, X, a);
                if n != "mtsa" {
                    self.a.alu(Op::Xor, W64, X, X, Src::Imm((w & 0xFFFF) as i32));
                }
                if n == "mtsah" {
                    self.a.alu(Op::And, W64, X, X, Src::Imm(7));
                    self.a.alu(Op::Add, W64, X, X, Src::Reg(X));
                } else {
                    self.a.alu(Op::And, W64, X, X, Src::Imm(15));
                }
                let h = self.dest(SAV, X);
                self.a.mov(W64, h, X);
                self.put(SAV, h);
            }
            n @ ("tge" | "tgeu" | "tlt" | "tltu" | "teq" | "tne" | "tgei" | "tgeiu" | "tlti" | "tltiu" | "teqi" | "tnei") => {
                let c = match n {
                    "tge" | "tgei" => Cond::Ge,
                    "tgeu" | "tgeiu" => Cond::Geu,
                    "tlt" | "tlti" => Cond::Lt,
                    "tltu" | "tltiu" => Cond::Ltu,
                    "teq" | "teqi" => Cond::Eq,
                    _ => Cond::Ne,
                };
                let a = self.get(s_, X);
                self.pur(P_FAULT);
                let imm = matches!(n, "tgei" | "tgeiu" | "tlti" | "tltiu" | "teqi" | "tnei");
                let b = if imm { Src::Imm(simm(w)) } else { Src::Reg(self.get(t, TG)) };
                self.a.cmp(W64, a, b);
                self.fault_if(c, s);
                self.pur(P_WORK);
            }
            other => unreachable!("{other} is not compiled in place"),
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
        self.a.alu(Op::And, Width::W32, X, X, Src::Imm(super::PAGES as i32 - 1));
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
        let out = self.cold_begin();
        self.a.bind(miss);
        self.a.mov(Width::W32, RDI, A);
        self.a.call_stub(stub::FILL);
        self.a.test(Width::W32, X, Src::Reg(X));
        let l = slow.label(&mut self.a, LEFT_PAGE);
        self.a.branch(Cond::Eq, l);
        self.page_entry();
        self.a.jump(known);
        self.cold_end(out);
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
    /// A load or store: the page's kind and the data cache's tags in place; everything else the interpreter's operation, called from the cold path, which returns past the access or leaves the block where it faults.
    fn memory(&mut self, i: usize, s: &Step) {
        let acc = s.info.access.expect("a load or store");
        let w = s.word;
        let vf = vf_memory(w);
        let fp = fp_memory(w);
        // What the data access reads is held before any slow path can be taken, which skips it.
        let t = rt(w);
        if t != 0 && !vf && !fp && shape::name(w) != "pref" && (acc.store || UNALIGNED.contains(&shape::name(w))) {
            self.pur(P_REGS);
            self.get(t, X);
            if acc.store && acc.bytes == 16 {
                self.get(32 + t, X);
            }
        }
        self.pur(P_REGS);
        self.get(acc.base as usize, A);
        let mut slow = Slow { used: Vec::new() };
        let done = self.a.label();
        self.pur(P_FAULT);
        if vf {
            self.a.cmp_mem(4, at(CTX, X_QUIET), Src::Imm(0));
            let l = slow.label(&mut self.a, LEFT_CALL);
            self.a.branch(Cond::Eq, l);
        }
        if vf || fp {
            self.a.load(4, false, X, at(EE, STATUS));
            self.a.test(Width::W32, X, Src::Imm((if vf { crate::ee::status::CU2 } else { crate::ee::status::CU1 }) as i32));
            let l = slow.label(&mut self.a, LEFT_CALL);
            self.a.branch(Cond::Eq, l);
        }
        self.pur(P_ADDR);
        let b = self.get(acc.base as usize, A);
        self.a.lea(Width::W32, A, at(b, simm(w)));
        let unit = UNALIGNED.contains(&shape::name(w));
        if ((2..=8).contains(&acc.bytes) && !unit) || vf {
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
        if vf {
            self.vf_quad(acc.store, t);
        } else if shape::name(w) == "pref" {
        } else if unit {
            self.unaligned(shape::name(w), acc.bytes, t);
        } else if fp {
            self.pur(P_DATA);
            let f = at(EE, FPR + 4 * t as i32);
            if acc.store {
                self.a.load(4, false, X, f);
                self.a.store(4, X, at(P, 0));
                self.dirty_line();
            } else {
                self.a.load(4, false, X, at(P, 0));
                self.a.store(4, X, f);
            }
        } else if acc.store {
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
            self.dirty_line();
        } else if t != 0 {
            self.pur(P_DATA);
            let signed = matches!(shape::name(w), "lb" | "lh" | "lw");
            let h = self.dest(t, X);
            self.a.load(acc.bytes.min(8), signed, h, at(P, 0));
            self.put(t, h);
            if acc.bytes == 16 {
                let h = self.dest(32 + t, X);
                self.a.load(8, false, h, at(P, 8));
                self.put(32 + t, h);
            }
        }
        self.st.steps += 1;
        self.a.bind(done);

        // The interpreter's operation in place of the access; after it no line or page an earlier access kept may be taken as held.
        let mut reload = 0u128;
        if !acc.store && !vf && !fp && t != 0 && shape::name(w) != "pref" {
            reload |= 1 << t;
            if acc.bytes == 16 {
                reload |= 1 << (32 + t);
            }
        }
        let here = self.a.current();
        self.a.section(Section::Cold);
        let k = self.rec(s, reload, true);
        let call = self.a.label();
        let used = slow.used;
        let n = used.len();
        for (j, (why, l)) in used.into_iter().enumerate() {
            self.a.bind(l);
            self.a.mov_imm(RSI, why);
            if j + 1 < n {
                self.a.jump(call);
            }
        }
        self.a.bind(call);
        self.a.lea_data(RDI, k);
        self.a.call_stub(stub::SLOW);
        for j in 0..=i {
            if self.home[rec_id(j, F_A)] != Home::None && (j == i || self.uses[j][F_A].is_some_and(|e| e > i)) {
                self.a.mov_imm(X, i64::MIN);
                self.keep(j, F_A, X);
            }
        }
        if self.home[rec_id(i, F_META)] != Home::None {
            self.a.mov_imm(X, 0);
            self.keep(i, F_META, X);
        }
        self.a.jump(done);
        self.a.section(here);
    }

    /// The line a store wrote marked dirty unless it is locked, TG its tag.
    fn dirty_line(&mut self) {
        self.pur(P_CACHE);
        self.a.cmp_mem(1, at(TG, TAG_LOCK), Src::Imm(0));
        self.a.set(Cond::Eq, X);
        self.a.alu_mem(Op::Or, 1, at(TG, TAG_DIRTY), Src::Reg(X));
        self.st.sync = true;
    }

    /// LQC2's and SQC2's data, P the quadword in memory: VF[t] through `Context::vf0`.
    fn vf_quad(&mut self, store: bool, t: usize) {
        self.pur(P_DATA);
        self.a.load(8, false, X, at(CTX, X_VF0));
        if store {
            for k in [0, 8] {
                self.a.load(8, false, A, at(X, 16 * t as i32 + k));
                self.a.store(8, A, at(P, k));
            }
            self.dirty_line();
        } else if t != 0 {
            for k in [0, 8] {
                self.a.load(8, false, A, at(P, k));
                self.a.store(8, A, at(X, 16 * t as i32 + k));
            }
        }
    }

    /// LWL to SDR (`ops.rs`'s `unit` and `store_unit`): the unit's address and pointer made from A and P, the shift from the byte within it, and the merge.
    fn unaligned(&mut self, name: &str, n: u8, t: usize) {
        use Width::{W32, W64};
        let left = name.as_bytes()[2] == b'l';
        let store = name.starts_with('s');
        let w = if n == 8 { W64 } else { W32 };
        let (toward, away) = if left { (ShiftOp::Shl, ShiftOp::Shr) } else { (ShiftOp::Shr, ShiftOp::Shl) };
        if !store && t == 0 {
            return;
        }
        self.pur(P_DATA);
        if !store {
        }
        self.a.mov(W32, X, A);
        self.a.alu(Op::And, W32, X, X, Src::Imm(i32::from(n) - 1));
        self.a.alu(Op::Sub, W32, A, A, Src::Reg(X));
        self.a.alu(Op::Sub, W64, P, P, Src::Reg(X));
        self.a.shift(ShiftOp::Shl, W32, X, X, By::Imm(3));
        if left {
            self.a.neg(W32, X, X);
            self.a.alu(Op::Add, W32, X, X, Src::Imm(8 * (i32::from(n) - 1)));
        }
        if store {
            self.a.push(X);
            self.dirty_line();
            self.pur(P_DATA);
            self.a.pop(X);
            let r = self.get(t, A);
            self.a.mov(W64, A, r);
            self.a.shift(away, w, A, A, By::Reg(X));
            self.a.mov_imm(TG, -1);
            self.a.shift(away, w, TG, TG, By::Reg(X));
            self.a.not(w, TG, TG);
            self.a.load(n, false, X, at(P, 0));
            self.a.alu(Op::And, w, X, X, Src::Reg(TG));
            self.a.alu(Op::Or, w, X, X, Src::Reg(A));
            self.a.store(n, X, at(P, 0));
            return;
        }
        self.a.load(n, false, P, at(P, 0));
        self.a.mov_imm(TG, -1);
        self.a.shift(toward, w, TG, TG, By::Reg(X));
        self.a.not(w, TG, TG);
        let r = self.get(t, A);
        self.a.mov(W64, A, r);
        self.a.alu(Op::And, w, TG, TG, Src::Reg(A));
        self.a.shift(toward, w, P, P, By::Reg(X));
        self.a.alu(Op::Or, w, P, P, Src::Reg(TG));
        let out = match name {
            "lwl" => {
                self.a.sext32(P, P);
                P
            }
            "lwr" => {
                // The word sign-extended where it is whole (k = 0), otherwise the register's upper word kept.
                self.a.sext32(TG, P);
                self.a.shift(ShiftOp::Shr, W64, A, A, By::Imm(32));
                self.a.shift(ShiftOp::Shl, W64, A, A, By::Imm(32));
                self.a.alu(Op::Or, W64, A, A, Src::Reg(P));
                self.a.test(W32, X, Src::Reg(X));
                self.a.select(Cond::Eq, A, TG);
                A
            }
            _ => P,
        };
        let h = self.dest(t, X);
        self.a.mov(W64, h, out);
        self.put(t, h);
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
        self.now = i;
        self.pre = self.st.dirty;
        if name.starts_with("bc1") {
            self.usable(crate::ee::status::CU1, &s);
            self.pur(P_WORK);
        }
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
            "bc1f" | "bc1fl" | "bc1t" | "bc1tl" => {
                self.a.load(4, false, X, at(EE, FCR31));
                self.a.test(Width::W32, X, Src::Imm(crate::ee::fpu::flag::C as i32));
                Some(if name.starts_with("bc1t") { Cond::Ne } else { Cond::Eq })
            }
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
            let h = self.dest(r, X);
            self.a.mov_imm(h, i64::from(after));
            self.put(r, h);
        }
        self.st.steps += 1;
        let (taken, not) = (self.cost.taken, self.cost.times[i + 1]);
        if likely {
            let c = cond.expect("a likely branch has a condition");
            let on = self.a.label();
            self.a.branch(c, on);
            let n = self.st.steps;
            self.write_back(n);
            self.end_at(after, self.cost.times[i]);
            self.a.bind(on);
        }
        self.slot = Some(known);
        self.now = i + 1;
        self.pre = self.st.dirty;
        self.straight(i + 1, &slot_step);
        self.slot = None;
        let n = self.st.steps;
        self.write_back(n);
        match known {
            Some(t) => self.end_at(t, taken),
            None if matches!(name, "jr" | "jalr") => {
                let r = self.get(NEXT, X);
                self.a.mov(Width::W32, X, r);
                self.end_through(taken);
            }
            None => {
                let r = self.get(NEXT, X);
                self.a.cmp(Width::W32, r, Src::Imm(after as i32));
                let fall = self.a.label();
                self.a.branch(Cond::Eq, fall);
                self.end_at(target, taken);
                self.a.bind(fall);
                self.end_at(after, not);
            }
        }
    }

    fn straight(&mut self, i: usize, s: &Step) {
        match s.info.class {
            Class::Memory => self.memory(i, s),
            _ if wide(s.word) => self.wide(s),
            _ if vf_move(s.word) => self.vf_transfer(s),
            _ if native(s.word) => self.fpu_move(s),
            _ if shape::class(s.word) == shape::Class::Call => self.call(s),
            _ => self.plain(s),
        }
    }

    /// The 32-bit or 16-bit lanes of `r`'s low word (its high word where `upper`) spread to every other lane of `w`, `m` a scratch register.
    fn spread(&mut self, w: Reg, r: Reg, upper: bool, bytes: bool, m: Reg) {
        use Width::W64;
        if upper {
            self.a.mov(W64, w, r);
            self.a.shift(ShiftOp::Shr, W64, w, w, By::Imm(32));
        } else {
            self.a.mov(Width::W32, w, r);
        }
        let steps: &[(u8, i64)] = if bytes { &[(16, 0x0000_FFFF_0000_FFFF), (8, 0x00FF_00FF_00FF_00FF)] } else { &[(16, 0x0000_FFFF_0000_FFFF)] };
        for &(k, mask) in steps {
            self.a.mov(W64, m, w);
            self.a.shift(ShiftOp::Shl, W64, m, m, By::Imm(k));
            self.a.alu(Op::Or, W64, w, w, Src::Reg(m));
            self.a.mov_imm(m, mask);
            self.a.alu(Op::And, W64, w, w, Src::Reg(m));
        }
    }

    /// The multimedia instructions that are 64-bit operations on each half of the registers (`mmi.rs`), in place.
    fn wide(&mut self, s: &Step) {
        use Width::W64;
        let w = s.word;
        let (a, b, d) = (rs(w), rt(w), rd(w));
        self.pur(P_WORK);
        let name = shape::name(w);
        if d != 0 {
            match name {
                "por" | "pand" | "pxor" | "pnor" => {
                    let op = match name {
                        "pand" => Op::And,
                        "pxor" => Op::Xor,
                        _ => Op::Or,
                    };
                    for k in [0, 32] {
                        self.binary(op, W64, false, d + k, a + k, Ok(b + k));
                        if name == "pnor" {
                            let h = self.dest(d + k, X);
                            self.a.not(W64, h, h);
                            self.put(d + k, h);
                        }
                    }
                }
                "pcpyld" => {
                    self.copy(32 + d, a);
                    self.copy(d, b);
                }
                "pcpyud" => {
                    self.copy(d, 32 + a);
                    self.copy(32 + d, 32 + b);
                }
                _ => {
                    // The interleaves: rd's low half in X and its high half in P, both from the sources' low (or high) halves before either is written.
                    let half = if name.ends_with("ub") || name.ends_with("uh") || name == "pextuw" { 32 } else { 0 };
                    let (lane, bytes) = match &name[name.len() - 1..] {
                        "b" => (8u8, true),
                        "h" => (16, false),
                        _ => (32, false),
                    };
                    for (out, upper) in [(X, false), (P, true)] {
                        if lane == 32 {
                            let t = self.get(b + half, TG);
                            if upper {
                                self.a.mov(W64, TG, t);
                                self.a.shift(ShiftOp::Shr, W64, TG, TG, By::Imm(32));
                            } else {
                                self.a.mov(Width::W32, TG, t);
                            }
                            let r = self.get(a + half, out);
                            if upper {
                                self.a.mov(W64, out, r);
                                self.a.mov_imm(A, -1i64 << 32);
                                self.a.alu(Op::And, W64, out, out, Src::Reg(A));
                            } else {
                                self.a.mov(W64, out, r);
                                self.a.shift(ShiftOp::Shl, W64, out, out, By::Imm(32));
                            }
                        } else {
                            let t = self.get(b + half, TG);
                            self.spread(TG, t, upper, bytes, A);
                            let r = self.get(a + half, out);
                            self.spread(out, r, upper, bytes, A);
                            self.a.shift(ShiftOp::Shl, W64, out, out, By::Imm(lane));
                        }
                        self.a.alu(Op::Or, W64, out, out, Src::Reg(TG));
                    }
                    for (v, r) in [(d, X), (32 + d, P)] {
                        let h = self.dest(v, r);
                        if h != r {
                            self.a.mov(W64, h, r);
                        }
                        self.put(v, h);
                    }
                }
            }
        }
        self.st.steps += 1;
    }

    /// The interpreter raises the coprocessor's unusable exception while Status's bit `cu` is clear.
    fn usable(&mut self, cu: u32, s: &Step) {
        let p = self.push_purpose(P_FAULT);
        self.a.load(4, false, X, at(EE, STATUS));
        self.a.test(Width::W32, X, Src::Imm(cu as i32));
        self.fault_if(Cond::Eq, s);
        self.pur(p);
    }

    /// MFC1 and MTC1 in place.
    fn fpu_move(&mut self, s: &Step) {
        let w = s.word;
        let (t, f) = (rt(w), FPR + 4 * rd(w) as i32);
        self.usable(crate::ee::status::CU1, s);
        self.pur(P_WORK);
        if shape::name(w) == "mfc1" {
            if t != 0 {
                let h = self.dest(t, X);
                self.a.load(4, true, h, at(EE, f));
                self.put(t, h);
            }
        } else {
            let r = self.get(t, X);
            self.a.store(4, r, at(EE, f));
        }
        self.st.steps += 1;
    }

    /// QMFC2 and QMTC2 in place while VU0 rests, nothing to interlock with; the interpreter's otherwise, or where COP2 is unusable.
    fn vf_transfer(&mut self, s: &Step) {
        let w = s.word;
        let (t, f) = (rt(w), rd(w) as i32);
        let from = shape::name(w) == "qmfc2";
        if !from && t != 0 {
            self.get(t, X);
            self.get(32 + t, X);
        }
        let (cold, done) = (self.a.label(), self.a.label());
        self.pur(P_FAULT);
        self.a.cmp_mem(4, at(CTX, X_QUIET), Src::Imm(0));
        self.a.branch(Cond::Eq, cold);
        self.a.load(4, false, X, at(EE, STATUS));
        self.a.test(Width::W32, X, Src::Imm(crate::ee::status::CU2 as i32));
        self.a.branch(Cond::Eq, cold);
        self.pur(P_WORK);
        if (from && t != 0) || (!from && f != 0) {
            self.a.load(8, false, X, at(CTX, X_VF0));
            for (k, v) in [(0, t), (8, 32 + t)] {
                if from {
                    let h = self.dest(v, A);
                    self.a.load(8, false, h, at(X, 16 * f + k));
                    self.put(v, h);
                } else {
                    let r = self.get(v, A);
                    self.a.store(8, r, at(X, 16 * f + k));
                }
            }
        }
        self.st.steps += 1;
        self.a.bind(done);
        let reload = if from && t != 0 { 1u128 << t | 1 << (32 + t) } else { 0 };
        let here = self.a.current();
        self.a.section(Section::Cold);
        self.a.bind(cold);
        let k = self.rec(s, reload, true);
        self.slow_call(k, LEFT_CALL);
        self.a.jump(done);
        self.a.section(here);
    }

    /// A call: every value the block wrote is in the core for the operation, and what it writes is read again where it is next used.
    fn call(&mut self, s: &Step) {
        let w = s.word;
        let op = ops::get(ops::index(w));
        let k = self.rec(s, 0, op.kind == ops::Kind::Cop2);
        self.pur(P_WORK);
        self.slow_call(k, LEFT_CALL);
        self.st.dirty = 0;
        for v in call_writes(w, op) {
            self.st.held[v] = false;
        }
        self.st.steps += 1;
    }
}

/// The block's code.
pub(crate) fn lower(plan: &Plan, cost: &Cost) -> Option<Code> {
    let n = plan.steps.len();
    let mut l = L { a: X64::new(), plan, cost, home: vec![Home::None; values(n)], uses: Vec::new(), st: St { held: vec![false; values(n)], dirty: 0, steps: 0, sync: false }, next_slot: 0, purpose: 0, data_len: 0, now: 0, pre: 0, slot: None, waits: waits(plan) };
    l.allocate();
    let mut homes = [0u8; GUEST + 1];
    for (v, h) in l.home.iter().take(GUEST + 1).enumerate() {
        homes[v] = match *h {
            Home::None => 0,
            Home::Reg(r) => 1 + POOL.iter().position(|&p| p == r).expect("a pool register") as u8,
            Home::Slot(k) => 8 + k as u8,
        };
    }
    let table = l.data(&homes);
    assert_eq!(table, 0, "the homes are the data's first");
    let body = l.a.label();
    l.a.bind(body);
    let mut k = 0;
    while k < n {
        let s = plan.steps[k];
        l.now = k;
        l.pre = l.st.dirty;
        match s.info.class {
            Class::Branch => {
                l.branch(k);
                break;
            }
            _ => {
                l.straight(k, &s);
                k += 1;
                if k == n {
                    let steps = l.st.steps;
                    l.write_back(steps);
                    l.end_at(s.pc.wrapping_add(4), cost.times[k - 1]);
                }
            }
        }
    }
    Some(l.a.finish(body))
}
