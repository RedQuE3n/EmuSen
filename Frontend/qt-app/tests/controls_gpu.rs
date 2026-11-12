//! The keyboard over a running game in the program (EmuSen_Qt.md §7.7.H6), on Vulkan in a compositor of the test's own:
//! the client fixture's console core, a PlayStation started from the List card, drawing the button of its pad held in its
//! picture's blue. A key of the console's keyboard map (Endymion's defaults: B on Z) held reaches it as its button; then
//! each hotkey at Endymion's default key does what Mistress's did: Pause / Resume, Save State and Load State, Fast
//! Forward and Rewind while held, Frame Advance, Screenshot, Fullscreen, and Exit to Library. A GPU job.

mod common;

use common::console::{blue, choose_row, console_owns_the_pad, list_card_start_console, picture_points, quit_game, until};
use common::{approved_console_core, sandbox, wait_for};

use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_play::host::Host;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::virtual_pad::Sdl;
use emusen_qt_test::{Harness, Scene};

/// Qt's numbers for the keys pressed: Z, P, Tab, Backspace, the full stop, F5, F8, F9, F11 and Escape.
const Z: i32 = 0x5a;
const P: i32 = 0x50;
const TAB: i32 = 0x0100_0001;
const BACKSPACE: i32 = 0x0100_0003;
const PERIOD: i32 = 0x2e;
const F5: i32 = 0x0100_0034;
const F8: i32 = 0x0100_0037;
const F9: i32 = 0x0100_0038;
const F11: i32 = 0x0100_003a;
const ESCAPE: i32 = 0x0100_0000;
/// QWindow::FullScreen.
const FULL_SCREEN: &str = "5";

/// A key held then let go, as a player taps it.
fn tap(scene: &mut Scene<'_>, key: i32) {
    scene.key_hold(key, true);
    scene.key_hold(key, false);
}

/// The notice over the game reading `words`, said for what was done (not for attention).
#[track_caller]
fn said(scene: &mut Scene<'_>, words: &str) {
    until(scene, 5, &format!("the notice \"{words}\""), |s| s.property("session", "notice") == words);
    assert_eq!(scene.property("session", "noticeAttention"), "false", "{words}");
}

#[test]
fn qt_7_7_h6_the_keyboard_plays_the_game_and_each_hotkey_acts() {
    unsafe { std::env::set_var("SDL_AUDIODRIVER", "dummy") };
    let kwin = Compositor::start(3840, 2160);
    kwin.watch(600);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let mut harness = Harness::new();
    let platform = Platform::load(library()).unwrap();
    let gpu = std::env::var("EMUSEN_GPU_DEVICE").ok();
    let host = Host::new(&platform, "wayland", gpu.as_deref(), false, &std::env::temp_dir().join("qt-app-controls-presets")).expect("a Vulkan device");
    let adoption = host.adoption();
    let world = sandbox("qt-app-controls", &[("nes", "Brass Lantern.nes")]);
    let cores = emusen_client_fixture::cores::installation_with_snes(&world.dir("lib"));
    approved_console_core(&cores);
    let sdl = Sdl::open();
    let pad = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);
    let mut scene = harness.load_prepared(MAIN_URL, 1280, 800, |window| unsafe { adoption.adopt(window.cast()) }.expect("Qt adopts the device"));
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("clock", "now", common::HELD_NOW));
    assert!(scene.set_property("program", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("program", "feed", &host.feed.address().to_string()));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 20000);
    until(&mut scene, 10, "the console listed", |s| s.property("library", "startable").contains("\"PlayStation\""));
    until(&mut scene, 20, "the virtual pad read", |s| s.property("pads", "connected") == "1");
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    list_card_start_console(&mut scene);
    scene.pad(UiButton::Accept);
    console_owns_the_pad(&mut scene, &sdl, &pad, "PlayStation");
    let points = picture_points(&mut scene);
    let rest = blue(&mut scene, &points);

    // Z, B's key: the console's B held while it is, and let go.
    scene.key_hold(Z, true);
    until(&mut scene, 5, &format!("Z reaching the console as B (blue {rest:.0} at rest)"), |s| blue(s, &points) > rest + 60.0);
    scene.key_hold(Z, false);
    until(&mut scene, 5, "Z let go", |s| blue(s, &points) < rest + 20.0);
    assert_eq!(scene.property("screens", "shown"), "Game", "Z is the game's");

    // P pauses with nothing over the game, and P again resumes it.
    tap(&mut scene, P);
    until(&mut scene, 5, "P pausing", |s| s.property("session", "paused") == "true");
    assert_eq!(scene.property("screens", "shown"), "Game", "paused in place, no card");
    tap(&mut scene, P);
    until(&mut scene, 5, "P resuming", |s| s.property("session", "paused") == "false");

    // F5 saves to the slot chosen, F8 loads it.
    tap(&mut scene, F5);
    said(&mut scene, "State saved to slot 1");
    tap(&mut scene, F8);
    said(&mut scene, "State loaded from slot 1");

    // Tab held: fast forward, said as it goes down.
    scene.key_hold(TAB, true);
    said(&mut scene, "Fast forward");
    scene.key_hold(TAB, false);

    // Backspace held: the game steps back in place of playing, so B held meanwhile is not drawn until it is let go.
    scene.wait(1500);
    scene.key_hold(BACKSPACE, true);
    scene.wait(200);
    scene.key_hold(Z, true);
    scene.wait(800);
    assert!(blue(&mut scene, &points) < rest + 20.0, "B held while rewinding is not played");
    assert_eq!(scene.property("session", "noticeAttention"), "false", "rewind steps: {}", scene.property("session", "notice"));
    scene.key_hold(BACKSPACE, false);
    until(&mut scene, 5, "playing again once Backspace is let go", |s| blue(s, &points) > rest + 60.0);
    scene.key_hold(Z, false);
    until(&mut scene, 5, "Z let go", |s| blue(s, &points) < rest + 20.0);

    // The full stop: one frame and the game held, B held after it not drawn; P lets it run.
    tap(&mut scene, PERIOD);
    said(&mut scene, "Paused: advanced one frame");
    scene.key_hold(Z, true);
    scene.wait(800);
    assert!(blue(&mut scene, &points) < rest + 20.0, "held after its one frame");
    tap(&mut scene, P);
    until(&mut scene, 5, "P letting the game run after Frame Advance", |s| blue(s, &points) > rest + 60.0);
    scene.key_hold(Z, false);

    // F9 takes a screenshot.
    tap(&mut scene, F9);
    said(&mut scene, "Screenshot saved");

    // Escape: the game suspended to the library, as Back to Library leaves it.
    tap(&mut scene, ESCAPE);
    until(&mut scene, 5, "Escape exiting to the library", |s| s.property("screens", "shown") == "Home" && s.property("session", "paused") == "true");
    assert_eq!(scene.property("session", "running"), "true", "suspended, not closed");

    // Start's Home card leads with Continue, which plays it again; then Quit Game to Home.
    scene.pad(UiButton::Menu);
    until(&mut scene, 5, "the Home card in place", |s| s.property("screens", "shown") == "HomeCard" && s.property("homeCardCanvas", "slide") == "0");
    choose_row(&mut scene, "Continue");
    console_owns_the_pad(&mut scene, &sdl, &pad, "PlayStation");

    // F11 turns the window's full screen on and off.
    let windowed = scene.property("program", "visibility");
    tap(&mut scene, F11);
    until(&mut scene, 5, &format!("F11 full screen (visibility {windowed} before)"), |s| s.property("program", "visibility") == FULL_SCREEN);
    tap(&mut scene, F11);
    until(&mut scene, 5, "F11 again windowed", |s| s.property("program", "visibility") == windowed);

    quit_game(&mut scene, &sdl, &pad, "PlayStation (no disc)", "Home");
    sdl.detach(pad);
    assert!(scene.set_property("program", "visible", "false"));
    scene.wait(50);
    drop(harness);
    drop(host);
}
