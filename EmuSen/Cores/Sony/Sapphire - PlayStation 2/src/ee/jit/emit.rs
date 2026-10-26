//! One EE block as Cranelift IR: a straight line with side exits. The guest registers and every field of the issue model
//! the block touches are SSA values from the entry to the exit; each instruction's issue cycle is `Timing::plan` and
//! `Timing::commit` with the instruction's category, row, reads and writes made constants, and folded where the
//! block's own earlier instructions fix a value. A load or store reaches the scratchpad, RDRAM or the data cache's line
//! through the translation cache, or leaves the block before it. Each exit hands the cycles to the replay, then writes
//! everything back. Sapphire_Native.md §44.4 and §48.4 record what is computed where.

use std::mem::offset_of;

use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::types::{I8, I32, I64};
use cranelift_codegen::ir::types::I16;
use cranelift_codegen::ir::{self, AbiParam, InstBuilder, MemFlagsData, SigRef, Value};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::JITModule;
use cranelift_module::Module;

use super::super::cop0::{self, cause};
use super::super::ops::{self, Op};
use super::super::timing::{self, CATEGORIES, Category, Pair, SA, layout, rows};
use super::super::Ee;
use super::super::cache::{DCache, DTag};
use super::shape::{self, Class};
use super::{Context, DONE, FINAL, Options, Page, ROLLBACK, SHORT, kind};

const GPR: i32 = offset_of!(Ee, gpr) as i32;
const HI: i32 = offset_of!(Ee, hi) as i32;
const LO: i32 = offset_of!(Ee, lo) as i32;
const SA_: i32 = offset_of!(Ee, sa) as i32;
const PC: i32 = offset_of!(Ee, pc) as i32;
const NEXT_PC: i32 = offset_of!(Ee, next_pc) as i32;
const DELAY: i32 = offset_of!(Ee, delay) as i32;
const COP0: i32 = offset_of!(Ee, cop0) as i32;
const GROUP: i32 = offset_of!(Ee, group) as i32;
const CYCLES_SEEN: i32 = offset_of!(Ee, cycles_seen) as i32;
const UCAB_VALID: i32 = offset_of!(Ee, ucab.valid) as i32;
const TIMING: i32 = offset_of!(Ee, timing) as i32;
const X_TIMES: i32 = offset_of!(Context, times) as i32;
const X_STEPS: i32 = offset_of!(Context, steps) as i32;
const X_LIMIT: i32 = offset_of!(Context, limit) as i32;
const X_REPLAY: i32 = offset_of!(Context, replay) as i32;
const X_HOOK: i32 = offset_of!(Context, hook) as i32;
const X_SPILL: i32 = offset_of!(Context, spill) as i32;
const X_FILL: i32 = offset_of!(Context, fill) as i32;
const X_PAGES: i32 = offset_of!(Context, pages) as i32;
const X_GEN1: i32 = offset_of!(Context, gen1) as i32;
const X_HORIZON: i32 = offset_of!(Context, horizon) as i32;
const X_GUARD: i32 = offset_of!(Context, guard) as i32;
const X_BASE: i32 = offset_of!(Context, base) as i32;
const X_LOGGED: i32 = offset_of!(Context, logged) as i32;
const X_UNCACHED: i32 = offset_of!(Context, uncached) as i32;
const X_DUMMY: i32 = offset_of!(Context, dummy) as i32;
const X_LOG: i32 = offset_of!(Context, log) as i32;
const X_LEFT: i32 = offset_of!(Context, left) as i32;
const X_UNCACHED_LOADS: i32 = offset_of!(Context, uncached_loads) as i32;
/// Why a load or store left its block, `Context::left`'s index.
const LEFT_HORIZON: i64 = 0;
const LEFT_ALIGN: i64 = 1;
const LEFT_PAGE: i64 = 2;
const LEFT_MISS: i64 = 3;
const LEFT_UNCACHED: i64 = 4;
const P_TAG: i32 = offset_of!(Page, tag) as i32;
const P_HOST: i32 = offset_of!(Page, host) as i32;
const P_META: i32 = offset_of!(Page, meta) as i32;
const D_TAGS: i32 = (offset_of!(Ee, dcache) + offset_of!(DCache, tags)) as i32;
const D_DATA: i32 = (offset_of!(Ee, dcache) + offset_of!(DCache, data)) as i32;
const TAG: i64 = size_of::<DTag>() as i64;
const TAG_PFN: i32 = offset_of!(DTag, pfn) as i32;
const TAG_VALID: i32 = offset_of!(DTag, valid) as i32;
const TAG_DIRTY: i32 = offset_of!(DTag, dirty) as i32;
const TAG_LOCK: i32 = offset_of!(DTag, lock) as i32;
const _: () = assert!(size_of::<DTag>() == 8 && size_of::<Page>() == 32, "a tag is undone as 8 bytes, a page found by a shift of 5");

const fn cop0_at(r: usize) -> i32 {
    COP0 + 4 * r as i32
}
const fn t_at(field: i32) -> i32 {
    TIMING + field
}

fn trusted() -> MemFlagsData {
    MemFlagsData::trusted()
}

/// The guest's memory as the host holds it, at any alignment.
fn guest() -> MemFlagsData {
    MemFlagsData::new().with_notrap()
}

/// What a load or store does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Access {
    Load { bytes: u32, signed: bool },
    Lq,
    /// LWL, LWR (`n` 4), LDL, LDR (8): the aligned unit merged into the register, from its high end when `left`.
    LoadUnit { n: u32, left: bool },
    Store { bytes: u32 },
    Sq,
    /// SWL, SWR, SDL, SDR: the register's bytes merged into the aligned unit.
    StoreUnit { n: u32, left: bool },
    Pref,
}

fn access(name: &str) -> Access {
    match name {
        "lb" => Access::Load { bytes: 1, signed: true },
        "lbu" => Access::Load { bytes: 1, signed: false },
        "lh" => Access::Load { bytes: 2, signed: true },
        "lhu" => Access::Load { bytes: 2, signed: false },
        "lw" => Access::Load { bytes: 4, signed: true },
        "lwu" => Access::Load { bytes: 4, signed: false },
        "ld" => Access::Load { bytes: 8, signed: false },
        "lq" => Access::Lq,
        "lwl" => Access::LoadUnit { n: 4, left: true },
        "lwr" => Access::LoadUnit { n: 4, left: false },
        "ldl" => Access::LoadUnit { n: 8, left: true },
        "ldr" => Access::LoadUnit { n: 8, left: false },
        "sb" => Access::Store { bytes: 1 },
        "sh" => Access::Store { bytes: 2 },
        "sw" => Access::Store { bytes: 4 },
        "sd" => Access::Store { bytes: 8 },
        "sq" => Access::Sq,
        "swl" => Access::StoreUnit { n: 4, left: true },
        "swr" => Access::StoreUnit { n: 4, left: false },
        "sdl" => Access::StoreUnit { n: 8, left: true },
        "sdr" => Access::StoreUnit { n: 8, left: false },
        "pref" => Access::Pref,
        other => unreachable!("{other} is not a load or store"),
    }
}

fn int_type(bytes: u32) -> ir::Type {
    match bytes {
        1 => I8,
        2 => I16,
        4 => I32,
        _ => I64,
    }
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
fn sa(w: u32) -> i64 {
    i64::from(w >> 6 & 31)
}
fn simm(w: u32) -> i64 {
    i64::from(w as u16 as i16)
}
fn uimm(w: u32) -> i64 {
    i64::from(w & 0xFFFF)
}

/// The resources the issue model tracks that an integer instruction can name: the GPRs, HI/LO of both pipes and SA.
const TRACKED: usize = SA + 1;

/// A value of the issue model: a constant the emitter knows, or one computed at run time. Every one is 64 bits; a flag is
/// 0 or 1.
#[derive(Clone, Copy, Debug)]
enum Tv {
    K(i64),
    D(Value),
}
use Tv::{D, K};

/// The issue model's fields as the block has them.
#[derive(Clone, Copy)]
struct Model {
    last: Tv,
    open: Tv,
    floor: Tv,
    group_ends: Tv,
    paired: Tv,
    category: Tv,
    pipe: Tv,
    /// `last_writes`' two halves: an integer instruction names resources below 64 only.
    writes_lo: Tv,
    writes_hi: Tv,
    mac0: Option<Tv>,
    mac1: Option<Tv>,
    mac_dirty: [bool; 2],
    ready: [Option<Tv>; TRACKED],
    ready_dirty: u64,
    /// The cycle the bus is free of an earlier wait, which an uncached load reads and sets.
    mem: Option<Tv>,
    mem_dirty: bool,
}

/// Everything the block holds between its entry and an exit.
#[derive(Clone, Copy)]
struct Held {
    regs: [Option<Value>; 32],
    dirty: u32,
    /// The GPRs' bits 127..64, which LQ writes and SQ reads.
    highs: [Option<Value>; 32],
    high_dirty: u32,
    /// The undo entries the stores so far have written.
    logged: u32,
    /// The state after the last load or store, `Emitter::snaps`' index.
    snap: Option<usize>,
    /// HI0, LO0, HI1, LO1.
    hilo: [Option<Value>; 4],
    hilo_dirty: u8,
    sa: Option<Value>,
    sa_dirty: bool,
    model: Model,
    random: Value,
    group: Tv,
    sync: bool,
    steps: u32,
}

/// Where the core stands at an exit.
#[derive(Clone, Copy)]
enum At {
    /// At `pc`, outside a delay slot.
    Line(u32),
    /// At a computed address after a branch's slot.
    Jumped(Tv),
    /// At a delay slot, `next` the branch's outcome.
    Slot { pc: u32, next: Tv },
}

/// The state after a load or store, which a rollback past it writes back, and the block doing it once an exit needs it.
struct Snap {
    held: Held,
    at: At,
    block: Option<ir::Block>,
}

/// The values a block with loads or stores loads at its entry.
#[derive(Clone, Copy)]
struct Memory {
    pages: Value,
    gen1: Value,
    horizon: Value,
    lines: Value,
}

struct Emitter<'a, 'b> {
    b: &'a mut FunctionBuilder<'b>,
    ee: Value,
    ctx: Value,
    helper: SigRef,
    options: Options,
    held: Held,
    snaps: Vec<Snap>,
    memory: Option<Memory>,
    /// Emitting an exit taken only on a rare condition: its blocks are laid out after the block's straight line.
    cold: bool,
    /// Entry values the exits read: Count, Compare, the cycles Count last saw, Wired.
    count0: Value,
    compare: Value,
    seen0: Value,
    wired: Value,
    issued0: Value,
}

impl Emitter<'_, '_> {
    fn k(&mut self, v: i64) -> Value {
        self.b.ins().iconst(I64, v)
    }

    /// `k - x`.
    fn rsub(&mut self, k: i64, x: Value) -> Value {
        let k = self.k(k);
        self.b.ins().isub(k, x)
    }

    fn v(&mut self, t: Tv) -> Value {
        match t {
            K(k) => self.k(k),
            D(v) => v,
        }
    }

    fn load(&mut self, ty: ir::Type, at: i32) -> Value {
        let ee = self.ee;
        self.b.ins().load(ty, trusted(), ee, at)
    }

    fn store(&mut self, v: Value, at: i32) {
        let ee = self.ee;
        self.b.ins().store(trusted(), v, ee, at);
    }

    fn store_narrow(&mut self, ty: ir::Type, t: Tv, at: i32) {
        let v = self.v(t);
        let n = self.b.ins().ireduce(ty, v);
        self.store(n, at);
    }

    // The issue model's arithmetic, folded where both sides are known.
    fn add(&mut self, a: Tv, k: i64) -> Tv {
        match a {
            K(x) => K(x.wrapping_add(k)),
            D(v) if k == 0 => D(v),
            D(v) => D(self.b.ins().iadd_imm_s(v, k)),
        }
    }

    fn add2(&mut self, a: Tv, c: Tv) -> Tv {
        match (a, c) {
            (K(x), K(y)) => K(x.wrapping_add(y)),
            (t, K(y)) | (K(y), t) => self.add(t, y),
            (D(x), D(y)) => D(self.b.ins().iadd(x, y)),
        }
    }

    fn umax(&mut self, a: Tv, c: Tv) -> Tv {
        match (a, c) {
            (K(x), K(y)) => K((x as u64).max(y as u64) as i64),
            (K(0), t) | (t, K(0)) => t,
            _ => {
                let (x, y) = (self.v(a), self.v(c));
                D(self.b.ins().umax(x, y))
            }
        }
    }

    fn flag(&mut self, cc: IntCC, a: Tv, c: Tv) -> Tv {
        if let (IntCC::UnsignedLessThanOrEqual, K(0), _) = (cc, a, c) {
            return K(1);
        }
        if let (K(x), K(y)) = (a, c) {
            let (ux, uy) = (x as u64, y as u64);
            return K(i64::from(match cc {
                IntCC::Equal => x == y,
                IntCC::NotEqual => x != y,
                IntCC::UnsignedLessThanOrEqual => ux <= uy,
                IntCC::UnsignedLessThan => ux < uy,
                IntCC::UnsignedGreaterThan => ux > uy,
                _ => unreachable!("an unfolded condition"),
            }));
        }
        let (x, y) = (self.v(a), self.v(c));
        let f = self.b.ins().icmp(cc, x, y);
        D(self.b.ins().uextend(I64, f))
    }

    fn and(&mut self, a: Tv, c: Tv) -> Tv {
        match (a, c) {
            (K(0), _) | (_, K(0)) => K(0),
            (K(_), t) | (t, K(_)) => t,
            (D(x), D(y)) => D(self.b.ins().band(x, y)),
        }
    }

    fn not(&mut self, a: Tv) -> Tv {
        match a {
            K(x) => K(i64::from(x == 0)),
            D(v) => D(self.b.ins().bxor_imm_s(v, 1)),
        }
    }

    fn select(&mut self, c: Tv, a: Tv, e: Tv) -> Tv {
        match c {
            K(0) => e,
            K(_) => a,
            D(cv) => {
                if let (K(x), K(y)) = (a, e)
                    && x == y
                {
                    return K(x);
                }
                let (x, y) = (self.v(a), self.v(e));
                D(self.b.ins().select(cv, x, y))
            }
        }
    }

    fn bits(&mut self, a: Tv, mask: i64) -> Tv {
        match a {
            K(x) => K(x & mask),
            D(v) => D(self.b.ins().band_imm_s(v, mask)),
        }
    }

    fn or_k(&mut self, a: Tv, mask: i64) -> Tv {
        match a {
            K(x) => K(x | mask),
            D(v) => D(self.b.ins().bor_imm_s(v, mask)),
        }
    }

    // The guest's registers.
    fn reg(&mut self, r: usize) -> Value {
        if r == 0 {
            return self.k(0);
        }
        if let Some(v) = self.held.regs[r] {
            return v;
        }
        let v = self.load(I64, GPR + 16 * r as i32);
        self.held.regs[r] = Some(v);
        v
    }

    fn set(&mut self, r: usize, v: Value) {
        if r != 0 {
            self.held.regs[r] = Some(v);
            self.held.dirty |= 1 << r;
        }
    }

    fn high(&mut self, r: usize) -> Value {
        if r == 0 {
            return self.k(0);
        }
        if let Some(v) = self.held.highs[r] {
            return v;
        }
        let v = self.load(I64, GPR + 16 * r as i32 + 8);
        self.held.highs[r] = Some(v);
        v
    }

    fn set_high(&mut self, r: usize, v: Value) {
        if r != 0 {
            self.held.highs[r] = Some(v);
            self.held.high_dirty |= 1 << r;
        }
    }

    fn hilo_at(i: usize) -> i32 {
        [HI, LO, HI + 8, LO + 8][i]
    }

    fn hilo(&mut self, i: usize) -> Value {
        if let Some(v) = self.held.hilo[i] {
            return v;
        }
        let v = self.load(I64, Self::hilo_at(i));
        self.held.hilo[i] = Some(v);
        v
    }

    fn set_hilo(&mut self, i: usize, v: Value) {
        self.held.hilo[i] = Some(v);
        self.held.hilo_dirty |= 1 << i;
    }

    fn sext32(&mut self, v: Value) -> Value {
        let n = self.b.ins().ireduce(I32, v);
        self.b.ins().sextend(I64, n)
    }

    fn low32(&mut self, v: Value) -> Value {
        self.b.ins().ireduce(I32, v)
    }

    // The issue model's fields, loaded when first named.
    fn ready(&mut self, r: usize) -> Tv {
        if let Some(t) = self.held.model.ready[r] {
            return t;
        }
        let t = D(self.load(I64, t_at(layout::READY) + 8 * r as i32));
        self.held.model.ready[r] = Some(t);
        t
    }

    fn mac(&mut self, n: usize) -> Tv {
        let slot = if n == 0 { self.held.model.mac0 } else { self.held.model.mac1 };
        if let Some(t) = slot {
            return t;
        }
        let t = D(self.load(I64, t_at(if n == 0 { layout::MAC0_FREE } else { layout::MAC1_FREE })));
        if n == 0 {
            self.held.model.mac0 = Some(t);
        } else {
            self.held.model.mac1 = Some(t);
        }
        t
    }

    fn mem_free(&mut self) -> Tv {
        if let Some(t) = self.held.model.mem {
            return t;
        }
        let t = D(self.load(I64, t_at(layout::MEM_FREE)));
        self.held.model.mem = Some(t);
        t
    }

    fn set_mac(&mut self, n: usize, t: Tv) {
        if n == 0 {
            self.held.model.mac0 = Some(t);
        } else {
            self.held.model.mac1 = Some(t);
        }
        self.held.model.mac_dirty[n] = true;
    }

    /// Table 1-3's partner for category `c` beside the last instruction: (may pair, its pipe, a Y cell's stall).
    fn partner(&mut self, c: Category, p0: bool, p1: bool) -> (Tv, Tv, Tv) {
        let known = |cat: i64, pipe: i64| -> (i64, i64, i64) {
            match CATEGORIES.get(cat as usize).and_then(|&last| timing::partner_of(c, p0, p1, last, pipe as u8)) {
                Some((p, cell)) => (1, i64::from(p), i64::from(cell == Pair::Y)),
                None => (0, 0, 0),
            }
        };
        let m = self.held.model;
        match (m.category, m.pipe) {
            (K(cat), K(pipe)) => {
                let (v, p, y) = known(cat, pipe);
                (K(v), K(p), K(y))
            }
            (K(cat), pipe) => {
                let (v0, p0_, y0) = known(cat, 0);
                let (v1, p1_, y1) = known(cat, 1);
                let is1 = self.flag(IntCC::NotEqual, pipe, K(0));
                (self.select(is1, K(v1), K(v0)), self.select(is1, K(p1_), K(p0_)), self.select(is1, K(y1), K(y0)))
            }
            (cat, pipe) => {
                let (mut valid, mut pipes, mut ys) = (0i64, 0i64, 0i64);
                for k in 0..CATEGORIES.len() as i64 {
                    for p in 0..2 {
                        let (v, q, y) = known(k, p);
                        let bit = 2 * k + p;
                        valid |= v << bit;
                        pipes |= q << bit;
                        ys |= y << bit;
                    }
                }
                let catv = self.v(cat);
                let pipev = self.v(pipe);
                let twice = self.b.ins().ishl_imm_s(catv, 1);
                let index = self.b.ins().iadd(twice, pipev);
                let pick = |e: &mut Self, mask: i64| -> Tv {
                    let m = e.k(mask);
                    let s = e.b.ins().ushr(m, index);
                    D(e.b.ins().band_imm_s(s, 1))
                };
                let v = pick(self, valid);
                let p = pick(self, pipes);
                let y = pick(self, ys);
                let in_table = self.flag(IntCC::UnsignedLessThan, cat, K(CATEGORIES.len() as i64));
                (self.and(v, in_table), p, y)
            }
        }
    }

    /// One instruction charged to the issue model as `Timing::issue` charges it, and the step's `advance` (Random and
    /// the group); its cycles after it stored for the replay. The issue cycle.
    fn issue(&mut self, op: &Op, w: u32, taken: Tv) -> Tv {
        self.issue_waiting(op, w, taken, K(0))
    }

    /// `issue` with the cycles the instruction's data access waited for memory (`Issue::data`), 0 or an uncached
    /// load's at run time.
    fn issue_waiting(&mut self, op: &Op, w: u32, taken: Tv, data: Tv) -> Tv {
        let i = ops::issue(op, w);
        let data = if self.options.fixed_cost { K(0) } else { data };
        let waits = self.flag(IntCC::NotEqual, data, K(0));
        let row = timing::row_of(i.row);
        let (p0, p1) = timing::logical(i.category);
        let mac0 = matches!(i.row, rows::MULT | rows::DIV | rows::MADD | rows::MMULT | rows::MMDIV);
        let mac1 = matches!(i.row, rows::MULT1 | rows::DIV1 | rows::MADD1 | rows::MMULT | rows::MMDIV);
        let (reads, writes) = (i.reads as u64, i.writes as u64);
        debug_assert!(i.reads >> TRACKED == 0 && i.writes >> TRACKED == 0, "an integer instruction names only the GPRs, HI/LO and SA");
        let m = self.held.model;
        let fixed = self.options.fixed_cost;
        let mut earliest = K(0);
        if !fixed {
            let mut r = reads;
            while r != 0 {
                let n = r.trailing_zeros() as usize;
                r &= r - 1;
                let t = self.ready(n);
                earliest = self.umax(earliest, t);
            }
            if mac0 {
                let t = self.mac(0);
                earliest = self.umax(earliest, t);
            }
            if mac1 {
                let t = self.mac(1);
                earliest = self.umax(earliest, t);
            }
            if !matches!(waits, K(0)) {
                let mem = self.mem_free();
                let mem = self.select(waits, mem, K(0));
                earliest = self.umax(earliest, mem);
            }
        }
        let pairs = if self.options.single || fixed {
            K(0)
        } else {
            let early = self.flag(IntCC::UnsignedLessThanOrEqual, earliest, m.last);
            let hit = self.bits(m.writes_lo, (reads | writes) as i64);
            let clear = self.flag(IntCC::Equal, hit, K(0));
            let (valid, _, _) = self.partner(i.category, p0, p1);
            let a = self.and(m.open, early);
            let b = self.and(clear, valid);
            self.and(a, b)
        };
        let (_, partner_pipe, partner_y) = if self.options.single || fixed { (K(0), K(0), K(0)) } else { self.partner(i.category, p0, p1) };
        let next = self.add(m.last, 1);
        let alone = self.umax(earliest, m.floor);
        let alone = self.umax(alone, next);
        let at = self.select(pairs, m.last, alone);
        let pipe = self.select(pairs, partner_pipe, K(if p0 { 0 } else { 1 }));
        let stall = self.select(pairs, partner_y, K(0));

        let latency = if fixed { 1 } else { i64::from(row.latency.max(1)) };
        let held_at = self.add2(at, stall);
        let done = self.add(held_at, latency);
        let done = self.add2(done, data);
        if !matches!(waits, K(0)) {
            let mem = self.mem_free();
            let after = self.add(at, 1);
            let after = self.add2(after, data);
            let mem = self.select(waits, after, mem);
            self.held.model.mem = Some(mem);
            self.held.model.mem_dirty = true;
        }
        let mut wr = writes;
        while wr != 0 {
            let n = wr.trailing_zeros() as usize;
            wr &= wr - 1;
            self.held.model.ready[n] = Some(done);
            self.held.model.ready_dirty |= 1 << n;
        }
        if !fixed {
            if mac0 {
                let t = self.add(at, i64::from(row.throughput));
                self.set_mac(0, t);
            }
            if mac1 {
                let t = self.add(at, i64::from(row.throughput));
                self.set_mac(1, t);
            }
        }
        let opened = self.not(pairs);
        let widened = self.or_k(m.writes_lo, writes as i64);
        let writes_lo = self.select(pairs, widened, K(writes as i64));
        let writes_hi = self.select(pairs, m.writes_hi, K(0));
        let floor = self.add(held_at, 1);
        let floor = if self.options.blocking { self.add2(floor, data) } else { floor };
        let ends = self.flag(IntCC::NotEqual, m.group_ends, K(0));
        let less = self.add(m.group_ends, -1);
        let counted = self.select(ends, less, K(0));
        let zero = self.flag(IntCC::Equal, counted, K(0));
        let closes = self.and(ends, zero);
        let open = self.select(closes, K(0), opened);
        let group_ends = self.select(taken, K(1), counted);
        self.held.model = Model {
            last: at,
            open,
            floor,
            group_ends,
            paired: pairs,
            category: K(i64::from(i.category as u8)),
            pipe,
            writes_lo,
            writes_hi,
            ..self.held.model
        };

        // The step's advance: Random once an issue cycle that does not pair (Wired is not written in a block).
        let r = self.held.random;
        let wired = self.wired;
        let low = self.b.ins().icmp(IntCC::UnsignedLessThanOrEqual, r, wired);
        let high = self.b.ins().icmp_imm_s(IntCC::UnsignedGreaterThan, r, 47);
        let wraps = self.b.ins().bor(low, high);
        let down = self.b.ins().iadd_imm_s(r, -1);
        let top = self.k(47);
        let moved = self.b.ins().select(wraps, top, down);
        self.held.random = match pairs {
            K(0) => moved,
            K(_) => r,
            D(p) => self.b.ins().select(p, r, moved),
        };
        self.held.group = self.select(pairs, K(2), K(1));

        let cycles = self.add(at, 1);
        let c = self.v(cycles);
        let ctx = self.ctx;
        self.b.ins().store(trusted(), c, ctx, X_TIMES + 8 * self.held.steps as i32);
        self.held.steps += 1;
        at
    }

    /// Writes back everything the block holds, and the core's place `at`.
    fn write_back(&mut self, at: At) {
        let h = self.held;
        for r in 1..32 {
            if h.dirty >> r & 1 == 1 {
                let v = h.regs[r].expect("a dirty register is held");
                self.store(v, GPR + 16 * r as i32);
            }
        }
        for r in 1..32 {
            if h.high_dirty >> r & 1 == 1 {
                let v = h.highs[r].expect("a dirty register is held");
                self.store(v, GPR + 16 * r as i32 + 8);
            }
        }
        for i in 0..4 {
            if h.hilo_dirty >> i & 1 == 1 {
                let v = h.hilo[i].expect("a dirty half is held");
                self.store(v, Self::hilo_at(i));
            }
        }
        if h.sa_dirty {
            let v = h.sa.expect("SA is held");
            self.store(v, SA_);
        }
        if h.steps > 0 {
            let m = h.model;
            for (field, t) in [(layout::LAST, m.last), (layout::FLOOR, m.floor), (layout::LAST_WRITES, m.writes_lo), (layout::LAST_WRITES + 8, m.writes_hi)] {
                let v = self.v(t);
                self.store(v, t_at(field));
            }
            for (field, t) in [(layout::OPEN, m.open), (layout::GROUP_ENDS, m.group_ends), (layout::PAIRED, m.paired), (layout::LAST_CATEGORY, m.category), (layout::LAST_PIPE, m.pipe)] {
                self.store_narrow(I8, t, t_at(field));
            }
            for r in 0..TRACKED {
                if m.ready_dirty >> r & 1 == 1 {
                    let v = self.v(m.ready[r].expect("a dirty ready cycle is held"));
                    self.store(v, t_at(layout::READY) + 8 * r as i32);
                }
            }
            if m.mem_dirty {
                let v = self.v(m.mem.expect("a dirty bus is held"));
                self.store(v, t_at(layout::MEM_FREE));
            }
            for n in 0..2 {
                if m.mac_dirty[n] {
                    let t = if n == 0 { m.mac0 } else { m.mac1 }.expect("a dirty unit is held");
                    let v = self.v(t);
                    self.store(v, t_at(if n == 0 { layout::MAC0_FREE } else { layout::MAC1_FREE }));
                }
            }
            let issued = self.b.ins().iadd_imm_s(self.issued0, i64::from(h.steps));
            self.store(issued, t_at(layout::INSTRUCTIONS));
            // Count, Compare's match and the cycles seen, as each step's advance would have left them in turn.
            let end = self.add(m.last, 1);
            let end = self.v(end);
            let delta = self.b.ins().isub(end, self.seen0);
            let count = self.b.ins().iadd(self.count0, delta);
            let count32 = self.b.ins().ireduce(I32, count);
            self.store(count32, cop0_at(cop0::COUNT));
            self.store(end, CYCLES_SEEN);
            let gap = self.b.ins().isub(self.compare, self.count0);
            let gap = self.b.ins().iadd_imm_s(gap, -1);
            let gap = self.b.ins().band_imm_s(gap, 0xFFFF_FFFF);
            let passed = self.b.ins().icmp(IntCC::UnsignedLessThan, gap, delta);
            let cause = self.load(I32, cop0_at(cop0::CAUSE));
            let ip7 = self.b.ins().bor_imm_s(cause, i64::from(cause::IP7));
            let cause = self.b.ins().select(passed, ip7, cause);
            self.store(cause, cop0_at(cop0::CAUSE));
            let r = self.b.ins().ireduce(I32, h.random);
            self.store(r, cop0_at(cop0::RANDOM));
            self.store_narrow(I8, h.group, GROUP);
        }
        if h.sync {
            let z = self.b.ins().iconst(I8, 0);
            self.store(z, UCAB_VALID);
        }
        let (pc, next, delay) = match at {
            At::Line(pc) => (K(i64::from(pc)), K(i64::from(pc.wrapping_add(4))), 0),
            At::Jumped(t) => {
                let n = self.add(t, 4);
                (t, n, 0)
            }
            At::Slot { pc, next } => (K(i64::from(pc)), next, 1),
        };
        self.store_narrow(I32, pc, PC);
        self.store_narrow(I32, next, NEXT_PC);
        let d = self.b.ins().iconst(I8, delay);
        self.store(d, DELAY);
    }

    fn steps_out(&mut self, steps: u32) {
        let s = self.b.ins().iconst(I32, i64::from(steps));
        let ctx = self.ctx;
        self.b.ins().store(trusted(), s, ctx, X_STEPS);
    }

    fn call_helper(&mut self, at: i32, steps: u32) -> Value {
        let ctx = self.ctx;
        let f = self.b.ins().load(I64, trusted(), ctx, at);
        let s = self.b.ins().iconst(I32, i64::from(steps));
        let call = self.b.ins().call_indirect(self.helper, f, &[ctx, s]);
        self.b.inst_results(call)[0]
    }

    fn ret(&mut self, exit: u32) {
        let v = self.b.ins().iconst(I32, i64::from(exit));
        self.b.ins().return_(&[v]);
    }

    /// The exit at the current point, the core at `at` after the block's steps so far.
    fn exit(&mut self, at: At) {
        let steps = self.held.steps;
        if steps == 0 || self.options.verify {
            // Nothing ran, or each step was written back and replayed by the verifier's calls.
            self.steps_out(steps);
            self.ret(DONE);
            return;
        }
        // The last load or store before this exit: the DMAC may not run within its index's times, and a rollback that
        // keeps it writes back the state after it (Sapphire_Native.md §48.5).
        let snap = self.held.snap;
        let base = snap.map_or(0, |i| self.snaps[i].held.steps);
        if self.memory.is_some() {
            let ctx = self.ctx;
            for (field, v) in [(X_GUARD, base.saturating_sub(1)), (X_BASE, base), (X_LOGGED, self.held.logged)] {
                let k = self.b.ins().iconst(I32, i64::from(v));
                self.b.ins().store(trusted(), k, ctx, field);
            }
        }
        let kept = self.call_helper(X_REPLAY, steps);
        let all = self.b.ins().icmp_imm_s(IntCC::Equal, kept, i64::from(steps));
        let commit = self.b.create_block();
        let back = self.b.create_block();
        self.b.ins().brif(all, commit, &[], back, &[]);
        self.b.seal_block(commit);
        self.b.seal_block(back);
        self.b.set_cold_block(back);
        if self.cold {
            self.b.set_cold_block(commit);
        }
        self.b.switch_to_block(back);
        match snap {
            Some(i) if base < steps => {
                let partial = self.partial(i);
                let undo = self.b.create_block();
                self.b.set_cold_block(undo);
                let past = self.b.ins().icmp_imm_s(IntCC::UnsignedGreaterThanOrEqual, kept, i64::from(base));
                self.b.ins().brif(past, partial, &[], undo, &[]);
                self.b.seal_block(undo);
                self.b.switch_to_block(undo);
                self.ret(ROLLBACK);
            }
            _ => self.ret(ROLLBACK),
        }
        self.b.switch_to_block(commit);
        self.write_back(at);
        self.steps_out(steps);
        self.ret(DONE);
    }

    /// The block writing back the state after the load or store `i`, made when an exit first needs it and filled at the
    /// function's end.
    fn partial(&mut self, i: usize) -> ir::Block {
        if let Some(b) = self.snaps[i].block {
            return b;
        }
        let b = self.b.create_block();
        self.b.set_cold_block(b);
        self.snaps[i].block = Some(b);
        b
    }

    /// Goes on in a new block where `ok` holds, to `out` otherwise.
    fn go_if(&mut self, ok: Value, out: ir::Block, why: i64) {
        let on = self.b.create_block();
        let w = self.k(why);
        self.b.ins().brif(ok, on, &[], out, &[w.into()]);
        self.b.seal_block(on);
        self.b.switch_to_block(on);
    }

    /// An exit taken when `cond` holds, the block going on otherwise.
    fn exit_if(&mut self, cond: Tv, at: At) {
        if let K(0) = cond {
            return;
        }
        let c = self.v(cond);
        let out = self.b.create_block();
        let on = self.b.create_block();
        self.b.ins().brif(c, out, &[], on, &[]);
        self.b.seal_block(out);
        self.b.seal_block(on);
        self.b.set_cold_block(out);
        self.b.switch_to_block(out);
        let kept = self.held;
        self.cold = true;
        self.exit(at);
        self.cold = false;
        self.held = kept;
        self.b.switch_to_block(on);
    }

    /// After a step: with the verifier, everything written back and its call made, leaving if an interrupt is due.
    fn after(&mut self, at: At, last: bool) {
        if self.options.spill {
            let h = self.held;
            let ctx = self.ctx;
            for r in 1..32 {
                if h.dirty >> r & 1 == 1 {
                    self.b.ins().store(trusted(), h.regs[r].expect("held"), ctx, X_SPILL + 8 * r as i32);
                }
            }
            for i in 0..4 {
                if h.hilo_dirty >> i & 1 == 1 {
                    self.b.ins().store(trusted(), h.hilo[i].expect("held"), ctx, X_SPILL + 8 * (32 + i as i32));
                }
            }
        }
        if !self.options.verify {
            return;
        }
        self.write_back(at);
        let steps = self.held.steps;
        let due = self.call_helper(X_HOOK, steps | if last { FINAL } else { 0 });
        let out = self.b.create_block();
        let on = self.b.create_block();
        self.b.ins().brif(due, out, &[], on, &[]);
        self.b.seal_block(out);
        self.b.seal_block(on);
        self.b.switch_to_block(out);
        self.steps_out(steps);
        self.ret(DONE);
        self.b.switch_to_block(on);
    }

    /// The condition under which an instruction raises its exception (overflow or a trap), before it changes anything.
    fn fault(&mut self, name: &str, w: u32) -> Tv {
        let word_overflow = |e: &mut Self, sub: bool, b: Value| -> Tv {
            let a = e.reg(rs(w));
            let (a, b) = (e.low32(a), e.low32(b));
            let s = if sub { e.b.ins().isub(a, b) } else { e.b.ins().iadd(a, b) };
            let x = e.b.ins().bxor(a, s);
            let y = if sub { e.b.ins().bxor(a, b) } else { e.b.ins().bxor(b, s) };
            let o = e.b.ins().band(x, y);
            let f = e.b.ins().icmp_imm_s(IntCC::SignedLessThan, o, 0);
            D(e.b.ins().uextend(I64, f))
        };
        let dword_overflow = |e: &mut Self, sub: bool, b: Value| -> Tv {
            let a = e.reg(rs(w));
            let s = if sub { e.b.ins().isub(a, b) } else { e.b.ins().iadd(a, b) };
            let x = e.b.ins().bxor(a, s);
            let y = if sub { e.b.ins().bxor(a, b) } else { e.b.ins().bxor(b, s) };
            let o = e.b.ins().band(x, y);
            let f = e.b.ins().icmp_imm_s(IntCC::SignedLessThan, o, 0);
            D(e.b.ins().uextend(I64, f))
        };
        let cmp = |e: &mut Self, cc: IntCC, imm: bool| -> Tv {
            let a = e.reg(rs(w));
            let b = if imm { e.k(simm(w)) } else { e.reg(rt(w)) };
            let f = e.b.ins().icmp(cc, a, b);
            D(e.b.ins().uextend(I64, f))
        };
        match name {
            "add" | "sub" => {
                let b = self.reg(rt(w));
                word_overflow(self, name == "sub", b)
            }
            "addi" => {
                let b = self.k(simm(w));
                word_overflow(self, false, b)
            }
            "dadd" | "dsub" => {
                let b = self.reg(rt(w));
                dword_overflow(self, name == "dsub", b)
            }
            "daddi" => {
                let b = self.k(simm(w));
                dword_overflow(self, false, b)
            }
            "tge" => cmp(self, IntCC::SignedGreaterThanOrEqual, false),
            "tgeu" => cmp(self, IntCC::UnsignedGreaterThanOrEqual, false),
            "tlt" => cmp(self, IntCC::SignedLessThan, false),
            "tltu" => cmp(self, IntCC::UnsignedLessThan, false),
            "teq" => cmp(self, IntCC::Equal, false),
            "tne" => cmp(self, IntCC::NotEqual, false),
            "tgei" => cmp(self, IntCC::SignedGreaterThanOrEqual, true),
            "tgeiu" => cmp(self, IntCC::UnsignedGreaterThanOrEqual, true),
            "tlti" => cmp(self, IntCC::SignedLessThan, true),
            "tltiu" => cmp(self, IntCC::UnsignedLessThan, true),
            "teqi" => cmp(self, IntCC::Equal, true),
            "tnei" => cmp(self, IntCC::NotEqual, true),
            _ => K(0),
        }
    }

    fn product(&mut self, w: u32, upper: bool, p: Value) {
        let lo = self.sext32(p);
        let high = self.b.ins().sshr_imm_s(p, 32);
        let hi = self.sext32(high);
        let (h, l) = if upper { (2, 3) } else { (0, 1) };
        self.set_hilo(l, lo);
        self.set_hilo(h, hi);
        self.set(rd(w), lo);
    }

    fn operands32(&mut self, w: u32, signed: bool) -> (Value, Value) {
        let (a, b) = (self.reg(rs(w)), self.reg(rt(w)));
        let (a, b) = (self.low32(a), self.low32(b));
        if signed { (self.b.ins().sextend(I64, a), self.b.ins().sextend(I64, b)) } else { (self.b.ins().uextend(I64, a), self.b.ins().uextend(I64, b)) }
    }

    fn accumulator(&mut self, upper: bool) -> Value {
        let (h, l) = if upper { (2, 3) } else { (0, 1) };
        let hi = self.hilo(h);
        let lo = self.hilo(l);
        let hi = self.b.ins().ishl_imm_s(hi, 32);
        let lo = self.b.ins().band_imm_s(lo, 0xFFFF_FFFF);
        self.b.ins().bor(hi, lo)
    }

    fn divide(&mut self, w: u32, upper: bool, signed: bool) {
        let (a, b) = (self.reg(rs(w)), self.reg(rt(w)));
        let (n, d) = (self.low32(a), self.low32(b));
        let zero = self.b.ins().icmp_imm_s(IntCC::Equal, d, 0);
        let (q, r) = if signed {
            let min = self.b.ins().icmp_imm_s(IntCC::Equal, n, i64::from(i32::MIN));
            let minus = self.b.ins().icmp_imm_s(IntCC::Equal, d, -1);
            let wraps = self.b.ins().band(min, minus);
            let unsafe_ = self.b.ins().bor(zero, wraps);
            let one = self.b.ins().iconst(I32, 1);
            let safe = self.b.ins().select(unsafe_, one, d);
            let q = self.b.ins().sdiv(n, safe);
            let r = self.b.ins().srem(n, safe);
            let negative = self.b.ins().icmp_imm_s(IntCC::SignedLessThan, n, 0);
            let all = self.b.ins().iconst(I32, -1);
            let by_zero = self.b.ins().select(negative, one, all);
            (self.b.ins().select(zero, by_zero, q), self.b.ins().select(zero, n, r))
        } else {
            let one = self.b.ins().iconst(I32, 1);
            let safe = self.b.ins().select(zero, one, d);
            let q = self.b.ins().udiv(n, safe);
            let r = self.b.ins().urem(n, safe);
            let all = self.b.ins().iconst(I32, -1);
            (self.b.ins().select(zero, all, q), self.b.ins().select(zero, n, r))
        };
        let (h, l) = if upper { (2, 3) } else { (0, 1) };
        let q = self.b.ins().sextend(I64, q);
        let r = self.b.ins().sextend(I64, r);
        self.set_hilo(l, q);
        self.set_hilo(h, r);
    }

    /// An instruction's effect on the registers, as `ops` computes it.
    fn compute(&mut self, name: &str, w: u32) {
        let (s, t, d) = (rs(w), rt(w), rd(w));
        macro_rules! three {
            ($f:ident) => {{
                let (a, b) = (self.reg(s), self.reg(t));
                let v = self.b.ins().$f(a, b);
                self.set(d, v);
            }};
        }
        macro_rules! word3 {
            ($f:ident) => {{
                let (a, b) = (self.reg(s), self.reg(t));
                let v = self.b.ins().$f(a, b);
                let v = self.sext32(v);
                self.set(d, v);
            }};
        }
        macro_rules! imm {
            ($f:ident, $k:expr) => {{
                let a = self.reg(s);
                let v = self.b.ins().$f(a, $k);
                self.set(t, v);
            }};
        }
        macro_rules! shift32 {
            ($f:ident, $amount:expr) => {{
                let a = self.reg(t);
                let a = self.low32(a);
                let v = self.b.ins().$f(a, $amount);
                let v = self.b.ins().sextend(I64, v);
                self.set(d, v);
            }};
        }
        macro_rules! shift64 {
            ($f:ident, $amount:expr) => {{
                let a = self.reg(t);
                let v = self.b.ins().$f(a, $amount);
                self.set(d, v);
            }};
        }
        macro_rules! set_flag {
            ($r:expr, $cc:expr, $a:expr, $b:expr) => {{
                let f = self.b.ins().icmp($cc, $a, $b);
                let v = self.b.ins().uextend(I64, f);
                self.set($r, v);
            }};
        }
        match name {
            "add" | "addu" => word3!(iadd),
            "sub" | "subu" => word3!(isub),
            "addi" | "addiu" => {
                let a = self.reg(s);
                let v = self.b.ins().iadd_imm_s(a, simm(w));
                let v = self.sext32(v);
                self.set(t, v);
            }
            "dadd" | "daddu" => three!(iadd),
            "dsub" | "dsubu" => three!(isub),
            "daddi" | "daddiu" => imm!(iadd_imm_s, simm(w)),
            "and" => three!(band),
            "or" => three!(bor),
            "xor" => three!(bxor),
            "nor" => {
                let (a, b) = (self.reg(s), self.reg(t));
                let v = self.b.ins().bor(a, b);
                let v = self.b.ins().bnot(v);
                self.set(d, v);
            }
            "andi" => imm!(band_imm_s, uimm(w)),
            "ori" => imm!(bor_imm_s, uimm(w)),
            "xori" => imm!(bxor_imm_s, uimm(w)),
            "lui" => {
                let v = self.k(i64::from((w << 16) as i32));
                self.set(t, v);
            }
            "slt" => {
                let (a, b) = (self.reg(s), self.reg(t));
                set_flag!(d, IntCC::SignedLessThan, a, b)
            }
            "sltu" => {
                let (a, b) = (self.reg(s), self.reg(t));
                set_flag!(d, IntCC::UnsignedLessThan, a, b)
            }
            "slti" => {
                let (a, b) = (self.reg(s), self.k(simm(w)));
                set_flag!(t, IntCC::SignedLessThan, a, b)
            }
            "sltiu" => {
                let (a, b) = (self.reg(s), self.k(simm(w)));
                set_flag!(t, IntCC::UnsignedLessThan, a, b)
            }
            "movz" | "movn" => {
                let (a, c, old) = (self.reg(s), self.reg(t), self.reg(d));
                let v = if name == "movz" { self.b.ins().select(c, old, a) } else { self.b.ins().select(c, a, old) };
                self.set(d, v);
            }
            "sll" => shift32!(ishl_imm_s, sa(w)),
            "srl" => shift32!(ushr_imm_s, sa(w)),
            "sra" => shift32!(sshr_imm_s, sa(w)),
            "sllv" | "srlv" | "srav" => {
                let amount = self.reg(s);
                let amount = self.low32(amount);
                let amount = self.b.ins().band_imm_s(amount, 31);
                match name {
                    "sllv" => shift32!(ishl, amount),
                    "srlv" => shift32!(ushr, amount),
                    _ => shift32!(sshr, amount),
                }
            }
            "dsll" => shift64!(ishl_imm_s, sa(w)),
            "dsrl" => shift64!(ushr_imm_s, sa(w)),
            "dsra" => shift64!(sshr_imm_s, sa(w)),
            "dsll32" => shift64!(ishl_imm_s, sa(w) + 32),
            "dsrl32" => shift64!(ushr_imm_s, sa(w) + 32),
            "dsra32" => shift64!(sshr_imm_s, sa(w) + 32),
            "dsllv" | "dsrlv" | "dsrav" => {
                let amount = self.reg(s);
                let amount = self.b.ins().band_imm_s(amount, 63);
                match name {
                    "dsllv" => shift64!(ishl, amount),
                    "dsrlv" => shift64!(ushr, amount),
                    _ => shift64!(sshr, amount),
                }
            }
            "mfhi" | "mflo" | "mfhi1" | "mflo1" => {
                let i = match name {
                    "mfhi" => 0,
                    "mflo" => 1,
                    "mfhi1" => 2,
                    _ => 3,
                };
                let v = self.hilo(i);
                self.set(d, v);
            }
            "mthi" | "mtlo" | "mthi1" | "mtlo1" => {
                let i = match name {
                    "mthi" => 0,
                    "mtlo" => 1,
                    "mthi1" => 2,
                    _ => 3,
                };
                let v = self.reg(s);
                self.set_hilo(i, v);
            }
            "mult" | "mult1" | "multu" | "multu1" => {
                let (a, b) = self.operands32(w, !name.starts_with("multu"));
                let p = self.b.ins().imul(a, b);
                self.product(w, name.ends_with('1'), p);
            }
            "madd" | "madd1" | "maddu" | "maddu1" => {
                let upper = name.ends_with('1');
                let (a, b) = self.operands32(w, !name.starts_with("maddu"));
                let p = self.b.ins().imul(a, b);
                let acc = self.accumulator(upper);
                let p = self.b.ins().iadd(acc, p);
                self.product(w, upper, p);
            }
            "div" | "div1" | "divu" | "divu1" => self.divide(w, name.ends_with('1'), !name.starts_with("divu")),
            "mfsa" => {
                let v = match self.held.sa {
                    Some(v) => v,
                    None => {
                        let v = self.load(I64, SA_);
                        self.held.sa = Some(v);
                        v
                    }
                };
                self.set(d, v);
            }
            "mtsa" | "mtsab" | "mtsah" => {
                let a = self.reg(s);
                let v = match name {
                    "mtsa" => self.b.ins().band_imm_s(a, 15),
                    "mtsab" => {
                        let x = self.b.ins().bxor_imm_s(a, uimm(w));
                        self.b.ins().band_imm_s(x, 15)
                    }
                    _ => {
                        let x = self.b.ins().bxor_imm_s(a, uimm(w));
                        let x = self.b.ins().band_imm_s(x, 7);
                        self.b.ins().ishl_imm_s(x, 1)
                    }
                };
                self.held.sa = Some(v);
                self.held.sa_dirty = true;
            }
            "sync" => self.held.sync = true,
            "sync.p" => {}
            n if n.starts_with('t') => {}
            other => unreachable!("{other} is not a plain instruction of S36"),
        }
    }

    /// A plain instruction at `pc`; `slot` the branch's outcome when it is a delay slot.
    fn plain(&mut self, w: u32, pc: u32, slot: Option<Tv>) {
        let name = shape::name(w);
        let op = ops::get(ops::index(w));
        let before = match slot {
            Some(next) => At::Slot { pc, next },
            None => At::Line(pc),
        };
        let fault = self.fault(name, w);
        self.exit_if(fault, before);
        self.compute(name, w);
        self.issue(op, w, K(0));
    }

    /// A plain or memory instruction at `pc`; `slot` the branch's outcome when it is a delay slot.
    fn straight(&mut self, w: u32, pc: u32, slot: Option<Tv>) {
        match shape::class(w) {
            Class::Memory => self.memory(w, pc, slot),
            _ => self.plain(w, pc, slot),
        }
    }

    /// The host address of the 4 KiB page of `a` and its translation cache's meaning, the fill called for a page not
    /// found; to `slow` where the fill cannot (Sapphire_Native.md §48.3).
    fn page(&mut self, a: Value, slow: ir::Block) -> (Value, Value) {
        let m = self.memory.expect("a block with loads or stores");
        // `jit::page_index`.
        let low = self.b.ins().ushr_imm_s(a, 12);
        let top = self.b.ins().ushr_imm_s(a, 24);
        let index = self.b.ins().bxor(low, top);
        let index = self.b.ins().band_imm_s(index, super::PAGES as i64 - 1);
        let offset = self.b.ins().ishl_imm_s(index, 5);
        let entry = self.b.ins().iadd(m.pages, offset);
        let tag = self.b.ins().load(I64, trusted(), entry, P_TAG);
        let page = self.b.ins().band_imm_s(a, 0xFFFF_F000);
        let want = self.b.ins().bor(m.gen1, page);
        let found = self.b.ins().icmp(IntCC::Equal, tag, want);
        let miss = self.b.create_block();
        self.b.set_cold_block(miss);
        let known = self.b.create_block();
        self.b.ins().brif(found, known, &[], miss, &[]);
        self.b.seal_block(miss);
        self.b.switch_to_block(miss);
        let ctx = self.ctx;
        let f = self.b.ins().load(I64, trusted(), ctx, X_FILL);
        let a32 = self.b.ins().ireduce(I32, a);
        let call = self.b.ins().call_indirect(self.helper, f, &[ctx, a32]);
        let filled = self.b.inst_results(call)[0];
        let w = self.k(LEFT_PAGE);
        self.b.ins().brif(filled, known, &[], slow, &[w.into()]);
        self.b.seal_block(known);
        self.b.switch_to_block(known);
        let host = self.b.ins().load(I64, trusted(), entry, P_HOST);
        let meta = self.b.ins().load(I64, trusted(), entry, P_META);
        (host, meta)
    }

    /// A load, store or PREF at `pc`, `slot` as in `plain`: its address translated through the translation cache, the
    /// scratchpad and RDRAM reached directly and a cached page through the data cache's tags, hits only; everything
    /// else (a misaligned address, a page the cache cannot serve, a miss, the board's next event reached) leaves the
    /// block before it, to the interpreter (Sapphire_Native.md §48.4).
    fn memory(&mut self, w: u32, pc: u32, slot: Option<Tv>) {
        let name = shape::name(w);
        let op = ops::get(ops::index(w));
        let acc = access(name);
        let before = match slot {
            Some(next) => At::Slot { pc, next },
            None => At::Line(pc),
        };
        let kept = self.held;
        let m = self.memory.expect("a block with loads or stores");
        let slow = self.b.create_block();
        self.b.set_cold_block(slow);
        let why = self.b.append_block_param(slow, I64);
        if kept.steps > 0 && !self.options.verify {
            // The times of the instructions before this one stay short of the board's next event, so that no DMAC step
            // falls between them and this access.
            let t = self.add(kept.model.last, 1);
            let t = self.v(t);
            let ok = self.b.ins().icmp(IntCC::UnsignedLessThan, t, m.horizon);
            self.go_if(ok, slow, LEFT_HORIZON);
        }
        let base = self.reg(rs(w));
        let ea = self.b.ins().iadd_imm_s(base, simm(w));
        let va = self.b.ins().band_imm_s(ea, 0xFFFF_FFFF);
        let a = match acc {
            Access::Load { bytes, .. } | Access::Store { bytes } if bytes > 1 => {
                let low = self.b.ins().band_imm_s(va, i64::from(bytes - 1));
                let ok = self.b.ins().icmp_imm_s(IntCC::Equal, low, 0);
                self.go_if(ok, slow, LEFT_ALIGN);
                va
            }
            Access::Lq | Access::Sq => self.b.ins().band_imm_s(va, 0xFFFF_FFF0),
            Access::LoadUnit { n, .. } | Access::StoreUnit { n, .. } => self.b.ins().band_imm_s(va, 0xFFFF_FFFF & !i64::from(n - 1)),
            _ => va,
        };
        let store = matches!(acc, Access::Store { .. } | Access::Sq | Access::StoreUnit { .. });
        let (host, meta) = self.page(a, slow);
        let class = self.b.ins().band_imm_s(meta, 3);
        let cached = self.b.ins().icmp_imm_s(IntCC::Equal, class, kind::CACHED as i64);
        let line = self.b.create_block();
        let flat = self.b.create_block();
        let join = self.b.create_block();
        let ptr = self.b.append_block_param(join, I64);
        let tagp = self.b.append_block_param(join, I64);
        let unc = self.b.append_block_param(join, I64);
        self.b.ins().brif(cached, line, &[], flat, &[]);
        self.b.seal_block(line);
        self.b.seal_block(flat);

        // A cached page: a hit in either way of the set, which the interpreter's `DCache::hit` finds way 0 first.
        self.b.switch_to_block(line);
        if store {
            let wr = self.b.ins().band_imm_s(meta, kind::WRITABLE as i64);
            self.go_if(wr, slow, LEFT_PAGE);
        }
        let set = self.b.ins().ushr_imm_s(a, 6);
        let set = self.b.ins().band_imm_s(set, 63);
        let tags_off = self.b.ins().imul_imm_s(set, 2 * TAG);
        let tags = self.b.ins().iadd(self.ee, tags_off);
        let ppage = self.b.ins().ushr_imm_s(meta, 32);
        let ppage = self.b.ins().ireduce(I32, ppage);
        let mut hits = [None; 2];
        for (way, h) in hits.iter_mut().enumerate() {
            let at = D_TAGS + (TAG * way as i64) as i32;
            let pfn = self.b.ins().load(I32, trusted(), tags, at + TAG_PFN);
            let valid = self.b.ins().load(I8, trusted(), tags, at + TAG_VALID);
            let same = self.b.ins().icmp(IntCC::Equal, pfn, ppage);
            let on = self.b.ins().icmp_imm_s(IntCC::NotEqual, valid, 0);
            *h = Some(self.b.ins().band(same, on));
        }
        let (hit0, hit1) = (hits[0].expect("way 0"), hits[1].expect("way 1"));
        let any = self.b.ins().bor(hit0, hit1);
        self.go_if(any, slow, LEFT_MISS);
        let one = self.k(1);
        let zero = self.k(0);
        let way = self.b.ins().select(hit0, zero, one);
        let set_bytes = self.b.ins().ishl_imm_s(set, 7);
        let way_bytes = self.b.ins().ishl_imm_s(way, 6);
        let within = self.b.ins().band_imm_s(a, 63);
        let l = self.b.ins().iadd(m.lines, set_bytes);
        let l = self.b.ins().iadd(l, way_bytes);
        let l = self.b.ins().iadd(l, within);
        let tag_way = self.b.ins().imul_imm_s(way, TAG);
        let t = self.b.ins().iadd(tags, tag_way);
        let t = self.b.ins().iadd_imm_s(t, i64::from(D_TAGS));
        self.b.ins().jump(join, &[l.into(), t.into(), zero.into()]);

        // The scratchpad, and RDRAM past the caches for a store.
        self.b.switch_to_block(flat);
        let is_spr = self.b.ins().icmp_imm_s(IntCC::Equal, class, kind::SCRATCHPAD as i64);
        let is_unc = self.b.ins().icmp_imm_s(IntCC::Equal, class, kind::UNCACHED as i64);
        let ok = match acc {
            Access::Store { .. } | Access::Sq => {
                let either = self.b.ins().bor(is_spr, is_unc);
                let wr = self.b.ins().band_imm_s(meta, kind::WRITABLE as i64);
                let wr = self.b.ins().icmp_imm_s(IntCC::NotEqual, wr, 0);
                self.b.ins().band(either, wr)
            }
            Access::StoreUnit { .. } => {
                let wr = self.b.ins().band_imm_s(meta, kind::WRITABLE as i64);
                let wr = self.b.ins().icmp_imm_s(IntCC::NotEqual, wr, 0);
                self.b.ins().band(is_spr, wr)
            }
            Access::Pref => self.b.ins().bor(is_spr, is_unc),
            _ => {
                // RDRAM past the caches, but not the accelerated buffer, which holds its own line.
                let accel = self.b.ins().band_imm_s(meta, kind::ACCELERATED as i64);
                let plain = self.b.ins().icmp_imm_s(IntCC::Equal, accel, 0);
                let unc = self.b.ins().band(is_unc, plain);
                self.b.ins().bor(is_spr, unc)
            }
        };
        let page = self.k(LEFT_PAGE);
        let reason = if store {
            page
        } else {
            let unc = self.k(LEFT_UNCACHED);
            self.b.ins().select(is_unc, unc, page)
        };
        let on = self.b.create_block();
        self.b.ins().brif(ok, on, &[], slow, &[reason.into()]);
        self.b.seal_block(on);
        self.b.switch_to_block(on);
        let within = self.b.ins().band_imm_s(a, 0xFFF);
        let p = self.b.ins().iadd(host, within);
        let ctx = self.ctx;
        let dummy = self.b.ins().iadd_imm_s(ctx, i64::from(X_DUMMY));
        let u = self.b.ins().uextend(I64, is_unc);
        self.b.ins().jump(join, &[p.into(), dummy.into(), u.into()]);
        self.b.seal_block(join);
        self.b.switch_to_block(join);

        let rt_ = rt(w);
        let loads = matches!(acc, Access::Load { .. } | Access::Lq | Access::LoadUnit { .. });
        // An uncached load waits for the bus (`timing::UNCACHED`) and is counted as the board counts `Bus::read`.
        let data = if loads {
            let ctx = self.ctx;
            let n = self.b.ins().load(I64, trusted(), ctx, X_UNCACHED_LOADS);
            let n = self.b.ins().iadd(n, unc);
            self.b.ins().store(trusted(), n, ctx, X_UNCACHED_LOADS);
            D(self.b.ins().imul_imm_s(unc, timing::UNCACHED as i64))
        } else {
            K(0)
        };
        match acc {
            Access::Load { bytes, signed } => {
                let v = self.b.ins().load(int_type(bytes), guest(), ptr, 0);
                let v = match (bytes, signed) {
                    (8, _) => v,
                    (_, true) => self.b.ins().sextend(I64, v),
                    (_, false) => self.b.ins().uextend(I64, v),
                };
                self.set(rt_, v);
            }
            Access::Lq => {
                let lo = self.b.ins().load(I64, guest(), ptr, 0);
                let hi = self.b.ins().load(I64, guest(), ptr, 8);
                self.set(rt_, lo);
                self.set_high(rt_, hi);
            }
            Access::LoadUnit { n, left } => {
                let raw = self.b.ins().load(int_type(n), guest(), ptr, 0);
                let mem = if n == 8 { raw } else { self.b.ins().uextend(I64, raw) };
                let k = self.b.ins().band_imm_s(va, i64::from(n - 1));
                let reg = self.reg(rt_);
                let v = self.unit_load(n, left, k, mem, reg);
                self.set(rt_, v);
            }
            Access::Store { bytes } => {
                self.log(ptr, a, tagp);
                let v = self.reg(rt_);
                let v = if bytes == 8 { v } else { self.b.ins().ireduce(int_type(bytes), v) };
                self.b.ins().store(guest(), v, ptr, 0);
                self.stored(tagp, unc);
            }
            Access::Sq => {
                self.log(ptr, a, tagp);
                let lo = self.reg(rt_);
                let hi = self.high(rt_);
                self.b.ins().store(guest(), lo, ptr, 0);
                self.b.ins().store(guest(), hi, ptr, 8);
                self.stored(tagp, unc);
            }
            Access::StoreUnit { n, left } => {
                self.log(ptr, a, tagp);
                let raw = self.b.ins().load(int_type(n), guest(), ptr, 0);
                let old = if n == 8 { raw } else { self.b.ins().uextend(I64, raw) };
                let k = self.b.ins().band_imm_s(va, i64::from(n - 1));
                let reg = self.reg(rt_);
                let (bits, keep) = self.unit_store(n, left, k, reg);
                let kept_bits = self.b.ins().band(old, keep);
                let new_bits = self.b.ins().band_not(bits, keep);
                let v = self.b.ins().bor(kept_bits, new_bits);
                let v = if n == 8 { v } else { self.b.ins().ireduce(I32, v) };
                self.b.ins().store(guest(), v, ptr, 0);
                self.stored(tagp, unc);
            }
            Access::Pref => {}
        }
        self.issue_waiting(op, w, K(0), data);
        self.snaps.push(Snap { held: self.held, at: At::Line(pc.wrapping_add(4)), block: None });
        self.held.snap = Some(self.snaps.len() - 1);

        // The exit before the instruction, for every way it left.
        let after = self.held;
        let on = self.b.create_block();
        self.b.ins().jump(on, &[]);
        self.b.seal_block(slow);
        self.b.switch_to_block(slow);
        let ctx = self.ctx;
        let at = self.b.ins().ishl_imm_s(why, 3);
        let at = self.b.ins().iadd(ctx, at);
        let n = self.b.ins().load(I64, trusted(), at, X_LEFT);
        let n = self.b.ins().iadd_imm_s(n, 1);
        self.b.ins().store(trusted(), n, at, X_LEFT);
        self.held = kept;
        self.cold = true;
        self.exit(before);
        self.cold = false;
        self.held = after;
        self.b.seal_block(on);
        self.b.switch_to_block(on);
    }

    /// The undo of a store at `ptr` (guest address `a`): the 16 bytes of its quadword and the 8 of its line's tag.
    fn log(&mut self, ptr: Value, a: Value, tagp: Value) {
        let k = self.held.logged as i32;
        assert!((k as usize) + 2 <= 2 * shape::MAX, "two undo entries a store");
        let ctx = self.ctx;
        let low = self.b.ins().band_imm_s(a, 15);
        let quad = self.b.ins().isub(ptr, low);
        let q0 = self.b.ins().load(I64, guest(), quad, 0);
        let q1 = self.b.ins().load(I64, guest(), quad, 8);
        let t0 = self.b.ins().load(I64, guest(), tagp, 0);
        let short = self.b.ins().bor_imm_s(tagp, SHORT as i64);
        let zero = self.k(0);
        for (n, (addr, lo, hi)) in [(quad, q0, q1), (short, t0, zero)].into_iter().enumerate() {
            let at = X_LOG + 24 * (k + n as i32);
            self.b.ins().store(trusted(), addr, ctx, at);
            self.b.ins().store(trusted(), lo, ctx, at + 8);
            self.b.ins().store(trusted(), hi, ctx, at + 16);
        }
        self.held.logged += 2;
    }

    /// After a store: the line's D set unless it is locked (§6.3.7), the UCAB disabled (§6.5.1), and a store past the
    /// caches counted.
    fn stored(&mut self, tagp: Value, unc: Value) {
        let lock = self.b.ins().load(I8, guest(), tagp, TAG_LOCK);
        let dirty = self.b.ins().load(I8, guest(), tagp, TAG_DIRTY);
        let free = self.b.ins().icmp_imm_s(IntCC::Equal, lock, 0);
        let set = self.b.ins().bor(dirty, free);
        self.b.ins().store(guest(), set, tagp, TAG_DIRTY);
        let ctx = self.ctx;
        let n = self.b.ins().load(I64, trusted(), ctx, X_UNCACHED);
        let n = self.b.ins().iadd(n, unc);
        self.b.ins().store(trusted(), n, ctx, X_UNCACHED);
        self.held.sync = true;
    }

    /// LWL, LWR, LDL and LDR as `ops` merges them: the unit `mem` (zero-extended) into `reg`, `k` the address's offset
    /// in the unit.
    fn unit_load(&mut self, n: u32, left: bool, k: Value, mem: Value, reg: Value) -> Value {
        let byte8 = self.b.ins().ishl_imm_s(k, 3);
        match (n, left) {
            (4, true) => {
                let shift = self.rsub(24, byte8);
                let moved = self.b.ins().ishl(mem, shift);
                let one = self.k(1);
                let mask = self.b.ins().ishl(one, shift);
                let mask = self.b.ins().iadd_imm_s(mask, -1);
                let low = self.b.ins().band(reg, mask);
                let v = self.b.ins().bor(moved, low);
                self.sext32(v)
            }
            (4, false) => {
                let moved = self.b.ins().ushr(mem, byte8);
                let width = self.rsub(32, byte8);
                let one = self.k(1);
                let mask = self.b.ins().ishl(one, width);
                let mask = self.b.ins().iadd_imm_s(mask, -1);
                let high = self.b.ins().band_not(reg, mask);
                let low = self.b.ins().bor(moved, high);
                let low = self.b.ins().band_imm_s(low, 0xFFFF_FFFF);
                let whole = self.sext32(low);
                let upper = self.b.ins().band_imm_s(reg, !0xFFFF_FFFF);
                let part = self.b.ins().bor(upper, low);
                let first = self.b.ins().icmp_imm_s(IntCC::Equal, k, 0);
                self.b.ins().select(first, whole, part)
            }
            (8, true) => {
                let shift = self.rsub(56, byte8);
                let moved = self.b.ins().ishl(mem, shift);
                let one = self.k(1);
                let mask = self.b.ins().ishl(one, shift);
                let mask = self.b.ins().iadd_imm_s(mask, -1);
                let low = self.b.ins().band(reg, mask);
                self.b.ins().bor(moved, low)
            }
            _ => {
                let moved = self.b.ins().ushr(mem, byte8);
                let width = self.rsub(64, byte8);
                let one = self.k(1);
                let mask = self.b.ins().ishl(one, width);
                let mask = self.b.ins().iadd_imm_s(mask, -1);
                let all = self.k(-1);
                let first = self.b.ins().icmp_imm_s(IntCC::Equal, k, 0);
                let mask = self.b.ins().select(first, all, mask);
                let high = self.b.ins().band_not(reg, mask);
                self.b.ins().bor(moved, high)
            }
        }
    }

    /// SWL, SWR, SDL and SDR as `ops` merges them: the register's bytes and the unit's bits kept.
    fn unit_store(&mut self, n: u32, left: bool, k: Value, reg: Value) -> (Value, Value) {
        let byte8 = self.b.ins().ishl_imm_s(k, 3);
        let one = self.k(1);
        match (n, left) {
            (4, true) => {
                let shift = self.rsub(24, byte8);
                let r = self.b.ins().band_imm_s(reg, 0xFFFF_FFFF);
                let bits = self.b.ins().ushr(r, shift);
                let ones = self.k(0xFFFF_FFFF);
                let below = self.b.ins().ushr(ones, shift);
                let keep = self.b.ins().bxor_imm_s(below, 0xFFFF_FFFF);
                (bits, keep)
            }
            (4, false) => {
                let bits = self.b.ins().ishl(reg, byte8);
                let bits = self.b.ins().band_imm_s(bits, 0xFFFF_FFFF);
                let keep = self.b.ins().ishl(one, byte8);
                let keep = self.b.ins().iadd_imm_s(keep, -1);
                (bits, keep)
            }
            (8, true) => {
                let shift = self.rsub(56, byte8);
                let bits = self.b.ins().ushr(reg, shift);
                let all = self.k(-1);
                let below = self.b.ins().ushr(all, shift);
                let keep = self.b.ins().bnot(below);
                (bits, keep)
            }
            _ => {
                let bits = self.b.ins().ishl(reg, byte8);
                let keep = self.b.ins().ishl(one, byte8);
                let keep = self.b.ins().iadd_imm_s(keep, -1);
                (bits, keep)
            }
        }
    }

    /// The branch at `pc` with its slot: the condition, the link and the target, then the slot, then the exit.
    fn branch(&mut self, w: u32, pc: u32, slot_word: u32) {
        let name = shape::name(w);
        let op = ops::get(ops::index(w));
        let slot = pc.wrapping_add(4);
        let after = pc.wrapping_add(8);
        let relative = i64::from(slot.wrapping_add((simm(w) as u32) << 2));
        let a = self.reg(rs(w));
        let cmp0 = |e: &mut Self, cc: IntCC| -> Tv {
            let f = e.b.ins().icmp_imm_s(cc, a, 0);
            D(e.b.ins().uextend(I64, f))
        };
        let (cond, target, likely, link): (Tv, Tv, bool, Option<usize>) = match name {
            "beq" | "bne" | "beql" | "bnel" => {
                let b = self.reg(rt(w));
                let cc = if name.starts_with("beq") { IntCC::Equal } else { IntCC::NotEqual };
                let f = self.b.ins().icmp(cc, a, b);
                (D(self.b.ins().uextend(I64, f)), K(relative), name.ends_with('l'), None)
            }
            "blez" | "blezl" => (cmp0(self, IntCC::SignedLessThanOrEqual), K(relative), name.ends_with('l'), None),
            "bgtz" | "bgtzl" => (cmp0(self, IntCC::SignedGreaterThan), K(relative), name.ends_with('l'), None),
            "bltz" | "bltzl" => (cmp0(self, IntCC::SignedLessThan), K(relative), name.ends_with('l'), None),
            "bgez" | "bgezl" => (cmp0(self, IntCC::SignedGreaterThanOrEqual), K(relative), name.ends_with('l'), None),
            "bltzal" | "bltzall" => (cmp0(self, IntCC::SignedLessThan), K(relative), name == "bltzall", Some(31)),
            "bgezal" | "bgezall" => (cmp0(self, IntCC::SignedGreaterThanOrEqual), K(relative), name == "bgezall", Some(31)),
            "j" | "jal" => (K(1), K(i64::from(slot & 0xF000_0000 | (w & 0x03FF_FFFF) << 2)), false, (name == "jal").then_some(31)),
            "jr" | "jalr" => {
                let t = self.b.ins().band_imm_s(a, 0xFFFF_FFFF);
                (K(1), D(t), false, (name == "jalr").then_some(rd(w)))
            }
            other => unreachable!("{other} is not a branch of S36"),
        };
        if let Some(r) = link {
            let v = self.k(i64::from(after));
            self.set(r, v);
        }
        let leaves = self.flag(IntCC::NotEqual, target, K(i64::from(after)));
        let taken = self.and(cond, leaves);
        self.issue(op, w, taken);
        let next = if likely { target } else { self.select(cond, target, K(i64::from(after))) };
        if likely {
            let skipped = self.not(cond);
            if let K(1) = skipped {
                unreachable!("a likely branch whose condition is false whatever the registers");
            }
            if let D(c) = skipped {
                let out = self.b.create_block();
                let on = self.b.create_block();
                self.b.ins().brif(c, out, &[], on, &[]);
                self.b.seal_block(out);
                self.b.seal_block(on);
                self.b.switch_to_block(out);
                let kept = self.held;
                self.after(At::Line(after), true);
                self.exit(At::Line(after));
                self.held = kept;
                self.b.switch_to_block(on);
            }
        }
        self.after(At::Slot { pc: slot, next }, false);
        self.straight(slot_word, slot, Some(next));
        self.after(At::Jumped(next), true);
        self.exit(At::Jumped(next));
    }
}

/// One block as a function `(core, context) -> exit`.
pub fn lower(module: &JITModule, func: &mut ir::Function, functions: &mut FunctionBuilderContext, words: &[u32], start: u32, options: Options) -> Result<(), String> {
    let ptr = module.target_config().pointer_type();
    func.signature.params.extend([AbiParam::new(ptr), AbiParam::new(ptr)]);
    func.signature.returns.push(AbiParam::new(I32));
    let mut helper = module.make_signature();
    helper.params.extend([AbiParam::new(ptr), AbiParam::new(I32)]);
    helper.returns.push(AbiParam::new(I32));

    let mut b = FunctionBuilder::new(func, functions);
    let head = b.create_block();
    b.append_block_params_for_function_params(head);
    b.switch_to_block(head);
    b.seal_block(head);
    let (ee, ctx) = (b.block_params(head)[0], b.block_params(head)[1]);
    let helper = b.import_signature(helper);
    let ld = |b: &mut FunctionBuilder, ty, at| b.ins().load(ty, trusted(), ee, at);
    let ldu = |b: &mut FunctionBuilder, ty, at| {
        let v = b.ins().load(ty, trusted(), ee, at);
        b.ins().uextend(I64, v)
    };
    let model = Model {
        last: D(ld(&mut b, I64, t_at(layout::LAST))),
        open: D(ldu(&mut b, I8, t_at(layout::OPEN))),
        floor: D(ld(&mut b, I64, t_at(layout::FLOOR))),
        group_ends: D(ldu(&mut b, I8, t_at(layout::GROUP_ENDS))),
        paired: D(ldu(&mut b, I8, t_at(layout::PAIRED))),
        category: D(ldu(&mut b, I8, t_at(layout::LAST_CATEGORY))),
        pipe: D(ldu(&mut b, I8, t_at(layout::LAST_PIPE))),
        writes_lo: D(ld(&mut b, I64, t_at(layout::LAST_WRITES))),
        writes_hi: D(ld(&mut b, I64, t_at(layout::LAST_WRITES + 8))),
        mac0: None,
        mac1: None,
        mac_dirty: [false; 2],
        mem: None,
        mem_dirty: false,
        ready: [None; TRACKED],
        ready_dirty: 0,
    };
    let count0 = ldu(&mut b, I32, cop0_at(cop0::COUNT));
    let compare = ldu(&mut b, I32, cop0_at(cop0::COMPARE));
    let seen0 = ld(&mut b, I64, CYCLES_SEEN);
    let random = ldu(&mut b, I32, cop0_at(cop0::RANDOM));
    let wired = ldu(&mut b, I32, cop0_at(cop0::WIRED));
    let wired = b.ins().band_imm_s(wired, 63);
    let group = D(ldu(&mut b, I8, GROUP));
    let issued0 = ld(&mut b, I64, t_at(layout::INSTRUCTIONS));
    let memory = words.iter().any(|&w| shape::class(w) == Class::Memory).then(|| {
        let x = |b: &mut FunctionBuilder, at| b.ins().load(I64, trusted(), ctx, at);
        Memory { pages: x(&mut b, X_PAGES), gen1: x(&mut b, X_GEN1), horizon: x(&mut b, X_HORIZON), lines: ld(&mut b, I64, D_DATA) }
    });
    let mut e = Emitter {
        b: &mut b,
        ee,
        ctx,
        helper,
        options,
        held: Held {
            regs: [None; 32],
            dirty: 0,
            highs: [None; 32],
            high_dirty: 0,
            logged: 0,
            snap: None,
            hilo: [None; 4],
            hilo_dirty: 0,
            sa: None,
            sa_dirty: false,
            model,
            random,
            group,
            sync: false,
            steps: 0,
        },
        snaps: Vec::new(),
        memory,
        cold: false,
        count0,
        compare,
        seen0,
        wired,
        issued0,
    };
    let mut i = 0;
    loop {
        let pc = start.wrapping_add(4 * i as u32);
        if i == words.len() {
            e.exit(At::Line(pc));
            break;
        }
        if i > 0 && pc & 63 == 0 {
            // A new line of the instruction cache: the block leaves here when the cache does not hold it
            // (`Context::limit`).
            let ctx = e.ctx;
            let limit = e.b.ins().load(I32, trusted(), ctx, X_LIMIT);
            let short = e.b.ins().icmp_imm_s(IntCC::UnsignedLessThanOrEqual, limit, i as i64);
            let short = e.b.ins().uextend(I64, short);
            e.exit_if(D(short), At::Line(pc));
        }
        match shape::class(words[i]) {
            Class::Plain | Class::Memory => {
                e.straight(words[i], pc, None);
                e.after(At::Line(pc.wrapping_add(4)), i + 1 == words.len());
                i += 1;
            }
            Class::Branch if i + 1 < words.len() => {
                e.branch(words[i], pc, words[i + 1]);
                break;
            }
            _ => return Err(format!("the word at {i} is not one a block holds")),
        }
    }
    // The write-backs of the state after a load or store that the exits' rollbacks reached.
    for i in 0..e.snaps.len() {
        if let Some(block) = e.snaps[i].block {
            e.b.switch_to_block(block);
            e.held = e.snaps[i].held;
            let at = e.snaps[i].at;
            e.write_back(at);
            e.ret(ROLLBACK);
        }
    }
    b.seal_all_blocks();
    b.finalize(module.target_config());
    Ok(())
}
