//! Which pad is which player (EmuSen_Input.md §8.9): the eight seats the program holds, a pad seated as a start or a
//! poll opens it, kept for it when it goes, moved and let go of as the Players page asks, each by Endymion's rules
//! (`emusen_endymion_slots_*`). No Qt and no thread: the pads' reader keeps the seats, and a page reads them through
//! the hand-over.

use emusen_platform_client::pads::{consts, PadEvent, Pads};
use emusen_platform_client::seats::{Holder, Pad, Seat, PLAYERS};
use emusen_platform_client::{Platform, Result};

/// A connected pad as the Players page lists it: its key, name, the player it plays as (0 for none), whether a button
/// is held on it, and what it holds as SDL reads it: a bit for each of SDL's gamepad buttons, and its six axes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Connected {
    pub key: u64,
    pub name: String,
    pub player: usize,
    pub pressed: bool,
    pub buttons: u32,
    pub axes: [f64; AXES],
}

/// SDL's gamepad buttons a pad reports, SOUTH to MISC6 (SDL_gamepad.h, SDL 3.4).
pub const SDL_BUTTONS: i32 = 26;
/// SDL's gamepad axes: the sticks' x and y, then the triggers.
pub const AXES: usize = 6;

/// A seat as a page shows it: the player, the pad's name, and whether the pad has gone and the seat is kept for it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Player {
    pub player: usize,
    pub pad: Option<String>,
    pub kept: bool,
}

/// A pad that came or went at a poll: its name, the player it took or whose seat is kept for it (0 for none).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Came {
    pub name: String,
    pub player: usize,
    pub connected: bool,
}

/// What a page asks of the seats, done by the reader on its next tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ask {
    /// The pad of this key plays as this player (0 for none), trading with whoever held it.
    Move { key: u64, player: usize },
    /// The seat kept for a pad that went is let go of.
    Forget { player: usize },
}

/// The seats and the pads in them: each seat's pad by GUID and path, its key while connected, and its name.
#[derive(Clone, Debug, Default)]
pub struct Seats {
    seats: [Seat; PLAYERS],
    keys: [Option<u64>; PLAYERS],
    names: [String; PLAYERS],
    /// Every pad connected, in the order it was opened, with its name and how a seat knows it.
    open: Vec<(u64, String, Pad)>,
}

fn pad_of(pads: &Pads, key: u64) -> Result<(Pad, String)> {
    let guid = pads.text(key, consts::GUID)?.unwrap_or_default();
    let path = pads.text(key, consts::PATH)?;
    let name = pads.text(key, consts::NAME)?.unwrap_or_default();
    Ok((Pad { guid, path }, name))
}

impl Seats {
    /// The pads a start or a poll opened and closed: an opened pad takes its seat by Endymion's rules, a closed one's seat
    /// is kept for it; each told in `came`.
    pub fn apply(&mut self, pads: &Pads, events: &[PadEvent], came: &mut Vec<Came>) -> Result<bool> {
        let mut changed = false;
        for event in events {
            if event.opened {
                if self.open.iter().any(|(k, _, _)| *k == event.key) {
                    continue;
                }
                let (pad, name) = pad_of(pads, event.key)?;
                self.open.push((event.key, name.clone(), pad.clone()));
                let player = pads.platform().seat_for(&self.seats, &pad)?;
                if player > 0 {
                    self.seats[player - 1] = Seat::Open(pad);
                    self.keys[player - 1] = Some(event.key);
                    self.names[player - 1] = name.clone();
                }
                came.push(Came { name, player, connected: true });
            } else {
                let name = self.open.iter().find(|(k, _, _)| *k == event.key).map(|(_, n, _)| n.clone()).unwrap_or_default();
                self.open.retain(|(k, _, _)| *k != event.key);
                let seat = self.keys.iter().position(|k| *k == Some(event.key));
                if let Some(i) = seat {
                    if let Seat::Open(pad) = std::mem::take(&mut self.seats[i]) {
                        self.seats[i] = Seat::Closed(pad);
                    }
                    self.keys[i] = None;
                }
                came.push(Came { name, player: seat.map_or(0, |i| i + 1), connected: false });
            }
            changed = true;
        }
        Ok(changed)
    }

    /// A page's ask done; whether the seats changed.
    pub fn ask(&mut self, platform: &Platform, ask: Ask) -> Result<bool> {
        match ask {
            Ask::Forget { player } => {
                if player == 0 || player > PLAYERS || !platform.seat_forget(&self.seats, player)? {
                    return Ok(false);
                }
                self.seats[player - 1] = Seat::Empty;
                self.names[player - 1].clear();
                Ok(true)
            }
            Ask::Move { key, player } => {
                let Some((_, name, pad)) = self.open.iter().find(|(k, _, _)| *k == key).cloned() else { return Ok(false) };
                let from = self.keys.iter().position(|k| *k == Some(key)).map_or(0, |i| i + 1);
                let Some(moved) = platform.seat_move(&self.seats, true, from, player)? else { return Ok(false) };
                let (old_seats, old_keys, old_names) = (self.seats.clone(), self.keys, self.names.clone());
                for (i, holder) in moved.iter().enumerate() {
                    (self.seats[i], self.keys[i], self.names[i]) = match holder {
                        Holder::Empty => (Seat::Empty, None, String::new()),
                        Holder::Seat(j) => (old_seats[*j].clone(), old_keys[*j], old_names[*j].clone()),
                        Holder::Mover => (Seat::Open(pad.clone()), Some(key), name.clone()),
                    };
                }
                Ok(true)
            }
        }
    }

    /// The pad of each player that is connected, player 1 first.
    pub fn keys(&self) -> [Option<u64>; PLAYERS] {
        self.keys
    }

    /// The seats as a page shows them, up to the highest seat a pad holds or is kept for.
    pub fn players(&self) -> Vec<Player> {
        let highest = self.seats.iter().rposition(|s| *s != Seat::Empty).map_or(0, |i| i + 1);
        (0..highest)
            .map(|i| Player {
                player: i + 1,
                pad: (self.seats[i] != Seat::Empty).then(|| self.names[i].clone()),
                kept: matches!(self.seats[i], Seat::Closed(_)),
            })
            .collect()
    }

    /// Every pad connected, in the order it was opened, with its player; `pressed` from the pads read now.
    pub fn connected(&self, pads: Option<&Pads>) -> Vec<Connected> {
        self.open
            .iter()
            .map(|(key, name, _)| {
                let buttons = pads.map_or(0, |p| (0..SDL_BUTTONS).filter(|b| p.get(*key, consts::RAW_PRESSED, *b).unwrap_or(0) != 0).fold(0, |m, b| m | 1 << b));
                // To a hundredth, so a stick's noise at rest is no change.
                let axes: [f64; AXES] = std::array::from_fn(|a| pads.map_or(0.0, |p| (p.axis(*key, consts::AXIS_RAW, a as u32).unwrap_or(0.0) * 100.0).round() / 100.0));
                let triggers = axes[4..].iter().any(|t| *t > emusen_kit_core::input::TRIGGER_THRESHOLD);
                Connected {
                    key: *key,
                    name: name.clone(),
                    player: self.keys.iter().position(|k| *k == Some(*key)).map_or(0, |i| i + 1),
                    pressed: buttons != 0 || triggers,
                    buttons,
                    axes,
                }
            })
            .collect()
    }
}
