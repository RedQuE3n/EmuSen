//! The shared half of the single-step CPU harnesses: load a suite's cases, grade each, tally per file, report.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use emusen_native::json::{self, Value};

/// The `.json` files of a suite's folder, sorted.
pub fn suite_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("the suite's folder")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    files
}

/// The cases of one suite file.
pub fn read_cases(path: &Path) -> Vec<Value> {
    match json::parse(&std::fs::read(path).expect("a suite file")) {
        Ok(Value::Array(cases)) => cases,
        other => panic!("{}: not an array of cases ({:?})", path.display(), other.err()),
    }
}

pub fn int(v: &Value, key: &str) -> i64 {
    v.get(key).and_then(Value::as_i64).unwrap_or_else(|| panic!("no integer {key}"))
}

/// The `[address, value]` pairs under `ram`, each address masked with `mask`.
pub fn ram(v: &Value, mask: u32) -> Vec<(u32, u8)> {
    v.get("ram")
        .and_then(Value::as_array)
        .expect("ram")
        .iter()
        .map(|pair| {
            let p = pair.as_array().expect("a pair");
            (p[0].as_i64().expect("an address") as u32 & mask, p[1].as_i64().expect("a value") as u8)
        })
        .collect()
}

/// The index of the first entry where the two lists differ, or where the shorter ends; None when they are equal.
pub fn first_mismatch<W, G>(want: &[W], got: &[G], same: impl Fn(&W, &G) -> bool) -> Option<usize> {
    let at = want.iter().zip(got).position(|(w, g)| !same(w, g));
    at.or_else(|| (want.len() != got.len()).then(|| want.len().min(got.len())))
}

/// What one case showed: which of the three checks held, and the first difference.
#[derive(Clone, Debug, Default)]
pub struct Outcome {
    pub registers: bool,
    pub memory: bool,
    pub cycles: bool,
    /// The suite's list ends at its cycle cap mid-instruction, so only the cycles are compared.
    pub capped: bool,
    /// A failure with a documented cause, counted apart from the rest.
    pub explained: bool,
    pub first_difference: Option<String>,
}

impl Outcome {
    /// Each check passes when its difference is None; the first difference reported is registers, then memory, then cycles.
    pub fn from_checks(registers: Option<String>, memory: Option<String>, cycles: Option<String>) -> Outcome {
        Outcome { registers: registers.is_none(), memory: memory.is_none(), cycles: cycles.is_none(), capped: false, explained: false, first_difference: registers.or(memory).or(cycles) }
    }

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
    /// Failing cases without a documented cause.
    pub unexplained: usize,
    pub first_failure: Option<String>,
    /// Cases whose registers and memory both end as the suite's, the cycles aside.
    pub state: usize,
}

impl FileReport {
    pub fn add(&mut self, name: &str, o: &Outcome) {
        self.cases += 1;
        self.passed += o.passed() as usize;
        self.registers += o.registers as usize;
        self.memory += o.memory as usize;
        self.cycles += o.cycles as usize;
        self.state += (o.registers && o.memory) as usize;
        self.capped += o.capped as usize;
        self.unexplained += (!o.passed() && !o.explained) as usize;
        if !o.passed() && self.first_failure.is_none() {
            self.first_failure = Some(format!("{name}: {}", o.first_difference.as_deref().unwrap_or("?")));
        }
    }
}

/// The first `limit` cases of one file, each graded by `grade`.
pub fn run_cases(path: &Path, limit: usize, mut grade: impl FnMut(&Value) -> Outcome) -> FileReport {
    let mut report = FileReport { file: path.file_name().unwrap().to_string_lossy().into_owned(), ..FileReport::default() };
    for case in read_cases(path).iter().take(limit) {
        report.add(case.get("name").and_then(Value::as_str).unwrap_or("?"), &grade(case));
    }
    report
}

/// Every file of a suite through `run`, on `threads` threads; the reports in file order.
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

/// Up to four threads, fewer on a small machine.
pub fn default_threads() -> usize {
    std::thread::available_parallelism().map_or(2, |n| n.get().min(4))
}

/// The files' tallies summed into one.
pub fn total(reports: &[FileReport]) -> FileReport {
    let mut t = FileReport::default();
    for r in reports {
        t.cases += r.cases;
        t.passed += r.passed;
        t.registers += r.registers;
        t.memory += r.memory;
        t.cycles += r.cycles;
        t.capped += r.capped;
        t.unexplained += r.unexplained;
        t.state += r.state;
    }
    t
}

/// The suite's totals as one line: cases, passed, and each check.
pub fn summary(reports: &[FileReport]) -> String {
    let t = total(reports);
    format!("{} files, {} cases: {} passed; registers {}, memory {}, cycles {}; {} capped", reports.len(), t.cases, t.passed, t.registers, t.memory, t.cycles, t.capped)
}

/// Prints, for up to `limit` files that missed a case, the counts and the first failure.
pub fn print_misses(reports: &[FileReport], limit: usize) {
    for r in reports.iter().filter(|r| r.passed != r.cases).take(limit) {
        eprintln!("  {}: {} of {}; {}", r.file, r.passed, r.cases, r.first_failure.as_deref().unwrap_or(""));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_mismatch_names_the_index_or_the_shorter_length() {
        let eq = |a: &i32, b: &i32| a == b;
        assert_eq!(first_mismatch(&[1, 2, 3], &[1, 2, 3], eq), None);
        assert_eq!(first_mismatch(&[1, 2, 3], &[1, 9, 3], eq), Some(1));
        assert_eq!(first_mismatch(&[1, 2, 3], &[1, 2], eq), Some(2));
    }

    #[test]
    fn from_checks_reports_registers_first_and_passes_only_when_all_hold() {
        assert!(Outcome::from_checks(None, None, None).passed());
        let o = Outcome::from_checks(Some("r".into()), Some("m".into()), Some("c".into()));
        assert!(!o.registers && !o.memory && !o.cycles);
        assert_eq!(o.first_difference.as_deref(), Some("r"));
        assert_eq!(Outcome::from_checks(None, None, Some("c".into())).first_difference.as_deref(), Some("c"));
    }

    #[test]
    fn a_suite_file_is_loaded_graded_tallied_and_run_in_order() {
        let dir = std::env::temp_dir().join(format!("emusen-singlestep-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for (name, body) in [("a.json", r#"[{"name":"x","ram":[[65537,7]]},{"name":"y"},{"name":"z"}]"#), ("b.json", r#"[{"name":"w"}]"#)] {
            std::fs::write(dir.join(name), body).unwrap();
        }
        let files = suite_files(&dir);
        assert_eq!(files.len(), 2);
        assert_eq!(ram(&read_cases(&files[0])[0], 0xFFFF), vec![(1, 7)]);
        let reports = run_suite(&files, 2, |p| {
            run_cases(p, 2, |c| {
                let bad = c.get("name").and_then(Value::as_str) == Some("y");
                Outcome::from_checks(bad.then(|| "y differs".to_string()), None, None)
            })
        });
        assert_eq!(reports.iter().map(|r| r.file.as_str()).collect::<Vec<_>>(), ["a.json", "b.json"]);
        let t = total(&reports);
        assert_eq!((t.cases, t.passed, t.unexplained), (3, 2, 1));
        assert_eq!(reports[0].first_failure.as_deref(), Some("y: y differs"));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
