//! One microprogram as Cranelift IR: a block for every pair it covers, entered through a table by the unit's PC. Each
//! pair is `Pipe::step` with its decoding done here: the plan of its T stage from the hazards it can meet, the stop at
//! the caller's cycle, the lower and upper operations (the integer unit's inline, the rest the interpreter's functions,
//! called), the write priority, the flag write for the S stage, and the flow (the M, E, D and T bits, a branch and its
//! delay slot). Sapphire_Native.md §45.3 says what is inline and why.

use std::mem::offset_of;

use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::types::{I8, I16, I32, I64};
use cranelift_codegen::ir::{self, AbiParam, Block, InstBuilder, JumpTableData, MemFlagsData, Value};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::JITModule;
use cranelift_module::Module;
use emusen_mips::vu::{self as table, I_BIT};

use super::{BOUND, EXT_XGKICK, EXT_XITOP, EXT_XTOP, Frame, KIND_DIV, KIND_EFU, KIND_PLAIN, OUTSIDE, Options, PROBING, RELEASE, STALL, STOP, TRACING, USE_LOW, USE_UP, NOW};
use crate::vu::ops::{self, Class};
use crate::vu::pipe::{self, FMAC, NO_JUMP, NONE, Pipe, State, ViWrite};
use crate::vu::{Link, Vu, fbrst, vpu_stat};

const P_STATE: i32 = offset_of!(Pipe, state) as i32;
const P_PC: i32 = offset_of!(Pipe, pc) as i32;
const P_NEXT_T: i32 = offset_of!(Pipe, next_t) as i32;
const P_LAST_T: i32 = offset_of!(Pipe, last_t) as i32;
const P_JUMP: i32 = offset_of!(Pipe, jump) as i32;
const P_ENDING: i32 = offset_of!(Pipe, ending) as i32;
const P_HALTING: i32 = offset_of!(Pipe, halting) as i32;
const P_VF_READY: i32 = offset_of!(Pipe, vf_ready) as i32;
const P_VI_READY: i32 = offset_of!(Pipe, vi_ready) as i32;
const P_VI_LAST_AT: i32 = offset_of!(Pipe, vi_last_at) as i32;
const P_VI_LAST_OLD: i32 = offset_of!(Pipe, vi_last_old) as i32;
const P_KICK_WAIT: i32 = offset_of!(Pipe, kick_wait) as i32;
const P_RELEASE_T: i32 = offset_of!(Pipe, release_t) as i32;
const P_STOPPED_BY: i32 = offset_of!(Pipe, stopped_by) as i32;
const V_VF: i32 = offset_of!(Vu, vf) as i32;
const V_VI: i32 = offset_of!(Vu, vi) as i32;
const V_I: i32 = offset_of!(Vu, i) as i32;
const V_TPC: i32 = offset_of!(Vu, tpc) as i32;
const V_DIV_FREE: i32 = offset_of!(Vu, div_free) as i32;
const V_STATUS: i32 = offset_of!(Vu, status) as i32;
const V_MAC: i32 = offset_of!(Vu, mac) as i32;
const V_CLIP: i32 = offset_of!(Vu, clip) as i32;
const V_EFU_FREE: i32 = offset_of!(Vu, efu_free) as i32;
const L_FBRST: i32 = offset_of!(Link, fbrst) as i32;
const L_IRQ: i32 = offset_of!(Link, irq) as i32;
const F_PIPE: i32 = offset_of!(Frame, pipe) as i32;
const F_VU: i32 = offset_of!(Frame, vu) as i32;
const F_LINK: i32 = offset_of!(Frame, link) as i32;
const F_DATA: i32 = offset_of!(Frame, data) as i32;
const F_LIMIT: i32 = offset_of!(Frame, limit) as i32;
const F_BUDGET: i32 = offset_of!(Frame, budget) as i32;
const F_RELEASE: i32 = offset_of!(Frame, release) as i32;
const F_FLAGS: i32 = offset_of!(Frame, flags) as i32;
const F_PLANNED: i32 = offset_of!(Frame, planned) as i32;

fn mem() -> MemFlagsData {
    MemFlagsData::new().with_notrap()
}

fn ft(w: u32) -> usize {
    (w >> 16 & 31) as usize
}
fn fs(w: u32) -> usize {
    (w >> 11 & 31) as usize
}
fn fd(w: u32) -> usize {
    (w >> 6 & 31) as usize
}

/// A vector register's 128 bits as two 64-bit halves: x and y, then z and w, each lane 32 bits, x lowest.
#[derive(Clone, Copy)]
struct Quad {
    lo: Value,
    hi: Value,
}

/// The lanes a field mask names (§4.1.3: bit 24 x), as the two halves' masks.
fn lane_mask(w: u32) -> (u64, u64) {
    let on = |lane: u32| if w >> (24 - lane) & 1 == 1 { 0xFFFF_FFFFu64 } else { 0 };
    (on(0) | on(1) << 32, on(2) | on(3) << 32)
}

/// The lowest lane a field mask names, which ILW and ILWR read (their pages: the lowest where it names more).
fn first_lane(w: u32) -> Option<u32> {
    (0..4).find(|&l| w >> (24 - l) & 1 == 1)
}

/// A branch's effect on the jump the pair leaves: none, a target known here, or one the code computes (`NO_JUMP` for a
/// branch not taken).
#[derive(Clone, Copy)]
enum Taken {
    No,
    Known(u16),
    Computed(Value),
}

/// The helpers' signatures.
struct Sigs {
    frame_u64: ir::SigRef,
    frame_t_u64: ir::SigRef,
    lower: ir::SigRef,
    upper: ir::SigRef,
    push: ir::SigRef,
    external: ir::SigRef,
    trace: ir::SigRef,
    admit: ir::SigRef,
    verify: ir::SigRef,
}

struct Emitter<'a, 'b> {
    b: &'a mut FunctionBuilder<'b>,
    options: Options,
    unit: usize,
    mask: u16,
    ptr: ir::Type,
    sigs: Sigs,
    f: Value,
    pipe: Value,
    vu: Value,
    link: Value,
    data: Value,
    limit: Value,
    release: Value,
    flags: Value,
    budget: Variable,
    land_at: Variable,
    blocks: Vec<Option<Block>>,
    dispatch: Block,
    exits: [Block; 5],
}

impl Emitter<'_, '_> {
    fn load(&mut self, ty: ir::Type, base: Value, offset: i32) -> Value {
        self.b.ins().load(ty, mem(), base, offset)
    }

    fn store(&mut self, v: Value, base: Value, offset: i32) {
        self.b.ins().store(mem(), v, base, offset);
    }

    /// A constant of `ty`, its bits above the type's width cleared.
    fn k(&mut self, ty: ir::Type, v: u64) -> Value {
        let v = if ty.bits() < 64 { v & ((1u64 << ty.bits()) - 1) } else { v };
        self.b.ins().iconst(ty, v as i64)
    }

    fn call(&mut self, sig: ir::SigRef, addr: usize, args: &[Value]) -> Option<Value> {
        let a = self.k(self.ptr, addr as u64);
        let inst = self.b.ins().call_indirect(sig, a, args);
        self.b.inst_results(inst).first().copied()
    }

    /// Leaves by exit `code` when `cond` holds; the code goes on in a new block otherwise.
    fn exit_if(&mut self, cond: Value, code: u32) {
        let next = self.b.create_block();
        let exit = self.exits[code as usize];
        self.b.ins().brif(cond, exit, &[], next, &[]);
        self.b.switch_to_block(next);
    }

    /// Runs `body` only when `cond` holds.
    fn when(&mut self, cond: Value, body: impl FnOnce(&mut Self)) {
        let then = self.b.create_block();
        let join = self.b.create_block();
        self.b.ins().brif(cond, then, &[], join, &[]);
        self.b.switch_to_block(then);
        body(self);
        self.b.ins().jump(join, &[]);
        self.b.switch_to_block(join);
    }

    fn not(&mut self, b: Value) -> Value {
        self.b.ins().bxor_imm_u(b, 1)
    }

    /// VI[r] as `Vu::vi` reads it: zero for a field naming 16 to 31.
    fn vi(&mut self, r: usize) -> Value {
        if r < 16 {
            let vu = self.vu;
            self.load(I16, vu, V_VI + 2 * r as i32)
        } else {
            self.k(I16, 0)
        }
    }

    /// `Vu::set_vi`: VI00 and the fields naming 16 to 31 write nothing.
    fn set_vi(&mut self, r: usize, v: Value) {
        if (1..16).contains(&r) {
            let vu = self.vu;
            self.store(v, vu, V_VI + 2 * r as i32);
        }
    }

    fn vf(&mut self, r: usize) -> Quad {
        let vu = self.vu;
        Quad { lo: self.load(I64, vu, V_VF + 16 * r as i32), hi: self.load(I64, vu, V_VF + 16 * r as i32 + 8) }
    }

    /// `Vu::set_vf`: VF00 keeps its constants.
    fn set_vf(&mut self, r: usize, v: Quad) {
        if r != 0 {
            let vu = self.vu;
            self.store(v.lo, vu, V_VF + 16 * r as i32);
            self.store(v.hi, vu, V_VF + 16 * r as i32 + 8);
        }
    }

    /// One lane of a register, 32 bits.
    fn vf_lane(&mut self, r: usize, lane: u32) -> Value {
        let vu = self.vu;
        self.load(I32, vu, V_VF + 16 * r as i32 + 4 * lane as i32)
    }

    fn merge(&mut self, m: u64, new: Value, old: Value) -> Value {
        match m {
            u64::MAX => new,
            0 => old,
            _ => {
                let a = self.b.ins().band_imm_u(new, m as i64);
                let b = self.b.ins().band_imm_u(old, !m as i64);
                self.b.ins().bor(a, b)
            }
        }
    }

    /// `new` in the lanes `w`'s field mask names, `old` in the rest.
    fn select_lanes(&mut self, w: u32, new: Quad, old: Quad) -> Quad {
        let (ml, mh) = lane_mask(w);
        Quad { lo: self.merge(ml, new.lo, old.lo), hi: self.merge(mh, new.hi, old.hi) }
    }

    /// A 32-bit value in every lane.
    fn splat(&mut self, v32: Value) -> Quad {
        let v = self.b.ins().uextend(I64, v32);
        let up = self.b.ins().ishl_imm_u(v, 32);
        let half = self.b.ins().bor(v, up);
        Quad { lo: half, hi: half }
    }

    /// The address of the qword at data index `a` of VU1's memory (`Vu::mem_q`: it mirrors past its end).
    fn data_at(&mut self, a: Value) -> Value {
        let a = self.b.ins().uextend(I64, a);
        let index = self.b.ins().band_imm_u(a, (crate::vu::VU1_MEM / 16 - 1) as i64);
        let offset = self.b.ins().ishl_imm_u(index, 4);
        self.b.ins().iadd(self.data, offset)
    }

    fn load_q(&mut self, at: Value) -> Quad {
        Quad { lo: self.load(I64, at, 0), hi: self.load(I64, at, 8) }
    }

    fn store_q(&mut self, at: Value, q: Quad) {
        self.store(q.lo, at, 0);
        self.store(q.hi, at, 8);
    }

    fn imm11(&mut self, w: u32) -> Value {
        self.k(I16, table::imm11(w) as u16 as u64)
    }

    /// The register's lanes `l` readable from `at`.
    fn ready(&mut self, r: usize, l: u8, at: Value) {
        for f in 0..4 {
            if l >> f & 1 == 1 {
                let pipe = self.pipe;
                self.store(at, pipe, P_VF_READY + 8 * (4 * r as i32 + f));
            }
        }
    }

    /// `Pipe::branch_vi`: VI[r] as it was before an integer-unit write in the pair just before (D-80).
    fn branch_vi(&mut self, r: usize, t: Value) -> Value {
        if r >= 16 {
            return self.k(I16, 0);
        }
        let pipe = self.pipe;
        let at = self.load(I64, pipe, P_VI_LAST_AT + 8 * r as i32);
        let old = self.load(I16, pipe, P_VI_LAST_OLD + 2 * r as i32);
        let cur = self.vi(r);
        let set = self.b.ins().icmp_imm_s(IntCC::NotEqual, at, NONE as i64);
        let next = self.b.ins().iadd_imm_s(at, 1);
        let just = self.b.ins().icmp(IntCC::Equal, next, t);
        let both = self.b.ins().band(set, just);
        self.b.ins().select(both, old, cur)
    }

    /// `Pipe::write_alu`: the value before remembered for a branch, then the register written.
    fn write_alu(&mut self, r: usize, v: Value, t: Value) {
        if r < 16 {
            let old = self.vi(r);
            let pipe = self.pipe;
            self.store(t, pipe, P_VI_LAST_AT + 8 * r as i32);
            self.store(old, pipe, P_VI_LAST_OLD + 2 * r as i32);
        }
        self.set_vi(r, v);
    }

    /// The earliest T stage of the pair, as `Pipe::plan` finds it.
    fn plan(&mut self, upper: u32, lower: u32) -> Value {
        let pipe = self.pipe;
        let next = self.load(I64, pipe, P_NEXT_T);
        let kick = self.load(I64, pipe, P_KICK_WAIT);
        let mut t = self.b.ins().umax(next, kick);
        let mut reads = Vec::new();
        if let Some((o, _)) = pipe::upper_op(upper) {
            reads.push(pipe::upper_access(o, upper));
        }
        let low = if upper & I_BIT == 0 { ops::of_lower(lower) } else { None };
        if let Some((o, _)) = low {
            reads.push(pipe::lower_access(o, lower));
        }
        let mut seen = Vec::new();
        for a in &reads {
            for &(r, l) in &a.vf {
                if r != 0 {
                    for f in 0..4 {
                        if l >> f & 1 == 1 && !seen.contains(&(r, f)) {
                            seen.push((r, f));
                            let ready = self.load(I64, pipe, P_VF_READY + 8 * (4 * r as i32 + f));
                            t = self.b.ins().umax(t, ready);
                        }
                    }
                }
            }
            for &r in &a.vi {
                if (1..16).contains(&r) && !seen.contains(&(r + 100, 0)) {
                    seen.push((r + 100, 0));
                    let ready = self.load(I64, pipe, P_VI_READY + 8 * r as i32);
                    t = self.b.ins().umax(t, ready);
                }
            }
        }
        if let Some((_, e)) = low {
            let vu = self.vu;
            let f = self.f;
            let wait = match e.class {
                Class::Div(_) => Some(self.load(I64, vu, V_DIV_FREE)),
                Class::WaitQ => self.call(self.sigs.frame_u64, super::h_q_at as *const () as usize, &[f]),
                Class::Efu(_) if self.unit == 1 => Some(self.load(I64, vu, V_EFU_FREE)),
                Class::WaitP => self.call(self.sigs.frame_u64, super::h_p_at as *const () as usize, &[f]),
                _ => None,
            };
            if let Some(w) = wait {
                t = self.b.ins().umax(t, w);
            }
        }
        t
    }

    /// One pair, from the checks before it to its flow.
    fn pair(&mut self, pc: u16, upper: u32, lower: u32) {
        let (pipe, f) = (self.pipe, self.f);
        // The interpreter's loop: under `release`, stop after a pair with M; at the bound, stop.
        let rt = self.load(I64, pipe, P_RELEASE_T);
        let set = self.b.ins().icmp_imm_s(IntCC::NotEqual, rt, NONE as i64);
        let asked = self.b.ins().icmp_imm_s(IntCC::NotEqual, self.release, 0);
        let released = self.b.ins().band(set, asked);
        self.exit_if(released, RELEASE);
        let budget = self.b.use_var(self.budget);
        let spent = self.b.ins().icmp_imm_s(IntCC::Equal, budget, 0);
        self.exit_if(spent, BOUND);

        let t = self.plan(upper, lower);
        self.store(t, f, F_PLANNED);
        let late = self.b.ins().icmp(IntCC::UnsignedGreaterThan, t, self.limit);
        self.exit_if(late, STALL);
        let probing = self.b.ins().band_imm_u(self.flags, PROBING as i64);
        self.when(probing, |e| {
            let pc32 = e.k(I32, pc as u64);
            let admitted = e.call(e.sigs.admit, super::h_admit as *const () as usize, &[f, pc32]).unwrap();
            let held = e.b.ins().icmp_imm_s(IntCC::Equal, admitted, 0);
            let zero = e.k(I64, 0);
            e.store(zero, f, F_PLANNED);
            e.exit_if(held, STALL);
        });
        let left = self.b.ins().iadd_imm_s(budget, -1);
        self.b.def_var(self.budget, left);

        // `execute`: the trace, then what lands by T.
        let tracing = self.b.ins().band_imm_u(self.flags, TRACING as i64);
        self.when(tracing, |e| {
            let pc32 = e.k(I32, pc as u64);
            e.call(e.sigs.trace, super::h_trace as *const () as usize, &[f, pc32, t]);
        });
        let land_at = self.b.use_var(self.land_at);
        let due = self.b.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, t, land_at);
        self.when(due, |e| {
            let next = e.call(e.sigs.frame_t_u64, super::h_land as *const () as usize, &[f, t]).unwrap();
            e.b.def_var(e.land_at, next);
        });

        let up = pipe::upper_op(upper);
        let low = if upper & I_BIT == 0 { ops::of_lower(lower) } else { None };
        let (taken, held, low_flags) = self.lower(pc, lower, low, t);

        // The upper operation, the I register, then the lower's write unless the upper wrote that register (§3.4.3).
        let mut which = 0;
        let mut up_write = None;
        if low_flags {
            which |= USE_LOW;
        }
        let mut pushed = false;
        if let Some((o, e)) = up {
            up_write = pipe::upper_access(o, upper).vf_write;
            if e.flags || o.name == "clipw" {
                which |= USE_UP;
            }
            if !self.inline_upper(o.name, upper) {
                let (run, word, wv) = (self.k_ptr(e.run as usize), self.k_word(upper), self.k(I32, (which | if self.options.c4 { NOW } else { 0 }) as u64));
                let next = self.call(self.sigs.upper, super::h_upper as *const () as usize, &[f, run, word, t, wv]).unwrap();
                self.b.def_var(self.land_at, next);
                pushed = true;
            }
        }
        if upper & I_BIT != 0 {
            let w = self.k(I32, lower as u64);
            let vu = self.vu;
            self.store(w, vu, V_I);
        }
        let t4 = self.b.ins().iadd_imm_s(t, FMAC as i64);
        if let Some((r, l)) = up_write {
            self.ready(r, l, t4);
        }
        if let Some((r, value, l)) = held
            && up_write.is_none_or(|(u, _)| u != r)
        {
            self.set_vf(r, value);
            self.ready(r, l, t4);
        }
        if which != 0 && !pushed {
            let w = self.k(I32, (which | if self.options.c4 { NOW } else { 0 }) as u64);
            let next = self.call(self.sigs.push, super::h_push as *const () as usize, &[f, t, w]).unwrap();
            self.b.def_var(self.land_at, next);
        }
        self.flow(pc, upper, t, taken);
    }

    fn k_ptr(&mut self, v: usize) -> Value {
        self.k(self.ptr, v as u64)
    }

    fn k_word(&mut self, w: u32) -> Value {
        self.k(I32, w as u64)
    }

    /// The lower instruction as `execute` runs it: the branch it takes, its register write held back (the register,
    /// the value, its lanes), and whether it leaves FCSET's or FSSET's write for the pair's flags.
    fn lower(&mut self, pc: u16, w: u32, low: Option<(&'static table::Operation, &'static ops::Entry)>, t: Value) -> (Taken, Option<(usize, Quad, u8)>, bool) {
        let Some((o, e)) = low else { return (Taken::No, None, false) };
        let f = self.f;
        match e.class {
            Class::Branch => return (self.branch(pc, o.name, w, t), None, false),
            Class::External => {
                let kind = match o.name {
                    "xgkick" if self.unit == 1 => Some(EXT_XGKICK),
                    "xtop" if self.unit == 1 => Some(EXT_XTOP),
                    "xitop" => Some(EXT_XITOP),
                    _ => None,
                };
                if let Some(kind) = kind {
                    let (wv, kv) = (self.k_word(w), self.k(I32, kind as u64));
                    self.call(self.sigs.external, super::h_external as *const () as usize, &[f, wv, t, kv]);
                }
                return (Taken::No, None, false);
            }
            Class::WaitQ | Class::WaitP => return (Taken::No, None, false),
            Class::Efu(_) if self.unit == 0 => return (Taken::No, None, false),
            _ => {}
        }
        let access = pipe::lower_access(o, w);
        let written = pipe::vi_write(o, e, w);
        let before = written.map(|x| {
            let r = match x {
                ViWrite::Load(r) | ViWrite::Check(r) | ViWrite::Alu(r) => r,
            };
            (x, self.vi(r))
        });
        let dest = access.vf_write.map(|(r, l)| (r, l, self.vf(r)));
        let inline = self.inline_lower(o.name, w, dest.map(|d| d.2));
        let mut held = None;
        match (inline, dest) {
            (Some(Some(value)), Some((r, l, _))) => held = Some((r, value, l)),
            (Some(None), None) => {}
            (Some(_), _) => panic!("{}: the inline form and the access disagree about the register written", o.name),
            (None, _) => {
                let kind = match e.class {
                    Class::Div(latency) => KIND_DIV | latency as u32,
                    Class::Efu(latency) => KIND_EFU | latency as u32,
                    _ => KIND_PLAIN,
                };
                let (run, wv, kv) = (self.k_ptr(e.run as usize), self.k_word(w), self.k(I32, kind as u64));
                let next = self.call(self.sigs.lower, super::h_lower as *const () as usize, &[f, run, wv, t, kv]).unwrap();
                self.b.def_var(self.land_at, next);
                if let Some((r, l, old)) = dest {
                    let value = self.vf(r);
                    self.set_vf(r, old);
                    held = Some((r, value, l));
                }
            }
        }
        let pipe = self.pipe;
        match before {
            Some((ViWrite::Load(r), _)) if (1..16).contains(&r) => {
                let ready = self.b.ins().iadd_imm_s(t, FMAC as i64);
                self.store(ready, pipe, P_VI_READY + 8 * r as i32);
            }
            Some((ViWrite::Alu(r), old)) if r < 16 => {
                self.store(t, pipe, P_VI_LAST_AT + 8 * r as i32);
                self.store(old, pipe, P_VI_LAST_OLD + 2 * r as i32);
            }
            Some((ViWrite::Check(r), _)) if r < 16 => {
                let (none, zero) = (self.k(I64, NONE), self.k(I16, 0));
                self.store(none, pipe, P_VI_LAST_AT + 8 * r as i32);
                self.store(zero, pipe, P_VI_LAST_OLD + 2 * r as i32);
            }
            _ => {}
        }
        (Taken::No, held, e.class == Class::FlagSet)
    }

    /// The lower operations compiled here rather than called: none for one left to its function; otherwise the new
    /// value of the register it writes, held back as `execute` holds it (none when it writes no vector register). The
    /// memory operations are VU1's alone, whose data memory has no window and no mirror below 16 KiB.
    fn inline_lower(&mut self, name: &str, w: u32, old: Option<Quad>) -> Option<Option<Quad>> {
        let vu1 = self.unit == 1;
        let none = Some(None);
        match name {
            "iadd" | "isub" | "iand" | "ior" => {
                let (a, b) = (self.vi(fs(w)), self.vi(ft(w)));
                let v = match name {
                    "iadd" => self.b.ins().iadd(a, b),
                    "isub" => self.b.ins().isub(a, b),
                    "iand" => self.b.ins().band(a, b),
                    _ => self.b.ins().bor(a, b),
                };
                self.set_vi(fd(w), v);
                none
            }
            "iaddi" => {
                let imm = (((w >> 6 & 31) << 27) as i32 >> 27) as u16;
                let a = self.vi(fs(w));
                let k = self.k(I16, imm as u64);
                let v = self.b.ins().iadd(a, k);
                self.set_vi(ft(w), v);
                none
            }
            "iaddiu" | "isubiu" => {
                let a = self.vi(fs(w));
                let k = self.k(I16, table::imm15(w) as u64);
                let v = if name == "iaddiu" { self.b.ins().iadd(a, k) } else { self.b.ins().isub(a, k) };
                self.set_vi(ft(w), v);
                none
            }
            "mtir" => {
                let v = self.vf_lane(fs(w), w >> 21 & 3);
                let v = self.b.ins().ireduce(I16, v);
                self.set_vi(ft(w), v);
                none
            }
            "move" => {
                let src = self.vf(fs(w));
                Some(Some(self.select_lanes(w, src, old?)))
            }
            "mr32" => {
                let v = self.vf(fs(w));
                let (a, b) = (self.b.ins().ushr_imm_u(v.lo, 32), self.b.ins().ishl_imm_u(v.hi, 32));
                let lo = self.b.ins().bor(a, b);
                let (a, b) = (self.b.ins().ushr_imm_u(v.hi, 32), self.b.ins().ishl_imm_u(v.lo, 32));
                let hi = self.b.ins().bor(a, b);
                Some(Some(self.select_lanes(w, Quad { lo, hi }, old?)))
            }
            "mfir" => {
                let v = self.vi(fs(w));
                let v = self.b.ins().sextend(I32, v);
                let q = self.splat(v);
                Some(Some(self.select_lanes(w, q, old?)))
            }
            "lq" | "lqi" | "lqd" if vu1 => {
                let base = self.vi(fs(w));
                let a = match name {
                    "lq" => {
                        let k = self.imm11(w);
                        self.b.ins().iadd(base, k)
                    }
                    "lqi" => base,
                    _ => self.b.ins().iadd_imm_s(base, -1),
                };
                match name {
                    "lqi" => {
                        let next = self.b.ins().iadd_imm_s(a, 1);
                        self.set_vi(fs(w), next);
                    }
                    "lqd" => self.set_vi(fs(w), a),
                    _ => {}
                }
                let at = self.data_at(a);
                let m = self.load_q(at);
                Some(Some(self.select_lanes(w, m, old?)))
            }
            "sq" | "sqi" | "sqd" if vu1 => {
                let base = self.vi(ft(w));
                let a = match name {
                    "sq" => {
                        let k = self.imm11(w);
                        self.b.ins().iadd(base, k)
                    }
                    "sqi" => base,
                    _ => self.b.ins().iadd_imm_s(base, -1),
                };
                if name == "sqd" {
                    self.set_vi(ft(w), a);
                }
                let at = self.data_at(a);
                let m = self.load_q(at);
                let v = self.vf(fs(w));
                let q = self.select_lanes(w, v, m);
                self.store_q(at, q);
                if name == "sqi" {
                    let next = self.b.ins().iadd_imm_s(a, 1);
                    self.set_vi(ft(w), next);
                }
                none
            }
            "ilw" | "ilwr" if vu1 => {
                let base = self.vi(fs(w));
                let a = if name == "ilw" {
                    let k = self.imm11(w);
                    self.b.ins().iadd(base, k)
                } else {
                    base
                };
                if let Some(lane) = first_lane(w) {
                    let at = self.data_at(a);
                    let v = self.load(I32, at, 4 * lane as i32);
                    let v = self.b.ins().ireduce(I16, v);
                    self.set_vi(ft(w), v);
                }
                none
            }
            "isw" | "iswr" if vu1 => {
                let base = self.vi(fs(w));
                let a = if name == "isw" {
                    let k = self.imm11(w);
                    self.b.ins().iadd(base, k)
                } else {
                    base
                };
                let at = self.data_at(a);
                let m = self.load_q(at);
                let v = self.vi(ft(w));
                let v = self.b.ins().uextend(I32, v);
                let q = self.splat(v);
                let q = self.select_lanes(w, q, m);
                self.store_q(at, q);
                none
            }
            "fcand" | "fceq" | "fcor" | "fcget" => {
                let vu = self.vu;
                let clip = self.load(I32, vu, V_CLIP);
                let imm = (w & 0xFF_FFFF) as i64;
                let v = match name {
                    "fcand" => {
                        let x = self.b.ins().band_imm_u(clip, imm);
                        let c = self.b.ins().icmp_imm_s(IntCC::NotEqual, x, 0);
                        self.b.ins().uextend(I16, c)
                    }
                    "fceq" => {
                        let c = self.b.ins().icmp_imm_u(IntCC::Equal, clip, imm);
                        self.b.ins().uextend(I16, c)
                    }
                    "fcor" => {
                        let x = self.b.ins().bor_imm_u(clip, imm);
                        let x = self.b.ins().band_imm_u(x, 0xFF_FFFF);
                        let c = self.b.ins().icmp_imm_u(IntCC::Equal, x, 0xFF_FFFF);
                        self.b.ins().uextend(I16, c)
                    }
                    _ => {
                        let x = self.b.ins().band_imm_u(clip, 0xFFF);
                        self.b.ins().ireduce(I16, x)
                    }
                };
                self.set_vi(if name == "fcget" { ft(w) } else { 1 }, v);
                none
            }
            "fmand" | "fmeq" | "fmor" => {
                let vu = self.vu;
                let mac = self.load(I32, vu, V_MAC);
                let mac = self.b.ins().ireduce(I16, mac);
                let s = self.vi(fs(w));
                let v = match name {
                    "fmand" => self.b.ins().band(mac, s),
                    "fmor" => self.b.ins().bor(mac, s),
                    _ => {
                        let c = self.b.ins().icmp(IntCC::Equal, mac, s);
                        self.b.ins().uextend(I16, c)
                    }
                };
                self.set_vi(ft(w), v);
                none
            }
            "fsand" | "fseq" | "fsor" => {
                let vu = self.vu;
                let status = self.load(I32, vu, V_STATUS);
                let imm = table::imm12(w) as i64;
                let v = match name {
                    "fsand" => {
                        let x = self.b.ins().band_imm_u(status, imm);
                        self.b.ins().ireduce(I16, x)
                    }
                    "fsor" => {
                        let x = self.b.ins().bor_imm_u(status, imm);
                        self.b.ins().ireduce(I16, x)
                    }
                    _ => {
                        let c = self.b.ins().icmp_imm_u(IntCC::Equal, status, imm);
                        self.b.ins().uextend(I16, c)
                    }
                };
                self.set_vi(ft(w), v);
                none
            }
            _ => None,
        }
    }

    /// The upper operations compiled here rather than called, all of them flagless: NOP, ABS, and MAX and MINI in their
    /// forms (`float::max` and `float::min` on the raw sign and magnitude, D-31); false for one left to its function.
    fn inline_upper(&mut self, name: &str, w: u32) -> bool {
        match name {
            "nop" => true,
            "abs" => {
                let (src, old) = (self.vf(fs(w)), self.vf(ft(w)));
                let magnitude = 0x7FFF_FFFF_7FFF_FFFFu64 as i64;
                let lo = self.b.ins().band_imm_u(src.lo, magnitude);
                let hi = self.b.ins().band_imm_u(src.hi, magnitude);
                let q = self.select_lanes(w, Quad { lo, hi }, old);
                self.set_vf(ft(w), q);
                true
            }
            _ if name.starts_with("max") || name.starts_with("mini") => {
                let larger = name.starts_with("max");
                let form = name.trim_start_matches("max").trim_start_matches("mini");
                let lanes: Vec<u32> = (0..4).filter(|&l| w >> (24 - l) & 1 == 1).collect();
                let mut results = Vec::new();
                for &lane in &lanes {
                    let a = self.vf_lane(fs(w), lane);
                    let b = match form {
                        "" => self.vf_lane(ft(w), lane),
                        "i" => {
                            let vu = self.vu;
                            self.load(I32, vu, V_I)
                        }
                        _ => self.vf_lane(ft(w), w & 3),
                    };
                    let key = |e: &mut Self, x: Value| {
                        let s = e.b.ins().sshr_imm_u(x, 31);
                        let m = e.b.ins().band_imm_u(s, 0x7FFF_FFFF);
                        e.b.ins().bxor(x, m)
                    };
                    let (ka, kb) = (key(self, a), key(self, b));
                    let cc = if larger { IntCC::SignedGreaterThanOrEqual } else { IntCC::SignedLessThanOrEqual };
                    let c = self.b.ins().icmp(cc, ka, kb);
                    results.push((lane, self.b.ins().select(c, a, b)));
                }
                let vu = self.vu;
                if fd(w) != 0 {
                    for (lane, v) in results {
                        self.store(v, vu, V_VF + 16 * fd(w) as i32 + 4 * lane as i32);
                    }
                }
                true
            }
            _ => false,
        }
    }

    /// `Pipe::branch_of`: the target when taken; BAL and JALR link the pair after the delay slot, JALR having read its
    /// target first.
    fn branch(&mut self, pc: u16, name: &str, w: u32, t: Value) -> Taken {
        let mask = self.mask;
        let rel = pc.wrapping_add(1).wrapping_add(table::imm11(w) as u16) & mask;
        let link = pc.wrapping_add(2) & mask;
        let target = |e: &mut Self, v: Value| {
            let m = e.b.ins().band_imm_u(v, mask as i64);
            e.b.ins().uextend(I32, m)
        };
        match name {
            "b" => Taken::Known(rel),
            "bal" => {
                let l = self.k(I16, link as u64);
                self.write_alu(ft(w), l, t);
                Taken::Known(rel)
            }
            "jr" => {
                let v = self.vi(fs(w));
                Taken::Computed(target(self, v))
            }
            "jalr" => {
                let v = self.vi(fs(w));
                let l = self.k(I16, link as u64);
                self.write_alu(ft(w), l, t);
                Taken::Computed(target(self, v))
            }
            _ => {
                let (a, b) = (self.branch_vi(ft(w), t), self.branch_vi(fs(w), t));
                let cond = match name {
                    "ibeq" => self.b.ins().icmp(IntCC::Equal, a, b),
                    "ibne" => self.b.ins().icmp(IntCC::NotEqual, a, b),
                    "ibltz" => self.b.ins().icmp_imm_s(IntCC::SignedLessThan, b, 0),
                    "ibgtz" => self.b.ins().icmp_imm_s(IntCC::SignedGreaterThan, b, 0),
                    "iblez" => self.b.ins().icmp_imm_s(IntCC::SignedLessThanOrEqual, b, 0),
                    _ => self.b.ins().icmp_imm_s(IntCC::SignedGreaterThanOrEqual, b, 0),
                };
                let (yes, no) = (self.k(I32, rel as u64), self.k(I32, NO_JUMP as u64));
                Taken::Computed(self.b.ins().select(cond, yes, no))
            }
        }
    }

    /// `execute`'s flow: the M bit; the pair after a taken branch's delay slot; the D and T bits; the E bit and its delay
    /// slot; then the next pair, or the exit when the unit stopped.
    fn flow(&mut self, pc: u16, upper: u32, t: Value, taken: Taken) {
        let (pipe, vu, link) = (self.pipe, self.vu, self.link);
        if upper >> 29 & 1 == 1 {
            let rt = self.load(I64, pipe, P_RELEASE_T);
            let none = self.b.ins().icmp_imm_s(IntCC::Equal, rt, NONE as i64);
            let v = self.b.ins().select(none, t, rt);
            self.store(v, pipe, P_RELEASE_T);
        }
        let succ = (pc + 1) & self.mask;
        let j = self.load(I32, pipe, P_JUMP);
        let jumping = self.b.ins().icmp_imm_s(IntCC::NotEqual, j, NO_JUMP as i64);
        let j16 = self.b.ins().ireduce(I16, j);
        let s16 = self.k(I16, succ as u64);
        let next_pc = self.b.ins().select(jumping, j16, s16);
        self.store(next_pc, pipe, P_PC);
        let taken_value = match taken {
            Taken::No => self.k(I32, NO_JUMP as u64),
            Taken::Known(target) => self.k(I32, target as u64),
            Taken::Computed(v) => v,
        };
        self.store(taken_value, pipe, P_JUMP);
        let t1 = self.b.ins().iadd_imm_s(t, 1);
        self.store(t1, pipe, P_NEXT_T);
        self.store(t, pipe, P_LAST_T);

        let (d_bit, t_bit, e_bit) = (upper >> 28 & 1 == 1, upper >> 27 & 1 == 1, upper >> 30 & 1 == 1);
        let halt = if d_bit || t_bit {
            let fb = self.load(I32, link, L_FBRST);
            let enables = self.b.ins().ushr_imm_u(fb, (fbrst::VU1 * self.unit as u32) as i64);
            let zero = self.k(I32, 0);
            let mut h = zero;
            if t_bit {
                let on = self.b.ins().band_imm_u(enables, fbrst::TE0 as i64);
                let v = self.k(I32, vpu_stat::VTS as u64);
                h = self.b.ins().select(on, v, h);
            }
            if d_bit {
                let on = self.b.ins().band_imm_u(enables, fbrst::DE0 as i64);
                let v = self.k(I32, vpu_stat::VDS as u64);
                h = self.b.ins().select(on, v, h);
            }
            Some(h)
        } else {
            None
        };
        let h0 = self.load(I32, pipe, P_HALTING);
        let e0 = self.load(I8, pipe, P_ENDING);
        let has_h = self.b.ins().icmp_imm_s(IntCC::NotEqual, h0, 0);
        let ending0 = self.b.ins().icmp_imm_s(IntCC::NotEqual, e0, 0);
        let (stop, new_halting, neither) = match halt {
            None => {
                let zero = self.k(I32, 0);
                (h0, zero, self.not(has_h))
            }
            Some(halt) => {
                let has_halt = self.b.ins().icmp_imm_s(IntCC::NotEqual, halt, 0);
                let taken_some = match taken {
                    Taken::No => self.k(I8, 0),
                    Taken::Known(_) => self.k(I8, 1),
                    Taken::Computed(v) => self.b.ins().icmp_imm_s(IntCC::NotEqual, v, NO_JUMP as i64),
                };
                let zero = self.k(I32, 0);
                let not_taken = self.not(taken_some);
                let stop_now = self.b.ins().band(has_halt, not_taken);
                let inner = self.b.ins().select(stop_now, halt, zero);
                let stop = self.b.ins().select(has_h, h0, inner);
                let later = self.b.ins().band(has_halt, taken_some);
                let inner = self.b.ins().select(later, halt, zero);
                let new_halting = self.b.ins().select(has_h, zero, inner);
                let nh = self.not(has_h);
                let nhalt = self.not(has_halt);
                (stop, new_halting, self.b.ins().band(nh, nhalt))
            }
        };
        let end = self.b.ins().band(neither, ending0);
        let fresh = if e_bit { self.not(ending0) } else { self.k(I8, 0) };
        let new_ending = self.b.ins().select(neither, fresh, ending0);
        self.store(new_halting, pipe, P_HALTING);
        self.store(new_ending, pipe, P_ENDING);
        let stopping = self.b.ins().icmp_imm_s(IntCC::NotEqual, stop, 0);
        let not_running = self.b.ins().bor(stopping, end);
        let stopped = self.b.create_block();
        let running = self.b.create_block();
        self.b.ins().brif(not_running, stopped, &[], running, &[]);

        self.b.switch_to_block(stopped);
        let (stop_state, ready_state) = (self.k(I8, State::Stop as u64), self.k(I8, State::Ready as u64));
        let state = self.b.ins().select(stopping, stop_state, ready_state);
        self.store(state, pipe, P_STATE);
        let by = self.load(I32, pipe, P_STOPPED_BY);
        let by = self.b.ins().select(stopping, stop, by);
        self.store(by, pipe, P_STOPPED_BY);
        let irq = self.load(I32, link, L_IRQ);
        let (bit, zero) = (self.k(I32, 1 << self.unit), self.k(I32, 0));
        let raised = self.b.ins().select(stopping, bit, zero);
        let irq = self.b.ins().bor(irq, raised);
        self.store(irq, link, L_IRQ);
        let pending = self.b.ins().icmp_imm_s(IntCC::NotEqual, taken_value, NO_JUMP as i64);
        let target = self.b.ins().ireduce(I16, taken_value);
        let tpc = self.b.ins().select(pending, target, next_pc);
        self.store(tpc, pipe, P_PC);
        let no = self.k(I32, NO_JUMP as u64);
        self.store(no, pipe, P_JUMP);
        self.store(tpc, vu, V_TPC);
        self.verify();
        let exit = self.exits[STOP as usize];
        self.b.ins().jump(exit, &[]);

        self.b.switch_to_block(running);
        self.verify();
        let follow = self.blocks[succ as usize].unwrap_or(self.exits[OUTSIDE as usize]);
        let dispatch = self.dispatch;
        self.b.ins().brif(jumping, dispatch, &[], follow, &[]);
    }

    fn verify(&mut self) {
        if self.options.verify {
            let f = self.f;
            self.call(self.sigs.verify, super::h_verify as *const () as usize, &[f]);
        }
    }
}

pub fn lower(module: &JITModule, func: &mut ir::Function, functions: &mut FunctionBuilderContext, words: &[u32], _start: u32, options: Options) -> Result<(), String> {
    if words.is_empty() || !words.len().is_multiple_of(3) {
        return Err("a microprogram is pairs of an address and two words".into());
    }
    let unit = options.unit as usize;
    let pairs = if unit == 0 { crate::vu::VU0_MEM } else { crate::vu::VU1_MEM } / 8;
    let mask = (pairs - 1) as u16;
    let ptr = module.target_config().pointer_type();
    func.signature.params.push(AbiParam::new(ptr));
    func.signature.returns.push(AbiParam::new(I32));
    let sig = |params: &[ir::Type], ret: Option<ir::Type>| {
        let mut s = module.make_signature();
        s.params.extend(params.iter().map(|&t| AbiParam::new(t)));
        s.returns.extend(ret.map(AbiParam::new));
        s
    };

    let mut b = FunctionBuilder::new(func, functions);
    let sigs = Sigs {
        frame_u64: b.import_signature(sig(&[ptr], Some(I64))),
        frame_t_u64: b.import_signature(sig(&[ptr, I64], Some(I64))),
        lower: b.import_signature(sig(&[ptr, ptr, I32, I64, I32], Some(I64))),
        upper: b.import_signature(sig(&[ptr, ptr, I32, I64, I32], Some(I64))),
        push: b.import_signature(sig(&[ptr, I64, I32], Some(I64))),
        external: b.import_signature(sig(&[ptr, I32, I64, I32], None)),
        trace: b.import_signature(sig(&[ptr, I32, I64], None)),
        admit: b.import_signature(sig(&[ptr, I32], Some(I32))),
        verify: b.import_signature(sig(&[ptr], None)),
    };
    let head = b.create_block();
    b.append_block_params_for_function_params(head);
    b.switch_to_block(head);
    let f = b.block_params(head)[0];
    let pipe = b.ins().load(ptr, mem(), f, F_PIPE);
    let vu = b.ins().load(ptr, mem(), f, F_VU);
    let link = b.ins().load(ptr, mem(), f, F_LINK);
    let data = b.ins().load(ptr, mem(), f, F_DATA);
    let limit = b.ins().load(I64, mem(), f, F_LIMIT);
    let release = b.ins().load(I32, mem(), f, F_RELEASE);
    let flags = b.ins().load(I32, mem(), f, F_FLAGS);
    let budget = b.declare_var(I64);
    let land_at = b.declare_var(I64);
    let initial = b.ins().load(I64, mem(), f, F_BUDGET);
    b.def_var(budget, initial);
    let pending = {
        let a = b.ins().iconst(ptr, super::h_pending as *const () as usize as i64);
        let inst = b.ins().call_indirect(sigs.frame_u64, a, &[f]);
        b.inst_results(inst)[0]
    };
    b.def_var(land_at, pending);

    let mut blocks = vec![None; pairs];
    let list: Vec<(u16, u32, u32)> = words.as_chunks::<3>().0.iter().map(|p| (p[0] as u16 & mask, p[1], p[2])).collect();
    for &(pc, _, _) in &list {
        blocks[pc as usize] = Some(b.create_block());
    }
    let dispatch = b.create_block();
    let exits = [b.create_block(), b.create_block(), b.create_block(), b.create_block(), b.create_block()];
    b.ins().jump(dispatch, &[]);

    let mut e = Emitter { b: &mut b, options, unit, mask, ptr, sigs, f, pipe, vu, link, data, limit, release, flags, budget, land_at, blocks, dispatch, exits };

    // The table: each covered pair's block, the outside exit for the rest.
    e.b.switch_to_block(dispatch);
    let pc = e.load(I16, pipe, P_PC);
    let pc32 = e.b.ins().uextend(I32, pc);
    let outside = e.exits[OUTSIDE as usize];
    let default = e.b.func.dfg.block_call(outside, &[]);
    let entries: Vec<_> = (0..pairs).map(|p| e.blocks[p].unwrap_or(outside)).collect();
    let calls: Vec<_> = entries.into_iter().map(|blk| e.b.func.dfg.block_call(blk, &[])).collect();
    let jt = e.b.create_jump_table(JumpTableData::new(default, &calls));
    e.b.ins().br_table(pc32, jt);

    for &(pc, upper, lower) in &list {
        let blk = e.blocks[pc as usize].unwrap();
        e.b.switch_to_block(blk);
        e.pair(pc, upper, lower);
    }
    for (code, blk) in exits.into_iter().enumerate() {
        e.b.switch_to_block(blk);
        let left = e.b.use_var(budget);
        e.store(left, f, F_BUDGET);
        let c = e.k(I32, code as u64);
        e.b.ins().return_(&[c]);
    }
    b.seal_all_blocks();
    b.finalize(module.target_config());
    Ok(())
}
