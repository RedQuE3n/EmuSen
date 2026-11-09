// EmuSen_Kit.md §7.37: A on a walkable is heard by the element's own Keys.onPressed first, which may take it; else the
// walkable activates and takes it, so it never reaches the items around it. The key and the pad's A alike.

use emusen_kit_core::input::UiButton;
use emusen_qt_test::{Harness, Scene};

const RETURN: i32 = 0x0100_0004;
const ENTER: i32 = 0x0100_0005;

fn focus(scene: &mut Scene<'_>, name: &str) {
    let tree = scene.tree();
    let index = tree.find_named(name).unwrap_or_else(|| panic!("{name} in the scene"));
    assert!(scene.focus_item(&tree, index), "{name} took the focus");
}

fn counts(scene: &mut Scene<'_>) -> [String; 5] {
    ["plain", "heard", "passed", "taken", "leaked"].map(|p| scene.property("scene", p))
}

#[test]
fn qt_7_37_a_walkable_s_own_handler_hears_a_and_a_activates_once_unless_taken() {
    let mut harness = Harness::new();
    let mut scene = harness.load("qrc:/qt/qml/EmuSen/Scenes/qml/WalkableKeysScene.qml", 1280, 800);
    focus(&mut scene, "plain");
    scene.key(RETURN, 0);
    scene.pad(UiButton::Accept);
    assert_eq!(counts(&mut scene), ["2", "0", "0", "0", "0"].map(String::from), "plain: Return and the pad's A each activate, nothing leaks");
    focus(&mut scene, "passes");
    scene.key(RETURN, 0);
    scene.pad(UiButton::Accept);
    scene.key(ENTER, 0);
    assert_eq!(counts(&mut scene), ["2", "3", "3", "0", "0"].map(String::from), "a handler that passes A on hears it and A still activates once each");
    focus(&mut scene, "takes");
    scene.key(RETURN, 0);
    scene.pad(UiButton::Accept);
    assert_eq!(counts(&mut scene), ["2", "3", "3", "0", "0"].map(String::from), "a handler that takes A: no activation, nothing leaks");
    scene.key(0x0100_0031, 0);
    assert_eq!(scene.property("scene", "leaked"), "1", "a key no one acts on still reaches the items around");
    scene.assert_quiet();
}
