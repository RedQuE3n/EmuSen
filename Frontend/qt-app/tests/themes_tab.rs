//! The Themes tab in the program (EmuSen_Qt.md §3.8): reached through Settings ▸ Library ▸ Themes, a theme checked before Use is
//! offered, Use storing the theme's id and switching the chosen theme's host at once, one engine however often it switches,
//! and none once the reference is chosen again.

mod common;

use common::{hold, sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::{DIR_THEMES, MODEL_APP_SETTINGS};
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_test::excuse::Excuses;
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

/// Copies the clean fixture to the store as `id`, named `name`.
/// A fixture under another id and name, its manifest's own name replaced.
fn named_copy(store: &Path, fixture: &str, was: &str, id: &str, name: &str) {
    copy(&Path::new(FIXTURES).join(fixture), &store.join(id));
    let manifest = store.join(id).join("theme.json");
    let text = std::fs::read_to_string(&manifest).unwrap().replace(&format!("\"{was}\""), &format!("\"{name}\""));
    std::fs::write(&manifest, text).unwrap();
}

/// theme-audit as this build made it, beside the test's own program in the target folder.
fn theme_audit() -> std::path::PathBuf {
    let exe = std::env::current_exe().unwrap();
    let audit = exe.parent().and_then(Path::parent).unwrap().join("theme-audit");
    assert!(audit.exists(), "theme-audit is built beside this test ({}): cargo build --release -p emusen-theme-audit", audit.display());
    audit
}

fn focused(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default()
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
    while scene.property("themesModel", "findings").starts_with("Checking") && waited < 180_000 {
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
    // Two themes the real theme-audit judges: titles cut with an ellipsis, a class R-20261111-44 accepts, and Moonrise's
    // caption's name on a Home of its own, which no ruling accepts (R-20261111-40 covers the caption in Moonrise alone).
    named_copy(&store, "elided-title", "Fixture elided title", "sage", "Fixture kept");
    named_copy(&store, "small-caption", "Fixture small caption", "tide", "Fixture held");
    let book = store.join("book");
    std::fs::create_dir_all(&book).unwrap();
    std::fs::write(book.join("capabilities.xml"), "<themeCapabilities><themeName>Picture Book</themeName></themeCapabilities>").unwrap();
    std::fs::write(book.join("theme.xml"), "<theme><view name=\"system\"><text name=\"t\"><text>x</text></text></view></theme>").unwrap();
    // theme-audit stood in for by a script that finds nothing for the first three, whose loaded step is theme-check's own
    // tests' to prove; sage and tide go to the real theme-audit, so what kit-core's table accepts is what reaches the tab.
    let audit = world.root.join("audit.sh");
    let ran = world.root.join("audit-ran");
    let script = format!(
        r#"#!/bin/bash
touch "{}"
package="${{@: -1}}"
case "$package" in
  *sage|*tide) exec "{}" "$@" ;;
esac
exit 0
"#,
        ran.display(),
        theme_audit().display()
    );
    std::fs::write(&audit, script).unwrap();
    assert!(std::process::Command::new("chmod").arg("+x").arg(&audit).status().unwrap().success(), "the stand-in made runnable");
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

    // Home's Settings place opens Settings, whose Library ▸ Themes opens the tab on the EmuSen kind, the reference first
    // and in use (EmuSen_Qt.md §7.7).
    for _ in 0..5 {
        scene.pad(UiButton::Right);
    }
    assert_eq!(focused(&mut scene), "place.5");
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
    assert_eq!(scene.property("themesModel", "findings"), "In use.");
    assert_eq!(scene.property("themesModel", "usable"), "false");
    scene.wait(300);
    let mut report = Vec::new();
    let mut excuses = Excuses::none().listing("qt-app-themes-tab");
    hold(&mut scene, "ThemesTab", &mockup::Mockup::default(), &mut excuses, &mut report);

    // A package is checked before Use is offered; Use stores its id and its host is made at once.
    check_next(&mut scene, "Fixture clean");
    assert_eq!(scene.property("themesModel", "findings"), "Checked: nothing found.");
    assert!(ran.exists(), "nothing found only once the loaded step ran");
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

    // An exemption of a class the tester accepted (R-20261111-44) is no finding: Use is offered and takes the theme.
    check_next(&mut scene, "Fixture kept");
    assert_eq!(scene.property("themesModel", "findings"), "Checked: nothing found.");
    assert_eq!(scene.property("themesModel", "usable"), "true");
    scene.pad(UiButton::Accept);
    scene.wait(200);
    assert_eq!(stored(&platform), "sage");

    // Words under the floor no ruling accepts are a finding: no Use, and A stores nothing.
    check_next(&mut scene, "Fixture held");
    assert_eq!(scene.property("themesModel", "usable"), "false");
    assert!(scene.property("themesModel", "findings").contains("moonriseCaption"), "{}", scene.property("themesModel", "findings"));
    scene.pad(UiButton::Accept);
    scene.wait(200);
    assert_eq!(stored(&platform), "sage");

    // The reference again: its id is stored and the chosen theme's host is gone with its engine.
    for _ in 0..5 {
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

    // B goes back to Settings, and B there goes Home.
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Settings", 2000);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    assert!(report.is_empty(), "{}", report.join("\n"));
}

