//! The program hosting an EmuSen theme (EmuSen_Qt.md §3.5): the theme's screens over the reference's where it provides them,
//! the reference wherever it does not or once it fails, its splash over Moonrise, and one engine however often it is switched.

mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_qt::MAIN_URL;

const THEMES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../theme-host/tests/themes");

fn choose(scene: &mut emusen_qt_test::Scene<'_>, theme: &str) {
    let folder = if theme.is_empty() { String::new() } else { format!("{THEMES}/{theme}") };
    assert!(scene.set_property("program", "theme", &folder));
    scene.wait(100);
}

#[test]
fn qt_3_5_the_theme_s_screens_replace_the_reference_s_and_a_failed_theme_shows_the_reference() {
    let _world = sandbox("qt-app-theme", &[("NES", "One.nes")]);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));

    // At the start a theme's splash is drawn over Moonrise, faded with it, and gone once Moonrise is done.
    choose(&mut scene, "splash");
    assert_eq!(scene.property("theme", "splash"), "true", "{}", scene.property("theme", "warnings"));
    assert_eq!(scene.property("splashLayer", "visible"), "true");
    assert_eq!(scene.property("stubSplash", "width"), "1280", "the splash fills the window");
    assert!(scene.set_property("program", "splash", "false"));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "theme", "splash", "false", 2000);
    assert_eq!(scene.property("splashLayer", "visible"), "false");

    // The theme's Home stands in for the reference's; Choose a console, which it omits, stays the reference's.
    assert_eq!(scene.property("theme", "home"), "true");
    assert_eq!(scene.property("splashHome", "visible"), "true");
    assert_eq!(scene.property("reference", "home"), "false", "the reference's Home gives way to a theme's");
    assert_eq!((scene.property("theme", "consoles").as_str(), scene.property("reference", "consoles").as_str()), ("false", "true"), "a screen the theme omits is the reference's");

    // A theme that fails to load is reported with the engine's words, and the reference is shown.
    choose(&mut scene, "models");
    assert_eq!(scene.property("theme", "failed"), "true");
    assert!(scene.property("theme", "warnings").contains("imports EmuSen.Models"), "{}", scene.property("theme", "warnings"));
    assert_eq!(scene.property("theme", "home"), "false");
    assert_eq!(scene.property("home", "visible"), "true", "the reference's Home is shown for a failed theme");

    // Switching at run time makes a new engine and lets the last one go; no theme, no engine.
    choose(&mut scene, "stub");
    assert_eq!(scene.property("stubHome", "visible"), "true");
    choose(&mut scene, "splash");
    choose(&mut scene, "stub");
    assert_eq!(scene.property("host.theme", "liveEngines"), "1", "switching twice leaks no engine");
    assert_eq!(scene.property("theme", "home"), "true");
    choose(&mut scene, "");
    assert_eq!(scene.property("chosenTheme", "status"), "Null", "no theme, no host and no engine");
    assert_eq!(scene.property("host.theme", "liveEngines"), "");
    assert_eq!(scene.property("home", "visible"), "true");
}
