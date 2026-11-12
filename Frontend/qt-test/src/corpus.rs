//! When the ES-DE oracle writes the day's corpus measure under `Testing/esde-corpus` (EmuSen_Qt.md §4.4): plainly after a
//! run with no failure, or under a ruling of the tester's after a run whose only failures are the rises that ruling's
//! file of settings accounts for, theme by theme. Rises and falls are said in the written file's header.

use std::collections::{BTreeMap, BTreeSet};
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

/// The rows of a corpus measure by theme, its columns by header name; `#` lines are notes, not rows.
pub fn rows(text: &str) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut lines = text.lines().filter(|l| !l.starts_with('#') && !l.is_empty());
    let header: Vec<&str> = lines.next().unwrap_or_default().split('\t').collect();
    lines
        .map(|l| {
            let row: BTreeMap<String, String> = header.iter().zip(l.split('\t')).map(|(h, v)| ((*h).to_owned(), v.to_owned())).collect();
            (row["theme"].clone(), row)
        })
        .collect()
}

/// A count of a theme's that moved between two measures, read from a line `<theme>: <column> <was> → <now>`.
#[derive(Debug, PartialEq)]
pub struct Change {
    pub theme: String,
    pub column: String,
    pub was: f64,
    pub now: f64,
}

impl Change {
    pub fn parse(line: &str) -> Option<Self> {
        let (theme, rest) = line.split_once(": ")?;
        let (column, figures) = rest.split_once(' ')?;
        let (was, now) = figures.split_once(" → ")?;
        Some(Change { theme: theme.to_owned(), column: column.to_owned(), was: was.parse().ok()?, now: now.parse().ok()? })
    }
}

/// Whether a worsened line is a rise: a count of unknown or unsupported pairs that grew.
pub fn is_rise(line: &str) -> bool {
    Change::parse(line).is_some_and(|c| (c.column == "unknown" || c.column == "unsupported") && c.now > c.was)
}

/// The counts that fell from `old` to `new` (set, unknown, unsupported) and the themes `new` no longer has.
pub fn falls(old: &str, new: &str) -> Vec<String> {
    let (old, new) = (rows(old), rows(new));
    let mut out = vec![];
    for (theme, was) in &old {
        let Some(now) = new.get(theme) else {
            out.push(format!("{theme}: missing from the new measure"));
            continue;
        };
        for c in ["set", "unknown", "unsupported"] {
            let num = |r: &BTreeMap<String, String>| r.get(c).and_then(|v| v.parse::<f64>().ok());
            if let (Some(a), Some(b)) = (num(was), num(now)) {
                if b < a {
                    out.push(format!("{theme}: {c} {a} → {b}"));
                }
            }
        }
    }
    out
}

/// What a ruling covers: the settings (`type.property` pairs) it moved into the unsupported count, and those it moved out
/// of it by being drawn, from `Testing/esde-corpus/rulings/<id>.tsv`.
#[derive(Debug, PartialEq)]
pub struct Cover {
    pub id: String,
    pub withdrawn: BTreeSet<String>,
    pub drawn: BTreeSet<String>,
}

impl Cover {
    /// The ruling's file beside the measures, or why there is none to read.
    pub fn read(committed: &Path, id: &str) -> Result<Self, String> {
        let path = committed.join("rulings").join(format!("{id}.tsv"));
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}: no file names what {id} covers", path.display()))?;
        Self::parse(id, &text)
    }

    /// Lines `pair<TAB>withdrawn|drawn` under a `pair\tway` header; `#` lines are notes.
    pub fn parse(id: &str, text: &str) -> Result<Self, String> {
        let mut cover = Cover { id: id.to_owned(), withdrawn: BTreeSet::new(), drawn: BTreeSet::new() };
        for line in text.lines().filter(|l| !l.starts_with('#') && !l.is_empty() && *l != "pair\tway") {
            match line.split_once('\t') {
                Some((pair, "withdrawn")) => cover.withdrawn.insert(pair.to_owned()),
                Some((pair, "drawn")) => cover.drawn.insert(pair.to_owned()),
                _ => return Err(format!("{id}: the line {line:?} is not pair<TAB>withdrawn|drawn")),
            };
        }
        if cover.withdrawn.is_empty() && cover.drawn.is_empty() {
            return Err(format!("{id}: its file names no setting"));
        }
        Ok(cover)
    }

    /// The rise in a theme's unsupported count the ruling accounts for, from its pairs' states now: each withdrawn pair it
    /// sets counted unsupported, less each drawn pair it sets counted drawn.
    pub fn expected(&self, pairs: &BTreeMap<String, String>) -> f64 {
        let has = |set: &BTreeSet<String>, state: &str| set.iter().filter(|p| pairs.get(*p).is_some_and(|s| s == state)).count();
        has(&self.withdrawn, "unsupported") as f64 - has(&self.drawn, "set") as f64
    }

    /// The rises this ruling does not account for, each with why: a rise in unknown pairs, or one in unsupported pairs that
    /// is not exactly the theme's count of the ruling's settings. `pairs` holds each theme's `type.property` states now.
    pub fn uncovered(&self, rises: &[Change], pairs: &BTreeMap<String, BTreeMap<String, String>>) -> Vec<String> {
        let none = BTreeMap::new();
        rises
            .iter()
            .filter_map(|r| {
                let rise = r.now - r.was;
                if r.column != "unsupported" {
                    return Some(format!("{}: {} {} → {}: {} covers unsupported pairs only", r.theme, r.column, r.was, r.now, self.id));
                }
                let want = self.expected(pairs.get(&r.theme).unwrap_or(&none));
                (rise != want).then(|| format!("{}: unsupported {} → {}: a rise of {rise}, where the theme's settings of {} account for {want}", r.theme, r.was, r.now, self.id))
            })
            .collect()
    }
}

/// Writes `measure` as `committed/file` when asked and nothing failed: under a named ruling its id and the rises' count
/// on the first line; `notes` (each rise and fall) as `#` lines after it, so a passing run leaves them readable. Whether it wrote.
pub fn record(committed: &Path, file: &str, measure: &str, notes: &[String], failures: &[String], how: &Recording) -> bool {
    let notes: String = notes.iter().map(|n| format!("# {n}\n")).collect();
    let text = match how {
        Recording::Plain if failures.is_empty() => format!("{notes}{measure}"),
        Recording::Ruled(id) if !id.is_empty() && failures.is_empty() => {
            let rises = notes.lines().filter(|l| l.starts_with("# rise: ")).count();
            format!("# ruled {id}: {rises} rises\n{notes}{measure}")
        }
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

    fn pairs(theme: &str, states: &[(&str, &str)]) -> BTreeMap<String, BTreeMap<String, String>> {
        BTreeMap::from([(theme.to_owned(), states.iter().map(|(p, s)| ((*p).to_owned(), (*s).to_owned())).collect())])
    }

    #[test]
    fn a_failing_run_leaves_the_committed_measure_alone() {
        let dir = scratch("plain");
        assert!(!record(&dir, "a.tsv", "m\n", &[], &["a box off".to_owned()], &Recording::Plain));
        assert!(!dir.join("a.tsv").exists(), "a failing run re-recorded the baseline");
        assert!(!record(&dir, "b.tsv", "m\n", &[], &[], &Recording::Off));
        assert!(!dir.join("b.tsv").exists());
        assert!(record(&dir, "c.tsv", "m\n", &[], &[], &Recording::Plain));
        assert_eq!(std::fs::read_to_string(dir.join("c.tsv")).unwrap(), "m\n");
        assert!(record(&dir, "d.tsv", "m\n", &["fall: t: set 9 → 8".to_owned()], &[], &Recording::Plain));
        assert_eq!(std::fs::read_to_string(dir.join("d.tsv")).unwrap(), "# fall: t: set 9 → 8\nm\n", "a fall is said in the file");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_ruled_run_writes_with_its_ruling_and_its_rises_said() {
        let dir = scratch("ruled");
        let ruled = Recording::of(Some("ruled:R-20261111-87"));
        assert_eq!(ruled, Recording::Ruled("R-20261111-87".to_owned()));
        assert_eq!(Recording::of(Some("1")), Recording::Plain);
        assert_eq!(Recording::of(None), Recording::Off);
        let notes = ["rise: t: unsupported 8 → 10".to_owned()];
        assert!(record(&dir, "rises.tsv", "m\n", &notes, &[], &ruled));
        assert_eq!(std::fs::read_to_string(dir.join("rises.tsv")).unwrap(), "# ruled R-20261111-87: 1 rises\n# rise: t: unsupported 8 → 10\nm\n");
        assert!(!record(&dir, "boxes.tsv", "m\n", &notes, &["box: a box off".to_owned()], &ruled), "a rise and a box failure: not written");
        assert!(!record(&dir, "noid.tsv", "m\n", &notes, &[], &Recording::of(Some("ruled:"))), "no id: not written");
        assert!(!dir.join("boxes.tsv").exists() && !dir.join("noid.tsv").exists());
        assert!(is_rise("t: unsupported 8 → 10") && is_rise("t: unknown 0 → 1"));
        assert!(!is_rise("t: rects_off 0 → 1") && !is_rise("t: no longer loads") && !is_rise("t: video undrawn"));
        assert_eq!(rows("# ruled R-1: 0 rises\n# rise: x\ntheme\tunsupported\nt\t8\n")["t"]["unsupported"], "8");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    // R-20261111-87's case: a theme rises by the withdrawn settings it sets, less one where it also sets the drawn one.
    #[test]
    fn a_ruling_covers_only_the_rise_its_settings_make() {
        let cover = Cover::parse("R-1", "# note\npair\tway\nhelpsystem.posDimmed\twithdrawn\nsound.path\twithdrawn\ntext.containerVerticalSnap\tdrawn\n").unwrap();
        let two = pairs("t", &[("helpsystem.posDimmed", "unsupported"), ("sound.path", "unsupported"), ("text.color", "set")]);
        let rise = |was: f64, now: f64, column: &str| Change { theme: "t".into(), column: column.into(), was, now };
        assert!(cover.uncovered(&[rise(8.0, 10.0, "unsupported")], &two).is_empty());
        assert_eq!(cover.uncovered(&[rise(8.0, 11.0, "unsupported")], &two).len(), 1, "one more than its settings");
        assert_eq!(cover.uncovered(&[rise(0.0, 2.0, "unknown")], &two).len(), 1, "an unknown rise is never a ruling's, even of the size its settings make");
        let snapped = pairs("t", &[("helpsystem.posDimmed", "unsupported"), ("sound.path", "unsupported"), ("text.containerVerticalSnap", "set")]);
        assert!(cover.uncovered(&[rise(8.0, 9.0, "unsupported")], &snapped).is_empty(), "one less where the drawn one is set");
        assert_eq!(cover.uncovered(&[rise(8.0, 9.0, "unsupported")], &pairs("u", &[])).len(), 1, "a theme that sets none of them");
        assert!(Cover::parse("R-2", "pair\tway\n").is_err() && Cover::parse("R-3", "a.b\tmaybe\n").is_err());
        assert_eq!(Change::parse("t: unsupported 8 → 10"), Some(rise(8.0, 10.0, "unsupported")));
    }

    #[test]
    fn a_fall_or_a_missing_theme_is_said() {
        let old = "theme\tset\tunknown\tunsupported\nA\t10\t1\t5\nB\t3\t0\t0\n";
        let new = "theme\tset\tunknown\tunsupported\nA\t9\t0\t7\nC\t1\t0\t0\n";
        assert_eq!(falls(old, new), ["A: set 10 → 9", "A: unknown 1 → 0", "B: missing from the new measure"]);
        assert!(falls(old, old).is_empty());
    }
}
