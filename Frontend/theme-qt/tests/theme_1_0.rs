//! Rule T (EmuSen_Qt.md §3.2): a theme written for EmuSen.Theme 1.0 still finds every 1.0 type and the tokens with
//! `import EmuSen.Theme 1.0`, and does not find a part added in 1.1. One harness test, as the harness is one per process.

use emusen_qt_test::Harness;

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/Theme10Scene.qml");

#[test]
fn qt_3_2_a_1_0_import_finds_the_1_0_types_and_not_a_later_part() {
    // The module's crate linked in, as a theme's host links it: its registration comes with it.
    std::hint::black_box(emusen_theme_qt::host::set_token_overrides as fn(_, _));
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 400, 300);
    assert_eq!(scene.property("scene", "framed"), "true", "EdgeFade, a 1.0 type, is placed");
    assert!(!scene.property("scene", "accent").is_empty(), "Tokens is read");
    // A part of 1.1 asked for under a 1.0 import, in a file of the test's own outside the tree (the linter would refuse it):
    // the Loader's status is Error and Qt says the type is unknown.
    let dir = std::env::temp_dir().join(format!("theme-1-0-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("Later.qml");
    std::fs::write(&file, "import QtQuick\nimport EmuSen.Theme 1.0\nFocusRing {\n}\n").unwrap();
    assert!(scene.set_property("later", "source", &format!("file://{}", file.display())));
    assert_eq!(scene.property("later", "status"), "Error", "FocusRing, of 1.1, is no type under a 1.0 import");
    let messages = harness.messages();
    assert!(messages.iter().any(|m| m.contains("FocusRing is not a type")), "Qt says why: {messages:?}");
    let _ = std::fs::remove_dir_all(&dir);
    harness.assert_quiet();
}
