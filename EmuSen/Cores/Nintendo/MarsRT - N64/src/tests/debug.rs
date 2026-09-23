//! The debugger's hooks, each driven by a few instructions at the boot entry, as `MarsDebugTests` drives the C# core. See Mars_Native.md §6.5.

use std::sync::Arc;

use crate::cpu::cop0::STATUS;
use crate::cpu::hooks::{LOG_CAPACITY, Range, stop};
use crate::ffi::space;
use crate::machine::Machine;
use crate::memory::mi::interrupt;
use crate::rom::RomImage;
use crate::rsp::Trace;
use crate::tests::support::SyntheticRom;

const ENTRY: u32 = 0xA400_0040;
const VI_BASE: u32 = 0x0440_0000;

/// lui a0,0xA010 / lw t0,0(a0) / addiu t0,t0,1 / sw t0,0(a0) / b the lw / nop
const COUNT_FOREVER: [u8; 24] =
    [0x3C, 0x04, 0xA0, 0x10, 0x8C, 0x88, 0x00, 0x00, 0x25, 0x08, 0x00, 0x01, 0xAC, 0x88, 0x00, 0x00, 0x10, 0x00, 0xFF, 0xFC, 0x00, 0x00, 0x00, 0x00];

/// jal 0xA4000060 / nop / b self / nop / four nops / jr ra / nop
const CALL_ONCE: [u8; 40] = [
    0x0D, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xE0, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00,
];

/// jal 0xA4000060 / nop / b self / nop / four nops / lui t0,0xA400 / ori t0,t0,0x70 / jr t0 / nop / b self / nop
const CALL_THEN_LEAVE: [u8; 56] = [
    0x0D, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3C, 0x08, 0xA4, 0x00, 0x35, 0x08, 0x00, 0x70, 0x01, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0xFF, 0xFF,
    0x00, 0x00, 0x00, 0x00,
];

/// `MarsDebugTests.Load`: the program at the boot entry, the VI programmed so a frame is a few thousand cycles.
fn load(program: &[u8]) -> Machine {
    let mut m = Machine::boot(Arc::new(RomImage::from_image(&SyntheticRom::patched(program).build()).unwrap()), false);
    m.bus.write32(VI_BASE + 0x18, 0x20);
    m.bus.write32(VI_BASE + 0x1C, 0x40);
    m
}

fn pc(m: &Machine) -> u32 {
    m.cpu.pc as u32
}

fn counter(m: &Machine) -> u32 {
    m.bus.rdram.be32(0x10_0000)
}

fn at(address: u32) -> (i32, i32) {
    (address as i32, address as i32)
}

#[test]
fn a_breakpoint_stops_in_front_of_its_instruction_and_a_resume_runs_it() {
    let mut m = load(&COUNT_FOREVER);
    m.cpu.hooks.breakpoints = vec![at(ENTRY + 12)];
    assert_eq!(m.run_frame_debug(false, false), stop::BREAKPOINT);
    assert_eq!(pc(&m), ENTRY + 12);
    assert_eq!(counter(&m), 0);
    assert_eq!(m.total_frames, 0);

    // Resumed with the breakpoint kept: the store runs, and the next lap stops in front of it again.
    assert_eq!(m.run_frame_debug(true, false), stop::BREAKPOINT);
    assert_eq!(pc(&m), ENTRY + 12);
    assert_eq!(counter(&m), 1);

    m.cpu.hooks.breakpoints.clear();
    assert_eq!(m.run_frame_debug(true, false), stop::FRAME);
    assert_eq!(m.total_frames, 1);
    assert!(counter(&m) > 1);
}

#[test]
fn stopping_before_each_instruction_counts_from_the_one_a_halt_stopped_in_front_of() {
    let mut m = load(&COUNT_FOREVER);
    m.cpu.hooks.breakpoints = vec![at(ENTRY)];
    assert_eq!(m.run_frame_debug(false, false), stop::BREAKPOINT);
    m.cpu.hooks.breakpoints.clear();
    m.cpu.hooks.each = true;
    for expected in [ENTRY + 4, ENTRY + 8, ENTRY + 12] {
        assert_eq!(m.run_frame_debug(true, false), stop::EACH);
        assert_eq!(pc(&m), expected);
    }
}

#[test]
fn a_call_is_pushed_as_it_executes_and_a_return_is_popped_once_its_slot_has_run() {
    let mut m = load(&CALL_ONCE);
    m.cpu.hooks.configure(true, false, false, true, false, false);
    let mut seen = Vec::new();
    for _ in 0..12 {
        assert_eq!(m.run_frame_debug(true, false), stop::EACH);
        seen.push((pc(&m), m.cpu.hooks.depth()));
    }
    // The call lands on the `jr ra` itself; the depth is one from the jal's own slot until the return's slot has run.
    let expected = [
        (ENTRY + 4, 1),
        (ENTRY + 0x20, 1),
        (ENTRY + 0x24, 1),
        (ENTRY + 8, 0),
        (ENTRY + 12, 0),
        (ENTRY + 8, 0),
        (ENTRY + 12, 0),
        (ENTRY + 8, 0),
        (ENTRY + 12, 0),
        (ENTRY + 8, 0),
        (ENTRY + 12, 0),
        (ENTRY + 8, 0),
    ];
    assert_eq!(seen, expected);
    assert_eq!(m.cpu.hooks.stack, Vec::new());
    assert_eq!(m.cpu.hooks.calls_log.len(), 2);
    assert_eq!((m.cpu.hooks.calls_log[0].kind, m.cpu.hooks.calls_log[0].source, m.cpu.hooks.calls_log[0].target), (1, ENTRY, ENTRY + 0x20));
    assert_eq!(m.cpu.hooks.calls_log[1].kind, 0);
}

#[test]
fn a_jump_through_another_register_does_not_return() {
    let mut m = load(&CALL_THEN_LEAVE);
    m.cpu.hooks.configure(true, false, false, false, false, false);
    assert_eq!(m.run_frame_debug(false, false), stop::FRAME);
    assert_eq!(m.cpu.hooks.depth(), 1);
    assert_eq!(m.cpu.hooks.unmatched_returns, 0);
}

#[test]
fn stepping_to_a_depth_stops_after_the_call_in_the_caller_and_out_of_it_at_the_same_place() {
    let mut m = load(&CALL_ONCE);
    m.cpu.hooks.configure(true, false, false, false, false, false);
    m.cpu.hooks.breakpoints = vec![at(ENTRY)];
    assert_eq!(m.run_frame_debug(false, false), stop::BREAKPOINT);
    m.cpu.hooks.breakpoints.clear();
    m.cpu.hooks.depth_target = 0;
    assert_eq!(m.run_frame_debug(true, false), stop::DEPTH);
    assert_eq!((pc(&m), m.cpu.hooks.depth()), (ENTRY + 8, 0));

    let mut m = load(&CALL_ONCE);
    m.cpu.hooks.configure(true, false, false, false, false, false);
    m.cpu.hooks.breakpoints = vec![at(ENTRY + 0x20)];
    assert_eq!(m.run_frame_debug(false, false), stop::BREAKPOINT);
    assert_eq!(m.cpu.hooks.stack, vec![(ENTRY, ENTRY + 0x20)]);
    m.cpu.hooks.breakpoints.clear();
    m.cpu.hooks.depth_target = 0;
    assert_eq!(m.run_frame_debug(true, false), stop::DEPTH);
    assert_eq!(pc(&m), ENTRY + 8);
}

#[test]
fn a_depth_guard_stops_as_soon_as_the_stack_is_deeper_than_it() {
    let mut m = load(&CALL_ONCE);
    m.cpu.hooks.configure(true, false, false, false, false, false);
    m.cpu.hooks.depth_guard = 0;
    assert_eq!(m.run_frame_debug(false, false), stop::DEPTH);
    assert_eq!((pc(&m), m.cpu.hooks.depth()), (ENTRY + 4, 1));
}

#[test]
fn a_data_breakpoint_stops_after_the_store_that_wrote_it_and_a_watch_logs_every_byte() {
    let mut m = load(&COUNT_FOREVER);
    m.cpu.hooks.configure(false, true, false, false, false, false);
    m.cpu.hooks.watch_ranges = vec![Range { space: space::RDRAM, start: 0x10_0000, end: 0x10_0003 }];
    m.cpu.hooks.break_ranges = vec![Range { space: space::RDRAM, start: 0x10_0003, end: 0x10_0003 }];
    assert_eq!(m.run_frame_debug(false, false), stop::DATA);
    assert_eq!(pc(&m), ENTRY + 16);
    assert_eq!(counter(&m), 1);
    let log = &m.cpu.hooks.writes_log;
    assert_eq!(log.len(), 4);
    assert_eq!(log.iter().map(|w| (w.space, w.address, w.value, w.pc)).collect::<Vec<_>>(), vec![
        (space::RDRAM, 0x10_0000, 0, ENTRY + 12),
        (space::RDRAM, 0x10_0001, 0, ENTRY + 12),
        (space::RDRAM, 0x10_0002, 0, ENTRY + 12),
        (space::RDRAM, 0x10_0003, 1, ENTRY + 12),
    ]);
}

#[test]
fn a_store_is_reported_only_while_something_listens_and_only_in_a_range_that_covers_it() {
    let mut m = load(&COUNT_FOREVER);
    m.cpu.hooks.watch_ranges = vec![Range { space: space::RDRAM, start: 0x10_0000, end: 0x10_0003 }];
    assert_eq!(m.run_frame_debug(false, false), stop::FRAME);
    assert!(m.cpu.hooks.writes_log.is_empty());
    m.cpu.hooks.configure(false, true, false, false, false, false);
    m.cpu.hooks.watch_ranges = vec![Range { space: space::DMEM, start: 0x10_0000, end: 0x10_0003 }];
    assert_eq!(m.run_frame_debug(false, false), stop::FRAME);
    assert!(m.cpu.hooks.writes_log.is_empty());
}

#[test]
fn a_store_into_a_latching_window_reports_the_whole_word_it_left() {
    // lui a0,0xA400 / ori a0,a0,0x0800 / addiu t0,zero,0x55 / sb t0,1(a0) / b self / nop: a byte into DMEM 0x800.
    let program = [0x3C04A400u32, 0x34840800, 0x24080055, 0xA0880001, 0x1000FFFF, 0];
    let bytes: Vec<u8> = program.iter().flat_map(|w| w.to_be_bytes()).collect();
    let mut m = load(&bytes);
    m.cpu.hooks.configure(false, true, false, false, false, false);
    m.cpu.hooks.watch_ranges = vec![Range { space: space::DMEM, start: 0x800, end: 0x803 }];
    assert_eq!(m.run_frame_debug(false, false), stop::FRAME);
    let log = &m.cpu.hooks.writes_log;
    assert_eq!(log.iter().map(|w| (w.space, w.address, w.value)).collect::<Vec<_>>(), vec![
        (space::DMEM, 0x800, 0),
        (space::DMEM, 0x801, 0x55),
        (space::DMEM, 0x802, 0),
        (space::DMEM, 0x803, 0),
    ]);
}

#[test]
fn an_interrupt_stops_at_the_handler() {
    let mut m = load(&COUNT_FOREVER);
    m.cpu.cop0[STATUS] = 0x3400_0401;
    m.cpu.cop0_written(&m.bus);
    m.bus.mi.mask = interrupt::VIDEO_INTERFACE;
    m.bus.mi.raise(interrupt::VIDEO_INTERFACE);
    m.cpu.hooks.configure(false, false, true, false, false, false);
    assert_eq!(m.run_frame_debug(false, false), stop::INTERRUPT);
    assert_eq!(pc(&m), 0x8000_0180);
    assert!(!m.cpu.hooks.interrupt_taken);
}

#[test]
fn coverage_records_what_the_processor_ran_only_while_armed_and_the_profiler_counts_every_instruction() {
    let mut m = load(&COUNT_FOREVER);
    m.run_frame();
    assert!(m.cpu.hooks.coverage.is_none());

    m.cpu.hooks.configure(false, false, false, false, true, true);
    let before = m.cpu.instructions;
    assert_eq!(m.run_frame_debug(false, false), stop::FRAME);
    let ran = m.cpu.instructions - before;
    let bits = m.cpu.hooks.coverage.as_deref().unwrap();
    let executed = |address: u32| bits[(address & 0xFF_FFFF) as usize >> 3] & (1 << (address & 7)) != 0;
    assert!(executed(ENTRY + 4));
    assert!(executed(ENTRY + 20));
    assert!(!executed(ENTRY + 24));
    assert_eq!(m.cpu.hooks.covered, ran);
    assert_eq!(m.cpu.hooks.profile.values().sum::<i64>(), ran);
    assert_eq!(m.cpu.hooks.profile[&0], ran);

    m.cpu.hooks.configure(false, false, false, false, false, false);
    let before = m.cpu.instructions;
    assert_eq!(m.run_frame_debug(false, false), stop::FRAME);
    assert!(m.cpu.instructions > before);
    assert!(m.cpu.hooks.coverage.is_none());
    assert_eq!(m.cpu.hooks.profile[&0], ran);
}

#[test]
fn the_profiler_charges_an_instruction_to_the_routine_it_ran_in() {
    let mut m = load(&CALL_ONCE);
    m.cpu.hooks.configure(true, false, false, false, false, true);
    m.cpu.hooks.breakpoints = vec![at(ENTRY + 8)];
    assert_eq!(m.run_frame_debug(false, false), stop::BREAKPOINT);
    // The jal outside any call, charged before it pushed; its slot, the jr and the jr's slot inside the call, the pop coming after the slot's step.
    assert_eq!(m.cpu.hooks.profile[&0], 1);
    assert_eq!(m.cpu.hooks.profile[&(ENTRY + 0x20)], 3);
}

#[test]
fn the_rsp_records_its_own_coverage_and_the_processor_does_not() {
    let mut m = load(&COUNT_FOREVER);
    *m.bus.sp.trace = Some(Box::new(Trace::new()));
    m.cpu.hooks.configure(false, false, false, false, true, false);
    m.bus.sp.processor.start(0x010);
    m.bus.rsp_step_one();
    m.bus.rsp_step_one();
    let trace = m.bus.sp.trace.as_ref().unwrap();
    let executed = |address: u32| trace.bits[address as usize >> 3] & (1 << (address & 7)) != 0;
    assert!(executed(0x010));
    assert!(executed(0x014));
    assert!(!executed(0x018));
    assert_eq!(trace.recorded, 2);
    assert!(m.cpu.hooks.coverage.as_deref().unwrap().iter().all(|&b| b == 0));
}

/// The program in RDRAM rather than at the boot entry, since the idle loop is passed whole only there (`try_idle`).
fn load_in_rdram(program: &[u8]) -> Machine {
    let mut m = load(&[]);
    for (i, word) in program.as_chunks::<4>().0.iter().enumerate() {
        m.bus.rdram.put_be32(0x1000 + 4 * i as u32, u32::from_be_bytes(*word));
    }
    m.cpu.pc = 0xFFFF_FFFF_8000_1000;
    m.cpu.next_pc = m.cpu.pc + 4;
    m.cpu.cop0_written(&m.bus);
    m
}

#[test]
fn the_rsp_is_recorded_through_the_frame_loop_and_the_idle_loop_steps_it_a_cycle_at_a_time() {
    // The RSP left running over IMEM's no-operations while the processor idles at `b self`; every address it passes is recorded,
    // and the plain machine's whole run of the processor through the idle loop leaves what the traced machine's steps leave.
    const IDLE: [u8; 8] = [0x10, 0x00, 0xFF, 0xFF, 0, 0, 0, 0];
    let mut plain = load_in_rdram(&IDLE);
    plain.bus.sp.processor.start(0x020);
    plain.run_frame();
    let mut traced = load_in_rdram(&IDLE);
    *traced.bus.sp.trace = Some(Box::new(Trace::new()));
    traced.bus.sp.processor.start(0x020);
    let start = traced.bus.cycles;
    assert_eq!(traced.run_frame_debug(false, false), stop::FRAME);
    assert_eq!(plain.save_state_vec(false).unwrap(), traced.save_state_vec(false).unwrap());
    assert!(plain.cpu.run.rsp_steps > 0 && traced.cpu.run.rsp_steps == 0, "the plain frame's idle loop ran the processor whole; the observed frame stepped it");
    assert!(plain.cpu.run.idle_turns_passed == 0, "no turn is passed while the processor runs");
    let trace = traced.bus.sp.trace.as_ref().unwrap();
    assert_eq!(trace.recorded, traced.bus.cycles - start);
    assert!(trace.bits[4..].iter().all(|&b| b == 0x11), "every word of IMEM past the boot head was passed");
}

#[test]
fn a_full_log_stops_the_frame_so_it_can_be_drained_and_loses_nothing() {
    let mut m = Machine::boot(Arc::new(RomImage::from_image(&SyntheticRom::patched(&COUNT_FOREVER).build()).unwrap()), false);
    m.cpu.hooks.configure(false, true, false, false, false, false);
    m.cpu.hooks.watch_ranges = vec![Range { space: space::RDRAM, start: 0x10_0000, end: 0x10_0003 }];
    let mut logged = 0;
    let mut stops = 0;
    let mut resuming = false;
    loop {
        let why = m.run_frame_debug(resuming, resuming);
        logged += m.cpu.hooks.writes_log.len();
        m.cpu.hooks.writes_log.clear();
        if why == stop::FRAME {
            break;
        }
        assert_eq!(why, stop::RING);
        assert!(logged >= LOG_CAPACITY);
        stops += 1;
        resuming = true;
    }
    assert!(stops >= 1);
    assert_eq!(logged as u32, counter(&m) * 4);
}

/// Every table armed and nothing firing: a breakpoint no instruction reaches, a watch that logs every lap, coverage, the profiler.
fn arm_everything(m: &mut Machine) {
    m.cpu.hooks.configure(true, true, true, false, true, true);
    *m.bus.sp.trace = Some(Box::new(Trace::new()));
    m.cpu.hooks.breakpoints = vec![at(0x8000_1000)];
    m.cpu.hooks.watch_ranges = vec![Range { space: space::RDRAM, start: 0x10_0000, end: 0x10_0003 }];
    m.cpu.hooks.depth_guard = 100_000;
}

#[test]
fn an_observed_frame_computes_what_a_plain_frame_computes_with_and_without_the_blocks() {
    for blocks in [false, true] {
        let mut plain = load(&COUNT_FOREVER);
        plain.set_recompiler(blocks);
        let mut observed = load(&COUNT_FOREVER);
        arm_everything(&mut observed);
        for frame in 1..=20 {
            plain.run_frame();
            assert_eq!(observed.run_frame_debug(false, false), stop::FRAME, "frame {frame}");
            observed.cpu.hooks.writes_log.clear();
            assert_eq!(plain.save_state_vec(false).unwrap(), observed.save_state_vec(false).unwrap(), "blocks {blocks}: the states part at frame {frame}");
        }
        assert_eq!(observed.cpu.hooks.covered, observed.cpu.instructions);
    }
}

#[test]
fn a_frame_halted_and_resumed_is_the_frame_run_through() {
    let mut plain = load(&COUNT_FOREVER);
    plain.set_recompiler(true);
    let mut halted = load(&COUNT_FOREVER);
    halted.cpu.hooks.configure(true, false, false, false, false, false);
    halted.cpu.hooks.breakpoints = vec![at(ENTRY + 12)];
    for frame in 1..=5 {
        let mut resuming = false;
        let mut halts = 0;
        loop {
            let why = halted.run_frame_debug(resuming, false);
            if why == stop::FRAME {
                break;
            }
            halts += 1;
            resuming = true;
            // The plain machine brought to the same instruction, through its blocks: the two states are one.
            let target = (halted.cpu.instructions + halted.cpu.run.exceptions) as u64;
            plain.run_steps(target - (plain.cpu.instructions + plain.cpu.run.exceptions) as u64);
            if halts % 50 == 1 {
                assert_eq!(plain.save_state_vec(false).unwrap(), halted.save_state_vec(false).unwrap(), "frame {frame}, halt {halts}");
            }
        }
        assert!(halts > 10, "frame {frame} halted {halts} times");
        plain.run_frame();
        assert_eq!(plain.total_frames, halted.total_frames);
        assert_eq!(plain.save_state_vec(false).unwrap(), halted.save_state_vec(false).unwrap(), "frame {frame}");
    }
}

#[test]
fn a_state_loaded_keeps_the_tables_and_the_stack() {
    let mut m = load(&CALL_ONCE);
    m.cpu.hooks.configure(true, false, false, false, false, false);
    m.cpu.hooks.breakpoints = vec![at(ENTRY + 0x20)];
    *m.bus.sp.trace = Some(Box::new(Trace::new()));
    assert_eq!(m.run_frame_debug(false, false), stop::BREAKPOINT);
    let state = m.save_state_vec(false).unwrap();
    m.restore_state(&state).unwrap();
    assert_eq!(m.cpu.hooks.stack, vec![(ENTRY, ENTRY + 0x20)]);
    assert_eq!(m.cpu.hooks.breakpoints, vec![at(ENTRY + 0x20)]);
    assert!(m.bus.sp.trace.is_some());
    m.cpu.hooks.depth_target = 0;
    assert_eq!(m.run_frame_debug(true, false), stop::DEPTH);
    assert_eq!(pc(&m), ENTRY + 8);
}
