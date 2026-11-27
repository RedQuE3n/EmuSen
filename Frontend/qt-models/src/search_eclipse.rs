//! Eclipse Search view model (#8) and shared Eclipse hub helpers.

use crate::esde_shelves::{release_date, year_of, Metadata};
use crate::library::{games_of_at, ALL};
use crate::words::{grouped, NO_GAMES_FOUND};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use emusen_kit_core::input::UiButton;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};

/// Four-row docked keyboard layouts (`ABC` and `123`) matching Eclipse board #8.
pub const ECLIPSE_KEYBOARD_LAYOUTS: &str = "ABC\nq w e r t y u i o p\na s d f g h j k l '\nz x c v b n m , . -\nNext:2 Space:5 Erase:2 Done:2\n\n123\n1 2 3 4 5 6 7 8 9 0\n! @ # $ % & * ( ) ?\n+ = / : ; _ ~ , . -\nNext:2 Space:5 Erase:2 Done:2";

/// Board SVG paths for Search's magnifier and erase key.
const MAGNIFIER_ICON: &str = "M18 11A7 7 0 1 1 4 11A7 7 0 1 1 18 11M16.5 16.5L21 21";
pub(crate) const ERASE_ICON: &str = "M9 5h11v14H9l-6-7zM12.5 9.5l5 5M17.5 9.5l-5 5";

/// Deterministic procedural cover swatches `[dot, wave, base]` from Eclipse boards #8 and #11.
const PALETTES: [[&str; 3]; 8] = [
    ["#B8FFE9", "#6A4CF0", "#12806F"],
    ["#FFE2A8", "#B5722E", "#5B3A1A"],
    ["#9FD0FF", "#1C4FB8", "#0C2A66"],
    ["#FFD1B0", "#B5432E", "#5B1C1A"],
    ["#D7F5B0", "#3E7A5A", "#1E3F36"],
    ["#E3CCFF", "#6C3FB8", "#2C1A57"],
    ["#F6D3DC", "#A63D62", "#4A1830"],
    ["#C9ECFF", "#2B7FA8", "#113A52"],
];

pub(crate) fn palette_colors(seed: usize) -> [&'static str; 3] {
    PALETTES[seed % PALETTES.len()]
}

pub(crate) fn cover_swatch(cover: &str, seed: usize) -> Value {
    let [dot, wave, bg] = palette_colors(seed);
    json!({ "cover": cover, "dot": dot, "wave": wave, "bg": bg })
}

pub(crate) fn help_entry(button: UiButton, label: &str, enabled: bool) -> Value {
    json!({ "button": button.index() as i32, "label": label, "pair": -1, "enabled": enabled })
}

pub(crate) fn help_pair(button: UiButton, pair: UiButton, label: &str, enabled: bool) -> Value {
    json!({ "button": button.index() as i32, "label": label, "pair": pair.index() as i32, "enabled": enabled })
}

pub(crate) fn nav_sections(active: &str) -> Vec<Value> {
    crate::sections::all()
        .iter()
        .map(|s| {
            json!({
                "key": s.key,
                "name": s.name,
                "active": s.key == active,
                "itemName": format!("nav.{}", s.key),
                "wordsName": format!("navWords.{}", s.key),
                "underlineName": format!("navUnderline.{}", s.key),
            })
        })
        .collect()
}

pub(crate) enum NavKey {
    Left,
    Up,
    Right,
    Down,
    Accept,
    Back,
}

pub(crate) fn qt_nav_key(key: i32) -> Option<NavKey> {
    match key {
        0x0100_0012 => Some(NavKey::Left),
        0x0100_0013 => Some(NavKey::Up),
        0x0100_0014 => Some(NavKey::Right),
        0x0100_0015 => Some(NavKey::Down),
        0x0100_0004 | 0x0100_0005 => Some(NavKey::Accept),
        0x0100_0000 => Some(NavKey::Back),
        _ => None,
    }
}

pub(crate) fn step_section_action(screens: *mut crate::screens::qobject::Screens, key: &str) -> bool {
    if screens.is_null() {
        return false;
    }
    let s = unsafe { Pin::new_unchecked(&mut *screens) };
    match key {
        crate::help::PREVIOUS => {
            s.step_section(-1);
            true
        }
        crate::help::NEXT => {
            s.step_section(1);
            true
        }
        _ => false,
    }
}

pub(crate) fn handle_grid_key(
    key: i32,
    screens: *mut crate::screens::qobject::Screens,
    on_move: impl FnOnce(i32, i32),
    on_accept: impl FnOnce(),
) -> bool {
    match qt_nav_key(key) {
        Some(NavKey::Left) => { on_move(-1, 0); true }
        Some(NavKey::Right) => { on_move(1, 0); true }
        Some(NavKey::Up) => { on_move(0, -1); true }
        Some(NavKey::Down) => { on_move(0, 1); true }
        Some(NavKey::Accept) => { on_accept(); true }
        Some(NavKey::Back) => {
            if !screens.is_null() {
                unsafe { Pin::new_unchecked(&mut *screens) }.back();
            }
            true
        }
        None => false,
    }
}

pub(crate) fn grid_step(current: i32, count: usize, cols: i32, dx: i32, dy: i32) -> i32 {
    if count == 0 {
        return -1;
    }
    let last = count as i32 - 1;
    let cur = current.clamp(0, last);
    if dx != 0 {
        return (cur + dx).clamp(0, last);
    }
    if dy < 0 {
        return if cur >= cols { cur - cols } else { cur };
    }
    if dy > 0 {
        let cur_row = cur / cols;
        let last_row = last / cols;
        if cur_row < last_row {
            return (cur + cols).min(last);
        }
    }
    cur
}

/// Groups non-empty scraped genres (`genre -> sorted member paths`), ordered by count desc then name asc.
pub(crate) fn genre_groups(metadata: &[(String, Metadata)], live_paths: &HashSet<&str>) -> Vec<(String, Vec<String>)> {
    let mut by_genre: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (path, m) in metadata {
        if !live_paths.contains(path.as_str()) {
            continue;
        }
        let genre = m.get("genre").map(|g| g.trim()).unwrap_or_default();
        if !genre.is_empty() {
            by_genre.entry(genre.to_owned()).or_default().push(path.clone());
        }
    }
    let mut list: Vec<(String, Vec<String>)> = by_genre.into_iter().collect();
    list.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.0.to_lowercase().cmp(&b.0.to_lowercase())));
    list
}

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("emusen-qt-models/src/library_model.cxxqt.h");
        type LibraryModel = crate::library_model::qobject::LibraryModel;
        include!("emusen-qt-models/src/game_model.cxxqt.h");
        type GameModel = crate::game_model::qobject::GameModel;
        include!("emusen-qt-models/src/screens.cxxqt.h");
        type Screens = crate::screens::qobject::Screens;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(*mut LibraryModel, library)]
        #[qproperty(*mut GameModel, games)]
        #[qproperty(*mut Screens, screens)]
        #[qproperty(QString, query)]
        #[qproperty(bool, on_keys, cxx_name = "onKeys")]
        #[qproperty(i32, focused_result, cxx_name = "focusedResult")]
        #[qproperty(QString, view, READ, NOTIFY)]
        #[qproperty(QString, footer_help, cxx_name = "footerHelp", READ, NOTIFY)]
        #[qproperty(QString, layouts, READ, CONSTANT)]
        #[qproperty(i32, result_count, cxx_name = "resultCount", READ, NOTIFY)]
        type SearchEclipseModel = super::SearchEclipseModelRust;
    }

    extern "RustQt" {
        #[qinvokable]
        fn refresh(self: Pin<&mut SearchEclipseModel>);

        #[qinvokable]
        #[cxx_name = "moveResult"]
        fn move_result(self: Pin<&mut SearchEclipseModel>, delta: i32) -> i32;

        #[qinvokable]
        #[cxx_name = "pathAt"]
        fn path_at(self: &SearchEclipseModel, index: i32) -> QString;

        #[qinvokable]
        #[cxx_name = "handleAction"]
        fn handle_action(self: Pin<&mut SearchEclipseModel>, key: &QString);

        #[qinvokable]
        #[cxx_name = "handleResultKey"]
        fn handle_result_key(self: Pin<&mut SearchEclipseModel>, key: i32) -> bool;
    }

    impl cxx_qt::Initialize for SearchEclipseModel {}
}

pub struct SearchEclipseModelRust {
    library: *mut qobject::LibraryModel,
    games: *mut qobject::GameModel,
    screens: *mut qobject::Screens,
    query: QString,
    on_keys: bool,
    focused_result: i32,
    view: QString,
    footer_help: QString,
    layouts: QString,
    result_count: i32,
    paths: Vec<String>,
}

impl Default for SearchEclipseModelRust {
    fn default() -> Self {
        SearchEclipseModelRust {
            library: std::ptr::null_mut(),
            games: std::ptr::null_mut(),
            screens: std::ptr::null_mut(),
            query: QString::default(),
            on_keys: true,
            focused_result: 0,
            view: QString::from("{}"),
            footer_help: QString::from("[]"),
            layouts: QString::from(ECLIPSE_KEYBOARD_LAYOUTS),
            result_count: 0,
            paths: Vec::new(),
        }
    }
}

impl cxx_qt::Initialize for qobject::SearchEclipseModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().refresh();
        self.as_mut().on_library_changed(|m| m.refresh()).release();
        self.as_mut().on_games_changed(|m| m.refresh()).release();
        self.as_mut().on_query_changed(|m| m.refresh()).release();
        self.as_mut().on_on_keys_changed(|m| m.refresh()).release();
        self.as_mut().on_focused_result_changed(|m| m.refresh()).release();
    }
}

impl qobject::SearchEclipseModel {
    pub fn move_result(mut self: Pin<&mut Self>, delta: i32) -> i32 {
        let len = self.paths.len() as i32;
        let next = if len == 0 { 0 } else { (self.focused_result + delta).clamp(0, len - 1) };
        self.as_mut().set_focused_result(next);
        next
    }

    pub fn path_at(&self, index: i32) -> QString {
        QString::from(self.paths.get(index.max(0) as usize).map_or("", String::as_str))
    }

    pub fn handle_action(self: Pin<&mut Self>, key: &QString) {
        if self.screens.is_null() {
            return;
        }
        let screens = unsafe { Pin::new_unchecked(&mut *self.screens) };
        match key.to_string().as_str() {
            crate::screens::TO_GRID => {
                if self.on_keys && !self.paths.is_empty() {
                    screens.act(&QString::from(crate::screens::TO_GRID));
                } else if !self.on_keys {
                    screens.keys_focused(true);
                }
            }
            crate::help::PREVIOUS | crate::help::NEXT if !self.on_keys => screens.keys_focused(true),
            _ => {}
        }
    }

    pub fn handle_result_key(mut self: Pin<&mut Self>, key: i32) -> bool {
        if self.on_keys {
            return false;
        }
        match qt_nav_key(key) {
            Some(NavKey::Left) => {
                self.as_mut().move_result(-1);
                true
            }
            Some(NavKey::Right) => {
                self.as_mut().move_result(1);
                true
            }
            Some(NavKey::Down) => {
                if !self.screens.is_null() {
                    unsafe { Pin::new_unchecked(&mut *self.screens) }.keys_focused(true);
                }
                true
            }
            Some(NavKey::Accept) => {
                let path = self.path_at(self.focused_result);
                if !path.is_empty() && !self.screens.is_null() {
                    unsafe { Pin::new_unchecked(&mut *self.screens) }.open_game(&path);
                }
                true
            }
            Some(NavKey::Back) => {
                if !self.screens.is_null() {
                    unsafe { Pin::new_unchecked(&mut *self.screens) }.back();
                }
                true
            }
            _ => false,
        }
    }

    pub fn refresh(mut self: Pin<&mut Self>) {
        let lib = unsafe { self.library.as_ref() };
        let games = unsafe { self.games.as_ref() };
        let query = self.query.to_string();
        let q = query.trim().to_owned();
        let q_lower = q.to_lowercase();

        let (total_count, meta_by_path, matching_consoles, matching_collections) = if let Some(lib) = lib {
            let view = lib.view();
            let catalog = &view.catalog;
            let section = games.map(|g| g.section().to_string()).unwrap_or_else(|| ALL.to_owned());
            let total = if section == ALL {
                catalog.games.len()
            } else {
                games_of_at(&section, &view.systems, catalog, crate::words::now()).len()
            };
            let meta = lib.effective_metadata();
            let meta_map: HashMap<&str, &Metadata> = meta.iter().map(|(p, m)| (p.as_str(), m)).collect();
            let live_paths: HashSet<&str> = catalog.games.iter().map(|g| g.path.as_str()).collect();
            let mut consoles: Vec<String> = Vec::new();
            let mut collections: Vec<String> = Vec::new();
            if !q_lower.is_empty() {
                let with_games: HashSet<&str> = catalog.games.iter().filter_map(|g| g.system.as_deref()).collect();
                for s in &view.systems {
                    if !with_games.contains(s.id.as_str()) {
                        continue;
                    }
                    let label = if s.display_name.is_empty() { &s.name } else { &s.display_name };
                    if label.to_lowercase().contains(&q_lower) || s.id.to_lowercase().contains(&q_lower) {
                        consoles.push(label.clone());
                    }
                }
                for fixed in ["Favourites", "Recently played"] {
                    if fixed.to_lowercase().contains(&q_lower) {
                        collections.push(fixed.to_owned());
                    }
                }
                for c in &catalog.collections {
                    if c.name.to_lowercase().contains(&q_lower) {
                        collections.push(c.name.clone());
                    }
                }
                for (genre, _) in genre_groups(meta, &live_paths) {
                    if genre.to_lowercase().contains(&q_lower) && !collections.contains(&genre) {
                        collections.push(genre);
                    }
                }
            }
            (total, meta_map, consoles, collections)
        } else {
            (0, HashMap::new(), Vec::new(), Vec::new())
        };

        let rows = games.map(|g| g.rows()).unwrap_or(&[]);
        let shown_count = rows.len();
        let total = total_count.max(shown_count);
        let count_line = if q.is_empty() {
            crate::words::games(total)
        } else {
            format!("{} of {}", grouped(shown_count), crate::words::games(total))
        };

        let mut cards: Vec<Value> = Vec::new();
        let mut paths: Vec<String> = Vec::new();
        for (i, row) in rows.iter().take(36).enumerate() {
            let year = meta_by_path.get(row.path.as_str()).and_then(|m| release_date(m)).and_then(|d| year_of(&d)).unwrap_or_default();
            let subtitle = if year.is_empty() { row.console.clone() } else { format!("{} · {year}", row.console) };
            let swatch = cover_swatch(&row.cover, i);
            paths.push(row.path.clone());
            cards.push(json!({
                "index": i,
                "title": row.title,
                "subtitle": subtitle,
                "cover": crate::game_model::cover_url(&row.cover),
                "dot": swatch["dot"],
                "wave": swatch["wave"],
                "bg": swatch["bg"],
                "path": row.path,
                "objectName": format!("searchResult.{i}"),
            }));
        }

        let note = if q.is_empty() {
            String::new()
        } else if matching_consoles.is_empty() && matching_collections.is_empty() {
            format!("No console or collection has \"{q}\" in its name.")
        } else {
            let mut parts = Vec::new();
            if !matching_consoles.is_empty() {
                parts.push(format!("Consoles: {}", matching_consoles.join(", ")));
            }
            if !matching_collections.is_empty() {
                parts.push(format!("Collections: {}", matching_collections.join(", ")));
            }
            parts.join(" · ")
        };

        let on_keys = self.on_keys;
        let has_cards = !cards.is_empty();
        let help = if on_keys {
            vec![
                help_entry(UiButton::Accept, "Type", true),
                help_entry(UiButton::Back, "Erase, then close when empty", true),
                help_entry(UiButton::Search, "Space", true),
                help_entry(UiButton::Last, "To the results", has_cards),
            ]
        } else {
            vec![
                help_entry(UiButton::Accept, "Open", has_cards),
                help_entry(UiButton::Back, "Back", true),
                help_entry(UiButton::Last, "To keyboard", true),
            ]
        };

        let clamped_focus = if paths.is_empty() { 0 } else { self.focused_result.clamp(0, paths.len() as i32 - 1) };
        let payload = json!({
            "query": query,
            "placeholder": "Search games",
            "countLine": count_line,
            "gamesHeading": "Games",
            "emptyLine": NO_GAMES_FOUND,
            "note": note,
            "cards": cards,
            "focusedResult": clamped_focus,
            "magnifierIcon": MAGNIFIER_ICON,
            "eraseIcon": ERASE_ICON,
            "keyPrefix": "searchKey.",
            "keyRingPrefix": "searchKeyRing.",
            "keyBgPrefix": "searchKeyBg.",
            "keyTextPrefix": "searchKeyText.",
            "actionLines": "space Search\ntoGrid Last\nprevious PageUp\nnext PageDown\n",
        });

        let view_str = QString::from(payload.to_string().as_str());
        let help_str = QString::from(Value::Array(help).to_string().as_str());
        let new_count = paths.len() as i32;
        self.as_mut().rust_mut().paths = paths;
        if self.result_count != new_count {
            self.as_mut().rust_mut().result_count = new_count;
            self.as_mut().result_count_changed();
        }
        if self.view != view_str {
            self.as_mut().rust_mut().view = view_str;
            self.as_mut().view_changed();
        }
        if self.footer_help != help_str {
            self.as_mut().rust_mut().footer_help = help_str;
            self.as_mut().footer_help_changed();
        }
    }
}
