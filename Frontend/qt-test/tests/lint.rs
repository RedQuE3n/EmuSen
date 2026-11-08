// EmuSen_Qt.md §8.4: qml-rules and qmllint over every .qml of the kit and the style, with the project's import paths.

use std::path::{Path, PathBuf};
use std::process::Command;

use emusen_qml_rules::{check_tree, Options};

fn frontend() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

#[test]
fn qt_8_4_qml_rules_pass_the_kit_and_the_style() {
    let (findings, summary) = check_tree(&frontend().join("kit-qt"), Options::default());
    let lines: Vec<String> = findings.iter().map(|f| f.to_string()).collect();
    assert_eq!(summary, "ok", "{}", lines.join("\n"));
    let (fixtures, summary) = check_tree(&frontend().join("qt-test").join("qml"), Options { fixture: true });
    let lines: Vec<String> = fixtures.iter().map(|f| f.to_string()).collect();
    assert_eq!(summary, "ok", "{}", lines.join("\n"));
}

#[test]
fn qt_8_4_qmllint_passes_the_kit_and_the_style() {
    let qmllint = ["/usr/lib64/qt6/bin/qmllint", "/usr/lib/qt6/bin/qmllint", "qmllint"]
        .into_iter()
        .find(|p| Command::new(p).arg("--version").output().is_ok())
        .expect("qmllint");
    let target = std::env::var("CARGO_TARGET_DIR").map(PathBuf::from).unwrap_or_else(|_| frontend().join("target"));
    let lint_root = target.join("lint");
    let style_copy = lint_root.join("EmuSen");
    let _ = std::fs::remove_dir_all(&style_copy);
    std::fs::create_dir_all(&style_copy).expect("the lint tree");
    for file in emusen_qml_rules::qml_files(&frontend().join("kit-qt").join("style")) {
        std::fs::copy(&file, style_copy.join(file.file_name().unwrap())).expect("copy");
    }
    std::fs::copy(frontend().join("kit-qt").join("style").join("qmldir"), style_copy.join("qmldir")).expect("qmldir");
    let mut files = emusen_qml_rules::qml_files(&frontend().join("kit-qt"));
    files.extend(emusen_qml_rules::qml_files(&frontend().join("qt-test").join("qml")));
    let mut complaints = Vec::new();
    for file in files {
        let out = Command::new(qmllint)
            .arg("-I")
            .arg(target.join("cxxqt").join("qml_modules"))
            .arg("-I")
            .arg(&lint_root)
            .arg("-I")
            .arg("/usr/lib64/qt6/qml")
            .arg(&file)
            .output()
            .expect("qmllint runs");
        let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        if text.contains("Warning") || text.contains("Error") {
            complaints.push(format!("{}\n{text}", file.display()));
        }
    }
    assert!(complaints.is_empty(), "{}", complaints.join("\n"));
}
