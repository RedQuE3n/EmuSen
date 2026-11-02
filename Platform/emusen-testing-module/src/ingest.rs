//! `ingest`: result files into the store. A TRX or JUnit file becomes a trial's per-test rows; `--backfill` reads the
//! logs land.sh left before the studio existed (each `land/<sha>/` folder, the `land-*.log` transcripts, the full-suite
//! logs), so the history starts on 23 October. A source is read once: its path is the trial's `source`. See
//! EmuSen_Testing.md §3.5 and `man ingest`.

use crate::catalogue::Snapshot;
use crate::reports::{self, Outcome, TestRow};
use crate::repo::Repo;
use crate::store::{iso, NewTrial, Store, UnitRun};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct Tally {
    pub trials: usize,
    pub green: usize,
    pub red: usize,
    pub unknown: usize,
    pub unit_runs: usize,
    pub tests: usize,
    pub skipped_sources: usize,
    pub land_dirs: usize,
    pub land_logs: usize,
    pub land_logs_matched: usize,
    pub full_logs: usize,
    pub cargo_logs: usize,
    pub dotnet_logs: usize,
    pub unresolved_crates: usize,
}

fn mtime(p: &Path) -> Option<std::time::SystemTime> {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
}

/// Records one trial's unit runs and rows in a transaction.
fn record(store: &Store, t: &NewTrial, runs: &[UnitRun], rows: &[(String, TestRow)], tally: &mut Tally) -> Result<i64, String> {
    store.exec("BEGIN IMMEDIATE")?;
    let go = || -> Result<i64, String> {
        let id = store.new_trial(t)?;
        for r in runs {
            store.selection(id, &r.unit, "backfill", t.source.as_deref().unwrap_or(""), None, None)?;
            store.unit_run(id, r)?;
        }
        for (u, r) in rows {
            store.unit(u)?;
            store.result(id, u, r)?;
        }
        Ok(id)
    };
    match go() {
        Ok(id) => {
            store.exec("COMMIT")?;
            tally.trials += 1;
            tally.unit_runs += runs.len();
            tally.tests += rows.len();
            match t.verdict.as_deref() {
                Some("GREEN") => tally.green += 1,
                Some("RED") => tally.red += 1,
                _ => tally.unknown += 1,
            }
            Ok(id)
        }
        Err(e) => {
            let _ = store.exec("ROLLBACK");
            Err(e)
        }
    }
}

/// A crate folder from land.sh's log name, which wrote ' ' and '/' as '_': matched against the revision's crates.
pub fn crate_from_log_name(snap: &Snapshot, mangled: &str) -> Option<String> {
    snap.graph.crates.keys().find(|d| d.replace(['/', ' '], "_") == mangled).cloned()
}

/// `ingest FILE... --runner R --tree SHA [--trial N] [--tier T]`.
pub fn files(store: &Store, paths: &[String], runner: &str, tree: &str, trial: Option<i64>, tier: &str) -> Result<String, String> {
    let mut out = String::new();
    for p in paths {
        let text = std::fs::read_to_string(p).map_err(|e| format!("{p}: {e}"))?;
        let (rows, kind): (Vec<TestRow>, &str) = if text.contains("<TestRun") { (reports::trx(&text), "dotnet") } else if text.contains("<testsuite") { (reports::junit(&text), "cargo") } else { return Err(format!("{p}: neither TRX nor JUnit")) };
        let source = std::fs::canonicalize(p).map(|c| c.display().to_string()).unwrap_or_else(|_| p.clone());
        if trial.is_none() && store.has_source(&source)? {
            out.push_str(&format!("skipped\t{p}\talready ingested\n"));
            continue;
        }
        let mut by: BTreeMap<String, Vec<TestRow>> = BTreeMap::new();
        for r in rows {
            let unit = if kind == "dotnet" { format!("dotnet:{}", r.group) } else { format!("cargo:{}", r.group.split("::").next().unwrap_or("")) };
            by.entry(unit).or_default().push(r);
        }
        let runs: Vec<UnitRun> = by.iter().map(|(u, rs)| unit_run_of(u, rs)).collect();
        let flat: Vec<(String, TestRow)> = by.into_iter().flat_map(|(u, rs)| rs.into_iter().map(move |r| (u.clone(), r))).collect();
        let red = runs.iter().any(|r| r.outcome == "fail");
        let id = match trial {
            Some(id) => {
                store.exec("BEGIN IMMEDIATE")?;
                for r in &runs {
                    store.selection(id, &r.unit, "ingest", &source, None, None)?;
                    store.unit_run(id, r)?;
                }
                for (u, r) in &flat {
                    store.result(id, u, r)?;
                }
                store.exec("COMMIT")?;
                id
            }
            None => {
                let mut t = NewTrial::new(tree, tier, runner);
                t.started = mtime(Path::new(p)).map(iso).unwrap_or_else(crate::store::now);
                t.finished = Some(t.started.clone());
                t.verdict = Some(if red { "RED" } else { "GREEN" }.into());
                t.seconds = Some(runs.iter().filter_map(|r| r.seconds).sum());
                t.source = Some(source);
                record(store, &t, &runs, &flat, &mut Tally::default())?
            }
        };
        out.push_str(&format!("ingested\t{p}\ttrial {id}\t{} units, {} tests\n", runs.len(), flat.len()));
    }
    Ok(out)
}

fn unit_run_of(unit: &str, rows: &[TestRow]) -> UnitRun {
    let count = |o: Outcome| rows.iter().filter(|r| r.outcome == o).count() as i64;
    let failed = count(Outcome::Fail);
    UnitRun {
        unit: unit.to_string(),
        outcome: if failed > 0 { "fail" } else if count(Outcome::Flaky) > 0 { "flaky" } else { "ok" }.into(),
        passed: count(Outcome::Pass),
        failed,
        skipped: count(Outcome::Skip),
        flaky: count(Outcome::Flaky),
        seconds: Some(rows.iter().filter_map(|r| r.seconds).sum()),
        build_seconds: None,
        log: None,
    }
}

/// times.txt's "cargo <dir> (<profile>): N s" and "dotnet build: N s" lines.
fn times(dir: &Path) -> BTreeMap<String, f64> {
    let mut m = BTreeMap::new();
    for l in std::fs::read_to_string(dir.join("times.txt")).unwrap_or_default().lines() {
        if let Some((k, v)) = l.rsplit_once(": ") {
            if let Some(n) = v.strip_suffix(" s").and_then(|n| n.trim().parse().ok()) {
                let key = k.strip_prefix("cargo ").map(|c| c.rsplit_once(" (").map_or(c, |x| x.0).to_string()).unwrap_or_else(|| k.to_string());
                m.insert(key, n);
            }
        }
    }
    m
}

/// One `land/<sha>/` folder as a trial.
fn land_dir(store: &Store, snap: &Snapshot, repo: &Repo, base: &Path, sha: &str, tally: &mut Tally) -> Result<(), String> {
    let dir = base.join("land").join(sha);
    let source = format!("land/{sha}");
    if store.has_source(&source)? {
        tally.skipped_sources += 1;
        return Ok(());
    }
    let entries: Vec<PathBuf> = std::fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten().map(|e| e.path()).collect();
    let stamps: Vec<std::time::SystemTime> = entries.iter().filter_map(|p| mtime(p)).collect();
    let (Some(first), Some(last)) = (stamps.iter().min(), stamps.iter().max()) else { return Ok(()) };
    if std::time::SystemTime::now().duration_since(*last).map(|d| d.as_secs() < 3600).unwrap_or(true) && !base.join("land").join(format!("{sha}.ok")).exists() {
        return Ok(()); // a land still running
    }
    let t_map = times(&dir);
    let mut runs = Vec::new();
    let mut rows = Vec::new();
    let mut red = false;
    for p in entries.iter().filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("cargo-") && n.ends_with(".log"))) {
        tally.cargo_logs += 1;
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let mangled = &name["cargo-".len()..name.len() - 4];
        let unit = match crate_from_log_name(snap, mangled) {
            Some(d) => format!("cargo:{d}"),
            None => {
                tally.unresolved_crates += 1;
                format!("cargo:?{mangled}")
            }
        };
        let log = reports::cargo_log(&std::fs::read_to_string(p).unwrap_or_default());
        let failed = log.failed || log.failures > 0;
        red |= failed;
        let wall = unit.strip_prefix("cargo:").and_then(|d| t_map.get(d)).copied();
        runs.push(UnitRun {
            unit: unit.clone(),
            outcome: if failed { "fail" } else { "ok" }.into(),
            passed: log.passed,
            failed: log.failures,
            skipped: log.ignored,
            flaky: 0,
            seconds: Some(wall.map_or(log.test_seconds, |w| (w - log.build_seconds.unwrap_or(0.0)).max(log.test_seconds))),
            build_seconds: log.build_seconds,
            log: Some(p.display().to_string()),
        });
        rows.extend(log.rows.into_iter().map(|r| (unit.clone(), r)));
    }
    let trx = dir.join("dotnet.trx");
    let dlog = dir.join("dotnet.log");
    if trx.exists() {
        tally.dotnet_logs += 1;
        let parsed = reports::trx(&std::fs::read_to_string(&trx).unwrap_or_default());
        let mut by: BTreeMap<String, Vec<TestRow>> = BTreeMap::new();
        for r in parsed {
            by.entry(format!("dotnet:{}", r.group)).or_default().push(r);
        }
        for (u, rs) in by {
            let mut run = unit_run_of(&u, &rs);
            run.build_seconds = t_map.get("dotnet build").copied();
            red |= run.outcome == "fail";
            runs.push(run);
            rows.extend(rs.into_iter().map(|r| (u.clone(), r)));
        }
    } else if dlog.exists() {
        tally.dotnet_logs += 1;
        let text = std::fs::read_to_string(&dlog).unwrap_or_default();
        let d = reports::dotnet_log(&text);
        let failed = d.failed > 0 || (!d.summary && !text.contains("No test matches"));
        red |= failed;
        runs.push(UnitRun {
            unit: "dotnet:EmuSen.WiseMan".into(),
            outcome: if !d.summary && !failed { "none".into() } else if failed { "fail".into() } else { "ok".into() },
            passed: d.passed,
            failed: d.failed,
            skipped: d.skipped,
            flaky: 0,
            seconds: d.seconds,
            build_seconds: t_map.get("dotnet build").copied(),
            log: Some(dlog.display().to_string()),
        });
        for f in d.failures {
            rows.push((format!("dotnet:{}", reports::class_from_name(&f)), TestRow { group: String::new(), test: f, outcome: Outcome::Fail, seconds: None, attempts: 1, message: None }));
        }
    }
    let ok = base.join("land").join(format!("{sha}.ok")).exists();
    let tree = repo.rev(sha).unwrap_or_else(|_| sha.to_string());
    let mut t = NewTrial::new(&tree, "land", "desktop");
    t.started = iso(*first);
    t.finished = Some(iso(*last));
    t.seconds = Some(last.duration_since(*first).map(|d| d.as_secs_f64()).unwrap_or(0.0).max(t_map.values().sum()));
    t.requested_by = Some("land".into());
    t.verdict = Some(if ok { "GREEN" } else if red { "RED" } else { "UNKNOWN" }.into());
    t.note = Some("backfilled from land.sh's logs".into());
    t.source = Some(source);
    tally.land_dirs += 1;
    record(store, &t, &runs, &rows, tally).map(|_| ())
}

/// One `land-<name>.log` transcript: names the branch of the trial its verdict line names, or becomes a trial itself.
fn land_log(store: &Store, repo: &Repo, path: &Path, tally: &mut Tally) -> Result<(), String> {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let source = format!("transcript/{name}");
    if store.has_source(&source)? {
        tally.skipped_sources += 1;
        return Ok(());
    }
    let text = std::fs::read_to_string(path).unwrap_or_default();
    tally.land_logs += 1;
    let branch = name.strip_prefix("land-").and_then(|n| n.strip_suffix(".log")).unwrap_or(name).to_string();
    let verdict = text.lines().rev().find_map(|l| {
        let mut w = l.split_whitespace();
        match (w.next(), w.next()) {
            (Some(v @ ("GREEN" | "RED")), Some(short)) => Some((v.to_string(), short.trim_end_matches(':').to_string())),
            _ => None,
        }
    });
    let Some((verdict, short)) = verdict else { return Ok(()) };
    let found = store.rows("SELECT id FROM trial WHERE tree LIKE $s || '%' AND source LIKE 'land/%' ORDER BY id DESC LIMIT 1", &[("$s", emusen_sqlite::Bound::Text(&short))])?;
    if let Some(id) = found.first().and_then(|r| r[0].clone()) {
        tally.land_logs_matched += 1;
        return store.run("UPDATE trial SET branch = COALESCE(branch, $b) WHERE id = $id", &[("$b", emusen_sqlite::Bound::Text(&branch)), ("$id", emusen_sqlite::Bound::Text(&id))]);
    }
    let mut runs = Vec::new();
    for l in text.lines() {
        let (word, rest) = match l.split_once(char::is_whitespace) {
            Some((w @ ("ok" | "FAIL"), r)) => (w, r.trim()),
            _ => continue,
        };
        let outcome = if word == "ok" { "ok" } else { "fail" };
        if let Some(c) = rest.strip_prefix("cargo ") {
            let dir = c.split(": ").next().unwrap_or(c).split(" (exit").next().unwrap_or(c).to_string();
            let passed = c.rsplit(": ").next().and_then(|p| p.strip_suffix(" passed")).and_then(|n| n.parse().ok()).unwrap_or(0);
            runs.push(UnitRun { unit: format!("cargo:{dir}"), outcome: outcome.into(), passed, ..UnitRun::default() });
        } else if rest.starts_with("dotnet") {
            let d = reports::dotnet_log(rest.split_once(": ").map_or(rest, |x| x.1));
            let passed = if d.summary { d.passed } else { rest.split("Passed:").nth(1).and_then(|p| p.trim().split([',', ' ']).next()).and_then(|n| n.parse().ok()).unwrap_or(0) };
            runs.push(UnitRun { unit: "dotnet:EmuSen.WiseMan".into(), outcome: outcome.into(), passed, failed: d.failed, skipped: d.skipped, seconds: d.seconds, ..UnitRun::default() });
        }
    }
    let tree = repo.rev(&short).unwrap_or(short);
    let mut t = NewTrial::new(&tree, "land", "desktop");
    let when = mtime(path).map(iso).unwrap_or_else(crate::store::now);
    t.started = when.clone();
    t.finished = Some(when);
    t.branch = Some(branch);
    t.requested_by = Some("land".into());
    t.verdict = Some(verdict);
    t.note = Some("backfilled from a land transcript; its land folder is gone".into());
    t.source = Some(source);
    record(store, &t, &runs, &[], tally).map(|_| ())
}

/// A full-suite log: one trial of tier `full` with WiseMan's counts.
fn full_log(store: &Store, path: &Path, tally: &mut Tally) -> Result<(), String> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let d = reports::dotnet_log(&text);
    if !d.summary || d.passed + d.failed + d.skipped < 7000 {
        return Ok(());
    }
    let source = format!("full/{}", path.file_name().and_then(|n| n.to_str()).unwrap_or(""));
    if store.has_source(&source)? {
        tally.skipped_sources += 1;
        return Ok(());
    }
    tally.full_logs += 1;
    let mut t = NewTrial::new("unknown", "full", "desktop");
    let when = mtime(path).map(iso).unwrap_or_else(crate::store::now);
    t.started = when.clone();
    t.finished = Some(when);
    t.seconds = d.seconds;
    t.verdict = Some(if d.failed > 0 { "RED" } else { "GREEN" }.into());
    t.note = Some(format!("backfilled from {}", path.display()));
    t.source = Some(source);
    let run = UnitRun { unit: "dotnet:EmuSen.WiseMan".into(), outcome: if d.failed > 0 { "fail" } else { "ok" }.into(), passed: d.passed, failed: d.failed, skipped: d.skipped, seconds: d.seconds, log: Some(path.display().to_string()), ..UnitRun::default() };
    record(store, &t, &[run], &[], tally).map(|_| ())
}

/// `ingest --backfill [DIR]`: every land folder, transcript and full-suite log under DIR (default ~/.cache/emusen/tmp).
pub fn backfill(store: &Store, repo: &Repo, base: &Path) -> Result<Tally, String> {
    let snap = Snapshot::read(repo, "HEAD")?;
    let mut tally = Tally::default();
    let land = base.join("land");
    let mut shas: Vec<(std::time::SystemTime, String)> = std::fs::read_dir(&land)
        .map_err(|e| format!("{}: {e}", land.display()))?
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| (mtime(&e.path()).unwrap_or(std::time::UNIX_EPOCH), e.file_name().to_string_lossy().into_owned()))
        .collect();
    shas.sort();
    for (_, sha) in &shas {
        land_dir(store, &snap, repo, base, sha, &mut tally)?;
    }
    let mut logs: Vec<PathBuf> = std::fs::read_dir(base).map_err(|e| e.to_string())?.flatten().map(|e| e.path()).filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("land-") && n.ends_with(".log"))).collect();
    logs.sort();
    for p in &logs {
        land_log(store, repo, p, &mut tally)?;
    }
    let wr = base.join("wiseman-rust");
    let mut fulls: Vec<PathBuf> = std::fs::read_dir(&wr).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("full") && n.ends_with(".log"))).collect();
    fulls.sort();
    for p in &fulls {
        full_log(store, p, &mut tally)?;
    }
    Ok(tally)
}
