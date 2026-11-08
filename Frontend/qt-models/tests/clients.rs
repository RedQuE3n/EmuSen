//! The library view's source over the real clients, built from this commit, in a scratch tree: the scan's request is
//! read from the player's settings and the shelves, games.db is where Galaxia names it, and a refresh of a synthetic game
//! makes the places and its console.

use emusen_client_fixture::{library, World};
use emusen_crystal_client::Config;
use emusen_platform_client::galaxia::consts::{FILE_GAMES, MODEL_APP_SETTINGS};
use emusen_platform_client::Platform;
use emusen_qt_models::clients::ClientSource;
use emusen_qt_models::library::{refresh, Kind};

#[test]
fn the_client_source_scans_the_player_s_folder_and_makes_the_rows() {
    let world = World::new("models-clients");
    let roms = world.dir("Roms");
    let path = library().to_str().unwrap();
    let platform = Platform::load(path).unwrap();
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = serde_json::json!(roms.to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();

    let none = Config { development: false, cores: Some(Vec::new()), player_cores: Some(Vec::new()) };
    let source = ClientSource::open(path, &none).unwrap();
    let request = source.scan_request().unwrap();
    assert_eq!(request.roots, [roms.to_str().unwrap()]);
    let shelves: Vec<String> = source.runtime.shelves().unwrap().into_iter().map(|s| s.system).collect();
    assert_eq!(request.systems, shelves, "the scan's systems are the shelves'");
    assert!(shelves.iter().any(|s| s == "nes"), "{shelves:?}");

    // A synthetic NES image, an iNES header and 16 KiB of zeros, is the library's one game and the NES its one console.
    let mut image = b"NES\x1a\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
    image.resize(16 + 16384, 0);
    std::fs::write(world.dir("Roms/NES").join("Synthetic.nes"), image).unwrap();
    let rows = refresh(&source).unwrap();
    let seen: Vec<(Option<Kind>, &str, usize)> = rows.iter().map(|r| (r.kind, r.key.as_str(), r.count)).collect();
    assert_eq!(
        seen,
        [
            (Some(Kind::Place), "all", 1),
            (Some(Kind::Place), "favourites", 0),
            (Some(Kind::Place), "recent", 0),
            (Some(Kind::Console), "nes", 1)
        ]
    );
    let nes = source.runtime.systems().unwrap().into_iter().find(|s| s.id == "nes").unwrap();
    assert_eq!((rows[3].name.as_str(), rows[3].hue), (nes.display_name.as_str(), nes.look.hue), "the table's name and hue");
    let games = platform.file(FILE_GAMES).unwrap();
    assert!(games.starts_with(&world.root) && games.is_file(), "games.db where Galaxia names it: {games:?}");

    let refused = ClientSource::open(world.root.join("libnot.so").to_str().unwrap(), &none).err().expect("no library there");
    assert!(refused.contains("libnot.so"), "{refused}");
}
