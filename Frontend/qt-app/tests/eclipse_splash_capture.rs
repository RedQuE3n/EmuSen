//! The frames of the tester's video of Eclipse's splash, a numbered BMP each in `EMUSEN_CAPTURE_OUT`; by hand, a GPU job (EmuSen_Qt.md §3.4.8, the splash).

// The modules the scene imports register through the program's crate, which a test must link.
extern crate emusen_qt;

mod eclipse_board;
#[allow(dead_code)]
mod eclipse_home_oracle;

use cxx_qt_lib::{QMap, QMapPair_QString_QVariant, QString, QStringList, QVariant};
use emusen_client_fixture::{cores, library, World};
use emusen_crystal_client::{Api, Config, Runtime};
use emusen_qt::MAIN_URL;
use emusen_qt_models::clients::ClientSource;
use emusen_qt_models::library::{installed_cores, LibrarySource};
use emusen_qt_models::themes;
use emusen_qt_test::compositor::Compositor;

/// The frames a second, and the tail after Home whole in seconds, as the tester asked them.
const RATE: i64 = 60;
const TAIL: f64 = 4.0;
/// Qt's key codes for the arrows, which a theme's walk answers as the pad's.
const RIGHT: i32 = 0x0100_0014;
const LEFT: i32 = 0x0100_0012;

/// The cores the program lists in this world, read as its library reads them, on the runtime's default folders.
fn listed() -> Vec<String> {
    let source = ClientSource::open(library().to_str().unwrap(), &Config::default()).unwrap();
    installed_cores(&source.systems().unwrap())
}

/// The test core approved in the world's /Cores as the Cores window approves one, then copies of its sidecar under other ids and names until the program lists `want` cores.
fn install(world: &World, want: usize) {
    let player = world.dir("Home/Cores");
    let core = world.player_core(&player);
    let rt = Runtime::new(&Api::load(library()).unwrap(), &Config::default()).unwrap();
    rt.core_approve(&cores::sha256(&core)).unwrap();
    drop(rt);
    let have = listed();
    assert!(have.len() <= want, "the world lists {} cores before any copy: {have:?}", have.len());
    eprintln!("listed before the copies: {have:?}");
    let sidecar = std::path::PathBuf::from(format!("{}.core.json", core.display()));
    let read: serde_json::Value = serde_json::from_slice(&std::fs::read(&sidecar).expect("the approval wrote the core's sidecar")).unwrap();
    let (id, name) = (read["info"]["id"].as_str().unwrap(), read["info"]["display_name"].as_str().or(read["info"]["name"].as_str()).unwrap());
    for i in 1..=want - have.len() {
        let mut copy = read.clone();
        copy["info"]["id"] = serde_json::json!(format!("{id}-{i}"));
        copy["info"]["display_name"] = serde_json::json!(format!("{name} {i}"));
        std::fs::write(player.join(format!("copy-{i}.core.json")), copy.to_string()).unwrap();
    }
    let now = listed();
    assert_eq!(now.len(), want, "the program lists {now:?}");
}

/// `EMUSEN_CAPTURE_CORES` = N gives the library N installed cores; `EMUSEN_CAPTURE_AT` = a second writes only that frame, `splash-<N>-cores.png`.
#[test]
#[ignore = "makes the tester's video's frames; run by hand with EMUSEN_CAPTURE_OUT"]
fn eclipse_splash_capture() {
    let out = std::path::PathBuf::from(std::env::var_os("EMUSEN_CAPTURE_OUT").expect("EMUSEN_CAPTURE_OUT names the frames' folder"));
    let want: Option<usize> = std::env::var("EMUSEN_CAPTURE_CORES").ok().map(|n| n.parse().expect("EMUSEN_CAPTURE_CORES is a count"));
    let at: Option<f64> = std::env::var("EMUSEN_CAPTURE_AT").ok().map(|s| s.parse().expect("EMUSEN_CAPTURE_AT is a second"));
    std::fs::create_dir_all(&out).unwrap();
    // A compositor larger than the window, so its decoration leaves the window 1920 by 1200 whole.
    let kwin = Compositor::start(3840, 2160);
    kwin.watch(1800);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    // A held clock is refused under a threaded render loop, which installs its own animation driver (EmuSen_Kit.md §7.36).
    std::env::set_var("QSG_RENDER_LOOP", "basic");
    std::env::set_var("LC_ALL", "en_US.UTF-8");
    std::env::set_var("TZ", "UTC");
    // The sound into a file of the world's, never the desk's speakers; the video takes the theme's file itself.
    let world = World::new("qt-app-eclipse-splash-capture");
    std::env::set_var("SDL_AUDIO_DRIVER", "disk");
    std::env::set_var("SDL_AUDIO_DISK_OUTPUT_FILE", world.root.join("heard.raw"));
    let mut harness = emusen_qt_test::Harness::new();
    let (_, media) = eclipse_home_oracle::synthetic(&world, &mut harness);
    if let Some(want) = want {
        install(&world, want);
    }
    let mut initial = QMap::<QMapPair_QString_QVariant>::default();
    initial.insert(QString::from("platformLibrary"), QVariant::from(&QString::from(library().to_str().unwrap())));
    initial.insert(QString::from("theme"), QVariant::from(&QString::from(&themes::built_in_folder("eclipse"))));
    initial.insert(QString::from("splash"), QVariant::from(&true));
    let mut stores = QStringList::default();
    stores.append(QString::from(media.to_str().unwrap()));
    initial.insert(QString::from("coverStores"), QVariant::from(&stores));
    // The clock held before the program's window is made, so the splash's first frame is the held clock's.
    let still = world.root.join("Still.qml");
    std::fs::write(&still, "import QtQuick\nWindow { visible: false }\n").unwrap();
    harness.load(&format!("file://{}", still.display()), 64, 64).hold_clock(Some(0));
    let mut scene = harness.load_initial(MAIN_URL, 1920, 1200, &initial, |_| {});
    assert_eq!(scene.property("themeSplash", "clock"), "0", "the splash's clock held at its first frame");
    let mut waited = 0;
    while scene.property("themeSplash", "set") != "true" && waited < 10_000 {
        scene.wait(25);
        waited += 25;
    }
    assert_eq!(scene.property("themeSplash", "set"), "true", "the start read and the sound ready");
    if let Some(at) = at {
        // The one frame, the clock set to it exactly as the splash's test sets its moments.
        let at_ms = (at * 1000.0).round() as i64;
        scene.hold_clock(Some(at_ms - 1000 / RATE));
        assert!(scene.set_property("themeSplash", "clock", &at_ms.to_string()));
        scene.wait(100);
        let time: f64 = scene.property("themeSplash", "time").parse().unwrap();
        assert!((time - at_ms as f64).abs() < 1.0, "the splash drawn at {time} ms, not {at_ms}");
        let count = want.unwrap_or_else(|| listed().len());
        let path = out.join(format!("splash-{count}-cores.png"));
        assert!(scene.grab(&path.display().to_string()), "the frame at {at_ms} ms grabbed");
        scene.hold_clock(None);
        drop(scene);
        drop(kwin);
        return;
    }
    let length: f64 = scene.property("themeSplash", "length").parse().unwrap();
    let frames = ((length / 1000.0 + TAIL) * RATE as f64).round() as i64;
    // The tail's presses on Home: three right and one left, a second apart from 1.2 s after Home whole.
    let presses: Vec<(i64, i32)> = [RIGHT, RIGHT, RIGHT, LEFT].iter().enumerate().map(|(i, k)| (((length / 1000.0 + 1.2 + i as f64 * 0.6) * RATE as f64) as i64, *k)).collect();
    for f in 0..=frames {
        scene.hold_clock(Some(f * 1000 / RATE));
        if let Some((_, key)) = presses.iter().find(|(at, _)| *at == f) {
            scene.key(*key, 0);
        }
        let time: f64 = scene.property("themeSplash", "time").parse().unwrap();
        assert!((time - (f * 1000 / RATE) as f64).abs() <= 16.0 || time >= length, "frame {f} draws the splash at {time} ms");
        let path = out.join(format!("{f:05}.bmp"));
        assert!(scene.grab(&path.display().to_string()), "frame {f} grabbed");
    }
    assert_eq!(scene.property("themeSplash", "done"), "true", "the splash ended into Home");
    scene.hold_clock(None);
    drop(scene);
    drop(kwin);
}
