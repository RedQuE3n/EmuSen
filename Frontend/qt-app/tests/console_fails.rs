//! Start Console and a game's page on the software backend (EmuSen_Qt.md §7.9), over the client fixture's console core,
//! a PlayStation that starts empty, approved in the player's cores folder alone. A PlayStation disc's page lists that
//! engine, so the page's runtime has the player's cores; a page that cannot be read is said each time it is opened; and a
//! console whose start fails at once (the software backend names no picture) gives way to the library's screen with the
//! failure on the notice layer, in the own look from the List card and under an ES-DE theme from the Home card.

mod common;

use common::console::{choose_row, grid_ready, row_focused, until};
use common::{approved_console_core, sandbox, wait_for};

use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::{FILE_SETTINGS, MODEL_APP_SETTINGS};
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_test::Scene;
use serde_json::json;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// Art Book Next, the theme esde.rs holds, from the corpus.
const THEME: &str = "art-book-next-es-de";

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

fn mode(path: &std::path::Path, mode: u32) {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
}

/// Home's Games, All Games, the grid.
fn grid(scene: &mut Scene<'_>) {
    for _ in 0..40 {
        let tree = scene.tree();
        if let Some(games) = tree.find_named("place.0") {
            if scene.focus(&tree) == Some(games) || scene.focus_item(&tree, games) {
                break;
            }
        }
        scene.wait(50);
    }
    scene.pad(UiButton::Accept);
    wait_for(scene, "screens", "shown", "Consoles", 1000);
    let tree = scene.tree();
    let all = tree.find_named("tile.0").expect("All Games");
    assert!(scene.focus_item(&tree, all));
    scene.pad(UiButton::Accept);
    wait_for(scene, "screens", "shown", "Grid", 1000);
    grid_ready(scene);
}

#[test]
fn qt_7_9_a_console_that_fails_to_start_gives_way_to_the_library_and_a_page_reads_the_player_s_cores() {
    let theme = PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/emusen/bigpicture/corpus/whole").join(THEME);
    assert!(theme.exists(), "the theme at {}", theme.display());
    let world = sandbox("qt-app-console-fails", &[]);
    std::fs::write(world.dir("Roms/psx").join("Basalt Run.iso"), b"not a disc").unwrap();
    let cores = world.dir("lib");
    approved_console_core(&cores);
    let platform = Platform::load(library()).unwrap();

    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 20000);
    until(&mut scene, 10, "the console listed", |s| s.property("library", "startable").contains("\"PlayStation\""));

    // The disc's page: its Options name the one engine, found in the player's cores folder alone.
    grid(&mut scene);
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "screens", "shown", "Page", 2000);
    wait_for(&mut scene, "pageModel", "ready", "true", 10000);
    assert_eq!(scene.property("pageModel", "title"), "Basalt Run");
    assert!(scene.set_property("pageModel", "section", "2"));
    until(&mut scene, 5, "the engine row", |s| s.tree().nodes.iter().any(|n| n.visible && n.text.as_deref() == Some("V1ConsoleCore (the only one)")));

    // A page that cannot be read, the settings it reads made unreadable: said, and said again when it is opened again.
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);
    let settings_file = platform.file(FILE_SETTINGS).unwrap();
    mode(&settings_file, 0o000);
    let mut first = String::new();
    for again in [false, true] {
        grid_ready(&mut scene);
        scene.pad(UiButton::Screensaver);
        wait_for(&mut scene, "screens", "shown", "Page", 2000);
        let words = said(&mut scene, "The game's page could not be read: ");
        if again {
            assert_eq!(words, first, "the same failure said again");
        }
        first = words;
        wait_for(&mut scene, "notices", "shown", "false", 6000);
        scene.pad(UiButton::Back);
        wait_for(&mut scene, "screens", "shown", "Grid", 2000);
    }
    mode(&settings_file, 0o644);

    // The own look: the List card's Start Console, A on the PlayStation; the start fails at once, and Home is shown.
    scene.pad(UiButton::Options);
    until(&mut scene, 5, "the List card in place", |s| s.property("screens", "shown") == "List" && s.property("listCardCanvas", "slide") == "0");
    choose_row(&mut scene, "Start Console");
    until(&mut scene, 5, "Start Console's page", |s| s.property("listCardModel", "page") == emusen_qt_models::list_card::START_CONSOLE);
    row_focused(&mut scene, "PlayStation");
    choose_row(&mut scene, "PlayStation");
    let words = said(&mut scene, "Failed to start PlayStation: ");
    assert_eq!(scene.property("notices", "attention"), "true");
    until(&mut scene, 5, "the library's screen", |s| s.property("screens", "shown") == "Home" && s.property("session", "running") == "false");
    // Still Home once the notice has gone, seconds later: no game's screen follows the failure.
    wait_for(&mut scene, "notices", "shown", "false", 6000);
    assert_eq!(scene.property("screens", "shown"), "Home", "no game's screen after the failure: {words}");

    // Under an ES-DE theme: Start's Home card, its Start Console, A on the PlayStation; the theme is shown again.
    let mut settings = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    settings["LibraryStyle"] = json!("Theme");
    settings["BigPictureTheme"] = json!(theme.to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    assert!(scene.set_property("program", "esdeChosen", "true"));
    assert!(scene.set_property("library", "platformLibrary", ""));
    assert!(scene.set_property("library", "platformLibrary", library().to_str().unwrap()));
    until(&mut scene, 30, "the theme in place", |s| s.property("screens", "shown") == "Esde" && s.property("esdeModel", "view") == "system");
    until(&mut scene, 10, "the console listed", |s| s.property("library", "startable").contains("\"PlayStation\""));
    scene.pad(UiButton::Menu);
    until(&mut scene, 5, "the Home card in place", |s| s.property("screens", "shown") == "HomeCard" && s.property("homeCardCanvas", "slide") == "0");
    choose_row(&mut scene, "Start Console");
    until(&mut scene, 5, "Start Console's page", |s| s.property("homeCardModel", "page") == emusen_qt_models::home_card::START_CONSOLE);
    row_focused(&mut scene, "PlayStation");
    choose_row(&mut scene, "PlayStation");
    let words = said(&mut scene, "Failed to start PlayStation: ");
    until(&mut scene, 5, "the theme's screen", |s| s.property("screens", "shown") == "Esde" && s.property("session", "running") == "false");
    wait_for(&mut scene, "notices", "shown", "false", 6000);
    assert_eq!(scene.property("screens", "shown"), "Esde", "no game's screen after the failure: {words}");
    assert_eq!(scene.property("esde", "activeFocus"), "true", "the theme takes the pad again");
    scene.assert_quiet();
}
