//! Eclipse boards #8 (Search), #11 (Collections), and #9 (Tools hub) held to their boards at every gate size.

use crate::common::{approved_console_core, sandbox, wait_for};
use crate::eclipse_board::{glyphs_in, held, kit_tokens, picture, read, rows, words_under};
use emusen_client_fixture::library;
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::FILE_GAMES;
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt::MAIN_URL;
use emusen_qt_models::themes;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::{Harness, Scene};
use serde_json::json;

const TABLE_SEARCH: &str = include_str!("../../../qt-app/tests/eclipse/search.tsv");
const TABLE_COLLECTIONS: &str = include_str!("../../../qt-app/tests/eclipse/collections.tsv");
const TABLE_TOOLS: &str = include_str!("../../../qt-app/tests/eclipse/tools.tsv");

fn midnight() -> i64 {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    now - now.rem_euclid(86_400)
}

fn records() -> Library {
    let platform = Platform::load(library()).unwrap();
    let sqlite = Lent::open("libsqlite3.so.0").unwrap();
    Library::open(&platform, &platform.file(FILE_GAMES).unwrap(), &sqlite).unwrap()
}

fn check_board(
    scene: &mut Scene<'_>,
    canvas: &str,
    unit_item: &str,
    table: &str,
    words: &[(&str, &str, bool)],
    help_bar: &str,
    help: &[&str],
    help_glyphs: &[&str],
    differences: &mut Vec<String>,
) {
    let (board_h, all_rows) = rows(table);
    let k = DESIGN_HEIGHT / board_h;
    let tokens = kit_tokens(scene);
    for (width, height) in SIZES {
        scene.resize(width, height);
        scene.wait(300);
        let tree = scene.tree();
        let unit: f64 = read(&tree, unit_item, "unit").parse().expect("canvas unit");
        let cw = width as f64 / unit;
        let at = format!("{canvas} {width}x{height}");
        let (present, missing): (Vec<_>, Vec<_>) = all_rows.iter().cloned().partition(|r| r.status != "built" || tree.find_named(&r.name).is_some());
        differences.extend(missing.iter().map(|r| format!("{at}: {} is not drawn", r.name)));
        held(&tree, &present, k, unit, cw, &tokens, &at, &|_, _, _| false, differences);
        for (name, want, upper) in words {
            let have = read(&tree, name, "text");
            let drawn = if *upper { have.to_uppercase() } else { have.clone() };
            if drawn != *want {
                differences.push(format!("{at}: {name} says {drawn:?} where board expects {want:?}"));
            }
        }
        let line = words_under(&tree, help_bar);
        if line != help {
            differences.push(format!("{at}: {help_bar} says {line:?} where board expects {help:?}"));
        }
        let letters = glyphs_in(&tree, help_bar);
        if letters != help_glyphs {
            differences.push(format!("{at}: {help_bar} glyphs read {letters:?} where board draws {help_glyphs:?}"));
        }
    }
    scene.resize(1280, 800);
    scene.wait(250);
}

fn wait_node(scene: &mut Scene<'_>, name: &str, property: &str, value: &str, ms: i32) {
    let mut waited = 0;
    let mut got = String::new();
    while waited < ms {
        let tree = scene.tree();
        got = read(&tree, name, property);
        if got == value {
            return;
        }
        scene.wait(50);
        waited += 50;
    }
    let tree = scene.tree();
    assert_eq!(read(&tree, name, property), value, "{name}.{property} after {ms} ms (last {got:?})");
}

fn type_word(scene: &mut Scene<'_>, word: &str) {
    for ch in word.chars() {
        let code = ch.to_ascii_uppercase() as i32;
        let mods = if ch.is_ascii_uppercase() { 0x0200_0000 } else { 0 };
        scene.key(code, mods);
    }
}

fn click_named(scene: &mut Scene<'_>, name: &str) {
    let tree = scene.tree();
    let idx = tree.nodes.iter().rposition(|n| n.name == name && n.visible).or_else(|| tree.find_named(name).map(|i| i as usize)).unwrap_or_else(|| panic!("{name} is drawn"));
    let n = &tree.nodes[idx];
    scene.click(n.x + n.w / 2.0, n.y + n.h / 2.0);
}

pub fn sys2(backend: &str) {
    std::env::set_var("TZ", "UTC");
    let world = sandbox(
        &format!("qt-app-eclipse-sys2-{backend}"),
        &[
            ("SNES", "Aurora Drift.sfc"),
            ("GB", "Laurel Crown.gb"),
            ("psx", "Minotaur Deep.iso"),
            ("NES", "Copper Echo.nes"),
        ],
    );
    let cores = world.dir("Cores");
    approved_console_core(&cores);
    let rom = |sub: &str, file: &str| world.root.join("Roms").join(sub).join(file).to_str().unwrap().to_owned();
    let aurora = rom("SNES", "Aurora Drift.sfc");
    let laurel = rom("GB", "Laurel Crown.gb");
    let minotaur = rom("psx", "Minotaur Deep.iso");
    let copper = rom("NES", "Copper Echo.nes");

    let db = records();
    for (p, year, genre) in [
        (&aurora, "1994-03-18", "Platformers"),
        (&laurel, "1992-07-01", "Role-playing"),
        (&minotaur, "1998-11-12", "Racing"),
        (&copper, "1989-05-01", "Platformers"),
    ] {
        let changes = json!({ "releasedate": year, "genre": genre });
        db.change(&json!({ "op": "save_edits", "path": p, "changes": changes, "at": "2026-11-13T10:00:00Z" })).unwrap();
    }
    db.change(&json!({ "op": "started", "path": aurora, "at": "2026-11-13T10:00:00Z" })).unwrap();
    db.change(&json!({ "op": "toggle_favourite", "path": aurora })).unwrap();
    let c1 = db.change(&json!({ "op": "create_collection", "name": "Couch co-op", "at": "2026-11-13T10:00:00Z" })).unwrap().as_i64().unwrap();
    db.change(&json!({ "op": "add_to_collection", "id": c1, "path": aurora })).unwrap();
    db.change(&json!({ "op": "add_to_collection", "id": c1, "path": copper })).unwrap();
    let c2 = db.change(&json!({ "op": "create_collection", "name": "Finished", "at": "2026-11-13T10:00:00Z" })).unwrap().as_i64().unwrap();
    db.change(&json!({ "op": "add_to_collection", "id": c2, "path": laurel })).unwrap();
    drop(db);

    let power = world.power_supply().join("BAT0");
    std::fs::create_dir_all(&power).unwrap();
    for (k, v) in [("type", "Battery"), ("capacity", "82"), ("status", "Discharging")] {
        std::fs::write(power.join(k), format!("{v}\n")).unwrap();
    }

    let mut harness = Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("clock", "now", &(midnight() + 21 * 3600 + 41 * 60).to_string()));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("program", "theme", &themes::built_in_folder("eclipse")));
    let tools_json = json!({
        "cores": { "installed": 12, "updates": 2 },
        "packages": { "sentence": "3 patches ready to apply. 14 discs could be made smaller." },
        "library": { "sentence": "Last check, 2 November: 4 files to look at.", "attention": true, "issues": 4 },
    });
    assert!(scene.set_property("library", "toolSummaries", &tools_json.to_string()));
    wait_for(&mut scene, "library", "loading", "false", 15000);
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    wait_for(&mut scene, "theme", "home", "true", 5000);
    scene.wait(300);

    let mut differences = Vec::new();

    // Board #8: Search.
    click_named(&mut scene, "search");
    wait_for(&mut scene, "screens", "shown", "Search", 3000);
    scene.wait(300);
    for ch in [0x41, 0x55, 0x52] {
        scene.key(ch, 0);
    }
    wait_for(&mut scene, "searchQueryText", "text", "aur", 3000);
    wait_for(&mut scene, "searchEclipseModel", "resultCount", "3", 3000);
    // Place key cursor on 'o' (row 0 col 8 = index 8).
    for _ in 0..8 {
        scene.pad(UiButton::Right);
    }
    wait_for(&mut scene, "typist", "current", "8", 2000);

    check_board(
        &mut scene,
        "searchCanvas",
        "searchCanvas",
        TABLE_SEARCH,
        &[
            ("searchQueryText", "aur", false),
            ("searchCount", "3 of 4 games", false),
            ("searchHeading", "Games", false),
            ("searchResultTitle.0", "Aurora Drift", false),
            ("searchResultMeta.0", "Super Nintendo · 1994", false),
            ("searchResultTitle.1", "Laurel Crown", false),
            ("searchResultMeta.1", "Game Boy · 1992", false),
            ("searchResultTitle.2", "Minotaur Deep", false),
            ("searchResultMeta.2", "PlayStation · 1998", false),
            ("searchNote", "No console or collection has \"aur\" in its name.", false),
        ],
        "searchHelp",
        &["Type", "Erase, then close when empty", "Space", "To the results"],
        &["A", "B", "Y", "RT"],
        &mut differences,
    );
    if backend == "card" {
        picture(&mut scene, "search.png");
    }

    // Search interactions: RT -> SearchGrid -> arrow -> open GamePage -> back -> RT back -> 123 toggle -> Space -> B erase/close.
    scene.pad(UiButton::Last);
    wait_for(&mut scene, "screens", "shown", "SearchGrid", 2000);
    scene.pad(UiButton::Right);
    wait_for(&mut scene, "searchEclipseModel", "focusedResult", "1", 2000);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Page", 3000);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "SearchGrid", 3000);
    scene.pad(UiButton::Last);
    wait_for(&mut scene, "screens", "shown", "Search", 2000);
    click_named(&mut scene, "searchKey.30");
    wait_node(&mut scene, "searchKeyText.30", "text", "ABC", 2000);
    click_named(&mut scene, "searchKey.30");
    wait_node(&mut scene, "searchKeyText.30", "text", "123", 2000);
    scene.pad(UiButton::Search);
    wait_for(&mut scene, "searchQueryText", "text", "aur ", 2000);
    for _ in 0..5 {
        scene.pad(UiButton::Back);
        scene.wait(80);
    }
    wait_for(&mut scene, "screens", "shown", "Home", 3000);

    // Board #11: Collections via Consoles -> RB.
    click_named(&mut scene, "nav.consoles");
    wait_for(&mut scene, "screens", "shown", "Consoles", 3000);
    scene.wait(300);
    scene.pad(UiButton::PageDown);
    wait_for(&mut scene, "screens", "shown", "Collections", 3000);
    wait_node(&mut scene, "collectionCardTitle.6", "text", "Role-playing", 5000);
    scene.wait(200);

    check_board(
        &mut scene,
        "collectionsCanvas",
        "collectionsCanvas",
        TABLE_COLLECTIONS,
        &[
            ("wordmark", "EMUSEN", true),
            ("navWords.collections", "Collections", false),
            ("battery", "82%", false),
            ("clock", "9:41 PM", false),
            ("collectionCardTitle.0", "Favourites", false),
            ("collectionCardMeta.0", "1 game · yours", false),
            ("collectionCardTitle.1", "Couch co-op", false),
            ("collectionCardMeta.1", "2 games · yours", false),
            ("collectionCardTitle.2", "Finished", false),
            ("collectionCardMeta.2", "1 game · yours", false),
            ("collectionCardTitle.3", "Recently played", false),
            ("collectionCardMeta.3", "1 game · kept by EmuSen", false),
            ("collectionCardTitle.4", "Platformers", false),
            ("collectionCardMeta.4", "2 games · kept by EmuSen", false),
            ("newCollectionLabel", "New collection", false),
            ("collectionsFooterSummary", "1 of 7 · 3 made by you, 4 kept by EmuSen", false),
        ],
        "collectionsHelp",
        &["Open", "Rename or remove", "Back"],
        &["A", "X", "B"],
        &mut differences,
    );
    if backend == "card" {
        picture(&mut scene, "collections.png");
    }

    // Create, rename, and remove collection in games.db + open genre/user collections into Games grid.
    click_named(&mut scene, "newCollectionCard");
    wait_for(&mut scene, "collectionsEclipseModel", "typingOpen", "true", 2000);
    type_word(&mut scene, "shmups");
    wait_node(&mut scene, "collectionTyped", "text", "shmups", 2000);
    click_named(&mut scene, "collectionKey.33");
    wait_for(&mut scene, "collectionsEclipseModel", "typingOpen", "false", 3000);
    wait_node(&mut scene, "collectionCardTitle.3", "text", "shmups", 5000);

    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "collectionsEclipseModel", "dialogOpen", "true", 2000);
    wait_for(&mut scene, "collectionsDialog", "opened", "true", 2000);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "collectionsEclipseModel", "typingOpen", "true", 2000);
    wait_for(&mut scene, "collectionsCanvas", "sheetOpen", "false", 2000);
    for _ in 0..6 {
        scene.key(0x0100_0003, 0);
    }
    type_word(&mut scene, "arcade");
    wait_node(&mut scene, "collectionTyped", "text", "arcade", 2000);
    click_named(&mut scene, "collectionKey.33");
    wait_for(&mut scene, "collectionsEclipseModel", "typingOpen", "false", 3000);
    wait_node(&mut scene, "collectionCardTitle.1", "text", "arcade", 5000);

    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "collectionsEclipseModel", "dialogOpen", "true", 2000);
    wait_for(&mut scene, "collectionsDialog", "opened", "true", 2000);
    scene.pad(UiButton::Down);
    scene.wait(50);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "collectionsEclipseModel", "dialogTitle", "Remove \"arcade\"?", 2000);
    wait_for(&mut scene, "collectionsDialog", "opened", "true", 2000);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "collectionsCanvas", "sheetOpen", "false", 2000);
    wait_node(&mut scene, "collectionCardTitle.3", "text", "Recently played", 5000);

    click_named(&mut scene, "collectionCard.4");
    wait_for(&mut scene, "screens", "shown", "Grid", 3000);
    wait_for(&mut scene, "gridGames", "count", "2", 3000);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Collections", 3000);

    // Board #9: Tools hub via RB from Collections.
    scene.pad(UiButton::PageDown);
    wait_for(&mut scene, "screens", "shown", "Tools", 3000);
    scene.wait(300);

    check_board(
        &mut scene,
        "toolsCanvas",
        "toolsCanvas",
        TABLE_TOOLS,
        &[
            ("wordmark", "EMUSEN", true),
            ("navWords.tools", "Tools", false),
            ("battery", "82%", false),
            ("clock", "9:41 PM", false),
            ("toolsHeading", "Tools", false),
            ("toolsSubtitle", "Each one says what it would do before it does it.", false),
            ("toolCardTitle.0", "Cores", false),
            ("toolBadge.0", "2 updates", false),
            ("toolCardTitle.1", "Scraper", false),
            ("toolCardLine.1", "4 of 4 games have art and facts.", false),
            ("toolCardTitle.2", "Packages", false),
            ("toolCardTitle.3", "Library Manager", false),
            ("toolBadge.3", "4 to check", false),
            ("toolCardTitle.4", "Themes", false),
            ("toolCardTitle.5", "Memory cards", false),
        ],
        "toolsHelp",
        &["Open", "Back", "Sections"],
        &["A", "B", "LB"],
        &mut differences,
    );
    if backend == "card" {
        picture(&mut scene, "tools.png");
    }

    // All 6 Tools routes open their target screens and B comes back to Tools.
    click_named(&mut scene, "toolCard.0");
    wait_for(&mut scene, "screens", "shown", "Cores", 3000);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Tools", 3000);

    for (idx, expected_tool) in [(1, "scraper"), (2, "packages"), (3, "library")] {
        click_named(&mut scene, &format!("toolCard.{idx}"));
        wait_for(&mut scene, "screens", "shown", "Tool", 3000);
        assert_eq!(scene.property("screens", "tool"), expected_tool);
        scene.pad(UiButton::Back);
        wait_for(&mut scene, "screens", "shown", "Tools", 3000);
    }

    click_named(&mut scene, "toolCard.4");
    wait_for(&mut scene, "screens", "shown", "Settings", 3000);
    wait_for(&mut scene, "settingsHeading", "text", "Themes", 3000);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Tools", 3000);

    click_named(&mut scene, "toolCard.5");
    wait_for(&mut scene, "screens", "shown", "Settings", 3000);
    scene.wait(200);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Tools", 3000);

    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
}
