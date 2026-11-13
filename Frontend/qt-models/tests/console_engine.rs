//! A console's engine chosen in Settings reaches the readers that show it (EmuSen_Qt.md §8.5): Settings stores the
//! choice under the console's name, which is not its system's id for the PlayStation (`PlayStation`, `psx`), and the
//! game page's Engine row and the library's engine for the console both name the chosen engine, not the first one.

use emusen_client_fixture::cores::{hosted, libretro_entry_as, sha256, RING_GAME};
use emusen_client_fixture::play::installation_with_cores;
use emusen_client_fixture::{library, World};
use emusen_crystal_client::{Api, Config, Runtime};
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt_models::clients::ClientSource;
use emusen_qt_models::consoles::{self, ENGINE_KEY};
use emusen_qt_models::consoles_source::ClientConsoles;
use emusen_qt_models::game_page::GamePageSource;
use emusen_qt_models::library::LibrarySource;

/// A scratch world with Diamond and the test core hosted as the PlayStation's, a game of the PlayStation, and the
/// console's second engine chosen as Settings chooses it: the world, the client source, the game and the engine chosen.
fn chosen_in_settings(name: &str) -> (World, ClientSource, std::path::PathBuf, String, Runtime) {
    let world = World::new(name);
    // SAFETY: set before any thread reads the environment; the proxy starts the runner it names.
    unsafe { std::env::set_var("EMUSEN_CORE_RUNNER", &hosted().runner) };
    let cores = installation_with_cores(&world.dir("lib"));
    let player = world.dir("Home/Cores");
    let entry = libretro_entry_as(&player, "playstation");
    let config = Config { cores: Some(vec![cores.to_str().unwrap().into()]), player_cores: Some(vec![player.to_str().unwrap().into()]), ..Config::default() };
    Runtime::new(&Api::load(library()).unwrap(), &config).unwrap().core_approve(&sha256(&entry)).unwrap();
    let game = world.dir("Roms/psx").join("Ring.iso");
    std::fs::write(&game, RING_GAME).unwrap();
    let platform = Platform::load(library()).unwrap();
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = serde_json::json!(world.root.join("Roms").to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();

    // Settings' PlayStation, by the console it lists, and its second engine chosen as its Engine page chooses one.
    let settings = ClientConsoles { runtime: Runtime::new(&Api::load(library()).unwrap(), &config).unwrap() };
    let read = consoles::read(&settings).unwrap();
    let console = read.consoles.iter().find(|c| c.systems.contains(&"psx".to_owned())).expect("the PlayStation listed");
    assert_ne!(console.console, "psx", "the console's name is not its system's id");
    assert_eq!(console.engines.len(), 2, "Diamond and the test core: {:?}", console.engines);
    let chosen = console.engines[1].clone();
    settings.set(&console.console, ENGINE_KEY, Some(&chosen)).unwrap();

    let source = ClientSource::open(library().to_str().unwrap(), &config).unwrap();
    (world, source, game, chosen, settings.runtime)
}

#[test]
fn a_console_s_engine_chosen_in_settings_is_the_one_the_library_names() {
    let (_world, source, _, chosen, _) = chosen_in_settings("models-console-engine-library");
    let systems = source.systems().unwrap();
    let engines = source.engines(&systems).unwrap();
    let shown = engines.iter().find(|(id, _)| id == "psx").map(|(_, e)| e.id.clone());
    assert_eq!(shown.as_deref(), Some(chosen.as_str()), "the library's engine for the PlayStation");
}

#[test]
fn a_console_s_engine_chosen_in_settings_is_the_game_page_s_default() {
    let (_world, source, game, chosen, runtime) = chosen_in_settings("models-console-engine-page");
    let file = source.games().unwrap().into_iter().find(|g| g.path == game.to_str().unwrap()).expect("the game scanned");
    let page = source.read(&file).unwrap();
    let default = page.engines.first().map(|c| c.text.clone()).unwrap_or_default();
    let named = emusen_qt_models::clients::engine_shown(&chosen, &runtime.cores().unwrap()).name;
    assert_eq!(default, format!("{named} (default)"), "the game page's Engine row; its choices {:?}", page.engines.iter().map(|c| &c.text).collect::<Vec<_>>());
}
