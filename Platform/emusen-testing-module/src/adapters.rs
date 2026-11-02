//! The adapters, one per runner kind: cargo through nextest (its JUnit per test, retries recorded), dotnet through
//! `dotnet test` with TRX (a failed test rerun once by name to tell a flake from a failure), and the sibling LunaP
//! suite. Each step runs in its own capped scope, in the shell's process group, so Ctrl-C at a terminal reaches it. The dotnet adapter is
//! the one file removed with the last C# test project. See EmuSen_Testing.md §3.4.

use crate::reports::{self, Outcome, TestRow};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// What a step runs.
#[derive(Clone, Debug)]
pub enum Work {
    /// A cargo root with the crates selected in it: (folder, package name); the root alone when it is one crate.
    Cargo { root: String, crates: Vec<(String, Option<String>)>, profile: String },
    /// A test project with the classes selected in it; every class when `all`.
    Dotnet { project: String, classes: Vec<String>, all: bool, skip_nightly: bool },
    Lunap { tests: PathBuf },
}

#[derive(Debug, Default)]
pub struct Outcomes {
    /// (unit id, test row)
    pub rows: Vec<(String, TestRow)>,
    pub build_seconds: Option<f64>,
    pub test_seconds: f64,
    pub log: PathBuf,
    /// A step that could not run or whose run says nothing per test: its words.
    pub error: Option<String>,
}

/// A command in the project's capped scope (`EMUSEN_TESTING_NO_SCOPE=1` runs it bare, for a machine without systemd).
pub fn scoped(program: &str, args: &[String], cwd: &Path) -> Command {
    let mut c = if std::env::var_os("EMUSEN_TESTING_NO_SCOPE").is_some() {
        let mut c = Command::new(program);
        c.args(args);
        c
    } else {
        let memory = format!("MemoryMax={}", std::env::var("EMUSEN_TESTING_MEMORY_MAX").unwrap_or_else(|_| "10G".into()));
        let mut c = Command::new("systemd-run");
        c.args(["--user", "--scope", "-q", "-p", "CPUWeight=20", "-p", &memory, "-p", "MemorySwapMax=2G", program]);
        c.args(args);
        c
    };
    c.current_dir(cwd).env("TMPDIR", crate::repo::home().join(".cache/emusen/tmp")).env("MSBUILDDISABLENODEREUSE", "1").env("UseSharedCompilation", "false");
    c
}

/// Runs a command to a log file, ending its process group when `stop` is set; the exit status, or none when stopped.
pub fn run_logged(mut c: Command, log: &Path, stop: &AtomicBool) -> Result<Option<i32>, String> {
    let file = std::fs::File::create(log).map_err(|e| format!("{}: {e}", log.display()))?;
    let err = file.try_clone().map_err(|e| e.to_string())?;
    let mut child = c.stdin(Stdio::null()).stdout(file).stderr(err).spawn().map_err(|e| format!("{:?} did not start: {e}", c.get_program()))?;
    loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            return Ok(Some(status.code().unwrap_or(-1)));
        }
        if stop.load(Ordering::Relaxed) {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(None);
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// nextest's configuration for a trial: one retry for every test, so a flake is seen and recorded, two for a
/// quarantined one, no fail-fast, and JUnit.
pub fn nextest_config(quarantined: &[String]) -> String {
    let mut c = String::from("[profile.studio]\nretries = 1\nfail-fast = false\nslow-timeout = { period = \"60s\" }\n\n[profile.studio.junit]\npath = \"junit.xml\"\nstore-success-output = false\nstore-failure-output = true\n");
    if !quarantined.is_empty() {
        let filter: Vec<String> = quarantined.iter().map(|t| format!("test(={})", t.replace('\'', ""))).collect();
        c.push_str(&format!("\n[[profile.studio.overrides]]\nfilter = '{}'\nretries = 2\n", filter.join(" | ")));
    }
    c
}

pub fn run_cargo(repo_root: &Path, work: &Work, dir: &Path, quarantined: &[String], stop: &AtomicBool) -> Outcomes {
    let Work::Cargo { root, crates, profile } = work else { unreachable!() };
    let tag = root.replace(['/', ' '], "_");
    let mut out = Outcomes { log: dir.join(format!("cargo-{tag}.log")), ..Outcomes::default() };
    let config = dir.join(format!("nextest-{tag}.toml"));
    if let Err(e) = std::fs::write(&config, nextest_config(quarantined)) {
        out.error = Some(format!("{}: {e}", config.display()));
        return out;
    }
    let root_dir = repo_root.join(root);
    let mut args: Vec<String> = vec!["nextest".into(), "run".into(), "--cargo-profile".into(), profile.clone(), "--profile".into(), "studio".into(), "--config-file".into(), config.display().to_string(), "--no-fail-fast".into(), "--no-tests=pass".into()];
    for (_, name) in crates {
        if let Some(n) = name {
            args.extend(["-p".to_string(), n.clone()]);
        }
    }
    let started = Instant::now();
    let status = match run_logged(scoped("cargo", &args, &root_dir), &out.log, stop) {
        Ok(Some(s)) => s,
        Ok(None) => {
            out.error = Some("stopped".into());
            return out;
        }
        Err(e) => {
            out.error = Some(e);
            return out;
        }
    };
    let text = std::fs::read_to_string(&out.log).unwrap_or_default();
    let log = reports::cargo_log(&text);
    out.build_seconds = log.build_seconds;
    out.test_seconds = started.elapsed().as_secs_f64() - log.build_seconds.unwrap_or(0.0);
    let target = std::env::var_os("CARGO_TARGET_DIR").map(PathBuf::from).unwrap_or_else(|| root_dir.join("target"));
    let junit = target.join("nextest/studio/junit.xml");
    let rows = std::fs::read_to_string(&junit).map(|x| reports::junit(&x)).unwrap_or_default();
    let _ = std::fs::copy(&junit, dir.join(format!("junit-{tag}.xml")));
    let _ = std::fs::remove_file(&junit);
    let by_name: Vec<(String, String)> = crates.iter().map(|(d, n)| (n.clone().unwrap_or_default(), d.clone())).collect();
    for r in rows {
        let package = r.group.split("::").next().unwrap_or("").to_string();
        let unit = by_name.iter().find(|(n, _)| *n == package).map(|(_, d)| d.clone()).or_else(|| (crates.len() == 1).then(|| crates[0].0.clone())).unwrap_or_else(|| root.clone());
        out.rows.push((format!("cargo:{unit}"), r));
    }
    let failing = out.rows.iter().any(|(_, r)| r.outcome == Outcome::Fail);
    if status != 0 && !failing {
        out.error = Some(format!("cargo nextest exited {status}{}; see {}", if out.rows.is_empty() { " with no test results" } else { "" }, out.log.display()));
    }
    out
}

/// The dotnet filter for a selection of classes: every class's name, or only the tier's exclusion when all are chosen.
/// A unit `Class#Key` is one group of the class's cases, those whose display name holds the key in quotes (with or
/// without a trailing size); `Class#` is its cases without arguments.
pub fn dotnet_filter(classes: &[String], all: bool, skip_nightly: bool) -> String {
    let nightly = if skip_nightly { "Category!=nightly" } else { "" };
    if all {
        return nightly.to_string();
    }
    let terms: Vec<String> = classes.iter().map(|c| unit_term(c)).collect();
    if nightly.is_empty() { format!("({})", terms.join("|")) } else { format!("({})&{nightly}", terms.join("|")) }
}

/// One unit's filter term.
pub fn unit_term(unit: &str) -> String {
    match unit.split_once('#') {
        None => format!("FullyQualifiedName~{unit}"),
        Some((class, "")) => format!("(FullyQualifiedName~{class}.&DisplayName!~\")"),
        Some((class, key)) => format!("(FullyQualifiedName~{class}.&(DisplayName~\"{key}\"|DisplayName~\"{key}-))"),
    }
}

fn dotnet_test(cwd: &Path, target: &str, filter: &str, dir: &Path, name: &str, stop: &AtomicBool) -> Result<(Option<i32>, PathBuf, Vec<TestRow>), String> {
    let log = dir.join(format!("{name}.log"));
    let mut args: Vec<String> = vec!["test".into(), target.into(), "--no-build".into(), "--results-directory".into(), dir.display().to_string(), "--logger".into(), format!("trx;LogFileName={name}.trx")];
    if !filter.is_empty() {
        args.extend(["--filter".to_string(), filter.to_string()]);
    }
    let status = run_logged(scoped("dotnet", &args, cwd), &log, stop)?;
    let rows = std::fs::read_to_string(dir.join(format!("{name}.trx"))).map(|x| reports::trx(&x)).unwrap_or_default();
    Ok((status, log, rows))
}

pub fn run_dotnet(repo_root: &Path, work: &Work, dir: &Path, stop: &AtomicBool) -> Outcomes {
    let (target, cwd, filter) = match work {
        Work::Dotnet { project, classes, all, skip_nightly } => (project.clone(), repo_root.to_path_buf(), dotnet_filter(classes, *all, *skip_nightly)),
        Work::Lunap { tests } => (tests.display().to_string(), tests.clone(), String::new()),
        Work::Cargo { .. } => unreachable!(),
    };
    let lunap = matches!(work, Work::Lunap { .. });
    let stem = if lunap { "lunap" } else { "dotnet" };
    let mut out = Outcomes { log: dir.join(format!("{stem}.log")), ..Outcomes::default() };
    let build_log = dir.join(format!("{stem}-build.log"));
    let started = Instant::now();
    match run_logged(scoped("dotnet", &["build".into(), target.clone(), "-p:UseSharedCompilation=false".into()], &cwd), &build_log, stop) {
        Ok(Some(0)) => {}
        Ok(Some(s)) => {
            out.error = Some(format!("dotnet build exited {s}; see {}", build_log.display()));
            return out;
        }
        Ok(None) => {
            out.error = Some("stopped".into());
            return out;
        }
        Err(e) => {
            out.error = Some(e);
            return out;
        }
    }
    out.build_seconds = Some(started.elapsed().as_secs_f64());
    let t0 = Instant::now();
    let (status, log, mut rows) = match dotnet_test(&cwd, &target, &filter, dir, stem, stop) {
        Ok(r) => r,
        Err(e) => {
            out.error = Some(e);
            return out;
        }
    };
    out.log = log;
    out.test_seconds = t0.elapsed().as_secs_f64();
    let Some(status) = status else {
        out.error = Some("stopped".into());
        return out;
    };
    if rows.is_empty() {
        let text = std::fs::read_to_string(&out.log).unwrap_or_default();
        out.error = Some(if text.contains("No test matches the given testcase filter") { "no test matched the filter".to_string() } else { format!("dotnet test exited {status} with no TRX results; see {}", out.log.display()) });
        return out;
    }
    // A failed test is run once more by its name: a pass then is a flake, recorded as one.
    let failed: Vec<usize> = rows.iter().enumerate().filter(|(_, r)| r.outcome == Outcome::Fail).map(|(i, _)| i).take(10).collect();
    for (k, i) in failed.iter().enumerate() {
        let name = rows[*i].test.split('(').next().unwrap_or("").to_string();
        if let Ok((Some(_), _, again)) = dotnet_test(&cwd, &target, &format!("FullyQualifiedName={name}"), dir, &format!("{stem}-rerun-{k}"), stop) {
            rows[*i].attempts = 2;
            if !again.is_empty() && again.iter().all(|r| r.outcome == Outcome::Pass) {
                rows[*i].outcome = Outcome::Flaky;
            }
        }
    }
    for r in rows {
        let unit = if lunap { "dotnet:lunap".to_string() } else { format!("dotnet:{}", r.group) };
        out.rows.push((unit, r));
    }
    out
}
