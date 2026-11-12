//! The game's side of the pads (EmuSen_Kit.md §7.17): the one reader of the pads also reads them as a running game hears
//! them, through each player's bindings, and hands each read to the game's frame thread, which routes it to the ports.
//! No Qt and no thread here: the reader's thread calls `GameReading::read`, the frame thread calls `GamePads::take`.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use emusen_platform_client::pads::{consts, PadEvent, Pads};
use emusen_platform_client::Result;

use crate::notices;
use crate::seats::{Ask, Came, Connected, Player, Seats};

/// The most players a game hears (Endymion's seats).
pub const PLAYERS: usize = 8;
/// PadButton's count, the RetroPad's sixteen.
const BUTTONS: u32 = 16;
/// PadAxis's LeftX: a game that reads it hears the left stick as a stick, not the d-pad.
const LEFT_X: u32 = 0;

/// What a running game asks of the pads: its console and game, for the bindings; its ports; the axes it reads, by PadAxis.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Want {
    pub console: String,
    pub game: Option<String>,
    pub ports: usize,
    pub axes: Vec<u32>,
}

/// One read as the game hears it: a mask by PadButton and the read axes for each player read, the players seated, and
/// whether player 1 also plays as player 2 (Mirror player 1 to player 2, the router's to apply).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Read {
    pub serial: u64,
    pub players: usize,
    pub held: [u16; PLAYERS],
    pub axes: Vec<f64>,
    pub seated: u32,
    pub mirror: bool,
}

/// The player's Controllers settings the pads act on (EmuSen_Qt.md §7.7.H3): the first three Endymion's, set on the pads
/// by the reader; the mirror the game's router's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PadSettings {
    pub first_controller_only: bool,
    pub stick_as_dpad: bool,
    /// None leaves Endymion's as it is, where the stored value could not be read.
    pub stick_deadzone: Option<f64>,
    pub mirror: bool,
}

/// A port's two motor levels, strong then weak, 0 to 65535.
pub type Motors = (u16, u16);

#[derive(Default)]
struct State {
    want: Option<Want>,
    want_serial: u64,
    owned: bool,
    quiet: bool,
    /// Players 1 and 2 swapped, for as long as the program runs (Mistress's `SwapPlayers`).
    swapped: bool,
    /// The game hears nothing while the interface holds a button of its own (Guide, for its chords).
    muted: bool,
    read: Read,
    /// The seats, kept by the reader; a page's asks waiting for its next tick; the seats as a page reads them.
    seats: Seats,
    asks: Vec<Ask>,
    players: Vec<Player>,
    connected: Vec<Connected>,
    stick_deadzone: f64,
    /// Settings waiting for the reader's next tick; the mirror as last settled.
    settings: Option<PadSettings>,
    mirror: bool,
    /// Each port's motors as the game last drove them.
    motors: [Motors; PLAYERS],
    /// The pads that came and went, in the notices' words, until the reader takes them.
    notices: Vec<String>,
}

/// The hand-over between the pads' reader and a game: what the game wants, whether it hears the pads, the newest read.
#[derive(Default)]
pub struct GamePads {
    state: Mutex<State>,
}

impl GamePads {
    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// A game opened: its wants, read from the next tick; none when it closes.
    pub fn want(&self, want: Option<Want>) {
        let mut state = self.state();
        state.want = want;
        state.want_serial += 1;
        state.motors = [(0, 0); PLAYERS];
        state.read = Read { serial: state.read.serial, ..Read::default() };
    }

    /// The game's buttons bound again from the binding files on the next tick, as after a profile or a binding changed.
    pub fn rebind(&self) {
        let mut state = self.state();
        if state.want.is_some() {
            state.want_serial += 1;
        }
    }

    /// The player's settings, set on the pads at the reader's next tick (Mistress's `ApplyKeys` and `SaveApp`).
    pub fn settle(&self, settings: PadSettings) {
        let mut state = self.state();
        state.mirror = settings.mirror;
        state.settings = Some(settings);
    }

    /// The reader's tick for the settings: any settled since the last set on Endymion's pads.
    pub fn apply_settings(&self, pads: &Pads) -> Result<()> {
        let Some(s) = self.state().settings.take() else { return Ok(()) };
        pads.set_setting(consts::SETTING_FIRST_CONTROLLER_ONLY, f64::from(u8::from(s.first_controller_only)))?;
        pads.set_setting(consts::SETTING_STICK_AS_DPAD, f64::from(u8::from(s.stick_as_dpad)))?;
        match s.stick_deadzone {
            Some(deadzone) => pads.set_setting(consts::SETTING_STICK_DEADZONE, deadzone),
            None => Ok(()),
        }
    }

    /// The game's motors after a frame, port 0 first; the reader drives each player's pad by its port's (EmuSen_Input.md §9.7).
    pub fn drive(&self, motors: &[Motors]) {
        let mut state = self.state();
        state.motors = [(0, 0); PLAYERS];
        state.motors.iter_mut().zip(motors).for_each(|(m, l)| *m = *l);
    }

    /// Whether the game hears the pads (running and on screen); nothing is handed over while it does not. Given back to
    /// it, it hears nothing until every button is let go, so the press that closed a menu is not the game's (Mistress's rule).
    pub fn set_owned(&self, owned: bool) {
        let mut state = self.state();
        if owned && !state.owned {
            state.quiet = true;
        }
        state.owned = owned;
    }

    pub fn owned(&self) -> bool {
        self.state().owned
    }

    /// While muted the game hears nothing; let go, it hears nothing until every button is let go, as after a menu
    /// (Mistress's `_padQuiet` while Guide is held for its chords).
    pub fn set_muted(&self, muted: bool) {
        let mut state = self.state();
        if state.muted && !muted {
            state.quiet = true;
        }
        state.muted = muted;
    }

    /// Players 1 and 2 swap pads from the next tick; false where player 2 has none.
    pub fn swap_players(&self) -> bool {
        let mut state = self.state();
        if state.read.seated & 0b11 != 0b11 {
            return false;
        }
        state.swapped = !state.swapped;
        true
    }

    /// The notices of pads that came and went since the last taken (EmuSen_Input.md §8.13.5).
    pub fn notices(&self) -> Vec<String> {
        std::mem::take(&mut self.state().notices)
    }

    /// The players seated at the newest read, a bit each.
    pub fn seated(&self) -> u32 {
        self.state().read.seated
    }

    /// A page's ask of the seats, done on the reader's next tick.
    pub fn ask(&self, ask: Ask) {
        self.state().asks.push(ask);
    }

    /// The seats as a page shows them, player 1 first.
    pub fn players(&self) -> Vec<Player> {
        self.state().players.clone()
    }

    /// Every pad connected, with its player and whether a button is held on it.
    pub fn connected(&self) -> Vec<Connected> {
        self.state().connected.clone()
    }

    /// Endymion's stick deadzone at the newest tick: past it a stick is heard as a direction.
    pub fn stick_deadzone(&self) -> f64 {
        self.state().stick_deadzone
    }

    /// The reader's tick for the seats: the pads opened and closed seated, the asks done, the page's view brought up to
    /// date; whether the view changed.
    pub fn keep_seats(&self, pads: &Pads, events: &[PadEvent]) -> Result<bool> {
        let mut state = self.state();
        let mut came: Vec<Came> = Vec::new();
        state.seats.apply(pads, events, &mut came)?;
        let first = state.seats.keys()[0].is_some();
        state.notices.extend(came.iter().map(|c| notices::said(c, first)));
        for ask in std::mem::take(&mut state.asks) {
            state.seats.ask(pads.platform(), ask)?;
        }
        let (players, connected) = (state.seats.players(), state.seats.connected(Some(pads))?);
        let deadzone = pads.setting(consts::SETTING_STICK_DEADZONE)?;
        let changed = players != state.players || connected != state.connected || deadzone != state.stick_deadzone;
        state.stick_deadzone = deadzone;
        state.players = players;
        state.connected = connected;
        Ok(changed)
    }

    fn seat_keys(&self) -> [Option<u64>; PLAYERS] {
        self.state().seats.keys()
    }

    /// The newest read into `out` when it is newer than `out`'s; true when it was.
    pub fn take(&self, out: &mut Read) -> bool {
        let state = self.state();
        if state.read.serial == out.serial {
            return false;
        }
        out.clone_from(&state.read);
        true
    }

    /// The hand-over as a number, for a QML property (a QML element is handed no Rust value); `at` gives it back.
    pub fn address(this: &Arc<GamePads>) -> u64 {
        Arc::as_ptr(this) as u64
    }

    /// The hand-over a number names, a reference of its own.
    ///
    /// # Safety
    /// `address` is 0 or `address` of a `GamePads` still alive.
    pub unsafe fn at(address: u64) -> Option<Arc<GamePads>> {
        let ptr = (address != 0).then_some(address as *const GamePads)?;
        // SAFETY: the caller's promise; the count is raised before the new reference is made.
        unsafe {
            Arc::increment_strong_count(ptr);
            Some(Arc::from_raw(ptr))
        }
    }
}

/// What the reader keeps between ticks for the game: the wants it has bound and each player's bound buttons.
#[derive(Default)]
pub struct GameReading {
    want_serial: u64,
    want: Option<Want>,
    bound: [[i32; BUTTONS as usize]; PLAYERS],
    seats: [Option<u64>; PLAYERS],
    read: Read,
    /// The motor levels last sent each player's pad.
    sent: [Motors; PLAYERS],
}

impl GameReading {
    /// One tick: the pads as the game hears them handed over, when a game wants them. Players are the seats' (EmuSen_Input.md
    /// §8.9), player 1 alone where the first controller only is read.
    pub fn read(&mut self, pads: &Pads, game: &GamePads) -> Result<()> {
        let (serial, want, swapped, mirror, owned, motors) = {
            let state = game.state();
            (state.want_serial, (state.want_serial != self.want_serial).then(|| state.want.clone()), state.swapped, state.mirror, state.owned, state.motors)
        };
        if let Some(want) = want {
            self.bind(pads, want)?;
            self.want_serial = serial;
        }
        let Some(want) = &self.want else { return Ok(()) };
        let mut keys = game.seat_keys();
        if pads.setting(consts::SETTING_FIRST_CONTROLLER_ONLY)? != 0.0 {
            keys[1..].iter_mut().for_each(|k| *k = None);
        }
        if swapped && keys[0].is_some() && keys[1].is_some() {
            keys.swap(0, 1);
        }
        if self.seats != keys {
            for (player, key) in keys.iter().enumerate() {
                if let Some(key) = key {
                    pads.set_player_index(*key, player as i32)?;
                }
            }
            self.seats = keys;
        }
        let players = want.ports.min(PLAYERS);
        drive(pads, &self.seats, &mut self.sent, players, if owned { motors } else { [(0, 0); PLAYERS] })?;
        let read = &mut self.read;
        read.players = players;
        read.mirror = mirror;
        read.held = [0; PLAYERS];
        read.axes.clear();
        read.axes.resize(players * want.axes.len(), 0.0);
        read.seated = 0;
        for (player, key) in self.seats.iter().enumerate().take(players) {
            let Some(key) = *key else { continue };
            read.seated |= 1 << player;
            for button in 0..BUTTONS {
                if pads.pressed(key, button, self.bound[player][button as usize])? {
                    read.held[player] |= 1 << button;
                }
            }
            for (k, &axis) in want.axes.iter().enumerate() {
                read.axes[player * want.axes.len() + k] = pads.axis(key, consts::AXIS_PAD, axis)?;
            }
        }
        let mut state = game.state();
        if state.want_serial != serial || !state.owned {
            return Ok(());
        }
        if state.quiet || state.muted {
            if state.muted || read.held.iter().any(|&h| h != 0) {
                read.held = [0; PLAYERS];
                read.axes.iter_mut().for_each(|a| *a = 0.0);
            } else {
                state.quiet = false;
            }
        }
        read.serial = state.read.serial + 1;
        state.read.clone_from(read);
        Ok(())
    }

    /// A game's wants taken: each player's buttons bound by Endymion's rules, the left stick a stick where the game reads it.
    fn bind(&mut self, pads: &Pads, want: Option<Want>) -> Result<()> {
        if let Some(want) = &want {
            for (player, bound) in self.bound.iter_mut().enumerate() {
                *bound = pads.platform().bindings_bound(&want.console, player as i32 + 1, want.game.as_deref())?;
            }
            pads.set_setting(consts::SETTING_LEFT_STICK_IS_ANALOG, f64::from(u8::from(want.axes.contains(&LEFT_X))))?;
        }
        self.want = want;
        Ok(())
    }
}

/// Each seated player's pad given its port's motors: a level held sent every tick, within SDL's 100 ms, and a level gone
/// to zero sent once (Mistress's `PassRumbleLevels`).
fn drive(pads: &Pads, seats: &[Option<u64>; PLAYERS], sent: &mut [Motors; PLAYERS], players: usize, motors: [Motors; PLAYERS]) -> Result<()> {
    for (player, key) in seats.iter().enumerate() {
        let level = if player < players { motors[player] } else { (0, 0) };
        if let Some(key) = *key {
            if level != (0, 0) || sent[player] != (0, 0) {
                pads.rumble(key, level.0, level.1)?;
            }
        }
        sent[player] = level;
    }
    Ok(())
}
