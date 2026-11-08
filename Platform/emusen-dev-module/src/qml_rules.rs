//! `qml-rules`: EmuSen_Qt.md §1.3's checker as an emusen.dev command, over the library at Frontend/qml-rules (EmuSen_Kit.md §7.9).

use std::path::Path;

use emusen_qml_rules::{check, check_tree, Options};

const USAGE: &str = "Usage: qml-rules [--fixture] <path>...\n  Exit 1 for a finding, 0 for none, 2 when it could not run. See man qml-rules.";

/// Runs the check over files and directories, each path resolved by the shell; the findings, one a line, then `ok` or `N findings`.
pub fn run(args: &[String], resolve: impl Fn(&str) -> Result<String, String>) -> Result<(String, i32), String> {
    let mut options = Options::default();
    let mut paths = Vec::new();
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--fixture" => options.fixture = true,
            "--help" => return Err(USAGE.to_owned()),
            other if other.starts_with("--") => return Err(format!("qml-rules: {other} is not an option\n{USAGE}")),
            path => paths.push(path.to_owned()),
        }
    }
    if paths.is_empty() {
        return Err(format!("qml-rules: a path is needed\n{USAGE}"));
    }
    let mut findings = Vec::new();
    for given in &paths {
        let real = resolve(given)?;
        let path = Path::new(&real);
        if path.is_dir() {
            findings.extend(check_tree(path, options).0);
        } else if path.is_file() {
            let text = std::fs::read_to_string(path).map_err(|e| format!("qml-rules: {given}: {e}"))?;
            findings.extend(check(given, &text, options));
        } else {
            return Err(format!("qml-rules: {given}: no such file or directory"));
        }
    }
    let mut out = String::new();
    for finding in &findings {
        out.push_str(&finding.to_string());
        out.push('\n');
    }
    if findings.is_empty() {
        out.push_str("ok\n");
    } else {
        out.push_str(&format!("{} findings\n", findings.len()));
    }
    Ok((out, i32::from(!findings.is_empty())))
}

#[cfg(test)]
mod tests {
    use super::run;

    fn plain(path: &str) -> Result<String, String> {
        Ok(path.to_owned())
    }

    #[test]
    fn a_path_is_needed_and_options_are_checked() {
        assert!(run(&["qml-rules".into()], plain).is_err());
        assert!(run(&["qml-rules".into(), "--loud".into(), ".".into()], plain).unwrap_err().contains("not an option"));
    }

    #[test]
    fn a_clean_file_answers_ok_and_a_finding_exits_1() {
        let dir = std::env::temp_dir().join(format!("qml-rules-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let clean = dir.join("Clean.qml");
        let dirty = dir.join("Dirty.qml");
        std::fs::write(&clean, "import QtQuick\nItem { width: 4 }\n").unwrap();
        std::fs::write(&dirty, "import QtQuick\nText { text: \"Hello\" }\n").unwrap();
        let (out, status) = run(&["qml-rules".into(), clean.display().to_string()], plain).unwrap();
        assert_eq!((out.as_str(), status), ("ok\n", 0));
        let (out, status) = run(&["qml-rules".into(), dir.display().to_string()], plain).unwrap();
        assert_eq!(status, 1);
        assert!(out.ends_with("1 findings\n"), "{out}");
        assert!(out.contains("Dirty.qml:2 words \"Hello\""), "{out}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
