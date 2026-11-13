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
    // emusen-esde-text, ES-DE's text rules, since 2026-11-09 (EmuSen_Qt.md §4.9.2).
    assert_eq!(
        names,
        ["cxx", "cxx-qt", "cxx-qt-lib", "emusen-esde", "emusen-esde-text", "serde", "serde_json", "emusen-kit-qt", "emusen-kit-core"],
        "the painter's dependencies are its design's (EmuSen_Qt.md §4.8)"
    );
    for (crate_dir, manifest) in [
        ("esde", include_str!("../../esde/Cargo.toml")),
        ("esde-text", include_str!("../../esde-text/Cargo.toml")),
        ("kit-qt", include_str!("../../kit-qt/Cargo.toml")),
    ] {
        for name in dependencies(manifest) {
            assert!(
                !["client", "crystal", "platform", "models", "play", "endymion"].iter().any(|w| name.contains(w)),
                "{crate_dir} brings {name} into the painter's tree"
            );
        }
    }
}

/// The `.qml` and `.js` files under `dir`, build output left out.
fn scripts(dir: &std::path::Path, found: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().map(Result::unwrap) {
        let path = entry.path();
        if path.is_dir() {
            if entry.file_name() != "target" {
                scripts(&path, found);
            }
        } else if path.extension().is_some_and(|e| e == "qml" || e == "js") {
            found.push(path);
        }
    }
}

#[test]
fn qt_5_3_the_removed_hover_is_called_by_no_qml_and_offered_by_no_registry() {
    let mut found = Vec::new();
    scripts(&frontend(), &mut found);
    assert!(found.len() > 200, "the walk found the front end's QML ({} files)", found.len());
    let callers: Vec<_> = found.iter().filter(|p| std::fs::read_to_string(p).unwrap().contains(".hover(")).collect();
    assert!(callers.is_empty(), "EsdeModel.hover(row) is gone in EmuSen.Esde 1.2 (R-20261111-108, CHANGELOG-esde.md); these call it: {callers:?}");
    let registry = include_str!("../../../Testing/qmltypes/EmuSen.Esde-1.5.qmltypes");
    assert!(registry.contains("name: \"hoverAt\"") && !registry.contains("name: \"hover\""), "the registry offers hoverAt and no hover");
}
