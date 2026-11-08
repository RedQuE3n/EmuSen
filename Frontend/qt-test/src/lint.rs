//! The checkers of EmuSen_Qt.md §8.4 over a set of QML directories: `qml-rules`, and `qmllint` with the project's import paths.

use std::path::{Path, PathBuf};
use std::process::Command;

use emusen_qml_rules::{check_tree, qml_files, Options};

/// The workspace root, the parent of this crate.
pub fn frontend() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn target() -> PathBuf {
    std::env::var("CARGO_TARGET_DIR").map(PathBuf::from).unwrap_or_else(|_| frontend().join("target"))
}

/// `qml-rules` over each directory, as views or as fixtures; the findings as lines.
pub fn rules(dirs: &[(&Path, bool)]) -> Vec<String> {
    let mut lines = Vec::new();
    for (dir, fixture) in dirs {
        let (findings, _) = check_tree(dir, Options { fixture: *fixture });
        lines.extend(findings.iter().map(ToString::to_string));
    }
    lines
}

/// `qmllint` over every `.qml` under the directories, with the built modules and a copy of the style on the import path; complaints as text.
pub fn qmllint(dirs: &[&Path]) -> Vec<String> {
    let qmllint = ["/usr/lib64/qt6/bin/qmllint", "/usr/lib/qt6/bin/qmllint", "qmllint"]
        .into_iter()
        .find(|p| Command::new(p).arg("--version").output().is_ok())
        .expect("qmllint");
    let target = target();
    let lint_root = target.join("lint");
    let style_copy = lint_root.join("EmuSen");
    let _ = std::fs::remove_dir_all(&style_copy);
    std::fs::create_dir_all(&style_copy).expect("the lint tree");
    let style = frontend().join("kit-qt").join("style");
    for file in qml_files(&style) {
        std::fs::copy(&file, style_copy.join(file.file_name().unwrap())).expect("copy");
    }
    std::fs::copy(style.join("qmldir"), style_copy.join("qmldir")).expect("qmldir");
    let mut complaints = Vec::new();
    for dir in dirs {
        for file in qml_files(dir) {
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
    }
    complaints
}
