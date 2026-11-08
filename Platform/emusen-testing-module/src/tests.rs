//! The studio's rules held on a small repository built in a temporary folder: a workspace, a core built into a C#
//! host by its part of xtask/libraries.tsv, a test project, a recording, and the inputs and tiers files. See EmuSen_Testing.md §7.

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
    { "path": "^Cores/dep/", "kind": "folder", "select": ["covered:Host/*.cs"], "since": "2026-11-02", "reason": "the host's files name dep's exports" },
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
    write(&dir, "Cores/core/Cargo.toml", "[package]\nname = \"core\"\n\n[dependencies]\ndep = { path = \"../dep\" }\n");
    write(&dir, "Cores/core/src/lib.rs", "pub fn c() {}\n");
    write(&dir, "Cores/dep/Cargo.toml", "[package]\nname = \"dep\"\n");
    write(&dir, "Cores/dep/src/lib.rs", "pub fn d() {}\n");
    write(&dir, "Cores/core/Glue.cs", "namespace Core { public class Glue {} }\n");
    write(&dir, "xtask/libraries.tsv", "# the list\ncore\tCores/core\tcore\t1\tships\tany\t-\tCore\tthe console\nplatform\tPlatform\tplatform\t-\tships\tany\t-\tthe platform\tthe platform\n");
    write(&dir, "Host/Host.csproj", "<Project><PropertyGroup><EmuSenNativePart>cores</EmuSenNativePart></PropertyGroup></Project>\n");
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
    pick_with(f, files, None)
}

fn pick_with(f: &Fixture, files: &[&str], map: Option<&crate::coverage::Map>) -> Vec<Pick> {
    let snap = Snapshot::read(&f.repo, "HEAD").unwrap();
    let t = tier(&snap);
    impact::impact(&f.repo, &snap, &Ask { files: files.iter().map(|s| s.to_string()).collect(), tier: &t, failed_last: Vec::new(), map, also: Vec::new(), added: Default::default() }).picks
}

#[test]
fn a_new_file_selects_through_the_changed_files_that_name_it_and_else_falls_to_the_graph() {
    let f = fixture("newfile");
    let (s, path) = store("newfile");
    crate::coverage::load(&s, RUN, false).unwrap();
    s.run("DELETE FROM coverage WHERE file = '*'", &[]).unwrap();
    let m = crate::coverage::Map::load(&s).unwrap().unwrap();
    write(&f.dir, "Host/Newly.cs", "namespace Host { public class Newly {} }\n");
    write(&f.dir, "Host/Thing.cs", "namespace Host { public class Thing { Newly n = new Newly(); } }\n");
    write(&f.dir, "Host/Lonely.cs", "namespace Host { public class Lonely {} }\n");
    git(&f.dir, &["add", "-A"]);
    git(&f.dir, &["commit", "-q", "-m", "a new file named by Thing, and one nothing names"]);
    let snap = Snapshot::read(&f.repo, "HEAD").unwrap();
    let t = tier(&snap);
    let ask = |files: &[&str], added: &[&str]| impact::impact(&f.repo, &snap, &Ask { files: files.iter().map(|s| s.to_string()).collect(), tier: &t, failed_last: Vec::new(), map: Some(&m), also: Vec::new(), added: added.iter().map(|s| s.to_string()).collect() });
    assert_eq!(ids(&ask(&["Host/Newly.cs", "Host/Thing.cs"], &["Host/Newly.cs"]).picks), vec!["dotnet:Wise.FooTests covered"], "Newly runs with the classes that executed Thing.cs, which names it");
    assert_eq!(ids(&ask(&["Host/Lonely.cs", "Host/Thing.cs"], &["Host/Lonely.cs"]).picks), vec!["dotnet:Wise.BarTests graph", "dotnet:Wise.FooTests graph"], "a new file nothing names falls to the graph, the project whole");
    let _ = std::fs::remove_file(path);
}

#[test]
fn a_recording_selects_its_view_group_and_a_toolkit_file_its_classes_or_the_tier() {
    let f = fixture("gaps");
    let (s, path) = store("gaps");
    crate::coverage::load(&s, RUN, false).unwrap();
    s.run("DELETE FROM coverage WHERE file = '*'", &[]).unwrap();
    let by_case = RUN.replace("covered\tWise.FooTests\t", "covered\tWise.FooTests#Main\t").replace("ran\tWise.FooTests\t", "ran\tWise.FooTests#Main\t").replace("none\tWise.BarTests\tno report (exit 1; no result line)\n", "");
    crate::coverage::load(&s, &by_case, true).unwrap();
    let m = crate::coverage::Map::load(&s).unwrap().unwrap();
    assert_eq!(m.group_of("Wise.FooTests", "Wise/Spec/Main@1280x800.spec").as_deref(), None, "the fixture's recording name is not keyed");
    assert_eq!(m.group_of("Wise.FooTests", "Wise/Spec/Main-1280x800.spec").as_deref(), Some("Wise.FooTests#Main"));
    assert_eq!(m.group_of("Wise.FooTests", "Wise/Spec/Other-1280x800.spec"), None, "a view the map has no group for runs the class whole");
    write(&f.dir, "Wise/Spec/Main-1280x800.spec", "window Main\n");
    git(&f.dir, &["add", "-A"]);
    git(&f.dir, &["commit", "-q", "-m", "a keyed recording"]);
    assert_eq!(ids(&pick_with(&f, &["Wise/Spec/Main-1280x800.spec"], Some(&m))), vec!["dotnet:Wise.FooTests#Main recording"], "the recording's own group, not the class");
    assert_eq!(ids(&pick_with(&f, &["Wise/Spec/Main@1280x800.spec"], Some(&m))), vec!["dotnet:Wise.FooTests recording"], "no group for it: the class whole");
    assert_eq!(ids(&pick_with(&f, &["lunap:src/Kit/Button.cs"], Some(&m))), vec!["dotnet:Wise.FooTests#Main covered"], "a toolkit file the map saw selects the units that executed it");
    let whole = pick_with(&f, &["lunap:src/Kit/Unseen.cs"], Some(&m));
    assert!(whole.len() >= 5 && whole.iter().all(|p| p.reason == "closed"), "a toolkit file the map never saw closes to the tier: {:?}", ids(&whole));
    let _ = std::fs::remove_file(path);
}

#[test]
fn machine_units_come_from_the_map_or_a_classs_own_sources() {
    let f = fixture("machine");
    let (s, path) = store("machine");
    write(&f.dir, "Wise/Pads.cs", "namespace Wise\n{\n    public class Pads\n    {\n        public static void Make() { var p = new UinputPad(); }\n    }\n}\n");
    write(&f.dir, "Wise/BarTests.cs", "namespace Wise\n{\n    public class BarTests\n    {\n        [Fact]\n        public void Holds() { new Host.Thing(); Pads.Make(); }\n    }\n}\n");
    git(&f.dir, &["add", "-A"]);
    git(&f.dir, &["commit", "-q", "-m", "a pad fixture"]);
    let snap = Snapshot::read(&f.repo, "HEAD").unwrap();
    let files = crate::shard::machine_files(&f.repo, &snap);
    assert_eq!(files.get("Wise/Pads.cs").map(String::as_str), Some("uinput"));
    assert!(!files.contains_key("Wise/BarTests.cs"), "the test's own file names no pattern");
    crate::coverage::load(&s, &RUN.replace("covered\tWise.FooTests\tHost/Thing.cs\n", "covered\tWise.FooTests\tHost/Thing.cs\ncovered\tWise.FooTests\tWise/Pads.cs\n"), false).unwrap();
    let m = crate::coverage::Map::load(&s).unwrap().unwrap();
    let units = crate::shard::machine_units(&snap, &files, Some(&m));
    assert!(units.contains_key("Wise.FooTests"), "by the map: it executed the pad fixture");
    assert!(units.contains_key("Wise.BarTests") == false, "mapped with no report, and its own sources are clean: {units:?}");
    let without = crate::shard::machine_units(&snap, &files, None);
    assert!(without.is_empty(), "no map and no class names a pattern in its own files: {without:?}");
    let _ = std::fs::remove_file(path);
}

#[test]
fn shards_are_dealt_by_time_with_a_collection_kept_together_unless_spread() {
    use crate::shard::deal;
    let units = vec![("A".to_string(), 100.0, Some("globals".to_string())), ("B".to_string(), 90.0, Some("globals".to_string())), ("C".to_string(), 80.0, None), ("D".to_string(), 10.0, None), ("E".to_string(), 5.0, Some("mars".to_string()))];
    let together = deal(&units, 3, false);
    let find = |shards: &Vec<crate::shard::Shard>, u: &str| shards.iter().position(|s| s.units.iter().any(|x| x == u)).unwrap();
    assert_eq!(find(&together, "A"), find(&together, "B"), "a collection stays in one shard");
    assert_eq!(together[0].units, vec!["A", "B"], "the heaviest item first");
    assert_eq!(together.iter().map(|s| s.seconds as i64).fold(0, i64::max), 190);
    let spread = deal(&units, 3, true);
    assert_ne!(find(&spread, "A"), find(&spread, "B"), "spread, the collection's classes go their own ways");
    assert_eq!(spread.iter().map(|s| s.seconds as i64).fold(0, i64::max), 100, "the wall is the longest unit");
    assert_eq!(deal(&units, 1, true).len(), 1);
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
    assert_eq!(ids(&p), vec!["cargo:Cores/core closed", "cargo:Cores/dep closed", "cargo:Platform/a closed", "cargo:Platform/b closed", "dotnet:Wise.BarTests closed", "dotnet:Wise.FooTests closed"]);
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

const RUN: &str = "# coverage 1\trev HEAD\tbuilt 2026-11-02T20:00:00Z\tjobs 2\tclasses 2\ncovered\tWise.FooTests\tHost/Thing.cs\ncovered\tWise.FooTests\tCores/core/Glue.cs\ncovered\tWise.FooTests\tlunap:src/Kit/Button.cs\ncovered\tWise.FooTests\tHost/Notes.fs\nran\tWise.FooTests\t12.5\tPassed!  - Failed: 0, Passed: 1\t4 files\nnone\tWise.BarTests\tno report (exit 1; no result line)\n";

#[test]
fn a_covered_file_selects_the_classes_that_executed_it_and_those_without_a_report() {
    let f = fixture("covered");
    let (s, path) = store("covered");
    crate::coverage::load(&s, RUN, false).unwrap();
    let m = crate::coverage::Map::load(&s).unwrap().unwrap();
    assert_eq!((m.classes, m.unknown.clone(), m.files.len()), (2, vec!["Wise.BarTests".to_string()], 4));
    assert_eq!(ids(&pick_with(&f, &["Host/Thing.cs"], Some(&m))), vec!["dotnet:Wise.BarTests covered", "dotnet:Wise.FooTests covered"], "BarTests has no report, so every covered change selects it");
    s.run("DELETE FROM coverage WHERE file = '*'", &[]).unwrap();
    let m = crate::coverage::Map::load(&s).unwrap().unwrap();
    assert_eq!(ids(&pick_with(&f, &["Host/Thing.cs"], Some(&m))), vec!["dotnet:Wise.FooTests covered"], "the graph's whole project is not added");
    assert_eq!(ids(&pick_with(&f, &["Cores/core/Glue.cs"], Some(&m))), vec!["cargo:Cores/core graph", "dotnet:Wise.FooTests covered"], "the folder line's classes step aside, its crates stay");
    assert_eq!(ids(&pick_with(&f, &["Host/Other.cs"], Some(&m))), vec!["dotnet:Wise.BarTests graph", "dotnet:Wise.FooTests graph"], "a file the map never saw falls to the graph");
    assert_eq!(ids(&pick_with(&f, &["Wise/BarTests.cs"], Some(&m))), vec!["dotnet:Wise.BarTests always"]);
    let _ = std::fs::remove_file(path);
}

#[test]
fn a_covered_entry_names_the_classes_that_executed_a_crates_surface_and_reached_lines_narrow_only_when_every_entry_has_one() {
    let f = fixture("surface");
    let (s, path) = store("surface");
    assert_eq!(ids(&pick(&f, &["Cores/dep/src/lib.rs"])), vec!["cargo:Cores/core graph", "cargo:Cores/dep graph", "dotnet:Wise.BarTests declared"], "without a map dep's covered: entry adds nothing; core, the one crate through which dep enters C#, has a line, so the project is narrowed to it");
    crate::coverage::load(&s, RUN, false).unwrap();
    s.run("DELETE FROM coverage WHERE file = '*'", &[]).unwrap();
    let m = crate::coverage::Map::load(&s).unwrap().unwrap();
    assert_eq!(ids(&pick_with(&f, &["Cores/dep/src/lib.rs"], Some(&m))), vec!["cargo:Cores/core graph", "cargo:Cores/dep graph", "dotnet:Wise.BarTests declared", "dotnet:Wise.FooTests declared"], "with a map, the class that executed Host/Thing.cs");
    write(&f.dir, "Testing/inputs.json", &INPUTS.replace("    { \"path\": \"^Cores/core/\", \"kind\": \"folder\", \"select\": [\"BarTests\"], \"since\": \"2026-11-01\", \"reason\": \"the core's C# class\" },\n", ""));
    git(&f.dir, &["commit", "-q", "-am", "core's line removed"]);
    assert_eq!(ids(&pick(&f, &["Cores/dep/src/lib.rs"])), vec!["cargo:Cores/core graph", "cargo:Cores/dep graph", "dotnet:Wise.BarTests graph", "dotnet:Wise.FooTests graph"], "an entry crate with no line leaves the project whole");
    let _ = std::fs::remove_file(path);
}

#[test]
fn a_group_of_cases_is_a_unit_of_its_class_with_a_display_name_filter_and_merges_into_the_map() {
    use crate::adapters::unit_term;
    assert_eq!(unit_term("W.FitTests"), "FullyQualifiedName~W.FitTests");
    assert_eq!(unit_term("W.FitTests#PauseCard"), "(FullyQualifiedName~W.FitTests.&(DisplayName~\"PauseCard\"|DisplayName~\"PauseCard-))");
    assert_eq!(unit_term("W.FitTests#"), "(FullyQualifiedName~W.FitTests.&DisplayName!~\")");
    assert_eq!(crate::coverage::case_key("W.FitTests.Holds(file: \"PauseCard-1280x800.spec\")").as_deref(), Some("PauseCard"));
    assert_eq!(crate::coverage::case_key("W.FitTests.Holds(view: \"HomeCardGame\", under: \"white\")").as_deref(), Some("HomeCardGame"));
    assert_eq!(crate::coverage::case_key("W.FitTests.Holds(width: 1280)"), None);
    let f = fixture("groups");
    let snap = Snapshot::read(&f.repo, "HEAD").unwrap();
    assert_eq!(snap.class_of_unit("Wise.FooTests#HomeCard").map(|c| c.fqn.as_str()), Some("Wise.FooTests"));
    assert!(snap.class_of_unit("Wise.Nothing#HomeCard").is_none());
    let (s, path) = store("groups");
    crate::coverage::load(&s, RUN, false).unwrap();
    let by_case = RUN.replace("covered\tWise.FooTests\tHost/Thing.cs\n", "covered\tWise.FooTests#HomeCard\tHost/Thing.cs\ncovered\tWise.FooTests#\tHost/Thing.cs\n").replace("covered\tWise.FooTests\t", "covered\tWise.FooTests#HomeCard\t").replace("ran\tWise.FooTests\t", "ran\tWise.FooTests#HomeCard\t").replace("none\tWise.BarTests\tno report (exit 1; no result line)\n", "");
    assert!(crate::coverage::load(&s, &by_case.replace("rev HEAD", "rev 0123456789abcdef0123456789abcdef01234567"), true).is_err(), "a merge keeps one revision");
    crate::coverage::load(&s, &by_case, true).unwrap();
    let m = crate::coverage::Map::load(&s).unwrap().unwrap();
    let mut groups = m.classes_of("Host/Thing.cs").cloned().unwrap_or_default();
    groups.sort();
    assert_eq!(groups, vec!["Wise.FooTests#".to_string(), "Wise.FooTests#HomeCard".to_string()], "the class's whole rows gave way to its groups");
    assert_eq!(m.unknown, vec!["Wise.BarTests".to_string()], "the other class's rows stayed");
    assert_eq!(ids(&pick_with(&f, &["Host/Thing.cs"], Some(&m))), vec!["dotnet:Wise.BarTests covered", "dotnet:Wise.FooTests# covered", "dotnet:Wise.FooTests#HomeCard covered"]);
    assert_eq!(ids(&pick_with(&f, &["Wise/FooTests.cs", "Host/Thing.cs"], Some(&m))), vec!["dotnet:Wise.BarTests covered", "dotnet:Wise.FooTests always"], "the class's own file picks it whole, and its groups step aside");
    let _ = std::fs::remove_file(path);
}

#[test]
fn a_report_counts_a_file_executed_only_beyond_the_discovery_baseline() {
    let dir = std::env::temp_dir().join(format!("studio-baseline-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let report = |a: u32, b: u32, c: u32| format!("<coverage>\n  <sources>\n    <source>/t/</source>\n  </sources>\n  <packages>\n    <package name=\"H\">\n      <classes>\n        <class name=\"H.A\" filename=\"A.cs\">\n          <lines>\n            <line number=\"1\" hits=\"{a}\" branch=\"False\" />\n          </lines>\n        </class>\n        <class name=\"H.A/&lt;&gt;c\" filename=\"A.cs\">\n          <lines>\n            <line number=\"1\" hits=\"{b}\" branch=\"False\" />\n          </lines>\n        </class>\n        <class name=\"H.B\" filename=\"B.cs\">\n          <lines>\n            <line number=\"4\" hits=\"{c}\" branch=\"False\" />\n          </lines>\n        </class>\n      </classes>\n    </package>\n  </packages>\n</coverage>\n");
    let (d1, d2, c) = (dir.join("d1.xml"), dir.join("d2.xml"), dir.join("c.xml"));
    std::fs::write(&d1, report(5, 2, 0)).unwrap();
    std::fs::write(&d2, report(5, 3, 1)).unwrap();
    let base = crate::coverage::least(crate::coverage::line_hits(&d1).unwrap(), &crate::coverage::line_hits(&d2).unwrap());
    assert_eq!(base["/t/A.cs"][&("H.A/&lt;&gt;c".to_string(), 1)], 2, "the least of the two runs; a class block is keyed as the report writes it");
    assert!(!base.contains_key("/t/B.cs"), "a line one run did not hit is no floor");
    std::fs::write(&c, report(5, 4, 0)).unwrap();
    assert_eq!(crate::coverage::executed_files(&c, Some(&base)).unwrap().into_iter().collect::<Vec<_>>(), vec!["/t/A.cs"], "the closure's block rose above its floor although the method's did not");
    std::fs::write(&c, report(5, 2, 0)).unwrap();
    assert!(crate::coverage::executed_files(&c, Some(&base)).unwrap().is_empty(), "discovery's own hits select nothing");
    assert_eq!(crate::coverage::executed_files(&c, None).unwrap().len(), 1, "without a baseline every hit counts");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_run_loads_into_the_store_and_replaces_the_previous_map() {
    let (s, path) = store("map");
    assert!(crate::coverage::Map::load(&s).unwrap().is_none());
    assert!(crate::coverage::load(&s, "covered\tA\tx.cs\n", false).is_err(), "a run without its header is refused");
    let first = crate::coverage::load(&s, RUN, false).unwrap();
    assert!(first.contains("2 classes (1 without a report)"), "{first}");
    let again = RUN.replace("Host/Thing.cs", "Host/Else.cs").replace("none\tWise.BarTests", "none\tWise.Other");
    crate::coverage::load(&s, &again, false).unwrap();
    let m = crate::coverage::Map::load(&s).unwrap().unwrap();
    assert_eq!(m.classes_of("Host/Thing.cs"), None, "the previous map is gone");
    assert_eq!(m.classes_of("Host/Else.cs"), Some(&vec!["Wise.FooTests".to_string()]));
    assert_eq!(m.classes_of("Cores/core/Glue.cs"), Some(&vec!["Wise.FooTests".to_string()]), "a row both maps hold belongs to the new one");
    assert_eq!(m.classes_of("Host/Notes.fs"), None, "only .cs files are answered");
    assert_eq!(m.unknown, vec!["Wise.Other".to_string()]);
    let _ = std::fs::remove_file(path);
}

#[test]
fn a_report_gives_its_executed_files_and_their_places() {
    let dir = std::env::temp_dir().join(format!("studio-report-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let xml = "<?xml version=\"1.0\"?>\n<coverage>\n  <sources>\n    <source>/home/t/</source>\n  </sources>\n  <packages>\n    <package name=\"Host\">\n      <classes>\n        <class name=\"Host.A\" filename=\"Projects/Repo/Host/A.cs\">\n          <lines>\n            <line number=\"1\" hits=\"0\" branch=\"False\" />\n            <line number=\"2\" hits=\"12\" branch=\"False\" />\n          </lines>\n        </class>\n        <class name=\"Host.B\" filename=\"Projects/Repo/Host/B.cs\">\n          <lines>\n            <line number=\"1\" hits=\"0\" branch=\"False\" />\n          </lines>\n        </class>\n        <class name=\"Kit.C\" filename=\"Projects/Kit/src/C.cs\">\n          <lines>\n            <line number=\"3\" hits=\"1\" branch=\"False\" />\n          </lines>\n        </class>\n      </classes>\n    </package>\n  </packages>\n</coverage>\n";
    let report = dir.join("coverage.cobertura.xml");
    std::fs::write(&report, xml).unwrap();
    let files: Vec<String> = crate::coverage::executed_files(&report, None).unwrap().into_iter().collect();
    assert_eq!(files, vec!["/home/t/Projects/Kit/src/C.cs", "/home/t/Projects/Repo/Host/A.cs"], "a file with no line hit is not executed");
    let place = |p: &str| crate::coverage::relative(p, "/home/t/Projects/Repo", Some("/home/t/Projects/Kit/"));
    assert_eq!(place(&files[1]).as_deref(), Some("Host/A.cs"));
    assert_eq!(place(&files[0]).as_deref(), Some("lunap:src/C.cs"));
    assert_eq!(place("/home/t/Projects/Repository/x.cs"), None, "a longer name is not the tree");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_stale_map_says_so_and_falls_back() {
    let f = fixture("stale");
    let (s, path) = store("stale");
    assert!(crate::coverage::fresh(&f.repo, &s, "HEAD", "HEAD", 1.0).unwrap_err().contains("no coverage map"));
    let head = f.repo.rev("HEAD").unwrap();
    crate::coverage::load(&s, &RUN.replace("rev HEAD", &format!("rev {head}")), false).unwrap();
    assert!(crate::coverage::fresh(&f.repo, &s, "HEAD", "HEAD", 1.0).unwrap().is_some());
    write(&f.dir, "README.md", "later\n");
    let later = Command::new("git").arg("-C").arg(&f.dir).args(["-c", "user.name=t", "-c", "user.email=t@t", "-c", "commit.gpgsign=false", "commit", "-q", "-am", "three days on"]).env("GIT_COMMITTER_DATE", "2036-01-04T00:00:00Z").env("GIT_AUTHOR_DATE", "2036-01-04T00:00:00Z").output().unwrap();
    assert!(later.status.success());
    let why = crate::coverage::fresh(&f.repo, &s, "HEAD", "HEAD", 1.0).unwrap_err();
    assert!(why.contains("days older than the base"), "{why}");
    assert!(crate::coverage::fresh(&f.repo, &s, "HEAD", "HEAD", 100000.0).unwrap().is_some(), "--map-age widens it");
    crate::coverage::load(&s, &RUN.replace("rev HEAD", "rev 0123456789abcdef0123456789abcdef01234567"), false).unwrap();
    assert!(crate::coverage::fresh(&f.repo, &s, "HEAD", "HEAD", 1.0).unwrap_err().contains("does not have"));
    let _ = std::fs::remove_file(path);
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
    let doc = serde_json::from_str::<serde_json::Value>(&String::from_utf8_lossy(&out.stdout)).unwrap();
    let prefix = format!("{}/", root.display());
    let rel = |p: &str| p.strip_prefix(&prefix).unwrap_or(p).trim_end_matches("/Cargo.toml").to_string();
    let mut theirs = std::collections::BTreeSet::new();
    let Some(serde_json::Value::Array(packages)) = doc.get("packages") else { panic!("no packages") };
    for p in packages {
        let me = rel(p.get("manifest_path").and_then(|v| v.as_str()).unwrap());
        if let Some(serde_json::Value::Array(deps)) = p.get("dependencies") {
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
