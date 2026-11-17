//! `Help`, the help bar's entries for what a theme's focus is on, from emusen-qt-models' help.rs (EmuSen_Qt.md §3.2, `Help`).

use crate::glue::ffi;
use crate::view::HELP_ROLES;
use core::pin::Pin;
use cxx_qt::casting::Upcast;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QMap, QMapPair_QString_QVariant, QModelIndex, QString, QVariant};
use emusen_qt_models::help::{self, action_lines, entries_grouped, entries_with, Entry, Screen};
use emusen_kit_core::input::UiButton;

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
        include!("cxx-qt-lib/qmap.h");
        type QMap_QString_QVariant = cxx_qt_lib::QMap<cxx_qt_lib::QMapPair_QString_QVariant>;
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
        Settings,
        SettingsPage,
        SettingsConsole,
        SettingsChoices,
        Page,
        PageResume,
        PageResumeSlots,
        PageFavourite,
        PageLoad,
        PageView,
        PageValue,
        PageChange,
        PageChoose,
        PageNone,
        PageEdit,
        PageCoverArt,
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
        /// The kind's list with the entries `group`, `back` and `accept` ask for (EmuSen_Qt.md §3.2, `Help`).
        #[qinvokable]
        #[cxx_name = "entries"]
        fn entries_with(self: &ThemeHelp, kind: HelpKind, flags: &QMap_QString_QVariant) -> *mut ThemeHelpEntries;

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
pub const KINDS: [HelpKind; 23] = [
    HelpKind::Continue,
    HelpKind::Game,
    HelpKind::Console,
    HelpKind::Place,
    HelpKind::Section,
    HelpKind::Search,
    HelpKind::SectionSearched,
    HelpKind::Settings,
    HelpKind::SettingsPage,
    HelpKind::SettingsConsole,
    HelpKind::SettingsChoices,
    HelpKind::Page,
    HelpKind::PageResume,
    HelpKind::PageResumeSlots,
    HelpKind::PageFavourite,
    HelpKind::PageLoad,
    HelpKind::PageView,
    HelpKind::PageValue,
    HelpKind::PageChange,
    HelpKind::PageChoose,
    HelpKind::PageNone,
    HelpKind::PageEdit,
    HelpKind::PageCoverArt,
];

/// The help bar a kind shows: Home's three, Choose a console's, the grid's and the search's, Settings' four and a game's
/// page's by what its focus is on.
pub fn screen_of(kind: HelpKind) -> Screen {
    match kind {
        HelpKind::Game => Screen::HomeShelf,
        HelpKind::Console => Screen::Consoles,
        HelpKind::Place => Screen::HomePlaces,
        HelpKind::Section => Screen::Grid,
        HelpKind::Search => Screen::Search,
        HelpKind::SectionSearched => Screen::GridSearched,
        HelpKind::Settings => Screen::Settings,
        HelpKind::SettingsPage => Screen::SettingsPage,
        HelpKind::SettingsConsole => Screen::SettingsConsole,
        HelpKind::SettingsChoices => Screen::SettingsChoices,
        HelpKind::Page => Screen::GamePage,
        HelpKind::PageResume => Screen::GamePageResume,
        HelpKind::PageResumeSlots => Screen::GamePageResumeSlots,
        HelpKind::PageFavourite => Screen::GamePageFavourite,
        HelpKind::PageLoad => Screen::GamePageLoad,
        HelpKind::PageView => Screen::GamePageView,
        HelpKind::PageValue => Screen::GamePageValue,
        HelpKind::PageChange => Screen::GamePageChange,
        HelpKind::PageChoose => Screen::GamePageChoose,
        HelpKind::PageNone => Screen::GamePageNone,
        HelpKind::PageEdit => Screen::GamePageEdit,
        HelpKind::PageCoverArt => Screen::GamePageCoverArt,
        _ => Screen::Home,
    }
}

/// The kind whose bar is a screen's, for Settings' and a game's page's models; none for a screen no kind shows (a bindings
/// page, a row listening, the media viewer). Home's kinds are a theme's own choice and never answered here.
pub fn kind_of(screen: Screen) -> Option<HelpKind> {
    KINDS.into_iter().skip_while(|k| *k != HelpKind::Settings).find(|k| screen_of(*k) == screen)
}

/// What a list may differ by, a kind's 72 lists: group, back, accept, actions kept, and `Names` (EmuSen_Qt.md §3.2).
pub const FLAGS: [(Option<Entry>, bool, bool, bool, Names); 72] = {
    const G: [Option<Entry>; 3] = [None, Some(help::GROUP), Some(help::MAKER)];
    const N: [Names; 3] = [Names::Own, Names::Card, Names::Options];
    let mut flags = [(None, false, true, true, Names::Own); 72];
    let mut f = 0;
    while f < 72 {
        flags[f] = (G[f % 3], (f / 3) % 2 == 1, (f / 6) % 2 == 0, (f / 12) % 2 == 0, N[f / 24]);
        f += 1;
    }
    flags
};

/// How a list's entries are named: the screen's own, Eclipse Home's, or Eclipse's Choose a console's Sort and Options.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Names {
    Own,
    Card,
    Options,
}

/// Seventy-two lists for each kind, one for each of `FLAGS`, the singleton's children.
pub struct ThemeHelpRust {
    lists: [[*mut qobject::ThemeHelpEntries; 72]; KINDS.len()],
}

impl Default for ThemeHelpRust {
    fn default() -> Self {
        ThemeHelpRust { lists: [[std::ptr::null_mut(); 72]; KINDS.len()] }
    }
}

impl cxx_qt::Initialize for qobject::ThemeHelp {
    fn initialize(mut self: Pin<&mut Self>) {
        for (i, kind) in KINDS.into_iter().enumerate() {
            for (f, (group, back, accept, actions, names)) in FLAGS.into_iter().enumerate() {
                let list = ffi::new_help_entries(self.as_mut().upcast_pin());
                // Safety: the list was just made, a child of this singleton.
                if let Some(entries) = unsafe { list.as_mut() } {
                    unsafe { Pin::new_unchecked(entries) }.show(screen_of(kind), group, back, accept, actions, names);
                }
                self.as_mut().rust_mut().lists[i][f] = list;
            }
        }
    }
}

impl qobject::ThemeHelp {
    pub fn entries(&self, kind: HelpKind) -> *mut qobject::ThemeHelpEntries {
        self.list(kind, None, false, true, true, Names::Own)
    }

    pub fn entries_with(&self, kind: HelpKind, flags: &QMap<QMapPair_QString_QVariant>) -> *mut qobject::ThemeHelpEntries {
        let flag = |name: &str, unset: bool| flags.get(&QString::from(name)).and_then(|v| v.value::<bool>()).unwrap_or(unset);
        let group = flag("group", false).then_some(if flag("triggers", false) { help::MAKER } else { help::GROUP });
        let names = if flag("card", false) {
            Names::Card
        } else if flag("options", false) {
            Names::Options
        } else {
            Names::Own
        };
        self.list(kind, group, flag("back", false), flag("accept", true), flag("actions", true), names)
    }

    /// A kind's list with the entries asked for added.
    pub fn list(&self, kind: HelpKind, group: Option<Entry>, back: bool, accept: bool, actions: bool, names: Names) -> *mut qobject::ThemeHelpEntries {
        let f = FLAGS.iter().position(|x| *x == (group, back, accept, actions, names)).unwrap_or(0);
        KINDS.iter().position(|k| *k == kind).map_or(std::ptr::null_mut(), |i| self.lists[i][f])
    }
}

/// A help bar's screen and its actions as the kit's frame reads them (EmuSen_Kit.md §7.19).
pub struct ThemeHelpEntriesRust {
    screen: Screen,
    list: Vec<Entry>,
    accept: bool,
    action_lines: QString,
}

impl Default for ThemeHelpEntriesRust {
    fn default() -> Self {
        ThemeHelpEntriesRust {
            screen: Screen::Home,
            list: entries_with(Screen::Home, false, false),
            accept: true,
            action_lines: QString::from(action_lines(Screen::Home).as_str()),
        }
    }
}

impl qobject::ThemeHelpEntries {
    /// The screen whose entries the list holds, with the added entries asked for, set once when it is made.
    fn show(mut self: Pin<&mut Self>, screen: Screen, group: Option<Entry>, back: bool, accept: bool, actions: bool, names: Names) {
        self.as_mut().rust_mut().screen = screen;
        self.as_mut().rust_mut().accept = accept;
        let list = entries_grouped(screen, group, back);
        let list = if actions { list } else { help::without_actions(list) };
        let (list, lines) = match names {
            Names::Own => (list, action_lines(screen)),
            Names::Card => (help::card_named(list), action_lines(screen)),
            Names::Options => {
                let named = help::options_named(list);
                let lines = help::lines_of(screen, &named);
                (named, lines)
            }
        };
        self.as_mut().rust_mut().list = list;
        self.as_mut().rust_mut().action_lines = QString::from(lines.as_str());
    }

    /// The help bar's screen, for the host's tests.
    pub fn screen(&self) -> Screen {
        self.screen
    }

    /// Whether the host would take A's action, as the list was asked for.
    pub fn accepts(&self) -> bool {
        self.accept
    }

    /// The entries as the bar shows them, for the host's tests.
    pub fn list(&self) -> &[Entry] {
        &self.list
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some((button, label, action)) = self.list.get(index.row() as usize) else {
            return QVariant::default();
        };
        match role {
            0 => QVariant::from(&(button.index() as i32)),
            1 => QVariant::from(&QString::from(*label)),
            2 => QVariant::from(&help::pair_of(*button).map_or(-1, |b| b.index() as i32)),
            3 => QVariant::from(&(self.accept || *button != UiButton::Accept)),
            4 => QVariant::from(&QString::from(action.unwrap_or_default())),
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
        self.list.len() as i32
    }
}
