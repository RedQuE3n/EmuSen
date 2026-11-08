//! `GameModel`, a list of games as a QAbstractListModel (EmuSen_Qt.md §7.1): the grid's cells, Home's shelf and Choose a
//! console's fan, each set from LibraryModel's `gamesOf` as plain data.

use crate::library::GameRow;
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

    /// The roles, from 0 as cxx-qt numbers them (§10.4); QML reads them by name.
    #[qenum(GameModel)]
    enum GameRole {
        Path,
        System,
        Title,
        Hue,
        Favourite,
    }

    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        #[qproperty(QString, games)]
        #[qproperty(i32, count, READ, NOTIFY)]
        type GameModel = super::GameModelRust;
    }

    extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn data(self: &GameModel, index: &QModelIndex, role: i32) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &GameModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &GameModel, parent: &QModelIndex) -> i32;

        /// A game's title by its row; empty where there is none.
        #[qinvokable]
        #[cxx_name = "titleAt"]
        fn title_at(self: &GameModel, row: i32) -> QString;

        /// A game's path by its row; empty where there is none.
        #[qinvokable]
        #[cxx_name = "pathAt"]
        fn path_at(self: &GameModel, row: i32) -> QString;

        /// A game's console's hue by its row; -1 where there is none.
        #[qinvokable]
        #[cxx_name = "hueAt"]
        fn hue_at(self: &GameModel, row: i32) -> f64;
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut GameModel>);

        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut GameModel>);
    }

    impl cxx_qt::Initialize for GameModel {}
}

use qobject::GameRole;

/// The model's state: the games as given and as read.
#[derive(Default)]
pub struct GameModelRust {
    games: QString,
    count: i32,
    rows: Vec<GameRow>,
}

impl cxx_qt::Initialize for qobject::GameModel {
    fn initialize(self: Pin<&mut Self>) {
        self.on_games_changed(|model| model.read()).release();
    }
}

impl qobject::GameModel {
    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some(row) = self.rows.get(index.row() as usize) else {
            return QVariant::default();
        };
        let text = |s: &str| QVariant::from(&QString::from(s));
        match (GameRole { repr: role }) {
            GameRole::Path => text(&row.path),
            GameRole::System => text(&row.system),
            GameRole::Title => text(&row.title),
            GameRole::Hue => QVariant::from(&row.hue),
            GameRole::Favourite => QVariant::from(&row.favourite),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
        let names = [
            (GameRole::Path, "path"),
            (GameRole::System, "system"),
            (GameRole::Title, "title"),
            (GameRole::Hue, "hue"),
            (GameRole::Favourite, "favourite"),
        ];
        for (role, name) in names {
            roles.insert(role.repr, QByteArray::from(name));
        }
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.rows.len() as i32
    }

    fn at(&self, row: i32) -> Option<&GameRow> {
        usize::try_from(row).ok().and_then(|i| self.rows.get(i))
    }

    pub fn title_at(&self, row: i32) -> QString {
        self.at(row).map_or_else(QString::default, |g| QString::from(g.title.as_str()))
    }

    pub fn path_at(&self, row: i32) -> QString {
        self.at(row).map_or_else(QString::default, |g| QString::from(g.path.as_str()))
    }

    pub fn hue_at(&self, row: i32) -> f64 {
        self.at(row).map_or(-1.0, |g| g.hue)
    }

    /// The games text read into rows, the model reset once.
    fn read(mut self: Pin<&mut Self>) {
        let rows = crate::games::from_json(&self.games.to_string());
        let count = rows.len() as i32;
        self.as_mut().begin_reset_model();
        self.as_mut().rust_mut().rows = rows;
        self.as_mut().end_reset_model();
        if self.count != count {
            self.as_mut().rust_mut().count = count;
            self.count_changed();
        }
    }
}
