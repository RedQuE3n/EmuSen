//! The baseline's lowering of one planned block to x86-64 (EmuSen_Recompiler.md §14.2 to §14.4)

use std::mem::offset_of;

use emusen_assembler::x64::{RAX, RBP, RBX, RCX, RDI, RDX, RSI, R8, R9, R10, R11, R13, R14, R15, RSP, X64};
use emusen_assembler::{Assembler, By, Code, Cond, Label, Mem, Op, Reg, Section, ShiftOp, Src, Width, at};
use emusen_mips_planner::{Class, Plan, Step};

use super::super::super::cache::{DCache, DTag};
use super::super::super::ops;
use super::super::super::timing::{Timing, layout};
use super::super::{Context, Options, Page, SHORT, WORD, kind, shape};
use super::{BRANCHED, Rec, SLOTS, SYNC, Static, X_MOVED, mode, prefix, stub};
use crate::ee::Ee;

const GPR: i32 = offset_of!(Ee, gpr) as i32;
const HI: i32 = offset_of!(Ee, hi) as i32;
const LO: i32 = offset_of!(Ee, lo) as i32;
const SA: i32 = offset_of!(Ee, sa) as i32;
const FPR: i32 = offset_of!(Ee, fpr) as i32;
const ACC: i32 = offset_of!(Ee, acc) as i32;
const FCR31: i32 = offset_of!(Ee, fcr31) as i32;
const STATUS: i32 = (offset_of!(Ee, cop0) + 4 * crate::ee::cop0::STATUS) as i32;
const X_VF0: i32 = offset_of!(Context, vf0) as i32;
const X_QUIET: i32 = offset_of!(Context, quiet) as i32;
const X_ROOM: i32 = offset_of!(Context, room) as i32;
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
const X_VALUES: i32 = offset_of!(Context, values) as i32;
const X_HANDED: i32 = offset_of!(Context, handed) as i32;

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

/// The loads and stores of a unit's part, which never leave for alignment.
const UNALIGNED: [&str; 8] = ["lwl", "lwr", "ldl", "ldr", "swl", "swr", "sdl", "sdr"];
/// Why a load or store left, `Context::left`'s index (as `emit.rs` counts them).
const LEFT_EVENT: i64 = 0;
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

/// Value ids: guest registers' low halves 0..32, high halves 32..64, LO, HI, LO1, HI1 and SA, the branch's outcome, then each access's record.
const LO0: usize = 64;
const HI0: usize = 65;
const LO1: usize = 66;
const HI1: usize = 67;
const SAV: usize = 68;
/// The guest's values, held and written back alike.
const GUEST: usize = 69;
const NEXT: usize = 69;
const F_A: usize = 0;
const F_HOST: usize = 1;
const F_META: usize = 2;
const F_PTR: usize = 3;
const F_TAG: usize = 4;
fn rec_id(j: usize, f: usize) -> usize {
    70 + 5 * j + f
}
const VALUES: usize = 70 + 5 * shape::MAX;

/// The undo log's entries, `Context::log`'s length; a block's entries stay below it (Sapphire_Speed.md §65).
const LOG_MOST: i32 = 4 * shape::MAX as i32 - 1;

/// The calls the lowering makes in place: MFC1 and MTC1, and the multimedia instructions that are 64-bit operations on each half.
fn native(w: u32) -> bool {
    matches!(shape::name(w), "mfc1" | "mtc1") || wide(w)
}

/// LQC2 and SQC2, whose quadword is VF[rt], not a GPR.
fn vf_memory(w: u32) -> bool {
    matches!(shape::name(w), "lqc2" | "sqc2")
}

/// The multimedia instructions lowered in place (Sapphire_Speed.md §65).
fn wide(w: u32) -> bool {
    matches!(shape::name(w), "por" | "pand" | "pxor" | "pnor" | "pcpyld" | "pcpyud" | "pextlw" | "pextuw" | "pextlh" | "pextuh" | "pextlb" | "pextub")
}

/// What a call writes: the core's bytes to log (offset, 4, 8 or 16 bytes) and the block's values to load again after it.
pub(crate) struct Writes {
    logs: [(i32, u8); 4],
    n_logs: usize,
    values: [usize; 6],
    n_values: usize,
}

impl Writes {
    pub(crate) fn logs(&self) -> &[(i32, u8)] {
        &self.logs[..self.n_logs]
    }
    fn values(&self) -> &[usize] {
        &self.values[..self.n_values]
    }
    fn log(&mut self, off: i32, n: u8) {
        self.logs[self.n_logs] = (off, n);
        self.n_logs += 1;
    }
    fn value(&mut self, v: usize) {
        self.values[self.n_values] = v;
        self.n_values += 1;
    }
}

/// A call's writes by its operation's fields (Sapphire_Speed.md §65).
pub(in crate::ee) fn call_writes(w: u32, op: &ops::Op) -> Writes {
    let mut out = Writes { logs: [(0, 0); 4], n_logs: 0, values: [0; 6], n_values: 0 };
    if op.kind == ops::Kind::Cop2 {
        if op.name == "qmfc2" && rt(w) != 0 {
            out.log(GPR + 16 * rt(w) as i32, 16);
            out.value(rt(w));
            out.value(32 + rt(w));
        }
        return out;
    }
    if op.kind == ops::Kind::Fpu {
        use crate::ee::fpu::uses::{ACC as A_, FCR, FD, FS, FT, GRT};
        if op.writes & GRT != 0 && rt(w) != 0 {
            out.log(GPR + 16 * rt(w) as i32, 8);
            out.value(rt(w));
        }
        for (f, n) in [(FS, rd(w)), (FT, rt(w)), (FD, (w >> 6 & 31) as usize)] {
            if op.writes & f != 0 {
                out.log(FPR + 4 * n as i32, 4);
            }
        }
        if op.writes & A_ != 0 {
            out.log(ACC, 4);
        }
        if op.writes & FCR != 0 {
            out.log(FCR31, 4);
        }
        return out;
    }
    use ops::uses::{HILO0, HILO1, RD};
    if op.writes & RD != 0 && rd(w) != 0 {
        out.log(GPR + 16 * rd(w) as i32, 16);
        out.value(rd(w));
        out.value(32 + rd(w));
    }
    if op.writes & (HILO0 | HILO1) != 0 {
        out.log(HI, 16);
        out.log(LO, 16);
        for v in [LO0, HI0, LO1, HI1] {
            out.value(v);
        }
    }
    out
}

/// The state after the last load or store, which a rollback past it writes back.
#[derive(Clone, Copy)]
struct SnapSt {
    base: u32,
    /// One past the last load or store's step, which a call's state carries on.
    guard: u32,
    dirty: u128,
    pc: u32,
    logged: u32,
    sync: bool,
    /// Slots holding a value as it stood here, saved before a later write.
    saved: [Option<i32>; GUEST],
}

/// What the lowering knows at a point of the straight line.
#[derive(Clone, Copy)]
struct St {
    held: [bool; VALUES],
    /// Guest values written, a bit a value id below `GUEST`.
    dirty: u128,
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
    /// The block's data so far, mirrored so that a record knows its own offset.
    data_len: u32,
    /// The last record laid and where, for the exits of one load or store, which share it.
    last_rec: Option<(Rec, u32)>,
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
            if shape::class(s.word) == shape::Class::Call && !native(s.word) && s.info.class != Class::Memory {
                continue;
            }
            if wide(s.word) {
                vals.extend([32 + rs(s.word), 32 + rt(s.word), 32 + rd(s.word)]);
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

    /// Before `v` is written: its value as the last load or store left it saved, where a rollback could write it back.
    fn before_write(&mut self, v: usize, tmp: Reg) {
        let Some(mut s) = self.st.snap else { return };
        if v >= GUEST || s.dirty >> v & 1 == 0 || s.saved[v].is_some() {
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
            guard: st.snap.map_or(0, |s| s.guard) as u8,
            pad: [0; 2],
            pc,
            next,
            after: self.after,
            table: self.data_len.next_multiple_of(8),
            snap: self.times.snaps[steps as usize][0],
        };
        if !st.branched {
            if let Some((l, at)) = self.last_rec
                && (Rec { table: 0, ..l }).bytes() == (Rec { table: 0, ..r }).bytes()
            {
                return at;
            }
            let at = self.data(r.bytes());
            self.last_rec = Some((r, at));
            return at;
        }
        let taken = self.times.snaps[steps as usize][1];
        // SAFETY: a plain `repr(C)` value read as its bytes, laid right after its record (`Rec::snap`).
        let tail = unsafe { std::slice::from_raw_parts((&taken as *const super::Snap).cast::<u8>(), size_of::<super::Snap>()) };
        let mut both = r.bytes().to_vec();
        both.extend_from_slice(tail);
        self.data(&both)
    }

    /// Bytes appended to the block's data, as `Assembler::data` lays them; their offset.
    fn data(&mut self, bytes: &[u8]) -> u32 {
        let at = self.a.data(bytes);
        assert_eq!(at, self.data_len.next_multiple_of(8), "the data laid as mirrored");
        self.data_len = at + bytes.len() as u32;
        at
    }

    /// Every dirty guest value stored to the core.
    fn store_all(&mut self, st: &St) {
        for v in 1..GUEST {
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
        for v in 1..GUEST {
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

    /// The rollback after an exit's replay kept fewer steps than it ran: the state after the last load or store where the replay kept it, nothing written otherwise.
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

    /// A load or store's exit after some steps of its block (`emit.rs`'s `leave`). See EmuSen_Recompiler.md §14.
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
        let to = self.a.label();
        self.a.branch(c, to);
        let out = self.cold_begin();
        self.a.bind(to);
        self.exit(st, place, None);
        self.cold_end(out);
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

    /// `d = s` for two values of any kind.
    fn copy(&mut self, d: usize, s: usize) {
        if d == 0 {
            return;
        }
        self.before_write(d, RDI);
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
            self.before_write(v, RDI);
            let h = self.dest(v, RDI);
            self.a.sext32(h, from);
            self.put(v, h);
        }
        if d != 0 {
            self.before_write(d, RDI);
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
            self.before_write(v, RDI);
            let h = self.dest(v, RDI);
            self.a.sext32(h, from);
            self.put(v, h);
        }
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
                self.before_write(SAV, RDI);
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
                let st = self.st;
                let a = self.get(s_, X);
                self.pur(P_FAULT);
                let imm = matches!(n, "tgei" | "tgeiu" | "tlti" | "tltiu" | "teqi" | "tnei");
                let b = if imm { Src::Imm(simm(w)) } else { Src::Reg(self.get(t, TG)) };
                self.a.cmp(W64, a, b);
                self.exit_if(c, &st, place);
                self.pur(P_WORK);
            }
            other => unreachable!("{other} is not in the baseline's classes"),
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
    /// A load or store (`emit.rs`'s `memory` under the cache, page, line and stub levers, with E0's paths).
    fn memory(&mut self, i: usize, s: &Step, place: At) {
        let acc = s.info.access.expect("a load or store");
        let w = s.word;
        let vf = vf_memory(w);
        if vf {
            self.cop2_may_run(place);
        }
        let pre = self.st;
        let mut slow = Slow { used: Vec::new() };
        if i > 0 {
            // Past the board's next event at one cycle an instruction: the block leaves before this access (Sapphire_Speed.md §65.5).
            self.pur(P_CHECKS);
            self.a.cmp_mem(4, at(CTX, X_ROOM), Src::Imm(i as i32));
            let l = slow.label(&mut self.a, LEFT_EVENT);
            self.a.branch(Cond::Leu, l);
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
        let t = rt(w);
        if vf {
            self.vf_quad(acc.store, t);
        } else if shape::name(w) == "pref" {
        } else if unit {
            self.unaligned(shape::name(w), acc.bytes, t);
        } else if acc.store {
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
        self.st.snap = Some(SnapSt { base: self.st.steps, guard: self.st.steps, dirty: self.st.dirty, pc: s.pc.wrapping_add(4), logged: self.st.logged, sync: self.st.sync, saved: [None; GUEST] });

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

    /// LQC2's and SQC2's data, P the quadword in memory: VF[t] through `Context::vf0`, LQC2's with its undo.
    fn vf_quad(&mut self, store: bool, t: usize) {
        self.pur(P_DATA);
        if store {
            if !self.prototype {
                self.log();
            }
            self.pur(P_DATA);
            self.a.load(8, false, X, at(CTX, X_VF0));
            for k in [0, 8] {
                self.a.load(8, false, A, at(X, 16 * t as i32 + k));
                self.a.store(8, A, at(P, k));
            }
            self.pur(P_CACHE);
            self.a.cmp_mem(1, at(TG, TAG_LOCK), Src::Imm(0));
            self.a.set(Cond::Eq, X);
            self.a.alu_mem(Op::Or, 1, at(TG, TAG_DIRTY), Src::Reg(X));
            self.st.sync = true;
        } else if t != 0 {
            self.a.load(8, false, X, at(CTX, X_VF0));
            self.a.alu(Op::Add, Width::W64, X, X, Src::Imm(16 * t as i32));
            let k = self.st.logged as i32;
            assert!(k < LOG_MOST, "a block's undo entries fit the log");
            let e = X_LOG + 24 * k;
            self.a.store(8, X, at(CTX, e));
            for j in [0, 8] {
                self.a.load(8, false, A, at(X, j));
                self.a.store(8, A, at(CTX, e + 8 + j));
                self.a.load(8, false, A, at(P, j));
                self.a.store(8, A, at(X, j));
            }
            self.st.logged += 1;
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
            self.before_write(t, X);
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
            let k = self.new_slot().expect("a slot for the shift");
            self.a.store(8, X, slot_mem(k));
            if !self.prototype {
                self.log();
            }
            self.pur(P_CACHE);
            self.a.cmp_mem(1, at(TG, TAG_LOCK), Src::Imm(0));
            self.a.set(Cond::Eq, X);
            self.a.alu_mem(Op::Or, 1, at(TG, TAG_DIRTY), Src::Reg(X));
            self.pur(P_DATA);
            self.a.load(8, false, X, slot_mem(k));
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
            self.st.sync = true;
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

    /// The store's undo: the 16 bytes of its quadword and its line's tag, two entries of the log.
    fn log(&mut self) {
        let p = self.push_purpose(P_LOG);
        let k = self.st.logged as i32;
        assert!(k + 2 <= LOG_MOST, "two undo entries a store");
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
            _ if wide(s.word) => self.wide(s),
            _ if native(s.word) => self.fpu_move(s, place),
            _ if shape::class(s.word) == shape::Class::Call => self.call(s, place),
            _ => self.plain(s, place),
        }
    }

    /// `off`'s `n` bytes in the core (4, 8 or 16) logged before a write, as a store's are.
    fn log_core(&mut self, off: i32, n: u8) {
        let k = self.st.logged as i32;
        assert!(k < LOG_MOST, "a block's undo entries fit the log");
        let e = X_LOG + 24 * k;
        self.a.lea(Width::W64, X, at(EE, off));
        self.a.load(n.min(8), false, A, at(X, 0));
        self.a.store(8, A, at(CTX, e + 8));
        if n == 16 {
            self.a.load(8, false, A, at(X, 8));
            self.a.store(8, A, at(CTX, e + 16));
        } else {
            self.a.mov_imm(TG, (if n == 8 { SHORT } else { WORD }) as i64);
            self.a.alu(Op::Or, Width::W64, X, X, Src::Reg(TG));
        }
        self.a.store(8, X, at(CTX, e));
        self.st.logged += 1;
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
                        self.before_write(v, TG);
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

    /// The block leaves before a COP2 instruction, for the interpreter, while VU0 was not quiet at the entry or COP2 is unusable (Sapphire_Speed.md §65).
    fn cop2_may_run(&mut self, place: At) {
        let st = self.st;
        let p = self.push_purpose(P_FAULT);
        self.a.cmp_mem(4, at(CTX, X_QUIET), Src::Imm(0));
        self.exit_if(Cond::Eq, &st, place);
        self.pur(p);
        self.usable(crate::ee::status::CU2, place);
    }

    /// The block leaves before a coprocessor's instruction while Status's bit `cu` is clear, for the interpreter to raise.
    fn usable(&mut self, cu: u32, place: At) {
        let st = self.st;
        let p = self.push_purpose(P_FAULT);
        self.a.load(4, false, X, at(EE, STATUS));
        self.a.test(Width::W32, X, Src::Imm(cu as i32));
        self.exit_if(Cond::Eq, &st, place);
        self.pur(p);
    }

    /// MFC1 and MTC1 in place (Sapphire_Speed.md §65): the FPR read from the core, or written there with its undo, the state after MTC1 the one a rollback past it keeps.
    fn fpu_move(&mut self, s: &Step, place: At) {
        let w = s.word;
        let (t, f) = (rt(w), FPR + 4 * rd(w) as i32);
        self.usable(crate::ee::status::CU1, place);
        self.pur(P_WORK);
        if shape::name(w) == "mfc1" {
            if t != 0 {
                self.before_write(t, RDI);
                let h = self.dest(t, X);
                self.a.load(4, true, h, at(EE, f));
                self.put(t, h);
            }
            self.st.steps += 1;
            return;
        }
        self.pur(P_LOG);
        self.log_core(f, 4);
        self.pur(P_WORK);
        let r = self.get(t, X);
        self.a.store(4, r, at(EE, f));
        self.st.steps += 1;
        let guard = self.st.snap.map_or(0, |s| s.guard);
        self.st.snap = Some(SnapSt { base: self.st.steps, guard, dirty: self.st.dirty, pc: s.pc.wrapping_add(4), logged: self.st.logged, sync: self.st.sync, saved: [None; GUEST] });
    }

    /// A call (Sapphire_Speed.md §65): every value the block wrote handed to the helper, which stores them to the core with their undo, logs what the operation writes and calls it; the state after it is the one a rollback past it keeps.
    fn call(&mut self, s: &Step, place: At) {
        let w = s.word;
        let index = ops::index(w);
        match ops::get(index).kind {
            ops::Kind::Fpu => self.usable(crate::ee::status::CU1, place),
            ops::Kind::Cop2 => self.cop2_may_run(place),
            ops::Kind::Cpu => {}
        }
        let p = self.push_purpose(P_LOG);
        let dirty = self.st.dirty & !(1 << 32);
        for v in (1..GUEST).filter(|v| dirty >> v & 1 == 1) {
            let r = match self.home[v] {
                Home::Reg(h) => h,
                Home::Slot(k) => {
                    self.a.load(8, false, P, slot_mem(k));
                    P
                }
                Home::None => unreachable!("a dirty value has a home"),
            };
            self.a.store(8, r, at(CTX, X_VALUES + 8 * v as i32));
        }
        for (k, half) in [dirty as u64, (dirty >> 64) as u64].into_iter().enumerate() {
            self.a.mov_imm(X, half as i64);
            self.a.store(8, X, at(CTX, X_HANDED + 8 * k as i32));
        }
        let written = call_writes(w, ops::get(index));
        let logged = self.st.logged;
        self.st.logged += dirty.count_ones() + written.logs().len() as u32;
        assert!(self.st.logged <= LOG_MOST as u32, "a block's undo entries fit the log");
        self.pur(P_WORK);
        self.a.mov_imm(RDI, i64::from(w));
        self.a.mov_imm(RSI, i64::from(logged) | i64::from(self.st.steps) << 16);
        self.a.mov_imm(RDX, i64::from(index));
        self.a.call_stub(stub::CALL);
        self.pur(p);
        self.st.dirty = 0;
        for &v in written.values() {
            self.st.held[v] = false;
        }
        self.st.steps += 1;
        let guard = self.st.snap.map_or(0, |s| s.guard);
        self.st.snap = Some(SnapSt { base: self.st.steps, guard, dirty: 0, pc: s.pc.wrapping_add(4), logged: self.st.logged, sync: self.st.sync, saved: [None; GUEST] });
    }
}

/// The block's code; `prototype` as `compile` says.
pub(crate) fn lower(plan: &Plan, times: &Static, options: Options, prototype: bool) -> Option<Code> {
    let after = plan.steps.iter().find(|s| s.info.class == Class::Branch).map_or(0, |s| s.pc.wrapping_add(8));
    let mut l = L { a: X64::new(), plan, times, home: vec![Home::None; VALUES], st: St { held: [false; VALUES], dirty: 0, steps: 0, logged: 0, sync: false, branched: false, snap: None }, next_slot: 0, after, prototype, purpose: 0, data_len: 0, last_rec: None };
    l.allocate();
    let table = l.data(&prefix::table_bytes(times, options.single, options.blocking));
    assert_eq!(table, 0, "the prefix's table is the data's first");
    let entry = l.a.label();
    let body = l.a.label();
    let entered = l.a.label();
    if !prototype {
        // The entry's test: at rest for what the block reads, or the prefix's flag set for the shared exit (§14.3, §15.2).
        let moved = l.a.label();
        l.pur(P_TEST);
        l.a.bind(moved);
        l.a.store_imm(4, 1, at(RSI, X_MOVED));
        l.a.jump(entered);
        l.a.bind(entry);
        let dynamic = moved;
        l.a.load(8, false, X, at(RDI, T + layout::LAST));
        l.a.alu(Op::Add, Width::W64, X, X, Src::Imm(1));
        l.a.cmp_mem(1, at(RDI, T + layout::OPEN), Src::Imm(0));
        l.a.branch(Cond::Ne, dynamic);
        l.a.cmp_mem(1, at(RDI, T + layout::GROUP_ENDS), Src::Imm(0));
        l.a.branch(Cond::Ne, dynamic);
        l.a.cmp_mem(8, at(RDI, T + layout::FLOOR), Src::Reg(X));
        l.a.branch(Cond::Gtu, dynamic);
        for late in [false, true] {
            if late {
                l.a.alu(Op::Sub, Width::W64, X, X, Src::Imm(1));
            }
            for r in (0..128).filter(|&r| times.exposed >> r & 1 == 1 && (Timing::read_late(r) != 0) == late) {
                let field = match r {
                    124 => layout::MAC0_FREE,
                    125 => layout::MAC1_FREE,
                    126 => layout::FDIV_FREE,
                    127 => layout::COP2_FLOOR,
                    _ => layout::READY + 8 * r as i32,
                };
                l.a.cmp_mem(8, at(RDI, T + field), Src::Reg(X));
                l.a.branch(Cond::Gtu, dynamic);
            }
        }
    } else {
        l.a.bind(entry);
    }
    l.a.bind(entered);
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
    Some(l.a.finish(entry))
}
