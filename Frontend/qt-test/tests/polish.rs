// EmuSen_Kit.md §7.49: the tree is read as the next frame will draw it. A positioner lays out at the polish before a frame,
// so a tree read between a change and that polish named every help entry at the Row's start, while a frame clock ran
// and `rest` had nothing to wait for.

use emusen_qt_test::{Harness, Scene};

fn xs(scene: &mut Scene<'_>) -> Vec<f64> {
    let tree = scene.tree();
    ["help-Open", "help-Search", "help-Home"].iter().map(|n| tree.nodes[tree.find_named(n).expect("an entry") as usize].x).collect()
}

#[test]
fn qt_7_41_the_tree_is_read_as_it_is_drawn() {
    let mut harness = Harness::new();
    let mut scene = harness.load("qrc:/qt/qml/EmuSen/Scenes/qml/PolishScene.qml", 1280, 800);
    scene.rest();
    let laid = xs(&mut scene);
    assert!(laid[0] < laid[1] && laid[1] < laid[2], "the entries stand in a row: {laid:?}");
    for alt in ["1", "0", "1"] {
        scene.set_property_now("scene", "alt", alt);
        scene.rest();
        assert_eq!(xs(&mut scene), laid, "the entries made again stand where they stood, with alt {alt}");
    }
    scene.assert_quiet();
}
