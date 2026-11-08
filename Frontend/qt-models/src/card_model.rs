//! `CardModel`, a card's page as a QAbstractListModel (EmuSen_Qt.md §7.1, stage 2): any card's pages (card.rs's
//! `Pages`, chosen by `card`) made from its `facts` and put on Qt as rows, title, the way back and the focused row's
//! pane, and a row chosen, stepped or B answered by the card's own rule, so QML writes no words and no logic.

use crate::card::{pages_of, Choice, Deck, Page, Pages};
use crate::row_roles;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QHash, QHashPair_i32_QByteArray, QList, QModelIndex, QString, QStringList, QVariant, QVector};

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
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qvector.h");
        type QVector_i32 = cxx_qt_lib::QVector<i32>;
    }

    /// The cards.
    #[qenum(CardModel)]
    enum Card {
        NoCard,
        Home,
        List,
    }

    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        #[qproperty(Card, card)]
        /// The page the card opens at, by its key; empty for its first.
        #[qproperty(QString, start)]
        /// Whether the card is shown: it opens at `start` each time this turns on.
        #[qproperty(bool, shown)]
        /// What the card's words are made from, as plain data its host's model gives.
        #[qproperty(QString, facts)]
        /// The page shown, by its key.
        #[qproperty(QString, page, READ, NOTIFY)]
        /// Whether the page shown is the one the card opened at.
        #[qproperty(bool, first, READ, NOTIFY)]
        #[qproperty(QString, title, READ, NOTIFY)]
        #[qproperty(QString, breadcrumb, READ, NOTIFY)]
        /// The line under the rows; empty for none.
        #[qproperty(QString, footer, READ, NOTIFY)]
        /// The focused row, which a page turn sets.
        #[qproperty(i32, current, READ, NOTIFY)]
        /// The focused row's pane: its lines and the muted line under each, or a picture with its caption.
        #[qproperty(QStringList, pane_lines, cxx_name = "paneLines", READ, NOTIFY)]
        #[qproperty(QStringList, pane_muted, cxx_name = "paneMuted", READ, NOTIFY)]
        #[qproperty(QString, pane_picture, cxx_name = "panePicture", READ, NOTIFY)]
        #[qproperty(QString, pane_caption, cxx_name = "paneCaption", READ, NOTIFY)]
        type CardModel = super::CardModelRust;
    }

    extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn data(self: &CardModel, index: &QModelIndex, role: i32) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &CardModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &CardModel, parent: &QModelIndex) -> i32;

        /// The card opened at `start`, else its first page, on its first row.
        #[qinvokable]
        fn open(self: Pin<&mut CardModel>);

        /// The card opened at a page of its own by its key, as Home's Power place opens the Home card at Power.
        #[qinvokable]
        #[cxx_name = "openAt"]
        fn open_at(self: Pin<&mut CardModel>, page: &QString);

        /// A row focused: the pane follows it.
        #[qinvokable]
        fn select(self: Pin<&mut CardModel>, index: i32);

        /// A row chosen, by its index on the page.
        #[qinvokable]
        fn choose(self: Pin<&mut CardModel>, index: i32);

        /// Left (-1) or Right (+1) on a row.
        #[qinvokable]
        fn step(self: Pin<&mut CardModel>, index: i32, by: i32);

        /// B on the page.
        #[qinvokable]
        fn back(self: Pin<&mut CardModel>);

        /// The card closes and the host carries out the action by its key.
        #[qsignal]
        fn acted(self: Pin<&mut CardModel>, key: QString);

        /// The host carries out the action by its key, and the card stays.
        #[qsignal]
        fn stayed(self: Pin<&mut CardModel>, key: QString);

        /// B on the page the card opened at: the card closes.
        #[qsignal]
        fn closed(self: Pin<&mut CardModel>);
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut CardModel>);

        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut CardModel>);

        #[inherit]
        fn index(self: &CardModel, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;

        #[inherit]
        #[qsignal]
        #[cxx_name = "dataChanged"]
        fn data_changed(self: Pin<&mut CardModel>, top_left: &QModelIndex, bottom_right: &QModelIndex, roles: &QVector_i32);
    }

    impl cxx_qt::Initialize for CardModel {}
}

pub struct CardModelRust {
    card: qobject::Card,
    start: QString,
    shown: bool,
    facts: QString,
    page: QString,
    first: bool,
    title: QString,
    breadcrumb: QString,
    footer: QString,
    current: i32,
    pane_lines: QStringList,
    pane_muted: QStringList,
    pane_picture: QString,
    pane_caption: QString,
    deck: Deck,
    page_shown: Page,
}

impl Default for CardModelRust {
    fn default() -> Self {
        CardModelRust {
            card: qobject::Card::NoCard,
            start: QString::default(),
            shown: false,
            facts: QString::default(),
            page: QString::default(),
            first: false,
            title: QString::default(),
            breadcrumb: QString::default(),
            footer: QString::default(),
            current: 0,
            pane_lines: QStringList::default(),
            pane_muted: QStringList::default(),
            pane_picture: QString::default(),
            pane_caption: QString::default(),
            deck: Deck::default(),
            page_shown: Page::default(),
        }
    }
}

impl cxx_qt::Initialize for qobject::CardModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_card_changed(|model| model.read()).release();
        self.as_mut().on_shown_changed(|model| if model.shown { model.open() }).release();
        self.on_facts_changed(|model| model.read()).release();
    }
}


fn list(words: impl Iterator<Item = String>) -> QStringList {
    let mut out = QList::<QString>::default();
    for w in words {
        out.append(QString::from(w.as_str()));
    }
    QStringList::from(&out)
}

impl qobject::CardModel {
    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let shown = self.page_shown.rows.get(index.row() as usize).map(|r| row_roles::Shown {
            key: &r.key,
            label: &r.label,
            value: &r.value,
            steps: r.steps,
            opens: r.opens,
            section_end: r.section_end,
            dimmed: false,
        });
        row_roles::data(shown, role)
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        row_roles::role_names()
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.page_shown.rows.len() as i32
    }

    fn pages(&self) -> Option<Box<dyn Pages>> {
        pages_of(match self.card {
            qobject::Card::Home => crate::home_card::CARD,
            qobject::Card::List => crate::list_card::CARD,
            _ => "",
        })
    }

    fn row_key(&self, index: i32) -> Option<String> {
        usize::try_from(index).ok().and_then(|i| self.page_shown.rows.get(i)).map(|r| r.key.clone())
    }

    pub fn open(self: Pin<&mut Self>) {
        let Some(first) = self.pages().map(|p| p.first()) else { return };
        let start = if self.start.is_empty() { QString::from(first) } else { self.start.clone() };
        self.open_at(&start);
    }

    pub fn open_at(mut self: Pin<&mut Self>, page: &QString) {
        self.as_mut().rust_mut().deck.open(&page.to_string());
        self.turned(0);
    }

    pub fn select(self: Pin<&mut Self>, index: i32) {
        self.focus(index, false);
    }

    pub fn choose(mut self: Pin<&mut Self>, index: i32) {
        let (Some(pages), Some(key)) = (self.pages(), self.row_key(index)) else { return };
        match pages.choose(self.deck.page(), &key) {
            Choice::Turn(page) => {
                self.as_mut().rust_mut().deck.turn(&page, index.max(0) as usize);
                self.turned(0);
            }
            Choice::Back => self.back(),
            Choice::Act(action) => self.as_mut().acted(QString::from(action.as_str())),
            Choice::Stay(action) => self.as_mut().stayed(QString::from(action.as_str())),
            Choice::Owed => {}
        }
    }

    pub fn step(mut self: Pin<&mut Self>, index: i32, by: i32) {
        let (Some(pages), Some(key)) = (self.pages(), self.row_key(index)) else { return };
        if let Some(action) = pages.step(self.deck.page(), &key, by) {
            self.as_mut().stayed(QString::from(action.as_str()));
        }
    }

    pub fn back(mut self: Pin<&mut Self>) {
        match self.as_mut().rust_mut().deck.back() {
            Some((_, at)) => self.turned(at as i32),
            None => self.as_mut().closed(),
        }
    }

    /// The page shown read again on row `at`; said on every turn, so a view moves to it even where the number is the
    /// same as the page before's.
    fn turned(mut self: Pin<&mut Self>, at: i32) {
        self.as_mut().read();
        self.focus(at, true);
    }

    fn focus(mut self: Pin<&mut Self>, index: i32, always: bool) {
        let at = index.clamp(0, (self.page_shown.rows.len() as i32 - 1).max(0));
        if self.current != at || always {
            self.as_mut().rust_mut().current = at;
            self.as_mut().current_changed();
        }
        let pane = self.page_shown.rows.get(at as usize).map(|r| r.pane.clone()).unwrap_or_default();
        let lines = list(pane.lines.iter().map(|(l, _)| l.clone()));
        let muted = list(pane.lines.iter().map(|(_, m)| m.clone()));
        if self.pane_lines != lines || self.pane_muted != muted {
            self.as_mut().rust_mut().pane_lines = lines;
            self.as_mut().rust_mut().pane_muted = muted;
            self.as_mut().pane_lines_changed();
            self.as_mut().pane_muted_changed();
        }
        let (picture, caption) = (QString::from(pane.picture.as_str()), QString::from(pane.caption.as_str()));
        if self.pane_picture != picture {
            self.as_mut().rust_mut().pane_picture = picture;
            self.as_mut().pane_picture_changed();
        }
        if self.pane_caption != caption {
            self.as_mut().rust_mut().pane_caption = caption;
            self.pane_caption_changed();
        }
    }

    /// The page's rows and words read again from the facts; the focused row is kept, clamped to the rows.
    fn read(mut self: Pin<&mut Self>) {
        let Some(pages) = self.pages() else { return };
        if self.deck.page().is_empty() {
            let first = pages.first();
            self.as_mut().rust_mut().deck.open(first);
        }
        let facts: serde_json::Value = serde_json::from_str(&self.facts.to_string()).unwrap_or(serde_json::Value::Null);
        let key = self.deck.page().to_owned();
        let shown = pages.page(&key, &facts);
        // The same page with the same rows changes their words in place, so a row's value steps under the focus; a
        // page turn resets the rows.
        let same = self.page.to_string() == key && self.page_shown.rows.len() == shown.rows.len() && !shown.rows.is_empty();
        if same {
            let count = shown.rows.len() as i32;
            self.as_mut().rust_mut().page_shown = shown;
            let (first, last) = (self.index(0, 0, &QModelIndex::default()), self.index(count - 1, 0, &QModelIndex::default()));
            self.as_mut().data_changed(&first, &last, &QVector::<i32>::default());
        } else {
            self.as_mut().begin_reset_model();
            self.as_mut().rust_mut().page_shown = shown;
            self.as_mut().end_reset_model();
        }
        let (page, first) = (QString::from(key.as_str()), self.deck.depth() <= 1);
        if self.page != page {
            self.as_mut().rust_mut().page = page;
            self.as_mut().page_changed();
        }
        if self.first != first {
            self.as_mut().rust_mut().first = first;
            self.as_mut().first_changed();
        }
        let (title, breadcrumb) = (QString::from(self.page_shown.title.as_str()), QString::from(self.page_shown.breadcrumb.as_str()));
        if self.title != title {
            self.as_mut().rust_mut().title = title;
            self.as_mut().title_changed();
        }
        if self.breadcrumb != breadcrumb {
            self.as_mut().rust_mut().breadcrumb = breadcrumb;
            self.as_mut().breadcrumb_changed();
        }
        let footer = QString::from(self.page_shown.footer.as_str());
        if self.footer != footer {
            self.as_mut().rust_mut().footer = footer;
            self.as_mut().footer_changed();
        }
        // Said after a reset, since a view's current row does not survive its model's reset.
        let current = self.current;
        self.focus(current, !same);
    }
}
