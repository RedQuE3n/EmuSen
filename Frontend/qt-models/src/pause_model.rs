//! `PauseModel`, the Pause Card's page as a QAbstractListModel (EmuSen_Qt.md §7.1 step 7, §7.3): pause.rs's card put
//! on Qt, its rows, words, pane and help bar; a row chosen or stepped, a declared action, B or a shoulder answered by
//! pause.rs's rule, and what the session must do said as a signal, so QML writes no words and no logic.

use crate::help_model::shown_screen;
use crate::row_roles;
use crate::pause::{Card, Facts, Outcome, Page};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QVariant, QVector};

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
        Ask,
        Options,
        Disc,
        Cards,
        Drawer,
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
        /// The game's system as the library keys it, from the facts, for `shelf`.
        #[qproperty(QString, system, READ, NOTIFY)]
        /// The console's games in the library (`LibraryModel.gamesOf(system)`), for Change Disc.
        #[qproperty(QString, shelf)]
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

        /// A key the focused row did not take: the shoulders and triggers, as their Qt keys, stride the reel.
        #[qinvokable]
        fn key(self: Pin<&mut PauseModel>, key: i32);

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

        /// Restart Game answered Yes: the game starts again; the card closes.
        #[qsignal]
        #[cxx_name = "restartAsked"]
        fn restart_asked(self: Pin<&mut PauseModel>);

        /// Quit Game answered Yes: the game closes and the library comes back.
        #[qsignal]
        #[cxx_name = "quitGameAsked"]
        fn quit_game_asked(self: Pin<&mut PauseModel>);

        /// X: the picture on show saved as a screenshot; the card stays.
        #[qsignal]
        #[cxx_name = "screenshotAsked"]
        fn screenshot_asked(self: Pin<&mut PauseModel>);

        /// Quit EmuSen answered Yes: the game closes, then the program.
        #[qsignal]
        #[cxx_name = "quitAsked"]
        fn quit_asked(self: Pin<&mut PauseModel>);

        /// The game's own disc of that index to put in; the card closes.
        #[qsignal]
        #[cxx_name = "discAsked"]
        fn disc_asked(self: Pin<&mut PauseModel>, index: u32);

        /// Another game's disc to put in, by its file; the card closes.
        #[qsignal]
        #[cxx_name = "insertAsked"]
        fn insert_asked(self: Pin<&mut PauseModel>, path: QString);

        /// The tray to open with no disc in; the card closes.
        #[qsignal]
        #[cxx_name = "ejectAsked"]
        fn eject_asked(self: Pin<&mut PauseModel>);

        /// Players 1 and 2 to swap; the card closes.
        #[qsignal]
        #[cxx_name = "swapAsked"]
        fn swap_asked(self: Pin<&mut PauseModel>);

        /// A console's setting to store: a card's mode or slot on Memory Cards; the card stays.
        #[qsignal]
        #[cxx_name = "settingAsked"]
        fn setting_asked(self: Pin<&mut PauseModel>, key: QString, value: QString);
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
    system: QString,
    shelf: QString,
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
            system: QString::default(),
            shelf: QString::default(),
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
        self.as_mut().on_facts_changed(|mut model| {
            let facts = Facts::from_json(&model.facts.to_string());
            let system = QString::from(&facts.system);
            model.as_mut().rust_mut().card.facts = facts;
            if model.system != system {
                model.as_mut().rust_mut().system = system;
                model.as_mut().system_changed();
            }
            model.shown();
        })
        .release();
        self.on_shelf_changed(|mut model| {
            let mut shelf = crate::games::from_json(&model.shelf.to_string());
            shelf.iter_mut().for_each(|g| g.cover = crate::game_model::cover_url(&g.cover));
            model.as_mut().rust_mut().card.shelf = shelf;
            model.shown();
        })
        .release();
    }
}

fn shown_page(page: Page) -> PausePage {
    match page {
        Page::First => PausePage::First,
        Page::Leave => PausePage::Leave,
        Page::Reel => PausePage::Reel,
        Page::Ask(_) => PausePage::Ask,
        Page::Options => PausePage::Options,
        Page::Disc => PausePage::Disc,
        Page::Cards => PausePage::Cards,
        Page::Drawer(_) => PausePage::Drawer,
    }
}

impl qobject::PauseModel {
    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let rows = self.card.rows();
        let shown = rows.get(index.row() as usize).map(|r| row_roles::Shown {
            key: r.key,
            label: &r.label,
            value: &r.value,
            steps: r.steps,
            opens: r.opens,
            section_end: r.section_end,
            dimmed: r.dimmed,
        });
        row_roles::data(shown, role)
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        row_roles::role_names()
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

    pub fn key(mut self: Pin<&mut Self>, key: i32) {
        let Some(button) = emusen_kit_qt::qt_keys::press_of(key, 0) else { return };
        let outcome = self.as_mut().rust_mut().card.shoulder(button);
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
            Outcome::Restart => {
                self.as_mut().restart_asked();
                self.as_mut().resumed();
            }
            Outcome::QuitGame => self.as_mut().quit_game_asked(),
            Outcome::QuitEmuSen => self.as_mut().quit_asked(),
            Outcome::Screenshot => self.as_mut().screenshot_asked(),
            Outcome::ChangeDisc(index) => {
                self.as_mut().disc_asked(index);
                self.as_mut().resumed();
            }
            Outcome::InsertDisc(at) => {
                let path = self.card.others().get(at).map(|g| QString::from(&g.path)).unwrap_or_default();
                self.as_mut().insert_asked(path);
                self.as_mut().resumed();
            }
            Outcome::Eject => {
                self.as_mut().eject_asked();
                self.as_mut().resumed();
            }
            Outcome::SwapPlayers => {
                self.as_mut().swap_asked();
                self.as_mut().resumed();
            }
            Outcome::Setting { key, value } => {
                self.as_mut().setting_asked(QString::from(key.as_str()), QString::from(value.as_str()));
                self.shown();
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
