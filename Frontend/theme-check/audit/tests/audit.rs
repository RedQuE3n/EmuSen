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

/// One run of theme-audit: its lines, its words on stderr, its exit status, and the screens it held.
struct Ran {
    out: String,
    err: String,
    status: i32,
}

impl Ran {
    /// The run, which must have exited of itself: a signal is never a result.
    fn of(command: &mut Command) -> Ran {
        let out = command.output().expect("theme-audit runs");
        let err = String::from_utf8_lossy(&out.stderr).into_owned();
        let status = out.status.code().unwrap_or_else(|| panic!("theme-audit was stopped ({}): {err}", out.status));
        Ran { out: String::from_utf8_lossy(&out.stdout).into_owned(), err, status }
    }

    /// As `of`, a run that judged the package: its findings (1) or none (0), never a failure to run (2).
    fn judged(command: &mut Command) -> Ran {
        let ran = Ran::of(command);
        assert!(matches!(ran.status, 0 | 1), "theme-audit could not run ({}): {}", ran.status, ran.err);
        ran
    }

    /// How many states of `screen` theme-audit judged to their end, and how many against its mockup, by its own count.
    fn held(&self, screen: &str) -> (usize, usize) {
        let prefix = format!("theme-audit: held {screen} in ");
        let Some(line) = self.err.lines().find_map(|l| l.strip_prefix(&prefix)) else { return (0, 0) };
        let numbers: Vec<usize> = line.split(|c: char| !c.is_ascii_digit()).filter_map(|n| n.parse().ok()).collect();
        (numbers[0], numbers[1])
    }
}

/// theme-audit's lines on a fixture, and its exit status.
fn audit(name: &str) -> Ran {
    Ran::judged(Command::new(PROGRAM).arg(themes().join(name)))
}

#[test]
fn qt_3_7_a_theme_that_keeps_the_rules_is_ok_through_both_steps() {
    let folder = themes().join("loaded").display().to_string();
    let (out, status) = run_with(&["theme-check".into(), folder], plain, Some(Path::new(PROGRAM))).unwrap();
    assert_eq!((out.as_str(), status), ("ok\n", 0));
}

#[test]
fn qt_3_7_a_games_screen_placed_for_800_rows_is_cut_at_1280_x_720() {
    let ran = audit("cut-games");
    let (out, status) = (ran.out.as_str(), ran.status);
    assert_eq!(status, 1);
    assert_eq!(ran.held("Games").0, 10, "the Games screen judged at five sizes in both variants: {}", ran.err);
    assert!(out.contains("cut-games/screens/Games.qml:1 fit 1280x720 dark: past its panel: "), "{out}");
    assert!(out.contains("QQuickText count at 64,730"), "the count is named with its box: {out}");
    assert!(!out.contains("1280x800 dark: past its panel"), "at 800 it fits: {out}");
    assert!(out.lines().all(|l| l.contains("/screens/Games.qml:1 fit ")), "Games alone, and fit alone: {out}");
}

#[test]
fn qt_3_7_a_place_no_walk_leads_to_is_unreachable_at_every_size() {
    let ran = audit("unreachable");
    let (out, status) = (ran.out.as_str(), ran.status);
    assert_eq!(status, 1);
    assert_eq!(ran.held("Home").0, 10, "{}", ran.err);
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
    let ran = audit("unsettled");
    let (out, status) = (ran.out.as_str(), ran.status);
    assert_eq!(status, 1);
    assert_eq!(ran.held("Home").0, 0, "the pass stopped before any state was judged: {}", ran.err);
    let stopped: Vec<&str> = out.lines().filter(|l| l.contains("unsettled/screens/Home.qml:1 stopped ")).collect();
    assert_eq!(stopped.len(), 1, "one line for the pass: {out}");
    // The view is moving from the start, so the pass stops at its first settling: the reduced-motion check at 1280 x 800.
    assert!(stopped[0].contains(":1 stopped 1280x800, judged still under reduced motion: the audit stopped here"), "where it stopped: {out}");
    assert!(stopped[0].contains("QQuickFlickable restless") && stopped[0].contains("still moving after 5000 ms"), "the harness's words: {out}");
    let folder = themes().join("unsettled").display().to_string();
    let (out, status) = run_with(&["theme-check".into(), folder], plain, Some(Path::new(PROGRAM))).unwrap();
    assert_eq!(status, 1, "{out}");
    assert!(out.contains("unsettled/screens/Home.qml:1 stopped "), "{out}");
}

#[test]
fn qt_3_7_a_transition_longer_than_the_kit_s_is_recorded_at_rest() {
    let state = std::env::temp_dir().join(format!("theme-audit-slow-{}", std::process::id()));
    let ran = Ran::judged(Command::new(PROGRAM).arg("--state").arg(&state).arg(themes().join("slow")));
    let lines = &ran.out;
    assert!(!lines.contains(" stopped "), "the veil comes to rest: {lines}");
    assert_eq!(ran.held("Home").0, 10, "{}", ran.err);
    let png = std::fs::read(state.join("Home-dark-1280x800.png")).expect("the recording's picture");
    let picture = cxx_qt_lib::QImage::from_data(&png, Some("PNG")).expect("a PNG");
    // The veil's centre, (900, 450) at one pixel a unit, in the kit's dark accent #4C9AE8 once its 1.5 s fade has ended.
    let ink = picture.pixel_color(900, 450);
    let (r, g, b) = (ink.red(), ink.green(), ink.blue());
    assert!((r - 0x4C).abs() <= 2 && (g - 0x9A).abs() <= 2 && (b - 0xE8).abs() <= 2, "the veil whole, not mid-fade: {r},{g},{b}");
    let _ = std::fs::remove_dir_all(&state);
}

#[test]
fn qt_3_7_words_over_a_pale_picture_are_measured_against_the_picture() {
    let ran = audit("dim-picture");
    let (out, status) = (ran.out.as_str(), ran.status);
    assert_eq!(status, 1, "{out}");
    assert_eq!(ran.held("Home").0, 10, "{}", ran.err);
    for size in ["1280x800", "1920x1200", "1920x1080", "1280x720", "3840x2160"] {
        let want = format!("dim-picture/screens/Home.qml:1 contrast {size} dark: low contrast: ");
        assert!(out.lines().any(|l| l.contains(&want) && l.contains("caption")), "{want}\n{out}");
    }
    // The tokens alone pass: the light variant's text is dark on the same picture, and the rows keep theirs in both.
    assert!(out.lines().all(|l| l.contains(" contrast ") && l.contains(" dark: ") && l.contains("caption")), "the caption in the dark alone: {out}");
}

#[test]
fn qt_3_7_consoles_laid_out_by_hand_do_not_survive_120_consoles() {
    let ran = audit("by-hand");
    let (out, status) = (ran.out.as_str(), ran.status);
    assert_eq!(status, 1);
    assert_eq!(ran.held("Consoles").0, 15, "both variants at five sizes, and the crowd by night at five: {}", ran.err);
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
    let ran = Ran::judged(Command::new(PROGRAM).arg("--state").arg(&state).arg("--catalogue").arg(&file).arg(themes().join("loaded")));
    assert_eq!(ran.status, 0, "the loaded fixture over the catalogue: {}{}", ran.out, ran.err);
    assert_eq!(ran.held("Consoles"), (10, 0), "Choose a console alone, at five sizes in both variants: {}", ran.err);
    assert!(!ran.err.contains("held Home") && !ran.err.contains("held Games"), "{}", ran.err);
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
    let ran = Ran::of(Command::new(PROGRAM).arg("--catalogue").arg(&bad).arg(themes().join("loaded")));
    assert_eq!(ran.status, 2);
    assert!(ran.err.contains("bad.tsv: line 1: the games count"), "the line named");
    assert_eq!(ran.held("Consoles"), (0, 0), "nothing held");
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
    let ran = Ran::judged(
        Command::new(PROGRAM)
            .env("QT_QPA_PLATFORM", format!("offscreen:configfile={}", screen.display()))
            .arg("--state").arg(&state).arg("--catalogue").arg(&file).arg(themes().join("loaded")),
    );
    let lines = ran.out.as_str();
    assert!(!lines.contains(" recording "), "every picture at its state's size: {lines}");
    assert_eq!(ran.status, 0, "{lines}");
    assert_eq!(ran.held("Consoles").0, 10, "{}", ran.err);
    let png = std::fs::read(state.join("Consoles-dark-one-1280x800.png")).expect("the recording's picture");
    let picture = cxx_qt_lib::QImage::from_data(&png, Some("PNG")).expect("a PNG");
    assert_eq!((picture.width(), picture.height()), (1280, 800), "one device pixel a unit");
    let _ = std::fs::remove_dir_all(&dir);
}

/// theme-audit's lines on a fixture in the states a list names, the default among them.
fn audit_states(name: &str, states: &str, record: Option<&Path>) -> Ran {
    let mut command = Command::new(PROGRAM);
    if let Some(dir) = record {
        command.arg("--state").arg(dir);
    }
    Ran::judged(command.arg("--states").arg(states).arg(themes().join(name)))
}

/// A fixture wrong in one state alone: found there, of one kind, at every size in both variants, and nowhere in the default.
fn found_only_in(name: &str, screen: &str, state: &str, kind: &str, words: &str) {
    let ran = audit_states(name, &format!("default,{state}"), None);
    let (out, status) = (ran.out.as_str(), ran.status);
    assert_eq!(status, 1, "{name}: {out}");
    assert_eq!(ran.held(screen).0, 20, "{screen} in both states, at five sizes in both variants: {}", ran.err);
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
    let ran = audit_states("state-splash", "default,splash-1s,splash-3s,splash-5s", Some(&dir));
    let (out, status) = (ran.out.as_str(), ran.status);
    assert_eq!(status, 1, "{out}");
    assert_eq!(ran.held("Splash").0, 40, "the default and three held times, at five sizes in both variants: {}", ran.err);
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
fn qt_3_7_a_held_clock_logs_nothing_where_the_environment_asks_for_a_threaded_render_loop() {
    let ran = Ran::judged(
        Command::new(PROGRAM).env("QSG_RENDER_LOOP", "threaded").arg("--states").arg("splash-1s,splash-3s,splash-5s").arg(themes().join("state-splash")),
    );
    let (stdout, stderr) = (ran.out.as_str(), ran.err.as_str());
    assert_eq!(ran.status, 1, "{stdout}{stderr}");
    assert_eq!(ran.held("Splash").0, 30, "three held times, at five sizes in both variants: {stderr}");
    assert!(stdout.lines().all(|l| l.contains("state-splash/screens/Splash.qml:1 fit ") && l.contains(", splash-3s: ")), "the held states' findings alone, no warning: {stdout}");
    assert!(!stderr.contains("QUnifiedTimer") && !stderr.contains("driver"), "the held clock installed and uninstalled once a hold, quietly: {stderr}");
}

#[test]
fn qt_3_7_a_declined_place_drawn_apart_is_exempt_and_a_dim_offered_one_is_not() {
    let ran = audit_states("state-declined", "default,disabled", None);
    let (out, status) = (ran.out.as_str(), ran.status);
    assert_eq!(status, 1, "{out}");
    assert_eq!(ran.held("Home").0, 20, "{}", ran.err);
    // Power, offered with its words at 38%, is low in both states; the Scraper, declined at 38% and so told apart, never is.
    assert!(out.lines().all(|l| l.contains("state-declined/screens/Home.qml:1 contrast ") && l.contains("\"Power\"")), "{out}");
    for state in ["", ", disabled"] {
        for size in ["1280x800", "1920x1200", "1920x1080", "1280x720", "3840x2160"] {
            for variant in ["dark", "light"] {
                let want = format!("contrast {size} {variant}{state}: low contrast");
                assert!(out.lines().any(|l| l.contains(&want)), "{want}\n{out}");
            }
        }
    }
}

#[test]
fn qt_3_7_words_are_judged_at_rest_not_while_their_opacity_changes() {
    let ran = audit_states("state-fade", "default,splash-1s,splash-3s,splash-5s", None);
    let (out, status) = (ran.out.as_str(), ran.status);
    assert_eq!(status, 1, "{out}");
    assert_eq!(ran.held("Splash").0, 40, "{}", ran.err);
    // The caption mid-fade at 1 s and the title mid-fade at 5 s are not judged; the caption at rest, at 0.3, is.
    assert!(out.lines().all(|l| l.contains("state-fade/screens/Splash.qml:1 contrast ") && l.contains("restsFaint")), "{out}");
    for state in [", splash-3s: ", ", splash-5s: ", " dark: ", " light: "] {
        assert!(out.lines().any(|l| l.contains(state)), "the caption at rest{state}\n{out}");
    }
    assert!(!out.contains(", splash-1s: "), "the caption mid-fade at 1 s is not judged: {out}");
}

#[test]
fn qt_3_7_states_list_and_an_unknown_one_is_refused() {
    let ran = Ran::of(Command::new(PROGRAM).arg("--list-states"));
    assert_eq!(ran.status, 0, "{}", ran.err);
    let listed = ran.out;
    assert!(listed.lines().any(|l| l.starts_with("nocover\tHome,Consoles,Games\t")), "{listed}");
    let ran = Ran::of(Command::new(PROGRAM).arg("--states").arg("nogame,lost").arg(themes().join("loaded")));
    assert_eq!(ran.status, 2);
    assert!(ran.err.contains("lost is not a state"));
}

/// theme-audit's lines on a package with its mockups in `dir`.
fn audit_mockups(folder: &Path, dir: &Path) -> Ran {
    Ran::judged(Command::new(PROGRAM).arg("--mockups").arg(dir).arg(folder))
}

#[test]
fn qt_3_7_a_screen_is_held_to_its_mockup_at_the_canvas_s_sizes_by_night() {
    let dir = std::env::temp_dir().join(format!("theme-audit-mockups-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let spec = |h: u32| format!("mockup 1\ncanvas Home 1280 800\nelement Home x=0 y=0 w=1280 h={h}\n");
    std::fs::write(dir.join("loaded-Home.mockup.spec"), spec(800)).unwrap();
    let ran = audit_mockups(&themes().join("loaded"), &dir);
    assert!(!ran.out.contains(" mockup "), "the screen as its mockup states it: {}", ran.out);
    assert_eq!(ran.held("Home").1, 2, "held to it at 1280 x 800 and 1920 x 1200 by night: {}", ran.err);
    std::fs::write(dir.join("loaded-Home.mockup.spec"), spec(700)).unwrap();
    let ran = audit_mockups(&themes().join("loaded"), &dir);
    let (out, status) = (ran.out.as_str(), ran.status);
    let found: Vec<&str> = out.lines().filter(|l| l.contains(" mockup ")).collect();
    assert_eq!(status, 1, "{out}");
    assert_eq!(found.len(), 2, "1280 x 800 and 1920 x 1200 by night, nothing else: {out}");
    for size in ["1280x800 dark", "1920x1200 dark"] {
        assert!(found.iter().any(|l| l.ends_with(&format!("loaded/screens/Home.qml:1 mockup {size}: Home: H 800.0, the mockup's 700"))), "{size}: {out}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// The built-in themes' mockups (Testing/themes): Home, Games and Choose a console held to the canvases they were drawn from.
fn built_in(theme: &str) {
    let mockups = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../Testing/themes");
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes").join(theme);
    let held = std::fs::read_dir(&mockups).unwrap().flatten().filter(|e| e.file_name().to_string_lossy().to_lowercase().starts_with(&format!("{theme}-"))).count();
    assert_eq!(held, 3, "{theme}: Home, Games and Consoles");
    let ran = audit_mockups(&folder, &mockups);
    for screen in ["Home", "Games", "Consoles"] {
        assert_eq!(ran.held(screen).1, 2, "{theme}: {screen} held to its mockup at the canvas's two sizes: {}", ran.err);
    }
    let found: Vec<&str> = ran.out.lines().filter(|l| l.contains(" mockup ")).collect();
    assert!(found.is_empty(), "{theme}: {} differences from its mockups:\n{}", found.len(), found.join("\n"));
}

#[test]
fn qt_3_4_3_horizon_is_held_to_its_mockups() {
    built_in("horizon");
}

#[test]
fn qt_3_4_4_meridian_is_held_to_its_mockups() {
    built_in("meridian");
}

#[test]
fn qt_3_4_6_orrery_is_held_to_its_mockups() {
    built_in("orrery");
}

/// Choose a console over the canvas's eleven consoles with its focus, judged against the canvas in a folder of `name`.
fn against_the_canvas(name: &str) -> Ran {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let dir = std::env::temp_dir().join(format!("theme-audit-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::copy(root.join("Testing/mockups/Consoles.mockup.spec"), dir.join("emusen-Consoles.mockup.spec")).unwrap();
    let ran = Ran::judged(
        Command::new(PROGRAM)
            .arg("--catalogue")
            .arg(root.join("Testing/themes/catalogues/eleven.tsv"))
            .args(["--focus", "Super Nintendo", "--mockups"])
            .arg(&dir)
            .arg(root.join("Frontend/themes/emusen")),
    );
    let _ = std::fs::remove_dir_all(&dir);
    ran
}

#[test]
fn qt_3_4_2_the_places_and_the_covers_stand_where_the_canvas_stands_them() {
    let ran = against_the_canvas("places");
    assert_eq!(ran.held("Consoles"), (10, 2), "{}", ran.err);
    let found: Vec<&str> = ran.out.lines().filter(|l| l.contains(" mockup ") && (l.contains(": place") || l.contains(": rowCover") && l.contains("W 73"))).collect();
    assert!(found.is_empty(), "the places' x (R-20261111-34) and the covers' shape:\n{}", found.join("\n"));
    let covers = ran.out.lines().filter(|l| l.contains(": rowCover.0: W ")).collect::<Vec<_>>();
    assert!(covers.iter().all(|l| l.contains("W 137.0")), "the Super Nintendo's covers 137 wide, its shape 1/0.73: {covers:?}");
}

#[test]
fn qt_3_4_2_choose_a_console_over_the_canvas_s_eleven_consoles_is_its_mockup() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let dir = std::env::temp_dir().join(format!("theme-audit-canvas-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    // The reference theme's approved canvas, under the name theme-audit finds a package's mockup by.
    std::fs::copy(root.join("Testing/mockups/Consoles.mockup.spec"), dir.join("emusen-Consoles.mockup.spec")).unwrap();
    let ran = Ran::judged(
        Command::new(PROGRAM)
            .arg("--catalogue")
            .arg(root.join("Testing/themes/catalogues/eleven.tsv"))
            .args(["--focus", "Super Nintendo", "--mockups"])
            .arg(&dir)
            .arg(root.join("Frontend/themes/emusen")),
    );
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(ran.held("Consoles"), (10, 2), "Choose a console judged in ten states, two against the canvas: {}", ran.err);
    let found: Vec<&str> = ran.out.lines().filter(|l| l.contains(" mockup ")).collect();
    assert!(found.is_empty(), "{} differences from the canvas:\n{}", found.len(), found.join("\n"));
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap().flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

#[test]
fn qt_3_4_1_home_s_places_lines_are_the_program_s_own_and_whole_at_every_size() {
    // The reference package with its Home alone, so the run holds no other screen.
    let package = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes/emusen");
    let dir = std::env::temp_dir().join(format!("theme-audit-home-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    copy_tree(&package, &dir);
    let manifest = std::fs::read_to_string(dir.join("theme.json")).unwrap();
    let home_only = manifest.replace(r#""screens": ["home", "consoles", "games"]"#, r#""screens": ["home"]"#);
    assert_ne!(home_only, manifest, "the manifest names its three screens");
    std::fs::write(dir.join("theme.json"), home_only).unwrap();
    let states = dir.join("states");
    std::fs::create_dir_all(&states).unwrap();
    let ran = Ran::judged(Command::new(PROGRAM).arg(&dir).arg("--states").arg("default,loud").arg("--state").arg(&states));
    assert_eq!(ran.held("Home").0, 20, "Home at five sizes in both variants, by default and loud: {}", ran.err);
    let cut: Vec<&str> = ran.out.lines().chain(ran.err.lines()).filter(|l| l.contains("placeStatus")).collect();
    assert!(cut.is_empty(), "the Settings place's line is the program's own and fits (R-20261111-35):\n{}", cut.join("\n"));
    // The line as each variant words it, the light one the longest it can be at full volume, drawn at every size.
    for (variant, line) in [("dark", "Dark · 100% sound"), ("light", "Light · 100% sound")] {
        for (w, h) in [(1280, 800), (1920, 1200), (1920, 1080), (1280, 720), (3840, 2160)] {
            let spec = std::fs::read_to_string(states.join(format!("Home-{variant}-loud-{w}x{h}.spec"))).unwrap_or_default();
            assert!(spec.contains(line), "Home {variant} at {w}x{h} draws {line:?}");
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn kit_7_109_a_title_cut_with_an_ellipsis_is_listed_under_its_ruling_and_fails_nothing() {
    let ran = audit("elided-title");
    assert_eq!(ran.held("Games").0, 10, "{}", ran.err);
    assert_eq!((ran.out.as_str(), ran.status), ("", 0), "{}", ran.err);
    let listed: Vec<&str> = ran.err.lines().filter(|l| l.starts_with("theme-audit: accepted by ")).collect();
    assert!(!listed.is_empty(), "{}", ran.err);
    assert!(
        listed.iter().all(|l| l.starts_with("theme-audit: accepted by R-20261111-44: ") && l.contains("data cut with an ellipsis") && l.contains("rowTitle.")),
        "{listed:?}"
    );
}

#[test]
fn kit_7_102_moonrise_s_caption_name_in_a_theme_s_own_moonrise_is_not_accepted() {
    let ran = audit("small-caption");
    assert_eq!(ran.held("Home").0, 10, "{}", ran.err);
    assert_eq!(ran.status, 1, "{}", ran.out);
    assert!(!ran.err.contains("theme-audit: accepted by"), "R-20261111-40 covers the kit's Moonrise.qml alone, not a theme's item named moonrise: {}", ran.err);
    let small: Vec<&str> = ran.out.lines().filter(|l| l.contains("small-caption/screens/Home.qml") && l.contains("too small") && l.contains("moonriseCaption")).collect();
    assert_eq!(small.len(), 10, "the floor's finding at five sizes in both variants:\n{}", ran.out);
}

/// The reference package copied to a folder of `name` with Choose a console alone in its manifest.
fn consoles_alone(name: &str) -> PathBuf {
    let package = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes/emusen");
    let dir = std::env::temp_dir().join(format!("theme-audit-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    copy_tree(&package, &dir);
    let manifest = std::fs::read_to_string(dir.join("theme.json")).unwrap();
    let alone = manifest.replace(r#""screens": ["home", "consoles", "games"]"#, r#""screens": ["consoles"]"#);
    assert_ne!(alone, manifest, "the manifest names its three screens");
    std::fs::write(dir.join("theme.json"), alone).unwrap();
    dir
}

/// A recorded item's box, `at=x,y,w,h`, and its laid-out words, `layout=w,h`, by its name in a state's spec.
fn boxes(spec: &str, prefix: &str) -> Vec<(String, [f64; 4], [f64; 2])> {
    let field = |line: &str, key: &str| -> Vec<f64> {
        line.split_whitespace().find_map(|w| w.strip_prefix(key)).map(|v| v.split(',').map(|n| n.parse().unwrap()).collect()).unwrap_or_default()
    };
    spec.lines()
        .filter_map(|line| {
            let name = line.split_whitespace().find_map(|w| w.strip_prefix("name="))?;
            if !name.starts_with(prefix) {
                return None;
            }
            let (at, layout) = (field(line, "at="), field(line, "layout="));
            Some((name.to_owned(), [at[0], at[1], at[2], at[3]], [layout[0], layout[1]]))
        })
        .collect()
}

#[test]
fn qt_3_4_2_a_full_name_far_longer_than_any_wraps_or_shrinks_short_of_every_count() {
    // The canvas's eleven consoles with the Super Nintendo's name far longer than any console's, and it focused.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let dir = consoles_alone("longest-name");
    let long = "Super Nintendo Entertainment System with the Satellaview and the Super Game Boy Two Attached";
    let eleven = std::fs::read_to_string(root.join("Testing/themes/catalogues/eleven.tsv")).unwrap();
    let catalogue = eleven.replace("\tSuper Nintendo\t", &format!("\t{long}\t"));
    assert_ne!(catalogue, eleven, "the catalogue names the Super Nintendo");
    std::fs::write(dir.join("longest.tsv"), catalogue).unwrap();
    let states = dir.join("states");
    let ran = Ran::judged(Command::new(PROGRAM).arg("--catalogue").arg(dir.join("longest.tsv")).args(["--focus", long, "--state"]).arg(&states).arg(&dir));
    assert_eq!(ran.held("Consoles").0, 10, "Choose a console at five sizes in both variants: {}", ran.err);
    let named: Vec<&str> = ran.out.lines().filter(|l| l.contains("consoleDetailName")).collect();
    assert!(named.is_empty(), "the full name is cut or meets nothing (R-20261112-2):\n{}", named.join("\n"));
    let elided: Vec<&str> = ran.err.lines().filter(|l| l.contains("consoleDetailName") && l.contains("ellipsis")).collect();
    assert!(elided.is_empty(), "the full name is never cut with an ellipsis:\n{}", elided.join("\n"));
    for variant in ["dark", "light"] {
        for (w, h) in [(1280, 800), (1920, 1200), (1920, 1080), (1280, 720), (3840, 2160)] {
            let file = states.join(format!("Consoles-{variant}-longest-{w}x{h}.spec"));
            let spec = std::fs::read_to_string(&file).unwrap_or_else(|_| panic!("{} recorded", file.display()));
            let (_, name, words) = boxes(&spec, "consoleDetailName").pop().unwrap_or_else(|| panic!("{variant} {w}x{h}: the full name drawn"));
            assert!(spec.contains(&format!("text=\"{long}\"")), "{variant} {w}x{h}: the whole name, never cut");
            assert!(words[0] <= name[2] + 1.0 && words[1] <= name[3] + 1.0, "{variant} {w}x{h}: the words {words:?} inside their box {name:?}");
            let counts = boxes(&spec, "consoleCount.");
            let meets: Vec<_> = counts
                .iter()
                .filter(|(_, c, _)| c[1] < name[1] + name[3] && c[1] + c[3] > name[1] && c[0] >= name[0] && c[0] < name[0] + name[2] - 1.0)
                .collect();
            assert!(meets.is_empty(), "{variant} {w}x{h}: the name's box {name:?} runs under {meets:?}");
            assert!(counts.iter().any(|(_, c, _)| c[1] < name[1] + name[3] && c[1] + c[3] > name[1] && c[0] >= name[0]), "{variant} {w}x{h}: a count shares the name's rows, so the bound is held");
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn qt_3_4_2_the_full_name_meets_no_count_over_120_consoles() {
    let dir = consoles_alone("detail-120");
    let ran = Ran::judged(Command::new(PROGRAM).arg(&dir));
    assert_eq!(ran.held("Consoles").0, 10, "{}", ran.err);
    let named: Vec<&str> = ran.out.lines().filter(|l| l.contains("consoleDetailName")).collect();
    assert!(named.is_empty(), "the full name's box over a count in the default state (R-20261112-2):\n{}", named.join("\n"));
    let _ = std::fs::remove_dir_all(&dir);
}
