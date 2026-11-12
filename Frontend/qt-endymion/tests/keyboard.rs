//! The keyboard over a running game (EmuSen_Qt.md §7.7.H6), through the platform library built from this commit: a key the
//! console's map binds held for the game by its PadControl and handed over with the keyboard's player, let go when a menu
//! takes the keyboard; a hotkey's key naming its action; a key bound to nothing nothing.

use emusen_client_fixture::{library, World};
use emusen_platform_client::bindings::consts as files;
use emusen_platform_client::pads::{Pads, SimSet};
use emusen_platform_client::Platform;
use emusen_qt_endymion::game::{GamePads, GameReading, KeyUse, PadSettings, Read, Want};
use emusen_qt_endymion::hotkeys::Hotkey;
use emusen_qt_endymion::keys::{heard, Heard};
use emusen_qt_endymion::reading::Reading;

const PLAIN: PadSettings = PadSettings { first_controller_only: false, stick_as_dpad: true, stick_deadzone: Some(0.5), mirror: false, keyboard_player: Some(1) };

/// One tick as the reader runs it; the read handed over, if any.
fn tick(pads: &Pads, reading: &mut Reading, game: &GamePads, game_reading: &mut GameReading) -> Option<Read> {
    game.apply_settings(pads).unwrap();
    let t = reading.poll(pads).unwrap();
    game.keep_seats(pads, &t.devices).unwrap();
    game_reading.read(pads, game).unwrap();
    let mut read = Read::default();
    game.take(&mut read).then_some(read)
}

/// The word of a key Qt names by its number, as the window hears it.
fn word(qt_key: i32) -> &'static str {
    match heard(qt_key, 0) {
        Heard::Key(word) => word,
        other => panic!("{qt_key:#x} is {other:?}"),
    }
}

#[test]
fn qt_7_7_h6_a_consoles_key_is_held_for_the_game_and_a_hotkey_names_its_action() {
    let _world = World::new("endymion-keyboard");
    let p = Platform::load(library()).unwrap();
    // The SNES's Start on N, a key no control has by default, saved as the Keyboard keys page saves a key.
    let mut keys = p.bindings_load(files::KEYS, &["SNES"]).unwrap();
    keys["SNES"]["Start"] = "N".into();
    p.bindings_save(files::KEYS, &keys).unwrap();
    let controls = p.bindings_controls().unwrap();
    let start = 1u32 << controls.as_array().unwrap().iter().position(|c| c["word"] == "Start").unwrap();

    let set = SimSet::new(&p).unwrap();
    let pads = Pads::simulated(&p, &set).unwrap();
    let game = GamePads::default();
    let (mut reading, mut game_reading) = (Reading::default(), GameReading::default());
    let opened = reading.start(&pads).unwrap();
    game.keep_seats(&pads, &opened.devices).unwrap();
    game.want(Some(Want { console: "SNES".into(), game: None, ports: 2, axes: Vec::new() }));
    game.settle(PadSettings { keyboard_player: Some(2), ..PLAIN });
    game.set_owned(true);
    let read = tick(&pads, &mut reading, &game, &mut game_reading).expect("a read");
    assert_eq!((read.keyboard, read.keyboard_player), (0, 2), "nothing held; the keyboard plays as player 2");

    // N down: Start held for the game; up: let go.
    assert_eq!(game.key(word(0x4e), true), KeyUse::Control);
    assert_eq!(tick(&pads, &mut reading, &game, &mut game_reading).unwrap().keyboard, start, "N holds Start");
    assert_eq!(game.key("n", false), KeyUse::Control, "a key in any case");
    assert_eq!(tick(&pads, &mut reading, &game, &mut game_reading).unwrap().keyboard, 0);

    // Held as a menu takes the keyboard: let go for the game, which hears nothing of it after.
    game.key("N", true);
    game.set_owned(false);
    game.set_owned(true);
    assert_eq!(tick(&pads, &mut reading, &game, &mut game_reading).unwrap().keyboard, 0, "a key under a menu is not the game's");

    // Endymion's hotkeys by their keys; a key bound to nothing is nothing.
    let hotkeys = p.bindings_defaults(files::HOTKEYS).unwrap();
    for (action, key) in hotkeys.as_object().unwrap() {
        let key = key.as_str().unwrap();
        assert_eq!(game.key(key, true), KeyUse::Hotkey(Hotkey::named(action).unwrap()), "{key} runs {action}");
    }
    assert_eq!(game.key(word(0x0100_0047), true), KeyUse::Nothing, "F24 is bound to nothing");
    assert_eq!(emusen_qt_endymion::keys::word(0x0100_0000, 0).map(|w| game.key(w, true)), Some(KeyUse::Hotkey(Hotkey::ExitToLibrary)), "Escape over a game is Exit to Library's");

    // A game closed: no key is anything until the next game's are read.
    game.want(None);
    assert_eq!(game.key("N", true), KeyUse::Nothing);
}

#[test]
fn qt_7_7_h6_every_action_of_endymions_hotkeys_has_its_action_here() {
    let _world = World::new("endymion-hotkeys");
    let p = Platform::load(library()).unwrap();
    let defaults = p.bindings_defaults(files::HOTKEYS).unwrap();
    let actions: Vec<&String> = defaults.as_object().unwrap().keys().collect();
    assert_eq!(actions.len(), 10, "{actions:?}");
    for action in actions {
        let hotkey = Hotkey::named(action).unwrap_or_else(|| panic!("{action} has no action here"));
        assert_eq!(hotkey.word(), action);
        assert_eq!(hotkey.held(), action == "FastForward" || action == "Rewind", "{action}: held only for Fast Forward and Rewind, as Mistress's IsHeld");
    }
}
