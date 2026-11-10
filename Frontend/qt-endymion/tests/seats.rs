//! The seats on Endymion's simulated pads, through the platform library built from this commit (EmuSen_Input.md §8.9,
//! EmuSen_Qt.md §7.7.H): a pad seated as it is opened, its seat kept when it goes and given back when it returns, a
//! pad moved to another player trading with the one there, a kept seat let go of, and the game hearing the seats.

use std::sync::Arc;

use emusen_client_fixture::{library, World};
use emusen_platform_client::pads::{consts, Pads, SimPad, SimSet};
use emusen_platform_client::Platform;
use emusen_qt_endymion::game::GamePads;
use emusen_qt_endymion::reading::Reading;
use emusen_qt_endymion::seats::{Ask, Player};

fn pad(p: &Arc<Platform>, name: &str, path: &str) -> SimPad {
    let pad = SimPad::new(p, Some(path)).unwrap();
    pad.set(consts::SIM_PAD_KIND, 3).unwrap();
    pad.set_text(consts::SIM_PAD_NAME, name).unwrap();
    pad
}

fn seats(game: &GamePads) -> Vec<(usize, Option<String>, bool)> {
    game.players().into_iter().map(|Player { player, pad, kept }| (player, pad, kept)).collect()
}

#[test]
fn qt_7_7_h_pads_take_seats_keep_them_while_gone_move_and_are_let_go_of() {
    let _world = World::new("endymion-seats");
    let p = Platform::load(library()).unwrap();
    let set = SimSet::new(&p).unwrap();
    let one = pad(&p, "Pad One", "/dev/one");
    let two = pad(&p, "Pad Two", "/dev/two");
    set.connect(&one).unwrap();
    set.connect(&two).unwrap();
    let pads = Pads::simulated(&p, &set).unwrap();
    let game = GamePads::default();
    let mut reading = Reading::default();
    let tick = reading.start(&pads).unwrap();
    assert!(game.keep_seats(&pads, &tick.devices).unwrap(), "the pads opened are seated");
    assert_eq!(seats(&game), [(1, Some("Pad One".into()), false), (2, Some("Pad Two".into()), false)]);
    let connected = game.connected();
    assert_eq!(connected.iter().map(|c| (c.name.as_str(), c.player)).collect::<Vec<_>>(), [("Pad One", 1), ("Pad Two", 2)]);

    // Pad Two pulled out: its seat is kept for it, and given back when it returns.
    set.disconnect(&two).unwrap();
    let tick = reading.poll(&pads).unwrap();
    game.keep_seats(&pads, &tick.devices).unwrap();
    assert_eq!(seats(&game)[1], (2, Some("Pad Two".into()), true));
    set.connect(&two).unwrap();
    let tick = reading.poll(&pads).unwrap();
    game.keep_seats(&pads, &tick.devices).unwrap();
    assert_eq!(seats(&game)[1], (2, Some("Pad Two".into()), false), "its own seat back");

    // Pad One moved to player 2: the two trade seats.
    let one_key = game.connected()[0].key;
    game.ask(Ask::Move { key: one_key, player: 2 });
    game.keep_seats(&pads, &[]).unwrap();
    assert_eq!(seats(&game), [(1, Some("Pad Two".into()), false), (2, Some("Pad One".into()), false)]);

    // Pad Two gone again, then its kept seat let go of.
    set.disconnect(&two).unwrap();
    let tick = reading.poll(&pads).unwrap();
    game.keep_seats(&pads, &tick.devices).unwrap();
    assert_eq!(seats(&game)[0], (1, Some("Pad Two".into()), true));
    game.ask(Ask::Forget { player: 1 });
    game.keep_seats(&pads, &[]).unwrap();
    assert_eq!(seats(&game), [(1, None, false), (2, Some("Pad One".into()), false)]);
    // A button held on a pad is in what the Players page reads.
    one.press(0, true).unwrap();
    reading.poll(&pads).unwrap();
    game.keep_seats(&pads, &[]).unwrap();
    let held = game.connected();
    assert!(held[0].pressed && held[0].buttons & 1 == 1, "{held:?}");
}

#[test]
fn qt_7_7_h_the_stick_threshold_is_endymion_s_stick_deadzone() {
    let _world = World::new("endymion-deadzone");
    let p = Platform::load(library()).unwrap();
    let set = SimSet::new(&p).unwrap();
    let pads = Pads::simulated(&p, &set).unwrap();
    let game = GamePads::default();
    game.keep_seats(&pads, &[]).unwrap();
    assert_eq!(game.stick_deadzone(), pads.setting(consts::SETTING_STICK_DEADZONE).unwrap(), "read from the pads, not restated");
    pads.set_setting(consts::SETTING_STICK_DEADZONE, 0.3).unwrap();
    assert!(game.keep_seats(&pads, &[]).unwrap(), "a new deadzone is a change the page hears");
    assert_eq!(game.stick_deadzone(), 0.3);
}
