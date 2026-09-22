//! One block as Cranelift IR: the fast path the dispatcher's guard admits, each instruction inline or a call to its handler.
//! See Mars_Native.md §5.8 for what the guard proves and where each field of the step is brought up to date.

use std::mem::offset_of;

use cranelift_codegen::Context;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::types::{I8, I16, I32, I64};
use cranelift_codegen::ir::{self, AbiParam, InstBuilder, MemFlagsData, Value};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_jit::JITModule;
use cranelift_module::Module;

use super::{Context_, DONE, FINISH, Options, RAISED};
use crate::cpu::Cpu;
use crate::cpu::blocks::ops::{self, Handler};
use crate::cpu::blocks::shape::{self, Class};
use crate::cpu::blocks::verify;
use crate::cpu::interp::THROUGH_BUS;
use crate::memory::bus::MemoryBus;

const GPR: i32 = offset_of!(Cpu, gpr) as i32;
const HI: i32 = offset_of!(Cpu, hi) as i32;
const LO: i32 = offset_of!(Cpu, lo) as i32;
const PC: i32 = offset_of!(Cpu, pc) as i32;
const NEXT_PC: i32 = offset_of!(Cpu, next_pc) as i32;
const CURRENT_PC: i32 = offset_of!(Cpu, current_pc) as i32;
const BRANCH_PENDING: i32 = offset_of!(Cpu, branch_pending) as i32;
const IN_DELAY_SLOT: i32 = offset_of!(Cpu, in_delay_slot) as i32;
const INSTRUCTIONS: i32 = offset_of!(Cpu, instructions) as i32;
const EXTRA_CYCLES: i32 = offset_of!(Cpu, extra_cycles) as i32;
const IDLE_AT: i32 = offset_of!(Cpu, run.0.idle_at) as i32;
const CYCLES: i32 = offset_of!(MemoryBus, cycles) as i32;
const WRITTEN: i32 = offset_of!(MemoryBus, written.0) as i32;
const HALTED: i32 = offset_of!(MemoryBus, sp.processor.halted) as i32;
const REPEATING: i32 = offset_of!(MemoryBus, mi.repeating) as i32;
const X_ENTRY: i32 = offset_of!(Context_, entry) as i32;
const X_STOP: i32 = offset_of!(Context_, stop) as i32;
const X_RDRAM: i32 = offset_of!(Context_, rdram) as i32;
const X_LEN: i32 = offset_of!(Context_, rdram_len) as i32;
const X_READ_MARKS: i32 = offset_of!(Context_, read_marks) as i32;
const X_WRITE_MARKS: i32 = offset_of!(Context_, write_marks) as i32;
const X_AT: i32 = offset_of!(Context_, at) as i32;
const X_WRITTEN: i32 = offset_of!(Context_, written) as i32;

const KERNEL_DIRECT_BASE: i64 = 0xFFFF_FFFF_8000_0000u64 as i64;
const PAGE_MASK: i64 = 2047;

fn trusted() -> MemFlagsData {
    MemFlagsData::trusted()
}


#[inline]
fn rs(i: u32) -> usize {
    ((i >> 21) & 0x1F) as usize
}
#[inline]
fn rt(i: u32) -> usize {
    ((i >> 16) & 0x1F) as usize
}
#[inline]
fn rd(i: u32) -> usize {
    ((i >> 11) & 0x1F) as usize
}
#[inline]
fn sa(i: u32) -> i64 {
    ((i >> 6) & 0x1F) as i64
}
#[inline]
fn simm(i: u32) -> i64 {
    i as i16 as i64
}
#[inline]
fn imm(i: u32) -> i64 {
    (i & 0xFFFF) as i64
}

fn bit(r: usize) -> u32 {
    if r == 0 { 0 } else { 1 << r }
}

/// The general registers a handler may read and write, for a call-out: the rest stay in host registers across it.
fn effects(i: u32) -> (u32, u32) {
    const ALL: u32 = !1;
    let (s, t, d) = (bit(rs(i)), bit(rt(i)), bit(rd(i)));
    match i >> 26 {
        0x00 => match i & 0x3F {
            0x18..=0x1F => (s | t, 0),
            0x20 | 0x22 | 0x2C | 0x2E => (s | t, d),
            0x30..=0x34 | 0x36 | 0x0C | 0x0D => (s | t, 0),
            _ => (ALL, ALL),
        },
        0x01 => (s, 0),
        0x08 | 0x18 => (s, t),
        0x10..=0x12 => (t, t),
        0x1A | 0x1B | 0x20..=0x27 | 0x30 | 0x34 | 0x37 => (s | t, t),
        0x28..=0x2E | 0x3F => (s | t, 0),
        0x38 | 0x3C => (s | t, t),
        0x2F | 0x31 | 0x35 | 0x39 | 0x3D => (s, 0),
        _ => (ALL, ALL),
    }
}

/// What the emitter does with a word.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Plan {
    Pure,
    Load { size: u32, signed: bool },
    Store { size: u32 },
    MultDiv(i64),
    Call,
    Branch,
    CallBranch,
    Ender,
}

fn plan(i: u32, options: Options) -> Plan {
    let class = shape::class(i);
    if class == Class::Ender {
        return Plan::Ender;
    }
    match i >> 26 {
        0x00 => match i & 0x3F {
            0x00 | 0x02 | 0x03 | 0x04 | 0x06 | 0x07 | 0x0F | 0x10..=0x14 | 0x16 | 0x17 | 0x21 | 0x23..=0x27 | 0x2A | 0x2B | 0x2D | 0x2F | 0x38 | 0x3A
            | 0x3B | 0x3C | 0x3E | 0x3F => Plan::Pure,
            0x08 | 0x09 => Plan::Branch,
            0x18..=0x1F => Plan::MultDiv(shape::cycles(i) - 1),
            _ => Plan::Call,
        },
        0x01 => match rt(i) {
            0x00..=0x03 | 0x10..=0x13 => Plan::Branch,
            _ => Plan::Call,
        },
        0x02..=0x07 | 0x14..=0x17 => Plan::Branch,
        0x09..=0x0F | 0x19 => Plan::Pure,
        0x11 if rs(i) == 0x08 => Plan::CallBranch,
        0x20 => Plan::Load { size: 1, signed: true },
        0x21 => Plan::Load { size: 2, signed: true },
        0x23 => Plan::Load { size: 4, signed: true },
        0x24 => Plan::Load { size: 1, signed: false },
        0x25 => Plan::Load { size: 2, signed: false },
        0x27 => Plan::Load { size: 4, signed: false },
        0x37 => Plan::Load { size: 8, signed: false },
        0x28 if options.inline_stores => Plan::Store { size: 1 },
        0x29 if options.inline_stores => Plan::Store { size: 2 },
        0x2B if options.inline_stores => Plan::Store { size: 4 },
        0x3F if options.inline_stores => Plan::Store { size: 8 },
        0x28..=0x2E | 0x38 | 0x39 | 0x3C | 0x3D | 0x3F => Plan::Store { size: 0 },
        _ => Plan::Call,
    }
}

/// How the step's fields stand after an instruction, for the exits and the verifier.
#[derive(Clone, Copy)]
enum After {
    /// A plain instruction outside a slot: the next word is next, nothing pending.
    Plain,
    /// A branch done inline: the slot is next, with `next_pc` as it chose.
    Branch(Value),
    /// The fields are in memory already: a slot, a call-out branch, or a handler that set them.
    Memory,
}

/// The guest registers: in memory (step 2), or in Cranelift variables written back where an observer could look (step 3).
struct Regs {
    cached: bool,
    vars: [Option<Variable>; 32],
    dirty: u32,
}

struct Emitter<'a, 'b> {
    b: &'a mut FunctionBuilder<'b>,
    words: &'a [u32],
    start: u32,
    options: Options,
    cpu: Value,
    bus: Value,
    x: Value,
    entry: Value,
    stop: Value,
    ram: Value,
    len: Value,
    read_marks: Value,
    write_marks: Value,
    c0: Variable,
    i0: Variable,
    consumed: i64,
    done: i64,
    regs: Regs,
    handler: ir::SigRef,
    hook: ir::SigRef,
    step: ir::SigRef,
    ptr: ir::Type,
}

impl Emitter<'_, '_> {
    fn load(&mut self, ty: ir::Type, base: Value, offset: i32) -> Value {
        self.b.ins().load(ty, trusted(), base, offset)
    }

    fn store(&mut self, value: Value, base: Value, offset: i32) {
        self.b.ins().store(trusted(), value, base, offset);
    }

    fn store_imm(&mut self, ty: ir::Type, value: i64, base: Value, offset: i32) {
        let v = self.b.ins().iconst(ty, value);
        self.store(v, base, offset);
    }

    /// The virtual address of the block's `k`th word.
    fn at(&mut self, k: usize) -> Value {
        self.b.ins().iadd_imm_s(self.entry, 4 * k as i64)
    }

    fn gpr(&mut self, r: usize) -> Value {
        if r == 0 {
            return self.b.ins().iconst(I64, 0);
        }
        match self.regs.vars[r] {
            Some(v) if self.regs.cached => self.b.use_var(v),
            _ => {
                let cpu = self.cpu;
                self.load(I64, cpu, GPR + 8 * r as i32)
            }
        }
    }

    fn set_gpr(&mut self, r: usize, value: Value) {
        if r == 0 {
            return;
        }
        match self.regs.vars[r] {
            Some(v) if self.regs.cached => {
                self.b.def_var(v, value);
                self.regs.dirty |= 1 << r;
            }
            _ => {
                let cpu = self.cpu;
                self.store(value, cpu, GPR + 8 * r as i32);
            }
        }
    }

    /// Dirty registers in `mask` written back; with `keep`, they stay dirty on this path, as an exit's copy leaves them.
    fn flush(&mut self, mask: u32, keep: bool) {
        let d = self.regs.dirty & mask;
        for r in 1..32 {
            if d & (1 << r) != 0 {
                let v = self.b.use_var(self.regs.vars[r].unwrap());
                let cpu = self.cpu;
                self.store(v, cpu, GPR + 8 * r as i32);
            }
        }
        if !keep {
            self.regs.dirty &= !mask;
        }
    }

    /// Registers a handler wrote, read back into their variables.
    fn reload(&mut self, mask: u32) {
        if !self.regs.cached {
            return;
        }
        for r in 1..32 {
            if mask & (1 << r) != 0
                && let Some(var) = self.regs.vars[r]
            {
                let cpu = self.cpu;
                let v = self.load(I64, cpu, GPR + 8 * r as i32);
                self.b.def_var(var, v);
                self.regs.dirty &= !(1 << r);
            }
        }
    }

    fn sext32(&mut self, v: Value) -> Value {
        let low = self.b.ins().ireduce(I32, v);
        self.b.ins().sextend(I64, low)
    }

    /// The counters as they stand before the instruction under emission, or after it with `after`.
    fn flush_counters(&mut self, cycles: i64, instructions: i64) {
        let c0 = self.b.use_var(self.c0);
        let c = self.b.ins().iadd_imm_s(c0, cycles);
        let bus = self.bus;
        self.store(c, bus, CYCLES);
        let i0 = self.b.use_var(self.i0);
        let n = self.b.ins().iadd_imm_s(i0, instructions);
        let cpu = self.cpu;
        self.store(n, cpu, INSTRUCTIONS);
    }

    /// The step's fields as they stand after instruction `k`.
    fn fields_after(&mut self, k: usize, after: After) {
        let cpu = self.cpu;
        match after {
            After::Plain => {
                let current = self.at(k);
                self.store(current, cpu, CURRENT_PC);
                let pc = self.at(k + 1);
                self.store(pc, cpu, PC);
                let next = self.at(k + 2);
                self.store(next, cpu, NEXT_PC);
            }
            After::Branch(next) => {
                let current = self.at(k);
                self.store(current, cpu, CURRENT_PC);
                let pc = self.at(k + 1);
                self.store(pc, cpu, PC);
                self.store(next, cpu, NEXT_PC);
                self.store_imm(I8, 1, cpu, BRANCH_PENDING);
            }
            After::Memory => {}
        }
    }

    /// With the verifier on, the interpreter steps once and the two are compared, everything written back first.
    fn verify(&mut self, k: usize, after: After) {
        if !self.options.verify {
            return;
        }
        self.flush_counters(self.consumed, self.done);
        self.fields_after(k, after);
        self.flush(!0, true);
        let f = self.b.ins().iconst(self.ptr, hook as *const () as usize as i64);
        let n = self.b.ins().iconst(I32, k as i64);
        let (cpu, bus, x) = (self.cpu, self.bus, self.x);
        self.b.ins().call_indirect(self.hook, f, &[cpu, bus, x, n]);
        if let After::Branch(_) = after {
            let cpu = self.cpu;
            self.store_imm(I8, 0, cpu, BRANCH_PENDING);
        }
    }

    /// Leaves the block: the counters and every dirty register written back, the exit's instruction named.
    fn exit(&mut self, code: u32, k: usize, cycles: i64, instructions: i64) {
        self.flush_counters(cycles, instructions);
        self.flush(!0, true);
        let x = self.x;
        self.store_imm(I32, k as i64, x, X_AT);
        let c = self.b.ins().iconst(I32, code as i64);
        self.b.ins().return_(&[c]);
    }

    /// A cold block for an exit, entered when `condition` holds; emission goes on in the other branch.
    fn exit_if(&mut self, condition: Value, body: impl FnOnce(&mut Self)) {
        let out = self.b.create_block();
        let on = self.b.create_block();
        self.b.set_cold_block(out);
        self.b.ins().brif(condition, out, &[], on, &[]);
        self.b.switch_to_block(out);
        let dirty = self.regs.dirty;
        body(self);
        self.regs.dirty = dirty;
        self.b.switch_to_block(on);
    }

    /// A call to the instruction's own handler; a raise leaves the block, and the handler's answer is returned.
    fn call(&mut self, k: usize, word: u32, slot: bool, sets_pc: bool) -> Value {
        let (reads, writes) = effects(word);
        self.flush_counters(self.consumed, self.done);
        self.flush(reads, false);
        let cpu = self.cpu;
        if !slot {
            let current = self.at(k);
            self.store(current, cpu, CURRENT_PC);
            if sets_pc {
                let pc = self.at(k + 1);
                self.store(pc, cpu, PC);
                let next = self.at(k + 2);
                self.store(next, cpu, NEXT_PC);
            }
        }
        let handler: Handler = ops::handler(word);
        let f = self.b.ins().iconst(self.ptr, handler as usize as i64);
        let w = self.b.ins().iconst(I32, word as i64);
        let bus = self.bus;
        let call = self.b.ins().call_indirect(self.handler, f, &[cpu, bus, w]);
        let answer = self.b.inst_results(call)[0];
        self.reload(writes);
        let raised = self.b.ins().icmp_imm_s(IntCC::Equal, answer, ops::RAISED as i64);
        let (consumed, done) = (self.consumed, self.done);
        self.exit_if(raised, |e| e.exit(RAISED, k, consumed, done));
        answer
    }

    /// The exit after a store that may have rewritten this block's words or reached a device: its tick is the dispatcher's.
    fn landing(&mut self, k: usize, landed: Value, slot: bool) {
        let bus_path = self.b.ins().icmp_imm_s(IntCC::Equal, landed, THROUGH_BUS as i64);
        let low = self.b.ins().iadd_imm_s(landed, 8);
        let above = self.b.ins().icmp_imm_s(IntCC::UnsignedGreaterThan, low, self.start as i64);
        let below = self.b.ins().icmp_imm_s(IntCC::UnsignedLessThan, landed, (self.start + 4 * self.words.len() as u32) as i64);
        let inside = self.b.ins().band(above, below);
        let rewrote = self.b.ins().bor(bus_path, inside);
        let (consumed, done) = (self.consumed, self.done);
        self.exit_if(rewrote, |e| {
            if !slot {
                e.fields_after(k, After::Plain);
            }
            e.exit(FINISH, k, consumed, done);
        });
    }

    fn pure(&mut self, i: u32) {
        let op = i >> 26;
        if op == 0 {
            let f = i & 0x3F;
            let value = match f {
                0x0F => return,
                0x10 => {
                    let cpu = self.cpu;
                    self.load(I64, cpu, HI)
                }
                0x12 => {
                    let cpu = self.cpu;
                    self.load(I64, cpu, LO)
                }
                0x11 | 0x13 => {
                    let v = self.gpr(rs(i));
                    let cpu = self.cpu;
                    self.store(v, cpu, if f == 0x11 { HI } else { LO });
                    return;
                }
                _ => {
                    let t = self.gpr(rt(i));
                    let s = self.gpr(rs(i));
                    let ins = self.b.ins();
                    match f {
                        0x00 => {
                            let r = ins.ishl_imm_s(t, sa(i));
                            self.sext32(r)
                        }
                        0x02 => {
                            let w = ins.ireduce(I32, t);
                            let r = self.b.ins().ushr_imm_s(w, sa(i));
                            self.b.ins().sextend(I64, r)
                        }
                        0x03 => {
                            let r = ins.sshr_imm_s(t, sa(i));
                            self.sext32(r)
                        }
                        0x04 => {
                            let n = ins.band_imm_s(s, 0x1F);
                            let r = self.b.ins().ishl(t, n);
                            self.sext32(r)
                        }
                        0x06 => {
                            let n = ins.band_imm_s(s, 0x1F);
                            let w = self.b.ins().ireduce(I32, t);
                            let r = self.b.ins().ushr(w, n);
                            self.b.ins().sextend(I64, r)
                        }
                        0x07 => {
                            let n = ins.band_imm_s(s, 0x1F);
                            let r = self.b.ins().sshr(t, n);
                            self.sext32(r)
                        }
                        0x14 => {
                            let n = ins.band_imm_s(s, 0x3F);
                            self.b.ins().ishl(t, n)
                        }
                        0x16 => {
                            let n = ins.band_imm_s(s, 0x3F);
                            self.b.ins().ushr(t, n)
                        }
                        0x17 => {
                            let n = ins.band_imm_s(s, 0x3F);
                            self.b.ins().sshr(t, n)
                        }
                        0x21 => {
                            let r = ins.iadd(s, t);
                            self.sext32(r)
                        }
                        0x23 => {
                            let r = ins.isub(s, t);
                            self.sext32(r)
                        }
                        0x24 => ins.band(s, t),
                        0x25 => ins.bor(s, t),
                        0x26 => ins.bxor(s, t),
                        0x27 => {
                            let r = ins.bor(s, t);
                            self.b.ins().bnot(r)
                        }
                        0x2A => {
                            let c = ins.icmp(IntCC::SignedLessThan, s, t);
                            self.b.ins().uextend(I64, c)
                        }
                        0x2B => {
                            let c = ins.icmp(IntCC::UnsignedLessThan, s, t);
                            self.b.ins().uextend(I64, c)
                        }
                        0x2D => ins.iadd(s, t),
                        0x2F => ins.isub(s, t),
                        0x38 => ins.ishl_imm_s(t, sa(i)),
                        0x3A => ins.ushr_imm_s(t, sa(i)),
                        0x3B => ins.sshr_imm_s(t, sa(i)),
                        0x3C => ins.ishl_imm_s(t, sa(i) + 32),
                        0x3E => ins.ushr_imm_s(t, sa(i) + 32),
                        0x3F => ins.sshr_imm_s(t, sa(i) + 32),
                        _ => unreachable!("not a pure special function: {f:02X}"),
                    }
                }
            };
            self.set_gpr(rd(i), value);
            return;
        }
        let s = self.gpr(rs(i));
        let ins = self.b.ins();
        let value = match op {
            0x09 => {
                let r = ins.iadd_imm_s(s, simm(i));
                self.sext32(r)
            }
            0x0A => {
                let c = ins.icmp_imm_s(IntCC::SignedLessThan, s, simm(i));
                self.b.ins().uextend(I64, c)
            }
            0x0B => {
                let c = ins.icmp_imm_s(IntCC::UnsignedLessThan, s, simm(i));
                self.b.ins().uextend(I64, c)
            }
            0x0C => ins.band_imm_s(s, imm(i)),
            0x0D => ins.bor_imm_s(s, imm(i)),
            0x0E => ins.bxor_imm_s(s, imm(i)),
            0x0F => ins.iconst(I64, (imm(i) << 16) as i32 as i64),
            0x19 => ins.iadd_imm_s(s, simm(i)),
            _ => unreachable!("not a pure opcode: {op:02X}"),
        };
        self.set_gpr(rt(i), value);
    }

    /// The direct kernel address `address`'s physical address, and whether an access of `size` there may skip the handler.
    fn direct(&mut self, address: Value, size: u32, marks: Value) -> (Value, Value) {
        let offset = self.b.ins().iadd_imm_s(address, -KERNEL_DIRECT_BASE);
        let direct = self.b.ins().icmp_imm_s(IntCC::UnsignedLessThan, offset, 0x4000_0000);
        let physical = self.b.ins().band_imm_s(address, 0x1FFF_FFFF);
        let inside = self.b.ins().icmp(IntCC::UnsignedLessThan, physical, self.len);
        let mut ok = self.b.ins().band(direct, inside);
        if size > 1 {
            let low = self.b.ins().band_imm_s(address, size as i64 - 1);
            let aligned = self.b.ins().icmp_imm_s(IntCC::Equal, low, 0);
            ok = self.b.ins().band(ok, aligned);
        }
        let page = self.b.ins().ushr_imm_s(physical, 12);
        let page = self.b.ins().band_imm_s(page, PAGE_MASK);
        let at = self.b.ins().ishl_imm_s(page, 3);
        let at = self.b.ins().iadd(marks, at);
        let mark = self.load(I64, at, 0);
        let clear = self.b.ins().icmp_imm_s(IntCC::Equal, mark, 0);
        ok = self.b.ins().band(ok, clear);
        (physical, ok)
    }

    fn load_inline(&mut self, k: usize, i: u32, size: u32, signed: bool, slot: bool) {
        let base = self.gpr(rs(i));
        let address = self.b.ins().iadd_imm_s(base, simm(i));
        let marks = self.read_marks;
        let (physical, ok) = self.direct(address, size, marks);
        let fast = self.b.create_block();
        let slow = self.b.create_block();
        let joined = self.b.create_block();
        self.b.set_cold_block(slow);
        self.b.ins().brif(ok, fast, &[], slow, &[]);

        let dirty = self.regs.dirty;
        self.b.switch_to_block(fast);
        let at = self.b.ins().iadd(self.ram, physical);
        // RDRAM is big-endian: a host load, swapped, then extended as the instruction says.
        let value = match size {
            1 if signed => self.b.ins().sload8(I64, trusted(), at, 0),
            1 => self.b.ins().uload8(I64, trusted(), at, 0),
            _ => {
                let ty = match size {
                    2 => I16,
                    4 => I32,
                    _ => I64,
                };
                let raw = self.b.ins().load(ty, trusted(), at, 0);
                let swapped = self.b.ins().bswap(raw);
                match (size, signed) {
                    (8, _) => swapped,
                    (_, true) => self.b.ins().sextend(I64, swapped),
                    _ => self.b.ins().uextend(I64, swapped),
                }
            }
        };
        self.set_gpr(rt(i), value);
        let fast_dirty = self.regs.dirty;
        self.b.ins().jump(joined, &[]);

        self.b.switch_to_block(slow);
        self.regs.dirty = dirty;
        self.call(k, i, slot, false);
        let slow_dirty = self.regs.dirty;
        self.b.ins().jump(joined, &[]);

        self.b.switch_to_block(joined);
        self.regs.dirty = fast_dirty | slow_dirty;
    }

    fn store_inline(&mut self, k: usize, i: u32, size: u32, slot: bool) {
        let base = self.gpr(rs(i));
        let address = self.b.ins().iadd_imm_s(base, simm(i));
        let marks = self.write_marks;
        let (physical, ok) = self.direct(address, size, marks);
        let bus = self.bus;
        let repeating = self.b.ins().uload8(I32, trusted(), bus, REPEATING);
        let quiet = self.b.ins().icmp_imm_s(IntCC::Equal, repeating, 0);
        let ok = self.b.ins().band(ok, quiet);
        let fast = self.b.create_block();
        let slow = self.b.create_block();
        let joined = self.b.create_block();
        let landed = self.b.append_block_param(joined, I64);
        self.b.set_cold_block(slow);
        self.b.ins().brif(ok, fast, &[], slow, &[]);

        let dirty = self.regs.dirty;
        self.b.switch_to_block(fast);
        let value = self.gpr(rt(i));
        let at = self.b.ins().iadd(self.ram, physical);
        if size == 1 {
            self.b.ins().istore8(trusted(), value, at, 0);
        } else {
            let ty = match size {
                2 => I16,
                4 => I32,
                _ => I64,
            };
            let narrow = if size == 8 { value } else { self.b.ins().ireduce(ty, value) };
            let swapped = self.b.ins().bswap(narrow);
            self.b.ins().store(trusted(), swapped, at, 0);
        }
        self.b.ins().jump(joined, &[ir::BlockArg::Value(physical)]);

        self.b.switch_to_block(slow);
        let answer = self.call(k, i, slot, false);
        let reported = self.b.ins().band_imm_s(answer, 0xFFFF_FFFF);
        self.b.ins().jump(joined, &[ir::BlockArg::Value(reported)]);

        self.b.switch_to_block(joined);
        self.regs.dirty = dirty;
        self.landing(k, landed, slot);
    }

    /// An inline branch at `k`: returns whether it was taken and the next counter it chose; a likely branch not taken leaves.
    fn branch(&mut self, k: usize, i: u32) -> (Value, Value) {
        let op = i >> 26;
        let cpu = self.cpu;
        let slot_at = self.at(k + 1);
        let after_slot = self.at(k + 2);
        let one = self.b.ins().iconst(I8, 1);
        let (taken, target, likely, link) = match op {
            0x00 => {
                let target = self.gpr(rs(i));
                if i & 0x3F == 0x09 {
                    self.set_gpr(rd(i), after_slot);
                }
                (one, target, false, 0)
            }
            0x02 | 0x03 => {
                let high = self.b.ins().band_imm_s(slot_at, 0xFFFF_FFFF_F000_0000u64 as i64);
                let target = self.b.ins().bor_imm_s(high, ((i & 0x03FF_FFFF) << 2) as i64);
                (one, target, false, if op == 0x03 { 31 } else { 0 })
            }
            _ => {
                let s = self.gpr(rs(i));
                let (condition, likely, link) = if op == 0x01 {
                    let r = rt(i) as u32;
                    let condition = if r & 1 == 0 { IntCC::SignedLessThan } else { IntCC::SignedGreaterThanOrEqual };
                    (self.b.ins().icmp_imm_s(condition, s, 0), r & 2 != 0, if r & 0x10 != 0 { 31 } else { 0 })
                } else {
                    let condition = match op & 3 {
                        0 => {
                            let t = self.gpr(rt(i));
                            self.b.ins().icmp(IntCC::Equal, s, t)
                        }
                        1 => {
                            let t = self.gpr(rt(i));
                            self.b.ins().icmp(IntCC::NotEqual, s, t)
                        }
                        2 => self.b.ins().icmp_imm_s(IntCC::SignedLessThanOrEqual, s, 0),
                        _ => self.b.ins().icmp_imm_s(IntCC::SignedGreaterThan, s, 0),
                    };
                    (condition, op >= 0x14, 0)
                };
                let target = self.b.ins().iadd_imm_s(slot_at, simm(i) << 2);
                (condition, target, likely, link)
            }
        };
        if link != 0 {
            self.set_gpr(link, after_slot);
        }

        // `Branch`: a taken branch to its own address marks where the idle loop may be.
        if op == 0x00 || op == 0x02 || op == 0x03 {
            let current = self.at(k);
            let itself = self.b.ins().icmp(IntCC::Equal, target, current);
            let mark = self.b.create_block();
            let on = self.b.create_block();
            self.b.ins().brif(itself, mark, &[], on, &[]);
            self.b.switch_to_block(mark);
            let current = self.at(k);
            self.store(current, cpu, IDLE_AT);
            self.b.ins().jump(on, &[]);
            self.b.switch_to_block(on);
        } else if simm(i) == -1 {
            let current = self.at(k);
            let mark = self.b.create_block();
            let on = self.b.create_block();
            self.b.ins().brif(taken, mark, &[], on, &[]);
            self.b.switch_to_block(mark);
            self.store(current, cpu, IDLE_AT);
            self.b.ins().jump(on, &[]);
            self.b.switch_to_block(on);
        }

        if likely {
            // Not taken, the slot is thrown away: the counters move past it, nothing pending, and the block ends after the branch's tick.
            let nullified = self.b.create_block();
            let on = self.b.create_block();
            self.b.ins().brif(taken, on, &[], nullified, &[]);
            self.b.switch_to_block(nullified);
            let dirty = self.regs.dirty;
            let current = self.at(k);
            self.store(current, cpu, CURRENT_PC);
            let pc = self.at(k + 2);
            self.store(pc, cpu, PC);
            let next = self.at(k + 3);
            self.store(next, cpu, NEXT_PC);
            self.tick_and_leave(k, 1, After::Memory);
            self.regs.dirty = dirty;
            self.b.switch_to_block(on);
        }
        let next = self.b.ins().select(taken, target, after_slot);
        (taken, next)
    }

    /// The delay slot's own step: the fields the interpreter sets before it, from where the branch left them.
    fn open_slot(&mut self, k: usize, next: Option<Value>) {
        let cpu = self.cpu;
        let current = self.at(k);
        self.store(current, cpu, CURRENT_PC);
        let next = match next {
            Some(v) => {
                self.store_imm(I8, 1, cpu, IN_DELAY_SLOT);
                v
            }
            None => {
                let pending = self.load(I8, cpu, BRANCH_PENDING);
                self.store(pending, cpu, IN_DELAY_SLOT);
                self.load(I64, cpu, NEXT_PC)
            }
        };
        self.store_imm(I8, 0, cpu, BRANCH_PENDING);
        self.store(next, cpu, PC);
        let after = self.b.ins().iadd_imm_s(next, 4);
        self.store(after, cpu, NEXT_PC);
    }

    /// The signal processor's share of the tick, in the variant that runs beside it: stepped as the tick steps it, and the
    /// block left at once if the step wrote memory or moved the interrupt line, as C#'s `RspRan` leaves.
    fn beside(&mut self, k: usize, cycles: i64, after: After) {
        if !self.options.beside {
            return;
        }
        let bus = self.bus;
        let halted = self.b.ins().uload8(I32, trusted(), bus, HALTED);
        let step = self.b.create_block();
        let on = self.b.create_block();
        self.b.ins().brif(halted, on, &[], step, &[]);
        self.b.switch_to_block(step);
        let c0 = self.b.use_var(self.c0);
        let now = self.b.ins().iadd_imm_s(c0, self.consumed);
        self.store(now, bus, CYCLES);
        let f = self.b.ins().iconst(self.ptr, rsp_tick as *const () as usize as i64);
        let n = self.b.ins().iconst(I32, cycles);
        let (cpu, x) = (self.cpu, self.x);
        let call = self.b.ins().call_indirect(self.step, f, &[cpu, bus, x, n]);
        let changed = self.b.inst_results(call)[0];
        let out = self.b.create_block();
        self.b.ins().brif(changed, out, &[], on, &[]);
        self.b.switch_to_block(out);
        let dirty = self.regs.dirty;
        self.leave(k, cycles, after);
        self.regs.dirty = dirty;
        self.b.switch_to_block(on);
    }

    /// Out after instruction `k` and its tick, at a step boundary: the fields, the counters, the verifier, the registers.
    fn leave(&mut self, k: usize, cycles: i64, after: After) {
        let (consumed, done) = (self.consumed, self.done);
        self.consumed += cycles;
        self.done += 1;
        self.verify(k, after);
        self.fields_after(k, after);
        let (c, d) = (self.consumed, self.done);
        self.exit(DONE, k, c, d);
        self.consumed = consumed;
        self.done = done;
    }

    /// The instruction's tick: the processor beside it if it runs, then the counters and the verifier.
    fn tick(&mut self, k: usize, cycles: i64, after: After) {
        self.beside(k, cycles, after);
        self.consumed += cycles;
        self.done += 1;
        self.verify(k, after);
    }

    /// A tick on a path that leaves right after it.
    fn tick_and_leave(&mut self, k: usize, cycles: i64, after: After) {
        self.beside(k, cycles, after);
        self.leave(k, cycles, after);
    }

    /// One instruction; returns the next counter and taken flag a branch chose, for its slot and the loop-back.
    fn instruction(&mut self, k: usize, slot: bool) -> Option<(Option<Value>, Option<Value>)> {
        let i = self.words[k];
        let after = if slot { After::Memory } else { After::Plain };
        let mut cycles = 1;
        match plan(i, self.options) {
            Plan::Pure => self.pure(i),
            Plan::Load { size, signed } => self.load_inline(k, i, size, signed, slot),
            Plan::Store { size: 0 } => {
                let answer = self.call(k, i, slot, false);
                let landed = self.b.ins().band_imm_s(answer, 0xFFFF_FFFF);
                self.landing(k, landed, slot);
                if self.options.beside {
                    // A store through the bus into RDRAM counts as a write; it is the block's own, and not the processor's.
                    let (bus, x) = (self.bus, self.x);
                    let written = self.load(I64, bus, WRITTEN);
                    self.store(written, x, X_WRITTEN);
                }
            }
            Plan::Store { size } => self.store_inline(k, i, size, slot),
            Plan::MultDiv(stall) => {
                self.call(k, i, slot, false);
                let cpu = self.cpu;
                self.store_imm(I32, 0, cpu, EXTRA_CYCLES);
                cycles += stall;
            }
            Plan::Call => {
                self.call(k, i, slot, false);
            }
            Plan::Ender => {
                self.call(k, i, slot, true);
                let (consumed, done) = (self.consumed, self.done);
                self.exit(FINISH, k, consumed, done);
                return None;
            }
            Plan::Branch => {
                let (taken, next) = self.branch(k, i);
                self.tick(k, 1, After::Branch(next));
                return Some((Some(next), Some(taken)));
            }
            Plan::CallBranch => {
                self.call(k, i, slot, true);
                let cpu = self.cpu;
                let pending = self.load(I8, cpu, BRANCH_PENDING);
                let nullified = self.b.create_block();
                let on = self.b.create_block();
                self.b.ins().brif(pending, on, &[], nullified, &[]);
                self.b.switch_to_block(nullified);
                let dirty = self.regs.dirty;
                self.tick_and_leave(k, 1, After::Memory);
                self.regs.dirty = dirty;
                self.b.switch_to_block(on);
                self.tick(k, 1, After::Memory);
                return Some((None, None));
            }
        }
        self.tick(k, cycles, after);
        Some((None, None))
    }
}

/// The signal processor's step inside a tick, and whether it wrote memory or moved the interrupt line since the block's entry.
unsafe extern "C" fn rsp_tick(cpu: *mut Cpu, bus: *mut MemoryBus, x: *mut Context_, cycles: u32) -> u32 {
    // SAFETY: the compiled block lends the machine and its context for the call.
    let (cpu, bus, x) = unsafe { (&mut *cpu, &mut *bus, &*x) };
    bus.cycles += cycles as i64;
    bus.sp_step(cycles as i64);
    (*bus.written != x.written || bus.mi.asserted() != cpu.run.asserted_seen) as u32
}

/// The verifier's call: the interpreter's step, then the comparison; the count a step leaves is written first.
unsafe extern "C" fn hook(cpu: *mut Cpu, bus: *mut MemoryBus, x: *mut Context_, _k: u32) {
    // SAFETY: the compiled block lends the machine and its context for the call.
    let (cpu, bus, x) = unsafe { (&mut *cpu, &mut *bus, &mut *x) };
    cpu.last_count = bus.count();
    if let Some(shadow) = unsafe { x.shadow.as_mut() } {
        verify::Shadow::follow(shadow, cpu, bus, "a compiled instruction");
    }
}

/// Emits and defines one block; returns its function and the bytes of its code.
pub fn compile(module: &mut JITModule, context: &mut Context, functions: &mut FunctionBuilderContext, words: &[u32], start: u32, options: Options, name: &str) -> Result<(cranelift_module::FuncId, usize), String> {
    module.clear_context(context);
    let ptr = module.target_config().pointer_type();
    context.func.signature.params.extend([AbiParam::new(ptr), AbiParam::new(ptr), AbiParam::new(ptr)]);
    context.func.signature.returns.push(AbiParam::new(I32));
    let mut handler = module.make_signature();
    handler.params.extend([AbiParam::new(ptr), AbiParam::new(ptr), AbiParam::new(I32)]);
    handler.returns.push(AbiParam::new(I64));
    let mut hook = module.make_signature();
    hook.params.extend([AbiParam::new(ptr), AbiParam::new(ptr), AbiParam::new(ptr), AbiParam::new(I32)]);
    let mut step = hook.clone();
    step.returns.push(AbiParam::new(I32));

    {
        let mut b = FunctionBuilder::new(&mut context.func, functions);
        let head = b.create_block();
        b.append_block_params_for_function_params(head);
        b.switch_to_block(head);
        let (cpu, bus, x) = (b.block_params(head)[0], b.block_params(head)[1], b.block_params(head)[2]);
        let handler = b.import_signature(handler);
        let hook = b.import_signature(hook);
        let step = b.import_signature(step);
        let entry = b.ins().load(I64, trusted(), x, X_ENTRY);
        let stop = b.ins().load(I64, trusted(), x, X_STOP);
        let ram = b.ins().load(ptr, trusted(), x, X_RDRAM);
        let len = b.ins().load(I64, trusted(), x, X_LEN);
        let read_marks = b.ins().load(ptr, trusted(), x, X_READ_MARKS);
        let write_marks = b.ins().load(ptr, trusted(), x, X_WRITE_MARKS);
        let c0 = b.declare_var(I64);
        let i0 = b.declare_var(I64);
        let cycles = b.ins().load(I64, trusted(), bus, CYCLES);
        b.def_var(c0, cycles);
        let instructions = b.ins().load(I64, trusted(), cpu, INSTRUCTIONS);
        b.def_var(i0, instructions);

        // Every register the block names is loaded once when they are held; the block's writes are its dirty set at the loop's head.
        let mut vars = [None; 32];
        let mut written = 0u32;
        if options.cached {
            for &w in words {
                let (s, t, d) = (rs(w), rt(w), rd(w));
                for r in [s, t, d, 31] {
                    if r != 0 && vars[r].is_none() {
                        let v = b.declare_var(I64);
                        let value = b.ins().load(I64, trusted(), cpu, GPR + 8 * r as i32);
                        b.def_var(v, value);
                        vars[r] = Some(v);
                    }
                }
                written |= bit(d) | bit(t) | bit(31);
            }
        }
        let top = b.create_block();
        b.ins().jump(top, &[]);
        b.switch_to_block(top);

        let loops = loops_back(words);
        let mut e = Emitter {
            b: &mut b,
            words,
            start,
            options,
            cpu,
            bus,
            x,
            entry,
            stop,
            ram,
            len,
            read_marks,
            write_marks,
            c0,
            i0,
            consumed: 0,
            done: 0,
            regs: Regs { cached: options.cached, vars, dirty: if loops { written } else { 0 } },
            handler,
            hook,
            step,
            ptr,
        };

        let n = words.len();
        let mut k = 0;
        let mut ended = false;
        while k < n {
            match e.instruction(k, false) {
                None => {
                    ended = true;
                    break;
                }
                Some((Some(next), taken)) => {
                    // The slot, then the block's end or its loop back to the head.
                    e.open_slot(k + 1, Some(next));
                    if e.instruction(k + 1, true).is_none() {
                        ended = true;
                        break;
                    }
                    finish_after_slot(&mut e, k + 1, loops.then_some(taken).flatten(), top);
                    ended = true;
                    break;
                }
                Some((None, None)) if shape::class(words[k]) == Class::Branch => {
                    e.open_slot(k + 1, None);
                    if e.instruction(k + 1, true).is_none() {
                        ended = true;
                        break;
                    }
                    finish_after_slot(&mut e, k + 1, None, top);
                    ended = true;
                    break;
                }
                Some(_) => k += 1,
            }
        }
        if !ended {
            // Ended by the cap or the page: the last word's successor is next.
            let last = n - 1;
            e.fields_after(last, After::Plain);
            let (consumed, done) = (e.consumed, e.done);
            e.exit(DONE, last, consumed, done);
        }
        b.seal_all_blocks();
        b.finalize(module.target_config());
    }

    let id = super::declare(module, name, context)?;
    module.define_function(id, context).map_err(|e| format!("{e:?}"))?;
    let bytes = context.compiled_code().map_or(0, |c| c.code_info().total_size as usize);
    Ok((id, bytes))
}

/// Whether the block's branch returns to its own first word, which a pass loops on rather than leaving; the idle loop is the frame loop's.
fn loops_back(words: &[u32]) -> bool {
    let n = words.len();
    if n < 2 || shape::class(words[n - 2]) != Class::Branch {
        return false;
    }
    let i = words[n - 2];
    let relative = matches!(i >> 26, 0x04..=0x07 | 0x14..=0x17) || (i >> 26 == 0x01 && matches!(rt(i), 0x00..=0x03 | 0x10..=0x13));
    let idle = n == 2 && i == 0x1000_FFFF && words[1] == 0;
    relative && !idle && simm(i) == -((n - 1) as i64)
}

/// After the slot: back to the head while the guard holds and the branch was taken, else out at a step boundary.
fn finish_after_slot(e: &mut Emitter, slot: usize, taken: Option<Value>, top: ir::Block) {
    let (consumed, done) = (e.consumed, e.done);
    if let Some(taken) = taken {
        let back = e.b.create_block();
        let out = e.b.create_block();
        e.b.ins().brif(taken, back, &[], out, &[]);

        e.b.switch_to_block(back);
        let c0 = e.b.use_var(e.c0);
        let cycles = e.b.ins().iadd_imm_s(c0, consumed);
        let i0 = e.b.use_var(e.i0);
        let instructions = e.b.ins().iadd_imm_s(i0, done);
        let max: i64 = e.words.iter().map(|&w| shape::cycles(w)).sum();
        let reach = e.b.ins().iadd_imm_s(cycles, max);
        let fits = e.b.ins().icmp(IntCC::SignedLessThan, reach, e.stop);
        let again = e.b.create_block();
        let stay_out = e.b.create_block();
        e.b.ins().brif(fits, again, &[], stay_out, &[]);

        e.b.switch_to_block(again);
        e.b.def_var(e.c0, cycles);
        e.b.def_var(e.i0, instructions);
        let cpu = e.cpu;
        e.store_imm(I8, 0, cpu, IN_DELAY_SLOT);
        e.b.ins().jump(top, &[]);

        e.b.switch_to_block(stay_out);
        e.exit(DONE, slot, consumed, done);

        e.b.switch_to_block(out);
    }
    e.exit(DONE, slot, consumed, done);
}
