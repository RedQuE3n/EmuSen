//! The program's window through the harness (EmuSen_Qt.md §7, §8.1): Moonrise over Home at the start with the systems'
//! planets, a pad press skipping it, and Home holding the focus once it has gone and the library is read.

mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;
use emusen_qt_test::Harness;

#[test]
fn moonrise_plays_over_home_and_a_press_skips_it() {
    let _world = sandbox("qt-app-program", &[("NES", "Synthetic.nes")]);
    assert!(emusen_qt::start::read(library()).splash, "the settings saved leave the splash on");

    let mut harness = Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert_eq!(scene.property("moonrise", "visible"), "true", "Moonrise covers the window from the start");
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "moonrise", "running", "true", 5000);
    for _ in 0..100 {
        if !scene.property("moonrise", "caption").is_empty() {
            break;
        }
        scene.wait(50);
    }
    let caption = scene.property("moonrise", "caption");
    assert!(caption.ends_with("CONSOLES") && caption != "0 CONSOLES", "the caption counts the systems with an engine: {caption}");
    assert_eq!(scene.property("program", "splashShown"), "true");
    assert_eq!(scene.property("moonrise", "finished"), "false", "the splash is still drawing");

    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "moonrise", "finished", "true", 1000);
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "moonrise", "visible", "false", 2000);
    assert_eq!(scene.property("program", "splashShown"), "false");
    assert_eq!(scene.property("home", "activeFocus"), "true", "Home takes the focus once the splash has gone");
    assert_eq!(scene.property("library", "selectedKey"), "all", "the press was the splash's alone");
    scene.assert_quiet();
}

#[test]
fn the_program_s_scenes_keep_the_qml_rules() {
    let qml = emusen_qt_test::lint::frontend().join("qt-app").join("qml");
    let lines = emusen_qt_test::lint::rules(&[(&qml, false)]);
    assert!(lines.is_empty(), "{}", lines.join("\n"));
    let complaints = emusen_qt_test::lint::qmllint(&[&qml]);
    assert!(complaints.is_empty(), "{}", complaints.join("\n"));
}

#[test]
fn qt_3_4_the_reference_theme_keeps_the_theme_rules() {
    // The package as any theme is held: its imports to the theme's list and §1.3's rules (EmuSen_Qt.md §3.5, §3.7).
    let package = emusen_qt_test::lint::frontend().join("themes").join("emusen");
    let mut found = Vec::new();
    for file in emusen_qml_rules::qml_files(&package) {
        let text = std::fs::read_to_string(&file).unwrap();
        found.extend(emusen_qml_rules::check_theme(&file.to_string_lossy(), &text).iter().map(ToString::to_string));
    }
    assert!(found.is_empty(), "{}", found.join("\n"));
    let complaints = emusen_qt_test::lint::qmllint(&[&package]);
    assert!(complaints.is_empty(), "{}", complaints.join("\n"));
}
