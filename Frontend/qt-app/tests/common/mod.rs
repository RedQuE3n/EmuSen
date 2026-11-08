//! What the program's tests share: a sandbox home with synthetic games, and waiting on a property.

use emusen_client_fixture::{library, World};
use emusen_crystal_client::Config;
use emusen_qt_models::clients::ClientSource;
use emusen_qt_models::library::{world_line_of, LibrarySource};
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

/// A scratch tree like `sandbox` at a library's scale: `count` games in each (folder, extension), each a hard link to one
/// synthetic image, so thousands of games take one file's room.
#[allow(dead_code)]
pub fn large_sandbox(name: &str, shelves: &[(&str, &str, usize)]) -> World {
    let world = sandbox(name, &[]);
    for (folder, ext, count) in shelves {
        let roms = world.dir(&format!("Roms/{folder}"));
        let first = roms.join(format!("Game 00000.{ext}"));
        let mut image = b"NES\x1a\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
        image.resize(16 + 16384, 0);
        std::fs::write(&first, image).unwrap();
        for n in 1..*count {
            std::fs::hard_link(&first, roms.join(format!("Game {n:05}.{ext}"))).unwrap();
        }
    }
    world
}

/// The courts canvas's consoles, as Mistress's picture states them (MoonrisePictureTool.Consoles).
pub const CONSOLES: [&str; 11] = ["nes", "snes", "n64", "gb", "gbc", "sms", "md", "mcd", "psx", "ps2", "dreamcast"];

/// The canvas's consoles' worlds as lines, from this commit's systems table in its order.
#[allow(dead_code)]
pub fn canvas_worlds() -> Vec<String> {
    let source = ClientSource::open(library().to_str().unwrap(), &Config::default()).unwrap();
    let worlds: Vec<String> =
        source.systems().unwrap().iter().filter(|s| CONSOLES.contains(&s.id.as_str())).map(world_line_of).collect();
    assert_eq!(worlds.len(), CONSOLES.len(), "every console of the canvas is in the table");
    worlds
}
