//! Settings ▸ Consoles in the program (EmuSen_Qt.md §7.7.E), over the world Mistress's recordings script
//! (`settings_consoles.json`): the list, a console's page, Engine as a page of choices, the shoulders to another console,
//! Firmware with Open Folder, Memory cards, and a folder row's choice in the system's folder dialog, each state held at the gate's sizes and recorded for the comparison.

mod common;

use common::{hold, sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_kit_core::tokens::{colour, ColourToken};
use emusen_kit_core::tokens::Variant;
use emusen_platform_client::galaxia::consts::DIR_FIRMWARE;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_test::{mockup, Scene};

/// The gate's findings this screen names for the tester, as Settings names them (`settings.rs`): the help bar's words
/// under the floor at 1280 × 720.
fn stored(platform: &Platform, key: &str) -> serde_json::Value {
    platform.model_load(emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap()[key].clone()
}

fn named(fault: &str, _: &emusen_qt_test::tree::Tree, size: (i32, i32)) -> bool {
    size == (1280, 720) && fault.contains("too small to read")
}

/// The pane's sentence, which names the console a page is of.
fn sentence(scene: &mut Scene<'_>) -> String {
    scene.property("settingsSentence", "text")
}

/// Down to the row at `row` from the first, and A.
fn open_row(scene: &mut Scene<'_>, row: i32) {
    for _ in 0..row {
        scene.pad(UiButton::Down);
    }
    wait_for(scene, "settingsModel", "focused", &row.to_string(), 1000);
    scene.pad(UiButton::Accept);
    scene.wait(200);
}

#[test]
fn qt_7_7_e_consoles_and_a_consoles_pages_hold_their_states_and_open_folder_gives_the_folders_url() {
    let games: Vec<(&str, String)> = (1..=5).map(|n| ("snes", format!("Game {n}.sfc"))).collect();
    let games: Vec<(&str, &str)> = games.iter().map(|(f, g)| (*f, g.as_str())).collect();
    let _world = sandbox("qt-app-settings-consoles", &games);
    let platform = Platform::load(library()).unwrap();
    let world = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/settings_consoles.json");
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("settingsModel", "world", world));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "settingsModel", "ready", "true", 5000);
    scene.wait(300);
    for _ in 0..5 {
        scene.pad(UiButton::Right);
    }
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Settings", 2000);
    for _ in 0..3 {
        scene.pad(UiButton::PageDown);
        scene.wait(30);
    }
    wait_for(&mut scene, "settingsModel", "section", "3", 1000);
    let mut report = Vec::new();

    // The list, on the Super Nintendo as Mistress's recording is.
    for _ in 0..5 {
        scene.pad(UiButton::Down);
    }
    wait_for(&mut scene, "settingsModel", "focused", "5", 1000);
    scene.wait(200);
    assert_eq!(scene.property("settingsSentence", "text"), "Super Nintendo · 5 games · engine VenusRT (Rust)");
    hold(&mut scene, "SettingsConsoles", &mockup::Mockup::default(), named, &mut report);

    // Its page: the engine, firmware, cards, the core's own row, then the picture's rows.
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "settingsModel", "help", "20", 1000);
    assert!(sentence(&mut scene).starts_with("VenusRT (Rust), the Super Nintendo's engine"), "{}", sentence(&mut scene));
    hold(&mut scene, "SettingsConsolePage", &mockup::Mockup::default(), named, &mut report);
    // The firmware line in Success, as Mistress tinted it: the kit's sentence in the tone the model gives.
    scene.pad(UiButton::Down);
    wait_for(&mut scene, "settingsModel", "focused", "1", 1000);
    scene.wait(100);
    assert_eq!(sentence(&mut scene), "✓ No firmware needed");
    let success = colour(Variant::Dark, ColourToken::Success).to_hex().to_lowercase();
    assert_eq!(scene.property("settingsSentence", "color").to_lowercase(), success);
    scene.pad(UiButton::Up);
    wait_for(&mut scene, "settingsModel", "focused", "0", 1000);

    // Engine as a page of choices, B keeping what was chosen.
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "settingsModel", "help", "21", 1000);
    assert_eq!(sentence(&mut scene), "VenusRT (Rust) runs every Super Nintendo game.");
    hold(&mut scene, "SettingsEngine", &mockup::Mockup::default(), named, &mut report);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "settingsModel", "help", "20", 1000);

    // RB twice: the Sega CD, then the PlayStation, among the consoles an engine runs.
    scene.pad(UiButton::PageDown);
    scene.wait(100);
    scene.pad(UiButton::PageDown);
    scene.wait(200);
    assert!(sentence(&mut scene).starts_with("Diamond, the PlayStation's engine"), "{}", sentence(&mut scene));

    // Firmware: one row for the three regional BIOS files, and Open EmuSen's folder through the program's one action.
    open_row(&mut scene, 1);
    wait_for(&mut scene, "settingsModel", "help", "18", 1000);
    assert!(sentence(&mut scene).starts_with("No PlayStation BIOS of your own"), "{}", sentence(&mut scene));
    hold(&mut scene, "SettingsFirmware", &mockup::Mockup::default(), named, &mut report);
    assert!(scene.set_property("folders", "catching", "true"));
    for _ in 0..4 {
        scene.pad(UiButton::Down);
    }
    wait_for(&mut scene, "settingsModel", "focused", "4", 1000);
    scene.pad(UiButton::Accept);
    scene.wait(200);
    let folder = platform.directory(DIR_FIRMWARE).unwrap().expect("Galaxia's firmware folder");
    let url = emusen_qt_models::folders::url_of(folder.to_str().unwrap());
    assert!(url.starts_with("file:///"), "{url}");
    assert_eq!(scene.property("folders", "caught"), url, "the folder's file: URL, taken in place of a file manager");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "settingsModel", "help", "20", 1000);
    wait_for(&mut scene, "settingsModel", "focused", "1", 1000);

    // Memory cards: the mode, the slots, the drawer, the saves and a new card.
    scene.pad(UiButton::Down);
    wait_for(&mut scene, "settingsModel", "focused", "2", 1000);
    scene.pad(UiButton::Accept);
    scene.wait(200);
    assert!(sentence(&mut scene).starts_with("Shared: every PlayStation game"), "{}", sentence(&mut scene));
    hold(&mut scene, "SettingsMemoryCards", &mockup::Mockup::default(), named, &mut report);

    // Saves on the cards: the card manager, as Mistress's Memory Cards window; the card in use by a running game.
    for _ in 0..4 {
        scene.pad(UiButton::Down);
    }
    wait_for(&mut scene, "settingsModel", "focused", "4", 1000);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "settingsModel", "focused", "0", 1000);
    scene.wait(200);
    assert_eq!(sentence(&mut scene), "Shared: Card 1 in Slot 1, Card 2 in Slot 2. 2 save(s), 11 of 15 blocks free.");
    hold(&mut scene, "MemoryCardsPlayStation", &mockup::Mockup::default(), named, &mut report);
    scene.pad(UiButton::Right);
    scene.wait(200);
    assert!(sentence(&mut scene).ends_with("In use by Amber Saga, which is running: its saves can be looked at, and changed when the game is closed."));
    hold(&mut scene, "MemoryCardsPlayStationHeld", &mockup::Mockup::default(), named, &mut report);
    // A save's page and the question before Delete, which No closes.
    scene.pad(UiButton::Left);
    scene.pad(UiButton::Down);
    wait_for(&mut scene, "settingsModel", "focused", "1", 1000);
    scene.pad(UiButton::Accept);
    scene.wait(200);
    for _ in 0..3 {
        scene.pad(UiButton::Down);
    }
    wait_for(&mut scene, "settingsModel", "focused", "3", 1000);
    scene.pad(UiButton::Accept);
    scene.wait(200);
    assert_eq!(sentence(&mut scene), "Delete DRACULA X CHRONICLE SAVE from Card 1? It cannot be brought back.");
    hold(&mut scene, "MemoryCardsDelete", &mockup::Mockup::default(), named, &mut report);
    scene.pad(UiButton::Accept);
    scene.wait(200);
    scene.pad(UiButton::Back);
    scene.pad(UiButton::Back);
    scene.pad(UiButton::Back);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "settingsModel", "help", "17", 1000);

    // Library ▸ ROM folder asks the system's folder dialog, caught here: the folder chosen is saved, Cancel saves nothing.
    assert!(scene.set_property("folders", "catching", "true"));
    scene.pad(UiButton::PageDown);
    wait_for(&mut scene, "settingsModel", "section", "4", 1000);
    scene.wait(200);
    let roms = stored(&platform, "RomDirectory");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "folders", "waiting", "true", 1000);
    assert_eq!(scene.property("folders", "asked"), roms.as_str().unwrap(), "the dialog starts at the folder the row holds");
    assert!(scene.set_property("folders", "cancel", "true"));
    wait_for(&mut scene, "folders", "waiting", "false", 1000);
    assert_eq!(stored(&platform, "RomDirectory"), roms, "Cancel saves nothing");
    assert!(scene.set_property("folders", "cancel", "false"));
    let chosen = _world.dir("Chosen Roms");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "folders", "waiting", "true", 1000);
    assert!(scene.set_property("folders", "answer", chosen.to_str().unwrap()));
    wait_for(&mut scene, "folders", "waiting", "false", 1000);
    scene.wait(200);
    assert_eq!(stored(&platform, "RomDirectory"), chosen.to_str().unwrap(), "the folder chosen, saved through AppSettings");
    assert!(scene.property("settingsSentence", "text").contains(chosen.to_str().unwrap()), "and shown at once");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    scene.assert_quiet();
    assert!(report.is_empty(), "{}", report.join("\n"));
}
