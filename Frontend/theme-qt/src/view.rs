//! What a theme sees of the library, as plain rows (EmuSen_Qt.md §3.2): each list's roles written once here and its rows
//! curated from emusen-qt-models' view, a cell for each role in the role's order. No Qt.

use emusen_qt_models::library::{self, GameRow, Kind, LibraryView};
use emusen_qt_models::listing::{self, Listing};
use emusen_qt_models::game_model::cover_url;
use emusen_qt_models::words;
use serde_json::Value;
use std::collections::HashMap;

/// One value of a row, as QML reads it.
#[derive(Clone, Debug, PartialEq)]
pub enum Cell {
    Text(String),
    Number(f64),
    Whole(i32),
    Flag(bool),
}

/// A row's cells, one for each of its list's roles, in their order.
pub type Row = Vec<Cell>;

/// `Library.consoles`: each console with games, in the shelves' order.
pub const CONSOLE_ROLES: [&str; 11] =
    ["key", "name", "displayName", "maker", "year", "count", "hue", "body", "engineName", "engineKind", "engineMark"];
/// `Library.places`: Home's destinations in Home's order.
pub const PLACE_ROLES: [&str; 6] = ["key", "name", "kind", "line", "attention", "count"];
/// `Library.shelves`: the library's three places, All, Favourites and Recent, which `gamesOf` also takes.
pub const SHELF_ROLES: [&str; 4] = ["key", "name", "line", "count"];
/// `Library.gamesOf`, `recent`, `favourites` and `search`: one game a row.
pub const GAME_ROLES: [&str; 18] = [
    "key",
    "title",
    "cover",
    "coverShape",
    "favourite",
    "lastPlayed",
    "played",
    "consoleId",
    "consoleHue",
    "players",
    "year",
    "genre",
    "rating",
    "description",
    "video",
    "hero",
    "lastPlayedWords",
    "facts",
];
/// `Library.collections`: the player's collections.
pub const COLLECTION_ROLES: [&str; 3] = ["key", "name", "count"];
/// `Help.entries`: the help bar's entries, as the kit's `HelpBar` reads them.
pub const HELP_ROLES: [&str; 3] = ["button", "label", "pair"];

/// Home's destinations' keys, in Home's order: Games, the four tools as their summaries name them, Settings, Power.
pub const PLACE_IDS: [&str; 7] =
    ["games", words::HOME_TOOLS[0], words::HOME_TOOLS[1], words::HOME_TOOLS[2], words::HOME_TOOLS[3], "settings", "power"];
/// What each destination opens: the library, a tool, the settings, or the power choices.
pub const PLACE_KINDS: [&str; 7] = ["library", "tool", "tool", "tool", "tool", "settings", "power"];

/// A console's engine kind, as a theme reads it.
pub fn engine_kind(libretro: bool) -> &'static str {
    if libretro {
        "libretro"
    } else {
        "emusen"
    }
}

fn text(s: &str) -> Cell {
    Cell::Text(s.to_owned())
}

/// The consoles with games, each with the systems table's facts and the engine it runs now.
pub fn consoles(view: &LibraryView) -> Vec<Row> {
    view.rows
        .rows()
        .iter()
        .filter(|r| r.kind == Some(Kind::Console))
        .map(|r| {
            vec![
                text(&r.key),
                text(&r.name),
                text(if r.full_name.is_empty() { &r.name } else { &r.full_name }),
                text(&r.maker),
                Cell::Whole(r.year),
                Cell::Whole(r.count as i32),
                Cell::Number(r.hue),
                text(&r.world),
                text(&r.engine_name),
                text(if r.engine_name.is_empty() { "" } else { engine_kind(r.libretro) }),
                text(&r.engine_mark),
            ]
        })
        .collect()
}

/// Home's lines as the library model publishes them (`homeLines`): `[{"status": .., "attention": ..}]`, one a destination.
pub fn lines_of(json: &str) -> Vec<(String, bool)> {
    let Ok(Value::Array(lines)) = serde_json::from_str::<Value>(json) else { return Vec::new() };
    lines
        .iter()
        .map(|l| {
            (
                l.get("status").and_then(Value::as_str).unwrap_or_default().to_owned(),
                l.get("attention").and_then(Value::as_bool).unwrap_or(false),
            )
        })
        .collect()
}

/// Home's seven destinations with their live lines; Games counts the consoles it leads to.
pub fn places(view: &LibraryView, lines: &[(String, bool)]) -> Vec<Row> {
    let consoles = view.rows.rows().iter().filter(|r| r.kind == Some(Kind::Console)).count();
    PLACE_IDS
        .iter()
        .zip(PLACE_KINDS)
        .zip(words::HOME_PLACES)
        .enumerate()
        .map(|(i, ((id, kind), name))| {
            let (line, attention) = lines.get(i).cloned().unwrap_or_default();
            vec![
                text(id),
                text(name),
                text(kind),
                Cell::Text(line),
                Cell::Flag(attention),
                Cell::Whole(if i == 0 { consoles as i32 } else { 0 }),
            ]
        })
        .collect()
}

/// The library's three places, each with its line and count.
pub fn shelves(view: &LibraryView) -> Vec<Row> {
    view.rows
        .rows()
        .iter()
        .filter(|r| r.kind == Some(Kind::Place))
        .map(|r| vec![text(&r.key), text(&r.name), text(&r.status), Cell::Whole(r.count as i32)])
        .collect()
}

/// A game's row at `now`, in seconds since 1970; `shapes` holds each console's cover shape, width over height.
fn game(g: &GameRow, shapes: &HashMap<&str, f64>, now: i64) -> Row {
    let played = g.played_at > 0;
    vec![
        text(&g.path),
        text(&g.title),
        Cell::Text(cover_url(&g.cover)),
        Cell::Number(shapes.get(g.system.as_str()).copied().unwrap_or(0.0)),
        Cell::Flag(g.favourite),
        Cell::Number(g.played_at as f64),
        Cell::Flag(played),
        text(&g.system),
        Cell::Number(g.hue),
        // Players, year, genre, rating, description and video wait on the scraper's facts in the own look's view (§3.2 as built).
        Cell::Whole(0),
        Cell::Whole(0),
        text(""),
        Cell::Number(0.0),
        text(""),
        text(""),
        Cell::Text(cover_url(if g.hero.is_empty() { &g.cover } else { &g.hero })),
        Cell::Text(if played { words::ago(g.played_at, now) } else { String::new() }),
        text(&g.facts),
    ]
}

/// Rows of games as a list shows them, each console's cover shape from the view (`Browse.games`, Home's shelf).
pub fn games(view: &LibraryView, list: &[GameRow], now: i64) -> Vec<Row> {
    let shapes: HashMap<&str, f64> = view.systems.iter().map(|s| (s.id.as_str(), s.cover_aspect)).collect();
    list.iter().map(|g| game(g, &shapes, now)).collect()
}

/// The games a console or a place leads to (`all`, `favourites`, `recent`); none for an id that is neither.
pub fn games_of(view: &LibraryView, id: &str, now: i64) -> Vec<Row> {
    games(view, &library::games_of_at(id, &view.systems, &view.catalog, now), now)
}

/// Home's shelf: the games played, latest first, less the suspended game, which Continue holds.
pub fn shelf(view: &LibraryView, suspended: &str, now: i64) -> Vec<Row> {
    let recent = library::games_of_at(library::RECENT, &view.systems, &view.catalog, now);
    games(view, &emusen_qt_models::games::without(recent, suspended), now)
}

/// Every game whose title holds the words, in any case, by title; none for no words.
pub fn search(view: &LibraryView, words: &str, now: i64) -> Vec<Row> {
    if words.trim().is_empty() {
        return Vec::new();
    }
    let all = library::games_of_at(library::ALL, &view.systems, &view.catalog, now);
    let listing = Listing { search: words.to_owned(), ..Listing::default() };
    games(view, &listing::shown(&all, library::ALL, &listing), now)
}
