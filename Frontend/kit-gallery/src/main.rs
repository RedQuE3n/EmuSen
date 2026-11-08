//! `kit-gallery`: the kit's gallery as a window, for a human look; the gate runs on the same scene headlessly (EmuSen_Kit.md §7.10).
//! `--grab <file.png>` saves the window's first swapped frame of its own size and ends, so a bundle is checked by its picture (EmuSen_Kit.md §7.13).

use core::pin::Pin;

use cxx_qt::casting::Upcast;
use cxx_qt::QObject;
use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QQuickStyle, QString, QUrl};
use emusen_kit_gallery::grab::ffi::{QCoreApplication, QQuickWindow};
use emusen_kit_gallery::grab::{settled, FRAMES};
use emusen_kit_gallery::GALLERY_URL;

/// The scene graph renders on Vulkan unless the environment names another backend (EmuSen_Qt.md §1.8, §9.2).
const RHI: &str = "vulkan";

fn main() {
    if std::env::var_os("QSG_RHI_BACKEND").is_none() {
        std::env::set_var("QSG_RHI_BACKEND", RHI);
    }
    let mut args = std::env::args().skip(1);
    let grab_to = (args.next().as_deref() == Some("--grab")).then(|| args.next()).flatten();
    QQuickStyle::set_fallback_style(&QString::from("Basic"));
    QQuickStyle::set_style(&QString::from("EmuSen"));
    let mut application = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();
    if let Some(mut engine) = engine.as_mut() {
        if let Some(path) = grab_to {
            engine
                .as_mut()
                .on_object_created(move |_, object, _| {
                    let window = unsafe { QQuickWindow::from_base_ptr(object as *const QObject) } as *mut QQuickWindow;
                    if window.is_null() {
                        return;
                    }
                    let path = path.clone();
                    let frames = std::cell::Cell::new(0u32);
                    unsafe { Pin::new_unchecked(&mut *window) }
                        .on_frame_swapped(move |window| {
                            let seen = frames.get();
                            if seen <= FRAMES {
                                frames.set(if grab(window, &path, seen) { FRAMES + 1 } else { seen + 1 });
                            }
                        })
                        .release();
                })
                .release();
        }
        engine.load(&QUrl::from(GALLERY_URL));
    }
    if let Some(application) = application.as_mut() {
        application.exec();
    }
}

/// Saves the frame just swapped when it is the window's own size, or after `FRAMES` frames whatever its size, and asks
/// the program to end, true once saved; a frame of another size asks for the next. The line names both sizes and the ratio.
fn grab(mut window: Pin<&mut QQuickWindow>, path: &str, seen: u32) -> bool {
    let image = window.as_mut().grab_window();
    let (frame, size, ratio) = (image.pixels(), (window.width(), window.height()), window.device_pixel_ratio());
    if !settled(frame, size, ratio) && seen < FRAMES {
        window.update();
        return false;
    }
    let saved = unsafe { image.save(&QString::from(path), core::ptr::null(), -1) };
    println!(
        "grab {path}: {}, frame {}x{}, window {}x{} at ratio {ratio}, after {seen} frames of another size",
        if saved { "saved" } else { "not saved" },
        frame.0,
        frame.1,
        size.0,
        size.1
    );
    QCoreApplication::quit();
    true
}
