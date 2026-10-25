//! COP1, the FPU (Sapphire_Plan.md §7, S9): one function per mnemonic as `emusen_mips::r5900` names it, each written
//! from its page in chapter 5 of the EE Core Instruction Set Manual and computed by `crate::float`, the one owner of the
//! PlayStation 2's single-precision format. The flags follow the pages and the EE Core User's Manual's Table 8-2: the
//! cause flags a page names are written, cleared where their condition does not hold, the sticky flags only set, and
//! the flags a page does not name kept (Sapphire_Disputes.md D-32). Sapphire_Native.md §19.2 records every reading.

use crate::float::{self, Outcome};

use super::ops::{Op, R, Stop, aligned, cond, likely, load_address, rd, rs, rt, sa, sext32, store_address_of};
use super::timing::{ACC as ACC_R, Category, FCR31 as FCR31_R, FPR0, Issue, rows};
use super::{Bus, Ee, Exception, status};

/// FCR0 as the console reads it: implementation 2Eh, revision 30h (ps2autotests' `cpu/ee_fpu/fcr` console text; the
/// EE Core User's Manual Table 8-1 gives the implementation number alone).
pub const FCR0: u32 = 0x2E30;
/// FCR31's bits fixed at 1, bit 24 and the rounding field's 01 (Figure 8-5, §8.7), and the bits CTC1 writes: C, the
/// cause flags I, D, O, U and the sticky flags (the same console text, `0183C079` after writing all ones).
pub const FCR31_FIXED: u32 = 0x0100_0001;
pub const FCR31_WRITABLE: u32 = 0x0083_C078;

/// FCR31's fields (Table 8-2).
pub mod flag {
    pub const C: u32 = 1 << 23;
    pub const I: u32 = 1 << 17;
    pub const D: u32 = 1 << 16;
    pub const O: u32 = 1 << 15;
    pub const U: u32 = 1 << 14;
    pub const SI: u32 = 1 << 6;
    pub const SD: u32 = 1 << 5;
    pub const SO: u32 = 1 << 4;
    pub const SU: u32 = 1 << 3;
}

/// The fields a COP1 instruction reads or writes, for the timing model.
pub(super) mod uses {
    pub const FS: u8 = 1;
    pub const FT: u8 = 2;
    pub const FD: u8 = 4;
    pub const ACC: u8 = 8;
    pub const FCR: u8 = 16;
    /// GPR[rt], the integer side of a move.
    pub const GRT: u8 = 32;
    /// GPR[base] of a load or store.
    pub const BASE: u8 = 64;
}
use uses::{ACC, BASE, FCR, FD, FS, FT, GRT};

fn fs(w: u32) -> usize {
    rd(w)
}
fn ft(w: u32) -> usize {
    rt(w)
}
fn fd(w: u32) -> usize {
    sa(w) as usize
}

/// Coprocessor 1 is usable when Status.CU1 is set, in every mode (EE Core User's Manual §3.2; Sapphire_Disputes.md D-8).
fn usable(e: &Ee) -> R {
    if e.status() & status::CU1 != 0 { Ok(()) } else { Err(Stop::Exception(Exception::Unusable(1))) }
}

/// O and U written from `o`, SO and SU set where they are.
fn exponent_flags(e: &mut Ee, o: Outcome) {
    let mut f = e.fcr31 & !(flag::O | flag::U);
    if o.overflow {
        f |= flag::O | flag::SO;
    }
    if o.underflow {
        f |= flag::U | flag::SU;
    }
    e.fcr31 = f;
}

/// I and D written, SI and SD set where they are.
fn divide_flags(e: &mut Ee, invalid: bool, divide: bool) {
    let mut f = e.fcr31 & !(flag::I | flag::D);
    if invalid {
        f |= flag::I | flag::SI;
    }
    if divide {
        f |= flag::D | flag::SD;
    }
    e.fcr31 = f;
}

/// FPR[fd] ← `f`(FPR[fs], FPR[ft]), O and U from the result.
fn arith(e: &mut Ee, w: u32, f: fn(u32, u32) -> Outcome) -> R {
    usable(e)?;
    let o = f(e.fpr[fs(w)], e.fpr[ft(w)]);
    e.fpr[fd(w)] = o.bits;
    exponent_flags(e, o);
    Ok(())
}
/// ACC ← `f`(FPR[fs], FPR[ft]), O and U from the result.
fn to_acc(e: &mut Ee, w: u32, f: fn(u32, u32) -> Outcome) -> R {
    usable(e)?;
    let o = f(e.fpr[fs(w)], e.fpr[ft(w)]);
    e.acc = o.bits;
    exponent_flags(e, o);
    Ok(())
}
fn add_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    arith(e, w, float::add)
}
fn sub_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    arith(e, w, float::sub)
}
fn mul_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    arith(e, w, float::mul)
}
fn adda_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    to_acc(e, w, float::add)
}
fn suba_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    to_acc(e, w, float::sub)
}
fn mula_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    to_acc(e, w, float::mul)
}

/// The multiply-add forms (MADD.S's Operation section, D-30): O and U from the sum, SU also from the product alone.
fn fused(e: &mut Ee, w: u32, subtract: bool, into_acc: bool) -> R {
    usable(e)?;
    let r = float::fused(e.acc, e.fpr[fs(w)], e.fpr[ft(w)], subtract);
    if into_acc {
        e.acc = r.value.bits;
    } else {
        e.fpr[fd(w)] = r.value.bits;
    }
    exponent_flags(e, r.value);
    if r.product_underflow {
        e.fcr31 |= flag::SU;
    }
    Ok(())
}
fn madd_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    fused(e, w, false, false)
}
fn msub_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    fused(e, w, true, false)
}
fn madda_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    fused(e, w, false, true)
}
fn msuba_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    fused(e, w, true, true)
}

/// DIV.S: I for 0/0, D for any other zero divisor; an overflow or underflow of the quotient raises no flag (its page).
fn div_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    usable(e)?;
    let q = float::div(e.fpr[fs(w)], e.fpr[ft(w)]);
    e.fpr[fd(w)] = q.value.bits;
    divide_flags(e, q.invalid, q.divide);
    Ok(())
}
/// SQRT.S of FPR[ft] (its page's operand field, D-24): I for a negative operand, D cleared.
fn sqrt_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    usable(e)?;
    let x = e.fpr[ft(w)];
    e.fpr[fd(w)] = float::sqrt(x).bits;
    divide_flags(e, x >> 31 == 1 && !float::is_zero(x), false);
    Ok(())
}
/// RSQRT.S: I for a negative FPR[ft], D for a zero one (its page; Table 8-5 differs, D-28).
fn rsqrt_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    usable(e)?;
    let q = float::rsqrt(e.fpr[fs(w)], e.fpr[ft(w)]);
    e.fpr[fd(w)] = q.value.bits;
    divide_flags(e, q.invalid, q.divide);
    Ok(())
}

/// ABS.S, NEG.S, MAX.S and MIN.S clear O and U; MOV.S and the conversions touch no flag.
fn unary(e: &mut Ee, w: u32, f: fn(u32) -> u32, clears: bool) -> R {
    usable(e)?;
    e.fpr[fd(w)] = f(e.fpr[fs(w)]);
    if clears {
        e.fcr31 &= !(flag::O | flag::U);
    }
    Ok(())
}
fn abs_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    unary(e, w, |x| x & float::MAX, true)
}
fn neg_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    unary(e, w, |x| x ^ float::SIGN, true)
}
fn mov_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    unary(e, w, |x| x, false)
}
fn cvt_w_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    unary(e, w, float::to_word, false)
}
fn cvt_s_w(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    unary(e, w, float::from_word, false)
}
fn pick(e: &mut Ee, w: u32, f: fn(u32, u32) -> u32) -> R {
    usable(e)?;
    e.fpr[fd(w)] = f(e.fpr[fs(w)], e.fpr[ft(w)]);
    e.fcr31 &= !(flag::O | flag::U);
    Ok(())
}
fn max_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    pick(e, w, float::max)
}
fn min_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    pick(e, w, float::min)
}

/// C.cond.S: C ← the comparison, exact, +0 equal to −0 (the pages); the would-be denormals compare as zero.
fn compare(e: &mut Ee, w: u32, f: fn(i64, i64) -> bool) -> R {
    usable(e)?;
    let c = f(float::order(e.fpr[fs(w)]), float::order(e.fpr[ft(w)]));
    e.fcr31 = if c { e.fcr31 | flag::C } else { e.fcr31 & !flag::C };
    Ok(())
}
fn c_f_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    compare(e, w, |_, _| false)
}
fn c_eq_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    compare(e, w, |a, b| a == b)
}
fn c_lt_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    compare(e, w, |a, b| a < b)
}
fn c_le_s(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    compare(e, w, |a, b| a <= b)
}

// The moves. CFC1 and CTC1 select FCR31 by bit 4 of the register number and FCR0 otherwise, as the console reads
// FCR1 to FCR15 as FCR0 and FCR16 to FCR30 as FCR31 (D-33); FCR0 is read-only.
fn mfc1(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    usable(e)?;
    e.set(rt(w), sext32(e.fpr[fs(w)]));
    Ok(())
}
fn mtc1(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    usable(e)?;
    e.fpr[fs(w)] = e.get(rt(w)) as u32;
    Ok(())
}
fn cfc1(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    usable(e)?;
    let v = if fs(w) & 16 != 0 { e.fcr31 } else { FCR0 };
    e.set(rt(w), sext32(v));
    Ok(())
}
fn ctc1(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    usable(e)?;
    if fs(w) & 16 != 0 {
        e.fcr31 = FCR31_FIXED | e.get(rt(w)) as u32 & FCR31_WRITABLE;
    }
    Ok(())
}
fn lwc1(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    usable(e)?;
    let a = load_address(e, w)?;
    aligned(a, 4, false)?;
    e.fpr[ft(w)] = e.read(b, a, 4)? as u32;
    Ok(())
}
fn swc1(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    usable(e)?;
    let v = e.fpr[ft(w)];
    let a = store_address_of(e, w, v)?;
    aligned(a, 4, true)?;
    e.write(b, a, 4, v as u128)?;
    Ok(())
}

// BC1F, BC1T and their likely forms test C (FCR31 bit 23).
fn bc1(e: &mut Ee, w: u32, pc: u32, on_true: bool, is_likely: bool) -> R {
    usable(e)?;
    let t = (e.fcr31 & flag::C != 0) == on_true;
    if is_likely { likely(e, w, pc, t) } else { cond(e, w, pc, t) }
}
fn bc1f(e: &mut Ee, _: &mut dyn Bus, w: u32, pc: u32) -> R {
    bc1(e, w, pc, false, false)
}
fn bc1t(e: &mut Ee, _: &mut dyn Bus, w: u32, pc: u32) -> R {
    bc1(e, w, pc, true, false)
}
fn bc1fl(e: &mut Ee, _: &mut dyn Bus, w: u32, pc: u32) -> R {
    bc1(e, w, pc, false, true)
}
fn bc1tl(e: &mut Ee, _: &mut dyn Bus, w: u32, pc: u32) -> R {
    bc1(e, w, pc, true, true)
}

const fn fop(name: &'static str, run: super::ops::Run, category: Category, row: usize, reads: u8, writes: u8) -> Op {
    Op { name, run, category, row, reads, writes, fpu: true }
}

use Category::{Branch, Cop1Move as Move, Cop1Operate as Operate};
use rows::{BRANCH, CVT, FADD, FDIV, FMADD, FMOVE, FMUL, FRSQRT, FSQRT, LWC1, MFC1, MTC1};

/// Every COP1 instruction by the decoder's name, with its category (Table 1-1: COP1 Move for the moves, loads and
/// stores, COP1 Operate for the rest) and its row of §2.12; the rows §2.12 does not give are Sapphire_Native.md §19.5's.
pub(super) static TABLE: &[Op] = &[
    fop("mfc1", mfc1, Move, MFC1, FS, GRT),
    fop("mtc1", mtc1, Move, MTC1, GRT, FS),
    fop("cfc1", cfc1, Move, MFC1, FCR, GRT),
    fop("ctc1", ctc1, Move, MTC1, GRT, FCR),
    fop("lwc1", lwc1, Move, LWC1, BASE, FT),
    fop("swc1", swc1, Move, LWC1, BASE | FT, 0),
    fop("bc1f", bc1f, Branch, BRANCH, FCR, 0),
    fop("bc1t", bc1t, Branch, BRANCH, FCR, 0),
    fop("bc1fl", bc1fl, Branch, BRANCH, FCR, 0),
    fop("bc1tl", bc1tl, Branch, BRANCH, FCR, 0),
    fop("add.s", add_s, Operate, FADD, FS | FT, FD | FCR),
    fop("sub.s", sub_s, Operate, FADD, FS | FT, FD | FCR),
    fop("mul.s", mul_s, Operate, FMUL, FS | FT, FD | FCR),
    fop("div.s", div_s, Operate, FDIV, FS | FT, FD | FCR),
    fop("sqrt.s", sqrt_s, Operate, FSQRT, FT, FD | FCR),
    fop("rsqrt.s", rsqrt_s, Operate, FRSQRT, FS | FT, FD | FCR),
    fop("abs.s", abs_s, Operate, FADD, FS, FD | FCR),
    fop("neg.s", neg_s, Operate, FADD, FS, FD | FCR),
    fop("mov.s", mov_s, Operate, FMOVE, FS, FD),
    fop("adda.s", adda_s, Operate, FADD, FS | FT, ACC | FCR),
    fop("suba.s", suba_s, Operate, FADD, FS | FT, ACC | FCR),
    fop("mula.s", mula_s, Operate, FMUL, FS | FT, ACC | FCR),
    fop("madd.s", madd_s, Operate, FMADD, FS | FT | ACC, FD | FCR),
    fop("msub.s", msub_s, Operate, FMADD, FS | FT | ACC, FD | FCR),
    fop("madda.s", madda_s, Operate, FMADD, FS | FT | ACC, ACC | FCR),
    fop("msuba.s", msuba_s, Operate, FMADD, FS | FT | ACC, ACC | FCR),
    fop("cvt.w.s", cvt_w_s, Operate, CVT, FS, FD),
    fop("cvt.s.w", cvt_s_w, Operate, CVT, FS, FD),
    fop("max.s", max_s, Operate, FADD, FS | FT, FD | FCR),
    fop("min.s", min_s, Operate, FADD, FS | FT, FD | FCR),
    fop("c.f.s", c_f_s, Operate, FADD, FS | FT, FCR),
    fop("c.eq.s", c_eq_s, Operate, FADD, FS | FT, FCR),
    fop("c.lt.s", c_lt_s, Operate, FADD, FS | FT, FCR),
    fop("c.le.s", c_le_s, Operate, FADD, FS | FT, FCR),
];

/// What the timing model is told of a COP1 instruction at `w`: its fields as resources.
pub(super) fn issue(op: &Op, w: u32) -> Issue {
    let bits = |u: u8| -> u128 {
        let mut m = 0u128;
        if u & FS != 0 {
            m |= 1 << (FPR0 + fs(w));
        }
        if u & FT != 0 {
            m |= 1 << (FPR0 + ft(w));
        }
        if u & FD != 0 {
            m |= 1 << (FPR0 + fd(w));
        }
        if u & ACC != 0 {
            m |= 1 << ACC_R;
        }
        if u & FCR != 0 {
            m |= 1 << FCR31_R;
        }
        if u & GRT != 0 && rt(w) != 0 {
            m |= 1 << rt(w);
        }
        if u & BASE != 0 && rs(w) != 0 {
            m |= 1 << rs(w);
        }
        m
    };
    Issue { reads: bits(op.reads), writes: bits(op.writes), ..Issue::new(op.category, op.row) }
}
