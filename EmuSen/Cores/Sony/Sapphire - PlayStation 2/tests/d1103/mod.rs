//! The project's program for dispute D-1103 (Sapphire_Native.md §59): `d1103-stc` sends VIF1 packets by DMA channel 1
//! in normal mode and reads VIF1's STAT (INT and VIS only, D-363 leaving FQC aside) and MASK, which an STMASK sets. Its
//! first case is D-189's stall, the code after the i bit already in the FIFO before STC; its second sends a NOP with
//! the i bit as a packet's last word, writes STC while INT is set and nothing follows, then sends an STMASK, which a
//! stall would hold. Run bare (`stc_bare`, tests/vif.rs) the program stores through kseg1; run by the fast boot on the
//! player's BIOS and on PCSX2 (`stc`, written as an ELF by examples/d1103.rs) it stores through a kernel-mode routine of
//! its own installed with SetSyscall (74h), as D-780's program does, and reads at the kuseg addresses.

#![allow(dead_code)]

use crate::s6::Program;

pub const D1_CHCR: u32 = 0x1000_9000;
pub const D1_MADR: u32 = 0x1000_9010;
pub const D1_QWC: u32 = 0x1000_9020;
pub const D_CTRL: u32 = 0x1000_E000;
pub const VIF1_STAT: u32 = 0x1000_3C00;
pub const VIF1_FBRST: u32 = 0x1000_3C10;
pub const VIF1_ERR: u32 = 0x1000_3C20;
pub const VIF1_MASK: u32 = 0x1000_3C70;
/// The packets' memory, physical; reached through kseg1 by the stores.
pub const PACKETS: u32 = 0x0030_0000;
/// The syscall the BIOS form installs for its stores: a0 the kseg1 address, a1 the value.
pub const KSTORE: u32 = 0x5A;
/// STAT's INT and VIS.
const FLAGS: u32 = 0xC00;
const STMASK: u32 = 0x2000_0000;
const NOP_I: u32 = 0x8000_0000;

fn li(p: &mut Program, reg: &str, v: u32) {
    p.emit(format!("lui {reg}, 0x{:x}", v >> 16));
    p.emit(format!("ori {reg}, {reg}, 0x{:x}", v & 0xFFFF));
}

#[derive(Clone, Copy, PartialEq)]
enum Form {
    Bare,
    Thread,
}

fn store(p: &mut Program, f: Form, addr: u32, v: u32) {
    match f {
        Form::Bare => {
            li(p, "t6", addr | 0xA000_0000);
            li(p, "t7", v);
            p.emit("sw t7, 0(t6)");
        }
        Form::Thread => {
            li(p, "a0", addr | 0xA000_0000);
            li(p, "a1", v);
            p.emit(format!("addiu v1, zero, 0x{KSTORE:x}"));
            p.emit("syscall 0");
            p.emit("nop");
        }
    }
    p.emit("sync");
}

fn read(p: &mut Program, f: Form, addr: u32, mask: u32) {
    li(p, "t6", if f == Form::Bare { addr | 0xA000_0000 } else { addr });
    p.emit("lw t2, 0(t6)");
    if mask != u32::MAX {
        li(p, "t7", mask);
        p.emit("and t2, t2, t7");
    }
}

/// About `n` loop turns, time for the VIF to take what it can.
fn pause(p: &mut Program, n: u32) {
    let l = p.label();
    li(p, "t3", n);
    p.place(&l);
    p.emit("addiu t3, t3, -1");
    p.emit(format!("bne t3, zero, {l}"));
    p.emit("nop");
}

/// The qwords written at `at` and sent by channel 1 in normal mode; then CHCR.STR read until clear or about 200,000
/// turns, and recorded (0, the transfer ended).
fn send(p: &mut Program, f: Form, at: u32, qwords: &[[u32; 4]]) {
    for (i, q) in qwords.iter().enumerate() {
        for (w, &v) in q.iter().enumerate() {
            store(p, f, at + 16 * i as u32 + 4 * w as u32, v);
        }
    }
    store(p, f, D1_MADR, at);
    store(p, f, D1_QWC, qwords.len() as u32);
    store(p, f, D1_CHCR, 0x101);
    let (turn, out) = (p.label(), p.label());
    li(p, "t3", 200_000);
    li(p, "t6", if f == Form::Bare { D1_CHCR | 0xA000_0000 } else { D1_CHCR });
    p.place(&turn);
    p.emit("lw t2, 0(t6)");
    p.emit("andi t2, t2, 0x100");
    p.emit(format!("beq t2, zero, {out}"));
    p.emit("nop");
    p.emit("addiu t3, t3, -1");
    p.emit(format!("bne t3, zero, {turn}"));
    p.emit("nop");
    p.place(&out);
    p.record("t2", Some(0));
    pause(p, 2_000);
}

fn flags_and_mask(p: &mut Program, f: Form, flags: u32, mask: u32) {
    read(p, f, VIF1_STAT, FLAGS);
    p.record("t2", Some(flags as u64));
    read(p, f, VIF1_MASK, u32::MAX);
    p.record("t2", Some(mask as u64));
}

fn stc(p: &mut Program, f: Form) {
    store(p, f, VIF1_FBRST, 8);
    pause(p, 2_000);
}

/// The program on the player's BIOS, its first `cases` cases (0 to 3).
pub fn stc_program(name: &'static str, cases: u32) -> Program {
    let mut p = Program::new(name);
    let (main, routine) = (p.label(), p.label());
    p.emit(format!("beq zero, zero, {main}"));
    p.emit("nop");
    let at = p.here();
    p.place(&routine);
    p.emit("sw a1, 0(a0)");
    p.emit("jr ra");
    p.emit("nop");
    p.place(&main);
    p.emit(format!("addiu a0, zero, 0x{KSTORE:x}"));
    li(&mut p, "a1", at);
    p.emit("addiu v1, zero, 0x74");
    p.emit("syscall 0");
    p.emit("nop");
    body(p, Form::Thread, cases)
}

/// The same program run bare from kseg0, the registers reached through kseg1.
pub fn stc_bare() -> Program {
    body(Program::kernel("d1103-stc-bare"), Form::Bare, 3)
}

/// The expected words are D-1103's reading, STC cancelling an i bit's stall not yet begun; the bare form also sets DMAE.
fn body(mut p: Program, f: Form, cases: u32) -> Program {
    if f == Form::Bare {
        store(&mut p, f, D_CTRL, 1);
    }
    store(&mut p, f, VIF1_FBRST, 1);
    store(&mut p, f, VIF1_ERR, 0);
    send(&mut p, f, PACKETS, &[[STMASK, 0x1111_1111, 0, 0]]);
    flags_and_mask(&mut p, f, 0, 0x1111_1111);
    if cases == 0 {
        return p;
    }
    // D-189: the next VIFcode already in the FIFO stalls, and STC lets it run.
    send(&mut p, f, PACKETS + 0x100, &[[NOP_I, STMASK, 0x2222_2222, 0]]);
    flags_and_mask(&mut p, f, 0xC00, 0x1111_1111);
    stc(&mut p, f);
    flags_and_mask(&mut p, f, 0, 0x2222_2222);
    if cases == 1 {
        return p;
    }
    // The i bit on a packet's last word: INT with nothing to stall yet; STC then; the next packet's STMASK.
    send(&mut p, f, PACKETS + 0x200, &[[0, 0, 0, NOP_I]]);
    flags_and_mask(&mut p, f, 0x800, 0x2222_2222);
    stc(&mut p, f);
    flags_and_mask(&mut p, f, 0, 0x2222_2222);
    send(&mut p, f, PACKETS + 0x300, &[[STMASK, 0x3333_3333, 0, 0]]);
    flags_and_mask(&mut p, f, 0, 0x3333_3333);
    if cases == 2 {
        return p;
    }
    // The same without STC: the next packet's STMASK stalls until it.
    send(&mut p, f, PACKETS + 0x400, &[[0, 0, 0, NOP_I]]);
    flags_and_mask(&mut p, f, 0x800, 0x3333_3333);
    send(&mut p, f, PACKETS + 0x500, &[[STMASK, 0x4444_4444, 0, 0]]);
    flags_and_mask(&mut p, f, 0xC00, 0x3333_3333);
    stc(&mut p, f);
    flags_and_mask(&mut p, f, 0, 0x4444_4444);
    store(&mut p, f, VIF1_FBRST, 1);
    p
}

pub fn all() -> Vec<Program> {
    vec![stc_program("d1103-stc", 3), stc_program("d1103-stc0", 0), stc_program("d1103-stc1", 1), stc_program("d1103-stc2", 2)]
}
