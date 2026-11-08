//! The game's side of the pads (EmuSen_Kit.md §7.17): the one reader of the pads also reads them as a running game hears
//! them, through each player's bindings, and hands each read to the game's frame thread, which routes it to the ports.
//! No Qt and no thread here: the reader's thread calls `GameReading::read`, the frame thread calls `GamePads::take`.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use emusen_platform_client::pads::{consts, Pads};
use emusen_platform_client::Result;

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
    seats: Vec<u64>,
    read: Read,
}

impl GameReading {
    /// One tick: the pads as the game hears them handed over, when a game wants them. Players are seated in the order the
    /// pads were opened, player 1 alone where the first controller only is read.
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
        let open = pads.open()?;
        let first_only = pads.setting(consts::SETTING_FIRST_CONTROLLER_ONLY)? != 0.0;
        let seated = open.len().min(if first_only { 1 } else { PLAYERS });
        let swap = swapped && seated >= 2;
        let seat_of = |player: usize| open[if swap && player < 2 { 1 - player } else { player }];
        if self.seats.len() != seated || (0..seated).any(|p| self.seats[p] != seat_of(p)) {
            self.seats.clear();
            self.seats.extend((0..seated).map(seat_of));
            for (player, &key) in self.seats.iter().enumerate() {
                pads.set_player_index(key, player as i32)?;
            }
        }
        let players = want.ports.min(PLAYERS);
        let read = &mut self.read;
        read.players = players;
        read.held = [0; PLAYERS];
        read.axes.clear();
        read.axes.resize(players * want.axes.len(), 0.0);
        read.seated = 0;
        for (player, &key) in self.seats.iter().enumerate().take(players) {
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
