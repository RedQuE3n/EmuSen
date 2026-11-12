//! Keyboard keys heard by a game (EmuSen_Qt.md §7.7.H6): a key bound through the bindings model Settings' Keyboard keys
//! page changes and saved as that page saves it, a key pressed as the window names it, and the client fixture's console
//! core, a PlayStation, drawing the button it hears in its picture's blue; the keyboard's player as the Players page sets
//! it. No window and no GPU: the reader's tick and the frame thread's work are run in turn, as controller_settings.rs runs
//! them.

use emusen_client_fixture::{console_core, library, World};
use emusen_crystal_client::{frame_info, Api, Config, Open, Runtime};
use emusen_platform_client::pads::{Pads, SimSet};
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
        let player = world.dir("Cores");
        std::fs::copy(console_core(), player.join(console_core().file_name().unwrap())).unwrap();
        let rt = Runtime::new(&Api::load(library()).unwrap(), &Config { development: false, cores: Some(Vec::new()), player_cores: Some(vec![player.to_str().unwrap().into()]) })
            .unwrap();
        let sha = rt.cores().unwrap().into_iter().find(|c| c.directory.as_deref() == Some("player")).and_then(|c| c.sha256).expect("the console core");
        rt.core_approve(&sha).unwrap();
        let set = SimSet::new(platform).unwrap();
        let pads = Pads::simulated(platform, &set).unwrap();
        let opened = pads.start().unwrap();
        let session = rt.open(&Open { console: Some("PlayStation".into()), ..Open::default() }).unwrap();
        let hand = Arc::new(GamePads::default());
        hand.keep_seats(&pads, &opened).unwrap();
        let mut input = GameInput::new(hand.clone(), platform).unwrap();
        input.start(&rt, session, "").unwrap();
        input.set_owned(true);
        Rig { rt, pads, hand, reading: GameReading::default(), input, session }
    }

    /// Frames as the program runs them, the reader's tick then the frame thread's; the picture's blue after the last.
    fn run(&mut self, frames: usize) -> u8 {
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
        rgba[2]
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
