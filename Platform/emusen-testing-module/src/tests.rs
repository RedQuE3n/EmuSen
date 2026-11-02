//! The studio's rules held on a small repository built in a temporary folder: a workspace, a core built into a C#
//! host by RustCores.props, a test project, a recording, and the inputs and tiers files. See EmuSen_Testing.md §7.

use crate::catalogue::{self, Snapshot};
use crate::config::{self, Tier};
use crate::impact::{self, Ask, Pick};
use crate::repo::Repo;
use crate::store::{NewTrial, Store, UnitRun};
use std::path::{Path, PathBuf};
use std::process::Command;

struct Fixture {
    dir: PathBuf,
    repo: Repo,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git").arg("-C").arg(dir).args(["-c", "user.name=t", "-c", "user.email=t@t", "-c", "commit.gpgsign=false"]).args(args).output().unwrap();
    assert!(ok.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&ok.stderr));
}

fn write(dir: &Path, path: &str, text: &str) {
    let p = dir.join(path);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

const INPUTS: &str = r#"{ "inputs": 1, "recording_endings": [".spec", ".golden"],
  "lines": [
    { "path": "^Wise/Spec/", "kind": "recording", "select": ["FooTests"], "since": "2026-11-01", "reason": "the fit audit reads them" },
    { "path": "^Cores/core/", "kind": "folder", "select": ["BarTests"], "since": "2026-11-01", "reason": "the core's C# class" },
    { "path": "\\.md$", "kind": "none", "select": [], "since": "2026-11-01", "reason": "documentation" } ] }"#;

const TIERS: &str = r#"{ "tiers": 1, "land": { "exclude": ["nightly"], "profile": "test-fast", "reuse": true, "reuse_days": 7 } }"#;

fn fixture(name: &str) -> Fixture {
    let dir = std::env::temp_dir().join(format!("studio-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    write(&dir, "Platform/Cargo.toml", "[workspace]\nmembers = [\"a\", \"b\"]\n");
    write(&dir, "Platform/Cargo.lock", "# lock\n");
    write(&dir, "Platform/a/Cargo.toml", "[package]\nname = \"a\"\n");
    write(&dir, "Platform/a/src/lib.rs", "pub fn a() {}\n");
    write(&dir, "Platform/b/Cargo.toml", "[package]\nname = \"b\"\n\n[dependencies]\na = { path = \"../a\" }\n");
    write(&dir, "Platform/b/src/lib.rs", "pub fn b() {}\n");
    write(&dir, "Cores/core/Cargo.toml", "[package]\nname = \"core\"\n");
    write(&dir, "Cores/core/src/lib.rs", "pub fn c() {}\n");
    write(&dir, "Host/RustCores.props", "<Project><ItemGroup><EmuSenRustCrate Include=\"core\"><Dir>$(MSBuildThisFileDirectory)..\\Cores\\core\\</Dir></EmuSenRustCrate></ItemGroup></Project>\n");
    write(&dir, "Host/Host.csproj", "<Project><Import Project=\"RustCores.props\" /></Project>\n");
    write(&dir, "Host/Thing.cs", "namespace Host { public class Thing {} }\n");
    write(&dir, "Wise/Wise.csproj", "<Project><ItemGroup><PackageReference Include=\"xunit\" /><ProjectReference Include=\"..\\Host\\Host.csproj\" /></ItemGroup></Project>\n");
    write(&dir, "Wise/FooTests.cs", "namespace Wise\n{\n    public class FooTests\n    {\n        [Fact]\n        public void Holds() {}\n    }\n}\n");
    write(&dir, "Wise/BarTests.cs", "namespace Wise\n{\n    public class BarTests\n    {\n        [Fact]\n        public void Holds() { new Host.Thing(); }\n    }\n}\n");
    write(&dir, "Wise/Spec/Main@1280x800.spec", "window Main\n");
    write(&dir, "Testing/inputs.json", INPUTS);
    write(&dir, "Testing/tiers.json", TIERS);
    write(&dir, "README.md", "the fixture\n");
    write(&dir, "tools/odd.sh", "echo\n");
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    let repo = Repo { root: dir.clone() };
    Fixture { dir, repo }
}

fn tier(snap: &Snapshot) -> Tier {
    config::parse_tier(snap.tiers_text.as_deref().unwrap(), "land").unwrap()
}

fn pick(f: &Fixture, files: &[&str]) -> Vec<Pick> {
    let snap = Snapshot::read(&f.repo, "HEAD").unwrap();
    let t = tier(&snap);
    impact::impact(&f.repo, &snap, &Ask { files: files.iter().map(|s| s.to_string()).collect(), tier: &t, failed_last: Vec::new() }).picks
}

fn ids(picks: &[Pick]) -> Vec<String> {
    let mut v: Vec<String> = picks.iter().map(|p| format!("{} {}", p.id(), p.reason)).collect();
    v.sort();
    v
}

#[test]
fn a_recording_selects_its_owner() {
    let f = fixture("recording");
    assert_eq!(ids(&pick(&f, &["Wise/Spec/Main@1280x800.spec"])), vec!["dotnet:Wise.FooTests recording"]);
}

#[test]
fn a_recording_no_line_owns_selects_the_tier() {
    let f = fixture("orphan");
    let p = pick(&f, &["Elsewhere/Main.golden"]);
    assert!(p.len() >= 5 && p.iter().all(|p| p.reason == "closed"), "{:?}", ids(&p));
}

#[test]
fn an_unknown_file_selects_the_tier_and_a_doc_selects_nothing() {
    let f = fixture("unknown");
    let p = pick(&f, &["tools/odd.sh"]);
    assert_eq!(ids(&p), vec!["cargo:Cores/core closed", "cargo:Platform/a closed", "cargo:Platform/b closed", "dotnet:Wise.BarTests closed", "dotnet:Wise.FooTests closed"]);
    assert!(pick(&f, &["README.md"]).is_empty());
}

#[test]
fn a_core_reaches_its_class_through_the_host_and_a_crate_its_dependents() {
    let f = fixture("graph");
    assert_eq!(ids(&pick(&f, &["Cores/core/src/lib.rs"])), vec!["cargo:Cores/core graph", "dotnet:Wise.BarTests declared"]);
    assert_eq!(ids(&pick(&f, &["Platform/a/src/lib.rs"])), vec!["cargo:Platform/a graph", "cargo:Platform/b graph"]);
    assert_eq!(ids(&pick(&f, &["Platform/b/src/lib.rs"])), vec!["cargo:Platform/b graph"]);
    assert_eq!(ids(&pick(&f, &["Platform/Cargo.lock"])), vec!["cargo:Platform graph", "cargo:Platform/a graph", "cargo:Platform/b graph"]);
}

#[test]
fn a_class_file_selects_its_class_and_a_host_file_with_no_line_its_whole_project() {
    let f = fixture("own");
    assert_eq!(ids(&pick(&f, &["Wise/BarTests.cs"])), vec!["dotnet:Wise.BarTests always"]);
    assert_eq!(ids(&pick(&f, &["Host/Thing.cs"])), vec!["dotnet:Wise.BarTests graph", "dotnet:Wise.FooTests graph"]);
}

#[test]
fn a_selection_line_reads_back() {
    let p = Pick { kind: "cargo".into(), unit: "EmuSen/Cores/Sony/Sapphire - PlayStation 2".into(), reason: "graph".into(), via: "a b.rs".into() };
    assert_eq!(Pick::parse(&p.line()), Some(p));
    assert_eq!(Pick::parse("dotnet   W.FooTests   always"), Some(Pick { kind: "dotnet".into(), unit: "W.FooTests".into(), reason: "always".into(), via: String::new() }));
    assert_eq!(Pick::parse("summary\tagree 3"), None);
}

#[test]
fn a_changed_input_changes_the_key_and_nothing_else_does() {
    let f = fixture("key");
    let key = |unit: &str| {
        let snap = Snapshot::read(&f.repo, "HEAD").unwrap();
        catalogue::key(&f.repo, &snap, unit, "test-fast", true).unwrap().map(|k| k.key)
    };
    let (b0, core0, foo0) = (key("cargo:Platform/b"), key("cargo:Cores/core"), key("dotnet:Wise.FooTests"));
    assert!(b0.is_some() && core0.is_some() && foo0.is_some());
    write(&f.dir, "Platform/a/src/lib.rs", "pub fn a() { let _ = 1; }\n");
    assert_eq!(key("cargo:Platform/b"), None, "an uncommitted input gives no key");
    git(&f.dir, &["commit", "-q", "-am", "change a"]);
    assert_ne!(key("cargo:Platform/b"), b0, "b depends on a");
    assert_eq!(key("cargo:Cores/core"), core0);
    write(&f.dir, "Cores/core/src/lib.rs", "pub fn c() { let _ = 2; }\n");
    git(&f.dir, &["commit", "-q", "-am", "change the core"]);
    assert_ne!(key("dotnet:Wise.FooTests"), foo0, "the host builds the core in");
    assert_eq!(catalogue::key(&f.repo, &Snapshot::read(&f.repo, "HEAD").unwrap(), "cargo:Platform/b", "release", true).unwrap().map(|k| k.key) == key("cargo:Platform/b"), false, "the profile is an input");
}

fn store(name: &str) -> (Store, PathBuf) {
    let path = std::env::temp_dir().join(format!("studio-{name}-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    (Store::open_at(path.clone()).unwrap(), path)
}

fn green(s: &Store, unit: &str, key: &str, outcome: &str, flaky: i64) -> i64 {
    let id = s.new_trial(&NewTrial::new("abc", "land", "desktop")).unwrap();
    s.selection(id, unit, "graph", "x", Some(key), None).unwrap();
    s.unit_run(id, &UnitRun { unit: unit.into(), outcome: outcome.into(), passed: 3, flaky, ..UnitRun::default() }).unwrap();
    s.finish_trial(id, "GREEN", 1.0, None).unwrap();
    id
}

#[test]
fn a_green_result_is_reused_only_under_its_own_key() {
    let (s, path) = store("reuse");
    assert_eq!(s.version().unwrap(), 1);
    let id = green(&s, "cargo:a", "k1", "ok", 0);
    assert_eq!(s.reusable("cargo:a", "k1", "desktop", 7.0).unwrap().map(|r| r.0), Some(id));
    assert_eq!(s.reusable("cargo:a", "k2", "desktop", 7.0).unwrap(), None, "a changed key is never reused");
    assert_eq!(s.reusable("cargo:a", "k1", "laptop", 7.0).unwrap(), None, "another runner's result is not");
    green(&s, "cargo:b", "k1", "fail", 0);
    assert_eq!(s.reusable("cargo:b", "k1", "desktop", 7.0).unwrap(), None, "a failure is not");
    green(&s, "cargo:c", "k1", "ok", 0);
    green(&s, "cargo:c", "k0", "flaky", 1);
    assert_eq!(s.reusable("cargo:c", "k1", "desktop", 7.0).unwrap(), None, "a unit flaky within 30 days is not");
    drop(s);
    let s = Store::open_at(path.clone()).unwrap();
    assert_eq!(s.version().unwrap(), 1, "reopening applies nothing twice");
    let _ = std::fs::remove_file(path);
}

#[test]
fn a_quarantine_has_an_owner_a_cause_and_an_expiry() {
    let (s, path) = store("quarantine");
    assert!(crate::trial::quarantine_add(&s, "t::racy", "", "", "cause", 14).is_err());
    assert!(crate::trial::quarantine_add(&s, "t::racy", "", "tester", "cause", 15).is_err());
    crate::trial::quarantine_add(&s, "t::racy", "cargo:a", "tester", "a timing race", 14).unwrap();
    assert!(crate::trial::quarantine_add(&s, "t::racy", "cargo:a", "tester", "a timing race", 14).unwrap_err().contains("new cause"));
    assert_eq!(s.quarantined().unwrap().len(), 1);
    s.run("UPDATE quarantine SET until = '2020-01-01T00:00:00Z'", &[]).unwrap();
    assert!(s.quarantined().unwrap().is_empty(), "an expired quarantine covers nothing");
    let _ = std::fs::remove_file(path);
}

#[test]
fn the_graph_agrees_with_cargo_metadata_on_the_platform_workspace() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let out = Command::new("cargo").args(["metadata", "--no-deps", "--format-version", "1", "--offline", "--manifest-path"]).arg(root.join("Platform/Cargo.toml")).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let doc = emusen_dotnet::json::parse(&String::from_utf8_lossy(&out.stdout)).unwrap();
    let prefix = format!("{}/", root.display());
    let rel = |p: &str| p.strip_prefix(&prefix).unwrap_or(p).trim_end_matches("/Cargo.toml").to_string();
    let mut theirs = std::collections::BTreeSet::new();
    let Some(emusen_dotnet::json::Value::Array(packages)) = doc.get("packages") else { panic!("no packages") };
    for p in packages {
        let me = rel(p.get("manifest_path").and_then(|v| v.as_str()).unwrap());
        if let Some(emusen_dotnet::json::Value::Array(deps)) = p.get("dependencies") {
            for d in deps {
                if let Some(path) = d.get("path").and_then(|v| v.as_str()) {
                    theirs.insert((me.clone(), rel(path)));
                }
            }
        }
    }
    let repo = Repo { root: root.to_path_buf() };
    let files = repo.files("HEAD").unwrap();
    let g = crate::graph::Graph::read(&repo, "HEAD", &files).unwrap();
    let mine: std::collections::BTreeSet<(String, String)> = g.deps.iter().filter(|(k, _)| k.starts_with("Platform/") && g.is_crate(k)).flat_map(|(k, v)| v.iter().filter(|d| g.is_crate(d) && d.as_str() != "Platform").map(move |d| (k.clone(), d.clone()))).collect();
    let missing: Vec<_> = theirs.difference(&mine).collect();
    let extra: Vec<_> = mine.iter().filter(|(_, d)| d.starts_with("Platform/")).filter(|e| !theirs.contains(*e)).collect();
    assert!(missing.is_empty() && extra.is_empty(), "missing {missing:?}, extra {extra:?}");
    assert!(theirs.len() > 30, "{} edges", theirs.len());
}
