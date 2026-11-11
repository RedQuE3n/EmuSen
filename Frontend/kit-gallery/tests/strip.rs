// The section strip with more names than its width (EmuSen_Kit.md §7.43): a scroller the wheel and the walk move, never
// dragged, whose names a click chooses, and the gate's pointer reach, reading a flickable's wheel from its `interactive`,
// finds no scroller the wheel leaves still. strip_fits.rs holds a strip whose names fit, which is no scroller (§7.105).

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
    scene.touch_drag((strip.x + strip.w - 20.0, strip.y + strip.h / 2.0), (strip.x + 20.0, strip.y + strip.h / 2.0));
    scene.wait(300);
    scene.settle(&tree);
    assert_eq!(scene.number(id, "contentX"), 0.0, "a touch drag does not move the strip");
    let before = scene.number(id, "contentX");
    scene.wheel(strip.x + strip.w / 2.0, strip.y + strip.h / 2.0, -3);
    scene.wait(300);
    let after = scene.number(id, "contentX");
    assert!(after > before, "the wheel moves the strip: contentX {before} then {after}");
    let wheel: Vec<Fault> = pointer_reach(&tree.root, &HashSet::new()).into_iter().filter(|f| matches!(f, Fault::WheelDoesNotMove { .. })).collect();
    assert!(wheel.is_empty(), "{wheel:?}");
    assert!(strip.sh && strip.wheel, "the gate reads the strip as a scroller the wheel moves");
    // A click on a whole name chooses it.
    let tree = scene.tree();
    let name = tree.nodes.iter().find(|n| n.visible && n.text.as_deref().is_some_and(|t| ["NES", "Super Nintendo", "Nintendo 64", "Game Boy"].contains(&t)) && n.x >= strip.x && n.x + n.w <= strip.x + strip.w).expect("a whole name").clone();
    scene.click(name.x + name.w / 2.0, name.y + name.h / 2.0);
    assert_eq!(scene.property("many", "selectedSection"), name.text.clone().unwrap(), "a click on a name chooses it");
    // The walk's last section brings its name into the strip, whole.
    assert!(scene.set_property("many", "selectedSection", "PlayStation 2"));
    scene.wait(300);
    let tree = scene.tree();
    let last = tree.nodes.iter().find(|n| n.text.as_deref() == Some("PlayStation 2")).expect("the last name");
    assert!(last.visible && last.x >= strip.x && last.x + last.w <= strip.x + strip.w + 0.5, "the walk brought the last name in whole: {} wide at {}", last.w, last.x);
    assert!(scene.number(id, "contentX") > 0.0, "the walk moved the strip");
    scene.assert_quiet();
}

