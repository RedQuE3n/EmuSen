//! The playback devices and the player's choice of one, against SDL's dummy driver (EmuSen_Audio_Sync.md §7.4).

use emusen_endymion::audio::{self, AudioPlayer};
use emusen_endymion::rate_control::DynamicRateControl;
use emusen_endymion::sdl::{self, Sdl};
use std::sync::Arc;

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
