//! An ES-DE theme chosen and configured from the Themes tab (EmuSen_Qt.md §3.8, §4.15): a small theme the test writes, of two
//! variants and two colour schemes, checked and used with the pad; the library drawn by it at once; its variant and colour
//! scheme stepped on the tab, each stored under the theme's folder and seen in the scene; then EmuSen's own theme used again.

mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::{DIR_THEMES, MODEL_APP_SETTINGS};
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_models::esde_shelves::theme_key;
use emusen_qt_test::Scene;
use serde_json::Value;

const W: i32 = 1280;
const H: i32 = 800;

const CAPABILITIES: &str = "<themeCapabilities><themeName>Fixture Pane</themeName>\
    <variant name=\"day\"><label>Day</label><selectable>true</selectable></variant>\
    <variant name=\"night\"><label>Night</label><selectable>true</selectable></variant>\
    <colorScheme name=\"warm\"><label>Warm</label></colorScheme><colorScheme name=\"cool\"><label>Cool</label></colorScheme>\
    </themeCapabilities>";

// The variant decides which word is drawn; the colour scheme's variables where the scheme's mark stands and its colour.
const THEME: &str = "<theme>\
    <colorScheme name=\"warm\"><variables><markX>0.05</markX><ink>FF8000</ink></variables></colorScheme>\
    <colorScheme name=\"cool\"><variables><markX>0.6</markX><ink>0080FF</ink></variables></colorScheme>\
    <variant name=\"day\"><view name=\"system\"><text name=\"day\"><pos>0.05 0.05</pos><size>0.3 0.1</size><text>Day</text><color>FFFFFF</color></text></view></variant>\
    <variant name=\"night\"><view name=\"system\"><text name=\"night\"><pos>0.05 0.05</pos><size>0.3 0.1</size><text>Night</text><color>FFFFFF</color></text></view></variant>\
    <view name=\"system\"><carousel name=\"systems\"><pos>0 0.3</pos><size>1 0.4</size><text>${system.fullName}</text><textColor>FFFFFF</textColor></carousel>\
    <text name=\"scheme\"><pos>${markX} 0.8</pos><size>0.2 0.1</size><text>Scheme</text><color>${ink}</color></text></view>\
    <view name=\"gamelist\"><textlist name=\"games\"><pos>0.05 0.1</pos><size>0.4 0.8</size><primaryColor>FFFFFF</primaryColor></textlist></view>\
    </theme>";

fn settings(platform: &Platform) -> Value {
    platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap()
}

/// The box of the first visible item of this objectName, in window pixels.
fn box_of(scene: &mut Scene<'_>, name: &str) -> Option<(f64, f64, f64, f64)> {
    scene.tree().nodes.iter().find(|n| n.name == name && n.visible).map(|n| (n.x, n.y, n.w, n.h))
}

/// Waits up to `seconds` for a condition the scene answers, letting Qt run between looks.
fn until(scene: &mut Scene<'_>, seconds: u64, what: &str, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    let start = std::time::Instant::now();
    while !done(scene) {
        let state = (scene.property("esdeModel", "problem"), scene.property("library", "themed"), scene.property("screens", "shown"));
        assert!(start.elapsed() < std::time::Duration::from_secs(seconds), "{what}: not within {seconds} s (problem, themed, shown: {state:?})");
        scene.wait(16);
    }
}

fn focused(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default()
}

#[test]
fn an_es_de_theme_is_used_from_the_themes_tab_its_choices_change_the_scene_and_emusen_s_own_comes_back() {
    let _world = sandbox("qt-app-themes-esde", &[("NES", "One.nes")]);
    let platform = Platform::load(library()).unwrap();
    let store = platform.directory(DIR_THEMES).unwrap().unwrap();
    let pane = store.join("pane");
    std::fs::create_dir_all(&pane).unwrap();
    std::fs::write(pane.join("capabilities.xml"), CAPABILITIES).unwrap();
    std::fs::write(pane.join("theme.xml"), THEME).unwrap();
    let key = theme_key(pane.to_str().unwrap());

    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, W, H);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    assert_eq!(scene.property("library", "themed"), "false", "EmuSen's own look at the start");
    until(&mut scene, 2, "Home's first place focused", |s| focused(s) == "place.0");

    // Home's Settings place, Library ▸ Themes, as themes_tab.rs walks there.
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

    // The ES-DE kind: the theme checked by its measure, then used; the library is drawn by it without a restart.
    scene.pad(UiButton::PageDown);
    wait_for(&mut scene, "themesModel", "name", "Fixture Pane", 2000);
    assert_eq!(scene.property("themesModel", "usable"), "false", "not before a check");
    scene.pad(UiButton::Screensaver);
    until(&mut scene, 60, "the check", |s| !s.property("themesModel", "findings").starts_with("Checking"));
    assert_eq!(scene.property("themesModel", "findings"), "Checked: nothing found.");
    assert_eq!(scene.property("themesModel", "usable"), "true");
    scene.pad(UiButton::Accept);
    until(&mut scene, 2, "the use stored", |s| s.property("themesModel", "findings") == "In use.");
    let stored = settings(&platform);
    assert_eq!((stored["LibraryStyle"].as_str(), stored["BigPictureTheme"].as_str()), (Some("Theme"), pane.to_str()));
    until(&mut scene, 20, "the library drawn by the theme", |s| s.property("library", "themed") == "true" && s.property("esdeModel", "themed") == "true");
    until(&mut scene, 5, "the first variant drawn", |s| box_of(s, "element-day").is_some());
    assert!(box_of(&mut scene, "element-night").is_none(), "one variant at a time");
    let warm = box_of(&mut scene, "element-scheme").expect("the scheme's mark drawn");
    assert!((warm.0 - 0.05 * f64::from(W)).abs() < 2.0, "the first colour scheme's place: {warm:?}");

    // The choices follow the theme in use: Right steps the variant, stored under the theme's folder, and the scene changes.
    scene.pad(UiButton::Down);
    wait_for(&mut scene, "themesModel", "focused", "1", 1000);
    assert_eq!(scene.property("themesModel", "name"), "Fixture Pane", "a choice's detail is its theme's");
    scene.pad(UiButton::Right);
    until(&mut scene, 2, "the variant stored", |_| settings(&platform)["BigPicture"][&key]["Variant"] == "night");
    until(&mut scene, 10, "the second variant drawn", |s| box_of(s, "element-night").is_some() && box_of(s, "element-day").is_none());
    assert_eq!(scene.property("library", "themed"), "true");

    // The colour scheme: Right stores the second, whose variables move the mark; A steps it on, round to the first.
    scene.pad(UiButton::Down);
    wait_for(&mut scene, "themesModel", "focused", "2", 1000);
    scene.pad(UiButton::Right);
    until(&mut scene, 2, "the scheme stored", |_| settings(&platform)["BigPicture"][&key]["ColorScheme"] == "cool");
    until(&mut scene, 10, "the second scheme drawn", |s| box_of(s, "element-scheme").is_some_and(|b| (b.0 - 0.6 * f64::from(W)).abs() < 2.0));
    scene.pad(UiButton::Accept);
    until(&mut scene, 2, "A stepped the scheme", |_| settings(&platform)["BigPicture"][&key]["ColorScheme"] == "warm");
    until(&mut scene, 10, "the first scheme drawn again", |s| box_of(s, "element-scheme").is_some_and(|b| (b.0 - warm.0).abs() < 2.0));
    assert_eq!(settings(&platform)["BigPicture"][&key]["Variant"], "night", "the variant kept");

    // EmuSen's own theme used again: the look stored, the ES-DE view let go, the theme's choices kept for a return.
    scene.pad(UiButton::PageUp);
    wait_for(&mut scene, "themesModel", "name", "EmuSen", 2000);
    assert_eq!(scene.property("themesModel", "usable"), "true", "the reference is not in use while the theme draws the library");
    scene.pad(UiButton::Accept);
    until(&mut scene, 2, "the reference in use", |s| s.property("themesModel", "findings") == "In use.");
    let stored = settings(&platform);
    assert_eq!((stored["LibraryStyle"].as_str(), stored["EmuSenTheme"].as_str()), (Some("Mistress"), Some("emusen")));
    assert_eq!(stored["BigPicture"][&key]["Variant"], "night");
    until(&mut scene, 20, "the theme let go", |s| s.property("library", "themed") == "false" && s.property("esde", "active") == "false");

    // B goes back to Settings, and B there to Home, EmuSen's own.
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Settings", 2000);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
}
