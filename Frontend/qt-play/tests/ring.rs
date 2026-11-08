//! A hosted core that draws in a ring the window lends it, in a real window (EmuSen_Qt.md §1.8.7): the libretro test
//! core, installed as a libretro entry under the player's cores with an installation whose manifest vouches for the proxy
//! and the libretro host, draws with Vulkan (its setting `tc_hw`); every frame crosses the feed as a slot of the ring,
//! the picture the window shows is read back and matches what the core made for the frame drawn, and the validation
//! layer finds nothing. The chord pauses the game for the read-back.

mod common;

use common::{approve, assert_qt_grab_only, device_and_feed, painted_differences, play, play_window, runtime_config, until, vulkan_qt, within};
use emusen_client_fixture::cores::{hosted, installation, libretro_entry, RING_GAME};
use emusen_client_fixture::{library, World};
use emusen_crystal_client::{Api, Open, Runtime, Scope};
use emusen_kit_core::input::PadControl;
use emusen_qt_endymion::reading::sdl_button;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::virtual_pad::Sdl;
use emusen_qt_test::Harness;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

#[test]
fn a_lent_ring_s_pictures_are_drawn_in_the_window_as_the_core_made_them() {
    let kwin = Compositor::start(1280, 800);
    vulkan_qt();
    let world = World::new("play-ring");
    // SAFETY: set before any thread reads the environment; the proxy starts the runner it names.
    unsafe { std::env::set_var("EMUSEN_CORE_RUNNER", &hosted().runner) };
    let cores = installation(&world.dir("lib"));
    let core = libretro_entry(&world.dir("Home/Cores"));
    approve(&core, Some(&cores), &world);
    let game = world.root.join("game.tcg");
    std::fs::write(&game, RING_GAME).unwrap();
    {
        // The core draws with Vulkan, as a player sets it in the console's settings.
        let rt = Runtime::new(&Api::load(library()).unwrap(), &runtime_config(Some(&cores), &world)).unwrap();
        let s = rt.open(&Open { path: Some(game.to_str().unwrap().into()), ..Open::default() }).unwrap();
        rt.settings_set(s, Scope::Console, None, "tc_hw", Some("vulkan")).unwrap();
        rt.close(s).unwrap();
    }
    let sdl = Sdl::open();
    let pad = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);

    let (device, feed) = device_and_feed(&world.dir("presets"));
    let mut harness = Harness::new();
    let mut scene = play_window(&mut harness, &device);
    assert!(scene.set_property("play", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("play", "cores", cores.to_str().unwrap()));
    play(&mut scene, &feed, &game);
    until(&mut scene, "a second of pictures", |_| feed.counts().shown.load(Ordering::Relaxed) >= 60);
    let counts = feed.counts();
    let (shown, ring) = (counts.shown.load(Ordering::Relaxed), counts.shown_ring.load(Ordering::Relaxed));
    assert_eq!(ring, shown, "every picture drawn came from the ring");
    assert_eq!(counts.shown_lent.load(Ordering::Relaxed), 0, "none from a lend");

    let guide = sdl_button(PadControl::Guide).unwrap();
    sdl.press(&pad, guide, true);
    until(&mut scene, "the chord pausing the game", |s| s.property("session", "paused") == "true");
    sdl.press(&pad, guide, false);
    scene.wait(100);
    assert_eq!(device.findings().unwrap(), Vec::<String>::new(), "the layer found nothing while the ring's pictures were drawn");
    let (serial, differ) = painted_differences(&mut scene, &feed);
    assert!(serial >= 60, "the frame drawn last is the core's frame {serial}");
    assert!(differ.is_empty(), "the window shows frame {serial} otherwise than the core made it, (x, y, shown, made): {differ:?}");
    scene.wait(100);
    assert_qt_grab_only(&device.findings().unwrap());

    sdl.press(&pad, guide, true);
    until(&mut scene, "the chord resuming the game", |s| s.property("session", "paused") == "false");
    sdl.press(&pad, guide, false);
    until(&mut scene, "pictures again", |_| feed.counts().shown.load(Ordering::Relaxed) > shown + 30);

    // A window that does not draw holds a game drawing on the device still, and the game closes at once: about 1.3 s, a
    // hosted core's runner stopping, against about 10 s when its frame waits on a ring slot nobody gives back (§1.8.7).
    assert!(scene.set_property("play", "visible", "false"));
    scene.wait(300);
    let handed = feed.counts().handed.load(Ordering::Relaxed);
    scene.wait(500);
    assert_eq!(feed.counts().handed.load(Ordering::Relaxed), handed, "a hidden window holds the game still");
    let closing = Instant::now();
    within(20, kwin.pid(), "closing the game behind a hidden window", || assert!(scene.set_property("play", "game", "")));
    until(&mut scene, "the game closed", |s| s.property("session", "running") == "false");
    let closed = closing.elapsed();
    assert!(closed < Duration::from_secs(5), "the game closed behind a hidden window in {closed:?}, its frame waiting on a ring slot");
    assert_eq!(feed.problem(), None, "the render thread met no failure");
    sdl.detach(pad);
    drop(harness);
    drop(feed);
    assert_eq!(device.findings().unwrap(), Vec::<String>::new());
}
