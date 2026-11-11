//! The page frame's tiles when their model is replaced in the middle of its reset (EmuSen_Kit.md §7.58): the new model
//! is shown, not the frame left with no model for good.

use emusen_kit_gallery::SCREENS_URL;
use emusen_qt_test::Harness;

#[test]
fn a_tile_model_replaced_while_it_resets_is_shown() {
    assert!(!SCREENS_URL.is_empty(), "the kit is linked into this test");
    let mut harness = Harness::new();
    let file = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/qml/PageFrameSwap.qml");
    let mut scene = harness.load(&format!("file://{file}"), 1280, 800);
    assert!(scene.set_property("scene", "swapping", "true"));
    assert!(scene.set_property("first", "layouts", "letters\na b c"));
    scene.wait(50);
    assert_eq!(scene.property("frame", "tilesShown"), "true", "the frame shows the model it was handed");
}
