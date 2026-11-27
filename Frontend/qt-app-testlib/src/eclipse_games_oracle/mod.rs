//! Eclipse's games view in the running program (EmuSen_Qt.md §3.4.10), driven by an SDL pad inside this process over a library of 120 games, three with covers, the first by name played and thirty favourites: every row of eclipse/games.tsv held at each of the gate's sizes, the fields and the disc in the focused game's colours; then over one game (a collection of one), the thirty favourites and all 120, at each size, every game reached by the pad, the pointer and the wheel, none cut and none under the help line; and the chips, the sort, the path, the letters and the buttons, each pressed by the pad and by the pointer. One test, as the harness is one per process.

use crate::common::{draw_covers, sandbox, wait_for};
use crate::eclipse_pad::{focused, press};
use emusen_client_fixture::library;
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::input::PadControl;
use emusen_kit_core::tokens::Colour;
use emusen_platform_client::galaxia::consts::{FILE_GAMES, MODEL_APP_SETTINGS, SAVE_RESUME_STATE, SAVE_STATE};
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt::MAIN_URL;
use emusen_qt_models::themes;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::tree::Tree;
use emusen_qt_test::virtual_pad::{Sdl, VirtualPad};
use emusen_qt_test::Scene;
use serde_json::json;

/// The board's elements, one a row (eclipse/games.tsv), held by eclipse_board's rules.
const BOARD: &str = include_str!("../../../qt-app/tests/eclipse/games.tsv");
/// The covers drawn, each a shape of its own: tall, wide and square, at sizes that are constants of this test.
const COVERS: [(&str, (u32, u32)); 3] = [("Aurora Drift", (300, 420)), ("Amber Relay", (420, 300)), ("Basalt Kings", (360, 360))];
/// The first by name, played, so the column shows its last play as the board does; with every fact the board draws.
const PLAYED: &str = "Amber Relay";
/// The second by name, with no fact at all.
const BARE: &str = "Aurora Drift";
/// Its facts, as edits in games.db, which the scraper's metadata reads first: the board's words for a game invented for the tests.
const DESCRIPTION: &str = "Amber Relay is a game invented for EmuSen's tests. It has no story and no pictures of its own; every image shown for it is a flat colour with its name written on it.";
/// The library's size, the most this fixture shelves, and how many of its games are favourites.
const GAMES: usize = 120;
const FAVOURITES: usize = 30;
/// The collection of one game, the fewest a list holds.
const COLLECTION: &str = "One";

/// The games' titles: the covers' three, then names under every letter but A, a long one among them that wraps on its tile.
fn titles() -> Vec<String> {
    const WORDS: [&str; 12] = ["Brass", "Cinder", "Dune", "Ember", "Fathom", "Granite", "Harbor", "Indigo", "Juniper", "Kestrel", "Lumen", "Marble"];
    let mut out: Vec<String> = COVERS.iter().map(|(t, _)| t.to_string()).collect();
    out.push("Nettle and Thorn of the Long Midsummer Lantern".into());
    let mut n = 0;
    while out.len() < GAMES {
        out.push(format!("{} Run {:03}", WORDS[n % WORDS.len()], n));
        n += 1;
    }
    out
}

/// games.db where Galaxia names it, as the program opens it.
fn records() -> Library {
    let platform = Platform::load(library()).unwrap();
    let sqlite = Lent::open("libsqlite3.so.0").unwrap();
    Library::open(&platform, &platform.file(FILE_GAMES).unwrap(), &sqlite).unwrap()
}

/// The visible item of the tree by name; of a Line (parts/Line.qml), whose words carry its name too, the words.
fn shown<'a>(tree: &'a Tree, name: &str) -> Option<&'a emusen_qt_test::tree::Node> {
    let named = || tree.nodes.iter().filter(|n| n.name == name && n.visible);
    named().find(|n| n.text.is_some()).or_else(|| named().next())
}

fn words(scene: &mut Scene<'_>, name: &str) -> String {
    let tree = scene.tree();
    shown(&tree, name).and_then(|n| n.text.clone()).unwrap_or_default()
}

/// Waits until the shown element of a name says `want`, as a delegate's words are found by the tree alone.
fn says(scene: &mut Scene<'_>, name: &str, want: &str) {
    for _ in 0..100 {
        if words(scene, name) == want {
            return;
        }
        scene.wait(50);
    }
    assert_eq!(words(scene, name), want, "{name} after 5 s");
}

fn middle(scene: &mut Scene<'_>, name: &str) -> (f64, f64) {
    let tree = scene.tree();
    let n = shown(&tree, name).unwrap_or_else(|| panic!("{name} shown"));
    (n.x + n.w / 2.0, n.y + n.h / 2.0)
}

/// A property of the shown element of a name, where a hidden one shares it.
fn read_shown(tree: &Tree, name: &str, property: &str) -> String {
    let Some(n) = shown(tree, name) else { return String::new() };
    emusen_qt_test::ffi::ffi::harness_property(crate::eclipse_board::object(n.id.parse().unwrap()), &cxx_qt_lib::QString::from(name), &cxx_qt_lib::QString::from(property)).to_string()
}

/// Every row of the board's table held to the screen at each of the gate's sizes with the played game focused; the fields in the board's saturation and lightness with the disc's hue; the line heights; five columns at the board's shape.
fn board_held(scene: &mut Scene<'_>) {
    let (board_height, rows) = crate::eclipse_board::rows(BOARD);
    let k = DESIGN_HEIGHT / board_height;
    let mut differences = Vec::new();
    for (width, height) in SIZES {
        crate::eclipse_board::resize_settle(scene, width, height);
        let tree = scene.tree();
        let at = format!("{width}x{height}");
        let unit: f64 = crate::eclipse_board::read(&tree, "gridCanvas", "unit").parse().expect("the canvas's unit");
        let screen: u64 = tree.nodes[tree.find_named("gridScreen").expect("Eclipse's games view") as usize].id.parse().unwrap();
        let tokens = crate::eclipse_board::tokens(screen);
        crate::eclipse_board::held(&tree, &rows, k, unit, f64::from(width) / unit, &tokens, &at, &|_, _, _| false, &mut differences);
        let scale = crate::eclipse_board::read(&tree, "gamesDisc", "pixelScale").parse::<f64>().unwrap_or(f64::NAN);
        if (scale - unit).abs() > 1e-9 {
            differences.push(format!("{at}: the disc's glows blurred at {scale} pixels a unit where the scene draws {unit}"));
        }
        // The rim takes the art's first hue; the first field the same hue, both fields the board's saturation and lightness. The games' disc's rim, not the hidden empty board's.
        let rim = Colour::parse(&read_shown(&tree, "coronaRim", "color")).map(|c| c.to_hsl());
        let token = tokens.get("coronaColor").and_then(|v| Colour::parse(v)).map(|c| c.to_hsl());
        let Some(rim) = rim.filter(|r| Some(*r) != token) else {
            differences.push(format!("{at}: the disc's rim is the look's, not the game's colour"));
            continue;
        };
        for row in rows.iter().filter(|r| r.kind.starts_with("field ")) {
            let board = Colour::parse(row.kind.split(' ').nth(3).unwrap()).expect("a field's board colour").to_hsl();
            let have = Colour::parse(&crate::eclipse_board::read(&tree, &row.name, "paint")).map(|p| p.to_hsl());
            let hue_ok = row.name != "backdropField.0" || have.is_some_and(|h| emusen_kit_core::world::wrap_hue(h.0 - rim.0).abs() <= 2.0);
            if !(hue_ok && have.is_some_and(|h| (h.1 - board.1).abs() <= 0.01 && (h.2 - board.2).abs() <= 0.01)) {
                differences.push(format!("{at}: {} painted {have:?} where the board's colour with the rim's hue {:.1} gives {board:?}", row.name, rim.0));
            }
        }
        for (name, size, height) in [("gamesTitle", 62.0, 1.04), ("tileTitle.0", 18.0, 1.2)] {
            let mode = crate::eclipse_board::read(&tree, name, "lineHeightMode");
            let have = crate::eclipse_board::read(&tree, name, "lineHeight").parse::<f64>().unwrap_or(f64::NAN);
            if mode != "FixedHeight" || (have - size * height).abs() > 1e-6 {
                differences.push(format!("{at}: {name} line height {have} (mode {mode}) where the board gives {} fixed", size * height));
            }
        }
        if f64::from(width) / f64::from(height) == 1.6 {
            let y = |name: &str| shown(&tree, name).map(|n| n.y);
            if y("gamesTile.4") != y("gamesTile.0") || y("gamesTile.5") == y("gamesTile.0") {
                differences.push(format!("{at}: not five columns at the board's shape"));
            }
        }
    }
    assert!(differences.is_empty(), "Eclipse's games view against its board:\n{}", differences.join("\n"));
}

/// The pictures for the tester, when a folder is named.
fn pictures(scene: &mut Scene<'_>, backend: &str, game: &str) {
    let Some(folder) = std::env::var_os("EMUSEN_ECLIPSE_PICTURES") else { return };
    std::fs::create_dir_all(&folder).unwrap();
    for (width, height) in [(1920, 1200), (1280, 800)] {
        scene.resize(width, height);
        scene.wait(500);
        let path = std::path::Path::new(&folder).join(format!("games-{game}-{width}x{height}-{backend}.png"));
        assert!(scene.grab(&path.display().to_string()), "{} is grabbed", path.display());
    }
}

/// The game with no facts focused: none of its rows or chips is made, and what follows moves up as the board's column flows.
fn bare_flows(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad) {
    press(scene, sdl, pad, PadControl::Right);
    wait_for(scene, "gamesTitle", "text", BARE, 2000);
    let mut tree = scene.tree();
    for _ in 0..100 {
        if shown(&tree, "gamesChips").is_none() && shown(&tree, "description").is_none() && shown(&tree, "gamesFacts").is_none() {
            break;
        }
        scene.wait(50);
        tree = scene.tree();
    }
    for name in ["gamesChips", "genreChip", "playersChip", "description", "gamesFacts", "gamesFact.0", "gamesFact.1", "gamesFact.2"] {
        assert!(shown(&tree, name).is_none(), "{name} is drawn for a game with none of it");
    }
    assert!(!words(scene, "gamesConsole").contains('·'), "no year for a game with none: {}", words(scene, "gamesConsole"));
    let unit: f64 = crate::eclipse_board::read(&tree, "gridCanvas", "unit").parse().unwrap();
    let px = DESIGN_HEIGHT / 1200.0 * unit;
    let title = shown(&tree, "gamesTitleLine").unwrap();
    let start = shown(&tree, "gamesStart").unwrap();
    let want = title.y + title.h + 28.0 * px;
    assert!((start.y - want).abs() < 0.5, "Start {} under the title where the column flows to {want}", start.y);
}

/// The focused tile's index.
fn at_tile(scene: &mut Scene<'_>) -> Option<usize> {
    focused(scene).strip_prefix("gamesTile.").and_then(|n| n.parse().ok())
}

/// Tile `i` drawn whole inside the grid's view, above the help line, its title not cut.
fn whole(tree: &Tree, i: usize) -> Result<(), String> {
    let tile = shown(tree, &format!("gamesTile.{i}")).ok_or(format!("gamesTile.{i} not drawn"))?;
    let view = shown(tree, "gamesGrid").ok_or("no grid")?;
    let help = shown(tree, "help").ok_or("no help line")?;
    let inside = tile.x >= view.x - 1e-6 && tile.y >= view.y - 1e-6 && tile.x + tile.w <= view.x + view.w + 1e-6 && tile.y + tile.h <= view.y + view.h + 1e-6;
    if !inside || tile.y + tile.h > help.y {
        return Err(format!("gamesTile.{i} at {:?} outside the view {:?} or under the help line at {}", (tile.x, tile.y, tile.w, tile.h), (view.x, view.y, view.w, view.h), help.y));
    }
    let title = format!("tileTitle.{i}");
    if crate::eclipse_board::read(tree, &title, "truncated") != "false" {
        return Err(format!("{title} is cut"));
    }
    Ok(())
}

// Checks wheel scroll + pointer hover over one row of tiles inside `gamesGrid`.
fn hover_row(scene: &mut Scene<'_>, size: &str, count: usize, start: usize, end: usize) {
    for i in start..end {
        let (cx, cy) = middle(scene, &format!("gamesTile.{i}"));
        scene.hover(cx, cy);
        let mut waited = 0;
        while at_tile(scene) != Some(i) && waited < 250 {
            scene.wait(8);
            waited += 8;
        }
        assert_eq!(at_tile(scene), Some(i), "{size}: the pointer on game {i} of {count}");
        if let Err(why) = whole(&scene.tree(), i) {
            panic!("{size}: by the wheel, {why}");
        }
    }
}

// Reaches every game at `1920x1200` via pad Right across all tiles and wheel + pointer across all rows.
fn reached_all_1920(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, count: usize) {
    let (width, height) = (1920, 1200);
    crate::eclipse_board::resize_settle(scene, width, height);
    let size = format!("{width}x{height}");
    let (x, y) = middle(scene, "gamesGrid");
    scene.wheel(x, y, 200);
    scene.wait(150);
    let tree = scene.tree();
    assert!(scene.focus_item(&tree, tree.find_named("gamesTile.0").expect("the first tile")));
    scene.wait(50);
    for i in 0..count {
        assert_eq!(at_tile(scene), Some(i), "{size}: the pad's walk over {count}");
        if let Err(why) = whole(&scene.tree(), i) {
            panic!("{size}: {why}");
        }
        if i + 1 < count {
            press(scene, sdl, pad, PadControl::Right);
        }
    }
    let columns: usize = crate::eclipse_board::read(&scene.tree(), "grid", "columns").parse().expect("the grid's columns");
    let (x, y) = middle(scene, "gamesGrid");
    scene.wheel(x, y, 200);
    scene.wait(250);
    assert!(whole(&scene.tree(), 0).is_ok(), "{size}: the wheel back to the first row");
    for row in 0..count.div_ceil(columns) {
        if row > 0 {
            scene.wheel(x, y, -1);
            scene.wait(80);
        }
        scene.hover(1.0, 1.0);
        scene.wait(16);
        hover_row(scene, &size, count, row * columns, ((row + 1) * columns).min(count));
    }
}

// Reaches first two rows and last row via pad and wheel + pointer at each non-1920x1200 gate size.
fn reached_boundary_other(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, count: usize) {
    for &(width, height) in SIZES.iter().filter(|&&(w, h)| (w, h) != (1920, 1200)) {
        crate::eclipse_board::resize_settle(scene, width, height);
        let size = format!("{width}x{height}");
        let (x, y) = middle(scene, "gamesGrid");
        scene.wheel(x, y, 200);
        scene.wait(150);
        let tree = scene.tree();
        assert!(scene.focus_item(&tree, tree.find_named("gamesTile.0").expect("the first tile")));
        scene.wait(50);
        let columns: usize = crate::eclipse_board::read(&scene.tree(), "grid", "columns").parse().expect("the grid's columns");
        let limit = (2 * columns).min(count);
        for i in 0..limit {
            assert_eq!(at_tile(scene), Some(i), "{size}: the pad's walk over {count}");
            if let Err(why) = whole(&scene.tree(), i) {
                panic!("{size}: {why}");
            }
            if i + 1 < limit {
                press(scene, sdl, pad, PadControl::Right);
            }
        }
        if count > 2 * columns {
            for _ in 0..columns - 1 {
                press(scene, sdl, pad, PadControl::Left);
            }
            assert_eq!(at_tile(scene), Some(columns), "{size}: aligned on column 0 of row 1");
            let last_row = (count - 1) / columns;
            for _ in 1..last_row {
                press(scene, sdl, pad, PadControl::Down);
            }
            for i in last_row * columns..count {
                assert_eq!(at_tile(scene), Some(i), "{size}: the pad's last-row walk over {count}");
                if let Err(why) = whole(&scene.tree(), i) {
                    panic!("{size}: {why}");
                }
                if i + 1 < count {
                    press(scene, sdl, pad, PadControl::Right);
                }
            }
        }
        let total_rows = count.div_ceil(columns);
        let (x, y) = middle(scene, "gamesGrid");
        scene.wheel(x, y, 200);
        scene.wait(150);
        assert!(whole(&scene.tree(), 0).is_ok(), "{size}: the wheel back to the first row");
        scene.hover(1.0, 1.0);
        scene.wait(16);
        hover_row(scene, &size, count, 0, columns.min(count));
        if total_rows > 1 {
            scene.wheel(x, y, -1);
            scene.wait(80);
            scene.hover(1.0, 1.0);
            scene.wait(16);
            hover_row(scene, &size, count, columns, (2 * columns).min(count));
        }
        if total_rows > 2 {
            scene.wheel(x, y, -200);
            scene.wait(150);
            scene.hover(1.0, 1.0);
            scene.wait(16);
            hover_row(scene, &size, count, (total_rows - 1) * columns, count);
        }
    }
}

/// A list narrowed as a theme narrows it, through the Actions the program gives every theme.
fn set_filter(scene: &mut Scene<'_>, key: &str, count: usize) {
    let tree = scene.tree();
    let screen: u64 = tree.nodes[tree.find_named("gridScreen").expect("Eclipse's games view") as usize].id.parse().unwrap();
    let actions = emusen_theme_qt::host::actions(crate::eclipse_board::object(screen)).expect("the theme's Actions");
    assert!(actions.set_filter(&cxx_qt_lib::QString::from(key)), "{key} taken");
    wait_for(scene, "gamesGrid", "count", &count.to_string(), 3000);
    scene.wait(200);
}

// Sets up the 120-game Eclipse library fixture, navigates to the Eclipse games grid, and passes scene + pad.
fn with_games_view(world_name: &str, f: impl FnOnce(&mut Scene<'_>, &Sdl, &VirtualPad)) {
    let titles = titles();
    let files: Vec<String> = titles.iter().map(|t| format!("{t}.sfc")).collect();
    let world = sandbox(world_name, &files.iter().map(|f| ("SNES", f.as_str())).collect::<Vec<_>>());
    let path = |title: &str| world.root.join("Roms/SNES").join(format!("{title}.sfc")).to_str().unwrap().to_owned();
    let games = records();
    games.change(&json!({ "op": "started", "path": path(PLAYED), "at": "2026-11-13T10:00:00Z" })).unwrap();
    let changes = json!({ "releasedate": "1994-03-18", "genre": "Action", "players": "1-2", "description": DESCRIPTION });
    games.change(&json!({ "op": "save_edits", "path": path(PLAYED), "changes": changes, "at": "2026-11-13T10:00:00Z" })).unwrap();
    // 3 h 20 min played, the board's.
    games.change(&json!({ "op": "played", "path": path(PLAYED), "seconds": 12_000.0, "max_hours": 24 })).unwrap();
    for title in titles.iter().skip(titles.len() - FAVOURITES) {
        games.change(&json!({ "op": "toggle_favourite", "path": path(title) })).unwrap();
    }
    let id = games.change(&json!({ "op": "create_collection", "name": COLLECTION, "at": "2026-11-13T10:00:00Z" })).unwrap();
    games.change(&json!({ "op": "add_to_collection", "id": id, "path": path(&titles[5]) })).unwrap();
    drop(games);
    let platform = Platform::load(library()).unwrap();
    let media = world.dir("Media");
    // Three states, the board's: two slots and the automatic one.
    for (kind, slot) in [(SAVE_STATE, 1), (SAVE_STATE, 2), (SAVE_RESUME_STATE, 0)] {
        let file = platform.save_path(kind, &path(PLAYED), None, slot).unwrap();
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, b"state").unwrap();
    }
    // Machine battery at the board's 82%, discharging, in the world's synthetic power_supply tree.
    let power = world.power_supply().join("BAT0");
    std::fs::create_dir_all(&power).unwrap();
    for (key, value) in [("type", "Battery"), ("capacity", "82"), ("status", "Discharging")] {
        std::fs::write(power.join(key), format!("{value}\n")).unwrap();
    }

    let sdl = Sdl::open();
    let pad = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);
    let mut harness = emusen_qt_test::Harness::new();
    let covers: Vec<_> = COVERS.iter().enumerate().map(|(i, (t, size))| (media.join("snes/covers").join(format!("{t}.png")), *size, i, t.to_string())).collect();
    draw_covers(&mut harness, &world.root, &covers, |_| {});
    let mut settings = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    settings["EsdeMediaDirectory"] = json!(media.to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();

    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("program", "coverStores", media.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "pads", "connected", "1", 5000);
    scene.run_pad_clock();
    wait_for(&mut scene, "screens", "shown", "Home", 3000);
    for _ in 0..40 {
        if !focused(&mut scene).is_empty() {
            break;
        }
        scene.wait(50);
    }
    for _ in 0..12 {
        let now = focused(&mut scene);
        if now == "place.0" {
            break;
        }
        press(&mut scene, &sdl, &pad, if now.starts_with("place.") { PadControl::Left } else { PadControl::Down });
    }
    press(&mut scene, &sdl, &pad, PadControl::South);
    wait_for(&mut scene, "screens", "shown", "Consoles", 2000);
    let places: usize = scene.property("library", "places").parse().unwrap();
    let tree = scene.tree();
    assert!(scene.focus_item(&tree, tree.find_named(&format!("tile.{places}")).expect("Super Nintendo on Choose a console")));
    press(&mut scene, &sdl, &pad, PadControl::South);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);
    assert!(scene.set_property("program", "theme", &themes::built_in_folder("eclipse")));
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    let warnings = scene.property("host.theme", "warnings");
    assert!(warnings.is_empty() && scene.property("host.theme", "failed") == "false", "Eclipse's screens load clean: {warnings}");
    wait_for(&mut scene, "gamesGrid", "count", &GAMES.to_string(), 5000);
    for _ in 0..40 {
        if focused(&mut scene) == "gamesTile.0" {
            break;
        }
        scene.wait(50);
    }
    assert_eq!(focused(&mut scene), "gamesTile.0", "the first game by name has the focus");
    wait_for(&mut scene, "gamesTitle", "text", PLAYED, 2000);
    says(&mut scene, "gamesFactValue.2", "3");
    says(&mut scene, "battery", "82%");
    f(&mut scene, &sdl, &pad);
}

fn collection_filter_key(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    let chip = tree.nodes.iter().find(|n| n.name == "gamesFilter.3").expect("the collection's chip, hidden");
    let id: u64 = chip.id.parse().unwrap();
    emusen_qt_test::ffi::ffi::harness_property(crate::eclipse_board::object(id), &cxx_qt_lib::QString::from("gamesFilter.3"), &cxx_qt_lib::QString::from("key")).to_string()
}

pub fn board_and_facts(backend: &str) {
    with_games_view("qt-app-eclipse-games-facts", |scene, sdl, pad| {
        scene.wait(200);
        board_held(scene);
        assert_eq!(words(scene, "genreChipWords"), "Action");
        assert_eq!(words(scene, "playersChipWords"), "1 to 2 players");
        assert_eq!(words(scene, "description"), DESCRIPTION);
        assert_eq!(words(scene, "gamesFactValue.1"), "3 h 20 min");
        assert!(words(scene, "gamesConsole").ends_with(" · 1994"), "the year from the scraper's release date: {}", words(scene, "gamesConsole"));
        pictures(scene, backend, "facts");
        crate::eclipse_board::resize_settle(scene, 1280, 800);
        bare_flows(scene, sdl, pad);
        pictures(scene, backend, "none");
        press(scene, sdl, pad, PadControl::Left);
        wait_for(scene, "gamesTitle", "text", PLAYED, 2000);
    });
}

pub fn controls_and_navigation() {
    with_games_view("qt-app-eclipse-games-nav", |scene, sdl, pad| {
        assert_eq!(words(scene, "gamesPathBack"), "Consoles");
        assert_eq!(words(scene, "gamesFilterWords.0"), format!("All · {GAMES}"));
        assert_eq!(words(scene, "gamesFilterWords.1"), format!("Favourites · {FAVOURITES}"));
        assert_eq!(words(scene, "gamesFilterWords.2"), format!("Not played · {}", GAMES - 1));
        assert!(shown(&scene.tree(), "gamesFilter.3").is_none(), "a collection is the List card's, no chip");
        assert_eq!(words(scene, "gamesSortWords"), "Name, A to Z");
        assert_eq!(words(scene, "buttonLabel.0"), "Start");
        assert_eq!(words(scene, "buttonLabel.1"), "Game page");
        assert_eq!(words(scene, "gamesLetterWords"), format!("Letter · 1 of {GAMES}"));
        assert_eq!(words(scene, "gamesFactLabel.0"), "Last played");
        let helped: Vec<String> = scene.tree().nodes.iter().filter(|n| n.visible && n.name.starts_with("help-")).map(|n| n.name.clone()).collect();
        assert_eq!(helped, ["help-Start", "help-Back", "help-Favourite"], "the foot names A, B and Y as the board does");

        assert_eq!(words(scene, "gamesRailLetterWords"), "A");
        press(scene, sdl, pad, PadControl::RightTrigger);
        wait_for(scene, "gamesRailLetterWords", "text", "B", 2000);
        let (x, y) = middle(scene, "gamesLetterBack");
        scene.click(x, y);
        wait_for(scene, "gamesRailLetterWords", "text", "A", 2000);
        let (x, y) = middle(scene, "gamesLetterOn");
        scene.click(x, y);
        wait_for(scene, "gamesRailLetterWords", "text", "B", 2000);
        press(scene, sdl, pad, PadControl::LeftTrigger);
        wait_for(scene, "gamesRailLetterWords", "text", "A", 2000);
        let tree = scene.tree();
        let track = shown(&tree, "gamesRailTrack").expect("the track");
        let (x, y) = (track.x + track.w / 2.0, track.y + track.h - 1.0);
        let last = words(scene, "gamesRailLast");
        scene.click(x, y);
        wait_for(scene, "gamesRailLetterWords", "text", &last, 2000);
        for _ in 0..30 {
            if words(scene, "gamesRailLetterWords") == "A" {
                break;
            }
            press(scene, sdl, pad, PadControl::LeftTrigger);
        }
        wait_for(scene, "gamesTitle", "text", PLAYED, 2000);
        assert_eq!(focused(scene), "gamesTile.0", "LT back to the first letter's first game");

        press(scene, sdl, pad, PadControl::Left);
        assert_eq!(focused(scene), "gamesStart");
        press(scene, sdl, pad, PadControl::Up);
        assert_eq!(focused(scene), "gamesPath", "Up from the column, the path above it");
        press(scene, sdl, pad, PadControl::Down);
        let tree = scene.tree();
        assert!(scene.focus_item(&tree, tree.find_named("gamesStart").unwrap()));
        press(scene, sdl, pad, PadControl::Right);
        assert_eq!(focused(scene), "gamesPage");
        press(scene, sdl, pad, PadControl::Right);
        assert_eq!(focused(scene), "gamesTile.0");
        press(scene, sdl, pad, PadControl::Up);
        assert_eq!(focused(scene), "gamesFilter.0");
        press(scene, sdl, pad, PadControl::Right);
        press(scene, sdl, pad, PadControl::Right);
        press(scene, sdl, pad, PadControl::Right);
        assert_eq!(focused(scene), "gamesSort", "past the last chip, the sort");
        press(scene, sdl, pad, PadControl::Left);
        assert_eq!(focused(scene), "gamesFilter.2");
        press(scene, sdl, pad, PadControl::Up);
        assert_eq!(focused(scene), "gamesPath");

        press(scene, sdl, pad, PadControl::Down);
        let tree = scene.tree();
        assert!(scene.focus_item(&tree, tree.find_named("gamesFilter.1").unwrap()));
        press(scene, sdl, pad, PadControl::South);
        wait_for(scene, "gamesGrid", "count", &FAVOURITES.to_string(), 3000);
        let (x, y) = middle(scene, "gamesFilter.0");
        scene.click(x, y);
        wait_for(scene, "gamesGrid", "count", &GAMES.to_string(), 3000);

        let tree = scene.tree();
        assert!(scene.focus_item(&tree, tree.find_named("gamesSort").unwrap()));
        press(scene, sdl, pad, PadControl::South);
        wait_for(scene, "screens", "shown", "List", 2000);
        scene.wait(500);
        press(scene, sdl, pad, PadControl::East);
        wait_for(scene, "screens", "shown", "Grid", 2000);
        let (x, y) = middle(scene, "gamesSort");
        scene.click(x, y);
        wait_for(scene, "screens", "shown", "List", 2000);
        scene.wait(500);
        press(scene, sdl, pad, PadControl::East);
        wait_for(scene, "screens", "shown", "Grid", 2000);

        let (x, y) = middle(scene, "gamesPath");
        scene.click(x, y);
        wait_for(scene, "screens", "shown", "Consoles", 2000);

        // Top-level Library ("all"): Brand wordmark, Library nav active, Search shown, breadcrumb hidden, LB/RB step Home<->Library<->Consoles, B->Home.
        press(scene, sdl, pad, PadControl::LeftShoulder);
        wait_for(scene, "screens", "shown", "Grid", 2000);
        assert_eq!(scene.property("screens", "key"), "all");
        assert_eq!(scene.property("screens", "section"), "library");
        wait_for(scene, "gamesGrid", "count", &GAMES.to_string(), 3000);
        scene.wait(200);
        {
            let tree = scene.tree();
            assert!(shown(&tree, "wordmark").is_some(), "top-level Library draws wordmark");
            assert!(shown(&tree, "nav").is_some(), "top-level Library draws section nav");
            assert!(shown(&tree, "navUnderline.library").is_some(), "Library tab underlined");
            assert!(shown(&tree, "search").is_some(), "top-level Library draws search button");
            assert!(shown(&tree, "gamesPath").is_none(), "top-level Library hides drill-down breadcrumb");
        }
        press(scene, sdl, pad, PadControl::LeftShoulder);
        wait_for(scene, "screens", "shown", "Home", 2000);
        assert_eq!(scene.property("screens", "section"), "home");
        press(scene, sdl, pad, PadControl::RightShoulder);
        wait_for(scene, "screens", "shown", "Grid", 2000);
        assert_eq!(scene.property("screens", "key"), "all");
        press(scene, sdl, pad, PadControl::RightShoulder);
        wait_for(scene, "screens", "shown", "Consoles", 2000);
        assert_eq!(scene.property("screens", "section"), "consoles");
        press(scene, sdl, pad, PadControl::LeftShoulder);
        wait_for(scene, "screens", "shown", "Grid", 2000);
        assert_eq!(scene.property("screens", "key"), "all");
        press(scene, sdl, pad, PadControl::East);
        wait_for(scene, "screens", "shown", "Home", 2000);
    });
}

pub fn grid_scroll_1920x1200() {
    with_games_view("qt-app-eclipse-games-scroll", |scene, sdl, pad| {
        let collection = collection_filter_key(scene);
        set_filter(scene, &collection, 1);
        reached_all_1920(scene, sdl, pad, 1);
        set_filter(scene, emusen_qt_models::list_card::FAVOURITES_ONLY, FAVOURITES);
        reached_all_1920(scene, sdl, pad, FAVOURITES);
        set_filter(scene, emusen_qt_models::list_card::EVERY_GAME, GAMES);
        reached_all_1920(scene, sdl, pad, GAMES);
    });
}

pub fn grid_fit_other_sizes() {
    with_games_view("qt-app-eclipse-games-sizes", |scene, sdl, pad| {
        let collection = collection_filter_key(scene);
        set_filter(scene, &collection, 1);
        reached_boundary_other(scene, sdl, pad, 1);
        set_filter(scene, emusen_qt_models::list_card::FAVOURITES_ONLY, FAVOURITES);
        reached_boundary_other(scene, sdl, pad, FAVOURITES);
        set_filter(scene, emusen_qt_models::list_card::EVERY_GAME, GAMES);
        reached_boundary_other(scene, sdl, pad, GAMES);
    });
}
