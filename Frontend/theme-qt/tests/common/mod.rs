//! The tests' library: the real clients over a scratch tree holding one synthetic NES game, read into the view a host feeds.

use emusen_client_fixture::{library, World};
use emusen_crystal_client::Config;
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt_models::clients::ClientSource;
use emusen_qt_models::library::{refresh, LibraryView};

/// The NES's cover shape as the systems table declares it (height over width), and the view read from the clients.
#[allow(dead_code)]
pub struct Read {
    pub view: LibraryView,
    pub nes_name: String,
    pub nes_shape: f64,
}

/// The view over `world`, whose ROM folder holds `Synthetic.nes`, an iNES header and 16 KiB of zeros.
pub fn read(world: &World) -> Read {
    let roms = world.dir("Roms");
    let mut image = b"NES\x1a\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
    image.resize(16 + 16384, 0);
    std::fs::write(world.dir("Roms/NES").join("Synthetic.nes"), image).unwrap();
    let path = library().to_str().unwrap();
    let platform = Platform::load(path).unwrap();
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = serde_json::json!(roms.to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    let none = Config { development: false, cores: Some(Vec::new()), player_cores: Some(Vec::new()) };
    let source = ClientSource::open(path, &none).unwrap();
    let nes = source.runtime.systems().unwrap().into_iter().find(|s| s.id == "nes").unwrap();
    let mut view = LibraryView::empty();
    view.apply(refresh(&source));
    assert!(view.problem.is_empty(), "{}", view.problem);
    Read { view, nes_name: nes.display_name, nes_shape: nes.cover_aspect }
}

/// Home's lines as the library model publishes them, with the scraper asking for the player.
pub const HOME_LINES: &str = r#"[{"status":"1 game","attention":false},{"status":"41 no art","attention":true},{"status":"Game packages","attention":false},{"status":"Checks dumps","attention":false},{"status":"The engines","attention":false},{"status":"Dark · 60% sound","attention":false},{"status":"Exit","attention":false}]"#;
