//! `theme-check` (EmuSen_Qt.md §3.7): a theme package's `theme.json` held to the schema and the API, its QML to §1.3 and the theme's
//! import allow-list through qml-rules, its token overrides to the kit's types and contrast floors; then, when that step is clean, its
//! screens loaded and audited by `theme-audit`, run as a process (`loaded`). See `man theme-check`.

pub mod loaded;
mod package;
pub mod tokens;

use std::fmt;
use std::path::{Path, PathBuf};

const USAGE: &str = "Usage: theme-check [--static] [--state DIR] <folder>...\n  Exit 1 for a finding, 0 for none, 2 when it could not run. See man theme-check.";

/// One finding: where, under which rule, and the words at fault.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub file: String,
    pub line: usize,
    pub kind: String,
    pub words: String,
}

impl Finding {
    pub fn new(file: &str, line: usize, kind: &str, words: String) -> Finding {
        Finding { file: file.to_owned(), line, kind: kind.to_owned(), words }
    }

    /// A note says a key was ignored; it does not fail the theme.
    pub fn is_note(&self) -> bool {
        self.kind == "note"
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{} {} {}", self.file, self.line, self.kind, self.words)
    }
}

/// Every finding and note of one package, the manifest's in line order then each `.qml`'s, `shown` being the folder as the caller named it.
pub fn check(folder: &Path, shown: &str) -> Vec<Finding> {
    let shown = shown.trim_end_matches('/');
    let file = format!("{shown}/theme.json");
    let mut findings = Vec::new();
    match std::fs::read_to_string(folder.join("theme.json")) {
        Err(_) => findings.push(Finding::new(&file, 1, "schema", "theme.json is missing".to_owned())),
        Ok(text) => match emusen_theme_manifest::parse(&text) {
            Err(f) => findings.push(Finding::new(&file, f.line, &f.kind.to_string(), f.words)),
            Ok((manifest, schema)) => {
                findings.extend(schema.into_iter().map(|f| Finding::new(&file, f.line, &f.kind.to_string(), f.words)));
                findings.extend(tokens::check(&manifest, &file));
                findings.extend(package::files(&manifest, folder, shown));
            }
        },
    }
    findings.sort_by_key(|f| f.line);
    findings.extend(package::qml(folder, shown));
    findings
}

/// The command with the loaded step's program found as `loaded::program` finds it.
pub fn run(args: &[String], resolve: impl Fn(&str) -> Result<String, String>) -> Result<(String, i32), String> {
    run_with(args, resolve, loaded::program().as_deref())
}

/// The command: each folder resolved by the shell; the lines, then `ok` or `N findings`, and the exit status. `program` is
/// `theme-audit`, which loads a package whose static step is clean; `--static` stops there.
pub fn run_with(args: &[String], resolve: impl Fn(&str) -> Result<String, String>, program: Option<&Path>) -> Result<(String, i32), String> {
    let mut folders = Vec::new();
    let (mut only_static, mut state) = (false, None);
    let mut words = args.iter().skip(1);
    while let Some(arg) = words.next() {
        match arg.as_str() {
            "--help" => return Err(USAGE.to_owned()),
            "--static" => only_static = true,
            "--state" => state = Some(PathBuf::from(resolve(words.next().ok_or(format!("theme-check: --state needs a folder\n{USAGE}"))?)?)),
            other if other.starts_with("--") => return Err(format!("theme-check: {other} is not an option\n{USAGE}")),
            folder => folders.push(folder.to_owned()),
        }
    }
    if folders.is_empty() {
        return Err(format!("theme-check: a folder is needed\n{USAGE}"));
    }
    let program = match (only_static, program) {
        (true, _) => None,
        (false, Some(p)) => Some(p),
        (false, None) => {
            return Err(format!("theme-check: the loaded step needs {}: name it in {} or put it on PATH, or pass --static", loaded::PROGRAM, loaded::VARIABLE));
        }
    };
    let mut out = String::new();
    let mut count = 0;
    for given in &folders {
        let real = PathBuf::from(resolve(given)?);
        if !real.is_dir() {
            return Err(format!("theme-check: {given}: no such folder"));
        }
        let mut findings = check(&real, given);
        if let Some(program) = program {
            let shown = given.trim_end_matches('/');
            if findings.iter().any(|f| !f.is_note()) {
                findings.push(Finding::new(&format!("{shown}/theme.json"), 1, "note", "the screens were not loaded: the findings above come first".to_owned()));
            } else {
                let folder_state = state.as_ref().map(|dir| if folders.len() > 1 { dir.join(folder_name(&real)) } else { dir.clone() });
                findings.extend(loaded::audit(program, &real, shown, folder_state.as_deref())?);
            }
        }
        for finding in findings {
            count += usize::from(!finding.is_note());
            out.push_str(&format!("{finding}\n"));
        }
    }
    out.push_str(&if count == 0 { "ok\n".to_owned() } else { format!("{count} findings\n") });
    Ok((out, i32::from(count > 0)))
}

/// A package's folder name, under which its recordings go when several packages are checked at once.
fn folder_name(folder: &Path) -> String {
    folder.file_name().map_or_else(|| "theme".to_owned(), |n| n.to_string_lossy().into_owned())
}
