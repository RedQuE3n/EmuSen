//! The interface player with SDL3 lent, on SDL's dummy audio driver so that no sound reaches the speakers. SDL is lent
//! once a process, so this is a binary of its own with one test.

use emusen_client_fixture::{World, library};
use emusen_platform_client::sounds::InterfacePlayer;
use emusen_platform_client::{Lent, Platform};

#[test]
fn a_remembered_sound_plays_once_sdl_is_lent() {
    // Set before SDL is first loaded, while this binary's one test is its only thread that reads the environment.
    unsafe { std::env::set_var("SDL_AUDIODRIVER", "dummy") };
    let _world = World::new("sound");
    let p = Platform::load(library()).unwrap();
    p.lend_sdl(&Lent::open("libSDL3.so.0").unwrap()).unwrap();
    let player = InterfacePlayer::new(&p).unwrap();
    // A tenth of a second of silence: 4,800 frames of 48 kHz stereo 32-bit float.
    player.remember("silence", &vec![0u8; 4800 * 2 * 4]).unwrap();
    assert!(!player.is_open().unwrap(), "the stream opens with the first sound");
    player.set_volume(0.5).unwrap();
    player.play("silence").unwrap();
    assert!(player.is_open().unwrap());
    assert!(!player.set_device(Some("no such device")).unwrap(), "the default plays where no device has the name");
}
