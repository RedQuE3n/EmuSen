//! The library screen's navigation as plain Rust (EmuSen_Qt.md §1.2.1): the places (all games, favourites, recently
//! played) and then each console that has games, in the systems table's order, each derived from what the systems and
//! the library declare, with nothing per console written here. A source answers; the rows are made from its answers.

use crate::keyed::{Keyed, KeyedRows};
use emusen_crystal_client::shapes::SystemRow;
use emusen_kit_core::world::{hex, shape_of, world_line, WorldBody, WorldRing};
use emusen_platform_client::library::GameFile;

/// What the library view reads, from the client crates or a test's fake.
pub trait LibrarySource: Send + Sync {
    /// The systems, the table's first in its order.
    fn systems(&self) -> Result<Vec<SystemRow>, String>;
    /// The library's games, found afresh.
    fn games(&self) -> Result<Vec<GameFile>, String>;
    /// How many of `games` are favourites.
    fn favourites(&self, games: &[GameFile]) -> Result<usize, String>;
    /// How many of `games` have been played.
    fn recent(&self, games: &[GameFile]) -> Result<usize, String>;
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
    /// The engine the console runs by default: the first the runtime lists for it.
    pub engine: String,
    /// The console's world as one line, from the systems table's look (EmuSen_Kit.md §7.16).
    pub world: String,
}

impl Keyed for LibraryRow {
    fn key(&self) -> &str {
        &self.key
    }
}

/// The places, their keys and their names, in order (EmuSen_Interface.md §24).
pub const PLACES: [(&str, &str); 3] = [("all", "All games"), ("favourites", "Favourites"), ("recent", "Recently played")];

/// The rows for what a source answered: the places with their counts, then each system that has games.
pub fn rows(systems: &[SystemRow], games: &[GameFile], favourites: usize, recent: usize) -> Vec<LibraryRow> {
    let counts = [games.len(), favourites, recent];
    let place = |((key, name), count): ((&str, &str), usize)| LibraryRow {
        kind: Some(Kind::Place),
        key: key.into(),
        name: name.into(),
        count,
        ..LibraryRow::default()
    };
    let mut out: Vec<LibraryRow> = PLACES.into_iter().zip(counts).map(place).collect();
    for s in systems {
        let count = games.iter().filter(|g| g.system.as_deref() == Some(s.id.as_str())).count();
        if count == 0 {
            continue;
        }
        out.push(LibraryRow {
            kind: Some(Kind::Console),
            key: s.id.clone(),
            name: if s.display_name.is_empty() { s.name.clone() } else { s.display_name.clone() },
            count,
            hue: s.look.hue,
            body: s.look.body.clone(),
            maker: s.manufacturer.clone(),
            year: s.release_year,
            engine: s.engines.first().cloned().unwrap_or_default(),
            world: world_line(s.look.hue, &world_of(s)),
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

/// Moonrise's planets: one world for each system the table lists with an engine, in the table's order, each the body its
/// tile shows (EmuSen_Interface.md §22, §24.6).
pub fn splash_worlds(systems: &[SystemRow]) -> Vec<String> {
    systems.iter().filter(|s| !s.engines.is_empty()).map(|s| world_line(s.look.hue, &world_of(s))).collect()
}

/// What one refresh read: the rows, or the words for why it could not.
pub type Refreshed = Result<Vec<LibraryRow>, String>;

/// One refresh of a source, run off the GUI thread.
pub fn refresh(source: &dyn LibrarySource) -> Refreshed {
    let systems = source.systems()?;
    let games = source.games()?;
    let favourites = source.favourites(&games)?;
    let recent = source.recent(&games)?;
    Ok(rows(&systems, &games, favourites, recent))
}

/// The view's state: the rows with the selection kept across refreshes, and the last refresh's problem.
#[derive(Clone, Debug, Default)]
pub struct LibraryView {
    pub rows: KeyedRows<LibraryRow>,
    pub problem: String,
}

impl LibraryView {
    /// A view with the places alone, before anything is read.
    pub fn empty() -> LibraryView {
        let mut view = LibraryView::default();
        view.rows.replace(rows(&[], &[], 0, 0));
        view
    }

    /// A refresh's answer applied: new rows with the selection kept, or the rows kept and the problem's words shown.
    pub fn apply(&mut self, refreshed: Refreshed) {
        match refreshed {
            Ok(rows) => {
                self.rows.replace(rows);
                self.problem.clear();
            }
            Err(words) => self.problem = words,
        }
    }
}
