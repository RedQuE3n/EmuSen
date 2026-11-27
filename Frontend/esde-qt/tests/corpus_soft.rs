//! Pure-CPU ES-DE XML corpus verification split out of `oracle.rs` so it runs without GPU lock.

mod common;

use std::collections::BTreeMap;

use common::corpus::{fell, measure_corpus, repo, worsened};
use common::Scratch;
use emusen_qt_test::corpus::{self, Change, Cover, Recording, is_rise, record};

#[test]
fn the_corpus_measures_load_without_regression() {
    let watched = BTreeMap::new();
    let (measure, all_load, pairs) = measure_corpus(&watched);
    let out = common::compare_dir().join("../oracle");
    std::fs::create_dir_all(&out).unwrap();
    let committed = repo().join("Testing/esde-corpus");
    let how = Recording::of(std::env::var("EMUSEN_ESDE_CORPUS").ok().as_deref());
    let (mut failures, mut rises, mut rise_lines, mut notes) = (vec![], vec![], vec![], vec![]);
    let mut earlier: Vec<std::path::PathBuf> = std::fs::read_dir(&committed).unwrap().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "tsv")).collect();
    earlier.sort();
    if let Some(last) = earlier.last() {
        let last_text = std::fs::read_to_string(last).unwrap();
        let against = last.file_name().unwrap().to_string_lossy().into_owned();
        for w in worsened(&last_text, &measure) {
            let line = format!("against {against}: {w}");
            match Change::parse(&w).filter(|_| is_rise(&w)) {
                Some(c) => {
                    rises.push(c);
                    notes.push(format!("rise: {line}"));
                    rise_lines.push(line);
                }
                None => failures.push(line),
            }
        }
        for f in corpus::falls(&last_text, &measure) {
            let line = format!("against {against}: {f}");
            if how == Recording::Off || f.ends_with("missing from the new measure") {
                failures.push(format!("{line}: a fall (EmuSen_Qt.md §4.4)"));
            }
            notes.push(format!("fall: {line}"));
        }
    }
    if !all_load {
        failures.push("a theme of the corpus does not load".to_owned());
    }
    let _scratch = Scratch::new("corpus-soft");
    std::fs::write(out.join("corpus.tsv"), &measure).unwrap();
    match &how {
        Recording::Ruled(id) if !id.is_empty() => match Cover::read(&committed, id) {
            Ok(cover) => failures.extend(cover.uncovered(&rises, &pairs).into_iter().map(|u| format!("not covered by {id}: {u}"))),
            Err(e) => failures.push(e),
        },
        _ => failures.extend(rise_lines),
    }
    for n in &notes {
        eprintln!("{n}");
    }
    if how != Recording::Off {
        let _ = record(&committed, "corpus-soft.tsv", &measure, &notes, &failures, &how);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn a_row_that_worsens_is_named() {
    let old = "theme\tloads\tunknown\tunsupported\tundrawn\tmae_system_1280\nA\tyes\t0\t5\t-\t1.00\nB\tyes\t0\t2\t-\t-\n";
    let new = "theme\tloads\tunknown\tunsupported\tundrawn\tmae_system_1280\nA\tno\t0\t6\tvideo\t1.20\nB\tyes\t0\t2\t-\t3.00\nC\tno\t1\t9\t-\t-\n";
    assert_eq!(worsened(old, new), ["A: no longer loads", "A: unsupported 5 → 6", "A: mae_system_1280 1 → 1.2", "A: video undrawn"]);
    assert!(worsened(new, new).is_empty());
    let watched_only = "theme\tmae_system_1280\nA\t1.20\nB\t3.00\n";
    assert_eq!(worsened(old, watched_only), ["A: mae_system_1280 1 → 1.2"]);
    assert_eq!(fell(new, old), ["A: mae_system_1280 1.2 → 1"]);
    assert!(fell(old, new).is_empty());
}
