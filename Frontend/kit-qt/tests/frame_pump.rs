// Frame pumps PadRouter immediately on pendingChanged without waiting a timer tick or causing a Timer binding loop, while preserving auto-repeat.

use emusen_qt_test::Harness;

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/FramePumpScene.qml");

#[test]
fn frame_pumps_pad_router_immediately_and_keeps_repeat_timer() {
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 640, 480);
    scene.wait(30);
    assert_eq!(scene.property("first", "activeFocus"), "true");

    // Push Down with clockRunning active and read focus before an 8 ms timer tick could fire.
    scene.run_pad_clock();
    assert!(scene.set_property_now("scene", "holdDown", "true"));
    assert_eq!(scene.property("second", "activeFocus"), "true", "pendingChanged pumps immediately");

    // Holding Down past the 400 ms repeat delay uses the 8 ms repeat timer to step to the third action.
    scene.wait(520);
    assert_eq!(scene.property("third", "activeFocus"), "true", "repeat timer continues advancing while held");

    assert!(scene.set_property("scene", "holdDown", "false"));

    // Single tap (press + immediate release) pumps cleanly with zero Timer.running binding loop warnings.
    assert!(scene.set_property_now("scene", "tapUp", "true"));
    scene.wait(20);
    assert_eq!(scene.property("second", "activeFocus"), "true", "single direction tap advances once cleanly");

    assert!(scene.set_property_now("scene", "tapAccept", "true"));
    scene.wait(20);
    assert_eq!(scene.property("scene", "activatedCount"), "1", "single accept tap activates without warnings");

    scene.assert_quiet();
}
