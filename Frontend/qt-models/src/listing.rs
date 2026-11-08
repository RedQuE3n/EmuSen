//! A section's games as the grid shows them, sorted, narrowed to the favourites and searched (EmuSen_Interface.md §18.1;
//! Mistress's `GamesScreen`): the rules alone, as plain data, so GameModel applies them off the GUI thread and the List
//! card reads the same words. No Qt here.

use crate::library::{GameRow, FAVOURITES, RECENT};
use std::cmp::Reverse;

/// The sorts the List card's Sort steps through, in order.
pub const SORTS: [&str; 5] = ["Name", "Last played", "Most played", "Year", "Rating"];

/// How a section is listed: the sort by its index in `SORTS`, whether the Filter page's Favourites narrows it, and the
/// search's words.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Listing {
    pub sort: usize,
    pub favourites_only: bool,
    pub search: String,
}

impl Listing {
    /// Whether the count says how many of the section are shown.
    pub fn narrowed(&self) -> bool {
        self.favourites_only || !self.search.trim().is_empty()
    }
}

/// The sort `by` steps from `sort`, wrapping both ways.
pub fn step_sort(sort: usize, by: i32) -> usize {
    (sort as i32 + by).rem_euclid(SORTS.len() as i32) as usize
}

/// The section's games as shown: the favourites where narrowed, those whose title holds the search's words in any case,
/// then sorted; the recently played keep their own order under the name sort, and every other sort breaks a tie by
/// name. The library reads no play counts, years or ratings yet, so those sorts are the name's (EmuSen_Qt.md §7.1).
pub fn shown(games: &[GameRow], section: &str, listing: &Listing) -> Vec<GameRow> {
    let words = listing.search.trim().to_lowercase();
    let favourites_only = listing.favourites_only && section != FAVOURITES;
    let mut out: Vec<GameRow> =
        games.iter().filter(|g| (!favourites_only || g.favourite) && (words.is_empty() || g.title.to_lowercase().contains(&words))).cloned().collect();
    if section == RECENT && listing.sort == 0 {
        return out;
    }
    out.sort_by_cached_key(|g| g.title.to_lowercase());
    if SORTS[listing.sort.min(SORTS.len() - 1)] == "Last played" {
        out.sort_by_key(|g| Reverse(g.played_at));
    }
    out
}

/// The count beside the section's name: its games, or how many of them are shown where narrowed ("12 of 1,284 games").
pub fn count_line(all: usize, shown: usize, narrowed: bool) -> String {
    let games = crate::words::games(all);
    if narrowed { format!("{} of {games}", crate::words::grouped(shown)) } else { games }
}

/// A title's letter, as Jump to reads it: its first letter in capitals, else `#`.
pub fn letter(title: &str) -> String {
    match title.chars().next() {
        Some(c) if c.is_alphabetic() => c.to_uppercase().collect(),
        _ => "#".to_owned(),
    }
}

/// Each first letter of the shown games, in order, once, with the row of its first game: read off the GUI thread, so a
/// jump is a step through at most a few dozen letters.
pub fn letter_rows(shown: &[GameRow]) -> Vec<(String, usize)> {
    let mut out: Vec<(String, usize)> = Vec::new();
    for (row, g) in shown.iter().enumerate() {
        let l = letter(&g.title);
        if !out.iter().any(|(seen, _)| *seen == l) {
            out.push((l, row));
        }
    }
    out
}

/// The row of the first game under the letter a step of Jump to reaches from the letter `here` (the first letter's,
/// where `here` is none of them).
pub fn jump_letter(letters: &[(String, usize)], here: &str, by: i32) -> Option<usize> {
    if letters.is_empty() {
        return None;
    }
    let at = letters.iter().position(|(l, _)| l == here).unwrap_or(0) as i32;
    Some(letters[(at + by).rem_euclid(letters.len() as i32) as usize].1)
}
