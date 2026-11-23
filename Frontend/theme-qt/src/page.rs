//! `Page`, a game's page as a theme reads it, read-only (EmuSen_Qt.md §3.2).

use crate::glue::ffi;
use crate::rows::qobject::ThemeRows;
use crate::view::{Cell, Row};
use core::pin::Pin;
use cxx_qt::casting::Upcast;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QList, QString, QStringList};
use emusen_qt_models::game_page_model::{ActionItem, Item};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("emusen-theme-qt/src/rows.cxxqt.h");
        type ThemeRows = crate::rows::qobject::ThemeRows;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element = "Page"]
        #[qml_singleton]
        #[qproperty(bool, ready, READ, NOTIFY = shown)]
        #[qproperty(QString, key, READ, NOTIFY = shown)]
        #[qproperty(QString, title, READ, NOTIFY = shown)]
        #[qproperty(QString, facts, READ, NOTIFY = shown)]
        #[qproperty(QString, cover, READ, NOTIFY = shown)]
        #[qproperty(QString, hero, READ, NOTIFY = shown)]
        #[qproperty(QString, clip, READ, NOTIFY = shown)]
        #[qproperty(*mut ThemeRows, actions, READ, CONSTANT)]
        #[qproperty(QStringList, sections, READ, NOTIFY = shown)]
        #[qproperty(i32, section, READ, NOTIFY = shown)]
        #[qproperty(i32, shows, READ, NOTIFY = shown)]
        #[qproperty(QString, words, READ, NOTIFY = shown)]
        #[qproperty(QString, picture, READ, NOTIFY = shown)]
        #[qproperty(*mut ThemeRows, rows, READ, CONSTANT)]
        #[qproperty(bool, cover_art, cxx_name = "coverArt", READ, NOTIFY = shown)]
        #[qproperty(i32, focused_action, cxx_name = "focusedAction", READ, NOTIFY = shown)]
        #[qproperty(i32, focused_content, cxx_name = "focusedContent", READ, NOTIFY = shown)]
        #[qproperty(i32, help, READ, NOTIFY = shown)]
        #[qproperty(bool, asking, READ, NOTIFY = shown)]
        #[qproperty(QString, question_title, cxx_name = "questionTitle", READ, NOTIFY = shown)]
        #[qproperty(QStringList, question_sentences, cxx_name = "questionSentences", READ, NOTIFY = shown)]
        #[qproperty(QStringList, question_choices, cxx_name = "questionChoices", READ, NOTIFY = shown)]
        #[qproperty(i32, question_back, cxx_name = "questionBack", READ, NOTIFY = shown)]
        /// 1.11: Eclipse's board's tabs and the one shown, its kicker and line, its buttons, its state cards, the words beside
        /// them where the game has none, and the help for the focus in its words (R-20261116-29, -30, -36).
        #[qproperty(QStringList, tabs, READ, NOTIFY = shown)]
        #[qproperty(i32, tab, READ, NOTIFY = shown)]
        #[qproperty(QString, kicker, READ, NOTIFY = shown)]
        #[qproperty(QString, line, READ, NOTIFY = shown)]
        #[qproperty(*mut ThemeRows, pills, READ, CONSTANT)]
        #[qproperty(*mut ThemeRows, cards, READ, CONSTANT)]
        #[qproperty(QString, note, READ, NOTIFY = shown)]
        #[qproperty(i32, board_help, cxx_name = "boardHelp", READ, NOTIFY = shown)]
        /// 1.11: the board's trail, the key of the section its first link opens and the console's id its second opens.
        #[qproperty(QStringList, trail, READ, NOTIFY = shown)]
        #[qproperty(QString, trail_section, cxx_name = "trailSection", READ, NOTIFY = shown)]
        #[qproperty(QString, console, READ, NOTIFY = shown)]
        type ThemePage = super::ThemePageRust;
    }

    unsafe extern "RustQt" {
        /// What the page shows changed: one signal for every member, as the host feeds them together.
        #[qsignal]
        fn shown(self: Pin<&mut ThemePage>);
    }

    impl cxx_qt::Initialize for ThemePage {}
}

/// `Page.actions`' roles, as the program's action list reads them.
pub const PAGE_ACTION_ROLES: [&str; 7] = ["label", "value", "button", "filled", "stepped", "stepLeft", "stepRight"];
/// `Page.rows`' roles: a row of the section (`steps`, `opens`) or a card (a state, a picture, a cover: `line`, `picture`, `marked`).
pub const PAGE_ROW_ROLES: [&str; 8] = ["key", "label", "value", "line", "picture", "steps", "opens", "marked"];

/// `Page.pills`' roles: the board's button, its pad button (-1 none), filled, whether it is the heart's and the heart on,
/// the place among `actions` of the action it is, which `pageAct` and `pageFocus` take, and the tile among `rows` it
/// chooses instead, -1 for none: Resume's is the resume point's (1.11).
pub const PAGE_PILL_ROLES: [&str; 8] = ["key", "label", "button", "filled", "heart", "on", "act", "row"];
/// `Page.cards`' roles: a state card or the empty one, its badge's words, and `row`, its tile among `rows`, -1 for the empty card (1.11).
pub const PAGE_CARD_ROLES: [&str; 7] = ["key", "label", "line", "picture", "marked", "badge", "row"];

/// Everything the singleton shows, as the host reads it from the program's model.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PageFacts {
    pub ready: bool,
    pub key: String,
    pub title: String,
    pub facts: String,
    pub cover: String,
    pub hero: String,
    pub clip: String,
    pub actions: Vec<Row>,
    pub sections: Vec<String>,
    pub section: i32,
    /// 0 the section's words and picture, 1 cards, 2 rows.
    pub shows: i32,
    pub words: String,
    pub picture: String,
    pub rows: Vec<Row>,
    pub cover_art: bool,
    pub focused: (i32, i32),
    /// The help bar's kind for the focus (`Help.entries`' kind), -1 where none is the page's (the media viewer).
    pub help: i32,
    pub asking: bool,
    pub question_title: String,
    pub question_sentences: Vec<String>,
    pub question_choices: Vec<String>,
    pub question_back: i32,
    pub tabs: Vec<String>,
    pub tab: i32,
    pub kicker: String,
    pub line: String,
    pub pills: Vec<Row>,
    pub cards: Vec<Row>,
    pub note: String,
    /// The help kind for the focus in the boards' words, -1 where they draw none and `help` is shown.
    pub board_help: i32,
    pub trail: Vec<String>,
    pub trail_section: String,
    pub console: String,
}

/// The board's buttons' rows: each with the place among the actions of the action it is.
pub fn pill_rows(pills: &[(emusen_qt_models::game_page::Action, i32, i32)]) -> Vec<Row> {
    pills
        .iter()
        .map(|(p, act, row)| {
            vec![
                Cell::Text(p.key.to_owned()),
                Cell::Text(p.label.to_owned()),
                Cell::Whole(p.button),
                Cell::Flag(p.filled),
                Cell::Flag(p.key == emusen_qt_models::game_page::FAVOURITE),
                Cell::Flag(p.value == emusen_qt_models::game_page::HEART_ON),
                Cell::Whole(*act),
                Cell::Whole(*row),
            ]
        })
        .collect()
}

/// The state cards' rows.
pub fn card_rows(cards: &[emusen_qt_models::game_page::Card]) -> Vec<Row> {
    cards
        .iter()
        .map(|c| {
            vec![
                Cell::Text(c.key.clone()),
                Cell::Text(c.label.clone()),
                Cell::Text(c.line.clone()),
                Cell::Text(c.picture.clone()),
                Cell::Flag(c.marked),
                Cell::Text(c.badge.clone()),
                Cell::Whole(c.row.map_or(-1, |r| r as i32)),
            ]
        })
        .collect()
}

/// The actions' rows.
pub fn action_rows(actions: &[ActionItem]) -> Vec<Row> {
    actions
        .iter()
        .map(|a| {
            vec![
                Cell::Text(a.label().to_owned()),
                Cell::Text(a.value().to_owned()),
                Cell::Whole(a.button()),
                Cell::Flag(a.filled()),
                Cell::Flag(a.stepped()),
                Cell::Flag(a.step_left()),
                Cell::Flag(a.step_right()),
            ]
        })
        .collect()
}

/// The content's rows.
pub fn content_rows(items: &[Item]) -> Vec<Row> {
    items
        .iter()
        .map(|i| {
            vec![
                Cell::Text(i.key().to_owned()),
                Cell::Text(i.label().to_owned()),
                Cell::Text(i.value().to_owned()),
                Cell::Text(i.line().to_owned()),
                Cell::Text(i.picture().to_owned()),
                Cell::Flag(i.steps()),
                Cell::Flag(i.opens()),
                Cell::Flag(i.marked()),
            ]
        })
        .collect()
}

fn list(words: &[String]) -> QStringList {
    let mut out = QList::<QString>::default();
    words.iter().for_each(|w| out.append(QString::from(w.as_str())));
    QStringList::from(&out)
}

pub struct ThemePageRust {
    ready: bool,
    key: QString,
    title: QString,
    facts: QString,
    cover: QString,
    hero: QString,
    clip: QString,
    actions: *mut ThemeRows,
    sections: QStringList,
    section: i32,
    shows: i32,
    words: QString,
    picture: QString,
    rows: *mut ThemeRows,
    cover_art: bool,
    focused_action: i32,
    focused_content: i32,
    help: i32,
    asking: bool,
    question_title: QString,
    question_sentences: QStringList,
    question_choices: QStringList,
    question_back: i32,
    tabs: QStringList,
    tab: i32,
    kicker: QString,
    line: QString,
    pills: *mut ThemeRows,
    cards: *mut ThemeRows,
    note: QString,
    board_help: i32,
    trail: QStringList,
    trail_section: QString,
    console: QString,
    last: PageFacts,
}

impl Default for ThemePageRust {
    fn default() -> Self {
        ThemePageRust {
            ready: false,
            key: QString::default(),
            title: QString::default(),
            facts: QString::default(),
            cover: QString::default(),
            hero: QString::default(),
            clip: QString::default(),
            actions: std::ptr::null_mut(),
            sections: QStringList::default(),
            section: 0,
            shows: 0,
            words: QString::default(),
            picture: QString::default(),
            rows: std::ptr::null_mut(),
            cover_art: false,
            focused_action: -1,
            focused_content: 0,
            help: -1,
            asking: false,
            question_title: QString::default(),
            question_sentences: QStringList::default(),
            question_choices: QStringList::default(),
            question_back: 0,
            tabs: QStringList::default(),
            tab: 0,
            kicker: QString::default(),
            line: QString::default(),
            pills: std::ptr::null_mut(),
            cards: std::ptr::null_mut(),
            note: QString::default(),
            board_help: -1,
            trail: QStringList::default(),
            trail_section: QString::default(),
            console: QString::default(),
            last: PageFacts { focused: (-1, 0), help: -1, board_help: -1, ..PageFacts::default() },
        }
    }
}

impl cxx_qt::Initialize for qobject::ThemePage {
    fn initialize(mut self: Pin<&mut Self>) {
        let made = |page: Pin<&mut Self>, roles: &'static [&'static str]| {
            let list = ffi::new_rows(page.upcast_pin());
            // Safety: the list was just made, a child of this singleton.
            if let Some(rows) = unsafe { list.as_mut() } {
                unsafe { Pin::new_unchecked(rows) }.set_roles(roles);
            }
            list
        };
        let actions = made(self.as_mut(), &PAGE_ACTION_ROLES);
        let rows = made(self.as_mut(), &PAGE_ROW_ROLES);
        let pills = made(self.as_mut(), &PAGE_PILL_ROLES);
        let cards = made(self.as_mut(), &PAGE_CARD_ROLES);
        self.as_mut().rust_mut().actions = actions;
        self.as_mut().rust_mut().rows = rows;
        self.as_mut().rust_mut().pills = pills;
        self.as_mut().rust_mut().cards = cards;
    }
}

/// A list's rows replaced; a null list is none.
fn fill(list: *mut ThemeRows, rows: &[Row]) {
    // Safety: each list is the singleton's child, made in `initialize`.
    if let Some(list) = unsafe { list.as_mut() } {
        unsafe { Pin::new_unchecked(list) }.replace(rows.to_vec());
    }
}

impl qobject::ThemePage {
    /// The host's feed; nothing is told where nothing changed.
    pub fn show(mut self: Pin<&mut Self>, facts: &PageFacts) {
        if self.last == *facts {
            return;
        }
        fill(self.actions, &facts.actions);
        fill(self.rows, &facts.rows);
        fill(self.pills, &facts.pills);
        fill(self.cards, &facts.cards);
        let text = |t: &str| QString::from(t);
        let mut me = self.as_mut().rust_mut();
        me.ready = facts.ready;
        me.key = text(&facts.key);
        me.title = text(&facts.title);
        me.facts = text(&facts.facts);
        me.cover = text(&facts.cover);
        me.hero = text(&facts.hero);
        me.clip = text(&facts.clip);
        me.sections = list(&facts.sections);
        me.section = facts.section;
        me.shows = facts.shows;
        me.words = text(&facts.words);
        me.picture = text(&facts.picture);
        me.cover_art = facts.cover_art;
        (me.focused_action, me.focused_content) = facts.focused;
        me.help = facts.help;
        me.asking = facts.asking;
        me.question_title = text(&facts.question_title);
        me.question_sentences = list(&facts.question_sentences);
        me.question_choices = list(&facts.question_choices);
        me.question_back = facts.question_back;
        me.tabs = list(&facts.tabs);
        me.tab = facts.tab;
        me.kicker = text(&facts.kicker);
        me.line = text(&facts.line);
        me.note = text(&facts.note);
        me.board_help = facts.board_help;
        me.trail = list(&facts.trail);
        me.trail_section = text(&facts.trail_section);
        me.console = text(&facts.console);
        me.last = facts.clone();
        drop(me);
        self.shown();
    }

    /// The facts last shown, for the host's tests.
    pub fn shown_facts(&self) -> &PageFacts {
        &self.last
    }
}
