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
    /// The screen's actions as the kit's frame reads them (EmuSen_Kit.md §7.17).
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

impl qobject::HelpModel {
    pub fn listed(&self) -> Screen {
        match self.screen {
            HelpScreen::Consoles => Screen::Consoles,
            HelpScreen::Grid => Screen::Grid,
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
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
        roles.insert(BUTTON, QByteArray::from("button"));
        roles.insert(LABEL, QByteArray::from("label"));
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        entries(self.listed()).len() as i32
    }
}
