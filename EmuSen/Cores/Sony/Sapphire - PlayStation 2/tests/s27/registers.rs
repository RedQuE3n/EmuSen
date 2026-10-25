//! The S27 register program (Sapphire_Native.md §34.10): a bare EE program on what no stream shows, the IPU's
//! registers and FIFOs reached by the EE's own loads and stores: IPU_CTRL's fields and RST, IPU_BP's IFC and FP as
//! qwords enter, IPU_TOP's BUSY below 32 bits, IPU_CMD before any command, BCLR's pointer, FDEC's FB up to and past 32,
//! VDEC's tables and its error and start codes, a command written while one waits, RST while one waits, INT_IPU at each
//! command's end, and CSC fed and drained through the FIFOs by the EE. The expected words come from the EE User's
//! Manual's chapter 8 and ps2tek's IPU chapter with the readings of Sapphire_Disputes.md D-300 onwards, apart from
//! `src/ipu/`.

#![allow(dead_code)]

use crate::s6::Program;
use crate::s11::{K1, li, peek, poke, rec, sx};

use super::manual;

pub const CMD: u32 = 0x1000_2000;
pub const CTRL: u32 = 0x1000_2010;
pub const BP: u32 = 0x1000_2020;
pub const TOP: u32 = 0x1000_2030;
pub const OUT: u32 = 0x1000_7000;
pub const IN: u32 = 0x1000_7010;
pub const I_STAT: u32 = 0x1000_F000;
const RST: u32 = 1 << 30;
const BUSY: u64 = 1 << 63;
/// Where the EE keeps what it drains from IPU_out_FIFO.
const BUF: u32 = 0x0034_0000;

/// The stream's 32 bits from bit `at`, the bytes' highest bits first (H.262 §6.2's order).
pub fn bits32(bytes: &[u8], at: usize) -> u32 {
    (0..32).fold(0, |a, k| {
        let i = at + k;
        a << 1 | bytes.get(i / 8).map_or(0, |b| u32::from(b >> (7 - i % 8) & 1))
    })
}

fn bytes_of(qs: &[u128]) -> Vec<u8> {
    qs.iter().flat_map(|q| q.to_le_bytes()).collect()
}

/// The 64-bit register at `pa`, recorded whole.
fn rec64(p: &mut Program, pa: u32, want: Option<u64>) {
    li(p, "t6", K1 | pa);
    p.emit("ld t2, 0(t6)");
    p.record("t2", want);
}

/// A qword into IPU_in_FIFO by the EE.
fn push(p: &mut Program, q: u128) {
    p.quad("t7", q);
    li(p, "t6", K1 | IN);
    p.emit("sq t7, 0(t6)");
}

fn command(p: &mut Program, v: u32) {
    poke(p, CMD, v);
}

/// I_STAT's IPU bit (8), recorded, then I_STAT cleared.
fn rec_irq(p: &mut Program, want: bool) {
    peek(p, I_STAT);
    p.emit("andi t2, t2, 0x100");
    p.record("t2", Some(if want { 0x100 } else { 0 }));
    poke(p, I_STAT, 0x7FFF);
}

/// Qwords of the given bytes, padded with zeros.
pub fn qwords(b: &[u8]) -> Vec<u128> {
    b.chunks(16).map(|c| {
        let mut a = [0u8; 16];
        a[..c.len()].copy_from_slice(c);
        u128::from_le_bytes(a)
    })
    .collect()
}

/// The pattern qwords the FIFO tests push: bytes 11h × (i + 1) + j.
pub fn pattern(i: usize) -> u128 {
    u128::from_le_bytes(core::array::from_fn(|j| (0x11 * (i as u32 + 1) + j as u32) as u8))
}

pub fn registers() -> Program {
    let mut p = Program::kernel("s27-registers");
    poke(&mut p, CTRL, RST);
    poke(&mut p, I_STAT, 0x7FFF);
    rec(&mut p, CTRL, Some(0));
    rec(&mut p, BP, Some(0));
    rec64(&mut p, CMD, Some(0));
    rec64(&mut p, TOP, Some(BUSY));

    // IPU_CTRL keeps IDP, AS, IVF, QST, MP1 and PCT (§8.3); the rest reads as the IPU's state.
    poke(&mut p, CTRL, 0xBFFF_FFFF);
    rec(&mut p, CTRL, Some(0x07F3_0000));
    poke(&mut p, CTRL, 0);
    rec(&mut p, CTRL, Some(0));

    // Qwords pushed by the EE stay in the FIFO, eight of them, a ninth refused, until a read takes them in (D-302).
    for n in 1..=10usize {
        push(&mut p, pattern(n - 1));
        let ifc = n.min(8) as u32;
        rec(&mut p, BP, Some(ifc << 8));
        rec(&mut p, CTRL, Some(ifc));
    }
    let pushed = bytes_of(&(0..8).map(pattern).collect::<Vec<_>>());
    // Before any command, IPU_CMD reads the stream's first 32 bits (ps2tek, D-311), taking the first qword in;
    // IPU_TOP the same, not busy.
    rec64(&mut p, CMD, Some(u64::from(bits32(&pushed, 0))));
    rec(&mut p, BP, Some(1 << 16 | 7 << 8));
    rec64(&mut p, TOP, Some(u64::from(bits32(&pushed, 0))));

    // BCLR: the FIFO emptied, the next qword read from BP; INT_IPU at its end (D-310).
    command(&mut p, 0x45);
    rec_irq(&mut p, true);
    rec(&mut p, BP, Some(0x45));
    rec64(&mut p, TOP, Some(BUSY));
    push(&mut p, pattern(0));
    push(&mut p, pattern(1));
    let two = bytes_of(&[pattern(0), pattern(1)]);
    rec(&mut p, BP, Some(0x45 | 2 << 8));
    rec64(&mut p, TOP, Some(u64::from(bits32(&two, 0x45))));
    rec(&mut p, BP, Some(0x45 | 2 << 8));
    // BCLR's BP is 7 bits (§8.4), ps2tek's 8 not taken (D-312).
    command(&mut p, 0x85);
    rec_irq(&mut p, true);
    rec(&mut p, BP, Some(0x05));

    // FDEC: FB bits taken first, then the 32 that follow returned and not taken; FB past 32 taken whole (D-313).
    command(&mut p, 0);
    rec_irq(&mut p, true);
    let data: Vec<u128> = (0..6).map(|i| pattern(i + 3)).collect();
    for q in &data {
        push(&mut p, *q);
    }
    let db = bytes_of(&data);
    let (mut at, mut fp) = (0usize, 0usize);
    for fb in [0u32, 12, 32, 40, 63] {
        command(&mut p, 4 << 28 | fb);
        let before = at / 128;
        at += fb as usize;
        fp = if at / 128 > before { 0 } else { fp };
        fp = fp.max((at % 128 + 32).div_ceil(128).min(2));
        rec_irq(&mut p, true);
        rec64(&mut p, CMD, Some(u64::from(bits32(&db, at))));
        rec(&mut p, BP, Some((at % 128) as u32 | (fp as u32) << 16 | ((data.len() - at / 128 - fp) as u32) << 8));
        rec(&mut p, CTRL, Some((data.len() - at / 128 - fp) as u32));
    }

    // VDEC (§8.5.3): a code of each table, an error code, and a start code.
    command(&mut p, 0);
    rec_irq(&mut p, true);
    let mut v = Vec::new();
    v.extend([0b0000_0011u8, 0b0011_0000]);
    v.extend([0xC0, 0x00]);
    v.extend([0x00, 0x00, 0x01, 0xB3]);
    v.extend([0b0000_0001, 0b0010_0000]);
    v.resize(64, 0xAA);
    for q in qwords(&v) {
        push(&mut p, q);
    }
    // motion_code '0000 0011 001' = −16 (Table B.10), 11 bits; the stream then at bit 11.
    command(&mut p, 3 << 28 | 2 << 26);
    rec_irq(&mut p, true);
    rec64(&mut p, CMD, Some(0x000B_FFF0));
    rec(&mut p, BP, Some(11 | 1 << 16 | 3 << 8));
    // dmvector '11' = −1 at bit 16 (FB 5).
    command(&mut p, 3 << 28 | 3 << 26 | 5);
    rec_irq(&mut p, true);
    rec64(&mut p, CMD, Some(0x0002_FFFF));
    // At bit 32, a start code: macroblock_address_increment finds no code, data 0, ECD set, the pointer unmoved (D-306).
    command(&mut p, 3 << 28 | 14);
    rec_irq(&mut p, true);
    rec64(&mut p, CMD, Some(0));
    rec(&mut p, CTRL, Some(0x4000 | 3));
    rec(&mut p, BP, Some(32 | 1 << 16 | 3 << 8));
    // At bit 64, '0000 0001 001', no code of Table B.1: data 0, ECD set (D-306).
    command(&mut p, 3 << 28 | 32);
    rec_irq(&mut p, true);
    rec64(&mut p, CMD, Some(0));
    rec(&mut p, CTRL, Some(0x4000 | 3));
    // macroblock_type with PCT 0, a reserved picture type: no table, an error code.
    command(&mut p, 3 << 28 | 1 << 26);
    rec_irq(&mut p, true);
    rec(&mut p, CTRL, Some(0x4000 | 3));

    // A command waiting for bits is busy, IPU_CMD's DATA meaningless; a command written meanwhile is ignored, but BCLR
    // ends the waiting one and acts (D-314); IPU_CMD then shows the stream again.
    command(&mut p, 0);
    rec_irq(&mut p, true);
    command(&mut p, 4 << 28 | 8);
    rec(&mut p, CTRL, Some(0x8000_0000));
    li(&mut p, "t6", K1 | CMD);
    p.emit("ld t2, 0(t6)");
    p.emit("dsrl32 t2, t2, 31");
    p.record("t2", Some(1));
    rec64(&mut p, TOP, Some(BUSY));
    command(&mut p, 5 << 28);
    rec_irq(&mut p, false);
    command(&mut p, 0x10);
    rec_irq(&mut p, true);
    rec(&mut p, CTRL, Some(0));
    push(&mut p, pattern(7));
    rec_irq(&mut p, false);
    rec64(&mut p, CMD, Some(u64::from(bits32(&bytes_of(&[pattern(7)]), 16))));
    rec(&mut p, BP, Some(16 | 1 << 16));

    // RST while a command waits: the command dropped, INT_IPU raised (ps2tek), every count and the pointer cleared.
    command(&mut p, 0);
    rec_irq(&mut p, true);
    command(&mut p, 4 << 28);
    poke(&mut p, CTRL, RST | 2 << 24);
    rec_irq(&mut p, true);
    rec(&mut p, CTRL, Some(2 << 24));
    rec(&mut p, BP, Some(0));

    // CSC through the FIFOs by the EE: SETTH, then one RAW8 macroblock pushed while IFC allows, the 64 RGB32 qwords
    // drained while OFC shows them.
    command(&mut p, 9 << 28 | 140 << 16 | 60);
    rec_irq(&mut p, true);
    let raw = super::raw8_inputs()[3];
    command(&mut p, 7 << 28 | 1);
    let input = qwords(&raw);
    li(&mut p, "s5", K1 | BUF);
    li(&mut p, "s6", 64);
    for q in &input {
        let l = p.label();
        p.place(&l);
        peek(&mut p, CTRL);
        p.emit("andi t2, t2, 0xf");
        p.emit("addiu t3, zero, 8");
        p.emit(format!("beq t2, t3, {l}"));
        p.emit("nop");
        push(&mut p, *q);
    }
    let (top, out) = (p.label(), p.label());
    li(&mut p, "t5", 100_000);
    p.place(&top);
    p.emit(format!("beq s6, zero, {out}"));
    p.emit("nop");
    p.emit("addiu t5, t5, -1");
    p.emit(format!("beq t5, zero, {out}"));
    p.emit("nop");
    peek(&mut p, CTRL);
    p.emit("srl t2, t2, 4");
    p.emit("andi t2, t2, 0xf");
    p.emit(format!("beq t2, zero, {top}"));
    p.emit("nop");
    li(&mut p, "t6", K1 | OUT);
    p.emit("lq t3, 0(t6)");
    p.emit("sq t3, 0(s5)");
    p.emit("daddiu s5, s5, 16");
    p.emit("addiu s6, s6, -1");
    p.emit(format!("beq zero, zero, {top}"));
    p.emit("nop");
    p.place(&out);
    rec_irq(&mut p, true);
    let px = manual::from_raw8(&raw, 60, 140);
    for (i, q) in manual::rgb32_qwords(&px).iter().enumerate() {
        li(&mut p, "t6", K1 | (BUF + 16 * i as u32));
        for w in 0..4 {
            p.emit(format!("lw t2, {}(t6)", 4 * w));
            p.record("t2", sx((q >> (32 * w)) as u32));
        }
    }
    rec(&mut p, CTRL, Some(2 << 24));
    p
}
