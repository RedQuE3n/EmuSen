//! The client against the platform library built from this commit, on the test core and synthetic images: every typed
//! reader reads what the runtime writes, the per-frame calls run, events are pulled, the session table answers, and a
//! failure carries the words its own last_error gave.

use emusen_client_fixture::{TEST_SYSTEM, World, library};
use emusen_crystal_client::{Api, Config, Error, Event, Open, Runtime, Scope, consts, frame_info, kinds, service_flags};

fn no_cores() -> Config {
    Config { development: false, cores: Some(Vec::new()), player_cores: Some(Vec::new()) }
}

/// A runtime whose only core is the test core, approved through the API as a player approves one.
fn with_test_core(world: &World) -> Runtime {
    let player = world.dir("Cores");
    world.player_core(&player);
    let api = Api::load(library()).unwrap();
    let rt = Runtime::new(
        &api,
        &Config { development: false, cores: Some(Vec::new()), player_cores: Some(vec![player.to_str().unwrap().into()]) },
    )
    .unwrap();
    let cores = rt.cores().unwrap();
    let row = cores
        .iter()
        .find(|c| c.directory.as_deref() == Some("player") && c.trust.as_deref() == Some("not_read"))
        .unwrap_or_else(|| panic!("the player's core, not yet read: {cores:?}"));
    rt.core_approve(row.sha256.as_deref().unwrap()).unwrap();
    rt
}

#[test]
fn the_library_s_version_is_this_binding_s_major() {
    let _world = World::new("version");
    let api = Api::load(library()).unwrap();
    assert_eq!(api.version().0, consts::MAJOR);
    assert!(!api.version_text().unwrap().is_empty());
}

#[test]
fn a_file_that_is_not_the_api_s_library_is_refused_with_words() {
    let world = World::new("not-a-library");
    let fake = world.root.join("libnot.so");
    std::fs::write(&fake, b"not a library").unwrap();
    let Err(Error::Load { path, words }) = Api::load(&fake) else { panic!("a file that is no library is refused") };
    assert!(path.ends_with("libnot.so") && !words.is_empty());
    let core = emusen_client_fixture::test_core();
    let Err(Error::Load { words, .. }) = Api::load(core) else { panic!("a library without the API is refused") };
    assert!(words.contains("emusen_crystal_abi_version"), "{words}");
}

#[test]
fn the_runtime_s_listings_read_into_their_records() {
    let _world = World::new("listings");
    let api = Api::load(library()).unwrap();
    let rt = Runtime::new(&api, &no_cores()).unwrap();
    let systems = rt.systems().unwrap();
    let nes = systems.iter().find(|s| s.id == "nes").expect("the table's NES");
    assert!(!nes.display_name.is_empty() && nes.extensions.iter().any(|e| e == ".nes") && nes.cover_aspect > 0.0);
    assert!(!nes.look.body.is_empty());
    assert!(rt.sessions().unwrap().is_empty());
    rt.shelves().unwrap();
    rt.engines().unwrap();
    rt.cores().unwrap();
    let codecs = rt.cheat_codec_list().unwrap();
    assert!(codecs.codecs.iter().any(|c| c.id == "snes.gg"), "{codecs:?}");
    assert!(rt.cheat_decode("no-such-codec", "0000").unwrap().is_none(), "an unknown codec is ABSENT, an answer");
    let encoded = rt.cheat_encode("snes.gg", 0x8000, 0xEA).unwrap().expect("the SNES's Game Genie encodes");
    let decoded = rt.cheat_decode("snes.gg", &encoded).unwrap().unwrap();
    assert_eq!(decoded.decode["value"], 0xEA, "{decoded:?}");
    rt.firmware_overview().unwrap();
    assert!(rt.firmware_status("no-such-system").unwrap().is_none());
}

#[test]
fn a_player_s_core_is_approved_routed_opened_and_run_through_the_api() {
    let world = World::new("session");
    let rt = with_test_core(&world);
    let engine =
        rt.engines().unwrap().into_iter().find(|e| e.systems.iter().any(|s| s == TEST_SYSTEM)).expect("the approved core's engine");
    assert!(engine.available, "{engine:?}");
    let test = rt.systems().unwrap().into_iter().find(|s| s.id == TEST_SYSTEM).expect("the system only the engine names");
    assert!(test.engines.contains(&engine.name) && test.controllers.ports >= 1, "{test:?}");
    let status = rt.firmware_status(TEST_SYSTEM).unwrap().expect("the engine names the system");
    // Not required by the core, but required here since it has no open replacement; the scratch tree holds none.
    assert_eq!(
        status.firmware.iter().map(|f| (f.file.as_str(), f.required, f.status.as_str())).collect::<Vec<_>>(),
        [("boot.rom", true, "missing")]
    );

    let game = world.game("game.tst", &[1, 2, 3, 4]);
    let path = game.to_str().unwrap();
    let route = rt.route(path).unwrap();
    assert_eq!((route.system.as_deref(), route.engine.as_deref()), (Some(TEST_SYSTEM), Some(engine.name.as_str())));

    let events = rt.subscribe(0, kinds::SESSION | kinds::SETTINGS).unwrap();
    let id = rt.open(&Open { path: Some(path.into()), flags: consts::OPEN_NO_BATTERY, ..Open::default() }).unwrap();
    assert!(id >= 1);
    let info = rt.session_info(id).unwrap();
    assert_eq!((info.system.as_str(), info.game.as_ref().map(|g| g.path.as_str())), (TEST_SYSTEM, Some(path)));
    assert_eq!(rt.sessions().unwrap().iter().map(|s| (s.id, s.current)).collect::<Vec<_>>(), [(id, true)]);

    let mut frame = frame_info();
    let (mut picture, mut sound) = (vec![0u8; 1 << 20], vec![0i16; 8192]);
    for i in 0..10 {
        rt.input_button(id, 0, 0, i % 2 == 0).unwrap();
        rt.frame(id, &mut frame).unwrap();
        assert_eq!(frame.ran, 1);
        let n = rt.picture_copy(id, &mut picture).unwrap();
        assert_eq!(n, (frame.width * frame.height * 4) as usize);
        rt.audio_drain(id, &mut sound).unwrap();
    }
    assert_eq!(frame.total_frames, 10);

    let schema = rt.settings_schema(id, None, None).unwrap();
    let rate = schema.iter().find(|s| s.key == "Rate").expect("the core's Rate");
    assert_eq!((rate.kind.as_str(), rate.choices.as_deref()), ("choice", Some(&["32000".to_string(), "48000".to_string()][..])));
    assert!(schema.iter().any(|s| s.key == "Engine"), "the engine's choice is a row");
    rt.settings_set(0, Scope::Console, Some(TEST_SYSTEM), "Rate", Some("48000")).unwrap();
    assert_eq!(rt.settings_get(0, Scope::Console, Some(TEST_SYSTEM), "Rate").unwrap().as_deref(), Some("48000"));
    assert!(rt.settings_get(0, Scope::Console, Some(TEST_SYSTEM), "NoSuchKey").unwrap().is_none());

    let table = rt.session_service(id, 0).unwrap();
    assert_eq!(table.info(id).unwrap().id, id);
    let state = world.root.join("one.state");
    table.state_save(id, Some(state.to_str().unwrap())).unwrap();
    table.state_load(id, state.to_str().unwrap()).unwrap();
    rt.close(id).unwrap();
    let closed = table.info(id).unwrap_err();
    assert_eq!(closed.status(), Some(consts::EMUSEN_SVC_NO_SESSION), "{closed}");

    assert!(events.wait(1000).unwrap());
    let seen = events.drain().unwrap();
    assert!(seen.iter().any(|e| matches!(e, Event::Session { op, .. } if op == "opened")), "{seen:?}");
    assert!(seen.iter().any(|e| matches!(e, Event::Settings { key, .. } if key == "Rate")), "{seen:?}");
    assert!(seen.iter().any(|e| matches!(e, Event::Session { op, .. } if op == "closed")), "{seen:?}");
}

#[test]
fn a_failure_carries_its_own_words() {
    let world = World::new("failures");
    let api = Api::load(library()).unwrap();
    let rt = Runtime::new(&api, &no_cores()).unwrap();
    let nothing = world.game("nothing.zzz", &[0]);
    let refused = rt.open(&Open { path: Some(nothing.to_str().unwrap().into()), ..Open::default() }).unwrap_err();
    assert_eq!(refused.status(), Some(consts::EMUSEN_CRYSTAL_NO_ENGINE), "{refused}");
    assert!(!refused.to_string().starts_with(" ("), "the runtime's words: {refused}");
    let no_game = rt.session_service(0, service_flags::NO_GAME).unwrap();
    let words = no_game.cheat_op(0, consts::CHEAT_LIST, &serde_json::json!({})).unwrap_err();
    assert_eq!(words.status(), Some(consts::EMUSEN_SVC_NO_SESSION));
    assert!(words.to_string().contains("No ROM loaded"), "the table's own last_error: {words}");
    assert!(no_game.sessions().unwrap().is_empty());
}

#[test]
fn a_cheat_change_is_made_once_and_answers_its_number() {
    let world = World::new("cheats");
    let rt = with_test_core(&world);
    let game = world.game("game.tst", &[1, 2, 3, 4]);
    let id = rt.open(&Open { path: Some(game.to_str().unwrap().into()), flags: consts::OPEN_NO_BATTERY, ..Open::default() }).unwrap();
    let table = rt.session_service(id, 0).unwrap();
    let file = serde_json::json!({ "Cheats": [{ "Kind": "RamPoke", "Description": "held", "Enabled": false,
        "Writes": [{ "Space": "RAM", "Address": "2", "Value": "5A" }] }] });
    assert_eq!(table.cheat_op(id, consts::CHEAT_IMPORT, &file).unwrap(), 1, "IMPORT answers the number loaded");
    let listed = table.cheat_op(id, consts::CHEAT_LIST, &serde_json::json!({})).unwrap();
    let cheats = listed["cheats"].as_array().unwrap();
    assert_eq!(cheats.len(), 1, "loaded once, not again by a length asked: {listed}");
    let cheat = cheats[0]["id"].as_i64().unwrap();
    assert_eq!(table.cheat_op(id, consts::CHEAT_ENABLE, &serde_json::json!({ "id": cheat })).unwrap(), 0);
    let mut frame = frame_info();
    let mut held = [0u8; 1];
    for _ in 0..3 {
        rt.frame(id, &mut frame).unwrap();
        table.space_read(id, "RAM", 2, &mut held).unwrap();
        assert_eq!(held[0], 0x5A, "switched on, the poke holds the byte the core counts");
    }
    assert_eq!(table.cheat_op(id, consts::CHEAT_DISABLE, &serde_json::json!({ "id": cheat })).unwrap(), 0);
    rt.frame(id, &mut frame).unwrap();
    table.space_read(id, "RAM", 2, &mut held).unwrap();
    assert_eq!(held[0], 0x5B, "switched off, the count runs on");
    rt.close(id).unwrap();
}
