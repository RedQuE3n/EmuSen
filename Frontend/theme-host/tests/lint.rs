// EmuSen_Qt.md §5.3, rule Q, and §2's fence: the host's registry as built is the committed one, its test scene keeps the QML
// rules, and its tree names no client and no runtime crate.

use emusen_qt_test::lint::{frontend, qmllint, rules};
use std::path::PathBuf;

const MANIFEST: &str = include_str!("../Cargo.toml");
const COMMITTED_TYPES: &str = include_str!("../../../Testing/qmltypes/EmuSen.ThemeHost-1.0.qmltypes");
const COMMITTED_QMLDIR: &str = include_str!("../../../Testing/qmltypes/EmuSen.ThemeHost-1.0.qmldir");

/// The registry without its line numbers and header file names, which move with every edit and mean nothing to a QML author.
fn normalized(text: &str) -> String {
    text.lines()
        .map(str::trim_end)
        .filter(|line| !(line.trim_start().starts_with("lineNumber:") || line.trim_start().starts_with("file:")))
        .collect::<Vec<_>>()
        .join("\n")
}

fn built(name: &str) -> String {
    let target = std::env::var("CARGO_TARGET_DIR").map(PathBuf::from).unwrap_or_else(|_| frontend().join("target"));
    let path = target.join("cxxqt").join("qml_modules").join("EmuSen").join("ThemeHost").join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("the build's {}: {e}", path.display()))
}

#[test]
fn qt_5_3_the_committed_registry_is_the_built_one() {
    assert_eq!(
        normalized(COMMITTED_TYPES),
        normalized(&built("plugin.qmltypes")),
        "EmuSen.ThemeHost's types changed: review it, then copy plugin.qmltypes to Testing/qmltypes/EmuSen.ThemeHost-1.0.qmltypes"
    );
    assert_eq!(normalized(COMMITTED_QMLDIR), normalized(&built("qmldir")), "EmuSen.ThemeHost's qmldir changed: review it, then copy it");
}

#[test]
fn qt_8_4_the_test_scene_keeps_the_qml_rules() {
    let scenes = frontend().join("theme-host").join("tests").join("qml");
    let lines = rules(&[(&scenes, true)]);
    assert!(lines.is_empty(), "{}", lines.join("\n"));
    let complaints = qmllint(&[&scenes]);
    assert!(complaints.is_empty(), "{}", complaints.join("\n"));
}

/// The crate's own dependencies, from its manifest's `[dependencies]` table.
fn dependencies(manifest: &str) -> Vec<String> {
    manifest
        .split("\n[")
        .find(|t| t.starts_with("dependencies]"))
        .map(|t| {
            t.lines().skip(1).filter(|l| !l.trim_start().starts_with('#')).filter_map(|l| l.split_once('=').map(|(n, _)| n.trim().split('.').next().unwrap_or_default().to_owned())).collect()
        })
        .unwrap_or_default()
}

#[test]
fn qt_3_5_the_host_names_no_client_no_runtime_and_no_model_crate() {
    assert_eq!(dependencies(MANIFEST), ["cxx", "cxx-qt", "cxx-qt-lib", "emusen-qml-rules", "emusen-theme-manifest"], "the host's dependencies are its design's (EmuSen_Qt.md §3.5)");
}
