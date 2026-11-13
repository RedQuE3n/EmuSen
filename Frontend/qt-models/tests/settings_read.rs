//! Settings' read off the GUI thread against a save made while it is out (EmuSen_Qt.md §7.7.H7): the read begun as
//! Settings opens has loaded the stored files and is waiting to be delivered when a row is turned and saved; once it is
//! delivered, the row keeps the value saved.

use emusen_client_fixture::{library, World};
use emusen_qt_models::settings::{self, SettingsView};
use emusen_qt_models::settings_source::ClientSettings;
use emusen_qt_test::Harness;

/// The Display section's index and its Appearance row's, as the model's view numbers them over the stored files.
fn appearance() -> (usize, usize) {
    let source = ClientSettings::open(&library().display().to_string()).unwrap();
    let mut view = SettingsView::new();
    view.read = settings::read(&source).unwrap();
    let section = view.sections().iter().position(|s| s == "Display").unwrap();
    view.choose_section(section);
    (section, view.rows().iter().position(|r| r.id == "variant").unwrap())
}

fn wait_for(scene: &mut emusen_qt_test::Scene<'_>, property: &str, value: &str) {
    let mut waited = 0;
    while scene.property("scene", property) != value && waited < 5000 {
        scene.wait(50);
        waited += 50;
    }
    assert_eq!(scene.property("scene", property), value, "{property} within 5 s; problem: {}", scene.property("scene", "problem"));
}

#[test]
fn qt_7_7_h7_a_save_made_while_the_opening_read_is_out_is_not_undone_by_it() {
    let _world = World::new("models-settings-read");
    let (section, row) = appearance();
    let mut harness = Harness::new();
    let scene_file = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/SettingsScene.qml");
    let mut scene = harness.load(&format!("file://{scene_file}"), 640, 480);
    assert!(scene.set_property("scene", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("scene", "open", "true"));
    wait_for(&mut scene, "ready", "true");
    assert!(scene.set_property("scene", "section", &section.to_string()));
    assert!(scene.set_property("scene", "row", &row.to_string()));
    assert_eq!(scene.property("scene", "light"), "false", "dark as stored");
    // Closed and opened again: a read begins. The GUI thread then sleeps without its event loop, so the read loads the
    // dark value and waits in the queue; the row is turned to Light and saved before it is delivered.
    assert!(scene.set_property_now("scene", "open", "false"));
    assert!(scene.set_property_now("scene", "open", "true"));
    std::thread::sleep(std::time::Duration::from_millis(1000));
    assert!(scene.set_property_now("scene", "turn", "1"));
    assert_eq!(scene.property("scene", "light"), "true", "the turn shows Light at once");
    scene.wait(1000);
    assert_eq!(scene.property("scene", "light"), "true", "the read begun before the save brought the old value back");
    assert_eq!(scene.property("scene", "problem"), "");
}
