// The section strip with more names than its width (EmuSen_Kit.md §7.43): the wheel over it moves it, and the gate's
// pointer reach finds no scroller the wheel leaves still. strip_fits.rs holds a strip whose names fit (§7.105).

use emusen_kit_core::audit::{pointer_reach, Fault};
use emusen_qt_test::Harness;
use std::collections::HashSet;

#[test]
fn kit_7_43_the_wheel_moves_a_section_strip_whose_names_run_past_it() {
    let mut harness = Harness::new();
    let file = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/StripScene.qml");
    let mut scene = harness.load(&format!("file://{file}"), 640, 240);
    scene.wait(100);
    let tree = scene.tree();
    let at = tree.find_named("strip").expect("the strip") as usize;
    let strip = tree.nodes[at].clone();
    let id: u64 = strip.id.parse().unwrap();
    assert!(scene.number(id, "contentWidth") > scene.number(id, "width"), "the names run past the strip");
    // The strip is not dragged: only the wheel and the shoulders move it (§7.105).
    scene.drag((strip.x + strip.w - 20.0, strip.y + strip.h / 2.0), (strip.x + 20.0, strip.y + strip.h / 2.0));
    scene.settle(&tree);
    assert_eq!(scene.number(id, "contentX"), 0.0, "a drag does not move the strip");
    let before = scene.number(id, "contentX");
    scene.wheel(strip.x + strip.w / 2.0, strip.y + strip.h / 2.0, -3);
    scene.wait(300);
    let after = scene.number(id, "contentX");
    assert!(after > before, "the wheel moves the strip: contentX {before} then {after}");
    let wheel: Vec<Fault> = pointer_reach(&tree.root, &HashSet::new()).into_iter().filter(|f| matches!(f, Fault::WheelDoesNotMove { .. })).collect();
    assert!(wheel.is_empty(), "{wheel:?}");
    scene.assert_quiet();
}

