//! `NOTICE` and the entry's folder, EmuSen_Cores.md §5.3 and §7.

use emusen_core_entry::{library_suffix, sha256_file, stem_of_core, stem_of_info, stem_of_zip, valid_stem, Entry, Notice, FORMAT};
use std::path::PathBuf;

fn sample() -> Notice {
    Notice {
        stem: "testcore".into(),
        name: "Test Core".into(),
        corename: "TESTC".into(),
        authors: vec!["First Author".into(), "Second Author".into()],
        license: "GPLv3".into(),
        source: format!("https://buildbot.libretro.com/nightly/linux/x86_64/latest/testcore_libretro.{}.zip", library_suffix()),
        build: "2026-10-31".into(),
        index_crc32: 0x3a22_4a58,
        sha256: "9f".repeat(32),
        fetched: "2026-10-31".into(),
        extra: Vec::new(),
    }
}

fn scratch(name: &str) -> PathBuf {
    let base = std::env::var_os("TMPDIR").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let dir = base.join(format!("emusen-core-entry-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_notice_reads_back_as_written() {
    let n = sample();
    let text = n.to_text().unwrap();
    assert!(text.contains("index_crc32: 3a224a58\n"));
    assert!(text.contains("authors: First Author|Second Author\n"));
    assert!(text.contains(&format!("format: {FORMAT}\n")));
    assert_eq!(Notice::parse(&text).unwrap(), n);
}

#[test]
fn keys_a_later_format_adds_are_kept_in_order() {
    let text = sample().to_text().unwrap() + "zeta: last\nalpha: first\n";
    let n = Notice::parse(&text).unwrap();
    assert_eq!(n.extra, vec![("zeta".to_string(), "last".to_string()), ("alpha".to_string(), "first".to_string())]);
    assert_eq!(Notice::parse(&n.to_text().unwrap()).unwrap(), n);
}

#[test]
fn crlf_comments_blank_lines_and_spaced_authors_are_read() {
    let text = sample().to_text().unwrap().replace("First Author|Second Author", "First Author | Second Author").replace('\n', "\r\n") + "\r\n# end\r\n";
    assert_eq!(Notice::parse(&text).unwrap(), sample());
}

#[test]
fn an_uppercase_hash_is_read_lowercased_and_never_written_so() {
    let text = sample().to_text().unwrap().replace(&"9f".repeat(32), &"9F".repeat(32));
    assert_eq!(Notice::parse(&text).unwrap().sha256, "9f".repeat(32));
    let mut n = sample();
    n.sha256 = "9F".repeat(32);
    assert!(n.to_text().is_err());
}

#[test]
fn every_broken_field_is_refused_with_its_name() {
    let good = sample().to_text().unwrap();
    let cases: &[(&str, &str, &str)] = &[
        ("format: 1", "format: 2", "newer"),
        ("format: 1", "format: one", "format"),
        ("stem: testcore", "stem: ../evil", "stem"),
        ("stem: testcore", "stem: .hidden", "stem"),
        ("stem: testcore", "stem: othercore", "source is not othercore"),
        ("corename: TESTC", "corename:", "corename"),
        ("source: https://", "source: http://", "source"),
        ("build: 2026-10-31", "build: 2026-13-01", "build"),
        ("fetched: 2026-10-31", "fetched: yesterday", "fetched"),
        ("index_crc32: 3a224a58", "index_crc32: +a224a58", "index_crc32"),
        ("index_crc32: 3a224a58", "index_crc32: 3a224a5", "index_crc32"),
        ("license: GPLv3", "license:", "license"),
        ("authors: First Author|Second Author", "authors: |", "authors"),
    ];
    for (from, to, word) in cases {
        let text = good.replace(from, to);
        let e = Notice::parse(&text).expect_err(to);
        assert!(e.0.contains(word), "{to}: {e}");
    }
    let short = good.replace(&"9f".repeat(32), &"9f".repeat(31));
    assert!(Notice::parse(&short).unwrap_err().0.contains("sha256"));
}

#[test]
fn a_missing_twice_named_or_unkeyed_line_is_refused() {
    let good = sample().to_text().unwrap();
    let missing: String = good.lines().filter(|l| !l.starts_with("sha256")).map(|l| format!("{l}\n")).collect();
    assert!(Notice::parse(&missing).unwrap_err().0.contains("no sha256"));
    assert!(Notice::parse(&(good.clone() + "name: Again\n")).unwrap_err().0.contains("twice"));
    assert!(Notice::parse(&(good.clone() + "just words\n")).unwrap_err().0.contains("key: value"));
    assert!(Notice::parse(&(good + "Name: x\n")).unwrap_err().0.contains("no key"));
}

#[test]
fn a_control_character_in_a_value_is_refused() {
    let mut n = sample();
    n.name = "Bad\u{1b}[31m".into();
    assert!(n.to_text().unwrap_err().0.contains("control"));
}

#[test]
fn stems_never_name_a_path() {
    for good in ["pcsx2", "mesen-s", "DoubleCherryGB", "mame2003_plus", "a.b"] {
        assert!(valid_stem(good), "{good}");
    }
    for bad in ["", ".", "..", ".staging", "a/b", "a\\b", "a b", &"x".repeat(65)] {
        assert!(!valid_stem(bad), "{bad:?}");
    }
    assert_eq!(stem_of_zip("../x_libretro.so.zip"), None);
    assert_eq!(stem_of_info("_libretro.info"), None);
    assert!(Entry::new(std::path::Path::new("/c"), "../x").is_err());
}

#[test]
fn an_entry_names_its_files_and_verifies_its_core() {
    let cores = scratch("verify");
    let e = Entry::new(&cores, "testcore").unwrap();
    std::fs::create_dir_all(&e.folder).unwrap();
    std::fs::write(e.core_path(), b"not really a library").unwrap();
    let mut n = sample();
    n.sha256 = sha256_file(&e.core_path()).unwrap();
    std::fs::write(e.notice_path(), n.to_text().unwrap()).unwrap();
    assert_eq!(e.verify().unwrap(), n.sha256);
    assert_eq!(e.core_path().file_name().unwrap().to_str().unwrap(), format!("testcore_libretro.{}", library_suffix()));
    assert_eq!(e.info_path().file_name().unwrap(), "testcore_libretro.info");
    assert_eq!(Entry::target_text(), format!("emusen-libretro-host.{}\n", library_suffix()));

    std::fs::write(e.core_path(), b"another library").unwrap();
    assert!(e.verify().unwrap_err().0.contains("not the file NOTICE records"));
    std::fs::remove_dir_all(&cores).unwrap();
}

#[test]
fn a_notice_naming_another_core_than_its_folder_is_refused() {
    let cores = scratch("other");
    let e = Entry::new(&cores, "othercore").unwrap();
    std::fs::create_dir_all(&e.folder).unwrap();
    std::fs::write(e.notice_path(), sample().to_text().unwrap()).unwrap();
    assert!(e.read_notice().unwrap_err().0.contains("its folder is othercore"));
    std::fs::remove_dir_all(&cores).unwrap();
}

#[test]
fn the_listing_leaves_out_staging_previous_files_and_bad_names() {
    let cores = scratch("list");
    assert!(Entry::list(&cores).unwrap().is_empty());
    let root = cores.join("libretro");
    for d in ["zeta", "alpha", ".staging", ".previous", "bad name"] {
        std::fs::create_dir_all(root.join(d)).unwrap();
    }
    std::fs::write(root.join("loose.so"), b"").unwrap();
    let names: Vec<String> = Entry::list(&cores).unwrap().into_iter().map(|e| e.stem).collect();
    assert_eq!(names, ["alpha", "zeta"]);
    std::fs::remove_dir_all(&cores).unwrap();
}

#[test]
fn the_folder_and_files_are_named_on_the_file_stem_never_the_corename() {
    // Two real cases: the buildbot ships corename PCEE2 as pcee2_libretro and corename LRPS2 as pcsx2_libretro.
    let suffix = library_suffix();
    let cores = scratch("stem");
    for (stem, corename) in [("pcee2", "PCEE2"), ("pcsx2", "LRPS2")] {
        let zip = format!("{stem}_libretro.{suffix}.zip");
        let info = format!("{stem}_libretro.info");
        assert_eq!(stem_of_zip(&zip), Some(stem));
        assert_eq!(stem_of_info(&info), Some(stem));
        assert_eq!(stem_of_core(&format!("{stem}_libretro.{suffix}")), Some(stem));
        let mut n = sample();
        n.stem = stem.into();
        n.corename = corename.into();
        n.source = format!("https://buildbot.libretro.com/nightly/linux/x86_64/latest/{zip}");
        let e = Entry::for_notice(&cores, &n).unwrap();
        assert_eq!(e.folder, cores.join("libretro").join(stem));
        assert_eq!(e.core_path().file_name().unwrap().to_str().unwrap(), format!("{stem}_libretro.{suffix}"));
        assert_eq!(e.info_path().file_name().unwrap().to_str().unwrap(), info);
        std::fs::create_dir_all(&e.folder).unwrap();
        std::fs::write(e.core_path(), corename.as_bytes()).unwrap();
        n.sha256 = sha256_file(&e.core_path()).unwrap();
        let text = n.to_text().unwrap();
        assert!(text.contains(&format!("stem: {stem}\n")) && text.contains(&format!("corename: {corename}\n")));
        std::fs::write(e.notice_path(), text).unwrap();
        assert_eq!(e.read_notice().unwrap().corename, corename);
        assert_eq!(e.verify().unwrap(), n.sha256);
    }
    let names: Vec<String> = Entry::list(&cores).unwrap().into_iter().map(|e| e.stem).collect();
    assert_eq!(names, ["pcee2", "pcsx2"]);
    std::fs::remove_dir_all(&cores).unwrap();
}
