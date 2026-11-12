// The section strip with more names than its width (EmuSen_Kit.md §7.43): a scroller the wheel and the walk move, never
// dragged, whose names a click chooses, and the gate's pointer reach, reading a flickable's wheel from its `interactive`,
// finds no scroller the wheel leaves still; on a page that scrolls, a finger's upright drag from such a strip is the page's and a
// short tap on a name chooses it. strip_fits.rs holds a strip whose names fit, which is no scroller (§7.105).

use emusen_kit_core::audit::{pointer_reach, Fault};
use emusen_qt_test::Harness;
use std::collections::HashSet;

#[test]
fn kit_7_43_the_wheel_moves_a_section_strip_whose_names_run_past_it() {
    let mut harness = Harness::new();
    let file = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/StripScene.qml");
    let mut scene = harness.load(&format!("file://{file}"), 640, 400);
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
    let name = tree.nodes.iter().find(|n| n.visible && n.text.as_deref().is_some_and(|t| ["NES", "Super Nintendo", "Nintendo 64", "Game Boy"].contains(&t)) && n.x >= strip.x && n.x + n.w <= strip.x + strip.w && n.y >= strip.y && n.y < strip.y + strip.h).expect("a whole name").clone();
    scene.click(name.x + name.w / 2.0, name.y + name.h / 2.0);
    assert_eq!(scene.property("many", "selectedSection"), name.text.clone().unwrap(), "a click on a name chooses it");
    // The walk's last section brings its name into the strip, whole.
    assert!(scene.set_property("many", "selectedSection", "PlayStation 2"));
    scene.wait(300);
    let tree = scene.tree();
    let last = tree.nodes.iter().find(|n| n.text.as_deref() == Some("PlayStation 2") && n.y >= strip.y && n.y < strip.y + strip.h).expect("the last name");
    assert!(last.visible && last.x >= strip.x && last.x + last.w <= strip.x + strip.w + 0.5, "the walk brought the last name in whole: {} wide at {}", last.w, last.x);
    assert!(scene.number(id, "contentX") > 0.0, "the walk moved the strip");
    // On a page that scrolls, a finger's upright drag that starts on a strip whose names run past it is the page's, and a
    // short tap on a name chooses it (§7.105).
    let tree = scene.tree();
    let node = |name: &str| tree.nodes[tree.find_named(name).unwrap_or_else(|| panic!("{name}")) as usize].clone();
    let (long, deep) = (node("long"), node("deep"));
    let (long_id, deep_id): (u64, u64) = (long.id.parse().unwrap(), deep.id.parse().unwrap());
    assert!(scene.number(long_id, "contentWidth") > scene.number(long_id, "width"), "the page's strip overflows");
    scene.touch_drag((long.x + long.w / 2.0, long.y + long.h - 4.0), (long.x + long.w / 2.0, long.y - 80.0));
    scene.wait(300);
    scene.settle(&tree);
    assert!(scene.number(deep_id, "contentY") > 0.0, "a finger's upright drag from the overflowing strip scrolls the page under it");
    assert_eq!(scene.number(long_id, "contentX"), 0.0, "and leaves the strip where it was");
    assert!(scene.set_property("deep", "contentY", "0"));
    scene.wait(100);
    let tree = scene.tree();
    let named = tree.nodes.iter().find(|n| n.visible && n.text.as_deref() == Some("NES") && n.y >= long.y && n.y < long.y + long.h).expect("a whole name on the page's strip").clone();
    let at = (named.x + named.w / 2.0, named.y + named.h / 2.0);
    scene.touch_drag(at, at);
    scene.wait(100);
    assert_eq!(scene.property("longer", "selectedSection"), "NES", "a short tap on a name of the overflowing strip chooses it");
    scene.assert_quiet();
}

