//! The loaded step: `theme-audit`, a Qt program of its own (Frontend/theme-check/audit), run as a process on one package; its
//! lines are read back as findings. This crate holds no Qt, since the back end's dev module links it.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::Finding;

/// The program's name on `PATH`, and the variable that names it anywhere else.
pub const PROGRAM: &str = "theme-audit";
pub const VARIABLE: &str = "EMUSEN_THEME_AUDIT";

/// `EMUSEN_THEME_AUDIT` when it is set, else `theme-audit` on `PATH`; none when neither is found.
pub fn program() -> Option<PathBuf> {
    if let Some(named) = std::env::var_os(VARIABLE).filter(|v| !v.is_empty()) {
        return Some(PathBuf::from(named));
    }
    std::env::split_paths(&std::env::var_os("PATH")?).map(|dir| dir.join(PROGRAM)).find(|p| p.is_file())
}

/// One line of the program's as a finding: `<real>/<file>:<line> <kind> <words>` with the folder shown as the caller named it.
fn finding(line: &str, real: &str, shown: &str) -> Finding {
    let parsed = line.strip_prefix(real).and_then(|rest| rest.strip_prefix('/')).and_then(|rest| {
        let (place, rest) = rest.split_once(' ')?;
        let (file, number) = place.rsplit_once(':')?;
        let (kind, words) = rest.split_once(' ')?;
        Some(Finding::new(&format!("{shown}/{file}"), number.parse().ok()?, kind, words.to_owned()))
    });
    parsed.unwrap_or_else(|| Finding::new(&format!("{shown}/theme.json"), 1, "audit", line.to_owned()))
}

/// The program's findings on the package at `real`, or its words when it could not run.
pub fn audit(program: &Path, real: &Path, shown: &str, state: Option<&Path>) -> Result<Vec<Finding>, String> {
    let mut command = Command::new(program);
    if let Some(dir) = state {
        command.arg("--state").arg(dir);
    }
    let real = real.display().to_string();
    let real = real.trim_end_matches('/');
    let out = command.arg(real).output().map_err(|e| format!("theme-check: {}: {e}", program.display()))?;
    if !matches!(out.status.code(), Some(0 | 1)) {
        let words = String::from_utf8_lossy(&out.stderr);
        return Err(format!("theme-check: {} did not run on {shown} ({}): {}", PROGRAM, out.status, words.trim()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).lines().filter(|l| !l.is_empty()).map(|l| finding(l, real, shown)).collect())
}
