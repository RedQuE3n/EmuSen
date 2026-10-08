//! Which pad is which player: the C# `PlayerSlots`'s rules. See EmuSen_Input.md §8.2 and §8.3.
//!
//! The seats hold whatever a host calls a pad; a rule asks only whether a seated pad is still connected, and for a
//! returning pad its identity, SDL's GUID and the device path.

pub const MAX_PLAYERS: usize = 8;

/// What a rule needs to know of a seated pad.
pub trait Pad {
    fn is_open(&self) -> bool;
    fn guid(&self) -> &str;
    fn path(&self) -> Option<&str>;
}

/// Eight seats; a closed pad in one keeps it reserved for its return.
#[derive(Clone, Debug)]
pub struct PlayerSlots<P> {
    pub seats: [Option<P>; MAX_PLAYERS],
}

/// A seat asked for that does not exist.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoSuchPlayer;

impl<P: Pad + Clone> PlayerSlots<P> {
    pub fn new(seats: [Option<P>; MAX_PLAYERS]) -> PlayerSlots<P> {
        PlayerSlots { seats }
    }

    /// The highest player with a pad seated, connected or reserved; 0 for none.
    pub fn highest(&self) -> usize {
        self.seats.iter().rposition(Option::is_some).map_or(0, |i| i + 1)
    }

    fn lowest_reserved(&self, matches: impl Fn(&P) -> bool) -> Option<usize> {
        self.seats.iter().position(|seat| seat.as_ref().is_some_and(|pad| !pad.is_open() && matches(pad)))
    }

    /// The lowest seat with no pad connected: empty, or reserved for one that has gone.
    fn lowest_free(&self) -> Option<usize> {
        self.seats.iter().position(|seat| !seat.as_ref().is_some_and(Pad::is_open))
    }

    /// A pad just connected: its own reserved seat by GUID and path, else by GUID, else the lowest seat with no pad connected. The player it became, or 0.
    pub fn seat(&mut self, pad: P) -> usize {
        let at = self
            .lowest_reserved(|seat| seat.guid() == pad.guid() && seat.path() == pad.path())
            .or_else(|| self.lowest_reserved(|seat| seat.guid() == pad.guid()))
            .or_else(|| self.lowest_free());
        match at {
            Some(at) => {
                self.seats[at] = Some(pad);
                at + 1
            }
            None => 0,
        }
    }

    /// The player chose a seat for the pad now seated as `from` (0 for none): they trade seats with whoever held it, and 0 takes the pad out. Whether anything changed.
    pub fn move_pad(&mut self, pad: P, from: usize, player: i32) -> Result<bool, NoSuchPlayer> {
        if player < 0 || player > MAX_PLAYERS as i32 {
            return Err(NoSuchPlayer);
        }
        let player = player as usize;
        if from == player {
            return Ok(false);
        }
        if player == 0 {
            self.seats[from - 1] = None;
            return Ok(true);
        }
        let occupant = self.seats[player - 1].replace(pad);
        if from > 0 {
            self.seats[from - 1] = occupant;
        } else if let Some(occupant) = occupant.filter(Pad::is_open)
            && let Some(free) = self.lowest_free()
        {
            // Asked after the pad took its seat, as C# asks it: a closed pad just seated counts as free.
            self.seats[free] = Some(occupant);
        }
        Ok(true)
    }

    /// Lets go of a reservation, so the seat is anybody's. Whether there was one.
    pub fn forget(&mut self, player: i32) -> bool {
        if !(1..=MAX_PLAYERS as i32).contains(&player) {
            return false;
        }
        let seat = &mut self.seats[player as usize - 1];
        if seat.as_ref().is_some_and(|pad| !pad.is_open()) {
            *seat = None;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct Test {
        name: &'static str,
        open: bool,
        guid: &'static str,
        path: Option<&'static str>,
    }

    impl Pad for Test {
        fn is_open(&self) -> bool {
            self.open
        }
        fn guid(&self) -> &str {
            self.guid
        }
        fn path(&self) -> Option<&str> {
            self.path
        }
    }

    fn pad(name: &'static str, guid: &'static str, path: Option<&'static str>) -> Test {
        Test { name, open: true, guid, path }
    }

    fn names(slots: &PlayerSlots<Test>) -> Vec<&'static str> {
        slots.seats.iter().map(|seat| seat.as_ref().map_or("-", |pad| pad.name)).collect()
    }

    #[test]
    fn pads_take_the_lowest_free_seat_and_a_returning_pad_finds_its_own() {
        let mut slots = PlayerSlots::new(Default::default());
        assert_eq!(slots.seat(pad("a", "g1", Some("/1"))), 1);
        assert_eq!(slots.seat(pad("b", "g1", Some("/2"))), 2);
        assert_eq!(slots.seat(pad("c", "g2", None)), 3);
        slots.seats[0].as_mut().unwrap().open = false;
        slots.seats[1].as_mut().unwrap().open = false;
        assert_eq!(slots.seat(pad("b again", "g1", Some("/2"))), 2);
        assert_eq!(slots.seat(pad("a elsewhere", "g1", Some("/9"))), 1);
        assert_eq!(slots.seat(pad("d", "g3", None)), 4);
        assert_eq!(slots.highest(), 4);
        assert_eq!(names(&slots)[..4], ["a elsewhere", "b again", "c", "d"]);
    }

    #[test]
    fn a_ninth_pad_is_no_player() {
        let mut slots = PlayerSlots::new(Default::default());
        for i in 0..8 {
            assert_eq!(slots.seat(pad("p", "g", None)), i + 1);
        }
        assert_eq!(slots.seat(pad("ninth", "g", None)), 0);
    }

    #[test]
    fn a_move_trades_seats_and_none_takes_the_pad_out() {
        let mut slots = PlayerSlots::new(Default::default());
        slots.seat(pad("a", "g", None));
        slots.seat(pad("b", "g", None));
        let a = slots.seats[0].clone().unwrap();
        assert_eq!(slots.move_pad(a.clone(), 1, 2), Ok(true));
        assert_eq!(names(&slots)[..2], ["b", "a"]);
        assert_eq!(slots.move_pad(a.clone(), 2, 2), Ok(false));
        assert_eq!(slots.move_pad(a.clone(), 2, 0), Ok(true));
        assert_eq!(names(&slots)[..2], ["b", "-"]);
        assert_eq!(slots.move_pad(a.clone(), 0, 9), Err(NoSuchPlayer));
        assert_eq!(slots.move_pad(a.clone(), 0, -1), Err(NoSuchPlayer));
        // An unseated pad taking seat 1 sends its connected occupant to the lowest free seat.
        assert_eq!(slots.move_pad(a, 0, 1), Ok(true));
        assert_eq!(names(&slots)[..2], ["a", "b"]);
    }

    #[test]
    fn a_closed_pad_moved_in_counts_as_free_for_the_occupant_it_displaced() {
        let mut slots = PlayerSlots::new(Default::default());
        slots.seat(pad("a", "g", None));
        let closed = Test { name: "gone", open: false, guid: "g", path: None };
        assert_eq!(slots.move_pad(closed, 0, 1), Ok(true));
        assert_eq!(names(&slots)[..2], ["a", "-"]);
    }

    #[test]
    fn only_a_reservation_can_be_forgotten() {
        let mut slots = PlayerSlots::new(Default::default());
        slots.seat(pad("a", "g", None));
        assert!(!slots.forget(1));
        slots.seats[0].as_mut().unwrap().open = false;
        assert!(!slots.forget(0) && !slots.forget(9));
        assert!(slots.forget(1));
        assert_eq!(slots.highest(), 0);
    }
}
