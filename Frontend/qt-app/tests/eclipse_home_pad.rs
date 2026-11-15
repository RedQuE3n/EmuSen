//! Eclipse's Home acts on what it draws (EmuSen_Qt.md §3.4.8), in the program under Eclipse with an in-process SDL pad: the
//! walk reaches every button and every tile drawn and the search; A on the first button starts the game shown, A on a tile
//! other than the first starts that tile's game, X and the second button open the game's page, Y and the third button turn
//! it a favourite, and A on the search opens the search.

mod common;

use common::wait_for;
use emusen_client_fixture::{library, World};
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::{FILE_GAMES, MODEL_APP_SETTINGS};
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt::MAIN_URL;
use emusen_qt_models::{themes, words};
use emusen_qt_test::virtual_pad::{Sdl, VirtualPad};
use emusen_qt_test::Scene;
use serde_json::json;

/// Six games in Recent's order, each with an extension only its console lists, as the oracle's library.
const GAMES: [(&str, &str); 6] = [
    ("Aurora Drift", "sfc"),
    ("Lumen Garden", "gb"),
    ("Tower Set", "nes"),
    ("Cobalt Harbor", "z64"),
    ("Ember Circuit", "sfc"),
    ("Juniper Vale", "gb"),
];

fn rom(world: &World, title: &str, ext: &str) -> String {
    world.dir("Roms").join(format!("{title}.{ext}")).to_string_lossy().into_owned()
}

/// games.db where Galaxia names it, as the program opens it.
fn records() -> Library {
    let platform = Platform::load(library()).unwrap();
    let sqlite = Lent::open("libsqlite3.so.0").unwrap();
    Library::open(&platform, &platform.file(FILE_GAMES).unwrap(), &sqlite).unwrap()
}

/// The games written and played an hour apart, the first the latest.
fn played_library() -> World {
    let world = World::new("qt-app-eclipse-home-pad");
    for (title, ext) in GAMES {
        std::fs::write(rom(&world, title, ext), [0u8; 4096]).unwrap();
    }
    let platform = Platform::load(library()).unwrap();
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = json!(world.root.join("Roms").to_string_lossy());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    let games = records();
    let now = words::now();
    for (i, (title, ext)) in GAMES.iter().enumerate() {
        games.change(&json!({ "op": "started", "path": rom(&world, title, ext), "at": words::utc(now - 3600 * (i as i64 + 1)) })).unwrap();
    }
    world
}

fn until(scene: &mut Scene<'_>, what: &str, ms: i32, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    let mut waited = 0;
    while !done(scene) && waited < ms {
        scene.wait(25);
        waited += 25;
    }
    assert!(done(scene), "{what} within {ms} ms");
}

fn focused(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default()
}

/// A press as a hand makes it on the virtual pad: down until the router holds it, then up until the router has seen it go.
fn hand(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, button: UiButton) {
    hand_until(scene, sdl, pad, button, |_| false);
}

/// `hand`, for a press whose action may take the focus to the program's own engine before its router is read: down until
/// the router holds it or `acted` says it acted, then up until the router of the engine now focused holds it no more.
fn hand_until(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, button: UiButton, acted: impl Fn(&mut Scene<'_>) -> bool) {
    let control = button.control(false);
    let sdl_button = emusen_qt_endymion::reading::sdl_button(control).unwrap();
    sdl.press(pad, sdl_button, true);
    until(scene, &format!("the router holding {control:?}"), 2000, |s| s.pad_held(button) || acted(s));
    sdl.press(pad, sdl_button, false);
    until(scene, &format!("the router letting {control:?} go"), 2000, |s| !s.pad_held(button));
    scene.wait(50);
}

/// Pressed until the focus is on `name`, at most `most` presses, each landing somewhere named.
fn walk_to(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, button: UiButton, name: &str, most: usize) {
    for _ in 0..most {
        if focused(scene) == name {
            return;
        }
        hand(scene, sdl, pad, button);
    }
    assert_eq!(focused(scene), name, "{button:?} pressed up to {most} times");
}

fn favourite(path: &str) -> bool {
    let all = records().get(&json!({ "what": "records" })).unwrap();
    all.get(path).and_then(|r| r.get("favourite")).and_then(serde_json::Value::as_bool).unwrap_or(false)
}

/// Back to Home from the program's own screen: its B through the router's queue, as the program's tests leave a page; the pad's
/// clock running again after.
fn home(scene: &mut Scene<'_>) {
    for _ in 0..6 {
        if scene.property("screens", "shown") == "Home" {
            break;
        }
        scene.pad(UiButton::Back);
        scene.wait(200);
    }
    wait_for(scene, "screens", "shown", "Home", 3000);
    let mut waited = 0;
    let on_home = |s: &mut Scene<'_>| ["tile.", "button.", "search"].iter().any(|p| focused(s).starts_with(p));
    while !on_home(scene) && waited < 3000 {
        scene.wait(25);
        waited += 25;
    }
    assert!(on_home(scene), "Home's focus back on what it draws: on {:?}", focused(scene));
    scene.run_pad_clock();
}

/// Down to the row from wherever Home's focus is, then Left to its first tile.
fn first_tile(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad) {
    for _ in 0..2 {
        if !focused(scene).starts_with("tile.") {
            hand(scene, sdl, pad, UiButton::Down);
        }
    }
    walk_to(scene, sdl, pad, UiButton::Left, "tile.0", GAMES.len());
}

/// Whether the press made the program start the game at `path`: the launch's start records it in games.db, which makes it
/// the library's last played, and asks for the game's screen. No game's picture is fed in the harness, so nothing opens it,
/// and the program is loaded again for the next press (EmuSen_Qt.md §3.4.8).
fn started(scene: &mut Scene<'_>, path: &str) {
    until(scene, &format!("{path} started"), 5000, |s| s.property("library", "lastPlayed") == path);
    until(scene, "the game's screen asked for", 3000, |s| s.property("screens", "shown") == "Game");
}

/// The program under Eclipse on the library, its first tile focused and the pad's clock running.
fn program(harness: &mut emusen_qt_test::Harness) -> Scene<'_> {
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    assert!(scene.set_property("program", "theme", &themes::built_in_folder("eclipse")));
    wait_for(&mut scene, "host.theme", "ready", "true", 3000);
    assert_eq!(scene.property("theme", "home"), "true", "Eclipse draws Home");
    until(&mut scene, "the first tile focused", 3000, |s| focused(s) == "tile.0");
    scene.run_pad_clock();
    scene
}

#[test]
fn qt_3_4_8_eclipse_s_home_acts_on_every_button_tile_and_the_search_from_the_pad() {
    let world = played_library();
    let paths: Vec<String> = GAMES.iter().map(|(t, e)| rom(&world, t, e)).collect();
    let sdl = Sdl::open();
    let pad = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = program(&mut harness);
    assert_eq!(scene.property("library", "lastPlayed"), paths[0]);

    // The walk: Right through every tile drawn, Up to the buttons, Right through them, Up to the search, Down back.
    let tree = scene.tree();
    let drawn: Vec<String> = (0..GAMES.len()).map(|i| format!("tile.{i}")).filter(|n| tree.find_named(n).is_some_and(|i| tree.nodes[i as usize].visible)).collect();
    assert_eq!(drawn.len(), GAMES.len(), "every game's tile is drawn at 1280 by 800");
    for name in &drawn[1..] {
        hand(&mut scene, &sdl, &pad, UiButton::Right);
        assert_eq!(&focused(&mut scene), name);
    }
    walk_to(&mut scene, &sdl, &pad, UiButton::Left, "tile.0", GAMES.len());
    let mut walked = vec![focused(&mut scene)];
    for button in [UiButton::Up, UiButton::Right, UiButton::Right, UiButton::Right, UiButton::Up, UiButton::Down, UiButton::Down] {
        hand(&mut scene, &sdl, &pad, button);
        walked.push(focused(&mut scene));
    }
    assert_eq!(walked, ["tile.0", "button.0", "button.1", "button.2", "button.2", "search", "button.0", "tile.0"], "the pad's walk");

    // X opens the shown game's page; so does A on the second button, for the tile shown.
    hand_until(&mut scene, &sdl, &pad, UiButton::Screensaver, |s| s.property("screens", "shown") == "Page");
    until(&mut scene, "X opening the game's page", 3000, |s| s.property("screens", "shown") == "Page");
    wait_for(&mut scene, "pageModel", "ready", "true", 10000);
    assert_eq!(scene.property("pageModel", "title"), GAMES[0].0, "X opens the page of the game shown");
    home(&mut scene);
    first_tile(&mut scene, &sdl, &pad);
    walk_to(&mut scene, &sdl, &pad, UiButton::Right, "tile.2", 3);
    hand(&mut scene, &sdl, &pad, UiButton::Up);
    walk_to(&mut scene, &sdl, &pad, UiButton::Right, "button.1", 2);
    hand_until(&mut scene, &sdl, &pad, UiButton::Accept, |s| s.property("screens", "shown") == "Page");
    until(&mut scene, "the second button opening the game's page", 3000, |s| s.property("screens", "shown") == "Page");
    wait_for(&mut scene, "pageModel", "ready", "true", 10000);
    assert_eq!(scene.property("pageModel", "title"), GAMES[2].0, "the second button opens the page of the game shown");
    home(&mut scene);

    // Y turns the shown game a favourite; A on the third button turns it back.
    first_tile(&mut scene, &sdl, &pad);
    assert!(!favourite(&paths[0]));
    hand(&mut scene, &sdl, &pad, UiButton::Search);
    until(&mut scene, "Y turning the game a favourite", 5000, |_| favourite(&paths[0]));
    hand(&mut scene, &sdl, &pad, UiButton::Up);
    walk_to(&mut scene, &sdl, &pad, UiButton::Right, "button.2", 3);
    hand(&mut scene, &sdl, &pad, UiButton::Accept);
    until(&mut scene, "the third button turning it back", 5000, |_| !favourite(&paths[0]));

    // A on the search opens the search; B gives Home back.
    hand(&mut scene, &sdl, &pad, UiButton::Up);
    assert_eq!(focused(&mut scene), "search");
    hand_until(&mut scene, &sdl, &pad, UiButton::Accept, |s| s.property("screens", "shown") == "Search");
    wait_for(&mut scene, "screens", "shown", "Search", 3000);
    home(&mut scene);

    // A on the first button starts the game shown: the second tile's, not the first's.
    first_tile(&mut scene, &sdl, &pad);
    hand(&mut scene, &sdl, &pad, UiButton::Right);
    hand(&mut scene, &sdl, &pad, UiButton::Up);
    assert_eq!(focused(&mut scene), "button.0");
    hand_until(&mut scene, &sdl, &pad, UiButton::Accept, |s| s.property("screens", "shown") == "Game");
    started(&mut scene, &paths[1]);
    scene.assert_quiet();
    drop(scene);

    // A on a tile other than the first starts that tile's game: Recent now puts the game just started first.
    let mut scene = program(&mut harness);
    assert_eq!(scene.property("library", "lastPlayed"), paths[1]);
    walk_to(&mut scene, &sdl, &pad, UiButton::Right, "tile.3", 3);
    hand_until(&mut scene, &sdl, &pad, UiButton::Accept, |s| s.property("screens", "shown") == "Game");
    started(&mut scene, &paths[3]);
    scene.assert_quiet();
    drop(scene);
    sdl.detach(pad);
}
