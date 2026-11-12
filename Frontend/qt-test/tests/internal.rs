//! EmuSen_Kit.md §7.101: Qt Multimedia's CPU-conversion line is Qt's own on the software backend alone; on the GPU it fails the test.

use emusen_qt_test::harness::qt_internal;
use emusen_qt_test::Harness;

const CPU: &str = "qt.multimedia: No RHI backend. Using CPU conversion.";

#[test]
fn kit_7_101_the_cpu_conversion_line_is_qt_s_on_the_software_backend_only() {
    assert!(qt_internal(CPU, true), "on the software backend");
    assert!(!qt_internal(CPU, false), "on the GPU");
    assert!(!qt_internal("qrc:/Frame.qml:3: TypeError", true), "ours");
    let mut harness = Harness::new();
    // Qt settles the backend when the first window is made.
    drop(harness.load("qrc:/qt/qml/EmuSen/Scenes/qml/SheetScene.qml", 320, 200));
    assert!(harness.software(), "the harness draws on the software backend unless told otherwise");
}
