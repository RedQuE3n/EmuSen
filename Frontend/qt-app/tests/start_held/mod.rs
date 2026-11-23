//! The program started as its start starts it, under a chosen theme, its animation clock held from the first frame (EmuSen_Qt.md §3.5.3).

use cxx_qt_lib::{QMap, QMapPair_QString_QVariant, QString, QStringList, QVariant};
use emusen_client_fixture::library;
use emusen_qt::MAIN_URL;
use emusen_qt_models::themes;
use emusen_qt_test::Scene;
use std::path::Path;

/// What the start reads from the settings and hands the window: the splash, reduced motion, navigation sounds.
pub struct Start {
    pub splash: bool,
    pub reduced: bool,
    pub sounds: bool,
}

pub const PLAIN: Start = Start { splash: true, reduced: false, sounds: true };

/// Eclipse's folder, as the Themes tab names it.
pub fn eclipse() -> String {
    themes::built_in_folder("eclipse")
}

/// The window's initial properties as the program's start gives them (qt-app/src/start.rs).
pub fn initial(theme: &str, start: &Start, media: &Path) -> QMap<QMapPair_QString_QVariant> {
    let mut map = QMap::<QMapPair_QString_QVariant>::default();
    map.insert(QString::from("platformLibrary"), QVariant::from(&QString::from(library().to_str().unwrap())));
    map.insert(QString::from("theme"), QVariant::from(&QString::from(theme)));
    map.insert(QString::from("splash"), QVariant::from(&start.splash));
    map.insert(QString::from("reducedMotion"), QVariant::from(&start.reduced));
    map.insert(QString::from("navigationSounds"), QVariant::from(&start.sounds));
    let mut stores = QStringList::default();
    stores.append(QString::from(media.to_str().unwrap()));
    map.insert(QString::from("coverStores"), QVariant::from(&stores));
    map
}

/// The program loaded with the animation clock held at 0 from before its window is made, on a still window of its own.
pub fn held<'a>(harness: &'a mut emusen_qt_test::Harness, size: (i32, i32), theme: &str, start: &Start, media: &Path) -> Scene<'a> {
    let still = media.parent().unwrap().join("Still.qml");
    std::fs::write(&still, "import QtQuick\nWindow { visible: false }\n").unwrap();
    {
        let mut window = harness.load(&format!("file://{}", still.display()), 64, 64);
        window.hold_clock(None);
        window.hold_clock(Some(0));
    }
    harness.load_initial(MAIN_URL, size.0, size.1, &initial(theme, start, media), |_| {})
}

/// Eclipse started, Moonrise never seen and the splash's clock at 0 until everything it waits for is in.
pub fn started<'a>(harness: &'a mut emusen_qt_test::Harness, size: (i32, i32), start: &Start, media: &Path) -> Scene<'a> {
    let mut scene = held(harness, size, &eclipse(), start, media);
    let mut waited = 0;
    while scene.property("themeSplash", "set") != "true" && waited < 10_000 && start.splash {
        assert_eq!(scene.property("moonrise", "visible"), "false", "Moonrise is never seen under Eclipse");
        assert_eq!(scene.property("themeSplash", "clock"), "0", "the splash's clock waits at its first frame");
        scene.wait(25);
        waited += 25;
    }
    scene
}

pub fn until(scene: &mut Scene<'_>, what: &str, ms: i32, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    let mut waited = 0;
    while !done(scene) && waited < ms {
        scene.wait(25);
        waited += 25;
    }
    assert!(done(scene), "{what} within {ms} ms");
}
