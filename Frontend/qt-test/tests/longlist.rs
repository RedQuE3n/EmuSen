// EmuSen_Kit.md §7.11: the walk puts back what each press moved, so a list of 9,000 rows keeps its focus on a row through
// the walk and through a resize, and the pad reaches every row on the screen at each of the gate's sizes, each a row a click reaches.

use emusen_kit_core::audit::{pad_reach, pointer_reach, reached};
use emusen_kit_core::input::{ElementId, UiButton};
use emusen_qt_test::harness::{edges, SIZES};
use emusen_qt_test::Harness;

fn key_of(button: UiButton) -> i32 {
    match button {
        UiButton::Left => 0x0100_0012,
        UiButton::Up => 0x0100_0013,
        UiButton::Right => 0x0100_0014,
        _ => 0x0100_0015,
    }
}

#[test]
fn qt_7_11_a_list_of_9000_rows_keeps_its_focus_on_a_row_through_the_walk_and_a_resize() {
    let mut harness = Harness::new();
    let mut scene = harness.load("qrc:/qt/qml/EmuSen/Scenes/qml/LongListScene.qml", 1280, 800);
    for (w, h) in SIZES {
        scene.resize(w, h);
        scene.wait(150);
        let tree = scene.tree();
        let focus = scene.focus(&tree).expect("a focus");
        assert_eq!(tree.nodes[focus as usize].name, "row.0", "at {w}x{h} the first row holds the focus, not {}", tree.nodes[focus as usize].ty);
        let walk = scene.walk_by(&tree, |s, button| s.key(key_of(button), 0));
        let walked = edges(&walk);
        let faults = pad_reach(&tree.root, Some(ElementId(focus)), &walked);
        assert!(faults.is_empty(), "at {w}x{h}: {faults:?}\n{walk:?}");
        // Down from the last row built lands on a row made as the list scrolled, none of the tree's, never on its content.
        let pointed = pointer_reach(&tree.root, &reached(Some(ElementId(focus)), &walked));
        assert!(pointed.is_empty(), "at {w}x{h}: {pointed:?}");
        let after = scene.tree();
        let back = scene.focus(&after).map(|f| after.nodes[f as usize].name.clone());
        assert_eq!(back.as_deref(), Some("row.0"), "at {w}x{h} the walk put the focus back on the first row");
        assert_eq!(scene.property("rows", "currentIndex"), "0");
    }
    scene.assert_quiet();
}
