//! Under a theme that draws no Settings, the feed never answers Settings' model (EmuSen_Qt.md §3.2, the proof paragraph).

mod common;

use common::{sandbox, wait_for};
use cxx_qt::casting::Upcast;
use emusen_client_fixture::library;
use emusen_qt::MAIN_URL;
use emusen_qt_models::library_model::qobject::QAbstractListModel;
use emusen_qt_models::settings_model::qobject::SettingsModel;
use emusen_qt_models::themes;
use std::pin::Pin;

#[test]
fn qt_3_2_a_built_in_theme_leaves_settings_to_the_program() {
    let _world = sandbox("qt-app-theme-pages-builtin", &[("NES", "One.nes"), ("NES", "Two.nes")]);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("program", "theme", &themes::built_in_folder("eclipse")));
    wait_for(&mut scene, "host.theme", "ready", "true", 3000);
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "settingsModel", "ready", "true", 5000);
    scene.wait(300);
    assert_eq!(scene.property("theme", "settings"), "false", "Eclipse draws no Settings");
    // Eclipse supplies the game page from EmuSen.Theme 1.11 (R-20261116-27, EmuSen_Qt.md §3.4.11).
    assert_eq!(scene.property("theme", "game"), "true", "a game's page is Eclipse's");
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
}
