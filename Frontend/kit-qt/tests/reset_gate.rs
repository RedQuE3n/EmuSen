// EmuSen_Kit.md §7.58: ResetGate says a model is unsettled from its modelAboutToBeReset to its modelReset, settles a model
// handed over in the middle of the old one's reset, and no longer hears the old one. One test, as the harness is one per process;
// the count of `settled`'s changes is from the first watch on.

use emusen_qt_test::Harness;

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/ResetGateScene.qml");

#[test]
fn kit_7_58_the_gate_is_unsettled_through_a_reset_and_settles_a_model_handed_over() {
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 640, 480);
    assert_eq!(scene.property("gate", "settled"), "true", "a model not resetting is settled");
    assert!(scene.watch("gate", "settled"));
    assert!(scene.set_property("first", "layouts", "letters\na b c"));
    assert_eq!(scene.notified("gate", "settled"), Some(2), "the reset unsettled the gate and its end settled it");
    assert_eq!(scene.property("gate", "settled"), "true");

    assert!(scene.set_property("scene", "swapping", "true"));
    assert!(scene.set_property("first", "layouts", "letters\nd e f"));
    assert_eq!(scene.property("gate", "holdsSecond"), "true", "the scene handed the gate the second model");
    assert_eq!(scene.property("gate", "settled"), "true", "the new model is settled, though the old one's reset never ended here");
    assert_eq!(scene.notified("gate", "settled"), Some(4), "unsettled by the old model's reset, settled by the handover");
    assert!(scene.set_property("first", "layouts", "letters\ng h i"));
    assert_eq!(scene.notified("gate", "settled"), Some(4), "the old model's reset is no longer heard");
    assert!(scene.set_property("second", "layouts", "letters\ng h i"));
    assert_eq!(scene.notified("gate", "settled"), Some(6), "the new model's reset is");
    scene.assert_quiet();
}
