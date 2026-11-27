// CPU/software mapping checks: catalog coverage, theme load checks, and per-kind non-shader pixel tests.

mod common;

use std::collections::BTreeSet;

use common::mapping::{all_cases, capabilities, element, errors, is_gpu_case, run_cases, Assets, Case, H, W};
use common::Scratch;
use emusen_esde::scene::mapping::{self, Status};
use emusen_qt_test::Harness;

const SOFT_KINDS: [&str; 8] = ["clock", "datetime", "gamelistinfo", "gameselector", "helpsystem", "systemstatus", "text", "textlist"];

fn run_kind_soft(kind: &str) {
    assert!(emusen_esde_qt::marks::mark_named("star").is_some(), "the module is linked into this test");
    // Held clocks require the basic render loop; software offscreen uses no compositor.
    std::env::set_var("QSG_RENDER_LOOP", "basic");
    let only = std::env::var("ESDE_MAPPING_ONLY").ok();
    let cases: Vec<Case> = all_cases()
        .into_iter()
        .filter(|c| c.kind == kind && !is_gpu_case(c) && mapping::status(&c.kind, &c.property) == Some(&Status::Drawn))
        .filter(|c| only.as_deref().is_none_or(|o| o.split(',').any(|w| w == c.to_string())))
        .collect();
    assert!(!cases.is_empty(), "no drawn software cases for {kind}");
    let mut harness = Harness::new();
    let mut scene = harness.load(common::VIEW, W, H);
    run_cases(&mut scene, &format!("mapping-{kind}"), &cases);
}

#[test]
fn every_pair_the_mapping_marks_drawn_has_a_case_and_every_case_is_mapped() {
    let claimed: BTreeSet<(String, String)> =
        mapping::pairs().filter(|(_, _, s)| **s == Status::Drawn).map(|(k, p, _)| (k.to_owned(), p.to_owned())).collect();
    let all = all_cases();
    let proved: BTreeSet<(String, String)> = all.iter().map(|c| (c.kind.clone(), c.property.clone())).collect();
    let unproved: Vec<_> = claimed.difference(&proved).collect();
    let unmapped: Vec<_> = proved.difference(&claimed).filter(|(k, p)| !matches!(mapping::status(k, p), Some(Status::Later(_)))).collect();
    assert!(unproved.is_empty(), "drawn in mapping.tsv with no case: {unproved:?}");
    assert!(unmapped.is_empty(), "a case for a pair mapping.tsv marks neither drawn nor later: {unmapped:?}");
    for c in all.iter().filter(|c| mapping::status(&c.kind, &c.property) == Some(&Status::Drawn) && !is_gpu_case(c)) {
        assert!(SOFT_KINDS.contains(&c.kind.as_str()), "software case {c} missing from SOFT_KINDS");
    }
}

// Heard pairs name the test that hears them as `<crate>/tests/file.rs::fn`.
#[test]
fn every_heard_pair_names_a_test_that_hears_it() {
    let heard: Vec<(String, String, String)> = mapping::pairs()
        .filter_map(|(k, p, s)| match s {
            Status::Heard(t) => Some((k.to_owned(), p.to_owned(), t.clone())),
            _ => None,
        })
        .collect();
    assert!(!heard.is_empty());
    for (kind, property, test) in heard {
        let (file, name) = test.split_once("::").unwrap_or_else(|| panic!("{kind}.{property}: {test} is not file::fn"));
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(file);
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{kind}.{property}: {}: {e}", path.display()));
        assert!(text.contains(&format!("#[test]\nfn {name}(")), "{kind}.{property}: no test {name} in {file}");
    }
}

#[test]
fn every_case_s_two_themes_load_without_errors() {
    let dir = Scratch::new("mapping-load");
    let assets = Assets::new(&dir.0);
    let mut failures = vec![];
    for (n, c) in all_cases().iter().enumerate() {
        for (side, value) in [("a", &c.a), ("b", &c.b)] {
            let theme = dir.path(&format!("theme-{n}-{side}"));
            common::write_theme(&theme, capabilities(c), &element(c, value, &assets));
            failures.extend(errors(&theme).into_iter().map(|e| format!("{c} = {value}: {e}")));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn drawn_pairs_paint_distinct_pixels_for_clock() {
    run_kind_soft("clock");
}

#[test]
fn drawn_pairs_paint_distinct_pixels_for_datetime() {
    run_kind_soft("datetime");
}

#[test]
fn drawn_pairs_paint_distinct_pixels_for_gamelistinfo() {
    run_kind_soft("gamelistinfo");
}

#[test]
fn drawn_pairs_paint_distinct_pixels_for_gameselector() {
    run_kind_soft("gameselector");
}

#[test]
fn drawn_pairs_paint_distinct_pixels_for_helpsystem() {
    run_kind_soft("helpsystem");
}

#[test]
fn drawn_pairs_paint_distinct_pixels_for_systemstatus() {
    run_kind_soft("systemstatus");
}

#[test]
fn drawn_pairs_paint_distinct_pixels_for_text() {
    run_kind_soft("text");
}

#[test]
fn drawn_pairs_paint_distinct_pixels_for_textlist() {
    run_kind_soft("textlist");
}
