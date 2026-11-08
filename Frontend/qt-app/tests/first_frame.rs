//! Moonrise's first frame (EmuSen_Qt.md §9.3.1): the splash's clock runs from the window's first frame, and the systems'
//! worlds arrive long before the row of worlds is due, so the opening is the approved draft's from its start.

mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_kit_core::world::Timeline;
use emusen_qt::MAIN_URL;
use emusen_qt_test::Harness;
use std::time::Instant;

#[test]
fn moonrise_runs_from_the_first_frame_and_its_worlds_come_before_the_row_appears() {
    let _world = sandbox("qt-app-first-frame", &[("NES", "Synthetic.nes")]);
    let mut harness = Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    let shown = Instant::now();
    assert_eq!(scene.property("moonrise", "running"), "true", "the splash's clock runs from the window's first frame");
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    for _ in 0..200 {
        if !scene.property("moonrise", "worlds").is_empty() {
            break;
        }
        scene.wait(10);
    }
    let came = shown.elapsed().as_secs_f64();
    let row = Timeline::MOONRISE.appear.from * Timeline::MOONRISE.pace;
    assert!(!scene.property("moonrise", "worlds").is_empty(), "the systems' worlds arrive");
    assert!(came < row, "the worlds came {came:.3} s after the first frame, after the row appears at {row:.3} s");
    assert_eq!(scene.property("moonrise", "finished"), "false");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "library", "loading", "false", 10000);
    scene.assert_quiet();
}
