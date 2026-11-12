//! Home's Continue under the pointer (EmuSen_Qt.md §7.1): with no game suspended it is not a box the pointer can focus.

mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_qt::MAIN_URL;

/// The focused item's name, from the tree.
fn focused(scene: &mut emusen_qt_test::Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default()
}

#[test]
fn qt_7_1_home_s_continue_takes_no_pointer_focus_while_no_game_is_suspended() {
    let _world = sandbox("qt-app-home-continue", &[("NES", "One.nes")]);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    for _ in 0..20 {
        if focused(&mut scene) == "place.0" {
            break;
        }
        scene.wait(50);
    }
    assert_eq!(focused(&mut scene), "place.0");
    let tree = scene.tree();
    let node = &tree.nodes[tree.find_named("continue").expect("Continue's walkable") as usize];
    scene.hover(node.x + node.w / 2.0, node.y + node.h / 2.0);
    assert_eq!(focused(&mut scene), "place.0", "the pointer over Continue with no game suspended leaves the focus where it is");
}
