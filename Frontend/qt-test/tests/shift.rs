// EmuSen_Kit.md §7.11: the walk over elements that slide when the focus moves, while items are made again for each focus:
// an element that stayed is still itself wherever it has moved, so every press from it is recorded.

use emusen_kit_core::audit::pad_reach;
use emusen_kit_core::input::{ElementId, UiButton};
use emusen_qt_test::harness::{edges, Landing};
use emusen_qt_test::Harness;

#[test]
fn qt_7_11_the_walk_follows_elements_that_slide_as_the_focus_moves() {
    let mut harness = Harness::new();
    let mut scene = harness.load("qrc:/qt/qml/EmuSen/Scenes/qml/ShiftScene.qml", 1280, 800);
    let tree = scene.tree();
    let focus = scene.focus(&tree).expect("a focus");
    let walk = scene.walk(&tree);
    let name = |i: u32| tree.nodes[i as usize].name.clone();
    for i in 0..5 {
        let from = tree.find_named(&format!("tile.{i}")).expect("a tile");
        let to = walk.iter().find(|s| s.from == from && s.button == UiButton::Right).map(|s| &s.to);
        assert!(matches!(to, Some(Landing::At(t)) if name(*t) == format!("tile.{}", i + 1)), "Right from tile {i}: {to:?}");
    }
    let faults = pad_reach(&tree.root, Some(ElementId(focus)), &edges(&walk));
    assert!(faults.is_empty(), "{faults:?}");
    scene.assert_quiet();
}
