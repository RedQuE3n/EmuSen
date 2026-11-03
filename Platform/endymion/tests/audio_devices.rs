//! The playback devices and the players' choice of one, the game's and the interface's, against SDL's dummy driver
//! (EmuSen_Audio_Sync.md §7.4, §7.5).

use emusen_endymion::audio::{self, AudioPlayer, UiSounds};
use emusen_endymion::rate_control::DynamicRateControl;
use emusen_endymion::sdl::{self, Sdl};
use std::sync::{Arc, Mutex};

/// SDL's audio subsystem is the process's; the tests take turns with it.
static SDL_TURN: Mutex<()> = Mutex::new(());

/// SDL3 as the system or `EMUSEN_SDL3` has it, on the dummy driver; none where no SDL3 loads here.
fn dummy_sdl() -> Option<Arc<Sdl>> {
    let named = std::env::var("EMUSEN_SDL3").ok();
    let found = named.iter().map(String::as_str).chain(["libSDL3.so.0", "libSDL3.so", "libSDL3.dylib", "SDL3.dll"]).find_map(|p| Sdl::open(p).ok());
    let Some(sdl) = found else {
        eprintln!("[SKIP] no SDL3 loads here; set EMUSEN_SDL3 to its file");
        return None;
    };
    assert!(sdl.set_hint(sdl::HINT_AUDIO_DRIVER, "dummy"));
    Some(Arc::new(sdl))
}

#[test]
fn the_dummy_driver_lists_its_device_and_a_player_plays_on_it_or_falls_back_to_the_default() {
    let _turn = SDL_TURN.lock().unwrap_or_else(|p| p.into_inner());
    let Some(sdl) = dummy_sdl() else { return };
    let devices = audio::playback_devices(&sdl);
    assert!(!devices.is_empty(), "the dummy driver has a playback device");
    assert!(devices.iter().all(|(id, name)| *id != 0 && !name.is_empty()));
    let named = devices[0].1.clone();

    let mut rate = DynamicRateControl::new(8192);
    let mut player = AudioPlayer::new(sdl.clone(), &mut rate, 32000, 256, 4096);
    assert!(player.is_available());
    assert_eq!(player.device(), None);

    // A listed name is played on, and the device reopens at the rate it had.
    assert!(player.set_device(&mut rate, Some(&named)));
    assert_eq!(player.device(), Some(named.as_str()));
    assert!(player.is_available());
    assert_eq!(player.sample_rate(), 32000);

    // A name no device has falls back to the system default, which still plays.
    assert!(!player.set_device(&mut rate, Some("No such sound output")));
    assert_eq!(player.device(), None);
    assert!(player.is_available());
    player.submit(&mut rate, Some(&[0i16; 64]), 44100).unwrap();
    assert_eq!(player.sample_rate(), 44100);

    // None and an empty name are both the system default.
    assert!(player.set_device(&mut rate, Some(&named)));
    assert!(player.set_device(&mut rate, Some("")));
    assert_eq!(player.device(), None);
    assert!(player.set_device(&mut rate, None));
    player.dispose();
}

#[test]
fn the_interface_sounds_follow_the_chosen_device_and_fall_back_to_the_default() {
    let _turn = SDL_TURN.lock().unwrap_or_else(|p| p.into_inner());
    let Some(sdl) = dummy_sdl() else { return };
    let named = audio::playback_devices(&sdl).first().map(|(_, n)| n.clone()).expect("the dummy driver has a playback device");
    let mut ui = UiSounds::new(sdl.clone());
    ui.remember("tick", vec![0u8; 4096]);

    // A listed name is kept and the first sound opens the stream on it.
    assert!(ui.set_device(Some(&named)));
    assert_eq!(ui.device(), Some(named.as_str()));
    ui.play("tick");
    assert!(ui.is_open());

    // A new choice lets the open stream go; the next sound opens it again.
    assert!(!ui.set_device(Some("No such sound output")));
    assert_eq!(ui.device(), None);
    assert!(!ui.is_open(), "the stream on the old device is let go");
    ui.play("tick");
    assert!(ui.is_open(), "the system default plays in its place");

    // None and an empty name are both the system default, and choosing the same again keeps the stream.
    assert!(ui.set_device(None));
    assert!(ui.is_open());
    assert!(ui.set_device(Some("")));
    ui.dispose();
}
