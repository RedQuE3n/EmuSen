//! The project's S20 programs (Sapphire_Native.md §29): `s20-timing` measures the video timing a program sees (the
//! fields' lengths in EE cycles, V-blank's length, H-blank's ticks and the GS's H-syncs a field, CSR's FIELD and NFIELD
//! at each edge) in four settings of SMODE2; the display programs are scripts of S16's form that fill the GS's memory,
//! set the PCRTC's registers and read the memory back, so the referees' pictures can be taken as S3 decided.

#![allow(dead_code)]

pub mod manual;

use crate::s6::Program;
use crate::s11::{K1, li, peek, poke, wait};
use crate::s16::{self, Op, Script, ga, upload, download};

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

/// PMODE (§7.2): CRTMD is always 001.
pub fn pmode(en1: bool, en2: bool, mmod: bool, amod: bool, slbg: bool, alp: u8) -> u64 {
    u64::from(en1) | u64::from(en2) << 1 | 1 << 2 | u64::from(mmod) << 5 | u64::from(amod) << 6 | u64::from(slbg) << 7 | u64::from(alp) << 8
}

pub fn dispfb(fbp: u32, fbw: u32, psm: u32, dbx: u32, dby: u32) -> u64 {
    u64::from(fbp) | u64::from(fbw) << 9 | u64::from(psm) << 15 | u64::from(dbx) << 32 | u64::from(dby) << 43
}

/// DISPLAY with MAGH and MAGV as magnifications (1 to 16, 1 to 4) and DW, DH as sizes, not sizes less one.
pub fn display(dx: u32, dy: u32, magh: u32, magv: u32, w: u32, h: u32) -> u64 {
    u64::from(dx) | u64::from(dy) << 12 | u64::from(magh - 1) << 23 | u64::from(magv - 1) << 27 | u64::from(w - 1) << 32 | u64::from(h - 1) << 44
}

/// Waits for `n` V-syncs (CSR's VSINT cleared, then waited for).
pub fn vsyncs(ops: &mut Vec<Op>, n: usize) {
    for _ in 0..n {
        ops.push(Op::Priv(ga::CSR, 8));
        ops.push(Op::Wait(ga::CSR, 8));
    }
}

/// Waits for `n` H-syncs likewise.
pub fn hsyncs(ops: &mut Vec<Op>, n: usize) {
    for _ in 0..n {
        ops.push(Op::Priv(ga::CSR, 4));
        ops.push(Op::Wait(ga::CSR, 4));
    }
}

/// A display program: SMODE2 and PMODE off first, the images uploaded, the circuits set, three V-syncs waited for,
/// and the images read back so that a referee's memory can be compared (S3's form).
fn show(name: &'static str, smode2: u64, images: &[(u32, (u32, u32), (u32, u32), u32)], regs: [(u32, u64); 6]) -> Script {
    let mut ops = vec![Op::Priv(ga::PMODE, 0), Op::Priv(ga::SMODE2, smode2), Op::Priv(ga::CSR, 0x1F)];
    for &(psm, place, (w, h), seed) in images {
        upload(&mut ops, psm, place, (0, 0, w, h), seed);
    }
    for (g, v) in regs {
        ops.push(Op::Priv(g, v));
    }
    vsyncs(&mut ops, 3);
    for &(psm, place, (w, h), _) in images {
        download(&mut ops, psm, place, (0, 0, w, h));
    }
    Script { name, ops }
}

use s16::manual::{CT16, CT16S, CT24, CT32};

/// The rectangle's place on the screen the programs share: DX 640 VCK, DY 50 rasters.
pub const DX: u32 = 640;
pub const DY: u32 = 50;

pub fn displays() -> Vec<Script> {
    use ga::*;
    vec![
        // Circuit 1 alone, PSMCT32, magnified four times across and twice down, read from (8, 4) in the buffer.
        show("s20-ct32", 0, &[(CT32, (0, 1), (64, 32), 0x2001)], [
            (DISPFB1, dispfb(0, 1, CT32, 8, 4)),
            (DISPLAY1, display(DX, DY, 4, 2, 4 * 48, 2 * 24)),
            (BGCOLOR, 0x80_4020),
            (DISPFB2, 0),
            (DISPLAY2, 0),
            (PMODE, pmode(true, false, true, false, true, 0xFF)),
        ]),
        // Circuit 1 in PSMCT24 and circuit 2 in PSMCT16, side by side; OUT2 shows where circuit 1 does not.
        show("s20-pair", 0, &[(CT24, (0, 1), (64, 32), 0x2002), (CT16, (64, 1), (64, 64), 0x2003)], [
            (DISPFB1, dispfb(0, 1, CT24, 0, 0)),
            (DISPLAY1, display(DX, DY, 4, 1, 4 * 64, 32)),
            (DISPFB2, dispfb(2, 1, CT16, 0, 0)),
            (DISPLAY2, display(DX + 4 * 64 + 40, DY + 8, 5, 1, 5 * 64, 48)),
            (BGCOLOR, 0x10_2030),
            (PMODE, pmode(true, true, true, false, false, 0xFF)),
        ]),
        // Circuit 1's own alpha (doubled, clamped) blends it over circuit 2 in PSMCT16S where they overlap.
        show("s20-alpha", 0, &[(CT32, (0, 1), (64, 32), 0x2004), (CT16S, (64, 1), (64, 64), 0x2005)], [
            (DISPFB1, dispfb(0, 1, CT32, 0, 0)),
            (DISPLAY1, display(DX, DY, 4, 1, 4 * 64, 32)),
            (DISPFB2, dispfb(2, 1, CT16S, 0, 0)),
            (DISPLAY2, display(DX + 4 * 32, DY + 16, 4, 1, 4 * 64, 48)),
            (BGCOLOR, 0x00_0000),
            (PMODE, pmode(true, true, false, false, false, 0)),
        ]),
        // ALP's fixed alpha over the background colour, circuit 2 on but not blended (SLBG).
        show("s20-alp", 0, &[(CT32, (0, 1), (64, 32), 0x2006), (CT16, (64, 1), (64, 64), 0x2007)], [
            (DISPFB1, dispfb(0, 1, CT32, 0, 0)),
            (DISPLAY1, display(DX, DY, 3, 1, 3 * 64, 32)),
            (DISPFB2, dispfb(2, 1, CT16, 0, 0)),
            (DISPLAY2, display(DX, DY, 3, 1, 3 * 64, 32)),
            (BGCOLOR, 0x30_C0F0),
            (PMODE, pmode(true, true, true, false, true, 0x60)),
        ]),
        // Interlaced in FIELD mode: each field reads every other line of a 64-line buffer.
        show("s20-field", 1, &[(CT32, (0, 1), (64, 64), 0x2008)], [
            (DISPFB1, dispfb(0, 1, CT32, 0, 0)),
            (DISPLAY1, display(DX, DY, 4, 1, 4 * 64, 64)),
            (BGCOLOR, 0x00_00FF),
            (DISPFB2, 0),
            (DISPLAY2, 0),
            (PMODE, pmode(true, false, true, false, true, 0xFF)),
        ]),
        // Interlaced in FRAME mode: each field reads every line of a 32-line buffer over 64 rasters.
        show("s20-frame", 3, &[(CT32, (0, 1), (64, 32), 0x2009)], [
            (DISPFB1, dispfb(0, 1, CT32, 0, 0)),
            (DISPLAY1, display(DX, DY, 4, 1, 4 * 64, 64)),
            (BGCOLOR, 0x00_FF00),
            (DISPFB2, 0),
            (DISPLAY2, 0),
            (PMODE, pmode(true, false, true, false, true, 0xFF)),
        ]),
    ]
}

/// H-syncs after the V-sync before `s20-midframe` switches circuit 1 to the second buffer.
pub const SWITCH_AFTER: usize = 40;
/// Its rectangle's first raster, so that the switch falls inside it.
pub const MIDFRAME_DY: u32 = 30;

/// `s20-midframe`'s setup: two buffers, circuit 1 on the first; each field then waits for its V-sync, sets the first,
/// waits `SWITCH_AFTER` H-syncs and sets the second (`midframe_field`).
pub fn midframe() -> Script {
    use ga::*;
    let mut s = show("s20-midframe", 0, &[(CT32, (0, 1), (64, 32), 0x200A), (CT32, (32, 1), (64, 32), 0x200B)], [
        (DISPFB1, dispfb(0, 1, CT32, 0, 0)),
        (DISPLAY1, display(DX, MIDFRAME_DY, 4, 1, 4 * 64, 32)),
        (BGCOLOR, 0x40_4040),
        (DISPFB2, 0),
        (DISPLAY2, 0),
        (PMODE, pmode(true, false, true, false, true, 0xFF)),
    ]);
    s.ops.extend(midframe_field());
    s
}

/// One field of `s20-midframe`'s switching.
pub fn midframe_field() -> Vec<Op> {
    let mut ops = Vec::new();
    vsyncs(&mut ops, 1);
    ops.push(Op::Priv(ga::DISPFB1, dispfb(0, 1, CT32, 0, 0)));
    hsyncs(&mut ops, SWITCH_AFTER);
    ops.push(Op::Priv(ga::DISPFB1, dispfb(1, 1, CT32, 0, 0)));
    ops
}

/// The display scripts as EE programs; `s20-midframe` goes on switching every field after its done word, so a
/// referee's picture taken at any frame shows the switch.
pub fn display_programs() -> Vec<Program> {
    let mut out: Vec<Program> = displays().iter().map(s16::program).collect();
    let mut p = s16::program(&midframe());
    p.emit("lui t0, 0x5a99");
    p.emit("ori t0, t0, 0xd00e");
    p.emit("lui t1, 0xa01f");
    p.emit("ori t1, t1, 0xfff0");
    p.emit("sw t0, 0(t1)");
    let top = p.label();
    p.place(&top);
    let tail = Script { name: "s20-midframe-loop", ops: midframe_field() };
    for op in &tail.ops {
        match op {
            Op::Priv(g, v) => {
                li(&mut p, "t6", K1 | s16::ee_address(*g));
                p.constant("t7", *v);
                p.emit("sd t7, 0(t6)");
            }
            Op::Wait(g, mask) => wait64(&mut p, s16::ee_address(*g), *mask),
            _ => unreachable!(),
        }
    }
    p.emit(format!("beq zero, zero, {top}"));
    p.emit("nop");
    out.push(p);
    out
}

/// Waits, bounded, until the doubleword at `pa` ANDed with `mask` is non-zero.
fn wait64(p: &mut Program, pa: u32, mask: u64) {
    let l = p.label();
    let out = p.label();
    li(p, "t6", K1 | pa);
    li(p, "t5", 4_000_000);
    p.place(&l);
    p.emit("ld t2, 0(t6)");
    p.emit(format!("andi t2, t2, 0x{mask:x}"));
    p.emit(format!("bne t2, zero, {out}"));
    p.emit("nop");
    p.emit("addiu t5, t5, -1");
    p.emit(format!("bne t5, zero, {l}"));
    p.emit("nop");
    p.place(&out);
}

pub fn all() -> Vec<Program> {
    let mut v = vec![timing()];
    v.extend(display_programs());
    v
}

/// The script run on S16's model of the GS: the memory and the registers it leaves.
pub fn memory(script: &Script) -> s16::manual::Gs {
    let mut gs = s16::manual::Gs::default();
    for op in &script.ops {
        match op {
            Op::Regs(list) => list.iter().for_each(|&(reg, v)| gs.write(reg, v)),
            Op::Image { seed, qwords } => s16::units(*seed, *qwords).into_iter().for_each(|u| gs.write(s16::r::HWREG, u)),
            Op::Priv(g, v) => gs.write_privileged(*g, *v),
            Op::Finish => {
                gs.write_privileged(ga::CSR, 2);
                gs.write(s16::r::FINISH, 0);
                gs.write_privileged(ga::CSR, 2);
            }
            Op::Readback(q) => {
                gs.write_privileged(ga::BUSDIR, 1);
                (0..2 * q).for_each(|_| {
                    gs.read();
                });
                gs.write_privileged(ga::BUSDIR, 0);
            }
            Op::Wait(_, m) => gs.event(*m as u32),
            _ => panic!("{}: an op the display programs do not use", script.name),
        }
    }
    gs
}
