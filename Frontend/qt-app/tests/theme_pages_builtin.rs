//! Under a chosen theme that draws neither Settings nor a game's page, the feed never answers Settings' model (EmuSen_Qt.md §3.2, the proof paragraph).

mod common;

use common::{eclipse_reference, sandbox, wait_for};
use cxx_qt::casting::Upcast;
use emusen_client_fixture::library;
use emusen_qt::MAIN_URL;
use emusen_qt_models::library_model::qobject::QAbstractListModel;
use emusen_qt_models::settings_model::qobject::SettingsModel;
use std::pin::Pin;

#[test]
fn qt_3_2_a_chosen_theme_that_supplies_neither_screen_leaves_settings_to_the_program() {
    let _world = sandbox("qt-app-theme-pages-builtin", &[("NES", "One.nes"), ("NES", "Two.nes")]);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    // No built-in but the reference, which takes no chosen host, now lacks Settings: a package of its screens, through the chosen host as Eclipse once was.
    let folder = eclipse_reference();
    assert!(scene.set_property("program", "theme", folder.to_str().unwrap()));
    wait_for(&mut scene, "host.theme", "ready", "true", 3000);
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "settingsModel", "ready", "true", 5000);
    scene.wait(300);
    assert_eq!(scene.property("theme", "settings"), "false", "the package draws no Settings");
    assert_eq!(scene.property("theme", "game"), "false", "nor a game's page");
    assert_eq!(scene.property("screens", "shown"), "Home");

    // A refocus told while Settings is not made, as a read landing tells one: no one writes the model's row.
    let object = scene.object("settingsModel");
    assert!(!object.is_null());
    // Safety: the window's model of that name is a SettingsModel, alive while the window is, and pinned where QML made it.
    let settings = unsafe { Pin::new_unchecked(&mut *(SettingsModel::from_base_ptr(object as *const QAbstractListModel) as *mut SettingsModel)) };
    let before = scene.property("settingsModel", "focused");
    let other = if before == "1" { 2 } else { 1 };
    settings.refocus(other);
    scene.wait(100);
    assert_eq!(scene.property("settingsModel", "focused"), before, "the row Settings reopens on is the one it was left on");
    scene.assert_quiet();
    let _ = std::fs::remove_dir_all(&folder);
}
