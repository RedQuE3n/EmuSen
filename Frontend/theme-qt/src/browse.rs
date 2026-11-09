//! `Browse`, the section a Games screen shows as the program holds it (EmuSen_Qt.md §3.2): its key and heading, its games
//! in the order shown (sorted, narrowed and searched by the List card and the search, which are EmuSen's), the neighbours
//! the shoulders step to, and whether the search or the List card is open over it. The host feeds it; a theme reports its
//! focused game through `Actions.focus`, and `focusRow` is where the program holds the focus, moved by the List card too.

use crate::rows::qobject::ThemeRows;
use crate::view::Row;
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
        }
    }
}

impl cxx_qt::Initialize for qobject::ThemeBrowse {
    fn initialize(mut self: Pin<&mut Self>) {
        let list = crate::glue::ffi::new_rows(self.as_mut().upcast_pin());
        // Safety: the list was just made, a child of this singleton.
        if let Some(rows) = unsafe { list.as_mut() } {
            unsafe { Pin::new_unchecked(rows) }.set_roles(&crate::view::GAME_ROLES);
        }
        self.as_mut().rust_mut().games = list;
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
    }

    /// The host's feed of the section's games as shown, replaced whole.
    pub fn show_games(&self, rows: Vec<Row>) {
        // Safety: the list is the singleton's child, made at its start and alive as long as it.
        if let Some(list) = unsafe { self.games.as_mut() } {
            unsafe { Pin::new_unchecked(list) }.replace(rows);
        }
    }
}
