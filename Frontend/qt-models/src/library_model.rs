//! `LibraryModel`, the library's navigation as a QAbstractListModel (EmuSen_Qt.md §1.2.1): library.rs's rows put on
//! Qt. A refresh runs off the GUI thread and comes back through qt_thread().queue, the one way in.

use crate::clients::ClientSource;
use crate::library::{refresh, LibrarySource, LibraryView, Refreshed};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QVariant};
use std::sync::Arc;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++Qt" {
        include!(<QtCore/QAbstractListModel>);
        #[qobject]
        type QAbstractListModel;
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
    #[qenum(LibraryModel)]
    enum Role {
        Kind,
        Key,
        Name,
        Count,
        Hue,
        Body,
        Maker,
        Year,
        Engine,
    }

    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        #[qproperty(i32, selected_index, cxx_name = "selectedIndex", READ, NOTIFY)]
        #[qproperty(QString, selected_key, cxx_name = "selectedKey", READ, NOTIFY)]
        #[qproperty(QString, problem, READ, NOTIFY)]
        #[qproperty(bool, loading, READ, NOTIFY)]
        #[qproperty(QString, platform_library, cxx_name = "platformLibrary")]
        #[qproperty(QString, games_database, cxx_name = "gamesDatabase")]
        type LibraryModel = super::LibraryModelRust;
    }

    extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn data(self: &LibraryModel, index: &QModelIndex, role: i32) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &LibraryModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &LibraryModel, parent: &QModelIndex) -> i32;

        /// Reads the source again off the GUI thread; the selection is kept by its key.
        #[qinvokable]
        fn refresh(self: Pin<&mut LibraryModel>);

        /// Selects a row; false where there is none.
        #[qinvokable]
        fn select(self: Pin<&mut LibraryModel>, index: i32) -> bool;
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut LibraryModel>);

        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut LibraryModel>);
    }

    impl cxx_qt::Threading for LibraryModel {}
    impl cxx_qt::Initialize for LibraryModel {}
}

use qobject::Role;

/// The model's state: the view, its source, and the properties QML reads.
pub struct LibraryModelRust {
    selected_index: i32,
    selected_key: QString,
    problem: QString,
    loading: bool,
    platform_library: QString,
    games_database: QString,
    view: LibraryView,
    source: Option<Arc<dyn LibrarySource>>,
}

impl Default for LibraryModelRust {
    fn default() -> Self {
        LibraryModelRust {
            selected_index: -1,
            selected_key: QString::default(),
            problem: QString::default(),
            loading: false,
            platform_library: QString::default(),
            games_database: QString::default(),
            view: LibraryView::empty(),
            source: None,
        }
    }
}

impl cxx_qt::Initialize for qobject::LibraryModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().publish();
        self.as_mut().on_platform_library_changed(|model| model.connect()).release();
        self.as_mut().on_games_database_changed(|model| model.connect()).release();
    }
}

impl qobject::LibraryModel {
    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some(row) = self.view.rows.rows().get(index.row() as usize) else {
            return QVariant::default();
        };
        let text = |s: &str| QVariant::from(&QString::from(s));
        match (Role { repr: role }) {
            Role::Kind => text(row.kind.map_or("", |k| k.name())),
            Role::Key => text(&row.key),
            Role::Name => text(&row.name),
            Role::Count => QVariant::from(&(row.count as i32)),
            Role::Hue => QVariant::from(&row.hue),
            Role::Body => text(&row.body),
            Role::Maker => text(&row.maker),
            Role::Year => QVariant::from(&row.year),
            Role::Engine => text(&row.engine),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
        let names = [
            (Role::Kind, "kind"),
            (Role::Key, "key"),
            (Role::Name, "name"),
            (Role::Count, "count"),
            (Role::Hue, "hue"),
            (Role::Body, "body"),
            (Role::Maker, "maker"),
            (Role::Year, "year"),
            (Role::Engine, "engine"),
        ];
        for (role, name) in names {
            roles.insert(role.repr, QByteArray::from(name));
        }
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.view.rows.rows().len() as i32
    }

    pub fn refresh(mut self: Pin<&mut Self>) {
        let Some(source) = self.source.clone() else { return };
        self.as_mut().show_loading(true);
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let refreshed = refresh(source.as_ref());
            let _ = qt.queue(move |model| model.apply(None, refreshed));
        });
    }

    pub fn select(mut self: Pin<&mut Self>, index: i32) -> bool {
        let taken = index >= 0 && self.as_mut().rust_mut().view.rows.select(index as usize);
        self.publish();
        taken
    }

    /// Opens the clients once both the platform library and games.db are named, then refreshes, off the GUI thread.
    fn connect(mut self: Pin<&mut Self>) {
        let (library, games) = (self.platform_library.to_string(), self.games_database.to_string());
        if library.is_empty() || games.is_empty() {
            return;
        }
        self.as_mut().show_loading(true);
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let (source, refreshed): (Option<Arc<dyn LibrarySource>>, Refreshed) =
                match ClientSource::open(&library, &emusen_crystal_client::Config::default(), &games) {
                    Ok(source) => {
                        let source: Arc<dyn LibrarySource> = Arc::new(source);
                        let refreshed = refresh(source.as_ref());
                        (Some(source), refreshed)
                    }
                    Err(words) => (None, Err(words)),
                };
            let _ = qt.queue(move |model| model.apply(source, refreshed));
        });
    }

    /// A refresh's answer applied on the GUI thread: the rows reset with the selection kept, the properties published.
    fn apply(mut self: Pin<&mut Self>, source: Option<Arc<dyn LibrarySource>>, refreshed: Refreshed) {
        if source.is_some() {
            self.as_mut().rust_mut().source = source;
        }
        let rows_change = refreshed.is_ok();
        if rows_change {
            self.as_mut().begin_reset_model();
        }
        self.as_mut().rust_mut().view.apply(refreshed);
        if rows_change {
            self.as_mut().end_reset_model();
        }
        self.as_mut().show_loading(false);
        self.publish();
    }

    /// Whether a refresh is running, as a read-only property.
    fn show_loading(mut self: Pin<&mut Self>, loading: bool) {
        if self.loading != loading {
            self.as_mut().rust_mut().loading = loading;
            self.loading_changed();
        }
    }

    /// The selection and the problem as read-only properties, each announced when it changed.
    fn publish(mut self: Pin<&mut Self>) {
        let index = self.view.rows.selected().map_or(-1, |i| i as i32);
        let key = QString::from(self.view.rows.selected_key().unwrap_or(""));
        let problem = QString::from(self.view.problem.as_str());
        if self.selected_index != index {
            self.as_mut().rust_mut().selected_index = index;
            self.as_mut().selected_index_changed();
        }
        if self.selected_key != key {
            self.as_mut().rust_mut().selected_key = key;
            self.as_mut().selected_key_changed();
        }
        if self.problem != problem {
            self.as_mut().rust_mut().problem = problem;
            self.problem_changed();
        }
    }
}
