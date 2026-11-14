//! The program's window comes up full screen on every machine, Big Picture being its only interface for now (the tester
//! decided, 2026-11-13; EmuSen_Qt.md §7.7.H7), at its screen's size and never a literal (the tester decided, 2026-11-14):
//! the window made with the start's own initial properties and shown by the start's own `window::show`, as main.rs shows
//! it, in a compositor of the test's own at a size the test chooses. Its size is read before it is shown, where no
//! compositor has answered, since one that names no size (gamescope) keeps what the window has; then the compositor's
//! answer is read back as visibility, and the Fullscreen hotkey gives a window at graphics.json's size, which the test
//! writes, and full screen again. For full_screen_gpu.rs and full_screen_small_gpu.rs, a GPU job each.
#![allow(dead_code)]

use super::console::until;
use super::{sandbox, wait_for};
use emusen_platform_client::galaxia::consts::{FILE_GRAPHICS, MODEL_GRAPHICS_CONFIG};
use emusen_platform_client::Platform;

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
/// The harness's own size for the window it shows, a test input a full-screen window must not keep.
const HARNESS_SIZE: (i32, i32) = (640, 400);
/// How long a size the test gave is watched for the program undoing it: a test input, past the compositor's answer to it.
const SETTLE_MS: i32 = 300;

fn tap(scene: &mut Scene<'_>, key: i32) {
    scene.key_hold(key, true);
    scene.key_hold(key, false);
}

fn size_of(scene: &mut Scene<'_>) -> (i32, i32) {
    let mut read = |name: &str| scene.property("program", name).parse::<f64>().expect("a number") as i32;
    (read("width"), read("height"))
}

/// Waits for the window to be `want`, naming the size it had when it was not.
fn until_size(scene: &mut Scene<'_>, what: &str, want: (i32, i32)) {
    let mut last = (0, 0);
    let start = std::time::Instant::now();
    while start.elapsed() < std::time::Duration::from_secs(5) {
        last = size_of(scene);
        if last == want {
            return;
        }
        scene.wait(50);
    }
    panic!("{what}: {last:?} after 5 s, not {want:?}");
}

/// F11 asking for full screen and the compositor giving it at the screen's size.
fn full_screen_again(scene: &mut Scene<'_>, screen: (i32, i32)) {
    tap(scene, F11);
    until(scene, 5, "F11 again asking for full screen", |s| s.property("program", "fullScreenAsked") == "true");
    until(scene, 5, "the compositor giving full screen again", |s| s.property("program", "visibility") == FULL_SCREEN);
    until(scene, 5, "the compositor's answer giving full screen again", |s| s.property("compositorAnswer", "fullScreen") == "true");
    until_size(scene, "full screen at the screen's size again", screen);
}

/// A size of the test's own for the window, other than graphics.json's: a test input, as a player's resize would give.
fn players_size(windowed: (i32, i32)) -> (i32, i32) {
    (windowed.0 * 3 / 4, windowed.1 * 3 / 4)
}

/// The test giving the window `size` while it is out of full screen, and the window keeping it.
fn resize_stays(scene: &mut Scene<'_>, turn: &str, size: (i32, i32)) {
    assert!(scene.set_property("program", "width", &size.0.to_string()));
    assert!(scene.set_property("program", "height", &size.1.to_string()));
    until_size(scene, &format!("{turn}: the test's size given"), size);
    scene.wait(SETTLE_MS);
    assert_eq!(size_of(scene), size, "{turn}: the test's size kept {SETTLE_MS} ms on");
}

/// graphics.json's window size in the sandbox, as the player's file would hold it.
fn write_window_size(windowed: (i32, i32), resizable: bool) {
    let platform = Platform::load(library()).unwrap();
    let mut graphics = platform.model_new(MODEL_GRAPHICS_CONFIG).unwrap();
    graphics["WindowWidth"] = serde_json::json!(windowed.0);
    graphics["WindowHeight"] = serde_json::json!(windowed.1);
    graphics["WindowResizable"] = serde_json::json!(resizable);
    let file = platform.file(FILE_GRAPHICS).unwrap();
    platform.model_save(MODEL_GRAPHICS_CONFIG, None, file.file_name().unwrap().to_str().unwrap(), &graphics).unwrap();
}

/// The start's window in a compositor of `width` by `height`, with graphics.json naming `windowed` and `resizable` (all
/// test inputs).
pub fn run(width: i32, height: i32, windowed: (i32, i32), resizable: bool) {
    let kwin = Compositor::start(width, height);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let _world = sandbox("qt-app-full-screen", &[("nes", "Brass Lantern.nes")]);
    write_window_size(windowed, resizable);
    let mut harness = Harness::new();
    let read = start::read(library());
    assert!(read.full_screen, "the start asks for full screen");
    let given = read.windowed.expect("the start reads graphics.json's window size whole");
    assert_eq!(((given.width, given.height), given.resizable), (windowed, resizable), "the start reads graphics.json's window size");
    let initial = start::initial(&read);
    let mut asked = (0, 0);
    let mut scene = harness.load_initial(MAIN_URL, HARNESS_SIZE.0, HARNESS_SIZE.1, &initial, |object| {
        let made = unsafe { QQuickWindow::from_base_ptr(object as *const cxx_qt::QObject) } as *mut QQuickWindow;
        assert!(!made.is_null(), "the program's root is a window");
        asked = unsafe { ((*made).width(), (*made).height()) };
        window::show(unsafe { std::pin::Pin::new_unchecked(&mut *made) }, read.full_screen);
    });
    assert_eq!(asked, (width, height), "the start's window has its screen's size before any compositor answers");
    until(&mut scene, 5, "the compositor giving the start's window full screen", |s| s.property("program", "visibility") == FULL_SCREEN);
    assert_eq!(size_of(&mut scene), (width, height), "full screen at the screen's size");
    assert_eq!(scene.property("program", "fullScreenAsked"), "true", "the start's request, as the hotkey's record");
    assert!(scene.set_property("program", "splash", "false"));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    until(&mut scene, 5, "the compositor's answer giving full screen", |s| s.property("compositorAnswer", "fullScreen") == "true");

    // The first press gives a window at graphics.json's size; the second full screen again. With WindowResizable on, two
    // more turns: the compositor (KWin) answers a window with the size it had before full screen, which is graphics.json's
    // the second time and the test's own the third, and a size the test then gives the window stays.
    tap(&mut scene, F11);
    until(&mut scene, 5, "F11 asking for a window", |s| s.property("program", "fullScreenAsked") == "false");
    until(&mut scene, 5, "the compositor giving a window", |s| s.property("program", "visibility") == WINDOWED);
    until(&mut scene, 5, "the compositor's answer giving a window", |s| s.property("compositorAnswer", "fullScreen") == "false");
    until_size(&mut scene, "the window at graphics.json's size", windowed);
    let least = (scene.property("program", "minimumWidth"), scene.property("program", "maximumHeight"));
    let held = (windowed.0.to_string(), windowed.1.to_string());
    assert_eq!(least == held, !resizable, "WindowResizable off holds the window at its size, on leaves it free: {least:?}");
    full_screen_again(&mut scene, (width, height));
    if resizable {
        for turn in ["the answer naming graphics.json's size", "the answer naming the test's size"] {
            tap(&mut scene, F11);
            until(&mut scene, 5, "the compositor's answer giving a window", |s| s.property("compositorAnswer", "fullScreen") == "false");
            until_size(&mut scene, &format!("{turn}: the window at graphics.json's size"), windowed);
            resize_stays(&mut scene, turn, players_size(windowed));
            full_screen_again(&mut scene, (width, height));
        }
    }

    // Hidden while its scene graph still holds its instance, as the other GPU tests leave the window.
    assert!(scene.set_property("program", "visible", "false"));
    scene.wait(50);
    drop(harness);
    drop(kwin);
}
