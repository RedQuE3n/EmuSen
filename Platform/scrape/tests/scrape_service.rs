//! The scraper's statuses held to `emusen_scrape_service.h`, its one surface since 2.0, and the handle API of
//! `emusen_scrape.h` held gone: no `emusen_scrape_` export is left (EmuSen_RustPlatform.md §47.25).

use emusen_scrape::abi::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn platform() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// `#define EMUSEN_SCRAPE_<status> (<number>)` of the service's header, the version and name defines left out.
fn statuses() -> BTreeMap<String, i64> {
    read(&platform().join("include/emusen_scrape_service.h"))
        .lines()
        .filter_map(|l| l.trim().strip_prefix("#define EMUSEN_SCRAPE_"))
        .filter(|l| !l.starts_with("SERVICE"))
        .filter_map(|l| {
            let mut words = l.split_whitespace();
            let name = words.next()?;
            let value = words.next()?.trim_matches(|c| c == '(' || c == ')').parse().ok()?;
            Some((name.to_string(), value))
        })
        .collect()
}

#[test]
fn every_status_is_the_service_header_s() {
    let rust: BTreeMap<String, i64> = [
        ("NULL", NULL),
        ("BAD_STRING", BAD_STRING),
        ("NOT_SUPPORTED", NOT_SUPPORTED),
        ("NEWER_FILE", NEWER_FILE),
        ("DATABASE", DATABASE),
        ("NO_SQLITE", NO_SQLITE),
        ("BAD_REQUEST", BAD_REQUEST),
        ("BUSY", BUSY),
        ("IO", IO),
        ("NO_LIBRARY", NO_LIBRARY),
        ("STORE_IN_USE", STORE_IN_USE),
    ]
    .into_iter()
    .map(|(n, v)| (n.to_string(), v as i64))
    .collect();
    assert_eq!(statuses(), rust);
}

/// The scraper's own statuses sit in the band §30.3 gave it.
#[test]
fn the_statuses_are_in_the_scraper_s_band() {
    for status in [NEWER_FILE, DATABASE, NO_SQLITE, BAD_REQUEST, BUSY, IO, NO_LIBRARY, STORE_IN_USE] {
        assert!((-1179..=-1160).contains(&status), "{status}");
    }
}

fn sources(dir: &Path, into: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() && entry.file_name() != "target" {
            sources(&path, into);
        } else if path.extension().is_some_and(|e| e == "rs") {
            into.push(path);
        }
    }
}

/// The handle API is gone: no source of the platform exports a function named `emusen_scrape_`.
#[test]
fn no_handle_export_is_left() {
    let mut all = Vec::new();
    sources(&platform(), &mut all);
    for source in all {
        let text = read(&source);
        let mut lines = text.lines();
        while let Some(line) = lines.next() {
            if line.trim() == "#[unsafe(no_mangle)]" {
                let next = lines.find(|l| l.contains(" fn ")).unwrap_or_default();
                assert!(!next.contains("fn emusen_scrape_"), "{} still exports {next}", source.display());
            }
        }
    }
}

/// The scraper is beside Crystal, never in it: its crate names neither Crystal nor the session service.
#[test]
fn the_scraper_depends_on_nothing_of_crystal_s() {
    let manifest = read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"));
    for name in ["emusen-crystal", "emusen-session", "session-service"] {
        assert!(!manifest.contains(name), "emusen-scrape depends on {name}");
    }
}
