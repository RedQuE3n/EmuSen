//! A core that hands its pictures through the device the window lends it, in a real window (EmuSen_Qt.md §1.8.7):
//! emusen-native's device test core, installed as a player's core, paints the libretro test core's picture on the lent
//! device; every frame crosses the feed as the lend's picture, the picture the window shows is read back and matches
//! what the core made for the frame drawn, and the validation layer finds nothing. The chord pauses the game for the
//! read-back, so the frame drawn stays the frame shown.

mod common;

use common::{approve, assert_qt_grab_only, device_and_feed, painted_differences, play, play_window, until, vulkan_qt};
use emusen_client_fixture::cores::{device_core, PAINTED};
use emusen_client_fixture::World;
use emusen_kit_core::input::PadControl;
use emusen_qt_endymion::reading::sdl_button;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::virtual_pad::Sdl;
use emusen_qt_test::Harness;
use std::sync::atomic::Ordering;

#[test]
fn a_lent_device_s_pictures_are_drawn_in_the_window_as_the_core_made_them() {
    let _kwin = Compositor::start(1280, 800);
    vulkan_qt();
    let world = World::new("play-device");
    let core = world.player_library(&world.dir("Home/Cores"), device_core());
    approve(&core, None, &world);
    let game = world.game("painted.tst", &[PAINTED, 1, 2, 3]);
    let sdl = Sdl::open();
    let pad = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);

    let (device, feed) = device_and_feed(&world.dir("presets"));
    let mut harness = Harness::new();
    let mut scene = play_window(&mut harness, &device);
    assert!(scene.set_property("play", "onlyPads", &pad.id.to_string()));
    play(&mut scene, &feed, &game);
    until(&mut scene, "a second of pictures", |_| feed.counts().shown.load(Ordering::Relaxed) >= 60);
    let counts = feed.counts();
    let (shown, lent) = (counts.shown.load(Ordering::Relaxed), counts.shown_lent.load(Ordering::Relaxed));
    assert_eq!(lent, shown, "every picture drawn came from the lend");
    assert_eq!(counts.shown_ring.load(Ordering::Relaxed), 0, "none from a ring");

    let guide = sdl_button(PadControl::Guide).unwrap();
    sdl.press(&pad, guide, true);
    until(&mut scene, "the chord pausing the game", |s| s.property("session", "paused") == "true");
    sdl.press(&pad, guide, false);
    scene.wait(100);
    assert_eq!(device.findings().unwrap(), Vec::<String>::new(), "the layer found nothing while the lend's pictures were drawn");
    let (serial, differ) = painted_differences(&mut scene, &feed);
    assert!(serial >= 60, "the frame drawn last is the core's frame {serial}");
    assert!(differ.is_empty(), "the window shows frame {serial} otherwise than the core made it, (x, y, shown, made): {differ:?}");

    scene.wait(100);
    assert_qt_grab_only(&device.findings().unwrap());

    sdl.press(&pad, guide, true);
    until(&mut scene, "the chord resuming the game", |s| s.property("session", "paused") == "false");
    sdl.press(&pad, guide, false);
    until(&mut scene, "pictures again", |_| feed.counts().shown.load(Ordering::Relaxed) > shown + 30);
    assert!(scene.set_property("play", "game", ""));
    until(&mut scene, "the game closed", |s| s.property("session", "running") == "false");
    assert_eq!(feed.problem(), None, "the render thread met no failure");
    sdl.detach(pad);
    drop(harness);
    drop(feed);
    assert_eq!(device.findings().unwrap(), Vec::<String>::new());
}
