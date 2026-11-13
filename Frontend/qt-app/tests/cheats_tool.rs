//! The Cheats tool in the program, over the platform library built from this commit and a scratch library: the pad opens
//! it from the Home card's Tools page; with no database it says so and lists nothing. With a database in the folder the
//! base reads one from (its `.cht` files written there by the test, as an install leaves them: no install call is made),
//! it lists the library's game the database has a file for under its console, not the game whose console has no file
//! of its name; the game's row reads "No list yet" until the game has its own list, then how many of its cheats are on,
//! and X lists them.

mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_crystal_client::consts::CHEAT_DB_STATUS;
use emusen_crystal_client::service_flags::NO_GAME;
use emusen_crystal_client::{Api, Config, Runtime};
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::MODEL_CHEAT_FILE;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_test::Scene;
use serde_json::json;
use std::path::PathBuf;

const GAME: &str = "Cobalt Harbor (Synthetic)";
const SNES: &str = "Nintendo - Super Nintendo Entertainment System";

fn focused_words(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    let Some(focus) = scene.focus(&tree) else { return String::new() };
    tree.nodes.iter().skip(focus as usize).find_map(|n| n.text.clone()).unwrap_or_default()
}

fn focused(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default()
}

/// `done` within `ms`, else the test fails saying `what`.
fn until(scene: &mut Scene<'_>, what: &str, ms: i32, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    let mut waited = 0;
    while !done(scene) && waited < ms {
        scene.wait(25);
        waited += 25;
    }
    assert!(done(scene), "{what} within {ms} ms: rows {:?}, summary {:?}", scene.property("jobsModel", "rows"), scene.property("jobsModel", "summary"));
}

/// The Home card's Tools page's Cheats row, from Home, by the pad; the tool's read ended.
fn open_cheats(scene: &mut Scene<'_>) {
    scene.pad(UiButton::Menu);
    wait_for(scene, "screens", "shown", "HomeCard", 2000);
    scene.wait(300);
    for row in ["Tools", "Cheats"] {
        for _ in 0..12 {
            if focused_words(scene).starts_with(row) {
                break;
            }
            scene.pad(UiButton::Down);
            scene.wait(50);
        }
        assert!(focused_words(scene).starts_with(row), "no row reads {row}: {}", focused_words(scene));
        scene.pad(UiButton::Accept);
        scene.wait(300);
    }
    wait_for(scene, "screens", "shown", "Tool", 2000);
    wait_for(scene, "jobsModel", "title", "Cheats", 2000);
    wait_for(scene, "jobsModel", "busy", "false", 10000);
}

/// B on the tool screen: Home again, holding the pad.
fn back_home(scene: &mut Scene<'_>) {
    scene.pad(UiButton::Back);
    wait_for(scene, "screens", "shown", "Home", 2000);
    until(scene, "Home holds the pad", 2000, |s| focused(s).starts_with("place."));
}

/// The folder the base reads the player's database from, as its status says it.
fn database_folder() -> PathBuf {
    let rt = Runtime::new(&Api::load(library()).unwrap(), &Config::default()).unwrap();
    let status = rt.session_service(0, NO_GAME).unwrap().cheat_op(0, CHEAT_DB_STATUS, &json!({})).unwrap();
    PathBuf::from(status["directory"].as_str().expect("the database's folder"))
}

#[test]
fn the_cheats_tool_lists_the_librarys_games_the_database_has_and_each_games_own_list() {
    let world = sandbox("qt-app-cheats-tool", &[("SNES", &format!("{GAME}.sfc")), ("NES", "Alpha.nes")]);
    let folder = database_folder();
    assert!(folder.starts_with(&world.root), "the database folder is the scratch tree's: {}", folder.display());
    assert!(!folder.exists(), "no database before the test writes one: {}", folder.display());

    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    until(&mut scene, "Home holds the pad", 2000, |s| focused(s) == "place.0");

    // No database: the tool says where it would be and lists nothing.
    open_cheats(&mut scene);
    let summary = scene.property("jobsModel", "summary");
    assert_eq!(summary, format!("No cheat database at {}", folder.display()));
    assert!(scene.property("jobsModel", "facts").contains("no database"), "{}", scene.property("jobsModel", "facts"));
    assert_eq!(scene.property("jobsModel", "rows"), "[]");
    back_home(&mut scene);

    // A database as an install leaves it: .cht files in system folders. The SNES game has one of its name; Alpha's
    // name is only under the SNES, which is not its console; Zeta is no game of the library.
    let cht = "cheats = 1\n\ncheat0_desc = \"Infinite lives\"\ncheat0_code = \"7E0DBE09\"\ncheat0_enable = false\n";
    for (system, game) in [(SNES, GAME), (SNES, "Alpha"), ("Sega - Mega Drive - Genesis", "Zeta")] {
        std::fs::create_dir_all(folder.join(system)).unwrap();
        std::fs::write(folder.join(system).join(format!("{game}.cht")), cht).unwrap();
    }
    open_cheats(&mut scene);
    assert_eq!(scene.property("jobsModel", "summary"), format!("3 cheat file(s) in {}", folder.display()));
    let facts = scene.property("jobsModel", "facts");
    assert!(facts.contains("3 files") && facts.contains("newest file "), "{facts}");
    let rows = scene.property("jobsModel", "rows");
    assert!(rows.contains(GAME) && rows.contains("No list yet"), "the SNES game is listed with no list of its own: {rows}");
    assert!(!rows.contains("Alpha") && !rows.contains("Zeta"), "only the library's games, each under its own console: {rows}");
    let snes = Runtime::new(&Api::load(library()).unwrap(), &Config::default()).unwrap().systems().unwrap().into_iter().find(|s| s.id == "snes").expect("the SNES");
    let name = if snes.display_name.is_empty() { snes.name } else { snes.display_name };
    let tree = scene.tree();
    assert!(tree.nodes.iter().any(|n| n.visible && n.text.as_deref() == Some(name.as_str())), "the console {name} is a section of the strip");
    let sentences = scene.property("jobsModel", "sentences");
    assert!(sentences.contains(&format!("{SNES}: {GAME}")), "the pane names the game's file: {sentences}");
    back_home(&mut scene);

    // The game's own list, the file the Pause Card and the game page switch: the row counts the cheats on, X lists them.
    let platform = Platform::load(library()).unwrap();
    let rom = world.root.join("Roms/SNES").join(format!("{GAME}.sfc"));
    let rt = Runtime::new(&Api::load(library()).unwrap(), &Config::default()).unwrap();
    let console = rt.systems().unwrap().into_iter().find(|s| s.id == "snes").expect("the SNES").console;
    let list = platform.cheat_game_list(Some(&console), rom.to_str().unwrap()).unwrap().expect("a list for the game");
    let cheat = |description: &str, on: bool| json!({ "Kind": "RamPoke", "Description": description, "Enabled": on,
        "Writes": [{ "Space": "WRAM", "Address": "7E0DBE", "Value": "09" }] });
    platform.model_save(MODEL_CHEAT_FILE, Some(&list.1), &format!("{}.json", list.0), &json!({ "Cheats": [cheat("Infinite lives", true), cheat("Moon jump", false)] })).unwrap();
    open_cheats(&mut scene);
    let rows = scene.property("jobsModel", "rows");
    assert!(rows.contains("1 of 2 on") && !rows.contains("No list yet"), "the row counts the list's cheats on: {rows}");
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "jobsModel", "cardOpen", "true", 1000);
    let lines = scene.property("jobsModel", "cardRows");
    assert!(lines.contains("Infinite lives") && lines.contains("Moon jump"), "X lists the game's cheats: {lines}");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "jobsModel", "cardOpen", "false", 1000);
    back_home(&mut scene);
    drop(world);
}
