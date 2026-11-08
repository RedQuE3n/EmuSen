//! The package's files: the screens it lists, the files its manifest names, and every `.qml` through qml-rules' theme profile.

use std::path::{Component, Path, PathBuf};

use emusen_qml_rules::{check_theme, local_imports, qml_files};
use emusen_theme_manifest::{Manifest, Screen};

use crate::Finding;

/// A path with `.` and `..` folded away, without touching the disk.
fn folded(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

/// Each listed screen's file present, each screen file listed, and each file the manifest names present.
pub fn files(manifest: &Manifest, folder: &Path, shown: &str) -> Vec<Finding> {
    let file = format!("{shown}/theme.json");
    let mut findings = Vec::new();
    for screen in Screen::ALL {
        let listed = manifest.screens.contains(&screen);
        let present = folder.join(screen.file()).is_file();
        if listed && !present {
            findings.push(Finding::new(&file, 1, "screen", format!("{} is listed and {} is missing", screen.name(), screen.file())));
        } else if !listed && present {
            findings.push(Finding::new(&file, 1, "screen", format!("{} is not listed in screens, so it never loads", screen.file())));
        }
    }
    for p in manifest.cues.iter().map(|(_, p)| p).chain(&manifest.previews) {
        if !folder.join(&p.path).is_file() {
            findings.push(Finding::new(&file, p.line, "path", format!("{} is missing", p.path)));
        }
    }
    findings
}

/// Every `.qml` of the package under the theme profile, and each folder it imports kept inside the package.
pub fn qml(folder: &Path, shown: &str) -> Vec<Finding> {
    let root = folded(folder);
    let mut findings = Vec::new();
    for path in qml_files(folder) {
        let relative = path.strip_prefix(folder).unwrap_or(&path).display().to_string();
        let name = format!("{shown}/{relative}");
        let Ok(text) = std::fs::read_to_string(&path) else {
            findings.push(Finding::new(&name, 1, "read", "not readable as UTF-8 text".to_owned()));
            continue;
        };
        for f in check_theme(&name, &text) {
            findings.push(Finding::new(&name, f.line, &f.rule.to_string(), f.words));
        }
        let dir = path.parent().unwrap_or(folder);
        for (line, import) in local_imports(&text) {
            if import.starts_with('/') || import.contains(':') {
                continue;
            }
            let target = folded(&dir.join(&import));
            if !target.starts_with(&root) {
                findings.push(Finding::new(&name, line, "path", format!("\"{import}\" leaves the package")));
            } else if !target.is_dir() {
                findings.push(Finding::new(&name, line, "path", format!("\"{import}\" is not a folder of the package")));
            }
        }
    }
    findings
}
