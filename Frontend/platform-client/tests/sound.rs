//! The interface player and the game player with SDL3 lent, on SDL's dummy audio driver so that no sound reaches the speakers. SDL is lent
//! once a process, so this is a binary of its own with one test.

use emusen_client_fixture::{World, library};
use emusen_platform_client::audio::{GameSound, SoundSettings};
use emusen_platform_client::sounds::{decode, InterfacePlayer};
use emusen_platform_client::{Lent, Platform};

#[test]
fn a_remembered_sound_and_a_games_sound_play_once_sdl_is_lent() {
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

    // A theme's WAV decoded as the player keeps sounds: a tenth of a second of 24 kHz mono 16-bit becomes 4,800 frames of
    // 48 kHz stereo float, more than any first buffer, so the count is asked and the rest taken; a missing file is none.
    let wav = _world.root.join("tick.wav");
    let samples: Vec<u8> = (0..2400i16).flat_map(|i| (i % 64 * 256).to_le_bytes()).collect();
    let mut file = b"RIFF".to_vec();
    file.extend((36 + samples.len() as u32).to_le_bytes());
    file.extend(b"WAVEfmt ");
    file.extend(16u32.to_le_bytes());
    file.extend([1u16, 1].iter().flat_map(|v| v.to_le_bytes()));
    file.extend([24000u32, 48000].iter().flat_map(|v| v.to_le_bytes()));
    file.extend([2u16, 16].iter().flat_map(|v| v.to_le_bytes()));
    file.extend(b"data");
    file.extend((samples.len() as u32).to_le_bytes());
    file.extend(&samples);
    std::fs::write(&wav, file).unwrap();
    let decoded = decode(&p, wav.to_str().unwrap()).unwrap().expect("SDL reads a WAV");
    assert_eq!(decoded.len(), 4800 * 2 * 4);
    player.remember("tick", &decoded).unwrap();
    player.play("tick").unwrap();
    assert_eq!(decode(&p, _world.root.join("none.wav").to_str().unwrap()).unwrap(), None);

    // The game player: its device open at the settings' rate, reopened at the game's, and holding what it was given.
    let mut game = GameSound::new(&p, SoundSettings::default()).unwrap();
    assert!(game.available().unwrap(), "the device opens with the player");
    assert_eq!(game.sample_rate().unwrap(), 32000);
    let tone: Vec<i16> = (0..800).flat_map(|_| [1000, -1000]).collect();
    game.submit(&tone, 48000).unwrap();
    assert_eq!(game.sample_rate().unwrap(), 48000, "the device opened at the game's rate");
    assert!(game.queued_frames().unwrap() > 0, "the tone is queued");
    assert_eq!(game.taken_frames().unwrap(), 800, "the rate control took the tone's frames");
    game.set_volume(0.25).unwrap();
    game.reset().unwrap();
    game.submit(&tone, 32000).unwrap();
    assert_eq!(game.sample_rate().unwrap(), 32000, "a new rate reopens the device");
}
