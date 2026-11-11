// The pointer focuses only as it moves (EmuSen_Kit.md §7.108): a walkable of a controller made under a resting pointer,
// and one enabled under it, take nothing; a move onto a walkable focuses it. It also holds Qt Quick's own behaviour, no
// rule of the kit's: a click on a disabled walkable is delivered to no TapHandler (§7.107). A test of its own, since one
// process holds one Qt application.

use emusen_qt_test::Harness;

#[test]
fn kit_7_108_only_the_pointers_own_move_focuses() {
    let mut harness = Harness::new();
    let file = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/PointerScene.qml");
    let mut scene = harness.load(&format!("file://{file}"), 960, 240);
    scene.wait(100);
    assert_eq!(scene.property("row", "activeFocus"), "true", "the card's row holds the focus");
    // A controller made under a pointer at rest, as Start makes the Home card: its walkable takes nothing.
    scene.hover(420.0, 100.0);
    scene.hover(424.0, 104.0);
    scene.wait(50);
    assert!(scene.set_property("late", "active", "true"));
    scene.wait(200);
    assert_eq!(scene.property("arrived", "visible"), "true", "the late card is under the pointer");
    assert_eq!(scene.property("row", "activeFocus"), "true", "the walkable made under the resting pointer took nothing");
    // A walkable enabled under a pointer at rest takes nothing either.
    scene.hover(740.0, 100.0);
    scene.hover(744.0, 104.0);
    scene.wait(50);
    assert!(scene.set_property("under", "enabled", "true"));
    scene.wait(200);
    assert_eq!(scene.property("cell", "enabled"), "true");
    assert_eq!(scene.property("row", "activeFocus"), "true", "the walkable enabled under the resting pointer took nothing");
    // Qt Quick delivers no tap to a disabled walkable, so the kit's click is not reached: the row hears no Return.
    assert!(scene.set_property("under", "enabled", "false"));
    scene.wait(50);
    scene.click(744.0, 104.0);
    scene.wait(50);
    assert_eq!(scene.property("scene", "returns"), "0", "Qt Quick delivered no tap to the disabled walkable");
    assert_eq!(scene.property("row", "activeFocus"), "true", "and left the focus on the row");
    // The pointer's own move onto a walkable focuses it.
    scene.hover(100.0, 120.0);
    scene.hover(104.0, 124.0);
    scene.wait(50);
    assert_eq!(scene.property("near", "activeFocus"), "true", "the pointer moved onto a walkable focuses it");
    scene.assert_quiet();
}
