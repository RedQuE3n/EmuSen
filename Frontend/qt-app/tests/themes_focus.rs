//! The Themes tab keeps the pad while a theme it uses loads behind it (EmuSen_Qt.md §3.5.2): Settings ▸ Library ▸ Themes,
//! Meridian, Horizon, Orrery and Constellation each checked and used with the pad, and after each the tab still walks its
//! rows, then B goes back to Settings. A theme's screens that are not shown never take the focus, whatever they call.

mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::DIR_THEMES;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_test::Scene;
use std::path::Path;

const THEMES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../themes");

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
    scene.focus(&tree).map(|i| format!("{} {}", tree.nodes[i as usize].ty, tree.nodes[i as usize].name)).unwrap_or_default()
}

/// The tab's rows walked with the pad to the theme named.
fn to(scene: &mut Scene<'_>, name: &str) {
    for _ in 0..6 {
        if scene.property("themesModel", "name") == name {
            return;
        }
        scene.pad(UiButton::Down);
        scene.wait(50);
    }
    wait_for(scene, "themesModel", "name", name, 500);
}

#[test]
fn qt_3_5_2_the_themes_tab_keeps_the_pad_while_each_theme_it_uses_loads() {
    let world = sandbox("qt-app-themes-focus", &[("NES", "One.nes")]);
    let platform = Platform::load(library()).unwrap();
    let store = platform.directory(DIR_THEMES).unwrap().unwrap();
    for theme in ["constellation", "horizon", "meridian", "orrery"] {
        copy(&Path::new(THEMES).join(theme), &store.join(theme));
    }
    // theme-audit stood in for by a script that finds nothing: the themes' own audits are theme-audit's tests'.
    let audit = world.root.join("audit.sh");
    std::fs::write(&audit, "#!/bin/bash\nexit 0\n").unwrap();
    assert!(std::process::Command::new("chmod").arg("+x").arg(&audit).status().unwrap().success());
    std::env::set_var("EMUSEN_THEME_AUDIT", &audit);

    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    for _ in 0..40 {
        if focused(&mut scene).ends_with("place.0") {
            break;
        }
        scene.wait(50);
    }
    for _ in 0..5 {
        scene.pad(UiButton::Right);
    }
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Settings", 2000);
    for _ in 0..4 {
        scene.pad(UiButton::PageDown);
    }
    wait_for(&mut scene, "settingsModel", "section", "4", 1000);
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Down);
    wait_for(&mut scene, "settingsModel", "focused", "2", 1000);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Themes", 2000);
    wait_for(&mut scene, "themesModel", "name", "EmuSen", 5000);

    for name in ["Meridian", "Horizon", "Orrery", "Constellation"] {
        while scene.property("themesModel", "name") != "EmuSen" {
            scene.pad(UiButton::Up);
            scene.wait(50);
        }
        to(&mut scene, name);
        scene.pad(UiButton::Screensaver);
        let mut waited = 0;
        while scene.property("themesModel", "findings") != "Checked: nothing found." && waited < 30_000 {
            scene.wait(50);
            waited += 50;
        }
        assert_eq!(scene.property("themesModel", "usable"), "true", "{name}: {}", scene.property("themesModel", "findings"));
        scene.pad(UiButton::Accept);
        wait_for(&mut scene, "host.theme", "name", name, 3000);
        wait_for(&mut scene, "host.theme", "ready", "true", 3000);
        scene.wait(500);
        let at = focused(&mut scene);
        // The tab still answers the pad: Up walks its rows, from the theme just used to the one above it.
        scene.pad(UiButton::Up);
        scene.wait(100);
        assert_ne!(scene.property("themesModel", "name"), name, "{name} used: the tab answers Up (focus on {at})");
        assert_eq!(scene.property("screens", "shown"), "Themes");
    }
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Settings", 2000);
}
