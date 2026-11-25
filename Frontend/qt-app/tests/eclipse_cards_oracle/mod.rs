//! Shared oracle for Eclipse's program-drawn Cards frame across boards #5, #44, #45 and #46 (R-20261113-5).
#![allow(dead_code)]

use crate::common::{draw_covers, sandbox, wait_for};
use crate::eclipse_board::{hold_card as hold_board, object, painted, picture, read, tokens};
use cxx_qt_lib::QString;
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_kit_core::tokens::Colour;
use emusen_platform_client::galaxia::consts::{FILE_GAMES, MODEL_APP_SETTINGS};
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt::MAIN_URL;
use emusen_qt_models::{home_card, words};
use emusen_qt_test::{Harness, Scene};
use serde_json::{json, Value};

const ECLIPSE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../themes/eclipse");
const DAY: i64 = 86_400;
const TABLE_HOME: &str = include_str!("../eclipse/card_home.tsv");
const TABLE_LIST: &str = include_str!("../eclipse/card_list.tsv");
const TABLE_FILTER: &str = include_str!("../eclipse/card_filter.tsv");
const TABLE_START: &str = include_str!("../eclipse/card_start_console.tsv");

const PSX_TITLES: [&str; 12] = [
    "Amber Coast", "Brass Lantern", "Cinder Mile", "Dune Signal", "Ember Drift", "Frost Gate",
    "Gilded Reef", "Harbor Echo", "Ivory Keep", "Jade Orbit", "Kestrel Bay", "Lumen Ridge",
];

fn midnight() -> i64 {
    words::now().div_euclid(DAY) * DAY
}

fn records(platform: &std::sync::Arc<Platform>) -> Library {
    let sqlite = Lent::open("libsqlite3.so.0").unwrap();
    Library::open(platform, &platform.file(FILE_GAMES).unwrap(), &sqlite).unwrap()
}

fn theme_object(scene: &mut Scene<'_>, name: &str) -> u64 {
    let tree = scene.tree();
    tree.nodes[tree.find_named(name).unwrap_or_else(|| panic!("{name} is placed")) as usize].id.parse().unwrap()
}

fn card_rows(scene: &mut Scene<'_>) -> Vec<(String, String)> {
    crate::common::console::card_words(scene)
}

fn hold_card(
    scene: &mut Scene<'_>,
    canvas: &str,
    screen: &str,
    table: &str,
    words: &[(&str, &str, bool)],
    help: &[&str],
    help_glyphs: &[&str],
    differences: &mut Vec<String>,
) {
    let tokens = tokens(theme_object(scene, screen));
    hold_board(scene, canvas, &tokens, table, words, help, help_glyphs, differences);
}

fn startable_board() -> Value {
    json!([
        { "console": "PlayStation", "name": "PlayStation", "engine": "Diamond", "firmware": "✓ Firmware found.", "system": "psx", "libretro": false },
        { "console": "PlayStation 2", "name": "PlayStation 2", "engine": "Sapphire", "firmware": "", "system": "ps2", "libretro": false }
    ])
}

/// Runs all 4 Eclipse Cards boards (#5, #44, #45, #46) and undrawn states on the active backend.
pub fn cards(backend: &str) {
    std::env::set_var("LC_ALL", "en_US.UTF-8");
    std::env::set_var("TZ", "UTC");
    let mut roms: Vec<(&str, String)> = vec![("SNES", "Aurora Drift.sfc".into())];
    for t in PSX_TITLES {
        roms.push(("psx", format!("{t}.iso")));
    }
    let rom_refs: Vec<(&str, &str)> = roms.iter().map(|(c, f)| (*c, f.as_str())).collect();
    let world = sandbox(&format!("qt-app-eclipse-cards-{backend}"), &rom_refs);
    let psx_dir = world.root.join("Roms/psx");
    let first_psx = psx_dir.join(format!("{}.iso", PSX_TITLES[0]));
    for n in 13..=212 {
        std::fs::hard_link(&first_psx, psx_dir.join(format!("Title {n:03}.iso"))).unwrap();
    }
    let cores = world.dir("Cores");
    crate::common::approved_console_core(&cores);
    let platform = Platform::load(library()).unwrap();
    let games = records(&platform);
    let snes_path = world.root.join("Roms/SNES/Aurora Drift.sfc").to_string_lossy().into_owned();
    let psx_path = |t: &str| world.root.join("Roms/psx").join(format!("{t}.iso")).to_string_lossy().into_owned();
    games.change(&json!({ "op": "started", "path": snes_path, "at": words::utc(words::now() - 3600) })).unwrap();
    games.change(&json!({ "op": "started", "path": psx_path("Brass Lantern"), "at": words::utc(words::now() - 7200) })).unwrap();
    games.change(&json!({ "op": "toggle_favourite", "path": psx_path("Amber Coast") })).unwrap();
    for (name, members) in [
        ("Couch nights", vec!["Amber Coast", "Brass Lantern"]),
        ("Finished", vec!["Cinder Mile"]),
        ("Rainy day", vec!["Dune Signal"]),
    ] {
        let id = games.change(&json!({ "op": "create_collection", "name": name, "at": "2026-11-13T10:00:00Z" })).unwrap();
        for m in members {
            games.change(&json!({ "op": "add_to_collection", "id": id, "path": psx_path(m) })).unwrap();
        }
    }
    drop(games);

    let mut harness = Harness::new();
    let media = world.dir("EsdeMedia");
    let mut pictures = vec![(media.join("snes/covers/Aurora Drift.png"), (560, 315), 0, String::new())];
    for (i, t) in PSX_TITLES.iter().enumerate() {
        pictures.push((media.join("psx/covers").join(format!("{t}.png")), (400, 400), i + 1, String::new()));
    }
    draw_covers(&mut harness, &world.root, &pictures, |_| {});
    let mut settings = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    settings["EsdeMediaDirectory"] = json!(media.to_str().unwrap());
    settings["OpenEmuFallback"] = json!(false);
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();

    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    assert!(scene.set_property("program", "coverStores", media.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("program", "theme", &std::fs::canonicalize(ECLIPSE).unwrap().display().to_string()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    assert_eq!(scene.property("theme", "failed"), "false", "{}", scene.property("theme", "warnings"));
    assert!(scene.set_property("clock", "now", &(midnight() + 21 * 3600 + 41 * 60).to_string()));
    let family = emusen_kit_qt::kinds::family_of(emusen_kit_core::glyph::PadFamily::Xbox).repr;
    assert!(scene.set_property("theme", "family", &family.to_string()));
    assert!(scene.set_property("theme", "battery", "82"));
    assert!(scene.set_property("homeCardCanvas", "battery", "82"));
    assert!(scene.set_property("listCardCanvas", "battery", "82"));
    scene.wait(400);

    let mut differences = Vec::new();

    // Undrawn state on #5: opening Home Card with no suspended game omits Continue and clears Quit hint.
    scene.pad(UiButton::Menu);
    wait_for(&mut scene, "screens", "shown", "HomeCard", 3000);
    scene.wait(350);
    assert_eq!(scene.property("homeCardModel", "menu"), "true");
    assert_eq!(scene.property("homeCardModel", "hasContinue"), "false");
    let idle_rows = card_rows(&mut scene);
    assert_eq!(idle_rows.last().unwrap(), &("Quit EmuSen".to_owned(), String::new()), "idle Quit hint empty: {idle_rows:?}");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 3000);
    scene.wait(300);

    // Board #5 Home Card (Menu) with suspended game and Continue card focused.
    let home_facts = json!({
        "suspended": "Aurora Drift",
        "suspendedConsole": "Super Nintendo",
        "suspendedPicture": "",
        "paused": "Super Nintendo · Paused just now",
        "totalGames": 1284,
        "inHand": "Aurora Drift",
        "inHandConsole": "Super Nintendo",
        "inHandCover": "",
        "inHandFavourite": true,
        "search": true,
        "startable": startable_board(),
        "gameMode": false,
        "tools": {},
        "unread": ""
    });
    assert!(scene.set_property("homeCardModel", "facts", &home_facts.to_string()));
    scene.pad(UiButton::Menu);
    wait_for(&mut scene, "screens", "shown", "HomeCard", 3000);
    scene.wait(400);
    assert_eq!(scene.property("homeCardModel", "hasContinue"), "true");
    assert_eq!(
        card_rows(&mut scene),
        vec![
            ("Continue".into(), "Aurora Drift".into()),
            ("Game page".into(), "Aurora Drift".into()),
            ("Search".into(), "1,284 games".into()),
            ("Library".into(), "".into()),
            ("Tools".into(), "Cores, Scraper, Packages".into()),
            ("Settings".into(), "".into()),
            ("Power".into(), "Sleep, restart, shut down".into()),
            ("Quit EmuSen".into(), "The game closes first".into()),
        ]
    );
    hold_card(
        &mut scene,
        "homeCardCanvas",
        "homeScreen",
        TABLE_HOME,
        &[
            ("cardTitle", "Menu", false),
            ("menuStatus", "82% · 9:41 PM", false),
            ("continueTag", "CONTINUE", true),
            ("continueTitle", "Aurora Drift", false),
            ("continueSubtitle", "Suspended · Super Nintendo", false),
        ],
        &["Choose", "Close"],
        &["A", "B"],
        &mut differences,
    );
    picture(&mut scene, &format!("card-home-1920x1200-{backend}.png"));

    // Undrawn state on #5: focusing a nav row switches Continue card to dark surface and highlights the nav row.
    scene.pad(UiButton::Down);
    scene.wait(200);
    let tree = scene.tree();
    assert_eq!(painted(&tree, "rowBg.0"), Colour::parse("#FF0F111A"), "unfocused Continue card dark fill");
    assert_eq!(painted(&tree, "rowBg.1"), Colour::parse("#FFF6E7D8"), "focused nav row cream fill");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 3000);
    scene.wait(300);

    // Board #46 Start Console opened from Home with Diamond (firmware found) and Sapphire, plus missing firmware rose check.
    assert!(scene.set_property("homeCardModel", "start", home_card::START_CONSOLE));
    scene.pad(UiButton::Menu);
    wait_for(&mut scene, "screens", "shown", "HomeCard", 3000);
    scene.wait(400);
    assert_eq!(scene.property("homeCardModel", "page"), home_card::START_CONSOLE);
    assert_eq!(
        card_rows(&mut scene),
        vec![("PlayStation".into(), "Diamond".into()), ("PlayStation 2".into(), "Sapphire".into())]
    );
    hold_card(
        &mut scene,
        "homeCardCanvas",
        "homeScreen",
        TABLE_START,
        &[
            ("cardBreadcrumb", "HOME", true),
            ("cardTitle", "Start Console", false),
            ("paneCoreName", "Diamond", false),
            ("paneLead.0", "Starts a PlayStation with no disc in it, on Diamond.", false),
            ("paneMuted.0", "Its BIOS screen comes up; Change Disc on the Pause Card puts a game in.", false),
            ("paneStatus", "✓ Firmware found.", false),
            ("cardBattery", "82%", false),
            ("cardClock", "9:41 PM", false),
            ("cardPosition", "1 of 2", false),
        ],
        &["Start", "Back", "Close"],
        &["A", "B", "MENU"],
        &mut differences,
    );
    picture(&mut scene, &format!("card-start-console-1920x1200-{backend}.png"));

    // Missing firmware on Start Console paints Eclipse rose #F3C9D2.
    let mut missing_fw = home_facts.clone();
    missing_fw["startable"] = json!([
        { "console": "PlayStation", "name": "PlayStation", "engine": "Diamond", "firmware": "✗ Not found: add it in Settings → Firmware", "system": "psx", "libretro": false }
    ]);
    assert!(scene.set_property("homeCardModel", "facts", &missing_fw.to_string()));
    scene.wait(150);
    assert_eq!(painted(&scene.tree(), "paneStatus"), Colour::parse("#FFF3C9D2"), "missing firmware paints rose");
    assert!(scene.set_property("homeCardModel", "start", ""));
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 3000);
    scene.wait(300);

    // Navigate Home -> Consoles -> PlayStation games view for boards #44 and #45.
    let home_id = theme_object(&mut scene, "homeScreen");
    assert!(emusen_theme_qt::host::actions(object(home_id)).expect("Actions").open_section(&QString::from("consoles")));
    wait_for(&mut scene, "screens", "shown", "Consoles", 3000);
    scene.wait(300);
    let cons_id = theme_object(&mut scene, "consolesScreen");
    assert!(emusen_theme_qt::host::actions(object(cons_id)).expect("Actions").open_console(&QString::from("psx")));
    wait_for(&mut scene, "screens", "shown", "Grid", 3000);
    wait_for(&mut scene, "gamesGrid", "count", "212", 5000);
    scene.wait(300);

    // Inject startable into listCardModel facts while keeping every live GameModel list field intact.
    let mut lf: Value = serde_json::from_str(&scene.property("gridGames", "listFacts")).expect("live listFacts");
    lf["startable"] = startable_board();
    assert!(scene.set_property("listCardModel", "facts", &lf.to_string()));
    scene.pad(UiButton::Options);
    wait_for(&mut scene, "screens", "shown", "List", 3000);
    scene.wait(400);
    // Focus row 1 (Sort), matching board #44.
    scene.pad(UiButton::Down);
    scene.wait(200);
    assert_eq!(
        card_rows(&mut scene),
        vec![
            ("Search".into(), "".into()),
            ("Sort".into(), "Name".into()),
            ("Filter".into(), "".into()),
            ("Jump to".into(), "A".into()),
            ("Random game".into(), "".into()),
            ("Start Console".into(), "".into()),
            ("Collections".into(), "".into()),
        ]
    );
    hold_card(
        &mut scene,
        "listCardCanvas",
        "gridScreen",
        TABLE_LIST,
        &[
            ("cardBreadcrumb", "LIST", true),
            ("cardTitle", "PlayStation · 212 games", false),
            ("paneLead.0", "Sorted by name, A to Z.", false),
            ("paneMuted.0", "Name · Last played · Most played · Year · Rating", false),
            ("cardBattery", "82%", false),
            ("cardClock", "9:41 PM", false),
            ("cardPosition", "2 of 7", false),
        ],
        &["Choose", "Back", "Close"],
        &["A", "B", "MENU"],
        &mut differences,
    );
    picture(&mut scene, &format!("card-list-1920x1200-{backend}.png"));

    // Board #45 List Card — Filter (retaining Not played per R-20261115-10, focusing Favourites = 2 of 6).
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    scene.wait(300);
    assert_eq!(scene.property("listCardModel", "page"), "filter");
    scene.pad(UiButton::Down);
    scene.wait(200);
    assert_eq!(
        card_rows(&mut scene),
        vec![
            ("Every game".into(), "".into()),
            ("Favourites".into(), "".into()),
            ("Not played".into(), "".into()),
            ("Couch nights".into(), "".into()),
            ("Finished".into(), "".into()),
            ("Rainy day".into(), "".into()),
        ]
    );
    hold_card(
        &mut scene,
        "listCardCanvas",
        "gridScreen",
        TABLE_FILTER,
        &[
            ("cardBreadcrumb", "LIST", true),
            ("cardTitle", "Filter", false),
            ("paneLead.0", "Shows the section's favourites.", false),
            ("paneMuted.0", "Choose Favourites to narrow it, or a collection to show it in the strip's place.", false),
            ("cardBattery", "82%", false),
            ("cardClock", "9:41 PM", false),
            ("cardPosition", "2 of 6", false),
        ],
        &["Choose", "Back", "Close"],
        &["A", "B", "MENU"],
        &mut differences,
    );
    picture(&mut scene, &format!("card-filter-1920x1200-{backend}.png"));

    // Verify B returns from Filter to List Card, and Start Console is also reachable from List Card.
    scene.pad(UiButton::Back);
    scene.wait(200);
    assert_eq!(scene.property("listCardModel", "page"), "list");
    for _ in 0..3 {
        scene.pad(UiButton::Down);
    }
    scene.pad(UiButton::Accept);
    scene.wait(250);
    assert_eq!(scene.property("listCardModel", "page"), home_card::START_CONSOLE);
    assert_eq!(read(&scene.tree(), "cardBreadcrumb", "text").to_uppercase(), "LIST");

    scene.assert_quiet();
    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
}
