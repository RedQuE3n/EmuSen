//! The List card's pages (EmuSen_Interface.md §18.1; EmuSen_Qt.md §7.1, stage 2): Mistress's `GamesScreen.ListRows`,
//! `ListPane` and `FilterRows` in her words, made from the grid's listing as `GameModel.listFacts` gives it, and what
//! each row does. The grid carries the actions out (`GameModel.listAct`).

use crate::card::{Choice, Page, Pages, Pane, Row};
use crate::listing::Sort;
use serde_json::Value;

pub const CARD: &str = "list";

pub const FIRST: &str = "list";
pub const FILTER: &str = "filter";

/// The rows' keys, and the actions the grid carries out by the same keys.
pub const SEARCH: &str = "search";
pub const SORT: &str = "sort";
pub const JUMP: &str = "jump";
pub const RANDOM: &str = "random";
pub const START_CONSOLE: &str = crate::home_card::START_CONSOLE;
pub const COLLECTIONS: &str = "collections";
/// The Filter page's rows: every game, and Favourites.
pub const EVERY_GAME: &str = "every";
pub const FAVOURITES_ONLY: &str = "favourites";
/// A collection's row on the Filter page and the grid's action by the same key: `collection:<id>`.
const COLLECTION: &str = "collection:";

/// One of the player's collections as the Filter page lists it and the grid narrows to it.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Listed {
    pub key: String,
    pub name: String,
    pub members: Vec<String>,
}

/// The player's collections as the library gives them to the grid, in games.db's order (by name).
pub fn collections_text(all: &[crate::gamelists::Collection]) -> String {
    let listed: Vec<Listed> = all.iter().map(|c| Listed { key: format!("{COLLECTION}{}", c.id), name: c.name.clone(), members: c.members.clone() }).collect();
    serde_json::to_string(&listed).unwrap_or_default()
}

/// The collections read back from that text; none from text that is not.
pub fn collections_of(text: &str) -> Vec<Listed> {
    serde_json::from_str(text).unwrap_or_default()
}

/// Whether an action is a collection's.
pub fn is_collection(action: &str) -> bool {
    action.starts_with(COLLECTION)
}

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

/// What the Sort row's pane says the list shows now; an ES-DE order is said by its own words.
fn sort_sentence(sort: &str) -> String {
    if let Some((key, order)) = sort.split_once(", ") {
        return format!("Sorted by {}, {order}.", key.to_lowercase());
    }
    match sort {
        "Last played" => "Sorted by when each was last played, newest first.",
        "Most played" => "Sorted by how often each was played, most first.",
        "Year" => "Sorted by year of release, oldest first.",
        "Rating" => "Sorted by rating, highest first.",
        _ => "Sorted by name, A to Z.",
    }
    .to_owned()
}

fn first_page(f: &Value) -> Page {
    let sort = words(f, "sort");
    let filter = words(f, "filter");
    // A theme's gamelist names its own orders (gamelists.rs); the grid's are `Sort::ALL`.
    let offered: Vec<&str> = match f.get("sorts").and_then(Value::as_array) {
        Some(sorts) => sorts.iter().filter_map(Value::as_str).collect(),
        None => Sort::ALL.iter().map(|s| s.words()).collect(),
    };
    let others: Vec<&str> = offered.into_iter().filter(|s| *s != sort).collect();
    let shows = match filter {
        "" => "Shows every game in the section.".to_owned(),
        "Favourites" => "Shows the section's favourites.".to_owned(),
        collection => format!("Shows {collection}."),
    };
    let mut rows = Vec::new();
    let mut footer = String::new();
    // A theme's gamelist of a console that starts empty offers it first (Mistress's `AddStartConsoleOption`); a console
    // the facts name and that cannot be read is said under the rows, never dropped unsaid.
    match f.get("console").cloned().map(serde_json::from_value::<crate::home_card::Startable>) {
        Some(Ok(s)) => {
            let page = crate::home_card::start_console_page(std::slice::from_ref(&s), "List");
            let pane = page.rows.into_iter().next().map(|r| r.pane).unwrap_or_default();
            rows.push(Row::new(&crate::home_card::start_console_action(&s.console), &format!("Start {} With No Disc", s.name)).pane(pane));
        }
        Some(Err(why)) => footer = console_unread(&why.to_string()),
        None => {}
    }
    rows.extend([
        Row::new(SEARCH, "Search").opens().pane(Pane::said("Type part of a name; the covers narrow as you type.", "The keyboard docks at the foot of the screen.")),
        Row::new(SORT, "Sort").value(sort).steps().pane(Pane::said(&sort_sentence(sort), &format!("Also: {}", others.join(" · ")))),
        Row::new(FILTER, "Filter").value(filter).opens().pane(Pane::said(&shows, "Choose Favourites to narrow it, or a collection to show it in the strip's place.")),
        Row::new(JUMP, "Jump to").value(words(f, "letter")).steps().pane(Pane::said("Moves to the first game under the letter.", words(f, "letters"))),
        Row::new(RANDOM, "Random game").pane(Pane::said("Moves to a game chosen at random.", "A plays it.")),
    ]);
    if !startable(f).is_empty() {
        rows.push(Row::new(START_CONSOLE, "Start Console").opens().pane(Pane::said("Starts a console with no disc in it.", "Change Disc on the Pause Card puts a game in.")));
    }
    rows.push(Row::new(COLLECTIONS, "Collections").opens().pane(Pane::said("Your collections, and the automatic ones.", "A opens a collection as a section of the library.")));
    Page { title: format!("{} · {}", words(f, "section"), words(f, "count")), breadcrumb: "List".to_owned(), rows, footer }
}

/// The words under the List card's rows where the console its gamelist starts could not be read.
pub fn console_unread(why: &str) -> String {
    format!("The console this list starts could not be read: {why}")
}

/// The consoles Start Console lists, from the facts.
fn startable(f: &Value) -> Vec<crate::home_card::Startable> {
    f.get("startable").cloned().and_then(|s| serde_json::from_value(s).ok()).unwrap_or_default()
}

/// The Filter page: every game, then Favourites except in the Favourites section, then the player's collections.
fn filter_page(f: &Value) -> Page {
    let mut rows = vec![Row::new(EVERY_GAME, "Every game")];
    if !f.get("favouritesSection").and_then(Value::as_bool).unwrap_or(false) {
        rows.push(Row::new(FAVOURITES_ONLY, "Favourites"));
    }
    for c in f.get("collections").and_then(Value::as_array).into_iter().flatten() {
        rows.push(Row::new(words(c, "key"), words(c, "name")));
    }
    Page { title: "Filter".to_owned(), breadcrumb: "List".to_owned(), rows, footer: String::new() }
}

/// The List card's facts for a theme's gamelist, from what its last listing summed up, the entry at `focused` selected.
pub fn esde_facts(s: &crate::esde_shelves::Summary, focused: usize, console: Option<&crate::home_card::Startable>) -> Value {
    let sorts: Vec<String> = crate::gamelists::GameSort::all(s.collection).into_iter().map(|o| o.label()).collect();
    serde_json::json!({
        "section": s.full_name,
        "count": crate::listing::count_line(s.counts.games, s.counts.shown, s.counts.filtered),
        "sort": s.sort,
        "sorts": sorts,
        "filter": if s.favourites_only { "Favourites" } else { "" },
        "letter": s.entry_letters.get(focused).cloned().unwrap_or_default(),
        "letters": s.letters.iter().map(|(l, _)| l.as_str()).collect::<Vec<_>>().join(" · "),
        "favouritesSection": s.favourites_shelf,
        "console": console,
    })
}

pub struct ListCard;

impl Pages for ListCard {
    fn first(&self) -> &'static str {
        FIRST
    }

    fn page(&self, key: &str, facts: &Value) -> Page {
        match key {
            FILTER => filter_page(facts),
            START_CONSOLE => crate::home_card::start_console_page(&startable(facts), "List"),
            _ => first_page(facts),
        }
    }

    fn choose(&self, page: &str, row: &str) -> Choice {
        match (page, row) {
            (FILTER, filter) => Choice::Act(filter.to_owned()),
            (_, FILTER | START_CONSOLE) => Choice::Turn(row.to_owned()),
            // Collections opens the Filter page, where the player's collections are listed after Favourites.
            (_, COLLECTIONS) => Choice::Turn(FILTER.to_owned()),
            (START_CONSOLE, console) => Choice::Act(console.to_owned()),
            (FIRST, row) if crate::home_card::console_to_start(row).is_some() => Choice::Act(row.to_owned()),
            (_, SORT | JUMP) => Choice::Stay(stepped(row, 1)),
            (_, SEARCH | RANDOM) => Choice::Act(row.to_owned()),
            _ => Choice::Owed,
        }
    }

    fn step(&self, page: &str, row: &str, by: i32) -> Option<String> {
        (page == FIRST && matches!(row, SORT | JUMP)).then(|| stepped(row, by))
    }

    /// Start Console's page names A "Start" (Mistress's `AcceptWord`).
    fn help(&self, page: &str) -> crate::help::Screen {
        if page == START_CONSOLE { crate::help::Screen::CardStart } else { crate::help::Screen::Card }
    }
}
