// EmuSen_Kit.md §7.24: a field and a combo box among a list's rows decline Up and Down, so the vertical walk passes them and
// neither the caret nor the value moves; open, the combo box's list takes them.

use emusen_kit_core::input::UiButton;
use emusen_qt_test::Harness;

fn focused(scene: &mut emusen_qt_test::Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|f| tree.nodes[f as usize].name.clone()).unwrap_or_default()
}

#[test]
fn qt_7_24_up_and_down_walk_past_a_field_and_a_combo_box() {
    let mut harness = Harness::new();
    let mut scene = harness.load("qrc:/qt/qml/EmuSen/Scenes/qml/WalkScene.qml", 1280, 800);
    assert_eq!(scene.property("rows", "currentIndex"), "0");
    scene.pad(UiButton::Down);
    assert_eq!(scene.property("rows", "currentIndex"), "1");
    assert_eq!(focused(&mut scene), "field");
    scene.pad(UiButton::Down);
    assert_eq!(scene.property("rows", "currentIndex"), "2", "Down passed the field");
    assert_eq!(scene.property("scene", "caret"), "2", "and left its caret");
    assert_eq!(focused(&mut scene), "combo");
    scene.pad(UiButton::Down);
    assert_eq!(scene.property("rows", "currentIndex"), "3", "Down passed the combo box");
    assert_eq!(scene.property("scene", "value"), "1", "and left its value");
    scene.pad(UiButton::Up);
    scene.pad(UiButton::Up);
    assert_eq!(scene.property("rows", "currentIndex"), "1", "Up passed both");
    assert_eq!(scene.property("scene", "value"), "1");
    scene.pad(UiButton::Up);
    assert_eq!(scene.property("rows", "currentIndex"), "0", "Up from the field to the row above");
    assert_eq!(scene.property("scene", "caret"), "2", "its caret unmoved");
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Down);
    assert_eq!(focused(&mut scene), "combo");
    scene.pad(UiButton::Accept);
    scene.wait(300);
    assert_eq!(scene.property("scene", "open"), "true", "A opened the combo box's list");
    scene.assert_quiet();
}
