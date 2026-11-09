//! The program's feed of an EmuSen theme (EmuSen_Qt.md §3.5.2): the theme's engine reads the program's library, places,
//! time, session and reduced motion through EmuSen.Theme, hears the pad through its own router, and its Actions are the
//! program's. One Qt program a process, so one test a file.

mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;

const FED: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/themes/fed");

#[test]
fn qt_3_5_2_the_feed_hands_a_theme_the_program_s_views_and_carries_its_actions() {
    let _world = sandbox("qt-app-theme-feed", &[("NES", "One.nes")]);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("program", "theme", FED));
    wait_for(&mut scene, "library", "loading", "false", 10000);

    // The library, Home's places, the time and the session as the program's models hold them.
    wait_for(&mut scene, "fedConsoles", "text", "1", 2000);
    assert_eq!(scene.property("fedPlaces", "text"), "7");
    assert_eq!(scene.property("fedClock", "text"), scene.property("clock", "time"));
    assert_eq!(scene.property("fedSession", "text"), "false");
    assert!(scene.set_property("program", "reducedMotion", "true"));
    assert_eq!(scene.property("fedMotion", "text"), "true", "the theme's engine's tokens follow the program's");

    // The pad reaches the theme's screen through its engine's router, and its action is the program's.
    wait_for(&mut scene, "fedGames", "activeFocus", "true", 2000);
    scene.pad(UiButton::Accept);
    assert_eq!(scene.property("fedTaken", "text"), "true", "the window took the action");
    assert_eq!(scene.property("screens", "shown"), "Consoles", "Home's Games place opens Choose a console");
    scene.assert_quiet();
}
