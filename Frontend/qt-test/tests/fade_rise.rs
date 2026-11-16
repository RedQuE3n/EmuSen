// EmuSen_Kit.md §7.35: an EdgeFade whose edge starts fading and stops again before the event queue clears keeps its layer
// until it does: the edge falls while Qt is still telling the fade's own listeners of its new size, and the layer made a
// moment before must not be deleted under them (Orrery's dial, at a window's resize). The software backend.

use emusen_qt_test::Harness;

const NAME: &str = "qt_kit_7_35_an_edge_fade_turned_on_and_off_before_the_queue_clears_keeps_its_layer_until_then";

// The test runs itself again with glibc's thread cache off and its M_PERTURB on, so every freed block is overwritten at
// once and a call into a deleted listener faults on every run; with the cache on, a freed block keeps its bytes (man 3 mallopt).
#[test]
fn qt_kit_7_35_an_edge_fade_turned_on_and_off_before_the_queue_clears_keeps_its_layer_until_then() {
    if std::env::var_os("EMUSEN_FADE_RISE_CHILD").is_none() {
        let out = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", NAME, "--nocapture", "--test-threads=1"])
            .env("EMUSEN_FADE_RISE_CHILD", "1")
            .env("GLIBC_TUNABLES", "glibc.malloc.tcache_count=0:glibc.malloc.perturb=165")
            .output()
            .expect("the test runs itself");
        assert!(out.status.success(), "the fade's run stopped ({}):\n{}", out.status, String::from_utf8_lossy(&out.stderr));
        return;
    }
    let dir = std::env::temp_dir().join(format!("emusen-fade-rise-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("FadeRise.qml");
    let scene = "import QtQuick\nimport EmuSen.Kit as Kit\nWindow {\n    id: scene\n    objectName: \"scene\"\n    width: 1280\n    height: 800\n    property real wide: 400\n    property real content: 300\n    Kit.EdgeFade {\n        objectName: \"fade\"\n        width: scene.wide\n        height: 100\n        fadeRight: row.fadeRight\n        length: 40\n        Flickable {\n            id: row\n            objectName: \"row\"\n            anchors.fill: parent\n            interactive: false\n            contentWidth: scene.content\n            contentHeight: height\n            readonly property bool fadeRight: contentWidth > width\n        }\n    }\n}\n";
    std::fs::write(&file, scene).unwrap();
    let mut harness = Harness::new();
    let mut scene = harness.load(&format!("file://{}", file.display()), 1280, 800);
    scene.wait(100);
    for _ in 0..2 {
        assert_eq!(scene.property("row", "fadeRight"), "false");
        // The edge starts fading, then the fade grows past what the row holds, with no event between the two.
        assert!(scene.set_property_now("scene", "content", "1000"));
        assert_eq!(scene.property("row", "fadeRight"), "true");
        assert!(scene.set_property_now("scene", "wide", "1200"));
        assert_eq!(scene.property("row", "fadeRight"), "false", "the row fits the fade 1200 wide");
        scene.wait(50);
        assert!(scene.set_property_now("scene", "wide", "400"));
        assert!(scene.set_property_now("scene", "content", "300"));
        scene.wait(50);
    }
    scene.assert_quiet();
    std::fs::remove_dir_all(&dir).unwrap();
}
