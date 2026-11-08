//! `GameModel`, a list of games as a QAbstractListModel (EmuSen_Qt.md §7.1): the grid's cells, Home's shelf and Choose a
//! console's fan, each set from LibraryModel's `gamesOf` as plain data, read off the GUI thread and taken back through
//! qt_thread().queue, so a list of thousands never stops the screens.

use crate::library::GameRow;
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
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
        Cover,
        Facts,
    }

    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        #[qproperty(QString, games)]
        #[qproperty(QString, leave_out, cxx_name = "leaveOut")]
        #[qproperty(i32, count, READ, NOTIFY)]
        #[qproperty(i32, revision, READ, NOTIFY)]
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

        /// A game's cover as a URL by its row; empty where it has none.
        #[qinvokable]
        #[cxx_name = "coverAt"]
        fn cover_at(self: &GameModel, row: i32) -> QString;

        /// A game's hero art as a URL by its row, else its cover's; empty where it has neither (EmuSen_Qt.md §7.1, step 6).
        #[qinvokable]
        #[cxx_name = "heroAt"]
        fn hero_at(self: &GameModel, row: i32) -> QString;

        /// A game's console's name by its row; empty where there is none.
        #[qinvokable]
        #[cxx_name = "consoleAt"]
        fn console_at(self: &GameModel, row: i32) -> QString;

        /// The row of the game at `path`; -1 where it is not in the list.
        #[qinvokable]
        #[cxx_name = "rowOf"]
        fn row_of(self: &GameModel, path: &QString) -> i32;

        /// A game's facts by its row: its console and when it was last played.
        #[qinvokable]
        #[cxx_name = "factsAt"]
        fn facts_at(self: &GameModel, row: i32) -> QString;

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
    impl cxx_qt::Threading for GameModel {}
}

use qobject::GameRole;

/// A cover's file as the URL an Image reads; empty for none.
pub fn cover_url(file: &str) -> String {
    if file.is_empty() {
        String::new()
    } else {
        cxx_qt_lib::QUrl::from_local_file(&QString::from(file)).to_string()
    }
}

/// The model's state: the games as given and as read.
#[derive(Default)]
pub struct GameModelRust {
    games: QString,
    /// A game's path the list leaves out: the suspended game, which Continue holds, on Home's shelf.
    leave_out: QString,
    count: i32,
    /// Counts the reads, announced after each, so a binding on a row's words sees the new rows.
    revision: i32,
    rows: Vec<GameRow>,
    /// Counts the reads asked for, so only the newest read's rows are taken.
    asked: u64,
}

impl cxx_qt::Initialize for qobject::GameModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_games_changed(|model| model.read()).release();
        self.on_leave_out_changed(|model| model.read()).release();
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
            GameRole::Cover => text(&cover_url(&row.cover)),
            GameRole::Facts => text(&row.facts),
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
            (GameRole::Cover, "cover"),
            (GameRole::Facts, "facts"),
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

    pub fn cover_at(&self, row: i32) -> QString {
        self.at(row).map_or_else(QString::default, |g| QString::from(cover_url(&g.cover).as_str()))
    }

    pub fn hero_at(&self, row: i32) -> QString {
        let art = |g: &GameRow| cover_url(if g.hero.is_empty() { &g.cover } else { &g.hero });
        self.at(row).map_or_else(QString::default, |g| QString::from(art(g).as_str()))
    }

    pub fn console_at(&self, row: i32) -> QString {
        self.at(row).map_or_else(QString::default, |g| QString::from(g.console.as_str()))
    }

    pub fn row_of(&self, path: &QString) -> i32 {
        let path = path.to_string();
        self.rows.iter().position(|g| g.path == path).map_or(-1, |i| i as i32)
    }

    pub fn facts_at(&self, row: i32) -> QString {
        self.at(row).map_or_else(QString::default, |g| QString::from(g.facts.as_str()))
    }

    pub fn hue_at(&self, row: i32) -> f64 {
        self.at(row).map_or(-1.0, |g| g.hue)
    }

    /// The games text read into rows off the GUI thread; the newest read's rows are taken, the model reset once.
    fn read(mut self: Pin<&mut Self>) {
        let (text, leave_out) = (self.games.to_string(), self.leave_out.to_string());
        let asked = self.asked.wrapping_add(1);
        self.as_mut().rust_mut().asked = asked;
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let rows = crate::games::without(crate::games::from_json(&text), &leave_out);
            let _ = qt.queue(move |model| model.take(asked, rows));
        });
    }

    fn take(mut self: Pin<&mut Self>, asked: u64, rows: Vec<GameRow>) {
        if self.asked != asked {
            return;
        }
        let count = rows.len() as i32;
        self.as_mut().begin_reset_model();
        self.as_mut().rust_mut().rows = rows;
        self.as_mut().end_reset_model();
        if self.count != count {
            self.as_mut().rust_mut().count = count;
            self.as_mut().count_changed();
        }
        let revision = self.revision.wrapping_add(1);
        self.as_mut().rust_mut().revision = revision;
        self.revision_changed();
    }
}
