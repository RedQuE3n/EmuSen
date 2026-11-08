//! `emusen-qt`: the Qt front end's window, Moonrise and then Home (EmuSen_Qt.md §7). `--platform-library PATH` names the
//! platform library; the player's settings are read through it, as Mistress reads them.

use cxx_qt_lib::{QGuiApplication, QMap, QMapPair_QString_QVariant, QQmlApplicationEngine, QQuickStyle, QString, QUrl, QVariant};
use emusen_qt::{start, MAIN_URL};

/// The scene graph renders on Vulkan unless the environment names another backend (EmuSen_Qt.md §1.8, §9.2).
const RHI: &str = "vulkan";

fn main() {
    if std::env::var_os("QSG_RHI_BACKEND").is_none() {
        std::env::set_var("QSG_RHI_BACKEND", RHI);
    }
    let args: Vec<String> = std::env::args().collect();
    let program = std::env::current_exe().unwrap_or_default();
    let library = start::platform_library(&args, std::env::var("EMUSEN_PLATFORM_LIBRARY").ok(), &program);
    let start = start::read(&library);
    QQuickStyle::set_fallback_style(&QString::from("Basic"));
    QQuickStyle::set_style(&QString::from("EmuSen"));
    let mut application = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();
    if let Some(mut engine) = engine.as_mut() {
        let mut initial = QMap::<QMapPair_QString_QVariant>::default();
        initial.insert(QString::from("platformLibrary"), QVariant::from(&QString::from(&start.platform_library)));
        initial.insert(QString::from("splash"), QVariant::from(&start.splash));
        initial.insert(QString::from("reducedMotion"), QVariant::from(&start.reduced_motion));
        engine.as_mut().set_initial_properties(&initial);
        engine.load(&QUrl::from(MAIN_URL));
    }
    if let Some(application) = application.as_mut() {
        application.exec();
    }
}
