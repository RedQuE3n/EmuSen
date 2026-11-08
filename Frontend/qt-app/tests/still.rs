//! Moonrise under reduced motion, in a process of its own: its last frame from the start, held for its length, and
//! skipped only by a key the kit's map makes a press (EmuSen_Interface.md §22, EmuSen_Qt.md §1.7 and §7.1).

mod common;

use common::{sandbox, wait_for};
use emusen_qt::MAIN_URL;
use emusen_qt_models::words::splash_caption;
use emusen_qt_test::Harness;

const KEY_TAB: i32 = 0x0100_0001;
const KEY_SHIFT: i32 = 0x0100_0020;
const KEY_A: i32 = 0x41;
const KEY_RETURN: i32 = 0x0100_0004;

#[test]
fn under_reduced_motion_the_last_frame_holds_and_only_the_map_s_keys_skip_it() {
    let _world = sandbox("qt-app-still", &[("NES", "Synthetic.nes")]);
    let worlds = common::canvas_worlds();
    let mut harness = Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 720);
    assert!(scene.set_property("program", "reducedMotion", "true"));
    assert!(scene.set_property("moonrise", "worlds", &worlds.join("\n")));
    assert!(scene.set_property("moonrise", "caption", &splash_caption(worlds.len())));
    assert!(scene.set_property("moonrise", "running", "true"));
    scene.wait(300);
    assert_eq!(scene.property("moonrise", "finished"), "false", "held for the splash's length");
    let tree = scene.tree();
    let marks = std::env::temp_dir().join("qt-app-still-difference.png").display().to_string();
    let faults = emusen_qt_test::moonrise::against(&mut scene, &tree, emusen_qt_test::moonrise::BODIES_REFERENCE, emusen_qt_test::moonrise::BODIES_SPARE, &marks);
    assert!(faults.is_empty(), "the last frame from the start: {}", faults.join("\n"));

    for (key, modifiers) in [(KEY_TAB, 0), (KEY_SHIFT, 0x0200_0000), (KEY_A, 0)] {
        scene.key(key, modifiers);
        assert_eq!(scene.property("moonrise", "finished"), "false", "key {key:#x} is no press in the kit's map");
        assert_eq!(scene.property("moonrise", "activeFocus"), "true", "key {key:#x} stays the splash's");
    }
    scene.key(KEY_RETURN, 0);
    wait_for(&mut scene, "moonrise", "finished", "true", 500);
    scene.assert_quiet();
}
