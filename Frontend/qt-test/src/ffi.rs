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
        /// Whether the window holds an item made since its tree was last read.
        fn harness_made_since(window: Pin<&mut QObject>) -> bool;
        /// Whether the view numbered `id` still turns to its current item: its highlight follows it and stands elsewhere.
        fn harness_turning(window: Pin<&mut QObject>, id: u64) -> bool;
        fn harness_click(window: Pin<&mut QObject>, x: f64, y: f64, button: i32);
        fn harness_hover(window: Pin<&mut QObject>, x: f64, y: f64);
        fn harness_key(window: Pin<&mut QObject>, key: i32, modifiers: i32);
        /// Presses a key without releasing it, or releases it: a press held.
        fn harness_key_hold(window: Pin<&mut QObject>, key: i32, down: bool);
        type HeldClock;
        /// Qt's animation clock replaced by one held at 0, until the one returned is dropped.
        fn harness_clock_hold() -> UniquePtr<HeldClock>;
        /// The held clock moved to `ms` since it was held, a frame's step at a time.
        fn harness_clock_move(clock: Pin<&mut HeldClock>, ms: i64);
        fn harness_wheel(window: Pin<&mut QObject>, x: f64, y: f64, delta: i32);
        fn harness_wait(ms: i32);
        fn harness_keep_messages();
        fn harness_messages() -> QString;
        /// Keeps Qt's account of each frame's dirty items (its `qt.quick.dirty` log) from now, or stops; either clears it.
        fn harness_watch_dirty(on: bool);
        /// The kept account, one line a frame's start and one an item, emptied as read.
        fn harness_dirty() -> QString;
        fn harness_focus(window: Pin<&mut QObject>) -> u64;
        fn harness_focus_item(window: Pin<&mut QObject>, id: u64) -> bool;
        fn harness_get(window: Pin<&mut QObject>, id: u64, property: &QString) -> f64;
        fn harness_put(window: Pin<&mut QObject>, id: u64, property: &QString, value: f64) -> bool;
        fn harness_scoped(window: Pin<&mut QObject>, scope: u64) -> u64;
        fn harness_give_focus(window: Pin<&mut QObject>, id: u64);
        fn harness_grab(window: Pin<&mut QObject>, path: &QString) -> bool;
        fn harness_property(root: Pin<&mut QObject>, object_name: &QString, property: &QString) -> QString;
        unsafe fn harness_singleton(engine: Pin<&mut QQmlApplicationEngine>, module: &QString, name: &QString) -> *mut QObject;
        /// The singleton of the engine that made the window's focused item, as a hosted theme's engine has its own; null for none.
        fn harness_focused_singleton(window: Pin<&mut QObject>, module: &QString, name: &QString) -> *mut QObject;
        fn harness_roots(engine: Pin<&mut QQmlApplicationEngine>) -> i32;
        fn harness_set_property(root: Pin<&mut QObject>, object_name: &QString, property: &QString, value: &QString) -> bool;
        fn harness_destroy(root: Pin<&mut QObject>, object_name: &QString) -> bool;
        fn harness_watch(root: Pin<&mut QObject>, object_name: &QString, property: &QString) -> bool;
        fn harness_notified(root: Pin<&mut QObject>, object_name: &QString, property: &QString) -> i32;
        fn harness_watch_rows(root: Pin<&mut QObject>, object_name: &QString) -> bool;
        fn harness_watch_order(root: Pin<&mut QObject>, object_name: &QString, property: &QString) -> bool;
        fn harness_rows_told(root: Pin<&mut QObject>, object_name: &QString) -> QString;
        fn harness_colours_at(window: Pin<&mut QObject>, points: &QString) -> QString;
        fn harness_hide_words(window: Pin<&mut QObject>, hidden: bool);
        fn harness_frames(window: Pin<&mut QObject>) -> u64;
        fn harness_running(window: Pin<&mut QObject>) -> QString;
        fn harness_difference(
            window: Pin<&mut QObject>,
            reference: &QString,
            tolerance: i32,
            reach: i32,
            loose: &QString,
            marks: &QString,
        ) -> i32;
        fn harness_drag(window: Pin<&mut QObject>, x0: f64, y0: f64, x1: f64, y1: f64);
    }
}
