//! Shared ES-DE corpus comparison helpers used by `corpus_soft` and `oracle`.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use emusen_qt_test::corpus;

pub fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Rows that worsened from `old` to `new` across corpus and watched columns.
pub fn worsened(old: &str, new: &str) -> Vec<String> {
    let (old, new) = (corpus::rows(old), corpus::rows(new));
    let mut out = vec![];
    for (theme, now) in &new {
        let Some(was) = old.get(theme) else { continue };
        let num = |r: &BTreeMap<String, String>, c: &str| r.get(c).and_then(|v| v.parse::<f64>().ok());
        if was.get("loads").map(String::as_str) == Some("yes") && now.get("loads").is_some_and(|v| v != "yes") {
            out.push(format!("{theme}: no longer loads"));
        }
        for c in ["unknown", "unsupported", "mae_system_1280", "mae_gamelist_1280", "mae_system_1920", "mae_gamelist_1920", "rects_off", "mae_viewer_1280", "mae_viewer_1920", "viewer_off"] {
            if let (Some(a), Some(b)) = (num(was, c), num(now, c)) {
                // MAE moves by up to 0.02 between runs (measured on 2026-11-09); a count by one.
                if b > a + if c.starts_with("mae") { 0.05 } else { 0.5 } {
                    out.push(format!("{theme}: {c} {a} → {b}"));
                }
            }
        }
        let kinds = |r: &BTreeMap<String, String>| r.get("undrawn").map(|v| v.split(',').filter(|k| *k != "-").map(str::to_owned).collect::<Vec<_>>()).unwrap_or_default();
        for k in kinds(now).into_iter().filter(|k| !kinds(was).contains(k)) {
            out.push(format!("{theme}: {k} undrawn"));
        }
    }
    out
}

/// Watched figures that fell from `old` to `new` by more than a run's noise.
pub fn fell(old: &str, new: &str) -> Vec<String> {
    let (old, new) = (corpus::rows(old), corpus::rows(new));
    let mut out = vec![];
    for (theme, now) in &new {
        let Some(was) = old.get(theme) else { continue };
        for c in ["mae_system_1280", "mae_gamelist_1280", "mae_system_1920", "mae_gamelist_1920", "mae_viewer_1280", "mae_viewer_1920"] {
            let num = |r: &BTreeMap<String, String>| r.get(c).and_then(|v| v.parse::<f64>().ok());
            if let (Some(a), Some(b)) = (num(was), num(now)) {
                if b < a - 0.05 {
                    out.push(format!("{theme}: {c} {a} → {b}"));
                }
            }
        }
    }
    out
}

/// Runs pure-CPU `check::measures` over the corpus XML tree and formats the report table.
pub fn measure_corpus(watched: &BTreeMap<String, BTreeMap<String, String>>) -> (String, bool, BTreeMap<String, BTreeMap<String, String>>) {
    use emusen_esde::check::{self, Table};
    let corpus_dir = PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/emusen/bigpicture/corpus/xml");
    let measured = check::measures(&[corpus_dir], &check::SYSTEMS);
    let report = check::report_of(&measured, Table::Summary);
    let pairs: BTreeMap<String, BTreeMap<String, String>> =
        measured.iter().map(|m| (m.theme.clone(), m.properties.iter().map(|(p, s)| (p.clone(), (*s).to_owned())).collect())).collect();
    let columns = ["mae_system_1280", "mae_gamelist_1280", "mae_system_1920", "mae_gamelist_1920", "rects_off", "mae_viewer_1280", "mae_viewer_1920", "viewer_off"];
    let measure: Vec<String> = report
        .text
        .lines()
        .enumerate()
        .map(|(i, l)| {
            if l.starts_with('#') {
                l.to_owned()
            } else if i == 0 {
                format!("{l}\t{}", columns.join("\t"))
            } else {
                let theme = l.split('\t').next().unwrap();
                let row = watched.get(theme);
                let cells: Vec<String> = columns.iter().map(|c| row.and_then(|r| r.get(*c)).cloned().unwrap_or_else(|| "-".to_owned())).collect();
                format!("{l}\t{}", cells.join("\t"))
            }
        })
        .collect();
    (measure.join("\n") + "\n", report.all_load, pairs)
}
