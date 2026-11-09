//! A game's page in the program (EmuSen_Qt.md §7.8): X on the grid opens it, the shoulders step its five sections, an
//! engine stepped is saved, Y turns the heart, B returns to the grid; the five states Mistress recorded are held at the
//! gate's sizes and recorded; and the hero's clip, a clip this test writes itself, plays after the still, stops when
//! the page loses the focus, and a game with no clip keeps its still.

mod common;

use common::{draw_covers, hold, sandbox, wait_for, write_clip};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::{FILE_GAMES, MODEL_APP_SETTINGS, SAVE_RESUME_STATE, SAVE_STATE};
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt::MAIN_URL;
use emusen_qt_test::{mockup, Scene};
use std::path::Path;
use std::time::{Duration, SystemTime};

const COBALT: &str = "Cobalt Harbor (Synthetic)";
const LANTERN: &str = "Lantern Bay (Synthetic)";

/// The gate's findings this screen names for the tester: the help bar's words under the floor at 1280 × 720, as every
/// big-screen state names them; and Favourite, which Right reaches from Resume only once Resume has stepped through its
/// states (LunaP's `MoveInActions`), where the walk presses each direction once (held below, EmuSen_Qt.md §7.8).
fn named(fault: &str, tree: &emusen_qt_test::tree::Tree, size: (i32, i32)) -> bool {
    let item = fault.rsplit(": ").next().and_then(|n| n.parse::<usize>().ok()).and_then(|i| tree.nodes.get(i));
    let favourite = fault.contains("the pad does not reach") && item.is_some_and(|n| n.ty.starts_with("ActionButton") && n.heard == "Favourite");
    favourite || (size == (1280, 720) && fault.contains("too small to read"))
}

/// A state's file, saved `ago` seconds before now.
fn state(platform: &Platform, kind: u32, rom: &Path, slot: i32, ago: u64) {
    let file = platform.save_path(kind, rom.to_str().unwrap(), None, slot).unwrap();
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, b"state").unwrap();
    let f = std::fs::File::options().write(true).open(&file).unwrap();
    f.set_modified(SystemTime::now() - Duration::from_secs(ago)).unwrap();
}

fn set_setting(platform: &Platform, key: &str, value: serde_json::Value) {
    let mut settings = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    settings[key] = value;
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
}

/// The grid's game at `index` in hand and its page opened by X.
fn open_page(scene: &mut Scene<'_>, index: usize) {
    for _ in 0..index {
        scene.pad(UiButton::Right);
    }
    scene.wait(100);
    scene.pad(UiButton::Screensaver);
    wait_for(scene, "screens", "shown", "Page", 2000);
    wait_for(scene, "pageModel", "ready", "true", 10000);
    scene.wait(300);
}

/// The section by its index, through the right shoulder.
fn section(scene: &mut Scene<'_>, index: i32) {
    for _ in 0..5 {
        if scene.property("pageModel", "section") == index.to_string() {
            break;
        }
        scene.pad(UiButton::PageDown);
        scene.wait(100);
    }
    assert_eq!(scene.property("pageModel", "section"), index.to_string());
    scene.wait(200);
}

/// The hero's colour a little under its top, where the OnArt gradient is still clear.
fn hero_colour(scene: &mut Scene<'_>) -> emusen_kit_core::tokens::Colour {
    scene.colours_at(&[(640.0, 30.0)])[0].expect("the hero drawn")
}

#[test]
fn qt_7_8_a_games_page_opens_from_the_grid_holds_its_states_and_plays_its_clip() {
    let world = sandbox("qt-app-game-page", &[("SNES", &format!("{COBALT}.sfc")), ("SNES", &format!("{LANTERN}.sfc"))]);
    let platform = Platform::load(library()).unwrap();
    let rom = world.root.join("Roms/SNES").join(format!("{COBALT}.sfc"));

    // Mistress's fixture's facts, as edits in games.db, which the scraper's metadata reads first.
    let sqlite = Lent::open("libsqlite3.so.0").unwrap();
    let games = Library::open(&platform, &platform.file(FILE_GAMES).unwrap(), &sqlite).unwrap();
    let changes = serde_json::json!({ "developer": "Lantern Works", "publisher": "Publisher P", "releasedate": "1994-03-18",
        "description": "Cobalt Harbor is a game invented for EmuSen's tests. It has no story and no pictures of its own." });
    games.change(&serde_json::json!({ "op": "save_edits", "path": rom.to_str().unwrap(), "changes": changes, "at": "2026-11-09T00:00:00Z" })).unwrap();
    drop(games);

    // Four states: slots 1 to 3 and the automatic one, as the recording's.
    state(&platform, SAVE_STATE, &rom, 1, 2 * 3600);
    state(&platform, SAVE_STATE, &rom, 2, 26 * 3600);
    state(&platform, SAVE_STATE, &rom, 3, 3 * 86_400);
    state(&platform, SAVE_RESUME_STATE, &rom, 0, 12 * 60);

    // Its pictures in the ES-DE media folder, and its clip; the other game has neither.
    let media = world.dir("Media");
    let mut harness = emusen_qt_test::Harness::new();
    let pictures: Vec<_> = ["screenshots", "titlescreens", "marquees", "covers"]
        .iter()
        .enumerate()
        .map(|(i, folder)| (media.join("snes").join(folder).join(format!("{COBALT}.png")), (512, 448), i, format!("snes {folder}")))
        .collect();
    draw_covers(&mut harness, &world.root, &pictures, |_| {});
    write_clip(&media.join("snes/videos").join(format!("{COBALT}.mkv")));
    set_setting(&platform, "EsdeMediaDirectory", serde_json::json!(media.to_str().unwrap()));
    // The recordings hold a still: the clip is off while the states are held, and on for the clip's own part.
    set_setting(&platform, "PlayVideos", serde_json::json!(false));

    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    // The folder the theme's covers may come from, as the program's start reads it from the settings.
    assert!(scene.set_property("program", "coverStores", media.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    scene.wait(300);
    // Games, then All Games, the grid.
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Consoles", 1000);
    let tree = scene.tree();
    let all = tree.find_named("tile.0").expect("All Games");
    assert!(scene.focus_item(&tree, all));
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);
    scene.wait(300);

    // X opens the focused game's page, on its Overview with the focus on Play.
    open_page(&mut scene, 0);
    assert_eq!(scene.property("pageModel", "title"), COBALT);
    let facts = scene.property("pageModel", "facts");
    assert!(facts.ends_with("1994 · Lantern Works · Publisher P · Never played"), "{facts}");
    assert!(scene.property("pageModel", "words").starts_with("Cobalt Harbor is a game invented"));
    assert_eq!(scene.property("pageModel", "clip"), "", "Play videos off: no clip");
    let mut report = Vec::new();
    // Right from Play is Resume; Right steps Resume through the four states, then reaches Favourite.
    let help = |s: emusen_qt_models::help::Screen| emusen_qt_models::help_model::shown_screen(s).to_string();
    scene.pad(UiButton::Right);
    wait_for(&mut scene, "pageModel", "help", &help(emusen_qt_models::help::Screen::GamePageResumeSlots), 1000);
    let first = scene.property("pageModel", "actions");
    scene.pad(UiButton::Right);
    scene.wait(100);
    assert_ne!(scene.property("pageModel", "actions"), first, "Right steps Resume's state");
    for _ in 0..3 {
        scene.pad(UiButton::Right);
        scene.wait(50);
    }
    wait_for(&mut scene, "pageModel", "help", &help(emusen_qt_models::help::Screen::GamePageFavourite), 1000);
    for _ in 0..6 {
        scene.pad(UiButton::Left);
        scene.wait(50);
    }
    wait_for(&mut scene, "pageModel", "help", &help(emusen_qt_models::help::Screen::GamePage), 1000);
    hold(&mut scene, "GamePage", &mockup::Mockup::default(), named, &mut report);

    section(&mut scene, 1);
    assert_eq!(scene.property("pageModel", "shows"), "1");
    hold(&mut scene, "GamePageStates", &mockup::Mockup::default(), named, &mut report);
    section(&mut scene, 2);
    assert_eq!(scene.property("pageModel", "shows"), "2");
    hold(&mut scene, "GamePageOptions", &mockup::Mockup::default(), named, &mut report);
    section(&mut scene, 3);
    hold(&mut scene, "GamePageMedia", &mockup::Mockup::default(), named, &mut report);
    section(&mut scene, 4);
    hold(&mut scene, "GamePageDetails", &mockup::Mockup::default(), named, &mut report);

    // Y turns the heart on, saved in games.db.
    let hearted = scene_actions_with(&mut scene, "♥");
    scene.pad(UiButton::Search);
    wait_for(&mut scene, "pageModel", "actions", &hearted, 2000);

    // B returns to the grid, the focus on the same game.
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);

    // With Play videos on: the still first, then the clip over it after the poster's delay, drawn on this backend.
    set_setting(&platform, "PlayVideos", serde_json::json!(true));
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "screens", "shown", "Page", 2000);
    for _ in 0..100 {
        if !scene.property("pageModel", "clip").is_empty() {
            break;
        }
        scene.wait(50);
    }
    assert!(scene.property("pageModel", "clip").ends_with(".mkv"));
    let still = hero_colour(&mut scene);
    assert!(still.g > 150 && still.r < 150, "the still first, the screenshot's green: {still:?}");
    wait_for(&mut scene, "videoPlayer", "playbackState", "PlayingState", 4000);
    wait_for(&mut scene, "pageClip", "showing", "true", 4000);
    assert!(scene.property("videoFrames", "frames").parse::<i32>().unwrap_or(0) > 0, "the frames drawn by the kit's sink");
    // The fade over the still done.
    scene.wait(1200);
    let clip = hero_colour(&mut scene);
    assert!(clip.r > 150 && clip.b > 150 && clip.g < 110, "the clip's magenta over the still: {clip:?}");

    // B: the page loses the focus, and the clip stops.
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);
    wait_for(&mut scene, "videoPlayer", "playbackState", "StoppedState", 2000);

    // The other game has no clip: its page keeps its still, and nothing plays.
    open_page(&mut scene, 1);
    assert_eq!(scene.property("pageModel", "title"), LANTERN);
    assert_eq!(scene.property("pageModel", "clip"), "");
    scene.wait(2500);
    assert_eq!(scene.property("videoPlayer", "playbackState"), "StoppedState");
    assert_eq!(scene.property("videoFrames", "frames"), "0");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);
    scene.assert_quiet();
    assert!(report.is_empty(), "{}", report.join("\n"));
}

/// The actions' text with the heart's mark in place of the one shown.
fn scene_actions_with(scene: &mut Scene<'_>, heart: &str) -> String {
    scene.property("pageModel", "actions").replace("\"♡\"", &format!("\"{heart}\""))
}
