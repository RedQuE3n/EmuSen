//! EmuSen_Kit.md §7.101: Qt Multimedia's CPU-conversion line is Qt's own on the software backend alone; on the GPU it fails the test. The main-thread line is held (R-20261111-42): counted and printed, failing nothing.

use emusen_qt_test::harness::{held_lines, qt_held, qt_internal};
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
fn kit_7_101_the_main_thread_line_is_held_counted_and_printed_never_accepted() {
    assert!(!qt_internal(MAIN_THREAD, true) && !qt_internal(MAIN_THREAD, false), "not accepted on either backend (R-20261111-42 is held)");
    assert_eq!(qt_held(MAIN_THREAD), Some("R-20261111-42"));
    assert_eq!(qt_held(CPU), None);
    let seen = [MAIN_THREAD.to_owned(), CPU.to_owned(), MAIN_THREAD.to_owned()];
    assert_eq!(held_lines(&seen), ["held, R-20261111-42: 2 × QApplication was not created in the main() thread"]);
    assert!(held_lines(&[CPU.to_owned()]).is_empty(), "nothing held, nothing said");
}
