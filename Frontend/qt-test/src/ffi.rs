//! The harness's bridge: QTest's verbs and the item tree, declared once over the C++ of `cpp/harness.cpp` (EmuSen_Qt.md §8.3).

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qobject.h");
        type QObject = cxx_qt::QObject;
        include!("cxx-qt-lib/qqmlapplicationengine.h");
        type QQmlApplicationEngine = cxx_qt_lib::QQmlApplicationEngine;

        include!("harness.h");
        fn harness_show(window: Pin<&mut QObject>, width: i32, height: i32) -> bool;
        fn harness_resize(window: Pin<&mut QObject>, width: i32, height: i32);
        fn harness_tree(window: Pin<&mut QObject>) -> QString;
        fn harness_click(window: Pin<&mut QObject>, x: f64, y: f64, button: i32);
        fn harness_key(window: Pin<&mut QObject>, key: i32, modifiers: i32);
        fn harness_wheel(window: Pin<&mut QObject>, x: f64, y: f64, delta: i32);
        fn harness_wait(ms: i32);
        fn harness_focus(window: Pin<&mut QObject>) -> u64;
        fn harness_focus_item(window: Pin<&mut QObject>, id: u64) -> bool;
        fn harness_grab(window: Pin<&mut QObject>, path: &QString) -> bool;
        fn harness_property(root: Pin<&mut QObject>, object_name: &QString, property: &QString) -> QString;
        unsafe fn harness_singleton(engine: Pin<&mut QQmlApplicationEngine>, module: &QString, name: &QString) -> *mut QObject;
        fn harness_roots(engine: Pin<&mut QQmlApplicationEngine>) -> i32;
    }
}
