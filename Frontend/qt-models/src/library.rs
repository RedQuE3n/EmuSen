//! The library screen's navigation as plain Rust (EmuSen_Qt.md §1.2.1): the places (all games, favourites, recently
//! played) and then each console that has games, in the systems table's order, each derived from what the systems and
//! the library declare, with nothing per console written here. A source answers; the rows are made from its answers.

use crate::keyed::{Keyed, KeyedRows};
use crate::words;
use emusen_crystal_client::shapes::SystemRow;
use emusen_kit_core::world::{hex, shape_of, world_line, WorldBody, WorldRing};
use emusen_platform_client::library::GameFile;

/// What the library view reads, from the client crates or a test's fake.
pub trait LibrarySource: Send + Sync {
    /// The systems, the table's first in its order.
    fn systems(&self) -> Result<Vec<SystemRow>, String>;
    /// The library's games, found afresh.
    fn games(&self) -> Result<Vec<GameFile>, String>;
    /// The paths of `games` that are favourites.
    fn favourites(&self, games: &[GameFile]) -> Result<Vec<String>, String>;
    /// The paths of `games` that have been played, the most recent first.
    fn recent(&self, games: &[GameFile]) -> Result<Vec<String>, String>;
    /// The engine each system runs now, by system id: the player's stored choice, else the console's default
    /// (EmuSen_Interface.md §24.6). A system left out shows no engine.
    fn engines(&self, systems: &[SystemRow]) -> Result<Vec<(String, Engine)>, String>;
}

/// An engine as a tile shows it: its name from its own info, and whether it is a hosted libretro core.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Engine {
    /// The name the choice stores.
    pub id: String,
    /// The name shown: a libretro core's corename, an EmuSen core's info name less the port's "RT".
    pub name: String,
    pub libretro: bool,
}

/// An EmuSen core's info name as shown: VenusRT reads Venus, the one named exception of §24.6.
pub fn without_port_mark(name: &str) -> &str {
    match name.strip_suffix("RT") {
        Some(rest) if rest.chars().last().is_some_and(char::is_lowercase) => rest,
        _ => name,
    }
}

/// A row's kind: a place, or a console.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Place,
    Console,
}

impl Kind {
    /// The kind as QML reads it.
    pub fn name(self) -> &'static str {
        match self {
            Kind::Place => "place",
            Kind::Console => "console",
        }
    }
}

/// One row of the library's navigation; every word a screen shows of it is here.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LibraryRow {
    pub kind: Option<Kind>,
    /// "all", "favourites", "recent", or the console's system id.
    pub key: String,
    pub name: String,
    pub count: usize,
    pub hue: f64,
    pub body: String,
    pub maker: String,
    pub year: i32,
    /// The console's full name, the systems table's `name`; a place's own name.
    pub full_name: String,
    /// The line under the name: the count, or a place's own words.
    pub status: String,
    /// The focused row's facts beneath the rows.
    pub facts: String,
    /// The engine the console runs now (the choice's stored name), its shown name and its kind.
    pub engine: String,
    pub engine_name: String,
    pub libretro: bool,
    /// The console's world as one line, from the systems table's look (EmuSen_Kit.md §7.16).
    pub world: String,
}

impl Keyed for LibraryRow {
    fn key(&self) -> &str {
        &self.key
    }
}

/// The places, their keys and their names, in order (EmuSen_Interface.md §24).
pub const PLACES: [(&str, &str); 3] = [("all", "All games"), (FAVOURITES, "Favourites"), (RECENT, "Recently played")];

/// The recently played games' key, which Home's shelf shows.
pub const RECENT: &str = "recent";

/// The favourites' key, which Choose a console's Y opens as a place.
pub const FAVOURITES: &str = "favourites";

/// What a refresh read of the games: each game, which are favourites, and which were played, the most recent first.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Catalog {
    pub games: Vec<GameFile>,
    pub favourites: Vec<String>,
    pub recent: Vec<String>,
}

/// One game as the screens show it: its title from its file name, its console, its console's hue.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GameRow {
    pub path: String,
    pub system: String,
    pub title: String,
    pub hue: f64,
    pub favourite: bool,
}

/// A game's title: its file name without the extension.
pub fn title_of(path: &str) -> String {
    let name = std::path::Path::new(path);
    name.file_stem().or(name.file_name()).map_or_else(|| path.to_owned(), |n| n.to_string_lossy().into_owned())
}

/// The games a row of the navigation leads to: all of them by title, the favourites by title, the recently played the
/// most recent first, or a console's by title.
pub fn games_of(key: &str, systems: &[SystemRow], catalog: &Catalog) -> Vec<GameRow> {
    let hue = |system: &str| systems.iter().find(|s| s.id == system).map_or(0.0, |s| s.look.hue);
    let row = |g: &GameFile| {
        let system = g.system.clone().unwrap_or_default();
        GameRow {
            title: title_of(&g.path),
            hue: hue(&system),
            favourite: catalog.favourites.contains(&g.path),
            path: g.path.clone(),
            system,
        }
    };
    if key == RECENT {
        return catalog.recent.iter().filter_map(|p| catalog.games.iter().find(|g| &g.path == p)).map(row).collect();
    }
    let mut out: Vec<GameRow> = catalog
        .games
        .iter()
        .map(row)
        .filter(|g| match key {
            "all" => true,
            FAVOURITES => g.favourite,
            system => g.system == system,
        })
        .collect();
    out.sort_by_key(|g| g.title.to_lowercase());
    out
}

/// The rows for what a source answered: the places with their counts, then each system that has games.
pub fn rows(systems: &[SystemRow], catalog: &Catalog, engines: &[(String, Engine)]) -> Vec<LibraryRow> {
    let last = catalog.recent.first().map(|p| title_of(p));
    let lines = [
        words::games(catalog.games.len()),
        words::FAVOURITES_LINE.to_owned(),
        words::last_played(last.as_deref()),
    ];
    let counts = [catalog.games.len(), catalog.favourites.len(), catalog.recent.len()];
    let mut out: Vec<LibraryRow> = PLACES
        .into_iter()
        .zip(counts)
        .zip(lines)
        .map(|(((key, name), count), status)| LibraryRow {
            kind: Some(Kind::Place),
            key: key.into(),
            name: name.into(),
            full_name: name.into(),
            facts: words::games(count),
            count,
            status,
            ..LibraryRow::default()
        })
        .collect();
    for s in systems {
        let count = catalog.games.iter().filter(|g| g.system.as_deref() == Some(s.id.as_str())).count();
        if count == 0 {
            continue;
        }
        let engine = engines.iter().find(|(id, _)| id == &s.id).map(|(_, e)| e.clone()).unwrap_or_default();
        let runs_on = if engine.name.is_empty() { String::new() } else { words::runs_on(&engine.name, engine.libretro) };
        out.push(LibraryRow {
            kind: Some(Kind::Console),
            key: s.id.clone(),
            name: if s.display_name.is_empty() { s.name.clone() } else { s.display_name.clone() },
            full_name: s.name.clone(),
            status: words::games(count),
            facts: words::facts(&s.manufacturer, s.release_year, count, &runs_on),
            count,
            hue: s.look.hue,
            body: s.look.body.clone(),
            maker: s.manufacturer.clone(),
            year: s.release_year,
            engine: engine.id,
            engine_name: engine.name,
            libretro: engine.libretro,
            world: world_line_of(s),
        });
    }
    out
}

/// A console's body from the systems table's look, the one owner of which console is which body (EmuSen_Interface.md
/// §24.6): its shape, its three colours where the table gives three, and its features.
pub fn world_of(s: &SystemRow) -> WorldBody {
    let colours: Vec<_> = s.look.palette.iter().filter_map(|c| hex(c)).collect();
    WorldBody {
        shape: shape_of(&s.look.body),
        palette: <[_; 3]>::try_from(colours).ok(),
        craters: s.look.craters,
        bands: s.look.bands,
        cap: s.look.cap,
        ring: WorldRing::None,
        moons: s.look.moons.min(2) as u8,
    }
}

/// A console's world as the kit's one line: its hue and its body.
pub fn world_line_of(s: &SystemRow) -> String {
    world_line(s.look.hue, &world_of(s))
}

/// Moonrise's planets: one world for each system the table lists with an engine, in the table's order, each the body its
/// tile shows (EmuSen_Interface.md §22, §24.6).
pub fn splash_worlds(systems: &[SystemRow]) -> Vec<String> {
    systems.iter().filter(|s| !s.engines.is_empty()).map(world_line_of).collect()
}

/// What one refresh read: the systems, the games and the rows made of them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Read {
    pub systems: Vec<SystemRow>,
    pub catalog: Catalog,
    pub rows: Vec<LibraryRow>,
}

/// One refresh's answer, or the words for why it could not.
pub type Refreshed = Result<Read, String>;

/// One refresh of a source, run off the GUI thread.
pub fn refresh(source: &dyn LibrarySource) -> Refreshed {
    let systems = source.systems()?;
    let games = source.games()?;
    let favourites = source.favourites(&games)?;
    let recent = source.recent(&games)?;
    let engines = source.engines(&systems)?;
    let catalog = Catalog { games, favourites, recent };
    let rows = rows(&systems, &catalog, &engines);
    Ok(Read { systems, catalog, rows })
}

/// The view's state: the rows with the selection kept across refreshes, and the last refresh's problem.
#[derive(Clone, Debug, Default)]
pub struct LibraryView {
    pub rows: KeyedRows<LibraryRow>,
    pub systems: Vec<SystemRow>,
    pub catalog: Catalog,
    pub problem: String,
}

impl LibraryView {
    /// "11 consoles · 1,566 games": the consoles shown and every game.
    pub fn summary(&self) -> String {
        let consoles = self.rows.rows().iter().filter(|r| r.kind == Some(Kind::Console)).count();
        words::summary(consoles, self.catalog.games.len())
    }

    /// A view with the places alone, before anything is read.
    pub fn empty() -> LibraryView {
        let mut view = LibraryView::default();
        view.rows.replace(rows(&[], &Catalog::default(), &[]));
        view
    }

    /// A refresh's answer applied: new rows with the selection kept, or the rows kept and the problem's words shown.
    pub fn apply(&mut self, refreshed: Refreshed) {
        match refreshed {
            Ok(read) => {
                self.rows.replace(read.rows);
                self.systems = read.systems;
                self.catalog = read.catalog;
                self.problem.clear();
            }
            Err(words) => self.problem = words,
        }
    }
}
