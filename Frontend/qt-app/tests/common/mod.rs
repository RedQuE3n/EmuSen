//! What the program's tests share: a sandbox home with synthetic games, and waiting on a property.

use emusen_client_fixture::{library, World};
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;

pub fn wait_for(scene: &mut emusen_qt_test::Scene<'_>, object: &str, property: &str, value: &str, ms: i32) {
    let mut waited = 0;
    while scene.property(object, property) != value && waited < ms {
        scene.wait(50);
        waited += 50;
    }
    assert_eq!(scene.property(object, property), value, "{object}.{property} after {ms} ms");
}

/// A scratch tree whose settings name its ROM folder, holding one synthetic image for each (folder, file) given.
pub fn sandbox(name: &str, games: &[(&str, &str)]) -> World {
    let world = World::new(name);
    for (folder, file) in games {
        let roms = world.dir(&format!("Roms/{folder}"));
        let mut image = b"NES\x1a\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
        image.resize(16 + 16384, 0);
        std::fs::write(roms.join(file), image).unwrap();
    }
    let platform = Platform::load(library()).unwrap();
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = serde_json::json!(world.root.join("Roms").to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    world
}
