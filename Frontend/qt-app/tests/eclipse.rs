//! Eclipse, the second built-in theme (EmuSen_Qt.md §3.4.7): listed after the reference in the Themes tab and chosen there,
//! its token overrides on its own engine and on the program's, every one as `theme.json` writes it; with the reference in
//! use, before and after, the program's tokens are the kit's own, as the reference's engine holds them.

mod common;

use common::{sandbox, wait_for};
use core::pin::Pin;
use cxx_qt::casting::Upcast;
use cxx_qt::QObject;
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_kit_core::tokens::{scaled_text, Colour};
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_models::themes;
use emusen_qt_test::Scene;
use emusen_theme_host::host::qobject::ThemeHost;
use std::collections::BTreeMap;

const ECLIPSE: &str = include_str!("../../themes/eclipse/theme.json");

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

/// The kit's tokens of the program's own engine, which Settings, the cards, the page and the notices read.
fn program_tokens(scene: &mut Scene<'_>) -> BTreeMap<String, String> {
    let tokens = scene.kit_tokens().expect("the program's tokens");
    Scene::constants(tokens.upcast_pin()).into_iter().collect()
}

fn stored(platform: &Platform) -> String {
    let settings = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    settings["EmuSenTheme"].as_str().unwrap_or_default().to_owned()
}

/// Whether a token read as text is the value `theme.json` gives it: a colour as the same colour, a number as the same number,
/// a type size as the kit's size of that base at the text scale in force (`scaled_text`).
fn holds(name: &str, read: &str, given: &str, text_scale: f64) -> bool {
    match (Colour::parse(given), given.parse::<f64>()) {
        (Some(colour), _) => Colour::parse(read) == Some(colour),
        (None, Ok(number)) => {
            let want = if name.starts_with("font") { scaled_text(number, text_scale) } else { number };
            read.parse::<f64>().is_ok_and(|r| (r - want).abs() < 1e-9)
        }
        _ => read == given,
    }
}

#[test]
fn qt_3_4_7_eclipse_is_listed_chosen_and_sets_every_override_on_both_engines() {
    let _world = sandbox("qt-app-eclipse", &[("NES", "One.nes")]);
    let platform = Platform::load(library()).unwrap();
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);

    // With the reference in use, nothing is set over the kit's tokens: the program's are the reference engine's, every one.
    let kit = engine_tokens(&mut scene, "host.reference");
    assert!(kit.len() > 50 && kit.contains_key("surface") && kit.contains_key("fontRow"), "{kit:?}");
    assert_eq!(program_tokens(&mut scene), kit, "the reference changes no token of the program's");

    // Settings ▸ Library ▸ Themes lists the built-ins first: EmuSen in use, then Eclipse, built clean, so Use is offered.
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
    scene.pad(UiButton::Down);
    wait_for(&mut scene, "themesModel", "name", "Eclipse", 2000);
    let api = serde_json::from_str::<serde_json::Value>(ECLIPSE).expect("Eclipse's theme.json")["api"].as_str().unwrap_or_default().to_owned();
    assert_eq!(scene.property("themesModel", "line"), format!("By The EmuSen project · EmuSen.Theme {api} · dark"));
    assert_eq!(scene.property("themesModel", "findings"), "Checked: nothing found.");
    assert_eq!(scene.property("themesModel", "usable"), "true");

    // Use stores its id and loads it from the resources; it draws no screen of its own, so every screen is the reference's.
    scene.pad(UiButton::Accept);
    scene.wait(200);
    assert_eq!(stored(&platform), "eclipse", "the id is stored, never a folder");
    assert_eq!(scene.property("program", "theme"), themes::built_in_folder("eclipse"));
    wait_for(&mut scene, "host.theme", "ready", "true", 3000);
    assert_eq!(scene.property("host.theme", "name"), "Eclipse");
    assert_eq!(scene.property("host.theme", "failed"), "false", "{}", scene.property("host.theme", "warnings"));
    assert_eq!((scene.property("theme", "home").as_str(), scene.property("reference", "home").as_str()), ("false", "true"));
    assert_eq!(scene.property("themesModel", "findings"), "In use.");

    // Every override, on the theme's engine and on the program's, is theme.json's value; every other token is the kit's.
    let package = emusen_theme_host::package::read(ECLIPSE).expect("Eclipse's theme.json is read");
    let overrides = package.overrides();
    assert!(!overrides.is_empty() && overrides.iter().all(|(light, _, _)| !light), "dark overrides only");
    let text_scale = *scene.kit_tokens().expect("the program's tokens").text_scale();
    let theirs = engine_tokens(&mut scene, "host.theme");
    let ours = program_tokens(&mut scene);
    for (engine, tokens) in [("the theme's engine", &theirs), ("the program's engine", &ours)] {
        for (_, name, value) in &overrides {
            let read = tokens.get(name).map(String::as_str).unwrap_or("(no such token)");
            assert!(holds(name, read, value, text_scale), "{engine}: {name} is {read}, theme.json gives {value}");
        }
        for (name, value) in tokens.iter().filter(|(name, _)| !overrides.iter().any(|(_, n, _)| n == *name)) {
            assert_eq!(Some(value), kit.get(name), "{engine}: {name}, which Eclipse leaves, is the kit's");
        }
    }

    // Eclipse's line colour, decided 2026-11-14 by the tester (R-20261114-35): the boards' #5A6076 lifted to #5E647B, which
    // keeps theme-check's 3:1 floor on surfaceRaised and inputSurface.
    for (engine, tokens) in [("the theme's engine", &theirs), ("the program's engine", &ours)] {
        let border = tokens.get("border").map(String::as_str).unwrap_or("(no border)");
        assert_eq!(Colour::parse(border), Colour::parse("#5E647B"), "{engine}: Eclipse's border");
    }

    // The reference again: the program's tokens are the kit's once more, and the chosen theme's host is gone.
    scene.pad(UiButton::Up);
    wait_for(&mut scene, "themesModel", "name", "EmuSen", 2000);
    scene.pad(UiButton::Accept);
    scene.wait(200);
    assert_eq!(stored(&platform), "emusen");
    assert_eq!(scene.property("program", "theme"), "");
    wait_for(&mut scene, "chosenTheme", "status", "Null", 2000);
    assert_eq!(program_tokens(&mut scene), kit, "the reference again leaves the program the kit's tokens");
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

/// Settings ▸ Library ▸ Themes from Home, the theme at `index` in the list used, then back to the Library's rows.
fn use_theme(scene: &mut Scene<'_>, down: usize, name: &str) {
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
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Down);
    wait_for(scene, "settingsModel", "focused", "2", 1000);
    scene.pad(UiButton::Accept);
    wait_for(scene, "screens", "shown", "Themes", 2000);
    wait_for(scene, "themesModel", "name", "EmuSen", 5000);
    for _ in 0..down {
        scene.pad(UiButton::Down);
    }
    wait_for(scene, "themesModel", "name", name, 2000);
    scene.pad(UiButton::Accept);
    wait_for(scene, "themesModel", "findings", "In use.", 3000);
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
    use_theme(&mut scene, 0, "EmuSen");
    let rows = scene.tree();
    let (asked, label) = drawn(&rows, "label", Some("ROM folder"));
    assert_eq!(asked, platform, "the reference's row asks for the application's font, as a text naming none does");
    assert_eq!(drawn(&rows, "value", Some("ROM folder")), (platform.clone(), label.clone()), "the reference's path in the same face");

    // Eclipse: every role in Sora but the codes, in JetBrains Mono; the families drawn, not only those asked for.
    use_theme(&mut scene, 1, "Eclipse");
    wait_for(&mut scene, "host.theme", "ready", "true", 3000);
    let rows = scene.tree();
    assert_eq!(drawn(&rows, "label", Some("ROM folder")), ("Sora".into(), "Sora".into()), "Eclipse's settings row");
    assert_eq!(drawn(&rows, "value", Some("ROM folder")), ("JetBrains Mono".into(), "JetBrains Mono".into()), "Eclipse's path, a code");
    assert_eq!(drawn(&rows, "label", Some("Themes")), ("Sora".into(), "Sora".into()), "a row whose value is no code");
    assert_ne!(drawn(&rows, "value", Some("Themes")).1, "JetBrains Mono", "a value that is no code is not in the codes' face");
    let faces = engine_tokens(&mut scene, "host.theme");
    assert_eq!((faces["faceText"].as_str(), faces["faceLabel"].as_str()), ("Sora", "Sora"), "Eclipse's engine gives its own texts Sora");

    // A theme's own text under Eclipse's faces: the reference's screens in a package that takes Eclipse's tokens, made from
    // the two built-ins' files, so the reference's Home is drawn on an engine holding Eclipse's overrides.
    let folder = eclipse_reference();
    assert!(scene.set_property("program", "theme", folder.to_str().unwrap()));
    wait_for(&mut scene, "theme", "home", "true", 5000);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    scene.wait(200);
    let home = scene.tree();
    assert_eq!(drawn(&home, "wordmark", None), ("Sora".into(), "Sora".into()), "a theme's wordmark under Eclipse's faces");
    assert_eq!(drawn(&home, "homeStatus", None), ("Sora".into(), "Sora".into()), "a theme's status line under Eclipse's faces");

    // The reference again: each role as it was.
    assert!(scene.set_property("program", "theme", ""));
    wait_for(&mut scene, "theme", "home", "false", 5000);
    scene.wait(200);
    let home = scene.tree();
    assert_eq!(drawn(&home, "wordmark", None), ("Barlow Condensed".into(), "Barlow Condensed".into()), "the reference's wordmark again");
    assert_eq!(drawn(&home, "homeStatus", None), ("Barlow".into(), "Barlow".into()), "the reference's status line again");
    let _ = std::fs::remove_dir_all(&folder);
}

/// A package of the reference's screens and parts with Eclipse's token overrides, in a folder of the test's own.
fn eclipse_reference() -> std::path::PathBuf {
    let themes = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../themes");
    let folder = std::env::temp_dir().join(format!("eclipse-reference-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&folder);
    for part in ["screens", "parts"] {
        std::fs::create_dir_all(folder.join(part)).unwrap();
        for file in std::fs::read_dir(themes.join("emusen").join(part)).unwrap().flatten() {
            std::fs::copy(file.path(), folder.join(part).join(file.file_name())).unwrap();
        }
    }
    let reference: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(themes.join("emusen/theme.json")).unwrap()).unwrap();
    let mut manifest: serde_json::Value = serde_json::from_str(ECLIPSE).unwrap();
    manifest["screens"] = reference["screens"].clone();
    std::fs::write(folder.join("theme.json"), manifest.to_string()).unwrap();
    folder
}
