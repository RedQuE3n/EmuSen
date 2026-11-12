// EmuSen_Qt.md §5.3, rule Q: the module's registry as qmltyperegistrar writes it is committed under Testing/qmltypes and compared on every build.

use std::path::PathBuf;

const COMMITTED_TYPES: &str = include_str!("../../../Testing/qmltypes/EmuSen.Kit-1.0.qmltypes");
const COMMITTED_QMLDIR: &str = include_str!("../../../Testing/qmltypes/EmuSen.Kit-1.0.qmldir");

/// The registry without its line numbers and header file names, which move with every edit and mean nothing to a QML author.
fn normalized(text: &str) -> String {
    text.lines()
        .map(str::trim_end)
        .filter(|line| {
            let trimmed = line.trim_start();
            !(trimmed.starts_with("lineNumber:") || trimmed.starts_with("file:"))
        })
        .map(without_line_number)
        .collect::<Vec<_>>()
        .join("\n")
}

/// A line without the `; lineNumber: N` qmltyperegistrar writes inside a one-line entry such as a signal's.
fn without_line_number(line: &str) -> String {
    const MARK: &str = "; lineNumber: ";
    match line.find(MARK) {
        Some(at) => {
            let rest = &line[at + MARK.len()..];
            let digits = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
            format!("{}{}", &line[..at], &rest[digits..])
        }
        None => line.to_owned(),
    }
}

#[test]
fn qt_5_3_line_numbers_are_dropped_in_both_forms() {
    let moved = "Component {\n    lineNumber: 12\n    Signal { name: \"sceneChanged\"; lineNumber: 282 }\n}";
    let here = "Component {\n    lineNumber: 40\n    Signal { name: \"sceneChanged\"; lineNumber: 302 }\n}";
    assert_eq!(normalized(moved), normalized(here));
    assert_eq!(normalized(here), "Component {\n    Signal { name: \"sceneChanged\" }\n}");
    assert_ne!(normalized(here), normalized(&here.replace("sceneChanged", "sceneMoved")), "a renamed signal still differs");
}

fn built(name: &str) -> String {
    let target = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("target"));
    let path = target.join("cxxqt").join("qml_modules").join("EmuSen").join("Kit").join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("the build's {}: {e}", path.display()))
}

#[test]
fn qt_5_3_the_committed_registry_is_the_built_one() {
    assert_eq!(
        normalized(COMMITTED_TYPES),
        normalized(&built("plugin.qmltypes")),
        "EmuSen.Kit's types changed: review it, bump the minor for an addition or the major for a removal, then copy plugin.qmltypes to Testing/qmltypes/EmuSen.Kit-1.0.qmltypes"
    );
}

#[test]
fn qt_5_3_the_committed_qmldir_is_the_built_one() {
    assert_eq!(
        normalized(COMMITTED_QMLDIR),
        normalized(&built("qmldir")),
        "EmuSen.Kit's qmldir changed: review the .qml types it lists, then copy it to Testing/qmltypes/EmuSen.Kit-1.0.qmldir"
    );
}

#[test]
fn qt_5_3_the_registry_names_the_kits_types() {
    for name in ["Tokens", "FrameController", "PadRouter", "PadButton", "Form", "Role", "Variant", "Tiers"] {
        assert!(COMMITTED_TYPES.contains(&format!("name: \"{name}\"")), "{name} is in the registry");
    }
    assert!(COMMITTED_TYPES.contains("exports: [\"EmuSen.Kit/Tokens 1.0\"]"));
    assert!(COMMITTED_TYPES.contains("isSingleton: true"));
    assert!(COMMITTED_QMLDIR.contains("Walkable 1.0 qml/Walkable.qml"));
}
