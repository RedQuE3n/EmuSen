//! Eclipse Schema 2 Desktop Face tests (`EmuSen.Theme 1.24`): slots, token layering, interactive views, `R-20261127-10` focus arbitration, and Big Picture parity.

use crate::common::{approved_console_core, draw_covers, sandbox, wait_for};
use crate::eclipse_board;

use core::pin::Pin;
use cxx_qt::casting::Upcast;
use cxx_qt::QObject;
use cxx_qt_lib::QString;
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_kit_core::tokens::Colour;
use emusen_kit_qt::kinds::Form;
use emusen_qt::MAIN_URL;
use emusen_qt_models::themes;
use emusen_qt_test::Scene;
use emusen_theme_host::host::qobject::ThemeHost;
use std::collections::BTreeMap;

fn engine_tokens(scene: &mut Scene<'_>, host: &str) -> BTreeMap<String, String> {
    let object = scene.object(host);
    assert!(!object.is_null(), "{host} missing");
    let host = unsafe { <ThemeHost as Upcast<QObject>>::from_base_ptr(object as *const QObject) } as *mut ThemeHost;
    let engine = unsafe { host.as_ref() }.expect("theme host").engine();
    assert!(!engine.is_null(), "{host:?} has no engine");
    let tokens = emusen_theme_qt::host::kit_tokens(unsafe { Pin::new_unchecked(&mut *engine) }).expect("engine tokens");
    Scene::constants(tokens.upcast_pin()).into_iter().collect()
}

fn program_tokens(scene: &mut Scene<'_>) -> (i32, BTreeMap<String, String>) {
    let tokens = scene.kit_tokens().expect("program tokens");
    let form = tokens.form().repr;
    let map = Scene::constants(tokens.upcast_pin()).into_iter().collect();
    (form, map)
}

fn is_visible(scene: &mut Scene<'_>, name: &str) -> bool {
    let tree = scene.tree();
    tree.nodes.iter().any(|n| n.name == name && n.visible && n.w > 0.0 && n.h > 0.0)
}

fn node_text(scene: &mut Scene<'_>, name: &str) -> String {
    let tree = scene.tree();
    tree.nodes
        .iter()
        .find(|n| n.name == name && n.visible)
        .or_else(|| tree.find_named(name).map(|i| &tree.nodes[i as usize]))
        .and_then(|n| n.text.clone())
        .unwrap_or_default()
}

fn click_named(scene: &mut Scene<'_>, name: &str) {
    let tree = scene.tree();
    let idx = tree
        .nodes
        .iter()
        .rposition(|n| n.name == name && n.visible && n.w > 0.0 && n.h > 0.0)
        .unwrap_or_else(|| panic!("{name} is not visible"));
    let n = &tree.nodes[idx];
    scene.click(n.x + n.w / 2.0, n.y + n.h / 2.0);
}

#[test]
fn qt_eclipse_desktop_face_slots_tokens_and_flows_hold() {
    let world = sandbox(
        "qt-app-eclipse-desktop",
        &[
            ("SNES", "Aurora Drift (Synthetic).sfc"),
            ("SNES", "Chrono Harbor (Synthetic).sfc"),
            ("GB", "Laurel Crown (Synthetic).gb"),
        ],
    );
    let cores = world.dir("Cores");
    approved_console_core(&cores);
    let media = world.dir("Media");
    let mut harness = emusen_qt_test::Harness::new();
    let cover_snes = media.join("snes/covers/Aurora Drift (Synthetic).png");
    let cover_gb = media.join("gb/covers/Laurel Crown (Synthetic).png");
    draw_covers(
        &mut harness,
        &world.root,
        &[
            (cover_snes, (300, 420), 0, "Aurora Drift".into()),
            (cover_gb, (300, 420), 1, "Laurel Crown".into()),
        ],
        |_| {},
    );

    let mut scene = harness.load(MAIN_URL, 1440, 900);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    assert!(scene.set_property("program", "coverStores", media.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("program", "theme", &themes::built_in_folder("eclipse")));
    wait_for(&mut scene, "library", "loading", "false", 15000);
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    let warnings = scene.property("host.theme", "warnings");
    assert!(warnings.is_empty() && scene.property("host.theme", "failed") == "false", "Eclipse loads clean: {warnings}");

    // Switch live into Desktop Mode and verify Schema 2 Desktop slots + token layering.
    assert!(scene.set_property("program", "bigScreen", "false"));
    wait_for(&mut scene, "theme", "hasDesktopFace", "true", 3000);
    wait_for(&mut scene, "theme", "desktopLibrary", "true", 3000);
    assert_eq!(scene.property("theme", "desktopTable"), "true");
    assert_eq!(scene.property("theme", "desktopPlaying"), "true");
    assert_eq!(scene.property("theme", "desktopSettings"), "true");
    assert_eq!(scene.property("desktopLibrary.theme", "active"), "true");
    assert!(!is_visible(&mut scene, "desktopTopBar"), "Reference chrome yields to Eclipse Desktop ribbon");
    assert!(is_visible(&mut scene, "eclipseDesktopRibbon"));
    assert!(is_visible(&mut scene, "eclipseDesktopLibrary"));
    assert!(is_visible(&mut scene, "eclipseDesktopRail"));
    assert!(is_visible(&mut scene, "eclipseDesktopInspector"));
    assert!(is_visible(&mut scene, "eclipseDesktopStatusBar"));
    assert_eq!(node_text(&mut scene, "eclipseDesktopWordmark"), "EmuSen");
    assert_eq!(node_text(&mut scene, "eclipseDesktopBadge"), "ECLIPSE");
    assert_eq!(node_text(&mut scene, "eclipseDesktopLibraryHeading"), "All Games");
    assert!(!node_text(&mut scene, "eclipseInspectorTitle").is_empty(), "Inspector renders selected game");

    let (desk_form, prog_tokens) = program_tokens(&mut scene);
    assert_eq!(desk_form, Form::Desktop.repr, "Kit.Tokens.form is Desktop");
    assert_eq!(prog_tokens.get("rowHeight").map(String::as_str), Some("38"));
    assert_eq!(prog_tokens.get("tileRadius").map(String::as_str), Some("14"));
    assert_eq!(Colour::parse(prog_tokens.get("surface").expect("surface")), Colour::parse("#07080F"));
    let theme_tokens = engine_tokens(&mut scene, "host.theme");
    assert_eq!(theme_tokens.get("rowHeight").map(String::as_str), Some("38"));
    assert_eq!(Colour::parse(theme_tokens.get("accent").expect("accent")), Colour::parse("#F6E7D8"));

    // Sidebar and Inspector toggle buttons stay operable without dropping focus.
    click_named(&mut scene, "eclipseDesktopInspectorBtn");
    wait_for(&mut scene, "desktopModel", "inspectorOpen", "false", 1000);
    click_named(&mut scene, "eclipseDesktopInspectorBtn");
    wait_for(&mut scene, "desktopModel", "inspectorOpen", "true", 1000);

    // Table view + multi-select batch bar.
    click_named(&mut scene, "eclipseDesktopViewTable");
    wait_for(&mut scene, "eclipseDesktopTable", "visible", "true", 2000);
    assert!(is_visible(&mut scene, "eclipseDesktopTableRows"));
    assert!(!is_visible(&mut scene, "eclipseDesktopTableBatchBar"), "Batch bar hidden before marks");
    click_named(&mut scene, "eclipseDesktopTableSelectAll");
    wait_for(&mut scene, "eclipseDesktopTableBatchBar", "visible", "true", 2000);
    let tree = scene.tree();
    assert!(eclipse_board::painted(&tree, "eclipseDesktopTableBatchBar").is_some());

    // Playing view (R-20261127-10): Workspace.drawerOpen arbitrates focus over pure #000000 frame.
    let theme_host_obj = scene.object("host.theme");
    let host_ref = unsafe { <ThemeHost as Upcast<QObject>>::from_base_ptr(theme_host_obj as *const QObject) } as *mut ThemeHost;
    let theme_engine = unsafe { host_ref.as_ref() }.expect("host").engine();
    let mut actions = emusen_theme_qt::host::actions(unsafe { Pin::new_unchecked(&mut *theme_engine) }).expect("Actions");
    let screens_qobj = scene.object("screens") as *const QObject;
    let screens_obj = unsafe { <emusen_qt_models::screens::qobject::Screens as Upcast<QObject>>::from_base_ptr(screens_qobj) }
        as *mut emusen_qt_models::screens::qobject::Screens;
    unsafe { Pin::new_unchecked(&mut *screens_obj) }.play();
    wait_for(&mut scene, "screens", "shown", "Game", 2000);
    wait_for(&mut scene, "eclipseDesktopPlaying", "visible", "true", 2000);
    assert!(scene.set_property("desktopModel", "drawerOpen", "false"));
    wait_for(&mut scene, "game", "activeFocus", "true", 1000);
    assert_eq!(scene.property("theme", "desktopPlayingInputs"), "false");
    assert_eq!(scene.property("chosenTheme", "activeFocus"), "false");
    assert!(is_visible(&mut scene, "eclipsePlayingStrip"));
    assert!(!is_visible(&mut scene, "eclipsePlayingDrawer"));

    assert!(actions.as_mut().workspace_set_drawer(true));
    wait_for(&mut scene, "eclipsePlayingDrawer", "visible", "true", 1000);
    wait_for(&mut scene, "chosenTheme", "activeFocus", "true", 1000);
    assert_eq!(scene.property("theme", "desktopPlayingInputs"), "true");
    assert_eq!(scene.property("game", "activeFocus"), "false");
    assert!(actions.as_mut().workspace_set_drawer(false));
    wait_for(&mut scene, "eclipsePlayingDrawer", "visible", "false", 1000);
    wait_for(&mut scene, "game", "activeFocus", "true", 1000);

    click_named(&mut scene, "eclipsePlayingBackLibrary");
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    wait_for(&mut scene, "eclipseDesktopLibrary", "visible", "true", 2000);
    assert_eq!(scene.property("desktopModel", "view"), "0", "< Library switches back to Library view");
    assert_eq!(scene.property("desktopModel", "drawerOpen"), "false", "< Library closes drawer");
    assert_eq!(scene.property("desktopModel", "gameShown"), "false", "< Library exits Playing view");

    // Shaders quick-door directly from Library opens #20 Shaders inside Settings.
    click_named(&mut scene, "eclipseDesktopNavShaders");
    wait_for(&mut scene, "settingsModel", "shadersShown", "true", 2000);
    wait_for(&mut scene, "shadersNavScope", "activeFocus", "true", 2000);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "settingsModel", "shadersShown", "false", 2000);

    // Unified Settings + Tools sidebar, Shaders #20 door, and Themes view.
    click_named(&mut scene, "eclipseDesktopNavSettings");
    wait_for(&mut scene, "eclipseDesktopSettings", "visible", "true", 2000);
    assert!(is_visible(&mut scene, "eclipseDesktopSettingsRail"));
    assert!(is_visible(&mut scene, "eclipseDesktopSettingsRows"));
    assert!(is_visible(&mut scene, "eclipseDesktopSettingsAside"));
    assert_eq!(scene.property("settingsModel", "themedSettings"), "true");

    assert!(actions.as_mut().open_desktop_door(&QString::from("shaders")), "Eclipse enables Shaders #20 door");
    wait_for(&mut scene, "settingsModel", "shadersShown", "true", 2000);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "settingsModel", "shadersShown", "false", 2000);

    assert!(actions.as_mut().open_desktop_door(&QString::from("themes")), "Themes view opens inside Settings");
    wait_for(&mut scene, "eclipseDesktopThemesRows", "visible", "true", 2000);
    assert!(actions.as_mut().settings_back());
    wait_for(&mut scene, "eclipseDesktopSettingsRows", "visible", "true", 2000);

    // Big Picture toggle strips desktop token layer and unloads desktop slots.
    click_named(&mut scene, "eclipseDesktopBigPictureBtn");
    wait_for(&mut scene, "program", "bigScreen", "true", 2000);
    scene.wait(100);
    assert_eq!(scene.property("desktopLibrary.theme", "active"), "false");
    let (bp_form, bp_tokens) = program_tokens(&mut scene);
    assert_eq!(bp_form, Form::BigScreen.repr, "Kit.Tokens.form reverts to BigScreen");
    assert_eq!(bp_tokens.get("rowHeight").map(String::as_str), Some("56"));

    // Toggle back to Desktop and ensure Desktop overrides return intact.
    assert!(scene.set_property("program", "bigScreen", "false"));
    wait_for(&mut scene, "eclipseDesktopSettings", "visible", "true", 2000);
    let (desk_form_again, desk_tokens_again) = program_tokens(&mut scene);
    assert_eq!(desk_form_again, Form::Desktop.repr);
    assert_eq!(desk_tokens_again.get("rowHeight").map(String::as_str), Some("38"));
    scene.assert_quiet();
}
