//! The project's program for dispute D-780 (Sapphire_Native.md §39.15): `d780-width` stores bytes and halfwords into
//! registers of five units, each read back by LW and some by LBU, LHU and LB: DMA channel 1's CHCR, MADR, QWC and TADR
//! (the channel never started), VIF1's MARK and ERR, GIF_MODE (seen through GIF_STAT), timer 1's COMP and MODE, and
//! the INTC's I_MASK, whose writes reverse bits (its change is recorded, not its value, which the kernel sets). The EE
//! User's Manual §2.2.1 calls these registers "only word-accessible" and says nothing of a narrower access, so every
//! expected word is the referee's measurement. Run bare (`width_bare`, tests/dmac.rs) the program stores directly
//! through kseg1; run by the fast boot on the player's BIOS and on PCSX2 (`width`, written as an ELF by
//! examples/d780.rs) it stores through a kernel-mode routine of its own installed with SetSyscall (74h), as D-700's
//! programs do, since the kernel maps some register pages read-only to its threads, and reads at the kuseg addresses.

#![allow(dead_code)]

use crate::s6::Program;

pub const D1_CHCR: u32 = 0x1000_9000;
pub const D1_MADR: u32 = 0x1000_9010;
pub const D1_QWC: u32 = 0x1000_9020;
pub const D1_TADR: u32 = 0x1000_9030;
pub const VIF1_ERR: u32 = 0x1000_3C20;
pub const VIF1_MARK: u32 = 0x1000_3C30;
pub const GIF_MODE: u32 = 0x1000_3010;
pub const GIF_STAT: u32 = 0x1000_3020;
pub const T1_MODE: u32 = 0x1000_0810;
pub const T1_COMP: u32 = 0x1000_0820;
pub const I_MASK: u32 = 0x1000_F010;
/// The syscall the BIOS form installs for its stores: a0 the kseg1 address, a1 the value, a2 0 SB, 1 SH, 2 SW.
pub const KSTORE: u32 = 0x5A;

fn li(p: &mut Program, reg: &str, v: u32) {
    p.emit(format!("lui {reg}, 0x{:x}", v >> 16));
    p.emit(format!("ori {reg}, {reg}, 0x{:x}", v & 0xFFFF));
}

/// How the program reaches the registers: bare, in kernel mode, or as a thread of the player's BIOS.
#[derive(Clone, Copy, PartialEq)]
enum Form {
    Bare,
    Thread,
}

fn store(p: &mut Program, f: Form, width: &str, addr: u32, v: u32) {
    match f {
        Form::Bare => {
            li(p, "t6", addr | 0xA000_0000);
            li(p, "t7", v);
            p.emit(format!("{width} t7, 0(t6)"));
        }
        Form::Thread => {
            li(p, "a0", addr | 0xA000_0000);
            li(p, "a1", v);
            let code = match width {
                "sb" => 0,
                "sh" => 1,
                _ => 2,
            };
            p.emit(format!("addiu a2, zero, {code}"));
            p.emit(format!("addiu v1, zero, 0x{KSTORE:x}"));
            p.emit("syscall 0");
            p.emit("nop");
        }
    }
    p.emit("sync");
}

fn base(f: Form, addr: u32) -> u32 {
    if f == Form::Bare { addr | 0xA000_0000 } else { addr }
}

/// The word at `addr` by LW into t2, masked by `mask`.
fn read(p: &mut Program, f: Form, addr: u32, mask: u32) {
    li(p, "t6", base(f, addr & !3));
    p.emit("lw t2, 0(t6)");
    if mask != u32::MAX {
        li(p, "t7", mask);
        p.emit("and t2, t2, t7");
    }
}

/// A store of `width` (sb, sh, sw) of `v` at `addr`, then the word read back by LW (masked) and recorded.
fn store_then_read(p: &mut Program, f: Form, width: &str, addr: u32, v: u32, mask: u32, expected: u32) {
    store(p, f, width, addr, v);
    read(p, f, addr, mask);
    p.record("t2", Some(expected as i32 as i64 as u64));
}

/// A narrow load (lbu, lhu, lb) at `addr`, recorded.
fn narrow_read(p: &mut Program, f: Form, width: &str, addr: u32, expected: u64) {
    li(p, "t6", base(f, addr));
    p.emit(format!("{width} t2, 0(t6)"));
    p.record("t2", Some(expected));
}

/// I_MASK: a narrow store's change to the register (after XOR before), then the register put back by a word store of
/// the change, which reverses the same bits.
fn mask_change(p: &mut Program, f: Form, width: &str, addr: u32, v: u32, expected: u32) {
    read(p, f, I_MASK, u32::MAX);
    p.emit("or s2, t2, zero");
    store(p, f, width, addr, v);
    read(p, f, I_MASK, u32::MAX);
    p.emit("xor t2, t2, s2");
    p.emit("or s3, t2, zero");
    p.record("t2", Some(expected as u64));
    match f {
        Form::Bare => {
            li(p, "t6", I_MASK | 0xA000_0000);
            p.emit("sw s3, 0(t6)");
        }
        Form::Thread => {
            li(p, "a0", I_MASK | 0xA000_0000);
            p.emit("or a1, s3, zero");
            p.emit("addiu a2, zero, 2");
            p.emit(format!("addiu v1, zero, 0x{KSTORE:x}"));
            p.emit("syscall 0");
            p.emit("nop");
        }
    }
    p.emit("sync");
    read(p, f, I_MASK, u32::MAX);
    p.emit("xor t2, t2, s2");
    p.record("t2", Some(0));
}

pub fn width() -> Program {
    let mut p = Program::new("d780-width");
    let (main, routine) = (p.label(), p.label());
    p.emit(format!("beq zero, zero, {main}"));
    p.emit("nop");
    let at = p.here();
    p.place(&routine);
    let (half, word) = (p.label(), p.label());
    p.emit(format!("bne a2, zero, {half}"));
    p.emit("nop");
    p.emit("sb a1, 0(a0)");
    p.emit("jr ra");
    p.emit("nop");
    p.place(&half);
    p.emit("addiu t0, zero, 1");
    p.emit(format!("bne a2, t0, {word}"));
    p.emit("nop");
    p.emit("sh a1, 0(a0)");
    p.emit("jr ra");
    p.emit("nop");
    p.place(&word);
    p.emit("sw a1, 0(a0)");
    p.emit("jr ra");
    p.emit("nop");
    p.place(&main);
    p.emit(format!("addiu a0, zero, 0x{KSTORE:x}"));
    li(&mut p, "a1", at);
    p.emit("addiu v1, zero, 0x74");
    p.emit("syscall 0");
    p.emit("nop");
    body(p, Form::Thread)
}

/// The same program run bare from kseg0, the registers reached through kseg1 as S11's programs reach them.
pub fn width_bare() -> Program {
    body(Program::kernel("d780-width-bare"), Form::Bare)
}

/// The expected words are PCSX2 2.8.2's, measured 2026-10-26 (D-780).
fn body(mut p: Program, f: Form) -> Program {
    let all = u32::MAX;
    // DMA channel 1.
    store_then_read(&mut p, f, "sw", D1_CHCR, 0, all, 0);
    store_then_read(&mut p, f, "sw", D1_CHCR, 0xABCD_0000, all, 0xABCD_0000);
    store_then_read(&mut p, f, "sb", D1_CHCR, 0x40, all, 0xABCD_0040);
    store_then_read(&mut p, f, "sb", D1_CHCR + 2, 0x12, all, 0xAB12_0040);
    store_then_read(&mut p, f, "sb", D1_CHCR + 1, 0, all, 0xAB12_0040);
    store_then_read(&mut p, f, "sb", D1_CHCR + 3, 0x34, all, 0x3412_0040);
    store_then_read(&mut p, f, "sh", D1_CHCR, 0x0045, all, 0x3412_0045);
    store_then_read(&mut p, f, "sh", D1_CHCR + 2, 0x5678, all, 0x5678_0045);
    narrow_read(&mut p, f, "lbu", D1_CHCR, 0x45);
    narrow_read(&mut p, f, "lbu", D1_CHCR + 2, 0x78);
    narrow_read(&mut p, f, "lhu", D1_CHCR + 2, 0x5678);
    narrow_read(&mut p, f, "lb", D1_CHCR + 3, 0x56);
    store_then_read(&mut p, f, "sw", D1_CHCR, 0, all, 0);
    store_then_read(&mut p, f, "sw", D1_MADR, 0x0123_4560, all, 0x0123_4560);
    store_then_read(&mut p, f, "sb", D1_MADR + 1, 0x89, all, 0x0123_8960);
    store_then_read(&mut p, f, "sb", D1_MADR + 3, 0x0B, all, 0x0B23_8960);
    store_then_read(&mut p, f, "sh", D1_MADR, 0xCDE0, all, 0x0B23_CDE0);
    store_then_read(&mut p, f, "sh", D1_MADR + 2, 0x0102, all, 0x0102_CDE0);
    store_then_read(&mut p, f, "sw", D1_QWC, 0x1234, all, 0x1234);
    store_then_read(&mut p, f, "sb", D1_QWC + 1, 0x56, all, 0x5634);
    store_then_read(&mut p, f, "sw", D1_TADR, 0x0030_0000, all, 0x0030_0000);
    store_then_read(&mut p, f, "sb", D1_TADR, 0x40, all, 0x0030_0040);
    // Dark Cloud 2's sequence: TTE set by a byte store, then CHCR read back.
    store_then_read(&mut p, f, "sw", D1_CHCR, 0, all, 0);
    store_then_read(&mut p, f, "sb", D1_CHCR, 0x40, all, 0x40);
    for r in [D1_CHCR, D1_MADR, D1_QWC, D1_TADR] {
        store(&mut p, f, "sw", r, 0);
    }
    // VIF1: MARK (16 bits) and ERR (3 bits).
    store_then_read(&mut p, f, "sw", VIF1_MARK, 0x1234, 0xFFFF, 0x1234);
    store_then_read(&mut p, f, "sb", VIF1_MARK + 1, 0x56, 0xFFFF, 0x5634);
    store_then_read(&mut p, f, "sh", VIF1_MARK, 0x789A, 0xFFFF, 0x789A);
    narrow_read(&mut p, f, "lbu", VIF1_MARK + 1, 0x78);
    store_then_read(&mut p, f, "sw", VIF1_MARK, 0, 0xFFFF, 0);
    store_then_read(&mut p, f, "sw", VIF1_ERR, 0, 7, 0);
    store_then_read(&mut p, f, "sb", VIF1_ERR, 6, 7, 6);
    store_then_read(&mut p, f, "sw", VIF1_ERR, 0, 7, 0);
    // The GIF: GIF_MODE's M3R and IMT, which GIF_STAT shows.
    store_then_read(&mut p, f, "sw", GIF_MODE, 0, all, 0);
    store(&mut p, f, "sb", GIF_MODE, 5);
    read(&mut p, f, GIF_STAT, 5);
    p.record("t2", Some(5));
    store(&mut p, f, "sw", GIF_MODE, 0);
    read(&mut p, f, GIF_STAT, 5);
    p.record("t2", Some(0));
    // Timer 1: COMP (16 bits) and MODE's low byte (the clock BUSCLK/256, not counting).
    store_then_read(&mut p, f, "sw", T1_MODE, 0, 0x3FF, 0);
    store_then_read(&mut p, f, "sw", T1_COMP, 0x1234, 0xFFFF, 0x1234);
    store_then_read(&mut p, f, "sb", T1_COMP + 1, 0x56, 0xFFFF, 0x5634);
    store_then_read(&mut p, f, "sh", T1_COMP, 0x9ABC, 0xFFFF, 0x9ABC);
    narrow_read(&mut p, f, "lhu", T1_COMP, 0x9ABC);
    store_then_read(&mut p, f, "sb", T1_MODE, 0x02, 0x3FF, 0x002);
    store_then_read(&mut p, f, "sw", T1_MODE, 0, 0x3FF, 0);
    store_then_read(&mut p, f, "sw", T1_COMP, 0, 0xFFFF, 0);
    // The INTC: I_MASK's writes reverse the bits written as 1; bare, mask bits are set in both lanes first, as the
    // kernel leaves them, so that a store repeating the other lanes would show.
    if f == Form::Bare {
        store(&mut p, f, "sw", I_MASK, 0x0A0A);
    }
    mask_change(&mut p, f, "sb", I_MASK, 0x10, 0x10);
    mask_change(&mut p, f, "sb", I_MASK + 1, 0x40, 0x4000);
    mask_change(&mut p, f, "sh", I_MASK, 0x0100, 0x0100);
    if f == Form::Bare {
        store(&mut p, f, "sw", I_MASK, 0x0A0A);
    }
    p
}

pub fn all() -> Vec<Program> {
    vec![width()]
}
