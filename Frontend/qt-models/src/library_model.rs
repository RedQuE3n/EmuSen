//! `LibraryModel`, the library's navigation as a QAbstractListModel (EmuSen_Qt.md §1.2.1): library.rs's rows put on
//! Qt. A refresh runs off the GUI thread and comes back through qt_thread().queue, the one way in.

use crate::clients::ClientSource;
use crate::library::{refresh, splash_worlds, LibrarySource, LibraryView, Refreshed};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QVariant};
use std::sync::Arc;

#[cxx_qt::bridge]
pub mod qobject {
    // Declared once in the crate, here: cxx-qt emits an upcast shim per declaration (EmuSen_Kit.md §7.6).
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
        World,
        FullName,
        Status,
        Facts,
        EngineName,
        Libretro,
        CountWords,
        EngineMark,
        Heading,
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
        #[qproperty(QString, splash_worlds, cxx_name = "splashWorlds", READ, NOTIFY)]
        #[qproperty(i32, splash_planets, cxx_name = "splashPlanets", READ, NOTIFY)]
        #[qproperty(QString, splash_caption, cxx_name = "splashCaption", READ, NOTIFY)]
        #[qproperty(QString, summary, READ, NOTIFY)]
        #[qproperty(i32, places, READ, NOTIFY)]
        #[qproperty(QString, recent, READ, NOTIFY)]
        #[qproperty(QString, cores)]
        #[qproperty(QString, tool_summaries, cxx_name = "toolSummaries")]
        #[qproperty(bool, dark)]
        #[qproperty(QString, home_lines, cxx_name = "homeLines", READ, NOTIFY)]
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

        /// A game's title from its path, for a game the library does not hold (one opened by its file).
        #[qinvokable]
        #[cxx_name = "titleOf"]
        fn title_of(self: &LibraryModel, path: &QString) -> QString;

        /// Records the game at `path` as started now, then reads the games again, so Recent and Continue lead with it.
        #[qinvokable]
        fn started(self: Pin<&mut LibraryModel>, path: &QString);

        /// The games a row leads to, as GameModel reads them: a JSON array, one object a game.
        #[qinvokable]
        #[cxx_name = "gamesOf"]
        fn games_of(self: &LibraryModel, key: &QString) -> QString;

        /// The row of a key, or -1.
        #[qinvokable]
        #[cxx_name = "indexOf"]
        fn index_of(self: &LibraryModel, key: &QString) -> i32;

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
    splash_worlds: QString,
    splash_planets: i32,
    splash_caption: QString,
    summary: QString,
    /// How many rows are places; they come first.
    places: i32,
    /// The recently played games as GameModel reads them, the most recent first.
    recent: QString,
    /// The cores directory the runtime scans; empty for the installation's own.
    cores: QString,
    /// The tools' summaries as Home shows them, `{"scraper": {"brief", "attention"}, ...}`; empty until a tool says.
    tool_summaries: QString,
    /// Whether the dark variant is shown, for the Settings place's line.
    dark: bool,
    /// Home's places' second lines as a JSON array of `{"status", "attention"}`.
    home_lines: QString,
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
            splash_worlds: QString::default(),
            splash_planets: 0,
            splash_caption: QString::default(),
            summary: QString::default(),
            places: 0,
            recent: QString::default(),
            cores: QString::default(),
            tool_summaries: QString::default(),
            dark: true,
            home_lines: QString::from("[]"),
            view: LibraryView::empty(),
            source: None,
        }
    }
}

impl cxx_qt::Initialize for qobject::LibraryModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().publish();
        self.as_mut().on_platform_library_changed(|model| model.connect()).release();
        self.as_mut().on_tool_summaries_changed(|model| model.publish()).release();
        self.as_mut().on_dark_changed(|model| model.publish()).release();
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
            Role::World => text(&row.world),
            Role::FullName => text(&row.full_name),
            Role::Status => text(&row.status),
            Role::Facts => text(&row.facts),
            Role::EngineName => text(&row.engine_name),
            Role::Libretro => QVariant::from(&row.libretro),
            Role::CountWords => text(&crate::words::games(row.count)),
            Role::EngineMark => text(&row.engine_mark),
            Role::Heading => text(&row.heading),
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
            (Role::World, "world"),
            (Role::FullName, "fullName"),
            (Role::Status, "status"),
            (Role::Facts, "facts"),
            (Role::EngineName, "engineName"),
            (Role::Libretro, "libretro"),
            (Role::CountWords, "countWords"),
            (Role::EngineMark, "engineMark"),
            (Role::Heading, "heading"),
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

    pub fn title_of(&self, path: &QString) -> QString {
        QString::from(crate::library::title_of(&path.to_string()).as_str())
    }

    pub fn started(mut self: Pin<&mut Self>, path: &QString) {
        let Some(source) = self.source.clone() else { return };
        self.as_mut().show_loading(true);
        let (qt, path) = (self.qt_thread(), path.to_string());
        std::thread::spawn(move || {
            let refreshed = source.started(&path).and_then(|()| refresh(source.as_ref()));
            let _ = qt.queue(move |model| model.apply(None, refreshed));
        });
    }

    pub fn games_of(&self, key: &QString) -> QString {
        let games = crate::library::games_of(&key.to_string(), &self.view.systems, &self.view.catalog);
        QString::from(crate::games::to_json(&games).as_str())
    }

    pub fn index_of(&self, key: &QString) -> i32 {
        let key = key.to_string();
        self.view.rows.rows().iter().position(|r| r.key == key).map_or(-1, |i| i as i32)
    }

    pub fn select(mut self: Pin<&mut Self>, index: i32) -> bool {
        let taken = index >= 0 && self.as_mut().rust_mut().view.rows.select(index as usize);
        self.publish();
        taken
    }

    /// Opens the clients once the platform library is named, then refreshes, off the GUI thread.
    fn connect(mut self: Pin<&mut Self>) {
        let library = self.platform_library.to_string();
        if library.is_empty() {
            return;
        }
        self.as_mut().show_loading(true);
        let cores = self.cores.to_string();
        let config = emusen_crystal_client::Config { cores: (!cores.is_empty()).then(|| vec![cores]), ..Default::default() };
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let (source, refreshed): (Option<Arc<dyn LibrarySource>>, Refreshed) =
                match ClientSource::open(&library, &config) {
                    Ok(source) => {
                        let source: Arc<dyn LibrarySource> = Arc::new(source);
                        // The systems come before the scan, so Moonrise's planets do not wait for the games.
                        if let Ok(systems) = source.systems() {
                            let worlds = splash_worlds(&systems);
                            let _ = qt.queue(move |model| model.show_splash(worlds));
                        }
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

    /// Moonrise's worlds, one line each, and its caption, from the systems.
    fn show_splash(mut self: Pin<&mut Self>, worlds: Vec<String>) {
        let caption = QString::from(crate::words::splash_caption(worlds.len()));
        let planets = worlds.len() as i32;
        self.as_mut().rust_mut().splash_worlds = QString::from(worlds.join("\n"));
        self.as_mut().splash_worlds_changed();
        self.as_mut().rust_mut().splash_planets = planets;
        self.as_mut().splash_planets_changed();
        self.as_mut().rust_mut().splash_caption = caption;
        self.splash_caption_changed();
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
        let summary = QString::from(self.view.summary().as_str());
        let places = self.view.rows.rows().iter().filter(|r| r.kind == Some(crate::library::Kind::Place)).count() as i32;
        if self.places != places {
            self.as_mut().rust_mut().places = places;
            self.as_mut().places_changed();
        }
        let recent = self.games_of(&QString::from(crate::library::RECENT));
        if self.recent != recent {
            self.as_mut().rust_mut().recent = recent;
            self.as_mut().recent_changed();
        }
        let tools = serde_json::from_str(&self.tool_summaries.to_string()).unwrap_or(serde_json::Value::Null);
        let settings = crate::words::settings_line(self.dark, &self.view.catalog.settings);
        let lines: Vec<serde_json::Value> = crate::words::home_lines(self.view.catalog.games.len(), &tools, &settings)
            .into_iter()
            .map(|(status, attention)| serde_json::json!({ "status": status, "attention": attention }))
            .collect();
        let home_lines = QString::from(serde_json::Value::Array(lines).to_string().as_str());
        if self.home_lines != home_lines {
            self.as_mut().rust_mut().home_lines = home_lines;
            self.as_mut().home_lines_changed();
        }
        if self.summary != summary {
            self.as_mut().rust_mut().summary = summary;
            self.as_mut().summary_changed();
        }
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
