//! The Controllers settings acting on Endymion's simulated pads, through the platform library built from this commit
//! (EmuSen_Qt.md §7.7.H3): Only the first controller, Left stick as the D-pad and Stick deadzone set on the pads at the
//! reader's tick and heard by the interface and the game, the mirror handed to the game, and the notices of pads that
//! come and go.

use std::sync::Arc;

use emusen_client_fixture::{library, World};
use emusen_platform_client::pads::{consts, Pads, SimPad, SimSet};
use emusen_platform_client::Platform;
use emusen_qt_endymion::game::{GamePads, GameReading, PadSettings, Read, Want};
use emusen_qt_endymion::reading::Reading;

fn pad(p: &Arc<Platform>, name: &str, path: &str) -> SimPad {
    let pad = SimPad::new(p, Some(path)).unwrap();
    pad.set(consts::SIM_PAD_KIND, 3).unwrap();
    pad.set_text(consts::SIM_PAD_NAME, name).unwrap();
    pad
}

const PLAIN: PadSettings = PadSettings { first_controller_only: false, stick_as_dpad: true, stick_deadzone: Some(0.5), mirror: false, keyboard_player: Some(1) };

/// One tick as the reader runs it: the settings, a poll, the seats, the game's read; the read handed over, if any.
fn tick(pads: &Pads, reading: &mut Reading, game: &GamePads, game_reading: &mut GameReading) -> (Vec<String>, Option<Read>) {
    game.apply_settings(pads).unwrap();
    let t = reading.poll(pads).unwrap();
    game.keep_seats(pads, &t.devices).unwrap();
    game_reading.read(pads, game).unwrap();
    let mut read = Read::default();
    (game.notices(), game.take(&mut read).then_some(read))
}

#[test]
fn qt_7_7_h3_the_settings_reach_endymion_s_pads_and_the_game() {
    let _world = World::new("endymion-settings");
    let p = Platform::load(library()).unwrap();
    let set = SimSet::new(&p).unwrap();
    let one = pad(&p, "Pad One", "/dev/one");
    let two = pad(&p, "Pad Two", "/dev/two");
    set.connect(&one).unwrap();
    set.connect(&two).unwrap();
    let pads = Pads::simulated(&p, &set).unwrap();
    let game = GamePads::default();
    let mut reading = Reading::default();
    let mut game_reading = GameReading::default();
    let start = reading.start(&pads).unwrap();
    game.keep_seats(&pads, &start.devices).unwrap();
    game.want(Some(Want { console: "SNES".into(), game: None, ports: 2, axes: Vec::new() }));
    game.set_owned(true);
    tick(&pads, &mut reading, &game, &mut game_reading);

    // Only the first controller: the interface reads one pad, the game hears player 1 alone; off, both again.
    game.settle(PadSettings { first_controller_only: true, ..PLAIN });
    let (_, read) = tick(&pads, &mut reading, &game, &mut game_reading);
    assert_eq!(pads.setting(consts::SETTING_FIRST_CONTROLLER_ONLY).unwrap(), 1.0);
    assert_eq!(pads.get(0, consts::FRONTEND_COUNT, 0).unwrap(), 1, "the menus hear the first pad alone");
    assert_eq!(read.expect("a read").seated, 0b01, "the game hears player 1 alone");
    game.settle(PLAIN);
    let (_, read) = tick(&pads, &mut reading, &game, &mut game_reading);
    assert_eq!(read.expect("a read").seated, 0b11);
    assert_eq!(pads.get(0, consts::FRONTEND_COUNT, 0).unwrap(), 2);

    // Left stick as the D-pad, past Stick deadzone: 0.4 of its travel is nothing at 0.5 and a direction at 0.3; with the
    // setting off, nothing however far.
    one.set_axis(0, 0.4).unwrap();
    let (_, read) = tick(&pads, &mut reading, &game, &mut game_reading);
    assert_eq!(read.expect("a read").held[0], 0, "under the deadzone");
    game.settle(PadSettings { stick_deadzone: Some(0.3), ..PLAIN });
    let (_, read) = tick(&pads, &mut reading, &game, &mut game_reading);
    assert_eq!(pads.setting(consts::SETTING_STICK_DEADZONE).unwrap(), 0.3);
    assert_ne!(read.expect("a read").held[0], 0, "past the deadzone the stick is the d-pad");
    game.settle(PadSettings { stick_as_dpad: false, stick_deadzone: Some(0.3), ..PLAIN });
    one.set_axis(0, 1.0).unwrap();
    let (_, read) = tick(&pads, &mut reading, &game, &mut game_reading);
    assert_eq!(pads.setting(consts::SETTING_STICK_AS_DPAD).unwrap(), 0.0);
    assert_eq!(read.expect("a read").held[0], 0, "the stick is no d-pad with the setting off");
    one.set_axis(0, 0.0).unwrap();

    // Mirror player 1 to player 2 is the game's to route: handed over with each read.
    game.settle(PadSettings { mirror: true, ..PLAIN });
    let (_, read) = tick(&pads, &mut reading, &game, &mut game_reading);
    assert!(read.expect("a read").mirror);
    game.settle(PLAIN);
    let (_, read) = tick(&pads, &mut reading, &game, &mut game_reading);
    assert!(!read.expect("a read").mirror);
}

#[test]
fn qt_7_7_h3_a_pad_coming_or_going_is_told_in_the_notice_s_words() {
    let _world = World::new("endymion-notices");
    let p = Platform::load(library()).unwrap();
    let set = SimSet::new(&p).unwrap();
    let one = pad(&p, "Pad One", "/dev/one");
    let two = pad(&p, "Pad Two", "/dev/two");
    let pads = Pads::simulated(&p, &set).unwrap();
    let game = GamePads::default();
    let mut reading = Reading::default();
    let mut game_reading = GameReading::default();
    reading.start(&pads).unwrap();

    set.connect(&one).unwrap();
    assert_eq!(tick(&pads, &mut reading, &game, &mut game_reading).0, ["Pad One: player 1"]);
    set.connect(&two).unwrap();
    assert_eq!(tick(&pads, &mut reading, &game, &mut game_reading).0, ["Pad Two: player 2"]);
    set.disconnect(&one).unwrap();
    assert_eq!(tick(&pads, &mut reading, &game, &mut game_reading).0, ["Pad One disconnected: player 1 is kept for it"]);
    set.disconnect(&two).unwrap();
    assert_eq!(tick(&pads, &mut reading, &game, &mut game_reading).0, ["Pad Two disconnected: player 2 is kept for it"]);
    // Pad Two back in its own kept seat while player 1's pad is still gone: player 1 has no controller.
    set.connect(&two).unwrap();
    assert_eq!(
        tick(&pads, &mut reading, &game, &mut game_reading).0,
        ["Pad Two: player 2, and player 1 has no controller. Swap Players under Game Settings makes it player 1."]
    );
    assert!(tick(&pads, &mut reading, &game, &mut game_reading).0.is_empty(), "each told once");
}
