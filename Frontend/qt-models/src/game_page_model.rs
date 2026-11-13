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
        #[qproperty(f64, clip_volume, cxx_name = "clipVolume", READ, NOTIFY)]
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
        /// The scraper's test seam, the `test` member of its settings as JSON (a loopback server and its root); empty for
        /// ScreenScraper itself.
        #[qproperty(QString, scraper_test, cxx_name = "scraperTest")]
        /// This page's scrape as the service's progress words it: idle, running, done, stopped or cancelled.
        #[qproperty(QString, scrape_state, cxx_name = "scrapeState", READ, NOTIFY)]
        /// The media viewer: whether it is open, what it shows (0 a picture, 1 a clip, 2 the manual's page), its file as
        /// a URL, its caption, the manual's page, zoom steps and pan steps, and its help bar's screen.
        #[qproperty(bool, viewing, READ, NOTIFY)]
        #[qproperty(i32, view_kind, cxx_name = "viewKind", READ, NOTIFY)]
        #[qproperty(QString, view_source, cxx_name = "viewSource", READ, NOTIFY)]
        /// The game's PDF manual as a URL, the last viewer's once it closes; empty for none.
        #[qproperty(QString, view_manual, cxx_name = "viewManual", READ, NOTIFY)]
        #[qproperty(QString, view_caption, cxx_name = "viewCaption", READ, NOTIFY)]
        #[qproperty(i32, view_page, cxx_name = "viewPage", READ, NOTIFY)]
        #[qproperty(i32, view_zoom, cxx_name = "viewZoom", READ, NOTIFY)]
        #[qproperty(i32, view_pan_x, cxx_name = "viewPanX", READ, NOTIFY)]
        #[qproperty(i32, view_pan_y, cxx_name = "viewPanY", READ, NOTIFY)]
        #[qproperty(i32, view_help, cxx_name = "viewHelp", READ, NOTIFY)]
        /// Words typed for a detail of Edit Details: whether they are asked, the question's title, and the words it starts from.
        #[qproperty(bool, typing, READ, NOTIFY)]
        #[qproperty(QString, typing_title, cxx_name = "typingTitle", READ, NOTIFY)]
        #[qproperty(QString, typing_start, cxx_name = "typingStart", READ, NOTIFY)]
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

        /// A detail of the game was edited: the library reads its names again.
        #[qsignal]
        fn edited(self: Pin<&mut GamePageModel>);

        /// The game was hidden from the library: the page closes.
        #[qsignal]
        fn hidden(self: Pin<&mut GamePageModel>);

        /// The media viewer or the keyboard closed: the page takes the focus back from it.
        #[qsignal]
        fn returned(self: Pin<&mut GamePageModel>);

        /// The question was answered: the page takes the focus back from it.
        #[qsignal]
        fn answered(self: Pin<&mut GamePageModel>);

        /// The focus moved: an action's index, else -1 and the content's index; the help bar follows it.
        #[qinvokable]
        fn focus(self: Pin<&mut GamePageModel>, action: i32, content: i32);

        /// B on the page: Edit Details goes back to Details (true); else the page is the screens' to leave.
        #[qinvokable]
        fn back(self: Pin<&mut GamePageModel>) -> bool;

        /// A key on the media viewer: whether it was the viewer's.
        #[qinvokable]
        #[cxx_name = "viewKey"]
        fn view_key(self: Pin<&mut GamePageModel>, key: i32, modifiers: i32) -> bool;

        /// The manual's page count, as Qt PDF read it.
        #[qinvokable]
        #[cxx_name = "viewPages"]
        fn view_pages(self: Pin<&mut GamePageModel>, pages: i32);

        /// The media viewer closed.
        #[qinvokable]
        #[cxx_name = "viewClose"]
        fn view_close(self: Pin<&mut GamePageModel>);

        /// The words typed for the detail asked, or none for a question let go; the detail is saved at once.
        #[qinvokable]
        fn typed(self: Pin<&mut GamePageModel>, kept: bool, words: &QString);

        /// The game is to start, from a state's file where one is named.
        #[qsignal]
        fn played(self: Pin<&mut GamePageModel>, path: QString, state: QString);

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

        #[inherit]
        #[qsignal]
        #[cxx_name = "dataChanged"]
        fn data_changed(self: Pin<&mut GamePageModel>, top_left: &QModelIndex, bottom_right: &QModelIndex, roles: &QVector_i32);

        #[inherit]
        fn index(self: &GamePageModel, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;
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

/// How the content's list hears a new content: reset where the section turned or the items are others, else in place.
#[derive(Debug, PartialEq, Eq)]
enum ContentChange {
    Reset,
    /// Each run of neighbouring rows whose words, picture or mark changed, first and last; unchanged rows in none.
    InPlace(Vec<(usize, usize)>),
    None,
}

fn content_change(old: &[Item], new: &[Item], turned: bool) -> ContentChange {
    if turned || old.len() != new.len() || old.iter().zip(new).any(|(a, b)| a.key != b.key) {
        return ContentChange::Reset;
    }
    let mut runs: Vec<(usize, usize)> = Vec::new();
    for (i, _) in old.iter().zip(new).enumerate().filter(|(_, (a, b))| a != b) {
        match runs.last_mut() {
            Some((_, last)) if *last + 1 == i => *last = i,
            _ => runs.push((i, i)),
        }
    }
    if runs.is_empty() { ContentChange::None } else { ContentChange::InPlace(runs) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tile(key: &str, marked: bool) -> Item {
        Item { key: key.into(), label: key.into(), marked, ..Item::default() }
    }

    #[test]
    fn a_mark_moved_between_the_same_tiles_is_heard_in_place() {
        let old = [tile("s1", true), tile("s2", false), tile("s3", false)];
        let new = [tile("s1", false), tile("s2", true), tile("s3", false)];
        assert_eq!(content_change(&old, &new, false), ContentChange::InPlace(vec![(0, 1)]));
        assert_eq!(content_change(&old, &old, false), ContentChange::None);
        assert_eq!(content_change(&old, &new, true), ContentChange::Reset, "a turned section is reset");
        assert_eq!(content_change(&old, &new[..2], false), ContentChange::Reset, "a tile fewer is reset");
        assert_eq!(content_change(&old, &[tile("s1", true), tile("s3", false), tile("s2", false)], false), ContentChange::Reset, "others in order");
    }

    #[test]
    fn only_the_rows_that_changed_are_told_in_place_and_others_reset() {
        let old = [tile("s1", false), tile("s2", false), tile("s3", false), tile("s4", false)];
        let worded = |i: usize, f: &dyn Fn(&mut Item)| {
            let mut new = old.clone();
            f(&mut new[i]);
            new
        };
        let label = worded(2, &|t| t.label = "Slot 3, saved again".into());
        assert_eq!(content_change(&old, &label, false), ContentChange::InPlace(vec![(2, 2)]), "a label alone");
        let picture = worded(1, &|t| t.picture = "file:///states/s2.png".into());
        assert_eq!(content_change(&old, &picture, false), ContentChange::InPlace(vec![(1, 1)]), "a picture alone");
        let apart = [tile("s1", true), tile("s2", false), tile("s3", false), tile("s4", true)];
        assert_eq!(content_change(&old, &apart, false), ContentChange::InPlace(vec![(0, 0), (3, 3)]), "two rows apart, the rows between untold");
        assert_eq!(content_change(&[], &[], false), ContentChange::None, "empty to empty");
        assert_eq!(content_change(&[], &old, false), ContentChange::Reset, "empty to some");
        assert_eq!(content_change(&old, &[], false), ContentChange::Reset, "some to empty");
        let replaced = worded(1, &|t| t.key = "s9".into());
        assert_eq!(content_change(&old, &replaced, false), ContentChange::Reset, "one key replaced at the same count");
    }
}

/// What a Yes to the page's question does: a change saved, or the game started from a state.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Pending {
    Change(crate::game_page::Change),
    Play(String),
    /// The page's scrape stopped.
    Stop,
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
    clip_volume: f64,
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
    /// How deep the page is in telling QML of a change; a focus told meanwhile is kept, and applied once it has finished.
    telling: u32,
    /// The clients, opened on the first read and kept.
    source: Option<Arc<ClientSource>>,
    asked: u64,
    scraper_test: QString,
    scrape_state: QString,
    viewing: bool,
    view_kind: i32,
    view_source: QString,
    view_manual: QString,
    view_caption: QString,
    view_page: i32,
    view_zoom: i32,
    view_pan_x: i32,
    view_pan_y: i32,
    view_help: i32,
    /// The viewer while it is open.
    viewer: Option<emusen_esde::scene::Viewer>,
    typing: bool,
    typing_title: QString,
    typing_start: QString,
    /// The detail the typed words are for.
    typing_for: Option<&'static str>,
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
            clip_volume: 1.0,
            cover_aspect: crate::library::COVER_SHAPE,
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
            telling: 0,
            source: None,
            asked: 0,
            scraper_test: QString::default(),
            scrape_state: QString::from("idle"),
            viewing: false,
            view_kind: 0,
            view_source: QString::default(),
            view_manual: QString::default(),
            view_caption: QString::default(),
            view_page: 0,
            view_zoom: 0,
            view_pan_x: 0,
            view_pan_y: 0,
            view_help: shown_screen(crate::help::Screen::GamePageViewer),
            viewer: None,
            typing: false,
            typing_title: QString::default(),
            typing_start: QString::default(),
            typing_for: None,
        }
    }
}

impl cxx_qt::Initialize for qobject::GamePageModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut()
            .on_game_changed(|mut model| {
                // A new game opens on its Overview, the focus on Play (EmuSen_Interface.md §4 screen 3).
                model.as_mut().rust_mut().page.section = 0;
                model.as_mut().rust_mut().page.editing = false;
                model.as_mut().rust_mut().page.run = None;
                model.as_mut().rust_mut().focus = (Some(0), 0);
                model.as_mut().view_close();
                if model.section != 0 {
                    model.as_mut().telling(|mut model| {
                        model.as_mut().rust_mut().section = 0;
                        model.section_changed();
                    });
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
        // Cleared at the start of each attempt, so the same failure twice is said twice (EmuSen_Qt.md §7.9).
        self.as_mut().said(String::new());
        let source = self.source.clone();
        let cores = self.cores.to_string();
        let thread = self.qt_thread();
        std::thread::spawn(move || {
            let source = match source {
                Some(s) => Ok(s),
                None => crate::clients::source_for(&library, &cores).map(Arc::new),
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
                    Err(words) => model.as_mut().said(format!("The game's page could not be read: {words}")),
                }
            });
        });
    }

    fn apply(self: Pin<&mut Self>, read: Read) {
        self.telling(|mut model| {
            let fresh = model.page.read.title.is_empty() || model.page.read.cover != read.cover || !model.ready;
            let now = crate::words::now();
            model.as_mut().rust_mut().page.now = now;
            model.as_mut().rust_mut().page.reread(read);
            let r = model.page.read.clone();
            macro_rules! put {
                ($field:ident, $changed:ident, $value:expr) => {{
                    let value = $value;
                    if model.$field != value {
                        model.as_mut().rust_mut().$field = value;
                        model.as_mut().$changed();
                    }
                }};
            }
            put!(title, title_changed, QString::from(r.title.as_str()));
            put!(cover_name, cover_name_changed, QString::from(format!("{}, cover", r.title).as_str()));
            put!(cover, cover_changed, QString::from(cover_url(&r.cover).as_str()));
            put!(hero, hero_changed, QString::from(cover_url(&r.hero).as_str()));
            put!(clip, clip_changed, QString::from(cover_url(&r.clip).as_str()));
            put!(clip_volume, clip_volume_changed, r.clip_volume);
            put!(cover_aspect, cover_aspect_changed, r.cover_aspect);
            model.as_mut().shown(fresh);
            if !model.ready {
                model.as_mut().rust_mut().ready = true;
                model.ready_changed();
            }
        });
    }

    /// `tell` sends the page's signals with the focus kept: a focus QML tells back meanwhile is applied once, after the
    /// outermost telling has finished, so no call back into the page works on it while it is still telling (EmuSen_Qt.md §7.8.1).
    fn telling(mut self: Pin<&mut Self>, tell: impl FnOnce(Pin<&mut Self>)) {
        self.as_mut().rust_mut().telling += 1;
        tell(self.as_mut());
        self.as_mut().rust_mut().telling -= 1;
        if self.telling == 0 {
            self.helped();
        }
    }

    /// What the page shows read again from the view: the facts, the actions, the content (reset where the section
    /// turned, else in place), the help.
    fn shown(self: Pin<&mut Self>, turned: bool) {
        self.telling(|mut model| {
            let facts = QString::from(model.page.facts().as_str());
            if model.facts != facts {
                model.as_mut().rust_mut().facts = facts;
                model.as_mut().facts_changed();
            }
            let actions: Vec<serde_json::Value> = model
                .page
                .actions()
                .iter()
                .map(|a| serde_json::json!({ "label": a.label, "value": a.value, "button": a.button, "filled": a.filled, "stepped": a.stepped, "stepLeft": a.step_left, "stepRight": a.step_right }))
                .collect();
            let actions = QString::from(serde_json::Value::from(actions).to_string().as_str());
            if model.actions != actions {
                model.as_mut().rust_mut().actions = actions;
                model.as_mut().actions_changed();
            }
            let content = model.page.content();
            let (shows, words, picture) = match &content {
                Content::Text { words, picture } => (0, words.clone(), cover_url(picture)),
                Content::Tiles(_) => (1, String::new(), String::new()),
                Content::Rows(_) => (2, String::new(), String::new()),
            };
            let items = items(&content);
            match content_change(&model.items, &items, turned) {
                ContentChange::Reset => {
                    model.as_mut().begin_reset_model();
                    model.as_mut().rust_mut().items = items;
                    model.as_mut().end_reset_model();
                }
                // The same items with new words or marks: in place, so each delegate and the list's focus stay (EmuSen_Qt.md §7.8.1).
                ContentChange::InPlace(runs) => {
                    model.as_mut().rust_mut().items = items;
                    let root = QModelIndex::default();
                    for (first, last) in runs {
                        let (first, last) = (model.index(first as i32, 0, &root), model.index(last as i32, 0, &root));
                        model.as_mut().data_changed(&first, &last, &cxx_qt_lib::QVector::<i32>::default());
                    }
                }
                ContentChange::None => {}
            }
            if model.shows != shows {
                model.as_mut().rust_mut().shows = shows;
                model.as_mut().shows_changed();
            }
            let (words, picture) = (QString::from(words.as_str()), QString::from(picture.as_str()));
            if model.words != words {
                model.as_mut().rust_mut().words = words;
                model.as_mut().words_changed();
            }
            if model.picture != picture {
                model.as_mut().rust_mut().picture = picture;
                model.as_mut().picture_changed();
            }
        });
    }

    fn helped(mut self: Pin<&mut Self>) {
        let (action, content) = self.focus;
        let help = shown_screen(self.page.help(action, content));
        if self.help != help {
            self.as_mut().rust_mut().help = help;
            self.help_changed();
        }
    }

    fn said(self: Pin<&mut Self>, words: String) {
        let words = QString::from(words.as_str());
        if self.problem != words {
            self.telling(|mut model| {
                model.as_mut().rust_mut().problem = words;
                model.problem_changed();
            });
        }
    }

    fn outcome(self: Pin<&mut Self>, outcome: Outcome) {
        self.telling(|mut model| {
            match outcome {
                Outcome::Nothing => {}
                Outcome::Changed(change) => {
                    let saved = model.source.as_ref().map_or(Ok(()), |s| s.change(&model.file, &change));
                    let ok = saved.is_ok();
                    let hid = ok && change == crate::game_page::Change::Hide;
                    model.as_mut().said(saved.err().map(|why| crate::game_page::change_failed(&change, &why)).unwrap_or_default());
                    if hid {
                        model.hidden();
                        return;
                    }
                    if ok && matches!(change, crate::game_page::Change::Edit(..)) {
                        model.as_mut().edited();
                    }
                    if ok {
                        model.as_mut().rust_mut().page.changed(&change);
                        model.as_mut().shown(false);
                    }
                    model.refresh();
                }
                Outcome::Ask(question, change) => model.ask(question, Pending::Change(change)),
                Outcome::Play(Some(state)) => model.play_from(state),
                Outcome::Play(None) => {
                    let path = QString::from(model.file.path.as_str());
                    model.played(path, QString::default());
                }
                Outcome::Edit => {
                    model.as_mut().rust_mut().page.editing = true;
                    model.as_mut().rust_mut().focus = (None, 0);
                    model.as_mut().shown(true);
                }
                Outcome::View(index) => {
                    let viewer = model.page.viewer(index);
                    model.as_mut().rust_mut().viewer = viewer;
                    model.as_mut().viewed();
                }
                Outcome::Scrape => model.scrape(),
                Outcome::Stop(question) => model.ask(question, Pending::Stop),
                Outcome::Type { field, title, start } => {
                    model.as_mut().rust_mut().typing_for = Some(field);
                    model.as_mut().rust_mut().typing_title = QString::from(title.as_str());
                    model.as_mut().rust_mut().typing_start = QString::from(start.as_str());
                    model.as_mut().typing_title_changed();
                    model.as_mut().typing_start_changed();
                    model.as_mut().rust_mut().typing = true;
                    model.as_mut().typing_changed();
                }
                Outcome::OpenFolder => {
                    let folder = std::path::Path::new(&model.file.path).parent().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
                    model.open_folder(QString::from(folder.as_str()));
                }
            }
        });
    }

    /// The question form opened on `question`, its Yes doing `pending`.
    fn ask(self: Pin<&mut Self>, question: crate::question::Question, pending: Pending) {
        self.telling(|mut model| {
            let back = crate::question::back_index(&question);
            model.as_mut().rust_mut().pending = Some(pending);
            model.as_mut().rust_mut().question_title = QString::from(question.title.as_str());
            model.as_mut().rust_mut().question_sentences = list(question.sentences.into_iter());
            model.as_mut().rust_mut().question_choices = list(question.choices.into_iter());
            model.as_mut().rust_mut().question_back = back;
            model.as_mut().question_title_changed();
            model.as_mut().question_sentences_changed();
            model.as_mut().question_choices_changed();
            model.as_mut().question_back_changed();
            model.as_mut().rust_mut().asking = true;
            model.asking_changed();
        });
    }

    /// The game started from a state, asking first where the state was saved from another copy of the game; the
    /// runtime's record and the game's hash are read off the GUI thread.
    fn play_from(self: Pin<&mut Self>, state: String) {
        let (source, file) = (self.source.clone(), self.file.clone());
        let thread = self.qt_thread();
        std::thread::spawn(move || {
            let other = source.map_or(Ok(None), |s| s.other_copy(&file, &state));
            let _ = thread.queue(move |model| {
                let name = std::path::Path::new(&file.path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                model.telling(|model| match other {
                    Ok(Some(saved_from)) => model.ask(crate::game_page::other_copy_question(&saved_from, &name), Pending::Play(state)),
                    Ok(None) => model.played(QString::from(file.path.as_str()), QString::from(state.as_str())),
                    Err(words) => model.said(format!("The state could not be read: {words}")),
                });
            });
        });
    }

    /// What the viewer shows, read again from it: closed where it is none.
    fn viewed(mut self: Pin<&mut Self>) {
        // A property told can load the manual at once, and its page count comes back into the viewer while it is told:
        // told again until the viewer stands still.
        for _ in 0..4 {
            let before = self.viewer.clone();
            self.as_mut().tell_viewer();
            if self.viewer == before {
                break;
            }
        }
    }

    fn tell_viewer(mut self: Pin<&mut Self>) {
        let shown = self.viewer.as_ref().map(|v| {
            let (kind, source, view) = match v.reading {
                Some(view) => (2, v.manual.clone().unwrap_or_default(), view),
                None => {
                    let entry = v.shown();
                    let clip = entry.is_some_and(|e| e.kind == "video");
                    (i32::from(clip), entry.map(|e| e.source.clone()).unwrap_or_default(), Default::default())
                }
            };
            let help = match (v.reading, &v.manual) {
                (Some(_), _) => crate::help::Screen::GamePageManual,
                (None, Some(_)) => crate::help::Screen::GamePageViewerManual,
                (None, None) => crate::help::Screen::GamePageViewer,
            };
            (kind, cover_url(&source), cover_url(v.manual.as_deref().unwrap_or("")), self.page.caption(v), view, shown_screen(help))
        });
        let viewing = shown.is_some();
        let (kind, source, manual, caption, view, help) = shown.unwrap_or((0, String::new(), String::new(), String::new(), Default::default(), self.view_help));
        macro_rules! put {
            ($field:ident, $changed:ident, $value:expr) => {{
                let value = $value;
                if self.$field != value {
                    self.as_mut().rust_mut().$field = value;
                    self.as_mut().$changed();
                }
            }};
        }
        // Kept as it was when the viewer closes, so the manual's page going away is not handed an empty file.
        if viewing {
            put!(view_manual, view_manual_changed, QString::from(manual.as_str()));
        }
        put!(view_kind, view_kind_changed, kind);
        put!(view_source, view_source_changed, QString::from(source.as_str()));
        put!(view_caption, view_caption_changed, QString::from(caption.as_str()));
        put!(view_page, view_page_changed, view.page as i32);
        put!(view_zoom, view_zoom_changed, view.zoom as i32);
        put!(view_pan_x, view_pan_x_changed, view.pan.0);
        put!(view_pan_y, view_pan_y_changed, view.pan.1);
        put!(view_help, view_help_changed, help);
        put!(viewing, viewing_changed, viewing);
    }

    pub fn view_key(mut self: Pin<&mut Self>, key: i32, modifiers: i32) -> bool {
        use emusen_esde::scene::Key;
        use emusen_kit_core::input::UiButton;
        if self.viewer.is_none() {
            return false;
        }
        // ES-DE's media viewer's buttons (USERGUIDE.md, "Game media viewer"): the triggers to the ends, the shoulders zoom.
        let key = match emusen_kit_qt::qt_keys::press_of(key, modifiers) {
            Some(UiButton::Left) => Key::Left,
            Some(UiButton::Right) => Key::Right,
            Some(UiButton::Up) => Key::Up,
            Some(UiButton::Down) => Key::Down,
            Some(UiButton::First) => Key::First,
            Some(UiButton::Last) => Key::Last,
            Some(UiButton::PageUp) => Key::ZoomOut,
            Some(UiButton::PageDown) => Key::ZoomIn,
            _ => return false,
        };
        let moved = self.as_mut().rust_mut().viewer.as_mut().and_then(|v| v.press(key)) == Some(true);
        if moved {
            self.viewed();
        }
        true
    }

    pub fn view_pages(mut self: Pin<&mut Self>, pages: i32) {
        let changed = self.as_mut().rust_mut().viewer.as_mut().is_some_and(|v| v.set_pages(pages.max(0) as usize));
        if changed {
            self.viewed();
        }
    }

    /// Closed once the press that closed it has been delivered, so the page, given the focus back, does not take it too.
    pub fn view_close(self: Pin<&mut Self>) {
        if self.viewer.is_some() || self.viewing {
            let _ = self.qt_thread().queue(|mut model| {
                model.as_mut().rust_mut().viewer = None;
                model.as_mut().viewed();
                model.returned();
            });
        }
    }

    pub fn back(mut self: Pin<&mut Self>) -> bool {
        let went = self.as_mut().rust_mut().page.back();
        if went {
            self.as_mut().rust_mut().focus = (None, 0);
            self.shown(true);
        }
        went
    }

    pub fn typed(mut self: Pin<&mut Self>, kept: bool, words: &QString) {
        let field = self.as_mut().rust_mut().typing_for.take();
        // The keyboard closed once its press has been delivered, as the viewer is.
        if self.typing {
            let _ = self.qt_thread().queue(|mut model| {
                model.as_mut().rust_mut().typing = false;
                model.as_mut().typing_changed();
                model.returned();
            });
        }
        if let (true, Some(field)) = (kept, field) {
            let words = words.to_string();
            // Words left empty put back what was scraped, as the editor's Clear on one field does.
            let value = Some(words.trim().to_owned()).filter(|w| !w.is_empty());
            self.outcome(Outcome::Changed(crate::game_page::Change::Edit(field, value)));
        }
    }

    /// The page's scrape: the scraper opened with its store, the game queued, its progress followed off the GUI thread and
    /// told to the Scrape row, and the page read again once it ends, the scraper back to finding pictures alone.
    fn scrape(mut self: Pin<&mut Self>) {
        let Some(source) = self.source.clone() else { return };
        let (file, test) = (self.file.clone(), self.scraper_test.to_string());
        let test: Option<serde_json::Value> = serde_json::from_str(&test).ok();
        self.as_mut().ran(Some((true, "Starting".into())), "running");
        let thread = self.qt_thread();
        std::thread::spawn(move || {
            if let Err(why) = source.scrape_start(&file, test.as_ref()) {
                let _ = thread.queue(move |mut model| {
                    model.as_mut().ran(Some((false, "It could not start".into())), "idle");
                    model.said(format!("The scrape could not start: {why}"));
                });
                return;
            }
            let mut serial = 0;
            let ended = loop {
                serial = source.scrape_wait(serial, 250).unwrap_or(serial);
                let progress = match source.scrape_progress() {
                    Ok(p) => p,
                    Err(why) => break Err(why),
                };
                let (going, words) = crate::game_page::run_words(&progress, &file.path);
                let state = progress["state"].as_str().unwrap_or("idle").to_owned();
                let told = (going, words.clone(), state.clone());
                let _ = thread.queue(move |model| model.ran(Some((told.0, told.1)), &told.2));
                if !going {
                    break Ok(());
                }
            };
            let closed = source.scrape_end();
            let _ = thread.queue(move |mut model| {
                if let Some(why) = ended.err().or(closed.err()) {
                    model.as_mut().said(format!("The scrape could not be followed: {why}"));
                }
                model.refresh();
            });
        });
    }

    /// The page's scrape told: the Scrape row's words and the service's state.
    fn ran(mut self: Pin<&mut Self>, run: Option<(bool, String)>, state: &str) {
        self.as_mut().rust_mut().page.run = run;
        let state = QString::from(state);
        if self.scrape_state != state {
            self.as_mut().rust_mut().scrape_state = state;
            self.as_mut().scrape_state_changed();
        }
        self.shown(false);
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
            crate::help::RESET => {
                let outcome = self.page.reset(self.focus.1);
                self.outcome(outcome);
            }
            crate::help::DELETE => {
                let outcome = self.page.delete(self.focus.1);
                self.outcome(outcome);
            }
            _ => self.passed(key.clone()),
        }
    }

    pub fn answer(self: Pin<&mut Self>, index: i32) {
        self.telling(|mut model| {
            let pending = model.as_mut().rust_mut().pending.take();
            if model.asking {
                model.as_mut().rust_mut().asking = false;
                model.as_mut().asking_changed();
                model.as_mut().answered();
            }
            match (index, pending) {
                (1, Some(Pending::Change(change))) => model.outcome(Outcome::Changed(change)),
                (1, Some(Pending::Play(state))) => {
                    let path = QString::from(model.file.path.as_str());
                    model.played(path, QString::from(state.as_str()));
                }
                (1, Some(Pending::Stop)) => {
                    if let Some(source) = model.source.clone() {
                        if let Err(why) = source.scrape_cancel() {
                            model.as_mut().said(format!("The scrape could not be stopped: {why}"));
                        }
                    }
                }
                _ => {}
            }
        });
    }

    /// Told while the page tells QML of a change, the focus is kept and applied once the telling has finished (EmuSen_Qt.md §7.8.1).
    pub fn focus(mut self: Pin<&mut Self>, action: i32, content: i32) {
        self.as_mut().rust_mut().focus = (usize::try_from(action).ok(), content.max(0) as usize);
        if self.telling == 0 {
            self.helped();
        }
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
