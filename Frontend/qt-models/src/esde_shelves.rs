//! The library as an ES-DE theme sees it (EmuSen_Qt.md §4.6): each shelf under its ES-DE names with every field a theme
//! can show of its games, the media found for them as an index, and the ES-DE settings, each written as one text once a
//! refresh, off the GUI thread, so a painter parses it on its own thread and nothing on the GUI thread grows with the
//! library. Sorting, filters, folders and collections are the gamelist rules' (gamelists.rs, after this).

use crate::library::{title_of, Catalog, LibrarySource};
use emusen_crystal_client::shapes::SystemRow;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

/// A shelf's kind as the painter reads it: a console's own games; the collections (gamelists.rs) add the others.
pub const REGULAR: &str = "regular";

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
    /// `{"style", "theme", "choices", "interface", "collections", "show_hidden"}`.
    pub text: String,
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
        "show_hidden": settings.get("ShowHiddenGames").and_then(Value::as_bool).unwrap_or(false),
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
    /// `[{"system", "esde_system", "esde_full_name", "kind", "games": [{...}]}]`.
    pub shelves: String,
    /// `MediaIndex::to_json`.
    pub media: String,
    /// `{esde_system: [type]}`.
    pub presence: String,
}

impl Default for Esde {
    fn default() -> Self {
        Esde { shelves: "[]".into(), media: "{}".into(), presence: "{}".into() }
    }
}

/// The ES-DE view's texts for a refresh: each shelf of `shelved` with games, in that order, its games in the library's
/// order, each with the scraper's metadata, games.db's plays and the media index's manual.
pub fn read(source: &dyn LibrarySource, shelved: &[SystemRow], catalog: &Catalog) -> Result<Esde, String> {
    let metadata: HashMap<String, Value> = source.metadata(&catalog.games)?.into_iter().collect();
    let plays: HashMap<String, Plays> = source.plays(&catalog.games)?.into_iter().collect();
    let systems: Vec<String> = shelved.iter().map(|s| s.esde_system.clone()).filter(|s| !s.is_empty()).collect();
    let mut media = source.media_index(&systems)?;
    let by_id: HashMap<&str, &SystemRow> = shelved.iter().map(|s| (s.id.as_str(), s)).collect();
    let system_of: HashMap<&str, &str> = catalog.games.iter().filter_map(|g| Some((g.path.as_str(), g.system.as_deref()?))).collect();
    for (path, cover) in &catalog.covers {
        let Some(s) = system_of.get(path.as_str()).and_then(|id| by_id.get(id)) else { continue };
        media.set(&s.esde_system, COVER_TYPE, &names_of(path, "")[0], cover);
    }
    Ok(texts(shelved, catalog, &metadata, &plays, &media))
}

/// ES-DE's type for a cover, the library's `COVER`.
const COVER_TYPE: &str = crate::library::COVER;

/// The texts from what was read: pure, so a test gives the parts itself.
pub fn texts(shelved: &[SystemRow], catalog: &Catalog, metadata: &HashMap<String, Value>, plays: &HashMap<String, Plays>, media: &MediaIndex) -> Esde {
    let favourites: HashSet<&str> = catalog.favourites.iter().map(String::as_str).collect();
    let played: HashMap<&str, i64> = catalog.played.iter().map(|(p, at)| (p.as_str(), *at)).collect();
    let mut by_system: HashMap<&str, Vec<&emusen_platform_client::library::GameFile>> = HashMap::new();
    for g in &catalog.games {
        if let Some(s) = g.system.as_deref() {
            by_system.entry(s).or_default().push(g);
        }
    }
    let (mut shelves, mut presence) = (Vec::new(), Map::new());
    for s in shelved {
        let Some(games) = by_system.get(s.id.as_str()) else { continue };
        let mut names = HashSet::new();
        let rows: Vec<Value> = games
            .iter()
            .map(|g| {
                let own = names_of(&g.path, "");
                let manual = media.find(&s.esde_system, "manual", &own).is_some();
                names.extend(own);
                let facts = Facts {
                    metadata: metadata.get(&g.path),
                    plays: plays.get(&g.path).copied().unwrap_or_default(),
                    last_played: played.get(g.path.as_str()).copied().unwrap_or(0),
                    favourite: favourites.contains(g.path.as_str()),
                    manual,
                };
                game_json(&g.path, &facts)
            })
            .collect();
        presence.insert(s.esde_system.clone(), json!(media.present(&s.esde_system, &names)));
        shelves.push(json!({
            "system": s.id,
            "esde_system": s.esde_system,
            "esde_full_name": s.esde_full_name,
            "kind": REGULAR,
            "games": rows,
        }));
    }
    Esde { shelves: Value::Array(shelves).to_string(), media: media.to_json(), presence: Value::Object(presence).to_string() }
}

/// What a game's row is made from besides its file.
struct Facts<'a> {
    metadata: Option<&'a Value>,
    plays: Plays,
    last_played: i64,
    favourite: bool,
    manual: bool,
}

/// One game as the painter reads it: every field Mistress's `SceneGame` carried, the metadata by the scraper's keys
/// (EmuSen_Backend.md §30.3, `metadata`), its edit else ScreenScraper's else the default already chosen there.
fn game_json(path: &str, f: &Facts) -> Value {
    let value = |key: &str| f.metadata.and_then(|m| m.get(key)).and_then(|v| v.get("value")).and_then(Value::as_str).filter(|v| !v.is_empty());
    let text = |key: &str| value(key).map_or(Value::Null, |v| json!(v));
    let flag = |key: &str| value(key) == Some("1");
    let rating = value("rating").and_then(|r| r.parse::<f64>().ok()).map_or(Value::Null, |r| json!(r.clamp(0.0, 1.0)));
    json!({
        "file": path,
        "name": value("name").map_or_else(|| title_of(path), str::to_owned),
        "sort_name": text("sortname"),
        "custom_sort_name": text("collectionsortname"),
        "description": text("description"),
        "developer": text("developer"),
        "publisher": text("publisher"),
        "genre": text("genre"),
        "players": text("players"),
        "release_date": text("releasedate"),
        "rating": rating,
        "last_played": f.last_played,
        "play_count": f.plays.count,
        "play_time": f.plays.seconds,
        "favourite": f.favourite,
        "completed": flag("completed"),
        "kid_game": flag("kidgame"),
        "broken": flag("broken"),
        "hidden": flag("hidden"),
        "not_counted": flag("nogamecount"),
        "hide_metadata": flag("hidemetadata"),
        "controller": text("controller"),
        "alt_emulator": value("altemulator").is_some(),
        "emulator": text("altemulator"),
        "manual": f.manual,
        "folder": false,
        "folder_path": "",
        "folder_link": Value::Null,
    })
}
