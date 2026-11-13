//! The words of the notice a pad's coming or going raises (EmuSen_Input.md §8.13.5), as Mistress's `ConnectedNotice` and
//! `DisconnectedNotice` say them; the program raises them where Controller notices is on.

use crate::seats::Came;

/// A pad's notice; `first_seated` whether player 1 has a pad connected once it came.
pub fn said(came: &Came, first_seated: bool) -> String {
    let Came { name, player, connected } = came;
    match (connected, player) {
        (true, 0) => format!("{name}: no player"),
        (true, 1) => format!("{name}: player 1"),
        (true, n) if !first_seated => {
            format!("{name}: player {n}, and player 1 has no controller. Swap Players under Game Settings makes it player 1.")
        }
        (true, n) => format!("{name}: player {n}"),
        (false, 0) => format!("{name} disconnected"),
        (false, n) => format!("{name} disconnected: player {n} is kept for it"),
    }
}
