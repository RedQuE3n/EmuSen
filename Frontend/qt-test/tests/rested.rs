// EmuSen_Kit.md §7.49: a FrameAnimation declares no end, so `rest` waits for one while it runs, as theme-audit needs for the
// sky's glow easing after a resize; only an animation whose loops are infinite is a resting motion it does not wait for.

use emusen_qt_test::Harness;

#[test]
fn qt_7_49_rest_waits_for_a_frame_clock_that_stops_by_itself() {
    let dir = std::env::temp_dir().join(format!("emusen-rested-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("Clock.qml");
    let scene = "import QtQuick\nWindow {\n    id: scene\n    objectName: \"scene\"\n    width: 1280\n    height: 800\n    property real eased: 0\n    Item {\n        objectName: \"sky\"\n        FrameAnimation {\n            running: scene.eased < 0.4\n            onTriggered: scene.eased += frameTime\n        }\n    }\n    Rectangle {\n        width: 8\n        height: 8\n        NumberAnimation on opacity {\n            from: 0.2\n            to: 1\n            duration: 800\n            loops: Animation.Infinite\n        }\n    }\n}\n";
    std::fs::write(&file, scene).unwrap();
    let mut harness = Harness::new();
    let mut scene = harness.load(&format!("file://{}", file.display()), 1280, 800);
    scene.rest();
    let eased: f64 = scene.property("scene", "eased").parse().unwrap();
    assert!(eased >= 0.4, "rest returned with the frame clock at {eased} s of 0.4");
    let running = scene.running();
    assert!(running.iter().all(|r| r.ends_with(" looping")), "only the resting motion runs: {running:?}");
    std::fs::remove_dir_all(&dir).unwrap();
}
