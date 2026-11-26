//! Settings' read off the GUI thread against a save made while it is out (EmuSen_Qt.md §7.7.H7): the read begun as
//! Settings opens has loaded the stored files and is waiting to be delivered when a row is turned and saved; once it is
//! delivered, the row keeps the value saved.

use emusen_client_fixture::{library, World};
use emusen_platform_client::galaxia::consts::{DIR_MEDIA, FILE_GAMES};
use emusen_platform_client::scrape::ScraperConfig;
use emusen_platform_client::{Lent, Platform};
use emusen_qt_models::settings::{self, SettingsView};
use emusen_qt_models::settings_model::qobject::SettingsModel;
use emusen_qt_models::settings_source::ClientSettings;
use emusen_qt_test::Harness;
use std::os::unix::fs::PermissionsExt;

const SQLITE: &str = "libsqlite3.so.0";

fn settings_model<'a>(scene: &'a mut emusen_qt_test::Scene<'_>) -> core::pin::Pin<&'a mut SettingsModel> {
    let ptr = scene.object("settings");
    unsafe { core::pin::Pin::new_unchecked(&mut *(ptr as *mut SettingsModel)) }
}

fn chmod(path: &std::path::Path, mode: u32) {
    let mut perms = std::fs::metadata(path).unwrap().permissions();
    perms.set_mode(mode);
    std::fs::set_permissions(path, perms).unwrap();
}

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
    let world = World::new("models-settings-read");
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

    // Regression: Outcome::Saved failure reverts in-memory toggle without wiping `problem`, and late close still emits `saved`.
    let config = world.root.join("Config");
    let config_files: Vec<_> = std::fs::read_dir(&config).unwrap().filter_map(Result::ok).map(|e| e.path()).filter(|p| p.is_file()).collect();
    for f in &config_files {
        chmod(f, 0o444);
    }
    chmod(&config, 0o555);
    assert!(scene.set_property_now("scene", "turn", "2"));
    let mut waited = 0;
    while !scene.property("scene", "problem").starts_with("The settings could not be saved:") && waited < 3000 {
        scene.wait(25);
        waited += 25;
    }
    assert!(scene.property("scene", "problem").starts_with("The settings could not be saved:"));
    waited = 0;
    while scene.property("scene", "light") != "true" && waited < 3000 {
        scene.wait(25);
        waited += 25;
    }
    assert_eq!(scene.property("scene", "light"), "true", "failed disk save reverted in-memory toggle");
    assert!(scene.property("scene", "problem").starts_with("The settings could not be saved:"), "revert kept failure notice");
    chmod(&config, 0o755);
    for f in &config_files {
        chmod(f, 0o644);
    }

    // Regression: SettingsModel::refresh_scrape stores `scrape_asked` and applies updated snapshots.
    let library_sec = settings_model(&mut scene).view().sections().iter().position(|s| s == "Library").unwrap();
    assert!(scene.set_property("scene", "section", &library_sec.to_string()));
    let scrape_row = settings_model(&mut scene).shown_rows().iter().position(|r| r.id == "scraperun").unwrap();
    assert!(settings_model(&mut scene).choose(scrape_row as i32));
    assert_eq!(settings_model(&mut scene).shown_rows()[0].value, "Not signed in");
    let platform = Platform::load(library()).unwrap();
    let media = platform.directory(DIR_MEDIA).unwrap().expect("media dir");
    let games_db = platform.file(FILE_GAMES).ok().map(|p| p.to_string_lossy().into_owned());
    let sqlite = Lent::open(SQLITE).expect("libsqlite3");
    let cfg = ScraperConfig {
        root: media.to_string_lossy().into_owned(),
        games_db,
        settings: serde_json::json!({ "openemu": false, "developer_files": true, "sources_alone": false, "test": { "offline": true } }),
        ..ScraperConfig::default()
    };
    platform.scraper_open(&cfg, Some(&sqlite)).expect("open offline scraper");
    std::fs::write(config.join("screenscraper.json"), serde_json::json!({ "ssid": "alice", "sspassword": "example-password" }).to_string()).unwrap();
    assert!(settings_model(&mut scene).choose(4));
    waited = 0;
    while !settings_model(&mut scene).view().scrape.signed_in && waited < 3000 {
        scene.wait(25);
        waited += 25;
    }
    assert!(settings_model(&mut scene).view().scrape.signed_in, "refresh_scrape delivered signed-in snapshot");
    assert_eq!(settings_model(&mut scene).shown_rows()[0].value, "alice");
    assert!(settings_model(&mut scene).back());

    // Closing Settings synchronously after an async save waits for disk completion before emitting `saved`.
    assert!(scene.set_property("scene", "section", &section.to_string()));
    assert!(scene.set_property("scene", "row", &row.to_string()));
    let before_saved: i32 = scene.property("scene", "savedCount").parse().unwrap();
    assert!(scene.set_property_now("scene", "turn", "3"));
    assert!(scene.set_property_now("scene", "open", "false"));
    waited = 0;
    while scene.property("scene", "savedCount").parse::<i32>().unwrap() <= before_saved && waited < 3000 {
        scene.wait(25);
        waited += 25;
    }
    assert_eq!(scene.property("scene", "savedCount").parse::<i32>().unwrap(), before_saved + 1);
}
