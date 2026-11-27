//! Eclipse's game page in the program, for the eclipse_game tests (EmuSen_Qt.md §3.4.12): a library of the board's game
//! with the states asked for, the program under Eclipse with its page opened, and the board's table held at every size.
#![allow(dead_code)]

use crate::common::{draw_covers, sandbox, wait_for};
use crate::eclipse_board::{held, object, read, rows, tokens, Row};
use cxx_qt::casting::Upcast;
use cxx_qt_lib::QString;
use emusen_client_fixture::library;
use emusen_crystal_client::{Api, Config, Runtime};
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::bindings::consts::PROFILES;
use emusen_platform_client::galaxia::consts::{
    DIR_PACKAGES, DIR_SCREENSHOTS, FILE_GAMES, MODEL_APP_SETTINGS, MODEL_CHEAT_FILE, SAVE_PACKAGE_RESUME_STATE, SAVE_PACKAGE_STATE, SAVE_RESUME_STATE, SAVE_STATE,
};
use emusen_platform_client::packages::Roots;
use emusen_platform_client::library::Library;
use emusen_platform_client::tools::{Step, ToolJob};
use emusen_platform_client::{Lent, Platform};
use emusen_qt::MAIN_URL;
use emusen_qt_models::game_page_model::qobject::GamePageModel;
use emusen_qt_models::library_model::qobject::QAbstractListModel;
use emusen_qt_models::words;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::{Harness, Scene};
use emusen_theme_qt::host;
use serde_json::json;
use std::path::{Path, PathBuf};

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
        platform
            .model_save(MODEL_CHEAT_FILE, Some(&list.1), &format!("{}.json", list.0), &json!({ "Cheats": [cheat("Infinite lives"), cheat("Moon jump")] }))
            .unwrap();
    }
    for saved in states {
        save_state(&world, *saved);
    }
    world
}

/// A state's file: the resume point's, or a slot's.
pub fn state_file(world: &emusen_client_fixture::World, slot: i32) -> PathBuf {
    let platform = Platform::load(library()).unwrap();
    let (kind, n) = if slot == RESUME { (SAVE_RESUME_STATE, 0) } else { (SAVE_STATE, slot) };
    platform.save_path(kind, &rom(world).to_string_lossy(), None, n).unwrap()
}

/// A state written at its moment, as the Pause Card saves one.
pub fn save_state(world: &emusen_client_fixture::World, (slot, at): Saved) {
    let file = state_file(world, slot);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, b"state").unwrap();
    let f = std::fs::File::options().write(true).open(&file).unwrap();
    f.set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(at as u64)).unwrap();
}

/// Every state of the game's gone, the resume point's and the slots'.
pub fn clear_states(world: &emusen_client_fixture::World) {
    for slot in std::iter::once(RESUME).chain(1..=emusen_qt_models::pause::SLOTS as i32) {
        let _ = std::fs::remove_file(state_file(world, slot));
    }
}

/// The game's page shown afresh: the page left by B and opened again through Eclipse's Home, so the page reads the states
/// as they stand, then its States tab.
pub fn reopen(scene: &mut Scene<'_>, world: &emusen_client_fixture::World) {
    scene.pad(UiButton::Back);
    wait_for(scene, "screens", "shown", "Home", 3000);
    open_page(scene, world);
    states_tab(scene);
}

/// The centre of the drawn item of `name`, in the window's pixels.
pub fn centre(scene: &mut Scene<'_>, name: &str) -> (f64, f64) {
    let tree = scene.tree();
    let n = tree.nodes.iter().rev().find(|n| n.name == name && n.visible).unwrap_or_else(|| panic!("{name} is drawn"));
    (n.x + n.w / 2.0, n.y + n.h / 2.0)
}

/// The game started by the page, and the state it was asked to start from, the resume point's or a slot's file, or none.
pub fn started(scene: &mut Scene<'_>) -> String {
    wait_for(scene, "screens", "shown", "Game", 3000);
    scene.property("session", "startState")
}

/// The program under Eclipse at 1280 by 800, its library read, Home placed, the clock held at today's 9:41 PM, one pad of
/// the family whose letters the board draws, and the machine's battery at the board's 82% (EclipseGame:38).
pub fn program<'a>(harness: &'a mut Harness) -> Scene<'a> {
    program_with(harness, None, None)
}

/// As `program`, naming a media store whose scraped pictures the theme may draw.
pub fn program_with_media<'a>(harness: &'a mut Harness, media: Option<&Path>) -> Scene<'a> {
    program_with(harness, media, None)
}

/// As `program`, naming an optional media store and an optional cores folder.
pub fn program_with<'a>(harness: &'a mut Harness, media: Option<&Path>, cores: Option<&Path>) -> Scene<'a> {
    program_full(harness, media, cores, None)
}

/// As `program_with`, also wiring an optional `scraperTest` JSON string.
pub fn program_full<'a>(harness: &'a mut Harness, media: Option<&Path>, cores: Option<&Path>, scraper_test: Option<&str>) -> Scene<'a> {
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    if let Some(dir) = media {
        assert!(scene.set_property("program", "coverStores", dir.to_str().unwrap()));
    }
    if let Some(dir) = cores {
        assert!(scene.set_property("library", "cores", dir.to_str().unwrap()));
    }
    if let Some(test) = scraper_test {
        assert!(scene.set_property("program", "scraperTest", test));
    }
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

/// The board's description saved as an edit for Aurora Drift (EclipseGameOverview:88).
pub const BOARD_DESCRIPTION: &str = "Aurora Drift is a game invented for EmuSen's tests. It has no story and no pictures of its own; every image shown for it is a flat colour with its name written on it.";

pub fn set_description(world: &emusen_client_fixture::World, text: &str) {
    let platform = Platform::load(library()).unwrap();
    let path = rom(world).to_string_lossy().into_owned();
    let games = records(&platform);
    games.change(&json!({ "op": "save_edits", "path": path, "changes": { "description": text }, "at": "2026-11-09T00:00:00Z" })).unwrap();
}

/// Draws cover and screenshot PNGs into the world's ES-DE media store and registers the folder in appsettings.json.
pub fn add_media(harness: &mut Harness, world: &emusen_client_fixture::World, folders: &[&str]) -> PathBuf {
    let platform = Platform::load(library()).unwrap();
    let media = world.dir("EsdeMedia");
    let pictures: Vec<_> = folders.iter().enumerate().map(|(i, f)| (media.join("snes").join(f).join(format!("{TITLE}.png")), (560, 315), i, String::new())).collect();
    draw_covers(harness, &world.root, &pictures, |_| {});
    let mut settings = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    settings["EsdeMediaDirectory"] = json!(media.to_str().unwrap());
    settings["OpenEmuFallback"] = json!(false);
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    media
}

/// Sets up Media board (#78) fixtures: two player screenshots, Title screen, Clip, Manual, plus a cover (returning `world.root` as cover store).
pub fn add_board_media(harness: &mut Harness, world: &emusen_client_fixture::World) -> PathBuf {
    let platform = Platform::load(library()).unwrap();
    let media = add_media(harness, world, &["covers", "titlescreens"]);
    let shots = platform.directory(DIR_SCREENSHOTS).unwrap().unwrap();
    std::fs::create_dir_all(&shots).unwrap();
    let rom_path = rom(world).to_string_lossy().into_owned();
    let taken = [
        (shots.join(emusen_qt_models::screenshots::name(&rom_path, "2026-11-09", "14.03.11.251")), (560, 315), 2, String::new()),
        (shots.join(emusen_qt_models::screenshots::name(&rom_path, "2026-11-09", "14.02.11.251")), (560, 315), 3, String::new()),
    ];
    draw_covers(harness, &world.root, &taken, |_| {});
    let clip = media.join("snes/videos").join(format!("{TITLE}.mkv"));
    std::fs::create_dir_all(clip.parent().unwrap()).unwrap();
    emusen_qt_test::clips::write(&clip, 2);
    let manual = media.join("snes/manuals").join(format!("{TITLE}.pdf"));
    std::fs::create_dir_all(manual.parent().unwrap()).unwrap();
    std::fs::write(&manual, b"%PDF-1.4\n%%EOF\n").unwrap();
    world.root.clone()
}

/// Replaces the game's cheat file with the named (description, enabled) switches.
pub fn add_cheats(world: &emusen_client_fixture::World, switches: &[(&str, bool)]) {
    let platform = Platform::load(library()).unwrap();
    let path = rom(world).to_string_lossy().into_owned();
    let rt = Runtime::new(&Api::load(library()).unwrap(), &Config::default()).unwrap();
    let console = rt.systems().unwrap().into_iter().find(|s| s.id == "snes").expect("the SNES").console;
    drop(rt);
    let list = platform.cheat_game_list(Some(&console), &path).unwrap().expect("a list for the game");
    let cheats: Vec<_> = switches
        .iter()
        .map(|(d, on)| json!({ "Kind": "RamPoke", "Description": d, "Enabled": on, "Writes": [{ "Space": "WRAM", "Address": "7E0DBE", "Value": "09" }] }))
        .collect();
    platform.model_save(MODEL_CHEAT_FILE, Some(&list.1), &format!("{}.json", list.0), &json!({ "Cheats": cheats })).unwrap();
}

/// The twelve cheats of board #81 (3 on, 9 off): five drawn rows plus seven below the fold.
pub const BOARD_CHEATS: [(&str, bool); 12] = [
    ("Infinite lives", true),
    ("Start with all keys", true),
    ("Moon jump", false),
    ("Every level open", true),
    ("Slow motion", false),
    ("Extra shield", false),
    ("Fast charge", false),
    ("Keep power-ups", false),
    ("No knockback", false),
    ("Double score", false),
    ("Stage select", false),
    ("Sound test", false),
];

/// Sets up Options board data: two SNES core sidecars with Venus override, two controller profiles, and a 2-layer package.
pub fn add_options(world: &emusen_client_fixture::World) -> PathBuf {
    let platform = Platform::load(library()).unwrap();
    let cores = world.dir("Cores");
    for (engine, name, hash) in [("VenusRT (Rust)", "VenusRT", "01"), ("Snes9x (Rust)", "Snes9x", "02")] {
        let systems = [json!({ "id": "snes", "name": "snes", "extensions": [], "regions": ["ntsc"], "controllers": [], "firmware": [] })];
        let sidecar = json!({ "sidecar": 1, "library": format!("lib{}.so", name.to_lowercase()), "sha256": hash, "abi": "1.19", "capabilities": 0,
            "info": { "abi": "1.19", "id": name.to_lowercase(), "name": name, "display_name": engine, "version": "0.1.0", "license": "GPL-3.0-or-later",
                "authors": ["EmuSen"], "description": "", "systems": systems, "capabilities": [], "host_requires": [], "deterministic": true }, "settings": [] });
        std::fs::write(cores.join(format!("lib{}.so.core.json", name.to_lowercase())), sidecar.to_string()).unwrap();
    }
    let rt = Runtime::new(&Api::load(library()).unwrap(), &Config { cores: Some(vec![cores.to_str().unwrap().into()]), ..Config::default() }).unwrap();
    let snes = rt.systems().unwrap().into_iter().find(|s| s.id == "snes").expect("the SNES");
    drop(rt);
    let path = rom(world).to_string_lossy().into_owned();
    let games = records(&platform);
    games.change(&json!({ "op": "save_edits", "path": path, "changes": { emusen_qt_models::edits::ALT_EMULATOR: "VenusRT (Rust)" }, "at": "2026-11-09T00:00:00Z" })).unwrap();
    drop(games);
    let mut profiles = platform.bindings_load(PROFILES, &[snes.console.as_str()]).unwrap_or_else(|_| json!({}));
    profiles[snes.console.as_str()] = json!({
        "Default": null,
        "Profiles": { "Fighter": { "1": {} }, "Arcade": { "1": {} } },
        "Games": {}
    });
    platform.bindings_save(PROFILES, &profiles).unwrap();
    let run_job = |req: serde_json::Value| {
        let job = ToolJob::start(&platform, &req).unwrap();
        loop {
            match job.next(1000).unwrap() {
                Step::Ended => break,
                _ => {}
            }
        }
    };
    let rel = format!("SNES/{TITLE}.sfc");
    run_job(json!({ "op": "pkg-make", "paths": [&rel] }));
    let patch_dir = world.dir("Patches");
    for (name, byte) in [("layer1.ips", 0x41u8), ("layer2.ips", 0x42u8)] {
        let p = patch_dir.join(name);
        std::fs::write(&p, [b'P', b'A', b'T', b'C', b'H', 0, 0, 0x10, 0, 1, byte, b'E', b'O', b'F']).unwrap();
        run_job(json!({ "op": "pkg-patch-add", "paths": [&rel], "patch": p.to_str().unwrap() }));
    }
    let packages = platform.directory(DIR_PACKAGES).unwrap().unwrap();
    let library_dir = world.root.join("Roms");
    let compressed = world.dir("Compressed");
    let index = platform
        .package_index(Roots {
            packages: packages.to_str().unwrap(),
            library: Some(library_dir.to_str().unwrap()),
            compressed: compressed.to_str().unwrap(),
        })
        .unwrap();
    if let Some(folder) = index["packages"].as_array().and_then(|a| a.first()).and_then(|p| p["folder"].as_str()) {
        for (slot, at) in board_states() {
            let (kind, n) = if slot == RESUME { (SAVE_PACKAGE_RESUME_STATE, 0) } else { (SAVE_PACKAGE_STATE, slot) };
            let file = platform.save_path(kind, folder, None, n).unwrap();
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(&file, b"state").unwrap();
            let f = std::fs::File::options().write(true).open(&file).unwrap();
            f.set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(at as u64)).unwrap();
        }
    }
    cores
}

/// The tab currently shown by the page's model (0 Overview, 1 States, 2 Media, 3 Details, 4 Options, 5 Cheats).
pub fn page_tab(scene: &mut Scene<'_>) -> i32 {
    let object = scene.object("pageModel");
    assert!(!object.is_null(), "the page's model");
    // Safety: the window's model of that name is a GamePageModel, alive while the window is.
    let page = unsafe { &*GamePageModel::from_base_ptr(object as *const QAbstractListModel) };
    page.tab()
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

/// The States tab opened by the right shoulder from Overview, and Eclipse's screen shown for it.
pub fn states_tab(scene: &mut Scene<'_>) {
    scene.pad(UiButton::PageDown);
    until(scene, "Eclipse's screen drawing the States tab", 3000, |s| page_tab(s) == 1 && s.property("theme", "pageDrawn") == "true");
    until(scene, "a card or a pill holding the focus", 3000, |s| {
        let f = focused(s);
        f.starts_with("card.") || f.starts_with("pill.")
    });
    scene.wait(300);
}

/// Opens a tab drawn by Eclipse (0 Overview, 1 States, 3 Details, 4 Options) by tapping it.
pub fn open_tab(scene: &mut Scene<'_>, tab: i32) {
    let (x, y) = centre(scene, &format!("tab.{tab}"));
    scene.click(x, y);
    until(scene, &format!("Eclipse drawing tab {tab}"), 3000, |s| page_tab(s) == tab && s.property("theme", "pageDrawn") == "true");
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
                    let p = |q: &str| {
                        emusen_qt_test::ffi::ffi::harness_property(object(id), &QString::from(tree.nodes[c].name.as_str()), &QString::from(q)).to_string()
                    };
                    if p("named") == "true" {
                        p("word")
                    } else {
                        p("letter")
                    }
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
        // A card or media tile the row has carried past the window's right margin is not drawn (R-20261123-33).
        held(
            &tree,
            &present,
            k,
            unit,
            canvas_width,
            &tokens,
            &at,
            &|name, x, w| (name.starts_with("card.") || name.starts_with("media.")) && x + w > canvas_width - 96.0 * k,
            differences,
        );
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
        if rows(table).1.iter().any(|r| r.name == "helpSections") {
            let shoulders = glyphs(&tree, &["ownGlyph-Section", "ownGlyph2-Section"]);
            let place = table.lines().find(|l| l.starts_with("helpSectionsPlace\t")).and_then(|l| l.split('\t').nth(2));
            let want = match place {
                Some(p) => vec!["Section", "·", p.trim_matches('"')],
                None => vec!["Section"],
            };
            if shoulders != ["LB", "RB"] || words_under(&tree, "helpSections") != want {
                differences.push(format!("{at}: the shoulders read {shoulders:?} {:?} where the board draws LB RB {want:?}", words_under(&tree, "helpSections")));
            }
        } else if tree.nodes.iter().any(|n| n.name == "helpSections" && n.visible) {
            differences.push(format!("{at}: helpSections is visible where the board draws none"));
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

/// Whether games.db holds the game as a favourite.
pub fn favourite(world: &emusen_client_fixture::World) -> bool {
    let platform = Platform::load(library()).unwrap();
    let all = records(&platform).get(&json!({ "what": "records" })).unwrap();
    all.get(rom(world).to_string_lossy().as_ref()).and_then(|r| r.get("favourite")).and_then(serde_json::Value::as_bool).unwrap_or(false)
}

/// Saves manual edit fields for the board's game in games.db.
pub fn set_edits(world: &emusen_client_fixture::World, changes: serde_json::Value) {
    let platform = Platform::load(library()).unwrap();
    let path = rom(world).to_string_lossy().into_owned();
    let games = records(&platform);
    games.change(&json!({ "op": "save_edits", "path": path, "changes": changes, "at": "2026-11-09T00:00:00Z" })).unwrap();
}

/// Saves a per-game display crop in games.db.
pub fn set_crop(world: &emusen_client_fixture::World, crop: emusen_qt_models::display::Crop) {
    let platform = Platform::load(library()).unwrap();
    let path = rom(world).to_string_lossy().into_owned();
    let games = records(&platform);
    games.change(&crop.set_record(&path)).unwrap();
}

/// Builds a minimal valid PDF of `pages` pages for Media Viewer tests.
pub fn mini_pdf(pages: usize) -> Vec<u8> {
    let kids: Vec<String> = (0..pages).map(|i| format!("{} 0 R", 3 + i)).collect();
    let mut objects = vec!["<< /Type /Catalog /Pages 2 0 R >>".to_owned(), format!("<< /Type /Pages /Kids [{}] /Count {pages} >>", kids.join(" "))];
    objects.extend((0..pages).map(|_| "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>".to_owned()));
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (i, object) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend(format!("{} 0 obj\n{object}\nendobj\n", i + 1).bytes());
    }
    let xref = out.len();
    out.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).bytes());
    for at in offsets {
        out.extend(format!("{at:010} 00000 n \n").bytes());
    }
    out.extend(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1).bytes());
    out
}

/// Sets up Media Viewer fixtures: two 512x448 player screenshots (mtime 1 November 2026), Scraped screenshot, Title screen, Clip, 3-page Manual, and Cover.
pub fn add_viewer_media(harness: &mut Harness, world: &emusen_client_fixture::World) -> (PathBuf, PathBuf) {
    let platform = Platform::load(library()).unwrap();
    let media = add_media(harness, world, &["covers", "screenshots", "titlescreens"]);
    let shots = platform.directory(DIR_SCREENSHOTS).unwrap().unwrap();
    std::fs::create_dir_all(&shots).unwrap();
    let rom_path = rom(world).to_string_lossy().into_owned();
    let shot1 = shots.join(emusen_qt_models::screenshots::name(&rom_path, "2026-11-01", "14.03.11.251"));
    let shot2 = shots.join(emusen_qt_models::screenshots::name(&rom_path, "2026-11-01", "14.02.11.251"));
    let taken = [(shot1.clone(), (512, 448), 2, String::new()), (shot2.clone(), (512, 448), 3, String::new())];
    draw_covers(harness, &world.root, &taken, |_| {});
    let nov1 = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_793_491_200);
    for p in [&shot1, &shot2] {
        std::fs::File::options().write(true).open(p).unwrap().set_modified(nov1).unwrap();
    }
    let clip = media.join("snes/videos").join(format!("{TITLE}.mkv"));
    std::fs::create_dir_all(clip.parent().unwrap()).unwrap();
    emusen_qt_test::clips::write(&clip, 2);
    let manual = media.join("snes/manuals").join(format!("{TITLE}.pdf"));
    std::fs::create_dir_all(manual.parent().unwrap()).unwrap();
    std::fs::write(&manual, mini_pdf(3)).unwrap();
    (world.root.clone(), shot1)
}

/// Loopback HTTPS ScreenScraper mock answering jeuRecherche, jeuInfos, and PNG media for Scrape This Game tests.
pub struct MockScraper {
    child: std::process::Child,
    pub test: String,
    asked: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl MockScraper {
    /// Starts the local HTTPS mock with a self-signed cert in `dir` and writes the developer credentials file in `config`.
    pub fn start(dir: &Path, config: &Path) -> Self {
        use std::io::{BufRead, BufReader};
        use std::process::{Command, Stdio};
        emusen_client_fixture::scraper::RecordedScraper::developer(config);
        std::fs::create_dir_all(dir).unwrap();
        let ca_cert = dir.join("ca.pem");
        let ca_key = dir.join("ca.key");
        let leaf_cert = dir.join("leaf.pem");
        let leaf_key = dir.join("leaf.key");
        let leaf_csr = dir.join("leaf.csr");
        let ext = dir.join("ext.cnf");
        std::fs::write(&ext, "subjectAltName=DNS:api.screenscraper.fr,DNS:neoclone.screenscraper.fr,DNS:localhost\nbasicConstraints=CA:FALSE\nkeyUsage=digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\n").unwrap();
        let run_ssl = |args: &[&str]| {
            let ok = Command::new("openssl").args(args).stdout(Stdio::null()).stderr(Stdio::null()).status().expect("openssl runs").success();
            assert!(ok, "openssl failed: {args:?}");
        };
        run_ssl(&["req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "1", "-keyout", ca_key.to_str().unwrap(), "-out", ca_cert.to_str().unwrap(), "-subj", "/CN=EmuSen Test Root", "-addext", "basicConstraints=critical,CA:TRUE", "-addext", "keyUsage=critical,keyCertSign,cRLSign"]);
        run_ssl(&["req", "-new", "-newkey", "rsa:2048", "-nodes", "-keyout", leaf_key.to_str().unwrap(), "-out", leaf_csr.to_str().unwrap(), "-subj", "/CN=api.screenscraper.fr"]);
        run_ssl(&["x509", "-req", "-in", leaf_csr.to_str().unwrap(), "-CA", ca_cert.to_str().unwrap(), "-CAkey", ca_key.to_str().unwrap(), "-CAcreateserial", "-out", leaf_cert.to_str().unwrap(), "-days", "1", "-extfile", ext.to_str().unwrap()]);
        let pem = std::fs::read_to_string(&ca_cert).unwrap();
        let script = r#"
import http.server, json, ssl, sys, urllib.parse
cert, key = sys.argv[1], sys.argv[2]
PNG = bytes([0x89,0x50,0x4E,0x47,0x0D,0x0A,0x1A,0x0A,0x00,0x00,0x00,0x0D,0x49,0x48,0x44,0x52,0x00,0x00,0x00,0x01,0x00,0x00,0x00,0x01,0x08,0x06,0x00,0x00,0x00,0x1F,0x15,0xC4,0x89,0x00,0x00,0x00,0x0A,0x49,0x44,0x41,0x54,0x78,0x9C,0x63,0x00,0x01,0x00,0x00,0x05,0x00,0x01,0x0D,0x0A,0x2D,0xB4,0x00,0x00,0x00,0x00,0x49,0x45,0x4E,0x44,0xAE,0x42,0x60,0x82]) + bytes(4096)
MATCHES = [
    {"id":"101","systeme":{"id":"4"},"noms":[{"region":"us","text":"Aurora Drift"}],"dates":[{"region":"us","text":"1994-03-14"}],"medias":[{"type":"box-2D","region":"us","url":"https://neoclone.screenscraper.fr/cover.png","format":"png"}]},
    {"id":"102","systeme":{"id":"9"},"noms":[{"region":"us","text":"Aurora Drift"}],"dates":[{"region":"us","text":"1995-05-01"}],"medias":[{"type":"box-2D","region":"us","url":"https://neoclone.screenscraper.fr/cover2.png","format":"png"}]},
    {"id":"103","systeme":{"id":"4"},"noms":[{"region":"us","text":"Aurora Drifters"}],"dates":[{"region":"us","text":"1996-08-12"}],"medias":[{"type":"box-2D","region":"us","url":"https://neoclone.screenscraper.fr/cover3.png","format":"png"}]}
]
class H(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    def log_message(self, *a): pass
    def send_body(self, code, ctype, body):
        self.send_response(code); self.send_header("Content-Type", ctype); self.send_header("Content-Length", str(len(body))); self.send_header("Connection", "close"); self.end_headers(); self.wfile.write(body)
    def do_GET(self):
        print("asked " + self.path, flush=True)
        q = urllib.parse.parse_qs(urllib.parse.urlsplit(self.path).query)
        if "jeuRecherche.php" in self.path:
            term = (q.get("recherche") or [""])[0].lower()
            if "fail" in term:
                self.send_body(500, "text/plain", b"server error"); return
            games = [] if "empty" in term else MATCHES
            self.send_body(200, "application/json", json.dumps({"response": {"jeux": games}}).encode())
        elif "jeuInfos.php" in self.path:
            self.send_body(200, "application/json", json.dumps({"response": {"jeu": MATCHES[0]}}).encode())
        else:
            self.send_body(200, "image/png", PNG)
srv = http.server.ThreadingHTTPServer(("127.0.0.1", 0), H)
ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
ctx.load_cert_chain(cert, key)
srv.socket = ctx.wrap_socket(srv.socket, server_side=True)
print(srv.server_address[1], flush=True)
srv.serve_forever()
"#;
        let mut child = Command::new("python3")
            .args(["-c", script, leaf_cert.to_str().unwrap(), leaf_key.to_str().unwrap()])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("python3 mock scraper starts");
        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
        let port: u16 = lines.next().and_then(Result::ok).expect("mock scraper port").trim().parse().unwrap();
        let asked = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let kept = asked.clone();
        std::thread::spawn(move || {
            for line in lines.map_while(Result::ok) {
                if let Some(p) = line.strip_prefix("asked ") {
                    kept.lock().unwrap_or_else(|e| e.into_inner()).push(p.to_owned());
                }
            }
        });
        let test = json!({ "connect": format!("127.0.0.1:{port}"), "roots": pem }).to_string();
        Self { child, test, asked }
    }

    /// Paths requested so far.
    pub fn asked(&self) -> Vec<String> {
        self.asked.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

impl Drop for MockScraper {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
