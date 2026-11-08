//! `kit-gallery`: the kit's gallery as a window, for a human look; the gate runs on the same scene headlessly (EmuSen_Kit.md §7.10).

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QQuickStyle, QString, QUrl};
use emusen_kit_gallery::GALLERY_URL;

fn main() {
    QQuickStyle::set_fallback_style(&QString::from("Basic"));
    QQuickStyle::set_style(&QString::from("EmuSen"));
    let mut application = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from(GALLERY_URL));
    }
    if let Some(application) = application.as_mut() {
        application.exec();
    }
}
