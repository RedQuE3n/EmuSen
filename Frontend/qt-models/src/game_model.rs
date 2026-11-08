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
        /// The grid's section by its row's key and its name; empty for a list shown as given (Home's shelf, the fan).
        #[qproperty(QString, section)]
        #[qproperty(QString, section_name, cxx_name = "sectionName")]
        /// The search's words: the section's games whose title holds them (EmuSen_Interface.md §18.1).
        #[qproperty(QString, search)]
        /// The focused row, whose letter the List card's Jump to names.
        #[qproperty(i32, focused)]
        /// The count beside the section's name: "8 games", or "1 of 8 games" where narrowed.
        #[qproperty(QString, count_line, cxx_name = "countLine", READ, NOTIFY)]
        /// The List card's facts as plain data (list_card.rs).
        #[qproperty(QString, list_facts, cxx_name = "listFacts", READ, NOTIFY)]
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
        /// A List card's action carried out on the listing (list_card.rs): a sort or a letter stepped, a random game, a
        /// filter chosen. Gives the row to focus: the focused row where the action moves none.
        #[qinvokable]
        #[cxx_name = "listAct"]
        fn list_act(self: Pin<&mut GameModel>, action: &QString) -> i32;
        /// B on the grid: the search cleared, else the section's favourites; with neither, `passedBack`.
        #[qinvokable]
        fn back(self: Pin<&mut GameModel>);
        /// B found nothing of the listing's to clear: the grid goes back a screen.
        #[qsignal]
        #[cxx_name = "passedBack"]
        fn passed_back(self: Pin<&mut GameModel>);
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
    section: QString,
    section_name: QString,
    search: QString,
    focused: i32,
    count_line: QString,
    list_facts: QString,
    /// The sort, global as Mistress's is, and the sections narrowed to their favourites, each keeping its own.
    sort: usize,
    favourites_in: std::collections::HashSet<String>,
    /// What the last read found: the section's count and each letter's first row in the shown list.
    read: Read,
}

/// What a read finds beside the rows, off the GUI thread.
#[derive(Clone, Debug, Default)]
pub struct Read {
    pub all: usize,
    pub letters: Vec<(String, usize)>,
}

impl cxx_qt::Initialize for qobject::GameModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_games_changed(|model| model.read()).release();
        self.as_mut().on_leave_out_changed(|model| model.read()).release();
        self.as_mut().on_section_changed(|model| model.read()).release();
        self.as_mut().on_section_name_changed(|model| model.said()).release();
        self.as_mut().on_search_changed(|model| model.read()).release();
        self.on_focused_changed(|model| model.said()).release();
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
        let (section, listing) = (self.section.to_string(), self.listing());
        let asked = self.asked.wrapping_add(1);
        self.as_mut().rust_mut().asked = asked;
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let rows = crate::games::without(crate::games::from_json(&text), &leave_out);
            let (rows, read) = if section.is_empty() {
                let all = rows.len();
                (rows, Read { all, letters: Vec::new() })
            } else {
                let shown = crate::listing::shown(&rows, &section, &listing);
                let letters = crate::listing::letter_rows(&shown);
                (shown, Read { all: rows.len(), letters })
            };
            let _ = qt.queue(move |model| model.take(asked, rows, read));
        });
    }

    /// The listing the section is read with now.
    fn listing(&self) -> crate::listing::Listing {
        let section = self.section.to_string();
        crate::listing::Listing { sort: self.sort, favourites_only: self.favourites_in.contains(&section), search: self.search.to_string() }
    }

    pub fn list_act(mut self: Pin<&mut Self>, action: &QString) -> i32 {
        use crate::list_card::{action_of, EVERY_GAME, FAVOURITES_ONLY, JUMP, RANDOM, SORT};
        let action = action.to_string();
        let section = self.section.to_string();
        match action_of(&action) {
            (SORT, by) => {
                let sort = crate::listing::step_sort(self.sort, by);
                self.as_mut().rust_mut().sort = sort;
                self.read();
                0
            }
            (JUMP, by) => {
                let here = self.at(self.focused).map(|g| crate::listing::letter(&g.title)).unwrap_or_default();
                crate::listing::jump_letter(&self.read.letters, &here, by).map_or(self.focused, |row| row as i32)
            }
            (RANDOM, _) if !self.rows.is_empty() => {
                let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.subsec_nanos() as usize);
                (nanos % self.rows.len()) as i32
            }
            (FAVOURITES_ONLY, _) => {
                self.as_mut().rust_mut().favourites_in.insert(section);
                self.read();
                0
            }
            (EVERY_GAME, _) => {
                self.as_mut().rust_mut().favourites_in.remove(&section);
                self.read();
                0
            }
            _ => self.focused,
        }
    }

    pub fn back(mut self: Pin<&mut Self>) {
        let section = self.section.to_string();
        if !self.search.is_empty() {
            self.as_mut().set_search(QString::default());
        } else if self.as_mut().rust_mut().favourites_in.remove(&section) {
            self.read();
        } else {
            self.passed_back();
        }
    }

    /// The count and the List card's facts said again from the last read and the focus.
    fn said(mut self: Pin<&mut Self>) {
        let listing = self.listing();
        let count = crate::listing::count_line(self.read.all, self.rows.len(), listing.narrowed());
        let letter = self.at(self.focused).or(self.rows.first()).map_or_else(|| "#".to_owned(), |g| crate::listing::letter(&g.title));
        let facts = serde_json::json!({
            "section": self.section_name.to_string(),
            "count": count,
            "sort": crate::listing::SORTS[self.sort],
            "filter": if listing.favourites_only { "Favourites" } else { "" },
            "letter": letter,
            "letters": self.read.letters.iter().map(|(l, _)| l.as_str()).collect::<Vec<_>>().join(" "),
            "favouritesSection": self.section.to_string() == crate::library::FAVOURITES,
        })
        .to_string();
        let (count, facts) = (QString::from(count.as_str()), QString::from(facts.as_str()));
        if self.count_line != count {
            self.as_mut().rust_mut().count_line = count;
            self.as_mut().count_line_changed();
        }
        if self.list_facts != facts {
            self.as_mut().rust_mut().list_facts = facts;
            self.list_facts_changed();
        }
    }

    fn take(mut self: Pin<&mut Self>, asked: u64, rows: Vec<GameRow>, read: Read) {
        if self.asked != asked {
            return;
        }
        self.as_mut().rust_mut().read = read;
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
        self.as_mut().revision_changed();
        self.said();
    }
}
