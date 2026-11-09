//! `Library`, the library as a theme reads it (EmuSen_Qt.md §3.2): the consoles, Home's places, the three shelves, the
//! recent and favourite games, and any console's or place's games, each a read-only list fed by the host from the view.

use crate::glue::ffi;
use crate::rows::qobject::ThemeRows;
use crate::view::{self, Row};
use core::pin::Pin;
use cxx_qt::casting::Upcast;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use emusen_qt_models::library::{LibraryView, FAVOURITES, RECENT};
use emusen_qt_models::words;
use std::collections::HashMap;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
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
    }

    impl cxx_qt::Initialize for ThemeLibrary {}
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
    games: HashMap<String, *mut ThemeRows>,
    firsts: HashMap<(String, usize), *mut ThemeRows>,
    found: *mut ThemeRows,
    words: String,
    view: LibraryView,
    /// Home's lines as last fed, and the places the host would decline.
    lines: Vec<(String, bool)>,
    declined: Vec<String>,
}

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
            games: HashMap::new(),
            firsts: HashMap::new(),
            found: none,
            words: String::new(),
            view: LibraryView::empty(),
            lines: Vec::new(),
            declined: Vec::new(),
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
        let mut me = self.as_mut().rust_mut();
        me.consoles = consoles;
        me.places = places;
        me.shelves = shelves;
        me.recent = recent;
        me.shelf = shelf;
        me.favourites = favourites;
        me.collections = collections;
        me.found = found;
        drop(me);
        self.show(&LibraryView::empty(), "[]");
    }
}

impl qobject::ThemeLibrary {
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
        self.as_mut().rust_mut().view = view.clone();
        self.as_mut().rust_mut().lines = lines;
    }

    /// The host's feed of the places whose `openPlace` it would decline, by key; their rows are not `available`.
    pub fn show_declined(mut self: Pin<&mut Self>, keys: &[String]) {
        if self.declined != keys {
            self.as_mut().rust_mut().declined = keys.to_vec();
            fill(self.places, view::places(&self.view, &self.lines, keys));
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
            _ => return None,
        };
        // Safety: as `fill`.
        unsafe { list.as_ref() }
    }
}
