//! The start of the program under a chosen theme, on the software backend: a theme's splash that enters Home runs at the
//! clock's rate, a key and a tap skip it, navigation sounds off leave it silent, and a chosen theme that fails shows
//! Moonrise and reaches Home (EmuSen_Qt.md §3.5.3).

// The modules the scene imports register through the program's crate, which a test must link.

#[allow(unused_imports)]
use crate::eclipse_board;
#[allow(unused_imports)]
use crate::eclipse_home_oracle;
#[allow(unused_imports)]
use crate::start_held;

use emusen_client_fixture::World;
use emusen_kit_core::input::Key;
use emusen_kit_qt::qt_keys::qt_key;
use emusen_qt_test::Scene;
use start_held::{held, started, until, Start, PLAIN};

const SIZE: (i32, i32) = (1920, 1200);
/// SHA-256 of splash-sound-chosen.wav's samples (R-20261111-118) as the player takes them, each 16-bit sample over 32768 as a
/// little-endian 32-bit float, interleaved stereo at 48 kHz: the theme's FLAC must decode to these and no others (EmuSen_Qt.md §3.5.3).
const CHOSEN_SOUND: &str = "bca08baf85ae348be6a3c1cb75e85e42582e6b79b3d1425be4432d93b5416c45";

fn length(scene: &mut Scene<'_>) -> f64 {
    scene.property("themeSplash", "length").parse().unwrap()
}

/// The splash's time once its hand-over has ended, where a skip takes it (R-20261123-24).
fn handed(scene: &mut Scene<'_>) -> f64 {
    length(scene) + number(scene, "themeSplash", "ease")
}

fn number(scene: &mut Scene<'_>, object: &str, property: &str) -> f64 {
    scene.property(object, property).parse().unwrap()
}

#[test]
fn qt_3_5_3_a_splash_that_enters_home_starts_as_its_settings_and_its_theme_say() {
    std::env::set_var("LC_ALL", "en_US.UTF-8");
    std::env::set_var("TZ", "UTC");
    let world = World::new("qt-app-splash-start");
    // The sound into a file, never the desk's speakers: SDL's disk driver (SDL 3, SDL_AUDIO_DISK_OUTPUT_FILE).
    std::env::set_var("SDL_AUDIO_DRIVER", "disk");
    std::env::set_var("SDL_AUDIO_DISK_OUTPUT_FILE", world.root.join("splash-heard.raw"));
    let mut harness = emusen_qt_test::Harness::new();
    let (_, media) = eclipse_home_oracle::synthetic(&world, &mut harness);

    // A chosen theme whose package cannot be read: Moonrise is the splash, and Home (the reference's) follows it.
    {
        let broken = world.root.join("Broken theme");
        std::fs::create_dir_all(&broken).unwrap();
        std::fs::write(broken.join("theme.json"), "{ this is no theme").unwrap();
        let mut scene = held(&mut harness, SIZE, broken.to_str().unwrap(), &PLAIN, &media);
        until(&mut scene, "the chosen theme read and failed", 5000, |s| s.property("theme", "failed") == "true");
        let mut at = 0;
        let mut seen = false;
        while scene.property("program", "splashShown") == "true" && at < 60_000 {
            seen |= scene.property("moonrise", "visible") == "true";
            at += 100;
            scene.hold_clock(Some(at));
        }
        assert!(seen, "Moonrise is seen when the chosen theme fails");
        assert_eq!(scene.property("program", "splashShown"), "false", "the splash ends");
        assert_eq!(scene.property("themeSplash", "visible"), "false", "no theme's splash leads");
        assert_eq!(scene.property("home", "visible"), "true", "Home, the reference's, follows");
        scene.hold_clock(None);
    }

    // The clock runs at the animation clock's rate from the moment it is set: a held second is a second of the splash.
    {
        let mut scene = started(&mut harness, SIZE, &PLAIN, &media);
        assert_eq!(scene.property("themeSplash", "set"), "true", "the start read and the sound ready");
        assert_eq!(scene.property("splashSoundFile", "digest"), CHOSEN_SOUND, "the theme's sound decodes to the chosen sound's samples, every one");
        scene.hold_clock(Some(1000));
        let from = number(&mut scene, "themeSplash", "clock");
        scene.hold_clock(Some(4000));
        let to = number(&mut scene, "themeSplash", "clock");
        assert!((to - from - 3000.0).abs() < 0.5, "three held seconds moved the splash's clock from {from} to {to}");
        let end = length(&mut scene) as i64;
        scene.hold_clock(Some(end + 1000));
        until(&mut scene, "the splash ends on its own", 3000, |s| s.property("themeSplash", "done") == "true");
        assert_eq!(scene.property("splashSound", "ends"), "0", "an unskipped splash's sound rings on into Home");
        scene.hold_clock(None);
    }

    // A key the kit's map makes a press skips to the last frame, the sound let go, and Home follows.
    {
        let mut scene = started(&mut harness, SIZE, &PLAIN, &media);
        scene.hold_clock(Some(1000));
        assert_eq!(scene.property("splashSound", "starts"), "1", "the sound plays");
        scene.key(qt_key(Key::Return).unwrap(), 0);
        assert_eq!(number(&mut scene, "themeSplash", "time"), handed(&mut scene), "a key skipped past the hand-over");
        assert_eq!(scene.property("splashSound", "ends"), "1", "the sound let go");
        // The key waits in the router for the splash's pump, a QML Timer, "synchronized with the animation timer" the test holds (Qt 6, "Timer").
        scene.hold_clock(Some(1100));
        until(&mut scene, "the splash gone after the key", 3000, |s| s.property("themeSplash", "done") == "true");
        scene.hold_clock(None);
    }

    // A tap skips as a press does.
    {
        let mut scene = started(&mut harness, SIZE, &PLAIN, &media);
        scene.hold_clock(Some(1000));
        scene.click(f64::from(SIZE.0) / 2.0, f64::from(SIZE.1) / 2.0);
        assert_eq!(number(&mut scene, "themeSplash", "time"), handed(&mut scene), "a tap skipped past the hand-over");
        assert_eq!(scene.property("splashSound", "ends"), "1", "the sound let go");
        until(&mut scene, "the splash gone after the tap", 3000, |s| s.property("themeSplash", "done") == "true");
        scene.hold_clock(None);
    }

    // Navigation sounds off: the splash plays, and no sound with it.
    {
        let mut scene = started(&mut harness, SIZE, &Start { sounds: false, ..PLAIN }, &media);
        assert_eq!(scene.property("themeSplash", "set"), "true", "the splash runs without its sound");
        scene.hold_clock(Some(1000));
        assert!(number(&mut scene, "themeSplash", "clock") > 0.0, "the clock runs");
        assert_eq!(scene.property("splashSound", "starts"), "0", "no sound with navigation sounds off");
        assert_eq!(scene.property("splashSound", "frames"), "0", "no player made for it");
        scene.hold_clock(None);
    }
    harness.assert_quiet();
}
