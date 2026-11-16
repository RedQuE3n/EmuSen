//! `Library`, the library as a theme reads it (EmuSen_Qt.md §3.2): the consoles, Home's places, the three shelves, the
//! recent and favourite games, and any console's or place's games, each a read-only list fed by the host from the view.

use crate::glue::ffi;
use crate::rows::qobject::ThemeRows;
use crate::view::{self, Row};
use core::pin::Pin;
use cxx_qt::casting::Upcast;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QString, QStringList};
use emusen_qt_models::console_list::{Order, ORDERS, SHOWS};
use cxx_qt_lib::QList;
use emusen_qt_models::library::{LibraryView, FAVOURITES, RECENT};
use emusen_qt_models::words;
use std::collections::HashMap;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qlist.h");
        type QList_f64 = cxx_qt_lib::QList<f64>;
        include!("emusen-theme-qt/src/rows.cxxqt.h");
        type ThemeRows = crate::rows::qobject::ThemeRows;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element = "Library"]
        #[qml_singleton]
        #[qproperty(*mut ThemeRows, consoles, READ, CONSTANT)]
        #[qproperty(*mut ThemeRows, places, READ, CONSTANT)]
        #[qproperty(*mut ThemeRows, shelves, READ, CONSTANT)]
        #[qproperty(*mut ThemeRows, recent, READ, CONSTANT)]
        #[qproperty(*mut ThemeRows, favourites, READ, CONSTANT)]
        #[qproperty(*mut ThemeRows, collections, READ, CONSTANT)]
        #[qproperty(*mut ThemeRows, shelf, READ, CONSTANT)]
        /// The program's top sections (1.6), and the key of the one shown, empty where the screen shown is none.
        #[qproperty(*mut ThemeRows, sections, READ, CONSTANT)]
        #[qproperty(QString, section, READ, NOTIFY)]
        /// Moves each time a picture's hues have been read and each time the library is fed, so a binding on `artHues` asks again.
        #[qproperty(i32, art_hues_read, cxx_name = "artHuesRead", READ, NOTIFY)]
        type ThemeLibrary = super::ThemeLibraryRust;
    }

    unsafe extern "RustQt" {
        /// The games of a console's id or a place's (`all`, `favourites`, `recent`), the same list each time it is asked.
        #[qinvokable]
        #[cxx_name = "gamesOf"]
        fn games_of(self: Pin<&mut ThemeLibrary>, id: &QString) -> *mut ThemeRows;

        /// Every game whose title holds the words, one list that each search refills.
        #[qinvokable]
        fn search(self: Pin<&mut ThemeLibrary>, words: &QString) -> *mut ThemeRows;
        /// The first `count` games of `gamesOf(id)`, the same list each time it is asked: a fan or a preview of a section.
        #[qinvokable]
        #[cxx_name = "firstOf"]
        fn first_of(self: Pin<&mut ThemeLibrary>, id: &QString, count: i32) -> *mut ThemeRows;
        /// The consoles a chip shows in an order, each maker's together, by name as plates show them (EmuSen_Qt.md §3.4.9).
        #[qinvokable]
        #[cxx_name = "consolesBy"]
        fn consoles_by(self: Pin<&mut ThemeLibrary>, show: i32, order: i32, shortened: &QStringList) -> *mut ThemeRows;

        /// The hues of a game's art (its hero, else its cover), most prominent first, at most `most`, in degrees; none
        /// while they are read off the GUI thread (`artHuesRead` moves when they are) and for art with no colour or none.
        #[qinvokable]
        #[cxx_name = "artHues"]
        fn art_hues(self: Pin<&mut ThemeLibrary>, key: &QString, most: i32) -> QList_f64;
    }

    impl cxx_qt::Initialize for ThemeLibrary {}
    impl cxx_qt::Threading for ThemeLibrary {}
}

/// The lists, each the singleton's child, and the view they were last fed from.
pub struct ThemeLibraryRust {
    consoles: *mut ThemeRows,
    places: *mut ThemeRows,
    shelves: *mut ThemeRows,
    recent: *mut ThemeRows,
    /// `recent` less the suspended game, which Continue holds.
    shelf: *mut ThemeRows,
    suspended: String,
    favourites: *mut ThemeRows,
    collections: *mut ThemeRows,
    sections: *mut ThemeRows,
    section: QString,
    games: HashMap<String, *mut ThemeRows>,
    firsts: HashMap<(String, usize), *mut ThemeRows>,
    listed: HashMap<(usize, usize, Vec<String>), *mut ThemeRows>,
    found: *mut ThemeRows,
    words: String,
    view: LibraryView,
    /// Home's lines as last fed, and the places the host would decline.
    lines: Vec<(String, bool)>,
    declined: Vec<String>,
    art_hues_read: i32,
    /// Each picture's hues as read, by its path, when its file last changed and how many were asked; a picture being read is
    /// listed with none, and one written again, or written after it was first asked for, is read again.
    hues: HashMap<Asked, Vec<f64>>,
    /// The one thread that reads pictures, one after another, started at the first asked for; it ends with the library.
    reader: Option<std::sync::mpsc::Sender<Asked>>,
}

/// A picture's hues asked for: its path, when its file last changed (none while there is no file), and how many.
type Asked = (String, Option<std::time::SystemTime>, usize);

impl Default for ThemeLibraryRust {
    fn default() -> Self {
        let none = std::ptr::null_mut();
        ThemeLibraryRust {
            consoles: none,
            places: none,
            shelves: none,
            recent: none,
            shelf: none,
            suspended: String::new(),
            favourites: none,
            collections: none,
            sections: none,
            section: QString::default(),
            games: HashMap::new(),
            firsts: HashMap::new(),
            listed: HashMap::new(),
            found: none,
            words: String::new(),
            view: LibraryView::empty(),
            lines: Vec::new(),
            declined: Vec::new(),
            art_hues_read: 0,
            hues: HashMap::new(),
            reader: None,
        }
    }
}

/// A list's rows replaced; a null list is none.
fn fill(list: *mut ThemeRows, rows: Vec<Row>) {
    // Safety: every list is the singleton's child, made in `made` and alive as long as the singleton.
    if let Some(list) = unsafe { list.as_mut() } {
        unsafe { Pin::new_unchecked(list) }.replace(rows);
    }
}

impl cxx_qt::Initialize for qobject::ThemeLibrary {
    fn initialize(mut self: Pin<&mut Self>) {
        let consoles = self.as_mut().made(&view::CONSOLE_ROLES);
        let places = self.as_mut().made(&view::PLACE_ROLES);
        let shelves = self.as_mut().made(&view::SHELF_ROLES);
        let recent = self.as_mut().made(&view::GAME_ROLES);
        let shelf = self.as_mut().made(&view::GAME_ROLES);
        let favourites = self.as_mut().made(&view::GAME_ROLES);
        let collections = self.as_mut().made(&view::COLLECTION_ROLES);
        let found = self.as_mut().made(&view::GAME_ROLES);
        let sections = self.as_mut().made(&view::SECTION_ROLES);
        fill(sections, view::sections());
        let mut me = self.as_mut().rust_mut();
        me.consoles = consoles;
        me.places = places;
        me.shelves = shelves;
        me.recent = recent;
        me.shelf = shelf;
        me.favourites = favourites;
        me.collections = collections;
        me.found = found;
        me.sections = sections;
        drop(me);
        self.show(&LibraryView::empty(), "[]");
    }
}

impl qobject::ThemeLibrary {
    pub fn art_hues(mut self: Pin<&mut Self>, key: &QString, most: i32) -> cxx_qt_lib::QList<f64> {
        let mut out = cxx_qt_lib::QList::<f64>::default();
        let Some(game) = self.view.game(&key.to_string()) else { return out };
        let path = if game.hero.is_empty() { game.cover } else { game.hero };
        if path.is_empty() || most <= 0 {
            return out;
        }
        let changed = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        let asked = (path, changed, most as usize);
        if let Some(hues) = self.hues.get(&asked) {
            hues.iter().for_each(|h| out.append(*h));
            return out;
        }
        self.as_mut().rust_mut().hues.insert(asked.clone(), Vec::new());
        if self.reader.is_none() {
            let (send, take) = std::sync::mpsc::channel::<Asked>();
            let qt = self.qt_thread();
            std::thread::spawn(move || {
                for asked in take {
                    let hues = emusen_qt_models::art_hues::hues_of_file(&asked.0, asked.2);
                    let sent = qt.queue(move |mut library| {
                        library.as_mut().rust_mut().hues.insert(asked, hues);
                        let read = library.art_hues_read.wrapping_add(1);
                        library.as_mut().rust_mut().art_hues_read = read;
                        library.as_mut().art_hues_read_changed();
                    });
                    // The library is gone: nothing is left to read for.
                    if sent.is_err() {
                        break;
                    }
                }
            });
            self.as_mut().rust_mut().reader = Some(send);
        }
        if let Some(reader) = &self.reader {
            let _ = reader.send(asked);
        }
        out
    }

    /// A list of the singleton's own with `roles`.
    fn made(self: Pin<&mut Self>, roles: &'static [&'static str]) -> *mut ThemeRows {
        let list = ffi::new_rows(self.upcast_pin());
        // Safety: the list was just made, a child of this singleton.
        if let Some(rows) = unsafe { list.as_mut() } {
            unsafe { Pin::new_unchecked(rows) }.set_roles(roles);
        }
        list
    }

    /// The host's feed: the library's view and Home's lines as the library model publishes them (`homeLines`).
    pub fn show(mut self: Pin<&mut Self>, view: &LibraryView, home_lines: &str) {
        let now = words::now();
        fill(self.consoles, view::consoles(view));
        let lines = view::lines_of(home_lines);
        fill(self.places, view::places(view, &lines, &self.declined));
        fill(self.shelves, view::shelves(view));
        fill(self.recent, view::games_of(view, RECENT, now));
        fill(self.shelf, view::shelf(view, &self.suspended, now));
        fill(self.favourites, view::games_of(view, FAVOURITES, now));
        // The own look's view reads no collections yet (§3.2 as built), so the list stays empty.
        fill(self.collections, Vec::new());
        for (id, list) in &self.games {
            fill(*list, view::games_of(view, id, now));
        }
        for ((id, count), list) in &self.firsts {
            fill(*list, view::first_of(view, id, *count, now));
        }
        fill(self.found, view::search(view, &self.words, now));
        for ((show, order, short), list) in &self.listed {
            fill(*list, view::consoles_listed(view, SHOWS[*show], ORDERS[*order], short));
        }
        // The own look reads the rows, the systems and the catalog alone; the lists' texts and the ES-DE state are not copied (EmuSen_Qt.md §4.15).
        self.as_mut().rust_mut().view = LibraryView { rows: view.rows.clone(), cored: view.cored.clone(), systems: view.systems.clone(), catalog: view.catalog.clone(), ..LibraryView::default() };
        self.as_mut().rust_mut().lines = lines;
        // The games' art may have changed with the view: a binding on `artHues` asks again.
        let read = self.art_hues_read.wrapping_add(1);
        self.as_mut().rust_mut().art_hues_read = read;
        self.as_mut().art_hues_read_changed();
    }

    /// The host's feed of the places whose `openPlace` it would decline, by key; their rows are not `available`.
    pub fn show_declined(mut self: Pin<&mut Self>, keys: &[String]) {
        if self.declined != keys {
            self.as_mut().rust_mut().declined = keys.to_vec();
            fill(self.places, view::places(&self.view, &self.lines, keys));
        }
    }

    /// The host's feed of the shown screen's section, by its key; empty for a screen that is none.
    pub fn show_section(mut self: Pin<&mut Self>, key: &str) {
        let key = QString::from(key);
        if self.section != key {
            self.as_mut().rust_mut().section = key;
            self.as_mut().section_changed();
        }
    }

    /// The host's feed of the suspended game's file, which the shelf leaves out; empty for none.
    pub fn show_suspended(mut self: Pin<&mut Self>, key: &str) {
        if self.suspended != key {
            self.as_mut().rust_mut().suspended = key.to_owned();
            fill(self.shelf, view::shelf(&self.view, key, words::now()));
        }
    }

    pub fn games_of(mut self: Pin<&mut Self>, id: &QString) -> *mut ThemeRows {
        let id = id.to_string();
        if let Some(list) = self.games.get(&id) {
            return *list;
        }
        let list = self.as_mut().made(&view::GAME_ROLES);
        fill(list, view::games_of(&self.view, &id, words::now()));
        self.as_mut().rust_mut().games.insert(id, list);
        list
    }

    pub fn first_of(mut self: Pin<&mut Self>, id: &QString, count: i32) -> *mut ThemeRows {
        let key = (id.to_string(), count.max(0) as usize);
        if let Some(list) = self.firsts.get(&key) {
            return *list;
        }
        let list = self.as_mut().made(&view::GAME_ROLES);
        fill(list, view::first_of(&self.view, &key.0, key.1, words::now()));
        self.as_mut().rust_mut().firsts.insert(key, list);
        list
    }

    pub fn consoles_by(mut self: Pin<&mut Self>, show: i32, order: i32, shortened: &QStringList) -> *mut ThemeRows {
        let (show, order) = (show.clamp(0, SHOWS.len() as i32 - 1) as usize, order.clamp(0, ORDERS.len() as i32 - 1) as usize);
        // Only the order by name reads the plates' names, so every other order is one list whatever the plates show.
        let mut short: Vec<String> = if ORDERS[order] == Order::Name { QList::<QString>::from(shortened).iter().map(|k| k.to_string()).collect() } else { Vec::new() };
        short.sort();
        short.dedup();
        let key = (show, order, short);
        if let Some(list) = self.listed.get(&key) {
            return *list;
        }
        let list = self.as_mut().made(&view::CONSOLE_ROLES);
        fill(list, view::consoles_listed(&self.view, SHOWS[key.0], ORDERS[key.1], &key.2));
        self.as_mut().rust_mut().listed.insert(key, list);
        list
    }

    pub fn search(mut self: Pin<&mut Self>, words: &QString) -> *mut ThemeRows {
        let words = words.to_string();
        fill(self.found, view::search(&self.view, &words, words::now()));
        self.as_mut().rust_mut().words = words;
        self.found
    }

    /// The list of a property, for the host's tests of what it fed.
    pub fn list(&self, name: &str) -> Option<&ThemeRows> {
        let list = match name {
            "consoles" => self.consoles,
            "places" => self.places,
            "shelves" => self.shelves,
            "recent" => self.recent,
            "shelf" => self.shelf,
            "favourites" => self.favourites,
            "collections" => self.collections,
            "sections" => self.sections,
            _ => return None,
        };
        // Safety: as `fill`.
        unsafe { list.as_ref() }
    }
}
