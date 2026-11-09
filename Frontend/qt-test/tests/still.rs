// EmuSen_Qt.md §9.3.2: a still scene swaps no frame for 2 s; a span in which the scene's clock read a new minute is measured
// once more, and only once, so a change without one, or a change in the second span too, still fails.

use emusen_qt_test::Harness;
use std::panic::{catch_unwind, AssertUnwindSafe};

const SCENE: &str = "qrc:/qt/qml/EmuSen/Scenes/qml/StillScene.qml";

/// `assert_still` over the fixture with `changes` colour changes `gap` ms apart, the first 400 ms in: its failure's words.
fn still(harness: &mut Harness, with_minute: bool, changes: u32, gap: u32) -> Result<(), String> {
    let mut scene = harness.load(SCENE, 320, 200);
    scene.assert_still("the fixture before its timer");
    assert!(scene.set_property("scene", "withMinute", &with_minute.to_string()));
    assert!(scene.set_property("change", "left", &changes.to_string()));
    assert!(scene.set_property("change", "gap", &gap.to_string()));
    assert!(scene.set_property("change", "running", "true"));
    catch_unwind(AssertUnwindSafe(|| scene.assert_still("the fixture"))).map_err(|e| e.downcast_ref::<String>().cloned().unwrap_or_default())
}

#[test]
fn qt_9_3_2_a_new_minute_is_measured_once_more_and_nothing_else_is_excused() {
    let mut harness = Harness::new();
    let fault = still(&mut harness, false, 1, 0).expect_err("a change with no new minute is a frame with nothing changing");
    assert!(fault.contains("id=\"square\"") && fault.contains("Content"), "the failure names what Qt found dirty: {fault}");
    assert_eq!(still(&mut harness, true, 1, 0), Ok(()), "the clock's new minute is a change: the next span is measured");
    assert!(still(&mut harness, true, 2, 2400).is_err(), "a change in the second span fails though the first had a new minute");
}
