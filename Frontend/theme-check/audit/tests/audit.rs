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
fn qt_3_7_a_view_that_never_settles_stops_its_pass_loudly_and_fails_the_check() {
    let (out, status) = audit("unsettled");
    assert_eq!(status, 1);
    let stopped: Vec<&str> = out.lines().filter(|l| l.contains("unsettled/screens/Home.qml:1 stopped ")).collect();
    assert_eq!(stopped.len(), 1, "one line for the pass: {out}");
    // The view is moving from the start, so the pass stops at its first settling: the reduced-motion check at 1280 x 800.
    assert!(stopped[0].contains(":1 stopped 1280x800, judged still under reduced motion: the audit stopped here"), "where it stopped: {out}");
    assert!(stopped[0].contains("QQuickFlickable restless") && stopped[0].contains("still moving after 5000 ms"), "the harness's words: {out}");
    let folder = themes().join("unsettled").display().to_string();
    let (out, status) = run_with(&["theme-check".into(), folder], plain, Some(Path::new(PROGRAM))).unwrap();
    assert_ne!(status, 0, "{out}");
    assert!(out.contains("unsettled/screens/Home.qml:1 stopped "), "{out}");
}

#[test]
fn qt_3_7_a_transition_longer_than_the_kit_s_is_recorded_at_rest() {
    let state = std::env::temp_dir().join(format!("theme-audit-slow-{}", std::process::id()));
    let out = Command::new(PROGRAM).arg("--state").arg(&state).arg(themes().join("slow")).output().expect("theme-audit runs");
    let lines = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(!lines.contains(" stopped "), "the veil comes to rest: {lines}");
    let png = std::fs::read(state.join("Home-dark-1280x800.png")).expect("the recording's picture");
    let picture = cxx_qt_lib::QImage::from_data(&png, Some("PNG")).expect("a PNG");
    // The veil's centre, (900, 450) at one pixel a unit, in the kit's dark accent #4C9AE8 once its 1.5 s fade has ended.
    let ink = picture.pixel_color(900, 450);
    let (r, g, b) = (ink.red(), ink.green(), ink.blue());
    assert!((r - 0x4C).abs() <= 2 && (g - 0x9A).abs() <= 2 && (b - 0xE8).abs() <= 2, "the veil whole, not mid-fade: {r},{g},{b}");
    let _ = std::fs::remove_dir_all(&state);
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

#[test]
fn qt_3_7_a_catalogue_records_choose_a_console_over_its_consoles_alone() {
    let dir = std::env::temp_dir().join(format!("theme-audit-catalogue-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("two.tsv");
    std::fs::write(&file, "# two consoles of two makers\nSega\tGenesis\t12\tNephrite\temusen\tmd\nSony\tPlayStation\t3\tDiamond\temusen\tpsx\n").unwrap();
    let state = dir.join("state");
    let out = Command::new(PROGRAM).arg("--state").arg(&state).arg("--catalogue").arg(&file).arg(themes().join("loaded")).output().expect("theme-audit runs");
    let lines = String::from_utf8_lossy(&out.stdout).into_owned();
    assert_eq!(out.status.code(), Some(0), "the loaded fixture over the catalogue: {lines}{}", String::from_utf8_lossy(&out.stderr));
    for variant in ["dark", "light"] {
        for size in ["1280x800", "1920x1200", "1920x1080", "1280x720", "3840x2160"] {
            let name = format!("Consoles-{variant}-two-{size}");
            assert!(state.join(format!("{name}.png")).exists(), "{name}.png");
            assert!(state.join(format!("{name}.spec")).exists(), "{name}.spec");
        }
    }
    let recorded: Vec<String> = std::fs::read_dir(&state).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
    assert!(recorded.iter().all(|f| f.starts_with("Consoles-")), "Choose a console alone: {recorded:?}");
    let spec = std::fs::read_to_string(state.join("Consoles-dark-two-1280x800.spec")).unwrap();
    assert!(spec.contains("Genesis") && spec.contains("PlayStation"), "the catalogue's consoles on the screen");
    let bad = dir.join("bad.tsv");
    std::fs::write(&bad, "Sega\tGenesis\tmany\n").unwrap();
    let out = Command::new(PROGRAM).arg("--catalogue").arg(&bad).arg(themes().join("loaded")).output().expect("theme-audit runs");
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("bad.tsv: line 1: the games count"), "the line named");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn qt_3_7_a_desktop_at_125_percent_still_records_one_pixel_a_unit() {
    let dir = std::env::temp_dir().join(format!("theme-audit-scaled-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    // An offscreen screen at 120 dpi over a base of 96, a scale of 1.25, as the desktop was when its pictures came out 1600 x 1000.
    let screen = dir.join("scaled.json");
    std::fs::write(&screen, r#"{"screens":[{"name":"scaled","x":0,"y":0,"width":2560,"height":1600,"logicalDpi":120,"logicalBaseDpi":96,"dpr":1.25}]}"#).unwrap();
    let file = dir.join("one.tsv");
    std::fs::write(&file, "Sega\tGenesis\t12\tNephrite\temusen\tmd\n").unwrap();
    let state = dir.join("state");
    let out = Command::new(PROGRAM)
        .env("QT_QPA_PLATFORM", format!("offscreen:configfile={}", screen.display()))
        .arg("--state").arg(&state).arg("--catalogue").arg(&file).arg(themes().join("loaded"))
        .output().expect("theme-audit runs");
    let lines = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(!lines.contains(" recording "), "every picture at its state's size: {lines}");
    assert_eq!(out.status.code(), Some(0), "{lines}");
    let png = std::fs::read(state.join("Consoles-dark-one-1280x800.png")).expect("the recording's picture");
    let picture = cxx_qt_lib::QImage::from_data(&png, Some("PNG")).expect("a PNG");
    assert_eq!((picture.width(), picture.height()), (1280, 800), "one device pixel a unit");
    let _ = std::fs::remove_dir_all(&dir);
}

/// theme-audit's lines on a fixture in the states a list names, the default among them.
fn audit_states(name: &str, states: &str, record: Option<&Path>) -> (String, i32) {
    let mut command = Command::new(PROGRAM);
    if let Some(dir) = record {
        command.arg("--state").arg(dir);
    }
    let out = command.arg("--states").arg(states).arg(themes().join(name)).output().expect("theme-audit runs");
    assert!(matches!(out.status.code(), Some(0 | 1)), "theme-audit could not run: {}", String::from_utf8_lossy(&out.stderr));
    (String::from_utf8_lossy(&out.stdout).into_owned(), out.status.code().unwrap_or(2))
}

/// A fixture wrong in one state alone: found there, of one kind, at every size in both variants, and nowhere in the default.
fn found_only_in(name: &str, screen: &str, state: &str, kind: &str, words: &str) {
    let (out, status) = audit_states(name, &format!("default,{state}"), None);
    assert_eq!(status, 1, "{name}: {out}");
    assert!(out.lines().all(|l| l.contains(&format!("{name}/screens/{screen}.qml:1 {kind} ")) && l.contains(&format!(", {state}: "))), "{state} alone, {kind} alone: {out}");
    for size in ["1280x800", "1920x1200", "1920x1080", "1280x720", "3840x2160"] {
        for variant in ["dark", "light"] {
            let want = format!("{kind} {size} {variant}, {state}: {words}");
            assert!(out.lines().any(|l| l.contains(&want)), "{want}\n{out}");
        }
    }
}

#[test]
fn qt_3_7_a_continue_entry_drawn_with_no_game_is_named_for_no_reader() {
    found_only_in("state-nogame", "Home", "nogame", "name", "no name a reader hears");
}

#[test]
fn qt_3_7_a_tile_titled_only_beside_its_cover_is_unnamed_without_one() {
    found_only_in("state-nocover", "Games", "nocover", "name", "no name a reader hears");
}

#[test]
fn qt_3_7_a_declined_place_that_cannot_hold_the_focus_is_unreachable() {
    found_only_in("state-disabled", "Home", "disabled", "reach", "the pad does not reach");
}

#[test]
fn qt_3_7_a_press_that_grows_its_row_over_the_next_overlaps() {
    found_only_in("state-pressed", "Home", "pressed", "fit", "overlap");
}

#[test]
fn qt_3_7_the_empty_words_in_too_narrow_a_box_elide_in_an_empty_library() {
    found_only_in("state-empty", "Games", "empty", "fit", "cut with an ellipsis");
}

#[test]
fn qt_3_7_a_count_set_near_the_right_edge_runs_past_it_at_9000_games() {
    found_only_in("state-many", "Games", "many", "fit", "past its panel");
}

#[test]
fn qt_3_7_a_splash_is_held_at_1_3_and_5_s_and_judged_at_each() {
    let dir = std::env::temp_dir().join(format!("theme-audit-splash-{}", std::process::id()));
    let (out, status) = audit_states("state-splash", "default,splash-1s,splash-3s,splash-5s", Some(&dir));
    assert_eq!(status, 1, "{out}");
    assert!(out.lines().all(|l| l.contains("state-splash/screens/Splash.qml:1 fit ") && l.contains(", splash-3s: ")), "at 3 s alone: {out}");
    assert!(out.contains("fit 1280x800 dark, splash-3s: past its panel") && out.contains("fit 3840x2160 light, splash-3s: past its panel"), "{out}");
    // Each held time is its own picture: the wordmark past the edge at 3 s, in place at 1 and 5.
    let picture = |state: &str| std::fs::read(dir.join(format!("Splash-dark-{state}-1280x800.png"))).expect(state);
    assert_ne!(picture("splash-1s"), picture("splash-3s"));
    assert_eq!(picture("splash-1s"), picture("splash-5s"), "the wordmark in place at both");
    let spec = std::fs::read_to_string(dir.join("Splash-light-splash-3s-1920x1200.spec")).unwrap();
    assert!(spec.contains("wordmark"), "the held state recorded");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn qt_3_7_states_list_and_an_unknown_one_is_refused() {
    let out = Command::new(PROGRAM).arg("--list-states").output().expect("theme-audit runs");
    let listed = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(listed.lines().any(|l| l.starts_with("nocover\tHome,Consoles,Games\t")), "{listed}");
    let out = Command::new(PROGRAM).arg("--states").arg("nogame,lost").arg(themes().join("loaded")).output().expect("theme-audit runs");
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("lost is not a state"));
}
