//! `trial`: a selection in, a verdict out. It groups the units by runner kind (a cargo root with its selected crates,
//! a test project with its selected classes), stands a green result in for a unit whose key has not changed when the
//! tier allows it, runs the rest concurrently in the queue's slots, judges every test (a flake fails unless
//! quarantined), prints one line per unit and the verdict, and records it all. See EmuSen_Testing.md §3.4–§3.5 and
//! `man trial`.

use crate::adapters::{self, Outcomes, Work};
use crate::catalogue::{self, Snapshot};
use crate::config::{self, Tier};
use crate::impact::{self, Pick};
use crate::reports::{Outcome, TestRow};
use crate::repo::{testing_home, Repo};
use crate::store::{NewTrial, Store, UnitRun};
use emusen_sqlite::Bound;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Default)]
pub struct Options {
    pub tier: String,
    pub from: Option<String>,
    pub no_reuse: bool,
    pub dry_run: bool,
    pub runner: String,
    pub requested_by: Option<String>,
    pub base: String,
    pub repo: Option<String>,
}

/// One unit's judgement.
#[derive(Clone, Debug, PartialEq)]
pub struct Judged {
    pub unit: String,
    /// `ok`, `FAIL` or `flaky` (a quarantined flake, which passes).
    pub word: &'static str,
    pub passed: i64,
    pub failed: i64,
    pub skipped: i64,
    pub flaky: i64,
    pub seconds: f64,
    pub notes: Vec<String>,
}

/// Whether a quarantine entry (test, unit) covers a test of a unit.
fn quarantined(q: &[(String, String)], unit: &str, test: &str) -> bool {
    q.iter().any(|(t, u)| t == test && (u.is_empty() || u == unit))
}

/// The policy: a failed test fails the unit; a flaky test (failed, then passed on a rerun) fails it too unless it is
/// quarantined by name; nothing is retried silently.
pub fn judge(unit: &str, rows: &[&TestRow], q: &[(String, String)]) -> Judged {
    let mut j = Judged { unit: unit.to_string(), word: "ok", passed: 0, failed: 0, skipped: 0, flaky: 0, seconds: 0.0, notes: Vec::new() };
    for r in rows {
        j.seconds += r.seconds.unwrap_or(0.0);
        match r.outcome {
            Outcome::Pass => j.passed += 1,
            Outcome::Skip => j.skipped += 1,
            Outcome::Fail => {
                j.failed += 1;
                j.notes.push(format!("{}: {}", r.test, r.message.as_deref().unwrap_or("failed")));
            }
            Outcome::Flaky => {
                j.flaky += 1;
                if quarantined(q, unit, &r.test) {
                    j.notes.push(format!("{}: flaky, quarantined (passed on attempt {})", r.test, r.attempts));
                } else {
                    j.failed += 1;
                    j.notes.push(format!("{}: flaky, not quarantined (failed, then passed on attempt {})", r.test, r.attempts));
                }
            }
        }
    }
    j.word = if j.failed > 0 { "FAIL" } else if j.flaky > 0 { "flaky" } else { "ok" };
    j
}

pub fn clock(secs: f64) -> String {
    let s = secs.round() as u64;
    format!("{}m {:02}s", s / 60, s % 60)
}

fn read_tier(snap: &Snapshot, name: &str) -> Result<Tier, String> {
    match &snap.tiers_text {
        Some(t) => config::parse_tier(t, name),
        None => Ok(Tier { name: name.to_string(), reuse_days: 7.0, ..Tier::default() }),
    }
}

/// A cargo root's profile for the tier: the tier's when the root's manifest defines it, else release.
fn profile_for(repo: &Repo, root: &str, wanted: Option<&str>) -> String {
    let manifest = std::fs::read_to_string(repo.root.join(root).join("Cargo.toml")).unwrap_or_default();
    match wanted {
        Some(p) if p == "release" || manifest.contains(&format!("[profile.{p}]")) => p.to_string(),
        _ => "release".to_string(),
    }
}

struct Planned {
    pick: Pick,
    key: Option<String>,
    reused: Option<(i64, String, i64)>,
    problem: Option<String>,
}

/// Runs in the foreground past the shell's ten-second budget, which it does not poll (EmuSen_DianaSh.md §6.6); Ctrl-C at a terminal reaches its steps' processes.
pub fn run(stdin: Option<String>, o: &Options) -> Result<(String, String, i32), String> {
    let repo = Repo::find(o.repo.as_deref())?;
    let head = repo.rev("HEAD")?;
    let branch = repo.git(&["rev-parse", "--abbrev-ref", "HEAD"]).map(|b| b.trim().to_string()).ok();
    let snap = Snapshot::read(&repo, &head)?;
    let tier = read_tier(&snap, &o.tier)?;
    let mut err = String::new();
    let store = match Store::open() {
        Ok(s) => Some(s),
        Err(e) => {
            err.push_str(&format!("trial: the store could not be opened ({e}); nothing is reused or recorded\n"));
            None
        }
    };
    let picks: Vec<Pick> = match (&o.from, stdin) {
        (Some(f), _) => std::fs::read_to_string(f).map_err(|e| format!("{f}: {e}"))?.lines().filter_map(Pick::parse).collect(),
        (None, Some(text)) => text.lines().filter_map(Pick::parse).collect(),
        (None, None) => {
            let mut files = repo.changed(&o.base, "HEAD")?;
            files.extend(repo.uncommitted()?);
            files.sort();
            files.dedup();
            impact::impact(&repo, &snap, &impact::Ask { files, tier: &tier, failed_last: Vec::new() }).picks
        }
    };
    let q: Vec<(String, String)> = store
        .as_ref()
        .and_then(|s| s.rows("SELECT test, COALESCE(unit, '') FROM quarantine WHERE until >= $now", &[("$now", Bound::Text(&crate::store::now()))]).ok())
        .unwrap_or_default()
        .into_iter()
        .map(|r| (r[0].clone().unwrap_or_default(), r[1].clone().unwrap_or_default()))
        .collect();

    // Keys and reuse.
    let mut dotnet_keys: BTreeMap<String, Option<String>> = BTreeMap::new();
    let mut planned: Vec<Planned> = Vec::new();
    for p in picks {
        let id = p.id();
        let (key, problem) = match p.kind.as_str() {
            "cargo" => match snap.graph.crates.get(&p.unit) {
                Some(c) => {
                    let profile = profile_for(&repo, &c.root, tier.profile.as_deref());
                    (catalogue::key(&repo, &snap, &id, &profile, true).ok().flatten().map(|k| k.key), None)
                }
                None => (None, Some(format!("no crate {} at {}", p.unit, &head[..9]))),
            },
            "dotnet" if p.unit == "lunap" => (catalogue::key(&repo, &snap, "dotnet:lunap", "Debug", true).ok().flatten().map(|k| k.key), catalogue::lunap_tests(&repo).is_none().then(|| "no LunaP tests beside LunaP.props".to_string())),
            "dotnet" => match snap.classes.get(&p.unit) {
                Some(c) => {
                    let k = dotnet_keys.entry(c.project.clone()).or_insert_with(|| catalogue::key(&repo, &snap, &id, "Debug", true).ok().flatten().map(|k| k.key)).clone();
                    (k, None)
                }
                None => (None, Some(format!("no test class {} at {}", p.unit, &head[..9]))),
            },
            _ => (None, Some("job units run from phase 2".to_string())),
        };
        let reused = match (&store, &key) {
            (Some(s), Some(k)) if tier.reuse && !o.no_reuse && problem.is_none() => s.reusable(&id, k, &o.runner, tier.reuse_days).ok().flatten(),
            _ => None,
        };
        planned.push(Planned { pick: p, key, reused, problem });
    }

    let mut out = String::new();
    if o.dry_run {
        for p in &planned {
            let what = match (&p.problem, &p.reused) {
                (Some(w), _) => format!("cannot run: {w}"),
                (None, Some((t, when, n))) => format!("would reuse trial {t} ({when}, {n} passed)"),
                (None, None) => format!("would run{}", if p.key.is_none() { " (no key: uncommitted inputs)" } else { "" }),
            };
            let word = if p.problem.is_some() { "FAIL" } else if p.reused.is_some() { "reuse" } else { "run" };
            out.push_str(&format!("{word}\t{} {}\t{what}\t{} {}\n", p.pick.kind, p.pick.unit, p.pick.reason, p.pick.via));
        }
        out.push_str(&format!("dry run\t{} units, {} would be reused, tier {}\n", planned.len(), planned.iter().filter(|p| p.reused.is_some()).count(), tier.name));
        return Ok((out, err, 0));
    }

    // The works.
    let mut works: Vec<Work> = Vec::new();
    let mut cargo: BTreeMap<String, Vec<(String, Option<String>)>> = BTreeMap::new();
    let mut dotnet: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut lunap = false;
    for p in planned.iter().filter(|p| p.reused.is_none() && p.problem.is_none()) {
        match p.pick.kind.as_str() {
            "cargo" => {
                let c = &snap.graph.crates[&p.pick.unit];
                let name = if c.root == c.dir { None } else { c.name.clone() };
                cargo.entry(c.root.clone()).or_default().push((c.dir.clone(), name));
            }
            "dotnet" if p.pick.unit == "lunap" => lunap = true,
            "dotnet" => dotnet.entry(snap.classes[&p.pick.unit].project.clone()).or_default().push(p.pick.unit.clone()),
            _ => {}
        }
    }
    for (root, crates) in cargo {
        let profile = profile_for(&repo, &root, tier.profile.as_deref());
        works.push(Work::Cargo { root, crates, profile });
    }
    let skip_nightly = tier.exclude.iter().any(|e| e == "nightly");
    for (project, classes) in dotnet {
        let every = snap.classes.values().filter(|c| c.project == project && !(skip_nightly && c.nightly)).count();
        works.push(Work::Dotnet { all: classes.len() >= every, project, classes, skip_nightly });
    }
    if lunap {
        if let Some(tests) = catalogue::lunap_tests(&repo) {
            works.push(Work::Lunap { tests });
        }
    }

    let mut t = NewTrial::new(&head, &tier.name, &o.runner);
    t.base = Some(o.base.clone());
    t.branch = branch.clone();
    t.checkout = Some(repo.root.display().to_string());
    t.requested_by = o.requested_by.clone();
    let trial_id = match store.as_ref().map(|s| s.new_trial(&t)) {
        Some(Ok(id)) => id,
        Some(Err(e)) => {
            err.push_str(&format!("trial: the store refused the trial ({e}); it is not recorded\n"));
            -(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0))
        }
        None => -(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)),
    };
    let dir = testing_home().join("trials").join(trial_id.abs().to_string());
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let started = Instant::now();

    // Run, two CPU steps at a time through the queue.
    let classes = config::runner_classes(std::fs::read_to_string(crate::repo::home().join(".config/EmuSen/testing/runners.json")).ok().as_deref(), &o.runner);
    let cpu = classes.iter().find(|(c, _)| c == "cpu").map_or(2, |(_, n)| *n);
    let stop = Arc::new(AtomicBool::new(false));
    let q_names: Vec<String> = q.iter().map(|(t, _)| t.clone()).collect();
    let mut handles = Vec::new();
    for w in works.clone() {
        let (stop, dir, root, qn) = (stop.clone(), dir.clone(), repo.root.clone(), q_names.clone());
        let who = format!("trial {} ({})", trial_id.abs(), branch.clone().unwrap_or_default());
        handles.push(std::thread::spawn(move || {
            let Some(_slot) = crate::queue::acquire("cpu", cpu, &who, &|| stop.load(Ordering::Relaxed)) else { return (w, Outcomes { error: Some("stopped".into()), ..Outcomes::default() }) };
            let o = match &w {
                Work::Cargo { .. } => adapters::run_cargo(&root, &w, &dir, &qn, &stop),
                _ => adapters::run_dotnet(&root, &w, &dir, &stop),
            };
            (w, o)
        }));
    }
    while handles.iter().any(|h| !h.is_finished()) {
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    let results: Vec<(Work, Outcomes)> = handles.into_iter().filter_map(|h| h.join().ok()).collect();

    // Judge.
    let mut rows_by_unit: BTreeMap<String, Vec<TestRow>> = BTreeMap::new();
    let mut step_error: BTreeMap<String, String> = BTreeMap::new();
    let mut lines: Vec<String> = Vec::new();
    let mut builds: BTreeMap<String, Option<f64>> = BTreeMap::new();
    for (w, o) in &results {
        let (label, units): (String, Vec<String>) = match w {
            Work::Cargo { root, crates, profile } => (format!("cargo {root} ({profile})"), crates.iter().map(|(d, _)| format!("cargo:{d}")).collect()),
            Work::Dotnet { project, classes, .. } => (format!("dotnet {project}"), classes.iter().map(|c| format!("dotnet:{c}")).collect()),
            Work::Lunap { .. } => ("dotnet lunap".to_string(), vec!["dotnet:lunap".to_string()]),
        };
        lines.push(format!("build\t{label}\t{}", o.build_seconds.map_or("-".to_string(), clock)));
        for u in &units {
            builds.insert(u.clone(), o.build_seconds);
            if let Some(e) = &o.error {
                step_error.insert(u.clone(), e.clone());
            }
            rows_by_unit.entry(u.clone()).or_default();
        }
        for (u, r) in &o.rows {
            rows_by_unit.entry(u.clone()).or_default().push(r.clone());
        }
    }
    let mut judged: Vec<Judged> = Vec::new();
    let mut any_red = false;
    for p in &planned {
        let id = p.pick.id();
        if let Some(w) = &p.problem {
            any_red = true;
            lines.push(format!("FAIL\t{} {}\t{w}", p.pick.kind, p.pick.unit));
        } else if let Some((t, when, n)) = &p.reused {
            lines.push(format!("reused\t{} {}\t{n} passed, trial {t} ({when}, key {})", p.pick.kind, p.pick.unit, p.key.as_deref().map_or("-", |k| &k[..12.min(k.len())])));
        }
        let _ = id;
    }
    for (unit, rows) in &rows_by_unit {
        let refs: Vec<&TestRow> = rows.iter().collect();
        let mut j = judge(unit, &refs, &q);
        if let Some(e) = step_error.get(unit) {
            if j.failed == 0 {
                j.word = "FAIL";
                j.notes.insert(0, e.clone());
            }
        }
        any_red |= j.word == "FAIL";
        let (kind, name) = unit.split_once(':').unwrap_or(("", unit));
        let counts = format!("{} passed{}{}{}", j.passed, if j.failed > 0 { format!(", {} failed", j.failed) } else { String::new() }, if j.skipped > 0 { format!(", {} skipped", j.skipped) } else { String::new() }, if j.flaky > 0 { format!(", {} flaky", j.flaky) } else { String::new() });
        lines.push(format!("{}\t{kind} {name}\t{counts}\t{}", j.word, clock(j.seconds)));
        for n in j.notes.iter().take(8) {
            lines.push(format!("\t\t{n}"));
        }
        judged.push(j);
    }
    let elapsed = started.elapsed().as_secs_f64();
    let verdict = if any_red { "RED" } else { "GREEN" };
    let unit_count = planned.len();
    let tail = if unit_count == 0 { " (nothing selected)".to_string() } else { String::new() };
    lines.push(format!("{verdict}\ttrial {}\ttree {}\t{unit_count} units{tail}\t{}\tresults: {}", trial_id.abs(), &head[..9], clock(elapsed), dir.display()));
    out.push_str(&lines.join("\n"));
    out.push('\n');

    // Record; a failure here is reported and never changes the verdict.
    if let (Some(s), true) = (&store, trial_id > 0) {
        let record = || -> Result<(), String> {
            s.exec("BEGIN IMMEDIATE")?;
            for p in &planned {
                s.selection(trial_id, &p.pick.id(), &p.pick.reason, &p.pick.via, p.key.as_deref(), p.reused.as_ref().map(|r| r.0))?;
                if let Some((_, _, n)) = &p.reused {
                    s.unit_run(trial_id, &UnitRun { unit: p.pick.id(), outcome: "reused".into(), passed: *n, ..UnitRun::default() })?;
                }
            }
            for j in &judged {
                if !planned.iter().any(|p| p.pick.id() == j.unit) {
                    s.selection(trial_id, &j.unit, "filter", "matched by the dotnet filter", None, None)?;
                }
                let outcome = match j.word {
                    "FAIL" => "fail",
                    w => w,
                };
                s.unit_run(trial_id, &UnitRun { unit: j.unit.clone(), outcome: outcome.into(), passed: j.passed, failed: j.failed, skipped: j.skipped, flaky: j.flaky, seconds: Some(j.seconds), build_seconds: builds.get(&j.unit).copied().flatten(), log: Some(dir.display().to_string()) })?;
                for r in rows_by_unit.get(&j.unit).into_iter().flatten() {
                    s.result(trial_id, &j.unit, r)?;
                }
            }
            s.finish_trial(trial_id, verdict, elapsed, None)?;
            s.exec("COMMIT")
        };
        if let Err(e) = record() {
            let _ = s.exec("ROLLBACK");
            err.push_str(&format!("trial: recording trial {trial_id} failed ({e}); the verdict stands\n"));
        }
    }
    Ok((out, err, if any_red { 1 } else { 0 }))
}

/// `trial verdict SHA`: the newest trial of a tree, as push.sh reads it.
pub fn verdict(store: &Store, sha: &str) -> Result<(String, i32), String> {
    let rows = store.rows(
        "SELECT id, verdict, started, tier FROM trial WHERE tree LIKE $s || '%' AND verdict IS NOT NULL ORDER BY id DESC LIMIT 1",
        &[("$s", Bound::Text(sha))],
    )?;
    let Some(r) = rows.first() else { return Ok((format!("NONE no trial of {sha}\n"), 2)) };
    let v = r[1].clone().unwrap_or_default();
    let failed = store.rows("SELECT unit FROM unit_run WHERE trial = $t AND outcome = 'fail'", &[("$t", Bound::Text(r[0].as_deref().unwrap_or("0")))])?.len();
    let status = if v == "GREEN" { 0 } else { 1 };
    Ok((format!("{v} trial {} {} tier {}: {failed} failed unit(s); see trial show {}\n", r[0].as_deref().unwrap_or(""), r[2].as_deref().unwrap_or(""), r[3].as_deref().unwrap_or(""), r[0].as_deref().unwrap_or("")), status))
}

pub fn ls(store: &Store, n: usize) -> Result<String, String> {
    let rows = store.rows(
        &format!("SELECT id, started, verdict, tier, substr(tree, 1, 9), COALESCE(branch, '-'), COALESCE(seconds, 0), COALESCE(source, '-') FROM trial ORDER BY id DESC LIMIT {n}"),
        &[],
    )?;
    Ok(rows.iter().map(|r| format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n", r[0].as_deref().unwrap_or(""), r[1].as_deref().unwrap_or(""), r[2].as_deref().unwrap_or("-"), r[3].as_deref().unwrap_or(""), r[4].as_deref().unwrap_or(""), r[5].as_deref().unwrap_or(""), clock(r[6].as_deref().unwrap_or("0").parse().unwrap_or(0.0)), r[7].as_deref().unwrap_or(""))).collect())
}

pub fn show(store: &Store, id: &str) -> Result<String, String> {
    let rows = store.rows(
        "SELECT unit, outcome, passed, failed, skipped, COALESCE(seconds, 0), COALESCE(build_seconds, 0) FROM unit_run WHERE trial = $t ORDER BY unit",
        &[("$t", Bound::Text(id))],
    )?;
    let mut out: String = rows
        .iter()
        .map(|r| format!("{}\t{}\t{} passed, {} failed, {} skipped\t{}\tbuild {}\n", r[1].as_deref().unwrap_or(""), r[0].as_deref().unwrap_or(""), r[2].as_deref().unwrap_or("0"), r[3].as_deref().unwrap_or("0"), r[4].as_deref().unwrap_or("0"), clock(r[5].as_deref().unwrap_or("0").parse().unwrap_or(0.0)), clock(r[6].as_deref().unwrap_or("0").parse().unwrap_or(0.0))))
        .collect();
    for r in store.rows("SELECT unit, test, outcome, COALESCE(message, '') FROM result WHERE trial = $t AND outcome IN ('fail', 'flaky') ORDER BY unit, test", &[("$t", Bound::Text(id))])? {
        out.push_str(&format!("\t{}\t{} {}: {}\n", r[2].as_deref().unwrap_or(""), r[0].as_deref().unwrap_or(""), r[1].as_deref().unwrap_or(""), r[3].as_deref().unwrap_or("")));
    }
    Ok(out)
}

/// `trial quarantine add TEST --owner O --cause C [--unit U] [--days N]`: at most 14 days; a renewal needs a new cause.
pub fn quarantine_add(store: &Store, test: &str, unit: &str, owner: &str, cause: &str, days: u64) -> Result<String, String> {
    if owner.trim().is_empty() || cause.trim().is_empty() {
        return Err("trial quarantine: a quarantine needs --owner and --cause".into());
    }
    if days == 0 || days > 14 {
        return Err("trial quarantine: --days is 1 to 14".into());
    }
    let old = store.rows("SELECT cause FROM quarantine WHERE test = $t", &[("$t", Bound::Text(test))])?;
    if old.first().and_then(|r| r[0].as_deref()) == Some(cause) {
        return Err(format!("trial quarantine: {test} is already quarantined for that cause; a renewal needs a new cause"));
    }
    let until = crate::store::iso(std::time::SystemTime::now() + std::time::Duration::from_secs(days * 86400));
    store.run(
        "INSERT OR REPLACE INTO quarantine (test, unit, owner, cause, since, until) VALUES ($t, $u, $o, $c, $s, $until)",
        &[("$t", Bound::Text(test)), ("$u", Bound::Text(unit)), ("$o", Bound::Text(owner)), ("$c", Bound::Text(cause)), ("$s", Bound::Text(&crate::store::now())), ("$until", Bound::Text(&until))],
    )?;
    Ok(format!("quarantined\t{test}\tuntil {until}\towner {owner}\t{cause}\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(test: &str, outcome: Outcome, attempts: u32) -> TestRow {
        TestRow { group: "g".into(), test: test.into(), outcome, seconds: Some(1.0), attempts, message: None }
    }

    #[test]
    fn a_flaky_test_fails_unless_quarantined() {
        let rows = [row("a", Outcome::Pass, 1), row("racy", Outcome::Flaky, 2)];
        let refs: Vec<&TestRow> = rows.iter().collect();
        let j = judge("cargo:x", &refs, &[]);
        assert_eq!((j.word, j.failed, j.flaky), ("FAIL", 1, 1));
        assert!(j.notes[0].contains("not quarantined"));
        let j = judge("cargo:x", &refs, &[("racy".into(), "cargo:x".into())]);
        assert_eq!((j.word, j.failed, j.flaky), ("flaky", 0, 1));
        let j = judge("cargo:x", &refs, &[("racy".into(), "cargo:other".into())]);
        assert_eq!(j.word, "FAIL", "a quarantine names its unit");
        let rows = [row("a", Outcome::Pass, 1), row("b", Outcome::Skip, 1)];
        let refs: Vec<&TestRow> = rows.iter().collect();
        assert_eq!(judge("cargo:x", &refs, &[]).word, "ok");
    }

    #[test]
    fn a_clock_is_minutes_and_seconds() {
        assert_eq!(clock(667.4), "11m 07s");
    }
}
