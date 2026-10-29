//! E0 of the tailored recompiler, behind the `baseline` feature (EmuSen_Recompiler.md §14): the EE's blocks of integer
//! ALU instructions, aligned loads and stores and branches, planned by `emusen-mips-planner`, lowered straight to x86-64
//! through `emusen-assembler`, in the static form: every issue cycle folded to an offset from the entry's last issue, valid
//! where the entry's model is at rest for what the block reads, and the block's Cranelift function otherwise.

mod lower;

use std::mem::offset_of;

use emusen_assembler::x64::{self, X64};
use emusen_assembler::{Assembler, Code, Op, Src, Width};
use emusen_mips_planner::{self as planner, Access, Class, Info, Isa, Plan};

use super::super::ops;
use super::super::timing::{SNAP_READY, Snap, Timing};
use super::{Context, Options, shape};
use crate::ee::Ee;

pub(crate) use lower::lower;

/// The instructions E0's slice compiles (§14.2): the integer ALU but the multipliers, HI/LO and SA, and the traps.
const PLAIN: &[&str] = &[
    "add", "addu", "sub", "subu", "addi", "addiu", "dadd", "daddu", "dsub", "dsubu", "daddi", "daddiu", "and", "or", "xor", "nor", "andi",
    "ori", "xori", "lui", "slt", "sltu", "slti", "sltiu", "movz", "movn", "sll", "srl", "sra", "sllv", "srlv", "srav", "dsll", "dsrl",
    "dsra", "dsll32", "dsrl32", "dsra32", "dsllv", "dsrlv", "dsrav", "sync", "sync.p",
];
/// The aligned loads and stores; the unaligned ones and PREF are E1's.
const MEMORY: &[&str] = &["lb", "lbu", "lh", "lhu", "lw", "lwu", "ld", "lq", "sb", "sh", "sw", "sd", "sq"];

fn bit(r: u32) -> u32 {
    if r == 0 { 0 } else { 1 << r }
}

/// The R5900's table for the planner, E0's slice; everything else ends a block.
pub(crate) struct R5900Isa;

impl Isa for R5900Isa {
    const MAX: usize = shape::MAX;

    fn info(&self, w: u32) -> Info {
        let (rs, rt, rd) = (w >> 21 & 31, w >> 16 & 31, w >> 11 & 31);
        let name = shape::name(w);
        let plain = |reads, writes| Info { class: Class::Plain, reads, writes, access: None, likely: false };
        let branch = |reads, writes, likely| Info { class: Class::Branch, reads, writes, access: None, likely };
        if PLAIN.contains(&name) {
            return match name {
                "sync" | "sync.p" => plain(0, 0),
                "lui" => plain(0, bit(rt)),
                "addi" | "addiu" | "daddi" | "daddiu" | "andi" | "ori" | "xori" | "slti" | "sltiu" => plain(bit(rs), bit(rt)),
                "sll" | "srl" | "sra" | "dsll" | "dsrl" | "dsra" | "dsll32" | "dsrl32" | "dsra32" => plain(bit(rt), bit(rd)),
                "movz" | "movn" => plain(bit(rs) | bit(rt) | bit(rd), bit(rd)),
                _ => plain(bit(rs) | bit(rt), bit(rd)),
            };
        }
        if MEMORY.contains(&name) {
            let (bytes, store) = match name {
                "lb" | "lbu" => (1, false),
                "lh" | "lhu" => (2, false),
                "lw" | "lwu" => (4, false),
                "ld" => (8, false),
                "lq" => (16, false),
                "sb" => (1, true),
                "sh" => (2, true),
                "sw" => (4, true),
                "sd" => (8, true),
                _ => (16, true),
            };
            let access = Access { base: rs as u8, disp: w as u16 as i16, bytes, store, align: if bytes == 16 { 0xFFFF_FFF0 } else { 0xFFFF_FFFF } };
            let (reads, writes) = if store { (bit(rs) | bit(rt), 0) } else { (bit(rs), bit(rt)) };
            return Info { class: Class::Memory, reads, writes, access: Some(access), likely: false };
        }
        match name {
            "beq" | "bne" => branch(bit(rs) | bit(rt), 0, false),
            "beql" | "bnel" => branch(bit(rs) | bit(rt), 0, true),
            "blez" | "bgtz" | "bltz" | "bgez" => branch(bit(rs), 0, false),
            "blezl" | "bgtzl" | "bltzl" | "bgezl" => branch(bit(rs), 0, true),
            "bltzal" | "bgezal" => branch(bit(rs), bit(31), false),
            "bltzall" | "bgezall" => branch(bit(rs), bit(31), true),
            "j" => branch(0, 0, false),
            "jal" => branch(0, bit(31), false),
            "jr" => branch(bit(rs), 0, false),
            "jalr" => branch(bit(rs), bit(rd), false),
            _ => Info { class: Class::Ender, reads: 0, writes: 0, access: None, likely: false },
        }
    }
}

/// Whether the dispatcher runs the baseline's code: `EMUSEN_SAPPHIRE_BASELINE=1`.
pub(crate) fn dispatched() -> bool {
    use std::sync::atomic::Ordering::Relaxed;
    match DISPATCHED.load(Relaxed) {
        0 => {
            let on = std::env::var("EMUSEN_SAPPHIRE_BASELINE").is_ok_and(|v| v == "1");
            DISPATCHED.store(if on { 2 } else { 1 }, Relaxed);
            on
        }
        v => v == 2,
    }
}

/// 0 until the environment is read, then 1 off and 2 on; `set_dispatched` for the tests.
static DISPATCHED: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// The baseline dispatched or not from now on, whatever the environment says, for blocks shaped after the call.
pub fn set_dispatched(on: bool) {
    DISPATCHED.store(if on { 2 } else { 1 }, std::sync::atomic::Ordering::Relaxed);
}

/// Whether the baseline compiles a block of these words with these options: every word in the slice, at least two, and
/// none of the measurement options.
pub(crate) fn covers(words: &[u32], options: Options) -> bool {
    !options.verify && !options.fixed_cost && !options.spill && options.memory && words.len() >= 2 && planner::plan(&R5900Isa, words, 0).is_ok()
}

/// The issue cycles of a static block as offsets from the entry's last issue, and the model after each step.
pub(crate) struct Static {
    /// The cycles after step `i`, as `Context::times` holds them, less the entry's last issue.
    pub times: [u16; shape::MAX],
    /// The model after `n` steps, for a branch not taken and taken (equal before the branch issues).
    pub snaps: Vec<[Snap; 2]>,
    /// The resources the block reads from its entry, which the entry's test finds ready.
    pub exposed: u64,
}

/// A base far from zero, so that every offset is positive.
const BASE: u64 = 1 << 40;

/// The static form's cycles: the core's own model run from rest, for the branch not taken and taken; none where the two
/// disagree on a cycle or an offset does not fit (EmuSen_Recompiler.md §14.3).
pub(crate) fn static_times(plan: &Plan, options: Options) -> Option<Static> {
    let n = plan.steps.len();
    let mut runs: [(Vec<u64>, Vec<Snap>); 2] = Default::default();
    let mut exposed = 0u64;
    for (taken, run) in runs.iter_mut().enumerate() {
        let mut t = Timing::at_rest(BASE);
        let (mut written, mut unpaired) = (0u64, 0u8);
        run.1.push(Snap::default());
        for s in &plan.steps {
            let mut i = ops::issue(ops::get(ops::index(s.word)), s.word);
            i.taken = taken == 1 && s.info.class == Class::Branch;
            i.single = options.single;
            i.blocking = options.blocking;
            let (reads, writes) = (i.reads as u64, i.writes as u64);
            if i.reads >> 64 != 0 || i.writes >> 64 != 0 {
                return None;
            }
            exposed |= reads & !written;
            let at = t.issue(i);
            written |= writes;
            unpaired += u8::from(!t.paired());
            run.0.push(at + 1);
            run.1.push(t.snap(BASE, written & ((1 << SNAP_READY) - 1), unpaired)?);
        }
    }
    if runs[0].0 != runs[1].0 {
        return None;
    }
    let mut times = [0u16; shape::MAX];
    for (k, &c) in runs[0].0.iter().enumerate() {
        times[k] = u16::try_from(c - BASE).ok()?;
    }
    let snaps = (0..=n).map(|k| [runs[0].1[k], runs[1].1[k]]).collect();
    Some(Static { times, snaps, exposed })
}

/// An exit's place: where the core stands after its steps.
pub(crate) mod mode {
    /// At `pc`, outside a delay slot.
    pub const LINE: u8 = 0;
    /// At the address the code passes, after a branch's slot.
    pub const JUMPED: u8 = 1;
    /// At the delay slot `pc`, the branch's outcome passed by the code.
    pub const SLOT: u8 = 2;
    /// At the delay slot `pc`, the outcome `next`.
    pub const SLOT_KNOWN: u8 = 3;
}
pub(crate) const SYNC: u8 = 1;
pub(crate) const BRANCHED: u8 = 2;

/// What an exit hands the shared exit's helpers: its steps and their cycles, the last load or store's steps and the undo
/// entries, its place, and the model after its steps for the branch not taken and taken (EmuSen_Recompiler.md §14.3).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct Rec {
    pub steps: u8,
    pub base: u8,
    pub logged: u8,
    pub flags: u8,
    pub mode: u8,
    pub pad: [u8; 3],
    pub pc: u32,
    pub next: u32,
    /// The address after the branch's slot: a branch is taken where the place it leads to is not this.
    pub after: u32,
    pub pad2: u32,
    pub times: [u16; shape::MAX],
    pub snap: [Snap; 2],
}

impl Rec {
    pub(crate) fn bytes(&self) -> &[u8] {
        // SAFETY: a plain `repr(C)` value read as its bytes; the helpers read it back as `Rec` from 8-aligned data.
        unsafe { std::slice::from_raw_parts((self as *const Rec).cast::<u8>(), size_of::<Rec>()) }
    }

    /// The place, the next address, the delay flag, and whether the branch was taken, `x` the value the code passed.
    pub(crate) fn place(&self, x: u64) -> (u32, u32, bool, bool) {
        let x = x as u32;
        let (pc, next, delay) = match self.mode {
            mode::LINE => (self.pc, self.pc.wrapping_add(4), false),
            mode::JUMPED => (x, x.wrapping_add(4), false),
            mode::SLOT => (self.pc, x, true),
            _ => (self.pc, self.next, true),
        };
        let flow = if delay { next } else { pc };
        (pc, next, delay, self.flags & BRANCHED != 0 && flow != self.after)
    }
}

impl Ee {
    /// The baseline's commit of an exit's steps once every one stands: the model from the record, Random and the group,
    /// then Count, the cycles seen, IP7, the instructions issued, the UCAB and the place as the shared exit commits them.
    pub(crate) fn commit_static(&mut self, rec: &Rec, x: u64) {
        use super::super::cop0::{RANDOM, WIRED};
        let (pc, next, delay, taken) = rec.place(x);
        let s = &rec.snap[usize::from(taken)];
        let base = self.timing.last();
        self.timing.apply(base, s);
        let wired = self.cop0[WIRED] & 63;
        let mut r = self.cop0[RANDOM];
        for _ in 0..s.unpaired {
            r = if r <= wired || r > 47 { 47 } else { r - 1 };
        }
        self.cop0[RANDOM] = r;
        self.group = if s.paired != 0 { 2 } else { 1 };
        let word = super::commit_word(u32::from(rec.steps), 0, 0, rec.flags & SYNC != 0, delay, None);
        let end = self.timing.last() + 1;
        self.commit_exit(word, u64::from(pc) | u64::from(next) << 32, end);
    }
}

/// The shared stubs, by id.
pub(crate) mod stub {
    pub const ENTER: u32 = 0;
    pub const DONE: u32 = 1;
    pub const ROLLBACK: u32 = 2;
    pub const DONE0: u32 = 3;
    pub const EXIT: u32 = 4;
    pub const LEAVE: u32 = 5;
    pub const COMMIT: u32 = 6;
    pub const FILL: u32 = 7;
}

/// The frame a block runs in below the six saved registers: slots at `[rsp + 8k]`, and rsp 16-byte aligned in the body.
pub(crate) const FRAME: i32 = 8 + 16 * 80;
pub(crate) const SLOTS: i32 = (FRAME - 8) / 8;

pub(crate) const X_STEPS: i32 = offset_of!(Context, steps) as i32;
pub(crate) const X_PAGES: i32 = offset_of!(Context, pages) as i32;
pub(crate) const X_STATIC: i32 = offset_of!(Context, entered_static) as i32;
const LAST: i32 = (offset_of!(Ee, timing) as i32) + super::super::timing::layout::LAST;
const SAVED: [emusen_assembler::Reg; 6] = [x64::RBX, x64::RBP, x64::R12, x64::R13, x64::R14, x64::R15];

/// The stubs: the entry (registers saved, the fixed ones set, the static entries counted), the three returns, and the
/// helper convention's trampolines to the exit, the leave, the commit and the translation cache's fill.
pub(crate) fn stubs(helpers: [u64; 4]) -> Vec<Code> {
    let mut out = Vec::new();
    let one = |f: &dyn Fn(&mut X64)| {
        let mut a = X64::new();
        let e = a.label();
        a.bind(e);
        f(&mut a);
        a.finish(e)
    };
    out.push(one(&|a| {
        // Entered by a jump from a block's entry, rdi the core, rsi the context, r11 the block's body.
        for x in SAVED {
            a.push(x);
        }
        a.alu(Op::Sub, Width::W64, x64::RSP, x64::RSP, Src::Imm(FRAME));
        a.mov(Width::W64, x64::R14, x64::RDI);
        a.mov(Width::W64, x64::R15, x64::RSI);
        a.load(8, false, x64::R12, emusen_assembler::at(x64::RDI, LAST));
        a.load(8, false, x64::R13, emusen_assembler::at(x64::RSI, X_PAGES));
        a.alu_mem(Op::Add, 8, emusen_assembler::at(x64::RSI, X_STATIC), Src::Imm(1));
        a.jump_reg(x64::R11);
    }));
    let ret = |a: &mut X64, exit: u32, zero_steps: bool| {
        if zero_steps {
            a.store_imm(4, 0, emusen_assembler::at(x64::R15, X_STEPS));
        }
        a.mov_imm(x64::RAX, i64::from(exit));
        a.alu(Op::Add, Width::W64, x64::RSP, x64::RSP, Src::Imm(FRAME));
        for x in SAVED.iter().rev() {
            a.pop(*x);
        }
        a.ret();
    };
    out.push(one(&|a| ret(a, super::DONE, false)));
    out.push(one(&|a| ret(a, super::ROLLBACK, false)));
    out.push(one(&|a| ret(a, super::DONE, true)));
    for h in helpers {
        out.push(one(&|a| x64::trampoline(a, x64::R15, h)));
    }
    out
}

/// The block's code, or none outside the slice; `prototype` drops the entry's test and the undo log, for the byte count on
/// the static prototype's terms only (§14.1).
pub(crate) fn compile(words: &[u32], start: u32, options: Options, prototype: bool) -> Option<(Code, u32)> {
    if !covers(words, options) {
        return None;
    }
    let plan = planner::plan(&R5900Isa, words, start).ok()?;
    let times = static_times(&plan, options)?;
    lower(&plan, &times, prototype)
}

#[cfg(test)]
mod tests;
