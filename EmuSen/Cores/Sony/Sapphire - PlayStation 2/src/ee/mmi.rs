//! The multimedia instructions (Sapphire_Plan.md §7, S8): the parallel operations on the 128-bit GPRs, HI and LO,
//! PLZCW and QFSRV, one function per mnemonic as `emusen_mips::r5900` names it, each written from its page in chapter 3
//! of the EE Core Instruction Set Manual. Where a page's Operation section and its Description differ, or its
//! pseudocode cannot be read literally, Sapphire_Native.md §18.2 records the reading taken and the dispute it opened.

use super::ops::{Op, R, op, quotient_signed, quotient_unsigned, rd, rs, rt, sa, sext32, uses};
use super::timing::{Category, rows};
use super::{Bus, Ee};

use uses::{HILO0 as H0, HILO1 as H1, RD, RS, RT, SA as S};

/// Lane `i` of `w` bits of a 128-bit value, lane 0 the least significant.
fn lane(v: u128, w: u32, i: u32) -> u64 {
    (v >> (i * w)) as u64 & ones(w)
}
fn ones(w: u32) -> u64 {
    if w == 64 { u64::MAX } else { (1 << w) - 1 }
}
/// A lane read as a signed integer.
fn signed(x: u64, w: u32) -> i64 {
    ((x << (64 - w)) as i64) >> (64 - w)
}
/// The 128-bit value whose lane `i` of `w` bits is `f(i)`.
fn build(w: u32, f: impl Fn(u32) -> u64) -> u128 {
    (0..128 / w).fold(0u128, |r, i| r | ((f(i) & ones(w)) as u128) << (i * w))
}
/// The lanes of `a` and `b` combined pairwise by `f`.
fn zip(a: u128, b: u128, w: u32, f: impl Fn(u32, u64, u64) -> u64) -> u128 {
    build(w, |i| f(i, lane(a, w, i), lane(b, w, i)))
}
/// A signed result clamped to the range of a `w`-bit lane, as the saturating pages' Descriptions give it.
fn clamp_signed(v: i64, w: u32) -> u64 {
    let (lo, hi) = (-(1i64 << (w - 1)), (1i64 << (w - 1)) - 1);
    v.clamp(lo, hi) as u64
}
fn with_word(v: u128, i: u32, x: u32) -> u128 {
    v & !((u32::MAX as u128) << (32 * i)) | (x as u128) << (32 * i)
}

/// GPR[rd] ← f(GPR[rs], GPR[rt]) over lanes of `w` bits.
fn lanes(e: &mut Ee, word: u32, w: u32, f: impl Fn(u32, u64, u64) -> u64) -> R {
    let v = zip(e.gpr[rs(word)], e.gpr[rt(word)], w, f);
    e.set_quad(rd(word), v);
    Ok(())
}

// Parallel add and subtract: wrapping, signed saturation and unsigned saturation.
fn paddb(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    lanes(e, w, 8, |_, a, b| a.wrapping_add(b))
}
fn paddh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    lanes(e, w, 16, |_, a, b| a.wrapping_add(b))
}
fn paddw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    lanes(e, w, 32, |_, a, b| a.wrapping_add(b))
}
fn psubb(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    lanes(e, w, 8, |_, a, b| a.wrapping_sub(b))
}
fn psubh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    lanes(e, w, 16, |_, a, b| a.wrapping_sub(b))
}
fn psubw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    lanes(e, w, 32, |_, a, b| a.wrapping_sub(b))
}
fn adds(e: &mut Ee, word: u32, w: u32) -> R {
    lanes(e, word, w, |_, a, b| clamp_signed(signed(a, w) + signed(b, w), w))
}
fn subs(e: &mut Ee, word: u32, w: u32) -> R {
    lanes(e, word, w, |_, a, b| clamp_signed(signed(a, w) - signed(b, w), w))
}
fn paddsb(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    adds(e, w, 8)
}
fn paddsh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    adds(e, w, 16)
}
fn paddsw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    adds(e, w, 32)
}
fn psubsb(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    subs(e, w, 8)
}
fn psubsh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    subs(e, w, 16)
}
fn psubsw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    subs(e, w, 32)
}
fn addu(e: &mut Ee, word: u32, w: u32) -> R {
    lanes(e, word, w, |_, a, b| (a + b).min(ones(w)))
}
fn subu(e: &mut Ee, word: u32, w: u32) -> R {
    lanes(e, word, w, |_, a, b| a.saturating_sub(b))
}
fn paddub(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    addu(e, w, 8)
}
fn padduh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    addu(e, w, 16)
}
fn padduw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    addu(e, w, 32)
}
fn psubub(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    subu(e, w, 8)
}
fn psubuh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    subu(e, w, 16)
}
fn psubuw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    subu(e, w, 32)
}
/// PADSBH: the low four halfwords subtract, the high four add.
fn padsbh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    lanes(e, w, 16, |i, a, b| if i < 4 { a.wrapping_sub(b) } else { a.wrapping_add(b) })
}

// Compare, maximum, minimum and absolute value, on signed lanes but for equality.
fn pceqb(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    lanes(e, w, 8, |_, a, b| if a == b { u64::MAX } else { 0 })
}
fn pceqh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    lanes(e, w, 16, |_, a, b| if a == b { u64::MAX } else { 0 })
}
fn pceqw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    lanes(e, w, 32, |_, a, b| if a == b { u64::MAX } else { 0 })
}
fn greater(e: &mut Ee, word: u32, w: u32) -> R {
    lanes(e, word, w, |_, a, b| if signed(a, w) > signed(b, w) { u64::MAX } else { 0 })
}
fn pcgtb(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    greater(e, w, 8)
}
fn pcgth(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    greater(e, w, 16)
}
fn pcgtw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    greater(e, w, 32)
}
fn max(e: &mut Ee, word: u32, w: u32) -> R {
    lanes(e, word, w, |_, a, b| if signed(a, w) > signed(b, w) { a } else { b })
}
fn min(e: &mut Ee, word: u32, w: u32) -> R {
    lanes(e, word, w, |_, a, b| if signed(a, w) > signed(b, w) { b } else { a })
}
fn pmaxh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    max(e, w, 16)
}
fn pmaxw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    max(e, w, 32)
}
fn pminh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    min(e, w, 16)
}
fn pminw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    min(e, w, 32)
}
/// PABSH and PABSW: the most negative value gives the most positive, as the Descriptions say.
fn abs(e: &mut Ee, word: u32, w: u32) -> R {
    lanes(e, word, w, |_, _, b| clamp_signed(signed(b, w).abs(), w))
}
fn pabsh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    abs(e, w, 16)
}
fn pabsw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    abs(e, w, 32)
}

// Logic on all 128 bits.
fn pand(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let v = e.gpr[rs(w)] & e.gpr[rt(w)];
    e.set_quad(rd(w), v);
    Ok(())
}
fn por(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let v = e.gpr[rs(w)] | e.gpr[rt(w)];
    e.set_quad(rd(w), v);
    Ok(())
}
fn pxor(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let v = e.gpr[rs(w)] ^ e.gpr[rt(w)];
    e.set_quad(rd(w), v);
    Ok(())
}
fn pnor(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let v = !(e.gpr[rs(w)] | e.gpr[rt(w)]);
    e.set_quad(rd(w), v);
    Ok(())
}

// Shifts. The halfword forms by sa3..0, the word forms by sa; the variable forms shift words 0 and 2 by GPR[rs]4..0
// and GPR[rs]68..64 and sign-extend each 32-bit result into its doubleword.
fn shift(e: &mut Ee, word: u32, w: u32, f: impl Fn(u64) -> u64) -> R {
    let v = build(w, |i| f(lane(e.gpr[rt(word)], w, i)));
    e.set_quad(rd(word), v);
    Ok(())
}
fn psllh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let s = sa(w) & 15;
    shift(e, w, 16, |x| x << s)
}
fn psrlh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let s = sa(w) & 15;
    shift(e, w, 16, |x| x >> s)
}
fn psrah(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let s = sa(w) & 15;
    shift(e, w, 16, |x| (signed(x, 16) >> s) as u64)
}
fn psllw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let s = sa(w);
    shift(e, w, 32, |x| x << s)
}
fn psrlw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let s = sa(w);
    shift(e, w, 32, |x| x >> s)
}
fn psraw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let s = sa(w);
    shift(e, w, 32, |x| (signed(x, 32) >> s) as u64)
}
fn variable(e: &mut Ee, word: u32, f: impl Fn(u32, u32) -> u32) -> R {
    let (t, s) = (e.gpr[rt(word)], e.gpr[rs(word)]);
    let v = build(64, |d| sext32(f(lane(t, 64, d) as u32, lane(s, 64, d) as u32 & 31)));
    e.set_quad(rd(word), v);
    Ok(())
}
fn psllvw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    variable(e, w, |x, s| x << s)
}
fn psrlvw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    variable(e, w, |x, s| x >> s)
}
fn psravw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    variable(e, w, |x, s| ((x as i32) >> s) as u32)
}

// Interleave, pack, copy and rearrange: each lane of GPR[rd] taken from a lane of GPR[rs] or GPR[rt] as the pages'
// Operation sections list them.
fn interleave(e: &mut Ee, word: u32, w: u32, from: u32) -> R {
    let (s, t) = (e.gpr[rs(word)], e.gpr[rt(word)]);
    let v = build(2 * w, |i| lane(s, w, from + i) << w | lane(t, w, from + i));
    e.set_quad(rd(word), v);
    Ok(())
}
fn pextlb(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    interleave(e, w, 8, 0)
}
fn pextub(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    interleave(e, w, 8, 8)
}
fn pextlh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    interleave(e, w, 16, 0)
}
fn pextuh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    interleave(e, w, 16, 4)
}
fn pextlw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    interleave(e, w, 32, 0)
}
fn pextuw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    interleave(e, w, 32, 2)
}
/// PPACB, PPACH, PPACW: the low half of each lane of `2w` bits, GPR[rt]'s into the low 64 bits and GPR[rs]'s above.
fn pack(e: &mut Ee, word: u32, w: u32) -> R {
    let (s, t) = (e.gpr[rs(word)], e.gpr[rt(word)]);
    let half = 64 / w;
    let v = build(w, |i| if i < half { lane(t, 2 * w, i) } else { lane(s, 2 * w, i - half) });
    e.set_quad(rd(word), v);
    Ok(())
}
fn ppacb(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    pack(e, w, 8)
}
fn ppach(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    pack(e, w, 16)
}
fn ppacw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    pack(e, w, 32)
}
/// PINTH: GPR[rs]'s high halfwords above GPR[rt]'s low ones, word by word.
fn pinth(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let (s, t) = (e.gpr[rs(w)], e.gpr[rt(w)]);
    let v = build(32, |i| lane(s, 16, 4 + i) << 16 | lane(t, 16, i));
    e.set_quad(rd(w), v);
    Ok(())
}
/// PINTEH: the even halfwords of both, GPR[rs]'s above.
fn pinteh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let (s, t) = (e.gpr[rs(w)], e.gpr[rt(w)]);
    let v = build(32, |i| lane(s, 16, 2 * i) << 16 | lane(t, 16, 2 * i));
    e.set_quad(rd(w), v);
    Ok(())
}
fn pcpyld(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let v = e.gpr[rs(w)] << 64 | e.gpr[rt(w)] & u64::MAX as u128;
    e.set_quad(rd(w), v);
    Ok(())
}
fn pcpyud(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let v = e.gpr[rt(w)] >> 64 << 64 | e.gpr[rs(w)] >> 64;
    e.set_quad(rd(w), v);
    Ok(())
}
/// GPR[rd]'s lanes of `w` bits taken from GPR[rt]'s lanes `from`.
fn permute(e: &mut Ee, word: u32, w: u32, from: &[u32]) -> R {
    let t = e.gpr[rt(word)];
    let v = build(w, |i| lane(t, w, from[i as usize]));
    e.set_quad(rd(word), v);
    Ok(())
}
fn pcpyh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    permute(e, w, 16, &[0, 0, 0, 0, 4, 4, 4, 4])
}
fn pexch(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    permute(e, w, 16, &[0, 2, 1, 3, 4, 6, 5, 7])
}
fn pexcw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    permute(e, w, 32, &[0, 2, 1, 3])
}
fn pexeh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    permute(e, w, 16, &[2, 1, 0, 3, 6, 5, 4, 7])
}
fn pexew(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    permute(e, w, 32, &[2, 1, 0, 3])
}
fn prevh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    permute(e, w, 16, &[3, 2, 1, 0, 7, 6, 5, 4])
}
fn prot3w(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    permute(e, w, 32, &[1, 2, 0, 3])
}
/// PEXT5: each word's 1-5-5-5 halfword spread to 8-8-8-8, the low three bits of each colour and seven of alpha zero.
fn pext5(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let t = e.gpr[rt(w)];
    let v = build(32, |i| {
        let h = lane(t, 32, i);
        (h >> 15 & 1) << 31 | (h >> 10 & 31) << 19 | (h >> 5 & 31) << 11 | (h & 31) << 3
    });
    e.set_quad(rd(w), v);
    Ok(())
}
/// PPAC5: each 8-8-8-8 word cut to 1-5-5-5 in its low halfword, the high halfword zero.
fn ppac5(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let t = e.gpr[rt(w)];
    let v = build(32, |i| {
        let x = lane(t, 32, i);
        (x >> 31 & 1) << 15 | (x >> 19 & 31) << 10 | (x >> 11 & 31) << 5 | (x >> 3 & 31)
    });
    e.set_quad(rd(w), v);
    Ok(())
}

// Moves between the GPRs and the whole of HI and LO.
fn pmfhi(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let v = e.hi;
    e.set_quad(rd(w), v);
    Ok(())
}
fn pmflo(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let v = e.lo;
    e.set_quad(rd(w), v);
    Ok(())
}
fn pmthi(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.hi = e.gpr[rs(w)];
    Ok(())
}
fn pmtlo(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.lo = e.gpr[rs(w)];
    Ok(())
}
fn pmfhl_lw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let (hi, lo) = (e.hi, e.lo);
    let v = build(32, |i| lane(if i % 2 == 0 { lo } else { hi }, 32, i & 2));
    e.set_quad(rd(w), v);
    Ok(())
}
fn pmfhl_uw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let (hi, lo) = (e.hi, e.lo);
    let v = build(32, |i| lane(if i % 2 == 0 { lo } else { hi }, 32, (i & 2) + 1));
    e.set_quad(rd(w), v);
    Ok(())
}
/// PMFHL.SLW: HI's and LO's words 0 and 2 joined as signed 64-bit values and clamped to a signed word.
fn pmfhl_slw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let (hi, lo) = (e.hi, e.lo);
    let v = build(64, |d| {
        let joined = (lane(hi, 32, 2 * d) << 32 | lane(lo, 32, 2 * d)) as i64;
        if joined >= i32::MAX as i64 {
            i32::MAX as u64
        } else if joined <= i32::MIN as i64 {
            i32::MIN as i64 as u64
        } else {
            sext32(lane(lo, 32, 2 * d) as u32)
        }
    });
    e.set_quad(rd(w), v);
    Ok(())
}
/// PMFHL.LH: the low halfword of every word of LO and HI, in the order the page lists.
fn pmfhl_lh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let (hi, lo) = (e.hi, e.lo);
    let v = build(16, |i| lane(if i % 4 < 2 { lo } else { hi }, 16, 2 * (i % 2) + 4 * (i / 4)));
    e.set_quad(rd(w), v);
    Ok(())
}
/// PMFHL.SH: every word of LO and HI clamped to a signed halfword, in PMFHL.LH's order.
fn pmfhl_sh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let (hi, lo) = (e.hi, e.lo);
    let v = build(16, |i| clamp_signed(signed(lane(if i % 4 < 2 { lo } else { hi }, 32, i % 2 + 2 * (i / 4)), 32), 16));
    e.set_quad(rd(w), v);
    Ok(())
}
/// PMTHL.LW: GPR[rs]'s words into LO's and HI's words 0 and 2; their words 1 and 3 are not changed.
fn pmthl_lw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let s = e.gpr[rs(w)];
    for d in 0..2 {
        e.lo = with_word(e.lo, 2 * d, lane(s, 32, 2 * d) as u32);
        e.hi = with_word(e.hi, 2 * d, lane(s, 32, 2 * d + 1) as u32);
    }
    Ok(())
}

/// PLZCW: in each word of GPR[rs]63..0, the leading bits equal to bit 31, less one; GPR[rd]127..64 are kept.
fn plzcw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let s = e.get(rs(w));
    let count = |x: u32| (if x >> 31 == 1 { (!x).leading_zeros() } else { x.leading_zeros() }) - 1;
    e.set(rd(w), (count((s >> 32) as u32) as u64) << 32 | count(s as u32) as u64);
    Ok(())
}

/// QFSRV: GPR[rs] || GPR[rt] shifted right by SA bytes, SA holding the byte count (Sapphire_Disputes.md D-9).
fn qfsrv(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let (s, t) = (e.gpr[rs(w)], e.gpr[rt(w)]);
    let bits = (e.sa & 15) as u32 * 8;
    let v = if bits == 0 { t } else { t >> bits | s << (128 - bits) };
    e.set_quad(rd(w), v);
    Ok(())
}

// The halfword multiplies. Product i goes to word (i mod 2) + 2 (i div 4) of LO when (i div 2) is even, of HI when
// odd; the even products' words are GPR[rd]'s (the pages' Operation sections and figures).
fn halfword_products(e: &Ee, w: u32) -> [u32; 8] {
    let (s, t) = (e.gpr[rs(w)], e.gpr[rt(w)]);
    std::array::from_fn(|i| (signed(lane(s, 16, i as u32), 16) * signed(lane(t, 16, i as u32), 16)) as u32)
}
fn multiply_halfwords(e: &mut Ee, w: u32, f: impl Fn(u32, u32) -> u32) -> R {
    let p = halfword_products(e, w);
    let mut out = e.gpr[rd(w)];
    for (i, &prod) in p.iter().enumerate() {
        let i = i as u32;
        let word = i % 2 + 2 * (i / 4);
        let reg = if (i / 2).is_multiple_of(2) { &mut e.lo } else { &mut e.hi };
        let v = f(lane(*reg, 32, word) as u32, prod);
        *reg = with_word(*reg, word, v);
        if i.is_multiple_of(2) {
            out = with_word(out, i / 2, v);
        }
    }
    e.set_quad(rd(w), out);
    Ok(())
}
fn pmulth(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    multiply_halfwords(e, w, |_, p| p)
}
fn pmaddh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    multiply_halfwords(e, w, |acc, p| acc.wrapping_add(p))
}
fn pmsubh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    multiply_halfwords(e, w, |acc, p| acc.wrapping_sub(p))
}
/// PHMADH and PHMSBH: sum j of products 2j+1 and 2j to LO's word 0, HI's 0, LO's 2 and HI's 2, and GPR[rd]'s word
/// j. The words 1 and 3 beside each, which the figures call undefined, take product 2j+1 (its complement for PHMSBH),
/// as ps2autotests' console record of `cpu/ee_simd/muldiv` shows (Sapphire_Disputes.md D-21).
fn horizontal(e: &mut Ee, w: u32, f: impl Fn(u32, u32) -> u32, beside: impl Fn(u32) -> u32) -> R {
    let p = halfword_products(e, w);
    let mut out = 0u128;
    for j in 0..4u32 {
        let (high, low) = (p[2 * j as usize + 1], p[2 * j as usize]);
        let v = f(high, low);
        let reg = if j % 2 == 0 { &mut e.lo } else { &mut e.hi };
        *reg = with_word(with_word(*reg, j & 2, v), (j & 2) + 1, beside(high));
        out = with_word(out, j, v);
    }
    e.set_quad(rd(w), out);
    Ok(())
}
fn phmadh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    horizontal(e, w, |a, b| a.wrapping_add(b), |high| high)
}
fn phmsbh(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    horizontal(e, w, |a, b| a.wrapping_sub(b), |high| !high)
}

// The word multiplies, on words 0 and 2: the 64-bit result into GPR[rd]'s doubleword, its low word sign-extended into
// LO's and its high word into HI's.
fn multiply_words(e: &mut Ee, w: u32, f: impl Fn(u64, u32, u32) -> u64) -> R {
    let (s, t) = (e.gpr[rs(w)], e.gpr[rt(w)]);
    let mut out = 0u128;
    for d in 0..2u32 {
        let acc = lane(e.hi, 32, 2 * d) << 32 | lane(e.lo, 32, 2 * d);
        let p = f(acc, lane(s, 64, d) as u32, lane(t, 64, d) as u32);
        let at = 64 * d;
        e.lo = e.lo & !((u64::MAX as u128) << at) | (sext32(p as u32) as u128) << at;
        e.hi = e.hi & !((u64::MAX as u128) << at) | (sext32((p >> 32) as u32) as u128) << at;
        out |= (p as u128) << at;
    }
    e.set_quad(rd(w), out);
    Ok(())
}
fn signed_word_product(a: u32, b: u32) -> u64 {
    (a as i32 as i64).wrapping_mul(b as i32 as i64) as u64
}
fn pmultw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    multiply_words(e, w, |_, a, b| signed_word_product(a, b))
}
fn pmultuw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    multiply_words(e, w, |_, a, b| a as u64 * b as u64)
}
fn pmaddw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    multiply_words(e, w, |acc, a, b| acc.wrapping_add(signed_word_product(a, b)))
}
fn pmadduw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    multiply_words(e, w, |acc, a, b| acc.wrapping_add(a as u64 * b as u64))
}
fn pmsubw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    multiply_words(e, w, |acc, a, b| acc.wrapping_sub(signed_word_product(a, b)))
}

// The divides. PDIVW and PDIVUW on words 0 and 2 as DIV and DIVU do (a zero divisor as Sapphire_Disputes.md D-6);
// PDIVBW every word by GPR[rt]15..0. Its HI word is the remainder's 32 bits: for a divisor other than zero that is the
// Operation section's sign extension of r15..0 (D-20), for zero D-6's dividend, as the console's record shows.
fn divide_words(e: &mut Ee, w: u32, f: impl Fn(u32, u32) -> (u32, u32)) -> R {
    let (s, t) = (e.gpr[rs(w)], e.gpr[rt(w)]);
    for d in 0..2u32 {
        let (q, r) = f(lane(s, 64, d) as u32, lane(t, 64, d) as u32);
        let at = 64 * d;
        e.lo = e.lo & !((u64::MAX as u128) << at) | (sext32(q) as u128) << at;
        e.hi = e.hi & !((u64::MAX as u128) << at) | (sext32(r) as u128) << at;
    }
    Ok(())
}
fn pdivw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    divide_words(e, w, |a, b| quotient_signed(a as i32, b as i32))
}
fn pdivuw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    divide_words(e, w, quotient_unsigned)
}
fn pdivbw(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let (s, d) = (e.gpr[rs(w)], e.gpr[rt(w)] as u16 as i16 as i32);
    for i in 0..4u32 {
        let (q, r) = quotient_signed(lane(s, 32, i) as i32, d);
        e.lo = with_word(e.lo, i, q);
        e.hi = with_word(e.hi, i, r);
    }
    Ok(())
}

use Category::{Lzc, WideOperate as Wide};
use rows::{ALU, HILO, MMDIV, MMULT};
const H: u8 = H0 | H1;

/// Every multimedia instruction by the decoder's name, with its category (Table 1-1: "128-bit multimedia
/// instructions" are Wide Operate, PLZCW is LZC) and its row of §2.12.
pub(super) static TABLE: &[Op] = &[
    op("paddb", paddb, Wide, ALU, RS | RT, RD),
    op("paddh", paddh, Wide, ALU, RS | RT, RD),
    op("paddw", paddw, Wide, ALU, RS | RT, RD),
    op("psubb", psubb, Wide, ALU, RS | RT, RD),
    op("psubh", psubh, Wide, ALU, RS | RT, RD),
    op("psubw", psubw, Wide, ALU, RS | RT, RD),
    op("paddsb", paddsb, Wide, ALU, RS | RT, RD),
    op("paddsh", paddsh, Wide, ALU, RS | RT, RD),
    op("paddsw", paddsw, Wide, ALU, RS | RT, RD),
    op("psubsb", psubsb, Wide, ALU, RS | RT, RD),
    op("psubsh", psubsh, Wide, ALU, RS | RT, RD),
    op("psubsw", psubsw, Wide, ALU, RS | RT, RD),
    op("paddub", paddub, Wide, ALU, RS | RT, RD),
    op("padduh", padduh, Wide, ALU, RS | RT, RD),
    op("padduw", padduw, Wide, ALU, RS | RT, RD),
    op("psubub", psubub, Wide, ALU, RS | RT, RD),
    op("psubuh", psubuh, Wide, ALU, RS | RT, RD),
    op("psubuw", psubuw, Wide, ALU, RS | RT, RD),
    op("padsbh", padsbh, Wide, ALU, RS | RT, RD),
    op("pceqb", pceqb, Wide, ALU, RS | RT, RD),
    op("pceqh", pceqh, Wide, ALU, RS | RT, RD),
    op("pceqw", pceqw, Wide, ALU, RS | RT, RD),
    op("pcgtb", pcgtb, Wide, ALU, RS | RT, RD),
    op("pcgth", pcgth, Wide, ALU, RS | RT, RD),
    op("pcgtw", pcgtw, Wide, ALU, RS | RT, RD),
    op("pmaxh", pmaxh, Wide, ALU, RS | RT, RD),
    op("pmaxw", pmaxw, Wide, ALU, RS | RT, RD),
    op("pminh", pminh, Wide, ALU, RS | RT, RD),
    op("pminw", pminw, Wide, ALU, RS | RT, RD),
    op("pabsh", pabsh, Wide, ALU, RT, RD),
    op("pabsw", pabsw, Wide, ALU, RT, RD),
    op("pand", pand, Wide, ALU, RS | RT, RD),
    op("por", por, Wide, ALU, RS | RT, RD),
    op("pxor", pxor, Wide, ALU, RS | RT, RD),
    op("pnor", pnor, Wide, ALU, RS | RT, RD),
    op("psllh", psllh, Wide, ALU, RT, RD),
    op("psrlh", psrlh, Wide, ALU, RT, RD),
    op("psrah", psrah, Wide, ALU, RT, RD),
    op("psllw", psllw, Wide, ALU, RT, RD),
    op("psrlw", psrlw, Wide, ALU, RT, RD),
    op("psraw", psraw, Wide, ALU, RT, RD),
    op("psllvw", psllvw, Wide, ALU, RS | RT, RD),
    op("psrlvw", psrlvw, Wide, ALU, RS | RT, RD),
    op("psravw", psravw, Wide, ALU, RS | RT, RD),
    op("pextlb", pextlb, Wide, ALU, RS | RT, RD),
    op("pextub", pextub, Wide, ALU, RS | RT, RD),
    op("pextlh", pextlh, Wide, ALU, RS | RT, RD),
    op("pextuh", pextuh, Wide, ALU, RS | RT, RD),
    op("pextlw", pextlw, Wide, ALU, RS | RT, RD),
    op("pextuw", pextuw, Wide, ALU, RS | RT, RD),
    op("ppacb", ppacb, Wide, ALU, RS | RT, RD),
    op("ppach", ppach, Wide, ALU, RS | RT, RD),
    op("ppacw", ppacw, Wide, ALU, RS | RT, RD),
    op("pinth", pinth, Wide, ALU, RS | RT, RD),
    op("pinteh", pinteh, Wide, ALU, RS | RT, RD),
    op("pcpyld", pcpyld, Wide, ALU, RS | RT, RD),
    op("pcpyud", pcpyud, Wide, ALU, RS | RT, RD),
    op("pcpyh", pcpyh, Wide, ALU, RT, RD),
    op("pexch", pexch, Wide, ALU, RT, RD),
    op("pexcw", pexcw, Wide, ALU, RT, RD),
    op("pexeh", pexeh, Wide, ALU, RT, RD),
    op("pexew", pexew, Wide, ALU, RT, RD),
    op("prevh", prevh, Wide, ALU, RT, RD),
    op("prot3w", prot3w, Wide, ALU, RT, RD),
    op("pext5", pext5, Wide, ALU, RT, RD),
    op("ppac5", ppac5, Wide, ALU, RT, RD),
    op("pmfhi", pmfhi, Wide, HILO, H, RD),
    op("pmflo", pmflo, Wide, HILO, H, RD),
    op("pmthi", pmthi, Wide, HILO, RS, H),
    op("pmtlo", pmtlo, Wide, HILO, RS, H),
    op("pmfhl.lw", pmfhl_lw, Wide, HILO, H, RD),
    op("pmfhl.uw", pmfhl_uw, Wide, HILO, H, RD),
    op("pmfhl.slw", pmfhl_slw, Wide, HILO, H, RD),
    op("pmfhl.lh", pmfhl_lh, Wide, HILO, H, RD),
    op("pmfhl.sh", pmfhl_sh, Wide, HILO, H, RD),
    op("pmthl.lw", pmthl_lw, Wide, HILO, RS | H, H),
    op("plzcw", plzcw, Lzc, ALU, RS, RD),
    op("qfsrv", qfsrv, Wide, ALU, RS | RT | S, RD),
    op("pmulth", pmulth, Wide, MMULT, RS | RT, RD | H),
    op("pmaddh", pmaddh, Wide, MMULT, RS | RT, RD | H),
    op("pmsubh", pmsubh, Wide, MMULT, RS | RT, RD | H),
    op("phmadh", phmadh, Wide, MMULT, RS | RT, RD | H),
    op("phmsbh", phmsbh, Wide, MMULT, RS | RT, RD | H),
    op("pmultw", pmultw, Wide, MMULT, RS | RT, RD | H),
    op("pmultuw", pmultuw, Wide, MMULT, RS | RT, RD | H),
    op("pmaddw", pmaddw, Wide, MMULT, RS | RT, RD | H),
    op("pmadduw", pmadduw, Wide, MMULT, RS | RT, RD | H),
    op("pmsubw", pmsubw, Wide, MMULT, RS | RT, RD | H),
    op("pdivw", pdivw, Wide, MMDIV, RS | RT, H),
    op("pdivuw", pdivuw, Wide, MMDIV, RS | RT, H),
    op("pdivbw", pdivbw, Wide, MMDIV, RS | RT, H),
];
