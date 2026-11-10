//! Rumble in the program on Vulkan (EmuSen_Qt.md §7.7.H3, EmuSen_Input.md §9.7): a game of the test core reporting a
//! console whose games drive a motor, its motor reaching the SDL virtual pad in this process, stopped while the Pause
//! Card holds the pad and by the Rumble switch turned mid-game; on the RX 6800 unless EMUSEN_GPU_DEVICE names another.

mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::{image, library};
use emusen_crystal_client::{Api, Config, Runtime};
use emusen_platform_client::Platform;
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;
use emusen_qt_play::host::Host;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::virtual_pad::{Sdl, VirtualPad};
use emusen_qt_test::{Harness, Scene};

/// The test core's image byte that makes it report a console whose games drive a motor (its `MOTOR_CONSOLE`).
const MOTOR_CONSOLE: u8 = 0x64;

fn until(scene: &mut Scene<'_>, what: &str, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    for _ in 0..100 {
        if done(scene) {
            return;
        }
        scene.wait(50);
    }
    panic!("{what} within 5 s");
}

/// SDL's Guide button.
const GUIDE: i32 = 5;

fn running(pad: &VirtualPad) -> bool {
    pad.rumble() != (0, 0)
}

#[test]
fn qt_7_7_h3_the_game_s_motor_runs_the_pad_and_stops_for_the_card_and_the_switch() {
    let kwin = Compositor::start(1920, 1200);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let mut harness = Harness::new();
    let platform = Platform::load(library()).unwrap();
    let gpu = std::env::var("EMUSEN_GPU_DEVICE").ok();
    let presets = std::env::temp_dir().join("qt-app-rumble-gpu-presets");
    let host = Host::new(&platform, "wayland", gpu.as_deref(), false, &presets).expect("a Vulkan device");
    let adoption = host.adoption();
    let world = sandbox("qt-app-rumble-gpu", &[]);
    let game = world.dir("Roms").join("Motor (Synthetic).tst");
    std::fs::write(&game, image(&[MOTOR_CONSOLE, 1, 2, 3])).unwrap();
    let player = world.dir("Home/Cores");
    let core = world.player_core(&player);
    let config = Config { player_cores: Some(vec![player.to_str().unwrap().into()]), ..Config::default() };
    Runtime::new(&Api::load(library()).unwrap(), &config).unwrap().core_approve(&emusen_client_fixture::cores::sha256(&core)).unwrap();
    let sdl = Sdl::open();
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
    until(&mut scene, "the game running", |s| s.property("session", "running") == "true");
    until(&mut scene, "the motor on the pad", |_| running(&pad));

    // The Pause Card takes the pad: the motor stops; back in the game, it runs again.
    // The router's clock on the wall clock, as a real pad needs it; Guide opens the card, as acceptance.rs's chord does.
    scene.run_pad_clock();
    sdl.press(&pad, GUIDE, true);
    until(&mut scene, "the card open", |s| s.property("session", "paused") == "true" && s.property("screens", "shown") == "Pause");
    sdl.press(&pad, GUIDE, false);
    until(&mut scene, "the motor stopped under the card", |_| !running(&pad));
    // B resumes the game once the card has slid into place.
    until(&mut scene, "the card in place", |s| s.property("cardCanvas", "slide") == "0");
    scene.pad(UiButton::Back);
    until(&mut scene, "the game again", |s| s.property("screens", "shown") == "Game");
    until(&mut scene, "the motor running again", |_| running(&pad));

    // Rumble turned off while the game runs, as Settings' row sets it: the motor stops; on, it runs again.
    assert!(scene.set_property("session", "rumble", "false"));
    until(&mut scene, "the motor stopped by the switch", |_| !running(&pad));
    scene.wait(300);
    assert!(!running(&pad), "it stays stopped");
    assert!(scene.set_property("session", "rumble", "true"));
    until(&mut scene, "the motor on again", |_| running(&pad));
    assert!(scene.set_property("game", "game", ""));
    until(&mut scene, "the game closed", |s| s.property("session", "running") == "false");
    // The window hidden and gone before the device it adopted, as acceptance.rs ends.
    assert!(scene.set_property("program", "visible", "false"));
    scene.wait(50);
    drop(harness);
    drop(host);
}
