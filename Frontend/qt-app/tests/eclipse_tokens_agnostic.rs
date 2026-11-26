//! Contrasting light-theme token override proof across program-owned Eclipse screens (R-20261125-18).
#![allow(dead_code)]

mod common;
mod eclipse_board;
mod eclipse_pad;
mod eclipse_settings_oracle;

use crate::common::{sandbox, wait_for};
use crate::eclipse_board::{painted, read};
use crate::eclipse_pad::{focused, press};
use crate::eclipse_settings_oracle::{at, centre, settings, settle};
use emusen_client_fixture::library;
use emusen_kit_core::input::{PadControl, UiButton};
use emusen_kit_core::tokens::Colour;
use emusen_qt::MAIN_URL;
use emusen_qt_test::virtual_pad::Sdl;
use emusen_qt_test::Scene;
use std::path::{Path, PathBuf};

const WORLD_JSON: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/eclipse/settings_s6_world.json");

fn copy_dir_all(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).expect("create theme subdir");
    for entry in std::fs::read_dir(src).expect("read theme dir") {
        let entry = entry.expect("dir entry");
        let target = dst.join(entry.file_name());
        if entry.file_type().expect("file type").is_dir() {
            copy_dir_all(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copy theme file");
        }
    }
}

fn write_light_eclipse_theme(root: &Path) -> PathBuf {
    let dest = root.join("light-eclipse");
    std::fs::create_dir_all(&dest).expect("create light theme dir");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../themes/eclipse");
    for entry in ["assets", "parts", "screens", "sounds"] {
        copy_dir_all(&src.join(entry), &dest.join(entry));
    }
    let mut doc: serde_json::Value = serde_json::from_str(include_str!("../../themes/eclipse/theme.json")).expect("parse eclipse theme.json");
    let dark = doc["tokens"]["dark"].as_object_mut().expect("eclipse dark tokens map");
    let overrides = [
        ("surface", "#F6F1E7"),
        ("inputSurface", "#ECE5D8"),
        ("surfaceRaised", "#E2D9C8"),
        ("text", "#181512"),
        ("muted", "#5A5248"),
        ("accent", "#C85A32"),
        ("overAccent", "#FFFFFF"),
        ("coronaColor", "#C85A32"),
        ("border", "#9A8F82"),
        ("glowColor", "#66C85A32"),
    ];
    for (key, value) in overrides {
        dark.insert(key.to_owned(), serde_json::json!(value));
    }
    dark.insert("coronaRim".to_owned(), serde_json::json!(2));
    std::fs::write(dest.join("theme.json"), serde_json::to_string_pretty(&doc).unwrap()).expect("write light theme.json");
    dest
}

fn click_named(scene: &mut Scene<'_>, name: &str) {
    let (x, y) = centre(scene, name);
    scene.click(x, y);
}

fn assert_light_paint(tree: &emusen_qt_test::tree::Tree, name: &str, expected: &str, context: &str) {
    let got = painted(tree, name).unwrap_or_else(|| panic!("{context}: {name} is not painted"));
    let want = Colour::parse(expected).expect("valid expected colour");
    let eclipse_surface = Colour::parse("#07080F").unwrap();
    let eclipse_accent = Colour::parse("#F6E7D8").unwrap();
    assert_ne!(got, eclipse_surface, "{context}: {name} stayed Eclipse surface #07080F");
    assert_ne!(got, eclipse_accent, "{context}: {name} stayed Eclipse accent #F6E7D8");
    assert_eq!(got, want, "{context}: {name} painted {got:?}, expected {expected}");
}

fn assert_light_border(tree: &emusen_qt_test::tree::Tree, name: &str, expected: &str, context: &str) {
    let raw = read(tree, name, "border.color");
    let got = Colour::parse(&raw).unwrap_or_else(|| panic!("{context}: {name} border.color missing ({raw})"));
    let want = Colour::parse(expected).expect("valid expected border colour");
    assert_eq!(got, want, "{context}: {name} border.color {got:?}, expected {expected}");
}

#[test]
fn contrasting_light_theme_reskins_program_eclipse_screens() {
    std::env::set_var("LC_ALL", "en_US.UTF-8");
    let world = sandbox("qt-app-eclipse-tokens-agnostic", &[("SNES", "Aurora Drift.sfc"), ("psx", "Tower Set.iso")]);
    let light_dir = write_light_eclipse_theme(&world.root);

    let sdl = Sdl::open();
    let pad = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "settingsModel", "ready", "true", 5000);
    wait_for(&mut scene, "pads", "connected", "1", 5000);
    scene.run_pad_clock();
    assert!(scene.set_property("program", "theme", light_dir.to_str().unwrap()));
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    assert_eq!(scene.property("theme", "failed"), "false", "light theme warnings: {}", scene.property("theme", "warnings"));
    wait_for(&mut scene, "theme", "home", "true", 5000);
    wait_for(&mut scene, "screens", "shown", "Home", 3000);
    scene.wait(300);

    // SearchEclipse under light theme overrides.
    click_named(&mut scene, "search");
    wait_for(&mut scene, "screens", "shown", "Search", 3000);
    scene.wait(200);
    for ch in [0x41, 0x55, 0x52] {
        scene.key(ch, 0);
    }
    for _ in 0..40 {
        let t = scene.tree();
        if t.nodes.iter().any(|n| n.name == "searchResultBg.0" && n.visible) {
            break;
        }
        scene.wait(50);
    }
    scene.wait(120);
    let search_tree = scene.tree();
    assert_light_paint(&search_tree, "searchInputBg", "#FFECE5D8", "SearchEclipse");
    assert_light_paint(&search_tree, "searchQueryText", "#FF181512", "SearchEclipse");
    assert_light_paint(&search_tree, "searchCount", "#FF5A5248", "SearchEclipse");
    assert_light_paint(&search_tree, "searchResultBg.0", "#FFECE5D8", "SearchEclipse");
    assert_light_border(&search_tree, "searchResultBg.0", "#619A8F82", "SearchEclipse");
    assert_light_paint(&search_tree, "searchResultTitle.0", "#FF181512", "SearchEclipse");
    assert_light_paint(&search_tree, "searchResultMeta.0", "#FF5A5248", "SearchEclipse");
    assert_light_paint(&search_tree, "searchHeading", "#FF39342D", "SearchEclipse");
    assert_light_paint(&search_tree, "searchKeyBg.1", "#FFE2D9C8", "SearchEclipse");
    assert_light_paint(&search_tree, "searchKeyText.1", "#FF181512", "SearchEclipse");
    assert_light_paint(&search_tree, "searchKeyBg.33", "#FFC85A32", "SearchEclipse");
    assert_light_paint(&search_tree, "searchKeyText.33", "#FFFFFFFF", "SearchEclipse");
    assert_light_paint(&search_tree, "searchHelp", "#FF39342D", "SearchEclipse");

    // CollectionsEclipse via Back x4 (erasing "AUR" then closing Search) -> Home -> Consoles -> RB.
    for _ in 0..4 {
        scene.pad(UiButton::Back);
        scene.wait(80);
    }
    wait_for(&mut scene, "screens", "shown", "Home", 3000);
    click_named(&mut scene, "nav.consoles");
    wait_for(&mut scene, "screens", "shown", "Consoles", 3000);
    scene.wait(200);
    scene.pad(UiButton::PageDown);
    wait_for(&mut scene, "screens", "shown", "Collections", 3000);
    scene.wait(200);
    let collections_tree = scene.tree();
    assert_light_paint(&collections_tree, "collectionCardBg.0", "#FFECE5D8", "CollectionsEclipse");
    assert_light_border(&collections_tree, "collectionCardBg.0", "#619A8F82", "CollectionsEclipse");
    assert_light_paint(&collections_tree, "collectionCardTitle.0", "#FF181512", "CollectionsEclipse");
    assert_light_paint(&collections_tree, "collectionCardMeta.0", "#FF5A5248", "CollectionsEclipse");
    assert_light_paint(&collections_tree, "newCollectionLabel", "#FF39342D", "CollectionsEclipse");
    assert_light_paint(&collections_tree, "collectionsHelp", "#FF39342D", "CollectionsEclipse");

    // ToolsEclipse via RB from Collections.
    scene.pad(UiButton::PageDown);
    wait_for(&mut scene, "screens", "shown", "Tools", 3000);
    scene.wait(200);
    let tools_tree = scene.tree();
    assert_light_paint(&tools_tree, "toolIconCircle.0", "#FFF6F1E7", "ToolsEclipse");
    assert_light_paint(&tools_tree, "toolsHeading", "#FF181512", "ToolsEclipse");
    assert_light_paint(&tools_tree, "toolsSubtitle", "#FF5A5248", "ToolsEclipse");
    assert_light_paint(&tools_tree, "toolCardBg.0", "#FFECE5D8", "ToolsEclipse");
    assert_light_border(&tools_tree, "toolCardBg.0", "#619A8F82", "ToolsEclipse");
    assert_light_paint(&tools_tree, "toolCardTitle.0", "#FF181512", "ToolsEclipse");
    assert_light_paint(&tools_tree, "toolCardLine.0", "#FF39342D", "ToolsEclipse");
    assert_light_paint(&tools_tree, "toolsHelp", "#FF39342D", "ToolsEclipse");

    // ShadersEclipse via Tools -> RB (Home) -> Menu (Home Card) -> Settings -> Display -> Shaders.
    assert!(scene.set_property("settingsModel", "world", WORLD_JSON));
    wait_for(&mut scene, "settingsModel", "ready", "true", 5000);
    scene.pad(UiButton::PageDown);
    wait_for(&mut scene, "screens", "shown", "Home", 3000);
    scene.wait(200);
    scene.pad(UiButton::Menu);
    wait_for(&mut scene, "screens", "shown", "HomeCard", 2000);
    scene.wait(250);
    for _ in 0..12 {
        let tree = scene.tree();
        let is_settings = scene.focus(&tree).is_some_and(|i| {
            let row_id = &tree.nodes[i as usize].id;
            tree.nodes.iter().any(|n| n.parent == *row_id && n.name == "label" && n.text.as_deref() == Some("Settings"))
        });
        if is_settings {
            break;
        }
        scene.pad(UiButton::Down);
        scene.wait(40);
    }
    scene.pad(UiButton::Accept);
    scene.run_pad_clock();
    wait_for(&mut scene, "screens", "shown", "Settings", 2000);
    for _ in 0..40 {
        if scene.property("theme", "settings") == "true" && focused(&mut scene).starts_with("row.") {
            break;
        }
        scene.wait(50);
    }
    press(&mut scene, &sdl, &pad, PadControl::RightShoulder);
    settle(&mut scene);
    assert_eq!(at(&mut scene).1, "display");
    let shaders_row = settings(&mut scene).shown_rows().iter().position(|r| r.id == "shaders").expect("shaders row");
    while (at(&mut scene).0 as usize) < shaders_row {
        press(&mut scene, &sdl, &pad, PadControl::Down);
        settle(&mut scene);
    }
    press(&mut scene, &sdl, &pad, PadControl::South);
    wait_for(&mut scene, "settingsModel", "shadersShown", "true", 3000);
    for _ in 0..40 {
        let t = scene.tree();
        if t.nodes.iter().any(|n| n.name == "shadersEclipseCanvas" && n.visible) {
            break;
        }
        scene.wait(50);
    }
    scene.wait(200);
    let shaders_tree = scene.tree();
    assert_light_paint(&shaders_tree, "backdropGround", "#FFF6F1E7", "ShadersEclipse");
    assert_light_paint(&shaders_tree, "shadersMark", "#FFC85A32", "ShadersEclipse");
    assert_light_paint(&shaders_tree, "crumbSettings", "#FF5A5248", "ShadersEclipse");
    assert_light_paint(&shaders_tree, "crumbScreenFilter", "#FF181512", "ShadersEclipse");
    assert_light_paint(&shaders_tree, "sourceSwitch", "#FFECE5D8", "ShadersEclipse");
    assert_light_border(&shaders_tree, "sourceSwitch", "#619A8F82", "ShadersEclipse");
    assert_light_paint(&shaders_tree, "packCard", "#FFECE5D8", "ShadersEclipse");
    assert_light_paint(&shaders_tree, "packTitle", "#FF181512", "ShadersEclipse");
    assert_light_paint(&shaders_tree, "packStatus", "#FF39342D", "ShadersEclipse");
    assert_light_paint(&shaders_tree, "searchBar", "#FFECE5D8", "ShadersEclipse");
    assert_light_paint(&shaders_tree, "searchText", "#FF5A5248", "ShadersEclipse");
    assert_light_paint(&shaders_tree, "scopeSwitch", "#FFECE5D8", "ShadersEclipse");
    assert_light_paint(&shaders_tree, "previewGlow", "#38C85A32", "ShadersEclipse");
    assert_light_paint(&shaders_tree, "previewDivider", "#FFC85A32", "ShadersEclipse");
    assert_light_paint(&shaders_tree, "previewHandle", "#FFF6F1E7", "ShadersEclipse");
    assert_light_paint(&shaders_tree, "previewAfterPill", "#FFC85A32", "ShadersEclipse");
    assert_light_paint(&shaders_tree, "previewAfterWords", "#FFFFFFFF", "ShadersEclipse");
    assert_light_paint(&shaders_tree, "presetHeading", "#FF181512", "ShadersEclipse");
    assert_light_paint(&shaders_tree, "presetSubtitle", "#FF39342D", "ShadersEclipse");
    assert_light_paint(&shaders_tree, "help", "#FF39342D", "ShadersEclipse");
}
