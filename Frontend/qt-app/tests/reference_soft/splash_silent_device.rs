//! A theme's splash whose sound device never answers: SDL's PipeWire driver waits on a server that takes the connection
//! and says nothing, so the sound is never ready; the splash waits its own length for it, then plays silent and Home
//! follows (EmuSen_Qt.md §3.5.3). In a process of its own, since SDL's audio stays waiting for the life of the process.

// The modules the scene imports register through the program's crate, which a test must link.

#[allow(unused_imports)]
use crate::eclipse_board;
#[allow(unused_imports)]
use crate::eclipse_home_oracle;
#[allow(unused_imports)]
use crate::start_held;

use emusen_client_fixture::World;
use start_held::{eclipse, held, until, PLAIN};
use std::os::unix::net::UnixListener;

#[test]
fn qt_3_5_3_a_splash_whose_sound_is_never_ready_plays_silent_after_its_length() {
    std::env::set_var("LC_ALL", "en_US.UTF-8");
    std::env::set_var("TZ", "UTC");
    let world = World::new("qt-app-splash-silent-device");
    // A PipeWire server that takes every connection and never answers (libpipewire reads PIPEWIRE_REMOTE for its socket).
    let socket = world.root.join("silent-pipewire");
    let listener = UnixListener::bind(&socket).unwrap();
    std::thread::spawn(move || {
        let mut kept = Vec::new();
        for stream in listener.incoming() {
            kept.extend(stream.ok());
        }
    });
    std::env::set_var("PIPEWIRE_REMOTE", &socket);
    std::env::set_var("SDL_AUDIO_DRIVER", "pipewire");
    let mut harness = emusen_qt_test::Harness::new();
    let (_, media) = eclipse_home_oracle::synthetic(&world, &mut harness);
    let mut scene = held(&mut harness, (1920, 1200), &eclipse(), &PLAIN, &media);

    // Everything else read and the file decoded, the sound still not ready: the splash holds its first frame.
    until(&mut scene, "the start read and the sound's file decoded", 10_000, |s| {
        s.property("themeSplash", "reading") == "false" && s.property("splashSoundFile", "read") == "true"
    });
    scene.wait(500);
    assert_eq!(scene.property("splashSound", "settled"), "false", "the device has not answered");
    assert_eq!(scene.property("themeSplash", "set"), "false", "the splash waits for its sound");
    let length: i64 = scene.property("themeSplash", "length").parse::<f64>().unwrap() as i64;
    assert!(length > 0, "the splash's length read");

    // Until its own length has passed on the clock, the splash waits.
    scene.hold_clock(Some(length - 100));
    assert_eq!(scene.property("themeSplash", "set"), "false", "still waiting before its length");
    assert_eq!(scene.property("themeSplash", "clock"), "0", "at its first frame");

    // Then it plays, silent, and says why.
    scene.hold_clock(Some(length + 100));
    assert_eq!(scene.property("splashSound", "settled"), "true", "the sound given up");
    assert_eq!(scene.property("themeSplash", "set"), "true", "the splash runs");
    assert!(scene.property("splashSound", "problem").contains("not ready"), "{}", scene.property("splashSound", "problem"));
    assert_eq!(scene.property("splashSound", "starts"), "0", "nothing played");
    scene.hold_clock(Some(length + 1100));
    let clock: f64 = scene.property("themeSplash", "clock").parse().unwrap();
    assert!(clock > 0.0, "the clock runs: {clock}");

    // On to its end, and Home.
    scene.hold_clock(Some(3 * length));
    until(&mut scene, "the splash gone", 3000, |s| s.property("themeSplash", "done") == "true");
    assert_eq!(scene.property("program", "splashShown"), "false", "the splash has ended");
    assert!(scene.tree().nodes.iter().any(|n| n.name == "disc" && n.visible), "Eclipse's Home is shown");
    assert_eq!(scene.property("splashSound", "starts"), "0", "and no sound started late");
    scene.hold_clock(None);
    harness.assert_quiet();
}
