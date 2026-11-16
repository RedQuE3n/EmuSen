//! Eclipse's games view in the running program (EmuSen_Qt.md §3.4.10), driven by an SDL pad inside this process over a
//! library of 120 games, three with covers, the first by name played and thirty favourites: every row of eclipse/games.tsv
//! held at each of the gate's sizes, the fields and the disc in the focused game's colours; then over one game (a
//! collection of one), the thirty favourites and all 120, at each size, every game reached by the pad, the pointer and
//! the wheel, none cut and none under the help line; and the chips, the sort, the path, the letters and the buttons, each
//! pressed by the pad and by the pointer. One test, as the harness is one per process.

use crate::common::{draw_covers, sandbox, wait_for};
use crate::eclipse_pad::{focused, press};
use emusen_client_fixture::library;
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::input::PadControl;
use emusen_kit_core::tokens::Colour;
use emusen_platform_client::galaxia::consts::{FILE_GAMES, MODEL_APP_SETTINGS};
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
const BOARD: &str = include_str!("../eclipse/games.tsv");
/// The covers drawn, each a shape of its own: tall, wide and square, at sizes that are constants of this test.
const COVERS: [(&str, (u32, u32)); 3] = [("Aurora Drift", (300, 420)), ("Amber Relay", (420, 300)), ("Basalt Kings", (360, 360))];
/// The first by name, played, so the column shows its last play as the board does.
const PLAYED: &str = "Amber Relay";
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

/// Every row of the board's table held to the screen at each of the gate's sizes with the played game focused; the fields
/// in the board's saturation and lightness with the disc's hue; the line heights; five columns at the board's shape.
fn board_held(scene: &mut Scene<'_>) {
    let (board_height, rows) = crate::eclipse_board::rows(BOARD);
    let k = DESIGN_HEIGHT / board_height;
    let mut differences = Vec::new();
    for (width, height) in SIZES {
        scene.resize(width, height);
        scene.wait(300);
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
        // The rim takes the art's first hue; the first field the same hue, both fields the board's saturation and lightness.
        // The games' disc's rim, not the hidden empty board's.
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
fn pictures(scene: &mut Scene<'_>, backend: &str) {
    let Some(folder) = std::env::var_os("EMUSEN_ECLIPSE_PICTURES") else { return };
    std::fs::create_dir_all(&folder).unwrap();
    for (width, height) in [(1920, 1200), (1280, 800)] {
        scene.resize(width, height);
        scene.wait(500);
        let path = std::path::Path::new(&folder).join(format!("games-{width}x{height}-{backend}.png"));
        assert!(scene.grab(&path.display().to_string()), "{} is grabbed", path.display());
    }
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

/// Over the list shown (`count` games) at every gate size: Right walks every game in order, each whole when focused; the
/// wheel brings every row in from the top; the pointer moved onto each tile of a row brought in focuses it.
fn reached(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, count: usize) {
    for (width, height) in SIZES {
        scene.resize(width, height);
        scene.wait(300);
        let size = format!("{width}x{height}");
        // Back to the top by the wheel, where the walk starts on the first game.
        let (x, y) = middle(scene, "gamesGrid");
        scene.wheel(x, y, 200);
        scene.wait(300);
        let tree = scene.tree();
        assert!(scene.focus_item(&tree, tree.find_named("gamesTile.0").expect("the first tile")));
        scene.wait(100);
        for i in 0..count {
            assert_eq!(at_tile(scene), Some(i), "{size}: the pad's walk over {count}");
            if let Err(why) = whole(&scene.tree(), i) {
                panic!("{size}: {why}");
            }
            if i + 1 < count {
                press(scene, sdl, pad, PadControl::Right);
            }
        }
        // The wheel up to the top, then down a row a notch: each row brought in, each of its tiles pressed by the pointer.
        let columns: usize = crate::eclipse_board::read(&scene.tree(), "grid", "columns").parse().expect("the grid's columns");
        let (x, y) = middle(scene, "gamesGrid");
        scene.wheel(x, y, 200);
        scene.wait(300);
        assert!(at_tile(scene).is_some_and(|i| i < columns), "{size}: the wheel back to the first row");
        for row in 0..count.div_ceil(columns) {
            if row > 0 {
                scene.wheel(x, y, -1);
                scene.wait(150);
            }
            for i in row * columns..((row + 1) * columns).min(count) {
                let (cx, cy) = middle(scene, &format!("gamesTile.{i}"));
                // The pointer moved onto the tile takes the focus there; a click would play it (the left button is A there).
                scene.hover(cx, cy);
                scene.wait(60);
                assert_eq!(at_tile(scene), Some(i), "{size}: the pointer on game {i} of {count}");
                if let Err(why) = whole(&scene.tree(), i) {
                    panic!("{size}: by the wheel, {why}");
                }
            }
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

pub fn oracle(backend: &str) {
    let titles = titles();
    let files: Vec<String> = titles.iter().map(|t| format!("{t}.sfc")).collect();
    let world = sandbox("qt-app-eclipse-games", &files.iter().map(|f| ("SNES", f.as_str())).collect::<Vec<_>>());
    let path = |title: &str| world.root.join("Roms/SNES").join(format!("{title}.sfc")).to_str().unwrap().to_owned();
    let games = records();
    games.change(&json!({ "op": "started", "path": path(PLAYED), "at": "2026-11-13T10:00:00Z" })).unwrap();
    for title in titles.iter().skip(titles.len() - FAVOURITES) {
        games.change(&json!({ "op": "toggle_favourite", "path": path(title) })).unwrap();
    }
    let id = games.change(&json!({ "op": "create_collection", "name": COLLECTION, "at": "2026-11-13T10:00:00Z" })).unwrap();
    games.change(&json!({ "op": "add_to_collection", "id": id, "path": path(&titles[5]) })).unwrap();
    drop(games);
    let platform = Platform::load(library()).unwrap();
    let media = world.dir("Media");

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
    // Home's Games place, the reference's Choose a console, its first console's games; then Eclipse chosen over them.
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
    let tree = scene.tree();
    assert!(scene.focus_item(&tree, tree.find_named("tile.0").expect("All games on Choose a console")));
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
    scene.wait(500);
    board_held(&mut scene);
    pictures(&mut scene, backend);
    scene.resize(1280, 800);
    scene.wait(300);

    // The words are the program's: the path, the chips, the sort, the buttons, the foot and the letters.
    assert_eq!(words(&mut scene, "gamesPathBack"), "Consoles");
    assert_eq!(words(&mut scene, "gamesFilterWords.0"), format!("All · {GAMES}"));
    assert_eq!(words(&mut scene, "gamesFilterWords.1"), format!("Favourites · {FAVOURITES}"));
    assert_eq!(words(&mut scene, "gamesFilterWords.2"), format!("Not played · {}", GAMES - 1));
    assert!(shown(&scene.tree(), "gamesFilter.3").is_none(), "a collection is the List card's, no chip");
    assert_eq!(words(&mut scene, "gamesSortWords"), "Name, A to Z");
    assert_eq!(words(&mut scene, "buttonLabel.0"), "Start");
    assert_eq!(words(&mut scene, "buttonLabel.1"), "Game page");
    assert_eq!(words(&mut scene, "gamesLetterWords"), format!("Letter · 1 of {GAMES}"));
    assert_eq!(words(&mut scene, "gamesFactLabel.0"), "Last played");
    let helped: Vec<String> = scene.tree().nodes.iter().filter(|n| n.visible && n.name.starts_with("help-")).map(|n| n.name.clone()).collect();
    assert_eq!(helped, ["help-Start", "help-Back", "help-Favourite"], "the foot names A, B and Y as the board does");

    // The triggers step letters, by the pad and by a press on their glyphs; the rail says the letter.
    assert_eq!(words(&mut scene, "gamesRailLetterWords"), "A");
    press(&mut scene, &sdl, &pad, PadControl::RightTrigger);
    wait_for(&mut scene, "gamesRailLetterWords", "text", "B", 2000);
    let (x, y) = middle(&mut scene, "gamesLetterBack");
    scene.click(x, y);
    wait_for(&mut scene, "gamesRailLetterWords", "text", "A", 2000);
    let (x, y) = middle(&mut scene, "gamesLetterOn");
    scene.click(x, y);
    wait_for(&mut scene, "gamesRailLetterWords", "text", "B", 2000);
    press(&mut scene, &sdl, &pad, PadControl::LeftTrigger);
    wait_for(&mut scene, "gamesRailLetterWords", "text", "A", 2000);
    // A press on the rail's track at its foot steps to the last letter.
    let tree = scene.tree();
    let track = shown(&tree, "gamesRailTrack").expect("the track");
    let (x, y) = (track.x + track.w / 2.0, track.y + track.h - 1.0);
    let last = words(&mut scene, "gamesRailLast");
    scene.click(x, y);
    wait_for(&mut scene, "gamesRailLetterWords", "text", &last, 2000);
    for _ in 0..30 {
        if words(&mut scene, "gamesRailLetterWords") == "A" {
            break;
        }
        press(&mut scene, &sdl, &pad, PadControl::LeftTrigger);
    }
    wait_for(&mut scene, "gamesTitle", "text", PLAYED, 2000);
    assert_eq!(focused(&mut scene), "gamesTile.0", "LT back to the first letter's first game");

    // The walk off the tiles: Left to Start, Right to Game page and back to the tiles; Up to the chip in force, Up to the path.
    press(&mut scene, &sdl, &pad, PadControl::Left);
    assert_eq!(focused(&mut scene), "gamesStart");
    press(&mut scene, &sdl, &pad, PadControl::Right);
    assert_eq!(focused(&mut scene), "gamesPage");
    press(&mut scene, &sdl, &pad, PadControl::Right);
    assert_eq!(focused(&mut scene), "gamesTile.0");
    press(&mut scene, &sdl, &pad, PadControl::Up);
    assert_eq!(focused(&mut scene), "gamesFilter.0");
    press(&mut scene, &sdl, &pad, PadControl::Right);
    press(&mut scene, &sdl, &pad, PadControl::Right);
    press(&mut scene, &sdl, &pad, PadControl::Right);
    assert_eq!(focused(&mut scene), "gamesSort", "past the last chip, the sort");
    press(&mut scene, &sdl, &pad, PadControl::Left);
    assert_eq!(focused(&mut scene), "gamesFilter.2");
    press(&mut scene, &sdl, &pad, PadControl::Up);
    assert_eq!(focused(&mut scene), "gamesPath");

    // The chips by the pad and by the pointer: Favourites shows the thirty, All all of them.
    press(&mut scene, &sdl, &pad, PadControl::Down);
    let tree = scene.tree();
    assert!(scene.focus_item(&tree, tree.find_named("gamesFilter.1").unwrap()));
    press(&mut scene, &sdl, &pad, PadControl::South);
    wait_for(&mut scene, "gamesGrid", "count", &FAVOURITES.to_string(), 3000);
    let (x, y) = middle(&mut scene, "gamesFilter.0");
    scene.click(x, y);
    wait_for(&mut scene, "gamesGrid", "count", &GAMES.to_string(), 3000);

    // The sort by the pad and by the pointer: View's action, the List card, which B closes.
    let tree = scene.tree();
    assert!(scene.focus_item(&tree, tree.find_named("gamesSort").unwrap()));
    press(&mut scene, &sdl, &pad, PadControl::South);
    wait_for(&mut scene, "screens", "shown", "List", 2000);
    // The card takes the pad once it has slid in.
    scene.wait(500);
    press(&mut scene, &sdl, &pad, PadControl::East);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);
    let (x, y) = middle(&mut scene, "gamesSort");
    scene.click(x, y);
    wait_for(&mut scene, "screens", "shown", "List", 2000);
    // The card takes the pad once it has slid in.
    scene.wait(500);
    press(&mut scene, &sdl, &pad, PadControl::East);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);

    // Every game reached at every size: one (the collection), the thirty favourites, all 120.
    let collection = {
        let tree = scene.tree();
        let chip = tree.nodes.iter().find(|n| n.name == "gamesFilter.3").expect("the collection's chip, hidden");
        let id: u64 = chip.id.parse().unwrap();
        emusen_qt_test::ffi::ffi::harness_property(crate::eclipse_board::object(id), &cxx_qt_lib::QString::from("gamesFilter.3"), &cxx_qt_lib::QString::from("key")).to_string()
    };
    set_filter(&mut scene, &collection, 1);
    reached(&mut scene, &sdl, &pad, 1);
    set_filter(&mut scene, emusen_qt_models::list_card::FAVOURITES_ONLY, FAVOURITES);
    reached(&mut scene, &sdl, &pad, FAVOURITES);
    set_filter(&mut scene, emusen_qt_models::list_card::EVERY_GAME, GAMES);
    reached(&mut scene, &sdl, &pad, GAMES);

    // The path by the pointer goes back, as B does.
    scene.resize(1280, 800);
    scene.wait(300);
    let (x, y) = middle(&mut scene, "gamesPath");
    scene.click(x, y);
    wait_for(&mut scene, "screens", "shown", "Consoles", 2000);
}
