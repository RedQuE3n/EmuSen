//! The project's S9 programs (Sapphire_Native.md §19.4): bare EE programs over every COP1 instruction and every class
//! of input, generated as assembly for `emusen-mips` in S6's form (`crate::s6::Program`). Each program first sets
//! Status.CU1; each case loads its operands into FPRs by MTC1 from constants, clears FCR31 by CTC1, runs the
//! instruction, and records the register it writes by MFC1 and FCR31 by CFC1, each sign-extended as MFC1 and CFC1
//! write them. The expected record comes from `manual`, chapter 5's pages transcribed apart from `src/ee/fpu.rs` and
//! `src/float.rs`.

#![allow(dead_code)]

pub mod manual;

use crate::s6::{self, AREA, Program};
use manual::FIXED;

/// The input classes (the gate of Sapphire_Plan.md §7, S9): both zeros, the smallest and largest normals, exponent
/// field 255 in the patterns IEEE 754 reads as infinities and NaNs, would-be denormals, exact halves and values next to
/// them, powers whose products overflow and underflow, and plain values.
pub const VALUES: [u32; 32] = [
    0x0000_0000, 0x8000_0000, // zeros
    0x0080_0000, 0x8080_0000, 0x0080_0001, // the smallest normals
    0x7FFF_FFFF, 0xFFFF_FFFF, 0x7F7F_FFFF, // the largest, and IEEE 754's largest
    0x7F80_0000, 0xFF80_0000, 0x7FC0_0000, 0x7F80_0001, // infinities and NaNs to IEEE 754
    0x0000_0001, 0x007F_FFFF, 0x8040_0000, // would-be denormals
    0x3F80_0000, 0xBF80_0000, 0x3F00_0000, 0x3FC0_0000, 0x4040_0000, 0x3FFF_FFFF, 0x3F80_0001, // ones and halves
    0x5F80_0000, 0x6000_0000, 0x1F80_0000, 0x2000_0000, // 2^64, 2^65, 2^-64, 2^-63
    0x4049_0FDB, 0x3DCC_CCCD, 0xC2F6_E979, 0x4B7F_FFFF, 0x4F00_0000, 0x3EAA_AAAB, // pi, 0.1, -123.456, ...
];
/// A smaller set for the instructions with three inputs and the pairwise picks.
pub const FEW: [u32; 12] = [
    0x0000_0000, 0x8000_0000, 0x0080_0000, 0x7FFF_FFFF, 0xFF80_0000, 0x0000_0001, 0x3F80_0000, 0xBFFF_FFFF, 0x5F80_0000, 0x1F80_0000, 0x4049_0FDB,
    0xC2F6_E979,
];
pub const ACCUMULATORS: [u32; 8] = [0x0000_0000, 0x8000_0000, 0x3F80_0000, 0xBF80_0000, 0x7FFF_FFFF, 0x7F80_0000, 0x0080_0000, 0x4049_0FDB];
/// Integers for CVT.S.W: the edges of 24 and 32 significant bits.
pub const INTEGERS: [u32; 12] =
    [0, 1, 0xFFFF_FFFF, 0x7FFF_FFFF, 0x8000_0000, 0x00FF_FFFF, 0x0100_0000, 0x0100_0001, 0x01FF_FFFF, 0x8000_0001, 0x1234_5678, 0xFF00_0001];
/// What every destination holds before the instruction, so that an unwritten one shows.
pub const SENTINEL: u32 = 0x5A5A_C3C3;

fn sext(v: u32) -> u64 {
    v as i32 as i64 as u64
}

/// A program with COP1 usable, the sentinel in t5, −0 in f30 and +0 in f31.
fn program(name: &'static str) -> Program {
    let mut p = Program::new(name);
    p.emit("mfc0 t0, Status");
    p.emit("lui t1, 0x2000");
    p.emit("or t0, t0, t1");
    p.emit("mtc0 t0, Status");
    p.emit("sync.p");
    p.constant("t5", SENTINEL as u64);
    p.constant("t0", 0x8000_0000);
    p.emit("mtc1 t0, f30");
    p.emit("mtc1 zero, f31");
    p
}
fn set(p: &mut Program, f: u32, v: u32) {
    p.constant("t0", v as u64);
    p.emit(format!("mtc1 t0, f{f}"));
}
fn record_fpr(p: &mut Program, f: u32, v: u32) {
    p.emit(format!("mfc1 t2, f{f}"));
    p.record("t2", Some(sext(v)));
}
fn record_fcr(p: &mut Program, v: u32) {
    p.emit("cfc1 t2, fcr31");
    p.record("t2", Some(sext(v)));
}

/// fd ← op(fs, ft) from a cleared FCR31: f1 and f2 the operands, f3 the destination.
fn binary(p: &mut Program, name: &str, a: u32, b: u32) {
    let o = manual::arith(name, a, b);
    set(p, 1, a);
    set(p, 2, b);
    p.emit("mtc1 t5, f3");
    p.emit("ctc1 zero, fcr31");
    p.emit(format!("{name} f3, f1, f2"));
    record_fpr(p, 3, o.value);
    record_fcr(p, o.fcr31);
}

/// ADD.S and SUB.S over every pair of the classes.
pub fn add() -> Program {
    let mut p = program("s9-add");
    for name in ["add.s", "sub.s"] {
        for &a in &VALUES {
            for &b in &VALUES {
                binary(&mut p, name, a, b);
            }
        }
    }
    p
}

/// MUL.S and DIV.S over every pair of the classes, each in both orders (the product is not commutative, D-26).
pub fn muldiv() -> Program {
    let mut p = program("s9-muldiv");
    for name in ["mul.s", "div.s"] {
        for &a in &VALUES {
            for &b in &VALUES {
                binary(&mut p, name, a, b);
            }
        }
    }
    p
}

/// ACC ← `acc` by ADDA.S with −0, which gives its first operand unchanged, either zero included.
fn preset_acc(p: &mut Program, acc: u32) {
    set(p, 4, acc);
    p.emit("adda.s f4, f30");
}
/// ACC read into f3 by MADD.S with a product of −0, which gives the accumulator unchanged (D-30); FCR31 after.
fn record_acc(p: &mut Program, acc: u32) {
    p.emit("madd.s f3, f30, f31");
    record_fpr(p, 3, acc);
}

/// The accumulator instructions: ADDA.S, SUBA.S and MULA.S over pairs, the four multiply-adds over accumulators and
/// pairs, with the register written and ACC after each.
pub fn accumulator() -> Program {
    let mut p = program("s9-acc");
    for name in ["adda.s", "suba.s", "mula.s"] {
        for &a in &FEW {
            for &b in &FEW {
                let o = manual::arith(name, a, b);
                set(&mut p, 1, a);
                set(&mut p, 2, b);
                p.emit("ctc1 zero, fcr31");
                p.emit(format!("{name} f1, f2"));
                record_fcr(&mut p, o.fcr31);
                record_acc(&mut p, o.value);
            }
        }
    }
    for name in ["madd.s", "msub.s", "madda.s", "msuba.s"] {
        for &acc in &ACCUMULATORS {
            for &a in &FEW {
                for &b in &FEW {
                    let o = manual::fused(name, acc, a, b);
                    preset_acc(&mut p, acc);
                    set(&mut p, 1, a);
                    set(&mut p, 2, b);
                    p.emit("mtc1 t5, f3");
                    p.emit("ctc1 zero, fcr31");
                    if name.ends_with("a.s") {
                        p.emit(format!("{name} f1, f2"));
                        record_fcr(&mut p, o.fcr31);
                        record_acc(&mut p, o.value);
                    } else {
                        p.emit(format!("{name} f3, f1, f2"));
                        record_fpr(&mut p, 3, o.value);
                        record_fcr(&mut p, o.fcr31);
                        record_acc(&mut p, acc);
                    }
                }
            }
        }
    }
    p
}

/// SQRT.S over the classes, its operand in ft (D-24) and fs holding another value; RSQRT.S over every pair.
pub fn roots() -> Program {
    let mut p = program("s9-roots");
    for &b in &VALUES {
        let o = manual::arith("sqrt.s", 0, b);
        set(&mut p, 1, 0x4049_0FDB);
        set(&mut p, 2, b);
        p.emit("mtc1 t5, f3");
        p.emit("ctc1 zero, fcr31");
        p.emit("sqrt.s f3, f2");
        record_fpr(&mut p, 3, o.value);
        record_fcr(&mut p, o.fcr31);
    }
    for &a in &VALUES {
        for &b in &VALUES {
            binary(&mut p, "rsqrt.s", a, b);
        }
    }
    p
}

/// fd ← op(fs) with FCR31 preset to O, U and their sticky flags, so that a clearing shows.
fn unary(p: &mut Program, name: &str, a: u32, value: u32, clears: bool) {
    let preset = manual::O | manual::U | manual::SO | manual::SU;
    set(p, 1, a);
    p.emit("mtc1 t5, f3");
    p.constant("t0", preset as u64);
    p.emit("ctc1 t0, fcr31");
    p.emit(format!("{name} f3, f1"));
    record_fpr(p, 3, value);
    record_fcr(p, FIXED | if clears { manual::SO | manual::SU } else { preset });
}

/// ABS.S, NEG.S, MOV.S, the conversions, MAX.S and MIN.S, the compares, the moves, the loads and stores, and the
/// branches.
pub fn misc() -> Program {
    let mut p = program("s9-misc");
    for &a in &VALUES {
        unary(&mut p, "abs.s", a, a & 0x7FFF_FFFF, true);
        unary(&mut p, "neg.s", a, a ^ 0x8000_0000, true);
        unary(&mut p, "mov.s", a, a, false);
        unary(&mut p, "cvt.w.s", a, manual::cvt_w(a), false);
    }
    for &w in INTEGERS.iter().chain(&VALUES) {
        unary(&mut p, "cvt.s.w", w, manual::cvt_s(w), false);
    }
    for name in ["max.s", "min.s"] {
        for &a in &FEW {
            for &b in &FEW {
                let preset = manual::O | manual::U;
                set(&mut p, 1, a);
                set(&mut p, 2, b);
                p.constant("t0", preset as u64);
                p.emit("ctc1 t0, fcr31");
                p.emit(format!("{name} f3, f1, f2"));
                record_fpr(&mut p, 3, manual::pick(name, a, b));
                record_fcr(&mut p, FIXED);
            }
        }
    }
    for name in ["c.f.s", "c.eq.s", "c.lt.s", "c.le.s"] {
        for &a in &FEW {
            for &b in &FEW {
                set(&mut p, 1, a);
                set(&mut p, 2, b);
                // C preset to the opposite of the answer, so that a write shows; the other flags kept.
                let c = manual::compare(name, a, b);
                let preset = manual::O | manual::SI | if c { 0 } else { manual::C };
                p.constant("t0", preset as u64);
                p.emit("ctc1 t0, fcr31");
                p.emit(format!("{name} f1, f2"));
                record_fcr(&mut p, FIXED | preset & !manual::C | if c { manual::C } else { 0 });
            }
        }
    }
    moves(&mut p);
    memory(&mut p);
    branches(&mut p);
    p
}

/// MFC1's sign extension, MTC1 of a register with bits above 31, CFC1 of every control register, CTC1's writable bits.
fn moves(p: &mut Program) {
    for v in [0x8000_0001u64, 0x7FFF_FFFF, 0xFFFF_FFFF_0000_0000 | 0x1234_5678, 0x0123_4567_89AB_CDEF] {
        p.constant("t0", v);
        p.emit("mtc1 t0, f5");
        p.constant("t2", s6::SENTINEL);
        p.emit("mfc1 t2, f5");
        p.record("t2", Some(sext(v as u32)));
    }
    p.emit("ctc1 zero, fcr31");
    for n in 0..32 {
        p.constant("t2", s6::SENTINEL);
        p.emit(format!("cfc1 t2, fcr{n}"));
        p.record("t2", Some(sext(if n >= 16 { FIXED } else { 0x2E30 })));
    }
    p.constant("t0", u64::MAX);
    p.emit("ctc1 t0, fcr31");
    record_fcr(p, 0x0183_C079);
    p.emit("ctc1 zero, fcr0");
    record_fcr(p, 0x0183_C079);
    p.emit("cfc1 t2, fcr0");
    p.record("t2", Some(0x2E30));
    p.constant("t0", 0x7E7C_3F87);
    p.emit("ctc1 t0, fcr31");
    record_fcr(p, FIXED | 0x7E7C_3F87 & 0x0083_C078);
    p.emit("ctc1 zero, fcr31");
}

/// LWC1 and SWC1 at each word of a quadword and at negative offsets.
fn memory(p: &mut Program) {
    let words = [0x3F80_0000u32, 0xDEAD_BEEF, 0x0000_1337, 0x7FFF_FFFF];
    p.emit(format!("lui t4, 0x{:x}", (AREA + 0x100) >> 16));
    p.emit(format!("ori t4, t4, 0x{:x}", (AREA + 0x100) & 0xFFFF));
    for (i, &w) in words.iter().enumerate() {
        p.constant("t0", w as u64);
        p.emit(format!("sw t0, {}(t4)", 4 * i));
    }
    for (i, &w) in words.iter().enumerate() {
        p.emit("mtc1 t5, f6");
        p.emit(format!("lwc1 f6, {}(t4)", 4 * i));
        record_fpr(p, 6, w);
    }
    p.emit("daddiu t6, t4, 16");
    p.emit("mtc1 t5, f6");
    p.emit("lwc1 f6, -4(t6)");
    record_fpr(p, 6, words[3]);
    for (i, &w) in words.iter().enumerate() {
        set(p, 7, !w);
        p.emit(format!("swc1 f7, {}(t4)", 4 * i));
    }
    p.emit("swc1 f30, -8(t6)");
    for (i, &w) in words.iter().enumerate() {
        p.emit(format!("lw t2, {}(t4)", 4 * i));
        p.record("t2", Some(sext(if i == 2 { 0x8000_0000 } else { !w })));
    }
}

/// BC1F, BC1T and their likely forms, C set and clear by C.EQ.S: the delay slot counts into t3, the path marks t4.
fn branches(p: &mut Program) {
    for name in ["bc1f", "bc1t", "bc1fl", "bc1tl"] {
        for c in [false, true] {
            set(p, 1, 0x3F80_0000);
            set(p, 2, if c { 0x3F80_0000 } else { 0x4000_0000 });
            p.emit("c.eq.s f1, f2");
            let taken = c == name.starts_with("bc1t");
            let likely = name.ends_with('l');
            let (target, join) = (p.label(), p.label());
            p.emit("daddu t3, zero, zero");
            p.emit("daddu t4, zero, zero");
            p.emit(format!("{name} {target}"));
            p.emit("addiu t3, t3, 1");
            p.emit(format!("beq zero, zero, {join}"));
            p.emit("addiu t4, t4, 2");
            p.place(&target);
            p.emit("addiu t4, t4, 4");
            p.place(&join);
            p.record("t3", Some(if taken || !likely { 1 } else { 0 }));
            p.record("t4", Some(if taken { 4 } else { 2 }));
        }
    }
}

/// The exceptions a COP1 instruction raises, through S6's handler: the unusable coprocessor with Status.CU1 clear,
/// LWC1's and SWC1's address errors, and the TX79's COP1 encodings the EE reserves (D-1).
pub fn exceptions() -> Program {
    let mut p = Program::new("s9-exc");
    p.emit("di");
    p.emit("mtc0 zero, Status");
    p.emit("sync.p");
    let words = emusen_mips::assemble(&s6::handler(), s6::VECTOR, &emusen_mips::Dialect::R5900).expect("the handler");
    p.emit("lui t9, 0xa000");
    for (i, w) in words.iter().enumerate() {
        p.emit(format!("lui t3, 0x{:x}", w >> 16));
        p.emit(format!("ori t3, t3, 0x{:x}", w & 0xFFFF));
        p.emit(format!("sw t3, 0x{:x}(t9)", 0x180 + 4 * i));
    }
    p.emit("sync.p");
    p.emit(format!("lui t4, 0x{:x}", AREA >> 16));
    const EXL: u32 = 2;
    const CU1: u32 = 1 << 29;
    for text in ["add.s f3, f1, f2", "mfc1 t2, f1", "ctc1 zero, fcr31", "lwc1 f1, 0(t4)", "swc1 f1, 0(t4)"] {
        let at = p.here();
        p.emit(text);
        s6::taken(&mut p, 11, at, false, 1, None, EXL);
    }
    let (at, after) = (p.here(), p.label());
    p.emit(format!("bc1t {after}"));
    s6::taken(&mut p, 11, at, false, 1, None, EXL);
    p.emit("nop");
    p.place(&after);
    p.emit("lui t0, 0x2000");
    p.emit("mtc0 t0, Status");
    p.emit("sync.p");
    for (text, store, k) in [("lwc1 f1, 2(t4)", false, 2u32), ("lwc1 f1, 1(t4)", false, 1), ("swc1 f1, 3(t4)", true, 3), ("swc1 f1, 6(t4)", true, 6)] {
        let at = p.here();
        p.emit(text);
        s6::taken(&mut p, if store { 5 } else { 4 }, at, false, 0, Some((AREA + k) as u64), EXL | CU1);
    }
    // CVT.D.S, ADD.D, C.UN.S, DMFC1 and LDC1 of the TX79: reserved on the EE.
    for w in [0x4600_0021u32, 0x4620_0000, 0x4600_0031, 0x4420_0000, 0xD480_0000] {
        let at = p.here();
        p.emit(format!(".word 0x{w:08x}"));
        s6::taken(&mut p, 10, at, false, 0, Some((AREA + 6) as u64), EXL | CU1);
    }
    p
}

/// Every program, in the order the referees run them.
pub fn all() -> Vec<Program> {
    vec![add(), muldiv(), accumulator(), roots(), misc(), exceptions()]
}
