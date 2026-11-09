//! `Help`, the help bar's entries for what a theme's focus is on (EmuSen_Qt.md §3.2): `entries(kind)` gives the same
//! list each time, with the roles the kit's `HelpBar` reads and the actions `Frame` declares, from emusen-qt-models' help.rs.

use crate::glue::ffi;
use crate::view::HELP_ROLES;
use core::pin::Pin;
use cxx_qt::casting::Upcast;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QVariant};
use emusen_qt_models::help::{self, action_lines, entries, Screen};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!(<QtCore/QAbstractListModel>);
        type QAbstractListModel = crate::rows::qobject::QAbstractListModel;
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    /// What the focus is on: Continue, a game on Home, a console, a place, a game of a section, the search.
    #[qenum(ThemeHelp)]
    enum HelpKind {
        Continue,
        Game,
        Console,
        Place,
        Section,
        Search,
        SectionSearched,
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element = "Help"]
        #[qml_singleton]
        type ThemeHelp = super::ThemeHelpRust;

        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element = "HelpEntries"]
        #[qml_uncreatable]
        #[qproperty(QString, action_lines, cxx_name = "actionLines", READ, CONSTANT)]
        type ThemeHelpEntries = super::ThemeHelpEntriesRust;
    }

    extern "RustQt" {
        #[qinvokable]
        fn entries(self: &ThemeHelp, kind: HelpKind) -> *mut ThemeHelpEntries;

        #[qinvokable]
        #[cxx_override]
        fn data(self: &ThemeHelpEntries, index: &QModelIndex, role: i32) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &ThemeHelpEntries) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &ThemeHelpEntries, parent: &QModelIndex) -> i32;
    }

    impl cxx_qt::Initialize for ThemeHelp {}
}

pub use qobject::HelpKind;

/// Every kind, in the enum's order.
pub const KINDS: [HelpKind; 7] = [
    HelpKind::Continue,
    HelpKind::Game,
    HelpKind::Console,
    HelpKind::Place,
    HelpKind::Section,
    HelpKind::Search,
    HelpKind::SectionSearched,
];

/// The help bar a kind shows: Home's three, Choose a console's, the grid's and the search's.
pub fn screen_of(kind: HelpKind) -> Screen {
    match kind {
        HelpKind::Game => Screen::HomeShelf,
        HelpKind::Console => Screen::Consoles,
        HelpKind::Place => Screen::HomePlaces,
        HelpKind::Section => Screen::Grid,
        HelpKind::Search => Screen::Search,
        HelpKind::SectionSearched => Screen::GridSearched,
        _ => Screen::Home,
    }
}

/// One list for each kind, the singleton's children.
pub struct ThemeHelpRust {
    lists: [*mut qobject::ThemeHelpEntries; 7],
}

impl Default for ThemeHelpRust {
    fn default() -> Self {
        ThemeHelpRust { lists: [std::ptr::null_mut(); 7] }
    }
}

impl cxx_qt::Initialize for qobject::ThemeHelp {
    fn initialize(mut self: Pin<&mut Self>) {
        for (i, kind) in KINDS.into_iter().enumerate() {
            let list = ffi::new_help_entries(self.as_mut().upcast_pin());
            // Safety: the list was just made, a child of this singleton.
            if let Some(entries) = unsafe { list.as_mut() } {
                unsafe { Pin::new_unchecked(entries) }.show(screen_of(kind));
            }
            self.as_mut().rust_mut().lists[i] = list;
        }
    }
}

impl qobject::ThemeHelp {
    pub fn entries(&self, kind: HelpKind) -> *mut qobject::ThemeHelpEntries {
        KINDS.iter().position(|k| *k == kind).map_or(std::ptr::null_mut(), |i| self.lists[i])
    }
}

/// A help bar's screen and its actions as the kit's frame reads them (EmuSen_Kit.md §7.19).
pub struct ThemeHelpEntriesRust {
    screen: Screen,
    action_lines: QString,
}

impl Default for ThemeHelpEntriesRust {
    fn default() -> Self {
        ThemeHelpEntriesRust { screen: Screen::Home, action_lines: QString::from(action_lines(Screen::Home).as_str()) }
    }
}

impl qobject::ThemeHelpEntries {
    /// The screen whose entries the list holds, set once when it is made.
    fn show(mut self: Pin<&mut Self>, screen: Screen) {
        self.as_mut().rust_mut().screen = screen;
        self.as_mut().rust_mut().action_lines = QString::from(action_lines(screen).as_str());
    }

    /// The help bar's screen, for the host's tests.
    pub fn screen(&self) -> Screen {
        self.screen
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some((button, label, _)) = entries(self.screen).get(index.row() as usize) else {
            return QVariant::default();
        };
        match role {
            0 => QVariant::from(&(button.index() as i32)),
            1 => QVariant::from(&QString::from(*label)),
            2 => QVariant::from(&help::pair_of(*button).map_or(-1, |b| b.index() as i32)),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
        for (i, name) in HELP_ROLES.iter().enumerate() {
            roles.insert(i as i32, QByteArray::from(*name));
        }
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        entries(self.screen).len() as i32
    }
}
