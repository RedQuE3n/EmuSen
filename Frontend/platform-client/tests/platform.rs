//! The client against the platform library built from this commit: the interface matched exactly, Galaxia's tree and
//! models in a scratch tree, the library scanning a synthetic game, simulated pads, the firmware folders, the scrape
//! service, Galaxia's files and Endymion's binding files, and failures carrying the words their own last_error gave.

use emusen_client_fixture::{World, library, test_core};
use emusen_platform_client::firmware::FoldersRequest;
use emusen_platform_client::bindings::consts as bind;
use emusen_platform_client::galaxia::consts::{DIR_CONFIG, DIR_LIBRARY, DIR_MEDIA, DIR_SAVE_STATES, FILE_GAMES, FILE_GRAPHICS, FILE_MEDIA, FILE_SETTINGS, MODEL_APP_SETTINGS, SAVE_PICTURE, SAVE_STATE};
use emusen_platform_client::library::{Library, ScanRequest};
use emusen_platform_client::pads::{Pads, SimPad, SimSet, consts as pad};
use emusen_platform_client::scrape::ScraperConfig;
use emusen_platform_client::sounds::InterfacePlayer;
use emusen_platform_client::{Error, INTERFACE, Lent, Platform};
use serde_json::json;

#[test]
fn the_library_is_matched_exactly_and_anything_else_refused_with_words() {
    let world = World::new("load");
    assert_eq!(Platform::load(library()).unwrap().interface(), INTERFACE);
    let fake = world.root.join("libnot.so");
    std::fs::write(&fake, b"not a library").unwrap();
    assert!(matches!(Platform::load(&fake), Err(Error::Load { words, .. }) if !words.is_empty()));
    let Err(Error::Load { words, .. }) = Platform::load(test_core()) else { panic!("a library without the interface is refused") };
    assert!(words.contains("emusen_platform_abi_version"), "{words}");
}

#[test]
fn galaxia_s_tree_and_models_are_the_scratch_tree_s() {
    let world = World::new("galaxia");
    let p = Platform::load(library()).unwrap();
    let config = p.directory(DIR_CONFIG).unwrap().unwrap();
    assert!(config.starts_with(&world.root), "{config:?}");
    assert!(p.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().is_none(), "no file yet");
    let mut settings = p.model_new(MODEL_APP_SETTINGS).unwrap();
    assert!(settings.get("RomDirectory").is_some(), "{settings}");
    settings["RomDirectory"] = json!(world.root.join("Roms").to_str().unwrap());
    p.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    let back = p.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    assert_eq!(back["RomDirectory"], settings["RomDirectory"]);
    assert!(p.config_path(None, "appsettings.json").unwrap().is_file());
    let states = p.directory(DIR_SAVE_STATES).unwrap().unwrap();
    let state = p.save_path(SAVE_STATE, "/games/Cobalt Harbor.sfc", None, 3).unwrap();
    assert_eq!(state, states.join("Cobalt Harbor.slot3.state"), "a slot's state in the tree's Save States");
    assert_eq!(p.save_path(SAVE_PICTURE, state.to_str().unwrap(), None, 0).unwrap(), states.join("Cobalt Harbor.slot3.png"));
}

#[test]
fn galaxia_names_its_files_and_endymion_reads_and_writes_the_binding_files() {
    let world = World::new("bindings");
    let p = Platform::load(library()).unwrap();
    assert_eq!(p.file(FILE_SETTINGS).unwrap(), p.config_path(None, "appsettings.json").unwrap());
    assert_eq!(p.file(FILE_GAMES).unwrap(), p.directory(DIR_LIBRARY).unwrap().unwrap().join("games.db"));
    assert_eq!(p.file(FILE_MEDIA).unwrap(), p.directory(DIR_MEDIA).unwrap().unwrap().join("media.db"));
    assert!(p.file(FILE_SETTINGS).unwrap().starts_with(&world.root));
    // Since 46, GraphicsConfig's file by its number, so a front end never names it.
    assert_eq!(p.file(FILE_GRAPHICS).unwrap(), p.config_path(None, "graphics.json").unwrap());
    assert!(matches!(p.file(99), Err(Error::Status { .. })));

    let keys = p.bindings_load(bind::KEYS, &["NES", "SNES"]).unwrap();
    assert_eq!(keys["SNES"], p.bindings_defaults(bind::KEYS).unwrap());
    let mut gamepad = p.bindings_load(bind::GAMEPAD, &["SNES"]).unwrap();
    gamepad["Players"]["SNES"]["2"] = json!({ "A": "west" });
    p.bindings_save(bind::GAMEPAD, &gamepad).unwrap();
    let written = std::fs::read_to_string(p.config_path(None, "gamepadbindings.json").unwrap()).unwrap();
    assert!(written.contains("\"SNES Player 2\": {\n    \"A\": \"West\""), "{written}");
    assert_eq!(p.bindings_load(bind::GAMEPAD, &["SNES"]).unwrap()["Players"]["SNES"]["2"]["A"], "West");
    // Interface 43: a player's buttons as SDL's numbers, player 2's own and player 3 with player 1's (South 0, West 2).
    let (a, b) = (8, 0);
    assert_eq!(p.bindings_bound("SNES", 2, None).unwrap()[a], 2);
    assert_eq!(p.bindings_bound("snes", 3, Some("/games/Some Game.sfc")).unwrap()[a], p.bindings_bound("SNES", 1, None).unwrap()[a]);
    assert_eq!(p.bindings_bound("SNES", 1, None).unwrap()[b], 0);
    assert!(matches!(p.bindings_bound("SNES", 9, None), Err(Error::Status { .. })));

    std::fs::write(p.config_path(None, "hotkeybindings.json").unwrap(), r#"{"FastFoward": "Tab"}"#).unwrap();
    assert_eq!(p.bindings_load(bind::HOTKEYS, &[]).unwrap(), p.bindings_defaults(bind::HOTKEYS).unwrap());
    let said = p.diagnostics().unwrap();
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].ends_with("hotkeybindings.json: 'FastFoward' is not a valid HotkeyAction. Did you mean 'FastForward'? Falling back to defaults."), "{said:?}");
    assert!(matches!(p.bindings_defaults(bind::PROFILES), Err(Error::Status { .. })));
    let Err(Error::Status { words, .. }) = p.bindings_save(bind::KEYS, &json!({ "NES": { "Up": "Upp" } })) else { panic!("a misspelled key is not saved") };
    assert!(words.contains("'Upp' is not a valid Key. Did you mean 'Up'?"), "{words}");
    // A game's name in the profiles is Endymion's (since 47): the file's name without its last extension.
    assert_eq!(p.bindings_game_key("/a.b/Game.v1.sfc").unwrap(), "Game.v1");
    assert_eq!(p.bindings_game_key("C:\\Roms\\Hero.cue").unwrap(), "Hero");
    assert_eq!(p.bindings_game_key(".hidden").unwrap(), ".hidden");
    // Each control's word with the button it is or the stick's way (since 47).
    let controls = p.bindings_controls().unwrap();
    assert_eq!(controls[0], json!({ "word": "B", "button": 0 }));
    assert_eq!(controls[19], json!({ "word": "LeftStickRight", "axis": 0, "toward": 1 }));
}

#[test]
fn the_library_scans_a_synthetic_game_and_keeps_its_records() {
    let world = World::new("library");
    let p = Platform::load(library()).unwrap();
    let sqlite = Lent::open("libsqlite3.so.0").unwrap();
    let roms = world.dir("Roms/NES");
    let game = roms.join("Synthetic.nes");
    let mut image = b"NES\x1a\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
    image.resize(16 + 16384, 0);
    std::fs::write(&game, image).unwrap();
    let lib = Library::open(&p, &world.root.join("games.db"), &sqlite).unwrap();
    let request = ScanRequest {
        roots: vec![world.root.join("Roms").to_str().unwrap().into()],
        systems: vec!["nes".into()],
        ..ScanRequest::default()
    };
    assert_eq!(lib.scan(&request).unwrap(), 1);
    assert!(!lib.progress().unwrap().running);
    let games = lib.games(None).unwrap();
    let path = game.to_str().unwrap();
    assert_eq!((games.status.as_str(), games.games.len()), ("ok", 1));
    assert_eq!((games.games[0].path.as_str(), games.games[0].system.as_deref()), (path, Some("nes")));
    assert!(lib.games(Some("snes")).unwrap().games.is_empty());

    assert!(lib.record(path).unwrap().is_none(), "never played");
    lib.change(&json!({ "op": "toggle_favourite", "path": path })).unwrap();
    assert!(lib.record(path).unwrap().unwrap().favourite);
    let id = lib.change(&json!({ "op": "create_collection", "name": "Synthetic", "at": "2026-11-08T00:00:00Z" })).unwrap();
    lib.change(&json!({ "op": "add_to_collection", "id": id, "path": path })).unwrap();
    let collections = lib.collections().unwrap();
    assert_eq!(collections.iter().map(|c| (c.name.as_str(), c.count)).collect::<Vec<_>>(), [("Synthetic", 1)]);
    lib.change(&json!({ "op": "started", "path": path, "at": "2026-11-08T00:00:00Z" })).unwrap();
    assert_eq!(lib.recent(5, &[path.to_string()]).unwrap(), [path]);

    let refused = lib.get(&json!({ "what": "no-such-read" })).unwrap_err();
    assert!(refused.status().is_some() && !refused.to_string().starts_with(" ("), "the library's own words: {refused}");
}

#[test]
fn simulated_pads_are_opened_read_driven_and_let_go() {
    let _world = World::new("pads");
    let p = Platform::load(library()).unwrap();
    let set = SimSet::new(&p).unwrap();
    let one = SimPad::new(&p, None).unwrap();
    set.connect(&one).unwrap();
    let pads = Pads::simulated(&p, &set).unwrap();
    let opened = pads.start().unwrap();
    assert_eq!(opened.len(), 1, "{opened:?}");
    assert!(opened[0].opened && !opened[0].announce, "present at start, not announced");
    let key = opened[0].key;
    assert_eq!(pads.open().unwrap(), [key]);
    assert_eq!(pads.text(key, pad::NAME).unwrap().as_deref(), Some("Simulated pad"));
    assert_eq!(pads.get(key, pad::FAMILY, 0).unwrap(), pad::FAMILY_GENERIC as i64, "a pad of no known type or name");
    one.set(pad::SIM_PAD_KIND, 6).unwrap();
    assert_eq!(pads.get(key, pad::FAMILY, 0).unwrap(), pad::FAMILY_PLAYSTATION as i64, "SDL's PS5 type");
    one.set(pad::SIM_PAD_KIND, 1).unwrap();
    one.set_text(pad::SIM_PAD_NAME, "Nintendo Switch Pro Controller").unwrap();
    assert_eq!(pads.get(key, pad::FAMILY, 0).unwrap(), pad::FAMILY_NINTENDO as i64, "a standard pad by its name");

    one.press(0, true).unwrap();
    pads.update().unwrap();
    assert_eq!(pads.get(key, pad::RAW_PRESSED, 0).unwrap(), 1, "SDL's south button held");
    pads.rumble(key, 1000, 2000).unwrap();
    assert_eq!(one.get(pad::SIM_PAD_RUMBLE).unwrap(), 1000 << 16 | 2000);
    pads.set_setting(pad::SETTING_STICK_DEADZONE, 0.25).unwrap();
    assert_eq!(pads.setting(pad::SETTING_STICK_DEADZONE).unwrap(), 0.25);

    set.disconnect(&one).unwrap();
    let closed = pads.poll().unwrap();
    assert!(closed.iter().any(|e| !e.opened && e.key == key), "{closed:?}");
    assert_eq!(pads.get(key, pad::FAMILY, 0).unwrap(), pad::FAMILY_NINTENDO as i64, "kept once it closes");
}

#[test]
fn the_interface_player_is_refused_with_words_before_sdl_is_lent() {
    let _world = World::new("no-sdl");
    let p = Platform::load(library()).unwrap();
    let Err(refused) = InterfacePlayer::new(&p) else { panic!("no player without SDL") };
    assert!(refused.status().is_some(), "{refused}");
}

#[test]
fn the_firmware_folders_follow_the_setting() {
    let world = World::new("firmware");
    let p = Platform::load(library()).unwrap();
    let home = world.dir("Home");
    let store = home.join("Firmware");
    let request = FoldersRequest { setting: None, home: home.to_str().unwrap().into(), store: store.to_str().unwrap().into() };
    let folders = p.firmware_folders(&request).unwrap();
    assert_eq!(folders.state, "Unset");
    assert!(folders.folders.iter().any(|f| f == store.to_str().unwrap()), "{folders:?}");
}

#[test]
fn the_scrape_service_opens_one_scraper_reads_and_closes_it() {
    let world = World::new("scrape");
    let p = Platform::load(library()).unwrap();
    let config = ScraperConfig { root: world.dir("Media").to_str().unwrap().into(), settings: json!({}), ..ScraperConfig::default() };
    let sqlite = Lent::open("libsqlite3.so.0").unwrap();
    let serial = p.scraper_open(&config, Some(&sqlite)).unwrap();
    assert!(serial > 0);
    let service = p.scrape_service().unwrap();
    let kinds = service.get(&json!({ "what": "kinds" })).unwrap();
    assert!(kinds.is_array() || kinds.is_object(), "{kinds}");
    let refused = service.get(&json!({ "what": "no-such-read" })).unwrap_err();
    assert_eq!(refused.status(), Some(-1163), "BAD_REQUEST, with the table's own words: {refused}");
    assert!(!refused.to_string().starts_with(" ("), "{refused}");
    assert!(p.scraper_close(serial).unwrap());
    assert!(!p.scraper_close(serial).unwrap(), "a closed scraper is no longer the one open");
}
