//! The Qt front end's runtimes are lent SQLite (EmuSen_Qt.md §7.9), so they keep their state records: a game's page's
//! Delete State forgets the state's record rather than being refused, as it was while every runtime was made without it.

use emusen_client_fixture::{library, World};
use emusen_crystal_client::{Api, Config, Open, Runtime};
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_platform_client::Lent;
use emusen_qt_models::clients::{runtime, runtime_config, runtime_lending, ClientSource, SQLITE};

#[test]
fn every_runtime_the_program_makes_keeps_state_records() {
    let world = World::new("qt-models-records");
    let platform = Platform::load(library()).unwrap();
    let settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    let api = Api::load(library().to_str().unwrap()).unwrap();
    let state = world.root.join("Saves/game.state1");
    let at = state.to_str().unwrap();

    // Made without the lend, the runtime refuses its records' calls: the defect this piece fixed.
    let bare = Runtime::new(&api, &Config::default()).unwrap();
    let refused = bare.state_forget(at).unwrap_err().to_string();
    assert!(refused.contains("SQLite"), "{refused}");

    // The models' runtime, with and without a cores folder named, and the library's source each lend it.
    for cores in ["", world.root.join("cores").to_str().unwrap()] {
        let config = runtime_config(&platform, cores).unwrap();
        let (runtime, unkept) = runtime(&api, &config).unwrap();
        assert_eq!(unkept, None, "nothing to say where SQLite is lent");
        runtime.state_forget(at).unwrap_or_else(|e| panic!("the record forgotten with cores '{cores}': {e}"));
        assert!(runtime.state_record(at).unwrap().is_none(), "no record left");
    }
    let source = ClientSource::open(library().to_str().unwrap(), &Config::default()).unwrap();
    source.runtime.state_forget(at).unwrap();
}

#[test]
fn without_sqlite_a_runtime_lends_nothing_opens_games_and_says_its_states_are_not_recorded() {
    let world = World::new("qt-models-no-sqlite");
    let platform = Platform::load(library()).unwrap();
    let settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    let api = Api::load(library().to_str().unwrap()).unwrap();
    let player = world.dir("Cores");
    world.player_core(&player);
    let config = Config { development: false, cores: Some(Vec::new()), player_cores: Some(vec![player.to_str().unwrap().into()]) };

    // The system's SQLite as it loads, then a soname no system has, as on a machine without SQLite.
    assert!(runtime_lending(&api, &config, Lent::open(SQLITE).map_err(|e| e.to_string())).unwrap().1.is_none());
    let (rt, unkept) = runtime_lending(&api, &config, Lent::open("libsqlite3-not-here.so.0").map_err(|e| e.to_string())).unwrap();
    let unkept = unkept.expect("the player is told");
    assert!(unkept.starts_with("The states' records are not kept, since SQLite could not be loaded: "), "{unkept}");
    let sha = rt.cores().unwrap().into_iter().find(|c| c.directory.as_deref() == Some("player")).and_then(|c| c.sha256).expect("the player's core");
    rt.core_approve(&sha).unwrap();
    let game = world.game("game.tst", &[1, 2, 3, 4]);
    let session = rt.open(&Open { path: Some(game.to_str().unwrap().into()), ..Open::default() }).expect("the game opens");
    rt.close(session).unwrap();
    let refused = rt.state_forget(world.root.join("Saves/game.state1").to_str().unwrap()).unwrap_err().to_string();
    assert!(refused.contains("SQLite"), "no records kept: {refused}");
}
