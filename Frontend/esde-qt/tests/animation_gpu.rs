//! A theme's GIF as the painter draws it on the GPU tier (EmuSen_Qt.md §4.5.9): a fixture of four coloured frames whose
//! delays differ, read through the real view against the scene's own clock, in each direction, at a second speed, and
//! held on its last frame once its plays are done.
//!
//! A resting scene's clock is the view's Timer, which runs on wall time and is not held by the harness's clock (measured
//! 2026-11-14: two ticks in 1150 held ms), so the test counts the scene's ticks: each publishes a revision and moves the
//! scene by `nextChangeMs`, one pacing while a GIF plays.

mod common;

use common::{box_of, settings, until, Scratch};
use emusen_qt_test::{Harness, Scene};

const W: i32 = 640;
const H: i32 = 400;

// The fixture's frames, one colour each; its delays in hundredths of a second, of which ES-DE paces by the first alone.
const COLOURS: [[u8; 3]; 4] = [[255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 255]];
const DELAYS: [u16; 4] = [10, 20, 30, 40];

const SHELVES: &str = "[{\"name\":\"snes\",\"full_name\":\"Super Nintendo\",\"theme\":\"snes\",\"kind\":\"regular\",\
    \"games\":[{\"file\":\"g.sfc\",\"name\":\"Game\",\"esde_system\":\"snes\"}]}]";

fn theme(extra: &str) -> String {
    format!(
        "<view name=\"gamelist\"><animation name=\"anim\"><pos>0.25 0.25</pos><size>0.5 0.5</size>\
         <path>./four.gif</path>{extra}</animation></view>"
    )
}

fn frame_of(scene: &mut Scene<'_>, at: (f64, f64)) -> Option<usize> {
    let c = scene.colours_at(&[at])[0]?;
    COLOURS.iter().position(|p| p[0].abs_diff(c.r) < 8 && p[1].abs_diff(c.g) < 8 && p[2].abs_diff(c.b) < 8)
}

/// The GIF installed afresh, then the frame drawn read against the scene's ticks until `ticks` have passed: each sample
/// is (ticks counted, frame drawn). Returns the samples and the scene's pacing in ms.
fn samples(scene: &mut Scene<'_>, dir: &Scratch, case: &str, extra: &str, ticks: usize) -> (Vec<(usize, Option<usize>)>, i32, (f64, f64)) {
    let folder = dir.path(case);
    common::write_theme(&folder, "", &theme(extra));
    common::gif(&folder.join("four.gif"), 8, 8, COLOURS, &DELAYS, |f, _, _| f as u8);
    if scene.property("esdeModel", "themed") == "true" {
        assert!(scene.set_property("view", "shelves", "[]"));
        until(scene, "the view emptied", |s| s.property("esdeModel", "themed") == "false");
    }
    assert!(scene.set_property("view", "now", common::HELD_NOW));
    assert!(scene.set_property("view", "settings", &settings(&folder, "\"StartupView\":\"gamelist\"")));
    assert!(scene.set_property("view", "shelves", SHELVES));
    until(scene, "the theme installed", |s| s.property("esdeModel", "themed") == "true");
    assert_eq!(scene.property("esdeModel", "problem"), "", "{case}");
    until(scene, "the animation drawn", |s| box_of(s, "element-anim").is_some());
    let (x, y, w, h) = box_of(scene, "element-anim").unwrap();
    let middle = (x + w / 2.0, y + h / 2.0);
    let pacing: i32 = scene.property("esdeModel", "nextChangeMs").parse().unwrap();
    until(scene, "the first frame drawn", |s| frame_of(s, middle).is_some());
    // One watch for the whole test, since a second one of the same property is not the one read; each case counts from here.
    if scene.notified("esdeModel", "revision").is_none() {
        assert!(scene.watch("esdeModel", "revision"));
    }
    let start = scene.notified("esdeModel", "revision").unwrap();
    let mut out = vec![];
    loop {
        let before = scene.notified("esdeModel", "revision").unwrap() - start;
        let frame = frame_of(scene, middle);
        // A sample counts only where no tick fell between the count and the picture.
        if scene.notified("esdeModel", "revision").unwrap() - start == before {
            out.push((before, frame));
        }
        if before >= ticks {
            return (out, pacing, middle);
        }
        scene.wait(5);
    }
}

/// Whether one shift makes every sample the frame `expected` gives at its tick: a tick may fall before the watch, and a
/// measurement's publish, queued on the event loop (`EsdeModel::measured`), may land after it.
fn fits(samples: &[(usize, Option<usize>)], expected: &[usize]) -> bool {
    (-1..=1).any(|shift: isize| samples.iter().all(|&(n, f)| f == n.checked_add_signed(shift).and_then(|i| expected.get(i).copied())))
}

#[test]
fn a_theme_s_gif_shows_each_frame_for_its_first_delay_over_the_speed_in_its_direction_and_stops_after_its_plays() {
    assert!(emusen_esde_qt::marks::mark_named("star").is_some(), "the module is linked into this test");
    let _kwin = common::start();
    let dir = Scratch::new("animation");
    let mut harness = Harness::new();
    let mut scene = harness.load(common::VIEW, W, H);
    // A GIF's delay is in hundredths of a second.
    let first_delay = i32::from(DELAYS[0]) * 10;

    // The file's first frame stands one pacing, then the sequence a frame a pacing, every frame for the first delay.
    let (s, pacing, _) = samples(&mut scene, &dir, "normal", "", 12);
    assert_eq!(pacing, first_delay, "every frame paced by the first delay");
    assert!(fits(&s, &[0, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0]), "normal: {s:?}");
    let (s, pacing, _) = samples(&mut scene, &dir, "reverse", "<direction>reverse</direction><speed>2</speed>", 12);
    assert_eq!(pacing, first_delay / 2, "twice the speed, half the pacing");
    assert!(fits(&s, &[0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3]), "reverse: {s:?}");
    let (s, _, _) = samples(&mut scene, &dir, "alternate", "<direction>alternate</direction>", 12);
    assert!(fits(&s, &[0, 0, 1, 2, 3, 2, 1, 0, 1, 2, 3, 2, 1, 0]), "alternate turns without showing an end twice: {s:?}");
    let (s, _, _) = samples(&mut scene, &dir, "alternateReverse", "<direction>alternateReverse</direction>", 12);
    assert!(fits(&s, &[0, 3, 2, 1, 0, 1, 2, 3, 2, 1, 0, 1, 2, 3]), "alternateReverse: {s:?}");

    // Its plays done, the last frame shown stays and the scene waits on nothing more; alternating, a count is a play each way.
    for (case, extra, last, ticks) in [
        ("once", "<iterationCount>1</iterationCount>", 3, 4),
        ("there-and-back", "<direction>alternate</direction><iterationCount>1</iterationCount>", 0, 7),
    ] {
        let (s, pacing, middle) = samples(&mut scene, &dir, case, extra, ticks);
        let mut expected = if last == 3 { vec![0, 0, 1, 2, 3] } else { vec![0, 0, 1, 2, 3, 2, 1, 0] };
        expected.extend([last; 4]);
        assert!(fits(&s, &expected), "{case}: {s:?}");
        until(&mut scene, "the plays done", |sc| sc.property("esdeModel", "nextChangeMs") == "0");
        let done = scene.notified("esdeModel", "revision").unwrap();
        scene.wait(3 * pacing);
        assert_eq!(scene.notified("esdeModel", "revision"), Some(done), "{case}: no tick once the plays are done");
        assert_eq!(frame_of(&mut scene, middle), Some(last), "{case}: the last frame shown stays");
    }
    scene.assert_quiet();
}
