//! The project's programs for dispute D-800 (Sapphire_Native.md §30.68): EE programs entered by the fast boot on the
//! player's BIOS that poll I_STAT in a tight loop with interrupts enabled while a source the kernel serves (VBON,
//! through AddIntcHandler, 10h) or one the program serves (timer 0's compare) rises, and keep, for each arrival, the
//! loop's count when the poll first saw the source's bit, how many polls saw it, and the count when the handler ran.
//! On a console that takes the interrupt at the first instruction after the INTC asserts INT0 no poll sees the bit;
//! the count of polls that do is the referee's measure of the delay. Every word ungraded: the documents give no delay.

#![allow(dead_code)]

use crate::d700::{done, li, status, store, syscall};
use crate::s6::Program;

pub const I_STAT: u32 = 0x1000_F000;
pub const T0_COUNT: u32 = 0x1000_0000;
pub const T0_MODE: u32 = 0x1000_0010;
pub const T0_COMP: u32 = 0x1000_0020;
/// The log, cached memory: arrivals, the loop's count as last stored, then per arrival the first sighting's count,
/// the handler's count and the sightings, 16 words each.
pub const LOG: u32 = 0x0038_0000;
pub const FIRST: u32 = 0x40;
pub const AT: u32 = 0x80;
pub const SEEN: u32 = 0xC0;
/// The arrivals waited for; the first is not recorded, the loop having begun at an unknown point of its period.
pub const ARRIVALS: u32 = 9;
/// Timer 0 for `d800-poll-tim`: BUSCLK/256, ZRET, CUE, CMPE, compare 800h (about 56 lines).
pub const TIM_MODE: u32 = 0x2 | 0x40 | 0x80 | 0x100;
pub const TIM_COMP: u32 = 0x800;

/// The handler the kernel calls: the loop's count kept for this arrival, the arrivals counted; with `timer`, timer 0's
/// compare flag cleared by writing it back as 1.
fn handler(p: &mut Program, name: &str, timer: bool) {
    let skip = p.label();
    p.place(name);
    li(p, "t0", LOG);
    p.emit("lw t1, 0(t0)");
    p.emit("sltiu t2, t1, 16");
    p.emit(format!("beq t2, zero, {skip}"));
    p.emit("nop");
    p.emit("sll t2, t1, 2");
    p.emit("addu t3, t0, t2");
    p.emit("lw t5, 4(t0)");
    p.emit(format!("sw t5, 0x{AT:x}(t3)"));
    p.place(&skip);
    p.emit("addiu t1, t1, 1");
    p.emit("sw t1, 0(t0)");
    if timer {
        li(p, "t4", T0_MODE);
        li(p, "t5", TIM_MODE | 0x400);
        p.emit("sw t5, 0(t4)");
        p.emit("sync");
    }
    p.emit("jr ra");
    p.emit("or v0, zero, zero");
}

/// The loop: I_STAT read, the count stored, a sighting of `bit` kept for the current arrival, until `ARRIVALS` have come.
fn poll(p: &mut Program, bit: u32) {
    let (top, nobit, notfirst) = (p.label(), p.label(), p.label());
    li(p, "t6", I_STAT);
    li(p, "t0", LOG);
    p.emit("or s2, zero, zero");
    p.place(&top);
    p.emit("lw t2, 0(t6)");
    p.emit(format!("andi t2, t2, 0x{:x}", 1u32 << bit));
    p.emit(format!("beq t2, zero, {nobit}"));
    p.emit("sw s2, 4(t0)");
    p.emit("lw t3, 0(t0)");
    p.emit("sll t4, t3, 2");
    p.emit("addu t4, t0, t4");
    p.emit(format!("lw t5, 0x{SEEN:x}(t4)"));
    p.emit(format!("bne t5, zero, {notfirst}"));
    p.emit("addiu t5, t5, 1");
    p.emit(format!("sw s2, 0x{FIRST:x}(t4)"));
    p.place(&notfirst);
    p.emit(format!("sw t5, 0x{SEEN:x}(t4)"));
    p.place(&nobit);
    p.emit("lw t3, 0(t0)");
    p.emit(format!("sltiu t4, t3, {ARRIVALS}"));
    p.emit(format!("bne t4, zero, {top}"));
    p.emit("addiu s2, s2, 1");
}

fn program(name: &'static str, cause: u32, timer: bool) -> Program {
    let mut p = Program::new(name);
    let (main, hname) = (p.label(), p.label());
    p.emit(format!("beq zero, zero, {main}"));
    p.emit("nop");
    let at = p.here();
    handler(&mut p, &hname, timer);
    p.place(&main);
    for i in 0..0x40 {
        store(&mut p, LOG + 4 * i, 0);
    }
    p.emit("ei");
    p.emit("sync.p");
    status(&mut p);
    p.emit(format!("addiu a0, zero, {cause}"));
    li(&mut p, "a1", at);
    p.emit("or a2, zero, zero");
    p.emit("or a3, zero, zero");
    p.emit("or t0, zero, zero");
    syscall(&mut p, 0x10);
    p.emit("or s4, v0, zero");
    p.record("v0", None);
    p.emit(format!("addiu a0, zero, {cause}"));
    syscall(&mut p, 0x14);
    p.record("v0", None);
    if timer {
        store(&mut p, T0_MODE, 0xC00);
        store(&mut p, T0_COUNT, 0);
        store(&mut p, T0_COMP, TIM_COMP);
        store(&mut p, T0_MODE, TIM_MODE);
    }
    poll(&mut p, cause);
    if timer {
        store(&mut p, T0_MODE, 0xC00);
    }
    p.record("s2", None);
    status(&mut p);
    for k in 1..ARRIVALS {
        for off in [FIRST, AT, SEEN] {
            li(&mut p, "t6", LOG + off + 4 * k);
            p.emit("lw t2, 0(t6)");
            p.record("t2", None);
        }
    }
    p.emit(format!("addiu a0, zero, {cause}"));
    p.emit("or a1, s4, zero");
    syscall(&mut p, 0x11);
    done(&mut p);
    p
}

/// `d800-poll`: V-blank's start (cause 2), which the kernel serves and clears.
pub fn vbon() -> Program {
    program("d800-poll", 2, false)
}

/// `d800-poll-tim`: timer 0's compare (cause 9), served by the program's handler.
pub fn timer() -> Program {
    program("d800-poll-tim", 9, true)
}

/// `d800-ei`'s array of stores, cached memory, and the rounds.
pub const STORES: u32 = 0x0038_1000;
pub const SLOTS: u32 = 16;
pub const ROUNDS: u32 = 6;

/// The handler for `d800-ei`: counts the array's words the thread stored before it ran, keeps the count for this
/// arrival, and clears the array.
fn count_handler(p: &mut Program, name: &str) {
    let (skip, top) = (p.label(), p.label());
    p.place(name);
    li(p, "t0", STORES);
    p.emit("or t1, zero, zero");
    p.emit("or t2, zero, zero");
    p.place(&top);
    p.emit("lw t3, 0(t0)");
    p.emit("sw zero, 0(t0)");
    p.emit("sltu t3, zero, t3");
    p.emit("addu t1, t1, t3");
    p.emit("addiu t2, t2, 1");
    p.emit(format!("sltiu t3, t2, {SLOTS}"));
    p.emit(format!("bne t3, zero, {top}"));
    p.emit("addiu t0, t0, 4");
    li(p, "t0", LOG);
    p.emit("lw t4, 0(t0)");
    p.emit("sltiu t2, t4, 16");
    p.emit(format!("beq t2, zero, {skip}"));
    p.emit("nop");
    p.emit("sll t2, t4, 2");
    p.emit("addu t3, t0, t2");
    p.emit(format!("sw t1, 0x{AT:x}(t3)"));
    p.place(&skip);
    p.emit("addiu t4, t4, 1");
    p.emit("sw t4, 0(t0)");
    p.emit("jr ra");
    p.emit("or v0, zero, zero");
}

/// `d800-ei`: with interrupts disabled by DI the thread waits until V-blank's start is latched in I_STAT, then executes
/// EI followed by `SLOTS` stores, each one instruction; the handler counts the stores that ran before it. On a console
/// that takes a pending interrupt at the first instruction after EI the count is 0.
pub fn ei() -> Program {
    let mut p = Program::new("d800-ei");
    let (main, hname) = (p.label(), p.label());
    p.emit(format!("beq zero, zero, {main}"));
    p.emit("nop");
    let at = p.here();
    count_handler(&mut p, &hname);
    p.place(&main);
    for i in 0..0x40 {
        store(&mut p, LOG + 4 * i, 0);
    }
    for i in 0..SLOTS {
        store(&mut p, STORES + 4 * i, 0);
    }
    p.emit("ei");
    p.emit("sync.p");
    status(&mut p);
    p.emit("addiu a0, zero, 2");
    li(&mut p, "a1", at);
    p.emit("or a2, zero, zero");
    p.emit("or a3, zero, zero");
    p.emit("or t0, zero, zero");
    syscall(&mut p, 0x10);
    p.emit("or s4, v0, zero");
    p.record("v0", None);
    p.emit("addiu a0, zero, 2");
    syscall(&mut p, 0x14);
    p.record("v0", None);
    li(&mut p, "s3", STORES);
    li(&mut p, "t6", I_STAT);
    p.emit("addiu s5, zero, 1");
    for _ in 0..ROUNDS {
        let (wait, out) = (p.label(), p.label());
        p.emit("di");
        p.emit("sync.p");
        li(&mut p, "t5", 0x0400_0000);
        p.place(&wait);
        p.emit("lw t2, 0(t6)");
        p.emit("andi t2, t2, 4");
        p.emit(format!("bne t2, zero, {out}"));
        p.emit("addiu t5, t5, -1");
        p.emit(format!("bne t5, zero, {wait}"));
        p.emit("nop");
        p.place(&out);
        p.emit("ei");
        for i in 0..SLOTS {
            p.emit(format!("sw s5, {}(s3)", 4 * i));
        }
        p.emit("sync.p");
    }
    status(&mut p);
    for k in 0..ROUNDS {
        li(&mut p, "t6", LOG + AT + 4 * k);
        p.emit("lw t2, 0(t6)");
        p.record("t2", None);
    }
    li(&mut p, "t6", LOG);
    p.emit("lw t2, 0(t6)");
    p.record("t2", None);
    p.emit("addiu a0, zero, 2");
    p.emit("or a1, s4, zero");
    syscall(&mut p, 0x11);
    done(&mut p);
    p
}

/// `d800-tight`: the shortest poll, I_STAT read and tested in a loop of five instructions with interrupts enabled. At
/// a sighting of V-blank's start the polls since the last arrival are kept for the arrival due, and the thread waits,
/// polling nothing, until the handler has run; the polls are counted from zero again after each arrival, so a sighting
/// straight after a handler returns shows a small count and one before the interrupt about a field's polls.
pub fn tight() -> Program {
    let mut p = Program::new("d800-tight");
    let (main, hname) = (p.label(), p.label());
    p.emit(format!("beq zero, zero, {main}"));
    p.emit("nop");
    let at = p.here();
    handler(&mut p, &hname, false);
    p.place(&main);
    for i in 0..0x40 {
        store(&mut p, LOG + 4 * i, 0);
    }
    p.emit("ei");
    p.emit("sync.p");
    p.emit("addiu a0, zero, 2");
    li(&mut p, "a1", at);
    p.emit("or a2, zero, zero");
    p.emit("or a3, zero, zero");
    p.emit("or t0, zero, zero");
    syscall(&mut p, 0x10);
    p.emit("or s4, v0, zero");
    p.emit("addiu a0, zero, 2");
    syscall(&mut p, 0x14);
    li(&mut p, "t6", I_STAT);
    li(&mut p, "t0", LOG);
    let (top, seen, wait, next, out) = (p.label(), p.label(), p.label(), p.label(), p.label());
    p.emit("lw s6, 0(t0)");
    p.emit("or s2, zero, zero");
    p.place(&top);
    p.emit("lw t2, 0(t6)");
    p.emit("andi t2, t2, 4");
    p.emit(format!("bne t2, zero, {seen}"));
    p.emit("nop");
    p.emit("lw t3, 0(t0)");
    p.emit(format!("beq t3, s6, {top}"));
    p.emit("addiu s2, s2, 1");
    p.emit(format!("beq zero, zero, {next}"));
    p.emit("nop");
    p.place(&seen);
    p.emit("sll t4, s6, 2");
    p.emit("addu t4, t0, t4");
    p.emit(format!("lw t5, 0x{SEEN:x}(t4)"));
    p.emit("addiu t5, t5, 1");
    p.emit(format!("sw t5, 0x{SEEN:x}(t4)"));
    p.emit(format!("sw s2, 0x{FIRST:x}(t4)"));
    p.place(&wait);
    p.emit("lw t3, 0(t0)");
    p.emit(format!("beq t3, s6, {wait}"));
    p.emit("nop");
    p.place(&next);
    p.emit("or s6, t3, zero");
    p.emit(format!("sltiu t4, s6, {ARRIVALS}"));
    p.emit(format!("beq t4, zero, {out}"));
    p.emit("nop");
    p.emit(format!("beq zero, zero, {top}"));
    p.emit("or s2, zero, zero");
    p.place(&out);
    for k in 1..ARRIVALS {
        for off in [FIRST, SEEN] {
            li(&mut p, "t6", LOG + off + 4 * k);
            p.emit("lw t2, 0(t6)");
            p.record("t2", None);
        }
    }
    p.emit("addiu a0, zero, 2");
    p.emit("or a1, s4, zero");
    syscall(&mut p, 0x11);
    done(&mut p);
    p
}

pub fn all() -> Vec<Program> {
    vec![vbon(), timer(), ei(), tight()]
}
