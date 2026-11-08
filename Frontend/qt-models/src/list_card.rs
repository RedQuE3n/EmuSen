//! The List card's pages (EmuSen_Interface.md §18.1; EmuSen_Qt.md §7.1, stage 2): Mistress's `GamesScreen.ListRows`,
//! `ListPane` and `FilterRows` in her words, made from the grid's listing as `GameModel.listFacts` gives it, and what
//! each row does. The grid carries the actions out (`GameModel.listAct`).

use crate::card::{Choice, Page, Pages, Pane, Row};
use crate::listing::SORTS;
use serde_json::Value;

pub const CARD: &str = "list";

pub const FIRST: &str = "list";
pub const FILTER: &str = "filter";

/// The rows' keys, and the actions the grid carries out by the same keys.
pub const SEARCH: &str = "search";
pub const SORT: &str = "sort";
pub const JUMP: &str = "jump";
pub const RANDOM: &str = "random";
pub const START_CONSOLE: &str = "startConsole";
pub const COLLECTIONS: &str = "collections";
/// The Filter page's rows: every game, and Favourites.
pub const EVERY_GAME: &str = "every";
pub const FAVOURITES_ONLY: &str = "favourites";

/// A step of a value as the action the grid carries out: `sort+1`, `jump-1`.
pub fn stepped(key: &str, by: i32) -> String {
    format!("{key}{by:+}")
}

/// An action read back: its key and its step, 0 for none.
pub fn action_of(action: &str) -> (&str, i32) {
    match action.find(['+', '-']) {
        Some(at) => (&action[..at], action[at..].parse().unwrap_or(0)),
        None => (action, 0),
    }
}

fn words<'a>(facts: &'a Value, key: &str) -> &'a str {
    facts.get(key).and_then(Value::as_str).unwrap_or_default()
}

/// What the Sort row's pane says the list shows now.
fn sort_sentence(sort: &str) -> &'static str {
    match sort {
        "Last played" => "Sorted by when each was last played, newest first.",
        "Most played" => "Sorted by how often each was played, most first.",
        "Year" => "Sorted by year of release, oldest first.",
        "Rating" => "Sorted by rating, highest first.",
        _ => "Sorted by name, A to Z.",
    }
}

fn first_page(f: &Value) -> Page {
    let sort = words(f, "sort");
    let filter = words(f, "filter");
    let others: Vec<&str> = SORTS.iter().copied().filter(|s| *s != sort).collect();
    let shows = match filter {
        "" => "Shows every game in the section.".to_owned(),
        "Favourites" => "Shows the section's favourites.".to_owned(),
        collection => format!("Shows {collection}."),
    };
    let mut rows = vec![
        Row::new(SEARCH, "Search").opens().pane(Pane::said("Type part of a name; the covers narrow as you type.", "The keyboard docks at the foot of the screen.")),
        Row::new(SORT, "Sort").value(sort).steps().pane(Pane::said(sort_sentence(sort), &format!("Also: {}", others.join(" · ")))),
        Row::new(FILTER, "Filter").value(filter).opens().pane(Pane::said(&shows, "Choose Favourites to narrow it, or a collection to show it in the strip's place.")),
        Row::new(JUMP, "Jump to").value(words(f, "letter")).steps().pane(Pane::said("Moves to the first game under the letter.", words(f, "letters"))),
        Row::new(RANDOM, "Random game").pane(Pane::said("Moves to a game chosen at random.", "A plays it.")),
    ];
    if f.get("startConsole").and_then(Value::as_bool).unwrap_or(false) {
        rows.push(Row::new(START_CONSOLE, "Start Console").opens().pane(Pane::said("Starts a console with no disc in it.", "Change Disc on the Pause Card puts a game in.")));
    }
    rows.push(Row::new(COLLECTIONS, "Collections").opens().pane(Pane::said("Your collections, and the automatic ones.", "A opens a collection as a section of the library.")));
    Page { title: format!("{} · {}", words(f, "section"), words(f, "count")), breadcrumb: "List".to_owned(), rows, footer: String::new() }
}

/// The Filter page: every game, then Favourites except in the Favourites section; the player's collections follow
/// once the library reads them.
fn filter_page(f: &Value) -> Page {
    let mut rows = vec![Row::new(EVERY_GAME, "Every game")];
    if !f.get("favouritesSection").and_then(Value::as_bool).unwrap_or(false) {
        rows.push(Row::new(FAVOURITES_ONLY, "Favourites"));
    }
    Page { title: "Filter".to_owned(), breadcrumb: "List".to_owned(), rows, footer: String::new() }
}

pub struct ListCard;

impl Pages for ListCard {
    fn first(&self) -> &'static str {
        FIRST
    }

    fn page(&self, key: &str, facts: &Value) -> Page {
        match key {
            FILTER => filter_page(facts),
            _ => first_page(facts),
        }
    }

    fn choose(&self, page: &str, row: &str) -> Choice {
        match (page, row) {
            (FILTER, filter) => Choice::Act(filter.to_owned()),
            (_, FILTER) => Choice::Turn(FILTER.to_owned()),
            (_, SORT | JUMP) => Choice::Stay(stepped(row, 1)),
            (_, SEARCH | RANDOM) => Choice::Act(row.to_owned()),
            _ => Choice::Owed,
        }
    }

    fn step(&self, page: &str, row: &str, by: i32) -> Option<String> {
        (page == FIRST && matches!(row, SORT | JUMP)).then(|| stepped(row, by))
    }
}
