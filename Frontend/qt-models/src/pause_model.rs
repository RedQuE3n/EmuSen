//! `PauseModel`, the Pause Card's page as a QAbstractListModel (EmuSen_Qt.md §7.1 step 7, §7.2): pause.rs's card put
//! on Qt, its rows, words, pane and help bar; a row chosen or stepped, a declared action, B or a shoulder answered by
//! pause.rs's rule, and what the session must do said as a signal, so QML writes no words and no logic.

use crate::help_model::shown_screen;
use crate::pause::{Card, Facts, Outcome, Page};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QVariant, QVector};

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
        include!("cxx-qt-lib/qvector.h");
        type QVector_i32 = cxx_qt_lib::QVector<i32>;
    }

    /// The card's pages.
    #[qenum(PauseModel)]
    enum PausePage {
        First,
        Leave,
        Reel,
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
        /// The row focused; a page opens on its own.
        #[qproperty(i32, current, READ, NOTIFY)]
        /// The context pane for the row focused, as plain data (pause.rs's `Pane`), read by `PaneModel`.
        #[qproperty(QString, pane, READ, NOTIFY)]
        /// The help bar the page shows, a `HelpModel` screen.
        #[qproperty(i32, help_screen, cxx_name = "helpScreen", READ, NOTIFY)]
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

        /// The row focused moved to `index`.
        #[qinvokable]
        #[cxx_name = "focusRow"]
        fn focus_row(self: Pin<&mut PauseModel>, index: i32);

        /// Left (-1) or Right (+1) on a row; `held` for a held repeat.
        #[qinvokable]
        fn step(self: Pin<&mut PauseModel>, index: i32, by: i32, held: bool);

        /// A shoulder (+1 further back, -1 nearer) or, with `end`, a trigger.
        #[qinvokable]
        fn stride(self: Pin<&mut PauseModel>, back: i32, end: bool);

        /// B on the page.
        #[qinvokable]
        fn back(self: Pin<&mut PauseModel>);

        /// The card closes and the game is to resume once it is gone.
        #[qsignal]
        fn resumed(self: Pin<&mut PauseModel>);

        /// Back to the library, the game kept suspended.
        #[qsignal]
        fn library(self: Pin<&mut PauseModel>);

        /// The slot Save and Load act on, chosen.
        #[qsignal]
        #[cxx_name = "slotChosen"]
        fn slot_chosen(self: Pin<&mut PauseModel>, slot: u32);

        /// The game's speed, chosen, in percent of its own.
        #[qsignal]
        #[cxx_name = "speedChosen"]
        fn speed_chosen(self: Pin<&mut PauseModel>, percent: u32);

        /// The slot to save; the card closes.
        #[qsignal]
        #[cxx_name = "saveAsked"]
        fn save_asked(self: Pin<&mut PauseModel>, slot: u32);

        /// The slot to load; the card closes.
        #[qsignal]
        #[cxx_name = "loadAsked"]
        fn load_asked(self: Pin<&mut PauseModel>, slot: u32);

        /// The moment to return the game to, by its frame; the card closes.
        #[qsignal]
        #[cxx_name = "rewindAsked"]
        fn rewind_asked(self: Pin<&mut PauseModel>, frame: u64);
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut PauseModel>);

        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut PauseModel>);

        #[inherit]
        fn index(self: &PauseModel, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;

        #[inherit]
        #[qsignal]
        #[cxx_name = "dataChanged"]
        fn data_changed(self: Pin<&mut PauseModel>, top_left: &QModelIndex, bottom_right: &QModelIndex, roles: &QVector_i32);
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
    pane: QString,
    help_screen: i32,
    card: Card,
    /// The page and the count of the rows the view holds, so a value's change is a data change and a page's a reset.
    rows_of: Option<(Page, usize)>,
}

impl Default for PauseModelRust {
    fn default() -> Self {
        let card = Card::default();
        PauseModelRust {
            page: PausePage::First,
            game: QString::default(),
            facts: QString::default(),
            title: QString::default(),
            breadcrumb: QString::default(),
            current: 0,
            pane: QString::from(card.pane().to_json().as_str()),
            help_screen: shown_screen(card.help()),
            card,
            rows_of: None,
        }
    }
}

impl cxx_qt::Initialize for qobject::PauseModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_game_changed(|model| model.shown()).release();
        self.on_facts_changed(|mut model| {
            let facts = Facts::from_json(&model.facts.to_string());
            model.as_mut().rust_mut().card.facts = facts;
            model.shown();
        })
        .release();
    }
}

const LABEL: i32 = 0;
const VALUE: i32 = 1;
const STEPS: i32 = 2;
const OPENS: i32 = 3;
const SECTION_END: i32 = 4;
const KEY: i32 = 5;
const DIMMED: i32 = 6;

fn shown_page(page: Page) -> PausePage {
    match page {
        Page::First => PausePage::First,
        Page::Leave => PausePage::Leave,
        Page::Reel => PausePage::Reel,
    }
}

impl qobject::PauseModel {
    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let rows = self.card.rows();
        let Some(row) = rows.get(index.row() as usize) else { return QVariant::default() };
        match role {
            LABEL => QVariant::from(&QString::from(row.label.as_str())),
            VALUE => QVariant::from(&QString::from(row.value.as_str())),
            STEPS => QVariant::from(&row.steps),
            OPENS => QVariant::from(&row.opens),
            SECTION_END => QVariant::from(&row.section_end),
            KEY => QVariant::from(&QString::from(row.key)),
            DIMMED => QVariant::from(&row.dimmed),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
        let names = [(LABEL, "label"), (VALUE, "value"), (STEPS, "steps"), (OPENS, "opens"), (SECTION_END, "sectionEnd"), (KEY, "key"), (DIMMED, "dimmed")];
        for (role, name) in names {
            roles.insert(role, QByteArray::from(name));
        }
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.card.rows().len() as i32
    }

    pub fn open(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().card.open();
        self.turned();
    }

    pub fn choose(mut self: Pin<&mut Self>, index: i32) {
        let outcome = self.as_mut().rust_mut().card.choose(index.max(0) as usize);
        self.answer(outcome);
    }

    pub fn act(mut self: Pin<&mut Self>, key: &QString) {
        let outcome = self.as_mut().rust_mut().card.act(&key.to_string());
        self.answer(outcome);
    }

    pub fn focus_row(mut self: Pin<&mut Self>, index: i32) {
        if self.as_mut().rust_mut().card.focus(index.max(0) as usize) == Outcome::Shown {
            self.as_mut().rust_mut().current = index;
            self.as_mut().current_changed();
            self.described();
        }
    }

    pub fn step(mut self: Pin<&mut Self>, index: i32, by: i32, held: bool) {
        let outcome = self.as_mut().rust_mut().card.step(index.max(0) as usize, by, held);
        self.answer(outcome);
    }

    pub fn stride(mut self: Pin<&mut Self>, back: i32, end: bool) {
        let outcome = self.as_mut().rust_mut().card.stride(back, end);
        self.answer(outcome);
    }

    pub fn back(mut self: Pin<&mut Self>) {
        let outcome = self.as_mut().rust_mut().card.back();
        self.answer(outcome);
    }

    fn answer(mut self: Pin<&mut Self>, outcome: Outcome) {
        match outcome {
            Outcome::Nothing => {}
            Outcome::Shown => {
                if shown_page(self.card.page) != self.page || self.card.current as i32 != self.current {
                    self.turned();
                } else {
                    self.shown();
                }
            }
            Outcome::Resume => self.as_mut().resumed(),
            Outcome::Library => self.as_mut().library(),
            Outcome::Slot(slot) => self.as_mut().slot_chosen(slot),
            Outcome::Speed(percent) => self.as_mut().speed_chosen(percent),
            Outcome::Save(slot) => {
                self.as_mut().save_asked(slot);
                self.as_mut().resumed();
            }
            Outcome::Load(slot) => {
                self.as_mut().load_asked(slot);
                self.as_mut().resumed();
            }
            Outcome::RewindTo(frame) => {
                self.as_mut().rewind_asked(frame);
                self.as_mut().resumed();
            }
        }
    }

    /// The card on a page and row it turned to: a page turn changes the rows, the title and the way back, nothing else.
    fn turned(mut self: Pin<&mut Self>) {
        let page = shown_page(self.card.page);
        if self.page != page {
            self.as_mut().rust_mut().page = page;
            self.as_mut().page_changed();
        }
        self.as_mut().shown();
        let at = self.card.current as i32;
        self.as_mut().rust_mut().current = at;
        // Said on every turn, so a view moves to it even where the number is the same as the page before's.
        self.as_mut().current_changed();
    }

    /// The page's rows and words read again from the card.
    fn shown(mut self: Pin<&mut Self>) {
        let game = self.game.to_string();
        let now = (self.card.page, self.card.rows().len());
        if self.rows_of != Some(now) {
            self.as_mut().begin_reset_model();
            self.as_mut().rust_mut().rows_of = Some(now);
            self.as_mut().end_reset_model();
        } else if now.1 > 0 {
            let (first, last) = (self.index(0, 0, &QModelIndex::default()), self.index(now.1 as i32 - 1, 0, &QModelIndex::default()));
            self.as_mut().data_changed(&first, &last, &QVector::<i32>::default());
        }
        let title = QString::from(self.card.title(&game).as_str());
        let breadcrumb = QString::from(self.card.breadcrumb(&game).as_str());
        if self.title != title {
            self.as_mut().rust_mut().title = title;
            self.as_mut().title_changed();
        }
        if self.breadcrumb != breadcrumb {
            self.as_mut().rust_mut().breadcrumb = breadcrumb;
            self.as_mut().breadcrumb_changed();
        }
        self.described();
    }

    /// The pane and the help bar for the row focused.
    fn described(mut self: Pin<&mut Self>) {
        let pane = QString::from(self.card.pane().to_json().as_str());
        if self.pane != pane {
            self.as_mut().rust_mut().pane = pane;
            self.as_mut().pane_changed();
        }
        let help = shown_screen(self.card.help());
        if self.help_screen != help {
            self.as_mut().rust_mut().help_screen = help;
            self.as_mut().help_screen_changed();
        }
    }
}
