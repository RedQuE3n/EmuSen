//! Settings is made only while shown (EmuSen_Qt.md §9.3.2), and opened again from Home it stands on the section and the row
//! it was left on, the frame's focus with the model's.

use crate::common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;

#[test]
fn settings_opened_again_stands_where_it_was_left() {
    let _world = sandbox("qt-app-settings-reopen", &[("NES", "One.nes"), ("NES", "Two.nes")]);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "program", "reading", "false", 10000);
    assert_eq!(scene.property("settingsSlot", "status"), "Null", "no Settings is made behind Home");
    // Home's Settings place, as settings.rs reaches it.
    for _ in 0..5 {
        scene.pad(UiButton::Right);
    }
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Settings", 2000);
    while scene.property("settingsModel", "section") != "2" {
        scene.pad(UiButton::PageDown);
        scene.wait(30);
    }
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Down);
    wait_for(&mut scene, "settingsModel", "focused", "2", 1000);
    assert_eq!(scene.property("settingsLayout", "current"), "2");

    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    wait_for(&mut scene, "settingsSlot", "status", "Null", 1000);
    assert_eq!(scene.property("settingsModel", "focused"), "2", "the model keeps the row while Settings is not made");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Settings", 2000);
    assert_eq!(scene.property("settingsModel", "section"), "2", "the section it was left on");
    assert_eq!(scene.property("settingsLayout", "section"), "2", "and the frame shows it");
    assert_eq!(scene.property("settingsLayout", "current"), "2", "the row it was left on, focused in the frame");
    assert_eq!(scene.property("settingsModel", "focused"), "2", "and not written back as the first row");
    scene.assert_quiet();
}
