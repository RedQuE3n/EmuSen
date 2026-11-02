//! The catalogue: every test unit of a revision, derived from its files. A cargo unit is a root's tests (a workspace
//! one unit, its crates sub-units); a dotnet unit is a test class of a test project, found in its sources; `lunap` is
//! the sibling toolkit's suite. Each unit's key hashes its inputs' git objects, the toolchain and the profile, so an
//! unchanged key means an unchanged unit. See EmuSen_Testing.md §3.1.

use crate::config::Inputs;
use crate::graph::Graph;
use crate::repo::{parent, Repo};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;
use std::sync::OnceLock;

#[derive(Clone, Debug, Default)]
pub struct Class {
    pub fqn: String,
    pub project: String,
    pub files: BTreeSet<String>,
    pub nightly: bool,
    pub abstract_: bool,
    pub base: Option<String>,
    pub has_tests: bool,
}

/// A revision as the studio sees it.
pub struct Snapshot {
    pub rev: String,
    pub files: Vec<String>,
    pub graph: Graph,
    pub inputs: Inputs,
    /// Test classes by full name.
    pub classes: BTreeMap<String, Class>,
    pub tiers_text: Option<String>,
}

fn class_line() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^(\s*)(?:(?:public|internal|private|protected|sealed|abstract|static|partial|unsafe|file)\s+)*class\s+([A-Za-z_][A-Za-z0-9_]*)(?:<[^>]*>)?(?:\([^)]*\))?\s*(?::\s*([A-Za-z_][A-Za-z0-9_.]*))?").unwrap())
}

fn test_attribute() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\[\s*(?:[A-Za-z0-9_]*\.)?[A-Za-z0-9_]*(?:Fact|Theory)\b").unwrap())
}

/// The top-level classes one C# file declares, in the file's namespace.
pub fn classes_in(file: &str, text: &str, project: &str) -> Vec<Class> {
    let namespace = text.lines().find_map(|l| l.trim_start().strip_prefix("namespace ").map(|n| n.trim().trim_end_matches(';').trim().to_string()));
    let lines: Vec<&str> = text.lines().collect();
    let decls: Vec<(usize, usize, String, bool, Option<String>)> = lines
        .iter()
        .enumerate()
        .filter_map(|(i, l)| {
            let c = class_line().captures(l)?;
            Some((i, c[1].len(), c[2].to_string(), l.contains("abstract "), c.get(3).map(|b| b.as_str().rsplit('.').next().unwrap_or("").to_string())))
        })
        .collect();
    let Some(top) = decls.iter().map(|d| d.1).min() else { return Vec::new() };
    let tops: Vec<&(usize, usize, String, bool, Option<String>)> = decls.iter().filter(|d| d.1 == top).collect();
    let mut out = Vec::new();
    for (k, (at, indent, name, abstract_, base)) in tops.iter().map(|d| (d.0, d.1, &d.2, d.3, &d.4)).enumerate() {
        let end = tops.get(k + 1).map_or(lines.len(), |n| n.0);
        let has_tests = lines[at..end].iter().any(|l| test_attribute().is_match(l));
        let mut nightly = false;
        let mut j = at;
        while j > 0 {
            j -= 1;
            let l = lines[j].trim();
            if !l.starts_with('[') || lines[j].len() - lines[j].trim_start().len() != indent {
                break;
            }
            nightly |= l.contains("\"Category\"") && l.contains("\"nightly\"");
        }
        let fqn = match &namespace {
            Some(n) => format!("{n}.{name}"),
            None => name.clone(),
        };
        out.push(Class { fqn, project: project.to_string(), files: BTreeSet::from([file.to_string()]), nightly, abstract_, base: base.clone(), has_tests });
    }
    out
}

/// The test classes of the given C# sources: classes with tests of their own, merged across partial files, and
/// classes that inherit an abstract class's tests; an abstract class is no unit.
pub fn test_classes(sources: &[(String, String, String)]) -> BTreeMap<String, Class> {
    let mut all: BTreeMap<String, Class> = BTreeMap::new();
    for (file, text, project) in sources {
        for c in classes_in(file, text, project) {
            let e = all.entry(c.fqn.clone()).or_insert_with(|| Class { fqn: c.fqn.clone(), project: c.project.clone(), ..Class::default() });
            e.files.extend(c.files);
            e.nightly |= c.nightly;
            e.abstract_ |= c.abstract_;
            e.has_tests |= c.has_tests;
            if e.base.is_none() {
                e.base = c.base;
            }
        }
    }
    let abstract_with_tests: BTreeSet<String> = all.values().filter(|c| c.abstract_ && c.has_tests).map(|c| c.fqn.rsplit('.').next().unwrap_or("").to_string()).collect();
    all.into_iter()
        .filter(|(_, c)| !c.abstract_ && (c.has_tests || c.base.as_ref().is_some_and(|b| abstract_with_tests.contains(b))))
        .collect()
}

impl Snapshot {
    pub fn read(repo: &Repo, rev: &str) -> Result<Snapshot, String> {
        let files = repo.files(rev)?;
        let graph = Graph::read(repo, rev, &files)?;
        let config: Vec<String> = ["Testing/inputs.json", "Testing/tiers.json"].iter().map(|s| s.to_string()).collect();
        let mut texts = repo.read_many(rev, &config)?;
        // A revision older than the studio is read with the checkout's configuration.
        for c in &config {
            if !texts.contains_key(c) {
                if let Ok(t) = std::fs::read_to_string(repo.root.join(c)) {
                    texts.insert(c.clone(), t);
                }
            }
        }
        let inputs = match texts.get("Testing/inputs.json") {
            Some(t) => Inputs::parse(t)?,
            None => Inputs::default(),
        };
        let test_projects: Vec<String> = graph.projects.iter().filter(|(_, t)| **t).map(|(p, _)| p.clone()).collect();
        let cs: Vec<(String, String)> = files
            .iter()
            .filter(|f| f.ends_with(".cs"))
            .filter_map(|f| test_projects.iter().find(|p| crate::repo::is_under(f, parent(p))).map(|p| (f.clone(), p.clone())))
            .collect();
        let paths: Vec<String> = cs.iter().map(|(f, _)| f.clone()).collect();
        let sources = repo.read_many(rev, &paths)?;
        let triples: Vec<(String, String, String)> = cs.into_iter().filter_map(|(f, p)| sources.get(&f).map(|t| (f.clone(), t.clone(), p))).collect();
        let classes = test_classes(&triples);
        Ok(Snapshot { rev: rev.to_string(), files, graph, inputs, classes, tiers_text: texts.get("Testing/tiers.json").cloned() })
    }

    /// The test classes a file declares.
    pub fn classes_of_file(&self, file: &str) -> Vec<&Class> {
        self.classes.values().filter(|c| c.files.contains(file)).collect()
    }

    /// The classes whose full name holds a filter term, as `FullyQualifiedName~term` selects them.
    pub fn classes_matching(&self, term: &str) -> Vec<&Class> {
        self.classes.values().filter(|c| c.fqn.contains(term)).collect()
    }

    pub fn has_lunap(&self) -> bool {
        self.files.iter().any(|f| f == "LunaP.props")
    }
}

/// One line of `units`: kind, unit, tier tag and needs.
pub fn unit_lines(snap: &Snapshot, kind: Option<&str>, tier: Option<&str>, crates: bool) -> Vec<String> {
    let mut out = Vec::new();
    if kind.is_none_or(|k| k == "cargo") && tier.is_none_or(|t| t == "land") {
        for r in snap.graph.roots() {
            out.push(format!("cargo\t{}\tland\t-", r.dir));
            if crates && r.virtual_root {
                for m in snap.graph.members_of(&r.dir) {
                    out.push(format!("crate\t{}\tland\t-\tof {}", m.dir, r.dir));
                }
            }
        }
    }
    if kind.is_none_or(|k| k == "dotnet") {
        for c in snap.classes.values() {
            let t = if c.nightly { "nightly" } else { "land" };
            if tier.is_none_or(|w| w == t) {
                out.push(format!("dotnet\t{}\t{t}\t-", c.fqn));
            }
        }
        if snap.has_lunap() && tier.is_none_or(|t| t == "land") {
            out.push("dotnet\tlunap\tland\t-".to_string());
        }
    }
    out
}

/// What a unit's key is made of, one input a line, and the key: the git hash of those lines.
pub struct Key {
    pub key: String,
    pub lines: Vec<String>,
}

fn toolchain(repo: &Repo, dir: &str, dotnet: bool) -> String {
    let (program, arg) = if dotnet { ("dotnet", "--version") } else { ("rustc", "-V") };
    Command::new(program).arg(arg).current_dir(repo.root.join(dir)).output().ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_else(|| format!("{program} not found"))
}

/// The input paths of a unit: the folders of everything it is built from, the root's lock file and declared extras.
pub fn input_paths(snap: &Snapshot, unit: &str) -> Result<Vec<String>, String> {
    let g = &snap.graph;
    let mut paths = BTreeSet::new();
    let mut nodes: Vec<String> = Vec::new();
    if let Some(dir) = unit.strip_prefix("cargo:") {
        let c = g.crates.get(dir).ok_or_else(|| format!("no crate {dir} at {}", snap.rev))?;
        if c.virtual_root {
            nodes.extend(g.members_of(dir).iter().map(|m| m.dir.clone()));
        } else {
            nodes.push(dir.to_string());
        }
        let lock = if c.root.is_empty() { "Cargo.lock".to_string() } else { format!("{}/Cargo.lock", c.root) };
        if snap.files.contains(&lock) {
            paths.insert(lock);
        }
        if g.crates.get(&c.root).is_some_and(|r| r.virtual_root) {
            paths.insert(format!("{}/Cargo.toml", c.root));
        }
    } else if let Some(fqn) = unit.strip_prefix("dotnet:") {
        let project = if fqn == "lunap" { None } else { Some(snap.classes.get(fqn).ok_or_else(|| format!("no test class {fqn} at {}", snap.rev))?.project.clone()) };
        if let Some(p) = project {
            nodes.push(p);
        } else {
            paths.insert("LunaP.props".to_string());
        }
    } else {
        return Err(format!("{unit} is not a cargo or dotnet unit"));
    }
    for n in nodes {
        for d in g.forward_closure(&n) {
            if g.crates.get(&d).is_some_and(|c| c.virtual_root) {
                paths.insert(format!("{d}/Cargo.toml"));
            } else if g.crates.contains_key(&d) {
                paths.insert(d.clone());
            } else if g.projects.contains_key(&d) {
                paths.insert(parent(&d).to_string());
            } else {
                paths.insert(d.clone());
            }
            for (u, extra) in &snap.inputs.extra {
                if u.strip_prefix("cargo:").is_some_and(|x| x == d) {
                    paths.extend(extra.iter().cloned());
                }
            }
        }
    }
    for (u, extra) in &snap.inputs.extra {
        if u == unit {
            paths.extend(extra.iter().cloned());
        }
    }
    paths.extend([".cargo/config.toml".to_string()].into_iter().filter(|p| snap.files.contains(p)));
    Ok(paths.into_iter().collect())
}

/// The key of a unit at the snapshot's revision under a profile; none when an input has uncommitted changes and the
/// revision is the working tree's, since git's objects would not describe what runs.
pub fn key(repo: &Repo, snap: &Snapshot, unit: &str, profile: &str, checkout: bool) -> Result<Option<Key>, String> {
    let paths = input_paths(snap, unit)?;
    if checkout && repo.dirty(&paths) {
        return Ok(None);
    }
    let mut args: Vec<String> = vec!["rev-parse".into()];
    args.extend(paths.iter().map(|p| format!("{}:{p}", snap.rev)));
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let ids = repo.git(&refs)?;
    let dotnet = unit.starts_with("dotnet:");
    let dir = unit.strip_prefix("cargo:").unwrap_or("");
    let mut lines = vec!["key 1".to_string(), format!("unit {unit}"), format!("profile {profile}"), format!("toolchain {}", toolchain(repo, dir, dotnet))];
    for (id, p) in ids.lines().zip(&paths) {
        lines.push(format!("{id} {p}"));
    }
    let user = crate::repo::home().join(".cargo/config.toml");
    if !dotnet {
        if let Ok(text) = std::fs::read_to_string(&user) {
            lines.push(format!("{} ~/.cargo/config.toml", hash_text(&text)?));
        }
    } else if let Some(state) = lunap_state(repo) {
        if state.ends_with("dirty") && checkout {
            return Ok(None);
        }
        lines.push(format!("lunap {state}"));
    }
    let key = hash_text(&lines.join("\n"))?;
    Ok(Some(Key { key, lines }))
}

/// git's object name for a text: the hash the key is.
pub fn hash_text(text: &str) -> Result<String, String> {
    use std::io::Write;
    let mut child = Command::new("git").args(["hash-object", "--stdin"]).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).spawn().map_err(|e| e.to_string())?;
    child.stdin.take().ok_or("no stdin")?.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// The sibling LunaP checkout's commit, and `dirty` when it has changes, as LunaP.props in the working tree names it.
fn lunap_state(repo: &Repo) -> Option<String> {
    let props = std::fs::read_to_string(repo.root.join("LunaP.props")).ok()?;
    let at = props.split('"').find(|s| s.ends_with("EmuSen.LunaP.csproj"))?;
    let path = at.replace("$(MSBuildThisFileDirectory)", &format!("{}/", repo.root.display()));
    let dir = std::path::Path::new(&path).parent()?.to_path_buf();
    let sibling = Repo { root: dir };
    let top = sibling.git(&["rev-parse", "--show-toplevel"]).ok()?;
    let sibling = Repo { root: top.trim().into() };
    let head = sibling.git(&["rev-parse", "HEAD"]).ok()?;
    let dirty = !sibling.git(&["status", "--porcelain", "--untracked-files=no"]).ok()?.trim().is_empty();
    Some(format!("{}{}", head.trim(), if dirty { " dirty" } else { "" }))
}

/// The folder of the sibling LunaP checkout's tests, for the lunap unit.
pub fn lunap_tests(repo: &Repo) -> Option<std::path::PathBuf> {
    let props = std::fs::read_to_string(repo.root.join("LunaP.props")).ok()?;
    let at = props.split('"').find(|s| s.ends_with("EmuSen.LunaP.csproj"))?;
    let path = at.replace("$(MSBuildThisFileDirectory)", &format!("{}/", repo.root.display()));
    let csproj = std::path::PathBuf::from(path);
    let top = csproj.parent()?.parent()?.parent()?;
    let tests = top.join("tests/EmuSen.LunaP.Tests");
    tests.exists().then_some(tests)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classes_are_found_across_partials_and_inherited_contracts() {
        let a = ("W/A.cs".to_string(), "namespace W.Cores\n{\n    [Trait(\"Category\", \"nightly\")]\n    public sealed partial class FitTests\n    {\n        [Fact]\n        public void One() {}\n        private sealed class Helper {}\n    }\n}\n".to_string(), "W/W.csproj".to_string());
        let b = ("W/A.Two.cs".to_string(), "namespace W.Cores\n{\n    public partial class FitTests\n    {\n        [UinputTheory]\n        public void Two(int x) {}\n    }\n}\n".to_string(), "W/W.csproj".to_string());
        let c = ("W/C.cs".to_string(), "namespace W.Cores;\n\npublic abstract class Contract : IDisposable\n{\n    [Fact] public void Holds() {}\n}\n\npublic sealed class ContractOnRt : Contract\n{\n}\n\ninternal static class NotATest\n{\n}\n".to_string(), "W/W.csproj".to_string());
        let found = test_classes(&[a, b, c]);
        assert_eq!(found.keys().cloned().collect::<Vec<_>>(), vec!["W.Cores.ContractOnRt".to_string(), "W.Cores.FitTests".to_string()]);
        assert!(found["W.Cores.FitTests"].nightly);
        assert_eq!(found["W.Cores.FitTests"].files.len(), 2);
        assert!(!found["W.Cores.ContractOnRt"].nightly);
    }
}
