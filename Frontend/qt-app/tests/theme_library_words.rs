//! The reference theme with games in the library (EmuSen_Qt.md §3.4.1): Home does not say how a folder is added.

mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_qt::MAIN_URL;

#[test]
fn the_reference_theme_with_games_does_not_say_how_a_folder_is_added() {
    let _world = sandbox("qt-app-theme-games", &[("NES", "One.nes")]);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    assert_eq!(scene.property("shelfAddFolder", "visible"), "false");
}
