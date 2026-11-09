//! The Themes tab in the program (EmuSen_Qt.md §3.8): reached from Home's Settings place, a theme checked before Use is
//! offered, Use storing the theme's id and switching the chosen theme's host at once, one engine however often it switches,
//! and none once the reference is chosen again.

mod common;

use common::{hold, sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::{DIR_THEMES, MODEL_APP_SETTINGS};
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_test::{mockup, Scene};
use std::path::Path;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../theme-check/tests/themes");

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap().flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn focused(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default()
}

fn named(fault: &str, _tree: &emusen_qt_test::tree::Tree, size: (i32, i32)) -> bool {
    size == (1280, 720) && fault.contains("too small to read")
}

/// The id the settings file holds.
fn stored(platform: &Platform) -> String {
    let settings = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    settings["EmuSenTheme"].as_str().unwrap_or_default().to_owned()
}

/// Down to the next row, then X checks it; waits for the check's words.
fn check_next(scene: &mut Scene<'_>, name: &str) {
    scene.pad(UiButton::Down);
    wait_for(scene, "themesModel", "name", name, 2000);
    scene.pad(UiButton::Screensaver);
    let mut waited = 0;
    while scene.property("themesModel", "findings").starts_with("Checking") && waited < 30000 {
        scene.wait(50);
        waited += 50;
    }
}

#[test]
fn qt_3_8_the_themes_tab_checks_uses_and_switches_the_chosen_theme_s_host() {
    let world = sandbox("qt-app-themes-tab", &[("NES", "One.nes")]);
    let platform = Platform::load(library()).unwrap();
    let store = platform.directory(DIR_THEMES).unwrap().unwrap();
    copy(&Path::new(FIXTURES).join("clean"), &store.join("dawn"));
    copy(&Path::new(FIXTURES).join("loaded"), &store.join("dusk"));
    copy(&Path::new(FIXTURES).join("literal"), &store.join("rough"));
    let book = store.join("book");
    std::fs::create_dir_all(&book).unwrap();
    std::fs::write(book.join("capabilities.xml"), "<themeCapabilities><themeName>Picture Book</themeName></themeCapabilities>").unwrap();
    std::fs::write(book.join("theme.xml"), "<theme><view name=\"system\"><text name=\"t\"><text>x</text></text></view></theme>").unwrap();
    // theme-audit stood in for by a script that finds nothing; the loaded step is theme-check's own tests' to prove.
    let audit = world.root.join("audit.sh");
    std::fs::write(&audit, "#!/bin/bash\nexit 0\n").unwrap();
    std::process::Command::new("chmod").arg("+x").arg(&audit).status().unwrap();
    std::env::set_var("EMUSEN_THEME_AUDIT", &audit);

    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    assert_eq!(scene.property("chosenTheme", "status"), "Null", "with the reference chosen no second host is made");
    for _ in 0..20 {
        if focused(&mut scene) == "place.0" {
            break;
        }
        scene.wait(50);
    }

    // Home's Settings place opens the tab on the EmuSen kind, the reference first and in use.
    for _ in 0..5 {
        scene.pad(UiButton::Right);
    }
    assert_eq!(focused(&mut scene), "place.5");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Themes", 2000);
    wait_for(&mut scene, "themesModel", "name", "EmuSen", 5000);
    assert_eq!(scene.property("themesModel", "findings"), "In use.");
    assert_eq!(scene.property("themesModel", "usable"), "false");
    scene.wait(300);
    let mut report = Vec::new();
    hold(&mut scene, "ThemesTab", &mockup::Mockup::default(), named, &mut report);

    // A package is checked before Use is offered; Use stores its id and its host is made at once.
    check_next(&mut scene, "Fixture clean");
    assert_eq!(scene.property("themesModel", "findings"), "Checked: nothing found.");
    assert_eq!(scene.property("themesModel", "usable"), "true");
    scene.pad(UiButton::Accept);
    scene.wait(200);
    assert_eq!(stored(&platform), "dawn", "the id is stored, never a folder");
    assert_eq!(scene.property("program", "theme"), store.join("dawn").to_str().unwrap());
    wait_for(&mut scene, "host.theme", "ready", "true", 2000);
    assert_eq!(scene.property("themesModel", "findings"), "In use.");

    // A second theme: the host switches, its last engine let go.
    check_next(&mut scene, "Fixture loaded");
    scene.pad(UiButton::Accept);
    scene.wait(200);
    assert_eq!(stored(&platform), "dusk");
    wait_for(&mut scene, "host.theme", "name", "Fixture loaded", 2000);
    assert_eq!(scene.property("host.theme", "liveEngines"), "1", "switching twice leaks no engine");

    // A theme with findings has no Use: A does nothing, and its findings are its detail.
    check_next(&mut scene, "Fixture literal");
    assert_eq!(scene.property("themesModel", "usable"), "false");
    assert!(scene.property("themesModel", "findings").starts_with("rough/"), "{}", scene.property("themesModel", "findings"));
    scene.pad(UiButton::Accept);
    scene.wait(200);
    assert_eq!(stored(&platform), "dusk");

    // The reference again: its id is stored and the chosen theme's host is gone with its engine.
    for _ in 0..3 {
        scene.pad(UiButton::Up);
    }
    wait_for(&mut scene, "themesModel", "name", "EmuSen", 2000);
    scene.pad(UiButton::Accept);
    scene.wait(200);
    assert_eq!(stored(&platform), "emusen");
    assert_eq!(scene.property("program", "theme"), "");
    wait_for(&mut scene, "chosenTheme", "status", "Null", 2000);
    assert_eq!(scene.property("host.theme", "liveEngines"), "", "no host, so no engine");

    // The other kind: an ES-DE theme is listed and checked, not chosen.
    scene.pad(UiButton::PageDown);
    wait_for(&mut scene, "themesModel", "name", "Picture Book", 2000);
    assert_eq!(scene.property("themesModel", "kind"), "ES-DE");
    assert_eq!(scene.property("themesModel", "usable"), "false");

    // Back to the first kind, its first row focused again and the walk going on from it.
    scene.pad(UiButton::PageUp);
    wait_for(&mut scene, "themesModel", "name", "EmuSen", 2000);
    scene.pad(UiButton::Down);
    wait_for(&mut scene, "themesModel", "name", "Fixture clean", 2000);

    // B goes Home.
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    assert!(report.is_empty(), "{}", report.join("\n"));
}
