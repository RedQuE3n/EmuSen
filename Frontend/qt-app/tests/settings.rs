//! Settings in the program (EmuSen_Qt.md §7.7): Home's Settings place opens it, the shoulders step the sections, a step
//! is saved through Galaxia at once, Library ▸ Themes opens the Themes tab and B comes back, and the six states Mistress
//! recorded are held at the gate's sizes and recorded for the comparison.

mod common;

use common::{hold, hold_in, sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::audit::Class;
use emusen_kit_core::input::UiButton;
use emusen_kit_core::tokens::Variant;
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_test::excuse::{Excuse, Excuses};
use emusen_qt_test::{mockup, Scene};

fn focused(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| format!("{} {}", tree.nodes[i as usize].ty, tree.nodes[i as usize].name)).unwrap_or_default()
}

/// The gate's finding this screen names for the tester: the ROM folder's path, longer than half its row, cut at its start
/// so the folder's own name shows, which the pane gives whole once the row is focused (held below; EmuSen_Qt.md §7.7).
const ROM_FOLDER_CUT: Excuse = Excuse {
    finding: "cut with an ellipsis",
    item: "value",
    with: "",
    words: "/Roms",
    with_words: "",
    size: None,
    why: "the path cut at its start, the pane giving it whole (EmuSen_Qt.md §7.7)",
    class: Some(Class::RomFolderCutAtStart),
};

fn stored(platform: &Platform, key: &str) -> serde_json::Value {
    platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap()[key].clone()
}

/// The section by its index from the first, through the right shoulder, and the row by its index through Down.
fn reach(scene: &mut Scene<'_>, section: i32, row: i32) {
    while scene.property("settingsModel", "section") != section.to_string() {
        scene.pad(UiButton::PageDown);
        scene.wait(30);
    }
    for _ in 0..row {
        scene.pad(UiButton::Down);
    }
    wait_for(scene, "settingsModel", "focused", &row.to_string(), 1000);
    scene.wait(200);
}

#[test]
fn qt_7_7_settings_opens_from_home_steps_saves_and_holds_its_states() {
    let world = sandbox("qt-app-settings", &[("NES", "One.nes"), ("NES", "Two.nes")]);
    let platform = Platform::load(library()).unwrap();
    // Two values as Mistress's own-look fixture holds them: the base's BigScreen on, which no row of Settings shows.
    let mut held = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    held["BigScreen"] = serde_json::json!(true);
    held["ResumeOnLaunch"] = serde_json::json!("Restart");
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &held).unwrap();
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "settingsModel", "ready", "true", 5000);
    for _ in 0..20 {
        if focused(&mut scene).ends_with("place.0") {
            break;
        }
        scene.wait(50);
    }

    // Home's Settings place opens Settings on Controllers.
    for _ in 0..5 {
        scene.pad(UiButton::Right);
    }
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Settings", 2000);
    assert_eq!(scene.property("settingsModel", "section"), "0");
    let mut report = Vec::new();
    let mut excuses = Excuses::new(&[ROM_FOLDER_CUT]).listing("qt-app-settings");

    reach(&mut scene, 2, 0);
    assert_eq!(scene.property("settingsModel", "preview"), "2", "Volume's pane holds its meter");
    assert_eq!(scene.property("SettingsMeter", "visible"), "true", "and draws it");
    hold(&mut scene, "SettingsSound", &mockup::Mockup::default(), &mut excuses, &mut report);
    reach(&mut scene, 4, 0);
    hold(&mut scene, "SettingsLibrary", &mockup::Mockup::default(), &mut excuses, &mut report);
    // The ROM folder's path, cut at its start on the row, is whole in the pane once the row is focused: the named cut's
    // condition.
    let roms = world.root.join("Roms").display().to_string();
    assert!(scene.property("settingsSentence", "text").contains(&roms), "the pane names {roms} whole");
    reach(&mut scene, 5, 0);
    // Big Picture is the program on every machine (the tester decided, 2026-11-13): EmuSen holds no Big picture row.
    let words: Vec<String> = scene.tree().nodes.iter().filter(|n| n.visible).filter_map(|n| n.text.clone()).collect();
    assert!(words.iter().any(|w| w == "Pause in the background"), "EmuSen's rows drawn: {words:?}");
    assert!(!words.iter().any(|w| w == "Big picture"), "no Big picture row: {words:?}");
    hold(&mut scene, "SettingsEmuSen", &mockup::Mockup::default(), &mut excuses, &mut report);
    // The shoulders wrap from EmuSen to Controllers, then on to Display.
    reach(&mut scene, 1, 5);
    assert_eq!(scene.property("settingsModel", "preview"), "4", "Text size's pane holds a line at each size");
    hold(&mut scene, "SettingsTextSize", &mockup::Mockup::default(), &mut excuses, &mut report);
    for _ in 0..4 {
        scene.pad(UiButton::Up);
    }
    wait_for(&mut scene, "settingsModel", "focused", "1", 1000);
    assert_eq!(scene.property("settingsModel", "caption"), "The last game · Off CRT", "the CRT's pane names its picture");
    hold(&mut scene, "SettingsDisplay", &mockup::Mockup::default(), &mut excuses, &mut report);

    // Light or dark stepped to Light: saved, and the whole program drawn in the light variant at once.
    for _ in 0..8 {
        scene.pad(UiButton::Down);
    }
    wait_for(&mut scene, "settingsModel", "focused", "9", 1000);
    scene.pad(UiButton::Right);
    wait_for(&mut scene, "settingsModel", "light", "true", 1000);
    assert_eq!(stored(&platform, "Appearance"), "light");
    scene.wait(200);
    hold_in(&mut scene, "SettingsDisplayLight", &mockup::Mockup::default(), &mut excuses, &mut report, Variant::Light);
    scene.pad(UiButton::Left);
    wait_for(&mut scene, "settingsModel", "light", "false", 1000);

    // Text size stepped: saved, and the kit's type scale follows at once, from the kit's own scale for the word.
    for _ in 0..4 {
        scene.pad(UiButton::Up);
    }
    scene.pad(UiButton::Right);
    wait_for(&mut scene, "settingsModel", "textSize", "large", 1000);
    assert_eq!(stored(&platform, "TextSize"), "large");
    assert_eq!(*scene.kit_tokens().unwrap().text_scale(), emusen_kit_core::tokens::text_scale_of("large"));
    assert_eq!(*scene.kit_tokens().unwrap().text_scale(), 1.15);
    scene.pad(UiButton::Left);
    wait_for(&mut scene, "settingsModel", "textSize", "default", 1000);
    assert_eq!(*scene.kit_tokens().unwrap().text_scale(), 1.0);

    // Sound ▸ Volume stepped down: a share saved as a number.
    reach(&mut scene, 2, 0);
    scene.pad(UiButton::Left);
    scene.wait(100);
    assert_eq!(stored(&platform, "Volume"), serde_json::json!(0.95));

    // Library ▸ Scraping opens a page under the section, which B closes; Themes opens the tab, and B comes back.
    reach(&mut scene, 4, 3);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "settingsModel", "help", "18", 1000);
    assert_eq!(scene.property("settingsModel", "focused"), "0", "the page opens on its first row");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "settingsModel", "help", "17", 1000);
    assert_eq!(scene.property("screens", "shown"), "Settings");
    for _ in 0..3 {
        scene.pad(UiButton::Up);
    }
    for _ in 0..2 {
        scene.pad(UiButton::Down);
    }
    wait_for(&mut scene, "settingsModel", "focused", "2", 1000);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Themes", 2000);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Settings", 2000);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    // What Settings saved reaches the library's ES-DE settings once Settings closes: the media viewer plays at the new Volume.
    let mut waited = 0;
    while !scene.property("library", "esdeSettings").contains("\"volume\":0.95") {
        assert!(waited < 5000, "the ES-DE view's settings carry the Volume saved: {}", scene.property("library", "esdeSettings"));
        scene.wait(50);
        waited += 50;
    }
    scene.assert_quiet();
    assert!(report.is_empty(), "{}", report.join("\n"));
}
