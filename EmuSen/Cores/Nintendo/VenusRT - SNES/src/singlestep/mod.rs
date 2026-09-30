//! The single-step harness over SingleStepTests' 65816 and SPC700 suites: registers, memory and the bus cycle by
//! cycle. The corpus is found through `EMUSEN_VENUSRT_CORPUS`; absent, the corpus tests pass unrun.
//! See VenusRT_Native.md §2.3 for the protocol, the capped cases and the controls.

pub mod spc700;
pub mod w65816;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use emusen_native::json::{self, Value};

pub const CORPUS_VARIABLE: &str = "EMUSEN_VENUSRT_CORPUS";

/// The suite's folder under the corpus, or None when the corpus is not here.
pub fn suite_dir(suite: &str) -> Option<PathBuf> {
    let root = std::env::var_os(CORPUS_VARIABLE)?;
    let dir = Path::new(&root).join("src").join(suite).join("v1");
    dir.is_dir().then_some(dir)
}

pub fn suite_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("the suite's folder")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    files
}

pub fn read_cases(path: &Path) -> Vec<Value> {
    let text = std::fs::read(path).expect("a suite file");
    match json::parse(&text) {
        Ok(Value::Array(cases)) => cases,
        other => panic!("{}: not an array of cases ({:?})", path.display(), other.err()),
    }
}

pub fn int(v: &Value, key: &str) -> i64 {
    v.get(key).and_then(Value::as_i64).unwrap_or_else(|| panic!("no integer {key}"))
}

/// `[address, value]` pairs.
pub fn ram(v: &Value) -> Vec<(u32, u8)> {
    v.get("ram")
        .and_then(Value::as_array)
        .expect("ram")
        .iter()
        .map(|pair| {
            let p = pair.as_array().expect("a pair");
            (p[0].as_i64().expect("an address") as u32, p[1].as_i64().expect("a value") as u8)
        })
        .collect()
}

/// What one case showed: which of the three checks held, and the first difference.
#[derive(Clone, Debug, Default)]
pub struct Outcome {
    pub registers: bool,
    pub memory: bool,
    pub cycles: bool,
    /// The suite's list ends at its cycle cap mid-instruction, so only the cycles are compared.
    pub capped: bool,
    pub first_difference: Option<String>,
}

impl Outcome {
    pub fn passed(&self) -> bool {
        self.registers && self.memory && self.cycles
    }
}

/// One file's tally.
#[derive(Clone, Debug, Default)]
pub struct FileReport {
    pub file: String,
    pub cases: usize,
    pub passed: usize,
    pub registers: usize,
    pub memory: usize,
    pub cycles: usize,
    pub capped: usize,
    pub first_failure: Option<String>,
}

impl FileReport {
    pub fn add(&mut self, name: &str, o: &Outcome) {
        self.cases += 1;
        self.passed += o.passed() as usize;
        self.registers += o.registers as usize;
        self.memory += o.memory as usize;
        self.cycles += o.cycles as usize;
        self.capped += o.capped as usize;
        if !o.passed() && self.first_failure.is_none() {
            self.first_failure = Some(format!("{name}: {}", o.first_difference.as_deref().unwrap_or("?")));
        }
    }
}

/// Every file of a suite through `run`, on a few threads; the reports in file order.
pub fn run_suite(files: &[PathBuf], threads: usize, run: impl Fn(&Path) -> FileReport + Sync) -> Vec<FileReport> {
    let next = Mutex::new(0usize);
    let reports = Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for _ in 0..threads.max(1) {
            s.spawn(|| {
                loop {
                    let i = {
                        let mut n = next.lock().unwrap();
                        *n += 1;
                        *n - 1
                    };
                    let Some(path) = files.get(i) else { break };
                    let report = run(path);
                    reports.lock().unwrap().push((i, report));
                }
            });
        }
    });
    let mut reports = reports.into_inner().unwrap();
    reports.sort_by_key(|(i, _)| *i);
    reports.into_iter().map(|(_, r)| r).collect()
}

/// The suite's totals as one line: cases, passed, and each check.
pub fn summary(reports: &[FileReport]) -> String {
    let sum = |f: fn(&FileReport) -> usize| reports.iter().map(f).sum::<usize>();
    format!(
        "{} files, {} cases: {} passed; registers {}, memory {}, cycles {}; {} capped",
        reports.len(),
        sum(|r| r.cases),
        sum(|r| r.passed),
        sum(|r| r.registers),
        sum(|r| r.memory),
        sum(|r| r.cycles),
        sum(|r| r.capped)
    )
}

pub fn threads() -> usize {
    std::env::var("EMUSEN_VENUSRT_THREADS").ok().and_then(|t| t.parse().ok()).unwrap_or(4)
}
