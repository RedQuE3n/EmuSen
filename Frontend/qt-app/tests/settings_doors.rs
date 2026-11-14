//! Settings' doors to what exists and the settings the program now reads, through the program on the software backend:
//! Library ▸ Compressed discs opens the Compressed Discs tool and B comes back to its row; Library ▸ Cheats opens the
//! cheats of the game last played on its page, and is held while no game played has any; the List card's Collections
//! opens the Filter page, whose collection narrows the grid to its members; Sound volume is the volume the ES-DE theme's
//! navigation sounds are played at, from the next sound after a step.

mod common;

use common::console::focused_words;
use common::{sandbox, wait_for};
use emusen_client_fixture::{library, World};
use emusen_crystal_client::{Api, Config, Runtime};
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::{DIR_THEMES, MODEL_APP_SETTINGS, MODEL_CHEAT_FILE};
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt::MAIN_URL;
use emusen_qt_test::Scene;
use serde_json::json;

const GAME: &str = "Cobalt Harbor (Synthetic)";

/// Waits up to `seconds` for a condition the scene answers, letting Qt run between looks.
#[track_caller]
fn until(scene: &mut Scene<'_>, seconds: u64, what: &str, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    let start = std::time::Instant::now();
    while !done(scene) {
        assert!(start.elapsed() < std::time::Duration::from_secs(seconds), "{what}: not within {seconds} s");
        scene.wait(16);
    }
}

fn reads(scene: &mut Scene<'_>, words: &str) -> bool {
    scene.tree().nodes.iter().any(|n| n.visible && n.text.as_deref() == Some(words))
}

/// The program on the library, Home's first place focused.
fn program(harness: &mut emusen_qt_test::Harness) -> Scene<'_> {
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "settingsModel", "ready", "true", 5000);
    scene.wait(300);
    scene
}

/// Settings from the Home card's row, whatever Home's shelf holds, then the section by its name through the right
/// shoulder.
fn settings_section(scene: &mut Scene<'_>, section: &str) {
    open_settings(scene);
    let names: Vec<String> = ["Controllers", "Display", "Sound", "Consoles", "Library", "EmuSen"].map(str::to_owned).to_vec();
    let at = names.iter().position(|n| n == section).unwrap();
    while scene.property("settingsModel", "section") != at.to_string() {
        scene.pad(UiButton::PageDown);
        scene.wait(30);
    }
    wait_for(scene, "settingsModel", "focused", "0", 1000);
}

/// Start, then the Home card's Settings row.
fn open_settings(scene: &mut Scene<'_>) {
    scene.pad(UiButton::Menu);
    wait_for(scene, "screens", "shown", "HomeCard", 2000);
    scene.wait(300);
    for _ in 0..12 {
        if focused_words(scene).starts_with("Settings") {
            break;
        }
        scene.pad(UiButton::Down);
    }
    scene.pad(UiButton::Accept);
    wait_for(scene, "screens", "shown", "Settings", 2000);
}

/// Down until the focused row's sentence begins with `sentence`; the row's index.
#[track_caller]
fn walk_to(scene: &mut Scene<'_>, sentence: &str) -> String {
    for _ in 0..20 {
        if scene.property("settingsModel", "sentence").starts_with(sentence) {
            return scene.property("settingsModel", "focused");
        }
        scene.pad(UiButton::Down);
        scene.wait(30);
    }
    panic!("no row says {sentence}");
}

/// The game in the sandbox's SNES folder, with a cheat list of two, recorded as the game last played.
fn played_with_cheats(world: &World) {
    let platform = Platform::load(library()).unwrap();
    let rom = world.root.join("Roms/SNES").join(format!("{GAME}.sfc"));
    let rt = Runtime::new(&Api::load(library()).unwrap(), &Config::default()).unwrap();
    let console = rt.systems().unwrap().into_iter().find(|s| s.id == "snes").expect("the SNES").console;
    drop(rt);
    let (name, category) = platform.cheat_game_list(Some(&console), rom.to_str().unwrap()).unwrap().expect("a list for the game");
    let cheat = |description: &str| json!({ "Kind": "RamPoke", "Description": description, "Enabled": false,
        "Writes": [{ "Space": "WRAM", "Address": "7E0DBE", "Value": "09" }] });
    platform.model_save(MODEL_CHEAT_FILE, Some(&category), &format!("{name}.json"), &json!({ "Cheats": [cheat("Infinite lives"), cheat("Moon jump")] })).unwrap();
    records().change(&json!({ "op": "started", "path": rom.to_str().unwrap(), "at": "2026-11-13T10:00:00Z" })).unwrap();
}

/// games.db where Galaxia names it, as the program opens it.
fn records() -> Library {
    let platform = Platform::load(library()).unwrap();
    let sqlite = Lent::open("libsqlite3.so.0").unwrap();
    Library::open(&platform, &platform.file(emusen_platform_client::galaxia::consts::FILE_GAMES).unwrap(), &sqlite).unwrap()
}

fn compressed_discs_and_cheats_open_from_settings_and_b_returns_to_the_row(harness: &mut emusen_qt_test::Harness) {
    let world = sandbox("qt-app-settings-doors", &[("SNES", &format!("{GAME}.sfc")), ("SNES", "Aurora Drift (Synthetic).sfc")]);
    played_with_cheats(&world);
    let mut scene = program(harness);
    assert_eq!(scene.property("library", "lastPlayed"), world.root.join("Roms/SNES").join(format!("{GAME}.sfc")).to_str().unwrap());
    settings_section(&mut scene, "Library");

    // Cheats: the game last played's page, opened at Options' Cheats, its switches shown; B back to Options, then Settings.
    let cheats = walk_to(&mut scene, "The cheats of the game last played");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Page", 2000);
    wait_for(&mut scene, "pageModel", "ready", "true", 10000);
    assert_eq!(scene.property("pageModel", "title"), GAME, "the game last played, not the one in hand");
    assert_eq!(scene.property("pageModel", "section"), "2", "Options");
    until(&mut scene, 5, "the cheats listed", |s| reads(s, "Infinite lives") && reads(s, "Moon jump"));
    scene.pad(UiButton::Back);
    until(&mut scene, 5, "Options with its Cheats row", |s| reads(s, "0 of 2 on"));
    assert_eq!(scene.property("screens", "shown"), "Page");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Settings", 2000);
    wait_for(&mut scene, "settingsModel", "focused", &cheats, 1000);

    // Compressed discs: the tool, on its first job; B back to the same row.
    let discs = walk_to(&mut scene, "The library's discs converted to CHD");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Tool", 2000);
    wait_for(&mut scene, "jobsModel", "title", "Compressed Discs", 2000);
    scene.wait(200);
    assert!(focused_words(&mut scene).starts_with("List disc games"), "{}", focused_words(&mut scene));
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Settings", 2000);
    wait_for(&mut scene, "settingsModel", "focused", &discs, 1000);
    assert_eq!(scene.property("settingsModel", "section"), "4", "Library still");
    scene.assert_quiet();
}

fn cheats_is_held_while_no_game_was_played(harness: &mut emusen_qt_test::Harness) {
    let _world = sandbox("qt-app-settings-doors-unplayed", &[("SNES", &format!("{GAME}.sfc"))]);
    let mut scene = program(harness);
    assert_eq!(scene.property("library", "lastPlayed"), "");
    settings_section(&mut scene, "Library");
    let cheats = walk_to(&mut scene, "The cheats of the game last played");
    scene.pad(UiButton::Accept);
    scene.wait(300);
    assert_eq!(scene.property("screens", "shown"), "Settings", "a held row opens nothing");
    assert_eq!(scene.property("settingsModel", "focused"), cheats);
    scene.assert_quiet();
}

fn the_list_cards_collections_narrow_the_grid_to_a_collection(harness: &mut emusen_qt_test::Harness) {
    let world = sandbox("qt-app-settings-doors-collections", &[("SNES", "Aurora Drift (Synthetic).sfc"), ("SNES", "Brass Lantern (Synthetic).sfc"), ("NES", "Fable of Tiles (Synthetic).nes")]);
    let games = records();
    let id = games.change(&json!({ "op": "create_collection", "name": "Couch", "at": "2026-11-13T10:00:00Z" })).unwrap();
    let brass = world.root.join("Roms/SNES/Brass Lantern (Synthetic).sfc");
    games.change(&json!({ "op": "add_to_collection", "id": id, "path": brass.to_str().unwrap() })).unwrap();
    drop(games);
    let mut scene = program(harness);

    // Home's Games, Choose a console's All Games, the grid of three.
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Consoles", 1000);
    let tree = scene.tree();
    let all = tree.find_named("tile.0").expect("All Games");
    assert!(scene.focus_item(&tree, all));
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);
    wait_for(&mut scene, "gridGames", "countLine", "3 games", 2000);

    // Select ▸ Collections opens the Filter page, the collection after Favourites; A on it narrows the grid.
    scene.pad(UiButton::Options);
    wait_for(&mut scene, "screens", "shown", "List", 1000);
    scene.wait(300);
    for _ in 0..12 {
        if focused_words(&mut scene).starts_with("Collections") {
            break;
        }
        scene.pad(UiButton::Down);
    }
    scene.pad(UiButton::Accept);
    until(&mut scene, 2, "the Filter page", |s| reads(s, "Every game") && reads(s, "Couch"));
    for _ in 0..4 {
        if focused_words(&mut scene).starts_with("Couch") {
            break;
        }
        scene.pad(UiButton::Down);
    }
    assert!(focused_words(&mut scene).starts_with("Couch"), "{}", focused_words(&mut scene));
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);
    wait_for(&mut scene, "gridGames", "countLine", "1 of 3 games", 2000);
    assert_eq!(scene.property("gridGames", "count"), "1");
    until(&mut scene, 2, "Brass Lantern's cover alone", |s| reads(s, "Brass Lantern (Synthetic)") && !reads(s, "Aurora Drift (Synthetic)"));
    let facts: serde_json::Value = serde_json::from_str(&scene.property("gridGames", "listFacts")).unwrap();
    assert_eq!(facts["filter"], "Couch", "the List card's Filter names it");
    // B clears it first, as it clears Favourites.
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "gridGames", "countLine", "3 games", 2000);
    assert_eq!(scene.property("screens", "shown"), "Grid");
    scene.assert_quiet();
}

const THEME: &str = "<theme><view name=\"system\"><carousel name=\"systems\"><pos>0 0.3</pos><size>1 0.4</size>\
    <text>${system.fullName}</text><textColor>FFFFFF</textColor></carousel></view>\
    <view name=\"gamelist\"><textlist name=\"games\"><pos>0.05 0.1</pos><size>0.4 0.8</size><primaryColor>FFFFFF</primaryColor></textlist></view></theme>";

fn the_navigation_sounds_play_at_sound_volume_changed_at_once_by_a_step(harness: &mut emusen_qt_test::Harness) {
    let _world = sandbox("qt-app-settings-doors-volume", &[("NES", "One.nes")]);
    let platform = Platform::load(library()).unwrap();
    let pane = platform.directory(DIR_THEMES).unwrap().unwrap().join("pane");
    std::fs::create_dir_all(&pane).unwrap();
    std::fs::write(pane.join("theme.xml"), THEME).unwrap();
    let mut settings = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    settings["LibraryStyle"] = json!("Theme");
    settings["BigPictureTheme"] = json!(pane.to_str().unwrap());
    settings["BigPictureInterface"]["NavigationVolume"] = json!(40);
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    let mut scene = program(harness);
    until(&mut scene, 20, "the library drawn by the theme", |s| s.property("library", "themed") == "true");
    until(&mut scene, 10, "the player at the stored volume", |s| s.property("cues", "heldVolume") == "0.4");
    assert_eq!(scene.property("cues", "problem"), "");

    // Start, the Home card's Settings, Sound ▸ Sound volume: Right steps it, and the player holds it at once.
    open_settings(&mut scene);
    while scene.property("settingsModel", "section") != "2" {
        scene.pad(UiButton::PageDown);
        scene.wait(30);
    }
    walk_to(&mut scene, "How loud the navigation sounds are");
    scene.pad(UiButton::Right);
    until(&mut scene, 2, "the step held by the player", |s| s.property("cues", "heldVolume") == "0.45");
    let stored = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    assert_eq!(stored["BigPictureInterface"]["NavigationVolume"], json!(45), "the step is saved");
    scene.pad(UiButton::Left);
    scene.pad(UiButton::Left);
    until(&mut scene, 2, "two steps down", |s| s.property("cues", "heldVolume") == "0.35");
}

/// One harness for the four, as a process holds one Qt application; each on its own scratch tree.
#[test]
fn settings_doors_and_the_settings_the_program_now_reads() {
    let mut harness = emusen_qt_test::Harness::new();
    compressed_discs_and_cheats_open_from_settings_and_b_returns_to_the_row(&mut harness);
    cheats_is_held_while_no_game_was_played(&mut harness);
    the_list_cards_collections_narrow_the_grid_to_a_collection(&mut harness);
    the_navigation_sounds_play_at_sound_volume_changed_at_once_by_a_step(&mut harness);
}
