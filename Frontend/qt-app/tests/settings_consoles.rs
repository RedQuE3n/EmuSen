//! Settings ▸ Consoles in the program (EmuSen_Qt.md §7.7.E), over the world Mistress's recordings script
//! (`settings_consoles.json`): the list, a console's page, Engine as a page of choices, the shoulders to another console,
//! Firmware with Open Folder, and Memory cards, each state held at the gate's sizes and recorded for the comparison.

mod common;

use common::{hold, sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::DIR_FIRMWARE;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_test::{mockup, Scene};

/// The gate's findings this screen names for the tester, as Settings names them (`settings.rs`): the help bar's words
/// under the floor at 1280 × 720.
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
    wait_for(&mut scene, "settingsModel", "help", "19", 1000);
    assert!(sentence(&mut scene).starts_with("VenusRT (Rust), the Super Nintendo's engine"), "{}", sentence(&mut scene));
    hold(&mut scene, "SettingsConsolePage", &mockup::Mockup::default(), named, &mut report);

    // Engine as a page of choices, B keeping what was chosen.
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "settingsModel", "help", "20", 1000);
    assert_eq!(sentence(&mut scene), "VenusRT (Rust) runs every Super Nintendo game.");
    hold(&mut scene, "SettingsEngine", &mockup::Mockup::default(), named, &mut report);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "settingsModel", "help", "19", 1000);

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
    wait_for(&mut scene, "settingsModel", "help", "19", 1000);
    wait_for(&mut scene, "settingsModel", "focused", "0", 1000);

    // Memory cards: the mode, the slots, the drawer, the saves and a new card.
    open_row(&mut scene, 2);
    assert!(sentence(&mut scene).starts_with("Shared: every PlayStation game"), "{}", sentence(&mut scene));
    hold(&mut scene, "SettingsMemoryCards", &mockup::Mockup::default(), named, &mut report);
    scene.pad(UiButton::Back);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "settingsModel", "help", "17", 1000);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    scene.assert_quiet();
    assert!(report.is_empty(), "{}", report.join("\n"));
}
