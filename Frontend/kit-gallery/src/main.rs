//! `kit-gallery`: the kit's gallery as a window, for a human look; the gate runs on the same scene headlessly (EmuSen_Kit.md §7.10).
//! `--grab <file.png>` saves the window's first swapped frame and ends, so a bundle is checked by its picture (EmuSen_Kit.md §7.13).

use core::pin::Pin;

use cxx_qt::casting::Upcast;
use cxx_qt::QObject;
use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QQuickStyle, QString, QUrl};
use emusen_kit_gallery::grab::ffi::{QCoreApplication, QQuickWindow};
use emusen_kit_gallery::GALLERY_URL;

/// The scene graph renders on Vulkan unless the environment names another backend (EmuSen_Qt.md §1.8, §9.2).
const RHI: &str = "vulkan";

fn main() {
    if std::env::var_os("QSG_RHI_BACKEND").is_none() {
        std::env::set_var("QSG_RHI_BACKEND", RHI);
    }
    let mut args = std::env::args().skip(1);
    let grab = (args.next().as_deref() == Some("--grab")).then(|| args.next()).flatten();
    QQuickStyle::set_fallback_style(&QString::from("Basic"));
    QQuickStyle::set_style(&QString::from("EmuSen"));
    let mut application = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();
    if let Some(mut engine) = engine.as_mut() {
        if let Some(path) = grab {
            engine
                .as_mut()
                .on_object_created(move |_, object, _| {
                    let window = unsafe { QQuickWindow::from_base_ptr(object as *const QObject) } as *mut QQuickWindow;
                    if window.is_null() {
                        return;
                    }
                    let path = path.clone();
                    let taken = std::cell::Cell::new(false);
                    unsafe { Pin::new_unchecked(&mut *window) }
                        .on_frame_swapped(move |window| {
                            if !taken.replace(true) {
                                save_and_quit(window, &path);
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

/// Saves the frame just swapped and asks the program to end; frames swapped before it ends are not taken.
fn save_and_quit(window: Pin<&mut QQuickWindow>, path: &str) {
    let image = window.grab_window();
    let saved = unsafe { image.save(&QString::from(path), core::ptr::null(), -1) };
    println!("grab {path}: {}", if saved { "saved" } else { "not saved" });
    QCoreApplication::quit();
}
