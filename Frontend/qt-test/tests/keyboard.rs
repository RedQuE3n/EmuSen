// EmuSen_Interface.md §18.1, EmuSen_Kit.md §7.22: the docked keyboard where Mistress's search draws it, typing into its field by
// the pad, the keyboard and the pointer; B erases then closes, X is Done, Y a space; the gate at five sizes, and still when idle.

use emusen_kit_core::input::{UiButton, WalkForm};
use emusen_kit_core::tokens::Variant;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::Harness;

const SCENE: &str = "qrc:/qt/qml/EmuSen/Scenes/qml/KeyboardScene.qml";

fn rect(tree: &emusen_qt_test::tree::Tree, name: &str) -> (f64, f64, f64, f64) {
    let n = &tree.nodes[tree.find_named(name).unwrap_or_else(|| panic!("{name}")) as usize];
    (n.x, n.y, n.w, n.h)
}

#[test]
fn qt_18_1_the_docked_keyboard_types_into_its_field_and_b_erases_then_closes() {
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 1280, 800);
    let tree = scene.tree();
    // Mistress's boxes (GamesSearch-1280x800.spec): the band 0,398 1280 x 304, 1 at 244,415, Shift at 84,583, Space at 444,639.
    assert_eq!(rect(&tree, "keyboard"), (0.0, 398.0, 1280.0, 304.0));
    assert_eq!(rect(&tree, "key.0"), (244.0, 415.0, 72.0, 48.0));
    assert_eq!(rect(&tree, "key.30"), (84.0, 583.0, 152.0, 48.0), "Shift");
    assert_eq!(rect(&tree, "key.43"), (444.0, 639.0, 392.0, 48.0), "Space");
    assert_eq!(scene.property("typist", "bandHeight"), "304");

    // The pad: Down to q, A types it into the field; Y a space; B erases it.
    scene.pad(UiButton::Down);
    assert_eq!(scene.property("typist", "current"), "10");
    scene.pad(UiButton::Accept);
    assert_eq!(scene.property("field", "text"), "q");
    scene.pad(UiButton::Search);
    assert_eq!(scene.property("field", "text"), "q ", "Y is a space");
    scene.pad(UiButton::Back);
    assert_eq!(scene.property("field", "text"), "q");
    // A click types the key under the pointer; a key typed on a keyboard goes in at the caret.
    let tree = scene.tree();
    let (x, y, w, h) = rect(&tree, "key.11");
    scene.click(x + w / 2.0, y + h / 2.0);
    assert_eq!(scene.property("field", "text"), "qw");
    assert_eq!(scene.property("typist", "current"), "11", "and the clicked key is current");
    // B on an empty field closes the keyboard, not keeping; B is the keyboard's and not the screen's (backDeclined is not raised).
    scene.pad(UiButton::Back);
    scene.pad(UiButton::Back);
    assert_eq!(scene.property("field", "text"), "");
    assert_eq!(scene.property("scene", "closings"), "");
    scene.pad(UiButton::Back);
    assert_eq!(scene.property("scene", "closings"), "dropped ");
    // X is Done.
    scene.pad(UiButton::Accept);
    scene.pad(UiButton::Screensaver);
    assert_eq!(scene.property("scene", "closings"), "dropped kept ");
    assert_eq!(scene.property("field", "text"), "w", "Done keeps the text");

    // The gate at the five sizes, the keyboard holding the focus, and the screen still once settled.
    for (w, h) in SIZES {
        scene.resize(w, h);
        let tree = scene.tree();
        let keyboard = tree.find_named("keyboard").expect("the keyboard");
        let edges = scene.record_edges(&tree);
        scene.focus_item(&tree, keyboard);
        let tree = scene.tree();
        let focus = scene.focus(&tree);
        let faults = scene.audit(&tree, WalkForm::BigScreen, Variant::Dark, focus, &edges, f64::from(w), f64::from(h));
        let lines: Vec<String> = faults.iter().map(|f| f.to_string()).collect();
        assert!(lines.is_empty(), "faults at {w}x{h}:\n{}\n-- items:\n{}", lines.join("\n"), tree.legend());
        if (w, h) == (1280, 800) || (w, h) == (1920, 1200) {
            let dir = std::env::var("EMUSEN_QT_COMPARE").unwrap_or_default();
            if !dir.is_empty() {
                scene.grab(&format!("{dir}/Keyboard-{w}x{h}.png"));
            }
        }
    }
    scene.assert_still("the keyboard");
    scene.assert_quiet();
}
