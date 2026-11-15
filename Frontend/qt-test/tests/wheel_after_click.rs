// EmuSen_Kit.md §7.36: a wheel turned at once after a click moves a list, as a player's would: the harness's inputs are
// apart in time as a hand's are.

use emusen_qt_test::Harness;

const SCENE: &str = "qrc:/qt/qml/EmuSen/Scenes/qml/WheelScene.qml";

#[test]
fn qt_7_36_a_wheel_turned_at_once_after_a_click_moves_the_list() {
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 320, 200);
    scene.click(160.0, 60.0);
    scene.wheel(160.0, 60.0, -3);
    let tree = scene.tree();
    scene.settle(&tree);
    let y: f64 = scene.property("list", "contentY").parse().unwrap();
    assert!(y > 0.0, "the wheel moved the list: contentY {y}");
}
