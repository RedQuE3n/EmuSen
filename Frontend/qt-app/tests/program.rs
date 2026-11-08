//! The program's window through the harness (EmuSen_Qt.md §7, §8.1): Moonrise over Home at the start with the systems'
//! planets, a pad press skipping it, and Home holding the focus once it has gone and the library is read.

use emusen_client_fixture::{library, World};
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_test::Harness;

fn wait_for(scene: &mut emusen_qt_test::Scene<'_>, object: &str, property: &str, value: &str, ms: i32) {
    let mut waited = 0;
    while scene.property(object, property) != value && waited < ms {
        scene.wait(50);
        waited += 50;
    }
    assert_eq!(scene.property(object, property), value, "{object}.{property} after {ms} ms");
}

/// A scratch tree whose settings name its ROM folder, holding one synthetic NES image.
fn sandbox(name: &str) -> World {
    let world = World::new(name);
    let roms = world.dir("Roms/NES");
    let mut image = b"NES\x1a\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
    image.resize(16 + 16384, 0);
    std::fs::write(roms.join("Synthetic.nes"), image).unwrap();
    let platform = Platform::load(library()).unwrap();
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = serde_json::json!(world.root.join("Roms").to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    world
}

#[test]
fn moonrise_plays_over_home_and_a_press_skips_it() {
    let _world = sandbox("qt-app-program");
    assert!(emusen_qt::start::read(library()).splash, "the settings saved leave the splash on");

    let mut harness = Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert_eq!(scene.property("moonrise", "visible"), "true", "Moonrise covers the window from the start");
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "moonrise", "running", "true", 5000);
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
