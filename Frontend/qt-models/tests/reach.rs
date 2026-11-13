//! A change made on Settings ▸ Controllers reaching a running game (EmuSen_Qt.md §7.7.H8): a button rebound, a profile
//! chosen, a key rebound, a hotkey rebound and a seat moved, each made through the screen's model over the stored files,
//! handed to the pads as SettingsModel hands it (`pads_reach`, `GamePads::ask`), then pressed, the effect seen at the
//! client fixture's console core; and the Players page's Keyboard read back from the stored file to the router. No window
//! and no GPU.

mod rig;

use emusen_client_fixture::{library, World};
use emusen_crystal_client::shapes::Controllers;
use emusen_platform_client::pads::{SimPad, SimSet};
use emusen_qt_endymion::game::KeyUse;
use emusen_qt_endymion::hotkeys::Hotkey;
use emusen_qt_models::consoles::{Console, ConsoleRead};
use emusen_qt_models::pads_reach;
use emusen_qt_models::settings::{self, Outcome, SettingsSource, SettingsView};
use emusen_qt_models::settings_controllers::Seated;
use emusen_qt_models::settings_source::ClientSettings;
use rig::{Rig, B_BLUE, PLAIN, SOUTH, TWO_PADS};

/// SDL's North button, as Endymion's reading numbers it.
fn north() -> i32 {
    emusen_qt_endymion::reading::sdl_button(emusen_kit_core::input::PadControl::North).unwrap()
}

/// Qt's numbers for Z (B's key by Endymion's default), M and F6.
const Z: i32 = 0x5a;
const M: i32 = 0x4d;
const F6: i32 = 0x0100_0035;

struct Screen {
    source: ClientSettings,
    view: SettingsView,
}

impl Screen {
    /// Settings as the program reads it, from the stored files, over the console core's PlayStation and the seats the
    /// pads' reader keeps; at Controllers.
    fn read(rig: &Rig) -> Screen {
        let source = ClientSettings::open(&library().display().to_string()).unwrap();
        let mut view = SettingsView::new();
        view.read = settings::read(&source).unwrap();
        let controllers = Controllers { buttons: (0..16).collect(), axes: Vec::new(), ports: 2, players: vec![Some("psx.pad".into()), Some("psx.pad".into())] };
        let console = Console { console: "PlayStation".into(), name: "PlayStation".into(), systems: vec!["psx".into()], engines: vec!["Engine".into()], year: 1994, controllers, ..Console::default() };
        view.read.consoles = ConsoleRead { consoles: vec![console], ..ConsoleRead::default() };
        view.read.bindings = source.bindings(&["PlayStation"]).unwrap();
        view.chooser = Some(source.chooser().unwrap());
        view.seats = Seated { players: rig.hand.players(), pads: rig.hand.connected(), deadzone: 0.5 };
        view.choose_section(0);
        Screen { source, view }
    }

    fn at(&self, id: &str) -> usize {
        let rows = self.view.rows();
        rows.iter().position(|r| r.id == id).unwrap_or_else(|| panic!("no row {id} in {:?}", rows.iter().map(|r| r.id.clone()).collect::<Vec<_>>()))
    }

    /// An outcome handed to the pads as SettingsModel hands it.
    fn hand(&self, rig: &Rig, outcome: Outcome) {
        match outcome {
            Outcome::Bindings(files) => pads_reach::bindings_saved(&self.source, &self.view.read.bindings, &files, Some(&rig.hand)).unwrap(),
            Outcome::Seat(ask) => rig.hand.ask(ask),
            Outcome::Saved(model, document) => self.source.save(model, &document).unwrap(),
            other => panic!("{other:?} reaches no pad"),
        }
    }

    /// A control of the drawing focused on the bindings page, and A to listen.
    fn listen(&mut self, control: &str) {
        let drawn = self.view.drawn().expect("the bindings page");
        let at = drawn.controls.iter().position(|c| c == control).unwrap_or_else(|| panic!("no {control} in {:?}", drawn.controls));
        assert!(self.view.binding_focus(at));
        assert!(self.view.binding_accept(), "A listens for {control}");
    }
}

/// The console core with one simulated pad, its settings settled.
fn running(name: &str, payload: Option<&[u8]>) -> (World, Rig, SimSet, SimPad) {
    let (world, mut rig, set, pad) = rig::with_pad(name, payload);
    rig.hand.settle(PLAIN);
    rig.run(3);
    (world, rig, set, pad)
}

/// The blue port 0 shows with a pad button held.
fn pressed(rig: &mut Rig, pad: &SimPad, button: i32) -> u8 {
    pad.press(button, true).unwrap();
    let blue = rig.run(3);
    pad.press(button, false).unwrap();
    rig.run(3);
    blue
}

#[test]
fn qt_7_7_h8_a_button_rebound_on_the_bindings_page_reaches_the_running_game() {
    let (_world, mut rig, _set, pad) = running("reach-button", None);
    assert_ne!(pressed(&mut rig, &pad, north()), B_BLUE, "North is not B before");
    let mut screen = Screen::read(&rig);
    screen.view.choose(screen.at("bindings"));
    screen.listen("B");
    let outcome = screen.view.heard_down(1 << 3);
    screen.hand(&rig, outcome);
    rig.run(1);
    assert_eq!(pressed(&mut rig, &pad, north()), B_BLUE, "North is B in the running game");
}

#[test]
fn qt_7_7_h8_a_profile_chosen_reaches_the_running_game() {
    let (_world, mut rig, _set, pad) = running("reach-profile", None);
    let mut screen = Screen::read(&rig);
    screen.view.choose(screen.at("profiles"));
    let Outcome::Type { typed, .. } = screen.view.choose(screen.at("newprofile")) else { panic!("New profile asks for a name") };
    let made = screen.view.typed(&typed, "Couch");
    screen.hand(&rig, made);
    // B on North in Couch alone: the game plays the console's own profile still.
    screen.view.choose(screen.at("profile:Couch"));
    screen.view.choose(screen.at("pbindings"));
    screen.listen("B");
    let outcome = screen.view.heard_down(1 << 3);
    screen.hand(&rig, outcome);
    rig.run(1);
    assert_ne!(pressed(&mut rig, &pad, north()), B_BLUE, "Couch is not chosen yet");
    screen.view.back();
    let chosen = screen.view.choose(screen.at("pdefault"));
    screen.hand(&rig, chosen);
    rig.run(1);
    assert_eq!(pressed(&mut rig, &pad, north()), B_BLUE, "Couch chosen: North is B in the running game");
}

#[test]
fn qt_7_7_h8_a_key_rebound_on_the_keyboard_keys_page_reaches_the_running_game() {
    let (_world, mut rig, _set, _pad) = running("reach-key", None);
    let mut screen = Screen::read(&rig);
    screen.view.choose(screen.at("keys"));
    screen.listen("B");
    let outcome = screen.view.heard_key(Some("M"));
    screen.hand(&rig, outcome);
    rig.run(1);
    assert_eq!(rig.key(M, true), KeyUse::Control);
    assert_eq!(rig.run(3), B_BLUE, "M is B in the running game");
    rig.key(M, false);
    rig.run(2);
    assert_eq!(rig.key(Z, true), KeyUse::Nothing, "Z is B's no longer");
}

#[test]
fn qt_7_7_h8_a_hotkey_rebound_on_the_hotkeys_page_reaches_the_running_game() {
    let (_world, mut rig, _set, _pad) = running("reach-hotkey", None);
    let mut screen = Screen::read(&rig);
    screen.view.choose(screen.at("hotkeys"));
    assert_eq!(screen.view.choose(screen.at("hk:SaveState")), Outcome::Shown, "A listens for Save State's key");
    let outcome = screen.view.heard_key(Some("F6"));
    screen.hand(&rig, outcome);
    rig.run(1);
    assert_eq!(rig.key(F6, true), KeyUse::Hotkey(Hotkey::SaveState), "F6 saves in the running game");
    assert_eq!(rig.hand.hotkey("F6"), Some(Hotkey::SaveState), "and over the library");
}

#[test]
fn qt_7_7_h8_a_seat_moved_on_players_reaches_the_running_game() {
    let (_world, mut rig, _set, pad) = running("reach-seat", Some(&[TWO_PADS, 1, 2, 3]));
    pad.press(SOUTH, true).unwrap();
    assert_eq!(rig.run_both(3), (B_BLUE, 0), "player 1's pad on port 0");
    pad.press(SOUTH, false).unwrap();
    rig.run(3);
    let mut screen = Screen::read(&rig);
    screen.view.choose(screen.at("players"));
    let moved = screen.view.step(screen.at("pad0"), 1);
    screen.hand(&rig, moved);
    rig.run(3);
    pad.press(SOUTH, true).unwrap();
    assert_eq!(rig.run_both(3), (0, B_BLUE), "the pad moved to player 2 plays on port 1");
    pad.press(SOUTH, false).unwrap();
}

#[test]
fn qt_7_7_h8_the_stored_keyboard_player_reaches_the_router() {
    let (_world, mut rig, _set, _pad) = running("reach-keyboard-player", None);
    let mut screen = Screen::read(&rig);
    screen.view.choose(screen.at("players"));
    let stored = screen.view.step(screen.at("keyboardplayer"), 1);
    screen.hand(&rig, stored);
    // The settings read again from the file, and settled as SettingsModel settles them.
    let screen = Screen::read(&rig);
    assert_eq!(pads_reach::settle(&screen.view, Some(&rig.hand)), None, "read without a fault");
    rig.run(1);
    rig.key(Z, true);
    assert_eq!(rig.run(3), 0, "the keyboard plays as player 2: port 0 hears nothing");
    rig.key(Z, false);
    rig.run(2);
    // Stepped back to player 1 and stored again: port 0 hears it.
    let mut screen = Screen::read(&rig);
    screen.view.choose(screen.at("players"));
    let stored = screen.view.step(screen.at("keyboardplayer"), -1);
    screen.hand(&rig, stored);
    pads_reach::settle(&Screen::read(&rig).view, Some(&rig.hand));
    rig.run(1);
    rig.key(Z, true);
    assert_eq!(rig.run(3), B_BLUE, "player 1's again");
}

#[test]
fn qt_7_7_h8_reset_every_binding_reaches_the_running_game() {
    let (_world, mut rig, _set, pad) = running("reach-reset", None);
    let mut screen = Screen::read(&rig);
    screen.view.choose(screen.at("bindings"));
    screen.listen("B");
    let outcome = screen.view.heard_down(1 << 3);
    screen.hand(&rig, outcome);
    rig.run(1);
    assert_eq!(pressed(&mut rig, &pad, north()), B_BLUE, "North is B");
    screen.view.back();
    let reset = screen.view.choose(screen.at("resetbindings"));
    screen.hand(&rig, reset);
    rig.run(1);
    assert_ne!(pressed(&mut rig, &pad, north()), B_BLUE, "North is B's no longer in the running game");
    assert_eq!(pressed(&mut rig, &pad, SOUTH), B_BLUE, "South is B again");
}
