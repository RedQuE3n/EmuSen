//! The Qt front end's runtimes are lent SQLite (EmuSen_Qt.md §7.9), so they keep their state records: a game's page's
//! Delete State forgets the state's record rather than being refused, as it was while every runtime was made without it.

use emusen_client_fixture::{library, World};
use emusen_crystal_client::{Api, Config, Runtime};
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt_models::clients::{runtime, runtime_config, ClientSource};

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
        let runtime = runtime(&api, &config).unwrap();
        runtime.state_forget(at).unwrap_or_else(|e| panic!("the record forgotten with cores '{cores}': {e}"));
        assert!(runtime.state_record(at).unwrap().is_none(), "no record left");
    }
    let source = ClientSource::open(library().to_str().unwrap(), &Config::default()).unwrap();
    source.runtime.state_forget(at).unwrap();
}
