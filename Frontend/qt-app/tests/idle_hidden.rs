//! A screen not shown costs no frame (EmuSen_Qt.md §9.3.2): the settings' first read landing after the library, while Home
//! stands still, fills Settings' rows behind it and swaps no frame; `program.reading` is false once every start read has answered.

mod common;

use common::{assert_still_saying, sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_qt::MAIN_URL;

#[test]
fn the_settings_read_after_the_library_swaps_no_frame_behind_home() {
    let _world = sandbox("qt-app-idle-hidden", &[("NES", "One.nes"), ("NES", "Two.nes"), ("GB", "Three.gb")]);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    // No pad of the machine's is read, so Home's status line counts none whatever is plugged in.
    assert!(scene.set_property("program", "onlyPads", "none"));
    let platform_library = library();
    let platform_library = platform_library.to_str().unwrap();

    // The library alone first, and Home still over it; then the rest of the start, in the order a slow settings read lands.
    assert!(scene.set_property("library", "platformLibrary", platform_library));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    assert_eq!(scene.property("settingsModel", "ready"), "false", "the settings are not read before the program is named");
    assert_eq!(scene.property("screens", "shown"), "Home");
    assert_still_saying(&mut scene, "Home, the library read");
    let before = scene.frames();
    assert!(scene.set_property("program", "platformLibrary", platform_library));
    wait_for(&mut scene, "program", "reading", "false", 10000);
    for (object, property, value) in [("library", "loading", "false"), ("settingsModel", "ready", "true"), ("pads", "answered", "true"), ("pads", "connected", "0")] {
        assert_eq!(scene.property(object, property), value, "{object}.{property} once the start's reads are in");
    }
    assert_still_saying(&mut scene, "Home, the settings read behind it");
    let swapped = scene.frames() - before;
    assert_eq!(swapped, 0, "Home: {swapped} frames swapped as the settings were read behind it");
    scene.assert_quiet();
}
