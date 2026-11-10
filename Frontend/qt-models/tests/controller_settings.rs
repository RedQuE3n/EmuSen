//! The Controllers settings a running game acts on (EmuSen_Qt.md §7.7.H3), through the interface's one reader and the
//! test core: the core's motor reaching the player's simulated pad each frame, stopped once the game no longer holds the
//! pad and by the Rumble switch turned mid-game; and Mirror player 1 to player 2 giving port 1 a controller. No window
//! and no GPU.

use emusen_client_fixture::{library, World};
use emusen_crystal_client::{frame_info, Api, Config, Open, Runtime};
use emusen_platform_client::pads::{consts, Pads, SimPad, SimSet};
use emusen_platform_client::Platform;
use emusen_qt_endymion::game::{GamePads, GameReading, PadSettings};
use emusen_qt_models::input::GameInput;
use std::sync::Arc;

/// The test core's image byte that makes it report a console whose games drive a motor (its `MOTOR_CONSOLE`).
const MOTOR_CONSOLE: u8 = 0x64;
/// The test core's image byte that gives its port 1 a controller too (its `TWO_PADS`).
const TWO_PADS: u8 = 0x22;
/// The test core's RAM byte whose bit n says port n has no controller (its `ABSENT`).
const ABSENT: u32 = 5;

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

/// The simulated pad's last motor levels, strong then weak.
fn motors(pad: &SimPad) -> (u16, u16) {
    let both = pad.get(consts::SIM_PAD_RUMBLE).unwrap() as u32;
    ((both >> 16) as u16, both as u16)
}

struct Rig {
    rt: Runtime,
    pads: Pads,
    hand: Arc<GamePads>,
    reading: GameReading,
    input: GameInput,
    session: u32,
}

impl Rig {
    fn open(world: &World, platform: &Arc<Platform>, set: &SimSet, payload: &[u8]) -> Rig {
        let rt = with_test_core(world);
        let pads = Pads::simulated(platform, set).unwrap();
        let opened = pads.start().unwrap();
        let game = world.game("game.tst", payload);
        let path = game.to_str().unwrap();
        let session = rt.open(&Open { path: Some(path.into()), ..Open::default() }).unwrap();
        let hand = Arc::new(GamePads::default());
        hand.keep_seats(&pads, &opened).unwrap();
        let mut input = GameInput::new(hand.clone(), platform).unwrap();
        input.start(&rt, session, path).unwrap();
        input.set_owned(true);
        Rig { rt, pads, hand, reading: GameReading::default(), input, session }
    }

    /// Frames as the program runs them: the reader's tick (the settings, the seats, the game's read and motors), then
    /// the frame thread's.
    fn run(&mut self, frames: usize) {
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
    }

    fn ram(&self, at: u32) -> u8 {
        let mut b = [0u8];
        self.rt.session_service(self.session, 0).unwrap().space_read(self.session, "RAM", at, &mut b).unwrap();
        b[0]
    }
}

#[test]
fn qt_7_7_h3_the_core_s_motor_drives_the_player_s_pad_until_the_pad_or_the_switch_says_no() {
    let world = World::new("rumble");
    let platform = Platform::load(library()).unwrap();
    let set = SimSet::new(&platform).unwrap();
    let pad = SimPad::new(&platform, None).unwrap();
    set.connect(&pad).unwrap();
    let mut rig = Rig::open(&world, &platform, &set, &[MOTOR_CONSOLE, 1, 2, 3]);

    // The core's strong motor, frames × 997, reaches player 1's pad frame by frame.
    rig.run(3);
    let first = motors(&pad);
    assert!(first.0 != 0 && first.1 == 0, "{first:?}");
    rig.run(1);
    assert_eq!(motors(&pad).0, first.0.wrapping_add(997), "each frame's level");

    // The Pause Card takes the pad: the motor is told to stop at once.
    rig.input.set_owned(false);
    rig.run(1);
    assert_eq!(motors(&pad), (0, 0), "stopped while a menu holds the pad");
    rig.input.set_owned(true);
    rig.run(2);
    assert_ne!(motors(&pad), (0, 0), "running again with the game");

    // Rumble turned off while the game runs: the pad is told to stop; on, the motor's own again.
    rig.rt.input_rumble_switch(rig.session, false).unwrap();
    rig.run(2);
    assert_eq!(motors(&pad), (0, 0), "switched off mid-game");
    rig.rt.input_rumble_switch(rig.session, true).unwrap();
    rig.run(2);
    assert_ne!(motors(&pad), (0, 0), "switched on again");
}

#[test]
fn qt_7_7_h3_a_console_with_no_motor_never_drives_the_pad() {
    let world = World::new("rumble-none");
    let platform = Platform::load(library()).unwrap();
    let set = SimSet::new(&platform).unwrap();
    let pad = SimPad::new(&platform, None).unwrap();
    set.connect(&pad).unwrap();
    let mut rig = Rig::open(&world, &platform, &set, &[1, 2, 3, 4]);
    rig.run(5);
    assert_eq!(motors(&pad), (0, 0), "the test system's row has no motor");
}

#[test]
fn qt_7_7_h3_mirror_player_1_to_player_2_gives_port_1_a_controller() {
    let world = World::new("mirror");
    let platform = Platform::load(library()).unwrap();
    let set = SimSet::new(&platform).unwrap();
    let pad = SimPad::new(&platform, None).unwrap();
    set.connect(&pad).unwrap();
    let mut rig = Rig::open(&world, &platform, &set, &[TWO_PADS, 1, 2, 3]);
    let plain = PadSettings { first_controller_only: false, stick_as_dpad: true, stick_deadzone: 0.5, mirror: false };
    rig.run(2);
    assert_eq!(rig.ram(ABSENT) & 0b10, 0b10, "one pad: port 1 has no controller");
    rig.hand.settle(PadSettings { mirror: true, ..plain });
    rig.run(2);
    assert_eq!(rig.ram(ABSENT) & 0b10, 0, "mirrored: player 1 plays port 1 too");
    rig.hand.settle(plain);
    rig.run(2);
    assert_eq!(rig.ram(ABSENT) & 0b10, 0b10, "the mirror off again");
}
