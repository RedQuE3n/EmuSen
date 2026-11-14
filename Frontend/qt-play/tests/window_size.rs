//! emusen-qt-play's start gives its window graphics.json's size, and WindowResizable off holds it there (EmuSen_Qt.md
//! §7.7.H7): the start's own `windowed_initial` over a scratch world whose graphics.json the test writes, the window made
//! with those initial properties and adopting Serenity's device in a compositor of the test's own, on the RX 6800
//! (EMUSEN_GPU_DEVICE names another). The compositor's size and the window's are test inputs. A GPU job.

mod common;

use common::{device_and_feed, vulkan_qt};
use cxx_qt::casting::Upcast;
use cxx_qt_lib::{QMap, QMapPair_QString_QVariant};
use emusen_client_fixture::{library, World};
use emusen_platform_client::galaxia::consts::{FILE_GRAPHICS, MODEL_GRAPHICS_CONFIG};
use emusen_platform_client::Platform;
use emusen_qt_play::window::ffi::QQuickWindow;
use emusen_qt_play::{windowed_initial, PLAY_URL};
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::{Harness, Scene};

/// The compositor's output and graphics.json's window: test inputs, the window smaller than the output.
const SCREEN: (i32, i32) = (1600, 1000);
const WINDOWED: (i32, i32) = (1100, 700);

fn number(scene: &mut Scene<'_>, name: &str) -> i32 {
    scene.property("play", name).parse::<f64>().expect("a number") as i32
}

#[test]
fn the_play_window_comes_up_at_graphics_json_s_size_held_there_when_not_resizable() {
    let _kwin = Compositor::start(SCREEN.0, SCREEN.1);
    vulkan_qt();
    let world = World::new("play-window-size");
    let platform = Platform::load(library()).unwrap();
    let mut graphics = platform.model_new(MODEL_GRAPHICS_CONFIG).unwrap();
    graphics["WindowWidth"] = serde_json::json!(WINDOWED.0);
    graphics["WindowHeight"] = serde_json::json!(WINDOWED.1);
    graphics["WindowResizable"] = serde_json::json!(false);
    let file = platform.file(FILE_GRAPHICS).unwrap();
    platform.model_save(MODEL_GRAPHICS_CONFIG, None, file.file_name().unwrap().to_str().unwrap(), &graphics).unwrap();

    let mut initial = QMap::<QMapPair_QString_QVariant>::default();
    windowed_initial(&platform, &mut initial);
    let (device, feed, adoption) = device_and_feed(&world.dir("presets"));
    let mut harness = Harness::new();
    let mut made = (0, 0);
    // The harness shows the window at the same size, so the size read before it is shown is the start's alone.
    let mut scene = harness.load_initial(PLAY_URL, WINDOWED.0, WINDOWED.1, &initial, |object| {
        let window = unsafe { QQuickWindow::from_base_ptr(object as *const cxx_qt::QObject) } as *mut QQuickWindow;
        assert!(!window.is_null(), "the play window's root is a window");
        made = unsafe { ((*window).width(), (*window).height()) };
        unsafe { adoption.adopt(object.cast()) }.expect("Qt adopts the device");
    });
    assert_eq!(made, WINDOWED, "the start gives the window graphics.json's size before it is shown");
    assert_eq!((number(&mut scene, "width"), number(&mut scene, "height")), WINDOWED, "shown at that size");
    let least = (number(&mut scene, "minimumWidth"), number(&mut scene, "minimumHeight"));
    let most = (number(&mut scene, "maximumWidth"), number(&mut scene, "maximumHeight"));
    assert_eq!((least, most), (WINDOWED, WINDOWED), "WindowResizable off holds the window at graphics.json's size");

    assert!(scene.set_property("play", "visible", "false"));
    scene.wait(50);
    drop(harness);
    drop(feed);
    assert_eq!(device.findings().unwrap(), Vec::<String>::new());
}
