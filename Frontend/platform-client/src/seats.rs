//! Which pad is which player: Endymion's rules over the eight seats as the host holds them (emusen_platform.h,
//! `emusen_endymion_seat` and `emusen_endymion_slots_*`; EmuSen_Input.md §8.9). The host keeps the seats; each call
//! reads them and answers, changing nothing it holds.

use crate::error::Result;
use crate::platform::Platform;
use crate::raw;

/// The seats a host holds, player 1 first.
pub const PLAYERS: usize = 8;

/// A pad as a seat names it: SDL's GUID and its device path, none being no path.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pad {
    pub guid: String,
    pub path: Option<String>,
}

/// A seat: empty, or a pad connected or gone (a seat kept for it).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Seat {
    #[default]
    Empty,
    Open(Pad),
    Closed(Pad),
}

/// Who holds a seat after a move: no one, the pad that was in that seat (0 for player 1), or the pad moved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Holder {
    Empty,
    Seat(usize),
    Mover,
}

/// The seats as the header lays them, borrowing the seats' words.
fn described(seats: &[Seat; PLAYERS]) -> [raw::emusen_endymion_seat; PLAYERS] {
    std::array::from_fn(|i| {
        let (state, pad) = match &seats[i] {
            Seat::Empty => (raw::EMUSEN_ENDYMION_SEAT_EMPTY, None),
            Seat::Open(p) => (raw::EMUSEN_ENDYMION_SEAT_OPEN, Some(p)),
            Seat::Closed(p) => (raw::EMUSEN_ENDYMION_SEAT_CLOSED, Some(p)),
        };
        let (guid, guid_len) = pad.map_or((std::ptr::null(), 0), |p| (p.guid.as_ptr(), p.guid.len()));
        let (path, path_len) = crate::out::text(pad.and_then(|p| p.path.as_deref()));
        raw::emusen_endymion_seat { state, guid, guid_len, path, path_len }
    })
}

impl Platform {
    /// The player a pad just connected becomes: its own kept seat by GUID and path, else by GUID, else the lowest with
    /// no pad connected; 0 when every seat holds a connected pad.
    pub fn seat_for(&self, seats: &[Seat; PLAYERS], pad: &Pad) -> Result<usize> {
        let d = described(seats);
        let (p, pl) = crate::out::text(pad.path.as_deref());
        let n = unsafe { self.lib.emusen_endymion_slots_seat(d.as_ptr(), pad.guid.as_ptr(), pad.guid.len(), p, pl) };
        self.status(n.into()).map(|n| n as usize)
    }

    /// A pad seated as `from` (0 for none, `mover_open` whether it is connected) moved to `player` (0 for none),
    /// trading seats with whoever held it: the seats after, or none when nothing changed.
    pub fn seat_move(&self, seats: &[Seat; PLAYERS], mover_open: bool, from: usize, player: usize) -> Result<Option<[Holder; PLAYERS]>> {
        let d = described(seats);
        let mut out = [-1i32; PLAYERS];
        let n = unsafe { self.lib.emusen_endymion_slots_move(d.as_ptr(), mover_open as u32, from as i32, player as i32, out.as_mut_ptr()) };
        let changed = self.status(n.into())? == 1;
        Ok(changed.then(|| {
            out.map(|h| match h {
                h if h < 0 => Holder::Empty,
                h if h as usize >= PLAYERS => Holder::Mover,
                h => Holder::Seat(h as usize),
            })
        }))
    }

    /// Whether `player`'s seat is kept for a pad that has gone, which the host then lets go of.
    pub fn seat_forget(&self, seats: &[Seat; PLAYERS], player: usize) -> Result<bool> {
        let d = described(seats);
        self.status(unsafe { self.lib.emusen_endymion_slots_forget(d.as_ptr(), player as i32) }.into()).map(|n| n == 1)
    }

    /// The highest player with a pad seated, connected or kept; 0 for none.
    pub fn seat_highest(&self, seats: &[Seat; PLAYERS]) -> Result<usize> {
        let d = described(seats);
        self.status(unsafe { self.lib.emusen_endymion_slots_highest(d.as_ptr()) }.into()).map(|n| n as usize)
    }
}
