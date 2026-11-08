//! A game hears the pads through the interface's one reader (EmuSen_Qt.md §1.8.5): Endymion's simulated pad, read by
//! the reader's game half, routed on the frame thread's side to the test core, whose RAM byte 3 counts up each frame its
//! port 0's first button is held. No window and no GPU.

use emusen_client_fixture::{library, World};
use emusen_crystal_client::{frame_info, Api, Config, Open, Runtime};
use emusen_platform_client::pads::{Pads, SimPad, SimSet};
use emusen_platform_client::Platform;
use emusen_qt_endymion::game::{GamePads, GameReading};
use emusen_qt_models::input::GameInput;
use std::sync::Arc;

/// RetroPad's A, the test core's first button.
const A: usize = 8;

fn with_test_core(world: &World) -> Runtime {
    let player = world.dir("Cores");
    world.player_core(&player);
    let api = Api::load(library()).unwrap();
    let rt = Runtime::new(&api, &Config { development: false, cores: Some(Vec::new()), player_cores: Some(vec![player.to_str().unwrap().into()]) })
        .unwrap();
    let sha = rt.cores().unwrap().into_iter().find(|c| c.directory.as_deref() == Some("player")).and_then(|c| c.sha256).expect("the player's core");
    rt.core_approve(&sha).unwrap();
    rt
}

#[test]
fn the_game_hears_the_pad_through_the_players_bindings_and_not_the_press_that_gave_it_the_pad() {
    let world = World::new("input");
    let rt = with_test_core(&world);
    let platform = Platform::load(library()).unwrap();
    let set = SimSet::new(&platform).unwrap();
    let pad = SimPad::new(&platform, None).unwrap();
    set.connect(&pad).unwrap();
    let pads = Pads::simulated(&platform, &set).unwrap();
    pads.start().unwrap();

    let game = world.game("game.tst", &[1, 2, 3, 4]);
    let path = game.to_str().unwrap();
    let session = rt.open(&Open { path: Some(path.into()), ..Open::default() }).unwrap();
    let service = rt.session_service(session, 0).unwrap();
    let console = rt.session_info(session).unwrap().console;
    let bound = platform.bindings_bound(&console, 1, Some(path)).unwrap()[A];
    assert!(bound >= 0, "A is bound for {console}");

    let hand = Arc::new(GamePads::default());
    let mut reading = GameReading::default();
    let mut input = GameInput::new(hand.clone(), &platform).unwrap();
    input.start(&rt, session, path).unwrap();
    let mut info = frame_info();
    let counted = || {
        let mut b = [0u8];
        service.space_read(session, "RAM", 3, &mut b).unwrap();
        b[0]
    };
    let mut run = |frames: usize, input: &mut GameInput, reading: &mut GameReading| {
        for _ in 0..frames {
            pads.poll().unwrap();
            pads.update().unwrap();
            reading.read(&pads, &hand).unwrap();
            input.frame(&rt, session).unwrap();
            rt.frame(session, &mut info).unwrap();
        }
    };

    // The press that gives the game the pad (the one that closed a menu) is not the game's until let go.
    // The game's image is the RAM from byte 1, so the count starts at its third byte.
    let base = counted();
    pad.press(bound, true).unwrap();
    run(3, &mut input, &mut reading);
    assert_eq!(counted(), base, "a game that does not own the pad hears nothing");
    input.set_owned(true);
    run(5, &mut input, &mut reading);
    assert_eq!(counted(), base, "a button held as the game took the pad is not heard");
    pad.press(bound, false).unwrap();
    run(2, &mut input, &mut reading);
    pad.press(bound, true).unwrap();
    run(10, &mut input, &mut reading);
    let held = counted();
    assert!(held >= base + 8, "A held for 10 frames is heard on port 0 ({held} from {base})");
    pad.press(bound, false).unwrap();
    run(5, &mut input, &mut reading);
    assert!(counted() <= held + 1, "let go, it is not heard");

    // Muted while Guide is held for its chords (Mistress's `_padQuiet`): A is not heard, nor after, until let go.
    hand.set_muted(true);
    let muted = counted();
    pad.press(bound, true).unwrap();
    run(5, &mut input, &mut reading);
    assert_eq!(counted(), muted, "a muted game hears nothing");
    hand.set_muted(false);
    run(5, &mut input, &mut reading);
    assert_eq!(counted(), muted, "A held through the mute is not heard after it");
    pad.press(bound, false).unwrap();
    run(2, &mut input, &mut reading);
    pad.press(bound, true).unwrap();
    run(5, &mut input, &mut reading);
    assert!(counted() >= muted + 3, "pressed again, it is heard");
    pad.press(bound, false).unwrap();
    run(5, &mut input, &mut reading);

    // A button bound to nothing on the pad's other buttons is never heard as A.
    let other = if bound == 3 { 2 } else { 3 };
    let quiet = counted();
    pad.press(other, true).unwrap();
    run(5, &mut input, &mut reading);
    assert_eq!(counted(), quiet, "another button is not A");

    drop(input);
    rt.close(session).unwrap();
}
