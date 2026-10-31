//! The EE's tier: the R5900's table for the planner, a block's folded cost, what a slow path is told, and the shared stubs (Sapphire_Speed.md §66).

use std::mem::offset_of;

use emusen_assembler::x64::{self, X64};
use emusen_assembler::{Assembler, Code, Op, Src, Width, at};
use emusen_mips_planner::{self as planner, Access, Class, Info, Isa, Plan};

use super::super::ops;
use super::super::timing::Timing;
use super::{Context, Options, lower, shape};
use crate::ee::Ee;

/// The plain instructions compiled in place.
const PLAIN: &[&str] = &[
    "add", "addu", "sub", "subu", "addi", "addiu", "dadd", "daddu", "dsub", "dsubu", "daddi", "daddiu", "and", "or", "xor", "nor", "andi",
    "ori", "xori", "lui", "slt", "sltu", "slti", "sltiu", "movz", "movn", "sll", "srl", "sra", "sllv", "srlv", "srav", "dsll", "dsrl",
    "dsra", "dsll32", "dsrl32", "dsra32", "dsllv", "dsrlv", "dsrav", "sync", "sync.p", "mfhi", "mflo", "mthi", "mtlo", "mfhi1", "mflo1",
    "mthi1", "mtlo1", "mult", "multu", "mult1", "multu1", "madd", "maddu", "madd1", "maddu1", "div", "divu", "div1", "divu1", "mfsa",
    "mtsa", "mtsab", "mtsah", "tge", "tgeu", "tlt", "tltu", "teq", "tne", "tgei", "tgeiu", "tlti", "tltiu", "teqi", "tnei",
];
/// The loads and stores.
const MEMORY: &[&str] = &[
    "lb", "lbu", "lh", "lhu", "lw", "lwu", "ld", "lq", "sb", "sh", "sw", "sd", "sq", "lwl", "lwr", "ldl", "ldr", "swl", "swr", "sdl", "sdr",
    "pref", "lwc1", "swc1",
];

fn bit(r: u32) -> u32 {
    if r == 0 { 0 } else { 1 << r }
}

/// The R5900's table for the planner; everything else ends a block.
pub(crate) struct R5900Isa;

impl Isa for R5900Isa {
    const MAX: usize = shape::MAX;

    fn info(&self, w: u32) -> Info {
        let (rs, rt, rd) = (w >> 21 & 31, w >> 16 & 31, w >> 11 & 31);
        let name = shape::name(w);
        let plain = |reads, writes| Info { class: Class::Plain, reads, writes, access: None, likely: false };
        let branch = |reads, writes, likely| Info { class: Class::Branch, reads, writes, access: None, likely };
        if shape::class(w) == shape::Class::Call {
            let op = ops::get(ops::index(w));
            match name {
                "qmfc2" | "cfc2" => return plain(0, bit(rt)),
                "qmtc2" | "ctc2" => return plain(bit(rt), 0),
                "lqc2" | "sqc2" => {
                    let access = Access { base: rs as u8, disp: w as u16 as i16, bytes: 16, store: name == "sqc2", align: 0xFFFF_FFFF };
                    return Info { class: Class::Memory, reads: bit(rs), writes: 0, access: Some(access), likely: false };
                }
                _ => {}
            }
            let regs = |u: u8| match op.kind {
                ops::Kind::Fpu if u & super::super::fpu::uses::GRT != 0 => bit(rt),
                ops::Kind::Fpu => 0,
                _ => {
                    use ops::uses::{RD, RS, RT};
                    (if u & RS != 0 { bit(rs) } else { 0 }) | (if u & RT != 0 { bit(rt) } else { 0 }) | (if u & RD != 0 { bit(rd) } else { 0 })
                }
            };
            return plain(regs(op.reads), regs(op.writes));
        }
        if PLAIN.contains(&name) {
            return match name {
                "sync" | "sync.p" => plain(0, 0),
                "mfhi" | "mflo" | "mfhi1" | "mflo1" | "mfsa" => plain(0, bit(rd)),
                "mthi" | "mtlo" | "mthi1" | "mtlo1" | "mtsa" | "mtsab" | "mtsah" => plain(bit(rs), 0),
                "mult" | "multu" | "mult1" | "multu1" | "madd" | "maddu" | "madd1" | "maddu1" => plain(bit(rs) | bit(rt), bit(rd)),
                "div" | "divu" | "div1" | "divu1" | "tge" | "tgeu" | "tlt" | "tltu" | "teq" | "tne" => plain(bit(rs) | bit(rt), 0),
                "tgei" | "tgeiu" | "tlti" | "tltiu" | "teqi" | "tnei" => plain(bit(rs), 0),
                "lui" => plain(0, bit(rt)),
                "addi" | "addiu" | "daddi" | "daddiu" | "andi" | "ori" | "xori" | "slti" | "sltiu" => plain(bit(rs), bit(rt)),
                "sll" | "srl" | "sra" | "dsll" | "dsrl" | "dsra" | "dsll32" | "dsrl32" | "dsra32" => plain(bit(rt), bit(rd)),
                "movz" | "movn" => plain(bit(rs) | bit(rt) | bit(rd), bit(rd)),
                _ => plain(bit(rs) | bit(rt), bit(rd)),
            };
        }
        if MEMORY.contains(&name) {
            let (bytes, store) = match name {
                "lb" | "lbu" | "pref" => (1, false),
                "lwl" | "lwr" => (4, false),
                "ldl" | "ldr" => (8, false),
                "swl" | "swr" => (4, true),
                "sdl" | "sdr" => (8, true),
                "lh" | "lhu" => (2, false),
                "lw" | "lwu" | "lwc1" => (4, false),
                "ld" => (8, false),
                "lq" => (16, false),
                "sb" => (1, true),
                "sh" => (2, true),
                "sw" | "swc1" => (4, true),
                "sd" => (8, true),
                _ => (16, true),
            };
            let unit = matches!(name, "lwl" | "lwr" | "ldl" | "ldr" | "swl" | "swr" | "sdl" | "sdr");
            let align = if bytes == 16 || unit { !(u32::from(bytes) - 1) } else { 0xFFFF_FFFF };
            let access = Access { base: rs as u8, disp: w as u16 as i16, bytes, store, align };
            let (reads, writes) = match name {
                "pref" | "lwc1" | "swc1" => (bit(rs), 0),
                _ if store => (bit(rs) | bit(rt), 0),
                _ if unit => (bit(rs) | bit(rt), bit(rt)),
                _ => (bit(rs), bit(rt)),
            };
            return Info { class: Class::Memory, reads, writes, access: Some(access), likely: false };
        }
        match name {
            "beq" | "bne" => branch(bit(rs) | bit(rt), 0, false),
            "beql" | "bnel" => branch(bit(rs) | bit(rt), 0, true),
            "blez" | "bgtz" | "bltz" | "bgez" => branch(bit(rs), 0, false),
            "blezl" | "bgtzl" | "bltzl" | "bgezl" => branch(bit(rs), 0, true),
            "bltzal" | "bgezal" => branch(bit(rs), bit(31), false),
            "bltzall" | "bgezall" => branch(bit(rs), bit(31), true),
            "bc1f" | "bc1t" => branch(0, 0, false),
            "bc1fl" | "bc1tl" => branch(0, 0, true),
            "j" => branch(0, 0, false),
            "jal" => branch(0, bit(31), false),
            "jr" => branch(bit(rs), 0, false),
            "jalr" => branch(bit(rs), bit(rd), false),
            _ => Info { class: Class::Ender, reads: 0, writes: 0, access: None, likely: false },
        }
    }
}

/// Whether this host runs the tier: x86-64 Unix; elsewhere the EE is interpreted.
pub(crate) fn supported() -> bool {
    cfg!(all(target_arch = "x86_64", unix))
}

/// The folded cycles of a block, from the core's own issue model run from rest.
pub(crate) struct Cost {
    /// The cycles after step `i`.
    pub times: Vec<u32>,
    /// The cycles after the block with its branch taken; `times`' last with it not taken.
    pub taken: u32,
}

/// A base far from zero, so that every offset is positive.
const BASE: u64 = 1 << 40;

/// Cycles a line of instructions costs on average in fetch stalls, in sixteenths, folded into each block (Sapphire_Speed.md §66).
pub(crate) const FETCH_PER_LINE: u32 = 0;

/// The block's folded cycles, for the branch not taken and taken.
pub(crate) fn cost(plan: &Plan, options: Options) -> Cost {
    let run = |taken: bool| {
        let mut t = Timing::at_rest(BASE);
        let times: Vec<u32> = plan
            .steps
            .iter()
            .map(|s| {
                let mut i = ops::issue(ops::get(ops::index(s.word)), s.word);
                i.taken = taken && s.info.class == Class::Branch;
                i.single = options.single;
                i.blocking = options.blocking;
                (t.issue(i) + 1 - BASE) as u32
            })
            .collect();
        times
    };
    let lines = 1 + (plan.start + 4 * plan.steps.len() as u32 - 4) / 64 - plan.start / 64;
    let fetch = lines * FETCH_PER_LINE / 16;
    let mut times = run(false);
    let taken = run(true).last().copied().unwrap_or(0) + fetch;
    if let Some(l) = times.last_mut() {
        *l += fetch;
    }
    Cost { times, taken }
}

/// What a slow path tells its helper, laid in the block's data after its homes.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct Rec {
    /// The guest values to store to the core first, and to load back after, a bit a value id.
    pub dirty: [u64; 2],
    pub reload: [u64; 2],
    pub word: u32,
    pub pc: u32,
    /// The cycles after the block's entry the instruction issues at.
    pub time: u32,
    /// Its step, and its operation's table index.
    pub step: u16,
    pub index: u16,
    /// The record's offset in the data, where the homes are at 0.
    pub at: u32,
    /// In a delay slot: the place after it, or `NEXT`'s value with `NEXT_HELD`.
    pub next: u32,
    pub flags: u32,
    pub pad: u32,
}

/// `Rec::flags`: a delay slot, its outcome in the value `NEXT`, and an operation that reaches the board.
pub(crate) const IN_SLOT: u32 = 1;
pub(crate) const NEXT_HELD: u32 = 2;
pub(crate) const BOARD: u32 = 4;

impl Rec {
    pub(crate) fn bytes(&self) -> &[u8] {
        // SAFETY: a plain `repr(C)` value read as its bytes; the helper reads it back as `Rec` from 8-aligned data.
        unsafe { std::slice::from_raw_parts((self as *const Rec).cast::<u8>(), size_of::<Rec>()) }
    }
}

/// A value's home as a slow path reads it: 0 none, 1 to 7 the pool's registers, then the frame's slots from 8.
pub(crate) type Homes = [u8; lower::GUEST + 1];

/// The shared stubs, by id.
pub(crate) mod stub {
    pub const ENTER: u32 = 0;
    pub const DONE: u32 = 1;
    pub const OUT: u32 = 2;
    pub const OUT0: u32 = 3;
    pub const SLOW: u32 = 4;
    pub const FILL: u32 = 5;
}

/// The frame a block runs in below the six saved registers: slots at `[rsp + 8k]`, and rsp 16-byte aligned in the body.
pub(crate) const FRAME: i32 = 8 + 16 * 80;
pub(crate) const SLOTS: i32 = (FRAME - 8) / 8;

const SAVED: [emusen_assembler::Reg; 6] = [x64::RBX, x64::RBP, x64::R12, x64::R13, x64::R14, x64::R15];
const X_TIME: i32 = offset_of!(Context, time) as i32;
const X_PAGES: i32 = offset_of!(Context, pages) as i32;
const X_LINK: i32 = offset_of!(Context, link) as i32;
const X_FROM: i32 = offset_of!(Context, from) as i32;
const X_TO: i32 = offset_of!(Context, to) as i32;
const X_SAVED: i32 = offset_of!(Context, saved) as i32;
const PC: i32 = offset_of!(Ee, pc) as i32;
const NEXT_PC: i32 = offset_of!(Ee, next_pc) as i32;
const DELAY: i32 = offset_of!(Ee, delay) as i32;

/// The stubs: the entry from Rust, the return, the block's way out with its place, the slow path's save and return, and the helpers' trampoline.
pub(crate) fn stubs(slow: u64, fill: u64) -> Vec<Code> {
    use x64::{R12, R13, R14, R15, RAX, RDI, RDX, RSI, RSP};
    let one = |f: &dyn Fn(&mut X64)| {
        let mut a = X64::new();
        let e = a.label();
        a.bind(e);
        f(&mut a);
        a.finish(e)
    };
    let mut out = vec![one(&|a| {
        // Called from Rust: rdi the core, rsi the context, rdx the block's body.
        for x in SAVED {
            a.push(x);
        }
        a.alu(Op::Sub, Width::W64, RSP, RSP, Src::Imm(FRAME));
        a.mov(Width::W64, R14, RDI);
        a.mov(Width::W64, R15, RSI);
        a.load(8, false, R12, at(RSI, X_TIME));
        a.load(8, false, R13, at(RSI, X_PAGES));
        a.jump_reg(RDX);
    })];
    out.push(one(&|a| {
        a.store(8, R12, at(R15, X_TIME));
        a.alu(Op::Add, Width::W64, RSP, RSP, Src::Imm(FRAME));
        for x in SAVED.iter().rev() {
            a.pop(*x);
        }
        a.ret();
    }));
    let leave = |a: &mut X64| {
        a.store(4, RDI, at(R14, PC));
        a.store(4, RDI, at(R15, X_TO));
        a.lea(Width::W32, RAX, at(RDI, 4));
        a.store(4, RAX, at(R14, NEXT_PC));
        a.store_imm(1, 0, at(R14, DELAY));
        a.store(8, RSI, at(R15, X_LINK));
        a.store(4, RDX, at(R15, X_FROM));
        a.jump_stub(stub::DONE);
    };
    // edi the place, rsi the link's slot, edx the block's address.
    out.push(one(&|a| leave(a)));
    // edi the place, no link: an indirect jump's, as a block's table entry not yet compiled reaches it.
    out.push(one(&|a| {
        a.mov_imm(RSI, 0);
        a.mov(Width::W32, RDX, RDI);
        leave(a);
    }));
    out.push(one(&|a| {
        // Called with rdi the record and rsi why; the pool saved where the helper reads and writes it.
        for (k, x) in super::lower::POOL.iter().enumerate() {
            a.store(8, *x, at(R15, X_SAVED + 8 * k as i32));
        }
        a.store(8, R12, at(R15, X_TIME));
        a.lea(Width::W64, x64::RCX, at(RSP, 8));
        a.alu(Op::Sub, Width::W64, RSP, RSP, Src::Imm(8));
        a.mov(Width::W64, RDX, RSI);
        a.mov(Width::W64, RSI, RDI);
        a.mov(Width::W64, RDI, R15);
        a.mov_imm(RAX, slow as i64);
        a.call_reg(RAX);
        a.alu(Op::Add, Width::W64, RSP, RSP, Src::Imm(8));
        a.load(8, false, R12, at(R15, X_TIME));
        for (k, x) in super::lower::POOL.iter().enumerate() {
            a.load(8, false, *x, at(R15, X_SAVED + 8 * k as i32));
        }
        a.test(Width::W32, RAX, Src::Reg(RAX));
        let left = a.label();
        a.branch(emusen_assembler::Cond::Ne, left);
        a.ret();
        a.bind(left);
        a.alu(Op::Add, Width::W64, RSP, RSP, Src::Imm(8));
        a.jump_stub(stub::DONE);
    }));
    out.push(one(&|a| x64::trampoline(a, R15, fill)));
    out
}

/// The block's code, or none where the planner refuses its words.
pub(crate) fn compile(words: &[u32], start: u32, options: Options) -> Option<Code> {
    let plan = planner::plan(&R5900Isa, words, start).ok()?;
    lower::lower(&plan, &cost(&plan, options))
}

#[cfg(test)]
mod tests;
