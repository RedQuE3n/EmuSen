// A walkable in a disabled screen under a card takes no focus from the pointer (EmuSen_Kit.md §7.107): the card keeps it,
// where the pointer's focus had handed the window's focus to its root and the card heard no press. A test of its own,
// since one process holds one Qt application.

use emusen_qt_test::Harness;

#[test]
fn kit_7_107_a_disabled_walkable_under_the_pointer_leaves_the_focus_where_it_is() {
    let mut harness = Harness::new();
    let file = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/DisabledScene.qml");
    let mut scene = harness.load(&format!("file://{file}"), 640, 240);
    scene.wait(100);
    assert_eq!(scene.property("row", "activeFocus"), "true", "the card's row holds the focus");
    scene.hover(100.0, 100.0);
    scene.hover(420.0, 100.0);
    scene.hover(424.0, 104.0);
    scene.wait(50);
    assert_eq!(scene.property("cell", "enabled"), "false", "the walkable is disabled with its screen");
    assert_eq!(scene.property("row", "activeFocus"), "true", "the pointer on the disabled walkable left the focus on the card's row");
    scene.assert_quiet();
}
