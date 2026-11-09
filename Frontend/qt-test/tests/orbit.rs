// EmuSen_Kit.md §7.40: a list laid on an orbit by the kit's OrbitSlot is deleted with its screen, as a window deletes a placed
// screen, after its covers were focused by the pad and by the pointer, and Qt logs nothing on the way.

use emusen_kit_core::input::UiButton;
use emusen_qt_test::{Harness, Scene};

fn centre(scene: &mut Scene<'_>, name: &str) -> (f64, f64) {
    let tree = scene.tree();
    let index = tree.find_named(name).unwrap_or_else(|| panic!("{name} in the scene"));
    let n = &tree.nodes[index as usize];
    (n.x + n.w / 2.0, n.y + n.h / 2.0)
}

#[test]
fn qt_7_40_an_orbit_deleted_with_its_screen_logs_nothing() {
    let mut harness = Harness::new();
    let mut scene = harness.load("qrc:/qt/qml/EmuSen/Scenes/qml/OrbitScene.qml", 1280, 800);
    scene.pad(UiButton::Right);
    scene.wait(200);
    let (x, y) = centre(&mut scene, "cover.3");
    scene.hover(x, y);
    scene.wait(300);
    assert_eq!(scene.property("orbit", "currentIndex"), "3", "the hovered cover became the orbit's current one");
    scene.assert_quiet();
    assert!(scene.destroy("screen"), "the screen deleted");
    scene.wait(200);
    assert!(scene.tree().find_named("orbit").is_none(), "the orbit is gone");
    scene.assert_quiet();
}
