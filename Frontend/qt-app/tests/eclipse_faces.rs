//! The face tokens in the program (EmuSen_Kit.md §7.117): under the reference each role draws the face it drew, under Eclipse
//! Sora and JetBrains Mono for codes, read as the family Qt matched for each text; one test, as the harness is one per process.

mod common;

use common::{eclipse_reference, sandbox, wait_for};
use core::pin::Pin;
use cxx_qt::casting::Upcast;
use cxx_qt::QObject;
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;
use emusen_qt_test::Scene;
use emusen_theme_host::host::qobject::ThemeHost;
use std::collections::BTreeMap;

/// The kit's tokens of a theme host's engine, by the host's objectName, as Qt writes each as text.
fn engine_tokens(scene: &mut Scene<'_>, host: &str) -> BTreeMap<String, String> {
    let object = scene.object(host);
    assert!(!object.is_null(), "{host} is made");
    let host = unsafe { <ThemeHost as Upcast<QObject>>::from_base_ptr(object as *const QObject) } as *mut ThemeHost;
    let engine = unsafe { host.as_ref() }.expect("a theme host").engine();
    assert!(!engine.is_null(), "the host has its engine");
    let tokens = emusen_theme_qt::host::kit_tokens(unsafe { Pin::new_unchecked(&mut *engine) }).expect("the engine's tokens");
    Scene::constants(tokens.upcast_pin()).into_iter().collect()
}

/// A text's face as it is drawn: the family asked for and the family Qt matched (`QFontInfo`), by the item's objectName, or
/// by a row's label words for a settings row's `label` or `value`.
fn drawn(tree: &emusen_qt_test::Tree, name: &str, row: Option<&str>) -> (String, String) {
    let node = match row {
        None => tree.nodes.iter().find(|n| n.name == name && n.visible),
        Some(label) => {
            let at = tree.nodes.iter().find(|n| n.name == "label" && n.visible && n.text.as_deref() == Some(label)).unwrap_or_else(|| panic!("the {label} row"));
            tree.nodes.iter().find(|n| n.name == name && n.parent == at.parent)
        }
    };
    let node = node.unwrap_or_else(|| panic!("{name} ({row:?}) is drawn"));
    (node.face.clone().unwrap_or_default(), node.drawn.clone().unwrap_or_default())
}

/// The weights the row's label, its value and the pane's sentence are drawn at (QFontInfo), as CSS weights.
fn weights(tree: &emusen_qt_test::Tree, row: &str) -> (f64, f64, f64) {
    let at = tree.nodes.iter().find(|n| n.name == "label" && n.visible && n.text.as_deref() == Some(row)).unwrap_or_else(|| panic!("the {row} row"));
    let value = tree.nodes.iter().find(|n| n.name == "value" && n.parent == at.parent).expect("the row's value");
    let sentence = tree.nodes.iter().find(|n| n.name == "settingsSentence" && n.visible).expect("the pane's sentence");
    (at.weight, value.weight, sentence.weight)
}

/// The weight a named text is drawn at.
fn weight_of(tree: &emusen_qt_test::Tree, name: &str) -> f64 {
    tree.nodes.iter().find(|n| n.name == name && n.visible).unwrap_or_else(|| panic!("{name} is drawn")).weight
}

/// Settings ▸ Library from Home, its rows shown.
fn open_library(scene: &mut Scene<'_>) {
    for _ in 0..20 {
        let tree = scene.tree();
        if scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).as_deref() == Some("place.0") {
            break;
        }
        scene.wait(50);
    }
    for _ in 0..5 {
        scene.pad(UiButton::Right);
    }
    scene.pad(UiButton::Accept);
    wait_for(scene, "screens", "shown", "Settings", 2000);
    for _ in 0..4 {
        scene.pad(UiButton::PageDown);
    }
    wait_for(scene, "settingsModel", "section", "4", 1000);
}

/// From the Library's first row, Themes, Eclipse used, and back to the Library's rows.
fn use_eclipse(scene: &mut Scene<'_>) {
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Down);
    wait_for(scene, "settingsModel", "focused", "2", 1000);
    scene.pad(UiButton::Accept);
    wait_for(scene, "screens", "shown", "Themes", 2000);
    wait_for(scene, "themesModel", "name", "EmuSen", 5000);
    scene.pad(UiButton::Down);
    wait_for(scene, "themesModel", "name", "Eclipse", 2000);
    scene.pad(UiButton::Accept);
    wait_for(scene, "themesModel", "findings", "In use.", 3000);
    wait_for(scene, "host.theme", "ready", "true", 3000);
    scene.pad(UiButton::Back);
    wait_for(scene, "screens", "shown", "Settings", 2000);
}

#[test]
fn kit_7_117_eclipse_draws_sora_and_its_codes_in_jetbrains_mono_and_the_reference_its_own_faces() {
    let _world = sandbox("qt-app-eclipse-faces", &[("NES", "One.nes")]);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    let platform = emusen_kit_qt::tokens::platform_family();
    assert!(!platform.is_empty(), "the application's font names a family");

    // The reference: its words in Barlow, its labels in Barlow Condensed, the kit's rows and codes in the application's font.
    let home = scene.tree();
    assert_eq!(drawn(&home, "wordmark", None), ("Barlow Condensed".into(), "Barlow Condensed".into()), "the reference's wordmark");
    assert_eq!(drawn(&home, "homeStatus", None), ("Barlow".into(), "Barlow".into()), "the reference's status line");
    assert_eq!(weight_of(&home, "wordmark"), 300.0, "the reference's wordmark, light");
    open_library(&mut scene);
    let rows = scene.tree();
    let (asked, label) = drawn(&rows, "label", Some("ROM folder"));
    assert_eq!(asked, platform, "the reference's row asks for the application's font, as a text naming none does");
    assert_eq!(drawn(&rows, "value", Some("ROM folder")), (platform.clone(), label.clone()), "the reference's path in the same face");
    assert_eq!(weights(&rows, "ROM folder"), (400.0, 400.0, 400.0), "the reference's label, value and sentence, regular");

    // Eclipse: every role in Sora but the codes, in JetBrains Mono; the families drawn, not only those asked for.
    use_eclipse(&mut scene);
    // Eclipse's own Settings (EmuSen_Qt.md §3.4.11): the Library's first row, the ROM folder, in Sora, its path in JetBrains Mono.
    wait_for(&mut scene, "theme", "settings", "true", 2000);
    scene.wait(300);
    let own = scene.tree();
    let rom = own.nodes.iter().find(|n| n.name.starts_with("rowName.") && n.visible && n.text.as_deref() == Some("ROM folder")).expect("Eclipse's ROM folder row").name.replace("rowName.", "");
    // A Line's words are named as the Line is: the Text, the last of the name in the tree, holds the face.
    let face = |name: String| own.nodes.iter().rev().find(|n| n.name == name && n.visible).map(|n| (n.face.clone().unwrap_or_default(), n.drawn.clone().unwrap_or_default())).unwrap_or_default();
    assert_eq!(face(format!("rowName.{rom}")), ("Sora".into(), "Sora".into()), "Eclipse's own settings row");
    assert_eq!(face(format!("rowWords.{rom}")), ("JetBrains Mono".into(), "JetBrains Mono".into()), "Eclipse's own path, a code");
    // The program's own Settings under Eclipse's faces: a package of the reference's screens, which draws no Settings, with Eclipse's tokens.
    let folder = eclipse_reference();
    assert!(scene.set_property("program", "theme", folder.to_str().unwrap()));
    wait_for(&mut scene, "theme", "home", "true", 5000);
    wait_for(&mut scene, "settingsSlot", "active", "true", 2000);
    scene.wait(300);
    let rows = scene.tree();
    assert_eq!(drawn(&rows, "label", Some("ROM folder")), ("Sora".into(), "Sora".into()), "Eclipse's settings row");
    assert_eq!(drawn(&rows, "value", Some("ROM folder")), ("JetBrains Mono".into(), "JetBrains Mono".into()), "Eclipse's path, a code");
    // The boards: a value 400 (EclipseSettings:47), the pane's sentence 300 (EclipseSettings:75); a row's label keeps the
    // reference's, the boards disagreeing (EmuSen_Kit.md §7.117).
    assert_eq!(weights(&rows, "ROM folder"), (400.0, 400.0, 300.0), "Eclipse's label, value and sentence");
    assert_eq!(drawn(&rows, "label", Some("Themes")), ("Sora".into(), "Sora".into()), "a row whose value is no code");
    assert_ne!(drawn(&rows, "value", Some("Themes")).1, "JetBrains Mono", "a value that is no code is not in the codes' face");
    let faces = engine_tokens(&mut scene, "host.theme");
    assert_eq!((faces["faceText"].as_str(), faces["faceLabel"].as_str()), ("Sora", "Sora"), "Eclipse's engine gives its own texts Sora");

    // A theme's own text under Eclipse's faces: the reference's screens in a package that takes Eclipse's tokens, made from
    // the two built-ins' files, so the reference's Home is drawn on an engine holding Eclipse's overrides.
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    scene.wait(200);
    let home = scene.tree();
    assert_eq!(drawn(&home, "wordmark", None), ("Sora".into(), "Sora".into()), "a theme's wordmark under Eclipse's faces");
    assert_eq!(drawn(&home, "homeStatus", None), ("Sora".into(), "Sora".into()), "a theme's status line under Eclipse's faces");
    assert_eq!(weight_of(&home, "wordmark"), 600.0, "the wordmark at the boards' 600 (Main:31)");

    // The reference again: each role as it was.
    assert!(scene.set_property("program", "theme", ""));
    wait_for(&mut scene, "chosenTheme", "status", "Null", 5000);
    scene.wait(200);
    let home = scene.tree();
    assert_eq!(drawn(&home, "wordmark", None), ("Barlow Condensed".into(), "Barlow Condensed".into()), "the reference's wordmark again");
    assert_eq!(drawn(&home, "homeStatus", None), ("Barlow".into(), "Barlow".into()), "the reference's status line again");
    assert_eq!(weight_of(&home, "wordmark"), 300.0, "the reference's wordmark light again");
    let _ = std::fs::remove_dir_all(&folder);
}
