// EmuSen_Qt.md §8.4: qml-rules and qmllint over every .qml of the painter and its test scenes, and the fence of §4.8.

use emusen_qt_test::lint::{frontend, qmllint, rules};

const MANIFEST: &str = include_str!("../Cargo.toml");

#[test]
fn qt_8_4_qml_rules_pass_the_painter() {
    let module = frontend().join("esde-qt").join("qml");
    let scenes = frontend().join("esde-qt").join("tests").join("qml");
    let lines = rules(&[(&module, false), (&scenes, true)]);
    assert!(lines.is_empty(), "{}", lines.join("\n"));
}

#[test]
fn qt_8_4_qmllint_passes_the_painter() {
    let complaints = qmllint(&[&frontend().join("esde-qt").join("qml"), &frontend().join("esde-qt").join("tests").join("qml")]);
    assert!(complaints.is_empty(), "{}", complaints.join("\n"));
}

/// The crate's own dependencies, from its manifest's `[dependencies]` table.
fn dependencies(manifest: &str) -> Vec<String> {
    manifest
        .split("\n[")
        .find(|t| t.starts_with("dependencies]"))
        .map(|t| t.lines().skip(1).filter_map(|l| l.split_once('=').map(|(n, _)| n.trim().split('.').next().unwrap_or_default().to_owned())).filter(|n| !n.starts_with('#')).collect())
        .unwrap_or_default()
}

#[test]
fn qt_4_8_the_painter_names_no_client_and_no_runtime() {
    let names = dependencies(MANIFEST);
    assert_eq!(names, ["cxx", "cxx-qt", "cxx-qt-lib", "emusen-esde", "serde_json", "emusen-kit-qt"], "the painter's dependencies are its design's (EmuSen_Qt.md §4.8)");
    for (crate_dir, manifest) in [("esde", include_str!("../../esde/Cargo.toml")), ("kit-qt", include_str!("../../kit-qt/Cargo.toml"))] {
        for name in dependencies(manifest) {
            assert!(
                !["client", "crystal", "platform", "models", "play", "endymion"].iter().any(|w| name.contains(w)),
                "{crate_dir} brings {name} into the painter's tree"
            );
        }
    }
}
