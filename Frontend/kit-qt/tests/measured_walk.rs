// EmuSen_Kit.md §7.122: the Qt MeasuredWalk walks rows of different heights drawn in a scene.

use emusen_qt_test::{Harness, Scene};

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/../qt-test/qml/MeasuredWalkScene.qml");

fn walk(scene: &mut Scene<'_>) -> (String, String, String, String) {
    let p = |scene: &mut Scene<'_>, name: &str| scene.property("walk", name);
    (p(scene, "count"), p(scene, "current"), p(scene, "first"), p(scene, "offset"))
}

fn down(scene: &mut Scene<'_>, presses: usize) {
    assert!(scene.set_property("scene", "presses", &presses.to_string()));
    scene.wait(20);
}

#[test]
fn kit_7_122_the_walk_follows_rows_of_their_own_heights_in_a_scene() {
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 640, 480);
    scene.wait(50);
    let at = |c: &str, f: &str, o: &str| ("5".to_owned(), c.to_owned(), f.to_owned(), o.to_owned());
    assert_eq!(walk(&mut scene), at("0", "0", "0"));
    down(&mut scene, 1);
    assert_eq!(walk(&mut scene), at("1", "0", "0"), "84 + 10 + 120 fits in 300");
    down(&mut scene, 2);
    assert_eq!(walk(&mut scene), at("2", "1", "94"), "rows 0 to 2 take 308: the window starts one row and its gap down");
    down(&mut scene, 3);
    assert_eq!(walk(&mut scene), at("3", "2", "224"), "84 + 10 + 160 fits, 120 more does not");
    // The first row drawn taller: its delegate tells the walk, and the window, two rows down, starts further down.
    assert!(scene.set_property("scene", "grown", "true"));
    scene.wait(20);
    assert_eq!(walk(&mut scene), at("3", "2", "340"), "200 + 10 + 120 + 10 above the window");
    scene.assert_quiet();
}
