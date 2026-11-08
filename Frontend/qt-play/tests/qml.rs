//! The program's window keeps the QML rules and passes qmllint against the built modules (EmuSen_Qt.md §1.3).

use emusen_qt_test::lint::{frontend, qmllint, rules};

#[test]
fn the_window_keeps_the_qml_rules() {
    let window = frontend().join("qt-play").join("qml");
    let findings = rules(&[(&window, false)]);
    assert!(findings.is_empty(), "qml-rules on the window:\n{}", findings.join("\n"));
    let complaints = qmllint(&[&window]);
    assert!(complaints.is_empty(), "qmllint on the window:\n{}", complaints.join("\n"));
}
