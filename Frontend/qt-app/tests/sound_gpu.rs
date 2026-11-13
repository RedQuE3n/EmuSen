//! A game's sound and window following the player's settings in the program on Vulkan (EmuSen_Qt.md §1.8.6): the test
//! core's sound player opened at the stored Volume and on the stored Output, both followed when Settings changes them
//! with the game suspended; Pause in background pausing the game as the window goes and resuming it as it comes back,
//! only where the setting is on; the frame rate over the game only where Frame rate is on. On the RX 6800 unless
//! EMUSEN_GPU_DEVICE names another; the sound on SDL's dummy driver.

mod bindings;
mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::{image, library};
use emusen_crystal_client::{Api, Config, Runtime};
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_play::host::Host;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::virtual_pad::Sdl;
use emusen_qt_test::Scene;

fn until(scene: &mut Scene<'_>, what: &str, done: impl FnMut(&mut Scene<'_>) -> bool) {
    common::console::until(scene, 5, what, done);
}

fn number(scene: &mut Scene<'_>, name: &str, property: &str) -> f64 {
    scene.property(name, property).parse().unwrap_or(f64::NAN)
}

/// The window hidden, so another has the front, then shown again; whether the game was paused while it was away.
fn away_and_back(scene: &mut Scene<'_>) -> bool {
    assert!(scene.set_property("program", "visible", "false"));
    until(scene, "the program in the background", |s| s.property("session", "background") == "true");
    scene.wait(200);
    let paused = scene.property("session", "paused") == "true";
    assert!(scene.set_property("program", "visible", "true"));
    until(scene, "the program in front again", |s| s.property("session", "background") == "false");
    paused
}

/// Settings' section by its index, through the right shoulder, and the row whose sentence begins with `words` stepped
/// once by `button` until its drawn value changes.
fn step_row(scene: &mut Scene<'_>, section: i32, words: &str, button: UiButton) {
    for _ in 0..8 {
        if scene.property("settingsModel", "section") == section.to_string() {
            break;
        }
        scene.pad(UiButton::PageDown);
        scene.wait(60);
    }
    assert_eq!(scene.property("settingsModel", "section"), section.to_string());
    bindings::focus_row(scene, words);
    let before = bindings::focused_row_words(scene);
    scene.pad(button);
    bindings::until(scene, &format!("the row '{words}' stepped from {before:?}"), |s| bindings::focused_row_words(s) != before);
}

#[test]
fn qt_1_8_6_the_game_s_sound_and_window_follow_the_settings() {
    // Set before SDL is first loaded, while this binary's one test is its only thread that reads the environment.
    unsafe { std::env::set_var("SDL_AUDIODRIVER", "dummy") };
    let kwin = Compositor::start(1920, 1200);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let mut harness = emusen_qt_test::Harness::new();
    let platform = Platform::load(library()).unwrap();
    let gpu = std::env::var("EMUSEN_GPU_DEVICE").ok();
    let presets = std::env::temp_dir().join("qt-app-sound-gpu-presets");
    let host = Host::new(&platform, "wayland", gpu.as_deref(), false, &presets).expect("a Vulkan device");
    let adoption = host.adoption();
    let world = sandbox("qt-app-sound-gpu", &[]);
    let game = world.dir("Roms").join("Sound (Synthetic).tst");
    std::fs::write(&game, image(&[1, 2, 3, 4])).unwrap();
    let player = world.dir("Home/Cores");
    let core = world.player_core(&player);
    let config = Config { player_cores: Some(vec![player.to_str().unwrap().into()]), ..Config::default() };
    Runtime::new(&Api::load(library()).unwrap(), &config).unwrap().core_approve(&emusen_client_fixture::cores::sha256(&core)).unwrap();

    // The settings stored before the program starts: a volume of its own, the first device SDL lists, the frame rate off.
    let sdl = Sdl::open();
    platform.lend_sdl(&emusen_platform_client::Lent::open("libSDL3.so.0").unwrap()).unwrap();
    let devices = platform.audio_devices().unwrap();
    let device = devices.first().expect("SDL's dummy driver lists a device").clone();
    let mut stored = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    stored["Volume"] = serde_json::json!(0.35);
    stored["SoundOutputDevice"] = serde_json::json!(device);
    stored["ShowFpsBar"] = serde_json::json!(false);
    stored["ShowStatusText"] = serde_json::json!(true);
    stored["PauseInBackground"] = serde_json::json!(true);
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &stored).unwrap();

    let pad = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);
    let mut scene = harness.load_prepared(MAIN_URL, 1280, 800, |window| unsafe { adoption.adopt(window.cast()) }.expect("Qt adopts the device"));
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("program", "feed", &host.feed.address().to_string()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 20000);
    wait_for(&mut scene, "settingsModel", "ready", "true", 5000);
    until(&mut scene, "the virtual pad read", |s| s.property("pads", "connected") == "1");

    assert!(scene.set_property("game", "game", game.to_str().unwrap()));
    wait_for(&mut scene, "screens", "shown", "Game", 2000);
    until(&mut scene, "the game running", |s| s.property("session", "running") == "true" && s.property("session", "sounding") == "true");
    // The player opened at the stored volume, on the stored device.
    assert!((number(&mut scene, "session", "soundVolume") - 0.35).abs() < 1e-6, "{}", scene.property("session", "soundVolume"));
    assert_eq!(scene.property("session", "soundDevice"), device);
    // The messages over the game, the frame rate not.
    until(&mut scene, "the status line drawn", |s| s.property("statusLine", "visible") == "true");
    assert_eq!(scene.property("statusLine", "text"), "Sound");
    assert_eq!(scene.property("fpsLine", "visible"), "false");

    // The window away: the game pauses, and runs again as it comes back.
    assert!(away_and_back(&mut scene), "Pause in background on: the game paused while the window was away");
    until(&mut scene, "the game resumed", |s| s.property("session", "paused") == "false");

    // Settings with the game suspended under Home: the window away and back leaves the suspended game paused; Volume a
    // step down, Output to the system default, Frame rate on, Pause in background off.
    bindings::in_settings_with_game_suspended(&mut scene, |scene| {
        away_and_back(scene);
        assert_eq!(scene.property("session", "paused"), "true", "a game paused before stays paused");
        step_row(scene, 2, "The game's volume", UiButton::Left);
        until(scene, "the suspended game's player at the new volume", |s| (number(s, "session", "soundVolume") - 0.3).abs() < 1e-6);
        step_row(scene, 2, "The game and the interface", UiButton::Left);
        until(scene, "the suspended game's player on the system default", |s| s.property("session", "soundDevice").is_empty());
        step_row(scene, 1, "The frames-per-second", UiButton::Right);
        step_row(scene, 5, "A game paused because another window", UiButton::Right);
    });
    assert!((number(&mut scene, "session", "volume") - 0.3).abs() < 1e-6);
    until(&mut scene, "the frame rate over the game", |s| s.property("fpsLine", "visible") == "true" && number(s, "session", "fps") > 0.0);
    assert!(scene.property("fpsLine", "text").ends_with(" FPS"), "{}", scene.property("fpsLine", "text"));

    // Pause in background off: the window away and back leaves the game running.
    assert!(!away_and_back(&mut scene), "Pause in background off: the game ran on while the window was away");
    assert_eq!(scene.property("session", "paused"), "false");

    assert!(scene.set_property("game", "game", ""));
    until(&mut scene, "the game closed", |s| s.property("session", "running") == "false");
    assert_eq!(scene.property("fpsLine", "visible"), "false");
    // The window hidden and gone before the device it adopted, as acceptance.rs ends.
    assert!(scene.set_property("program", "visible", "false"));
    scene.wait(50);
    drop(harness);
    drop(host);
}
