//! `libraries.tsv`, the one list of the native libraries and programs an installation carries; the export prefix CI counts is CI's column alone.

use crate::host;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Core,
    Part,
    Program,
    Test,
    Platform,
}

#[derive(Clone, Debug)]
pub struct Library {
    pub name: String,
    pub dir: String,
    pub role: Role,
    pub abi_v1: bool,
    pub ships: bool,
    pub linux_only: bool,
    pub core: String,
    pub console: String,
}

impl Library {
    /// Whether a build for the platform `rid` carries it.
    pub fn builds_for(&self, rid: &str) -> bool {
        !self.linux_only || (host::is_linux() && rid.starts_with("linux"))
    }

    pub fn folder(&self, repo: &Path) -> PathBuf {
        repo.join(&self.dir)
    }
}

pub fn path(repo: &Path) -> PathBuf {
    repo.join("xtask").join("libraries.tsv")
}

pub fn read(repo: &Path) -> Result<Vec<Library>, String> {
    let file = path(repo);
    let text = std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
    parse(&text).map_err(|e| format!("{}: {e}", file.display()))
}

pub fn parse(text: &str) -> Result<Vec<Library>, String> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() != 9 {
            return Err(format!("line {} has {} columns, not 9", n + 1, f.len()));
        }
        let role = match f[2] {
            "core" => Role::Core,
            "part" => Role::Part,
            "program" => Role::Program,
            "test" => Role::Test,
            "platform" => Role::Platform,
            other => return Err(format!("line {}: no role {other}", n + 1)),
        };
        let flag = |v: &str, yes: &str, no: &str| match v {
            v if v == yes => Ok(true),
            v if v == no => Ok(false),
            other => Err(format!("line {}: {other} is neither {yes} nor {no}", n + 1)),
        };
        out.push(Library {
            name: f[0].to_string(),
            dir: f[1].to_string(),
            role,
            abi_v1: flag(f[3], "1", "-")?,
            ships: flag(f[4], "ships", "development")?,
            linux_only: flag(f[5], "linux", "any")?,
            core: f[7].to_string(),
            console: f[8].to_string(),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_repository_list_reads_and_names_the_platform_library_once() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let all = read(&repo).unwrap();
        assert_eq!(all.iter().filter(|l| l.role == Role::Platform).count(), 1);
        for l in &all {
            assert!(l.folder(&repo).join("Cargo.toml").is_file(), "{} names {}, which holds no crate", l.name, l.dir);
        }
    }

    #[test]
    fn a_bad_line_is_refused() {
        assert!(parse("a\tb\tcore\t1\tships\tany\t-\tA\n").is_err());
        assert!(parse("a\tb\tengine\t1\tships\tany\t-\tA\tthe A\n").is_err());
        assert!(parse("a\tb\tcore\t2\tships\tany\t-\tA\tthe A\n").is_err());
        let ok = parse("# comment\na\tb\tcore\t1\tdevelopment\tlinux\t-\tA\tthe A\n").unwrap();
        assert!(ok[0].abi_v1 && !ok[0].ships && ok[0].linux_only);
    }
}
