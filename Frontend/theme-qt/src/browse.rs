//! `Browse`, the section a Games screen shows as the program holds it (EmuSen_Qt.md §3.2): its key and heading, its games
//! in the order shown (sorted, narrowed and searched by the List card and the search, which are EmuSen's), the neighbours
//! the shoulders step to, and whether the search or the List card is open over it. The host feeds it; a theme reports its
//! focused game through `Actions.focus`, and `focusRow` is where the program holds the focus, moved by the List card too.

use crate::rows::qobject::ThemeRows;
use crate::view::Row;
use emusen_qt_models::library::LibraryView;
use emusen_qt_models::words;
use core::pin::Pin;
use cxx_qt::casting::Upcast;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;

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
        #[qml_element = "Browse"]
        #[qml_singleton]
        #[qproperty(QString, section, READ, NOTIFY)]
        #[qproperty(QString, heading, READ, NOTIFY)]
        #[qproperty(QString, world, READ, NOTIFY)]
        #[qproperty(f64, hue, READ, NOTIFY)]
        #[qproperty(QString, count_line, cxx_name = "countLine", READ, NOTIFY)]
        #[qproperty(QString, previous, READ, NOTIFY)]
        #[qproperty(QString, next, READ, NOTIFY)]
        #[qproperty(i32, focus_row, cxx_name = "focusRow", READ, NOTIFY)]
        #[qproperty(bool, searching, READ, NOTIFY)]
        #[qproperty(bool, typing, READ, NOTIFY)]
        #[qproperty(bool, list_open, cxx_name = "listOpen", READ, NOTIFY)]
        #[qproperty(*mut ThemeRows, games, READ, CONSTANT)]
        #[qproperty(bool, searched, READ, NOTIFY)]
        /// Which list the section is: `all`, `favourites`, `recent` or `console` (1.7, R-20261115-13).
        #[qproperty(QString, kind, READ, NOTIFY)]
        /// What the list says with nothing in it, by the list (1.7, R-20261115-13).
        #[qproperty(QString, empty_words, cxx_name = "emptyWords", READ, NOTIFY)]
        /// The Filter page's choices, each `key`, `name`, `count` and `line` (name and count as a chip says them), and the key of the one in force (1.7, R-20261115-10).
        #[qproperty(*mut ThemeRows, filters, READ, CONSTANT)]
        #[qproperty(QString, filter, READ, NOTIFY)]
        /// The sort in force in the List card's words (1.7, R-20261115-11).
        #[qproperty(QString, sort, READ, NOTIFY)]
        /// The shown games' first letters in order, each a `letter`, and the focused game's (1.7, R-20261115-12).
        #[qproperty(*mut ThemeRows, letters, READ, CONSTANT)]
        #[qproperty(QString, letter, READ, NOTIFY)]
        type ThemeBrowse = super::ThemeBrowseRust;
    }

    impl cxx_qt::Initialize for ThemeBrowse {}
}

/// What the host knows of the section shown.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BrowseFacts {
    pub section: String,
    pub heading: String,
    /// The console's world as one line (the kit's `Body.world`), empty for a shelf.
    pub world: String,
    pub hue: f64,
    /// The count under the heading: the search's or the section's.
    pub count_line: String,
    /// The names of the rows before and after on Choose a console, which the shoulders step to.
    pub previous: String,
    pub next: String,
    /// The row the focus is on, as the program holds it; -1 for none.
    pub focus_row: i32,
    pub searching: bool,
    pub typing: bool,
    pub list_open: bool,
    /// Whether the search's words narrow the section, kept after its keyboard closes.
    pub searched: bool,
    pub kind: String,
    pub empty_words: String,
    pub filter: String,
    pub sort: String,
    pub letter: String,
}

pub struct ThemeBrowseRust {
    section: QString,
    heading: QString,
    world: QString,
    hue: f64,
    count_line: QString,
    previous: QString,
    next: QString,
    focus_row: i32,
    searching: bool,
    typing: bool,
    list_open: bool,
    games: *mut ThemeRows,
    searched: bool,
    kind: QString,
    empty_words: QString,
    filters: *mut ThemeRows,
    filter: QString,
    sort: QString,
    letters: *mut ThemeRows,
    letter: QString,
}

impl Default for ThemeBrowseRust {
    fn default() -> Self {
        ThemeBrowseRust {
            section: QString::default(),
            heading: QString::default(),
            world: QString::default(),
            hue: 0.0,
            count_line: QString::default(),
            previous: QString::default(),
            next: QString::default(),
            focus_row: -1,
            searching: false,
            typing: false,
            list_open: false,
            games: std::ptr::null_mut(),
            searched: false,
            kind: QString::default(),
            empty_words: QString::default(),
            filters: std::ptr::null_mut(),
            filter: QString::default(),
            sort: QString::default(),
            letters: std::ptr::null_mut(),
            letter: QString::default(),
        }
    }
}

impl cxx_qt::Initialize for qobject::ThemeBrowse {
    fn initialize(mut self: Pin<&mut Self>) {
        let mut made = |roles: &'static [&'static str]| {
            let list = crate::glue::ffi::new_rows(self.as_mut().upcast_pin());
            // Safety: the list was just made, a child of this singleton.
            if let Some(rows) = unsafe { list.as_mut() } {
                unsafe { Pin::new_unchecked(rows) }.set_roles(roles);
            }
            list
        };
        let (games, filters, letters) = (made(&crate::view::GAME_ROLES), made(&crate::view::FILTER_ROLES), made(&crate::view::LETTER_ROLES));
        self.as_mut().rust_mut().games = games;
        self.as_mut().rust_mut().filters = filters;
        self.as_mut().rust_mut().letters = letters;
    }
}

/// Sets a text property where it changed, and says so.
macro_rules! text {
    ($me:ident, $field:ident, $changed:ident, $value:expr) => {{
        let value = QString::from($value.as_str());
        if $me.$field != value {
            $me.as_mut().rust_mut().$field = value;
            $me.as_mut().$changed();
        }
    }};
}

/// Sets a plain property where it changed, and says so.
macro_rules! plain {
    ($me:ident, $field:ident, $changed:ident, $value:expr) => {{
        if $me.$field != $value {
            $me.as_mut().rust_mut().$field = $value;
            $me.as_mut().$changed();
        }
    }};
}

impl qobject::ThemeBrowse {
    /// The host's feed of the section's facts; each property notifies only when it changed.
    pub fn show(mut self: Pin<&mut Self>, facts: &BrowseFacts) {
        text!(self, section, section_changed, facts.section);
        text!(self, heading, heading_changed, facts.heading);
        text!(self, world, world_changed, facts.world);
        plain!(self, hue, hue_changed, facts.hue);
        text!(self, count_line, count_line_changed, facts.count_line);
        text!(self, previous, previous_changed, facts.previous);
        text!(self, next, next_changed, facts.next);
        plain!(self, focus_row, focus_row_changed, facts.focus_row);
        plain!(self, searching, searching_changed, facts.searching);
        plain!(self, typing, typing_changed, facts.typing);
        plain!(self, list_open, list_open_changed, facts.list_open);
        plain!(self, searched, searched_changed, facts.searched);
        text!(self, kind, kind_changed, facts.kind);
        text!(self, empty_words, empty_words_changed, facts.empty_words);
        text!(self, filter, filter_changed, facts.filter);
        text!(self, sort, sort_changed, facts.sort);
        text!(self, letter, letter_changed, facts.letter);
    }

    /// The host's feed of the Filter page's choices and the shown games' letters, each replaced whole.
    pub fn show_listing(&self, filters: Vec<Row>, letters: Vec<Row>) {
        for (list, rows) in [(self.filters, filters), (self.letters, letters)] {
            // Safety: each list is the singleton's child, made at its start and alive as long as it.
            if let Some(list) = unsafe { list.as_mut() } {
                unsafe { Pin::new_unchecked(list) }.replace(rows);
            }
        }
    }

    /// The host's feed of the section's games as shown, replaced whole.
    pub fn show_games(&self, rows: Vec<Row>) {
        // Safety: the list is the singleton's child, made at its start and alive as long as it.
        if let Some(list) = unsafe { self.games.as_mut() } {
            unsafe { Pin::new_unchecked(list) }.replace(rows);
        }
    }
}

/// The section's facts as Choose a console's row for its key holds them; the shoulders' neighbours are the rows either
/// side, and the count is the search's while one narrows the section.
pub fn facts_of(view: &LibraryView, section: &str, count_line: &str, focus_row: i32, open: (bool, bool, bool), searched: bool) -> BrowseFacts {
    let rows = view.rows.rows();
    let at = rows.iter().position(|r| r.key == section);
    let row = at.map(|i| &rows[i]);
    let beside = |by: usize| at.filter(|_| rows.len() > 1).map(|i| rows[(i + by) % rows.len()].name.clone()).unwrap_or_default();
    BrowseFacts {
        section: section.to_owned(),
        heading: row.map(|r| r.heading.clone()).unwrap_or_default(),
        world: row.map(|r| r.world.clone()).unwrap_or_default(),
        hue: row.map_or(0.0, |r| r.hue),
        count_line: if count_line.is_empty() { row.map(|r| words::games(r.count)).unwrap_or_default() } else { count_line.to_owned() },
        previous: beside(rows.len().saturating_sub(1)),
        next: beside(1),
        focus_row,
        searching: open.0,
        typing: open.1,
        list_open: open.2,
        searched,
        kind: emusen_qt_models::library::section_kind(section).to_owned(),
        ..BrowseFacts::default()
    }
}
