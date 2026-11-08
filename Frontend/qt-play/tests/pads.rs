//! Play.qml's pads in a real window (EmuSen_Qt.md §1.8.5): an SDL virtual pad, read by the window's one PadReader, plays
//! the test core while the window runs it; the router gives only the chord while the game owns the pad, and the chord
//! pauses and resumes the game. The test core counts in its battery each frame port 0's first button is held, so what the
//! game heard is read from the save the session leaves. Qt adopts Serenity's device in a compositor of the test's own, on
//! the RX 6800 (EMUSEN_GPU_DEVICE names another); the game's sound plays on SDL's dummy driver, which `Sdl::open` sets.

mod common;

use std::path::{Path, PathBuf};

use common::{approve, device_and_feed, play_window, until, vulkan_qt};
use emusen_client_fixture::{library, World};
use emusen_kit_core::input::PadControl;
use emusen_qt_endymion::reading::sdl_button;
use emusen_qt_play::window::ffi::QGuiApplication as App;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::virtual_pad::Sdl;
use emusen_qt_test::Harness;

/// The battery files under `dir`, by name.
fn saves(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(saves(&path));
        } else if path.extension().is_some_and(|e| e == "srm") {
            found.push(path);
        }
    }
    found
}

#[test]
fn a_virtual_pad_plays_the_game_in_the_window_and_its_chord_pauses_and_resumes_it() {
    let _kwin = Compositor::start(1280, 800);
    vulkan_qt();
    let world = World::new("play-pads");
    approve(&world.player_core(&world.dir("Home/Cores")), None, &world);
    let game = world.game("game.tst", &[1, 2, 3, 4]);
    let sdl = Sdl::open();
    let pad = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);

    let (device, feed) = device_and_feed(&world.dir("presets"));
    let mut harness = Harness::new();
    assert_eq!(App::platform_name().to_string(), "wayland", "Qt on the test's compositor");
    let mut scene = play_window(&mut harness, &device);
    assert!(scene.set_property("play", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("play", "platformLibrary", library().to_str().unwrap()));
    until(&mut scene, "the virtual pad read", |s| s.property("pads", "connected") == "1");
    assert!(!scene.pad_chord_only(), "the interface owns the pad before a game");
    assert!(scene.set_property("play", "feed", &feed.address().to_string()));
    assert!(scene.set_property("play", "game", game.to_str().unwrap()));
    until(&mut scene, "the game running", |s| {
        let problem = s.property("session", "problem");
        assert!(problem.is_empty(), "the session: {problem}");
        s.property("session", "running") == "true"
    });
    until(&mut scene, "the first picture", |s| s.property("session", "firstPictureMs").parse::<f64>().is_ok_and(|ms| ms > 0.0));
    assert!(scene.pad_chord_only(), "a running game owns the pad: only the chord leaves the router");
    assert_eq!(scene.property("session", "sounding"), "true", "the game's sound reaches a player (SDL's dummy driver)");

    // A, the test core's first button (East by Endymion's defaults), held about half a second.
    let east = sdl_button(PadControl::East).unwrap();
    sdl.press(&pad, east, true);
    scene.wait(500);
    sdl.press(&pad, east, false);

    // The chord (the guide button) reaches the program: the game pauses, the walk has the pad, then it resumes.
    let guide = sdl_button(PadControl::Guide).unwrap();
    sdl.press(&pad, guide, true);
    until(&mut scene, "the chord pausing the game", |s| s.property("session", "paused") == "true");
    sdl.press(&pad, guide, false);
    until(&mut scene, "the interface owning the pad while paused", |s| !s.pad_chord_only());
    sdl.press(&pad, guide, true);
    until(&mut scene, "the chord resuming the game", |s| s.property("session", "paused") == "false");
    sdl.press(&pad, guide, false);
    assert!(scene.pad_chord_only(), "the game owns the pad again");

    assert!(scene.set_property("play", "game", ""));
    until(&mut scene, "the game closed", |s| s.property("session", "running") == "false");
    let found = saves(&world.root);
    assert_eq!(found.len(), 1, "one battery file: {found:?}");
    let held = std::fs::read(&found[0]).unwrap()[0];
    assert!(held >= 10, "A held half a second was heard on port 0 for {held} frames");
    assert_eq!(scene.property("pads", "problem"), "", "the reader said no failure");
    assert_eq!(scene.property("session", "problem"), "", "the session said no failure");
    sdl.detach(pad);
    drop(harness);
    drop(feed);
    assert_eq!(device.findings().unwrap(), Vec::<String>::new());
}
