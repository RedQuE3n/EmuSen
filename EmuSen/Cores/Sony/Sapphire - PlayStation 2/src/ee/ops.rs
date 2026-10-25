//! The integer instructions' meanings, one function per mnemonic as `emusen_mips::r5900` names it, each written from
//! its Operation section in the EE Core Instruction Set Manual (Sapphire_Native.md §16.2 has the readings the
//! manual leaves open). Nothing here decodes: the table is looked up by the decoder's name, so it holds no encoding.

use std::collections::HashMap;
use std::sync::OnceLock;

use emusen_mips::r5900::{self, Op as Decoded};

use super::timing::{Category, HILO0, HILO1, Issue, SA, rows};
use super::{Bus, Ee, Exception, cop0, status};

pub(super) enum Stop {
    Exception(Exception),
    Halt,
}

type Run = fn(&mut Ee, &mut dyn Bus, u32, u32) -> Result<(), Stop>;

/// The registers an instruction reads or writes, by the field naming them.
pub(super) mod uses {
    pub const RS: u8 = 1;
    pub const RT: u8 = 2;
    pub const RD: u8 = 4;
    pub const RA: u8 = 8;
    pub const HILO0: u8 = 16;
    pub const HILO1: u8 = 32;
    pub const SA: u8 = 64;
}
use uses::{HILO0 as H0, HILO1 as H1, RA, RD, RS, RT, SA as S};

pub(super) struct Op {
    pub name: &'static str,
    pub run: Run,
    pub category: Category,
    pub row: usize,
    pub reads: u8,
    pub writes: u8,
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
fn sa(w: u32) -> u32 {
    w >> 6 & 31
}
fn simm(w: u32) -> u64 {
    w as u16 as i16 as i64 as u64
}
fn uimm(w: u32) -> u64 {
    (w & 0xFFFF) as u64
}
fn sext32(v: u32) -> u64 {
    v as i32 as i64 as u64
}
fn exc(e: Exception) -> Result<(), Stop> {
    Err(Stop::Exception(e))
}

type R = Result<(), Stop>;

// Arithmetic: a word operation computes on bits 31..0 and writes the sign-extended result to bits 63..0.
fn add(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    match (e.get(rs(w)) as i32).checked_add(e.get(rt(w)) as i32) {
        Some(v) => {
            e.set(rd(w), v as i64 as u64);
            Ok(())
        }
        None => exc(Exception::Overflow),
    }
}
fn addu(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), sext32((e.get(rs(w)) as u32).wrapping_add(e.get(rt(w)) as u32)));
    Ok(())
}
fn sub(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    match (e.get(rs(w)) as i32).checked_sub(e.get(rt(w)) as i32) {
        Some(v) => {
            e.set(rd(w), v as i64 as u64);
            Ok(())
        }
        None => exc(Exception::Overflow),
    }
}
fn subu(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), sext32((e.get(rs(w)) as u32).wrapping_sub(e.get(rt(w)) as u32)));
    Ok(())
}
fn addi(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    match (e.get(rs(w)) as i32).checked_add(simm(w) as i32) {
        Some(v) => {
            e.set(rt(w), v as i64 as u64);
            Ok(())
        }
        None => exc(Exception::Overflow),
    }
}
fn addiu(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rt(w), sext32((e.get(rs(w)) as u32).wrapping_add(simm(w) as u32)));
    Ok(())
}
fn dadd(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    match (e.get(rs(w)) as i64).checked_add(e.get(rt(w)) as i64) {
        Some(v) => {
            e.set(rd(w), v as u64);
            Ok(())
        }
        None => exc(Exception::Overflow),
    }
}
fn daddu(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), e.get(rs(w)).wrapping_add(e.get(rt(w))));
    Ok(())
}
fn dsub(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    match (e.get(rs(w)) as i64).checked_sub(e.get(rt(w)) as i64) {
        Some(v) => {
            e.set(rd(w), v as u64);
            Ok(())
        }
        None => exc(Exception::Overflow),
    }
}
fn dsubu(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), e.get(rs(w)).wrapping_sub(e.get(rt(w))));
    Ok(())
}
fn daddi(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    match (e.get(rs(w)) as i64).checked_add(simm(w) as i64) {
        Some(v) => {
            e.set(rt(w), v as u64);
            Ok(())
        }
        None => exc(Exception::Overflow),
    }
}
fn daddiu(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rt(w), e.get(rs(w)).wrapping_add(simm(w)));
    Ok(())
}

// Logic, comparison and conditional moves on bits 63..0.
fn and(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), e.get(rs(w)) & e.get(rt(w)));
    Ok(())
}
fn or(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), e.get(rs(w)) | e.get(rt(w)));
    Ok(())
}
fn xor(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), e.get(rs(w)) ^ e.get(rt(w)));
    Ok(())
}
fn nor(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), !(e.get(rs(w)) | e.get(rt(w))));
    Ok(())
}
fn andi(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rt(w), e.get(rs(w)) & uimm(w));
    Ok(())
}
fn ori(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rt(w), e.get(rs(w)) | uimm(w));
    Ok(())
}
fn xori(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rt(w), e.get(rs(w)) ^ uimm(w));
    Ok(())
}
fn lui(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rt(w), sext32(w << 16));
    Ok(())
}
fn slt(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), ((e.get(rs(w)) as i64) < e.get(rt(w)) as i64) as u64);
    Ok(())
}
fn sltu(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), (e.get(rs(w)) < e.get(rt(w))) as u64);
    Ok(())
}
fn slti(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rt(w), ((e.get(rs(w)) as i64) < simm(w) as i64) as u64);
    Ok(())
}
fn sltiu(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rt(w), (e.get(rs(w)) < simm(w)) as u64);
    Ok(())
}
fn movz(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    if e.get(rt(w)) == 0 {
        e.set(rd(w), e.get(rs(w)));
    }
    Ok(())
}
fn movn(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    if e.get(rt(w)) != 0 {
        e.set(rd(w), e.get(rs(w)));
    }
    Ok(())
}

// Shifts: the word forms on bits 31..0, the doubleword forms on 63..0.
fn sll(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), sext32((e.get(rt(w)) as u32) << sa(w)));
    Ok(())
}
fn srl(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), sext32((e.get(rt(w)) as u32) >> sa(w)));
    Ok(())
}
fn sra(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), ((e.get(rt(w)) as i32) >> sa(w)) as i64 as u64);
    Ok(())
}
fn sllv(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), sext32((e.get(rt(w)) as u32) << (e.get(rs(w)) & 31)));
    Ok(())
}
fn srlv(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), sext32((e.get(rt(w)) as u32) >> (e.get(rs(w)) & 31)));
    Ok(())
}
fn srav(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), ((e.get(rt(w)) as i32) >> (e.get(rs(w)) & 31)) as i64 as u64);
    Ok(())
}
fn dsll(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), e.get(rt(w)) << sa(w));
    Ok(())
}
fn dsrl(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), e.get(rt(w)) >> sa(w));
    Ok(())
}
fn dsra(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), ((e.get(rt(w)) as i64) >> sa(w)) as u64);
    Ok(())
}
fn dsll32(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), e.get(rt(w)) << (sa(w) + 32));
    Ok(())
}
fn dsrl32(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), e.get(rt(w)) >> (sa(w) + 32));
    Ok(())
}
fn dsra32(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), ((e.get(rt(w)) as i64) >> (sa(w) + 32)) as u64);
    Ok(())
}
fn dsllv(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), e.get(rt(w)) << (e.get(rs(w)) & 63));
    Ok(())
}
fn dsrlv(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), e.get(rt(w)) >> (e.get(rs(w)) & 63));
    Ok(())
}
fn dsrav(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), ((e.get(rt(w)) as i64) >> (e.get(rs(w)) & 63)) as u64);
    Ok(())
}

// HI and LO: pipeline 0 owns bits 63..0 (HI0, LO0), pipeline 1 bits 127..64 (HI1, LO1).
const LOW: u128 = u64::MAX as u128;
fn half(v: u128, upper: bool) -> u64 {
    if upper { (v >> 64) as u64 } else { v as u64 }
}
fn put(v: &mut u128, upper: bool, x: u64) {
    *v = if upper { *v & LOW | (x as u128) << 64 } else { *v & !LOW | x as u128 };
}
fn mfhi(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), half(e.hi, false));
    Ok(())
}
fn mflo(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), half(e.lo, false));
    Ok(())
}
fn mthi(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let v = e.get(rs(w));
    put(&mut e.hi, false, v);
    Ok(())
}
fn mtlo(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let v = e.get(rs(w));
    put(&mut e.lo, false, v);
    Ok(())
}
fn mfhi1(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), half(e.hi, true));
    Ok(())
}
fn mflo1(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), half(e.lo, true));
    Ok(())
}
fn mthi1(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let v = e.get(rs(w));
    put(&mut e.hi, true, v);
    Ok(())
}
fn mtlo1(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let v = e.get(rs(w));
    put(&mut e.lo, true, v);
    Ok(())
}

/// The 64-bit product (or accumulated sum) into LO, HI and rd, as the multiply forms' Operation sections write it.
fn product(e: &mut Ee, w: u32, upper: bool, prod: u64) {
    put(&mut e.lo, upper, sext32(prod as u32));
    put(&mut e.hi, upper, sext32((prod >> 32) as u32));
    e.set(rd(w), sext32(prod as u32));
}
fn signed_product(e: &Ee, w: u32) -> u64 {
    (e.get(rs(w)) as i32 as i64).wrapping_mul(e.get(rt(w)) as i32 as i64) as u64
}
fn unsigned_product(e: &Ee, w: u32) -> u64 {
    (e.get(rs(w)) as u32 as u64) * (e.get(rt(w)) as u32 as u64)
}
fn accumulator(e: &Ee, upper: bool) -> u64 {
    (half(e.hi, upper) as u32 as u64) << 32 | half(e.lo, upper) as u32 as u64
}
fn mult(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let p = signed_product(e, w);
    product(e, w, false, p);
    Ok(())
}
fn multu(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let p = unsigned_product(e, w);
    product(e, w, false, p);
    Ok(())
}
fn mult1(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let p = signed_product(e, w);
    product(e, w, true, p);
    Ok(())
}
fn multu1(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let p = unsigned_product(e, w);
    product(e, w, true, p);
    Ok(())
}
fn madd(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let p = accumulator(e, false).wrapping_add(signed_product(e, w));
    product(e, w, false, p);
    Ok(())
}
fn maddu(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let p = accumulator(e, false).wrapping_add(unsigned_product(e, w));
    product(e, w, false, p);
    Ok(())
}
fn madd1(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let p = accumulator(e, true).wrapping_add(signed_product(e, w));
    product(e, w, true, p);
    Ok(())
}
fn maddu1(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let p = accumulator(e, true).wrapping_add(unsigned_product(e, w));
    product(e, w, true, p);
    Ok(())
}

/// A signed word division; a zero divisor's result, which the manual leaves undefined, is Sapphire_Disputes.md D-6's.
fn quotient_signed(n: i32, d: i32) -> (u32, u32) {
    if d == 0 { (if n < 0 { 1 } else { u32::MAX }, n as u32) } else { (n.wrapping_div(d) as u32, n.wrapping_rem(d) as u32) }
}
fn quotient_unsigned(n: u32, d: u32) -> (u32, u32) {
    match (n.checked_div(d), n.checked_rem(d)) {
        (Some(q), Some(r)) => (q, r),
        _ => (u32::MAX, n),
    }
}
fn divide(e: &mut Ee, upper: bool, (q, r): (u32, u32)) {
    put(&mut e.lo, upper, sext32(q));
    put(&mut e.hi, upper, sext32(r));
}
fn div(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let qr = quotient_signed(e.get(rs(w)) as i32, e.get(rt(w)) as i32);
    divide(e, false, qr);
    Ok(())
}
fn divu(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let qr = quotient_unsigned(e.get(rs(w)) as u32, e.get(rt(w)) as u32);
    divide(e, false, qr);
    Ok(())
}
fn div1(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let qr = quotient_signed(e.get(rs(w)) as i32, e.get(rt(w)) as i32);
    divide(e, true, qr);
    Ok(())
}
fn divu1(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let qr = quotient_unsigned(e.get(rs(w)) as u32, e.get(rt(w)) as u32);
    divide(e, true, qr);
    Ok(())
}

// SA, the funnel shift's amount, kept as the byte count MFSA reads back (Sapphire_Disputes.md D-9).
fn mfsa(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.set(rd(w), e.sa);
    Ok(())
}
fn mtsa(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.sa = e.get(rs(w)) & 15;
    Ok(())
}
fn mtsab(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.sa = (e.get(rs(w)) ^ uimm(w)) & 15;
    Ok(())
}
fn mtsah(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    e.sa = ((e.get(rs(w)) ^ uimm(w)) & 7) * 2;
    Ok(())
}

// Branches and jumps: `pc` is the branch's own address; its delay slot is pc + 4.
fn target(pc: u32, w: u32) -> u32 {
    pc.wrapping_add(4).wrapping_add((simm(w) as u32) << 2)
}
fn cond(e: &mut Ee, w: u32, pc: u32, taken: bool) -> R {
    e.delay_slot();
    if taken {
        e.branch(target(pc, w));
    }
    Ok(())
}
fn likely(e: &mut Ee, w: u32, pc: u32, taken: bool) -> R {
    if taken {
        e.delay_slot();
        e.branch(target(pc, w));
    } else {
        e.nullify();
    }
    Ok(())
}
fn link(e: &mut Ee, r: usize, pc: u32) {
    e.set(r, pc.wrapping_add(8) as u64);
}
fn beq(e: &mut Ee, _: &mut dyn Bus, w: u32, pc: u32) -> R {
    let t = e.get(rs(w)) == e.get(rt(w));
    cond(e, w, pc, t)
}
fn bne(e: &mut Ee, _: &mut dyn Bus, w: u32, pc: u32) -> R {
    let t = e.get(rs(w)) != e.get(rt(w));
    cond(e, w, pc, t)
}
fn blez(e: &mut Ee, _: &mut dyn Bus, w: u32, pc: u32) -> R {
    let t = e.get(rs(w)) as i64 <= 0;
    cond(e, w, pc, t)
}
fn bgtz(e: &mut Ee, _: &mut dyn Bus, w: u32, pc: u32) -> R {
    let t = e.get(rs(w)) as i64 > 0;
    cond(e, w, pc, t)
}
fn bltz(e: &mut Ee, _: &mut dyn Bus, w: u32, pc: u32) -> R {
    let t = (e.get(rs(w)) as i64) < 0;
    cond(e, w, pc, t)
}
fn bgez(e: &mut Ee, _: &mut dyn Bus, w: u32, pc: u32) -> R {
    let t = e.get(rs(w)) as i64 >= 0;
    cond(e, w, pc, t)
}
fn bltzal(e: &mut Ee, _: &mut dyn Bus, w: u32, pc: u32) -> R {
    let t = (e.get(rs(w)) as i64) < 0;
    link(e, 31, pc);
    cond(e, w, pc, t)
}
fn bgezal(e: &mut Ee, _: &mut dyn Bus, w: u32, pc: u32) -> R {
    let t = e.get(rs(w)) as i64 >= 0;
    link(e, 31, pc);
    cond(e, w, pc, t)
}
fn beql(e: &mut Ee, _: &mut dyn Bus, w: u32, pc: u32) -> R {
    let t = e.get(rs(w)) == e.get(rt(w));
    likely(e, w, pc, t)
}
fn bnel(e: &mut Ee, _: &mut dyn Bus, w: u32, pc: u32) -> R {
    let t = e.get(rs(w)) != e.get(rt(w));
    likely(e, w, pc, t)
}
fn blezl(e: &mut Ee, _: &mut dyn Bus, w: u32, pc: u32) -> R {
    let t = e.get(rs(w)) as i64 <= 0;
    likely(e, w, pc, t)
}
fn bgtzl(e: &mut Ee, _: &mut dyn Bus, w: u32, pc: u32) -> R {
    let t = e.get(rs(w)) as i64 > 0;
    likely(e, w, pc, t)
}
fn bltzl(e: &mut Ee, _: &mut dyn Bus, w: u32, pc: u32) -> R {
    let t = (e.get(rs(w)) as i64) < 0;
    likely(e, w, pc, t)
}
fn bgezl(e: &mut Ee, _: &mut dyn Bus, w: u32, pc: u32) -> R {
    let t = e.get(rs(w)) as i64 >= 0;
    likely(e, w, pc, t)
}
fn bltzall(e: &mut Ee, _: &mut dyn Bus, w: u32, pc: u32) -> R {
    let t = (e.get(rs(w)) as i64) < 0;
    link(e, 31, pc);
    likely(e, w, pc, t)
}
fn bgezall(e: &mut Ee, _: &mut dyn Bus, w: u32, pc: u32) -> R {
    let t = e.get(rs(w)) as i64 >= 0;
    link(e, 31, pc);
    likely(e, w, pc, t)
}
fn j(e: &mut Ee, _: &mut dyn Bus, w: u32, pc: u32) -> R {
    e.delay_slot();
    e.branch(pc.wrapping_add(4) & 0xF000_0000 | (w & 0x03FF_FFFF) << 2);
    Ok(())
}
fn jal(e: &mut Ee, b: &mut dyn Bus, w: u32, pc: u32) -> R {
    link(e, 31, pc);
    j(e, b, w, pc)
}
fn jr(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let t = e.get(rs(w)) as u32;
    e.delay_slot();
    e.branch(t);
    Ok(())
}
fn jalr(e: &mut Ee, _: &mut dyn Bus, w: u32, pc: u32) -> R {
    let t = e.get(rs(w)) as u32;
    link(e, rd(w), pc);
    e.delay_slot();
    e.branch(t);
    Ok(())
}

// Exceptions raised by instruction.
fn syscall(_: &mut Ee, _: &mut dyn Bus, _: u32, _: u32) -> R {
    exc(Exception::Syscall)
}
fn break_(_: &mut Ee, _: &mut dyn Bus, _: u32, _: u32) -> R {
    exc(Exception::Break)
}
fn trap_if(t: bool) -> R {
    if t { exc(Exception::Trap) } else { Ok(()) }
}
fn tge(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    trap_if(e.get(rs(w)) as i64 >= e.get(rt(w)) as i64)
}
fn tgeu(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    trap_if(e.get(rs(w)) >= e.get(rt(w)))
}
fn tlt(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    trap_if((e.get(rs(w)) as i64) < e.get(rt(w)) as i64)
}
fn tltu(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    trap_if(e.get(rs(w)) < e.get(rt(w)))
}
fn teq(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    trap_if(e.get(rs(w)) == e.get(rt(w)))
}
fn tne(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    trap_if(e.get(rs(w)) != e.get(rt(w)))
}
fn tgei(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    trap_if(e.get(rs(w)) as i64 >= simm(w) as i64)
}
fn tgeiu(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    trap_if(e.get(rs(w)) >= simm(w))
}
fn tlti(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    trap_if((e.get(rs(w)) as i64) < simm(w) as i64)
}
fn tltiu(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    trap_if(e.get(rs(w)) < simm(w))
}
fn teqi(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    trap_if(e.get(rs(w)) == simm(w))
}
fn tnei(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    trap_if(e.get(rs(w)) != simm(w))
}
fn nothing(_: &mut Ee, _: &mut dyn Bus, _: u32, _: u32) -> R {
    Ok(())
}

// Loads and stores. The address is bits 31..0 of base + offset; a misaligned natural access raises an address error.
fn address(e: &Ee, w: u32) -> u32 {
    e.get(rs(w)).wrapping_add(simm(w)) as u32
}
fn physical(e: &Ee, v: u32, align: u32, store: bool) -> Result<u32, Stop> {
    if v & (align - 1) != 0 {
        return Err(Stop::Exception(if store { Exception::AddressStore(v) } else { Exception::AddressLoad(v) }));
    }
    e.translate(v, store).map_err(Stop::Exception)
}
fn load(e: &mut Ee, b: &mut dyn Bus, w: u32, bytes: u32, signed: bool) -> R {
    let p = physical(e, address(e, w), bytes, false)?;
    let v = b.read(p, bytes as usize) as u64;
    let bits = bytes * 8;
    let v = if bits == 64 {
        v
    } else if signed {
        ((v << (64 - bits)) as i64 >> (64 - bits)) as u64
    } else {
        v
    };
    e.set(rt(w), v);
    Ok(())
}
fn lb(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    load(e, b, w, 1, true)
}
fn lbu(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    load(e, b, w, 1, false)
}
fn lh(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    load(e, b, w, 2, true)
}
fn lhu(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    load(e, b, w, 2, false)
}
fn lw(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    load(e, b, w, 4, true)
}
fn lwu(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    load(e, b, w, 4, false)
}
fn ld(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    load(e, b, w, 8, false)
}
fn lq(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    let p = physical(e, address(e, w) & !15, 16, false)?;
    let v = b.read(p, 16);
    if rt(w) != 0 {
        e.gpr[rt(w)] = v;
    }
    Ok(())
}
fn store(e: &mut Ee, b: &mut dyn Bus, w: u32, bytes: u32) -> R {
    let p = physical(e, address(e, w), bytes, true)?;
    b.write(p, bytes as usize, e.get(rt(w)) as u128);
    Ok(())
}
fn sb(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    store(e, b, w, 1)
}
fn sh(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    store(e, b, w, 2)
}
fn sw(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    store(e, b, w, 4)
}
fn sd(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    store(e, b, w, 8)
}
fn sq(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    let p = physical(e, address(e, w) & !15, 16, true)?;
    b.write(p, 16, e.gpr[rt(w)]);
    Ok(())
}

/// The unaligned forms: the aligned unit of `n` bytes holding the address, merged with the register as the manual's
/// byte tables show (little-endian: LWL/LDL fill from the high end, LWR/LDR from the low end).
fn unit(e: &Ee, b: &mut dyn Bus, w: u32, n: u32, store: bool) -> Result<(u32, u32, u64), Stop> {
    let v = address(e, w);
    let p = physical(e, v & !(n - 1), n, store)?;
    Ok((p, v & (n - 1), b.read(p, n as usize) as u64))
}
fn mask(bytes: u32) -> u64 {
    if bytes >= 8 { u64::MAX } else { (1u64 << (bytes * 8)) - 1 }
}
fn lwl(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    let (_, k, m) = unit(e, b, w, 4, false)?;
    let shift = (3 - k) * 8;
    let reg = e.get(rt(w)) as u32;
    let v = (m as u32) << shift | reg & mask(shift / 8) as u32;
    e.set(rt(w), sext32(v));
    Ok(())
}
fn lwr(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    let (_, k, m) = unit(e, b, w, 4, false)?;
    let shift = k * 8;
    let reg = e.get(rt(w));
    let low = (m as u32) >> shift | (reg as u32) & !(mask(4 - k) as u32);
    let v = if k == 0 { sext32(low) } else { reg & !0xFFFF_FFFF | low as u64 };
    e.set(rt(w), v);
    Ok(())
}
fn ldl(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    let (_, k, m) = unit(e, b, w, 8, false)?;
    let shift = (7 - k) * 8;
    let v = m << shift | e.get(rt(w)) & mask(shift / 8);
    e.set(rt(w), v);
    Ok(())
}
fn ldr(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    let (_, k, m) = unit(e, b, w, 8, false)?;
    let shift = k * 8;
    let v = m >> shift | e.get(rt(w)) & !mask(8 - k);
    e.set(rt(w), v);
    Ok(())
}
fn swl(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    let (p, k, m) = unit(e, b, w, 4, true)?;
    let shift = (3 - k) * 8;
    let keep = !(mask(4) as u32 >> shift);
    let v = (m as u32) & keep | (e.get(rt(w)) as u32) >> shift;
    b.write(p, 4, v as u128);
    Ok(())
}
fn swr(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    let (p, k, m) = unit(e, b, w, 4, true)?;
    let shift = k * 8;
    let keep = mask(k) as u32;
    let v = (m as u32) & keep | (e.get(rt(w)) as u32) << shift;
    b.write(p, 4, v as u128);
    Ok(())
}
fn sdl(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    let (p, k, m) = unit(e, b, w, 8, true)?;
    let shift = (7 - k) * 8;
    let v = m & !(u64::MAX >> shift) | e.get(rt(w)) >> shift;
    b.write(p, 8, v as u128);
    Ok(())
}
fn sdr(e: &mut Ee, b: &mut dyn Bus, w: u32, _: u32) -> R {
    let (p, k, m) = unit(e, b, w, 8, true)?;
    let shift = k * 8;
    let v = m & mask(k) | e.get(rt(w)) << shift;
    b.write(p, 8, v as u128);
    Ok(())
}

// COP0, as far as exceptions need it: the moves, ERET, EI and DI; the rest is S7's.
fn cop0_usable(e: &Ee) -> R {
    if e.kernel() || e.status() & status::CU0 != 0 { Ok(()) } else { exc(Exception::Unusable(0)) }
}
/// The bits MTC0 may change: Status's r/w fields (§3.2's table); Cause, BadVAddr and PRId are read-only.
fn writable(r: usize) -> u32 {
    match r {
        cop0::STATUS => 0xF0C7_9C1F,
        cop0::CAUSE | cop0::BADVADDR | cop0::PRID => 0,
        _ => u32::MAX,
    }
}
fn mfc0(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    cop0_usable(e)?;
    e.set(rt(w), sext32(e.cop0[rd(w)]));
    Ok(())
}
fn mtc0(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    cop0_usable(e)?;
    let (r, m) = (rd(w), writable(rd(w)));
    e.cop0[r] = e.cop0[r] & !m | e.get(rt(w)) as u32 & m;
    Ok(())
}
fn eret(e: &mut Ee, _: &mut dyn Bus, _: u32, _: u32) -> R {
    cop0_usable(e)?;
    let s = e.cop0[cop0::STATUS];
    if s & status::ERL != 0 {
        e.cop0[cop0::STATUS] = s & !status::ERL;
        e.jump_now(e.cop0[cop0::ERROREPC]);
    } else {
        e.cop0[cop0::STATUS] = s & !status::EXL;
        e.jump_now(e.cop0[cop0::EPC]);
    }
    Ok(())
}
fn eidi(e: &mut Ee, on: bool) -> R {
    cop0_usable(e)?;
    if e.kernel() || e.status() & status::EDI != 0 {
        let s = &mut e.cop0[cop0::STATUS];
        *s = if on { *s | status::EIE } else { *s & !status::EIE };
    }
    Ok(())
}
fn ei(e: &mut Ee, _: &mut dyn Bus, _: u32, _: u32) -> R {
    eidi(e, true)
}
fn di(e: &mut Ee, _: &mut dyn Bus, _: u32, _: u32) -> R {
    eidi(e, false)
}

/// A reserved encoding (the decoder's `None`): the reserved instruction exception.
fn reserved(_: &mut Ee, _: &mut dyn Bus, _: u32, _: u32) -> R {
    exc(Exception::Reserved)
}
/// An instruction a later stage executes; a coprocessor's raises its unusable exception first, as §4.2.14 says.
fn later(e: &mut Ee, _: &mut dyn Bus, w: u32, _: u32) -> R {
    let n = match w >> 26 {
        0x10 | 0x2F => 0,
        0x11 | 0x31 | 0x39 => 1,
        0x12 | 0x36 | 0x3E => 2,
        _ => return Err(Stop::Halt),
    };
    let usable = if n == 0 { e.kernel() || e.status() & status::CU0 != 0 } else { e.status() >> (28 + n) & 1 != 0 };
    if usable { Err(Stop::Halt) } else { exc(Exception::Unusable(n)) }
}

const fn op(name: &'static str, run: Run, category: Category, row: usize, reads: u8, writes: u8) -> Op {
    Op { name, run, category, row, reads, writes }
}

use Category::{Alu, Branch, Cop0, Eret as EretC, LoadStore, Mac0, Mac1, SaOperate, Sync};
use rows::{ALU, BRANCH, CMOVE, DIV as RDIV, DIV1 as RDIV1, HILO, LOAD, MADD as RMADD, MADD1 as RMADD1, MULT as RMULT, MULT1 as RMULT1, SHIFT, STORE};

/// Every integer instruction of S6, by the decoder's name, with its category (Table 1-1) and its row of §2.12.
static TABLE: &[Op] = &[
    op("reserved", reserved, Alu, ALU, 0, 0),
    op("later", later, Alu, ALU, 0, 0),
    op("add", add, Alu, ALU, RS | RT, RD),
    op("addu", addu, Alu, ALU, RS | RT, RD),
    op("sub", sub, Alu, ALU, RS | RT, RD),
    op("subu", subu, Alu, ALU, RS | RT, RD),
    op("addi", addi, Alu, ALU, RS, RT),
    op("addiu", addiu, Alu, ALU, RS, RT),
    op("dadd", dadd, Alu, ALU, RS | RT, RD),
    op("daddu", daddu, Alu, ALU, RS | RT, RD),
    op("dsub", dsub, Alu, ALU, RS | RT, RD),
    op("dsubu", dsubu, Alu, ALU, RS | RT, RD),
    op("daddi", daddi, Alu, ALU, RS, RT),
    op("daddiu", daddiu, Alu, ALU, RS, RT),
    op("and", and, Alu, ALU, RS | RT, RD),
    op("or", or, Alu, ALU, RS | RT, RD),
    op("xor", xor, Alu, ALU, RS | RT, RD),
    op("nor", nor, Alu, ALU, RS | RT, RD),
    op("andi", andi, Alu, ALU, RS, RT),
    op("ori", ori, Alu, ALU, RS, RT),
    op("xori", xori, Alu, ALU, RS, RT),
    op("lui", lui, Alu, SHIFT, 0, RT),
    op("slt", slt, Alu, ALU, RS | RT, RD),
    op("sltu", sltu, Alu, ALU, RS | RT, RD),
    op("slti", slti, Alu, ALU, RS, RT),
    op("sltiu", sltiu, Alu, ALU, RS, RT),
    op("movz", movz, Alu, CMOVE, RS | RT | RD, RD),
    op("movn", movn, Alu, CMOVE, RS | RT | RD, RD),
    op("sll", sll, Alu, SHIFT, RT, RD),
    op("srl", srl, Alu, SHIFT, RT, RD),
    op("sra", sra, Alu, SHIFT, RT, RD),
    op("sllv", sllv, Alu, SHIFT, RS | RT, RD),
    op("srlv", srlv, Alu, SHIFT, RS | RT, RD),
    op("srav", srav, Alu, SHIFT, RS | RT, RD),
    op("dsll", dsll, Alu, SHIFT, RT, RD),
    op("dsrl", dsrl, Alu, SHIFT, RT, RD),
    op("dsra", dsra, Alu, SHIFT, RT, RD),
    op("dsll32", dsll32, Alu, SHIFT, RT, RD),
    op("dsrl32", dsrl32, Alu, SHIFT, RT, RD),
    op("dsra32", dsra32, Alu, SHIFT, RT, RD),
    op("dsllv", dsllv, Alu, SHIFT, RS | RT, RD),
    op("dsrlv", dsrlv, Alu, SHIFT, RS | RT, RD),
    op("dsrav", dsrav, Alu, SHIFT, RS | RT, RD),
    op("mfhi", mfhi, Mac0, HILO, H0, RD),
    op("mflo", mflo, Mac0, HILO, H0, RD),
    op("mthi", mthi, Mac0, HILO, RS, H0),
    op("mtlo", mtlo, Mac0, HILO, RS, H0),
    op("mfhi1", mfhi1, Mac1, HILO, H1, RD),
    op("mflo1", mflo1, Mac1, HILO, H1, RD),
    op("mthi1", mthi1, Mac1, HILO, RS, H1),
    op("mtlo1", mtlo1, Mac1, HILO, RS, H1),
    op("mult", mult, Mac0, RMULT, RS | RT, RD | H0),
    op("multu", multu, Mac0, RMULT, RS | RT, RD | H0),
    op("mult1", mult1, Mac1, RMULT1, RS | RT, RD | H1),
    op("multu1", multu1, Mac1, RMULT1, RS | RT, RD | H1),
    op("madd", madd, Mac0, RMADD, RS | RT, RD | H0),
    op("maddu", maddu, Mac0, RMADD, RS | RT, RD | H0),
    op("madd1", madd1, Mac1, RMADD1, RS | RT, RD | H1),
    op("maddu1", maddu1, Mac1, RMADD1, RS | RT, RD | H1),
    op("div", div, Mac0, RDIV, RS | RT, H0),
    op("divu", divu, Mac0, RDIV, RS | RT, H0),
    op("div1", div1, Mac1, RDIV1, RS | RT, H1),
    op("divu1", divu1, Mac1, RDIV1, RS | RT, H1),
    op("mfsa", mfsa, SaOperate, ALU, S, RD),
    op("mtsa", mtsa, SaOperate, ALU, RS, S),
    op("mtsab", mtsab, SaOperate, ALU, RS, S),
    op("mtsah", mtsah, SaOperate, ALU, RS, S),
    op("beq", beq, Branch, BRANCH, RS | RT, 0),
    op("bne", bne, Branch, BRANCH, RS | RT, 0),
    op("blez", blez, Branch, BRANCH, RS, 0),
    op("bgtz", bgtz, Branch, BRANCH, RS, 0),
    op("bltz", bltz, Branch, BRANCH, RS, 0),
    op("bgez", bgez, Branch, BRANCH, RS, 0),
    op("bltzal", bltzal, Branch, BRANCH, RS, RA),
    op("bgezal", bgezal, Branch, BRANCH, RS, RA),
    op("beql", beql, Branch, BRANCH, RS | RT, 0),
    op("bnel", bnel, Branch, BRANCH, RS | RT, 0),
    op("blezl", blezl, Branch, BRANCH, RS, 0),
    op("bgtzl", bgtzl, Branch, BRANCH, RS, 0),
    op("bltzl", bltzl, Branch, BRANCH, RS, 0),
    op("bgezl", bgezl, Branch, BRANCH, RS, 0),
    op("bltzall", bltzall, Branch, BRANCH, RS, RA),
    op("bgezall", bgezall, Branch, BRANCH, RS, RA),
    op("j", j, Branch, BRANCH, 0, 0),
    op("jal", jal, Branch, BRANCH, 0, RA),
    op("jr", jr, Branch, BRANCH, RS, 0),
    op("jalr", jalr, Branch, BRANCH, RS, RD),
    op("syscall", syscall, Alu, ALU, 0, 0),
    op("break", break_, Alu, ALU, 0, 0),
    op("tge", tge, Alu, ALU, RS | RT, 0),
    op("tgeu", tgeu, Alu, ALU, RS | RT, 0),
    op("tlt", tlt, Alu, ALU, RS | RT, 0),
    op("tltu", tltu, Alu, ALU, RS | RT, 0),
    op("teq", teq, Alu, ALU, RS | RT, 0),
    op("tne", tne, Alu, ALU, RS | RT, 0),
    op("tgei", tgei, Alu, ALU, RS, 0),
    op("tgeiu", tgeiu, Alu, ALU, RS, 0),
    op("tlti", tlti, Alu, ALU, RS, 0),
    op("tltiu", tltiu, Alu, ALU, RS, 0),
    op("teqi", teqi, Alu, ALU, RS, 0),
    op("tnei", tnei, Alu, ALU, RS, 0),
    op("sync", nothing, Sync, ALU, 0, 0),
    op("sync.p", nothing, Sync, ALU, 0, 0),
    op("pref", nothing, LoadStore, LOAD, RS, 0),
    op("lb", lb, LoadStore, LOAD, RS, RT),
    op("lbu", lbu, LoadStore, LOAD, RS, RT),
    op("lh", lh, LoadStore, LOAD, RS, RT),
    op("lhu", lhu, LoadStore, LOAD, RS, RT),
    op("lw", lw, LoadStore, LOAD, RS, RT),
    op("lwu", lwu, LoadStore, LOAD, RS, RT),
    op("ld", ld, LoadStore, LOAD, RS, RT),
    op("lq", lq, LoadStore, LOAD, RS, RT),
    op("lwl", lwl, LoadStore, LOAD, RS | RT, RT),
    op("lwr", lwr, LoadStore, LOAD, RS | RT, RT),
    op("ldl", ldl, LoadStore, LOAD, RS | RT, RT),
    op("ldr", ldr, LoadStore, LOAD, RS | RT, RT),
    op("sb", sb, LoadStore, STORE, RS | RT, 0),
    op("sh", sh, LoadStore, STORE, RS | RT, 0),
    op("sw", sw, LoadStore, STORE, RS | RT, 0),
    op("sd", sd, LoadStore, STORE, RS | RT, 0),
    op("sq", sq, LoadStore, STORE, RS | RT, 0),
    op("swl", swl, LoadStore, STORE, RS | RT, 0),
    op("swr", swr, LoadStore, STORE, RS | RT, 0),
    op("sdl", sdl, LoadStore, STORE, RS | RT, 0),
    op("sdr", sdr, LoadStore, STORE, RS | RT, 0),
    op("mfc0", mfc0, Cop0, ALU, 0, RT),
    op("mtc0", mtc0, Cop0, ALU, RT, 0),
    op("eret", eret, EretC, ALU, 0, 0),
    op("ei", ei, Cop0, ALU, 0, 0),
    op("di", di, Cop0, ALU, 0, 0),
];

pub(super) const RESERVED: u16 = 0;
pub(super) const LATER: u16 = 1;

/// The names this stage executes, the decoder's spelling.
pub fn names() -> impl Iterator<Item = &'static str> {
    TABLE[2..].iter().map(|o| o.name)
}

fn by_name() -> &'static HashMap<&'static str, u16> {
    static MAP: OnceLock<HashMap<&'static str, u16>> = OnceLock::new();
    MAP.get_or_init(|| TABLE.iter().enumerate().skip(2).map(|(i, o)| (o.name, i as u16)).collect())
}

/// The table's index for a word, through the decoder alone.
pub(super) fn index(w: u32) -> u16 {
    match r5900::operation(w) {
        None => RESERVED,
        Some(Decoded::Cpu(name, _)) => by_name().get(name).copied().unwrap_or(LATER),
        Some(Decoded::Vu0(_)) => LATER,
    }
}

pub(super) fn get(i: u16) -> &'static Op {
    &TABLE[i as usize]
}

/// What the timing model is told of `op` at `w`: the resources by register number.
pub(super) fn issue(op: &Op, w: u32) -> Issue {
    let bits = |u: u8| -> u64 {
        let mut m = 0u64;
        if u & RS != 0 {
            m |= 1 << rs(w);
        }
        if u & RT != 0 {
            m |= 1 << rt(w);
        }
        if u & RD != 0 {
            m |= 1 << rd(w);
        }
        if u & RA != 0 {
            m |= 1 << 31;
        }
        if u & H0 != 0 {
            m |= 1 << HILO0;
        }
        if u & H1 != 0 {
            m |= 1 << HILO1;
        }
        if u & S != 0 {
            m |= 1 << SA;
        }
        m & !1
    };
    Issue { category: op.category, row: op.row, reads: bits(op.reads), writes: bits(op.writes), taken: false }
}
