// A section strip whose names fit (EmuSen_Kit.md §7.105): no scroller at all, it leaves the wheel and a finger's drag to
// the page under it, a drag does not move it, and a click on a name chooses it. A test of its own, since one process holds one Qt application.

use emusen_qt_test::Harness;

#[test]
fn kit_7_105_a_strip_whose_names_fit_leaves_the_wheel_to_what_is_under_it_and_a_click_chooses() {
    let mut harness = Harness::new();
    let file = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/StripScene.qml");
    let mut scene = harness.load(&format!("file://{file}"), 640, 240);
    scene.wait(100);
    let tree = scene.tree();
    let node = |name: &str| tree.nodes[tree.find_named(name).unwrap_or_else(|| panic!("{name}")) as usize].clone();
    let (strip, page) = (node("fits"), node("page"));
    let (strip_id, page_id): (u64, u64) = (strip.id.parse().unwrap(), page.id.parse().unwrap());
    assert!(scene.number(strip_id, "contentWidth") <= scene.number(strip_id, "width"), "the names fit the strip");
    assert!(!strip.sh && !strip.wheel, "a strip whose names fit is no scroller at all, to the gate or the wheel");
    let centre = (strip.x + strip.w / 2.0, strip.y + strip.h / 2.0);
    scene.wheel(centre.0, centre.1, -3);
    scene.settle(&tree);
    assert!(scene.number(page_id, "contentY") > 0.0, "the wheel over a strip whose names fit moves the page under it");
    assert_eq!(scene.number(strip_id, "contentX"), 0.0, "and not the strip");
    assert!(scene.set_property("page", "contentY", "0"));
    let tree = scene.tree();
    let strip = tree.nodes[tree.find_named("fits").unwrap() as usize].clone();
    scene.drag((strip.x + strip.w - 20.0, strip.y + strip.h / 2.0), (strip.x + 20.0, strip.y + strip.h / 2.0));
    scene.settle(&tree);
    assert_eq!(scene.number(strip_id, "contentX"), 0.0, "a drag does not move the strip");
    // A finger's drag up from the strip is the page's: the strip takes no touch where it is no scroller.
    scene.touch_drag((strip.x + strip.w / 2.0, strip.y + strip.h - 4.0), (strip.x + strip.w / 2.0, strip.y - 80.0));
    scene.wait(300);
    scene.settle(&tree);
    assert!(scene.number(page_id, "contentY") > 0.0, "a finger's drag from a strip whose names fit moves the page");
    assert!(scene.set_property("page", "contentY", "0"));
    scene.wait(50);
    let saturn = tree.nodes.iter().find(|n| n.visible && n.text.as_deref() == Some("Saturn")).expect("the Saturn tab").clone();
    scene.click(saturn.x + saturn.w / 2.0, saturn.y + saturn.h / 2.0);
    assert_eq!(scene.property("scene", "chosen"), "Saturn", "a click on a name chooses it");
    scene.assert_quiet();
}
