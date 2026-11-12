//! Keyboard keys heard by a game (EmuSen_Qt.md §7.7.H6): a key bound through the bindings model Settings' Keyboard keys
//! page changes and saved as that page saves it, a key pressed as the window names it, and the client fixture's console
//! core, a PlayStation, drawing the button it hears in its picture's blue; the keyboard's player as the Players page sets
//! it; and the pad's own settings the game hears, the left stick as the d-pad and player 1 mirrored to port 1, read in the
//! picture's blue (port 0) and green (port 1). No window and no GPU: the reader's tick and the frame thread's work are run
//! in turn, as controller_settings.rs runs them.

use emusen_client_fixture::{console_core, library, World};
use emusen_crystal_client::{frame_info, Api, Config, Open, Runtime};
use emusen_platform_client::pads::{Pads, SimPad, SimSet};
use emusen_platform_client::Platform;
use emusen_qt_endymion::game::{GamePads, GameReading, KeyUse, PadSettings};
use emusen_qt_endymion::keys::{heard, Heard};
use emusen_qt_models::bindings::File;
use emusen_qt_models::input::GameInput;
use emusen_qt_models::settings_source::ClientSettings;
use std::sync::Arc;

const PLAIN: PadSettings = PadSettings { first_controller_only: false, stick_as_dpad: true, stick_deadzone: Some(0.5), mirror: false, keyboard_player: Some(1) };

struct Rig {
    rt: Runtime,
    pads: Pads,
    hand: Arc<GamePads>,
    reading: GameReading,
    input: GameInput,
    session: u32,
}

impl Rig {
    fn open(world: &World, platform: &Arc<Platform>) -> Rig {
        Rig::open_with(world, platform, &SimSet::new(platform).unwrap(), None)
    }

    /// The console started empty, or a game of these bytes opened on it, the pads of `set` read.
    fn open_with(world: &World, platform: &Arc<Platform>, set: &SimSet, payload: Option<&[u8]>) -> Rig {
        let player = world.dir("Cores");
        std::fs::copy(console_core(), player.join(console_core().file_name().unwrap())).unwrap();
        let rt = Runtime::new(&Api::load(library()).unwrap(), &Config { development: false, cores: Some(Vec::new()), player_cores: Some(vec![player.to_str().unwrap().into()]) })
            .unwrap();
        let sha = rt.cores().unwrap().into_iter().find(|c| c.directory.as_deref() == Some("player")).and_then(|c| c.sha256).expect("the console core");
        rt.core_approve(&sha).unwrap();
        let pads = Pads::simulated(platform, set).unwrap();
        let opened = pads.start().unwrap();
        let game = payload.map(|bytes| world.game("game.tst", bytes).to_str().unwrap().to_owned());
        let open = match &game {
            Some(path) => Open { path: Some(path.clone()), ..Open::default() },
            None => Open { console: Some("PlayStation".into()), ..Open::default() },
        };
        let session = rt.open(&open).unwrap();
        let hand = Arc::new(GamePads::default());
        hand.keep_seats(&pads, &opened).unwrap();
        let mut input = GameInput::new(hand.clone(), platform).unwrap();
        input.start(&rt, session, game.as_deref().unwrap_or("")).unwrap();
        input.set_owned(true);
        Rig { rt, pads, hand, reading: GameReading::default(), input, session }
    }

    /// Frames as the program runs them, the reader's tick then the frame thread's; the picture's blue after the last.
    fn run(&mut self, frames: usize) -> u8 {
        self.run_both(frames).0
    }

    /// Frames so; the picture's blue and green after the last: what ports 0 and 1 hear.
    fn run_both(&mut self, frames: usize) -> (u8, u8) {
        let mut info = frame_info();
        for _ in 0..frames {
            self.hand.apply_settings(&self.pads).unwrap();
            let events = self.pads.poll().unwrap();
            self.pads.update().unwrap();
            self.hand.keep_seats(&self.pads, &events).unwrap();
            self.reading.read(&self.pads, &self.hand).unwrap();
            self.input.frame(&self.rt, self.session).unwrap();
            self.rt.frame(self.session, &mut info).unwrap();
        }
        let mut rgba = vec![0u8; info.width as usize * info.height as usize * 4];
        self.rt.picture_copy(self.session, &mut rgba).unwrap();
        (rgba[2], rgba[1])
    }

    /// A key as the window hears it, by Qt's number, pressed or let go over the game.
    fn key(&self, qt_key: i32, pressed: bool) -> KeyUse {
        let Heard::Key(word) = heard(qt_key, 0) else { panic!("{qt_key:#x} has no word") };
        self.hand.key(word, pressed)
    }
}

/// Qt's numbers for N and M.
const N: i32 = 0x4e;
const M: i32 = 0x4d;

#[test]
fn qt_7_7_h6_a_key_bound_on_the_keyboard_keys_page_presses_its_button_in_the_game() {
    let world = World::new("keyboard-game");
    let platform = Platform::load(library()).unwrap();
    let source = ClientSettings::open(&library().display().to_string()).unwrap();
    // The page's model: Start on N and B on M, saved as the page saves a changed map.
    let mut bindings = source.bindings(&["PlayStation"]).unwrap();
    let mut touched = bindings.rebind_key("PlayStation", "Start", "N");
    touched.extend(bindings.rebind_key("PlayStation", "B", "M"));
    assert!(touched.contains(&File::Keys));
    source.save_bindings(&bindings, &touched).unwrap();

    let mut rig = Rig::open(&world, &platform);
    rig.hand.settle(PLAIN);
    let rest = rig.run(3);
    assert_eq!(rest, 0, "nothing held");
    assert_eq!(rig.key(N, true), KeyUse::Control);
    let start = rig.run(2);
    assert_ne!(start, 0, "N held reaches the console as a button");
    assert_eq!(rig.key(N, false), KeyUse::Control);
    assert_eq!(rig.run(2), 0, "N let go");
    rig.key(M, true);
    let b = rig.run(2);
    rig.key(M, false);
    assert!(b != 0 && b != start, "M reaches it as another button: blue {b} against Start's {start}");
    assert_eq!(rig.run(2), 0);

    // The keyboard as player 2: the console's one port, player 1's, hears nothing of it.
    rig.hand.settle(PadSettings { keyboard_player: Some(2), ..PLAIN });
    rig.run(1);
    rig.key(N, true);
    assert_eq!(rig.run(2), 0, "player 2's keys are not player 1's");
    rig.key(N, false);
    rig.hand.settle(PLAIN);
    rig.run(1);
    rig.key(N, true);
    assert_eq!(rig.run(2), start, "player 1's again");
}

/// The console core's picture for Left held (its `PRESSED_BLUES`) and for B (South) held.
const LEFT_BLUE: u8 = 0xC0;
const B_BLUE: u8 = 0xFF;
/// The test core's image byte that gives its port 1 a controller (its `TWO_PADS`).
const TWO_PADS: u8 = 0x22;
/// SDL's South button and left stick's x axis.
const SOUTH: i32 = 0;
const LEFT_X: i32 = 0;

/// The console core with one simulated pad connected; the set and the pad live as long as the test holds them.
fn with_pad(name: &str, payload: Option<&[u8]>) -> (World, Rig, SimSet, SimPad) {
    let world = World::new(name);
    let platform = Platform::load(library()).unwrap();
    let set = SimSet::new(&platform).unwrap();
    let pad = SimPad::new(&platform, None).unwrap();
    set.connect(&pad).unwrap();
    let rig = Rig::open_with(&world, &platform, &set, payload);
    (world, rig, set, pad)
}

#[test]
fn qt_7_7_h3_the_left_stick_presses_the_d_pad_only_while_the_setting_says_so() {
    let (_world, mut rig, _set, pad) = with_pad("stick-dpad", None);
    rig.hand.settle(PLAIN);
    assert_eq!(rig.run(3), 0, "nothing held");
    pad.set_axis(LEFT_X, -1.0).unwrap();
    assert_eq!(rig.run(3), LEFT_BLUE, "the stick pushed left is the d-pad's Left");
    pad.set_axis(LEFT_X, 0.0).unwrap();
    assert_eq!(rig.run(3), 0);
    rig.hand.settle(PadSettings { stick_as_dpad: false, ..PLAIN });
    rig.run(1);
    pad.set_axis(LEFT_X, -1.0).unwrap();
    assert_eq!(rig.run(3), 0, "the setting off: the stick is no d-pad");
    pad.set_axis(LEFT_X, 0.0).unwrap();
}

#[test]
fn qt_7_7_h3_mirror_gives_port_1_player_1_s_buttons_only_while_on() {
    let (_world, mut rig, _set, pad) = with_pad("mirror-buttons", Some(&[TWO_PADS, 1, 2, 3]));
    rig.hand.settle(PLAIN);
    rig.run(3);
    pad.press(SOUTH, true).unwrap();
    assert_eq!(rig.run_both(3), (B_BLUE, 0), "player 1's B reaches port 0 alone");
    pad.press(SOUTH, false).unwrap();
    rig.run(2);
    // Mirrored while a menu holds the pads, as Settings' row turns it with the game suspended.
    rig.input.set_owned(false);
    rig.run(2);
    rig.hand.settle(PadSettings { mirror: true, ..PLAIN });
    rig.run(2);
    rig.input.set_owned(true);
    rig.run(1);
    pad.press(SOUTH, true).unwrap();
    assert_eq!(rig.run_both(3), (B_BLUE, B_BLUE), "mirrored: port 1 hears player 1's B too");
    pad.press(SOUTH, false).unwrap();
    assert_eq!(rig.run_both(3), (0, 0));
}
