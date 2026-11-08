// EmuSen_Qt.md §5.3, rule Q: the module's registry as qmltyperegistrar writes it is committed under Testing/qmltypes and compared on every build.

use std::path::PathBuf;

const COMMITTED_TYPES: &str = include_str!("../../../Testing/qmltypes/EmuSen.Esde-1.0.qmltypes");
const COMMITTED_QMLDIR: &str = include_str!("../../../Testing/qmltypes/EmuSen.Esde-1.0.qmldir");

/// The registry without its line numbers and header file names, which move with every edit and mean nothing to a QML author.
fn normalized(text: &str) -> String {
    text.lines()
        .map(str::trim_end)
        .filter(|line| {
            let trimmed = line.trim_start();
            !(trimmed.starts_with("lineNumber:") || trimmed.starts_with("file:"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn built(name: &str) -> String {
    let target = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("target"));
    let path = target.join("cxxqt").join("qml_modules").join("EmuSen").join("Esde").join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("the build's {}: {e}", path.display()))
}

#[test]
fn qt_5_3_the_committed_registry_is_the_built_one() {
    assert_eq!(
        normalized(COMMITTED_TYPES),
        normalized(&built("plugin.qmltypes")),
        "EmuSen.Esde's types changed: review it, bump the minor for an addition or the major for a removal, then copy plugin.qmltypes to Testing/qmltypes/EmuSen.Esde-1.0.qmltypes"
    );
}

#[test]
fn qt_5_3_the_committed_qmldir_is_the_built_one() {
    assert_eq!(
        normalized(COMMITTED_QMLDIR),
        normalized(&built("qmldir")),
        "EmuSen.Esde's qmldir changed: review the .qml types it lists, then copy it to Testing/qmltypes/EmuSen.Esde-1.0.qmldir"
    );
}

#[test]
fn qt_5_3_the_registry_names_the_painter_s_types() {
    assert!(COMMITTED_TYPES.contains("exports: [\"EmuSen.Esde/MarkShapes 1.0\"]"));
    for file in ["Picture 1.0 qml/Picture.qml", "Mark 1.0 qml/Mark.qml", "TextBlock 1.0 qml/TextBlock.qml"] {
        assert!(COMMITTED_QMLDIR.contains(file), "{file} is in the qmldir");
    }
}
