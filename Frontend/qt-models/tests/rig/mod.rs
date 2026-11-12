//! The console core run as the program runs a game, with no window and no GPU: the reader's tick and the frame
//! thread's work in turn, the picture's blue (port 0) and green (port 1) read after them. Shared by keyboard.rs and
//! reach.rs.
#![allow(dead_code)]

use emusen_client_fixture::{console_core, library, World};
use emusen_crystal_client::{frame_info, Api, Config, Open, Runtime};
use emusen_platform_client::pads::{Pads, SimPad, SimSet};
use emusen_platform_client::Platform;
use emusen_qt_endymion::game::{GamePads, GameReading, KeyUse, PadSettings};
use emusen_qt_endymion::keys::{heard, Heard};
use emusen_qt_models::input::GameInput;
use std::sync::Arc;

pub const PLAIN: PadSettings = PadSettings { first_controller_only: false, stick_as_dpad: true, stick_deadzone: Some(0.5), mirror: false, keyboard_player: Some(1) };

pub struct Rig {
    pub rt: Runtime,
    pub pads: Pads,
    pub hand: Arc<GamePads>,
    pub reading: GameReading,
    pub input: GameInput,
    pub session: u32,
}

impl Rig {
    pub fn open(world: &World, platform: &Arc<Platform>) -> Rig {
        Rig::open_with(world, platform, &SimSet::new(platform).unwrap(), None)
    }

    /// The console started empty, or a game of these bytes opened on it, the pads of `set` read.
    pub fn open_with(world: &World, platform: &Arc<Platform>, set: &SimSet, payload: Option<&[u8]>) -> Rig {
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
    pub fn run(&mut self, frames: usize) -> u8 {
        self.run_both(frames).0
    }

    /// Frames so; the picture's blue and green after the last: what ports 0 and 1 hear.
    pub fn run_both(&mut self, frames: usize) -> (u8, u8) {
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
    pub fn key(&self, qt_key: i32, pressed: bool) -> KeyUse {
        let Heard::Key(word) = heard(qt_key, 0) else { panic!("{qt_key:#x} has no word") };
        self.hand.key(word, pressed)
    }
}

/// The console core's picture for Left held (its `PRESSED_BLUES`) and for B (South) held.
pub const LEFT_BLUE: u8 = 0xC0;
pub const B_BLUE: u8 = 0xFF;
/// The test core's image byte that gives its port 1 a controller (its `TWO_PADS`).
pub const TWO_PADS: u8 = 0x22;
/// SDL's South button and left stick's x axis, by Endymion's reading's numbers.
pub const SOUTH: i32 = 0;
pub const LEFT_X: i32 = emusen_qt_endymion::reading::sdl::LEFT_X as i32;

/// The console core with one simulated pad connected; the set and the pad live as long as the test holds them.
pub fn with_pad(name: &str, payload: Option<&[u8]>) -> (World, Rig, SimSet, SimPad) {
    let world = World::new(name);
    let platform = Platform::load(library()).unwrap();
    let set = SimSet::new(&platform).unwrap();
    let pad = SimPad::new(&platform, None).unwrap();
    set.connect(&pad).unwrap();
    let rig = Rig::open_with(&world, &platform, &set, payload);
    (world, rig, set, pad)
}

