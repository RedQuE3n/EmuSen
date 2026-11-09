//! `HelpModel`, a screen's help bar as a QAbstractListModel with the roles the kit's HelpBar reads (button, label), from
//! help.rs, so no screen writes its entries in QML (EmuSen_Qt.md §1.3).

use crate::help::{action_lines, entries, Screen};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QVariant};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!(<QtCore/QAbstractListModel>);
        type QAbstractListModel = crate::library_model::qobject::QAbstractListModel;
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    /// The screens with a help bar.
    #[qenum(HelpModel)]
    enum HelpScreen {
        Home,
        Consoles,
        Grid,
        HomeShelf,
        HomePlaces,
        Pause,
        PausePage,
        Card,
        Search,
        PauseSlots,
        PauseReel,
        Question,
        QuestionBack,
        GridSearched,
        PauseDisc,
        Themes,
        ThemesChecked,
        Settings,
        SettingsPage,
        GamePage,
        GamePageResume,
        GamePageResumeSlots,
        GamePageFavourite,
        GamePageLoad,
        GamePageView,
        GamePageValue,
        GamePageChange,
        GamePageChoose,
        GamePageNone,
    }

    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        #[qproperty(HelpScreen, screen)]
        #[qproperty(QString, action_lines, cxx_name = "actionLines", READ, NOTIFY)]
        type HelpModel = super::HelpModelRust;
    }

    extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn data(self: &HelpModel, index: &QModelIndex, role: i32) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &HelpModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &HelpModel, parent: &QModelIndex) -> i32;
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut HelpModel>);

        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut HelpModel>);
    }

    impl cxx_qt::Initialize for HelpModel {}
}

use qobject::HelpScreen;

/// The model's state: the screen whose entries it lists.
pub struct HelpModelRust {
    screen: HelpScreen,
    /// The screen's actions as the kit's frame reads them (EmuSen_Kit.md §7.19).
    action_lines: QString,
}

impl Default for HelpModelRust {
    fn default() -> Self {
        HelpModelRust { screen: HelpScreen::Home, action_lines: QString::from(action_lines(Screen::Home).as_str()) }
    }
}

impl cxx_qt::Initialize for qobject::HelpModel {
    fn initialize(self: Pin<&mut Self>) {
        self.on_screen_changed(|mut model| {
            model.as_mut().begin_reset_model();
            model.as_mut().end_reset_model();
            let lines = QString::from(action_lines(model.listed()).as_str());
            model.as_mut().rust_mut().action_lines = lines;
            model.as_mut().action_lines_changed();
        })
        .release();
    }
}

/// The roles: the button's number as PadRouter takes it, and the words.
const BUTTON: i32 = 0;
const LABEL: i32 = 1;
/// The second button an entry names, the right shoulder beside the left; -1 for none.
const PAIR: i32 = 2;

impl qobject::HelpModel {
    pub fn listed(&self) -> Screen {
        match self.screen {
            HelpScreen::Consoles => Screen::Consoles,
            HelpScreen::Grid => Screen::Grid,
            HelpScreen::HomeShelf => Screen::HomeShelf,
            HelpScreen::HomePlaces => Screen::HomePlaces,
            HelpScreen::Pause => Screen::Pause,
            HelpScreen::PausePage => Screen::PausePage,
            HelpScreen::Card => Screen::Card,
            HelpScreen::Search => Screen::Search,
            HelpScreen::PauseSlots => Screen::PauseSlots,
            HelpScreen::PauseReel => Screen::PauseReel,
            HelpScreen::Question => Screen::Question,
            HelpScreen::QuestionBack => Screen::QuestionBack,
            HelpScreen::GridSearched => Screen::GridSearched,
            HelpScreen::PauseDisc => Screen::PauseDisc,
            HelpScreen::Themes => Screen::Themes,
            HelpScreen::ThemesChecked => Screen::ThemesChecked,
            HelpScreen::Settings => Screen::Settings,
            HelpScreen::SettingsPage => Screen::SettingsPage,
            HelpScreen::GamePage => Screen::GamePage,
            HelpScreen::GamePageResume => Screen::GamePageResume,
            HelpScreen::GamePageResumeSlots => Screen::GamePageResumeSlots,
            HelpScreen::GamePageFavourite => Screen::GamePageFavourite,
            HelpScreen::GamePageLoad => Screen::GamePageLoad,
            HelpScreen::GamePageView => Screen::GamePageView,
            HelpScreen::GamePageValue => Screen::GamePageValue,
            HelpScreen::GamePageChange => Screen::GamePageChange,
            HelpScreen::GamePageChoose => Screen::GamePageChoose,
            HelpScreen::GamePageNone => Screen::GamePageNone,
            _ => Screen::Home,
        }
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some((button, label, _)) = entries(self.listed()).get(index.row() as usize) else {
            return QVariant::default();
        };
        match role {
            BUTTON => QVariant::from(&(button.index() as i32)),
            LABEL => QVariant::from(&QString::from(*label)),
            PAIR => QVariant::from(&crate::help::pair_of(*button).map_or(-1, |b| b.index() as i32)),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
        roles.insert(BUTTON, QByteArray::from("button"));
        roles.insert(LABEL, QByteArray::from("label"));
        roles.insert(PAIR, QByteArray::from("pair"));
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        entries(self.listed()).len() as i32
    }
}

/// A screen's number as QML names it (`HelpModel.Pause`), for a model that says which bar its page shows.
pub fn shown_screen(screen: Screen) -> i32 {
    let shown = match screen {
        Screen::Home => HelpScreen::Home,
        Screen::Consoles => HelpScreen::Consoles,
        Screen::Grid => HelpScreen::Grid,
        Screen::HomeShelf => HelpScreen::HomeShelf,
        Screen::HomePlaces => HelpScreen::HomePlaces,
        Screen::Pause => HelpScreen::Pause,
        Screen::PausePage => HelpScreen::PausePage,
        Screen::PauseSlots => HelpScreen::PauseSlots,
        Screen::PauseReel => HelpScreen::PauseReel,
        Screen::Card => HelpScreen::Card,
        Screen::Search => HelpScreen::Search,
        Screen::Question => HelpScreen::Question,
        Screen::QuestionBack => HelpScreen::QuestionBack,
        Screen::GridSearched => HelpScreen::GridSearched,
        Screen::PauseDisc => HelpScreen::PauseDisc,
        Screen::Themes => HelpScreen::Themes,
        Screen::ThemesChecked => HelpScreen::ThemesChecked,
        Screen::Settings => HelpScreen::Settings,
        Screen::SettingsPage => HelpScreen::SettingsPage,
        Screen::GamePage => HelpScreen::GamePage,
        Screen::GamePageResume => HelpScreen::GamePageResume,
        Screen::GamePageResumeSlots => HelpScreen::GamePageResumeSlots,
        Screen::GamePageFavourite => HelpScreen::GamePageFavourite,
        Screen::GamePageLoad => HelpScreen::GamePageLoad,
        Screen::GamePageView => HelpScreen::GamePageView,
        Screen::GamePageValue => HelpScreen::GamePageValue,
        Screen::GamePageChange => HelpScreen::GamePageChange,
        Screen::GamePageChoose => HelpScreen::GamePageChoose,
        Screen::GamePageNone => HelpScreen::GamePageNone,
    };
    shown.repr
}
