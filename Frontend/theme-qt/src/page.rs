//! `Page`, a game's page as a theme reads it (EmuSen_Qt.md §3.2): the game's words and pictures, the actions, the
//! sections (tabs) and the section's rows or state cards, the focus and the question, a read-only view the host feeds
//! from the program's GamePageModel. A theme changes them only through `Actions`' page calls.

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
}

/// The actions' rows.
pub fn action_rows(actions: &[ActionItem]) -> Vec<Row> {
    actions
        .iter()
        .map(|a| {
            vec![
                Cell::Text(a.label.clone()),
                Cell::Text(a.value.clone()),
                Cell::Whole(a.button),
                Cell::Flag(a.filled),
                Cell::Flag(a.stepped),
                Cell::Flag(a.step_left),
                Cell::Flag(a.step_right),
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
                Cell::Text(i.key.clone()),
                Cell::Text(i.label.clone()),
                Cell::Text(i.value.clone()),
                Cell::Text(i.line.clone()),
                Cell::Text(i.picture.clone()),
                Cell::Flag(i.steps),
                Cell::Flag(i.opens),
                Cell::Flag(i.marked),
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
            last: PageFacts { focused: (-1, 0), help: -1, ..PageFacts::default() },
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
        self.as_mut().rust_mut().actions = actions;
        self.as_mut().rust_mut().rows = rows;
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
        me.last = facts.clone();
        drop(me);
        self.shown();
    }

    /// The facts last shown, for the host's tests.
    pub fn shown_facts(&self) -> &PageFacts {
        &self.last
    }
}
