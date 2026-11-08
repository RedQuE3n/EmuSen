// EmuSen_Qt.md §1.5, §8.1: the styled controls paint only the tokens, pass the gate's rules, and a grab is saved for a human look.

use emusen_kit_core::input::WalkForm;
use emusen_kit_core::tokens::Variant;
use emusen_qt_test::Harness;

#[test]
fn qt_1_5_the_styled_controls_paint_the_tokens_and_pass_the_gate() {
    let mut harness = Harness::new();
    let mut scene = harness.load("qrc:/qt/qml/EmuSen/Scenes/qml/ControlsScene.qml", 1280, 800);
    let tree = scene.tree();
    for name in ["play", "primary", "check", "radio", "switch", "slider", "field", "combo", "progress", "tabs", "label"] {
        assert!(tree.find_named(name).is_some(), "{name} is in the tree");
    }
    let play = tree.find_named("play").expect("play");
    scene.focus_item(&tree, play);
    let tree = scene.tree();
    let focus = scene.focus(&tree);
    let edges = scene.record_edges(&tree);
    let faults = scene.audit(&tree, WalkForm::BigScreen, Variant::Dark, focus, &edges, 1280.0, 800.0);
    let lines: Vec<String> = faults.iter().map(|f| f.to_string()).collect();
    let walked: Vec<String> = edges.iter().map(|e| format!("{} -{:?}-> {}", e.from.0, e.button, e.to.0)).collect();
    assert!(lines.is_empty(), "faults:\n{}\n-- items:\n{}\n-- edges:\n{}", lines.join("\n"), tree.legend(), walked.join("\n"));
    let path = std::env::temp_dir().join("emusen-qt-test-controls.png");
    assert!(scene.grab(&path.display().to_string()), "the grab saved");
    assert!(path.metadata().map(|m| m.len() > 1000).unwrap_or(false), "a picture was written");
}
