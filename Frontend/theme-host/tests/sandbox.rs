//! The sandbox of EmuSen_Qt.md §3.5 on stub themes: what a theme may reach, what it may not, the words when it may not, and the
//! engine each theme gets, destroyed when the next is made. Offscreen, the software backend; no request leaves the process.

use emusen_qt_test::{Harness, Scene};

extern crate emusen_qt_models;
extern crate emusen_theme_host;

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/Host.qml");
const THEMES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/themes");

fn load(scene: &mut Scene<'_>, theme: &str) {
    let folder = if theme.is_empty() { String::new() } else { format!("{THEMES}/{theme}") };
    assert!(scene.set_property("host", "folder", &folder), "the host takes a folder");
    scene.wait(100);
}

/// Waits for a picture to finish loading, either way, and gives its status.
fn picture(scene: &mut Scene<'_>, name: &str) -> String {
    let mut waited = 0;
    while waited < 3000 && scene.property(name, "status") == "Loading" {
        scene.wait(20);
        waited += 20;
    }
    scene.property(name, "status")
}

#[test]
fn qt_3_5_a_theme_reaches_its_folder_and_nothing_else() {
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 640, 400);

    // A stub theme of plain QtQuick screens loads, fills its slot, and reads a picture from its own folder.
    load(&mut scene, "stub");
    assert_eq!(scene.property("host", "ready"), "true", "{}", scene.property("host", "warnings"));
    assert_eq!(scene.property("scene", "placed"), "true", "{}", scene.property("host", "warnings"));
    assert_eq!(scene.property("stubHome", "width"), "640", "the screen fills its slot");
    assert_eq!(picture(&mut scene, "stubPicture"), "Ready", "a picture in the theme's folder loads");
    assert_eq!(scene.property("host", "warnings"), "", "a theme that keeps the rules is quiet");
    assert_eq!(scene.property("host", "liveEngines"), "1");

    // Importing the program's own models is refused before the screen compiles, with words naming the module and the file.
    load(&mut scene, "models");
    let words = scene.property("host", "warnings");
    assert_eq!(scene.property("scene", "placed"), "false");
    assert_eq!(scene.property("host", "failed"), "true");
    assert!(words.contains("screens/Home.qml:2 imports EmuSen.Models, which a theme may not import"), "{words}");
    assert_eq!(scene.property("host", "liveEngines"), "1", "the stub's engine went when this one came");

    // A remote picture is refused by the interceptor and by the network, and fails with a diagnostic.
    load(&mut scene, "remote");
    assert_eq!(scene.property("scene", "placed"), "true");
    assert_eq!(picture(&mut scene, "remotePicture"), "Error", "a remote picture fails");
    let words = scene.property("host", "warnings");
    assert!(words.contains("http://example.invalid/cover.png: a theme reaches no network"), "{words}");

    // A file outside the folder and the cover store is refused.
    load(&mut scene, "outside");
    assert_eq!(picture(&mut scene, "outsidePicture"), "Error", "a file outside the folder fails");
    let words = scene.property("host", "warnings");
    assert!(words.contains("file:///etc/hostname is outside the theme's folder and the cover store"), "{words}");

    // An inline component cannot reach the program's models either: the module does not resolve in a theme's engine.
    load(&mut scene, "inline");
    let words = scene.property("host", "warnings");
    assert_eq!(scene.property("inlineHome", "made"), "", "the inline component was not made");
    assert!(words.contains("EmuSen.Models is a module no theme may use"), "{words}");

    // A theme for a newer API is refused with words naming both versions, and no engine is made for it.
    load(&mut scene, "newer");
    let words = scene.property("host", "warnings");
    assert_eq!((scene.property("host", "ready").as_str(), scene.property("host", "failed").as_str()), ("false", "true"));
    assert!(words.contains("written for EmuSen.Theme 2.0; this build has 1.0"), "{words}");
    assert_eq!(scene.property("host", "liveEngines"), "0");

    // Switching twice leaves one engine alive, and none once no theme is chosen.
    load(&mut scene, "stub");
    load(&mut scene, "remote");
    load(&mut scene, "stub");
    assert_eq!(scene.property("host", "liveEngines"), "1", "switching leaks no engine");
    load(&mut scene, "");
    assert_eq!(scene.property("host", "liveEngines"), "0");
    assert_eq!(scene.property("host", "ready"), "false");
}
