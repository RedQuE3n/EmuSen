// EmuSen_Qt.md §8.4: qml-rules and qmllint over every .qml of the kit, the style and the scenes, with the project's import paths.

use emusen_qt_test::lint::{frontend, qmllint, rules};

#[test]
fn qt_8_4_qml_rules_pass_the_kit_and_the_style() {
    let kit = frontend().join("kit-qt");
    let scenes = frontend().join("qt-test").join("qml");
    let lines = rules(&[(&kit, false), (&scenes, true)]);
    assert!(lines.is_empty(), "{}", lines.join("\n"));
}

#[test]
fn qt_8_4_qmllint_passes_the_kit_and_the_style() {
    let complaints = qmllint(&[&frontend().join("kit-qt"), &frontend().join("qt-test").join("qml")]);
    assert!(complaints.is_empty(), "{}", complaints.join("\n"));
}
