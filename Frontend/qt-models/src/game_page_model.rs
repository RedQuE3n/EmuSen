//! `GamePageModel`, a game's page as a QAbstractListModel (EmuSen_Qt.md §7.8): game_page.rs's content (the section's
//! rows or tiles) with the roles the kit's page frame reads, the hero, the facts and the Overview as properties, and the
//! actions as plain data for `PageActionModel`, the frame's action list. A read runs off the GUI thread and comes back
//! through qt_thread().queue; a change is saved through the source at once and the game read again.

use crate::clients::ClientSource;
use crate::game_model::cover_url;
use crate::game_page::{Content, GamePage, GamePageSource, Outcome, Read};
use crate::help_model::shown_screen;
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QList, QModelIndex, QString, QStringList, QVariant};
use emusen_platform_client::library::GameFile;
use std::sync::Arc;

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
    }

    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        /// The platform library and the cores folder, as LibraryModel opens the clients with.
        #[qproperty(QString, platform_library, cxx_name = "platformLibrary")]
        #[qproperty(QString, cores)]
        /// The game's file as LibraryModel.gameFile gives it; the page is read each time it changes.
        #[qproperty(QString, game)]
        #[qproperty(bool, ready, READ, NOTIFY)]
        #[qproperty(QString, title, READ, NOTIFY)]
        #[qproperty(QString, facts, READ, NOTIFY)]
        /// Pictures and the clip as URLs; empty for none.
        #[qproperty(QString, cover, READ, NOTIFY)]
        #[qproperty(QString, hero, READ, NOTIFY)]
        #[qproperty(QString, clip, READ, NOTIFY)]
        #[qproperty(f64, cover_aspect, cxx_name = "coverAspect", READ, NOTIFY)]
        #[qproperty(QString, cover_name, cxx_name = "coverName", READ, NOTIFY)]
        #[qproperty(QStringList, sections, READ, CONSTANT)]
        /// The section shown, which the strip's shoulders and clicks choose.
        #[qproperty(i32, section)]
        /// What the section shows: 0 the Overview's words and picture, 1 tiles, 2 rows.
        #[qproperty(i32, shows, READ, NOTIFY)]
        #[qproperty(QString, words, READ, NOTIFY)]
        #[qproperty(QString, picture, READ, NOTIFY)]
        /// The actions as plain data, PageActionModel's `actions`.
        #[qproperty(QString, actions, READ, NOTIFY)]
        /// The help bar's screen (HelpModel's numbers) for what `focus` names.
        #[qproperty(i32, help, READ, NOTIFY)]
        #[qproperty(QString, problem, READ, NOTIFY)]
        /// A question asked before a change that cannot be undone, in the kit's Question's parts.
        #[qproperty(bool, asking, READ, NOTIFY)]
        #[qproperty(QString, question_title, cxx_name = "questionTitle", READ, NOTIFY)]
        #[qproperty(QStringList, question_sentences, cxx_name = "questionSentences", READ, NOTIFY)]
        #[qproperty(QStringList, question_choices, cxx_name = "questionChoices", READ, NOTIFY)]
        #[qproperty(i32, question_back, cxx_name = "questionBack", READ, NOTIFY)]
        type GamePageModel = super::GamePageModelRust;
    }

    extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn data(self: &GamePageModel, index: &QModelIndex, role: i32) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &GamePageModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &GamePageModel, parent: &QModelIndex) -> i32;

        /// The game read again.
        #[qinvokable]
        fn refresh(self: Pin<&mut GamePageModel>);

        /// A on an action, by its index.
        #[qinvokable]
        fn act(self: Pin<&mut GamePageModel>, index: i32);

        /// Left (-1) or Right (+1) on an action: whether it stepped its value (Resume's states); else the focus moves on.
        #[qinvokable]
        #[cxx_name = "stepAction"]
        fn step_action(self: Pin<&mut GamePageModel>, index: i32, by: i32) -> bool;

        /// A on a row or a tile of the content.
        #[qinvokable]
        fn choose(self: Pin<&mut GamePageModel>, index: i32);

        /// Left or Right on a row of the content.
        #[qinvokable]
        fn step(self: Pin<&mut GamePageModel>, index: i32, by: i32);

        /// A help bar's declared action: the page answers Y, the heart, itself, and passes every other on.
        #[qinvokable]
        fn declared(self: Pin<&mut GamePageModel>, key: &QString);

        /// A declared action the screens answer (Start).
        #[qsignal]
        fn passed(self: Pin<&mut GamePageModel>, key: QString);

        /// The question answered, by the answer's index: the second makes the change asked about.
        #[qinvokable]
        fn answer(self: Pin<&mut GamePageModel>, index: i32);

        /// The game was hidden from the library: the page closes.
        #[qsignal]
        fn hidden(self: Pin<&mut GamePageModel>);

        /// The question was answered: the page takes the focus back from it.
        #[qsignal]
        fn answered(self: Pin<&mut GamePageModel>);

        /// The focus moved: an action's index, else -1 and the content's index; the help bar follows it.
        #[qinvokable]
        fn focus(self: Pin<&mut GamePageModel>, action: i32, content: i32);

        /// The game is to start, from a state's file where one is named.
        #[qsignal]
        fn played(self: Pin<&mut GamePageModel>, path: QString, state: QString);

        /// A door to a later piece's page, by its row's key.
        #[qsignal]
        fn door(self: Pin<&mut GamePageModel>, key: QString);

        /// The folder holding the game's file, for the program's one folder action (`Folders.open`).
        #[qsignal]
        #[cxx_name = "openFolder"]
        fn open_folder(self: Pin<&mut GamePageModel>, path: QString);
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut GamePageModel>);

        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut GamePageModel>);
    }

    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        /// GamePageModel's `actions`.
        #[qproperty(QString, actions)]
        type PageActionModel = super::PageActionModelRust;
    }

    extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn data(self: &PageActionModel, index: &QModelIndex, role: i32) -> QVariant;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &PageActionModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &PageActionModel, parent: &QModelIndex) -> i32;
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        fn begin_reset_model(self: Pin<&mut PageActionModel>);

        #[inherit]
        #[cxx_name = "endResetModel"]
        fn end_reset_model(self: Pin<&mut PageActionModel>);

        #[inherit]
        #[qsignal]
        #[cxx_name = "dataChanged"]
        fn data_changed(self: Pin<&mut PageActionModel>, top_left: &QModelIndex, bottom_right: &QModelIndex, roles: &QVector_i32);

        #[inherit]
        fn index(self: &PageActionModel, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/qvector.h");
        type QVector_i32 = cxx_qt_lib::QVector<i32>;
    }

    impl cxx_qt::Threading for GamePageModel {}
    impl cxx_qt::Initialize for GamePageModel {}
    impl cxx_qt::Initialize for PageActionModel {}
}

fn list(words: impl Iterator<Item = String>) -> QStringList {
    let mut out = QList::<QString>::default();
    for w in words {
        out.append(QString::from(w.as_str()));
    }
    QStringList::from(&out)
}

/// One item of the content as the frame's delegates read it.
#[derive(Clone, Debug, Default, PartialEq)]
struct Item {
    key: String,
    label: String,
    value: String,
    line: String,
    picture: String,
    steps: bool,
    opens: bool,
    marked: bool,
}

fn items(content: &Content) -> Vec<Item> {
    match content {
        Content::Text { .. } => Vec::new(),
        Content::Tiles(tiles) => tiles
            .iter()
            .map(|t| Item { key: t.key.clone(), label: t.label.clone(), line: t.line.clone(), picture: cover_url(&t.picture), marked: t.marked, ..Item::default() })
            .collect(),
        Content::Rows(rows) => rows
            .iter()
            .map(|r| Item { key: r.key.into(), label: r.label.into(), value: r.value.clone(), steps: r.steps, opens: r.opens, ..Item::default() })
            .collect(),
    }
}

/// What a Yes to the page's question does: a change saved, or the game started from a state.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Pending {
    Change(crate::game_page::Change),
    Play(String),
}

pub struct GamePageModelRust {
    platform_library: QString,
    cores: QString,
    game: QString,
    ready: bool,
    title: QString,
    facts: QString,
    cover: QString,
    hero: QString,
    clip: QString,
    cover_aspect: f64,
    cover_name: QString,
    sections: QStringList,
    section: i32,
    shows: i32,
    words: QString,
    picture: QString,
    actions: QString,
    help: i32,
    problem: QString,
    asking: bool,
    question_title: QString,
    question_sentences: QStringList,
    question_choices: QStringList,
    question_back: i32,
    /// What the question asks about.
    pending: Option<Pending>,
    page: GamePage,
    file: GameFile,
    items: Vec<Item>,
    /// The focus as last told: an action's index or none, and the content's index.
    focus: (Option<usize>, usize),
    /// The clients, opened on the first read and kept.
    source: Option<Arc<ClientSource>>,
    asked: u64,
}

impl Default for GamePageModelRust {
    fn default() -> Self {
        let page = GamePage::default();
        GamePageModelRust {
            platform_library: QString::default(),
            cores: QString::default(),
            game: QString::default(),
            ready: false,
            title: QString::default(),
            facts: QString::default(),
            cover: QString::default(),
            hero: QString::default(),
            clip: QString::default(),
            cover_aspect: 0.75,
            cover_name: QString::default(),
            sections: list(page.sections().into_iter()),
            section: 0,
            shows: 0,
            words: QString::default(),
            picture: QString::default(),
            actions: QString::default(),
            help: shown_screen(crate::help::Screen::GamePage),
            problem: QString::default(),
            asking: false,
            question_title: QString::default(),
            question_sentences: QStringList::default(),
            question_choices: QStringList::default(),
            question_back: -1,
            pending: None,
            page,
            file: GameFile::default(),
            items: Vec::new(),
            focus: (Some(0), 0),
            source: None,
            asked: 0,
        }
    }
}

impl cxx_qt::Initialize for qobject::GamePageModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut()
            .on_game_changed(|mut model| {
                // A new game opens on its Overview, the focus on Play (EmuSen_Interface.md §4 screen 3).
                model.as_mut().rust_mut().page.section = 0;
                model.as_mut().rust_mut().focus = (Some(0), 0);
                if model.section != 0 {
                    model.as_mut().rust_mut().section = 0;
                    model.as_mut().section_changed();
                }
                model.refresh();
            })
            .release();
        self.on_section_changed(|mut model| {
            let section = model.section.max(0) as usize;
            if model.page.section != section {
                model.as_mut().rust_mut().page.choose_section(section);
                model.as_mut().rust_mut().focus = (Some(0), 0);
                model.shown(true);
            }
        })
        .release();
    }
}

const KEY: i32 = 0;
const LABEL: i32 = 1;
const VALUE: i32 = 2;
const LINE: i32 = 3;
const PICTURE: i32 = 4;
const STEPS: i32 = 5;
const OPENS: i32 = 6;
const MARKED: i32 = 7;

impl qobject::GamePageModel {
    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some(item) = self.items.get(index.row() as usize) else { return QVariant::default() };
        let text = |s: &str| QVariant::from(&QString::from(s));
        match role {
            KEY => text(&item.key),
            LABEL => text(&item.label),
            VALUE => text(&item.value),
            LINE => text(&item.line),
            PICTURE => text(&item.picture),
            STEPS => QVariant::from(&item.steps),
            OPENS => QVariant::from(&item.opens),
            MARKED => QVariant::from(&item.marked),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
        let names = [(KEY, "key"), (LABEL, "label"), (VALUE, "value"), (LINE, "line"), (PICTURE, "picture"), (STEPS, "steps"), (OPENS, "opens"), (MARKED, "marked")];
        for (role, name) in names {
            roles.insert(role, QByteArray::from(name));
        }
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.items.len() as i32
    }

    pub fn refresh(mut self: Pin<&mut Self>) {
        let (library, game) = (self.platform_library.to_string(), self.game.to_string());
        let file: GameFile = serde_json::from_str(&game).unwrap_or_default();
        if library.is_empty() || file.path.is_empty() {
            return;
        }
        self.as_mut().rust_mut().file = file.clone();
        let asked = self.asked.wrapping_add(1);
        self.as_mut().rust_mut().asked = asked;
        let source = self.source.clone();
        let cores = self.cores.to_string();
        let thread = self.qt_thread();
        std::thread::spawn(move || {
            let source = match source {
                Some(s) => Ok(s),
                None => {
                    let config = emusen_crystal_client::Config { cores: (!cores.is_empty()).then(|| vec![cores]), ..Default::default() };
                    ClientSource::open(&library, &config).map(Arc::new)
                }
            };
            let read = source.as_ref().map_err(Clone::clone).and_then(|s| s.read(&file));
            let _ = thread.queue(move |mut model| {
                if let Ok(s) = source {
                    model.as_mut().rust_mut().source = Some(s);
                }
                if model.asked != asked {
                    return;
                }
                match read {
                    Ok(read) => {
                        model.as_mut().said(String::new());
                        model.as_mut().apply(read);
                    }
                    Err(words) => model.as_mut().said(words),
                }
            });
        });
    }

    fn apply(mut self: Pin<&mut Self>, read: Read) {
        let fresh = self.page.read.title.is_empty() || self.page.read.cover != read.cover || !self.ready;
        let now = crate::words::now();
        self.as_mut().rust_mut().page.now = now;
        self.as_mut().rust_mut().page.reread(read);
        let r = self.page.read.clone();
        macro_rules! put {
            ($field:ident, $changed:ident, $value:expr) => {{
                let value = $value;
                if self.$field != value {
                    self.as_mut().rust_mut().$field = value;
                    self.as_mut().$changed();
                }
            }};
        }
        put!(title, title_changed, QString::from(r.title.as_str()));
        put!(cover_name, cover_name_changed, QString::from(format!("{}, cover", r.title).as_str()));
        put!(cover, cover_changed, QString::from(cover_url(&r.cover).as_str()));
        put!(hero, hero_changed, QString::from(cover_url(&r.hero).as_str()));
        put!(clip, clip_changed, QString::from(cover_url(&r.clip).as_str()));
        put!(cover_aspect, cover_aspect_changed, r.cover_aspect);
        self.as_mut().shown(fresh);
        if !self.ready {
            self.as_mut().rust_mut().ready = true;
            self.ready_changed();
        }
    }

    /// What the page shows read again from the view: the facts, the actions, the content (reset where the section
    /// turned, else in place), the help.
    fn shown(mut self: Pin<&mut Self>, turned: bool) {
        let facts = QString::from(self.page.facts().as_str());
        if self.facts != facts {
            self.as_mut().rust_mut().facts = facts;
            self.as_mut().facts_changed();
        }
        let actions: Vec<serde_json::Value> = self
            .page
            .actions()
            .iter()
            .map(|a| serde_json::json!({ "label": a.label, "value": a.value, "button": a.button, "filled": a.filled, "stepped": a.stepped, "stepLeft": a.step_left, "stepRight": a.step_right }))
            .collect();
        let actions = QString::from(serde_json::Value::from(actions).to_string().as_str());
        if self.actions != actions {
            self.as_mut().rust_mut().actions = actions;
            self.as_mut().actions_changed();
        }
        let content = self.page.content();
        let (shows, words, picture) = match &content {
            Content::Text { words, picture } => (0, words.clone(), cover_url(picture)),
            Content::Tiles(_) => (1, String::new(), String::new()),
            Content::Rows(_) => (2, String::new(), String::new()),
        };
        let items = items(&content);
        if turned || items != self.items {
            self.as_mut().begin_reset_model();
            self.as_mut().rust_mut().items = items;
            self.as_mut().end_reset_model();
        }
        if self.shows != shows {
            self.as_mut().rust_mut().shows = shows;
            self.as_mut().shows_changed();
        }
        let (words, picture) = (QString::from(words.as_str()), QString::from(picture.as_str()));
        if self.words != words {
            self.as_mut().rust_mut().words = words;
            self.as_mut().words_changed();
        }
        if self.picture != picture {
            self.as_mut().rust_mut().picture = picture;
            self.as_mut().picture_changed();
        }
        self.helped();
    }

    fn helped(mut self: Pin<&mut Self>) {
        let (action, content) = self.focus;
        let help = shown_screen(self.page.help(action, content));
        if self.help != help {
            self.as_mut().rust_mut().help = help;
            self.help_changed();
        }
    }

    fn said(mut self: Pin<&mut Self>, words: String) {
        let words = QString::from(words.as_str());
        if self.problem != words {
            self.as_mut().rust_mut().problem = words;
            self.problem_changed();
        }
    }

    fn outcome(mut self: Pin<&mut Self>, outcome: Outcome) {
        match outcome {
            Outcome::Nothing => {}
            Outcome::Changed(change) => {
                let saved = self.source.as_ref().map_or(Ok(()), |s| s.change(&self.file, &change));
                let hid = saved.is_ok() && change == crate::game_page::Change::Hide;
                self.as_mut().said(saved.err().unwrap_or_default());
                if hid {
                    self.hidden();
                } else {
                    self.refresh();
                }
            }
            Outcome::Ask(question, change) => self.ask(question, Pending::Change(change)),
            Outcome::Play(Some(state)) => self.play_from(state),
            Outcome::Play(None) => {
                let path = QString::from(self.file.path.as_str());
                self.played(path, QString::default());
            }
            Outcome::Door(key) => self.door(QString::from(key)),
            Outcome::OpenFolder => {
                let folder = std::path::Path::new(&self.file.path).parent().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
                self.open_folder(QString::from(folder.as_str()));
            }
        }
    }

    /// The question form opened on `question`, its Yes doing `pending`.
    fn ask(mut self: Pin<&mut Self>, question: crate::question::Question, pending: Pending) {
        let back = crate::question::back_index(&question);
        self.as_mut().rust_mut().pending = Some(pending);
        self.as_mut().rust_mut().question_title = QString::from(question.title.as_str());
        self.as_mut().rust_mut().question_sentences = list(question.sentences.into_iter());
        self.as_mut().rust_mut().question_choices = list(question.choices.into_iter());
        self.as_mut().rust_mut().question_back = back;
        self.as_mut().question_title_changed();
        self.as_mut().question_sentences_changed();
        self.as_mut().question_choices_changed();
        self.as_mut().question_back_changed();
        self.as_mut().rust_mut().asking = true;
        self.asking_changed();
    }

    /// The game started from a state, asking first where the state was saved from another copy of the game; the
    /// runtime's record and the game's hash are read off the GUI thread.
    fn play_from(self: Pin<&mut Self>, state: String) {
        let (source, file) = (self.source.clone(), self.file.clone());
        let thread = self.qt_thread();
        std::thread::spawn(move || {
            let other = source.map_or(Ok(None), |s| s.other_copy(&file, &state));
            let _ = thread.queue(move |mut model| {
                let name = std::path::Path::new(&file.path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                match other {
                    Ok(Some(saved_from)) => model.ask(crate::game_page::other_copy_question(&saved_from, &name), Pending::Play(state)),
                    Ok(None) => model.played(QString::from(file.path.as_str()), QString::from(state.as_str())),
                    Err(words) => model.as_mut().said(words),
                }
            });
        });
    }

    pub fn act(self: Pin<&mut Self>, index: i32) {
        let outcome = self.page.act(index.max(0) as usize);
        self.outcome(outcome);
    }

    pub fn step_action(mut self: Pin<&mut Self>, index: i32, by: i32) -> bool {
        let stepped = self.as_mut().rust_mut().page.step_action(index.max(0) as usize, by);
        if stepped {
            self.shown(false);
        }
        stepped
    }

    pub fn choose(self: Pin<&mut Self>, index: i32) {
        let index = index.max(0) as usize;
        let outcome = match self.page.content() {
            Content::Tiles(_) => self.page.choose_tile(index),
            _ => self.page.choose_row(index),
        };
        self.outcome(outcome);
    }

    pub fn step(self: Pin<&mut Self>, index: i32, by: i32) {
        let outcome = self.page.step_row(index.max(0) as usize, by);
        self.outcome(outcome);
    }

    pub fn declared(self: Pin<&mut Self>, key: &QString) {
        match key.to_string().as_str() {
            crate::help::FAVOURITE => self.outcome(Outcome::Changed(crate::game_page::Change::Favourite)),
            crate::help::DELETE => {
                let outcome = self.page.delete(self.focus.1);
                self.outcome(outcome);
            }
            _ => self.passed(key.clone()),
        }
    }

    pub fn answer(mut self: Pin<&mut Self>, index: i32) {
        let pending = self.as_mut().rust_mut().pending.take();
        if self.asking {
            self.as_mut().rust_mut().asking = false;
            self.as_mut().asking_changed();
            self.as_mut().answered();
        }
        match (index, pending) {
            (1, Some(Pending::Change(change))) => self.outcome(Outcome::Changed(change)),
            (1, Some(Pending::Play(state))) => {
                let path = QString::from(self.file.path.as_str());
                self.played(path, QString::from(state.as_str()));
            }
            _ => {}
        }
    }

    pub fn focus(mut self: Pin<&mut Self>, action: i32, content: i32) {
        self.as_mut().rust_mut().focus = (usize::try_from(action).ok(), content.max(0) as usize);
        self.helped();
    }
}

/// One action as the frame's delegate reads it.
#[derive(Clone, Debug, Default, PartialEq)]
struct ActionItem {
    label: String,
    value: String,
    button: i32,
    filled: bool,
    stepped: bool,
    step_left: bool,
    step_right: bool,
}

/// GamePageModel's actions text read back.
fn action_items(text: &str) -> Vec<ActionItem> {
    let all: serde_json::Value = serde_json::from_str(text).unwrap_or_default();
    let item = |a: &serde_json::Value| {
        let text = |k: &str| a[k].as_str().unwrap_or("").to_owned();
        let flag = |k: &str| a[k].as_bool().unwrap_or(false);
        ActionItem {
            label: text("label"),
            value: text("value"),
            button: a["button"].as_i64().unwrap_or(-1) as i32,
            filled: flag("filled"),
            stepped: flag("stepped"),
            step_left: flag("stepLeft"),
            step_right: flag("stepRight"),
        }
    };
    all.as_array().map(|all| all.iter().map(item).collect()).unwrap_or_default()
}

#[derive(Default)]
pub struct PageActionModelRust {
    actions: QString,
    items: Vec<ActionItem>,
}

impl cxx_qt::Initialize for qobject::PageActionModel {
    fn initialize(self: Pin<&mut Self>) {
        self.on_actions_changed(|mut model| {
            let items = action_items(&model.actions.to_string());
            if items.len() == model.items.len() {
                // The same actions with new words: in place, so the focused one keeps its delegate and its focus.
                model.as_mut().rust_mut().items = items;
                let count = model.items.len() as i32;
                if count > 0 {
                    let (first, last) = (model.index(0, 0, &QModelIndex::default()), model.index(count - 1, 0, &QModelIndex::default()));
                    model.as_mut().data_changed(&first, &last, &cxx_qt_lib::QVector::<i32>::default());
                }
            } else {
                model.as_mut().begin_reset_model();
                model.as_mut().rust_mut().items = items;
                model.as_mut().end_reset_model();
            }
        })
        .release();
    }
}

impl qobject::PageActionModel {
    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some(a) = self.items.get(index.row() as usize) else { return QVariant::default() };
        match role {
            0 => QVariant::from(&QString::from(a.label.as_str())),
            1 => QVariant::from(&QString::from(a.value.as_str())),
            2 => QVariant::from(&a.button),
            3 => QVariant::from(&a.filled),
            4 => QVariant::from(&a.stepped),
            5 => QVariant::from(&a.step_left),
            6 => QVariant::from(&a.step_right),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
        for (role, name) in ["label", "value", "button", "filled", "stepped", "stepLeft", "stepRight"].iter().enumerate() {
            roles.insert(role as i32, QByteArray::from(*name));
        }
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.items.len() as i32
    }
}
