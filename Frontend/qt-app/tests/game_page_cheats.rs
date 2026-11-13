//! A game's cheats from its page in the program, the game not running: Options' Cheats row counts the cheats on in the
//! game's own list and opens them, each a switch; A turns one, which is saved to the list at once, and B goes back to
//! Options with the count turned.

mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_crystal_client::{Api, Config, Runtime};
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::MODEL_CHEAT_FILE;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_test::Scene;
use serde_json::json;

const GAME: &str = "Cobalt Harbor (Synthetic)";

fn until(scene: &mut Scene<'_>, what: &str, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    for _ in 0..100 {
        if done(scene) {
            return;
        }
        scene.wait(50);
    }
    panic!("{what} within 5 s");
}

fn reads(scene: &mut Scene<'_>, words: &str) -> bool {
    scene.tree().nodes.iter().any(|n| n.visible && n.text.as_deref() == Some(words))
}

/// What a reader hears of the item holding the focus.
fn heard(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].heard.clone()).unwrap_or_default()
}

#[test]
fn the_game_page_opens_the_games_cheats_and_a_switch_is_saved_to_its_list() {
    let world = sandbox("qt-app-game-page-cheats", &[("SNES", &format!("{GAME}.sfc"))]);
    let platform = Platform::load(library()).unwrap();
    let rom = world.root.join("Roms/SNES").join(format!("{GAME}.sfc"));
    let rt = Runtime::new(&Api::load(library()).unwrap(), &Config::default()).unwrap();
    let console = rt.systems().unwrap().into_iter().find(|s| s.id == "snes").expect("the SNES").console;
    drop(rt);
    let list = platform.cheat_game_list(Some(&console), rom.to_str().unwrap()).unwrap().expect("a list for the game");
    let file = format!("{}.json", list.0);
    let cheat = |description: &str, on: bool| json!({ "Kind": "RamPoke", "Description": description, "Enabled": on,
        "Writes": [{ "Space": "WRAM", "Address": "7E0DBE", "Value": "09" }] });
    platform.model_save(MODEL_CHEAT_FILE, Some(&list.1), &file, &json!({ "Cheats": [cheat("Infinite lives", true), cheat("Moon jump", false)] })).unwrap();
    let saved = |n: usize| platform.model_load(MODEL_CHEAT_FILE, Some(&list.1), &file).unwrap().unwrap()["Cheats"][n]["Enabled"].as_bool();

    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    scene.wait(300);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Consoles", 1000);
    let tree = scene.tree();
    let all = tree.find_named("tile.0").expect("All Games");
    assert!(scene.focus_item(&tree, all));
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Grid", 1000);
    scene.wait(300);
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "screens", "shown", "Page", 2000);
    wait_for(&mut scene, "pageModel", "ready", "true", 10000);
    assert_eq!(scene.property("pageModel", "title"), GAME);
    for _ in 0..2 {
        let now = scene.property("pageModel", "section");
        scene.pad(UiButton::PageDown);
        until(&mut scene, "the next section", |s| s.property("pageModel", "section") != now);
    }
    assert_eq!(scene.property("pageModel", "section"), "2", "Options");
    until(&mut scene, "the Cheats row", |s| reads(s, "1 of 2 on"));
    for _ in 0..6 {
        if heard(&mut scene).starts_with("Cheats") {
            break;
        }
        scene.pad(UiButton::Down);
        scene.wait(80);
    }
    assert!(heard(&mut scene).starts_with("Cheats"), "the focus on Cheats: {}", heard(&mut scene));

    // A opens the game's cheats, each a switch; A on the second turns it on, saved to the list at once.
    scene.pad(UiButton::Accept);
    until(&mut scene, "the cheats listed", |s| reads(s, "Infinite lives") && reads(s, "Moon jump"));
    assert!(!reads(&mut scene, "Television standard"), "Cheats' page in place of Options' rows");
    for _ in 0..4 {
        if heard(&mut scene).starts_with("Moon jump") {
            break;
        }
        scene.pad(UiButton::Down);
        scene.wait(80);
    }
    assert!(heard(&mut scene).starts_with("Moon jump"), "the focus on the second cheat: {}", heard(&mut scene));
    scene.pad(UiButton::Accept);
    until(&mut scene, "the switch saved", |_| saved(1) == Some(true));
    until(&mut scene, "both shown on", |s| s.tree().nodes.iter().filter(|n| n.visible && n.text.as_deref() == Some("On")).count() == 2);
    // Left turns the first off, saved too.
    scene.pad(UiButton::Up);
    scene.wait(80);
    scene.pad(UiButton::Left);
    until(&mut scene, "the first switched off", |_| saved(0) == Some(false));

    // B goes back to Options, the count as the list now holds it.
    scene.pad(UiButton::Back);
    until(&mut scene, "Options again", |s| reads(s, "1 of 2 on") && !reads(s, "Moon jump"));
    assert_eq!(scene.property("screens", "shown"), "Page", "B left Cheats, not the page");
    scene.assert_quiet();
}
