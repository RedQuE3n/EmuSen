//! Real games, threaded or deferred, against the same machine unthreaded and at once: state, picture and sound every frame. Behind `EMUSEN_MARSRT_STATES`; see Mars_Native.md §5.6.

use std::sync::Arc;

use crate::cpu::hooks::{Range, stop};
use crate::ffi::space;
use crate::machine::Machine;
use crate::rom::RomImage;
use crate::rsp::Trace;
use crate::vi::scan::Scanout;

/// How the machine under test runs.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Mode {
    pub threaded: bool,
    pub deferred: bool,
    pub workers: usize,
    /// The recompiler on; the reference is always the interpreter.
    pub blocks: bool,
    /// Every debugger table armed and nothing that halts: the observed loop, stopped and resumed wherever a table fires (Mars_Native.md §6.5).
    pub observed: bool,
    /// The multiple the picture is drawn at beside the machine's own; the reference draws at the same, and a machine at one is compared in state (Mars_Native.md §6.4).
    pub scale: i32,
    /// The multiple shaded on the device, when one exists; the reference is the processor at the same multiple (Mars_Gpu.md §11).
    pub gpu: bool,
    /// The antialiasing average the scan-out takes of the multiple, one for none (Mars_Video.md §2.10).
    pub average: i32,
}

impl Mode {
    const PLAIN: Mode = Mode { threaded: false, deferred: false, workers: 1, blocks: false, observed: false, scale: 1, gpu: false, average: 1 };
}

/// How its state is compared: joined every frame, or a snapshot every frame replayed into a scratch machine, which leaves the drain running across frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Compare {
    Join,
    Snapshot,
}

pub(crate) struct Run {
    pub machine: Machine,
    pub scanout: Scanout,
}

impl Run {
    pub fn load(folder: &str, rom: &str, state: Option<&str>, mode: Mode) -> Run {
        let image = RomImage::from_image(&std::fs::read(format!("{folder}/{rom}")).unwrap()).unwrap();
        let mut machine = Machine::load_rom(Arc::new(image), true, None, None);
        machine.set_rdp_workers(mode.workers);
        machine.set_threaded_rdp(mode.threaded);
        machine.set_deferred(mode.deferred);
        machine.set_recompiler(mode.blocks);
        machine.set_scale(mode.scale);
        machine.set_gpu(mode.gpu);
        let mut scanout = Scanout::default();
        scanout.repeat_rows = true;
        scanout.average = mode.average;
        if let Some(state) = state {
            machine.restore_state(&std::fs::read(format!("{folder}/{state}")).unwrap()).unwrap();
            scanout.forget();
            machine.present_now(&mut scanout);
        }
        if mode.observed {
            arm_everything(&mut machine, hot_page(rom));
        }
        Run { machine, scanout }
    }

    pub fn frame(&mut self, n: u64) {
        drive(&mut self.machine, n);
        if self.machine.cpu.hooks.armed() {
            let mut resuming = false;
            while self.machine.run_frame_debug(resuming, resuming) != stop::FRAME {
                resuming = true;
                drain(&mut self.machine);
            }
            drain(&mut self.machine);
        } else {
            self.machine.run_frame();
        }
        self.machine.present(&mut self.scanout);
    }

    /// The rest of a frame a `run_steps` began, and its picture: what `frame` does after `drive`.
    pub fn frame_after_steps(&mut self) {
        self.machine.run_frame();
        self.machine.present(&mut self.scanout);
    }

    pub fn picture(&self) -> (u32, u32, u32, Vec<u8>) {
        (self.scanout.width, self.scanout.height, self.scanout.row_repeat, self.scanout.frame.clone())
    }
}

/// The same input to every machine: a button pattern and a stick sweep that change every frame, as `MarsRtTests.Drive`.
pub(crate) fn drive(m: &mut Machine, frame: u64) {
    const BUTTONS: [u16; 6] = [0x8000, 0x4000, 0x1000, 0x0800, 0x2000, 0x0010];
    let button = BUTTONS[(frame % 6) as usize];
    let pressed = (frame / 6).is_multiple_of(2);
    for b in BUTTONS {
        m.press(0, b, b == button && pressed);
    }
    let (x, y, c) = ((frame as f64 * 0.37).sin(), (frame as f64 * 0.23).cos(), ((frame % 7) as f64 - 3.0) / 3.0);
    m.set_stick(0, 0, (x * 127.0).round_ties_even() as i8);
    m.set_stick(0, 1, (-y * 127.0).round_ties_even() as i8);
    m.press(0, 0x0002, c <= -0.5);
    m.press(0, 0x0001, c >= 0.5);
    m.press(0, 0x0008, -c <= -0.5);
    m.press(0, 0x0004, -c >= 0.5);
}

/// The 4 KB page of RDRAM each game's state writes most (measured over twenty frames: 17, 44 and 62 thousand bytes a frame), so
/// that the watch and the data breakpoint below see stores every frame.
pub(crate) fn hot_page(rom: &str) -> u32 {
    match rom {
        "sm64.z64" => 0x20_1000,
        "oot.z64" => 0x11_C000,
        _ => 0x3A_9000,
    }
}

/// Every table armed, none of it able to halt a game: a breakpoint at an address no game runs, a watch over the game's busiest page
/// of RDRAM and a data breakpoint over its first 256 bytes (the stop after each such store is taken and resumed, as the host resumes
/// when the registry says no), a stop after every interrupt, coverage on both processors, the profiler, and a depth guard no game
/// reaches.
fn arm_everything(m: &mut Machine, page: u32) {
    m.cpu.hooks.configure(true, true, true, false, true, true);
    *m.bus.sp.trace = Some(Box::new(Trace::new()));
    m.cpu.hooks.breakpoints = vec![(0, 0)];
    m.cpu.hooks.watch_ranges = vec![Range { space: space::RDRAM, start: page, end: page + 0xFFF }];
    m.cpu.hooks.break_ranges = vec![Range { space: space::RDRAM, start: page, end: page + 0xFF }];
    m.cpu.hooks.depth_guard = 100_000;
}

/// What the host does with the logs at a stop: reads them and forgets them.
fn drain(m: &mut Machine) {
    let hooks = &mut m.cpu.hooks;
    hooks.writes_log.clear();
    hooks.calls_log.clear();
    hooks.flush();
    hooks.profile.clear();
    if let Some(bits) = hooks.coverage.as_deref_mut() {
        bits.fill(0);
    }
    if let Some(trace) = m.bus.sp.trace.as_mut() {
        trace.bits.fill(0);
    }
}

fn state_of(m: &mut Machine) -> Vec<u8> {
    m.settle();
    m.save_state_vec(false).unwrap()
}

/// The subject's state as a snapshot, loaded with its words run, which is what a joined state would have held.
fn snapshot_of(m: &mut Machine, scratch: &mut Machine) -> Vec<u8> {
    m.settle();
    let snapshot = m.save_state_vec(true).unwrap();
    scratch.load_state(&snapshot).unwrap();
    scratch.bus.dp_replay_pending();
    scratch.save_state_vec(false).unwrap()
}

fn first_difference(a: &[u8], b: &[u8]) -> usize {
    a.iter().zip(b).position(|(x, y)| x != y).unwrap_or(a.len().min(b.len()))
}

/// Runs a game in both machines and returns the frames compared; panics at the first frame they part.
pub(crate) fn compare(folder: &str, rom: &str, state: Option<&str>, frames: u64, mode: Mode, how: Compare) -> u64 {
    // Averaged on the device, the reference is the device at once, since the processor's picture parts from it after a decline (Mars_Native.md §6.4).
    let on_device = mode.gpu && mode.average > 1;
    let mut reference = Run::load(folder, rom, state, Mode { scale: mode.scale, average: mode.average, gpu: on_device, ..Mode::PLAIN });
    let mut subject = Run::load(folder, rom, state, mode);
    if mode.gpu {
        assert!(subject.machine.bus.dp.multiple.can_scan_out(), "{rom}: the device was asked for and not got: {}", subject.machine.bus.dp.gpu_report());
    }
    let mut at_one = (mode.scale > 1).then(|| Run::load(folder, rom, state, Mode::PLAIN));
    let mut scratch = Machine::new(reference.machine.rdram_bytes()).unwrap();
    let mut previous = reference.picture();
    assert!(previous == subject.picture(), "{rom}: the pictures after the load differ");
    let (mut shown_at_the_multiple, mut parted_after_a_decline) = (0u64, 0u64);
    for n in 1..=frames {
        reference.frame(n);
        subject.frame(n);
        if let Some(one) = at_one.as_mut() {
            one.frame(n);
            let (want, got) = (state_of(&mut one.machine), state_of(&mut reference.machine));
            assert!(want == got, "{rom} {state:?} at {}: the multiple changed the state at frame {n}, byte {}", mode.scale, first_difference(&want, &got));
            let (at_one, at_scale) = (one.picture(), reference.picture());
            if reference.scanout.raster_scaled().1 == mode.scale {
                shown_at_the_multiple += 1;
            } else if mode.average == 1 {
                assert!(at_scale == at_one, "{rom}: frame {n} at {} is neither the multiple's nor the console's picture", mode.scale);
            }
        }

        let want = state_of(&mut reference.machine);
        let got = if how == Compare::Snapshot { snapshot_of(&mut subject.machine, &mut scratch) } else { state_of(&mut subject.machine) };
        assert!(want == got, "{rom} {state:?} {mode:?}: the states part at frame {n}, byte {}", first_difference(&want, &got));

        let picture = reference.picture();
        let shown = subject.picture();
        let expected = if mode.deferred { &previous } else { &picture };
        assert!(expected.0 == shown.0 && expected.1 == shown.1 && expected.2 == shown.2, "{rom}: frame {n} shows {:?} not {:?}", (shown.0, shown.1, shown.2), (expected.0, expected.1, expected.2));
        // The device draws nothing of a primitive it declines, as C#'s does (Mars_Native.md §6.4.3); only then may its picture part from the processor's.
        let declined = subject.machine.bus.dp.multiple.gpu.as_ref().map_or(0, |g| g.counters.primitives_not_shaded);
        if mode.gpu && !on_device && declined > 0 && expected.3 != shown.3 {
            parted_after_a_decline += 1;
        } else {
            assert!(expected.3 == shown.3, "{rom} {mode:?}: the picture after frame {n} differs from byte {}", first_difference(&expected.3, &shown.3));
        }
        previous = picture;

        let (a, b) = (reference.machine.bus.ai.drain(1 << 20), subject.machine.bus.ai.drain(1 << 20));
        assert!(a == b, "{rom}: frame {n} played {} samples and {}", a.len(), b.len());
        let clock = reference.machine.bus.vi.video_clock();
        assert_eq!(reference.machine.bus.ai.sample_rate(clock), subject.machine.bus.ai.sample_rate(subject.machine.bus.vi.video_clock()));
    }
    if let Some(t) = subject.machine.bus.dp.threads.as_deref() {
        let (c, s) = (&t.counters, t.shared());
        eprintln!(
            "  drain: {} words in {} starts; waits by site {:?}; bystanders {}, reads narrowed {} freed {}; joins {}",
            s.drain_words.load(std::sync::atomic::Ordering::Relaxed),
            s.drain_starts.load(std::sync::atomic::Ordering::Relaxed),
            c.waits_per_site,
            c.bystanders,
            c.reads_narrowed,
            c.reads_freed,
            c.joins
        );
        if mode.threaded && state.is_some() {
            assert!(s.drain_words.load(std::sync::atomic::Ordering::Relaxed) > 0, "{rom}: the drain ran nothing, so nothing was compared");
        }
    }
    if mode.deferred {
        eprintln!("  scans repeated and not walked: {}", subject.scanout.repeated_scans);
    }
    if mode.scale > 1 {
        if let Some(gpu) = subject.machine.bus.dp.multiple.gpu.as_ref() {
            eprintln!("  on the device: {:?}; pictures parted from the processor's after a decline: {parted_after_a_decline}", gpu.counters);
        }
        eprintln!("  frames shown at the multiple: {shown_at_the_multiple} of {frames}");
        assert!(state.is_none() || shown_at_the_multiple > frames / 2, "{rom}: the multiple was shown in {shown_at_the_multiple} frames of {frames}");
    }
    frames
}

const GAMES: [(&str, Option<&str>); 6] =
    [("sm64.z64", None), ("oot.z64", None), ("ge.z64", None), ("sm64.z64", Some("sm64.state")), ("oot.z64", Some("oot.state")), ("ge.z64", Some("ge-dam.state"))];

fn frames() -> u64 {
    std::env::var("EMUSEN_MARSRT_FRAMES").ok().and_then(|v| v.parse().ok()).unwrap_or(if cfg!(debug_assertions) { 20 } else { 300 })
}

fn each_game(mode: Mode, how: Compare) {
    let Ok(folder) = std::env::var("EMUSEN_MARSRT_STATES") else {
        eprintln!("EMUSEN_MARSRT_STATES unset, not run");
        return;
    };
    let only = std::env::var("EMUSEN_MARSRT_GAME").ok();
    for (rom, state) in GAMES {
        if only.as_deref().is_some_and(|g| !rom.starts_with(g)) || !std::path::Path::new(&format!("{folder}/{rom}")).exists() {
            continue;
        }
        let started = std::time::Instant::now();
        let n = compare(&folder, rom, state, frames(), mode, how);
        eprintln!("{rom} {}: {n} frames identical, {mode:?} {how:?}, {:?}", state.unwrap_or("from power-on"), started.elapsed());
    }
}

#[test]
fn a_threaded_machine_joined_every_frame_is_the_machine_at_once() {
    each_game(Mode { threaded: true, ..Mode::PLAIN }, Compare::Join);
}

#[test]
fn a_threaded_machine_snapshot_every_frame_is_the_machine_at_once() {
    each_game(Mode { threaded: true, ..Mode::PLAIN }, Compare::Snapshot);
}

#[test]
fn a_deferred_picture_is_the_immediate_picture_of_the_frame_before() {
    each_game(Mode { deferred: true, ..Mode::PLAIN }, Compare::Join);
}

#[test]
fn a_threaded_and_deferred_machine_is_the_machine_at_once_a_picture_late() {
    each_game(Mode { threaded: true, deferred: true, ..Mode::PLAIN }, Compare::Snapshot);
}

#[test]
fn a_machine_whose_list_several_processors_share_is_the_machine_at_once() {
    let counts = std::env::var("EMUSEN_MARSRT_WORKERS").unwrap_or_else(|_| "2,3,4".into());
    for workers in counts.split(',').filter_map(|n| n.trim().parse().ok()) {
        each_game(Mode { threaded: true, deferred: workers % 2 == 1, workers, ..Mode::PLAIN }, Compare::Snapshot);
    }
}

/// At two and at four: the split with four workers, deferred, against the machine at once at the same multiple, picture for picture, and
/// the machine at one in state (Mars_Native.md §6.4).
#[test]
fn a_machine_at_a_multiple_split_and_deferred_is_the_machine_at_once_at_that_multiple() {
    for scale in [2, 4] {
        each_game(Mode { threaded: true, deferred: true, workers: 4, blocks: true, observed: false, scale, gpu: false, average: 1 }, Compare::Snapshot);
    }
}

/// At two and at four on the device, split and deferred, against the processor at once at the same multiple: the device's frames are the
/// processor's, picture for picture, on real games (Mars_Gpu.md §11, Mars_Native.md §6.4). Stands down without a Vulkan device.
#[test]
fn a_machine_at_a_multiple_on_the_device_is_the_machine_at_once_on_the_processor() {
    if crate::rdp::gpu::GpuDevice::device_names().is_empty() {
        eprintln!("no Vulkan device: not run");
        return;
    }
    for scale in [2, 4] {
        each_game(Mode { threaded: true, deferred: true, workers: 4, blocks: true, observed: false, scale, gpu: true, average: 1 }, Compare::Snapshot);
    }
}

/// Averaged on the device, split and deferred, against the device at once at the same multiple and averaging: exact, since the two
/// decline the same primitives; the processor at once is compared too, and must agree while nothing has been declined (Mars_Gpu.md §15).
#[test]
fn a_machine_averaged_on_the_device_split_and_deferred_is_the_device_at_once() {
    if crate::rdp::gpu::GpuDevice::device_names().is_empty() {
        eprintln!("no Vulkan device: not run");
        return;
    }
    for (scale, average) in [(4, 2), (4, 4), (2, 2)] {
        each_game(Mode { threaded: true, deferred: true, workers: 4, blocks: true, observed: false, scale, gpu: true, average }, Compare::Snapshot);
    }
}

#[test]
fn a_recompiled_machine_is_the_interpreted_machine() {
    each_game(Mode { blocks: true, ..Mode::PLAIN }, Compare::Join);
}

#[test]
fn a_recompiled_machine_on_four_workers_deferred_is_the_interpreted_machine_a_picture_late() {
    each_game(Mode { threaded: true, deferred: true, workers: 4, blocks: true, observed: false, scale: 1, gpu: false, average: 1 }, Compare::Snapshot);
}

/// Every debugger table armed and nothing halting, unthreaded and on four workers deferred: the observed loop leaves what the plain
/// one leaves in state, picture and sound, every frame (Mars_Native.md §6.5).
#[test]
fn an_observed_machine_with_every_table_armed_is_the_machine_plain() {
    each_game(Mode { observed: true, ..Mode::PLAIN }, Compare::Join);
    each_game(Mode { threaded: true, deferred: true, workers: 4, observed: true, ..Mode::PLAIN }, Compare::Snapshot);
}

/// A breakpoint at the general exception vector on every third frame, halted at and resumed, the frames between run plain through the
/// blocks: against the same game run through unstopped, the state is one at every halt's instruction count and state, picture and
/// sound are one at every frame's end. Unthreaded, then four workers deferred.
#[test]
fn a_game_halted_at_breakpoints_and_resumed_is_the_game_run_through() {
    let Ok(folder) = std::env::var("EMUSEN_MARSRT_STATES") else {
        eprintln!("EMUSEN_MARSRT_STATES unset, not run");
        return;
    };
    let frames = frames();
    for threaded in [false, true] {
        let mode = Mode { threaded, deferred: threaded, workers: if threaded { 4 } else { 1 }, blocks: true, observed: false, scale: 1, gpu: false, average: 1 };
        for (rom, state) in GAMES {
            let Some(state) = state else { continue };
            if !std::path::Path::new(&format!("{folder}/{rom}")).exists() {
                continue;
            }
            let started = std::time::Instant::now();
            let mut through = Run::load(&folder, rom, Some(state), mode);
            let mut halted = Run::load(&folder, rom, Some(state), mode);
            let (mut halts, mut compared) = (0u64, 0u64);
            for n in 1..=frames {
                drive(&mut through.machine, n);
                drive(&mut halted.machine, n);
                let armed = n % 3 == 0;
                halted.machine.cpu.hooks.configure(armed, false, false, false, false, false);
                halted.machine.cpu.hooks.breakpoints = if armed { vec![(0x8000_0180u32 as i32, 0x8000_0180u32 as i32)] } else { Vec::new() };
                if armed {
                    let mut resuming = false;
                    loop {
                        let why = halted.machine.run_frame_debug(resuming, false);
                        if why == stop::FRAME {
                            break;
                        }
                        assert_eq!(why, stop::BREAKPOINT, "{rom}: frame {n}");
                        assert_eq!(halted.machine.cpu.pc as u32, 0x8000_0180);
                        halts += 1;
                        resuming = true;
                        halted.machine.cpu.hooks.calls_log.clear();
                        // The unstopped machine brought to the halt's instruction, so its frame resumes where the halted one's does.
                        let taken = |m: &Machine| (m.cpu.instructions + m.cpu.run.exceptions) as u64;
                        through.machine.run_steps(taken(&halted.machine) - taken(&through.machine));
                        if n % 10 != 0 {
                            continue;
                        }
                        let (want, got) = (state_of(&mut through.machine), state_of(&mut halted.machine));
                        assert!(want == got, "{rom} {mode:?}: the states part at frame {n}, halt {halts}, byte {}", first_difference(&want, &got));
                        compared += 1;
                    }
                } else {
                    halted.machine.run_frame();
                }
                halted.machine.present(&mut halted.scanout);
                through.frame_after_steps();
                let (want, got) = (state_of(&mut through.machine), state_of(&mut halted.machine));
                assert!(want == got, "{rom} {mode:?}: the states part at the end of frame {n}, byte {}", first_difference(&want, &got));
                // Both machines present the same way, so the pictures are compared as shown, a frame late or not.
                assert!(through.picture() == halted.picture(), "{rom} {mode:?}: the pictures part at frame {n}");
                let (a, b) = (through.machine.bus.ai.drain(1 << 20), halted.machine.bus.ai.drain(1 << 20));
                assert!(a == b, "{rom}: frame {n} played {} samples and {}", a.len(), b.len());
            }
            assert!(halts >= frames / 3, "{rom}: the vector was reached {halts} times in {frames} frames");
            eprintln!("{rom} {state} {mode:?}: {frames} frames, {halts} halts, {compared} mid-frame states compared, all identical, {:?}", started.elapsed());
        }
    }
}

/// A run that carries a history of blocks against one started from its state, which has none: `Mars_Recompiler.md` §13's
/// asymmetry, which is how the C# core's stale shape was found. Both machines recompile, and both must agree.
#[test]
fn a_run_that_carries_its_blocks_and_one_loaded_from_its_state_agree() {
    let Ok(folder) = std::env::var("EMUSEN_MARSRT_STATES") else {
        eprintln!("EMUSEN_MARSRT_STATES unset, not run");
        return;
    };
    let frames = frames();
    let mode = Mode { blocks: true, ..Mode::PLAIN };
    for (rom, state) in GAMES {
        if !std::path::Path::new(&format!("{folder}/{rom}")).exists() {
            continue;
        }
        let mut carried = Run::load(&folder, rom, state, mode);
        for n in 1..=frames / 2 {
            carried.frame(n);
        }
        carried.machine.settle();
        let taken = carried.machine.save_state_vec(false).unwrap();
        let mut fresh = Run::load(&folder, rom, state, mode);
        fresh.machine.restore_state(&taken).unwrap();
        fresh.scanout.forget();
        fresh.machine.present_now(&mut fresh.scanout);
        // The carried machine is loaded from its own state as well, since a load is observable (§5.2.1).
        carried.machine.restore_state(&taken).unwrap();
        for n in frames / 2 + 1..=frames {
            carried.frame(n);
            fresh.frame(n);
            carried.machine.settle();
            fresh.machine.settle();
            assert!(
                carried.machine.save_state_vec(false).unwrap() == fresh.machine.save_state_vec(false).unwrap(),
                "{rom} {state:?}: the run with a history of blocks parts from the run without one at frame {n}"
            );
        }
        eprintln!(
            "{rom} {}: {frames} frames, the carried blocks and the fresh ones agree; carried {} live, fresh {} live",
            state.unwrap_or("from power-on"),
            carried.machine.blocks.live(),
            fresh.machine.blocks.live()
        );
    }
}

