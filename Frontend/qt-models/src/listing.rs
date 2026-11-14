//! A section's games as the grid shows them, sorted, narrowed to the favourites and searched (EmuSen_Interface.md §18.1;
//! Mistress's `GamesScreen`): the rules alone, as plain data, so GameModel applies them off the GUI thread and the List
//! card reads the same words. No Qt here.

use crate::library::{GameRow, FAVOURITES, RECENT};
use std::cmp::Reverse;

/// A sort of the grid, which the List card's Sort steps through.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sort {
    Name,
    LastPlayed,
    MostPlayed,
    Year,
    Rating,
}

impl Sort {
    /// The sorts in the order Sort steps through them; a listing names one by its index here.
    pub const ALL: [Sort; 5] = [Sort::Name, Sort::LastPlayed, Sort::MostPlayed, Sort::Year, Sort::Rating];

    /// The sort at `index` in `ALL`, the last for an index past it.
    pub fn at(index: usize) -> Sort {
        Sort::ALL[index.min(Sort::ALL.len() - 1)]
    }

    /// The words the List card shows for it.
    pub fn words(self) -> &'static str {
        match self {
            Sort::Name => "Name",
            Sort::LastPlayed => "Last played",
            Sort::MostPlayed => "Most played",
            Sort::Year => "Year",
            Sort::Rating => "Rating",
        }
    }
}

/// How a section is listed: the sort by its index in `Sort::ALL`, whether the Filter page's Favourites narrows it, the
/// search's words, and the members of the collection the Filter page narrows it to.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Listing {
    pub sort: usize,
    pub favourites_only: bool,
    pub search: String,
    pub collection: Option<std::collections::BTreeSet<String>>,
}

impl Listing {
    /// Whether the count says how many of the section are shown.
    pub fn narrowed(&self) -> bool {
        self.favourites_only || self.collection.is_some() || !self.search.trim().is_empty()
    }
}

/// Whether the sort orders by the scraper's facts, which the library reads only while one does.
pub fn by_facts(sort: usize) -> bool {
    matches!(Sort::at(sort), Sort::Year | Sort::Rating)
}

/// The sort `by` steps from `sort`, wrapping both ways.
pub fn step_sort(sort: usize, by: i32) -> usize {
    (sort as i32 + by).rem_euclid(Sort::ALL.len() as i32) as usize
}

/// The section's games as shown: the favourites where narrowed, those whose title holds the search's words in any case,
/// then sorted; the recently played keep their own order under the name sort, and every other sort breaks a tie by
/// name, a game without the sort's datum after those with it (Mistress's `GamesScreen.Sorted`; EmuSen_Qt.md §7.9).
pub fn shown(games: &[GameRow], section: &str, listing: &Listing) -> Vec<GameRow> {
    let words = listing.search.trim().to_lowercase();
    let favourites_only = listing.favourites_only && section != FAVOURITES;
    let mut out: Vec<GameRow> =
        games.iter().filter(|g| (!favourites_only || g.favourite) && listing.collection.as_ref().is_none_or(|c| c.contains(&g.path)) && (words.is_empty() || g.title.to_lowercase().contains(&words))).cloned().collect();
    if section == RECENT && listing.sort == 0 {
        return out;
    }
    out.sort_by_cached_key(|g| g.title.to_lowercase());
    // Each sort is stable over the name's order, so a tie keeps it.
    match Sort::at(listing.sort) {
        Sort::LastPlayed => out.sort_by_key(|g| Reverse(g.played_at)),
        Sort::MostPlayed => out.sort_by_key(|g| Reverse(g.plays)),
        Sort::Year => out.sort_by(|a, b| (a.year.is_empty(), &a.year).cmp(&(b.year.is_empty(), &b.year))),
        Sort::Rating => out.sort_by(|a, b| match (a.rating, b.rating) {
            (Some(x), Some(y)) => y.total_cmp(&x),
            (x, y) => y.is_some().cmp(&x.is_some()),
        }),
        Sort::Name => {}
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
