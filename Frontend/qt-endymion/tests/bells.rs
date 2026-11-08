//! Every cue kit-core renders is remembered by Endymion's interface player and plays under its name, on SDL's dummy
//! audio driver so nothing reaches the speakers: each play queues the cue's samples on the stream (EmuSen_Kit.md §7.15).

use std::time::{Duration, Instant};

use emusen_client_fixture::{library, World};
use emusen_kit_core::sound::Cue;
use emusen_qt_endymion::bell_player::{key, open_player};

#[test]
fn every_cue_is_remembered_and_queues_its_samples_when_played() {
    std::env::set_var("SDL_AUDIODRIVER", "dummy");
    let _world = World::new("bells");
    let player = open_player(library().to_str().unwrap()).unwrap();
    for cue in Cue::ALL {
        let start = Instant::now();
        while player.queued().unwrap() > 0 {
            assert!(start.elapsed() < Duration::from_secs(3), "the cue before {cue:?} still queued");
            std::thread::sleep(Duration::from_millis(20));
        }
        player.play(key(cue)).unwrap();
        let bytes = (cue.samples().len() * 4) as i64;
        let queued = player.queued().unwrap();
        assert!(queued > 0 && queued <= bytes, "{cue:?}: {queued} of {bytes} bytes queued");
    }
}
