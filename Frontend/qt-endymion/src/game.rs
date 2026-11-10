//! The game's side of the pads (EmuSen_Kit.md §7.17): the one reader of the pads also reads them as a running game hears
//! them, through each player's bindings, and hands each read to the game's frame thread, which routes it to the ports.
//! No Qt and no thread here: the reader's thread calls `GameReading::read`, the frame thread calls `GamePads::take`.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use emusen_platform_client::pads::{consts, PadEvent, Pads};
use emusen_platform_client::Result;

use crate::seats::{Ask, Connected, Player, Seats};

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

/// One read as the game hears it: a mask by PadButton and the read axes for each player read, and the players seated.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Read {
    pub serial: u64,
    pub players: usize,
    pub held: [u16; PLAYERS],
    pub axes: Vec<f64>,
    pub seated: u32,
}

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
        state.read = Read { serial: state.read.serial, ..Read::default() };
    }

    /// The game's buttons bound again from the binding files on the next tick, as after a profile or a binding changed.
    pub fn rebind(&self) {
        let mut state = self.state();
        if state.want.is_some() {
            state.want_serial += 1;
        }
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

    /// The reader's tick for the seats: the pads opened and closed seated, the asks done, the page's view brought up to
    /// date; whether the view changed.
    pub fn keep_seats(&self, pads: &Pads, events: &[PadEvent]) -> Result<bool> {
        let mut state = self.state();
        state.seats.apply(pads, events)?;
        for ask in std::mem::take(&mut state.asks) {
            state.seats.ask(pads.platform(), ask)?;
        }
        let (players, connected) = (state.seats.players(), state.seats.connected(Some(pads)));
        let changed = players != state.players || connected != state.connected;
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
}

impl GameReading {
    /// One tick: the pads as the game hears them handed over, when a game wants them. Players are the seats' (EmuSen_Input.md
    /// §8.9), player 1 alone where the first controller only is read.
    pub fn read(&mut self, pads: &Pads, game: &GamePads) -> Result<()> {
        let (serial, want, swapped) = {
            let state = game.state();
            (state.want_serial, (state.want_serial != self.want_serial).then(|| state.want.clone()), state.swapped)
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
        let read = &mut self.read;
        read.players = players;
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
