//! A text the load cannot read is the model's problem, said in its words, never an empty view or a text read as nothing
//! (EmuSen_Qt.md §4.9): driven through `EsdeModel`'s own load, offscreen. One test, as the harness is one per process.

use emusen_qt_test::{Harness, Scene};

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/Load.qml");

fn until_problem(scene: &mut Scene<'_>, words: &str) {
    for _ in 0..200 {
        if scene.property("model", "problem").contains(words) {
            return;
        }
        scene.wait(10);
    }
    panic!("no problem saying {words:?}: {:?}", scene.property("model", "problem"));
}

#[test]
fn qt_4_9_a_text_the_load_cannot_read_is_the_model_s_problem() {
    assert!(emusen_esde_qt::marks::mark_named("star").is_some(), "the module is linked into this test");
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 320, 200);
    assert!(scene.set_property("model", "media", "{}"));
    assert!(scene.set_property("model", "presence", "{}"));
    assert!(scene.set_property("model", "shelves", "[{"));
    assert!(scene.set_property("model", "settings", "{\"theme\":\"t\"}"));
    until_problem(&mut scene, "The library's shelves could not be read");
    assert_eq!(scene.property("model", "themed"), "false", "nothing installed");

    // Shelves that read, and a media presence that does not: said, not read as no media.
    assert!(scene.set_property("model", "shelves", "[]"));
    assert!(scene.set_property("model", "presence", "{\"snes\":"));
    until_problem(&mut scene, "The library's media presence could not be read");
    assert_eq!(scene.property("model", "themed"), "false");
    scene.assert_quiet();
}
