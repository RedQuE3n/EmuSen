//! The library screen's navigation as plain Rust (EmuSen_Qt.md §1.2.1): the places (all games, favourites, recently
//! played) and then each console that has games, in the systems table's order, each derived from what the systems and
//! the library declare, with nothing per console written here. A source answers; the rows are made from its answers.

use crate::keyed::{Keyed, KeyedRows};
use std::collections::{HashMap, HashSet};
use crate::esde_shelves::{Esde, EsdeSettings, EsdeState, MediaIndex, Plays};
use crate::gamelists::{Collection, Gamelists};
use std::sync::Arc;
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
    /// The paths of `games` the player hid from the library (games.db's `hidden` edit); a source with no records hides none.
    fn hidden(&self, _games: &[GameFile]) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }
    /// The names the player gave games in Edit Details (games.db's `name` edit), by path; a source with no records has none.
    fn names(&self, _games: &[GameFile]) -> Result<Vec<(String, String)>, String> {
        Ok(Vec::new())
    }
    /// The paths of `games` that have been played, the most recent first.
    fn recent(&self, games: &[GameFile]) -> Result<Vec<String>, String>;
    /// The engine each system runs now, by system id: the player's stored choice, else the console's default
    /// (EmuSen_Interface.md §24.6). A system left out shows no engine.
    fn engines(&self, systems: &[SystemRow]) -> Result<Vec<(String, Engine)>, String>;
    /// Each game's file of the first of `types` the media store has, by path (EmuSen_Interface.md §21); a source with no
    /// store has none.
    fn media(&self, _systems: &[SystemRow], _games: &[GameFile], _types: &[&str]) -> Result<Vec<(String, String)>, String> {
        Ok(Vec::new())
    }
    /// When each played game was last played, by path, in seconds since 1970.
    fn played(&self, _games: &[GameFile]) -> Result<Vec<(String, i64)>, String> {
        Ok(Vec::new())
    }
    /// Records the game at `path` as started now, as a game the player opens is (games.db's `started`); a source with no
    /// records keeps none.
    fn started(&self, _path: &str) -> Result<(), String> {
        Ok(())
    }
    /// Turns the game at `path` a favourite or back (games.db's `toggle_favourite`); a source with no records keeps none.
    fn toggle_favourite(&self, _path: &str) -> Result<(), String> {
        Ok(())
    }
    /// The shelves' systems in the runtime's order, which Choose a console follows (EmuSen_Interface.md §24.1); none
    /// leaves the systems' own order.
    fn shelves(&self) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }
    /// The player's settings as Galaxia's model reads them; null where there are none.
    fn settings(&self) -> Result<serde_json::Value, String> {
        Ok(serde_json::Value::Null)
    }
    /// Each game's metadata as the scraper resolves it (EmuSen_Backend.md §30.3, `metadata`), by path; a source with no
    /// scraper has none, and the file names the game.
    fn metadata(&self, _games: &[GameFile]) -> Result<Vec<(String, crate::esde_shelves::Metadata)>, String> {
        Ok(Vec::new())
    }
    /// How often and how long each played game was played, by path, as games.db records it.
    fn plays(&self, _games: &[GameFile]) -> Result<Vec<(String, Plays)>, String> {
        Ok(Vec::new())
    }
    /// A system's firmware files with the status the runtime finds for each (`firmware_status`); none where it names
    /// none, the runtime's words where it cannot say.
    fn firmware(&self, _system: &str) -> Result<Vec<emusen_crystal_client::shapes::FirmwareFile>, String> {
        Ok(Vec::new())
    }
    /// The media found for the shelves named by their ES-DE systems: the media store's and the ES-DE folder's files, the
    /// store's first (esde_shelves::MediaIndex); a source with neither has none.
    fn media_index(&self, _systems: &[String]) -> Result<MediaIndex, String> {
        Ok(MediaIndex::default())
    }
    /// The player's collections and their members, from games.db.
    fn collections(&self) -> Result<Vec<Collection>, String> {
        Ok(Vec::new())
    }
    /// Each ES-DE folder link the player set, by the folder's path on disk, to the linked game's file.
    fn folder_links(&self) -> Result<Vec<(String, String)>, String> {
        Ok(Vec::new())
    }
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
    /// The grid's heading for the row: a place's short name, a console's name.
    pub heading: String,
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
    /// The mark before the engine's name: its astronomical symbol, its maker's glyph, or libretro's badge (§24.6).
    pub engine_mark: String,
    /// The console's world as one line, from the systems table's look (EmuSen_Kit.md §7.18).
    pub world: String,
    /// The row's first games as GameModel reads them, the covers Choose a console fans; empty until a refresh.
    pub fan: String,
}

impl Keyed for LibraryRow {
    fn key(&self) -> &str {
        &self.key
    }
}

/// The places, their keys and their names, in order (EmuSen_Interface.md §24).
pub const PLACES: [(&str, &str); 3] = [("all", "All games"), (FAVOURITES, "Favourites"), (RECENT, "Recently played")];

/// The places' headings on the grid, as the accepted render writes them (EmuSen_Interface.md §24.2).
pub const PLACE_HEADINGS: [&str; 3] = ["All", "Favourites", "Recent"];

/// The recently played games' key, which Home's shelf shows.
pub const RECENT: &str = "recent";
/// Every game's key.
pub const ALL: &str = "all";

/// The setting that lists hidden games with the rest (`AppSettings.ShowHiddenGames`).
pub const SHOW_HIDDEN: &str = "ShowHiddenGames";

/// The favourites' key, which Choose a console's Y opens as a place.
pub const FAVOURITES: &str = "favourites";

/// What a refresh read of the games: each game, which are favourites, and which were played, the most recent first.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Catalog {
    pub games: Vec<GameFile>,
    pub favourites: Vec<String>,
    pub recent: Vec<String>,
    /// Each game's cover file, by path.
    pub covers: Vec<(String, String)>,
    /// Each played game's hero art, by path: the first of `HERO` the media store has.
    pub heroes: Vec<(String, String)>,
    /// When each played game was last played, by path, in seconds since 1970.
    pub played: Vec<(String, i64)>,
    /// The player's settings.
    pub settings: serde_json::Value,
    /// How often and how long each played game was played, by path (games.db).
    pub plays: Vec<(String, Plays)>,
    /// Each game's metadata as the scraper resolves it, by path; read once a refresh for the grid's sorts and the ES-DE view.
    pub metadata: Vec<(String, crate::esde_shelves::Metadata)>,
    /// The firmware line of each system that starts empty, by its id: Start Console's pane (Mistress's `FirmwareLine`).
    pub firmware: Vec<(String, String)>,
    /// The names the player gave games, by path; a game with none is named by its file.
    pub names: Vec<(String, String)>,
}

/// One game as the screens show it: its title from its file name, its console, its console's hue, its cover and its facts.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GameRow {
    pub path: String,
    pub system: String,
    pub title: String,
    pub hue: f64,
    pub favourite: bool,
    /// The cover's file; empty for a game with no art, which shows the generated card.
    pub cover: String,
    /// The hero art's file, read for played games alone; empty for none.
    pub hero: String,
    /// The console's name, as the facts begin.
    pub console: String,
    /// The console's name and when it was last played: "Super Nintendo · 3 h ago".
    pub facts: String,
    /// When it was last played, in seconds since 1970; 0 for never.
    pub played_at: i64,
    /// How many times it was played (games.db); 0 for never.
    pub plays: i64,
    /// Its year of release, from the scraper's release date; empty where none is known.
    pub year: String,
    /// Its rating from 0 to 1, the scraper's; none where none is known.
    pub rating: Option<f64>,
}

/// The media store's type for a cover.
pub const COVER: &str = "cover";

/// A game's hero art, the picture Home's top shows for a game with no session: fan art, else a screenshot, else the
/// title screen, as Mistress's game art chooses it (EmuSen_Qt.md §7.1, step 6).
pub const HERO: [&str; 3] = ["fanart", "screenshot", "titlescreen"];

/// A game's title: its file name without the extension.
pub fn title_of(path: &str) -> String {
    let name = std::path::Path::new(path);
    name.file_stem().or(name.file_name()).map_or_else(|| path.to_owned(), |n| n.to_string_lossy().into_owned())
}

/// The games a row of the navigation leads to: all of them by title, the favourites by title, the recently played the
/// most recent first, or a console's by title.
pub fn games_of(key: &str, systems: &[SystemRow], catalog: &Catalog) -> Vec<GameRow> {
    games_of_at(key, systems, catalog, words::now())
}

/// A console's name as a game's facts begin: its display name, else the table's name.
pub fn console_name(s: &SystemRow) -> String {
    if s.display_name.is_empty() { s.name.clone() } else { s.display_name.clone() }
}

/// As `games_of`, each game's facts said at `now`, in seconds since 1970.
pub fn games_of_at(key: &str, systems: &[SystemRow], catalog: &Catalog, now: i64) -> Vec<GameRow> {
    let all = game_rows(systems, catalog, now);
    if key == RECENT {
        return recent_of(&all, catalog);
    }
    let mut out: Vec<GameRow> = all
        .into_iter()
        .filter(|g| match key {
            "all" => true,
            FAVOURITES => g.favourite,
            system => g.system == system,
        })
        .collect();
    out.sort_by_cached_key(|g| g.title.to_lowercase());
    out
}

/// Every game as a row, in the catalog's order: one pass, each fact found by its path in a map.
fn game_rows(systems: &[SystemRow], catalog: &Catalog, now: i64) -> Vec<GameRow> {
    let by_path = |pairs: &[(String, String)]| -> HashMap<String, String> { pairs.iter().cloned().collect() };
    let (covers, heroes, names) = (by_path(&catalog.covers), by_path(&catalog.heroes), by_path(&catalog.names));
    let played: HashMap<&str, i64> = catalog.played.iter().map(|(p, at)| (p.as_str(), *at)).collect();
    let favourites: HashSet<&str> = catalog.favourites.iter().map(String::as_str).collect();
    let consoles: HashMap<&str, &SystemRow> = systems.iter().map(|s| (s.id.as_str(), s)).collect();
    let plays: HashMap<&str, i64> = catalog.plays.iter().map(|(p, n)| (p.as_str(), n.count)).collect();
    let metadata: HashMap<&str, &crate::esde_shelves::Metadata> = catalog.metadata.iter().map(|(p, m)| (p.as_str(), m)).collect();
    catalog
        .games
        .iter()
        .map(|g| {
            let system = g.system.clone().unwrap_or_default();
            let of = consoles.get(system.as_str());
            let console = of.map(|s| console_name(s)).unwrap_or_default();
            let when = played.get(g.path.as_str()).map(|at| words::ago(*at, now));
            GameRow {
                title: names.get(&g.path).cloned().unwrap_or_else(|| title_of(&g.path)),
                hue: of.map_or(0.0, |s| s.look.hue),
                favourite: favourites.contains(g.path.as_str()),
                cover: covers.get(&g.path).cloned().unwrap_or_default(),
                hero: heroes.get(&g.path).cloned().unwrap_or_default(),
                facts: [Some(console.clone()), when].into_iter().flatten().filter(|w| !w.is_empty()).collect::<Vec<_>>().join(" · "),
                played_at: played.get(g.path.as_str()).copied().unwrap_or(0),
                plays: plays.get(g.path.as_str()).copied().unwrap_or(0),
                year: metadata.get(g.path.as_str()).and_then(|m| crate::esde_shelves::release_date(m)).and_then(|d| crate::esde_shelves::year_of(&d)).unwrap_or_default(),
                rating: metadata.get(g.path.as_str()).and_then(|m| crate::esde_shelves::rating(m)),
                path: g.path.clone(),
                system,
                console,
            }
        })
        .collect()
}

/// The recently played of `all`, the most recent first.
fn recent_of(all: &[GameRow], catalog: &Catalog) -> Vec<GameRow> {
    let at: HashMap<&str, &GameRow> = all.iter().map(|g| (g.path.as_str(), g)).collect();
    catalog.recent.iter().filter_map(|p| at.get(p.as_str()).map(|g| (*g).clone())).collect()
}

/// How many covers Choose a console fans for the focused row (EmuSen_Interface.md §24.1).
pub const FAN: usize = 4;

/// Each row's games as GameModel reads them, by the row's key, and the rows given their fans: worked out once a
/// refresh, off the GUI thread, so a screen's step or a grid's opening never walks the library (EmuSen_Qt.md §7.1).
pub fn lists_of(rows: &mut [LibraryRow], systems: &[SystemRow], catalog: &Catalog, now: i64) -> HashMap<String, String> {
    let all = game_rows(systems, catalog, now);
    let mut sorted: Vec<&GameRow> = all.iter().collect();
    sorted.sort_by_cached_key(|g| g.title.to_lowercase());
    let mut lists = HashMap::new();
    let recent = recent_of(&all, catalog);
    for row in rows.iter_mut() {
        let games: Vec<&GameRow> = if row.key == RECENT {
            recent.iter().collect()
        } else {
            sorted
                .iter()
                .copied()
                .filter(|g| match row.key.as_str() {
                    "all" => true,
                    FAVOURITES => g.favourite,
                    system => g.system == system,
                })
                .collect()
        };
        row.fan = crate::games::to_json(games[..games.len().min(FAN)].iter().copied());
        lists.insert(row.key.clone(), crate::games::to_json(games));
    }
    lists.entry(RECENT.to_owned()).or_insert_with(|| crate::games::to_json(&recent));
    lists
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
        .zip(PLACE_HEADINGS)
        .map(|((((key, name), count), status), heading)| LibraryRow {
            kind: Some(Kind::Place),
            key: key.into(),
            name: name.into(),
            heading: heading.into(),
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
        let mark = if engine.name.is_empty() { String::new() } else { engine_mark(&engine.name, engine.libretro, &s.look.body) };
        let runs_on = if engine.name.is_empty() { String::new() } else { words::runs_on(&engine.name, engine.libretro) };
        out.push(LibraryRow {
            kind: Some(Kind::Console),
            key: s.id.clone(),
            name: if s.display_name.is_empty() { s.name.clone() } else { s.display_name.clone() },
            heading: if s.display_name.is_empty() { s.name.clone() } else { s.display_name.clone() },
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
            engine_mark: mark,
            libretro: engine.libretro,
            world: world_line_of(s),
            fan: String::new(),
        });
    }
    out
}

/// The cores named after the Moon and the planets, whose engine line carries their astronomical symbol (§24.6).
pub const SKY_NAMED: [&str; 4] = ["moon", "mercury", "venus", "mars"];

/// The mark before an engine's name, a property of the engine (§24.6): "libretro" for a hosted core, a sky-named core's
/// own symbol, else its maker's body glyph ("shard", "gem") or "planet".
pub fn engine_mark(name: &str, libretro: bool, body: &str) -> String {
    let lower = name.to_lowercase();
    if libretro {
        "libretro".into()
    } else if SKY_NAMED.contains(&lower.as_str()) {
        lower
    } else if body.contains("shard") {
        "shard".into()
    } else if body.contains("gem") {
        "gem".into()
    } else {
        "planet".into()
    }
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
    /// Each row's games as GameModel reads them, by the row's key (`lists_of`).
    pub lists: HashMap<String, String>,
    /// The ES-DE view's settings, and its texts while a theme is the library's look (esde_shelves).
    pub esde_settings: EsdeSettings,
    pub esde: Esde,
    /// What the gamelist rules were given and built, so a choice of the List card lists again without reading.
    pub esde_state: EsdeState,
}

/// One refresh's answer, or the words for why it could not.
pub type Refreshed = Result<Read, String>;

/// A source's first read for an ES-DE theme: the systems, the games and the player's records without the metadata, the
/// plays or the media (so the grid's Most played, Year and Rating sort by name until then), which the whole read brings after it, so the theme's view is shown first (EmuSen_Qt.md §4.15).
pub struct FirstLook<'a>(pub &'a dyn LibrarySource);

impl LibrarySource for FirstLook<'_> {
    fn systems(&self) -> Result<Vec<SystemRow>, String> {
        self.0.systems()
    }
    fn games(&self) -> Result<Vec<GameFile>, String> {
        self.0.games()
    }
    fn favourites(&self, games: &[GameFile]) -> Result<Vec<String>, String> {
        self.0.favourites(games)
    }
    fn recent(&self, games: &[GameFile]) -> Result<Vec<String>, String> {
        self.0.recent(games)
    }
    fn engines(&self, systems: &[SystemRow]) -> Result<Vec<(String, Engine)>, String> {
        self.0.engines(systems)
    }
    fn played(&self, games: &[GameFile]) -> Result<Vec<(String, i64)>, String> {
        self.0.played(games)
    }
    fn shelves(&self) -> Result<Vec<String>, String> {
        self.0.shelves()
    }
    fn settings(&self) -> Result<serde_json::Value, String> {
        self.0.settings()
    }
    fn collections(&self) -> Result<Vec<Collection>, String> {
        self.0.collections()
    }
    fn folder_links(&self) -> Result<Vec<(String, String)>, String> {
        self.0.folder_links()
    }
}

/// One refresh of a source, run off the GUI thread, every gamelist as it first lists.
pub fn refresh(source: &dyn LibrarySource) -> Refreshed {
    refresh_with(source, &Gamelists::default(), false)
}

/// One refresh, each gamelist listed with the choices the player made of it.
/// The scraper's metadata is read where it is shown: for an ES-DE theme, and for the own look's grid while it sorts by
/// Year or Rating (`by_facts`); games.db's plays, one answer, always.
pub fn refresh_with(source: &dyn LibrarySource, gamelists: &Gamelists, by_facts: bool) -> Refreshed {
    let systems = source.systems()?;
    let mut games = source.games()?;
    let settings = source.settings()?;
    let themed = crate::esde_shelves::settings_of(&settings).themed;
    // A hidden game leaves every view until Settings lists hidden games again (Mistress's `Listed`).
    if !settings.get(SHOW_HIDDEN).and_then(serde_json::Value::as_bool).unwrap_or(false) {
        let hidden: HashSet<String> = source.hidden(&games)?.into_iter().collect();
        games.retain(|g| !hidden.contains(&g.path));
    }
    let favourites = source.favourites(&games)?;
    let recent = source.recent(&games)?;
    let engines = source.engines(&systems)?;
    let covers = source.media(&systems, &games, &[COVER])?;
    let played_games: Vec<GameFile> = games.iter().filter(|g| recent.contains(&g.path)).cloned().collect();
    let heroes = source.media(&systems, &played_games, &HERO)?;
    let played = source.played(&games)?;
    let plays = source.plays(&games)?;
    let metadata = if themed || by_facts { source.metadata(&games)? } else { Vec::new() };
    // Read for the consoles that start empty alone, a call each, so Start Console's pane says what the runtime finds.
    let firmware = systems.iter().filter(|s| s.starts_empty && !s.engines.is_empty()).map(|s| (s.id.clone(), firmware_line(source.firmware(&s.id)))).collect();
    let names = source.names(&games)?;
    let catalog = Catalog { games, favourites, recent, covers, heroes, played, settings, plays, metadata, firmware, names };
    let shelves = source.shelves()?;
    let mut shelved: Vec<SystemRow> = systems.clone();
    shelved.sort_by_key(|s| shelves.iter().position(|id| id == &s.id).unwrap_or(usize::MAX));
    let mut rows = rows(&shelved, &catalog, &engines);
    let lists = lists_of(&mut rows, &systems, &catalog, words::now());
    let esde_settings = crate::esde_shelves::settings_of(&catalog.settings);
    let (esde, esde_state) = if esde_settings.themed {
        let on_shelves: Vec<SystemRow> = shelved.iter().filter(|s| shelves.is_empty() || shelves.contains(&s.id)).cloned().collect();
        let base = crate::esde_shelves::read(source, &on_shelves, &catalog, &catalog.settings)?;
        crate::esde_shelves::build(Arc::new(base), gamelists)
    } else {
        (Esde::default(), EsdeState::default())
    };
    Ok(Read { systems, catalog, rows, lists, esde_settings, esde, esde_state })
}

/// A system's firmware as Start Console's pane says it, in Mistress's words (her `FirmwareLine`): found, or not found;
/// nothing where none is needed; and where the runtime cannot say, that it could not, never read as none needed.
pub fn firmware_line(read: Result<Vec<emusen_crystal_client::shapes::FirmwareFile>, String>) -> String {
    use crate::firmware::{have, Have};
    match read.as_deref().map(have) {
        Ok(Have::Found) => "✓ Firmware found.".to_owned(),
        Ok(Have::Missing) => "✗ Not found: add it in Settings → Firmware".to_owned(),
        Ok(Have::NotNeeded | Have::Nothing) => String::new(),
        Err(why) => format!("⚠ EmuSen's runtime could not say whether you have it: {why}"),
    }
}

/// A cover's shape where the systems table gives none: the kit's, 3 wide by 4 tall.
pub use emusen_kit_core::page::COVER_SHAPE;

/// A console's cover shape as a theme draws it, width over height: the systems table's `cover_aspect` (height over width)
/// turned, or `COVER_SHAPE` where it gives none. The one place the turn is made (EmuSen_Qt.md §3.4.2).
pub fn cover_shape(system: Option<&SystemRow>) -> f64 {
    system.map(|s| s.cover_aspect).filter(|a| *a > 0.0).map_or(COVER_SHAPE, |a| 1.0 / a)
}

/// The view's state: the rows with the selection kept across refreshes, and the last refresh's problem.
#[derive(Clone, Debug, Default)]
pub struct LibraryView {
    pub rows: KeyedRows<LibraryRow>,
    pub systems: Vec<SystemRow>,
    pub catalog: Catalog,
    pub lists: HashMap<String, String>,
    pub esde_settings: EsdeSettings,
    pub esde: Esde,
    pub esde_state: EsdeState,
    /// The player's choices of each gamelist, kept across refreshes.
    pub gamelists: Gamelists,
    pub problem: String,
}

impl LibraryView {
    /// A game's console's name, as its facts begin (`GameRow::console`); empty for a game the library does not hold.
    pub fn console_of(&self, path: &str) -> String {
        let system = self.catalog.games.iter().find(|g| g.path == path).and_then(|g| g.system.as_deref());
        let of = system.and_then(|id| self.systems.iter().find(|s| s.id == id));
        of.map(console_name).unwrap_or_default()
    }

    /// A game's row as every game's list holds it, by its file; none for a game the library does not hold.
    pub fn game(&self, path: &str) -> Option<GameRow> {
        games_of_at(ALL, &self.systems, &self.catalog, words::now()).into_iter().find(|g| g.path == path)
    }

    /// The consoles that start with nothing in them, each once by its first system, in Mistress's order: by maker, then
    /// year; each with the engine its row shows, else the first that runs it (Mistress's `StartConsoleEntries`).
    pub fn startable(&self) -> Vec<crate::home_card::Startable> {
        let mut seen: Vec<&SystemRow> = Vec::new();
        for s in self.systems.iter().filter(|s| s.starts_empty && !s.engines.is_empty()) {
            let console = if s.console.is_empty() { &s.id } else { &s.console };
            if !seen.iter().any(|t| (if t.console.is_empty() { &t.id } else { &t.console }).eq_ignore_ascii_case(console)) {
                seen.push(s);
            }
        }
        seen.sort_by(|a, b| (a.manufacturer.to_lowercase(), a.release_year).cmp(&(b.manufacturer.to_lowercase(), b.release_year)));
        let row = |id: &str| self.rows.rows().iter().find(|r| r.key == id).map(|r| r.engine_name.clone()).filter(|e| !e.is_empty());
        seen.into_iter()
            .map(|s| crate::home_card::Startable {
                console: if s.console.is_empty() { s.id.clone() } else { s.console.clone() },
                name: console_name(s),
                engine: row(&s.id).unwrap_or_else(|| s.engines[0].clone()),
                firmware: self.catalog.firmware.iter().find(|(id, _)| *id == s.id).map(|(_, line)| line.clone()).unwrap_or_default(),
                system: s.id.clone(),
            })
            .collect()
    }

    /// The console an ES-DE shelf shows, where it starts empty: Start Console's row first in its Select menu (Mistress's
    /// `AddStartConsoleOption`).
    pub fn startable_on_shelf(&self, shelf: &str) -> Option<crate::home_card::Startable> {
        let console = self.systems.iter().find(|s| s.esde_system == shelf).map(|s| if s.console.is_empty() { &s.id } else { &s.console })?;
        self.startable().into_iter().find(|s| s.console.eq_ignore_ascii_case(console))
    }

    /// "11 consoles · 1,566 games": the consoles shown and every game.
    pub fn summary(&self) -> String {
        let consoles = self.rows.rows().iter().filter(|r| r.kind == Some(Kind::Console)).count();
        words::summary(consoles, self.catalog.games.len())
    }

    /// A row's games as GameModel reads them, from the last refresh; none for a key that is no row.
    pub fn games_text(&self, key: &str) -> &str {
        self.lists.get(key).map_or("[]", String::as_str)
    }

    /// A view with the places alone, before anything is read.
    pub fn empty() -> LibraryView {
        let mut view = LibraryView::default();
        view.rows.replace(rows(&[], &Catalog::default(), &[]));
        view
    }

    /// A refresh's answer applied: new rows with the selection kept, or the rows kept and the problem said (§7.9).
    pub fn apply(&mut self, refreshed: Refreshed) {
        drop(self.swap_in(refreshed));
    }

    /// `apply`, handing back what the read replaced, so the GUI thread can drop it elsewhere (EmuSen_Qt.md §4.15).
    pub fn swap_in(&mut self, refreshed: Refreshed) -> Option<Read> {
        use std::mem::replace;
        match refreshed {
            Ok(read) => {
                self.problem.clear();
                Some(Read {
                    rows: self.rows.replace(read.rows),
                    systems: replace(&mut self.systems, read.systems),
                    catalog: replace(&mut self.catalog, read.catalog),
                    lists: replace(&mut self.lists, read.lists),
                    esde_settings: replace(&mut self.esde_settings, read.esde_settings),
                    esde: replace(&mut self.esde, read.esde),
                    esde_state: replace(&mut self.esde_state, read.esde_state),
                })
            }
            Err(words) => {
                self.problem = format!("The library could not be read: {words}");
                None
            }
        }
    }
}
