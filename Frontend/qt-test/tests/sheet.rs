// EmuSen_Qt.md §1.7, EmuSen_Kit.md §7.8 and §7.24, EmuSen_Interface.md §15.6: the question takes the pad first; its card is
// Mistress's question form (CoresPlan-1280x800.spec), the first answer focused; A chooses, B gives the back answer, a key
// declined behind it does not loop (§7.14); the gate at five sizes, and still once open.

use emusen_kit_core::input::{UiButton, WalkForm};
use emusen_kit_core::tokens::Variant;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::Harness;

const KEY_F1: i32 = 0x0100_0030;

fn rect(tree: &emusen_qt_test::tree::Tree, name: &str) -> (f64, f64, f64, f64) {
    let n = &tree.nodes[tree.find_named(name).unwrap_or_else(|| panic!("{name}")) as usize];
    (n.x.round(), n.y.round(), n.w.round(), n.h.round())
}

#[test]
fn qt_1_7_a_question_takes_b_as_its_back_answer_and_a_as_the_choice() {
    let mut harness = Harness::new();
    let mut scene = harness.load("qrc:/qt/qml/EmuSen/Scenes/qml/SheetScene.qml", 1280, 800);
    assert_eq!(scene.property("question", "opened"), "false");
    scene.key(KEY_F1, 0);
    scene.wait(300);
    assert_eq!(scene.property("question", "opened"), "true", "F1 opened the question");

    // The card as Mistress's: 720 wide, the words 33 inside it, the title 39 high, the answers 654 x 54 four apart, the first focused.
    let tree = scene.tree();
    let (cx, cy, cw, ch) = rect(&tree, "questionCard");
    assert_eq!((cx, cw), (280.0, 720.0));
    assert_eq!(rect(&tree, "questionCover"), (0.0, 0.0, 1280.0, 742.0), "the cover stops at the help bar");
    assert!((cy + ch / 2.0 - 371.0).abs() <= 1.0, "centred over the cover: {cy} {ch}");
    let (tx, ty, tw, th) = rect(&tree, "questionTitle");
    assert_eq!((tx, ty - cy, tw, th), (313.0, 33.0, 654.0, 39.0));
    let (ax, ay, aw, ah) = rect(&tree, "answer.0");
    let (_, by, _, _) = rect(&tree, "answer.1");
    assert_eq!((ax, aw, ah, by - ay), (313.0, 654.0, 54.0, 58.0));
    assert_eq!(cy + ch - (by + ah), 33.0, "the answers 33 above the card's foot");
    let focus = scene.focus(&tree).expect("a focus");
    assert_eq!(tree.nodes[focus as usize].name, "answer.0", "No first and focused");

    scene.pad(UiButton::Back);
    scene.wait(300);
    assert_eq!(scene.property("question", "opened"), "false", "B closed it");
    assert_eq!(scene.property("scene", "chosen"), "0", "answering No");

    scene.key(KEY_F1, 0);
    scene.wait(300);
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    scene.wait(300);
    assert_eq!(scene.property("scene", "chosen"), "1", "Down then A chose Yes");

    // A click on an answer is A there.
    scene.key(KEY_F1, 0);
    scene.wait(300);
    let tree = scene.tree();
    let (x, y, w, h) = rect(&tree, "answer.1");
    scene.set_property("scene", "chosen", "-2");
    scene.click(x + w / 2.0, y + h / 2.0);
    scene.wait(300);
    assert_eq!(scene.property("scene", "chosen"), "1", "a click chose Yes");

    // A key the frame declines while a question is open reaches the sheet once and is never sent round again (EmuSen_Kit.md §7.14).
    scene.key(KEY_F1, 0);
    scene.wait(300);
    let tree = scene.tree();
    let frame = tree.find_named("frame").expect("the frame");
    assert!(scene.focus_item(&tree, frame));
    scene.pad(UiButton::Right);
    scene.assert_quiet();

    // The gate at the five sizes with the question open over the frame, and the scene still.
    for (w, h) in SIZES {
        scene.resize(w, h);
        let tree = scene.tree();
        let first = tree.find_named("answer.0").expect("the first answer");
        let edges = scene.record_edges(&tree);
        scene.focus_item(&tree, first);
        let tree = scene.tree();
        let focus = scene.focus(&tree);
        let faults = scene.audit(&tree, WalkForm::BigScreen, Variant::Dark, focus, &edges, f64::from(w), f64::from(h));
        let lines: Vec<String> = faults.iter().map(|f| f.to_string()).collect();
        assert!(lines.is_empty(), "faults at {w}x{h}:\n{}\n-- items:\n{}", lines.join("\n"), tree.legend());
        let dir = std::env::var("EMUSEN_QT_COMPARE").unwrap_or_default();
        if !dir.is_empty() && ((w, h) == (1280, 800) || (w, h) == (1920, 1200)) {
            scene.grab(&format!("{dir}/Question-{w}x{h}.png"));
        }
    }
    scene.assert_still("the question");
    scene.assert_quiet();
}
