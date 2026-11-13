//! The library as an ES-DE theme sees it (EmuSen_Qt.md §4.6): each shelf under its ES-DE names with every field a theme
//! can show of its games, the media found for them as an index, and the ES-DE settings, each written as one text once a
//! refresh, off the GUI thread, so a painter parses it on its own thread and nothing on the GUI thread grows with the
//! library. Sorting, filters, folders and collections are the gamelist rules' (gamelists.rs), applied here.

use crate::gamelists::{write_listing, Collection, Counts, Gamelists, Settings, Shelf};
use crate::library::{title_of, Catalog, LibrarySource};
use emusen_crystal_client::shapes::SystemRow;
use emusen_platform_client::library::GameFile;
use emusen_esde::shelves::GameRow;
use std::borrow::Cow;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

/// A game's plays as games.db records them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Plays {
    pub count: i64,
    pub seconds: i64,
}

/// The ES-DE settings the view reads, from the player's settings: whether a theme is the library's look, the theme,
/// its stored choices, and the interface's and collections' settings as Galaxia's model holds them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EsdeSettings {
    /// `LibraryStyle` is "Theme" and `BigPictureTheme` names a folder.
    pub themed: bool,
    /// `{"style", "theme", "choices", "interface", "collections", "show_hidden", "volume"}`.
    pub text: String,
}

/// `AppSettings`' volume, a share of full: the one EmuSen plays sound at, the media viewer's video's included (EmuSen_Qt.md §4.14).
pub const VOLUME: &str = "Volume";

/// The volume the settings give, a share of full; full where they give none.
pub fn volume_of(settings: &Value) -> f64 {
    settings.get(VOLUME).and_then(Value::as_f64).map_or(1.0, |v| v.clamp(0.0, 1.0))
}

/// The settings for the ES-DE view: the choices are those stored under the theme folder's full path less a trailing
/// separator, as Mistress's `ThemeSettingsWindow.Key` keys them.
pub fn settings_of(settings: &Value) -> EsdeSettings {
    let text = |key: &str| settings.get(key).and_then(Value::as_str).unwrap_or("").to_owned();
    let object = |v: Option<&Value>| v.filter(|v| v.is_object()).cloned().unwrap_or_else(|| json!({}));
    let (style, theme) = (text("LibraryStyle"), text("BigPictureTheme"));
    let key = theme_key(&theme);
    let choices = object(settings.get("BigPicture").and_then(|m| m.get(&key)));
    let out = json!({
        "style": style,
        "theme": theme,
        "choices": choices,
        "interface": object(settings.get("BigPictureInterface")),
        "collections": object(settings.get("BigPictureCollections")),
        "show_hidden": settings.get(crate::library::SHOW_HIDDEN).and_then(Value::as_bool).unwrap_or(false),
        "volume": settings.get(VOLUME).and_then(Value::as_f64),
    });
    EsdeSettings { themed: style == "Theme" && !theme.trim().is_empty(), text: out.to_string() }
}

/// A theme folder's key among the stored choices: its absolute path without a trailing separator.
pub fn theme_key(theme: &str) -> String {
    if theme.trim().is_empty() {
        return String::new();
    }
    let full = std::path::absolute(theme).map_or_else(|_| theme.to_owned(), |p| p.to_string_lossy().into_owned());
    match full.trim_end_matches('/') {
        "" => "/".to_owned(),
        trimmed => trimmed.to_owned(),
    }
}

/// The media found for a refresh, keyed (ES-DE system, ES-DE type, name), where a name is the file's path below its
/// type's folder less the extension, '/'-separated: ES-DE's `downloaded_media/<system>/<folder>/<name>.<ext>`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MediaIndex {
    by: BTreeMap<String, BTreeMap<String, BTreeMap<String, String>>>,
}

impl MediaIndex {
    /// Adds a file unless the key has one already, so the root walked first wins.
    pub fn insert(&mut self, system: &str, kind: &str, name: &str, path: &str) {
        let names = self.by.entry(system.to_owned()).or_default().entry(kind.to_owned()).or_default();
        names.entry(name.to_owned()).or_insert_with(|| path.to_owned());
    }

    /// Puts a file at the key whatever it held: the scraper's own answer for a game's cover.
    pub fn set(&mut self, system: &str, kind: &str, name: &str, path: &str) {
        self.by.entry(system.to_owned()).or_default().entry(kind.to_owned()).or_default().insert(name.to_owned(), path.to_owned());
    }

    /// The file of a type for the first of `names` that has one.
    pub fn find(&self, system: &str, kind: &str, names: &[String]) -> Option<&str> {
        let of = self.by.get(system)?.get(kind)?;
        names.iter().find_map(|n| of.get(n)).map(String::as_str)
    }

    /// Every file of one type folder below `root`, sorted by name so a name with two extensions keeps the first.
    pub fn walk(&mut self, root: &Path, system: &str, kind: &str, folder: &str) {
        let top = root.join(system).join(folder);
        let mut files = Vec::new();
        list_files(&top, &mut files);
        files.sort();
        for file in files {
            let Ok(below) = file.strip_prefix(&top) else { continue };
            let name = below.with_extension("").to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");
            self.insert(system, kind, &name, &file.to_string_lossy());
        }
    }

    /// The types some of a shelf's games have a file of, in name order: the variant triggers (MediaPresence).
    pub fn present(&self, system: &str, names: &HashSet<String>) -> Vec<String> {
        let Some(kinds) = self.by.get(system) else { return Vec::new() };
        kinds.iter().filter(|(_, files)| files.keys().any(|n| names.contains(n))).map(|(k, _)| k.clone()).collect()
    }

    /// `{system: {type: {name: path}}}`.
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.by).unwrap_or_else(|_| "{}".into())
    }

    pub fn is_empty(&self) -> bool {
        self.by.is_empty()
    }
}

/// The files below `dir`, at any depth; a folder that cannot be read lists nothing.
fn list_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let path = e.path();
        match e.file_type() {
            Ok(t) if t.is_dir() => list_files(&path, out),
            Ok(_) if path.is_file() => out.push(path),
            _ => {}
        }
    }
}

/// The names a game's files may have below a type folder: its folder then its stem, and its stem alone.
pub fn names_of(file: &str, folder_path: &str) -> Vec<String> {
    let stem = title_of(file);
    if folder_path.is_empty() {
        vec![stem]
    } else {
        vec![format!("{folder_path}/{stem}"), stem]
    }
}

/// The ES-DE view's three texts: the shelves, the media index and each shelf's media presence.
#[derive(Clone, Debug, PartialEq)]
pub struct Esde {
    /// Each shelf as listed now (`gamelists::write_listing`), in the carousel's order.
    pub shelves: String,
    /// `MediaIndex::to_json`.
    pub media: String,
    /// `{shelf name: [type]}`.
    pub presence: String,
}

impl Default for Esde {
    fn default() -> Self {
        Esde { shelves: "[]".into(), media: "{}".into(), presence: "{}".into() }
    }
}

/// One game, or a folder entry, as the painter reads it: every field Mistress's `SceneGame` carried.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EsdeGame {
    pub name: String,
    pub file: String,
    /// The game's own console by its ES-DE names, which a collection lists beside others.
    pub esde_system: String,
    pub esde_full_name: String,
    pub sort_name: Option<String>,
    pub custom_sort_name: Option<String>,
    pub description: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub genre: Option<String>,
    pub players: Option<String>,
    /// The metadata's `yyyy-MM-dd`.
    pub release_date: Option<String>,
    /// 0 to 1.
    pub rating: Option<f64>,
    /// Seconds since 1970; 0 for never.
    pub last_played: i64,
    pub play_count: i64,
    /// Seconds.
    pub play_time: i64,
    pub favourite: bool,
    pub completed: bool,
    pub kid_game: bool,
    pub broken: bool,
    pub hidden: bool,
    pub not_counted: bool,
    pub hide_metadata: bool,
    pub controller: Option<String>,
    /// The alternative emulator's name, where the player chose one.
    pub emulator: Option<String>,
    pub manual: bool,
    pub folder: bool,
    /// The folders between the console folder and the file, or a folder entry's own path; '/'-separated.
    pub folder_path: String,
    /// A folder's ES-DE folder link: the file A launches in place of entering it.
    pub folder_link: Option<String>,
    /// A grouped custom collection listed as a folder.
    pub is_collection: bool,
    /// For a collection's folder entry, the game whose media it shows.
    pub face: Option<String>,
}

impl EsdeGame {
    /// The row as the shelves' text carries it: every field but the description, which `description_of` gives on demand.
    pub fn row(&self) -> GameRow<'_> {
        self.row_of(false)
    }

    /// The row with its description, as a collection's folder entry carries its own.
    pub fn row_whole(&self) -> GameRow<'_> {
        self.row_of(true)
    }

    /// An entry read back from its text, as the painter hands one in to be entered: the shelves' own row, so a key it does
    /// not name is an error, never a field read as absent.
    pub fn from_text(text: &str) -> Result<EsdeGame, String> {
        let row: GameRow<'_> = serde_json::from_str(text).map_err(|e| format!("The entry could not be read: {e}"))?;
        Ok(EsdeGame {
            name: row.name.into_owned(),
            file: row.file.into_owned(),
            folder: row.folder,
            folder_path: row.folder_path.into_owned(),
            is_collection: row.is_collection,
            ..EsdeGame::default()
        })
    }

    // The shelves' row, borrowed (emusen-esde's `shelves`, the schema both sides keep).
    fn row_of(&self, whole: bool) -> GameRow<'_> {
        fn text(t: &Option<String>) -> Option<Cow<'_, str>> {
            t.as_deref().map(Cow::Borrowed)
        }
        GameRow {
            alt_emulator: self.emulator.is_some(),
            broken: self.broken,
            completed: self.completed,
            controller: text(&self.controller),
            custom_sort_name: text(&self.custom_sort_name),
            description: if whole { text(&self.description) } else { None },
            developer: text(&self.developer),
            emulator: text(&self.emulator),
            esde_full_name: Cow::Borrowed(&self.esde_full_name),
            esde_system: Cow::Borrowed(&self.esde_system),
            face: text(&self.face),
            favourite: self.favourite,
            file: Cow::Borrowed(&self.file),
            folder: self.folder,
            folder_link: text(&self.folder_link),
            folder_path: Cow::Borrowed(&self.folder_path),
            genre: text(&self.genre),
            hidden: self.hidden,
            hide_metadata: self.hide_metadata,
            in_collection: false,
            is_collection: self.is_collection,
            kid_game: self.kid_game,
            last_played: self.last_played,
            manual: self.manual,
            name: Cow::Borrowed(&self.name),
            not_counted: self.not_counted,
            play_count: self.play_count,
            play_time: self.play_time,
            players: text(&self.players),
            publisher: text(&self.publisher),
            rating: self.rating,
            release_date: text(&self.release_date),
            sort_name: text(&self.sort_name),
        }
    }
}

/// What a refresh read for the ES-DE view, before the gamelist rules list it: the consoles' shelves in the systems
/// sorting's order, the player's collections, the recently played, the folder links, the settings and the media.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Base {
    pub regular: Vec<Shelf>,
    pub custom: Vec<Collection>,
    pub recent: Vec<String>,
    /// Each folder link by the folder's path on disk, to the game's file.
    pub links: HashMap<String, String>,
    pub settings: Settings,
    pub media: MediaIndex,
}

/// What the List card reads of a list without listing it again: its counts, letters, sort and filter.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Summary {
    pub key: String,
    pub full_name: String,
    pub collection: bool,
    pub favourites_shelf: bool,
    pub counts: Counts,
    pub letters: Vec<(String, usize)>,
    /// Each entry's quick-selector letter, so a jump starts from the focused entry's.
    pub entry_letters: Vec<String>,
    pub sort: String,
    pub favourites_only: bool,
}

/// The ES-DE view's state after a write: what was read, the shelves the rules built from it, each list's summary.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EsdeState {
    pub base: Arc<Base>,
    pub shelves: Vec<Shelf>,
    pub summaries: HashMap<String, Summary>,
}

impl EsdeState {
    pub fn shelf(&self, name: &str) -> Option<&Shelf> {
        self.shelves.iter().find(|s| s.name == name)
    }

    /// A game's description by its file, from its console's shelf.
    pub fn description_of(&self, file: &str) -> Option<&str> {
        self.base.regular.iter().flat_map(|s| s.games.iter()).find(|g| g.file == file).and_then(|g| g.description.as_deref())
    }
}

/// The ES-DE view's reads for a refresh: each shelf of `shelved` with games, its games the library's less the hidden
/// unless the player shows them, each with the scraper's metadata and games.db's plays as the catalog read them, and its folder.
pub fn read(source: &dyn LibrarySource, shelved: &[SystemRow], catalog: &Catalog, settings: &Value) -> Result<Base, String> {
    let show_hidden = settings.get(crate::library::SHOW_HIDDEN).and_then(Value::as_bool).unwrap_or(false);
    let collections = Settings::from_json(settings.get("BigPictureCollections").unwrap_or(&Value::Null));
    let metadata: HashMap<&str, &Metadata> = catalog.metadata.iter().map(|(p, m)| (p.as_str(), m)).collect();
    let plays: HashMap<&str, Plays> = catalog.plays.iter().map(|(p, n)| (p.as_str(), *n)).collect();
    let systems: Vec<String> = shelved.iter().map(|s| s.esde_system.clone()).filter(|s| !s.is_empty()).collect();
    let mut media = source.media_index(&systems)?;
    let by_id: HashMap<&str, &SystemRow> = shelved.iter().map(|s| (s.id.as_str(), s)).collect();
    let game_of: HashMap<&str, &GameFile> = catalog.games.iter().map(|g| (g.path.as_str(), g)).collect();
    for (path, cover) in &catalog.covers {
        let Some(g) = game_of.get(path.as_str()) else { continue };
        let Some(s) = g.system.as_deref().and_then(|id| by_id.get(id)) else { continue };
        media.set(&s.esde_system, COVER_TYPE, &names_of(path, &g.folder)[0], cover);
    }
    let favourites: HashSet<&str> = catalog.favourites.iter().map(String::as_str).collect();
    let played: HashMap<&str, i64> = catalog.played.iter().map(|(p, at)| (p.as_str(), *at)).collect();
    let mut by_system: HashMap<&str, Vec<&GameFile>> = HashMap::new();
    for g in &catalog.games {
        if let Some(s) = g.system.as_deref() {
            by_system.entry(s).or_default().push(g);
        }
    }
    let mut regular = Vec::new();
    for s in shelved {
        let Some(files) = by_system.get(s.id.as_str()) else { continue };
        let games: Vec<EsdeGame> = files
            .iter()
            .map(|g| {
                let facts = Facts {
                    metadata: metadata.get(g.path.as_str()).copied(),
                    plays: plays.get(g.path.as_str()).copied().unwrap_or_default(),
                    last_played: played.get(g.path.as_str()).copied().unwrap_or(0),
                    favourite: favourites.contains(g.path.as_str()),
                    manual: media.find(&s.esde_system, "manual", &names_of(&g.path, &g.folder)).is_some(),
                };
                game_of_file(g, s, &facts)
            })
            .filter(|g| show_hidden || !g.hidden)
            .collect();
        if games.is_empty() {
            continue;
        }
        let flatten = collections.flattened.iter().any(|f| *f == s.esde_system);
        regular.push(Shelf { flatten, ..Shelf::regular(&s.id, &s.esde_system, &s.esde_full_name, games) });
    }
    let years: HashMap<String, i32> = shelved.iter().rev().map(|s| (s.esde_system.clone(), s.release_year)).filter(|(_, y)| *y > 0).collect();
    let sorting = settings.get("BigPictureInterface").and_then(|i| i.get("SystemsSorting")).and_then(Value::as_str).unwrap_or("release");
    crate::gamelists::systems_order(&mut regular, sorting, &years);
    let listed: HashSet<&str> = catalog.games.iter().map(|g| g.path.as_str()).collect();
    let links = source.folder_links()?.into_iter().filter(|(_, file)| listed.contains(file.as_str())).collect();
    Ok(Base { regular, custom: source.collections()?, recent: catalog.recent.clone(), links, settings: collections, media })
}

/// ES-DE's type for a cover, the library's `COVER`.
const COVER_TYPE: &str = crate::library::COVER;

/// The shelves the rules build from what was read, each listed as `gamelists` chose, as the texts and the state the
/// List card reads; pure, so a test gives the parts itself.
pub fn build(base: Arc<Base>, gamelists: &Gamelists) -> (Esde, EsdeState) {
    let themed = |name: &str| gamelists.theme_folders.iter().any(|t| t == name);
    let shelves: Vec<Shelf> = crate::gamelists::with_collections(base.regular.clone(), &base.custom, &base.settings, &themed, &base.recent)
        .into_iter()
        .filter(|s| !s.games.is_empty() || s.folders.as_ref().is_some_and(|f| !f.is_empty()))
        .collect();
    let (mut texts, mut presence, mut summaries) = (b"[".to_vec(), Map::new(), HashMap::new());
    for shelf in &shelves {
        let listing = gamelists.listing(shelf, &base.links, &base.settings);
        if texts.len() > 1 {
            texts.push(b',');
        }
        write_listing(&mut texts, shelf, &listing);
        presence.insert(shelf.name.clone(), json!(present(&base.media, &shelf.games)));
        let summary = Summary {
            key: listing.key.clone(),
            full_name: shelf.full_name.clone(),
            collection: shelf.is_collection(),
            favourites_shelf: shelf.name == crate::gamelists::FAVORITES && shelf.kind == crate::gamelists::Kind::Auto,
            counts: listing.counts,
            entry_letters: listing.entries.iter().map(|g| crate::gamelists::first_letter(crate::gamelists::sort_key(g))).collect(),
            letters: listing.letters,
            sort: listing.sort.label(),
            favourites_only: gamelists.is_favourites_only(shelf),
        };
        summaries.insert(shelf.name.clone(), summary);
    }
    texts.push(b']');
    let esde = Esde { shelves: String::from_utf8(texts).unwrap_or_default(), media: base.media.to_json(), presence: Value::Object(presence).to_string() };
    (esde, EsdeState { base, shelves, summaries })
}

/// The media types some of the games have, each looked up under its own console: the variant triggers.
fn present(media: &MediaIndex, games: &[EsdeGame]) -> Vec<String> {
    let mut names: HashMap<&str, HashSet<String>> = HashMap::new();
    for g in games {
        names.entry(g.esde_system.as_str()).or_default().extend(names_of(&g.file, &g.folder_path));
    }
    let found: BTreeSet<String> = names.iter().flat_map(|(system, names)| media.present(system, names)).collect();
    found.into_iter().collect()
}

/// A game's metadata as the view reads it: each scraper key's chosen value (EmuSen_Backend.md §30.3), the empty ones left
/// out. A game's answer is made this as it is read, so the library's answers are never held whole (EmuSen_Qt.md §4.15).
pub type Metadata = BTreeMap<String, String>;

/// The scraper's keys for a game's release date (`yyyy-MM-dd`) and its rating (0 to 1).
pub const RELEASE_DATE: &str = "releasedate";
pub const RATING: &str = "rating";

/// A game's release date as the scraper keeps it; the one reader of it, which the view, the grid's sorts and a game's
/// page share.
pub fn release_date(m: &Metadata) -> Option<String> {
    m.get(RELEASE_DATE).filter(|d| !d.is_empty()).cloned()
}

/// A game's rating from 0 to 1, as the scraper keeps it.
pub fn rating(m: &Metadata) -> Option<f64> {
    m.get(RATING).and_then(|r| r.parse::<f64>().ok()).filter(|r| r.is_finite()).map(|r| r.clamp(0.0, 1.0))
}

/// A release date's year, its first four digits; none for a date that does not begin with them.
pub fn year_of(date: &str) -> Option<String> {
    let year: String = date.chars().take(4).collect();
    (year.len() == 4 && year.chars().all(|c| c.is_ascii_digit())).then_some(year)
}

/// The scraper's `metadata` answer, `{key: {"value", …}}`, as the view keeps it.
pub fn metadata_of(answer: &Value) -> Metadata {
    let chosen = |(k, f): (&String, &Value)| f.get("value")?.as_str().filter(|v| !v.is_empty()).map(|v| (k.clone(), v.to_owned()));
    answer.as_object().into_iter().flatten().filter_map(chosen).collect()
}

/// What a game's row is made from besides its file.
struct Facts<'a> {
    metadata: Option<&'a Metadata>,
    plays: Plays,
    last_played: i64,
    favourite: bool,
    manual: bool,
}

/// One game: the metadata by the scraper's keys (EmuSen_Backend.md §30.3, `metadata`), its edit else ScreenScraper's
/// else the default already chosen there.
fn game_of_file(g: &GameFile, s: &SystemRow, f: &Facts) -> EsdeGame {
    let value = |key: &str| f.metadata.and_then(|m| m.get(key)).map(String::as_str);
    let text = |key: &str| value(key).map(str::to_owned);
    let flag = |key: &str| value(key) == Some(crate::edits::YES);
    EsdeGame {
        name: value("name").map_or_else(|| title_of(&g.path), str::to_owned),
        file: g.path.clone(),
        esde_system: s.esde_system.clone(),
        esde_full_name: s.esde_full_name.clone(),
        sort_name: text("sortname"),
        custom_sort_name: text("collectionsortname"),
        description: text("description"),
        developer: text("developer"),
        publisher: text("publisher"),
        genre: text("genre"),
        players: text("players"),
        release_date: f.metadata.and_then(release_date),
        rating: f.metadata.and_then(rating),
        last_played: f.last_played,
        play_count: f.plays.count,
        play_time: f.plays.seconds,
        favourite: f.favourite,
        completed: flag("completed"),
        kid_game: flag("kidgame"),
        broken: flag("broken"),
        hidden: flag(crate::edits::HIDDEN),
        not_counted: flag("nogamecount"),
        hide_metadata: flag("hidemetadata"),
        controller: text("controller"),
        emulator: text(crate::edits::ALT_EMULATOR),
        manual: f.manual,
        folder_path: g.folder.clone(),
        ..EsdeGame::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_entry_is_read_back_through_the_shelves_row() {
        let game = EsdeGame { name: "Cobalt Harbor".into(), file: "/roms/nes/c.nes".into(), folder: true, folder_path: "nes/Harbors".into(), is_collection: true, ..EsdeGame::default() };
        let text = serde_json::to_string(&game.row()).unwrap();
        assert_eq!(EsdeGame::from_text(&text), Ok(game), "the five fields the gamelists enter by, as written");
        assert!(EsdeGame::from_text(r#"{"names":"Cobalt Harbor"}"#).is_err(), "a key the row does not name is an error");
        assert!(EsdeGame::from_text(r#"{"folder":"yes"}"#).is_err(), "a field of another type is an error");
    }
}
