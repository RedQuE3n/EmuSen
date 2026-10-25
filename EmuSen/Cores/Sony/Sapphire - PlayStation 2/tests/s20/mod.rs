//! The project's S20 programs (Sapphire_Native.md §29): `s20-timing` measures the video timing a program sees (the
//! fields' lengths in EE cycles, V-blank's length, H-blank's ticks and the GS's H-syncs a field, CSR's FIELD and NFIELD
//! at each edge) in four settings of SMODE2; the display programs are scripts of S16's form that fill the GS's memory,
//! set the PCRTC's registers and read the memory back, so the referees' pictures can be taken as S3 decided.

#![allow(dead_code)]

use crate::s6::Program;
use crate::s11::{K1, li, peek, poke, wait};

pub const I_STAT: u32 = 0x1000_F000;
pub const I_MASK: u32 = 0x1000_F010;
pub const D_CTRL: u32 = 0x1000_E000;
pub const CSR: u32 = 0x1200_1000;
pub const SMODE2: u32 = 0x1200_0020;

pub const fn t_count(n: u32) -> u32 {
    0x1000_0000 + n * 0x800
}
pub const fn t_mode(n: u32) -> u32 {
    0x1000_0010 + n * 0x800
}

/// I_STAT's V-blank bits (EE User's Manual §3.1).
pub const VB_ON: u32 = 1 << 2;
pub const VB_OFF: u32 = 1 << 3;

fn sd(p: &mut Program, pa: u32, v: u64) {
    li(p, "t6", K1 | pa);
    p.constant("t7", v);
    p.emit("sd t7, 0(t6)");
}

fn prologue(p: &mut Program) {
    p.emit("di");
    p.emit("mtc0 zero, Status");
    p.emit("sync.p");
    peek(p, I_MASK);
    p.emit("sw t2, 0(t6)");
    poke(p, I_STAT, 0x7FFF);
    for n in 0..4 {
        poke(p, t_mode(n), 0xC00);
    }
}

/// Clears an I_STAT bit and waits for it to be latched again.
fn edge(p: &mut Program, bit: u32) {
    poke(p, I_STAT, bit);
    wait(p, I_STAT, bit, true, 4_000_000);
}

fn csr_bits(p: &mut Program, mask: u32) {
    li(p, "t6", K1 | CSR);
    p.emit("ld t2, 0(t6)");
    p.emit(format!("andi t2, t2, 0x{mask:x}"));
}

/// The settings the timing is measured in: SMODE2 as the firmware left it, then written non-interlaced, interlaced in
/// field mode and interlaced in frame mode.
pub const SETTINGS: [Option<u64>; 4] = [None, Some(0), Some(1), Some(3)];
/// Fields measured in each setting.
pub const FIELDS: usize = 5;
/// H-syncs and H-blank ticks timed after a V-blank's start.
pub const TICKS: usize = 6;

/// Words recorded in each setting: per field the Count, timer 0 (H-blank as its clock), timer 1 (BUSCLK/256), timer 2
/// (BUSCLK/256 gated off during H-blank) and CSR's field bits at V-blank's start, Count and CSR's field bits at its
/// end; then Count at a V-blank's start, at each of the next H-syncs and at each of the next H-blank ticks; the
/// H-syncs in one field.
pub const PER_SETTING: usize = FIELDS * 7 + 1 + 2 * TICKS + 1 + 4;

/// From V-blank's end, one polling pass watching I_STAT's VB_ON, CSR's VSINT and CSR's FIELD at once: Count at
/// V-blank's start, then VSINT's and FIELD's change after it (signed), and FIELD before.
fn phases(p: &mut Program) {
    edge(p, VB_OFF);
    csr_bits(p, 0x2000);
    p.emit("or s5, t2, zero");
    sd(p, CSR, 8);
    poke(p, I_STAT, VB_ON);
    for r in ["s2", "s3", "s4"] {
        p.emit(format!("ori {r}, zero, 0"));
    }
    li(p, "t4", K1 | CSR);
    li(p, "t3", K1 | I_STAT);
    li(p, "t5", 4_000_000);
    let top = p.label();
    let (a, b, c, d) = (p.label(), p.label(), p.label(), p.label());
    p.place(&top);
    p.emit("mfc0 t0, Count");
    p.emit(format!("bne s2, zero, {a}"));
    p.emit("nop");
    p.emit("lw t2, 0(t3)");
    p.emit("andi t2, t2, 4");
    p.emit(format!("beq t2, zero, {a}"));
    p.emit("nop");
    p.emit("or s2, t0, zero");
    p.place(&a);
    p.emit("ld t1, 0(t4)");
    p.emit(format!("bne s3, zero, {b}"));
    p.emit("nop");
    p.emit("andi t2, t1, 8");
    p.emit(format!("beq t2, zero, {b}"));
    p.emit("nop");
    p.emit("or s3, t0, zero");
    p.place(&b);
    p.emit(format!("bne s4, zero, {c}"));
    p.emit("nop");
    p.emit("andi t2, t1, 0x2000");
    p.emit(format!("beq t2, s5, {c}"));
    p.emit("nop");
    p.emit("or s4, t0, zero");
    p.place(&c);
    p.emit("addiu t5, t5, -1");
    p.emit(format!("beq t5, zero, {d}"));
    p.emit("nop");
    p.emit(format!("beq s2, zero, {top}"));
    p.emit("nop");
    p.emit(format!("beq s3, zero, {top}"));
    p.emit("nop");
    p.emit(format!("beq s4, zero, {top}"));
    p.emit("nop");
    p.place(&d);
    p.record("s2", None);
    p.emit("subu t2, s3, s2");
    p.record("t2", None);
    p.emit("subu t2, s4, s2");
    p.record("t2", None);
    p.record("s5", None);
}

/// `s20-timing`: every word ungraded; Sapphire's own reading is graded in the tests from `manual`.
pub fn timing() -> Program {
    let mut prog = Program::kernel("s20-timing");
    let p = &mut prog;
    prologue(p);
    for s in SETTINGS {
        if let Some(v) = s {
            sd(p, SMODE2, v);
        }
        poke(p, t_count(0), 0);
        poke(p, t_mode(0), 0x83);
        poke(p, t_count(1), 0);
        poke(p, t_mode(1), 0x82);
        poke(p, t_count(2), 0);
        poke(p, t_mode(2), 0x86);
        edge(p, VB_ON);
        edge(p, VB_ON);
        for _ in 0..FIELDS {
            p.emit("mfc0 a0, Count");
            li(p, "t6", K1 | t_count(0));
            p.emit("lw a1, 0(t6)");
            p.emit("lw a2, 0x800(t6)");
            p.emit("lw a3, 0x1000(t6)");
            p.record("a0", None);
            p.record("a1", None);
            p.record("a2", None);
            p.record("a3", None);
            csr_bits(p, 0x3000);
            p.record("t2", None);
            edge(p, VB_OFF);
            p.emit("mfc0 a0, Count");
            p.record("a0", None);
            csr_bits(p, 0x3000);
            p.record("t2", None);
            edge(p, VB_ON);
        }
        // The H-syncs after a V-blank's start, each timed by Count.
        edge(p, VB_ON);
        p.emit("mfc0 a0, Count");
        p.record("a0", None);
        for _ in 0..TICKS {
            sd(p, CSR, 4);
            wait(p, CSR, 4, true, 100_000);
            p.emit("mfc0 a0, Count");
            p.record("a0", None);
        }
        // The H-blank ticks of timer 0, each timed by Count.
        for _ in 0..TICKS {
            let l = p.label();
            li(p, "t6", K1 | t_count(0));
            p.emit("lw a1, 0(t6)");
            li(p, "t5", 100_000);
            p.place(&l);
            p.emit("lw a2, 0(t6)");
            p.emit("addiu t5, t5, -1");
            p.emit(format!("beq t5, zero, {l}x"));
            p.emit("nop");
            p.emit(format!("beq a1, a2, {l}"));
            p.emit("nop");
            p.place(&format!("{l}x"));
            p.emit("mfc0 a0, Count");
            p.record("a0", None);
        }
        // The H-syncs in one field: HSINT cleared and waited for until V-blank starts again.
        poke(p, I_STAT, VB_ON);
        wait(p, I_STAT, VB_ON, true, 4_000_000);
        poke(p, I_STAT, VB_ON);
        p.emit("ori a1, zero, 0");
        let top = p.label();
        let poll = p.label();
        let out = p.label();
        li(p, "t4", K1 | CSR);
        li(p, "t3", K1 | I_STAT);
        p.constant("t7", 4);
        p.place(&top);
        p.emit("sd t7, 0(t4)");
        p.place(&poll);
        p.emit("lw t2, 0(t3)");
        p.emit("andi t2, t2, 4");
        p.emit(format!("bne t2, zero, {out}"));
        p.emit("nop");
        p.emit("ld t2, 0(t4)");
        p.emit("andi t2, t2, 4");
        p.emit(format!("beq t2, zero, {poll}"));
        p.emit("nop");
        p.emit("addiu a1, a1, 1");
        p.emit(format!("beq zero, zero, {top}"));
        p.emit("nop");
        p.place(&out);
        p.record("a1", None);
        phases(p);
    }
    for n in 0..4 {
        poke(p, t_mode(n), 0xC00);
    }
    poke(p, I_STAT, 0x7FFF);
    std::mem::replace(p, Program::new("s20-timing"))
}

pub fn all() -> Vec<Program> {
    vec![timing()]
}
