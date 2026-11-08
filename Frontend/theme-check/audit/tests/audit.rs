// EmuSen_Qt.md §3.7, the loaded step: theme-audit run as theme-check runs it, on fixture packages each written for one finding,
// and the loaded fixture ok through both steps. Each run is a process of its own, offscreen on the software backend.

use std::path::{Path, PathBuf};
use std::process::Command;

use emusen_theme_check::run_with;

// The package's Qt crates, linked so the build's initializers find them; the program under test is the binary.
extern crate emusen_kit_qt;
extern crate emusen_qt_models;
extern crate emusen_theme_host;
extern crate emusen_theme_qt;

const PROGRAM: &str = env!("CARGO_BIN_EXE_theme-audit");

fn themes() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/themes")
}

fn plain(path: &str) -> Result<String, String> {
    Ok(path.to_owned())
}

/// theme-audit's lines on a fixture, and its exit status.
fn audit(name: &str) -> (String, i32) {
    let out = Command::new(PROGRAM).arg(themes().join(name)).output().expect("theme-audit runs");
    assert!(matches!(out.status.code(), Some(0 | 1)), "theme-audit could not run: {}", String::from_utf8_lossy(&out.stderr));
    (String::from_utf8_lossy(&out.stdout).into_owned(), out.status.code().unwrap_or(2))
}

#[test]
fn qt_3_7_a_theme_that_keeps_the_rules_is_ok_through_both_steps() {
    let folder = themes().join("loaded").display().to_string();
    let (out, status) = run_with(&["theme-check".into(), folder], plain, Some(Path::new(PROGRAM))).unwrap();
    assert_eq!((out.as_str(), status), ("ok\n", 0));
}

#[test]
fn qt_3_7_a_games_screen_placed_for_800_rows_is_cut_at_1280_x_720() {
    let (out, status) = audit("cut-games");
    assert_eq!(status, 1);
    assert!(out.contains("cut-games/screens/Games.qml:1 fit 1280x720 dark: past its panel: "), "{out}");
    assert!(out.contains("QQuickText count at 64,730"), "the count is named with its box: {out}");
    assert!(!out.contains("1280x800 dark: past its panel"), "at 800 it fits: {out}");
    assert!(out.lines().all(|l| l.contains("/screens/Games.qml:1 fit ")), "Games alone, and fit alone: {out}");
}

#[test]
fn qt_3_7_a_place_no_walk_leads_to_is_unreachable_at_every_size() {
    let (out, status) = audit("unreachable");
    assert_eq!(status, 1);
    for size in ["1280x800", "1920x1200", "1920x1080", "1280x720", "3840x2160"] {
        for variant in ["dark", "light"] {
            let want = format!("unreachable/screens/Home.qml:1 reach {size} {variant}: the pad does not reach: ");
            assert!(out.lines().any(|l| l.contains(&want) && l.contains("Walkable stranded")), "{want}\n{out}");
        }
    }
    assert_eq!(out.lines().count(), 10, "nothing else: {out}");
}

#[test]
fn qt_3_7_consoles_laid_out_by_hand_do_not_survive_120_consoles() {
    let (out, status) = audit("by-hand");
    assert_eq!(status, 1);
    assert!(out.lines().all(|l| l.contains("by-hand/screens/Consoles.qml:1 ") && l.contains("dark, 120 consoles: ")), "the crowd alone: {out}");
    for size in ["1280x800", "1920x1200", "1920x1080", "1280x720", "3840x2160"] {
        let want = format!(" reach {size} dark, 120 consoles: the pad's sweep heard ");
        assert!(out.contains(&want), "the rows past the window's foot are not seen: {want}\n{out}");
    }
}

#[test]
fn qt_3_7_a_package_with_static_findings_is_not_loaded_and_each_fixture_is_named() {
    for (name, want) in [
        ("literal", "literal/parts/Shelf.qml:8 words \"CONTINUE PLAYING\""),
        ("multimedia", "multimedia/screens/Home.qml:4 import QtMultimedia"),
        ("low-contrast", "low-contrast/theme.json:11 contrast accent on surface is 2.13:1"),
    ] {
        let folder = themes().join(name).display().to_string();
        let (out, status) = run_with(&["theme-check".into(), folder], plain, Some(Path::new(PROGRAM))).unwrap();
        assert!(out.contains(want) && status == 1, "{name}: {out}");
        assert!(out.contains(&format!("{name}/theme.json:1 note the screens were not loaded")), "{name}: {out}");
    }
}

#[test]
fn qt_8_4_the_audit_s_scene_keeps_the_qml_rules() {
    use emusen_qt_test::lint::{frontend, qmllint, rules};
    let scenes = frontend().join("theme-check").join("audit").join("qml");
    let lines = rules(&[(&scenes, true)]);
    assert!(lines.is_empty(), "{}", lines.join("\n"));
    let complaints = qmllint(&[&scenes]);
    assert!(complaints.is_empty(), "{}", complaints.join("\n"));
}
