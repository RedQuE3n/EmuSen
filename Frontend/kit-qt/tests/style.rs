// EmuSen_Qt.md §1.5, EmuSen_Kit.md §7.8: the EmuSen style is a resource module; its qmldir and .qrc name every control file, and only files that exist.

use std::collections::BTreeSet;
use std::path::Path;

fn style_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("style")
}

fn control_files() -> BTreeSet<String> {
    std::fs::read_dir(style_dir())
        .expect("the style directory")
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".qml"))
        .collect()
}

#[test]
fn qt_1_5_the_qmldir_lists_every_control_file_once() {
    let qmldir = std::fs::read_to_string(style_dir().join("qmldir")).expect("qmldir");
    assert!(qmldir.starts_with("module QtQuick.Controls.EmuSen\n"), "the module Qt Quick Controls looks for");
    assert!(qmldir.contains("depends EmuSen.Kit\n"), "the style reads Tokens");
    let listed: Vec<String> =
        qmldir.lines().filter(|l| l.ends_with(".qml")).map(|l| l.rsplit(' ').next().unwrap_or("").to_owned()).collect();
    let set: BTreeSet<String> = listed.iter().cloned().collect();
    assert_eq!(listed.len(), set.len(), "no file twice");
    assert_eq!(set, control_files());
    for line in qmldir.lines().filter(|l| l.ends_with(".qml")) {
        let mut words = line.split(' ');
        let (name, version, file) = (words.next().unwrap_or(""), words.next().unwrap_or(""), words.next().unwrap_or(""));
        assert_eq!(format!("{name}.qml"), file, "{line}");
        assert_eq!(version, "1.0", "{line}");
    }
}

#[test]
fn qt_1_5_the_qrc_carries_the_qmldir_and_every_control_file() {
    let qrc = std::fs::read_to_string(style_dir().join("EmuSen.qrc")).expect("EmuSen.qrc");
    assert!(qrc.contains("<qresource prefix=\"/qt/qml/QtQuick/Controls/EmuSen\">"), "where a style module is found");
    assert!(qrc.contains("<file>qmldir</file>"));
    let carried: BTreeSet<String> = qrc
        .lines()
        .filter_map(|l| l.trim().strip_prefix("<file>"))
        .filter_map(|l| l.strip_suffix("</file>"))
        .map(str::to_owned)
        .filter(|n| n.ends_with(".qml"))
        .collect();
    assert_eq!(carried, control_files());
}

#[test]
fn qt_1_5_every_control_file_composes_a_template_in_the_tokens() {
    for file in control_files() {
        let text = std::fs::read_to_string(style_dir().join(&file)).expect(&file);
        assert!(text.contains("import QtQuick.Templates as T"), "{file} composes its template");
        assert!(text.contains("import EmuSen.Kit"), "{file} reads the tokens");
        assert!(text.contains("Tokens."), "{file} paints in the tokens");
        assert!(!text.contains("palette."), "{file} reads no system palette");
        assert!(!text.contains("import QtQuick.Controls.Basic"), "{file} borrows nothing from Basic");
    }
}
