//! Stage S31's gate on the project's programs (Sapphire_Native.md §41.6): every table of the debugger armed, and every
//! processor stopped in front of each of its steps and resumed, give the plain run's picture, sound and state after
//! every frame. The programs reach DMA chains, VU microprograms, VIF, the GIF's paths, GS drawing, the PCRTC, the
//! SPU2's voices on the IOP and the IPU.

mod s6;
mod s9;
mod s11;
mod s12;
mod s13;
mod s14;
mod s15;
mod s16;
mod s17;
mod s20;
mod s21;
mod s22;
mod s25;
mod s26;
mod s27;
mod s33;

use emusen_native::core::Core;
use emusen_native::debug::{flag, range, run, stop};
use emusen_native::ffi::StateMachine;
use sapphire::board::System;
use sapphire::debugger::{EE, IOP, VU0, VU1};
use sapphire::machine::{Machine, Market, RAM_ID};
use sapphire::media::Media;

/// A console running a project's program on the board as `make` builds it, the BIOS's place taken by the program.
fn machine(make: &dyn Fn() -> System) -> Machine {
    let mut m = Machine::new(Media::read(&sapphire::media::program(&[0], s6::ORIGIN)).unwrap(), Market::Americas, [Some("ps2.dualshock2"), None]);
    m.sys = Box::new(make());
    m.booted = true;
    m
}

fn ee(p: s6::Program) -> (String, Box<dyn Fn() -> System>) {
    let name = p.name.to_string();
    let words = emusen_mips::assemble(&p.finish().0, s6::ORIGIN, &emusen_mips::Dialect::R5900).unwrap();
    (
        name,
        Box::new(move || {
            let mut sys = System::new(s6::ORIGIN);
            sys.board.map.load(s6::ORIGIN, &words);
            sys
        }),
    )
}

fn iop(p: s21::Program) -> (String, Box<dyn Fn() -> System>) {
    let name = p.name.to_string();
    let words = s21::assemble(&p.finish().0);
    (
        name,
        Box::new(move || {
            let mut sys = System::new(0);
            sys.board.iop.bus.load(s21::ORIGIN, &words);
            sys.board.iop.bus.load(0x80, &s21::vector());
            sys.board.iop.enter(s21::K1 | s21::ORIGIN);
            sys
        }),
    )
}

fn state(m: &Machine) -> Vec<u8> {
    let mut v = vec![0; m.state_size()];
    let n = m.save_state(&mut v).unwrap();
    v.truncate(n);
    v
}

/// The fields of the state's listing whose bytes differ, for a failure's words.
fn differing(m: &Machine, a: &[u8], b: &[u8]) -> Vec<String> {
    m.layout()
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split(' ').collect();
            let (at, len) = (f[0].parse::<usize>().ok()?, f[1].parse::<usize>().ok()?);
            (a.get(at..at + len) != b.get(at..at + len)).then(|| if len <= 64 { format!("{} {:?} {:?}", f[3], &a[at..at + len], &b[at..at + len]) } else { f[3].to_string() })
        })
        .collect()
}

/// The open frame run to its end as a host runs it, each stop resumed unasked and the logs drained; the stops.
fn run_out(m: &mut Machine) -> u64 {
    let (mut stops, mut flags) = (0, 0);
    loop {
        let why = Core::debug_run_frame(m, flags, &mut 0).unwrap();
        let h = &mut m.debug.hooks;
        h.calls_log.clear();
        h.writes_log.clear();
        h.reads_log.clear();
        if why == stop::FRAME {
            return stops;
        }
        stops += 1;
        flags = run::UNCHECKED;
    }
}

/// Every table armed: calls, stores, loads, interrupts, the profile, every processor's coverage, a breakpoint on each
/// processor at an address none runs from, watches and data breakpoints in RAM and the scratchpad.
fn arm_everything(m: &mut Machine) {
    let coverage = (0..4).fold(0, |f, n| f | 1 << (flag::COVERAGE + n));
    m.debug.hooks.configure(flag::CALLS | flag::WRITES | flag::READS | flag::INTERRUPTS | flag::PROFILING | coverage, i32::MIN, -1);
    for p in [EE, IOP] {
        assert!(m.debug_breakpoints(p, &[-2, -2]));
    }
    for p in [VU0, VU1] {
        assert!(m.debug_breakpoints(p, &[4, 4]));
    }
    m.debug.hooks.set_ranges(range::WATCH, &[RAM_ID, 0, 0x1FF_FFFF, 2, 0, 0x3FFF]);
    m.debug.hooks.set_ranges(range::BREAK, &[RAM_ID, 0x1FF_FFF0, 0x1FF_FFFF]);
    m.debug.hooks.set_ranges(range::READ_WATCH, &[RAM_ID, 0, 0x1FF_FFFF, 2, 0, 0x3FFF]);
    m.debug.hooks.set_ranges(range::READ_BREAK, &[2, 0x3FF0, 0x3FFF]);
}

/// Every processor stopped in front of each of its steps: the EE by `EACH`, the others by a breakpoint over every
/// address.
fn step_everything(m: &mut Machine) {
    m.debug.hooks.configure(flag::EACH | flag::CALLS, i32::MIN, -1);
    for p in [IOP, VU0, VU1] {
        assert!(m.debug_breakpoints(p, &[i32::MIN, i32::MAX]));
    }
}

/// `frames` frames of the plain machine and of each debugged one, their picture, sound and state equal after each.
fn the_debugger_changes_nothing(name: &str, make: &dyn Fn() -> System, frames: usize, stepped: bool, between: fn(&mut Machine)) {
    let mut plain = machine(make);
    let mut armed = machine(make);
    arm_everything(&mut armed);
    let mut steps = machine(make);
    step_everything(&mut steps);
    let mut chunk = vec![0i16; 1 << 16];
    let mut sound = |m: &mut Machine| {
        let n = m.drain_audio(&mut chunk, usize::MAX);
        chunk[..2 * n].to_vec()
    };
    let mut stops = 0;
    for frame in 0..frames {
        for m in [&mut plain, &mut armed, &mut steps] {
            between(m);
        }
        plain.advance();
        let heard = sound(&mut plain);
        let mut others = vec![&mut armed];
        run_out(others[0]);
        if stepped {
            stops += run_out(&mut steps);
            others.push(&mut steps);
        }
        for (k, m) in others.into_iter().enumerate() {
            let which = ["armed", "stepped"][k];
            assert!(m.frames == plain.frames && m.frame() == plain.frame() && m.frame_info() == plain.frame_info(), "{name}, {which}: frame {frame}'s picture");
            assert!(sound(m) == heard, "{name}, {which}: frame {frame}'s sound");
            let (a, b) = (state(m), state(&plain));
            assert!(a == b, "{name}, {which}: frame {frame}'s state differs in {:?}", differing(&plain, &a, &b));
        }
    }
    let h = &armed.debug.hooks;
    assert!(h.coverage[0].as_ref().unwrap().covered > 0, "{name}: the EE's coverage recorded");
    assert!(!stepped || stops > 100_000, "{name}: {stops} stops");
}

#[test]
fn dma_chains_and_the_ipu_are_unchanged_by_the_debugger() {
    let (name, make) = ee(s11::chain());
    the_debugger_changes_nothing(&name, &*make, 3, true, |_| {});
    let p = s27::programs().into_iter().next().unwrap();
    let image = p.image.clone();
    the_debugger_changes_nothing(&p.name, &move || {
        let mut sys = System::new(s27::program::ORIGIN);
        sys.board.map.load(s27::program::ORIGIN, &image);
        sys
    }, 3, false, |_| {});
}

#[test]
fn vu_microprograms_and_vif_are_unchanged_by_the_debugger() {
    for p in s13::all().into_iter().take(2).chain([s14::micro_program(), s14::unpack()]) {
        let (name, make) = ee(p);
        the_debugger_changes_nothing(&name, &*make, 3, true, |_| {});
    }
}

#[test]
fn the_gifs_paths_gs_drawing_and_the_pcrtc_are_unchanged_by_the_debugger() {
    for p in [s15::paths(), s17::program(&s17::triangles()), s20::timing()] {
        let (name, make) = ee(p);
        the_debugger_changes_nothing(&name, &*make, 4, true, |_| {});
    }
}

#[test]
fn the_spu2s_voices_on_the_iop_are_unchanged_by_the_debugger() {
    let (name, make) = iop(s26::gate::voices().finish().0);
    the_debugger_changes_nothing(&name, &*make, 4, true, |_| {});
}

/// The IOP released at its program once the EE has sent it, as `tests/sif.rs` releases it, here at a frame's start.
fn enter_the_iop(m: &mut Machine) {
    let b = &mut m.sys.board;
    if b.iop.held && b.iop.bus.word(0x88) == s21::vector()[2] {
        b.iop.enter(s21::K1 | s21::ORIGIN);
    }
}

#[test]
fn the_sifs_messages_between_the_ee_and_the_iop_are_unchanged_by_the_debugger() {
    for p in [s22::sif1(), s22::sif0(), s22::sif2()] {
        let image = p.image.clone();
        let make = move || {
            let mut sys = System::new(s22::EE_ORIGIN);
            sys.board.map.load(s22::EE_ORIGIN, &image);
            sys
        };
        the_debugger_changes_nothing(p.name, &make, 4, true, enter_the_iop);
    }
}

/// The IOP polling the SIF while the EE's DMA sends it a packet (S33, Sapphire_Native.md §47.8): its counts of polls
/// measure its time against the EE's, so a probe that let it lag behind the board would change the state.
#[test]
fn the_iop_polling_the_sif_while_the_ees_dma_sends_is_unchanged_by_the_debugger() {
    let mut m = machine(&s33::system);
    for _ in 0..2 {
        m.advance();
    }
    let [landed, flag, first, last] = s33::record(&m.sys);
    assert!(landed > 10 && flag > 10, "the IOP counted its polls: {landed} and {flag}");
    assert_eq!((first, last), (0x5300_0000, 0x5300_003F), "the packet landed whole");
    the_debugger_changes_nothing("s33-sif-poll", &s33::system, 3, true, |_| {});
}
