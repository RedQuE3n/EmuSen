//! The keyboard over a running game in the program (EmuSen_Qt.md §7.7.H6), on Vulkan in a compositor of the test's own:
//! the client fixture's console core, a PlayStation started from the List card, drawing the button of its pad held in its
//! picture's blue. A key of the console's keyboard map (Endymion's defaults: B on Z) held reaches it as its button; then
//! each hotkey at Endymion's default key does what Mistress's did: Pause / Resume, Save State and Load State, Fast
//! Forward and Rewind while held, Frame Advance, Screenshot, Fullscreen, and Exit to Library, each by its effect (a
//! state's file and its frame read back, frames counted, a picture's file); then over the library with the game
//! suspended, and a hotkey rebound on Settings pressed in the game and over Home (EmuSen_Qt.md §7.7.H8). A GPU job.

mod bindings;
mod common;

use common::console::{blue, choose_row, green, console_owns_the_pad, list_card_start_console, picture_points, quit_game, until};
use common::{approved_console_core, sandbox, wait_for};

use emusen_client_fixture::{image, library};
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
const F6: i32 = 0x0100_0035;
/// The test core's image byte that gives its port 1 a controller (its `TWO_PADS`).
const TWO_PADS: u8 = 0x22;
/// SDL's left stick across and South button.
const LEFT_X: i32 = 0;
const SOUTH: i32 = 0;
/// QWindow::FullScreen, as the harness names a property's enumeration value.
const FULL_SCREEN: &str = "FullScreen";

/// A key held then let go, as a player taps it.
fn tap(scene: &mut Scene<'_>, key: i32) {
    scene.key_hold(key, true);
    scene.key_hold(key, false);
}

/// The game's frame as its session service reads it.
fn frame(scene: &mut Scene<'_>) -> u64 {
    scene.property("session", "frame").parse().expect("the session's frame")
}

/// The frames the game runs in `ms`.
fn frames_over(scene: &mut Scene<'_>, ms: i32) -> u64 {
    let start = frame(scene);
    scene.wait(ms);
    frame(scene).saturating_sub(start)
}

/// The files in one of Galaxia's folders, and in its folders below, counted.
fn files_in(platform: &Platform, folder: u32) -> usize {
    fn count(dir: &std::path::Path) -> usize {
        std::fs::read_dir(dir).into_iter().flatten().flatten().map(|e| if e.path().is_dir() { count(&e.path()) } else { 1 }).sum()
    }
    platform.directory(folder).ok().flatten().map_or(0, |d| count(&d))
}

fn state_files(platform: &Platform) -> usize {
    files_in(platform, emusen_platform_client::galaxia::consts::DIR_SAVE_STATES)
}

fn screenshot_files(platform: &Platform) -> usize {
    files_in(platform, emusen_platform_client::galaxia::consts::DIR_SCREENSHOTS)
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

    // F5 saves to the slot chosen, its file written; F8 a second later loads it: the game's frame goes back to the save's.
    let states = state_files(&platform);
    let saved_at = frame(&mut scene);
    tap(&mut scene, F5);
    said(&mut scene, "State saved to slot 1");
    assert!(state_files(&platform) > states, "the slot's file written");
    scene.wait(1000);
    let before_load = frame(&mut scene);
    tap(&mut scene, F8);
    said(&mut scene, "State loaded from slot 1");
    let loaded = frame(&mut scene);
    assert!(before_load > saved_at + 30 && loaded < saved_at + 30, "the state read back: frame {saved_at} saved, {before_load} before the load, {loaded} after");

    // Tab held: fast forward, said as it goes down, the frames run faster than at the game's own speed.
    let normal = frames_over(&mut scene, 1000);
    scene.key_hold(TAB, true);
    said(&mut scene, "Fast forward");
    let fast = frames_over(&mut scene, 1000);
    scene.key_hold(TAB, false);
    assert!(fast as f64 > normal as f64 * 1.5, "fast forward: {fast} frames in a second against {normal}");

    // Backspace held: the game steps back in place of playing, its frame going back, so B held meanwhile is not drawn
    // until it is let go.
    scene.wait(1500);
    scene.key_hold(BACKSPACE, true);
    scene.wait(200);
    let rewinding = frame(&mut scene);
    scene.key_hold(Z, true);
    scene.wait(800);
    assert!(blue(&mut scene, &points) < rest + 20.0, "B held while rewinding is not played");
    assert!(frame(&mut scene) < rewinding, "the game's frame goes back while Backspace is held");
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

    // F9 takes a screenshot: a picture file in Galaxia's screenshots folder.
    let pictures = screenshot_files(&platform);
    tap(&mut scene, F9);
    said(&mut scene, "Screenshot saved");
    until(&mut scene, 5, "the screenshot's file", |_| screenshot_files(&platform) > pictures);

    // Escape: the game suspended to the library, as Back to Library leaves it.
    tap(&mut scene, ESCAPE);
    until(&mut scene, 5, "Escape exiting to the library", |s| s.property("screens", "shown") == "Home" && s.property("session", "paused") == "true");
    assert_eq!(scene.property("session", "running"), "true", "suspended, not closed");

    // Over the library with the game suspended, as Mistress's acted with the library up (EmuSen_Qt.md §7.7.H8): F5 saves
    // the suspended game; P and the full stop do nothing; Escape is the library's own key.
    tap(&mut scene, F5);
    said(&mut scene, "State saved to slot 1");
    tap(&mut scene, P);
    tap(&mut scene, PERIOD);
    tap(&mut scene, ESCAPE);
    scene.wait(500);
    assert_eq!((scene.property("screens", "shown"), scene.property("session", "paused")), ("Home".into(), "true".into()), "Pause, Frame Advance and Escape leave the game suspended");

    // Start's Home card leads with Continue, which plays it again; then Quit Game to Home.
    scene.pad(UiButton::Menu);
    until(&mut scene, 5, "the Home card in place", |s| s.property("screens", "shown") == "HomeCard" && s.property("homeCardCanvas", "slide") == "0");
    choose_row(&mut scene, "Continue");
    console_owns_the_pad(&mut scene, &sdl, &pad, "PlayStation");

    // F11 turns the window's full screen on and off: what the program asks of the window, then what the compositor
    // gives it, each held on its own.
    let windowed = scene.property("program", "visibility");
    tap(&mut scene, F11);
    until(&mut scene, 5, "F11 asking for full screen", |s| s.property("program", "fullScreenAsked") == "true");
    until(&mut scene, 5, &format!("the compositor giving full screen (visibility {windowed} before)"), |s| s.property("program", "visibility") == FULL_SCREEN);
    tap(&mut scene, F11);
    until(&mut scene, 5, "F11 again asking for a window", |s| s.property("program", "fullScreenAsked") == "false");
    until(&mut scene, 5, "F11 again windowed", |s| s.property("program", "visibility") == windowed);

    quit_game(&mut scene, &sdl, &pad, "PlayStation (no disc)", "Home");

    // A game of the console core whose port 1 holds a controller too: Left stick as the D-pad and Mirror player 1 to
    // player 2, each turned on its Settings row with the game suspended, seen at the core (port 0 in the picture's
    // blue, port 1 in its green).
    let game = world.dir("Roms").join("Two Pads (Synthetic).tst");
    std::fs::write(&game, image(&[TWO_PADS, 1, 2, 3])).unwrap();
    assert!(scene.set_property("game", "game", game.to_str().unwrap()));
    until(&mut scene, 20, "the two-pad game running", |s| s.property("session", "running") == "true" && s.property("screens", "shown") == "Game");
    until(&mut scene, 20, "its first picture", |s| s.property("session", "firstPictureMs").parse::<f64>().is_ok_and(|ms| ms > 0.0));
    let points = picture_points(&mut scene);
    common::console::game_hears_the_pads(&mut scene);
    let (rest, rest_green) = (blue(&mut scene, &points), green(&mut scene, &points));
    scene.run_pad_clock();
    // On by default: the stick pushed left is the d-pad's Left.
    sdl.axis(&pad, LEFT_X, -32767);
    until(&mut scene, 5, &format!("the stick reaching the console as Left (blue {rest:.0} at rest)"), |s| blue(s, &points) > rest + 60.0);
    sdl.axis(&pad, LEFT_X, 0);
    until(&mut scene, 5, "the stick let go", |s| blue(s, &points) < rest + 20.0);
    bindings::turn_row_with_game_suspended(&mut scene, "In games whose console has no stick");
    scene.run_pad_clock();
    sdl.axis(&pad, LEFT_X, -32767);
    scene.wait(800);
    assert!(blue(&mut scene, &points) < rest + 20.0, "the stick is no d-pad once its row is Off");
    sdl.axis(&pad, LEFT_X, 0);
    // Off by default: B reaches port 0 alone; mirrored from its row, port 1 too.
    sdl.press(&pad, SOUTH, true);
    until(&mut scene, 5, "B reaching port 0", |s| blue(s, &points) > rest + 60.0);
    assert!(green(&mut scene, &points) < rest_green + 20.0, "port 1 hears nothing of player 1 unmirrored");
    sdl.press(&pad, SOUTH, false);
    until(&mut scene, 5, "B let go", |s| blue(s, &points) < rest + 20.0);
    bindings::turn_row_with_game_suspended(&mut scene, "Player 1's controller also plays as player 2");
    scene.run_pad_clock();
    sdl.press(&pad, SOUTH, true);
    let start = std::time::Instant::now();
    let mut seen = (0.0, 0.0);
    while start.elapsed().as_secs() < 5 && seen.1 <= rest_green + 60.0 {
        seen = (blue(&mut scene, &points), green(&mut scene, &points));
        scene.wait(8);
    }
    assert!(seen.1 > rest_green + 60.0, "B reaching port 1 mirrored: blue {:.0}, green {:.0} (at rest {rest:.0}, {rest_green:.0})", seen.0, seen.1);
    sdl.press(&pad, SOUTH, false);
    until(&mut scene, 5, "B let go on both ports", |s| green(s, &points) < rest_green + 20.0 && blue(s, &points) < rest + 20.0);
    // Last, since Escape suspends no game after it: Exit to Library rebound to F6 on Settings' Hotkeys page with the game suspended: F6 in the game suspends it, and
    // F6 over Home plays it again, as Mistress's ToggleLibrary.
    bindings::in_settings_with_game_suspended(&mut scene, |scene| {
        // Settings opens at the row last turned, below Hotkeys: up to it.
        until(scene, 10, "the Hotkeys row", |s| {
            if s.property("settingsModel", "sentence").starts_with("The emulator's own keys") {
                return true;
            }
            s.pad(UiButton::Up);
            false
        });
        scene.pad(UiButton::Accept);
        until(scene, 5, "the Hotkeys page", |s| s.property("settingsModel", "sentence").starts_with("Fires once") || s.property("settingsModel", "sentence").starts_with("Applies while"));
        until(scene, 5, "Exit to Library's row", |s| {
            if bindings::focused_row_words(s).iter().any(|w| w == "Exit to Library") {
                return true;
            }
            s.pad(UiButton::Down);
            false
        });
        scene.pad(UiButton::Accept);
        until(scene, 5, "Exit to Library listening", |s| s.property("settingsModel", "listening") == "true");
        tap(scene, F6);
        until(scene, 5, "Exit to Library on F6", |s| bindings::focused_row_words(s).iter().any(|w| w == "F6"));
        scene.pad(UiButton::Back);
    });
    tap(&mut scene, F6);
    until(&mut scene, 5, "F6 exiting to the library", |s| s.property("screens", "shown") == "Home" && s.property("session", "paused") == "true");
    tap(&mut scene, F6);
    until(&mut scene, 5, "F6 over Home playing the game again", |s| s.property("screens", "shown") == "Game" && s.property("session", "paused") == "false");

    assert!(scene.set_property("game", "game", ""));
    until(&mut scene, 10, "the two-pad game closed", |s| s.property("session", "running") == "false");
    sdl.detach(pad);
    assert!(scene.set_property("program", "visible", "false"));
    scene.wait(50);
    drop(harness);
    drop(host);
}
