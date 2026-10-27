//! `emusen.dev`: `dupes`, which runs PMD's CPD over the repository's tracked files and reports the duplicated blocks that
//! overlap the lines a change added. What counts as a finding is `finding`'s; this module runs git and CPD and prints.
//! See EmuSen_DianaSh.md and `man dupes`.

mod finding;

use dianash_module::{Answer, Call, Manifest, Module};
use finding::{evaluate, parse_accept, parse_csv, parse_diff, Block, Ignore};
use std::path::{Path, PathBuf};
use std::process::Command;

pub use finding::{Block as DupeBlock, Finding, Place, Report};

pub const COMMANDS: &[(&str, bool, &str)] = &[("dupes", false, "  dupes [--base <rev>] [--min-tokens N] [--warn-tokens N] [--lang cs|rust|all] [--repo <dir>]\n                                duplicated blocks that overlap the lines added since <rev> (the whole tree without --base)")];

const USAGE: &str = "Usage: dupes [--base <rev>] [--min-tokens N] [--warn-tokens N] [--lang cs|rust|all] [--repo <dir>]\n  Exit 1 for a finding of --min-tokens (100) or more, 0 otherwise, 2 when it could not run. See man dupes.";

pub struct DevCommands;

pub static MODULE: DevCommands = DevCommands;

/// What a command line asks for.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub base: Option<String>,
    pub min: u32,
    pub warn: u32,
    pub rust: bool,
    pub cs: bool,
    pub repo: Option<String>,
}

pub fn options(args: &[String]) -> Result<Options, String> {
    let mut o = Options { base: None, min: 100, warn: 75, rust: true, cs: true, repo: None };
    let mut words = args.iter().skip(1);
    while let Some(w) = words.next() {
        let mut value = |name: &str| words.next().cloned().ok_or_else(|| format!("dupes: {name} needs a value\n{USAGE}"));
        let number = |name: &str, v: String| v.parse::<u32>().ok().filter(|&n| n > 0).ok_or_else(|| format!("dupes: {name} needs a number above 0\n{USAGE}"));
        match w.as_str() {
            "--base" => o.base = Some(value("--base")?),
            "--min-tokens" => o.min = number("--min-tokens", value("--min-tokens")?)?,
            "--warn-tokens" => o.warn = number("--warn-tokens", value("--warn-tokens")?)?,
            "--repo" => o.repo = Some(value("--repo")?),
            "--lang" => match value("--lang")?.as_str() {
                "cs" => (o.cs, o.rust) = (true, false),
                "rust" => (o.cs, o.rust) = (false, true),
                "all" => (o.cs, o.rust) = (true, true),
                other => return Err(format!("dupes: --lang is cs, rust or all, not {other}\n{USAGE}")),
            },
            other => return Err(format!("dupes: {other} is not an option\n{USAGE}")),
        }
    }
    if o.warn > o.min {
        return Err(format!("dupes: --warn-tokens {} is above --min-tokens {}\n{USAGE}", o.warn, o.min));
    }
    Ok(o)
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("git").arg("-C").arg(root).args(args).output().map_err(|e| format!("dupes: git did not start: {e}"))?;
    if !out.status.success() {
        return Err(format!("dupes: git {} failed: {}", args.join(" "), String::from_utf8_lossy(&out.stderr).trim()));
    }
    Ok(out.stdout)
}

fn cpd_program() -> PathBuf {
    match std::env::var_os("EMUSEN_CPD") {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".cache/emusen/toolchains/pmd-7.28.0/bin/pmd"),
    }
}

/// CPD's CSV for the tracked files of one language that are not ignored.
fn run_cpd(program: &Path, root: &Path, files: &[&str], language: &str, warn: u32) -> Result<String, String> {
    if files.is_empty() {
        return Ok(String::new());
    }
    let list = std::env::temp_dir().join(format!("dupes-{}-{language}.txt", std::process::id()));
    let paths: Vec<String> = files.iter().map(|f| root.join(f).display().to_string()).collect();
    std::fs::write(&list, paths.join("\n")).map_err(|e| format!("dupes: {}: {e}", list.display()))?;
    let mut cmd = Command::new(program);
    cmd.args(["cpd", "--language", language, "--minimum-tokens", &warn.to_string(), "--format", "csv", "--file-list"]).arg(&list);
    if language == "cs" {
        cmd.arg("--ignore-usings");
    }
    let out = cmd.output();
    let _ = std::fs::remove_file(&list);
    let out = out.map_err(|e| format!("dupes: {}: {e}", program.display()))?;
    // CPD exits 4 when it found duplicates.
    match out.status.code() {
        Some(0 | 4) => Ok(String::from_utf8_lossy(&out.stdout).into_owned()),
        _ => Err(format!("dupes: CPD failed ({}): {}", out.status, String::from_utf8_lossy(&out.stderr).lines().take(5).collect::<Vec<_>>().join("\n"))),
    }
}

/// The report's lines for stdout and the summary for stderr, and the exit status; `Err` is an exit 2 and its words.
pub fn dupes(o: &Options) -> Result<(String, String, i32), String> {
    let program = cpd_program();
    if !program.is_file() {
        return Err(format!("dupes: PMD not found at {}; install it and set EMUSEN_CPD to its pmd program", program.display()));
    }
    let root = match &o.repo {
        Some(r) => PathBuf::from(r),
        None => match std::env::var_os("EMUSEN_REPO") {
            Some(r) => PathBuf::from(r),
            None => PathBuf::from(String::from_utf8_lossy(&git(Path::new("."), &["rev-parse", "--show-toplevel"])?).trim()),
        },
    };
    let ignore = Ignore::parse(&std::fs::read_to_string(root.join("dupes.ignore")).unwrap_or_default());
    let accept = parse_accept(&std::fs::read_to_string(root.join("dupes.accept")).unwrap_or_default())?;
    let added = match &o.base {
        Some(base) => Some(parse_diff(&String::from_utf8_lossy(&git(&root, &["-c", "core.quotepath=false", "diff", "-U0", "--no-color", "--no-ext-diff", &format!("{base}..HEAD")])?))),
        None => None,
    };
    let listing = git(&root, &["-c", "core.quotepath=false", "ls-files", "-z"])?;
    let tracked: Vec<&str> = std::str::from_utf8(&listing).map_err(|_| "dupes: a tracked file name is not UTF-8".to_string())?.split('\0').filter(|f| !f.is_empty() && !ignore.matches(f)).collect();
    let root_text = root.display().to_string();
    let mut blocks: Vec<Block> = Vec::new();
    for (wanted, ext, language) in [(o.rust, ".rs", "rust"), (o.cs, ".cs", "cs")] {
        if wanted {
            let files: Vec<&str> = tracked.iter().copied().filter(|f| f.ends_with(ext)).collect();
            blocks.extend(parse_csv(&run_cpd(&program, &root, &files, language, o.warn)?, &root_text));
        }
    }
    let report = evaluate(&blocks, added.as_ref(), &accept, o.warn, o.min);
    let mut out = String::new();
    for f in &report.failures {
        out.push_str(&f.line());
        out.push('\n');
    }
    for f in &report.warnings {
        out.push_str(&format!("warning: {}\n", f.line()));
    }
    let scope = o.base.as_deref().map_or("the whole tree".to_string(), |b| format!("the lines added since {b}"));
    let summary = format!("dupes: {} at or over {} tokens, {} warnings from {}, in {scope}\n", report.failures.len(), o.min, report.warnings.len(), o.warn);
    Ok((out, summary, report.status()))
}

impl Module for DevCommands {
    fn manifest(&self) -> &'static str {
        static MANIFEST: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        MANIFEST.get_or_init(|| Manifest { id: "emusen.dev", version: env!("CARGO_PKG_VERSION"), services: &[], commands: COMMANDS, aliases: &[], jobs: &[] }.json())
    }

    fn run(&self, call: &Call) -> i32 {
        let answer = match call.command().as_str() {
            "dupes" => match options(&call.args()).and_then(|o| dupes(&o)) {
                Ok((out, summary, status)) => Answer { out, err: summary, status },
                Err(words) => Answer { out: String::new(), err: words, status: 2 },
            },
            other => Answer::fault(format!("emusen.dev has no command '{other}'")),
        };
        call.answer(answer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_string).collect()
    }

    #[test]
    fn a_command_line_is_the_options_or_the_usage() {
        let o = options(&args("dupes")).unwrap();
        assert_eq!((o.base, o.min, o.warn, o.rust, o.cs), (None, 100, 75, true, true));
        let o = options(&args("dupes --base WiseMan~1 --min-tokens 120 --warn-tokens 90 --lang rust")).unwrap();
        assert_eq!((o.base.as_deref(), o.min, o.warn, o.rust, o.cs), (Some("WiseMan~1"), 120, 90, true, false));
        assert!(options(&args("dupes --lang cobol")).unwrap_err().contains("cs, rust or all"));
        assert!(options(&args("dupes --min-tokens 0")).unwrap_err().contains("above 0"));
        assert!(options(&args("dupes --base")).unwrap_err().contains("needs a value"));
        assert!(options(&args("dupes --warn-tokens 200")).unwrap_err().contains("above --min-tokens"));
        assert!(options(&args("dupes -x")).unwrap_err().contains("not an option"));
    }

    #[test]
    fn a_missing_cpd_is_exit_2_with_a_clear_message() {
        // SAFETY: the only test that sets the variable.
        unsafe { std::env::set_var("EMUSEN_CPD", "/nonexistent/pmd") };
        let words = dupes(&options(&args("dupes")).unwrap()).unwrap_err();
        assert!(words.contains("PMD not found at /nonexistent/pmd") && words.contains("EMUSEN_CPD"), "{words}");
    }

    #[test]
    fn the_manifest_names_dupes() {
        let m = DevCommands.manifest();
        assert!(m.contains("\"id\":\"emusen.dev\"") && m.contains("\"name\":\"dupes\"") && m.contains("man/dupes.1"), "{m}");
    }
}
