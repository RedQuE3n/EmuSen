//! One harness test (EmuSen_Qt.md §8.1): the QML side sees LibraryModel's roles by name and its properties, and a
//! refresh over the real clients, run off the GUI thread, reaches it through qt_thread().queue.

use emusen_client_fixture::{library, World};
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt_test::Harness;

#[test]
fn qml_reads_the_library_model_s_rows_through_its_roles() {
    assert_eq!(emusen_qt_models::library::PLACES.len(), 3, "the module is linked into this test");
    let mut harness = Harness::new();
    let scene_file = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/LibraryScene.qml");
    let mut scene = harness.load(&format!("file://{scene_file}"), 640, 480);
    assert_eq!(scene.property("scene", "rows"), "3", "the places, before any source is named");
    assert_eq!(scene.property("scene", "firstName"), "All games");
    assert_eq!(scene.property("scene", "secondKey"), "favourites");
    assert_eq!(scene.property("scene", "thirdKind"), "place");
    assert_eq!(scene.property("scene", "thirdCount"), "0");
    assert_eq!(scene.property("scene", "thirdShown"), "Recently played", "the delegate shows the name role");
    assert_eq!(scene.property("scene", "selectedKey"), "all");
    assert_eq!(scene.property("scene", "selectedIndex"), "0");
    // The Home card's facts follow the lists the binding is given, not what the model held when it was first read.
    let given = r#"[{"console":"psx","name":"PlayStation","engine":"V1ConsoleCore"}]"#;
    assert!(scene.set_property("scene", "startableGiven", given));
    assert!(scene.set_property("scene", "toolsGiven", r#"{"cores":{"brief":"2 engines"}}"#));
    let facts: serde_json::Value = serde_json::from_str(&scene.property("scene", "homeFacts")).unwrap();
    assert_eq!((facts["startable"][0]["name"].as_str(), facts["tools"]["cores"]["brief"].as_str()), (Some("PlayStation"), Some("2 engines")), "{facts}");

    // The real clients named through the model's properties, on a scratch tree holding one synthetic NES game.
    let world = World::new("models-qml");
    let roms = world.dir("Roms/NES");
    let mut image = b"NES\x1a\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
    image.resize(16 + 16384, 0);
    std::fs::write(roms.join("Synthetic.nes"), image).unwrap();
    let platform = Platform::load(library()).unwrap();
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = serde_json::json!(world.root.join("Roms").to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    assert!(scene.set_property("scene", "platformLibrary", library().to_str().unwrap()));
    let mut waited = 0;
    while scene.property("scene", "rows") != "4" && waited < 5000 {
        scene.wait(50);
        waited += 50;
    }
    assert_eq!(scene.property("scene", "fourthKey"), "nes", "the NES's row arrived from the background refresh");
    assert_eq!(scene.property("scene", "loading"), "false");
    assert_eq!(scene.property("scene", "selectedKey"), "all", "the selection kept by its key");
}

#[test]
fn the_scene_keeps_the_qml_rules() {
    let scenes = emusen_qt_test::lint::frontend().join("qt-models").join("tests");
    let lines = emusen_qt_test::lint::rules(&[(&scenes, true)]);
    assert!(lines.is_empty(), "{}", lines.join("\n"));
}
