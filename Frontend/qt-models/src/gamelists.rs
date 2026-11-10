//! ES-DE's gamelist rules over the library as a theme sees it (EmuSen_Qt.md §4.7; EmuSen_BigPicture.md §22, §30): the
//! sort orders, the filters, the quick selector's letters, the random entry, a console's folders as entries entered and
//! left, the automatic and custom collections as shelves, and the counts a gamelistinfo shows. Mistress's
//! `GamelistOptions`, `CollectionShelves` and `ThemedLibrary.Gamelists`/`.Folders` as plain data, with no view behind
//! them: the view model writes what they list as text, off the GUI thread.

use crate::esde_shelves::EsdeGame;
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// ES-DE's documented sort keys, in its user guide's order; System applies to collections only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SortKey {
    Name,
    Rating,
    ReleaseDate,
    Developer,
    Publisher,
    Genre,
    Players,
    LastPlayed,
    TimesPlayed,
    PlayTime,
    System,
}

impl SortKey {
    pub const ALL: [SortKey; 11] = [
        SortKey::Name,
        SortKey::Rating,
        SortKey::ReleaseDate,
        SortKey::Developer,
        SortKey::Publisher,
        SortKey::Genre,
        SortKey::Players,
        SortKey::LastPlayed,
        SortKey::TimesPlayed,
        SortKey::PlayTime,
        SortKey::System,
    ];

    fn word(self) -> &'static str {
        match self {
            SortKey::Name => "name",
            SortKey::Rating => "rating",
            SortKey::ReleaseDate => "release date",
            SortKey::Developer => "developer",
            SortKey::Publisher => "publisher",
            SortKey::Genre => "genre",
            SortKey::Players => "players",
            SortKey::LastPlayed => "last played",
            SortKey::TimesPlayed => "times played",
            SortKey::PlayTime => "play time",
            SortKey::System => "system",
        }
    }
}

/// One of ES-DE's sort orders: a key, ascending or descending, the name ascending after it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GameSort {
    pub key: SortKey,
    pub descending: bool,
}

impl Default for GameSort {
    fn default() -> Self {
        GameSort { key: SortKey::Name, descending: false }
    }
}

impl GameSort {
    pub fn new(key: SortKey, descending: bool) -> GameSort {
        GameSort { key, descending }
    }

    /// Every order, each key ascending then descending; System for a collection alone.
    pub fn all(collection: bool) -> Vec<GameSort> {
        SortKey::ALL
            .iter()
            .filter(|k| collection || **k != SortKey::System)
            .flat_map(|k| [GameSort::new(*k, false), GameSort::new(*k, true)])
            .collect()
    }

    /// As ES-DE's DefaultSortOrder setting spells it: "name, ascending".
    pub fn stored(self) -> String {
        format!("{}, {}", self.key.word(), if self.descending { "descending" } else { "ascending" })
    }

    /// The stored spelling with a capital: "Name, ascending".
    pub fn label(self) -> String {
        let stored = self.stored();
        let mut chars = stored.chars();
        chars.next().map_or_else(String::new, |c| c.to_uppercase().chain(chars).collect())
    }

    /// The order a stored spelling names; the default for any other text.
    pub fn parse(stored: &str) -> GameSort {
        let wanted = stored.trim().to_lowercase();
        GameSort::all(true).into_iter().find(|s| s.stored() == wanted).unwrap_or_default()
    }
}

/// The filters ES-DE's user guide lists that the library has data for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FilterField {
    Rating,
    Developer,
    Publisher,
    Genre,
    Players,
    Favorite,
    Completed,
    KidGame,
    Broken,
}

/// A gamelist's filter: a name to find anywhere, and for each field the values kept.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GameFilter {
    pub name: String,
    pub chosen: BTreeMap<FilterField, BTreeSet<String>>,
}

impl GameFilter {
    pub fn is_active(&self) -> bool {
        !self.name.trim().is_empty() || self.chosen.values().any(|v| !v.is_empty())
    }

    /// The filter with a field's values replaced; none removes the field.
    pub fn with(mut self, field: FilterField, values: &[&str]) -> GameFilter {
        if values.is_empty() {
            self.chosen.remove(&field);
        } else {
            self.chosen.insert(field, values.iter().map(|v| (*v).to_owned()).collect());
        }
        self
    }
}

/// The word for a game with no value of a filter's field.
pub const UNKNOWN: &str = "Unknown";
/// The quick selector's entry for favourites sorted on top, and for folders sorted on top.
pub const STAR: &str = "★";
pub const FOLDER_ENTRY: &str = "Folders";

/// Two texts compared as .NET's OrdinalIgnoreCase compares them: each character's capital, then its code.
pub fn cmp_ignore_case(a: &str, b: &str) -> Ordering {
    let upper = |s: &str| s.chars().map(|c| c.to_uppercase().next().unwrap_or(c)).collect::<Vec<_>>();
    upper(a).cmp(&upper(b))
}

/// ES-DE's sortname where the player set one, else the name; the quick selector indexes by it too.
pub fn sort_key(g: &EsdeGame) -> &str {
    g.sort_name.as_deref().filter(|s| !s.trim().is_empty()).unwrap_or(&g.name)
}

fn text(value: &Option<String>) -> Option<&str> {
    value.as_deref().map(str::trim).filter(|v| !v.is_empty())
}

/// "1-4" counts as four players: the highest number the text names.
fn player_count(players: &Option<String>) -> Option<i64> {
    text(players)?.split(['-', ',', '+', ' ', '/']).filter_map(|p| p.parse::<i64>().ok()).max()
}

/// A game's value for a sort key, ordered; none for a game without one, which sorts after those with one.
#[derive(PartialEq, PartialOrd)]
enum Key<'a> {
    Number(f64),
    Text(&'a str),
}

fn key_of(g: &EsdeGame, key: SortKey) -> Option<Key<'_>> {
    match key {
        SortKey::Rating => g.rating.map(Key::Number),
        SortKey::ReleaseDate => text(&g.release_date).map(Key::Text),
        SortKey::Developer => text(&g.developer).map(Key::Text),
        SortKey::Publisher => text(&g.publisher).map(Key::Text),
        SortKey::Genre => text(&g.genre).map(Key::Text),
        SortKey::Players => player_count(&g.players).map(|n| Key::Number(n as f64)),
        SortKey::LastPlayed => (g.last_played > 0).then_some(Key::Number(g.last_played as f64)),
        SortKey::TimesPlayed => Some(Key::Number(g.play_count as f64)),
        SortKey::PlayTime => Some(Key::Number(g.play_time as f64)),
        SortKey::System => (!g.esde_full_name.is_empty()).then_some(Key::Text(&g.esde_full_name)),
        SortKey::Name => Some(Key::Text(&g.name)),
    }
}

fn compare(a: &EsdeGame, b: &EsdeGame, order: GameSort) -> Ordering {
    let by = if order.key == SortKey::Name {
        cmp_ignore_case(sort_key(a), sort_key(b))
    } else {
        match (key_of(a, order.key), key_of(b, order.key)) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Greater,
            (Some(_), None) => return Ordering::Less,
            (Some(Key::Text(x)), Some(Key::Text(y))) => cmp_ignore_case(x, y),
            (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
        }
    };
    if order.descending { by.reverse() } else { by }
}

/// Favourites first when asked, each part in the order's key, games without the key's value after those with it, then
/// the sort name ascending, then the order given.
pub fn sort(games: &mut [EsdeGame], order: GameSort, favourites_first: bool) {
    games.sort_by(|a, b| {
        let favourites = if favourites_first { b.favourite.cmp(&a.favourite) } else { Ordering::Equal };
        favourites.then_with(|| compare(a, b, order)).then_with(|| cmp_ignore_case(sort_key(a), sort_key(b)))
    });
}

/// Half-star steps, as ES-DE draws a rating: "2.5 stars", "1 star".
pub fn stars(rating: f64) -> String {
    let stars = (rating.clamp(0.0, 1.0) * 10.0).round() / 2.0;
    let number = if stars.fract() == 0.0 { format!("{stars:.0}") } else { format!("{stars:.1}") };
    format!("{number} {}", if stars == 1.0 { "star" } else { "stars" })
}

fn yes_no(value: bool) -> String {
    (if value { "Yes" } else { "No" }).to_owned()
}

/// A game's value for a filter, as the filter's page lists it; none for a game without one.
pub fn value_of(g: &EsdeGame, field: FilterField) -> Option<String> {
    match field {
        FilterField::Rating => g.rating.map(stars),
        FilterField::Developer => text(&g.developer).map(str::to_owned),
        FilterField::Publisher => text(&g.publisher).map(str::to_owned),
        FilterField::Genre => text(&g.genre).map(str::to_owned),
        FilterField::Players => text(&g.players).map(str::to_owned),
        FilterField::Favorite => Some(yes_no(g.favourite)),
        FilterField::Completed => Some(yes_no(g.completed)),
        FilterField::KidGame => Some(yes_no(g.kid_game)),
        FilterField::Broken => Some(yes_no(g.broken)),
    }
}

/// The values a filter offers, from the games themselves; none when there is nothing to tell them apart by.
pub fn values(games: &[EsdeGame], field: FilterField) -> Vec<String> {
    let known: Vec<Option<String>> = games.iter().filter(|g| !g.folder).map(|g| value_of(g, field)).collect();
    if known.iter().all(Option::is_none) {
        return Vec::new();
    }
    let mut out: Vec<String> = Vec::new();
    for v in known.into_iter().map(|v| v.unwrap_or_else(|| UNKNOWN.to_owned())) {
        if !out.iter().any(|o| cmp_ignore_case(o, &v) == Ordering::Equal) {
            out.push(v);
        }
    }
    if out.len() < 2 {
        return Vec::new();
    }
    if field == FilterField::Rating {
        let number = |v: &str| if v == UNKNOWN { f64::MAX } else { v.split(' ').next().and_then(|n| n.parse().ok()).unwrap_or(f64::MAX) };
        out.sort_by(|a, b| number(a).partial_cmp(&number(b)).unwrap_or(Ordering::Equal));
    } else {
        out.sort_by(|a, b| (a == UNKNOWN).cmp(&(b == UNKNOWN)).then_with(|| cmp_ignore_case(a, b)));
    }
    out
}

/// Every chosen field holds, a field held when the game's value is one chosen; the name matches anywhere in any case;
/// a folder always passes.
pub fn passes(g: &EsdeGame, filter: &GameFilter) -> bool {
    if g.folder {
        return true;
    }
    let name = filter.name.trim();
    if !name.is_empty() && !g.name.to_lowercase().contains(&name.to_lowercase()) {
        return false;
    }
    filter.chosen.iter().all(|(field, chosen)| chosen.is_empty() || chosen.contains(&value_of(g, *field).unwrap_or_else(|| UNKNOWN.to_owned())))
}

/// A list's first letter for the quick selector: the first character in capitals, `#` for none.
pub fn first_letter(name: &str) -> String {
    name.trim_start().chars().next().map_or_else(|| "#".to_owned(), |c| c.to_uppercase().collect())
}

/// The quick selector's entries: the folders and a star for the favourites when each is sorted on top among other
/// entries, then each first character in list order, with the index of its first entry.
pub fn letters(games: &[EsdeGame], favourites_on_top: bool, folders_on_top: bool) -> Vec<(String, usize)> {
    let mut out: Vec<(String, usize)> = Vec::new();
    let folders = folders_on_top && games.iter().any(|g| g.folder && !g.is_collection) && games.iter().any(|g| !g.folder);
    if folders {
        out.push((FOLDER_ENTRY.to_owned(), 0));
    }
    let files: Vec<&EsdeGame> = games.iter().filter(|g| !g.folder).collect();
    let star = favourites_on_top && files.iter().any(|g| g.favourite) && files.iter().any(|g| !g.favourite);
    if star {
        out.push((STAR.to_owned(), games.iter().position(|g| !g.folder).unwrap_or(0)));
    }
    for (i, g) in games.iter().enumerate() {
        if (folders && g.folder) || (star && g.favourite && !g.folder) {
            continue;
        }
        let first = first_letter(sort_key(g));
        if !out.iter().any(|(l, _)| *l == first) {
            out.push((first, i));
        }
    }
    out
}

/// Another entry than the one selected where there is another, from a random number the caller draws.
pub fn random_index(count: usize, current: usize, draw: u64) -> usize {
    if count <= 1 {
        return current.min(count.saturating_sub(1));
    }
    let pick = (draw % (count as u64 - 1)) as usize;
    if pick >= current { pick + 1 } else { pick }
}

/// The folder that holds a folder path, one level up; "" at the top.
pub fn parent(folder: &str) -> &str {
    folder.rfind('/').map_or("", |at| &folder[..at])
}

fn depth(folder: &str) -> usize {
    if folder.is_empty() { 0 } else { folder.matches('/').count() + 1 }
}

/// Where a game's folder lies below a folder: none when it is not inside, "" when directly there, else the next
/// folder's name.
pub fn child_within<'a>(game_folder: &'a str, folder: &str) -> Option<&'a str> {
    if folder.is_empty() {
        return Some(game_folder.split('/').next().unwrap_or(""));
    }
    if game_folder == folder {
        return Some("");
    }
    game_folder.strip_prefix(folder)?.strip_prefix('/')?.split('/').next()
}

/// The folder on disk a folder path names, found by walking up from a game's own file.
pub fn on_disk(game_file: &str, game_folder: &str, folder: &str) -> String {
    let mut dir = std::path::Path::new(game_file).parent().map(|p| p.to_path_buf()).unwrap_or_default();
    for _ in 0..depth(game_folder).saturating_sub(depth(folder)) {
        dir = dir.parent().map(|p| p.to_path_buf()).unwrap_or(dir);
    }
    dir.to_string_lossy().into_owned()
}

/// A shelf's kind: a console's own games, ES-DE's automatic collections, the player's custom ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Regular,
    Auto,
    Custom,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Regular => "regular",
            Kind::Auto => "auto",
            Kind::Custom => "custom",
        }
    }
}

/// One system of the carousel: its ES-DE names and the theme folder it is drawn with, its games, and how its gamelist
/// lists them.
#[derive(Clone, Debug, PartialEq)]
pub struct Shelf {
    /// ES-DE's system name: "snes", "all", a custom collection's name, "collections".
    pub name: String,
    pub full_name: String,
    /// The theme folder the shelf is drawn with: "snes", "auto-allgames", "custom-collections".
    pub theme: String,
    /// EmuSen's system for a console's shelf; empty for a collection.
    pub system: String,
    pub kind: Kind,
    pub games: Vec<EsdeGame>,
    /// The custom collections a grouped shelf holds, each listed as a folder.
    pub folders: Option<Vec<Shelf>>,
    pub favourites_first: bool,
    /// Whether a textlist marks favourites with a star here.
    pub stars: bool,
    pub default_sort: Option<GameSort>,
    /// Listed as given until the player picks a sort, as Last Played is.
    pub in_given_order: bool,
    pub collection_id: Option<i64>,
    /// The player's flatten switch for this console: its games in one list, no folders.
    pub flatten: bool,
}

impl Shelf {
    pub fn regular(system: &str, name: &str, full_name: &str, games: Vec<EsdeGame>) -> Shelf {
        Shelf {
            name: name.to_owned(),
            full_name: full_name.to_owned(),
            theme: name.to_owned(),
            system: system.to_owned(),
            kind: Kind::Regular,
            games,
            folders: None,
            favourites_first: true,
            stars: true,
            default_sort: None,
            in_given_order: false,
            collection_id: None,
            flatten: false,
        }
    }

    fn collection(name: &str, full_name: &str, theme: &str, kind: Kind, games: Vec<EsdeGame>) -> Shelf {
        Shelf { name: name.to_owned(), full_name: full_name.to_owned(), theme: theme.to_owned(), system: String::new(), kind, ..Shelf::regular("", "", "", games) }
    }

    /// Whether the gamelist may be sorted by system: a collection.
    pub fn is_collection(&self) -> bool {
        self.kind != Kind::Regular
    }
}

/// One of the player's collections from games.db.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Collection {
    pub id: i64,
    pub name: String,
    pub members: Vec<String>,
}

/// ES-DE's collection and gamelist settings, Galaxia's `BigPictureCollections`.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub auto: Vec<String>,
    pub hidden_custom: Vec<i64>,
    pub group_custom: String,
    pub favourites_first: bool,
    pub favourites_first_custom: bool,
    pub stars_custom: bool,
    pub default_sort: GameSort,
    pub folders_on_top: bool,
    pub flattened: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings::from_json(&Value::Null)
    }
}

/// The automatic collections' names as the settings store them.
pub const ALL_GAMES: &str = "all";
pub const FAVORITES: &str = "favorites";
pub const LAST_PLAYED: &str = "recent";
/// The grouping of custom collections: by the theme, always, never.
pub const GROUP_UNTHEMED: &str = "unthemed";
pub const GROUP_ALWAYS: &str = "always";
pub const GROUP_NEVER: &str = "never";

impl Settings {
    /// From the settings' `BigPictureCollections`; a missing value is Galaxia's default.
    pub fn from_json(c: &Value) -> Settings {
        let flag = |key: &str, default: bool| c.get(key).and_then(Value::as_bool).unwrap_or(default);
        let words = |key: &str| c.get(key).and_then(Value::as_array).map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_owned)).collect::<Vec<_>>());
        Settings {
            auto: words("AutoCollections").unwrap_or_else(|| vec![ALL_GAMES.into(), FAVORITES.into(), LAST_PLAYED.into()]),
            hidden_custom: c.get("HiddenCustomCollections").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_i64).collect()).unwrap_or_default(),
            group_custom: c.get("GroupCustomCollections").and_then(Value::as_str).unwrap_or(GROUP_UNTHEMED).to_owned(),
            favourites_first: flag("FavoritesFirst", true),
            favourites_first_custom: flag("FavoritesFirstCustom", false),
            stars_custom: flag("StarsCustom", false),
            default_sort: GameSort::parse(c.get("DefaultSortOrder").and_then(Value::as_str).unwrap_or("")),
            folders_on_top: flag("FoldersOnTop", true),
            flattened: words("FlattenedSystems").unwrap_or_default(),
        }
    }
}

/// The custom collections' prefix on a grouped collection's folder entry.
pub const FOLDER_PREFIX: &str = "collection:";

/// ES-DE's characters a collection name cannot hold.
pub const FORBIDDEN_IN_NAME: &str = "*\",./:;<>\\|";

pub fn clean(name: &str) -> String {
    name.chars().filter(|c| !FORBIDDEN_IN_NAME.contains(*c)).collect::<String>().trim().to_owned()
}

/// ES-DE's rule for a name already taken: a number in brackets, the first one free.
pub fn unique(name: &str, taken: &[&str]) -> String {
    let free = |n: &str| !taken.iter().any(|t| t.to_lowercase() == n.to_lowercase());
    if free(name) {
        return name.to_owned();
    }
    (1..).map(|n| format!("{name} ({n})")).find(|n| free(n)).unwrap_or_default()
}

/// The regular systems, then the grouped custom collections with the discrete ones beside them, then all games,
/// favorites and last played, as ES-DE 3.4.1 was measured to order them (EmuSen_BigPicture.md §22.2). A game excluded
/// from the game counter joins no collection.
pub fn with_collections(systems: Vec<Shelf>, custom: &[Collection], settings: &Settings, themed_by_theme: &dyn Fn(&str) -> bool, recent: &[String]) -> Vec<Shelf> {
    let all: Vec<EsdeGame> = systems.iter().flat_map(|s| s.games.iter().filter(|g| !g.not_counted).cloned()).collect();
    let by_file: HashMap<&str, &EsdeGame> = all.iter().rev().map(|g| (g.file.as_str(), g)).collect();
    let mut shelves: Vec<Shelf> = systems.iter().map(|s| Shelf { favourites_first: settings.favourites_first, ..s.clone() }).collect();
    let mut shown: Vec<&Collection> = custom.iter().filter(|c| !settings.hidden_custom.contains(&c.id)).collect();
    shown.sort_by(|a, b| cmp_ignore_case(&a.name, &b.name));
    let (mut grouped, mut discrete) = (Vec::new(), Vec::new());
    for c in shown {
        let games = c.members.iter().filter_map(|f| by_file.get(f.as_str())).map(|g| in_custom((*g).clone())).collect();
        let shelf = Shelf {
            favourites_first: settings.favourites_first_custom,
            stars: settings.stars_custom,
            collection_id: Some(c.id),
            ..Shelf::collection(&c.name, &c.name, &c.name, Kind::Custom, games)
        };
        let alone = match settings.group_custom.as_str() {
            GROUP_NEVER => true,
            GROUP_ALWAYS => false,
            _ => themed_by_theme(&c.name),
        };
        if alone { discrete.push(shelf) } else { grouped.push(shelf) }
    }
    if !grouped.is_empty() {
        let mut seen = HashSet::new();
        let games = grouped.iter().flat_map(|f| f.games.iter()).filter(|g| seen.insert(g.file.clone())).cloned().collect();
        shelves.push(Shelf { folders: Some(grouped), ..Shelf::collection("collections", "collections", "custom-collections", Kind::Custom, games) });
    }
    shelves.extend(discrete);
    for auto in [ALL_GAMES, FAVORITES, LAST_PLAYED] {
        if !settings.auto.iter().any(|a| a == auto) {
            continue;
        }
        shelves.push(match auto {
            ALL_GAMES => Shelf { favourites_first: settings.favourites_first, ..Shelf::collection(ALL_GAMES, "all games", "auto-allgames", Kind::Auto, all.clone()) },
            FAVORITES => Shelf {
                favourites_first: settings.favourites_first,
                stars: false,
                ..Shelf::collection(FAVORITES, "favorites", "auto-favorites", Kind::Auto, all.iter().filter(|g| g.favourite).cloned().collect())
            },
            _ => Shelf {
                favourites_first: false,
                default_sort: Some(GameSort::new(SortKey::LastPlayed, true)),
                in_given_order: true,
                ..Shelf::collection(LAST_PLAYED, "last played", "auto-lastplayed", Kind::Auto, recent.iter().filter_map(|f| by_file.get(f.as_str())).map(|g| (*g).clone()).collect())
            },
        });
    }
    shelves
}

/// ES-DE's custom collections sortname wins over the sortname in every custom collection.
fn in_custom(mut g: EsdeGame) -> EsdeGame {
    if let Some(custom) = g.custom_sort_name.clone().filter(|c| !c.is_empty()) {
        g.sort_name = Some(custom);
    }
    g
}

/// A small seeded generator, so a random order is the same for the same seed.
fn next(state: &mut u64) -> u64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *state >> 33
}

/// A grouped collection as a folder entry: its name, and ES-DE's description naming its games in a random order, the
/// first of them the one whose media it shows.
pub fn collection_folder(collection: &Shelf, seed: u64) -> EsdeGame {
    let mut state = seed ^ 0x9E37_79B9_7F4A_7C15;
    let mut games: Vec<&EsdeGame> = collection.games.iter().collect();
    for i in (1..games.len()).rev() {
        games.swap(i, (next(&mut state) % (i as u64 + 1)) as usize);
    }
    let names: Vec<String> = games.iter().map(|g| format!("'{} [{}]'", g.name, g.esde_system.to_uppercase())).collect();
    let list = match names.as_slice() {
        [] => String::new(),
        [one] => format!(": {one}"),
        [rest @ .., last] => format!(": {} and {last}", rest.join(", ")),
    };
    let n = games.len();
    EsdeGame {
        name: collection.name.clone(),
        file: format!("{FOLDER_PREFIX}{}", collection.name),
        folder: true,
        is_collection: true,
        hide_metadata: true,
        description: Some(format!("This collection contains {n} {}{list}", if n == 1 { "game" } else { "games" })),
        face: games.first().map(|g| g.file.clone()),
        ..EsdeGame::default()
    }
}

/// The shelves in ES-DE's "Systems sorting": the release order given, by full name, or by the first release year, a
/// system of no known year last and ties in the order given.
pub fn systems_order(shelves: &mut [Shelf], sorting: &str, years: &HashMap<String, i32>) {
    match sorting {
        "fullnames" => shelves.sort_by(|a, b| cmp_ignore_case(&a.full_name, &b.full_name)),
        "releaseyear" => shelves.sort_by_key(|s| years.get(&s.name).copied().unwrap_or(i32::MAX)),
        _ => {}
    }
}

/// What a gamelistinfo counts: the games and favourites of the whole list, how many the search and filters keep,
/// whether a filter is on and whether a folder is open. Folders are not games, nor a game excluded from the counter.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub games: usize,
    pub favorites: usize,
    pub shown: usize,
    pub filtered: bool,
    pub in_folder: bool,
}

impl Counts {
    pub fn of<'a>(all: impl Iterator<Item = &'a EsdeGame>, kept: impl Iterator<Item = &'a EsdeGame>, filtered: bool, in_folder: bool) -> Counts {
        let counts = |g: &&EsdeGame| !g.folder && !g.not_counted;
        let games: Vec<&EsdeGame> = all.filter(counts).collect();
        Counts { games: games.len(), favorites: games.iter().filter(|g| g.favourite).count(), shown: kept.filter(counts).count(), filtered, in_folder }
    }
}

/// A gamelist as listed: its entries, what it counts, the quick selector's letters, the folder it is in, its sort,
/// and the key its sort, filter and selection are kept by.
#[derive(Clone, Debug, PartialEq)]
pub struct Listing {
    pub key: String,
    pub entries: Vec<EsdeGame>,
    pub counts: Counts,
    pub letters: Vec<(String, usize)>,
    pub folder: String,
    pub sort: GameSort,
    pub favourites_on_top: bool,
}

/// Each gamelist's own choices: its sort and filter, the folder a console's list is in, the grouped collection open,
/// and the search over every list.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Gamelists {
    pub sorts: HashMap<String, GameSort>,
    pub filters: HashMap<String, GameFilter>,
    pub path: HashMap<String, String>,
    pub open: HashMap<String, String>,
    pub search: String,
    /// The seed of the grouped collections' random descriptions.
    pub seed: u64,
    /// The theme's system folders, by which a custom collection the theme styles is a system of its own ("unthemed").
    pub theme_folders: Vec<String>,
}

/// What a list is drawn from: its games, whether favourites come first, its own default sort, whether it lists folders.
struct Source<'a> {
    games: &'a [EsdeGame],
    favourites_first: bool,
    default_sort: Option<GameSort>,
    grouped: bool,
}

impl Gamelists {
    fn open_folder<'a>(&self, shelf: &'a Shelf) -> Option<&'a Shelf> {
        let name = self.open.get(&shelf.name)?;
        shelf.folders.as_ref()?.iter().find(|f| &f.name == name)
    }

    /// A console's folders are shown unless the player flattened it, once a game of it sits in one.
    pub fn foldered(shelf: &Shelf) -> bool {
        shelf.kind == Kind::Regular && !shelf.flatten && shelf.folders.is_none() && shelf.games.iter().any(|g| !g.folder_path.is_empty())
    }

    /// Each list, and each grouped collection, keeps its own sort and filters.
    pub fn list_key(&self, shelf: &Shelf) -> String {
        match self.open_folder(shelf) {
            Some(folder) => format!("{}/{}", shelf.name, folder.name),
            None => shelf.name.clone(),
        }
    }

    fn source<'a>(&self, shelf: &'a Shelf) -> Source<'a> {
        if let Some(folder) = self.open_folder(shelf) {
            return Source { games: &folder.games, favourites_first: folder.favourites_first, default_sort: folder.default_sort, grouped: false };
        }
        if shelf.folders.is_some() {
            return Source { games: &[], favourites_first: false, default_sort: None, grouped: true };
        }
        Source { games: &shelf.games, favourites_first: shelf.favourites_first, default_sort: shelf.default_sort, grouped: false }
    }

    pub fn sort_of(&self, shelf: &Shelf, settings: &Settings) -> GameSort {
        self.sorts.get(&self.list_key(shelf)).copied().or(self.source(shelf).default_sort).unwrap_or(settings.default_sort)
    }

    fn filter_of(&self, shelf: &Shelf) -> GameFilter {
        self.filters.get(&self.list_key(shelf)).cloned().unwrap_or_default()
    }

    fn searched(&self, g: &EsdeGame) -> bool {
        let words = self.search.trim();
        words.is_empty() || g.name.to_lowercase().contains(&words.to_lowercase())
    }

    /// The games a searched and filtered console keeps over every folder, as ES-DE's filters apply to "the complete
    /// game system, including all folder content".
    fn kept<'a>(&self, shelf: &'a Shelf) -> Vec<&'a EsdeGame> {
        let filter = self.filter_of(shelf);
        shelf.games.iter().filter(|g| self.searched(g) && passes(g, &filter)).collect()
    }

    /// The folder a console's list is in, walked up past any a search, a filter or a change on disk has emptied.
    fn folder_in(&self, shelf: &Shelf, kept: &[&EsdeGame]) -> String {
        let Some(mut path) = self.path.get(&shelf.name).filter(|_| Gamelists::foldered(shelf)).map(String::as_str) else { return String::new() };
        while !path.is_empty() && !kept.iter().any(|g| child_within(&g.folder_path, path).is_some()) {
            path = parent(path);
        }
        path.to_owned()
    }

    /// One folder's entries: its folders holding a kept game, then its games, or all in one order when folders are not
    /// on top.
    fn folder_listing(&self, shelf: &Shelf, kept: &[&EsdeGame], path: &str, links: &HashMap<String, String>, settings: &Settings) -> Vec<EsdeGame> {
        let (mut here, mut folders): (Vec<EsdeGame>, Vec<EsdeGame>) = (Vec::new(), Vec::new());
        for g in kept {
            match child_within(&g.folder_path, path) {
                Some("") => here.push((*g).clone()),
                Some(child) if !folders.iter().any(|f| f.name == child) => {
                    let folder = if path.is_empty() { child.to_owned() } else { format!("{path}/{child}") };
                    let disk = on_disk(&g.file, &g.folder_path, &folder);
                    let link = links.get(&disk).cloned();
                    folders.push(EsdeGame { name: child.to_owned(), file: disk, folder: true, folder_path: folder, folder_link: link, ..EsdeGame::default() });
                }
                _ => {}
            }
        }
        let order = self.sort_of(shelf, settings);
        let favourites_first = self.source(shelf).favourites_first;
        if !settings.folders_on_top {
            here.extend(folders);
            sort(&mut here, order, favourites_first);
            return here;
        }
        sort(&mut folders, order, false);
        sort(&mut here, order, favourites_first);
        folders.extend(here);
        folders
    }

    /// A shelf's gamelist as listed now.
    pub fn listing(&self, shelf: &Shelf, links: &HashMap<String, String>, settings: &Settings) -> Listing {
        let source = self.source(shelf);
        let order = self.sort_of(shelf, settings);
        let filter = self.filter_of(shelf);
        let filtered = !self.search.trim().is_empty() || filter.is_active();
        let key = self.list_key(shelf);
        if Gamelists::foldered(shelf) {
            let kept = self.kept(shelf);
            let folder = self.folder_in(shelf, &kept);
            let entries = self.folder_listing(shelf, &kept, &folder, links, settings);
            let counts = Counts::of(shelf.games.iter(), kept.iter().copied(), filtered, !folder.is_empty());
            let letters = letters(&entries, source.favourites_first, settings.folders_on_top);
            return Listing { key, entries, counts, letters, folder, sort: order, favourites_on_top: source.favourites_first };
        }
        let mut entries: Vec<EsdeGame> = if source.grouped {
            let mut folders: Vec<&Shelf> = shelf.folders.iter().flatten().collect();
            folders.sort_by(|a, b| cmp_ignore_case(&a.name, &b.name));
            folders.into_iter().map(|f| collection_folder(f, self.seed)).collect()
        } else {
            source.games.to_vec()
        };
        entries.retain(|g| g.folder || (self.searched(g) && passes(g, &filter)));
        let as_given = self.open_folder(shelf).is_none() && shelf.in_given_order && !self.sorts.contains_key(&key);
        if !source.grouped && !as_given {
            sort(&mut entries, order, source.favourites_first);
        }
        let whole = self.open_folder(shelf).map_or(&shelf.games[..], |f| &f.games[..]);
        let all = if whole.is_empty() { &entries[..] } else { whole };
        let counts = Counts::of(all.iter(), entries.iter(), filtered, self.open_folder(shelf).is_some());
        let on_top = !source.grouped && source.favourites_first;
        let letters = letters(&entries, on_top, settings.folders_on_top);
        Listing { key, entries, counts, letters, folder: String::new(), sort: order, favourites_on_top: on_top }
    }

    /// Into a folder entry of the list: a console's folder, or a grouped collection; false for any other entry.
    pub fn enter(&mut self, shelf: &Shelf, entry: &EsdeGame) -> bool {
        if !entry.folder {
            return false;
        }
        if entry.is_collection {
            if shelf.folders.is_none() {
                return false;
            }
            self.open.insert(shelf.name.clone(), entry.name.clone());
        } else if Gamelists::foldered(shelf) {
            self.path.insert(shelf.name.clone(), entry.folder_path.clone());
        } else {
            return false;
        }
        true
    }

    /// Up a level: the folder's parent, or out of a grouped collection; false at the top.
    pub fn leave(&mut self, shelf: &Shelf) -> bool {
        let kept = self.kept(shelf);
        let path = self.folder_in(shelf, &kept);
        if !path.is_empty() {
            match parent(&path) {
                "" => self.path.remove(&shelf.name),
                up => self.path.insert(shelf.name.clone(), up.to_owned()),
            };
            return true;
        }
        self.open.remove(&shelf.name).is_some()
    }

    /// The sort the List card's Sort steps to from the list's own, through every order the list offers.
    pub fn step_sort(&mut self, shelf: &Shelf, settings: &Settings, by: i32) -> GameSort {
        let all = GameSort::all(shelf.is_collection());
        let at = all.iter().position(|s| *s == self.sort_of(shelf, settings)).unwrap_or(0) as i32;
        let sort = all[(at + by).rem_euclid(all.len() as i32) as usize];
        self.sorts.insert(self.list_key(shelf), sort);
        sort
    }

    /// The list's filter on the favourites alone, or none.
    pub fn favourites_only(&mut self, shelf: &Shelf, on: bool) {
        let key = self.list_key(shelf);
        let filter = self.filters.remove(&key).unwrap_or_default().with(FilterField::Favorite, if on { &["Yes"] } else { &[] });
        if filter.is_active() {
            self.filters.insert(key, filter);
        }
    }

    /// Whether the list is narrowed to its favourites.
    pub fn is_favourites_only(&self, shelf: &Shelf) -> bool {
        self.filters.get(&self.list_key(shelf)).and_then(|f| f.chosen.get(&FilterField::Favorite)).is_some_and(|v| v.len() == 1 && v.contains("Yes"))
    }
}

/// A listing as the painter reads it, one shelf of the shelves' text.
pub fn listing_json(shelf: &Shelf, listing: &Listing) -> Value {
    json!({
        "name": shelf.name,
        "full_name": shelf.full_name,
        "theme": shelf.theme,
        "system": shelf.system,
        "kind": shelf.kind.name(),
        "collection_id": shelf.collection_id,
        "stars": shelf.stars,
        "key": listing.key,
        "folder": listing.folder,
        "sort": listing.sort.stored(),
        "favourites_on_top": listing.favourites_on_top,
        "counts": {
            "games": listing.counts.games, "favorites": listing.counts.favorites, "shown": listing.counts.shown,
            "filtered": listing.counts.filtered, "in_folder": listing.counts.in_folder,
        },
        "letters": listing.letters.iter().map(|(l, i)| json!([l, i])).collect::<Vec<_>>(),
        "games": listing.entries.iter().map(|g| entry_json(shelf, g)).collect::<Vec<_>>(),
        "unlisted": unlisted(shelf, listing),
    })
}

/// A console's games that the listing's level does not show (inside its folders, or outside the folder open), whole: ES-DE's gameselectors draw from
/// every game of the system (EmuSen_Qt.md §4.11).
fn unlisted(shelf: &Shelf, listing: &Listing) -> Vec<Value> {
    if shelf.kind != Kind::Regular {
        return Vec::new();
    }
    let listed: std::collections::HashSet<&str> = listing.entries.iter().map(|g| g.file.as_str()).collect();
    shelf.games.iter().filter(|g| !g.folder && !listed.contains(g.file.as_str())).map(EsdeGame::to_json).collect()
}

/// An entry of a listing: a console's game as its whole row; a collection's game by its file alone, its row the one
/// its console's shelf carries, so the text holds each game once; a folder entry whole, with its description.
fn entry_json(shelf: &Shelf, g: &EsdeGame) -> Value {
    match (shelf.kind, g.folder) {
        (_, true) => g.to_json_whole(),
        (Kind::Regular, false) => g.to_json(),
        (_, false) => json!({ "file": g.file, "esde_system": g.esde_system, "sort_name": g.sort_name }),
    }
}
