//! The seats on Endymion's simulated pads, through the platform library built from this commit (EmuSen_Input.md §8.9,
//! EmuSen_Qt.md §7.7.H): a pad seated as it is opened, its seat kept when it goes and given back when it returns, a
//! pad moved to another player trading with the one there, and a kept seat let go of. A running game hearing a moved seat
//! is qt-models' reach.rs (EmuSen_Qt.md §7.7.H8).

use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use emusen_client_fixture::{library, World};
use emusen_platform_client::pads::{consts, Pads, SimPad, SimSet};
use emusen_platform_client::Platform;
use emusen_qt_endymion::game::{GamePads, Want};
use emusen_qt_endymion::reader::Reader;
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
    // A trigger past half its travel is held too, by Endymion's bit for it (PADS_HELD, since interface 47).
    one.press(0, false).unwrap();
    one.set_axis(5, 0.8).unwrap();
    reading.poll(&pads).unwrap();
    game.keep_seats(&pads, &[]).unwrap();
    let held = game.connected();
    assert!(held[0].pressed && held[0].buttons == consts::HELD_RIGHT_TRIGGER, "{held:?}");
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

#[test]
fn gameplay_buttons_and_sticks_suppress_live_seat_changes_while_topology_still_updates() {
    let _world = World::new("endymion-seats-gameplay-suppress");
    let p = Platform::load(library()).unwrap();
    let set = Arc::new(SimSet::new(&p).unwrap());
    let one = pad(&p, "Pad One", "/dev/one");
    let two = pad(&p, "Pad Two", "/dev/two");
    set.connect(&one).unwrap();
    let game = Arc::new(GamePads::default());
    let pads = Pads::simulated(&p, &set).unwrap();
    let mut reading = Reading::default();
    let tick = reading.start(&pads).unwrap();
    assert!(game.keep_seats(&pads, &tick.devices).unwrap());
    assert!(!game.keep_seats(&pads, &[]).unwrap());

    game.want(Some(Want { console: "PlayStation".into(), game: None, ports: 2, axes: vec![] }));
    game.set_owned(true);
    one.press(0, true).unwrap();
    let _ = reading.poll(&pads).unwrap();
    assert!(!game.keep_seats(&pads, &[]).unwrap(), "gameplay button press does not dirty seats");
    one.set_axis(0, 0.9).unwrap();
    one.set_axis(2, -0.9).unwrap();
    one.set_axis(5, 0.8).unwrap();
    let _ = reading.poll(&pads).unwrap();
    assert!(!game.keep_seats(&pads, &[]).unwrap(), "gameplay axes do not dirty seats");
    one.press(0, false).unwrap();
    one.set_axis(0, 0.0).unwrap();
    one.set_axis(2, 0.0).unwrap();
    one.set_axis(5, 0.0).unwrap();
    let _ = reading.poll(&pads).unwrap();
    assert!(!game.keep_seats(&pads, &[]).unwrap());

    // Seat topology changes still notify while playing.
    set.connect(&two).unwrap();
    let tick = reading.poll(&pads).unwrap();
    assert!(game.keep_seats(&pads, &tick.devices).unwrap(), "connecting a pad still updates seats");
    set.disconnect(&two).unwrap();
    let tick = reading.poll(&pads).unwrap();
    assert!(game.keep_seats(&pads, &tick.devices).unwrap(), "disconnecting a pad still updates seats");
    game.ask(Ask::Forget { player: 2 });
    assert!(game.keep_seats(&pads, &[]).unwrap(), "forgetting a seat still updates seats");

    // Pausing gameplay restores live seat updates for Players/Tester/PadListen UIs.
    game.set_owned(false);
    one.press(0, true).unwrap();
    let _ = reading.poll(&pads).unwrap();
    assert!(game.keep_seats(&pads, &[]).unwrap(), "menus/pause still see live held buttons");
    one.press(0, false).unwrap();
    let _ = reading.poll(&pads).unwrap();
    assert!(game.keep_seats(&pads, &[]).unwrap());
    game.set_owned(true);
    game.want(None);
    one.press(0, true).unwrap();
    let _ = reading.poll(&pads).unwrap();
    assert!(game.keep_seats(&pads, &[]).unwrap(), "without an open game, seats still update live");
    one.press(0, false).unwrap();
    drop(pads);

    // Reader thread also keeps tick.seats false on gameplay button presses.
    game.want(Some(Want { console: "PlayStation".into(), game: None, ports: 2, axes: vec![] }));
    game.set_owned(true);
    let (tx, rx) = mpsc::channel();
    let opener = (Arc::clone(&p), Arc::clone(&set));
    let reader = Reader::spawn(move || Pads::simulated(&opener.0, &opener.1).map_err(|e| e.to_string()), None, Some(Arc::clone(&game)), move |d| tx.send(d).is_ok());
    let wait = Duration::from_secs(2);
    let _first = rx.recv_timeout(wait).unwrap().unwrap();
    one.press(1, true).unwrap();
    let press_tick = rx.recv_timeout(wait).unwrap().unwrap();
    assert!(!press_tick.seats, "reader tick must keep seats=false on gameplay press");
    drop(reader);
}
