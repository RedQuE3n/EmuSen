//! `emusen-qt`: the Qt front end's window, Moonrise and then Home (EmuSen_Qt.md §7). `--platform-library PATH` names the
//! platform library; the player's settings are read through it, as Mistress reads them. The window adopts Serenity's
//! device before it is first shown, so a game's picture is drawn where it lies (§1.8); `EMUSEN_GPU_DEVICE` names a card.

use cxx_qt::casting::Upcast;
use cxx_qt::QObject;
use cxx_qt_lib::{QGuiApplication, QMap, QMapPair_QString_QVariant, QQmlApplicationEngine, QQuickStyle, QString, QUrl, QVariant};
use emusen_platform_client::Platform;
use emusen_qt::{start, MAIN_URL};
use emusen_qt_play::host::Host;
use emusen_qt_play::window::ffi::{QGuiApplication as App, QQuickWindow};

/// The scene graph renders on Vulkan unless the environment names another backend (EmuSen_Qt.md §1.8, §9.2).
const RHI: &str = "vulkan";

fn main() {
    if std::env::var_os("QSG_RHI_BACKEND").is_none() {
        std::env::set_var("QSG_RHI_BACKEND", RHI);
    }
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let args: Vec<String> = std::env::args().collect();
    let program = std::env::current_exe().unwrap_or_default();
    let library = start::platform_library(&args, std::env::var("EMUSEN_PLATFORM_LIBRARY").ok(), &program);
    let start = start::read(&library);
    QQuickStyle::set_fallback_style(&QString::from("Basic"));
    QQuickStyle::set_style(&QString::from("EmuSen"));
    let mut application = QGuiApplication::new();
    // Without a device the window still opens, and plays no game.
    let gpu = std::env::var("EMUSEN_GPU_DEVICE").ok();
    let presets = std::env::temp_dir().join("emusen-qt");
    let host = Platform::load(&library)
        .map_err(|e| e.to_string())
        .and_then(|platform| Host::new(&platform, &App::platform_name().to_string(), gpu.as_deref(), false, &presets))
        .inspect_err(|words| eprintln!("No game can be played: {words}"))
        .ok();
    let mut engine = QQmlApplicationEngine::new();
    if let Some(mut engine) = engine.as_mut() {
        let mut initial = QMap::<QMapPair_QString_QVariant>::default();
        initial.insert(QString::from("platformLibrary"), QVariant::from(&QString::from(&start.platform_library)));
        initial.insert(QString::from("splash"), QVariant::from(&start.splash));
        initial.insert(QString::from("reducedMotion"), QVariant::from(&start.reduced_motion));
        if let Some(host) = &host {
            initial.insert(QString::from("feed"), QVariant::from(&host.feed.address()));
        }
        engine.as_mut().set_initial_properties(&initial);
        let adopting = host.as_ref().map(Host::adoption);
        engine
            .as_mut()
            .on_object_created(move |_, object, _| {
                let window = unsafe { QQuickWindow::from_base_ptr(object as *const QObject) } as *mut QQuickWindow;
                if window.is_null() {
                    return;
                }
                if let Some(adoption) = &adopting {
                    if let Err(words) = unsafe { adoption.adopt(window.cast()) } {
                        eprintln!("No game can be played: {words}");
                    }
                }
                unsafe { std::pin::Pin::new_unchecked(&mut *window) }.show();
            })
            .release();
        engine.load(&QUrl::from(MAIN_URL));
    }
    if let Some(application) = application.as_mut() {
        application.exec();
    }
    drop(engine);
    drop(host);
}
