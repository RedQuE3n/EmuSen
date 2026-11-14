//! The program's window comes up full screen on every machine, Big Picture being its only interface for now (the tester
//! decided, 2026-11-13; EmuSen_Qt.md §7.7.H7): the window made with the start's own initial properties and shown by the
//! start's own `window::show`, as main.rs shows it, in a compositor of the test's own, whose answer is read back as the
//! window's visibility; then the Fullscreen hotkey gives a window and full screen again. A GPU job.

mod common;

use common::console::until;
use common::{sandbox, wait_for};

use cxx_qt::casting::Upcast;
use emusen_client_fixture::library;
use emusen_qt::{start, window, MAIN_URL};
use emusen_qt_play::window::ffi::QQuickWindow;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::{Harness, Scene};

/// Qt's number for F11, Endymion's default Fullscreen key.
const F11: i32 = 0x0100_003a;
/// QWindow's visibilities, as the harness names a property's enumeration value.
const FULL_SCREEN: &str = "FullScreen";
const WINDOWED: &str = "Windowed";

fn tap(scene: &mut Scene<'_>, key: i32) {
    scene.key_hold(key, true);
    scene.key_hold(key, false);
}

#[test]
fn the_start_shows_the_window_full_screen_and_the_hotkey_turns_it() {
    let kwin = Compositor::start(1920, 1200);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let _world = sandbox("qt-app-full-screen", &[("nes", "Brass Lantern.nes")]);
    let mut harness = Harness::new();
    let read = start::read(library());
    assert!(read.full_screen, "the start asks for full screen");
    let initial = start::initial(&read);
    let mut scene = harness.load_initial(MAIN_URL, 1280, 800, &initial, |object| {
        let made = unsafe { QQuickWindow::from_base_ptr(object as *const cxx_qt::QObject) } as *mut QQuickWindow;
        assert!(!made.is_null(), "the program's root is a window");
        window::show(unsafe { std::pin::Pin::new_unchecked(&mut *made) }, read.full_screen);
    });
    until(&mut scene, 5, "the compositor giving the start's window full screen", |s| s.property("program", "visibility") == FULL_SCREEN);
    assert_eq!(scene.property("program", "fullScreenAsked"), "true", "the start's request, as the hotkey's record");
    assert!(scene.set_property("program", "splash", "false"));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);

    // The first press gives a window, the second full screen again: what was asked, then what the compositor gives.
    tap(&mut scene, F11);
    until(&mut scene, 5, "F11 asking for a window", |s| s.property("program", "fullScreenAsked") == "false");
    until(&mut scene, 5, "the compositor giving a window", |s| s.property("program", "visibility") == WINDOWED);
    tap(&mut scene, F11);
    until(&mut scene, 5, "F11 again asking for full screen", |s| s.property("program", "fullScreenAsked") == "true");
    until(&mut scene, 5, "the compositor giving full screen again", |s| s.property("program", "visibility") == FULL_SCREEN);

    // Hidden while its scene graph still holds its instance, as the other GPU tests leave the window.
    assert!(scene.set_property("program", "visible", "false"));
    scene.wait(50);
    drop(harness);
    drop(kwin);
}
