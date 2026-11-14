//! The library view's source over the real clients, built from this commit, in a scratch tree: the scan's request is
//! read from the player's settings and the shelves, games.db is where Galaxia names it, and a refresh of a synthetic game
//! makes the places and its console.

use emusen_client_fixture::{library, World};
use emusen_crystal_client::Config;
use emusen_esde::loader::ThemeSystem;
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
    let rows = refresh(&source).unwrap().rows;
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
    // The NES's engine through the runtime's Setting(console, "Engine") with nothing stored: the console's default, named
    // from the cores listing, so a stored choice would be read the same way.
    let engine = source.runtime.settings_get(0, emusen_crystal_client::Scope::Console, Some("nes"), "Engine").unwrap().unwrap_or_default();
    assert_eq!(rows[3].engine, if engine.is_empty() { nes.engines.first().cloned().unwrap_or_default() } else { engine });
    assert!(!rows[3].engine_name.is_empty() && rows[3].facts.ends_with("an EmuSen core"), "{}", rows[3].facts);
    let games = platform.file(FILE_GAMES).unwrap();
    assert!(games.starts_with(&world.root) && games.is_file(), "games.db where Galaxia names it: {games:?}");

    let refused = ClientSource::open(world.root.join("libnot.so").to_str().unwrap(), &none).err().expect("no library there");
    assert!(refused.contains("libnot.so"), "{refused}");
}

#[test]
fn with_a_theme_chosen_the_real_library_s_rows_carry_their_es_de_names() {
    let world = World::new("models-clients-esde");
    let roms = world.dir("Roms");
    let path = library().to_str().unwrap();
    let platform = Platform::load(path).unwrap();
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = serde_json::json!(roms.to_str().unwrap());
    settings["LibraryStyle"] = serde_json::json!("Theme");
    settings["BigPictureTheme"] = serde_json::json!(world.dir("Themes/plain").to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    let mut image = b"NES\x1a\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
    image.resize(16 + 16384, 0);
    std::fs::write(world.dir("Roms/NES").join("Synthetic.nes"), &image).unwrap();
    std::fs::write(world.dir("Roms/NES/Puzzle").join("Foldered.nes"), &image).unwrap();
    std::fs::write(world.dir("Roms/SNES").join("Synthetic.sfc"), vec![0u8; 0x8000]).unwrap();
    std::fs::write(world.dir("Roms/GB").join("Synthetic.gb"), vec![0u8; 0x8000]).unwrap();

    let none = Config { development: false, cores: Some(Vec::new()), player_cores: Some(Vec::new()) };
    let source = ClientSource::open(path, &none).unwrap();
    let read = refresh(&source).unwrap();
    assert!(read.esde_settings.themed);
    let shelves: serde_json::Value = serde_json::from_str(&read.esde.shelves).unwrap();
    let nes = source.runtime.systems().unwrap().into_iter().find(|s| s.id == "nes").unwrap();
    assert_eq!(shelves[0]["system"].as_str(), Some("nes"), "{shelves}");
    assert_eq!((shelves[0]["name"].as_str(), shelves[0]["full_name"].as_str()), (Some(nes.esde_system.as_str()), Some(nes.esde_full_name.as_str())), "the systems table's names");
    // ES-DE 3.4.1's user guide, "Supported game systems": a theme's ${system.fullName}, never the program's display_name.
    for (system, full) in [("nes", "Nintendo Entertainment System"), ("snes", "Nintendo SNES (Super Nintendo)"), ("gb", "Nintendo Game Boy")] {
        let shelf = shelves.as_array().unwrap().iter().find(|s| s["system"] == system).unwrap_or_else(|| panic!("no {system} shelf: {shelves}"));
        let seen = ThemeSystem::new(shelf["name"].as_str().unwrap(), shelf["full_name"].as_str().unwrap(), shelf["theme"].as_str().unwrap());
        assert_eq!(seen.variables()["system.fullName"], full, "{system}");
    }
    let folder = &shelves[0]["games"][0];
    assert_eq!((folder["name"].as_str(), folder["folder"].as_bool(), folder["folder_path"].as_str()), (Some("Puzzle"), Some(true), Some("Puzzle")), "the scan's folder (emusen_library.h 1.3) makes a folder entry");
    let game = &shelves[0]["games"][1];
    assert!(game["file"].as_str().unwrap().ends_with("Synthetic.nes"), "{game}");
    assert_eq!((game["name"].as_str(), game["play_count"].as_i64(), game["favourite"].as_bool()), (Some("Synthetic"), Some(0), Some(false)));
    eprintln!("the row: {game}");
}

#[test]
fn qt_7_7_h_the_profile_every_console_has_is_named_by_endymion() {
    let _world = World::new("models-default-profile");
    let source = emusen_qt_models::settings_source::ClientSettings::open(library().to_str().unwrap()).unwrap();
    let bindings = source.bindings(&["SNES"]).unwrap();
    let platform = Platform::load(library()).unwrap();
    let none = serde_json::json!({});
    let named = platform.bindings_choose(Some(&none), Some(&none), "SNES", 1, None, Some("no such profile")).unwrap();
    assert_eq!(bindings.default_profile, named["profile"].as_str().unwrap(), "the name bindings_choose gives a profile the console lacks");
    assert_eq!(bindings.profiles("SNES"), [bindings.default_profile.clone()]);
}

#[test]
fn a_refresh_reads_the_collections_and_the_last_games_cheats_and_play_time_is_kept_under_the_limit() {
    use emusen_platform_client::galaxia::consts::MODEL_CHEAT_FILE;
    use emusen_qt_models::library::{refresh, LibrarySource};
    let world = World::new("models-clients-doors");
    let roms = world.dir("Roms");
    let path = library().to_str().unwrap();
    let platform = Platform::load(path).unwrap();
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = serde_json::json!(roms.to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    let mut image = b"NES\x1a\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
    image.resize(16 + 16384, 0);
    let game = world.dir("Roms/NES").join("Synthetic.nes");
    std::fs::write(&game, image).unwrap();
    let game = game.to_str().unwrap();
    let none = Config { development: false, cores: Some(Vec::new()), player_cores: Some(Vec::new()) };
    let source = ClientSource::open(path, &none).unwrap();

    let read = refresh(&source).unwrap();
    assert_eq!((read.catalog.collections.len(), read.catalog.last_cheats), (0, 0), "nothing played, no collections");
    let id = source.library.change(&serde_json::json!({ "op": "create_collection", "name": "Couch", "at": "2026-11-13T10:00:00Z" })).unwrap();
    source.library.change(&serde_json::json!({ "op": "add_to_collection", "id": id, "path": game })).unwrap();
    source.library.change(&serde_json::json!({ "op": "started", "path": game, "at": "2026-11-13T10:00:00Z" })).unwrap();
    let console = source.runtime.systems().unwrap().into_iter().find(|s| s.id == "nes").unwrap().console;
    let (name, category) = platform.cheat_game_list(Some(&console), game).unwrap().unwrap();
    let cheat = serde_json::json!({ "Kind": "RamPoke", "Description": "Lives", "Enabled": false, "Writes": [] });
    platform.model_save(MODEL_CHEAT_FILE, Some(&category), &format!("{name}.json"), &serde_json::json!({ "Cheats": [cheat] })).unwrap();
    let read = refresh(&source).unwrap();
    assert_eq!(read.catalog.collections.iter().map(|c| (c.name.as_str(), c.members.clone())).collect::<Vec<_>>(), [("Couch", vec![game.to_owned()])]);
    assert_eq!(read.catalog.last_cheats, 1, "the game last played has one cheat");

    // Play time: none with tracking off, none for a launch past the limit, else kept and added.
    assert_eq!(source.played_for(game, 60.0, 0).unwrap(), None);
    assert_eq!(source.played_for(game, 7_200.0, 1).unwrap(), None);
    assert_eq!(source.played_for(game, 90.0, 8).unwrap(), Some(90.0));
    assert_eq!(source.played_for(game, 30_000.0, 24).unwrap(), Some(30_000.0), "24 is no limit");
    assert_eq!(source.library.record(game).unwrap().unwrap().play_seconds, 30_090.0);
}
