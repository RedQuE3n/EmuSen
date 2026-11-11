//! A game's page in the program (EmuSen_Qt.md §7.8): X on the grid opens it, the shoulders step its five sections, the
//! television standard stepped is saved, Open Folder gives the folder's URL, Y turns the heart, B returns to the grid;
//! the five states Mistress recorded are held at the gate's sizes, the pad reaching every action, and recorded; the
//! player's own screenshot is listed in Media; the hero's clip, a clip this test writes itself, plays after the still,
//! stops when the page loses the focus, and a game with no clip keeps its still; a hidden game leaves the grid; and a
//! state chosen is the one the game starts from.

mod common;

use common::{draw_covers, hold, sandbox, wait_for, write_clip};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::{DIR_SCREENSHOTS, FILE_GAMES, MODEL_APP_SETTINGS, SAVE_RESUME_STATE, SAVE_STATE};
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt::MAIN_URL;
use emusen_qt_test::{mockup, Scene};
use std::path::Path;
use std::time::{Duration, SystemTime};

const COBALT: &str = "Cobalt Harbor (Synthetic)";
const LANTERN: &str = "Lantern Bay (Synthetic)";

/// No finding is named for the tester: the gate holds every state whole.
fn nothing_named(_: &str, _: &emusen_qt_test::tree::Tree, _: (i32, i32)) -> bool {
    false
}

/// The focus moved by `button` until it is on the item heard as `heard`.
fn move_to(scene: &mut Scene<'_>, button: UiButton, heard: &str) {
    let mut seen = Vec::new();
    for _ in 0..10 {
        let tree = scene.tree();
        let now = scene.focus(&tree).map(|i| tree.nodes[i as usize].heard.clone()).unwrap_or_default();
        if now == heard {
            return;
        }
        seen.push(now);
        scene.pad(button);
        scene.wait(80);
    }
    panic!("{button:?} never reached {heard}: {seen:?}");
}

/// The question opened, its enter transition done, and its first answer holding the focus, so a press is the question's
/// however long it took to open.
fn answers_focused(scene: &mut Scene<'_>) {
    for _ in 0..120 {
        if scene.property("pageQuestion", "opened") == "true" {
            let tree = scene.tree();
            if scene.focus(&tree).is_some_and(|i| tree.nodes[i as usize].name == "answer.0") {
                return;
            }
        }
        scene.wait(25);
    }
    panic!("the question never opened with its first answer focused");
}

/// `done` within two seconds, else the test fails saying `what`.
fn until(scene: &mut Scene<'_>, what: &str, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    for _ in 0..80 {
        if done(scene) {
            return;
        }
        scene.wait(25);
    }
    assert!(done(scene), "{what} within two seconds");
}

/// X declared as Delete… by the page's help, and its words on the bar, the question's answers gone from the screen.
fn delete_declared(scene: &mut Scene<'_>) {
    until(scene, "the question gone", |s| !s.tree().nodes.iter().any(|n| n.visible && n.name == "answer.0"));
    until(scene, "X declared as Delete…", |s| s.property("pageHelp", "actionLines").contains(emusen_qt_models::help::DELETE));
    assert!(comes_to_read(scene, "Delete…"), "X reads Delete…");
}

/// What the focused item is heard as.
fn heard(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].heard.clone()).unwrap_or_default()
}

/// The page's tiles holding the focus, or letting it go.
fn tiles_hold_focus(scene: &mut Scene<'_>, held: bool) {
    until(scene, &format!("the tiles' focus {held}"), |s| s.property("pageTiles", "activeFocus") == held.to_string());
}

/// The page's tiles holding the focus on the tile at `index`, within two seconds.
fn tiles_focused_at(scene: &mut Scene<'_>, index: i32) {
    let now = |s: &mut Scene<'_>| (s.property("pageTiles", "activeFocus"), s.property("pageTiles", "currentIndex"));
    for _ in 0..40 {
        if now(scene) == ("true".to_owned(), index.to_string()) {
            return;
        }
        scene.wait(50);
    }
    assert_eq!(now(scene), ("true".to_owned(), index.to_string()), "the tiles focused on tile {index}");
}

fn down_to(scene: &mut Scene<'_>, row: &str) {
    move_to(scene, UiButton::Down, row);
}

/// Whether a text on the screen reads `words`.
fn reads(scene: &mut Scene<'_>, words: &str) -> bool {
    scene.tree().nodes.iter().any(|n| n.visible && n.text.as_deref() == Some(words))
}

/// Whether `words` come on the screen within two seconds: a row's value is read from the library after the row is shown.
fn comes_to_read(scene: &mut Scene<'_>, words: &str) -> bool {
    for _ in 0..40 {
        if reads(scene, words) {
            return true;
        }
        scene.wait(50);
    }
    false
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

/// The section by its index, through the right shoulder, each press waited for before the next, so none overshoots.
fn section(scene: &mut Scene<'_>, index: i32) {
    for _ in 0..5 {
        let now = scene.property("pageModel", "section");
        if now == index.to_string() {
            break;
        }
        scene.pad(UiButton::PageDown);
        for _ in 0..40 {
            if scene.property("pageModel", "section") != now {
                break;
            }
            scene.wait(25);
        }
    }
    assert_eq!(scene.property("pageModel", "section"), index.to_string());
    scene.wait(200);
}

/// The hero's colour a little under its top, where the OnArt gradient is still clear.
fn help(screen: emusen_qt_models::help::Screen) -> String {
    emusen_qt_models::help_model::shown_screen(screen).to_string()
}

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
    assert_eq!(scene.property("pageClip", "running"), "false", "no clip, no player: Qt Multimedia not started");
    let mut report = Vec::new();
    // Right from Play is Resume; Right steps Resume through the four states, then reaches Favourite.
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
    hold(&mut scene, "GamePage", &mockup::Mockup::default(), nothing_named, &mut report);

    section(&mut scene, 1);
    assert_eq!(scene.property("pageModel", "shows"), "1");
    hold(&mut scene, "GamePageStates", &mockup::Mockup::default(), nothing_named, &mut report);
    // A step of Resume marks its state's tile in place, with no reset: the list is told of the two tiles whose mark
    // moved and no other, and Down comes back to the tile the focus left.
    scene.pad(UiButton::Down);
    tiles_hold_focus(&mut scene, true);
    move_to(&mut scene, UiButton::Right, "Slot 3");
    scene.pad(UiButton::Up);
    tiles_hold_focus(&mut scene, false);
    move_to(&mut scene, UiButton::Right, "Resume");
    let before = scene.property("pageModel", "actions");
    assert!(scene.watch_rows("pageModel"));
    assert!(scene.watch("pageModel", "help"));
    scene.pad(UiButton::Left);
    until(&mut scene, "Left steps Resume's state", |s| s.property("pageModel", "actions") != before);
    assert_eq!(scene.rows_told("pageModel"), "1-2", "the two neighbouring tiles the mark moved between, told in place as one run");
    scene.pad(UiButton::Down);
    tiles_focused_at(&mut scene, 2);
    assert_eq!(heard(&mut scene), "Slot 3", "the tiles kept their focused tile through Resume's step");
    scene.pad(UiButton::Up);
    tiles_hold_focus(&mut scene, false);
    move_to(&mut scene, UiButton::Right, "Resume");
    scene.pad(UiButton::Right);
    until(&mut scene, "Right steps Resume back", |s| s.property("pageModel", "actions") == before);
    assert_eq!(scene.rows_told("pageModel"), "1-2", "and back, in place");
    // Left to Slot 2, to Slot 1, then to the automatic state, the newest and drawn after the last slot in use: that step
    // moves the mark from the first tile to the fourth, two runs apart, each told alone; then Right back to Slot 3.
    for (button, told) in [(UiButton::Left, "1-2"), (UiButton::Left, "0-1"), (UiButton::Left, "0-0,3-3"), (UiButton::Right, "0-0,3-3"), (UiButton::Right, "0-1"), (UiButton::Right, "1-2")] {
        let now = scene.property("pageModel", "actions");
        scene.pad(button);
        until(&mut scene, "Resume steps its state", |s| s.property("pageModel", "actions") != now);
        assert_eq!(scene.rows_told("pageModel"), told, "the tiles the mark moved between, by runs");
    }
    assert_eq!(scene.property("pageModel", "actions"), before, "back on Slot 3");
    // States' tiles scrolled and Options' rows at once, three times: the tiles still being made as the section turns, and
    // Qt logs nothing (EmuSen_Kit.md §7.58).
    scene.pad(UiButton::Down);
    for _ in 0..3 {
        for _ in 0..6 {
            scene.pad(UiButton::Right);
        }
        scene.pad(UiButton::PageDown);
        scene.wait(16);
        scene.pad(UiButton::PageUp);
        scene.wait(16);
    }
    scene.wait(300);
    scene.assert_quiet();
    section(&mut scene, 1);
    scene.pad(UiButton::Up);
    tiles_hold_focus(&mut scene, false);
    // X on Slot 1 asks first, No focused: A keeps the state; X again, Down to Delete, A deletes it, its picture and its record.
    let slot1 = platform.save_path(SAVE_STATE, rom.to_str().unwrap(), None, 1).unwrap();
    scene.pad(UiButton::Down);
    tiles_hold_focus(&mut scene, true);
    move_to(&mut scene, UiButton::Left, "Slot 1");
    wait_for(&mut scene, "pageModel", "help", &help(emusen_qt_models::help::Screen::GamePageLoad), 1000);
    // The help bar's actions follow the help screen a frame later: X is Delete… once it shows.
    delete_declared(&mut scene);
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "pageModel", "asking", "true", 1000);
    assert_eq!(scene.property("pageModel", "questionTitle"), "Delete Slot 1?");
    answers_focused(&mut scene);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "pageModel", "asking", "false", 1000);
    assert!(slot1.exists(), "No keeps the state");
    tiles_focused_at(&mut scene, 0);
    delete_declared(&mut scene);
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "pageModel", "asking", "true", 1000);
    answers_focused(&mut scene);
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "pageModel", "asking", "false", 1000);
    for _ in 0..40 {
        if !slot1.exists() {
            break;
        }
        scene.wait(50);
    }
    assert!(!slot1.exists(), "Delete takes the state away");
    // The focused state deleted, the focus is on the first tile, as Mistress's page put it (EmuSen_Qt.md §7.8.1); then
    // Slot 3, two tiles on, where a list that kept its index would leave the focus on the emptied slot.
    tiles_focused_at(&mut scene, 0);
    let slot3 = platform.save_path(SAVE_STATE, rom.to_str().unwrap(), None, 3).unwrap();
    move_to(&mut scene, UiButton::Right, "Slot 3");
    assert_eq!(scene.property("pageTiles", "currentIndex"), "2");
    wait_for(&mut scene, "pageModel", "help", &help(emusen_qt_models::help::Screen::GamePageLoad), 1000);
    delete_declared(&mut scene);
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "pageModel", "asking", "true", 1000);
    assert_eq!(scene.property("pageModel", "questionTitle"), "Delete Slot 3?");
    answers_focused(&mut scene);
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "pageModel", "asking", "false", 1000);
    for _ in 0..40 {
        if !slot3.exists() {
            break;
        }
        scene.wait(50);
    }
    assert!(!slot3.exists(), "Delete takes Slot 3 away");
    tiles_focused_at(&mut scene, 0);
    // The tiles' list told the page its focus inside the page's resets; the page kept it until each ended (EmuSen_Qt.md §7.8.1).
    let told = scene.rows_told("pageModel");
    assert!(told.contains("reset"), "the sections turned and the deletions reset the tiles: {told}");
    assert!(!told.contains(":help"), "no help changed inside a reset: {told}");
    scene.pad(UiButton::Up);
    section(&mut scene, 2);
    assert_eq!(scene.property("pageModel", "shows"), "2");
    hold(&mut scene, "GamePageOptions", &mockup::Mockup::default(), nothing_named, &mut report);
    // The television standard, offered for a television's console: the game's own, stepped to NTSC and saved in games.db,
    // then back; the package's state in Mistress's words.
    assert!(comes_to_read(&mut scene, "Not a package"), "the package's state");
    down_to(&mut scene, "Television standard");
    assert!(comes_to_read(&mut scene, "The game's"), "the game's own standard");
    let standard = |platform: &std::sync::Arc<Platform>| {
        let games = Library::open(platform, &platform.file(FILE_GAMES).unwrap(), &sqlite).unwrap();
        games.get(&serde_json::json!({ "what": "standard", "path": rom.to_str().unwrap() })).unwrap()["value"].clone()
    };
    scene.pad(UiButton::Right);
    wait_for(&mut scene, "pageModel", "ready", "true", 1000);
    for _ in 0..40 {
        if reads(&mut scene, "NTSC") {
            break;
        }
        scene.wait(50);
    }
    assert!(reads(&mut scene, "NTSC"), "the row reads the standard saved");
    assert_eq!(standard(&platform), "ntsc");
    scene.pad(UiButton::Left);
    for _ in 0..40 {
        if reads(&mut scene, "The game's") {
            break;
        }
        scene.wait(50);
    }
    assert_eq!(standard(&platform), serde_json::Value::Null, "the game's own again");
    // Right then Left at once, before the first save's read is back: each steps from the one before, so the game's own stays.
    // Each step is saved and shown in place as it is pressed; of the two read-backs the older is dropped and the newer
    // reads what is shown, so nothing is told after the presses return (EmuSen_Qt.md §7.8.1).
    scene.rows_told("pageModel");
    scene.pad(UiButton::Right);
    scene.pad(UiButton::Left);
    assert_eq!(scene.rows_told("pageModel"), "2-2,2-2", "each step told in place on the standard's row as it was pressed");
    assert!(comes_to_read(&mut scene, "The game's"), "two steps back to back come back to the game's own");
    assert_eq!(standard(&platform), serde_json::Value::Null, "and the game's own is what is saved");
    section(&mut scene, 3);
    hold(&mut scene, "GamePageMedia", &mockup::Mockup::default(), nothing_named, &mut report);
    // Media's tiles and Details' rows at once, three times, the tiles scrolled: a section's items come while the last
    // one's are still being made, and Qt logs nothing.
    for _ in 0..3 {
        scene.pad(UiButton::Down);
        for _ in 0..6 {
            scene.pad(UiButton::Right);
        }
        scene.pad(UiButton::PageDown);
        scene.wait(16);
        scene.pad(UiButton::PageUp);
        scene.wait(16);
    }
    scene.wait(300);
    scene.pad(UiButton::Up);
    section(&mut scene, 4);
    hold(&mut scene, "GamePageDetails", &mockup::Mockup::default(), nothing_named, &mut report);
    // Open Folder: the program's one folder action, given the folder holding the game's file, caught as its URL.
    assert!(scene.set_property("folders", "catching", "true"));
    down_to(&mut scene, "Open Folder");
    scene.pad(UiButton::Accept);
    let url = emusen_qt_models::folders::url_of(rom.parent().unwrap().to_str().unwrap());
    wait_for(&mut scene, "folders", "caught", &url, 1000);

    // Y turns the heart on, saved in games.db.
    let hearted = scene_actions_with(&mut scene, "♥");
    scene.pad(UiButton::Search);
    wait_for(&mut scene, "pageModel", "actions", &hearted, 2000);

    // B returns to the grid, the focus on the same game.
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);

    // The player's own screenshot, named as the program names one in Galaxia's folder, leads Media.
    let shots = platform.directory(DIR_SCREENSHOTS).unwrap().unwrap();
    std::fs::create_dir_all(&shots).unwrap();
    std::fs::copy(&pictures[0].0, shots.join(emusen_qt_models::screenshots::name(rom.to_str().unwrap(), "2026-11-09", "14.03.11.251"))).unwrap();
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "screens", "shown", "Page", 2000);
    wait_for(&mut scene, "pageModel", "ready", "true", 10000);
    // Left on Favourite, the page opens again on Play, as Mistress's does.
    move_to(&mut scene, UiButton::Right, "Favourite");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "screens", "shown", "Page", 2000);
    scene.wait(300);
    let tree = scene.tree();
    let focused = scene.focus(&tree).map(|i| tree.nodes[i as usize].heard.clone());
    assert_eq!(focused.as_deref(), Some("Play"), "the page opens on Play");
    section(&mut scene, 3);
    assert!(comes_to_read(&mut scene, "Screenshot 1"), "the player's screenshot in Media");
    // The shoulder to Details with the focus on a tile: the focus goes on to Details' first row, which is shown.
    scene.pad(UiButton::Down);
    move_to(&mut scene, UiButton::Left, "Screenshot 1");
    scene.pad(UiButton::PageDown);
    wait_for(&mut scene, "pageModel", "section", "4", 1000);
    scene.wait(200);
    let tree = scene.tree();
    let focused = scene.focus(&tree).map(|i| (tree.nodes[i as usize].heard.clone(), tree.nodes[i as usize].visible));
    assert_eq!(focused, Some(("Scrape This Game".to_string(), true)), "the focus on the list shown");
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
    assert!(scene.property("pageClip", "frames").parse::<i32>().unwrap_or(0) > 0, "the frames drawn by the kit's sink");
    // The fade over the still done.
    scene.wait(1200);
    let clip = hero_colour(&mut scene);
    assert!(clip.r > 150 && clip.b > 150 && clip.g < 110, "the clip's magenta over the still: {clip:?}");

    // B: the page loses the focus, the clip stops and its player goes.
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);
    wait_for(&mut scene, "pageClip", "running", "false", 2000);

    // The other game has no clip: its page keeps its still, and nothing plays.
    open_page(&mut scene, 1);
    assert_eq!(scene.property("pageModel", "title"), LANTERN);
    assert_eq!(scene.property("pageModel", "clip"), "");
    scene.wait(2500);
    assert_eq!(scene.property("pageClip", "running"), "false", "no player made for a game with no clip");
    assert_eq!(scene.property("pageClip", "frames"), "0");
    // Details ▸ Hide from Library asks first; Hide saves the edit and the page closes to the grid.
    section(&mut scene, 4);
    for _ in 0..5 {
        scene.pad(UiButton::Down);
        scene.wait(50);
    }
    let tree = scene.tree();
    let focus = scene.focus(&tree).map(|i| tree.nodes[i as usize].heard.clone()).unwrap_or_default();
    assert_eq!(focus, "Hide from Library", "Down reaches Details' last row");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "pageModel", "asking", "true", 1000);
    assert_eq!(scene.property("pageModel", "questionTitle"), format!("Hide {LANTERN}?"));
    answers_focused(&mut scene);
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);
    let games = Library::open(&platform, &platform.file(FILE_GAMES).unwrap(), &sqlite).unwrap();
    let lantern = world.root.join("Roms/SNES").join(format!("{LANTERN}.sfc"));
    let edits = games.get(&serde_json::json!({ "what": "edits", "path": lantern.to_str().unwrap() })).unwrap();
    assert_eq!(edits["hidden"], "1", "hidden in games.db: {edits}");
    // Hidden, the game leaves the grid, as it leaves Mistress's every view.
    wait_for(&mut scene, "gridGames", "count", "1", 5000);

    // A state chosen on the page is the one the game starts from: the session is named it as the game is started.
    open_page(&mut scene, 0);
    assert_eq!(scene.property("pageModel", "title"), COBALT);
    section(&mut scene, 1);
    scene.pad(UiButton::Down);
    scene.wait(200);
    move_to(&mut scene, UiButton::Left, "Slot 1");
    move_to(&mut scene, UiButton::Right, "Slot 2");
    wait_for(&mut scene, "pageModel", "help", &help(emusen_qt_models::help::Screen::GamePageLoad), 1000);
    scene.pad(UiButton::Accept);
    let slot2 = platform.save_path(SAVE_STATE, rom.to_str().unwrap(), None, 2).unwrap();
    for _ in 0..40 {
        if scene.property("session", "startState") == slot2.to_str().unwrap() {
            break;
        }
        scene.wait(50);
    }
    let said = (scene.property("pageModel", "problem"), scene.property("screens", "shown"), scene.property("launch", "asking"));
    assert_eq!(scene.property("session", "startState"), slot2.to_str().unwrap(), "the state named for the start: {said:?}");
    scene.assert_quiet();
    assert!(report.is_empty(), "{}", report.join("\n"));
}

/// The actions' text with the heart's mark in place of the one shown.
fn scene_actions_with(scene: &mut Scene<'_>, heart: &str) -> String {
    scene.property("pageModel", "actions").replace("\"♡\"", &format!("\"{heart}\""))
}
