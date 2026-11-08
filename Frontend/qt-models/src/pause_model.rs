//! `PauseModel`, the Pause Card's page as a QAbstractListModel (EmuSen_Qt.md §7.1 step 7): pause.rs's rows and words
//! put on Qt, and a row chosen, a declared action or B answered by pause.rs's rule, so QML writes no words and no logic.

use crate::pause::{self, Choice, Facts, Page, Row};
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

    /// The card's pages.
    #[qenum(PauseModel)]
    enum PausePage {
        First,
        Leave,
    }

    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        #[qproperty(PausePage, page, READ, NOTIFY)]
        /// The game's name, the first page's title.
        #[qproperty(QString, game)]
        /// The session's facts as plain data (`SessionModel.cardFacts`).
        #[qproperty(QString, facts)]
        #[qproperty(QString, title, READ, NOTIFY)]
        #[qproperty(QString, breadcrumb, READ, NOTIFY)]
        /// The row a page opens on.
        #[qproperty(i32, current, READ, NOTIFY)]
        type PauseModel = super::PauseModelRust;
    }

    extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn data(self: &PauseModel, index: &QModelIndex, role: i32) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &PauseModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &PauseModel, parent: &QModelIndex) -> i32;

        /// The card opened: its first page, on Resume.
        #[qinvokable]
        fn open(self: Pin<&mut PauseModel>);

        /// A row chosen, by its index on the page.
        #[qinvokable]
        fn choose(self: Pin<&mut PauseModel>, index: i32);

        /// A declared action, by its key (help.rs).
        #[qinvokable]
        fn act(self: Pin<&mut PauseModel>, key: &QString);

        /// B on the page.
        #[qinvokable]
        fn back(self: Pin<&mut PauseModel>);

        /// The card closes and the game is to resume once it is gone.
        #[qsignal]
        fn resumed(self: Pin<&mut PauseModel>);

        /// Back to the library, the game kept suspended.
        #[qsignal]
        fn library(self: Pin<&mut PauseModel>);
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut PauseModel>);

        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut PauseModel>);
    }

    impl cxx_qt::Initialize for PauseModel {}
}

use qobject::PausePage;

pub struct PauseModelRust {
    page: PausePage,
    game: QString,
    facts: QString,
    title: QString,
    breadcrumb: QString,
    current: i32,
    rows: Vec<Row>,
}

impl Default for PauseModelRust {
    fn default() -> Self {
        PauseModelRust {
            page: PausePage::First,
            game: QString::default(),
            facts: QString::default(),
            title: QString::default(),
            breadcrumb: QString::default(),
            current: 0,
            rows: pause::rows(Page::First, &Facts::default()),
        }
    }
}

impl cxx_qt::Initialize for qobject::PauseModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_game_changed(|model| model.shown()).release();
        self.on_facts_changed(|model| model.shown()).release();
    }
}

const LABEL: i32 = 0;
const VALUE: i32 = 1;
const STEPS: i32 = 2;
const OPENS: i32 = 3;
const SECTION_END: i32 = 4;
const KEY: i32 = 5;

fn page_of(page: PausePage) -> Page {
    match page {
        PausePage::Leave => Page::Leave,
        _ => Page::First,
    }
}

fn shown_page(page: Page) -> PausePage {
    match page {
        Page::First => PausePage::First,
        Page::Leave => PausePage::Leave,
    }
}

impl qobject::PauseModel {
    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some(row) = self.rows.get(index.row() as usize) else { return QVariant::default() };
        match role {
            LABEL => QVariant::from(&QString::from(row.label.as_str())),
            VALUE => QVariant::from(&QString::from(row.value.as_str())),
            STEPS => QVariant::from(&row.steps),
            OPENS => QVariant::from(&row.opens),
            SECTION_END => QVariant::from(&row.section_end),
            KEY => QVariant::from(&QString::from(row.key)),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
        for (role, name) in [(LABEL, "label"), (VALUE, "value"), (STEPS, "steps"), (OPENS, "opens"), (SECTION_END, "sectionEnd"), (KEY, "key")] {
            roles.insert(role, QByteArray::from(name));
        }
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.rows.len() as i32
    }

    pub fn open(self: Pin<&mut Self>) {
        self.turn(Page::First, 0);
    }

    pub fn choose(self: Pin<&mut Self>, index: i32) {
        let Some(key) = self.rows.get(index.max(0) as usize).map(|r| r.key) else { return };
        self.answer(pause::choose(key));
    }

    pub fn act(self: Pin<&mut Self>, key: &QString) {
        self.answer(pause::choose(&key.to_string()));
    }

    pub fn back(mut self: Pin<&mut Self>) {
        match pause::back(page_of(self.page)) {
            Some((page, from)) => {
                let at = pause::rows(page, &Facts::default()).iter().position(|r| r.key == from).unwrap_or(0);
                self.turn(page, at as i32);
            }
            None => self.as_mut().resumed(),
        }
    }

    fn answer(mut self: Pin<&mut Self>, choice: Choice) {
        match choice {
            Choice::Resume => self.as_mut().resumed(),
            Choice::Turn(page) => self.turn(page, 0),
            Choice::Library => self.as_mut().library(),
            Choice::Owed => {}
        }
    }

    /// The card on `page` at row `at`: a page turn changes the rows, the title and the way back, nothing else.
    fn turn(mut self: Pin<&mut Self>, page: Page, at: i32) {
        let page = shown_page(page);
        if self.page != page {
            self.as_mut().rust_mut().page = page;
            self.as_mut().page_changed();
        }
        self.as_mut().shown();
        if self.current != at {
            self.as_mut().rust_mut().current = at;
        }
        // Said on every turn, so a view moves to it even where the number is the same as the page before's.
        self.as_mut().current_changed();
    }

    /// The page's rows and words read again from the facts and the game's name.
    fn shown(mut self: Pin<&mut Self>) {
        let page = page_of(self.page);
        let facts = Facts::from_json(&self.facts.to_string());
        let game = self.game.to_string();
        self.as_mut().begin_reset_model();
        self.as_mut().rust_mut().rows = pause::rows(page, &facts);
        self.as_mut().end_reset_model();
        let (title, breadcrumb) = (QString::from(pause::title(page, &game).as_str()), QString::from(pause::breadcrumb(page, &game).as_str()));
        if self.title != title {
            self.as_mut().rust_mut().title = title;
            self.as_mut().title_changed();
        }
        if self.breadcrumb != breadcrumb {
            self.as_mut().rust_mut().breadcrumb = breadcrumb;
            self.as_mut().breadcrumb_changed();
        }
    }
}
