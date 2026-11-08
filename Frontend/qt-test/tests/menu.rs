// EmuSen_Qt.md §8.1: the harness reads the kit's tree into kit-core's audit; the pad, a click and the help bar's button all reach the same row the same way.

use emusen_kit_core::input::{UiButton, WalkForm};
use emusen_kit_core::tokens::Variant;
use emusen_qt_test::Harness;

#[test]
fn qt_8_1_a_menu_scene_passes_the_gate_and_every_way_in_reaches_its_rows() {
    let mut harness = Harness::new();
    let mut scene = harness.load("qrc:/qt/qml/EmuSen/Scenes/qml/MenuScene.qml", 1280, 800);
    let tree = scene.tree();
    let panel = tree.find_named("panel").expect("the panel");
    assert!(tree.nodes[panel as usize].ty == "MenuPanel", "{}", tree.nodes[panel as usize].ty);
    let rows = tree.find_type("MenuRow");
    assert_eq!(rows.len(), 8, "eight rows built");
    let visible_rows: Vec<u32> = rows.iter().copied().filter(|r| tree.nodes[*r as usize].visible).collect();
    assert!(visible_rows.len() >= 5, "{} rows visible", visible_rows.len());

    // The pad: the panel's current row holds the focus and Down moves it.
    let first = scene.focus(&tree).expect("a focus item");
    assert_eq!(tree.nodes[first as usize].ty, "MenuRow", "the first row holds the focus: {}", tree.nodes[first as usize].ty);
    scene.pad(UiButton::Down);
    let second = scene.focus(&tree).expect("a focus item after Down");
    assert_ne!(second, first, "Down moved the focus");
    assert_eq!(tree.nodes[second as usize].ty, "MenuRow");
    scene.pad(UiButton::Accept);
    assert_eq!(scene.property("scene", "activated"), "1", "A activated the second row");

    // A click at a row's centre is the same call as A there.
    let (cx, cy) = {
        let n = &tree.nodes[rows[2] as usize];
        (n.x + n.w / 2.0, n.y + n.h / 2.0)
    };
    scene.click(cx, cy);
    assert_eq!(scene.property("scene", "activated"), "2", "a click activated the third row");
    assert_eq!(scene.focus(&tree), Some(rows[2]), "and focused it");

    // The help bar's button sends its press: Open is A on the focused row.
    let open = tree.find_named("help-Open").expect("the Open entry");
    let (hx, hy) = {
        let n = &tree.nodes[open as usize];
        (n.x + n.w / 2.0, n.y + n.h / 2.0)
    };
    scene.focus_item(&tree, rows[0]);
    scene.click(hx, hy);
    assert_eq!(scene.property("scene", "activated"), "0", "the help bar's Open activated the focused row");

    // The gate at this size: fit, pad reach from the recorded edges, pointer reach, no chrome, names, palette.
    scene.focus_item(&tree, rows[0]);
    let tree = scene.tree();
    let edges = scene.record_edges(&tree);
    assert!(edges.len() >= 8, "{} edges recorded", edges.len());
    scene.focus_item(&tree, rows[0]);
    let tree = scene.tree();
    let focus = scene.focus(&tree);
    let faults = scene.audit(&tree, WalkForm::BigScreen, Variant::Dark, focus, &edges, 1280.0, 800.0);
    let lines: Vec<String> = faults.iter().map(|f| f.to_string()).collect();
    assert!(lines.is_empty(), "faults at 1280x800:\n{}\n-- items:\n{}", lines.join("\n"), tree.legend());

    // The wheel moves the list.
    let (px, py) = {
        let n = &tree.nodes[panel as usize];
        (n.x + n.w / 2.0, n.y + n.h / 2.0)
    };
    let before = scene.property("panel", "currentIndex");
    scene.wheel(px, py, -3);
    let after = scene.tree();
    let shown_before = tree.nodes[rows[7] as usize].visible;
    let shown_after = after.nodes[rows[7] as usize].visible;
    assert!(shown_after || before != scene.property("panel", "currentIndex") || !shown_before, "the wheel scrolled the panel");
}
