//! The single-step harness over SingleStepTests' 65816 and SPC700 suites: registers, memory and the bus cycle by
//! cycle. The corpus is found through `EMUSEN_VENUSRT_CORPUS`; absent, the corpus tests pass unrun.
//! See VenusRT_Native.md §2.3 for the protocol, the capped cases and the controls.

pub mod cpu65816;
pub mod spc700;
pub mod spc700_cpu;
pub mod w65816;

use std::path::{Path, PathBuf};

pub use emusen_singlestep::{FileReport, Outcome, int, read_cases, run_cases, run_suite, suite_files, summary};
use emusen_native::json::Value;

pub const CORPUS_VARIABLE: &str = "EMUSEN_VENUSRT_CORPUS";

/// The suite's folder under the corpus, or None when the corpus is not here.
pub fn suite_dir(suite: &str) -> Option<PathBuf> {
    let root = std::env::var_os(CORPUS_VARIABLE)?;
    let dir = Path::new(&root).join("src").join(suite).join("v1");
    dir.is_dir().then_some(dir)
}

/// `[address, value]` pairs.
pub fn ram(v: &Value) -> Vec<(u32, u8)> {
    emusen_singlestep::ram(v, u32::MAX)
}

pub fn threads() -> usize {
    std::env::var("EMUSEN_VENUSRT_THREADS").ok().and_then(|t| t.parse().ok()).unwrap_or(4)
}
