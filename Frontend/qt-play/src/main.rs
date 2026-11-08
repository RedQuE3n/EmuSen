//! `emusen-qt-play`: the game picture in a real window (EmuSen_Qt.md §1.8).
//! `--seconds <n>` runs for that long and prints what was counted and what the validation layer said
//! (`EMUSEN_VALIDATE=1`); `--game <path>` plays a game through SessionModel, else a moving test picture is drawn. The
//! platform library is `EMUSEN_PLATFORM_LIBRARY`, and `EMUSEN_CORES` a cores folder in place of the runtime's default,
//! `EMUSEN_ENGINE` an engine by name in place of the settings' choice.

use core::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use cxx_qt::casting::Upcast;
use cxx_qt::QObject;
use cxx_qt_lib::{QGuiApplication, QMap, QMapPair_QString_QVariant, QQmlApplicationEngine, QString, QUrl, QVariant};
use emusen_platform_client::Platform;
use emusen_qt_models::picture::{Frame, Handoff};
use emusen_qt_play::host::Host;
use emusen_qt_play::window::ffi::{QCoreApplication, QGuiApplication as App, QQuickWindow};
use emusen_qt_play::PLAY_URL;

/// Seconds left out of the gate's counts at the start.
const WARM: u64 = 2;

fn main() {
    // Vulkan and the threaded render loop on every platform (EmuSen_Qt.md §1.8).
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let (mut seconds, mut game) = (10u64, None::<String>);
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seconds" => seconds = args.next().and_then(|n| n.parse().ok()).unwrap_or(seconds),
            "--game" => game = args.next(),
            _ => {}
        }
    }
    let validate = std::env::var("EMUSEN_VALIDATE").is_ok_and(|v| v == "1");
    let library = std::env::var("EMUSEN_PLATFORM_LIBRARY").expect("EMUSEN_PLATFORM_LIBRARY names the platform library");
    let mut application = QGuiApplication::new();
    let platform = Platform::load(&library).unwrap_or_else(|e| panic!("{e}"));
    let presets = std::env::temp_dir().join("emusen-qt-play");
    let host = Host::new(&platform, &App::platform_name().to_string(), None, validate, &presets).unwrap_or_else(|e| panic!("{e}"));
    let (device, feed) = (&host.device, &host.feed);
    let adoption = host.adoption();

    let mut engine = QQmlApplicationEngine::new();
    if let Some(mut engine) = engine.as_mut() {
        let mut initial = QMap::<QMapPair_QString_QVariant>::default();
        initial.insert(QString::from("feed"), QVariant::from(&feed.address()));
        initial.insert(QString::from("platformLibrary"), QVariant::from(&QString::from(&library)));
        if let Ok(cores) = std::env::var("EMUSEN_CORES") {
            initial.insert(QString::from("cores"), QVariant::from(&QString::from(&cores)));
        }
        if let Ok(engine) = std::env::var("EMUSEN_ENGINE") {
            initial.insert(QString::from("engine"), QVariant::from(&QString::from(&engine)));
        }
        if let Some(game) = &game {
            initial.insert(QString::from("game"), QVariant::from(&QString::from(game)));
        }
        engine.as_mut().set_initial_properties(&initial);
        engine
            .as_mut()
            .on_object_created(move |_, object, _| {
                let window = unsafe { QQuickWindow::from_base_ptr(object as *const QObject) } as *mut QQuickWindow;
                if window.is_null() {
                    return;
                }
                if let Err(words) = unsafe { adoption.adopt(window.cast()) } {
                    eprintln!("{words}");
                    QCoreApplication::quit();
                    return;
                }
                unsafe { Pin::new_unchecked(&mut *window) }.show();
            })
            .release();
        engine.load(&QUrl::from(PLAY_URL));
    }

    let stop = Arc::new(AtomicBool::new(false));
    let playing = game.is_some();
    let frames = std::thread::spawn({
        let (handoff, stop) = (feed.handoff(), stop.clone());
        move || {
            if !playing {
                test_picture(&handoff, &stop)
            }
        }
    });
    // Play to the first picture: from the window's load, which names the game, to the first frame drawn.
    let first = std::thread::spawn({
        let (stop, shown) = (stop.clone(), feed.handoff());
        let loaded = Instant::now();
        move || {
            while !stop.load(Ordering::Relaxed) && shown.shown() == 0 {
                std::thread::sleep(Duration::from_millis(1));
            }
            loaded.elapsed()
        }
    });
    // The gate counts from the second second on, past the start (EmuSen_Qt.md §1.8.4).
    let settled = std::thread::spawn({
        let handoff = feed.handoff();
        move || {
            std::thread::sleep(Duration::from_secs(WARM.min(seconds)));
            let at = handoff.counted();
            std::thread::sleep(Duration::from_secs(seconds.saturating_sub(WARM)));
            QCoreApplication::quit();
            at
        }
    });
    let started = Instant::now();
    if let Some(application) = application.as_mut() {
        application.exec();
    }
    stop.store(true, Ordering::Relaxed);
    let _ = frames.join();
    let first = first.join().unwrap_or_default();
    let settled = settled.join().unwrap_or_default();
    drop(engine);
    let counts = feed.counts();
    let findings = device.findings().unwrap_or_default();
    println!(
        "seconds {:.1} first_picture_ms {:.1} swaps {} shown {} dropped {} repeated {} findings {} problem {:?}",
        started.elapsed().as_secs_f64(),
        first.as_secs_f64() * 1000.0,
        counts.swaps.load(Ordering::Relaxed),
        counts.shown.load(Ordering::Relaxed),
        counts.dropped.load(Ordering::Relaxed),
        counts.repeated.load(Ordering::Relaxed),
        findings.len(),
        feed.problem()
    );
    let now = feed.handoff().counted();
    println!(
        "after {WARM} s: swaps {} shown {} dropped {} repeated {}",
        now[0] - settled[0],
        now[1] - settled[1],
        now[2] - settled[2],
        now[3] - settled[3]
    );
    for finding in findings.iter().take(5) {
        println!("finding: {finding}");
    }
}

/// A moving test picture, 256 by 224, one frame a swap, until `stop`.
fn test_picture(handoff: &Handoff, stop: &AtomicBool) {
    let mut frame = Frame { rgba: vec![0; 256 * 224 * 4], width: 256, height: 224, row_repeat: 1, ..Frame::default() };
    handoff.set_running(true);
    let mut seen = handoff.swaps();
    while !stop.load(Ordering::Relaxed) {
        frame.serial += 1;
        let shift = frame.serial as usize;
        frame.rgba.resize(256 * 224 * 4, 0);
        frame.width = 256;
        frame.height = 224;
        for (i, pixel) in frame.rgba.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let (x, y) = (i % 256, i / 256);
            pixel.copy_from_slice(&[((x + shift) % 256) as u8, ((y + shift) % 224) as u8, ((x ^ y) % 256) as u8, 255]);
        }
        handoff.hand_over(&mut frame);
        seen = handoff.wait_swap(seen, Duration::from_millis(100));
    }
    handoff.set_running(false);
}
