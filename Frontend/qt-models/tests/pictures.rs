//! The card's runtime half over the platform library built from this commit, no window and no device (EmuSen_Qt.md
//! §7.2): rewind kept as Mistress keeps it, the reel's moments read into the card's facts with their pictures as the
//! provider serves them, a slot saved with its picture beside it where Galaxia names it, loaded, and the game returned
//! to a moment of the reel.

use emusen_client_fixture::{library, World};
use emusen_crystal_client::{frame_info, Api, Config, Open, Runtime};
use emusen_platform_client::galaxia::consts::SAVE_PICTURE;
use emusen_platform_client::Platform;
use emusen_qt_models::pause::{facts_of, Card, Outcome};
use emusen_qt_models::{pictures, reel};

fn with_test_core(world: &World) -> Runtime {
    let player = world.dir("Cores");
    world.player_core(&player);
    let api = Api::load(library()).unwrap();
    let config = Config { development: false, cores: Some(Vec::new()), player_cores: Some(vec![player.to_str().unwrap().into()]) };
    let rt = Runtime::new(&api, &config).unwrap();
    let sha = rt.cores().unwrap().into_iter().find(|c| c.directory.as_deref() == Some("player")).and_then(|c| c.sha256).unwrap();
    rt.core_approve(&sha).unwrap();
    rt
}

#[test]
fn rewind_is_kept_its_pictures_served_and_a_slot_saved_loaded_and_rewound() {
    let world = World::new("pictures");
    let runtime = with_test_core(&world);
    let platform = Platform::load(library()).unwrap();
    let game = world.game("game.tst", &[1, 2, 3, 4]);
    let session = runtime.open(&Open { path: Some(game.to_str().unwrap().into()), ..Open::default() }).unwrap();
    let info = runtime.session_info(session).unwrap();
    assert_eq!(info.rewind_capture, Some(true), "the test core's states are kept");

    reel::keep(&runtime, session).unwrap();
    let mut frame = frame_info();
    for _ in 0..200 {
        runtime.frame(session, &mut frame).unwrap();
    }
    let held = runtime.rewind_reel(session).unwrap();
    assert_eq!(held.interval_frames, reel::interval_for(info.frame_rate_hz), "two snapshots a second");
    let facts = facts_of(&runtime.session_info(session).unwrap(), Some(&held));
    assert!(facts.rewind_why.is_none() && facts.held.len() >= 6, "{facts:?}");

    let newest = *facts.held.last().unwrap();
    let (rgba, width, height) = pictures::picture(&runtime, session, &format!("reel/{newest}")).expect("a moment's picture");
    let wide = (info.picture.width as u32).min(reel::PICTURE_WIDTH);
    assert_eq!((width as u32, rgba.len()), (wide, (width * height * 4) as usize), "the reel's pictures 160 wide, or the game's own where narrower");
    let (rgba, width, height) = pictures::picture(&runtime, session, "now").expect("the picture on show");
    assert_eq!((width, height, rgba.len()), (info.picture.width, info.picture.height * info.picture.row_repeat.max(1), (width * height * 4) as usize));
    assert!(pictures::picture(&runtime, session, "reel/7").is_none(), "a moment not held has no picture");

    // Saved to slot 3 with the picture beside it where Galaxia names it, then loaded.
    runtime.state_slot_save(session, 3).unwrap();
    let slot = runtime.state_slots(session).unwrap().into_iter().find(|s| s.slot == 3).unwrap();
    assert!(slot.exists && slot.saved_at.is_some(), "{slot:?}");
    assert!(platform.save_path(SAVE_PICTURE, &slot.path, None, 0).unwrap().is_file(), "the slot's picture beside it");
    assert!(runtime.state_slot_load(session, 3).unwrap());
    assert!(!runtime.state_slot_load(session, 4).unwrap(), "an empty slot loads nothing");

    // The card's reel over the runtime's, A on its first stop, and the runtime back at that moment.
    for _ in 0..200 {
        runtime.frame(session, &mut frame).unwrap();
    }
    let info = runtime.session_info(session).unwrap();
    let mut card = Card { facts: facts_of(&info, runtime.rewind_reel(session).ok().as_ref()), ..Card::default() };
    assert_eq!(card.choose(1), Outcome::Shown, "the reel opens");
    let Outcome::RewindTo(moment) = card.choose(0) else { panic!("A goes back to a moment") };
    runtime.rewind_to(session, moment).unwrap();
    assert_eq!(runtime.rewind_reel(session).unwrap().moments.last().map(|m| m.frame), Some(moment), "the history after it dropped");
    runtime.close(session).unwrap();
}
