//! The project's programs for disputes D-700 and D-701 (Sapphire_Native.md §30.52 to §30.59): EE programs entered by
//! the fast boot on the player's BIOS, on Sapphire and on a referee, that read EE timer 3, the INTC and COP0 Status
//! themselves. `d700-timer` records how the kernel left timer 3 and the INTC at the program's entry and timer 3's count
//! against timer 0's on BUSCLK/256, then programs timer 3 as the EE User's Manual's chapter 4 says, with a handler for
//! the INTC's cause 12 added through the kernel's AddIntcHandler (ps2tek "BIOS EE Syscalls", 10h) and the mask through
//! _EnableIntc (14h), and records each compare interrupt's arrival. `d700-alarm` calls SetAlarm (FCh) and ReleaseAlarm
//! (FEh), the numbers ps2tek's "BIOS EE Patches" gives, whose arguments no permitted document gives: their raw results
//! and timer 3's registers around them, ungraded. `d700-thread`, `d700-exec` and `d700-kmode` record Status.EIE in a
//! thread the kernel creates, in a main thread ExecPS2 starts, and inside a kernel-mode routine. Registers are read at
//! their kuseg addresses and timer 3's written through kseg1 by a routine installed with SetSyscall (74h), since the
//! kernel maps timer 3's page read-only to its threads; the record is cached memory written back by FlushCache (64h).
//! `d760-kdi` and `d760-hazard` (D-760, Sapphire_Native.md §30.60 to §30.67) read Status straight after EI, DI and
//! MTC0 in a kernel-mode routine. Expected words are the manual's; whatever depends on the kernel is ungraded.

#![allow(dead_code)]

use crate::s6::Program;

pub const T0_COUNT: u32 = 0x1000_0000;
pub const T0_MODE: u32 = 0x1000_0010;
pub const T3_COUNT: u32 = 0x1000_1800;
pub const T3_MODE: u32 = 0x1000_1810;
pub const T3_COMP: u32 = 0x1000_1820;
pub const I_STAT: u32 = 0x1000_F000;
pub const I_MASK: u32 = 0x1000_F010;
/// The handler's log, cached memory: the arrivals, then MODE, COUNT and I_STAT at each of the first eight.
pub const LOG: u32 = 0x0038_0000;
/// `d700-thread`'s words shared by its two threads (the child's Status, the main thread's id), its ThreadParam and the
/// child's stack.
pub const SHARED: u32 = 0x0038_6000;
pub const PARAM: u32 = 0x0038_4000;
pub const STACK: u32 = 0x0039_0000;
/// Timer 3's setting for part B: BUSCLK/256, ZRET, CUE, CMPE; the compare value; the arrivals waited for.
pub const B_MODE: u32 = 0x2 | 0x40 | 0x80 | 0x100;
pub const B_COMP: u32 = 0x1000;
pub const ARRIVALS: u32 = 4;
/// The syscall the program installs through SetSyscall (74h) for its stores to timer 3: 5Ah, which ps2tek's "BIOS EE
/// Patches" says the SDKs take for their own kernel-mode routine.
pub const KSTORE: u32 = 0x5A;
/// The one value SetAlarm is given in a0 in part C, its meaning unknown.
pub const C_TICKS: u32 = 0x100;

pub fn li(p: &mut Program, reg: &str, v: u32) {
    p.emit(format!("lui {reg}, 0x{:x}", v >> 16));
    p.emit(format!("ori {reg}, {reg}, 0x{:x}", v & 0xFFFF));
}

pub fn store(p: &mut Program, addr: u32, v: u32) {
    li(p, "t6", addr);
    li(p, "t7", v);
    p.emit("sw t7, 0(t6)");
}

/// A word stored at timer 3's register through kseg1 by the program's own syscall, in kernel mode: the kernel maps
/// timer 3's page read-only to its threads.
fn kstore(p: &mut Program, addr: u32, v: u32) {
    li(p, "a0", 0xA000_0000 | addr);
    li(p, "a1", v);
    syscall(p, KSTORE);
}

fn load(p: &mut Program, addr: u32) {
    li(p, "t6", addr);
    p.emit("lw t2, 0(t6)");
}

fn rec(p: &mut Program, addr: u32, v: Option<u64>) {
    load(p, addr);
    p.record("t2", v);
}

/// Records COP0 Status, which the thread may read while the kernel leaves CU0 set.
pub fn status(p: &mut Program) {
    p.emit("mfc0 t2, Status");
    p.emit("sync.p");
    p.record("t2", None);
}

pub fn syscall(p: &mut Program, n: u32) {
    p.emit(format!("addiu v1, zero, 0x{n:x}"));
    p.emit("syscall 0");
    p.emit("nop");
}

/// The handler the kernel calls with the cause in a0: logs the arrival and returns 0; the flag is cleared by the program.
fn handler(p: &mut Program, name: &str) {
    let skip = p.label();
    p.place(name);
    li(p, "t0", LOG);
    p.emit("lw t1, 0(t0)");
    p.emit("sltiu t2, t1, 8");
    p.emit(format!("beq t2, zero, {skip}"));
    p.emit("nop");
    p.emit("sll t2, t1, 2");
    p.emit("addu t3, t0, t2");
    li(p, "t4", T3_MODE);
    p.emit("lw t5, 0(t4)");
    p.emit("sw t5, 4(t3)");
    li(p, "t4", T3_COUNT);
    p.emit("lw t5, 0(t4)");
    p.emit("sw t5, 0x24(t3)");
    li(p, "t4", I_STAT);
    p.emit("lw t5, 0(t4)");
    p.emit("sw t5, 0x44(t3)");
    p.place(&skip);
    p.emit("addiu t1, t1, 1");
    p.emit("sw t1, 0(t0)");
    p.emit("jr ra");
    p.emit("or v0, zero, zero");
}

/// Waits, bounded, until the word at `addr` masked by `mask` is at least `at_least`.
fn wait_at_least(p: &mut Program, addr: u32, mask: u32, at_least: u32) {
    let (top, out) = (p.label(), p.label());
    li(p, "t5", 0x0100_0000);
    li(p, "t6", addr);
    p.place(&top);
    p.emit("lw t2, 0(t6)");
    li(p, "t7", mask);
    p.emit("and t2, t2, t7");
    p.emit(format!("sltiu t3, t2, 0x{at_least:x}"));
    p.emit(format!("beq t3, zero, {out}"));
    p.emit("nop");
    p.emit("addiu t5, t5, -1");
    p.emit(format!("bne t5, zero, {top}"));
    p.emit("nop");
    p.place(&out);
}

/// The program's start: a jump over the handler, the handler, the log cleared.
fn start(name: &'static str) -> (Program, u32) {
    let mut p = Program::new(name);
    let (main, hname) = (p.label(), p.label());
    p.emit(format!("beq zero, zero, {main}"));
    p.emit("nop");
    let at = p.here();
    handler(&mut p, &hname);
    let kstore = p.here();
    p.emit("sw a1, 0(a0)");
    p.emit("jr ra");
    p.emit("nop");
    p.place(&main);
    status(&mut p);
    p.emit(format!("addiu a0, zero, 0x{KSTORE:x}"));
    li(&mut p, "a1", kstore);
    syscall(&mut p, 0x74);
    for i in 0..32 {
        store(&mut p, LOG + 4 * i, 0);
    }
    (p, at)
}

/// The done word written and the data cache written back, as the kernel's FlushCache(0) does.
pub fn done(p: &mut Program) {
    store(p, 0x001F_FFF0, 0x5A99_D00E);
    p.emit("or a0, zero, zero");
    syscall(p, 0x64);
}

/// `d700-timer`: parts A and B.
pub fn timer() -> Program {
    let (mut p, hname) = start("d700-timer");
    status(&mut p);
    // A: timer 3 and the INTC as the kernel left them at the program's entry.
    rec(&mut p, T3_MODE, None);
    rec(&mut p, T3_COMP, None);
    rec(&mut p, T3_COUNT, None);
    rec(&mut p, I_MASK, None);
    rec(&mut p, I_STAT, None);
    // Timer 3's ticks while timer 0 counts 1000h on BUSCLK/256 (1,048,576 BUSCLK).
    store(&mut p, T0_MODE, 0xC00);
    store(&mut p, T0_COUNT, 0);
    store(&mut p, T0_MODE, 0x82);
    load(&mut p, T3_COUNT);
    p.emit("or s2, t2, zero");
    wait_at_least(&mut p, T0_COUNT, 0xFFFF, 0x1000);
    load(&mut p, T3_COUNT);
    p.emit("subu t2, t2, s2");
    p.emit("andi t2, t2, 0xffff");
    p.record("t2", None);
    store(&mut p, T0_MODE, 0);
    rec(&mut p, T3_MODE, None);
    // B: timer 3 programmed by the manual, its compare taken through the kernel's handler for cause 12.
    p.emit("addiu a0, zero, 12");
    li(&mut p, "a1", hname);
    p.emit("or a2, zero, zero");
    p.emit("or a3, zero, zero");
    p.emit("or t0, zero, zero");
    syscall(&mut p, 0x10);
    p.emit("or s4, v0, zero");
    p.record("v0", None);
    status(&mut p);
    p.emit("addiu a0, zero, 12");
    syscall(&mut p, 0x14);
    p.record("v0", None);
    status(&mut p);
    rec(&mut p, I_MASK, None);
    load(&mut p, T3_MODE);
    p.emit("or s5, t2, zero");
    load(&mut p, T3_COMP);
    p.emit("or s6, t2, zero");
    kstore(&mut p, T3_MODE, 0xC00);
    kstore(&mut p, T3_COUNT, 0);
    kstore(&mut p, T3_COMP, B_COMP);
    kstore(&mut p, T3_MODE, B_MODE);
    for n in 1..=ARRIVALS {
        wait_at_least(&mut p, LOG, 0xFFFF_FFFF, n);
        if n == 1 {
            // Timer 3 and I_STAT after the first wait, before its flag is cleared: EQUF set, whatever the handler did.
            rec(&mut p, T3_MODE, Some(u64::from(B_MODE | 0x400)));
            rec(&mut p, T3_COUNT, None);
            rec(&mut p, I_STAT, None);
            status(&mut p);
        }
        kstore(&mut p, T3_MODE, B_MODE | 0x400);
    }
    kstore(&mut p, T3_MODE, 0xC00);
    load(&mut p, LOG);
    p.emit(format!("sltiu t2, t2, 0x{ARRIVALS:x}"));
    p.record("t2", Some(0));
    for n in 0..ARRIVALS {
        rec(&mut p, LOG + 4 + 4 * n, Some(u64::from(B_MODE | 0x400)));
    }
    for n in 0..ARRIVALS {
        rec(&mut p, LOG + 0x24 + 4 * n, None);
    }
    for n in 0..ARRIVALS {
        rec(&mut p, LOG + 0x44 + 4 * n, None);
    }
    p.emit("addiu a0, zero, 12");
    p.emit("or a1, s4, zero");
    syscall(&mut p, 0x11);
    p.record("v0", None);
    // Timer 3 put back as the kernel left it, its count excepted.
    li(&mut p, "a0", 0xA000_0000 | T3_COMP);
    p.emit("or a1, s6, zero");
    syscall(&mut p, KSTORE);
    li(&mut p, "a0", 0xA000_0000 | T3_MODE);
    p.emit("andi a1, s5, 0x3ff");
    p.emit("ori a1, a1, 0xc00");
    syscall(&mut p, KSTORE);
    rec(&mut p, T3_MODE, None);
    done(&mut p);
    p
}

/// `d700-alarm`: part C, SetAlarm (FCh) with one value in a0 and zeros in a1 and a2, and ReleaseAlarm (FEh) with SetAlarm's
/// result in a0, each with timer 3's registers and I_MASK before and after; every word ungraded, the arguments' meaning
/// unknown (decided 2026-10-26 by the tester).
pub fn alarm() -> Program {
    let (mut p, _) = start("d700-alarm");
    rec(&mut p, T3_MODE, None);
    rec(&mut p, T3_COMP, None);
    rec(&mut p, I_MASK, None);
    li(&mut p, "a0", C_TICKS);
    p.emit("or a1, zero, zero");
    p.emit("or a2, zero, zero");
    syscall(&mut p, 0xFC);
    p.emit("or s7, v0, zero");
    p.record("v0", None);
    rec(&mut p, T3_MODE, None);
    rec(&mut p, T3_COMP, None);
    rec(&mut p, T3_COUNT, None);
    rec(&mut p, I_MASK, None);
    // About 1,048,576 BUSCLK (112 lines) by timer 0, then timer 3 again.
    store(&mut p, T0_MODE, 0xC00);
    store(&mut p, T0_COUNT, 0);
    store(&mut p, T0_MODE, 0x82);
    wait_at_least(&mut p, T0_COUNT, 0xFFFF, 0x1000);
    store(&mut p, T0_MODE, 0);
    rec(&mut p, T3_MODE, None);
    rec(&mut p, T3_COMP, None);
    rec(&mut p, T3_COUNT, None);
    p.emit("or a0, s7, zero");
    syscall(&mut p, 0xFE);
    p.record("v0", None);
    rec(&mut p, T3_MODE, None);
    rec(&mut p, T3_COMP, None);
    done(&mut p);
    p
}

/// `d700-thread`: a thread made by the kernel's CreateThread (20h) and StartThread (22h) records COP0 Status as it
/// begins, wakes the main thread (WakeupThread, 33h) and exits (ExitThread, 23h); the main thread sleeps meanwhile
/// (SleepThread, 32h). Every word ungraded: what is compared is the kernel's thread's Status on each console.
pub fn thread() -> Program {
    thread_after("d700-thread", None)
}

/// `d700-thread` with the main thread's EI or DI first (Status.EDI lets a thread use them), so that the child's
/// Status shows whether the kernel takes EIE from the thread that creates it.
pub fn thread_after(name: &'static str, first: Option<&str>) -> Program {
    let mut p = Program::new(name);
    let main = p.label();
    p.emit(format!("beq zero, zero, {main}"));
    p.emit("nop");
    let child = p.here();
    p.emit("mfc0 t2, Status");
    p.emit("sync.p");
    li(&mut p, "t0", SHARED);
    p.emit("sw t2, 0(t0)");
    p.emit("lw a0, 4(t0)");
    syscall(&mut p, 0x33);
    syscall(&mut p, 0x23);
    let spin = p.label();
    p.place(&spin);
    p.emit(format!("beq zero, zero, {spin}"));
    p.emit("nop");
    p.place(&main);
    status(&mut p);
    if let Some(op) = first {
        p.emit(op);
        p.emit("sync.p");
    }
    status(&mut p);
    store(&mut p, SHARED, 0);
    syscall(&mut p, 0x2F);
    p.record("v0", None);
    li(&mut p, "t0", SHARED);
    p.emit("sw v0, 4(t0)");
    for (i, v) in [0, child, STACK, 0x2000, 0, 1, 0, 0, 0].into_iter().enumerate() {
        store(&mut p, PARAM + 4 * i as u32, v);
    }
    li(&mut p, "a0", PARAM);
    syscall(&mut p, 0x20);
    p.emit("or s3, v0, zero");
    p.record("v0", None);
    p.emit("or a0, s3, zero");
    p.emit("or a1, zero, zero");
    syscall(&mut p, 0x22);
    status(&mut p);
    syscall(&mut p, 0x32);
    rec(&mut p, SHARED, None);
    status(&mut p);
    done(&mut p);
    p
}

/// The registers `d700-exec` keeps at each pass: the INTC's, the DMAC's D_CTRL, D_STAT and D_PCR, timer 3's MODE, the SIF's
/// MSCOM, SMCOM, MSFLG, SMFLG and CTRL, and SIF1's CHCR.
pub const EXEC_REGS: [u32; 12] =
    [I_STAT, I_MASK, 0x1000_E000, 0x1000_E010, 0x1000_E020, T3_MODE, 0x1000_F200, 0x1000_F210, 0x1000_F220, 0x1000_F230, 0x1000_F240, 0x1000_C400];

/// `d700-exec`: the program calls the kernel's ExecPS2 (07h) on itself twice, once after EI and once after DI, and
/// keeps COP0 Status as each main thread begins, so that the main thread's Status.EIE shows whether ExecPS2 takes it
/// from its caller. The pass count and each pass's word are kept in the program's own image, which ExecPS2 does not
/// load again, written back by FlushCache before each call.
pub fn exec() -> Program {
    exec_with("d700-exec", false)
}

/// `d700-exec`, with SMFLG's bit 18 (40000h, "IOP has finished booting", ps2tek "SIF Registers") cleared by the EE
/// before each ExecPS2 when `clear` is set.
pub fn exec_with(name: &'static str, clear: bool) -> Program {
    let mut p = Program::new(name);
    let (second, third, go) = (p.label(), p.label(), p.label());
    p.emit(format!("beq zero, zero, {go}"));
    p.emit("nop");
    let words = p.here();
    for _ in 0..(1 + 3 * EXEC_REGS.len() as u32 + 3) {
        p.emit(".word 0");
    }
    p.place(&go);
    p.emit("mfc0 t3, Status");
    p.emit("sync.p");
    li(&mut p, "t0", words);
    p.emit("lw t1, 0(t0)");
    p.emit(format!("addiu t2, zero, {}", 4 * (EXEC_REGS.len() + 1)));
    p.emit("mult t2, t1, t2");
    p.emit("addu t2, t2, t0");
    p.emit("sw t3, 4(t2)");
    for (i, &r) in EXEC_REGS.iter().enumerate() {
        li(&mut p, "t6", r);
        p.emit("lw t4, 0(t6)");
        p.emit(format!("sw t4, {}(t2)", 8 + 4 * i));
    }
    p.emit("addiu t1, t1, 1");
    p.emit("sw t1, 0(t0)");
    p.emit("addiu t2, zero, 2");
    p.emit(format!("beq t1, t2, {second}"));
    p.emit("nop");
    p.emit("addiu t2, zero, 3");
    p.emit(format!("beq t1, t2, {third}"));
    p.emit("nop");
    for op in ["ei", "di"] {
        if clear {
            store(&mut p, 0x1000_F230, 0x4_0000);
        }
        p.emit("sync.p");
        p.emit("or a0, zero, zero");
        syscall(&mut p, 0x64);
        p.emit(op);
        p.emit("sync.p");
        li(&mut p, "a0", crate::s6::ORIGIN);
        p.emit("or a1, zero, zero");
        p.emit("or a2, zero, zero");
        p.emit("or a3, zero, zero");
        syscall(&mut p, 0x07);
        let spin = p.label();
        p.place(&spin);
        p.emit(format!("beq zero, zero, {spin}"));
        p.emit("nop");
        p.place(if op == "ei" { &second } else { &third });
    }
    for i in 0..3 * (EXEC_REGS.len() as u32 + 1) {
        rec(&mut p, words + 4 + 4 * i, None);
    }
    done(&mut p);
    p
}

/// `d700-kmode`: a routine installed as syscall 5Ah records COP0 Status as the kernel calls it, then after DI, after
/// EI, after an MTC0 clearing EIE and after an MTC0 putting Status back; the main thread calls it once after EI and once
/// after DI and records Status on its return. Every word ungraded.
pub fn kmode() -> Program {
    let mut p = Program::new("d700-kmode");
    let main = p.label();
    p.emit(format!("beq zero, zero, {main}"));
    p.emit("nop");
    let routine = p.here();
    li(&mut p, "t0", LOG);
    p.emit("lw t1, 0x7c(t0)");
    p.emit("addu t0, t0, t1");
    p.emit("mfc0 t2, Status");
    p.emit("sync.p");
    p.emit("or t3, t2, zero");
    p.emit("sw t2, 0(t0)");
    p.emit("di");
    p.emit("sync.p");
    p.emit("mfc0 t2, Status");
    p.emit("sw t2, 4(t0)");
    p.emit("ei");
    p.emit("sync.p");
    p.emit("mfc0 t2, Status");
    p.emit("sw t2, 8(t0)");
    li(&mut p, "t4", !(1u32 << 16));
    p.emit("and t2, t3, t4");
    p.emit("mtc0 t2, Status");
    p.emit("sync.p");
    p.emit("mfc0 t2, Status");
    p.emit("sw t2, 0xc(t0)");
    p.emit("mtc0 t3, Status");
    p.emit("sync.p");
    p.emit("mfc0 t2, Status");
    p.emit("sw t2, 0x10(t0)");
    p.emit("jr ra");
    p.emit("nop");
    p.place(&main);
    for i in 0..32 {
        store(&mut p, LOG + 4 * i, 0);
    }
    p.emit(format!("addiu a0, zero, 0x{KSTORE:x}"));
    li(&mut p, "a1", routine);
    syscall(&mut p, 0x74);
    for (k, op) in ["ei", "di"].into_iter().enumerate() {
        store(&mut p, LOG + 0x7C, 0x20 * k as u32);
        p.emit(op);
        p.emit("sync.p");
        status(&mut p);
        syscall(&mut p, KSTORE);
        status(&mut p);
        for i in 0..5 {
            rec(&mut p, LOG + 0x20 * k as u32 + 4 * i, None);
        }
    }
    done(&mut p);
    p
}

/// `d760-kdi` (D-760): a routine installed as syscall 5Ah executes DI, or EI, in kernel mode and returns without
/// undoing it; it records Status before the operation, straight after it with no SYNC.P between, and after a SYNC.P.
/// The main thread calls the DI routine after EI and the EI routine after DI, and records Status before each call and
/// on its return, so that the record shows what the kernel's return from a system call leaves of a change made inside.
pub fn kdi() -> Program {
    let mut p = Program::new("d760-kdi");
    let main = p.label();
    p.emit(format!("beq zero, zero, {main}"));
    p.emit("nop");
    let mut routines = Vec::new();
    for op in ["di", "ei"] {
        routines.push(p.here());
        li(&mut p, "t0", LOG);
        p.emit("lw t1, 0x7c(t0)");
        p.emit("addu t0, t0, t1");
        p.emit("mfc0 t2, Status");
        p.emit("sync.p");
        p.emit("sw t2, 0(t0)");
        p.emit(op);
        p.emit("mfc0 t2, Status");
        p.emit("sync.p");
        p.emit("sw t2, 4(t0)");
        p.emit("mfc0 t2, Status");
        p.emit("sw t2, 8(t0)");
        p.emit("jr ra");
        p.emit("nop");
    }
    p.place(&main);
    for i in 0..32 {
        store(&mut p, LOG + 4 * i, 0);
    }
    for (k, (first, routine)) in [("ei", routines[0]), ("di", routines[1])].into_iter().enumerate() {
        p.emit(format!("addiu a0, zero, 0x{KSTORE:x}"));
        li(&mut p, "a1", routine);
        syscall(&mut p, 0x74);
        store(&mut p, LOG + 0x7C, 0x20 * k as u32);
        p.emit(first);
        p.emit("sync.p");
        status(&mut p);
        syscall(&mut p, KSTORE);
        status(&mut p);
        for i in 0..3 {
            rec(&mut p, LOG + 0x20 * k as u32 + 4 * i, None);
        }
    }
    done(&mut p);
    p
}

/// `d760-hazard` (D-760): in a kernel-mode routine installed as syscall 5Ah, Status read by MFC0 after DI, after EI
/// and after an MTC0 clearing or setting EIE, with none to three NOPs between and none after a SYNC.P; each operation
/// starts from EIE in the other state, settled by a SYNC.P; then an MTC0 straight after DI or EI, and DI and EI back to
/// back, each read one instruction later. The routine puts Status back as it found it.
pub fn hazard() -> Program {
    let mut p = Program::new("d760-hazard");
    let main = p.label();
    p.emit(format!("beq zero, zero, {main}"));
    p.emit("nop");
    let routine = p.here();
    li(&mut p, "t0", LOG);
    p.emit("mfc0 t3, Status");
    p.emit("sync.p");
    li(&mut p, "t4", !(1u32 << 16));
    p.emit("and t5, t3, t4");
    li(&mut p, "t4", 1u32 << 16);
    p.emit("or t6, t3, t4");
    let mut at = 0;
    for (set, op) in [("ei", "di"), ("di", "ei"), ("mtc0 t6, Status", "mtc0 t5, Status"), ("mtc0 t5, Status", "mtc0 t6, Status")] {
        for gap in 0..5 {
            p.emit(set);
            p.emit("sync.p");
            p.emit(op);
            if gap == 4 {
                p.emit("sync.p");
            } else {
                for _ in 0..gap {
                    p.emit("nop");
                }
            }
            p.emit("mfc0 t2, Status");
            p.emit("sync.p");
            p.emit(format!("sw t2, {at}(t0)"));
            at += 4;
        }
    }
    let seqs: [&[&str]; 4] = [
        &["ei", "sync.p", "di", "mtc0 t6, Status", "sync.p"],
        &["di", "sync.p", "ei", "mtc0 t5, Status", "sync.p"],
        &["ei", "sync.p", "di", "ei"],
        &["di", "sync.p", "ei", "di"],
    ];
    for seq in seqs {
        for &line in seq {
            p.emit(line);
        }
        p.emit("mfc0 t2, Status");
        p.emit("sync.p");
        p.emit(format!("sw t2, {at}(t0)"));
        at += 4;
    }
    p.emit("mtc0 t3, Status");
    p.emit("sync.p");
    p.emit("jr ra");
    p.emit("nop");
    p.place(&main);
    for i in 0..32 {
        store(&mut p, LOG + 4 * i, 0);
    }
    p.emit(format!("addiu a0, zero, 0x{KSTORE:x}"));
    li(&mut p, "a1", routine);
    syscall(&mut p, 0x74);
    p.emit("ei");
    p.emit("sync.p");
    syscall(&mut p, KSTORE);
    status(&mut p);
    for i in 0..24 {
        rec(&mut p, LOG + 4 * i, None);
    }
    done(&mut p);
    p
}

pub fn all() -> Vec<Program> {
    vec![
        timer(),
        alarm(),
        thread(),
        thread_after("d700-thread-ei", Some("ei")),
        thread_after("d700-thread-di", Some("di")),
        exec(),
        exec_with("d700-exec-clear", true),
        kmode(),
        kdi(),
        hazard(),
    ]
}
