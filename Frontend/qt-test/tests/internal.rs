//! EmuSen_Kit.md §7.101: Qt Multimedia's CPU-conversion line is Qt's own on the software backend alone; on the GPU it fails the test. The main-thread line is accepted in test programs only (R-20261111-110): counted and printed, failing nothing.

use emusen_qt_test::harness::{qt_internal, qt_tests_only, tests_only_lines};
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

const MAIN_THREAD: &str = "QApplication was not created in the main() thread.";

#[test]
fn kit_7_105_the_main_thread_line_is_accepted_in_tests_counted_and_printed() {
    assert!(!qt_internal(MAIN_THREAD, true) && !qt_internal(MAIN_THREAD, false), "not one of Qt's own lines on any backend");
    assert_eq!(qt_tests_only(MAIN_THREAD), Some("R-20261111-110"));
    assert_eq!(qt_tests_only(CPU), None);
    let seen = [MAIN_THREAD.to_owned(), CPU.to_owned(), MAIN_THREAD.to_owned()];
    assert_eq!(tests_only_lines(&seen), ["accepted in tests, R-20261111-110: 2 × QApplication was not created in the main() thread"]);
    assert!(tests_only_lines(&[CPU.to_owned()]).is_empty(), "nothing seen, nothing said");
}
