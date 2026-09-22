//! Real games, threaded or deferred, against the same machine unthreaded and at once: state, picture and sound every frame. Behind `EMUSEN_MARSRT_STATES`; see Mars_Native.md §5.6.

use std::sync::Arc;

use crate::machine::Machine;
use crate::rom::RomImage;
use crate::vi::scan::Scanout;

/// How the machine under test runs.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Mode {
    pub threaded: bool,
    pub deferred: bool,
    pub workers: usize,
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
        let mut scanout = Scanout::default();
        scanout.repeat_rows = true;
        if let Some(state) = state {
            machine.restore_state(&std::fs::read(format!("{folder}/{state}")).unwrap()).unwrap();
            scanout.forget();
            machine.present_now(&mut scanout);
        }
        Run { machine, scanout }
    }

    pub fn frame(&mut self, n: u64) {
        drive(&mut self.machine, n);
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
    let pressed = (frame / 6) % 2 == 0;
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
    let mut reference = Run::load(folder, rom, state, Mode { threaded: false, deferred: false, workers: 1 });
    let mut subject = Run::load(folder, rom, state, mode);
    let mut scratch = Machine::new(reference.machine.rdram_bytes()).unwrap();
    let mut previous = reference.picture();
    assert!(previous == subject.picture(), "{rom}: the pictures after the load differ");
    for n in 1..=frames {
        reference.frame(n);
        subject.frame(n);

        let want = state_of(&mut reference.machine);
        let got = if how == Compare::Snapshot { snapshot_of(&mut subject.machine, &mut scratch) } else { state_of(&mut subject.machine) };
        assert!(want == got, "{rom} {state:?} {mode:?}: the states part at frame {n}, byte {}", first_difference(&want, &got));

        let picture = reference.picture();
        let shown = subject.picture();
        let expected = if mode.deferred { &previous } else { &picture };
        assert!(expected.0 == shown.0 && expected.1 == shown.1 && expected.2 == shown.2, "{rom}: frame {n} shows {:?} not {:?}", (shown.0, shown.1, shown.2), (expected.0, expected.1, expected.2));
        assert!(expected.3 == shown.3, "{rom} {mode:?}: the picture after frame {n} differs from byte {}", first_difference(&expected.3, &shown.3));
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
        if mode.threaded && frames >= 20 {
            assert!(s.drain_words.load(std::sync::atomic::Ordering::Relaxed) > 0, "{rom}: the drain ran nothing, so nothing was compared");
        }
    }
    if mode.deferred {
        eprintln!("  scans repeated and not walked: {}", subject.scanout.repeated_scans);
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
    each_game(Mode { threaded: true, deferred: false, workers: 1 }, Compare::Join);
}

#[test]
fn a_threaded_machine_snapshot_every_frame_is_the_machine_at_once() {
    each_game(Mode { threaded: true, deferred: false, workers: 1 }, Compare::Snapshot);
}

#[test]
fn a_deferred_picture_is_the_immediate_picture_of_the_frame_before() {
    each_game(Mode { threaded: false, deferred: true, workers: 1 }, Compare::Join);
}

#[test]
fn a_threaded_and_deferred_machine_is_the_machine_at_once_a_picture_late() {
    each_game(Mode { threaded: true, deferred: true, workers: 1 }, Compare::Snapshot);
}
