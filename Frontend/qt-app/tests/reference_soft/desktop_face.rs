//! Phase 4B-1 verification: dual-face host wiring, Reference Desktop screens, Schema 1 & 2 reskinning, F10 hotkey, and ES-DE toggle.


#[allow(unused_imports)]
use crate::common;
#[allow(unused_imports)]
use crate::eclipse_board;

use common::{sandbox, wait_for};
use core::pin::Pin;
use cxx_qt::casting::Upcast;
use cxx_qt::QObject;
use cxx_qt_lib::{QMap, QMapPair_QString_QVariant, QString, QVariant};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_kit_core::tokens::Colour;
use emusen_kit_qt::kinds::Form;
use emusen_qt::start::{self, Start};
use emusen_qt::MAIN_URL;
use emusen_qt_test::Scene;
use emusen_theme_host::host::qobject::ThemeHost;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const KEY_F10: i32 = 0x01000039;

fn copy_dir_all(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).expect("create dir");
    for entry in std::fs::read_dir(src).expect("read dir") {
        let entry = entry.expect("entry");
        let target = dst.join(entry.file_name());
        if entry.file_type().expect("type").is_dir() {
            copy_dir_all(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copy file");
        }
    }
}

fn write_schema1_theme(root: &Path, dir_name: &str, light_variant: bool) -> PathBuf {
    let dest = root.join(dir_name);
    std::fs::create_dir_all(&dest).expect("create schema1 theme dir");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../themes/eclipse");
    for entry in ["assets", "parts", "screens", "sounds"] {
        copy_dir_all(&src.join(entry), &dest.join(entry));
    }
    let mut doc: serde_json::Value = serde_json::from_str(include_str!("../../../themes/eclipse/theme.json")).expect("parse eclipse theme.json");
    if let Some(map) = doc.as_object_mut() {
        map.insert("schema".into(), serde_json::json!(1));
        map.remove("faces");
        if !map.contains_key("screens") {
            let bp = serde_json::json!(["home", "consoles", "games", "settings", "game"]);
            map.insert("screens".into(), bp);
        }
    }
    let dark = doc["tokens"]["dark"].as_object_mut().expect("dark token map");
    if let Some(d) = dark.get("desktop").cloned() {
        let _ = d;
        dark.remove("desktop");
    }
    if light_variant {
        dark.insert("voidFill".into(), serde_json::json!("#F6F1E7"));
        dark.insert("surface".into(), serde_json::json!("#F6F1E7"));
        dark.insert("inputSurface".into(), serde_json::json!("#ECE5D8"));
        dark.insert("surfaceRaised".into(), serde_json::json!("#E2D9C8"));
        dark.insert("text".into(), serde_json::json!("#181512"));
        dark.insert("muted".into(), serde_json::json!("#5A5248"));
        dark.insert("accent".into(), serde_json::json!("#C85A32"));
        dark.insert("overAccent".into(), serde_json::json!("#FFFFFF"));
        dark.insert("coronaColor".into(), serde_json::json!("#C85A32"));
        dark.insert("border".into(), serde_json::json!("#9A8F82"));
    } else {
        dark.insert("voidFill".into(), serde_json::json!("#07080F"));
        dark.insert("surface".into(), serde_json::json!("#0D1020"));
    }
    std::fs::write(dest.join("theme.json"), serde_json::to_string_pretty(&doc).unwrap()).expect("write schema1 theme.json");
    dest
}

fn write_schema2_dual_theme(root: &Path) -> PathBuf {
    let dest = root.join("schema2-dual");
    std::fs::create_dir_all(dest.join("screens/desktop")).expect("create schema2 desktop screens");
    let manifest = serde_json::json!({
        "schema": 2,
        "api": "1.22",
        "name": "DualSpec",
        "author": "Test",
        "licence": "GPL-3.0-or-later",
        "variants": ["dark"],
        "faces": {
            "bigPicture": ["home", "settings"],
            "desktop": ["library", "table", "settings"]
        },
        "tokens": {
            "dark": {
                "voidFill": "#080A12",
                "surface": "#080A12",
                "accent": "#4FD1C5",
                "rowHeight": 56.0,
                "desktop": {
                    "voidFill": "#101420",
                    "surface": "#101420",
                    "accent": "#F59E0B",
                    "rowHeight": 38.0
                }
            }
        }
    });
    std::fs::write(dest.join("theme.json"), serde_json::to_string_pretty(&manifest).unwrap()).unwrap();
    let bp_home = r#"import QtQuick
import EmuSen.Theme

ThemeScreen {
    role: ThemeScreen.Home
    Rectangle {
        objectName: "dualBpHomeBg"
        anchors.fill: parent
        color: Tokens.surface
        Text {
            objectName: "dualBpSummary"
            text: (Device.bigScreen ? "BIG" : "DESK") + ":" + Device.form + ":" + Tokens.rowHeight
        }
    }
}
"#;
    let bp_settings = r#"import QtQuick
import EmuSen.Theme

ThemeScreen {
    role: ThemeScreen.Settings
    Item { objectName: "dualBpSettings" }
}
"#;
    let desk_lib = r#"import QtQuick
import EmuSen.Theme

ThemeScreen {
    role: ThemeScreen.DesktopLibrary
    Rectangle {
        objectName: "dualDeskLibBg"
        anchors.fill: parent
        color: Tokens.surface
        Text {
            objectName: "dualDeskSummary"
            anchors.centerIn: parent
            text: (Device.bigScreen ? "BIG" : "DESK") + ":" + Device.form + ":" + Tokens.rowHeight
            color: Tokens.accent
        }
        MouseArea {
            anchors.fill: parent
            onClicked: Actions.setBigScreen(true)
        }
    }
}
"#;
    let desk_table = r#"import QtQuick
import EmuSen.Theme

ThemeScreen {
    role: ThemeScreen.DesktopTable
    Rectangle { objectName: "dualDeskTable"; anchors.fill: parent; color: Tokens.surface }
}
"#;
    let desk_settings = r#"import QtQuick
import EmuSen.Theme

ThemeScreen {
    role: ThemeScreen.DesktopSettings
    Rectangle { objectName: "dualDeskSettings"; anchors.fill: parent; color: Tokens.surface }
}
"#;
    std::fs::write(dest.join("screens/Home.qml"), bp_home).unwrap();
    std::fs::write(dest.join("screens/Settings.qml"), bp_settings).unwrap();
    std::fs::write(dest.join("screens/desktop/Library.qml"), desk_lib).unwrap();
    std::fs::write(dest.join("screens/desktop/Table.qml"), desk_table).unwrap();
    std::fs::write(dest.join("screens/desktop/Settings.qml"), desk_settings).unwrap();
    dest
}

fn engine_tokens(scene: &mut Scene<'_>, host_name: &str) -> BTreeMap<String, String> {
    let object = scene.object(host_name);
    assert!(!object.is_null(), "{host_name} exists");
    let host = unsafe { <ThemeHost as Upcast<QObject>>::from_base_ptr(object as *const QObject) } as *mut ThemeHost;
    let engine = unsafe { host.as_ref() }.expect("host").engine();
    assert!(!engine.is_null(), "host engine active");
    let tokens = emusen_theme_qt::host::kit_tokens(unsafe { Pin::new_unchecked(&mut *engine) }).expect("kit tokens");
    Scene::constants(tokens.upcast_pin()).into_iter().collect()
}

fn program_tokens(scene: &mut Scene<'_>) -> (i32, BTreeMap<String, String>) {
    let tokens = scene.kit_tokens().expect("program kit tokens");
    let form = tokens.form().repr;
    let map = Scene::constants(tokens.upcast_pin()).into_iter().collect();
    (form, map)
}

fn node_text(scene: &mut Scene<'_>, name: &str) -> String {
    let tree = scene.tree();
    tree.nodes
        .iter()
        .find(|n| n.name == name)
        .and_then(|n| n.text.clone())
        .unwrap_or_default()
}

fn is_visible(scene: &mut Scene<'_>, name: &str) -> bool {
    let tree = scene.tree();
    tree.nodes.iter().any(|n| n.name == name && n.visible)
}

#[test]
fn start_big_screen_initial_property_follows_desktop_flag() {
    let start = Start {
        platform_library: String::new(),
        splash: false,
        reduced_motion: false,
        game_mode: false,
        guide_chords: false,
        theme: String::new(),
        theme_words: String::new(),
        cover_stores: Vec::new(),
        esde: false,
        navigation_sounds: true,
        full_screen: true,
        big_screen: false,
        windowed: None,
    };
    let initial: QMap<QMapPair_QString_QVariant> = start::initial(&start);
    let value: QVariant = initial.get(&QString::from("bigScreen")).expect("bigScreen initial property");
    assert_eq!(value.value::<bool>(), Some(false));
}

#[test]
fn desktop_face_covers_host_reference_and_schema2() {
    std::env::set_var("LC_ALL", "en_US.UTF-8");
    let world = sandbox(
        "qt-app-desktop-face",
        &[("SNES", "Aurora Drift.sfc"), ("SNES", "Brass Lantern.sfc"), ("psx", "Basalt Kings.iso")],
    );
    let dark_s1 = write_schema1_theme(&world.root, "schema1-dark", false);
    let light_s1 = write_schema1_theme(&world.root, "schema1-light", true);
    let dual_s2 = write_schema2_dual_theme(&world.root);

    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert_eq!(scene.property("program", "bigScreen"), "true", "Big Picture is default");
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "settingsModel", "ready", "true", 5000);
    wait_for(&mut scene, "pads", "answered", "true", 5000);
    assert_eq!(scene.property("pads", "problem"), "", "PadReader hotkeys ready");
    wait_for(&mut scene, "screens", "shown", "Home", 3000);
    scene.wait(150);

    // 1. Default startup state invariants: Big Picture form (1), Desktop shell inactive.
    assert_eq!(scene.property("desktopShellLoader", "active"), "false", "Desktop chrome does not load in Big Picture");
    assert_eq!(scene.property("home", "visible"), "true");
    let lib_ptr = scene.object("library") as usize;
    assert_ne!(lib_ptr, 0);

    // 2. Live switch to Reference Desktop Mode without reloading the library.
    assert!(scene.set_property("program", "bigScreen", "false"));
    wait_for(&mut scene, "desktopShellLoader", "active", "true", 2000);
    wait_for(&mut scene, "desktopLibrary", "visible", "true", 2000);
    scene.wait(120);
    assert_eq!(scene.object("library") as usize, lib_ptr, "LibraryModel persists without remount");
    assert_eq!(scene.property("library", "loading"), "false", "Library does not reload on face toggle");
    assert!(is_visible(&mut scene, "desktopLibrary"), "Desktop Library visible");
    assert_eq!(node_text(&mut scene, "desktopLibraryHeading"), "All Games");
    assert!(!node_text(&mut scene, "desktopInspectorTitle").is_empty(), "Inspector renders real game title");

    // View 0 -> 1 (Table) -> 2 (Settings) -> 0 (Library).
    assert!(scene.set_property("desktopModel", "view", "1"));
    wait_for(&mut scene, "desktopTable", "visible", "true", 2000);
    scene.wait(100);
    assert!(is_visible(&mut scene, "desktopTableRows"), "Table rows visible");
    assert!(scene.set_property("desktopModel", "view", "2"));
    wait_for(&mut scene, "desktopSettings", "visible", "true", 2000);
    scene.wait(100);
    assert!(is_visible(&mut scene, "desktopSettingsRows"), "Settings rows visible");
    assert!(scene.set_property("desktopModel", "view", "0"));
    wait_for(&mut scene, "desktopLibrary", "visible", "true", 2000);

    // 3. Schema 1 Dark and Light reskinning on Reference Desktop chrome.
    assert!(scene.set_property("program", "theme", dark_s1.to_str().unwrap()));
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    scene.wait(120);
    assert_eq!(scene.property("theme", "desktopLibrary"), "false", "Schema 1 has no Desktop slots");
    let dark_tree = scene.tree();
    let dark_bg = eclipse_board::painted(&dark_tree, "desktopWorkspaceBg").expect("dark bg painted");
    assert_eq!(dark_bg, Colour::parse("#FF07080F").unwrap(), "Dark Schema 1 sets Desktop voidFill");

    assert!(scene.set_property("program", "theme", light_s1.to_str().unwrap()));
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    scene.wait(120);
    let light_tree = scene.tree();
    let light_bg = eclipse_board::painted(&light_tree, "desktopWorkspaceBg").expect("light bg painted");
    assert_eq!(light_bg, Colour::parse("#FFF6F1E7").unwrap(), "Light Schema 1 reskins Desktop voidFill");

    // 4. Schema 2 Dual-Face host slots, token layering, and Actions.setBigScreen(true).
    assert!(scene.set_property("program", "theme", dual_s2.to_str().unwrap()));
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    wait_for(&mut scene, "theme", "hasDesktopFace", "true", 2000);
    wait_for(&mut scene, "theme", "desktopLibrary", "true", 2000);
    scene.wait(120);
    assert_eq!(scene.property("theme", "failed"), "false", "Schema 2 warnings: {}", scene.property("theme", "warnings"));
    assert_eq!(node_text(&mut scene, "dualDeskSummary"), "DESK:0:38", "Schema 2 reads desktop form and rowHeight");
    assert!(!is_visible(&mut scene, "desktopTopBar"), "Reference top bar yields to Schema 2 Desktop slot");
    let desk_tokens = engine_tokens(&mut scene, "host.theme");
    assert_eq!(desk_tokens.get("rowHeight").map(String::as_str), Some("38"));
    let desk_surface = Colour::parse(desk_tokens.get("surface").expect("surface")).unwrap();
    assert_eq!(desk_surface, Colour::parse("#FF101420").unwrap());
    let (prog_desk_form, prog_desk_tokens) = program_tokens(&mut scene);
    assert_eq!(prog_desk_form, Form::Desktop.repr, "Program Kit.Tokens.form is Desktop");
    assert_eq!(prog_desk_tokens.get("rowHeight").map(String::as_str), Some("38"), "Program Kit.Tokens receives desktop rowHeight");
    assert_eq!(scene.property("desktopLibrary.theme", "active"), "true");

    assert!(scene.set_property("desktopModel", "view", "2"));
    wait_for(&mut scene, "theme", "desktopSettings", "true", 2000);
    assert!(scene.set_property("desktopModel", "view", "0"));
    wait_for(&mut scene, "theme", "desktopLibrary", "true", 2000);

    // Actions.setBigScreen(true) from Schema 2 screen switches back to Big Picture and removes desktop layer.
    assert!(is_visible(&mut scene, "dualDeskLibBg"));
    scene.click(640.0, 400.0);
    wait_for(&mut scene, "program", "bigScreen", "true", 2000);
    scene.wait(120);
    assert_eq!(scene.property("desktopLibrary.theme", "active"), "false", "Desktop slot unloads lazily in Big Picture");
    let bp_tokens = engine_tokens(&mut scene, "host.theme");
    assert_eq!(bp_tokens.get("rowHeight").map(String::as_str), Some("56"), "Big Picture removes desktop token layer");
    let (prog_bp_form, prog_bp_tokens) = program_tokens(&mut scene);
    assert_eq!(prog_bp_form, Form::BigScreen.repr, "Program Kit.Tokens.form reverts to BigScreen");
    assert_eq!(prog_bp_tokens.get("rowHeight").map(String::as_str), Some("56"), "Program Kit.Tokens reverts rowHeight");
    assert_eq!(node_text(&mut scene, "dualBpSummary"), "BIG:1:56", "Big Picture restores Device.bigScreen and form 1");

    // Toggling back to Desktop re-applies desktop token overrides cleanly on both engines.
    assert!(scene.set_property("program", "bigScreen", "false"));
    wait_for(&mut scene, "theme", "desktopLibrary", "true", 2000);
    scene.wait(100);
    assert_eq!(scene.property("desktopLibrary.theme", "active"), "true");
    let desk_tokens_again = engine_tokens(&mut scene, "host.theme");
    assert_eq!(desk_tokens_again.get("rowHeight").map(String::as_str), Some("38"));
    let (prog_desk_form_again, prog_desk_tokens_again) = program_tokens(&mut scene);
    assert_eq!(prog_desk_form_again, Form::Desktop.repr);
    assert_eq!(prog_desk_tokens_again.get("rowHeight").map(String::as_str), Some("38"));
    assert_eq!(node_text(&mut scene, "dualDeskSummary"), "DESK:0:38");

    // 5. F10 hotkey toggles Desktop <-> Big Picture from Home, Consoles, and Settings.
    assert!(scene.set_property("program", "theme", ""));
    assert!(scene.set_property("program", "bigScreen", "true"));
    wait_for(&mut scene, "home", "visible", "true", 2000);
    scene.wait(100);
    scene.key_hold(KEY_F10, true);
    scene.key_hold(KEY_F10, false);
    wait_for(&mut scene, "program", "bigScreen", "false", 2000);
    wait_for(&mut scene, "desktopLibrary", "visible", "true", 2000);
    scene.key_hold(KEY_F10, true);
    scene.key_hold(KEY_F10, false);
    wait_for(&mut scene, "program", "bigScreen", "true", 2000);
    assert_eq!(scene.property("desktopShellLoader", "active"), "false");

    // Navigate to Consoles in Big Picture, toggle F10 -> Desktop -> F10 -> Consoles, then Settings.
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Consoles", 2000);
    scene.key_hold(KEY_F10, true);
    scene.key_hold(KEY_F10, false);
    wait_for(&mut scene, "program", "bigScreen", "false", 2000);
    wait_for(&mut scene, "desktopLibrary", "visible", "true", 2000);
    scene.key_hold(KEY_F10, true);
    scene.key_hold(KEY_F10, false);
    wait_for(&mut scene, "program", "bigScreen", "true", 2000);
    assert_eq!(scene.property("screens", "shown"), "Consoles", "Returning from Desktop preserves Consoles screen");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);

    for _ in 0..5 {
        scene.pad(UiButton::Right);
    }
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Settings", 2000);
    scene.key_hold(KEY_F10, true);
    scene.key_hold(KEY_F10, false);
    wait_for(&mut scene, "program", "bigScreen", "false", 2000);
    wait_for(&mut scene, "desktopLibrary", "visible", "true", 2000);
    scene.key_hold(KEY_F10, true);
    scene.key_hold(KEY_F10, false);
    wait_for(&mut scene, "program", "bigScreen", "true", 2000);
    assert_eq!(scene.property("screens", "shown"), "Settings", "Returning from Desktop preserves Settings screen");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);

    // 6. ES-DE preference stays preserved when switching to Desktop Mode and back.
    assert!(scene.set_property("program", "esdeChosen", "true"));
    scene.wait(60);
    assert_eq!(scene.property("esde", "active"), "true");
    scene.key_hold(KEY_F10, true);
    scene.key_hold(KEY_F10, false);
    wait_for(&mut scene, "program", "bigScreen", "false", 2000);
    assert_eq!(scene.property("program", "esdeChosen"), "true", "Desktop switch leaves esdeChosen untouched");
    scene.key_hold(KEY_F10, true);
    scene.key_hold(KEY_F10, false);
    wait_for(&mut scene, "program", "bigScreen", "true", 2000);
    assert_eq!(scene.property("program", "esdeChosen"), "true");
    assert!(scene.set_property("program", "esdeChosen", "false"));

    // 7. Optional 1920x1200 visual screenshots of all Reference Desktop views.
    if let Ok(dir) = std::env::var("EMUSEN_DESKTOP_PICTURES") {
        let out = PathBuf::from(dir);
        std::fs::create_dir_all(&out).expect("create EMUSEN_DESKTOP_PICTURES");
        scene.resize(1920, 1200);
        assert!(scene.set_property("program", "bigScreen", "false"));
        wait_for(&mut scene, "desktopLibrary", "visible", "true", 2000);
        scene.wait(180);
        assert!(scene.grab(out.join("01-desktop-library.png").to_str().unwrap()));

        assert!(scene.set_property("desktopModel", "view", "1"));
        wait_for(&mut scene, "desktopTable", "visible", "true", 2000);
        scene.wait(180);
        assert!(scene.grab(out.join("02-desktop-table.png").to_str().unwrap()));

        assert!(scene.set_property("desktopModel", "view", "2"));
        wait_for(&mut scene, "desktopSettings", "visible", "true", 2000);
        scene.wait(180);
        assert!(scene.grab(out.join("03-desktop-settings.png").to_str().unwrap()));

        scene.resize(1280, 800);
    }

    scene.assert_quiet();
}
