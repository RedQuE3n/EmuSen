//! The exception vector, the syscalls, the priority chains with the hook, unresolved exceptions through A(40h),
//! interrupts with the documented events, and the GTE command an interrupt meets.

use diamond_kernel_harness::image;
use diamond_kernel_harness::machine::Machine;

const LOG: u32 = 0x8018_0000;
const LOG_POINTER: u32 = 0x8017_0FF0;

fn booted() -> Machine {
    let mut m = Machine::boot(&image::committed());
    m.rig.put(LOG_POINTER, LOG);
    m
}

/// A chain function at `at` that appends `mark` to the log, stores a0 after it, and returns `value`.
fn marker(m: &mut Machine, at: u32, mark: char, value: u32) {
    m.code(
        at,
        &format!(
            "lui t0, 0x{:x}\nlw t1, 0x{:x}(t0)\nnop\naddiu t2, zero, {}\nsb t2, 0(t1)\nsb a0, 1(t1)\naddiu t1, t1, 1\nsw t1, 0x{:x}(t0)\nlui v0, 0x{:x}\njr ra\nori v0, v0, 0x{:x}",
            LOG_POINTER >> 16,
            LOG_POINTER & 0xFFFF,
            mark as u32,
            LOG_POINTER & 0xFFFF,
            value >> 16,
            value & 0xFFFF
        ),
    );
}

fn log(m: &Machine) -> String {
    let n = m.rig.word(LOG_POINTER) - LOG;
    String::from_utf8(m.rig.bytes(LOG, n as usize)).unwrap()
}

fn element(m: &mut Machine, at: u32, first: u32, second: u32) -> u32 {
    m.rig.put(at, 0xDEAD_BEEF);
    m.rig.put(at + 4, second);
    m.rig.put(at + 8, first);
    m.rig.put(at + 12, 0);
    at
}

/// An exit structure whose RA acknowledges every IRQ and returns from the exception.
fn acknowledging_hook(m: &mut Machine) {
    m.code(0x8015_0300, "lui t1, 0x1F80\nsw zero, 0x1070(t1)\naddiu t2, zero, 0xB0\njr t2\naddiu t1, zero, 0x17");
    m.rig.put(0x8016_0200, 0x8015_0300);
    m.rig.put(0x8016_0204, 0x801F_C000);
    m.b(0x19, &[0x8016_0200]);
}

#[test]
fn enter_and_exit_critical_section_by_syscall() {
    let mut m = booted();
    m.run_program(
        "addiu a0, zero, 1\nsyscall\nor s0, v0, zero\naddiu a0, zero, 1\nsyscall\nor s1, v0, zero\nmfc0 s2, SR\nnop\n\
         addiu a0, zero, 2\nsyscall\nmfc0 s3, SR\nnop\naddiu a0, zero, 0\naddiu v0, zero, 7\nsyscall\nor s4, v0, zero\ndone: j done\nnop",
        10_000,
    );
    let r = &m.cpu.regs.r;
    assert_eq!((r[16], r[17]), (1, 0), "1 when IEc and IM2 were both set, then 0");
    assert_eq!(r[18] & 0x405, 0, "IEc and IM2 clear after EnterCriticalSection");
    assert_eq!(r[19] & 0x405, 0x401, "and set after ExitCriticalSection");
    assert_eq!(r[20], 7, "syscall 0 changes no register");
    assert_eq!(m.exceptions, [8, 8, 8, 8]);
    let tcb = m.rig.word(m.rig.word(0x8000_0108));
    assert_eq!(m.rig.word(tcb + 0x88), 0x8001_0000 + 4 * 15, "TCB[88h]: the EPC after the last syscall");
    assert_eq!(m.rig.word(tcb + 0x98) >> 2 & 0x1F, 8, "TCB[98h]: its Cause");
}

#[test]
fn an_unknown_syscall_delivers_f0000010h_4000h_and_returns() {
    let mut m = booted();
    let e = m.b(0x08, &[0xF000_0010, 0x4000, 0x2000, 0]);
    assert_eq!(e, 0xF100_0000);
    assert_eq!(m.b(0x0C, &[e]), 1);
    m.run_program("addiu a0, zero, 4\nsyscall\naddiu s0, zero, 9\ndone: j done\nnop", 10_000);
    assert_eq!(m.cpu.regs.r[16], 9);
    assert_eq!(m.b(0x0B, &[e]), 1, "ready");
    assert_eq!(m.b(0x0B, &[e]), 0, "and busy again once tested");
}

#[test]
fn an_unresolved_exception_delivers_its_event_and_calls_a40h_through_the_table() {
    let mut m = booted();
    let e = m.b(0x08, &[0xF000_0010, 0x1000, 0x2000, 0]);
    m.b(0x0C, &[e]);
    // The handler, as a test program installs one at A0[40h]: the flag set, the TCB's EPC moved past the fault.
    m.code(
        0x8015_0000,
        "lui t0, 0x8000\nlw t0, 0x108(t0)\nnop\nlw t0, 0(t0)\nnop\nlw t1, 0x88(t0)\nnop\naddiu t1, t1, 4\nsw t1, 0x88(t0)\n\
         lui t2, 0x8017\naddiu t3, zero, 1\njr ra\nsw t3, 0(t2)",
    );
    m.rig.put(0x8000_0200 + 4 * 0x40, 0x8015_0000);
    m.run_program(
        "break\naddiu s0, zero, 5\nlw s1, 0(zero)\nlui t0, 0x8000\nlw s1, 1(t0)\naddiu s2, zero, 6\ndone: j done\nnop",
        10_000,
    );
    assert_eq!(m.exceptions, [9, 4]);
    assert_eq!((m.cpu.regs.r[16], m.cpu.regs.r[18]), (5, 6), "both faults returned past");
    assert_eq!(m.rig.word(0x8017_0000), 1);
    assert_eq!(m.b(0x0B, &[e]), 1, "F0000010h,1000h delivered");
}

#[test]
fn the_chains_run_by_priority_newest_first_then_the_exit_structure() {
    let mut m = booted();
    marker(&mut m, 0x8015_0000, 'S', 0);
    marker(&mut m, 0x8015_0100, 'F', 0);
    marker(&mut m, 0x8015_0200, 'D', 0);
    let first = element(&mut m, 0x8016_0000, 0x8015_0000, 0);
    let second = element(&mut m, 0x8016_0010, 0x8015_0100, 0);
    let low = element(&mut m, 0x8016_0020, 0x8015_0200, 0);
    m.c(0x02, &[0, first]);
    m.c(0x02, &[0, second]);
    m.c(0x02, &[2, low]);
    acknowledging_hook(&mut m);
    let e = m.b(0x08, &[0xF000_0003, 0x1000, 0x2000, 0]);
    m.b(0x0C, &[e]);
    m.rig.i_mask = 1 << 2;
    let exe = diamond_kernel_harness::psexe::Exe::program("loop: addiu s0, s0, 1\nj loop\nnop", 0x8001_0000);
    exe.sideload(&mut m);
    for _ in 0..50 {
        m.step();
    }
    let interrupted = m.cpu.regs.r[16];
    m.rig.raise(2);
    m.step();
    m.run_to(0x8001_0000, 1000).unwrap();
    assert_eq!(log(&m), "FSD", "priority 0 newest first, then priority 2");
    assert_eq!(m.exceptions, [0]);
    assert!(m.cpu.regs.r[16] >= interrupted, "the program resumes with its registers");
    assert_eq!(m.b(0x0B, &[e]), 1, "the default handler delivered F0000003h,1000h");
}

#[test]
fn the_second_function_runs_with_the_firsts_result_when_it_is_not_zero() {
    let mut m = booted();
    marker(&mut m, 0x8015_0000, '1', 0x41);
    marker(&mut m, 0x8015_0100, '2', 0);
    let e = element(&mut m, 0x8016_0000, 0x8015_0000, 0x8015_0100);
    m.c(0x02, &[0, e]);
    m.rig.i_mask = 1 << 9;
    m.rig.raise(9);
    let exe = diamond_kernel_harness::psexe::Exe::program("loop: j loop\nnop", 0x8001_0000);
    exe.sideload(&mut m);
    for _ in 0..3000 {
        m.step();
        if log(&m).len() == 2 {
            break;
        }
    }
    assert_eq!(log(&m), "12");
    assert_eq!(m.rig.bytes(LOG + 2, 1), [0x41], "a0 = the first function's v0");
    m.rig.i_stat = 0;
    assert_eq!(m.c(0x03, &[0, e]), 0);
    assert_eq!(m.rig.word(m.rig.word(0x8000_0100)), m.rig.word(e), "dequeued: the chain's head is what followed it");
}

#[test]
fn the_hook_runs_after_the_chains_with_v0_1_and_reset_entry_int_gives_the_default() {
    let mut m = booted();
    marker(&mut m, 0x8015_0000, 'C', 0);
    let e = element(&mut m, 0x8016_0000, 0x8015_0000, 0);
    m.c(0x02, &[3, e]);
    // The hook: v0 stored, the IRQ acknowledged, then ReturnFromException.
    m.code(
        0x8015_0200,
        "lui t0, 0x8017\nsw v0, 0x100(t0)\nlui t1, 0x1F80\nsw zero, 0x1070(t1)\naddiu t2, zero, 0xB0\njr t2\naddiu t1, zero, 0x17",
    );
    let hook = 0x8016_0100;
    m.rig.put(hook, 0x8015_0200);
    m.rig.put(hook + 4, 0x801F_C000);
    m.b(0x19, &[hook]);
    m.rig.i_mask = 1 << 9;
    m.rig.raise(9);
    let exe = diamond_kernel_harness::psexe::Exe::program("loop: addiu s0, s0, 1\nj loop\nnop", 0x8001_0000);
    exe.sideload(&mut m);
    for _ in 0..3000 {
        m.step();
    }
    assert_eq!(log(&m), "C");
    assert_eq!(m.rig.word(0x8017_0100), 1, "the hook is entered with v0 = 1");
    assert_eq!(m.exceptions, [0]);
    assert!(m.cpu.regs.r[16] > 0, "and ReturnFromException resumed the program");
    let exit = m.b(0x18, &[]);
    let b0 = m.b(0x57, &[]);
    assert_eq!(m.rig.word(exit), m.rig.word(b0 + 4 * 0x17), "RA: ReturnFromException");
    let sp = m.rig.word(exit + 4);
    assert!(
        (0x8000_1500..0x8000_DF80).contains(&sp) && (sp + 4) % 8 == 0,
        "SP: the exception stack's top less 4, {sp:X}"
    );
    assert!((2..12).all(|i| m.rig.word(exit + 4 * i) == 0), "FP, S0-S7 and GP zero");
}

#[test]
fn a_timer_irq_delivers_its_root_counter_event_and_is_acknowledged_by_default() {
    let mut m = booted();
    let e = m.b(0x08, &[0xF200_0000, 2, 0x2000, 0]);
    m.b(0x0C, &[e]);
    assert_eq!(m.b(0x04, &[0]), 1, "enable_timer_irq(0)");
    assert_eq!(m.rig.i_mask, 1 << 4);
    let exe = diamond_kernel_harness::psexe::Exe::program("loop: addiu s0, s0, 1\nj loop\nnop", 0x8001_0000);
    exe.sideload(&mut m);
    m.rig.raise(4);
    for _ in 0..3000 {
        m.step();
    }
    assert_eq!(m.rig.i_stat, 0, "acknowledged by the timer's handler");
    assert_eq!(m.exceptions, [0], "once");
    assert_eq!(m.b(0x0B, &[e]), 1);
    assert_eq!(m.c(0x0A, &[0, 0]), 1, "ChangeClearRCnt returns the old flag");
    assert_eq!(m.c(0x0A, &[3, 1]), 0, "vblank's is clear at boot");
}

#[test]
fn an_interrupt_on_a_gte_command_does_not_run_it_twice() {
    let mut m = booted();
    m.rig.i_mask = 1 << 4;
    let exe = diamond_kernel_harness::psexe::Exe::program(
        "lui t0, 0x4000\nmtc0 t0, SR\nnop\nnop\ncop2 0x0180001\naddiu s0, zero, 1\ndone: j done\nnop",
        0x8001_0000,
    );
    exe.sideload(&mut m);
    m.run_to(0x8001_0010, 100).unwrap();
    m.cpu.regs.sr |= 0x401;
    m.rig.raise(4);
    m.run_to(0x8001_0018, 10_000).unwrap();
    assert_eq!(m.exceptions, [0]);
    assert_eq!(m.rig.cop2_commands, [0x0180001], "the command ran at the interrupt and was skipped on return");
    assert_eq!(m.cpu.regs.r[16], 1);
}
