//! When the ES-DE oracle writes the day's corpus measure under `Testing/esde-corpus` (EmuSen_Qt.md §4.4): plainly after a
//! run with no failure, or under a ruling of the tester's after a run whose only failures are rises.

use std::path::Path;

/// How a run may write the measure: `EMUSEN_ESDE_CORPUS` unset, `1`, or `ruled:<ruling id>`.
#[derive(Debug, PartialEq, Eq)]
pub enum Recording {
    Off,
    Plain,
    Ruled(String),
}

impl Recording {
    /// The recording the variable's value asks for.
    pub fn of(var: Option<&str>) -> Self {
        match var {
            None => Self::Off,
            Some(v) => v.strip_prefix("ruled:").map_or(Self::Plain, |id| Self::Ruled(id.trim().to_owned())),
        }
    }
}

/// Whether a worsened line (`<theme>: <column> <was> → <now>`) is a rise: a count of unknown or unsupported pairs that grew.
pub fn is_rise(line: &str) -> bool {
    line.split(": ").nth(1).is_some_and(|c| c.starts_with("unknown ") || c.starts_with("unsupported "))
}

/// Writes `measure` as `committed/file`: plainly when asked and nothing failed or rose, under a named ruling when the rises
/// are the only failures, its id and their count on the file's first line; whether it wrote.
pub fn record(committed: &Path, file: &str, measure: &str, rises: &[String], failures: &[String], how: &Recording) -> bool {
    let text = match how {
        Recording::Plain if rises.is_empty() && failures.is_empty() => measure.to_owned(),
        Recording::Ruled(id) if !id.is_empty() && failures.is_empty() => format!("# ruled {id}: {} rises\n{measure}", rises.len()),
        _ => return false,
    };
    std::fs::write(committed.join(file), text).unwrap();
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(what: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("emusen-corpus-{what}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_failing_run_leaves_the_committed_measure_alone() {
        let dir = scratch("plain");
        let rise = ["t: unsupported 8 → 10".to_owned()];
        assert!(!record(&dir, "a.tsv", "m\n", &[], &["a box off".to_owned()], &Recording::Plain));
        assert!(!dir.join("a.tsv").exists(), "a failing run re-recorded the baseline");
        assert!(!record(&dir, "b.tsv", "m\n", &[], &[], &Recording::Off));
        assert!(!record(&dir, "b.tsv", "m\n", &rise, &[], &Recording::Plain), "a plain run wrote over a rise");
        assert!(!dir.join("b.tsv").exists());
        assert!(record(&dir, "c.tsv", "m\n", &[], &[], &Recording::Plain));
        assert_eq!(std::fs::read_to_string(dir.join("c.tsv")).unwrap(), "m\n");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_ruled_rise_is_recorded_only_alone_and_with_its_ruling() {
        let dir = scratch("ruled");
        let rise = ["t: unsupported 8 → 10".to_owned()];
        let ruled = Recording::of(Some("ruled:R-20261111-87"));
        assert_eq!(ruled, Recording::Ruled("R-20261111-87".to_owned()));
        assert_eq!(Recording::of(Some("1")), Recording::Plain);
        assert_eq!(Recording::of(None), Recording::Off);
        assert!(record(&dir, "rises.tsv", "m\n", &rise, &[], &ruled), "rises only: written");
        assert_eq!(std::fs::read_to_string(dir.join("rises.tsv")).unwrap(), "# ruled R-20261111-87: 1 rises\nm\n");
        assert!(!record(&dir, "boxes.tsv", "m\n", &rise, &["box: a box off".to_owned()], &ruled), "a rise and a box failure: not written");
        assert!(!record(&dir, "noid.tsv", "m\n", &rise, &[], &Recording::of(Some("ruled:"))), "no id: not written");
        assert!(!dir.join("boxes.tsv").exists() && !dir.join("noid.tsv").exists());
        assert!(is_rise("t: unsupported 8 → 10") && is_rise("t: unknown 0 → 1"));
        assert!(!is_rise("t: rects_off 0 → 1") && !is_rise("t: no longer loads") && !is_rise("t: video undrawn"));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
