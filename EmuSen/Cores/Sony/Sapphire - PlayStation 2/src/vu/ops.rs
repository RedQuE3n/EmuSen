//! The operations both vector units execute, one function per operation as `emusen_mips::vu` names it, each written
//! from its page in the VU User's Manual (§4.2, §4.3; macro mode's §6.2 refers to them), the arithmetic on
//! `crate::float`. The flags are §3.3's: each FMAC lane in the field mask sets its MAC bits from its result (the sign
//! bit, a zero, an exponent's overflow or underflow), the others clear; the status flag's Z, S, U and O collect the MAC
//! groups and their sticky flags take them and, for a multiply-add, the product's own (§3.3.2). Sapphire_Native.md §21.3
//! records every reading. Nothing here decodes beyond the fields; timing is the caller's.

use std::collections::HashMap;
use std::sync::OnceLock;

use emusen_mips::vu::{self as table, Operation};

use super::{Vu, status};
use crate::float::{self, Outcome};

/// What an operation works on: its unit, and for VU0 the window onto VU1's registers (§1.1.5); a divide's quotient is
/// handed back in `q_out` for the caller to land in Q at the divider's latency.
pub struct Ctx<'a> {
    pub vu: &'a mut Vu,
    pub window: Option<&'a mut Vu>,
    pub q_out: Option<u32>,
}

/// An operation's pipeline, for the timing (§3.4, §7.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// The FMAC pipeline, latency 4: the upper operations, the moves, the loads and stores, RGET and RNEXT.
    Fmac,
    /// The integer unit, latency 1: the integer ALU, MTIR, RINIT, RXOR.
    Ialu,
    /// The divider: DIV and SQRT 7, RSQRT 13.
    Div(u8),
    WaitQ,
    /// CALLMS and CALLMSR, which macro mode carries out itself.
    Callms,
}

pub struct Entry {
    pub name: &'static str,
    pub run: fn(&mut Ctx, u32),
    pub class: Class,
    /// Whether it writes the MAC and status flags (§3.3.5's table).
    pub flags: bool,
}

const fn e(name: &'static str, run: fn(&mut Ctx, u32), class: Class, flags: bool) -> Entry {
    Entry { name, run, class, flags }
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
/// The field mask's lanes: x is bit 24 (§4.1.3).
fn lane_on(w: u32, lane: usize) -> bool {
    w >> (24 - lane) & 1 == 1
}
fn fsf(w: u32) -> usize {
    (w >> 21 & 3) as usize
}
fn ftf(w: u32) -> usize {
    (w >> 23 & 3) as usize
}

/// The second operand by the operation's family: VF[ft]'s same lane, its bc field, I or Q.
const FT: u8 = 0;
const BC: u8 = 1;
const I: u8 = 2;
const Q: u8 = 3;

fn second<const B: u8>(vu: &Vu, w: u32, lane: usize) -> u32 {
    match B {
        FT => vu.vf(ft(w))[lane],
        BC => vu.vf(ft(w))[(w & 3) as usize],
        I => vu.i,
        _ => vu.q,
    }
}

/// The MAC bits of one lane's result (§3.3.1): Z for a zero, S for the sign bit, U and O from the exponent.
fn mac_bits(o: Outcome, lane: usize) -> u32 {
    let at = 3 - lane as u32;
    let z = u32::from(float::is_zero(o.bits)) << at;
    let s = (o.bits >> 31) << (4 + at);
    let u = u32::from(o.underflow) << (8 + at);
    let v = u32::from(o.overflow) << (12 + at);
    z | s | u | v
}

/// The four groups of the MAC flags as the status flag's Z, S, U, O.
fn collect(mac: u32) -> u32 {
    (0..4).fold(0, |a, g| a | u32::from(mac >> (4 * g) & 0xF != 0) << g)
}

/// Writes the MAC flags and the status flag's Z, S, U, O from them; the sticky flags take those and `extra` (a
/// multiply-add's product, as Z, S, U, O).
fn set_flags(vu: &mut Vu, mac: u32, extra: u32) {
    vu.mac = mac;
    let f = collect(mac);
    vu.status = vu.status & !0xF | f | (f | extra) << status::STICKY;
}

/// The lanes' results of `f`, written to VF[fd] (or ACC) where the mask allows, and the flags set from them.
fn lanes(c: &mut Ctx, w: u32, to_acc: bool, f: impl Fn(&Vu, usize) -> (Outcome, Option<Outcome>)) {
    let vu = &mut *c.vu;
    let mut out = if to_acc { vu.acc } else { vu.vf(fd(w)) };
    let (mut mac, mut extra) = (0, 0);
    for (lane, slot) in out.iter_mut().enumerate() {
        if !lane_on(w, lane) {
            continue;
        }
        let (o, product) = f(vu, lane);
        *slot = o.bits;
        mac |= mac_bits(o, lane);
        if let Some(p) = product {
            extra |= collect(mac_bits(p, lane));
        }
    }
    if to_acc {
        vu.acc = out;
    } else {
        vu.set_vf(fd(w), out);
    }
    set_flags(vu, mac, extra);
}

fn arith<const B: u8, const A: bool>(c: &mut Ctx, w: u32, f: fn(u32, u32) -> Outcome) {
    lanes(c, w, A, |vu, l| (f(vu.vf(fs(w))[l], second::<B>(vu, w, l)), None));
}

fn fused<const B: u8, const A: bool, const SUB: bool>(c: &mut Ctx, w: u32) {
    lanes(c, w, A, |vu, l| {
        let r = float::fused(vu.acc[l], vu.vf(fs(w))[l], second::<B>(vu, w, l), SUB);
        (r.value, Some(r.product))
    });
}

fn add<const B: u8, const A: bool>(c: &mut Ctx, w: u32) {
    arith::<B, A>(c, w, float::add);
}
fn sub<const B: u8, const A: bool>(c: &mut Ctx, w: u32) {
    arith::<B, A>(c, w, float::sub);
}
fn mul<const B: u8, const A: bool>(c: &mut Ctx, w: u32) {
    arith::<B, A>(c, w, float::mul);
}
fn madd<const B: u8, const A: bool>(c: &mut Ctx, w: u32) {
    fused::<B, A, false>(c, w);
}
fn msub<const B: u8, const A: bool>(c: &mut Ctx, w: u32) {
    fused::<B, A, true>(c, w);
}

/// OPMULA: ACC's x, y and z take the outer product's first terms, the fields fixed (its Remarks).
fn opmula(c: &mut Ctx, w: u32) {
    lanes(c, w & !(0xF << 21) | 0xE << 21, true, |vu, l| {
        let (a, b) = (vu.vf(fs(w)), vu.vf(ft(w)));
        (float::mul(a[(l + 1) % 3], b[(l + 2) % 3]), None)
    });
}
/// OPMSUB: VF[fd]'s x, y and z take ACC less the outer product's second terms.
fn opmsub(c: &mut Ctx, w: u32) {
    lanes(c, w & !(0xF << 21) | 0xE << 21, false, |vu, l| {
        let (a, b) = (vu.vf(fs(w)), vu.vf(ft(w)));
        let r = float::fused(vu.acc[l], a[(l + 1) % 3], b[(l + 2) % 3], true);
        (r.value, Some(r.product))
    });
}

/// VF[fd] ← `f`(VF[fs], the second operand) lane by lane, no flag changed (MAX, MINI).
fn pick<const B: u8>(c: &mut Ctx, w: u32, f: fn(u32, u32) -> u32) {
    let vu = &mut *c.vu;
    let mut out = vu.vf(fd(w));
    for (lane, o) in out.iter_mut().enumerate() {
        if lane_on(w, lane) {
            *o = f(vu.vf(fs(w))[lane], second::<B>(vu, w, lane));
        }
    }
    vu.set_vf(fd(w), out);
}
fn max<const B: u8>(c: &mut Ctx, w: u32) {
    pick::<B>(c, w, float::max);
}
fn mini<const B: u8>(c: &mut Ctx, w: u32) {
    pick::<B>(c, w, float::min);
}

/// VF[ft] ← `f`(VF[fs]) lane by lane, no flag changed: ABS, the conversions, MOVE.
fn unary(c: &mut Ctx, w: u32, f: impl Fn(u32) -> u32) {
    let vu = &mut *c.vu;
    let src = vu.vf(fs(w));
    let mut out = vu.vf(ft(w));
    for lane in 0..4 {
        if lane_on(w, lane) {
            out[lane] = f(src[lane]);
        }
    }
    vu.set_vf(ft(w), out);
}
fn abs(c: &mut Ctx, w: u32) {
    unary(c, w, |x| x & float::MAX);
}
fn ftoi<const N: u32>(c: &mut Ctx, w: u32) {
    unary(c, w, |x| float::to_fixed(x, N));
}
fn itof<const N: u32>(c: &mut Ctx, w: u32) {
    unary(c, w, |x| float::from_fixed(x, N));
}
fn mov(c: &mut Ctx, w: u32) {
    unary(c, w, |x| x);
}
/// MR32: each lane takes the next lane up, w taking x.
fn mr32(c: &mut Ctx, w: u32) {
    let v = c.vu.vf(fs(w));
    let mut out = c.vu.vf(ft(w));
    for lane in 0..4 {
        if lane_on(w, lane) {
            out[lane] = v[(lane + 1) % 4];
        }
    }
    c.vu.set_vf(ft(w), out);
}

/// CLIPw: the judgement of x, y and z of VF[fs] against ±|w| of VF[ft] shifted in at the low end (§3.3.3); compared as
/// the FPU's compares compare, every zero equal and exponent 255 a number (Sapphire_Disputes.md D-49).
fn clip(c: &mut Ctx, w: u32) {
    let vu = &mut *c.vu;
    let (v, limit) = (vu.vf(fs(w)), vu.vf(ft(w))[3] & float::MAX);
    let (hi, lo) = (float::order(limit), float::order(limit | float::SIGN));
    let mut j = 0;
    for (lane, &x) in v.iter().take(3).enumerate() {
        let o = float::order(x);
        j |= u32::from(o > hi) << (2 * lane) | u32::from(o < lo) << (2 * lane + 1);
    }
    vu.clip = (vu.clip << 6 | j) & 0xFF_FFFF;
}

/// D and I written, IS and DS set where they are.
fn divide_flags(vu: &mut Vu, invalid: bool, divide: bool) {
    let f = (u32::from(invalid) * status::I) | (u32::from(divide) * status::D);
    vu.status = vu.status & !(status::I | status::D) | f | f << status::STICKY;
}
/// DIV: Q ← fsf of VF[fs] over ftf of VF[ft], rounded as DIV.S; I for 0/0, D for another zero divisor (§3.3.2).
fn div(c: &mut Ctx, w: u32) {
    let q = float::div(c.vu.vf(fs(w))[fsf(w)], c.vu.vf(ft(w))[ftf(w)]);
    divide_flags(c.vu, q.invalid, q.divide);
    c.q_out = Some(q.value.bits);
}
/// SQRT: Q ← the root of |ftf of VF[ft]|; I for a negative value, D cleared (§3.3.2, D-46).
fn sqrt(c: &mut Ctx, w: u32) {
    let x = c.vu.vf(ft(w))[ftf(w)];
    divide_flags(c.vu, x >> 31 == 1 && !float::is_zero(x), false);
    c.q_out = Some(float::sqrt(x).bits);
}
/// RSQRT: Q ← fsf of VF[fs] over the root of |ftf of VF[ft]|; D for a zero root under a non-zero dividend, I for a
/// negative value; 0/√0 raises neither (§3.3.2's "except 0/0", D-46).
fn rsqrt(c: &mut Ctx, w: u32) {
    let (a, b) = (c.vu.vf(fs(w))[fsf(w)], c.vu.vf(ft(w))[ftf(w)]);
    let q = float::rsqrt(a, b);
    divide_flags(c.vu, q.invalid, q.divide && !float::is_zero(a));
    c.q_out = Some(q.value.bits);
}

// The integer unit (§4.3): 16-bit results.
fn iadd(c: &mut Ctx, w: u32) {
    let v = c.vu.vi(fs(w)).wrapping_add(c.vu.vi(ft(w)));
    c.vu.set_vi(fd(w), v);
}
fn isub(c: &mut Ctx, w: u32) {
    let v = c.vu.vi(fs(w)).wrapping_sub(c.vu.vi(ft(w)));
    c.vu.set_vi(fd(w), v);
}
fn iand(c: &mut Ctx, w: u32) {
    let v = c.vu.vi(fs(w)) & c.vu.vi(ft(w));
    c.vu.set_vi(fd(w), v);
}
fn ior(c: &mut Ctx, w: u32) {
    let v = c.vu.vi(fs(w)) | c.vu.vi(ft(w));
    c.vu.set_vi(fd(w), v);
}
/// IADDI: "Imm5 is considered as a signed integer".
fn iaddi(c: &mut Ctx, w: u32) {
    let imm = ((w >> 6 & 31) << 27) as i32 >> 27;
    let v = c.vu.vi(fs(w)).wrapping_add(imm as u16);
    c.vu.set_vi(ft(w), v);
}
/// MTIR: VI[it] ← the low 16 bits of fsf of VF[fs].
fn mtir(c: &mut Ctx, w: u32) {
    let v = c.vu.vf(fs(w))[fsf(w)] as u16;
    c.vu.set_vi(ft(w), v);
}
/// MFIR: VF[ft]'s lanes ← VI[is] sign-extended to 32 bits.
fn mfir(c: &mut Ctx, w: u32) {
    let v = c.vu.vi(fs(w)) as i16 as i32 as u32;
    unary(c, w & !(31 << 11), |_| v);
}

/// The qword at data index `a` (VI × 16 bytes): VU0 reaches VU1's registers from 4000h (index 400h), its own 4 KiB
/// mirrored below that (§1.1.5, §5.5.3 to §5.5.5) [A].
fn load_q(c: &Ctx, a: u16) -> u128 {
    match &c.window {
        Some(vu1) if a & 0x400 != 0 => vu1.window_read((a & 0x3F) as usize),
        _ => c.vu.mem_q(a as usize),
    }
}
fn store_q(c: &mut Ctx, a: u16, q: u128) {
    match &mut c.window {
        Some(vu1) if a & 0x400 != 0 => vu1.window_write((a & 0x3F) as usize, q),
        _ => c.vu.set_mem_q(a as usize, q),
    }
}
fn lanes_of(q: u128) -> [u32; 4] {
    [q as u32, (q >> 32) as u32, (q >> 64) as u32, (q >> 96) as u32]
}
fn quad_of(l: [u32; 4]) -> u128 {
    l.iter().rev().fold(0u128, |a, &f| a << 32 | f as u128)
}
fn load_into(c: &mut Ctx, w: u32, a: u16) {
    let m = lanes_of(load_q(c, a));
    let mut out = c.vu.vf(ft(w));
    for lane in 0..4 {
        if lane_on(w, lane) {
            out[lane] = m[lane];
        }
    }
    c.vu.set_vf(ft(w), out);
}
/// Only the masked lanes of the qword change.
fn store_from(c: &mut Ctx, w: u32, a: u16) {
    let mut m = lanes_of(load_q(c, a));
    let v = c.vu.vf(fs(w));
    for lane in 0..4 {
        if lane_on(w, lane) {
            m[lane] = v[lane];
        }
    }
    store_q(c, a, quad_of(m));
}
fn lqi(c: &mut Ctx, w: u32) {
    let a = c.vu.vi(fs(w));
    load_into(c, w, a);
    c.vu.set_vi(fs(w), a.wrapping_add(1));
}
fn lqd(c: &mut Ctx, w: u32) {
    let a = c.vu.vi(fs(w)).wrapping_sub(1);
    c.vu.set_vi(fs(w), a);
    load_into(c, w, a);
}
fn sqi(c: &mut Ctx, w: u32) {
    let a = c.vu.vi(ft(w));
    store_from(c, w, a);
    c.vu.set_vi(ft(w), a.wrapping_add(1));
}
fn sqd(c: &mut Ctx, w: u32) {
    let a = c.vu.vi(ft(w)).wrapping_sub(1);
    c.vu.set_vi(ft(w), a);
    store_from(c, w, a);
}
/// ILWR: VI[it] ← the low 16 bits of the field the mask names, the lowest lane where it names more (undefined, §4.3).
fn ilwr(c: &mut Ctx, w: u32) {
    let m = lanes_of(load_q(c, c.vu.vi(fs(w))));
    if let Some(lane) = (0..4).find(|&l| lane_on(w, l)) {
        c.vu.set_vi(ft(w), m[lane] as u16);
    }
}
/// ISWR: each masked field ← VI[it] zero-extended.
fn iswr(c: &mut Ctx, w: u32) {
    let a = c.vu.vi(fs(w));
    let mut m = lanes_of(load_q(c, a));
    let v = c.vu.vi(ft(w)) as u32;
    for (lane, f) in m.iter_mut().enumerate() {
        if lane_on(w, lane) {
            *f = v;
        }
    }
    store_q(c, a, quad_of(m));
}

/// The random unit's M series: R shifted left one place, bit 22 XOR bit 4 entering at bit 0 (Sapphire_Disputes.md
/// D-48, from ps2autotests' `vu/lower/random` console text).
pub fn next_r(r: u32) -> u32 {
    (r << 1 | (r >> 22 ^ r >> 4) & 1) & 0x7F_FFFF
}
/// R as a float in [1, 2): 0 01111111 then R as the fraction (RGET's Operation).
fn r_float(r: u32) -> u32 {
    0x3F80_0000 | r
}
fn rinit(c: &mut Ctx, w: u32) {
    c.vu.r = c.vu.vf(fs(w))[fsf(w)] & 0x7F_FFFF;
}
fn rxor(c: &mut Ctx, w: u32) {
    c.vu.r ^= c.vu.vf(fs(w))[fsf(w)] & 0x7F_FFFF;
}
fn rget(c: &mut Ctx, w: u32) {
    let v = r_float(c.vu.r);
    unary(c, w & !(31 << 11), |_| v);
}
fn rnext(c: &mut Ctx, w: u32) {
    c.vu.r = next_r(c.vu.r);
    rget(c, w);
}

fn nothing(_: &mut Ctx, _: u32) {}

use Class::{Callms, Div, Fmac, Ialu, WaitQ};

/// Every operation of the shared table by its micro name, with its pipeline and whether it sets the MAC flags.
pub static TABLE: &[Entry] = &[
    e("addx", add::<BC, false>, Fmac, true), e("addy", add::<BC, false>, Fmac, true), e("addz", add::<BC, false>, Fmac, true), e("addw", add::<BC, false>, Fmac, true),
    e("subx", sub::<BC, false>, Fmac, true), e("suby", sub::<BC, false>, Fmac, true), e("subz", sub::<BC, false>, Fmac, true), e("subw", sub::<BC, false>, Fmac, true),
    e("maddx", madd::<BC, false>, Fmac, true), e("maddy", madd::<BC, false>, Fmac, true), e("maddz", madd::<BC, false>, Fmac, true), e("maddw", madd::<BC, false>, Fmac, true),
    e("msubx", msub::<BC, false>, Fmac, true), e("msuby", msub::<BC, false>, Fmac, true), e("msubz", msub::<BC, false>, Fmac, true), e("msubw", msub::<BC, false>, Fmac, true),
    e("maxx", max::<BC>, Fmac, false), e("maxy", max::<BC>, Fmac, false), e("maxz", max::<BC>, Fmac, false), e("maxw", max::<BC>, Fmac, false),
    e("minix", mini::<BC>, Fmac, false), e("miniy", mini::<BC>, Fmac, false), e("miniz", mini::<BC>, Fmac, false), e("miniw", mini::<BC>, Fmac, false),
    e("mulx", mul::<BC, false>, Fmac, true), e("muly", mul::<BC, false>, Fmac, true), e("mulz", mul::<BC, false>, Fmac, true), e("mulw", mul::<BC, false>, Fmac, true),
    e("mulq", mul::<Q, false>, Fmac, true), e("maxi", max::<I>, Fmac, false), e("muli", mul::<I, false>, Fmac, true), e("minii", mini::<I>, Fmac, false),
    e("addq", add::<Q, false>, Fmac, true), e("maddq", madd::<Q, false>, Fmac, true), e("addi", add::<I, false>, Fmac, true), e("maddi", madd::<I, false>, Fmac, true),
    e("subq", sub::<Q, false>, Fmac, true), e("msubq", msub::<Q, false>, Fmac, true), e("subi", sub::<I, false>, Fmac, true), e("msubi", msub::<I, false>, Fmac, true),
    e("add", add::<FT, false>, Fmac, true), e("madd", madd::<FT, false>, Fmac, true), e("mul", mul::<FT, false>, Fmac, true), e("max", max::<FT>, Fmac, false),
    e("sub", sub::<FT, false>, Fmac, true), e("msub", msub::<FT, false>, Fmac, true), e("opmsub", opmsub, Fmac, true), e("mini", mini::<FT>, Fmac, false),
    e("iadd", iadd, Ialu, false), e("isub", isub, Ialu, false), e("iaddi", iaddi, Ialu, false), e("iand", iand, Ialu, false), e("ior", ior, Ialu, false),
    e("callms", nothing, Callms, false), e("callmsr", nothing, Callms, false),
    e("addax", add::<BC, true>, Fmac, true), e("adday", add::<BC, true>, Fmac, true), e("addaz", add::<BC, true>, Fmac, true), e("addaw", add::<BC, true>, Fmac, true),
    e("subax", sub::<BC, true>, Fmac, true), e("subay", sub::<BC, true>, Fmac, true), e("subaz", sub::<BC, true>, Fmac, true), e("subaw", sub::<BC, true>, Fmac, true),
    e("maddax", madd::<BC, true>, Fmac, true), e("madday", madd::<BC, true>, Fmac, true), e("maddaz", madd::<BC, true>, Fmac, true), e("maddaw", madd::<BC, true>, Fmac, true),
    e("msubax", msub::<BC, true>, Fmac, true), e("msubay", msub::<BC, true>, Fmac, true), e("msubaz", msub::<BC, true>, Fmac, true), e("msubaw", msub::<BC, true>, Fmac, true),
    e("itof0", itof::<0>, Fmac, false), e("itof4", itof::<4>, Fmac, false), e("itof12", itof::<12>, Fmac, false), e("itof15", itof::<15>, Fmac, false),
    e("ftoi0", ftoi::<0>, Fmac, false), e("ftoi4", ftoi::<4>, Fmac, false), e("ftoi12", ftoi::<12>, Fmac, false), e("ftoi15", ftoi::<15>, Fmac, false),
    e("mulax", mul::<BC, true>, Fmac, true), e("mulay", mul::<BC, true>, Fmac, true), e("mulaz", mul::<BC, true>, Fmac, true), e("mulaw", mul::<BC, true>, Fmac, true),
    e("mulaq", mul::<Q, true>, Fmac, true), e("abs", abs, Fmac, false), e("mulai", mul::<I, true>, Fmac, true), e("clipw", clip, Fmac, false),
    e("addaq", add::<Q, true>, Fmac, true), e("maddaq", madd::<Q, true>, Fmac, true), e("addai", add::<I, true>, Fmac, true), e("maddai", madd::<I, true>, Fmac, true),
    e("subaq", sub::<Q, true>, Fmac, true), e("msubaq", msub::<Q, true>, Fmac, true), e("subai", sub::<I, true>, Fmac, true), e("msubai", msub::<I, true>, Fmac, true),
    e("adda", add::<FT, true>, Fmac, true), e("madda", madd::<FT, true>, Fmac, true), e("mula", mul::<FT, true>, Fmac, true),
    e("suba", sub::<FT, true>, Fmac, true), e("msuba", msub::<FT, true>, Fmac, true), e("opmula", opmula, Fmac, true), e("nop", nothing, Fmac, false),
    e("move", mov, Fmac, false), e("mr32", mr32, Fmac, false),
    e("lqi", lqi, Fmac, false), e("sqi", sqi, Fmac, false), e("lqd", lqd, Fmac, false), e("sqd", sqd, Fmac, false),
    e("div", div, Div(7), false), e("sqrt", sqrt, Div(7), false), e("rsqrt", rsqrt, Div(13), false), e("waitq", nothing, WaitQ, false),
    e("mtir", mtir, Ialu, false), e("mfir", mfir, Fmac, false), e("ilwr", ilwr, Fmac, false), e("iswr", iswr, Fmac, false),
    e("rnext", rnext, Fmac, false), e("rget", rget, Fmac, false), e("rinit", rinit, Ialu, false), e("rxor", rxor, Ialu, false),
];

fn by_name() -> &'static HashMap<&'static str, usize> {
    static MAP: OnceLock<HashMap<&'static str, usize>> = OnceLock::new();
    MAP.get_or_init(|| TABLE.iter().enumerate().map(|(i, e)| (e.name, i)).collect())
}

/// The entry for a decoded operation.
pub fn entry(o: &Operation) -> &'static Entry {
    &TABLE[by_name()[o.name]]
}

/// The entry for the operation in a word's low 25 bits, through the decoder alone.
pub fn of_word(w: u32) -> Option<(&'static Operation, &'static Entry)> {
    table::operation(w).map(|o| (o, entry(o)))
}
