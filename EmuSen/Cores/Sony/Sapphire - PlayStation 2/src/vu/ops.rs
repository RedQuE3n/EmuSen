//! The operations both vector units execute, one function per operation as `emusen_mips::vu` names it, each written
//! from its page in the VU User's Manual (§4.2, §4.3; macro mode's §6.2 refers to them), the arithmetic on
//! `crate::float`. The flags are §3.3's: each FMAC lane in the field mask sets its MAC bits from its result (the sign
//! bit, a zero, an exponent's overflow or underflow), the others clear; the status flag's Z, S, U and O collect the MAC
//! groups and their sticky flags take them and, for a multiply-add, the product's own (§3.3.2). Sapphire_Native.md §21.3
//! records every reading; the micro-only lower operations (S13) are §23 there. Nothing here decodes beyond the fields;
//! timing is the caller's: in micro mode (`Ctx::micro`) the flags an operation sets are handed back in `Deferred` for
//! the pipeline to write at the S stage, and the branches, the external units and the waits are the pipeline's own.

use std::sync::OnceLock;

use emusen_mips::vu::{self as table, LOWER, Operation, SPECIAL1, SPECIAL2};

use super::{Vu, efu, status};
use crate::float::{self, Outcome};

/// What an operation works on: its unit, and for VU0 the window onto VU1's registers (§1.1.5); a divide's quotient is
/// handed back in `q_out` for the caller to land in Q at the divider's latency.
pub struct Ctx<'a> {
    pub vu: &'a mut Vu,
    pub window: Option<&'a mut Vu>,
    pub q_out: Option<u32>,
    /// Micro mode: the flags are not written but handed back in `out` (§3.4.4: "set at the S stage").
    pub defer: bool,
    pub out: Deferred,
}

/// What an operation in micro mode leaves for the pipeline to write later: the FMAC's MAC flags with the status flag's
/// Z, S, U, O and the sticky bits they add; CLIP's judgement; FSSET's sticky flags and FCSET's clipping flag; a divide's
/// I and D (with their sticky bits); the EFU's result for P.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Deferred {
    pub mac: Option<u32>,
    pub zsuo: u32,
    pub sticky: u32,
    pub clip_shift: Option<u32>,
    pub clip_set: Option<u32>,
    pub sticky_set: Option<u32>,
    pub divide: Option<u32>,
    pub p: Option<u32>,
}

impl<'a> Ctx<'a> {
    /// Macro mode: every flag written as the operation runs.
    pub fn new(vu: &'a mut Vu, window: Option<&'a mut Vu>) -> Ctx<'a> {
        Ctx { vu, window, q_out: None, defer: false, out: Deferred::default() }
    }
    /// Micro mode: the flags deferred to the pipeline.
    pub fn micro(vu: &'a mut Vu, window: Option<&'a mut Vu>) -> Ctx<'a> {
        Ctx { defer: true, ..Ctx::new(vu, window) }
    }
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
    /// The flag instructions that read a flag into VI, whose result a branch may read at once (§3.4.8).
    Fcheck,
    /// FSSET and FCSET, which write their flag at the S stage (their pages).
    FlagSet,
    /// The EFU (VU1 only): its latency; another EFU operation may start a cycle before it lands (§3.4.6).
    Efu(u8),
    WaitP,
    /// B, BAL, JR, JALR and the conditional branches, which the pipeline carries out.
    Branch,
    /// XGKICK, XTOP and XITOP, which reach units outside the VU.
    External,
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
/// multiply-add's product, as Z, S, U, O). In micro mode they are handed back for the S stage.
fn set_flags(c: &mut Ctx, mac: u32, extra: u32) {
    let f = collect(mac);
    if c.defer {
        c.out.mac = Some(mac);
        c.out.zsuo = f;
        c.out.sticky = f | extra;
    } else {
        apply_fmac_flags(c.vu, mac, f, f | extra);
    }
}

/// The FMAC's flags as the S stage writes them: MAC replaced, Z, S, U and O replaced, the sticky bits added.
pub fn apply_fmac_flags(vu: &mut Vu, mac: u32, zsuo: u32, sticky: u32) {
    vu.mac = mac;
    vu.status = vu.status & !0xF | zsuo | sticky << status::STICKY;
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
    set_flags(c, mac, extra);
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
    if c.defer {
        c.out.clip_shift = Some(j);
    } else {
        vu.clip = shift_clip(vu.clip, j);
    }
}

/// The clipping flag with a new judgement shifted in (§3.3.3).
pub fn shift_clip(clip: u32, j: u32) -> u32 {
    (clip << 6 | j) & 0xFF_FFFF
}

/// D and I written, IS and DS set where they are; in micro mode handed back for the divider's F stage.
fn divide_flags(c: &mut Ctx, invalid: bool, divide: bool) {
    let f = (u32::from(invalid) * status::I) | (u32::from(divide) * status::D);
    if c.defer {
        c.out.divide = Some(f);
    } else {
        apply_divide_flags(c.vu, f);
    }
}

/// A divide's I and D as they land: written, and added to IS and DS.
pub fn apply_divide_flags(vu: &mut Vu, f: u32) {
    vu.status = vu.status & !(status::I | status::D) | f | f << status::STICKY;
}
/// DIV: Q ← fsf of VF[fs] over ftf of VF[ft], rounded as DIV.S; I for 0/0, D for another zero divisor (§3.3.2).
fn div(c: &mut Ctx, w: u32) {
    let q = float::div(c.vu.vf(fs(w))[fsf(w)], c.vu.vf(ft(w))[ftf(w)]);
    divide_flags(c, q.invalid, q.divide);
    c.q_out = Some(q.value.bits);
}
/// SQRT: Q ← the root of |ftf of VF[ft]|; I for a negative value, D cleared (§3.3.2, D-46).
fn sqrt(c: &mut Ctx, w: u32) {
    let x = c.vu.vf(ft(w))[ftf(w)];
    divide_flags(c, x >> 31 == 1 && !float::is_zero(x), false);
    c.q_out = Some(float::sqrt(x).bits);
}
/// RSQRT: Q ← fsf of VF[fs] over the root of |ftf of VF[ft]|; D for a zero root under a non-zero dividend, I for a
/// negative value; 0/√0 raises neither (§3.3.2's "except 0/0", D-46).
fn rsqrt(c: &mut Ctx, w: u32) {
    let (a, b) = (c.vu.vf(fs(w))[fsf(w)], c.vu.vf(ft(w))[ftf(w)]);
    let q = float::rsqrt(a, b);
    divide_flags(c, q.invalid, q.divide && !float::is_zero(a));
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

// The micro-only lower operations (§4.3), each from its page; Imm11 and the 16-bit address arithmetic wrap.
fn offset(c: &Ctx, w: u32) -> u16 {
    c.vu.vi(fs(w)).wrapping_add(table::imm11(w) as u16)
}
/// LQ: VF[ft]'s masked fields ← the qword at Imm11 + VI[is].
fn lq(c: &mut Ctx, w: u32) {
    let a = offset(c, w);
    load_into(c, w, a);
}
/// SQ: the qword at Imm11 + VI[it] ← VF[fs]'s masked fields.
fn sq(c: &mut Ctx, w: u32) {
    let a = c.vu.vi(ft(w)).wrapping_add(table::imm11(w) as u16);
    store_from(c, w, a);
}
/// ILW: VI[it] ← the low 16 bits of the field dest names, the lowest where it names more (as ILWR).
fn ilw(c: &mut Ctx, w: u32) {
    let m = lanes_of(load_q(c, offset(c, w)));
    if let Some(lane) = (0..4).find(|&l| lane_on(w, l)) {
        c.vu.set_vi(ft(w), m[lane] as u16);
    }
}
/// ISW: each masked field ← VI[it] zero-extended.
fn isw(c: &mut Ctx, w: u32) {
    let a = offset(c, w);
    let mut m = lanes_of(load_q(c, a));
    let v = c.vu.vi(ft(w)) as u32;
    for (lane, f) in m.iter_mut().enumerate() {
        if lane_on(w, lane) {
            *f = v;
        }
    }
    store_q(c, a, quad_of(m));
}
fn iaddiu(c: &mut Ctx, w: u32) {
    let v = c.vu.vi(fs(w)).wrapping_add(table::imm15(w) as u16);
    c.vu.set_vi(ft(w), v);
}
fn isubiu(c: &mut Ctx, w: u32) {
    let v = c.vu.vi(fs(w)).wrapping_sub(table::imm15(w) as u16);
    c.vu.set_vi(ft(w), v);
}
/// FCAND: VI01 ← 1 when the clipping flag AND Imm24 has a bit set.
fn fcand(c: &mut Ctx, w: u32) {
    c.vu.set_vi(1, u16::from(c.vu.clip & w & 0xFF_FFFF != 0));
}
/// FCEQ: VI01 ← 1 when the clipping flag equals Imm24.
fn fceq(c: &mut Ctx, w: u32) {
    c.vu.set_vi(1, u16::from(c.vu.clip == w & 0xFF_FFFF));
}
/// FCOR: VI01 ← 1 when the clipping flag OR Imm24 is all ones.
fn fcor(c: &mut Ctx, w: u32) {
    c.vu.set_vi(1, u16::from((c.vu.clip | w) & 0xFF_FFFF == 0xFF_FFFF));
}
/// FCGET: VI[it] ← the clipping flag's low 12 bits.
fn fcget(c: &mut Ctx, w: u32) {
    c.vu.set_vi(ft(w), (c.vu.clip & 0xFFF) as u16);
}
/// FCSET: the clipping flag ← Imm24, at the S stage.
fn fcset(c: &mut Ctx, w: u32) {
    if c.defer { c.out.clip_set = Some(w & 0xFF_FFFF) } else { c.vu.clip = w & 0xFF_FFFF }
}
fn fmand(c: &mut Ctx, w: u32) {
    c.vu.set_vi(ft(w), c.vu.mac as u16 & c.vu.vi(fs(w)));
}
fn fmeq(c: &mut Ctx, w: u32) {
    c.vu.set_vi(ft(w), u16::from(c.vu.mac as u16 == c.vu.vi(fs(w))));
}
fn fmor(c: &mut Ctx, w: u32) {
    c.vu.set_vi(ft(w), c.vu.mac as u16 | c.vu.vi(fs(w)));
}
fn fsand(c: &mut Ctx, w: u32) {
    c.vu.set_vi(ft(w), (c.vu.status & table::imm12(w)) as u16);
}
fn fseq(c: &mut Ctx, w: u32) {
    c.vu.set_vi(ft(w), u16::from(c.vu.status == table::imm12(w)));
}
fn fsor(c: &mut Ctx, w: u32) {
    c.vu.set_vi(ft(w), (c.vu.status | table::imm12(w)) as u16);
}
/// FSSET: the sticky flags ← Imm12's upper six bits, at the S stage; the lower six ignored.
fn fsset(c: &mut Ctx, w: u32) {
    let v = table::imm12(w) >> status::STICKY;
    if c.defer { c.out.sticky_set = Some(v) } else { c.vu.status = c.vu.status & 0x3F | v << status::STICKY }
}
/// MFP: VF[ft]'s masked fields ← P.
fn mfp(c: &mut Ctx, w: u32) {
    let v = c.vu.p;
    unary(c, w & !(31 << 11), |_| v);
}
/// The EFU's vector operations take VF[fs] whole, the scalar ones its fsf field; the result is handed back for P.
fn efu_vector(c: &mut Ctx, w: u32, f: fn([u32; 4]) -> u32) {
    c.out.p = Some(f(c.vu.vf(fs(w))));
}
fn efu_scalar(c: &mut Ctx, w: u32, f: fn(u32) -> u32) {
    c.out.p = Some(f(c.vu.vf(fs(w))[fsf(w)]));
}
fn esadd(c: &mut Ctx, w: u32) {
    efu_vector(c, w, efu::esadd);
}
fn ersadd(c: &mut Ctx, w: u32) {
    efu_vector(c, w, efu::ersadd);
}
fn eleng(c: &mut Ctx, w: u32) {
    efu_vector(c, w, efu::eleng);
}
fn erleng(c: &mut Ctx, w: u32) {
    efu_vector(c, w, efu::erleng);
}
fn eatanxy(c: &mut Ctx, w: u32) {
    efu_vector(c, w, efu::eatanxy);
}
fn eatanxz(c: &mut Ctx, w: u32) {
    efu_vector(c, w, efu::eatanxz);
}
fn esum(c: &mut Ctx, w: u32) {
    efu_vector(c, w, efu::esum);
}
fn esqrt(c: &mut Ctx, w: u32) {
    efu_scalar(c, w, efu::esqrt);
}
fn ersqrt(c: &mut Ctx, w: u32) {
    efu_scalar(c, w, efu::ersqrt);
}
fn ercpr(c: &mut Ctx, w: u32) {
    efu_scalar(c, w, efu::ercpr);
}
fn esin(c: &mut Ctx, w: u32) {
    efu_scalar(c, w, efu::esin);
}
fn eatan(c: &mut Ctx, w: u32) {
    efu_scalar(c, w, efu::eatan);
}
fn eexp(c: &mut Ctx, w: u32) {
    efu_scalar(c, w, efu::eexp);
}

use Class::{Branch, Callms, Div, Efu, External, Fcheck, FlagSet, Fmac, Ialu, WaitP, WaitQ};

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
    // Micro mode alone (§4.3), with the latencies of §7.4.
    e("lq", lq, Fmac, false), e("sq", sq, Fmac, false), e("ilw", ilw, Fmac, false), e("isw", isw, Fmac, false),
    e("iaddiu", iaddiu, Ialu, false), e("isubiu", isubiu, Ialu, false),
    e("fcand", fcand, Fcheck, false), e("fceq", fceq, Fcheck, false), e("fcor", fcor, Fcheck, false), e("fcget", fcget, Fcheck, false),
    e("fmand", fmand, Fcheck, false), e("fmeq", fmeq, Fcheck, false), e("fmor", fmor, Fcheck, false),
    e("fsand", fsand, Fcheck, false), e("fseq", fseq, Fcheck, false), e("fsor", fsor, Fcheck, false),
    e("fcset", fcset, FlagSet, false), e("fsset", fsset, FlagSet, false),
    e("b", nothing, Branch, false), e("bal", nothing, Branch, false), e("jr", nothing, Branch, false), e("jalr", nothing, Branch, false),
    e("ibeq", nothing, Branch, false), e("ibne", nothing, Branch, false), e("ibltz", nothing, Branch, false),
    e("ibgtz", nothing, Branch, false), e("iblez", nothing, Branch, false), e("ibgez", nothing, Branch, false),
    e("mfp", mfp, Fmac, false), e("waitp", nothing, WaitP, false),
    e("esadd", esadd, Efu(11), false), e("ersadd", ersadd, Efu(18), false), e("eleng", eleng, Efu(18), false), e("erleng", erleng, Efu(24), false),
    e("eatanxy", eatanxy, Efu(54), false), e("eatanxz", eatanxz, Efu(54), false), e("esum", esum, Efu(12), false),
    e("esqrt", esqrt, Efu(12), false), e("ersqrt", ersqrt, Efu(18), false), e("ercpr", ercpr, Efu(12), false),
    e("esin", esin, Efu(29), false), e("eatan", eatan, Efu(54), false), e("eexp", eexp, Efu(44), false),
    e("xgkick", nothing, External, false), e("xtop", nothing, External, false), e("xitop", nothing, External, false),
];

const NONE: u16 = u16::MAX;

/// The TABLE index of the operation in each decoder slot, SPECIAL1 then SPECIAL2 then LOWER.
fn slots() -> &'static [u16] {
    static MAP: OnceLock<Box<[u16]>> = OnceLock::new();
    MAP.get_or_init(|| {
        let index = |o: &Option<Operation>| o.as_ref().map_or(NONE, |o| TABLE.iter().position(|e| e.name == o.name).expect("an operation without an entry") as u16);
        SPECIAL1.iter().chain(SPECIAL2.iter()).chain(LOWER.iter()).map(index).collect()
    })
}

/// The decoder slot of an operation, from its address in one of the three tables (Sapphire_Speed.md section 52).
fn slot(o: &Operation) -> usize {
    let at = std::ptr::from_ref(o) as usize;
    let size = std::mem::size_of::<Option<Operation>>();
    let mut first = 0;
    for (base, len) in [(SPECIAL1.as_ptr(), SPECIAL1.len()), (SPECIAL2.as_ptr(), SPECIAL2.len()), (LOWER.as_ptr(), LOWER.len())] {
        let from = at.wrapping_sub(base as usize);
        if from < len * size {
            return first + from / size;
        }
        first += len;
    }
    panic!("an operation outside the decoder's tables")
}

/// The entry for a decoded operation.
pub fn entry(o: &Operation) -> &'static Entry {
    &TABLE[slots()[slot(o)] as usize]
}

/// The entry for the operation in a word's low 25 bits, through the decoder alone.
pub fn of_word(w: u32) -> Option<(&'static Operation, &'static Entry)> {
    table::operation(w).map(|o| (o, entry(o)))
}

/// The entry for a lower word in micro mode, the micro-only operations included.
pub fn of_lower(w: u32) -> Option<(&'static Operation, &'static Entry)> {
    table::lower(w).map(|o| (o, entry(o)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_decoder_operation_maps_to_its_entry_by_name() {
        let all = SPECIAL1.iter().chain(SPECIAL2.iter()).chain(LOWER.iter()).filter_map(Option::as_ref);
        let mut seen = 0;
        for o in all {
            let by_name = TABLE.iter().find(|e| e.name == o.name).expect("a table entry");
            assert!(std::ptr::eq(entry(o), by_name), "{}", o.name);
            seen += 1;
        }
        assert!(seen > 100);
    }
}
