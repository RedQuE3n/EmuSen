//! The coverage map: for each test class, the source files its tests executed. `coverage run` runs every class of the
//! land tier in its own `dotnet test` process under coverlet, N at a time from hardlinked copies of one build, and
//! prints one line per executed file; `coverage load` keeps a run in the store with the revision it was built at;
//! impact's `covered` rule reads the newest map. See EmuSen_Testing.md §3.7 and `man coverage`.

use crate::adapters::{run_logged, scoped};
use crate::catalogue::{lunap_root, Snapshot};
use crate::repo::Repo;
use crate::store::{NewTrial, Store};
use emusen_sqlite::Bound;
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Mutex};
use std::time::Instant;

/// The file a class without a report is recorded under; the rule selects such a class for every covered change.
pub const NO_REPORT: &str = "*";

/// The map as the store holds it: a file to the classes that executed it, and the classes no report describes.
#[derive(Debug)]
pub struct Map {
    pub trial: i64,
    pub rev: String,
    pub built: String,
    pub files: BTreeMap<String, Vec<String>>,
    pub unknown: Vec<String>,
    pub classes: usize,
    /// Every unit the map holds, classes and groups of cases alike.
    pub units: BTreeSet<String>,
}

impl Map {
    /// The newest complete map, if the store has one.
    pub fn load(store: &Store) -> Result<Option<Map>, String> {
        let head = store.rows("SELECT id, tree, finished FROM trial WHERE tier = 'coverage' AND verdict = 'MAP' ORDER BY id DESC LIMIT 1", &[])?;
        let Some(h) = head.first() else { return Ok(None) };
        let trial: i64 = h[0].as_deref().unwrap_or("0").parse().unwrap_or(0);
        let mut m = Map { trial, rev: h[1].clone().unwrap_or_default(), built: h[2].clone().unwrap_or_default(), files: BTreeMap::new(), unknown: Vec::new(), classes: 0, units: BTreeSet::new() };
        let mut classes = BTreeSet::new();
        for r in store.rows("SELECT unit, file FROM coverage WHERE recorded = $t", &[("$t", Bound::Integer(trial))])? {
            let unit = r[0].clone().unwrap_or_default();
            let unit = unit.strip_prefix("dotnet:").unwrap_or(&unit).to_string();
            let file = r[1].clone().unwrap_or_default();
            classes.insert(unit.split('#').next().unwrap_or(&unit).to_string());
            m.units.insert(unit.clone());
            if file == NO_REPORT {
                m.unknown.push(unit);
            } else {
                m.files.entry(file).or_default().push(unit);
            }
        }
        m.classes = classes.len();
        Ok(Some(m))
    }

    /// The classes that executed a source file; none when the map never saw it.
    pub fn classes_of(&self, file: &str) -> Option<&Vec<String>> {
        if file.ends_with(".cs") { self.files.get(file) } else { None }
    }

    /// The group of a class's cases a recording belongs to (`Class#Key`, the key the recording's name gives), when
    /// the class is mapped by case and has that group.
    pub fn group_of(&self, class: &str, recording: &str) -> Option<String> {
        let name = recording.rsplit('/').next().unwrap_or(recording);
        let key = case_key(&format!("(\"{name}\")"))?;
        let unit = format!("{class}#{key}");
        self.units.contains(&unit).then_some(unit)
    }
}

/// The newest map when its revision is at most `days` older than the merge base of `base` and `head`; else why not.
pub fn fresh(repo: &Repo, store: &Store, base: &str, head: &str, days: f64) -> Result<Option<Map>, String> {
    let Some(m) = Map::load(store)? else { return Err("no coverage map in the store; the covered rule is off".into()) };
    let short = m.rev.chars().take(9).collect::<String>();
    let mb = repo.git(&["merge-base", base, head])?;
    let at = |rev: &str| -> Option<i64> { repo.git(&["log", "-1", "--format=%ct", rev]).ok()?.trim().parse().ok() };
    let Some(t_map) = at(&m.rev) else { return Err(format!("the coverage map was built at {short}, a revision this checkout does not have; the covered rule is off")) };
    let t_base = at(mb.trim()).unwrap_or(t_map);
    let older = (t_base - t_map) as f64 / 86400.0;
    if older > days {
        return Err(format!("the coverage map (built at {short}, {}) is {older:.1} days older than the base, more than {days}; the covered rule is off", m.built));
    }
    Ok(Some(m))
}

/// Reads a run's lines into the store as the map, replacing the previous one, or with `merge` only the rows of the
/// classes the run holds (a by-case run of one class merged into a whole map of the same revision); a summary line.
pub fn load(store: &Store, text: &str, merge: bool) -> Result<String, String> {
    let (mut rev, mut built) = (None, None);
    let mut covered: Vec<(String, String)> = Vec::new();
    let mut none: Vec<String> = Vec::new();
    let (mut ran, mut seconds) = (0usize, 0.0f64);
    let mut runs: Vec<(String, f64, bool)> = Vec::new();
    // A process's own cost, from the baseline's two no-test runs, taken off each unit's seconds for `shard`.
    let mut overhead = 0.0f64;
    for l in text.lines() {
        if l.starts_with("# baseline\t") {
            if let Some(s) = l.split('\t').last().and_then(|f| f.trim_end_matches(" s").parse::<f64>().ok()) {
                overhead = s / 2.0;
            }
            continue;
        }
        if let Some(h) = l.strip_prefix("# coverage 1") {
            for field in h.split('\t') {
                if let Some(r) = field.trim().strip_prefix("rev ") {
                    rev = Some(r.trim().to_string());
                } else if let Some(b) = field.trim().strip_prefix("built ") {
                    built = Some(b.trim().to_string());
                }
            }
            continue;
        }
        let p: Vec<&str> = l.split('\t').collect();
        match (p.first().copied(), p.get(1), p.get(2)) {
            (Some("covered"), Some(c), Some(f)) => covered.push((c.to_string(), f.to_string())),
            (Some("none"), Some(c), _) => none.push(c.to_string()),
            (Some("ran"), Some(c), Some(s)) => {
                ran += 1;
                let secs = s.parse::<f64>().unwrap_or(0.0);
                seconds += secs;
                runs.push((c.to_string(), secs, p.get(3).is_some_and(|r| r.contains("Failed!"))));
            }
            _ => {}
        }
    }
    let rev = rev.ok_or("no '# coverage 1' header with its rev")?;
    if covered.is_empty() {
        return Err("no covered lines: nothing to load".into());
    }
    let classes: BTreeSet<&str> = covered.iter().map(|(c, _)| c.as_str()).chain(none.iter().map(String::as_str)).collect();
    let previous = if merge {
        let m = Map::load(store)?.ok_or("--merge needs a map in the store to merge into")?;
        if m.rev != rev {
            return Err(format!("--merge: the run is at {} and the map at {}; a merge keeps one revision", &rev[..9.min(rev.len())], &m.rev[..9.min(m.rev.len())]));
        }
        Some(m)
    } else {
        None
    };
    let mut t = NewTrial::new(&rev, "coverage", "desktop");
    t.started = built.clone().unwrap_or_else(crate::store::now);
    let owners: BTreeSet<&str> = classes.iter().map(|c| c.split('#').next().unwrap_or(c)).collect();
    let note = match &previous {
        Some(p) => format!("coverage map: {} classes merged into the map of {} classes ({} units, {} without a report), {} test-minutes", owners.len(), p.classes, classes.len(), none.len(), (seconds / 60.0).round()),
        None => format!("coverage map: {} classes ({} without a report), {} files, {} test-minutes", classes.len(), none.len(), covered.iter().map(|(_, f)| f.as_str()).collect::<BTreeSet<_>>().len(), (seconds / 60.0).round()),
    };
    let id = store.new_trial(&t)?;
    for c in &classes {
        store.unit(&format!("dotnet:{c}"))?;
    }
    store.exec("BEGIN IMMEDIATE")?;
    let written = (|| -> Result<(), String> {
        if let Some(p) = &previous {
            for c in &owners {
                store.run("DELETE FROM coverage WHERE unit = $u OR unit LIKE $g", &[("$u", Bound::Text(&format!("dotnet:{c}"))), ("$g", Bound::Text(&format!("dotnet:{c}#%")))])?;
            }
            store.run("UPDATE coverage SET recorded = $t WHERE recorded = $p", &[("$t", Bound::Integer(id)), ("$p", Bound::Integer(p.trial))])?;
        } else {
            store.run("DELETE FROM coverage", &[])?;
        }
        for (c, f) in &covered {
            store.run("INSERT OR IGNORE INTO coverage (unit, file, recorded) VALUES ($u, $f, $t)", &[("$u", Bound::Text(&format!("dotnet:{c}"))), ("$f", Bound::Text(f)), ("$t", Bound::Integer(id))])?;
        }
        for c in &none {
            store.run("INSERT OR IGNORE INTO coverage (unit, file, recorded) VALUES ($u, $f, $t)", &[("$u", Bound::Text(&format!("dotnet:{c}"))), ("$f", Bound::Text(NO_REPORT)), ("$t", Bound::Integer(id))])?;
        }
        // Each unit's seconds under coverage, the time `shard` deals by until a land records a better one.
        for (c, secs, failed) in &runs {
            store.unit_run(id, &crate::store::UnitRun { unit: format!("dotnet:{c}"), outcome: if *failed { "fail".into() } else { "ok".into() }, seconds: Some((secs - overhead).max(0.5)), ..Default::default() })?;
        }
        store.finish_trial(id, "MAP", seconds, Some(&note))
    })();
    match written {
        Ok(()) => store.exec("COMMIT")?,
        Err(e) => {
            let _ = store.exec("ROLLBACK");
            return Err(e);
        }
    }
    Ok(format!("loaded\ttrial {id}\trev {rev}\t{note}; {ran} ran\n"))
}

pub struct RunOptions {
    /// The test project's build output, holding its dll and every assembly it loads.
    pub bin: PathBuf,
    /// The tree the build was made in, whose paths the pdbs carry.
    pub tree: PathBuf,
    pub rev: String,
    pub jobs: usize,
    pub nightly: bool,
    pub only: Vec<String>,
    /// Classes mapped one group of cases at a time, by the first quoted argument of each case's display name.
    pub by_case: Vec<String>,
    pub out: PathBuf,
}

/// The first quoted argument of a case's display name, less a trailing size (`-1280x800`) and what follows it.
pub fn case_key(display_name: &str) -> Option<String> {
    let q = regex::Regex::new(r#""([^"]+)""#).unwrap().captures(display_name)?[1].to_string();
    Some(regex::Regex::new(r"^(.*?)-\d+x\d+").unwrap().captures(&q).map_or(q.clone(), |c| c[1].to_string()))
}

const RUNSETTINGS: &str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<RunSettings>\n  <DataCollectionRunSettings>\n    <DataCollectors>\n      <DataCollector friendlyName=\"XPlat Code Coverage\">\n        <Configuration>\n          <Format>cobertura</Format>\n          <Include>[EmuSen*]*</Include>\n          <IncludeTestAssembly>true</IncludeTestAssembly>\n          <SkipAutoProps>true</SkipAutoProps>\n          <UseSourceLink>false</UseSourceLink>\n        </Configuration>\n      </DataCollector>\n    </DataCollectors>\n  </DataCollectionRunSettings>\n</RunSettings>\n";

/// A full copy of the build for one worker: never a hard link, since coverlet rewrites assemblies in place and a build rewrites the rest.
fn mirror(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    for e in std::fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))? {
        let e = e.map_err(|e| e.to_string())?;
        let (p, t) = (e.path(), to.join(e.file_name()));
        if p.is_dir() {
            mirror(&p, &t)?;
        } else {
            std::fs::copy(&p, &t).map_err(|e| format!("{}: {e}", t.display()))?;
        }
    }
    Ok(())
}

/// xunit's runner configuration in a copy: theories enumerated at run time (discovery then evaluates no MemberData,
/// so a theory's data and the static constructors it reaches count for the class that runs it), or at discovery for a
/// class mapped by case, whose groups are filtered by display name.
fn pre_enumeration(copy: &Path, on: bool) -> Result<(), String> {
    let path = copy.join("xunit.runner.json");
    let value = if on { "true" } else { "false" };
    let text = match std::fs::read_to_string(&path) {
        Ok(t) if t.contains("\"preEnumerateTheories\"") => regex::Regex::new(r#""preEnumerateTheories"\s*:\s*(true|false)"#).unwrap().replace(&t, format!("\"preEnumerateTheories\": {value}").as_str()).into_owned(),
        Ok(t) => t.replacen('{', &format!("{{ \"preEnumerateTheories\": {value},"), 1),
        Err(_) => format!("{{ \"preEnumerateTheories\": {value} }}\n"),
    };
    std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))
}

/// The display names of a class's cases, as `dotnet test --list-tests` prints them from a copy.
fn list_cases(copy: &Path, dll: &str, fqn: &str, tree: &Path, adapter: &Path, work: &Path) -> Result<Vec<String>, String> {
    let _ = std::fs::create_dir_all(work);
    let log = work.join("list.log");
    let args: Vec<String> = vec!["test".into(), copy.join(dll).display().to_string(), "--no-build".into(), "--list-tests".into(), "--filter".into(), format!("FullyQualifiedName~{fqn}."), "--test-adapter-path".into(), adapter.display().to_string()];
    run_logged(scoped("dotnet", &args, tree), &log, &AtomicBool::new(false))?;
    Ok(std::fs::read_to_string(&log).unwrap_or_default().lines().filter(|l| l.starts_with("    ")).map(|l| l.trim().to_string()).collect())
}

fn attr(tag: &str, name: &str) -> Option<String> {
    let key = format!("{name}=\"");
    let i = tag.find(&key)? + key.len();
    let j = tag[i..].find('"')?;
    Some(tag[i..i + j].to_string())
}

/// Hit counts by file, then by (class block, line), for the lines hit at all: what discovery alone executes.
pub type Hits = HashMap<String, HashMap<(String, u32), u32>>;

/// Walks a cobertura report's lines: (file as the pdb names it, class block, line number, hits).
fn walk_report(report: &Path, mut each: impl FnMut(&str, &str, u32, u32)) -> Result<(), String> {
    let f = std::fs::File::open(report).map_err(|e| format!("{}: {e}", report.display()))?;
    let (mut source, mut file, mut class) = (String::new(), String::new(), String::new());
    for line in std::io::BufReader::new(f).lines() {
        let l = line.map_err(|e| e.to_string())?;
        let t = l.trim_start();
        if source.is_empty() && t.starts_with("<source>") {
            source = t["<source>".len()..].split("</source>").next().unwrap_or("").to_string();
        } else if t.starts_with("<class ") {
            file = attr(t, "filename").map(|c| if c.starts_with('/') { c } else { format!("{source}{c}") }).unwrap_or_default();
            class = attr(t, "name").unwrap_or_default();
        } else if t.starts_with("<line ") && !file.is_empty() {
            if let (Some(n), Some(h)) = (attr(t, "number").and_then(|n| n.parse().ok()), attr(t, "hits").and_then(|h| h.parse().ok())) {
                each(&file, &class, n, h);
            }
        }
    }
    Ok(())
}

/// The lines a report shows hit, with their counts.
pub fn line_hits(report: &Path) -> Result<Hits, String> {
    let mut out = Hits::new();
    walk_report(report, |file, class, n, h| {
        if h > 0 {
            let e = out.entry(file.to_string()).or_default().entry((class.to_string(), n)).or_insert(0);
            *e = (*e).max(h);
        }
    })?;
    Ok(out)
}

/// The least of two baselines, line by line: a line one run did not hit counts as unhit, so noise can only widen.
pub fn least(a: Hits, b: &Hits) -> Hits {
    let mut out = Hits::new();
    for (file, lines) in a {
        let theirs = b.get(&file);
        for (key, hits) in lines {
            let other = theirs.and_then(|m| m.get(&key)).copied().unwrap_or(0);
            if hits.min(other) > 0 {
                out.entry(file.clone()).or_default().insert(key, hits.min(other));
            }
        }
    }
    out
}

/// The source files a report shows executed beyond the baseline: a line hit more often than discovery alone hit it.
pub fn executed_files(report: &Path, baseline: Option<&Hits>) -> Result<BTreeSet<String>, String> {
    let mut out = BTreeSet::new();
    walk_report(report, |file, class, n, h| {
        if h > 0 && !out.contains(file) {
            let floor = baseline.and_then(|b| b.get(file)).and_then(|m| m.get(&(class.to_string(), n))).copied().unwrap_or(0);
            if h > floor {
                out.insert(file.to_string());
            }
        }
    })?;
    Ok(out)
}

/// A pdb path as the map records it: relative to the tree, `lunap:` and relative to the toolkit's checkout, or none.
pub fn relative(path: &str, tree: &str, lunap: Option<&str>) -> Option<String> {
    let under = |root: &str| path.strip_prefix(root.trim_end_matches('/')).and_then(|r| r.strip_prefix('/')).map(str::to_string);
    under(tree).or_else(|| lunap.and_then(under).map(|r| format!("lunap:{r}")))
}

fn result_line(log: &Path) -> String {
    std::fs::read_to_string(log).unwrap_or_default().lines().filter(|l| l.contains("Passed!") || l.contains("Failed!")).last().map(|l| l.trim().rsplit_once(" - ").map_or(l.trim(), |(counts, _)| counts).to_string()).unwrap_or_else(|| "no result line".into())
}

fn find_report(dir: &Path) -> Option<PathBuf> {
    for e in std::fs::read_dir(dir).ok()?.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "In") {
                continue;
            }
            if let Some(r) = find_report(&p) {
                return Some(r);
            }
        } else if p.file_name().is_some_and(|n| n == "coverage.cobertura.xml") {
            return Some(p);
        }
    }
    None
}

/// Runs the classes and sends the lines in turn; how many classes gave a report and how many did not. It runs in the
/// foreground past the shell's ten-second budget, which it does not poll, as `trial` does not (EmuSen_DianaSh.md §6.6).
pub fn run(repo: &Repo, snap: &Snapshot, o: &RunOptions, emit: &dyn Fn(&str)) -> Result<(usize, usize), String> {
    let mut classes: Vec<&crate::catalogue::Class> = snap.classes.values().filter(|c| o.nightly || !c.nightly).filter(|c| o.only.is_empty() || o.only.iter().any(|w| c.fqn.contains(w.as_str()))).collect();
    classes.sort_by_key(|c| c.fqn.clone());
    let Some(first) = classes.first() else { return Err("no test class to run".into()) };
    let project = first.project.clone();
    let dll = format!("{}.dll", Path::new(&project).file_stem().map(|s| s.to_string_lossy()).unwrap_or_default());
    if !o.bin.join(&dll).exists() {
        return Err(format!("{} has no {dll}; --bin names the test project's build output", o.bin.display()));
    }
    let csproj = std::fs::read_to_string(repo.root.join(&project)).unwrap_or_default();
    let version = regex::Regex::new(r#"coverlet\.collector"\s+Version="([^"]+)""#).unwrap().captures(&csproj).map(|c| c[1].to_string()).ok_or("the test project references no coverlet.collector")?;
    let adapter = crate::repo::home().join(format!(".nuget/packages/coverlet.collector/{version}/build/netstandard2.0"));
    if !adapter.join("coverlet.collector.dll").exists() {
        return Err(format!("{} is not in the NuGet cache; restore the test project", adapter.display()));
    }
    std::fs::create_dir_all(&o.out).map_err(|e| format!("{}: {e}", o.out.display()))?;
    let settings = o.out.join("coverlet.runsettings");
    std::fs::write(&settings, RUNSETTINGS).map_err(|e| format!("{}: {e}", settings.display()))?;
    let tree = o.tree.display().to_string();
    let lunap = lunap_root(&Repo { root: o.tree.clone() }).map(|p| p.display().to_string());
    let mut copies = Vec::new();
    for i in 1..=o.jobs.max(1) {
        let copy = o.bin.parent().and_then(Path::parent).ok_or("--bin is too shallow for a copy beside it")?.join(format!("coverage-{i}")).join(o.bin.file_name().ok_or("--bin has no name")?);
        if copy == o.bin {
            return Err(format!("{} is where a copy goes; name the build itself", o.bin.display()));
        }
        if copy.exists() {
            std::fs::remove_dir_all(&copy).map_err(|e| format!("{}: {e}", copy.display()))?;
        }
        mirror(&o.bin, &copy)?;
        pre_enumeration(&copy, false)?;
        copies.push(copy);
    }
    // The jobs: a class whole, or one group of a by-case class's cases, keyed by the first quoted argument.
    let mut jobs: VecDeque<(String, String)> = VecDeque::new();
    let mut groups = 0usize;
    for c in &classes {
        if o.by_case.iter().any(|w| c.fqn.contains(w.as_str())) {
            pre_enumeration(&copies[0], true)?;
            let names = list_cases(&copies[0], &dll, &c.fqn, &o.tree, &adapter, &o.out.join("list"));
            pre_enumeration(&copies[0], false)?;
            let names = names?;
            let mut keys = BTreeSet::new();
            let mut plain = false;
            for n in &names {
                match case_key(n) {
                    Some(k) => {
                        keys.insert(k);
                    }
                    None => plain = true,
                }
            }
            if names.is_empty() {
                return Err(format!("{} lists no case to map by", c.fqn));
            }
            for k in keys.into_iter().chain(plain.then(String::new)) {
                let unit = format!("{}#{k}", c.fqn);
                jobs.push_back((unit.clone(), format!("{}&Category!=nightly", crate::adapters::unit_term(&unit))));
                groups += 1;
            }
        } else {
            jobs.push_back((c.fqn.clone(), format!("(FullyQualifiedName~{0}.|FullyQualifiedName~{0}+)&Category!=nightly", c.fqn)));
        }
    }
    emit(&format!("# coverage 1\trev {}\tbuilt {}\tbin {}\tjobs {}\tclasses {}\tgroups {}", o.rev, crate::store::now(), o.bin.display(), copies.len(), classes.len(), groups));
    let stop = AtomicBool::new(false);
    // The discovery baseline: two runs whose filter matches no test, the least hit count of each line kept; one with
    // theories enumerated at run time, and one with them enumerated at discovery for the by-case groups.
    let baseline_of = |on: bool, label: &str| -> Option<Hits> {
        let work = o.out.join("baseline");
        let mut runs: Vec<Hits> = Vec::new();
        let t0 = Instant::now();
        let _ = pre_enumeration(&copies[0], on);
        for _ in 0..2 {
            let _ = std::fs::remove_dir_all(&work);
            let _ = std::fs::create_dir_all(&work);
            let args: Vec<String> = vec!["test".into(), copies[0].join(&dll).display().to_string(), "--no-build".into(), "--filter".into(), "FullyQualifiedName~Nothing.Matches.This.".into(), "--collect:XPlat Code Coverage".into(), "--settings".into(), settings.display().to_string(), "--test-adapter-path".into(), adapter.display().to_string(), "--results-directory".into(), work.display().to_string()];
            let _ = run_logged(scoped("dotnet", &args, &o.tree), &work.join("log"), &stop);
            match find_report(&work).map(|r| line_hits(&r)) {
                Some(Ok(h)) => runs.push(h),
                _ => break,
            }
        }
        let _ = pre_enumeration(&copies[0], false);
        let _ = std::fs::remove_dir_all(&work);
        if runs.len() == 2 {
            let b = least(runs.remove(0), &runs[0]);
            emit(&format!("# baseline{label}\t{} files\t{} lines\t{:.1} s", b.len(), b.values().map(HashMap::len).sum::<usize>(), t0.elapsed().as_secs_f64()));
            Some(b)
        } else {
            emit(&format!("# baseline{label}\tnone: a run with no test gave no report, so discovery's lines count as executed by every class"));
            None
        }
    };
    let baseline = baseline_of(false, "");
    let baseline_on = if groups > 0 { baseline_of(true, " (theories enumerated at discovery)") } else { None };
    let queue = Mutex::new(jobs);
    let (tx, rx) = mpsc::channel::<String>();
    let (mut ran, mut none) = (0usize, 0usize);
    std::thread::scope(|s| {
        for (i, copy) in copies.iter().enumerate() {
            let tx = tx.clone();
            let (queue, stop, lunap, tree, adapter, settings, dll, baseline, baseline_on) = (&queue, &stop, &lunap, &tree, &adapter, &settings, &dll, &baseline, &baseline_on);
            let work = o.out.join(format!("w{}", i + 1));
            s.spawn(move || {
                while let Some((cls, filter)) = queue.lock().ok().and_then(|mut q| q.pop_front()) {
                    if stop.load(Ordering::Relaxed) {
                        break;
                    }
                    let grouped = cls.contains('#');
                    let _ = pre_enumeration(copy, grouped);
                    let _ = std::fs::remove_dir_all(&work);
                    let _ = std::fs::create_dir_all(&work);
                    let log = work.join("log");
                    let args: Vec<String> = vec!["test".into(), copy.join(dll).display().to_string(), "--no-build".into(), "--filter".into(), filter, "--collect:XPlat Code Coverage".into(), "--settings".into(), settings.display().to_string(), "--test-adapter-path".into(), adapter.display().to_string(), "--results-directory".into(), work.display().to_string()];
                    let t0 = Instant::now();
                    let mut status = run_logged(scoped("dotnet", &args, Path::new(tree)), &log, stop);
                    if find_report(&work).is_none() && !stop.load(Ordering::Relaxed) {
                        // Once more after a pause: the previous class's process may still hold a module coverlet backs up.
                        std::thread::sleep(std::time::Duration::from_secs(5));
                        let _ = std::fs::copy(&log, work.join("first-try.log"));
                        status = run_logged(scoped("dotnet", &args, Path::new(tree)), &log, stop);
                    }
                    let secs = t0.elapsed().as_secs_f64();
                    let result = result_line(&log);
                    let mut lines = Vec::new();
                    match find_report(&work) {
                        Some(report) => {
                            let mut n = 0;
                            match executed_files(&report, if grouped { baseline_on.as_ref() } else { baseline.as_ref() }) {
                                Ok(files) => {
                                    for f in files {
                                        match relative(&f, tree, lunap.as_deref()) {
                                            Some(r) => {
                                                lines.push(format!("covered\t{cls}\t{r}"));
                                                n += 1;
                                            }
                                            None => lines.push(format!("# outside\t{cls}\t{f}")),
                                        }
                                    }
                                    lines.push(format!("ran\t{cls}\t{secs:.1}\t{result}\t{n} files"));
                                }
                                Err(e) => lines.push(format!("none\t{cls}\tunreadable report: {e}")),
                            }
                        }
                        None => {
                            let _ = std::fs::copy(&log, work.parent().unwrap_or(&work).join(format!("none-{cls}.log")));
                            lines.push(format!("none\t{cls}\tno report (exit {}; {result})", status.ok().flatten().map_or("stopped".to_string(), |s| s.to_string())));
                        }
                    }
                    let _ = std::fs::remove_dir_all(&work);
                    for l in lines {
                        let _ = tx.send(l);
                    }
                }
            });
        }
        drop(tx);
        for line in rx {
            if line.starts_with("ran\t") {
                ran += 1;
            } else if line.starts_with("none\t") {
                none += 1;
            }
            emit(&line);
        }
    });
    for c in &copies {
        let _ = std::fs::remove_dir_all(c.parent().unwrap_or(c));
    }
    emit(&format!("# done\t{ran} with a report, {none} without"));
    Ok((ran, none))
}
