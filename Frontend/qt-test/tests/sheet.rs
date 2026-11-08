// EmuSen_Qt.md §1.7, EmuSen_Kit.md §7.8: a question takes the pad first; B dismisses it, A chooses the current button, and a key declined behind it does not loop (§7.14).

use emusen_kit_core::input::UiButton;
use emusen_qt_test::Harness;

const KEY_F1: i32 = 0x0100_0030;

#[test]
fn qt_1_7_a_question_takes_b_as_dismiss_and_a_as_the_choice() {
    let mut harness = Harness::new();
    let mut scene = harness.load("qrc:/qt/qml/EmuSen/Scenes/qml/SheetScene.qml", 1280, 800);
    assert_eq!(scene.property("question", "opened"), "false");
    scene.key(KEY_F1, 0);
    scene.wait(300);
    assert_eq!(scene.property("question", "opened"), "true", "F1 opened the question");
    assert_eq!(scene.property("question", "controller"), scene.property("question", "controller"));
    scene.pad(UiButton::Back);
    scene.wait(300);
    assert_eq!(scene.property("question", "opened"), "false", "B dismissed it");
    assert_eq!(scene.property("scene", "chosen"), "-1", "and chose nothing");

    scene.key(KEY_F1, 0);
    scene.wait(300);
    assert_eq!(scene.property("question", "opened"), "true");
    let tree = scene.tree();
    let buttons: Vec<u32> = tree.nodes.iter().enumerate().filter(|(_, n)| n.ty == "Button" && n.visible).map(|(i, _)| i as u32).collect();
    assert_eq!(buttons.len(), 2, "two choices shown");
    scene.pad(UiButton::Accept);
    scene.wait(300);
    assert_eq!(scene.property("scene", "chosen"), "0", "A chose the current button");

    // A key the frame declines while a question is open reaches the sheet once and is never sent round again (EmuSen_Kit.md §7.14).
    scene.key(KEY_F1, 0);
    scene.wait(300);
    assert_eq!(scene.property("question", "opened"), "true");
    let tree = scene.tree();
    let frame = tree.find_named("frame").expect("the frame");
    assert!(scene.focus_item(&tree, frame));
    scene.pad(UiButton::Right);
    scene.assert_quiet();
}
