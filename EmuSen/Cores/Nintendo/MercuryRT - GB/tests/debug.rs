//! The debugger's hooks driven through the machine alone: what an observed frame stops for, what it records, and that a frame
//! observed, halted or refused and resumed is the frame run plain. See Mercury_Native.md §8.5.

use mercuryrt::debug::{Call, Range, Write, stop};
use mercuryrt::machine::{Machine, RUN_CONTINUING, RUN_UNCHECKED};
use mercuryrt::memory::cartridge::{HEADER_CHECKSUM_ADDRESS, header_checksum};

const ENTRY: usize = 0x150;

/// A 32K image with no board, the program at the entry point and each `(address, bytes)` placed where it says.
fn rom(program: &[u8], more: &[(usize, &[u8])]) -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];
    rom[0x100..0x104].copy_from_slice(&[0x00, 0xC3, 0x50, 0x01]);
    rom[ENTRY..ENTRY + program.len()].copy_from_slice(program);
    for (at, bytes) in more {
        rom[*at..*at + bytes.len()].copy_from_slice(bytes);
    }
    rom[HEADER_CHECKSUM_ADDRESS] = header_checksum(&rom);
    rom
}

/// ld hl,$C000 / inc (hl) / jr the inc.
const COUNT_FOREVER: &[u8] = &[0x21, 0x00, 0xC0, 0x34, 0x18, 0xFD];

/// call $0160 / jr self; at $0160 four nops and ret.
fn call_once() -> Vec<u8> {
    rom(&[0xCD, 0x60, 0x01, 0x18, 0xFE], &[(0x160, &[0x00, 0x00, 0x00, 0x00, 0xC9])])
}

/// The vblank interrupt into a handler that counts in WRAM and HRAM, calls a routine and returns with reti; the main loop halts.
fn interrupts() -> Vec<u8> {
    rom(
        &[0x31, 0xFE, 0xFF, 0x3E, 0x01, 0xE0, 0xFF, 0xFB, 0x76, 0x00, 0x18, 0xFC],
        &[(0x40, &[0xC3, 0x00, 0x02]), (0x200, &[0xF5, 0xE5, 0x21, 0x00, 0xC0, 0x34, 0xCD, 0x20, 0x02, 0xE1, 0xF1, 0xD9]), (0x220, &[0xF0, 0x90, 0x3C, 0xE0, 0x90, 0xC9])],
    )
}

fn load(image: Vec<u8>) -> Machine {
    Machine::load_rom(image, None).expect("a board with no mapper")
}

fn state(m: &Machine) -> Vec<u8> {
    let mut out = vec![0u8; m.state_size()];
    m.save_state(&mut out).expect("a state");
    out
}

/// Runs an observed frame to its end, resuming every stop as the host does when the registry says no; returns the stops.
fn run_through(m: &mut Machine, mut flags: u32) -> Vec<(u32, u16)> {
    let mut stops = Vec::new();
    loop {
        let why = m.run_frame_debug(flags).expect("a legal program");
        if why == stop::FRAME {
            return stops;
        }
        stops.push((why, m.cpu.pc));
        flags = RUN_UNCHECKED | RUN_CONTINUING;
    }
}

#[test]
fn a_breakpoint_stops_in_front_of_its_instruction_and_a_resume_runs_it() {
    let mut m = load(rom(COUNT_FOREVER, &[]));
    m.hooks.breakpoints = vec![(0x153, 0x153)];
    assert_eq!(m.run_frame_debug(0), Ok(stop::BREAKPOINT));
    assert_eq!((m.cpu.pc, m.bus.wram[0]), (0x153, 0));
    m.hooks.breakpoints.clear();
    assert_eq!(m.run_frame_debug(RUN_UNCHECKED), Ok(stop::FRAME));
    assert!(m.bus.wram[0] > 0);
    assert_eq!(m.total_frames, 1);
}

#[test]
fn stopping_before_each_instruction_stops_at_every_step_and_the_first_runs_unchecked() {
    let mut m = load(rom(COUNT_FOREVER, &[]));
    m.hooks.each = true;
    assert_eq!(m.run_frame_debug(0), Ok(stop::EACH));
    assert_eq!(m.cpu.pc, 0x100);
    let mut pcs = Vec::new();
    for _ in 0..6 {
        assert_eq!(m.run_frame_debug(RUN_UNCHECKED | RUN_CONTINUING), Ok(stop::EACH));
        pcs.push(m.cpu.pc);
    }
    assert_eq!(pcs, [0x101, 0x150, 0x153, 0x154, 0x153, 0x154]);
    assert_eq!(m.bus.wram[0], 2);
}

#[test]
fn coverage_records_what_the_processor_ran_only_while_armed() {
    let mut m = load(rom(COUNT_FOREVER, &[]));
    m.run_frame_debug(0).unwrap();
    assert!(m.hooks.coverage.is_none());
    m.hooks.configure(false, false, false, false, true, false);
    m.run_frame_debug(0).unwrap();
    let bits = m.hooks.coverage.as_deref().unwrap();
    let ran = |a: usize| bits[a >> 3] & (1 << (a & 7)) != 0;
    assert!(ran(0x153) && ran(0x154));
    assert!(!ran(0x150) && !ran(0x151) && !ran(0x156));
    assert!(m.hooks.covered > 1000);
    m.hooks.configure(false, false, false, false, false, false);
    m.run_frame_debug(0).unwrap();
    assert!(m.hooks.coverage.is_none());
}

#[test]
fn a_data_breakpoint_stops_after_the_store_that_wrote_it_and_a_watch_logs_every_byte() {
    let mut m = load(rom(COUNT_FOREVER, &[]));
    m.hooks.configure(false, true, false, false, false, false);
    m.hooks.watch_ranges = vec![Range { space: 3, start: 0, end: 0 }];
    m.hooks.break_ranges = vec![Range { space: 3, start: 0, end: 0 }];
    assert_eq!(m.run_frame_debug(0), Ok(stop::DATA));
    assert_eq!((m.cpu.pc, m.bus.wram[0]), (0x154, 1));
    assert_eq!(m.hooks.writes_log, vec![Write { space: 3, address: 0, value: 1, pc: 0x153 }]);
    m.hooks.break_ranges.clear();
    m.hooks.writes_log.clear();
    m.run_frame_debug(RUN_UNCHECKED | RUN_CONTINUING).unwrap();
    let n = m.bus.wram[0];
    assert_eq!(m.hooks.writes_log.len() as u8, n - 1);
    assert_eq!(m.hooks.writes_log.last(), Some(&Write { space: 3, address: 0, value: n as u32, pc: 0x153 }));
}

#[test]
fn a_store_is_reported_only_while_something_listens_and_only_in_a_range_that_covers_it() {
    let mut m = load(rom(COUNT_FOREVER, &[]));
    m.hooks.watch_ranges = vec![Range { space: 3, start: 0, end: 0 }];
    m.run_frame_debug(0).unwrap();
    assert!(m.hooks.writes_log.is_empty());
    m.hooks.configure(false, true, false, false, false, false);
    m.hooks.watch_ranges = vec![Range { space: 3, start: 1, end: 0x1FFF }, Range { space: 5, start: 0, end: 0x7E }];
    m.run_frame_debug(0).unwrap();
    assert!(m.hooks.writes_log.is_empty());
}

#[test]
fn a_host_write_to_the_cpu_bus_is_reported_as_the_bus_reports_it() {
    let mut m = load(rom(COUNT_FOREVER, &[]));
    m.hooks.configure(false, true, false, false, false, false);
    m.hooks.watch_ranges = vec![Range { space: 3, start: 0x10, end: 0x10 }, Range { space: 2, start: 0, end: 0x1FFF }];
    m.run_frame_debug(0).unwrap();
    m.hooks.writes_log.clear();
    m.write_space(6, 0xC010, 0x5A);
    m.write_space(3, 0x10, 0x5B);
    m.write_space(6, 0xA005, 0x77);
    let pc = m.cpu.last_instruction_pc as u32;
    assert_eq!(m.hooks.writes_log, vec![Write { space: 3, address: 0x10, value: 0x5A, pc }, Write { space: 2, address: 5, value: 0x77, pc }]);
}

#[test]
fn a_call_is_logged_and_stepping_to_a_depth_stops_back_in_the_caller() {
    let mut m = load(call_once());
    m.hooks.configure(true, false, false, false, false, false);
    m.hooks.breakpoints = vec![(0x150, 0x150)];
    assert_eq!(m.run_frame_debug(0), Ok(stop::BREAKPOINT));
    m.hooks.breakpoints.clear();
    m.hooks.depth_target = 0;
    assert_eq!(m.run_frame_debug(RUN_UNCHECKED), Ok(stop::DEPTH));
    assert_eq!(m.cpu.pc, 0x153);
    assert_eq!(m.hooks.calls_log, vec![Call { kind: 1, source: 0x150, target: 0x160 }, Call::default()]);
    assert_eq!(m.hooks.depth(), 0);

    let mut m = load(call_once());
    m.hooks.configure(true, false, false, false, false, false);
    m.hooks.breakpoints = vec![(0x164, 0x164)];
    assert_eq!(m.run_frame_debug(0), Ok(stop::BREAKPOINT));
    assert_eq!(m.hooks.depth(), 1);
    m.hooks.breakpoints.clear();
    m.hooks.depth_target = 0;
    assert_eq!(m.run_frame_debug(RUN_UNCHECKED), Ok(stop::DEPTH));
    assert_eq!(m.cpu.pc, 0x153);
}

#[test]
fn an_interrupt_stops_before_its_vector_and_is_pushed_as_one() {
    let mut m = load(interrupts());
    m.hooks.configure(true, false, true, false, false, false);
    assert_eq!(m.run_frame_debug(0), Ok(stop::INTERRUPT));
    assert_eq!(m.cpu.pc, 0x40);
    assert_eq!(m.hooks.calls_log, vec![Call { kind: 2, source: 0x159, target: 0x40 }]);
    assert_eq!(m.hooks.depth(), 1);
}

#[test]
fn the_profiler_charges_the_steps_and_the_calls_they_ran_in() {
    let mut m = load(interrupts());
    m.hooks.configure(true, false, false, false, true, true);
    for _ in 0..3 {
        run_through(&mut m, 0);
    }
    m.hooks.flush();
    let charged: i64 = m.hooks.profile.values().sum();
    assert_eq!(charged, m.hooks.covered);
    let calls = m.bus.high_ram[0x10] as i64;
    assert!(calls > 0 && m.hooks.depth() == 0);
    assert_eq!((m.hooks.profile[&0x220], m.hooks.profile[&0x40]), (4 * calls, 9 * calls));
}

#[test]
fn a_full_log_stops_the_frame_and_nothing_is_lost_once_it_is_drained() {
    let mut m = load(rom(COUNT_FOREVER, &[]));
    m.hooks.capacity = 16;
    m.hooks.configure(false, true, false, false, false, false);
    m.hooks.watch_ranges = vec![Range { space: 3, start: 0, end: 0 }];
    let mut logged = 0;
    let mut flags = 0;
    loop {
        let why = m.run_frame_debug(flags).unwrap();
        logged += m.hooks.writes_log.len();
        m.hooks.writes_log.clear();
        if why == stop::FRAME {
            break;
        }
        assert_eq!(why, stop::RING);
        flags = RUN_UNCHECKED | RUN_CONTINUING;
    }
    let mut plain = load(rom(COUNT_FOREVER, &[]));
    plain.run_frame().unwrap();
    assert_eq!(state(&m), state(&plain));
    let increments = plain.cpu.cycles / 24;
    assert!(logged as i64 >= increments - 1 && logged as i64 <= increments + 1, "{logged} stores logged, about {increments} made");
}

#[test]
fn an_illegal_opcode_in_an_observed_frame_is_the_plain_frames_error() {
    let mut m = load(rom(&[0x00, 0xD3], &[]));
    let mut plain = m.clone();
    m.hooks.configure(true, true, true, false, true, true);
    assert_eq!(m.run_frame_debug(0).unwrap_err(), plain.run_frame().unwrap_err());
    assert_eq!(state(&m), state(&plain));
}

/// KEY1 and STOP into double speed mid-frame, then a count: the frame the switch is in ends on the budget it began with, 70,224
/// CPU cycles, before the slowed PPU completes, so a stop refused after the switch must not begin the budget again.
#[test]
fn a_frame_continued_past_a_refused_stop_keeps_the_budget_it_began_with() {
    let mut image = rom(&[0x3E, 0x01, 0xE0, 0x4D, 0x10, 0x00, 0x21, 0x00, 0xC0, 0x34, 0x18, 0xFD], &[]);
    image[0x143] = 0x80;
    image[HEADER_CHECKSUM_ADDRESS] = header_checksum(&image);
    let mut plain = load(image.clone());
    let mut observed = load(image);
    observed.hooks.breakpoints = vec![(0x159, 0x159)];
    for f in 0..4 {
        plain.run_frame().unwrap();
        let stops = run_through(&mut observed, 0);
        assert!(!stops.is_empty());
        assert_eq!(state(&observed), state(&plain), "frame {f}");
    }
    assert!(plain.bus.double_speed);
}

/// Every table armed and every stop resumed, against the same machine run plain: state, picture and sound after every frame.
fn assert_armed_is_plain(image: Vec<u8>, frames: usize) {
    let mut plain = load(image.clone());
    let mut armed = load(image);
    armed.hooks.configure(true, true, true, false, true, true);
    armed.hooks.breakpoints = vec![(0xFEFF, 0xFEFF), (0x40, 0x40)];
    armed.hooks.watch_ranges = vec![Range { space: 3, start: 0, end: 0x0FFF }, Range { space: 5, start: 0, end: 0x7E }];
    armed.hooks.break_ranges = vec![Range { space: 3, start: 0, end: 0xFF }];
    armed.hooks.depth_guard = 400;
    let (mut a, mut b) = (vec![0i16; 1 << 14], vec![0i16; 1 << 14]);
    for f in 0..frames {
        plain.bus.joypad.start = f % 90 < 5;
        armed.bus.joypad.start = f % 90 < 5;
        plain.run_frame().unwrap();
        run_through(&mut armed, 0);
        armed.hooks.writes_log.clear();
        armed.hooks.calls_log.clear();
        assert_eq!(state(&armed), state(&plain), "frame {f}: states differ");
        assert!(armed.bus.ppu.frame_rgba == plain.bus.ppu.frame_rgba, "frame {f}: pictures differ");
        let (n, m) = (plain.bus.apu.drain(&mut a, usize::MAX), armed.bus.apu.drain(&mut b, usize::MAX));
        assert!(a[..n] == b[..m], "frame {f}: sound differs");
    }
}

#[test]
fn an_observed_machine_with_every_table_armed_is_the_machine_plain() {
    assert_armed_is_plain(interrupts(), 120);
    assert_armed_is_plain(call_once(), 30);
    assert_armed_is_plain(rom(COUNT_FOREVER, &[]), 30);
}

/// The games in `EMUSEN_MERCURYRT_ROMS`, scratch copies; absent, nothing runs.
#[test]
fn games_with_every_table_armed_are_the_games_plain() {
    let Ok(folder) = std::env::var("EMUSEN_MERCURYRT_ROMS") else { return };
    let mut paths: Vec<_> = std::fs::read_dir(folder).unwrap().map(|e| e.unwrap().path()).filter(|p| p.extension().is_some_and(|e| e == "gb" || e == "gbc")).collect();
    paths.sort();
    for path in paths {
        assert_armed_is_plain(std::fs::read(&path).unwrap(), 600);
        println!("{}: 600 frames armed, identical", path.display());
    }
}

/// A breakpoint at the vblank vector on every third frame, halted at and resumed as a host resumes; the unstopped run compared at
/// every frame's end.
#[test]
fn a_game_halted_at_breakpoints_and_resumed_is_the_game_run_through() {
    let mut plain = load(interrupts());
    let mut halted = load(interrupts());
    let mut halts = 0;
    for f in 0..300 {
        plain.run_frame().unwrap();
        halted.hooks.breakpoints = if f % 3 == 0 { vec![(0x40, 0x40)] } else { Vec::new() };
        let mut flags = 0;
        loop {
            let why = halted.run_frame_debug(flags).unwrap();
            if why == stop::FRAME {
                break;
            }
            halts += 1;
            flags = RUN_UNCHECKED;
        }
        assert_eq!(state(&halted), state(&plain), "frame {f}");
    }
    assert_eq!(halts, 100);
}
