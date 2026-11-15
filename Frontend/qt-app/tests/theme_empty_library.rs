//! The reference theme with an empty library (EmuSen_Qt.md §3.4.1): Home and Choose a console say how a folder is added.

mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_qt::MAIN_URL;

#[test]
fn the_reference_theme_with_an_empty_library_says_how_a_folder_is_added_on_home_and_choose_a_console() {
    // Decided by the tester, 2026-11-13: the program's own words, as the other built-in themes show them (EmuSen_Qt.md §3.4.1).
    use emusen_qt_test::{excuse::Excuses, mockup};
    let _world = sandbox("qt-app-theme-empty", &[]);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    let words = emusen_qt_models::words::ADD_FOLDER;
    assert_eq!((scene.property("shelfAddFolder", "visible").as_str(), scene.property("shelfAddFolder", "text").as_str()), ("true", words));
    let (mut excuses, mut report) = (Excuses::new(&[]).listing("qt-app-theme-empty"), Vec::new());
    common::hold(&mut scene, "HomeEmptyLibrary", &mockup::Mockup::default(), &mut excuses, &mut report);

    // Games, the first place, holds the focus and opens Choose a console, empty, its words and the same line under them.
    for _ in 0..40 {
        let tree = scene.tree();
        if tree.find_named("place.0").is_some_and(|games| scene.focus(&tree) == Some(games)) {
            break;
        }
        scene.wait(50);
    }
    scene.pad(emusen_kit_core::input::UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Consoles", 1000);
    assert_eq!(scene.property("noConsoles", "visible"), "true");
    assert_eq!((scene.property("noConsolesAddFolder", "visible").as_str(), scene.property("noConsolesAddFolder", "text").as_str()), ("true", words));
    common::hold(&mut scene, "ConsolesEmptyLibrary", &mockup::Mockup::default(), &mut excuses, &mut report);
    assert!(report.is_empty(), "{}", report.join("\n"));
}
