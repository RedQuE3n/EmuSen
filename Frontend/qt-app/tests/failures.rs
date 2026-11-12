//! What fails is said where the player is (EmuSen_Qt.md §7.9), on the software backend: each failure is made to happen in
//! the program and its words are found on the notice layer. A settings save, a hide and a state's deletion each fail
//! twice and are said twice; Cores that cannot be opened and a library that cannot be read are said once.

mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::{FILE_GAMES, SAVE_STATE};
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_test::Scene;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

const COBALT: &str = "Cobalt Harbor (Synthetic)";
const LANTERN: &str = "Lantern Bay (Synthetic)";

/// The notice layer showing words that begin `start`, within five seconds; the words.
fn said(scene: &mut Scene<'_>, start: &str) -> String {
    for _ in 0..100 {
        if scene.property("notices", "shown") == "true" && scene.property("notices", "words").starts_with(start) {
            return scene.property("notices", "words");
        }
        scene.wait(50);
    }
    panic!("never said \"{start}…\"; the notice layer reads \"{}\" (shown {})", scene.property("notices", "words"), scene.property("notices", "shown"));
}

/// The notice gone once its hold has run out, so the next one said is seen to be raised again.
fn gone(scene: &mut Scene<'_>) {
    wait_for(scene, "notices", "shown", "false", 6000);
}

fn mode(path: &Path, mode: u32) {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
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

/// The page's question opened with its first answer focused, then Down to its second, and A.
fn answer_second(scene: &mut Scene<'_>) {
    wait_for(scene, "pageModel", "asking", "true", 1000);
    for _ in 0..120 {
        if scene.property("pageQuestion", "opened") == "true" {
            let tree = scene.tree();
            if scene.focus(&tree).is_some_and(|i| tree.nodes[i as usize].name == "answer.0") {
                break;
            }
        }
        scene.wait(25);
    }
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    wait_for(scene, "pageModel", "asking", "false", 1000);
}

/// The page's section by its index, through the right shoulder.
fn section(scene: &mut Scene<'_>, index: i32) {
    for _ in 0..5 {
        if scene.property("pageModel", "section") == index.to_string() {
            break;
        }
        scene.pad(UiButton::PageDown);
        scene.wait(200);
    }
    assert_eq!(scene.property("pageModel", "section"), index.to_string());
}

/// Home's place at `index`, by Right from the first, chosen by A.
fn home_place(scene: &mut Scene<'_>, index: usize) {
    for _ in 0..40 {
        let tree = scene.tree();
        if let Some(first) = tree.find_named("place.0") {
            if scene.focus(&tree) == Some(first) || scene.focus_item(&tree, first) {
                break;
            }
        }
        scene.wait(50);
    }
    for _ in 0..index {
        scene.pad(UiButton::Right);
    }
    scene.pad(UiButton::Accept);
}

#[test]
fn qt_7_9_each_failure_is_said_on_the_notice_layer_and_said_again_when_it_happens_again() {
    let world = sandbox("qt-app-failures", &[("SNES", &format!("{COBALT}.sfc")), ("SNES", &format!("{LANTERN}.sfc"))]);
    let platform = Platform::load(library()).unwrap();
    let rom = world.root.join("Roms/SNES").join(format!("{COBALT}.sfc"));
    let slot1 = platform.save_path(SAVE_STATE, rom.to_str().unwrap(), None, 1).unwrap();
    std::fs::create_dir_all(slot1.parent().unwrap()).unwrap();
    std::fs::write(&slot1, b"state").unwrap();

    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "settingsModel", "ready", "true", 5000);
    // games.db read-only from here: the library keeps the connection it opened, a game's page opens its own, which
    // cannot write.
    let games_db = platform.file(FILE_GAMES).unwrap();
    mode(&games_db, 0o444);

    // A settings save that fails, twice: the folder the settings are written in made read-only. Sound ▸ Volume stepped,
    // and stepped back.
    home_place(&mut scene, emusen_qt_models::screens::SETTINGS_PLACE);
    wait_for(&mut scene, "screens", "shown", "Settings", 2000);
    let config = world.root.join("Config");
    let held: Vec<_> = std::fs::read_dir(&config).unwrap().map(|e| e.unwrap().path()).filter(|p| p.is_file()).collect();
    for file in &held {
        mode(file, 0o444);
    }
    mode(&config, 0o555);
    while scene.property("settingsModel", "section") != "2" {
        scene.pad(UiButton::PageDown);
        scene.wait(30);
    }
    wait_for(&mut scene, "settingsModel", "focused", "0", 1000);
    scene.wait(200);
    scene.pad(UiButton::Right);
    let first = said(&mut scene, "The settings could not be saved: ");
    assert_eq!(scene.property("notices", "attention"), "true");
    gone(&mut scene);
    scene.pad(UiButton::Left);
    assert_eq!(said(&mut scene, "The settings could not be saved: "), first, "the same failure said again");
    gone(&mut scene);
    mode(&config, 0o755);
    for file in &held {
        mode(file, 0o644);
    }
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);

    // Cores that cannot be opened: its source's words.
    assert!(scene.set_property("program", "coresWorld", world.root.join("no-such-world.json").to_str().unwrap()));
    home_place(&mut scene, emusen_qt_models::screens::CORES_PLACE);
    wait_for(&mut scene, "screens", "shown", "Cores", 2000);
    let words = said(&mut scene, world.root.join("no-such-world.json").to_str().unwrap());
    assert_eq!(words, scene.property("coresModel", "problem"));
    gone(&mut scene);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);

    // The grid, then Cobalt Harbor's page.
    home_place(&mut scene, 0);
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
    assert_eq!(scene.property("pageModel", "title"), COBALT);

    // A state's deletion that fails, twice: its folder made read-only, so its file cannot be taken away.
    let saves = slot1.parent().unwrap().to_path_buf();
    mode(&saves, 0o555);
    section(&mut scene, 1);
    scene.pad(UiButton::Down);
    scene.wait(200);
    move_to(&mut scene, UiButton::Left, "Slot 1");
    let mut first = String::new();
    for again in [false, true] {
        scene.wait(300);
        scene.pad(UiButton::Screensaver);
        answer_second(&mut scene);
        let words = said(&mut scene, "The state could not be deleted: ");
        if again {
            assert_eq!(words, first, "the same failure said again");
        }
        first = words;
        gone(&mut scene);
    }
    assert!(slot1.exists(), "the state is kept");
    mode(&saves, 0o755);

    // A hide that fails, twice: games.db cannot be written by the page.
    scene.pad(UiButton::Up);
    section(&mut scene, 4);
    for _ in 0..5 {
        scene.pad(UiButton::Down);
        scene.wait(50);
    }
    let mut first = String::new();
    for again in [false, true] {
        move_to(&mut scene, UiButton::Down, "Hide from Library");
        scene.pad(UiButton::Accept);
        answer_second(&mut scene);
        let words = said(&mut scene, "The game could not be hidden: ");
        if again {
            assert_eq!(words, first, "the same failure said again");
        }
        first = words;
        assert_eq!(scene.property("screens", "shown"), "Page", "the page stays");
        gone(&mut scene);
    }
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);

    // A library that cannot be read: games.db unreadable, and the library read again.
    mode(&games_db, 0o000);
    assert!(scene.set_property("library", "platformLibrary", ""));
    assert!(scene.set_property("library", "platformLibrary", library().to_str().unwrap()));
    said(&mut scene, "The library could not be read: ");
    assert_eq!(scene.property("gridGames", "count"), "2", "the rows kept");
    mode(&games_db, 0o644);
    scene.assert_quiet();
}
