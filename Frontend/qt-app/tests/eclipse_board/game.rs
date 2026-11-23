//! Eclipse's game page in the program, for the eclipse_game tests (EmuSen_Qt.md §3.4.11): a library of the board's game
//! with the states asked for, the program under Eclipse with its page opened, and the board's table held at every size.
#![allow(dead_code)]

use crate::common::{sandbox, wait_for};
use crate::eclipse_board::{held, object, read, rows, tokens, Row};
use cxx_qt_lib::QString;
use emusen_client_fixture::library;
use emusen_crystal_client::{Api, Config, Runtime};
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::{FILE_GAMES, MODEL_CHEAT_FILE, SAVE_RESUME_STATE, SAVE_STATE};
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt::MAIN_URL;
use emusen_qt_models::words;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::{Harness, Scene};
use emusen_theme_qt::host;
use serde_json::json;
use std::path::PathBuf;

pub const ECLIPSE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../themes/eclipse");
/// The board's game (EclipseGame:34).
pub const TITLE: &str = "Aurora Drift";
/// The resume point's slot as `Saved` names it.
pub const RESUME: i32 = 0;

const DAY: i64 = 86_400;

/// A state of the game's: its slot (RESUME for the resume point) and the moment its file was written.
pub type Saved = (i32, i64);

/// Today's midnight in UTC, the zone the tests set, so the board's clock times fall on the days the board names.
pub fn midnight() -> i64 {
    words::now().div_euclid(DAY) * DAY
}

/// The board's four states (EclipseGame:107-112): the resume point and slot 1 yesterday at 9:12 and 8:40 PM, slot 2 and
/// slot 3 two and three weeks back at 7:05 and 10:31 PM.
pub fn board_states() -> Vec<Saved> {
    let today = midnight();
    vec![
        (RESUME, today - DAY + 21 * 3600 + 12 * 60),
        (1, today - DAY + 20 * 3600 + 40 * 60),
        (2, today - 15 * DAY + 19 * 3600 + 5 * 60),
        (3, today - 21 * DAY + 22 * 3600 + 31 * 60),
    ]
}

const MONTHS: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

/// A day's date in the proleptic Gregorian calendar (H. Hinnant, "chrono-Compatible Low-Level Date Algorithms", civil_from_days).
pub fn civil(at: i64) -> (i64, usize, i64) {
    let z = at.div_euclid(DAY) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m as usize, d)
}

/// A state's when-line as the board writes it and the tester ruled the rest (EclipseGame:108-111, R-20261123-34), read here
/// from the calendar, not from the program: "Today", "Yesterday", "8 November", "8 November 2025", then the US clock time.
pub fn when_words(at: i64) -> String {
    let secs = at.rem_euclid(DAY);
    let (h, m) = (secs / 3600, secs % 3600 / 60);
    let time = format!("{}:{m:02} {}", if h % 12 == 0 { 12 } else { h % 12 }, if h < 12 { "AM" } else { "PM" });
    let (today, day) = (midnight() / DAY, at.div_euclid(DAY));
    let (y, mo, d) = civil(at);
    match today - day {
        0 => format!("Today, {time}"),
        1 => format!("Yesterday, {time}"),
        _ if civil(midnight()).0 == y => format!("{d} {}, {time}", MONTHS[mo - 1]),
        _ => format!("{d} {} {y}, {time}", MONTHS[mo - 1]),
    }
}

/// The game's file in the world.
pub fn rom(world: &emusen_client_fixture::World) -> PathBuf {
    world.root.join("Roms/SNES").join(format!("{TITLE}.sfc"))
}

fn records(platform: &std::sync::Arc<Platform>) -> Library {
    let sqlite = Lent::open("libsqlite3.so.0").unwrap();
    Library::open(platform, &platform.file(FILE_GAMES).unwrap(), &sqlite).unwrap()
}

/// The board's game in a world of its own: its release year, genre and players as edits, which the page's facts read first;
/// when `played`, last played 30 hours ago for 3 h 20 min and a favourite (EclipseGame:45, 49); two cheats when `cheats`,
/// so the Cheats tab shows (R-20261116-29); and each state of `states`, its file written at its moment. A state's picture
/// lies in the saves folder, which no theme may read (the host's cover stores), so none is written.
pub fn world(name: &str, states: &[Saved], played: bool, cheats: bool) -> emusen_client_fixture::World {
    // The board's zone and the US clock form, as the program's `QLocale::system()` and local zone read them.
    std::env::set_var("LC_ALL", "en_US.UTF-8");
    std::env::set_var("TZ", "UTC");
    let world = sandbox(name, &[("SNES", &format!("{TITLE}.sfc"))]);
    let platform = Platform::load(library()).unwrap();
    let path = rom(&world).to_string_lossy().into_owned();
    let games = records(&platform);
    let changes = json!({ "releasedate": "1994-03-18", "genre": "Action", "players": "1-2" });
    games.change(&json!({ "op": "save_edits", "path": path, "changes": changes, "at": "2026-11-09T00:00:00Z" })).unwrap();
    if played {
        games.change(&json!({ "op": "started", "path": path, "at": words::utc(words::now() - 30 * 3600) })).unwrap();
        games.change(&json!({ "op": "played", "path": path, "seconds": 12_000.0, "max_hours": 24 })).unwrap();
        games.change(&json!({ "op": "toggle_favourite", "path": path })).unwrap();
    }
    drop(games);
    if cheats {
        let rt = Runtime::new(&Api::load(library()).unwrap(), &Config::default()).unwrap();
        let console = rt.systems().unwrap().into_iter().find(|s| s.id == "snes").expect("the SNES").console;
        drop(rt);
        let list = platform.cheat_game_list(Some(&console), &path).unwrap().expect("a list for the game");
        let cheat = |description: &str| json!({ "Kind": "RamPoke", "Description": description, "Enabled": false, "Writes": [{ "Space": "WRAM", "Address": "7E0DBE", "Value": "09" }] });
        platform.model_save(MODEL_CHEAT_FILE, Some(&list.1), &format!("{}.json", list.0), &json!({ "Cheats": [cheat("Infinite lives"), cheat("Moon jump")] })).unwrap();
    }
    for (slot, at) in states {
        let (kind, n) = if *slot == RESUME { (SAVE_RESUME_STATE, 0) } else { (SAVE_STATE, *slot) };
        let file = platform.save_path(kind, &path, None, n).unwrap();
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, b"state").unwrap();
        let f = std::fs::File::options().write(true).open(&file).unwrap();
        f.set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(*at as u64)).unwrap();
    }
    world
}

/// The program under Eclipse at 1280 by 800, its library read, Home placed, the clock held at today's 9:41 PM, one pad of
/// the family whose letters the board draws, and the machine's battery at the board's 82% (EclipseGame:38).
pub fn program<'a>(harness: &'a mut Harness) -> Scene<'a> {
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("program", "theme", &std::fs::canonicalize(ECLIPSE).unwrap().display().to_string()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    assert_eq!(scene.property("theme", "failed"), "false", "{}", scene.property("theme", "warnings"));
    assert!(scene.set_property("clock", "now", &(midnight() + 21 * 3600 + 41 * 60).to_string()));
    let family = emusen_kit_qt::kinds::family_of(emusen_kit_core::glyph::PadFamily::Xbox).repr;
    assert!(scene.set_property("theme", "family", &family.to_string()));
    assert!(scene.set_property("theme", "battery", "82"));
    scene.wait(300);
    scene
}

/// The name of the item holding the focus.
pub fn focused(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default()
}

pub fn until(scene: &mut Scene<'_>, what: &str, ms: i32, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    let mut waited = 0;
    while !done(scene) && waited < ms {
        scene.wait(25);
        waited += 25;
    }
    let tree = scene.tree();
    let at = scene.focus(&tree).map(|i| (tree.nodes[i as usize].name.clone(), tree.nodes[i as usize].cls.clone()));
    let cards = ["currentIndex", "count"].map(|p| read(&tree, "gameCards", p));
    assert!(done(scene), "{what} within {ms} ms; the focus on {at:?}; the cards' {cards:?}; {}", scene.property("theme", "warnings"));
}

/// The theme's object of `name`, by its number in the tree, for its views.
pub fn theme_object(scene: &mut Scene<'_>, name: &str) -> u64 {
    let tree = scene.tree();
    tree.nodes[tree.find_named(name).unwrap_or_else(|| panic!("{name} is placed")) as usize].id.parse().unwrap()
}

/// The game's page opened as Eclipse's Home opens it, through the theme's own `Actions.openPage`, at its first tab, which
/// the program draws.
pub fn open_page(scene: &mut Scene<'_>, world: &emusen_client_fixture::World) {
    until(scene, "Eclipse's Home placed", 3000, |s| s.tree().find_named("homeScreen").is_some());
    let home = theme_object(scene, "homeScreen");
    let key = QString::from(rom(world).to_string_lossy().as_ref());
    assert!(host::actions(object(home)).expect("Actions").open_page(&key), "Actions.openPage is taken");
    wait_for(scene, "screens", "shown", "Page", 3000);
    wait_for(scene, "pageModel", "ready", "true", 10000);
    scene.wait(300);
}

/// The States tab opened by the right shoulder from the Overview, the program's, and Eclipse's screen shown for it.
pub fn states_tab(scene: &mut Scene<'_>) {
    scene.pad(UiButton::PageDown);
    until(scene, "Eclipse's screen drawing the States tab", 3000, |s| s.property("theme", "pageDrawn") == "true");
    until(scene, "a card or a pill holding the focus", 3000, |s| {
        let f = focused(s);
        f.starts_with("card.") || f.starts_with("pill.")
    });
    scene.wait(300);
}

/// The texts of the visible words under the named item, in the tree's order.
pub fn words_under(tree: &emusen_qt_test::tree::Tree, name: &str) -> Vec<String> {
    let Some(top) = tree.nodes.iter().rposition(|n| n.name == name && n.visible) else { return Vec::new() };
    let top = tree.nodes[top].id.clone();
    let under = |n: &emusen_qt_test::tree::Node| {
        let mut at = n.parent.clone();
        while let Some(p) = tree.nodes.iter().find(|p| p.id == at) {
            if p.id == top {
                return true;
            }
            at = p.parent.clone();
        }
        false
    };
    tree.nodes.iter().filter(|n| n.visible && n.ty.contains("Text") && under(n)).filter_map(|n| n.text.clone()).filter(|t| !t.is_empty()).collect()
}

/// The letters or names of a help line's glyphs, each the kit's glyph under Eclipse's `ownGlyph-<label>`.
pub fn glyphs(tree: &emusen_qt_test::tree::Tree, names: &[&str]) -> Vec<String> {
    names
        .iter()
        .map(|g| {
            let g = tree.nodes.iter().rposition(|n| n.name == *g && n.visible);
            let child = g.and_then(|g| tree.nodes.iter().position(|n| n.parent == tree.nodes[g].id));
            child
                .map(|c| {
                    let id: u64 = tree.nodes[c].id.parse().unwrap();
                    let p = |q: &str| emusen_qt_test::ffi::ffi::harness_property(object(id), &QString::from(tree.nodes[c].name.as_str()), &QString::from(q)).to_string();
                    if p("named") == "true" { p("word") } else { p("letter") }
                })
                .unwrap_or_default()
        })
        .collect()
}

/// Every row of `table` held at each of the gate's sizes, the words of `words` with them (`true`: compared in capitals), the
/// help line's words and glyphs, and the shoulders' Section; each difference pushed.
pub fn hold_board(scene: &mut Scene<'_>, table: &str, words: &[(&str, String, bool)], help: &[&str], help_glyphs: &[&str], differences: &mut Vec<String>) {
    let screen = theme_object(scene, "gameScreen");
    let tokens = tokens(screen);
    let board_height = rows(table).0;
    let k = DESIGN_HEIGHT / board_height;
    let focus = focused(scene);
    for (width, height) in SIZES {
        scene.resize(width, height);
        scene.wait(300);
        let tree = scene.tree();
        let unit: f64 = read(&tree, "gameCanvas", "unit").parse().expect("the canvas's unit");
        let canvas_width = width as f64 / unit;
        let at = format!("{width}x{height}");
        if focused(scene) != focus {
            differences.push(format!("{at}: the focus moved from {focus} to {}", focused(scene)));
        }
        // A built row with nothing of its name in the tree is a difference, and the rest are held.
        let (present, missing): (Vec<Row>, Vec<Row>) = rows(table).1.into_iter().partition(|r| r.status != "built" || tree.find_named(&r.name).is_some());
        differences.extend(missing.iter().map(|r| format!("{at}: {} is not drawn", r.name)));
        // A card the row has carried past the window's right margin is not drawn (R-20261123-33).
        held(&tree, &present, k, unit, canvas_width, &tokens, &at, &|name, x, w| name.starts_with("card.") && x + w > canvas_width - 96.0 * k, differences);
        for (name, want, upper) in words {
            let have = read(&tree, name, "text");
            let drawn = if *upper { have.to_uppercase() } else { have.clone() };
            if drawn != *want {
                differences.push(format!("{at}: {name} says {drawn:?} where the board says {want:?}"));
            }
        }
        let line = words_under(&tree, "help");
        if line != help {
            differences.push(format!("{at}: the help line says {line:?} where the board says {help:?}"));
        }
        let names: Vec<String> = help.iter().map(|w| format!("ownGlyph-{w}")).collect();
        let letters = glyphs(&tree, &names.iter().map(String::as_str).collect::<Vec<_>>());
        if letters != help_glyphs {
            differences.push(format!("{at}: the help line's glyphs read {letters:?} where the board draws {help_glyphs:?}"));
        }
        let shoulders = glyphs(&tree, &["ownGlyph-Section", "ownGlyph2-Section"]);
        if shoulders != ["LB", "RB"] || words_under(&tree, "helpSections") != ["Section"] {
            differences.push(format!("{at}: the shoulders read {shoulders:?} {:?} where the board draws LB RB Section", words_under(&tree, "helpSections")));
        }
        // Nothing the board draws whole is cut.
        for n in tree.nodes.iter().filter(|n| n.visible && n.text.is_some() && !n.name.is_empty()) {
            if read(&tree, &n.name, "truncated") == "true" {
                differences.push(format!("{at}: {} is cut: {:?}", n.name, read(&tree, &n.name, "text")));
            }
        }
    }
    scene.resize(1280, 800);
    scene.wait(300);
}

/// With `EMUSEN_ECLIPSE_PICTURES` naming a folder, the page grabbed at the board's shape into it as `name`.
pub fn picture(scene: &mut Scene<'_>, name: &str) {
    let Some(folder) = std::env::var_os("EMUSEN_ECLIPSE_PICTURES") else { return };
    std::fs::create_dir_all(&folder).unwrap();
    scene.resize(1920, 1200);
    scene.wait(500);
    let path = std::path::Path::new(&folder).join(name);
    assert!(scene.grab(&path.display().to_string()), "{} is grabbed", path.display());
    scene.resize(1280, 800);
    scene.wait(300);
}
