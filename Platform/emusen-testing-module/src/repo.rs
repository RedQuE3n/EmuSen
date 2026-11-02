//! The repository as the studio reads it: changed files between two revisions, a revision's file list, many files'
//! text in one `git cat-file --batch`, and `git grep` at a revision. Everything is read from git objects, so a branch
//! that is not checked out is read as it is. See EmuSen_Testing.md §3.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub struct Repo {
    pub root: PathBuf,
}

impl Repo {
    /// The repository: `--repo`, else `EMUSEN_REPO`, else the one holding the process's working folder.
    pub fn find(given: Option<&str>) -> Result<Repo, String> {
        let root = match given.map(PathBuf::from).or_else(|| std::env::var_os("EMUSEN_REPO").map(PathBuf::from)) {
            Some(r) => r,
            None => {
                let out = Command::new("git").args(["rev-parse", "--show-toplevel"]).output().map_err(|e| format!("git did not start: {e}"))?;
                if !out.status.success() {
                    return Err("no repository: pass --repo DIR or set EMUSEN_REPO".to_string());
                }
                PathBuf::from(String::from_utf8_lossy(&out.stdout).trim())
            }
        };
        if !root.join(".git").exists() {
            return Err(format!("{} is not a git checkout", root.display()));
        }
        Ok(Repo { root })
    }

    pub fn git(&self, args: &[&str]) -> Result<String, String> {
        let out = Command::new("git").arg("-C").arg(&self.root).arg("-c").arg("core.quotepath=false").args(args).output().map_err(|e| format!("git did not start: {e}"))?;
        if !out.status.success() {
            return Err(format!("git {} failed: {}", args.join(" "), String::from_utf8_lossy(&out.stderr).trim()));
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }

    /// The full object name of a revision.
    pub fn rev(&self, rev: &str) -> Result<String, String> {
        Ok(self.git(&["rev-parse", "--verify", &format!("{rev}^{{commit}}")])?.trim().to_string())
    }

    /// The files that differ between the merge base of `base` and `head`, and `head`.
    pub fn changed(&self, base: &str, head: &str) -> Result<Vec<String>, String> {
        let mb = self.git(&["merge-base", base, head])?;
        let out = self.git(&["diff", "--name-only", "-z", "--no-renames", mb.trim(), head])?;
        Ok(out.split('\0').filter(|f| !f.is_empty()).map(str::to_string).collect())
    }

    /// Tracked files changed in the working tree against HEAD, staged or not.
    pub fn uncommitted(&self) -> Result<Vec<String>, String> {
        let out = self.git(&["diff", "--name-only", "-z", "--no-renames", "HEAD"])?;
        Ok(out.split('\0').filter(|f| !f.is_empty()).map(str::to_string).collect())
    }

    /// Every file of a revision.
    pub fn files(&self, rev: &str) -> Result<Vec<String>, String> {
        let out = self.git(&["ls-tree", "-r", "-z", "--name-only", rev])?;
        Ok(out.split('\0').filter(|f| !f.is_empty()).map(str::to_string).collect())
    }

    /// The text of each path at `rev`, read through one `git cat-file --batch`; a missing path is left out.
    pub fn read_many(&self, rev: &str, paths: &[String]) -> Result<HashMap<String, String>, String> {
        let mut child = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(["cat-file", "--batch"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("git cat-file did not start: {e}"))?;
        let mut stdin = child.stdin.take().ok_or("git cat-file has no stdin")?;
        let requests: String = paths.iter().map(|p| format!("{rev}:{p}\n")).collect();
        let writer = std::thread::spawn(move || {
            let _ = stdin.write_all(requests.as_bytes());
        });
        let mut reader = BufReader::new(child.stdout.take().ok_or("git cat-file has no stdout")?);
        let mut found = HashMap::new();
        for p in paths {
            let mut header = String::new();
            if reader.read_line(&mut header).map_err(|e| e.to_string())? == 0 {
                break;
            }
            if header.trim_end().ends_with(" missing") || header.trim_end().ends_with(" ambiguous") {
                continue;
            }
            let size: usize = header.split_whitespace().nth(2).and_then(|s| s.parse().ok()).ok_or_else(|| format!("git cat-file said '{}'", header.trim()))?;
            let mut body = vec![0u8; size + 1];
            reader.read_exact(&mut body).map_err(|e| e.to_string())?;
            body.pop();
            found.insert(p.clone(), String::from_utf8_lossy(&body).into_owned());
        }
        let _ = writer.join();
        let _ = child.wait();
        Ok(found)
    }

    /// Files under `pathspecs` at `rev` whose text matches the extended pattern, as plain paths.
    pub fn grep_files(&self, rev: &str, pattern: &str, pathspecs: &[String], word: bool) -> Vec<String> {
        let mut args: Vec<String> = vec!["grep".into(), "-l".into(), "-E".into()];
        if word {
            args.push("-w".into());
        }
        args.extend([pattern.to_string(), rev.to_string(), "--".to_string()]);
        args.extend(pathspecs.iter().cloned());
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let prefix = format!("{rev}:");
        self.git(&refs).map(|out| out.lines().map(|l| l.strip_prefix(&prefix).unwrap_or(l).to_string()).collect()).unwrap_or_default()
    }

    /// Whether any of the paths has uncommitted changes in the working tree.
    pub fn dirty(&self, paths: &[String]) -> bool {
        if paths.is_empty() {
            return false;
        }
        let mut args = vec!["status", "--porcelain", "--untracked-files=no", "--"];
        args.extend(paths.iter().map(String::as_str));
        self.git(&args).map(|o| o.lines().any(|l| !l.ends_with("LunaP.props"))).unwrap_or(true)
    }
}

/// `a/b/../c` as `a/c`, with `\` read as `/`.
pub fn normalise(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for p in path.split(['/', '\\']) {
        match p {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    parts.join("/")
}

/// The folder of a repository path, empty at the root.
pub fn parent(path: &str) -> &str {
    path.rfind('/').map_or("", |i| &path[..i])
}

pub fn join(dir: &str, rel: &str) -> String {
    if dir.is_empty() { normalise(rel) } else { normalise(&format!("{dir}/{rel}")) }
}

pub fn home() -> PathBuf {
    PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
}

/// Where trials keep their files: `EMUSEN_TESTING_HOME`, else `~/.cache/emusen/testing`.
pub fn testing_home() -> PathBuf {
    std::env::var_os("EMUSEN_TESTING_HOME").map(PathBuf::from).unwrap_or_else(|| home().join(".cache/emusen/testing"))
}

pub fn is_under(path: &str, dir: &str) -> bool {
    dir.is_empty() || path == dir || (path.starts_with(dir) && path.as_bytes().get(dir.len()) == Some(&b'/'))
}
