//! One harness test (EmuSen_Qt.md §8.1): BellPlayer made off the GUI thread through qt_thread().queue, every cue played
//! through Endymion's interface player on SDL's dummy audio driver, so nothing reaches the speakers; and the QML rules.

// The module registers through its crate's initializer; the test must link it.
extern crate emusen_qt_endymion;

use emusen_client_fixture::{library, World};
use emusen_kit_core::sound::Cue;
use emusen_qt_test::Harness;

#[test]
fn every_cue_plays_through_the_interface_player() {
    std::env::set_var("SDL_AUDIODRIVER", "dummy");
    let _world = World::new("bells-qml");
    let mut harness = Harness::new();
    let scene_file = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/BellScene.qml");
    let mut scene = harness.load(&format!("file://{scene_file}"), 320, 200);
    assert_eq!(scene.property("scene", "ready"), "false", "no player before the platform library is named");
    assert!(scene.set_property("scene", "platformLibrary", library().to_str().unwrap()));
    let mut waited = 0;
    while scene.property("scene", "ready") != "true" && waited < 5000 {
        scene.wait(50);
        waited += 50;
    }
    assert_eq!(scene.property("scene", "ready"), "true", "the player made: {}", scene.property("scene", "problem"));
    for (i, cue) in Cue::ALL.iter().enumerate() {
        assert!(scene.set_property("scene", "cue", &i.to_string()));
        assert_eq!(scene.property("scene", "problem"), "", "{cue:?} played");
    }
    scene.assert_quiet();
}

#[test]
fn the_scenes_keep_the_qml_rules() {
    let scenes = emusen_qt_test::lint::frontend().join("qt-endymion").join("tests");
    let lines = emusen_qt_test::lint::rules(&[(&scenes, true)]);
    assert!(lines.is_empty(), "{}", lines.join("\n"));
}
