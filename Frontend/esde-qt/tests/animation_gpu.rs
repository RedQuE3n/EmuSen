//! A theme's GIF as the painter draws it on the GPU tier (EmuSen_Qt.md §4.5.9): a fixture of four coloured frames whose
//! delays differ, read through the real view on the held clock at the middle of each pacing, in each direction, at a
//! second speed, and held on its last frame once its plays are done.

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

/// The frame drawn at the middle of each of `count` pacings from the view's showing, the theme's clock held throughout.
fn frames_drawn(scene: &mut Scene<'_>, dir: &Scratch, case: &str, extra: &str, speed: f64, count: usize) -> Vec<Option<usize>> {
    let folder = dir.path(case);
    common::write_theme(&folder, "", &theme(extra));
    common::gif(&folder.join("four.gif"), 8, 8, COLOURS, &DELAYS, |f, _, _| f as u8);
    if scene.property("esdeModel", "themed") == "true" {
        assert!(scene.set_property("view", "shelves", "[]"));
        until(scene, "the view emptied", |s| s.property("esdeModel", "themed") == "false");
    }
    // The clock is held before the view is shown, so the GIF's restart is the held clock's 0.
    scene.hold_clock(None);
    scene.hold_clock(Some(0));
    assert!(scene.set_property("view", "now", common::HELD_NOW));
    assert!(scene.set_property("view", "settings", &settings(&folder, "\"StartupView\":\"gamelist\"")));
    assert!(scene.set_property("view", "shelves", SHELVES));
    until(scene, "the theme installed", |s| s.property("esdeModel", "themed") == "true");
    assert_eq!(scene.property("esdeModel", "problem"), "", "{case}");
    until(scene, "the animation drawn", |s| box_of(s, "element-anim").is_some());
    let (x, y, w, h) = box_of(scene, "element-anim").unwrap();
    let middle = (x + w / 2.0, y + h / 2.0);
    // A GIF's delay is in hundredths of a second.
    let pacing = f64::from(DELAYS[0]) * 10.0 / speed;
    (0..count)
        .map(|k| {
            scene.hold_clock(Some(((k as f64 + 0.5) * pacing).round() as i64));
            let c = scene.colours_at(&[middle])[0];
            c.and_then(|c| COLOURS.iter().position(|p| p[0].abs_diff(c.r) < 8 && p[1].abs_diff(c.g) < 8 && p[2].abs_diff(c.b) < 8))
        })
        .collect()
}

#[test]
fn a_theme_s_gif_shows_each_frame_for_its_first_delay_over_the_speed_in_its_direction_and_stops_after_its_plays() {
    assert!(emusen_esde_qt::marks::mark_named("star").is_some(), "the module is linked into this test");
    let _kwin = common::start();
    // A held clock is refused under a threaded render loop, which installs its own animation driver (EmuSen_Kit.md §7.36).
    std::env::set_var("QSG_RENDER_LOOP", "basic");
    let dir = Scratch::new("animation");
    let mut harness = Harness::new();
    let mut scene = harness.load(common::VIEW, W, H);
    let s = |v: &[usize]| v.iter().map(|f| Some(*f)).collect::<Vec<_>>();

    // The file's first frame stands one pacing, then the sequence a frame a pacing, every frame for the first delay.
    let normal = frames_drawn(&mut scene, &dir, "normal", "", 1.0, 12);
    assert_eq!(normal, s(&[0, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2]), "normal, for ever");
    let reverse = frames_drawn(&mut scene, &dir, "reverse", "<direction>reverse</direction><speed>2</speed>", 2.0, 12);
    assert_eq!(reverse, s(&[0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1]), "reverse at twice the speed");
    let alternate = frames_drawn(&mut scene, &dir, "alternate", "<direction>alternate</direction>", 1.0, 12);
    assert_eq!(alternate, s(&[0, 0, 1, 2, 3, 2, 1, 0, 1, 2, 3, 2]), "alternate turns without showing an end twice");
    let back = frames_drawn(&mut scene, &dir, "alternateReverse", "<direction>alternateReverse</direction>", 1.0, 12);
    assert_eq!(back, s(&[0, 3, 2, 1, 0, 1, 2, 3, 2, 1, 0, 1]), "alternateReverse");

    // Its plays done, the last frame shown stays and the scene waits on nothing more; alternating, a count is a play each way.
    let once = frames_drawn(&mut scene, &dir, "once", "<iterationCount>1</iterationCount>", 1.0, 9);
    assert_eq!(once, s(&[0, 0, 1, 2, 3, 3, 3, 3, 3]), "one play");
    assert_eq!(scene.property("esdeModel", "nextChangeMs"), "0");
    let there_and_back = frames_drawn(&mut scene, &dir, "there-and-back", "<direction>alternate</direction><iterationCount>1</iterationCount>", 1.0, 11);
    assert_eq!(there_and_back, s(&[0, 0, 1, 2, 3, 2, 1, 0, 0, 0, 0]), "one count of alternate: there and back");
    assert_eq!(scene.property("esdeModel", "nextChangeMs"), "0");
    scene.hold_clock(None);
    scene.assert_quiet();
}
