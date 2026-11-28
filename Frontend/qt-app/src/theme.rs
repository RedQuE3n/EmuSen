//! `ThemeFeed`: the program's side of an EmuSen theme (EmuSen_Qt.md §3.5.2). It hands the theme's engine the views of
//! `EmuSen.Theme` from the program's models, carries the theme's `Actions` back to the window as signals, and keeps the
//! kit's per-engine singletons in step: reduced motion, and the pad's presses while a theme's screen holds the focus.
//! What it last showed is kept, so a new engine is fed it all at once.

use core::pin::Pin;
use cxx_qt::casting::Upcast;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QList, QString, QStringList};
use emusen_kit_qt::kinds::Form;
use emusen_qt_models::collections_eclipse::qobject::CollectionsEclipseModel;
use emusen_qt_models::desktop::qobject::DesktopModel;
use emusen_qt_models::game_model::qobject::GameModel;
use emusen_qt_models::game_page_model::qobject::GamePageModel;
use emusen_qt_models::library_model::qobject::{LibraryModel, QAbstractListModel};
use emusen_qt_models::session_model::qobject::SessionModel;
use emusen_qt_models::settings_model::qobject::SettingsModel;
use emusen_qt_models::themes_model::qobject::ThemesModel;
use emusen_qt_models::tools_hub::qobject::ToolsHubModel;
use emusen_qt_models::{help, list_card, screens, themes, words};
use emusen_theme_host::host::qobject::Role;
use emusen_theme_qt::actions::{Action, ActionSink, ScreenAction, SessionAction, ThemesAction, ToolsAction, WorkspaceAction};
use emusen_theme_qt::browse::facts_of;
use emusen_theme_qt::device::DeviceFacts;
use emusen_theme_qt::host;
use emusen_theme_qt::page::{action_rows, board_row_rows, card_rows, content_rows, media_tile_rows, pill_rows, PageFacts};
use emusen_theme_qt::session::SessionFacts;
use emusen_theme_qt::{collections, session as theme_session, themes as theme_themes, tools as theme_tools, view, workspace};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc::Sender;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("emusen-theme-host/src/host.cxxqt.h");
        type ThemeHost = emusen_theme_host::host::qobject::ThemeHost;
        include!("emusen-theme-qt/src/session.cxxqt.h");
        type QQuickItem = emusen_theme_qt::session::qobject::QQuickItem;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        /// The host whose engine is fed; the window's `ThemeHost`.
        #[qproperty(*mut ThemeHost, host)]
        /// The pads connected, their glyphs' family, and the time as Home's status line says it: `Device`.
        #[qproperty(i32, controllers)]
        #[qproperty(i32, family)]
        #[qproperty(QString, clock)]
        /// The time in the machine's own short form: `Device.shortTime`.
        #[qproperty(QString, short_time, cxx_name = "shortTime")]
        /// The machine's battery, 0 to 100 or -1 for none, and whether it charges: `Device.battery` and `charging`.
        #[qproperty(i32, battery)]
        #[qproperty(bool, charging)]
        /// Reduced motion, as the window binds the kit's tokens.
        #[qproperty(bool, reduced_motion, cxx_name = "reducedMotion")]
        /// Swap A and B in menus, as the window binds its own PadRouter, for the engine's (EmuSen_Qt.md §7.7.H3).
        #[qproperty(bool, swap)]
        /// The program's clock's time, in seconds since 1970, 0 for the wall clock: the theme's `Words.now`.
        #[qproperty(i64, now)]
        /// Whether the window shows Settings and the game's page, so a theme's calls are taken only on the screen shown.
        #[qproperty(bool, settings_shown, cxx_name = "settingsShown")]
        #[qproperty(bool, page_shown, cxx_name = "pageShown")]
        /// The key of the program's section shown (`Screens.section`): the theme's `Library.section`.
        #[qproperty(QString, shown_section, cxx_name = "shownSection")]
        /// A splash that enters Home shown, and its time in ms: the theme's `Splash` (1.10).
        #[qproperty(bool, splash_shown, cxx_name = "splashShown")]
        #[qproperty(f64, splash_time, cxx_name = "splashTime")]
        /// The hand-over's length in ms after Home whole, the theme's `Splash.ease` (1.10).
        #[qproperty(f64, splash_ease, cxx_name = "splashEase")]
        #[qproperty(bool, big_screen, cxx_name = "bigScreen")]
        type ThemeFeed = super::ThemeFeedRust;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        /// Where the reference theme's package lives in the program's resources.
        #[qproperty(QString, reference, READ, CONSTANT)]
        /// Each built-in theme's `theme.json`, in `themes::BUILT_IN`'s order, for the Themes tab's rows.
        #[qproperty(QStringList, manifests, READ, CONSTANT)]
        type BuiltInThemes = super::BuiltInThemesRust;
    }

    extern "RustQt" {
        /// Merges updated settings cover stores into `current` without disturbing unrelated stores.
        #[qinvokable]
        #[cxx_name = "refreshCoverStores"]
        fn refresh_cover_stores(self: &BuiltInThemes, library: &QString, current: &QStringList) -> QStringList;
    }

    unsafe extern "RustQt" {
        /// The library's view and Home's lines, after each read of the library: the window's `LibraryModel`.
        #[qinvokable]
        #[cxx_name = "showLibrary"]
        unsafe fn show_library(self: Pin<&mut ThemeFeed>, library: *mut QObject);

        /// The suspended game: its file, whether it runs, when it was paused (s since 1970, 0 for not), and its picture.
        #[qinvokable]
        #[cxx_name = "showSession"]
        unsafe fn show_session(self: Pin<&mut ThemeFeed>, game: &QString, running: bool, paused_at: f64, frame: *mut QQuickItem);

        /// The section the Games screen shows as the window's grid model holds it (the window's `GameModel`), its key,
        /// and whether the search, its keyboard and the List card are open.
        #[qinvokable]
        #[cxx_name = "showBrowse"]
        unsafe fn show_browse(self: Pin<&mut ThemeFeed>, games: *mut QObject, section: &QString, searching: bool, typing: bool, list_open: bool);

        /// Settings as the window's SettingsModel holds it, for a theme that draws Settings (EmuSen_Qt.md §3.2, `Settings`).
        #[qinvokable]
        #[cxx_name = "showSettings"]
        unsafe fn show_settings(self: Pin<&mut ThemeFeed>, settings: *mut QObject);

        /// The game's page as the window's GamePageModel holds it, for a theme that draws the page (§3.2, `Page`).
        #[qinvokable]
        #[cxx_name = "showPage"]
        unsafe fn show_page(self: Pin<&mut ThemeFeed>, page: *mut QObject);

        /// Desktop Workspace, Themes, Tools, Collections, and live Session feeds (1.23).
        #[qinvokable]
        #[cxx_name = "showDesktop"]
        unsafe fn show_desktop(self: Pin<&mut ThemeFeed>, desktop: *mut QObject);
        #[qinvokable]
        #[cxx_name = "showThemes"]
        unsafe fn show_themes(self: Pin<&mut ThemeFeed>, themes: *mut QObject);
        #[qinvokable]
        #[cxx_name = "showTools"]
        unsafe fn show_tools(self: Pin<&mut ThemeFeed>, tools: *mut QObject);
        #[qinvokable]
        #[cxx_name = "showCollections"]
        unsafe fn show_collections(self: Pin<&mut ThemeFeed>, collections: *mut QObject);
        #[qinvokable]
        #[cxx_name = "showSessionLive"]
        unsafe fn show_session_live(self: Pin<&mut ThemeFeed>, session: *mut QObject);

        /// The host's engine as it is now: its `Actions` carried by this feed, and everything shown so far shown again.
        #[qinvokable]
        fn attach(self: Pin<&mut ThemeFeed>);

        /// A pad's button down or up, for the theme's engine's router.
        #[qinvokable]
        fn push(self: Pin<&mut ThemeFeed>, button: i32, down: bool);

        /// The theme's `Actions`, each as the window carries it out: a game played by its file.
        #[qsignal]
        fn played(self: Pin<&mut ThemeFeed>, game: QString);
        /// The suspended game chosen: it resumes.
        #[qsignal]
        fn resumed(self: Pin<&mut ThemeFeed>);
        /// One of Home's places, by its index in Home's order.
        #[qsignal]
        #[cxx_name = "placeChosen"]
        fn place_chosen(self: Pin<&mut ThemeFeed>, index: i32);
        /// A console's or a shelf's grid, by its key.
        #[qsignal]
        fn opened(self: Pin<&mut ThemeFeed>, key: QString);
        /// One of the program's sections, by its key.
        #[qsignal]
        #[cxx_name = "sectionOpened"]
        fn section_opened(self: Pin<&mut ThemeFeed>, key: QString);
        /// B that the theme's screen did not keep.
        #[qsignal]
        fn backed(self: Pin<&mut ThemeFeed>);
        /// A screen's declared action, by the help bar's key (`emusen_qt_models::help`).
        #[qsignal]
        fn acted(self: Pin<&mut ThemeFeed>, key: QString);
        /// A step or a choice of the List card's, by its action's key (`emusen_qt_models::list_card`), for the grid to carry out.
        #[qsignal]
        fn listed(self: Pin<&mut ThemeFeed>, key: QString);
        /// The game the theme's focus is on, by its file.
        #[qsignal]
        fn focused(self: Pin<&mut ThemeFeed>, game: QString);
        /// The row of Choose a console the theme's focus is on, by its index in the library's rows.
        #[qsignal]
        fn selected(self: Pin<&mut ThemeFeed>, index: i32);
        /// A game's favourite turned, by its file.
        #[qsignal]
        fn favourite(self: Pin<&mut ThemeFeed>, game: QString);
        #[qsignal]
        #[cxx_name = "bigScreenRequested"]
        fn big_screen_requested(self: Pin<&mut ThemeFeed>, big_screen: bool);
        #[qsignal]
        #[cxx_name = "desktopDoorRequested"]
        fn desktop_door_requested(self: Pin<&mut ThemeFeed>, key: QString);
        #[qsignal]
        #[cxx_name = "toolRequested"]
        fn tool_requested(self: Pin<&mut ThemeFeed>, key: QString);
        #[qsignal]
        #[cxx_name = "fullScreenToggleRequested"]
        fn full_screen_toggle_requested(self: Pin<&mut ThemeFeed>);
        #[qsignal]
        #[cxx_name = "desktopLibraryRequested"]
        fn desktop_library_requested(self: Pin<&mut ThemeFeed>);
        #[qsignal]
        #[cxx_name = "desktopShadersRequested"]
        fn desktop_shaders_requested(self: Pin<&mut ThemeFeed>);
    }

    impl cxx_qt::Initialize for ThemeFeed {}
    impl cxx_qt::Threading for ThemeFeed {}
}

/// The built-in themes' places in the resources (`emusen_qt_models::themes::BUILT_IN`) and their manifests.
pub struct BuiltInThemesRust {
    reference: QString,
    manifests: QStringList,
}

impl Default for BuiltInThemesRust {
    fn default() -> Self {
        let manifests: QList<QString> = themes::BUILT_IN
            .iter()
            .map(|id| {
                let mut manifest = QString::default();
                let file = QString::from(format!("{}theme.json", themes::built_in_folder(id)).as_str());
                // Read through Qt, as the host reads any package, since the folder is in the resources.
                emusen_theme_host::sandbox::ffi::sandbox_read(&file, &mut manifest);
                manifest
            })
            .collect();
        BuiltInThemesRust {
            reference: QString::from(crate::reference_theme().as_str()),
            manifests: QStringList::from(&manifests),
        }
    }
}

impl qobject::BuiltInThemes {
    pub fn refresh_cover_stores(&self, library: &QString, current: &QStringList) -> QStringList {
        let lib = library.to_string();
        if lib.is_empty() {
            return current.clone();
        }
        let cur: Vec<String> = current.iter().map(|s| s.to_string()).collect();
        let Some(fresh) = crate::start::cover_stores_update(std::path::Path::new(&lib), &cur) else {
            return current.clone();
        };
        let mut out = QList::<QString>::default();
        fresh.iter().for_each(|s| out.append(QString::from(s.as_str())));
        QStringList::from(&out)
    }
}

/// The suspended game as the window last showed it.
#[derive(Clone)]
struct Shown {
    game: String,
    running: bool,
    paused_at: f64,
    frame: *mut qobject::QQuickItem,
}

/// The section as the window last showed it.
#[derive(Clone)]
struct Section {
    games: *const GameModel,
    key: String,
    searching: bool,
    typing: bool,
    list_open: bool,
}

struct FactsRequest {
    source: std::sync::Arc<dyn emusen_qt_models::library::LibrarySource>,
    game: emusen_platform_client::library::GameFile,
    seconds: i64,
    qt: cxx_qt::CxxQtThread<qobject::ThemeFeed>,
}

/// The feed's state: the window's host and models, and what it last showed.
pub struct ThemeFeedRust {
    host: *mut qobject::ThemeHost,
    library: *const LibraryModel,
    /// The window's Settings and page models, which a theme's screens read and act through.
    settings: *mut SettingsModel,
    page: *mut GamePageModel,
    desktop: *mut DesktopModel,
    themes_model: *mut ThemesModel,
    tools_hub: *mut ToolsHubModel,
    collections_model: *mut CollectionsEclipseModel,
    session_model: *mut SessionModel,
    /// A feed of Settings or of the page queued and not yet run, so the changes of one press feed once.
    settings_queued: bool,
    page_queued: bool,
    desktop_queued: bool,
    themes_queued: bool,
    tools_queued: bool,
    collections_queued: bool,
    session_live_queued: bool,
    pictures_served_gen: Cell<i32>,
    session: Option<Shown>,
    section: Option<Section>,
    settings_shown: bool,
    page_shown: bool,
    controllers: i32,
    family: i32,
    clock: QString,
    short_time: QString,
    battery: i32,
    charging: bool,
    reduced_motion: bool,
    swap: bool,
    now: i64,
    /// The game whose facts were last asked for, so a read that comes back for another is dropped.
    facts_for: RefCell<String>,
    facts_worker: RefCell<Option<(Sender<FactsRequest>, std::thread::JoinHandle<()>)>>,
    library_gen: Cell<u64>,
    browse_key: RefCell<Option<(i32, u64, i32, String, i64)>>,
    shown_section: QString,
    splash_shown: bool,
    splash_time: f64,
    splash_ease: f64,
    big_screen: bool,
    /// The engine's `Splash` and the generation it was found in, looked up again only in a new engine: fed every frame, a name's lookup allocates.
    splash: Cell<(i32, *mut emusen_theme_qt::splash::qobject::ThemeSplash)>,
    /// The feed as the sink sees it, emptied when the feed goes, so a sink that outlives it takes nothing.
    me: Rc<Cell<*mut qobject::ThemeFeed>>,
}

impl Default for ThemeFeedRust {
    fn default() -> Self {
        ThemeFeedRust {
            host: std::ptr::null_mut(),
            library: std::ptr::null(),
            settings: std::ptr::null_mut(),
            page: std::ptr::null_mut(),
            desktop: std::ptr::null_mut(),
            themes_model: std::ptr::null_mut(),
            tools_hub: std::ptr::null_mut(),
            collections_model: std::ptr::null_mut(),
            session_model: std::ptr::null_mut(),
            settings_queued: false,
            page_queued: false,
            desktop_queued: false,
            themes_queued: false,
            tools_queued: false,
            collections_queued: false,
            session_live_queued: false,
            pictures_served_gen: Cell::new(-1),
            session: None,
            section: None,
            settings_shown: false,
            page_shown: false,
            controllers: 0,
            family: 0,
            clock: QString::default(),
            short_time: QString::default(),
            battery: -1,
            charging: false,
            reduced_motion: false,
            swap: false,
            now: 0,
            facts_for: RefCell::new(String::new()),
            facts_worker: RefCell::new(None),
            library_gen: Cell::new(0),
            browse_key: RefCell::new(None),
            shown_section: QString::default(),
            splash_shown: false,
            splash_time: 0.0,
            splash_ease: 0.0,
            big_screen: true,
            splash: Cell::new((-1, std::ptr::null_mut())),
            me: Rc::new(Cell::new(std::ptr::null_mut())),
        }
    }
}

impl Drop for ThemeFeedRust {
    fn drop(&mut self) {
        self.me.set(std::ptr::null_mut());
        if let Some((tx, handle)) = self.facts_worker.borrow_mut().take() {
            drop(tx);
            let _ = handle.join();
        }
    }
}

/// A model the window hands as a QObject, as its own type: a list model's QObject base sits at its address.
///
/// # Safety
/// `object` is a `T` or null.
unsafe fn model<T: Upcast<QAbstractListModel>>(object: *mut cxx_qt::QObject) -> *const T {
    unsafe { T::from_base_ptr(object as *const QAbstractListModel) }
}

unsafe fn qobj_model<T: Upcast<cxx_qt::QObject>>(object: *mut cxx_qt::QObject) -> *mut T {
    unsafe { T::from_base_ptr(object as *const cxx_qt::QObject) as *mut T }
}

/// What the window does for an action: each of the theme's `Actions` as one of the feed's signals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Carried {
    Played(String),
    Resumed,
    Place(i32),
    Opened(String),
    Section(String),
    Backed,
    Acted(String),
    Focused(String),
    Selected(i32),
    /// A game's page: the game taken in hand, then its page opened (EmuSen_Qt.md §7.8).
    Page(String),
    /// The List card's filter, sort or letter step, by its action's key (EmuSen_Qt.md §3.4.10).
    Listed(String),
    /// A game's favourite turned, by its file; empty for the game in hand, as the help's Y names no game.
    Favourite(String),
}

/// The signal an action becomes, `suspended` being the game a session holds and `rows` the library's rows' keys; none
/// for one the program has no answer to yet (the Home card opened by a theme), which is declined.
pub fn carried(action: &Action, suspended: Option<&str>, rows: &[&str]) -> Option<Carried> {
    Some(match action {
        Action::Play(key) if suspended == Some(key.as_str()) => Carried::Resumed,
        Action::Play(key) => Carried::Played(key.clone()),
        Action::OpenPage(key) => Carried::Page(key.clone()),
        Action::Favourite(key) => Carried::Favourite(key.clone()),
        Action::Random(_) => Carried::Acted(help::RANDOM.into()),
        Action::OpenConsole(key) => Carried::Opened(key.clone()),
        Action::ConsoleOptions(key) => Carried::Acted(format!("{}:{key}", help::CONSOLE_OPTIONS)),
        Action::OpenPlace(key) => match view::PLACE_IDS.iter().position(|id| id == key) {
            Some(i) => Carried::Place(i as i32),
            None => Carried::Opened(key.clone()),
        },
        Action::OpenSection(key) => Carried::Section(key.clone()),
        Action::Home => Carried::Acted(help::HOME.into()),
        Action::Back => Carried::Backed,
        Action::StepSection(by) => Carried::Acted(if *by < 0 { help::PREVIOUS } else { help::NEXT }.into()),
        Action::OpenHomeCard => return None,
        Action::OpenListCard => Carried::Acted(screens::LIST.into()),
        Action::Search => Carried::Acted(screens::SEARCH.into()),
        Action::Act(key) if key == list_card::JUMP_BACK || key == list_card::JUMP_ON => Carried::Listed(key.clone()),
        Action::Act(key) if key == help::FAVOURITE => Carried::Favourite(String::new()),
        Action::Act(key) => Carried::Acted(key.clone()),
        Action::SetFilter(key) => Carried::Listed(key.clone()),
        Action::StepSort(by) => Carried::Listed(format!("{}{by:+}", list_card::SORT)),
        Action::StepLetter(by) => Carried::Listed(format!("{}{by:+}", list_card::JUMP)),
        Action::Focus(key) => match rows.iter().position(|r| r == key) {
            Some(i) => Carried::Selected(i as i32),
            None => Carried::Focused(key.clone()),
        },
    })
}

/// The theme's `Actions` as the window's signals.
struct Sink {
    feed: Rc<Cell<*mut qobject::ThemeFeed>>,
    /// The game the theme's focus is on, which the help's Y turns a favourite.
    in_hand: String,
    /// The Settings row to return to when leaving a page's aside chips (Scraping, R-20261116-22).
    chip_return: i32,
}

impl ActionSink for Sink {
    fn act(&mut self, action: &Action) -> bool {
        // Safety: the cell holds the feed while it lives and null after; the theme's engine runs on the feed's thread.
        let Some(feed) = (unsafe { self.feed.get().as_mut() }) else { return false };
        // X back to default, Hotkeys X clear, Themes X check, Consoles Y filter, and LT/RT letter jump on a theme's Settings (R-20261116-22, -24).
        if matches!(action, Action::Act(key) if matches!(key.as_str(), help::DEFAULT | help::CLEAR | help::CHECK | help::CONSOLES_FILTER | help::CONSOLES_LETTER_BACK | help::CONSOLES_LETTER_ON)) {
            let settings = feed.settings_taken().then(|| unsafe { feed.settings.as_mut() }).flatten().map(|m| unsafe { Pin::new_unchecked(m) });
            return settings.is_some_and(|s| match action {
                Action::Act(key) if key == help::DEFAULT => {
                    let row = (*s.focused()).max(s.view().firmware_need_count() as i32);
                    s.to_default(row)
                }
                Action::Act(key) if key == help::CLEAR => {
                    s.act(&QString::from(help::CLEAR));
                    true
                }
                Action::Act(key) if key == help::CHECK => {
                    s.act(&QString::from(help::CHECK));
                    true
                }
                Action::Act(key) if key == help::CONSOLES_FILTER => s.toggle_consoles_with_games(),
                Action::Act(key) if key == help::CONSOLES_LETTER_BACK => s.jump_console_letter(-1),
                Action::Act(key) if key == help::CONSOLES_LETTER_ON => s.jump_console_letter(1),
                _ => false,
            });
        }
        let mut feed = unsafe { Pin::new_unchecked(feed) };
        let suspended = feed.session.as_ref().filter(|s| s.running).map(|s| s.game.clone());
        let rows: Vec<String> = feed.library().map(|l| l.view().rows.rows().iter().map(|r| r.key.clone()).collect()).unwrap_or_default();
        let keys: Vec<&str> = rows.iter().map(String::as_str).collect();
        let Some(carried) = carried(action, suspended.as_deref(), &keys) else { return false };
        match carried {
            Carried::Played(game) => feed.as_mut().played(QString::from(game.as_str())),
            Carried::Resumed => feed.as_mut().resumed(),
            Carried::Place(index) => feed.as_mut().place_chosen(index),
            Carried::Opened(key) => feed.as_mut().opened(QString::from(key.as_str())),
            Carried::Section(key) => feed.as_mut().section_opened(QString::from(key.as_str())),
            Carried::Backed => feed.as_mut().backed(),
            Carried::Acted(key) => feed.as_mut().acted(QString::from(key.as_str())),
            Carried::Focused(game) => {
                feed.as_mut().focused(QString::from(game.as_str()));
                self.in_hand = game;
            }
            Carried::Selected(index) => feed.as_mut().selected(index),
            Carried::Listed(key) => feed.as_mut().listed(QString::from(key.as_str())),
            Carried::Page(game) => {
                feed.as_mut().focused(QString::from(game.as_str()));
                feed.as_mut().acted(QString::from(help::GAME_PAGE));
                self.in_hand = game;
            }
            Carried::Favourite(game) => {
                let game = if game.is_empty() { self.in_hand.clone() } else { game };
                if game.is_empty() {
                    return false;
                }
                feed.as_mut().favourite(QString::from(game.as_str()));
            }
        }
        true
    }

    fn would(&self, action: &Action) -> bool {
        !self.feed.get().is_null() && carried(action, None, &[]).is_some()
    }

    fn screen(&mut self, action: &ScreenAction) -> bool {
        // Safety: as `act`; the models are the window's, alive as long as the feed, and pinned where QML made them.
        let Some(feed) = (unsafe { self.feed.get().as_mut() }) else { return false };
        let settings = feed.settings_taken().then(|| unsafe { feed.settings.as_mut() }).flatten().map(|m| unsafe { Pin::new_unchecked(m) });
        let page = feed.page_taken().then(|| unsafe { feed.page.as_mut() }).flatten().map(|m| unsafe { Pin::new_unchecked(m) });
        let within = |at: i32, count: usize| usize::try_from(at).is_ok_and(|at| at < count);
        match (action, settings, page) {
            (ScreenAction::SettingsStep(row, by), Some(s), _) => {
                let target = *row + s.view().firmware_need_count() as i32;
                if let Some(start) = s.view().chip_start().filter(|&st| target >= 0 && (target as usize) < st) {
                    if s.view().facets().get(target as usize).is_some_and(|f| f.switch.is_some()) {
                        if *by > 0 {
                            self.chip_return = target;
                            s.set_focused(start as i32);
                            return true;
                        }
                        return false;
                    }
                }
                s.step(target, *by)
            }
            (ScreenAction::SettingsChoose(row), Some(s), _) => {
                let target = *row + s.view().firmware_need_count() as i32;
                s.choose(target)
            }
            (ScreenAction::SettingsSlide(row, amount), Some(s), _) => {
                let target = *row + s.view().firmware_need_count() as i32;
                s.slide(target, *amount)
            }
            (ScreenAction::SettingsSection(section), Some(s), _) => {
                let took = within(*section, s.sections().iter().count());
                if took {
                    self.chip_return = 0;
                    if !feed.big_screen {
                        if let Some(desktop) = unsafe { feed.desktop.as_mut() } {
                            unsafe { Pin::new_unchecked(desktop) }.set_settings_tool(-1);
                        }
                    }
                    s.set_section(*section);
                }
                took
            }
            (ScreenAction::SettingsFocus(row), Some(s), _) => {
                let target = *row + s.view().firmware_need_count() as i32;
                let took = *row >= 0 && within(target, s.shown_rows().len());
                if took {
                    if s.view().chip_start().is_some_and(|st| (target as usize) < st) {
                        self.chip_return = target;
                    }
                    s.set_focused(target);
                }
                took
            }
            (ScreenAction::SettingsShoulder(by), Some(s), _) => s.shoulder(*by),
            (ScreenAction::SettingsInlinePill(index), Some(s), _) => s.choose_inline_pill(*index),
            (ScreenAction::SettingsBack, Some(s), _) => {
                if let Some(start) = s.view().chip_start().filter(|&st| *s.focused() >= st as i32) {
                    s.set_focused(self.chip_return.clamp(0, start.saturating_sub(1) as i32));
                    return true;
                }
                self.chip_return = 0;
                if !feed.big_screen {
                    if let Some(desktop) = unsafe { feed.desktop.as_mut() } {
                        if *desktop.settings_tool_index() >= 0 {
                            unsafe { Pin::new_unchecked(desktop) }.settings_back();
                            return true;
                        }
                    }
                }
                s.back()
            }
            (ScreenAction::PageSection(section), _, Some(p)) => {
                let took = within(*section, p.sections().iter().count());
                if took {
                    p.set_section(*section);
                }
                took
            }
            (ScreenAction::PageAct(action), _, Some(p)) => p.act(*action),
            (ScreenAction::PageStepAction(action, by), _, Some(p)) => p.step_action(*action, *by),
            (ScreenAction::PageChoose(row), _, Some(p)) => p.choose(*row),
            (ScreenAction::PageStep(row, by), _, Some(p)) => p.step(*row, *by),
            (ScreenAction::PageFocus(action, row), _, Some(p)) => {
                let actions = p.shown_actions().len();
                let count = p.shown_items().len().max(p.board_rows().len());
                let took = (*action == -1 || *action == emusen_qt_models::game_page::RESUME_ACTION || within(*action, actions)) && (*row == 0 || within(*row, count));
                if took {
                    p.focus(*action, *row);
                }
                took
            }
            (ScreenAction::PageDeclared(key), _, Some(p)) => p.declared(&QString::from(key.as_str())),
            (ScreenAction::PageAnswer(choice), _, Some(p)) => p.answer(*choice),
            (ScreenAction::PageBack, _, Some(p)) => p.back(),
            (ScreenAction::PageTab(tab), _, Some(p)) => p.choose_tab(*tab),
            (ScreenAction::PageViewKey(key, modifiers), _, Some(p)) => p.view_key(*key, *modifiers),
            (ScreenAction::PageViewPages(pages), _, Some(p)) => {
                p.view_pages(*pages);
                true
            }
            (ScreenAction::SettingsSetFilter(filter), Some(_), _) => {
                let Some(d) = (unsafe { feed.desktop.as_mut() }) else { return false };
                unsafe { Pin::new_unchecked(d) }.set_settings_filter(QString::from(filter.as_str()));
                true
            }
            (ScreenAction::SettingsSetModifiedOnly(on), Some(_), _) => {
                let Some(d) = (unsafe { feed.desktop.as_mut() }) else { return false };
                unsafe { Pin::new_unchecked(d) }.set_settings_modified_only(*on);
                true
            }
            (ScreenAction::SettingsOpenTool(index), Some(_), _) => {
                let Some(d) = (unsafe { feed.desktop.as_mut() }) else { return false };
                unsafe { Pin::new_unchecked(d) }.open_settings_tool(*index);
                true
            }
            _ => false,
        }
    }

    fn set_big_screen(&mut self, big_screen: bool) -> bool {
        let Some(feed) = (unsafe { self.feed.get().as_mut() }) else { return false };
        let mut feed = unsafe { Pin::new_unchecked(feed) };
        feed.as_mut().big_screen_requested(big_screen);
        true
    }

    fn workspace(&mut self, action: &WorkspaceAction) -> bool {
        let Some(feed) = (unsafe { self.feed.get().as_mut() }) else { return false };
        if feed.big_screen || !feed.draws_any_desktop() {
            return false;
        }
        let Some(d) = (unsafe { feed.desktop.as_mut() }) else { return false };
        let mut desktop = unsafe { Pin::new_unchecked(d) };
        match action {
            WorkspaceAction::View(v) => {
                desktop.set_view(*v);
                true
            }
            WorkspaceAction::SetSidebar(open) => {
                desktop.set_sidebar_open(*open);
                true
            }
            WorkspaceAction::SetInspector(open) => {
                desktop.set_inspector_open(*open);
                true
            }
            WorkspaceAction::SetDrawer(open) => {
                desktop.set_drawer_open(*open);
                true
            }
            WorkspaceAction::SetDrawerTab(tab) => {
                desktop.set_drawer_tab(*tab);
                true
            }
            WorkspaceAction::SelectConsole(key) => {
                desktop.set_console_key(QString::from(key.as_str()));
                true
            }
            WorkspaceAction::SetConsoleFilter(filter) => {
                desktop.set_console_filter(QString::from(filter.as_str()));
                true
            }
            WorkspaceAction::SetConsoleScope(scope) => {
                desktop.set_console_scope(workspace::from_workspace_scope(*scope));
                true
            }
            WorkspaceAction::SetFilter(key) => {
                desktop.set_filter_key(QString::from(key.as_str()));
                true
            }
            WorkspaceAction::SetSearch(query) => {
                desktop.set_search_query(QString::from(query.as_str()));
                true
            }
            WorkspaceAction::SetSort(index) => {
                desktop.set_sort_index(*index);
                true
            }
            WorkspaceAction::SortByColumn(col) => {
                desktop.sort_by_column(*col);
                true
            }
            WorkspaceAction::SelectGame(key) => {
                desktop.select_game(&QString::from(key.as_str()));
                true
            }
            WorkspaceAction::MarkToggle(key) => {
                desktop.mark_toggle(&QString::from(key.as_str()));
                true
            }
            WorkspaceAction::MarkAll => {
                desktop.mark_all();
                true
            }
            WorkspaceAction::ClearMarked => {
                desktop.clear_marked();
                true
            }
            WorkspaceAction::BatchFavourite(on) => {
                desktop.batch_favourite(*on);
                true
            }
            WorkspaceAction::ClearFilters => {
                desktop.clear_filters();
                true
            }
            WorkspaceAction::OpenFolder(key) => desktop.open_folder_for(key),
            WorkspaceAction::PlayFrom(game, state) => desktop.launch_from(game, state),
            WorkspaceAction::OpenDesktopDoor(key) => {
                if desktop.open_door(key) {
                    true
                } else if key == "shaders" || key == "cards" {
                    false
                } else {
                    unsafe { Pin::new_unchecked(feed) }.desktop_door_requested(QString::from(key.as_str()));
                    true
                }
            }
            WorkspaceAction::OpenTool(key) => open_desktop_tool(feed, key),
            WorkspaceAction::ToggleFullScreen => {
                unsafe { Pin::new_unchecked(feed) }.full_screen_toggle_requested();
                true
            }
            WorkspaceAction::AddGamesToCollection(games, cid) => desktop.add_games_to_collection(games.clone(), cid),
            WorkspaceAction::OpenLibrary => {
                desktop.as_mut().set_drawer_open(false);
                desktop.set_view(0);
                unsafe { Pin::new_unchecked(feed) }.desktop_library_requested();
                true
            }
            WorkspaceAction::OpenShaders => {
                desktop.as_mut().set_drawer_open(false);
                if desktop.open_door("shaders") {
                    unsafe { Pin::new_unchecked(feed) }.desktop_shaders_requested();
                    true
                } else {
                    false
                }
            }
        }
    }

    fn session(&mut self, action: &SessionAction) -> bool {
        let Some(feed) = (unsafe { self.feed.get().as_mut() }) else { return false };
        let allowed = !feed.big_screen
            && (feed.draws(Role::DesktopPlaying)
                || (matches!(action, SessionAction::AnswerResume(_)) && (feed.draws(Role::DesktopLibrary) || feed.draws(Role::DesktopTable))));
        if !allowed {
            return false;
        }
        let Some(sm) = (unsafe { feed.session_model.as_mut() }) else { return false };
        let mut s = unsafe { Pin::new_unchecked(sm) };
        match action {
            SessionAction::Pause => {
                s.as_mut().pause();
                true
            }
            SessionAction::Resume => {
                s.as_mut().resume();
                true
            }
            SessionAction::Restart => {
                s.as_mut().restart();
                true
            }
            SessionAction::Close => {
                s.as_mut().close();
                true
            }
            SessionAction::Screenshot => {
                s.as_mut().screenshot();
                true
            }
            SessionAction::ChooseSlot(slot) => {
                s.as_mut().choose_slot((*slot).clamp(0, 99) as u32);
                true
            }
            SessionAction::SaveState(slot) => {
                s.as_mut().save_state((*slot).clamp(0, 99) as u32);
                true
            }
            SessionAction::LoadState(slot) => {
                s.as_mut().load_state((*slot).clamp(0, 99) as u32);
                true
            }
            SessionAction::ChooseSpeed(percent) => {
                s.as_mut().choose_speed((*percent).max(0) as u32);
                true
            }
            SessionAction::RewindTo(frame) => {
                s.as_mut().rewind_to((*frame).max(0.0) as u64);
                true
            }
            SessionAction::SetMasterCheats(on) => {
                s.as_mut().set_master_cheats(*on);
                true
            }
            SessionAction::SwitchCheat(index) => {
                let current_on = unsafe { feed.desktop.as_ref() }
                    .and_then(|d| d.playing_data().cheats.into_iter().find(|c| c.index == *index))
                    .is_some_and(|c| c.on);
                s.as_mut().switch_cheat(*index, !current_on);
                true
            }
            SessionAction::StepConsoleOption(key, by) => {
                let Some(opt) = unsafe { feed.desktop.as_ref() }
                    .and_then(|d| d.playing_data().console_options.into_iter().find(|o| o.key == *key && !o.choices.is_empty()))
                else {
                    return false;
                };
                let cur = opt.choices.iter().position(|c| c == &opt.value).unwrap_or(0) as isize;
                let next_idx = (cur + *by as isize).rem_euclid(opt.choices.len() as isize) as usize;
                let next_val = &opt.choices[next_idx];
                s.as_mut().set_console_setting(&QString::from(key.as_str()), &QString::from(next_val.as_str()));
                true
            }
            SessionAction::SwapPlayers => {
                s.as_mut().swap_players();
                true
            }
            SessionAction::AnswerResume(resume) => {
                s.as_mut().answer_resume(if *resume { 0 } else { 1 });
                true
            }
        }
    }

    fn themes(&mut self, action: &ThemesAction) -> bool {
        let Some(feed) = (unsafe { self.feed.get().as_mut() }) else { return false };
        if feed.big_screen || !feed.draws(Role::DesktopSettings) {
            return false;
        }
        let Some(tm) = (unsafe { feed.themes_model.as_mut() }) else { return false };
        let mut t = unsafe { Pin::new_unchecked(tm) };
        match action {
            ThemesAction::SetKind(kind) => {
                t.as_mut().set_kind(QString::from(kind.as_str()));
                true
            }
            ThemesAction::Focus(row) => {
                t.as_mut().set_focused(*row);
                true
            }
            ThemesAction::Use(id) => {
                t.as_mut().use_theme(&QString::from(id.as_str()));
                true
            }
            ThemesAction::Step(row, by) => {
                t.as_mut().step(*row, *by);
                true
            }
            ThemesAction::Check(id) => {
                t.as_mut().check(&QString::from(id.as_str()));
                true
            }
        }
    }

    fn tools(&mut self, action: &ToolsAction) -> bool {
        let Some(feed) = (unsafe { self.feed.get().as_mut() }) else { return false };
        if feed.big_screen || (!feed.draws(Role::DesktopSettings) && !feed.draws(Role::DesktopLibrary)) {
            return false;
        }
        let Some(tm) = (unsafe { feed.tools_hub.as_mut() }) else { return false };
        let t = unsafe { Pin::new_unchecked(tm) };
        match action {
            ToolsAction::Focus(index) => {
                t.set_focused(*index);
                true
            }
            ToolsAction::Open(key) => open_desktop_tool(feed, key),
        }
    }
}

fn open_desktop_tool(feed: &mut qobject::ThemeFeed, key: &str) -> bool {
    if key == "cards" {
        return unsafe { feed.desktop.as_mut() }.is_some_and(|d| unsafe { Pin::new_unchecked(d) }.open_door("cards"));
    }
    if emusen_qt_models::screens::tool_screen(key).is_none() {
        return false;
    }
    unsafe { Pin::new_unchecked(feed) }.tool_requested(QString::from(key));
    true
}

impl cxx_qt::Initialize for qobject::ThemeFeed {
    fn initialize(mut self: Pin<&mut Self>) {
        // Safety: the address is only read back through the sink's cell, which the feed empties as it goes.
        let me = unsafe { self.as_mut().get_unchecked_mut() } as *mut qobject::ThemeFeed;
        self.rust().me.set(me);
        self.as_mut().on_host_changed(|feed| feed.attach()).release();
        self.as_mut().on_controllers_changed(|feed| feed.feed_device()).release();
        self.as_mut().on_family_changed(|feed| feed.feed_device()).release();
        self.as_mut().on_clock_changed(|feed| feed.feed_device()).release();
        self.as_mut().on_short_time_changed(|feed| feed.feed_device()).release();
        self.as_mut().on_battery_changed(|feed| feed.feed_device()).release();
        self.as_mut().on_charging_changed(|feed| feed.feed_device()).release();
        self.as_mut()
            .on_big_screen_changed(|feed| {
                feed.feed_device();
                feed.feed_tokens();
            })
            .release();
        self.as_mut().on_reduced_motion_changed(|feed| feed.feed_motion()).release();
        self.as_mut().on_swap_changed(|feed| feed.feed_swap()).release();
        self.as_mut().on_shown_section_changed(|feed| feed.feed_section()).release();
        self.as_mut().on_splash_shown_changed(|feed| feed.feed_splash()).release();
        self.as_mut().on_splash_time_changed(|feed| feed.feed_splash()).release();
        self.as_mut().on_splash_ease_changed(|feed| feed.feed_splash()).release();
        self.as_mut().on_now_changed(|feed| {
            feed.feed_words();
            feed.feed_browse();
        })
        .release();
    }
}

impl qobject::ThemeFeed {
    /// The theme's engine, or none while no theme is loaded.
    fn engine(&self) -> Option<Pin<&mut cxx_qt::QObject>> {
        // Safety: the host is the window's, alive as long as the feed it is handed to.
        let host = unsafe { self.host.as_ref() }?;
        unsafe { host.engine().as_mut() }.map(|e| unsafe { Pin::new_unchecked(e) })
    }

    fn library(&self) -> Option<&LibraryModel> {
        // Safety: the library model is the window's, alive as long as the feed.
        unsafe { self.library.as_ref() }
    }

    /// # Safety
    /// `library` is the window's `LibraryModel`, alive as long as the feed.
    pub unsafe fn show_library(mut self: Pin<&mut Self>, library: *mut cxx_qt::QObject) {
        self.as_mut().rust_mut().library = unsafe { model::<LibraryModel>(library) };
        // The library read again: the focused game's facts may have changed with it.
        self.facts_for.borrow_mut().clear();
        self.feed_library();
        // The session's console and the section's words are read from the library too.
        self.feed_session();
        self.feed_browse();
    }

    fn feed_library(&self) {
        self.library_gen.set(self.library_gen.get().wrapping_add(1));
        self.browse_key.borrow_mut().take();
        let (Some(library), Some(engine)) = (self.library(), self.engine()) else { return };
        let filter = unsafe { self.host.as_ref() }.map(|h| h.media_filter());
        if let Some(mut view) = host::library(engine) {
            view.as_mut().set_url_filter(filter);
            view.as_mut().show(library.view(), &library.home_lines().to_string());
        }
    }

    /// # Safety
    /// `frame` is the game's picture item, alive while the session runs.
    pub unsafe fn show_session(mut self: Pin<&mut Self>, game: &QString, running: bool, paused_at: f64, frame: *mut qobject::QQuickItem) {
        self.as_mut().rust_mut().session = Some(Shown { game: game.to_string(), running, paused_at, frame });
        self.feed_session();
    }

    fn feed_session(&self) {
        let Some(shown) = self.session.clone() else { return };
        let running = shown.running;
        let library = self.library().filter(|_| running);
        let found = library.and_then(|l| l.view().game(&shown.game));
        let facts = SessionFacts {
            has_game: running,
            key: if running { shown.game.clone() } else { String::new() },
            title: library.map(|l| l.title_of(&QString::from(shown.game.as_str())).to_string()).unwrap_or_default(),
            console_id: found.as_ref().map(|g| g.system.clone()).unwrap_or_default(),
            console: found.as_ref().map(|g| g.console.clone()).unwrap_or_default(),
            hue: found.as_ref().map_or(-1.0, |g| g.hue),
            paused_since: if running { shown.paused_at as i64 } else { 0 },
            // games.db's seconds played, as the library last read them.
            played: library.and_then(|l| l.view().catalog.plays.iter().find(|(p, _)| *p == shown.game).map(|(_, n)| n.seconds)).unwrap_or(0),
        };
        let Some(mut engine) = self.engine() else { return };
        if let Some(library) = host::library(engine.as_mut()) {
            library.show_suspended(&facts.key);
        }
        if let Some(mut session) = host::session(engine) {
            session.as_mut().show(&facts);
            session.show_frame(if running { shown.frame } else { std::ptr::null_mut() });
        }
    }

    /// # Safety
    /// `games` is the window's grid `GameModel`, alive as long as the feed.
    pub unsafe fn show_browse(mut self: Pin<&mut Self>, games: *mut cxx_qt::QObject, section: &QString, searching: bool, typing: bool, list_open: bool) {
        let games = unsafe { model::<GameModel>(games) };
        self.as_mut().rust_mut().section = Some(Section { games, key: section.to_string(), searching, typing, list_open });
        self.feed_browse();
    }

    fn feed_browse(&self) {
        let Some(section) = self.section.clone() else { return };
        // Safety: the grid's model is the window's, alive as long as the feed.
        let (Some(games), Some(library)) = (unsafe { section.games.as_ref() }, self.library()) else { return };
        let Some(engine) = self.engine() else { return };
        let host_gen = unsafe { self.host.as_ref() }.map_or(-1, |h| *h.generation());
        let open = (section.searching, section.typing, section.list_open);
        let searched = !games.search().is_empty();
        let mut facts = facts_of(library.view(), &section.key, &games.count_line().to_string(), *games.focused(), open, searched);
        let (letters, letter) = games.letters();
        facts.empty_words = games.empty_words().to_owned();
        facts.filter = games.filter_key();
        facts.sort = games.sort_words().to_owned();
        facts.sort_ascending = games.sort_ascending();
        facts.letter = letter;
        let cache_key = (host_gen, self.library_gen.get(), *games.revision(), section.key.clone(), self.now);
        let rebuild = *self.browse_key.borrow() != Some(cache_key.clone());
        let focused = usize::try_from(*games.focused()).ok().and_then(|i| games.rows().get(i)).map(|g| g.path.clone()).unwrap_or_default();
        self.ask_facts(library, focused);
        if let Some(mut browse) = host::browse(engine) {
            if rebuild {
                let mut rows = view::games(library.view(), games.rows(), self.at());
                view::map_game_urls(&mut rows, &|u| self.thumb_or_url(u));
                let (filters, letters) = (view::filters(games.choices()), view::letters(&letters));
                browse.as_mut().show_games(rows);
                browse.as_mut().show_listing(filters, letters);
                *self.browse_key.borrow_mut() = Some(cache_key);
            }
            browse.show(&facts);
        }
        if let Some(host_mut) = unsafe { self.host.as_mut() } {
            unsafe { Pin::new_unchecked(host_mut) }.sync_warnings();
        }
    }

    /// # Safety
    /// `settings` is the window's `SettingsModel`, alive as long as the feed.
    pub unsafe fn show_settings(mut self: Pin<&mut Self>, settings: *mut cxx_qt::QObject) {
        self.as_mut().rust_mut().settings = unsafe { model::<SettingsModel>(settings) } as *mut SettingsModel;
        if self.settings_queued {
            return;
        }
        self.as_mut().rust_mut().settings_queued = true;
        // Fed once the model has finished telling its change, never from inside a call of the model's.
        let queued = self.qt_thread().queue(|mut feed| {
            feed.as_mut().rust_mut().settings_queued = false;
            feed.feed_settings();
        });
        if queued.is_err() {
            // A queue refused (cxx-qt's ThreadingQueueError) feeds at once, so no change is lost.
            self.as_mut().rust_mut().settings_queued = false;
            self.feed_settings();
        }
    }

    /// Whether the host's theme draws a role, so a view no screen reads is never made in its engine.
    fn draws(&self, role: Role) -> bool {
        // Safety: the host is the window's, alive as long as the feed it is handed to.
        unsafe { self.host.as_ref() }.is_some_and(|h| h.provides(role))
    }

    fn draws_any_desktop(&self) -> bool {
        self.draws(Role::DesktopLibrary) || self.draws(Role::DesktopTable) || self.draws(Role::DesktopPlaying) || self.draws(Role::DesktopSettings)
    }

    /// Whether a theme's call of Settings is taken: the theme draws Settings or DesktopSettings and the window shows it.
    fn settings_taken(&self) -> bool {
        self.settings_shown && self.draws(if self.big_screen { Role::Settings } else { Role::DesktopSettings })
    }

    fn page_taken(&self) -> bool {
        self.page_shown && self.draws(Role::Game)
    }

    fn feed_settings(&self) {
        // Safety: the model is the window's, alive as long as the feed.
        let Some(settings) = (unsafe { self.settings.as_ref() }) else { return };
        if !self.draws(Role::Settings) && !self.draws(Role::DesktopSettings) {
            return;
        }
        let mut facts = match emusen_theme_qt::settings::facts_of(settings.view(), settings.shown_rows(), *settings.focused()) {
            Ok(facts) => facts,
            Err(why) => {
                eprintln!("theme feed: Settings not shown to the theme: {why}");
                return;
            }
        };
        if !self.big_screen {
            if let Some(desktop) = unsafe { self.desktop.as_ref() } {
                facts.filter = desktop.settings_filter().to_string();
                facts.modified_only = *desktop.settings_modified_only();
                facts.tool_index = *desktop.settings_tool_index();
                facts.can_back = !settings.trail().is_empty() || facts.tool_index >= 0;
            }
        }
        if let Some(view) = self.engine().and_then(host::settings) {
            view.show(&facts);
        }
    }

    /// # Safety: `desktop` is the window's `DesktopModel`, alive as long as the feed.
    pub unsafe fn show_desktop(mut self: Pin<&mut Self>, desktop: *mut cxx_qt::QObject) {
        self.as_mut().rust_mut().desktop = unsafe { qobj_model::<DesktopModel>(desktop) };
        if self.desktop_queued {
            return;
        }
        self.as_mut().rust_mut().desktop_queued = true;
        let queued = self.qt_thread().queue(|mut feed| {
            feed.as_mut().rust_mut().desktop_queued = false;
            feed.feed_desktop();
            feed.feed_session_live();
            feed.feed_settings();
        });
        if queued.is_err() {
            self.as_mut().rust_mut().desktop_queued = false;
            self.feed_desktop();
            self.feed_session_live();
            self.feed_settings();
        }
    }

    fn feed_desktop(&self) {
        if !self.draws_any_desktop() {
            return;
        }
        let Some(desktop) = (unsafe { self.desktop.as_ref() }) else { return };
        let Some(engine) = self.engine() else { return };
        let ws = desktop.workspace_data();
        let facts = workspace::facts_of(&ws, desktop.state(), *desktop.full_screen(), &|u| self.thumb_or_url(u));
        if let Some(view) = host::workspace(engine) {
            view.show(&facts);
        }
    }

    /// # Safety: `themes` is a `ThemesModel`, alive as long as the feed.
    pub unsafe fn show_themes(mut self: Pin<&mut Self>, themes: *mut cxx_qt::QObject) {
        self.as_mut().rust_mut().themes_model = unsafe { model::<ThemesModel>(themes) } as *mut ThemesModel;
        if self.themes_queued {
            return;
        }
        self.as_mut().rust_mut().themes_queued = true;
        let queued = self.qt_thread().queue(|mut feed| {
            feed.as_mut().rust_mut().themes_queued = false;
            feed.feed_themes();
        });
        if queued.is_err() {
            self.as_mut().rust_mut().themes_queued = false;
            self.feed_themes();
        }
    }

    fn feed_themes(&self) {
        if !self.draws_any_desktop() {
            return;
        }
        let Some(themes) = (unsafe { self.themes_model.as_ref() }) else { return };
        let Some(engine) = self.engine() else { return };
        let snap = themes.snapshot();
        let facts = theme_themes::facts_of(&snap, &|u| self.thumb_or_url(u));
        if let Some(view) = host::themes(engine) {
            view.show(&facts);
        }
    }

    /// # Safety: `tools` is a `ToolsHubModel`, alive as long as the feed.
    pub unsafe fn show_tools(mut self: Pin<&mut Self>, tools: *mut cxx_qt::QObject) {
        self.as_mut().rust_mut().tools_hub = unsafe { qobj_model::<ToolsHubModel>(tools) };
        if self.tools_queued {
            return;
        }
        self.as_mut().rust_mut().tools_queued = true;
        let queued = self.qt_thread().queue(|mut feed| {
            feed.as_mut().rust_mut().tools_queued = false;
            feed.feed_tools();
        });
        if queued.is_err() {
            self.as_mut().rust_mut().tools_queued = false;
            self.feed_tools();
        }
    }

    fn feed_tools(&self) {
        if !self.draws_any_desktop() {
            return;
        }
        let Some(tools) = (unsafe { self.tools_hub.as_ref() }) else { return };
        let Some(engine) = self.engine() else { return };
        let snap = tools.snapshot();
        let facts = theme_tools::facts_of(&snap);
        if let Some(view) = host::tools(engine) {
            view.show(&facts);
        }
    }

    /// # Safety: `collections` is a `CollectionsEclipseModel`, alive as long as the feed.
    pub unsafe fn show_collections(mut self: Pin<&mut Self>, collections_ptr: *mut cxx_qt::QObject) {
        self.as_mut().rust_mut().collections_model = unsafe { qobj_model::<CollectionsEclipseModel>(collections_ptr) };
        if self.collections_queued {
            return;
        }
        self.as_mut().rust_mut().collections_queued = true;
        let queued = self.qt_thread().queue(|mut feed| {
            feed.as_mut().rust_mut().collections_queued = false;
            feed.feed_collections();
        });
        if queued.is_err() {
            self.as_mut().rust_mut().collections_queued = false;
            self.feed_collections();
        }
    }

    fn feed_collections(&self) {
        if !self.draws_any_desktop() {
            return;
        }
        let Some(col) = (unsafe { self.collections_model.as_ref() }) else { return };
        let Some(engine) = self.engine() else { return };
        let snap = col.snapshot();
        let facts = collections::facts_of(&snap);
        if let Some(view) = host::collections(engine) {
            view.show(&facts);
        }
    }

    /// # Safety: `session` is the window's `SessionModel`, alive as long as the feed.
    pub unsafe fn show_session_live(mut self: Pin<&mut Self>, session_ptr: *mut cxx_qt::QObject) {
        self.as_mut().rust_mut().session_model = unsafe { qobj_model::<SessionModel>(session_ptr) };
        if self.session_live_queued {
            return;
        }
        self.as_mut().rust_mut().session_live_queued = true;
        let queued = self.qt_thread().queue(|mut feed| {
            feed.as_mut().rust_mut().session_live_queued = false;
            feed.feed_session_live();
        });
        if queued.is_err() {
            self.as_mut().rust_mut().session_live_queued = false;
            self.feed_session_live();
        }
    }

    fn ensure_session_pictures_on_engine(&self) {
        let host_gen = unsafe { self.host.as_ref() }.map_or(-1, |h| *h.generation());
        if host_gen < 0 || self.pictures_served_gen.get() == host_gen {
            return;
        }
        let Some(sm) = (unsafe { self.session_model.as_ref() }) else { return };
        let Some(engine) = self.engine() else { return };
        if sm.serve_pictures_on(&*engine) {
            self.pictures_served_gen.set(host_gen);
        }
    }

    fn feed_session_live(&self) {
        self.ensure_session_pictures_on_engine();
        if !self.draws_any_desktop() {
            return;
        }
        let Some(sm) = (unsafe { self.session_model.as_ref() }) else { return };
        let Some(engine) = self.engine() else { return };
        let play = unsafe { self.desktop.as_ref() }.map_or_else(Default::default, |d| d.playing_data());
        let resume = (
            *sm.resume_asked(),
            sm.resume_subject().to_string(),
            sm.resume_picture().to_string(),
            sm.resume_when().to_string(),
            sm.resume_console().to_string(),
            sm.resume_core().to_string(),
            *sm.resume_planet(),
        );
        let facts = theme_session::live_facts_of(
            &play,
            *sm.sound_volume(),
            (resume.0, &resume.1, &resume.2, &resume.3, &resume.4, &resume.5, resume.6),
            &|u| self.thumb_or_url(u),
        );
        if let Some(view) = host::session(engine) {
            view.show_live(&facts);
        }
    }

    /// # Safety
    /// `page` is the window's `GamePageModel`, alive as long as the feed.
    pub unsafe fn show_page(mut self: Pin<&mut Self>, page: *mut cxx_qt::QObject) {
        self.as_mut().rust_mut().page = unsafe { model::<GamePageModel>(page) } as *mut GamePageModel;
        self.queue_page();
    }

    /// As Settings: fed once the model has finished telling its change, at once where the queue is refused.
    fn queue_page(mut self: Pin<&mut Self>) {
        if self.page_queued {
            return;
        }
        self.as_mut().rust_mut().page_queued = true;
        let queued = self.qt_thread().queue(|mut feed| {
            feed.as_mut().rust_mut().page_queued = false;
            feed.feed_page();
        });
        if queued.is_err() {
            self.as_mut().rust_mut().page_queued = false;
            self.feed_page();
        }
    }

    /// Proxies out-of-store regular image files through the host's `image://thumb/<id>` provider.
    fn thumb_or_url(&self, url: String) -> String {
        proxy_thumb_url(unsafe { self.host.as_ref() }, url)
    }

    fn feed_page(&self) {
        // Safety: the model is the window's, alive as long as the feed.
        let Some(page) = (unsafe { self.page.as_ref() }) else { return };
        if !self.draws(Role::Game) {
            return;
        }
        let text = |t: &QString| t.to_string();
        let words = |l: &QStringList| QList::<QString>::from(l).iter().map(|w| w.to_string()).collect::<Vec<_>>();
        let (row_sentence, row_place) = page.board_aside();
        let (crop_picture, crop_ratios, crop_edge) = page.crop_preview();
        let mut cards = card_rows(&page.cards());
        // ThemeRows `cards` places `picture` at column 3 (page::PAGE_CARD_ROLES).
        for row in &mut cards {
            if let Some(view::Cell::Text(u)) = row.get_mut(3) {
                *u = self.thumb_or_url(std::mem::take(u));
            }
        }
        let mut media_tiles = media_tile_rows(&page.media_cards());
        // ThemeRows `mediaTiles` places `picture` at column 2 (page::PAGE_MEDIA_TILE_ROLES).
        for row in &mut media_tiles {
            if let Some(view::Cell::Text(u)) = row.get_mut(2) {
                *u = self.thumb_or_url(std::mem::take(u));
            }
        }
        let mut rows = content_rows(page.shown_items());
        // ThemeRows `rows` places `picture` at column 4 (page::PAGE_ROW_ROLES).
        for row in &mut rows {
            if let Some(view::Cell::Text(u)) = row.get_mut(4) {
                *u = self.thumb_or_url(std::mem::take(u));
            }
        }
        let view_kind = *page.view_kind();
        let view_source = if view_kind == 0 { self.thumb_or_url(text(page.view_source())) } else { text(page.view_source()) };
        let facts = PageFacts {
            ready: *page.ready(),
            key: page.game_key().to_owned(),
            title: text(page.title()),
            facts: text(page.facts()),
            cover: self.thumb_or_url(text(page.cover())),
            hero: self.thumb_or_url(text(page.hero())),
            clip: text(page.clip()),
            actions: action_rows(&page.shown_actions()),
            sections: words(page.sections()),
            section: *page.section(),
            shows: *page.shows(),
            words: text(page.words()),
            picture: self.thumb_or_url(text(page.picture())),
            rows,
            cover_art: *page.cover_art(),
            focused: page.focused(),
            help: emusen_theme_qt::help::kind_of(page.help_screen()).map_or(-1, |k| k.repr),
            asking: *page.asking(),
            question_title: text(page.question_title()),
            question_sentences: words(page.question_sentences()),
            question_choices: words(page.question_choices()),
            question_back: *page.question_back(),
            tabs: page.tabs(),
            tab: page.tab(),
            kicker: page.kicker(),
            line: page.line(),
            pills: pill_rows(&page.pills()),
            cards,
            note: page.cards_note().to_owned(),
            trail: page.trail(),
            trail_section: emusen_qt_models::screens::CONSOLES_KEY.to_owned(),
            console: page.console().to_owned(),
            board_help: page.board_help_screen().and_then(emusen_theme_qt::help::kind_of).map_or(-1, |k| k.repr),
            has_description: page.has_description(),
            board_rows: board_row_rows(&page.board_rows()),
            row_sentence,
            row_place,
            defaultable: page.defaultable(),
            media_tiles,
            media_note: page.media_note().to_owned(),
            coverable: page.coverable(),
            subpage: page.subpage() as i32,
            subpage_title: page.subpage_title().to_owned(),
            subpage_subtitle: page.subpage_subtitle(),
            view_kind,
            view_source,
            view_manual: text(page.view_manual()),
            view_caption: page.board_view_caption(),
            view_page: *page.view_page(),
            view_zoom: *page.view_zoom(),
            view_pan_x: *page.view_pan_x(),
            view_pan_y: *page.view_pan_y(),
            crop_picture: self.thumb_or_url(crop_picture),
            crop_left: crop_ratios[0],
            crop_right: crop_ratios[1],
            crop_top: crop_ratios[2],
            crop_bottom: crop_ratios[3],
            crop_edge,
        };
        if let Some(view) = self.engine().and_then(host::page) {
            view.show(&facts);
        }
        if let Some(host_mut) = unsafe { self.host.as_mut() } {
            unsafe { Pin::new_unchecked(host_mut) }.sync_warnings();
        }
    }

    /// Reads the focused game's facts on a single coalescing worker thread when the focus moves.
    fn ask_facts(&self, library: &LibraryModel, path: String) {
        if *self.facts_for.borrow() == path {
            return;
        }
        self.facts_for.replace(path.clone());
        let game = library.view().catalog.games.iter().find(|g| g.path == path).cloned();
        let (Some(game), Some(source)) = (game, library.source()) else {
            self.show_facts(emusen_qt_models::library::GameFacts::default());
            return;
        };
        let seconds = emusen_qt_models::library::seconds_played(&library.view().catalog, &path);
        let qt = self.qt_thread();
        let mut req = FactsRequest { source, game, seconds, qt };
        let mut worker = self.facts_worker.borrow_mut();
        if let Some((tx, handle)) = worker.take() {
            match tx.send(req) {
                Ok(()) => {
                    *worker = Some((tx, handle));
                    return;
                }
                Err(std::sync::mpsc::SendError(returned)) => {
                    req = returned;
                    drop(tx);
                    let _ = handle.join();
                }
            }
        }
        // Respawn only on first call or if a stale receiver exited after a failed queue.
        let (send, take) = std::sync::mpsc::channel::<FactsRequest>();
        let handle = std::thread::spawn(move || {
            while let Ok(mut latest) = take.recv() {
                while let Ok(newer) = take.try_recv() {
                    latest = newer;
                }
                let facts = emusen_qt_models::library::facts_of(latest.source.as_ref(), &latest.game, latest.seconds)
                    .unwrap_or_else(|_| emusen_qt_models::library::GameFacts { path: latest.game.path.clone(), ..Default::default() });
                let sent = latest.qt.queue(move |feed| {
                    if *feed.facts_for.borrow() == facts.path {
                        feed.show_facts(facts);
                    }
                });
                if sent.is_err() {
                    break;
                }
            }
        });
        let _ = send.send(req);
        *worker = Some((send, handle));
    }

    fn show_facts(&self, facts: emusen_qt_models::library::GameFacts) {
        if let Some(browse) = self.engine().and_then(host::browse) {
            browse.show_facts(&facts);
        }
    }

    fn feed_section(&self) {
        if let Some(library) = self.engine().and_then(host::library) {
            library.show_section(&self.shown_section.to_string());
        }
    }

    fn feed_splash(&self) {
        // Safety: the host is the window's, alive as long as the feed it is handed to.
        let Some(generation) = unsafe { self.host.as_ref() }.map(|h| *h.generation()) else { return };
        if self.engine().is_none() {
            return;
        }
        let (at, kept) = self.splash.get();
        let splash = if at == generation && !kept.is_null() {
            kept
        } else {
            // Safety: the engine keeps its singleton alive as long as itself, and a new engine moves the generation.
            let found = self.engine().and_then(host::splash).map_or(std::ptr::null_mut(), |s| unsafe { s.get_unchecked_mut() } as *mut _);
            self.splash.set((generation, found));
            found
        };
        // Safety: as above.
        if let Some(splash) = unsafe { splash.as_mut() } {
            unsafe { Pin::new_unchecked(splash) }.show(self.splash_shown, self.splash_time, self.splash_ease);
        }
    }

    fn feed_device(&self) {
        let facts = DeviceFacts {
            controllers: self.controllers,
            family: self.family,
            clock: self.clock.to_string(),
            short_time: self.short_time.to_string(),
            battery: self.battery,
            charging: self.charging,
            big_screen: self.big_screen,
            form: if self.big_screen { Form::BigScreen.repr } else { Form::Desktop.repr },
            ..DeviceFacts::default()
        };
        if let Some(device) = self.engine().and_then(host::device) {
            device.show(&facts);
        }
    }

    /// Now, held or the wall clock's.
    fn at(&self) -> i64 {
        if self.now > 0 { self.now } else { words::now() }
    }

    fn feed_words(&self) {
        if let Some(words) = self.engine().and_then(host::words) {
            words.show_now(self.now as f64);
        }
    }

    fn feed_motion(&self) {
        if let Some(tokens) = self.engine().and_then(host::kit_tokens) {
            tokens.set_reduced_motion(self.reduced_motion);
        }
    }

    fn feed_swap(&self) {
        if let Some(router) = self.engine().and_then(host::pad_router) {
            router.set_swap_accept_back(self.swap);
        }
    }

    /// The theme's own tokens on its engine's; `ThemeStyle` sets the chosen theme's on the program's engine by the same call.
    fn feed_tokens(&self) {
        let overrides = host_overrides(self.host, self.big_screen);
        let form = if self.big_screen { Form::BigScreen } else { Form::Desktop };
        if let Some(tokens) = self.engine().and_then(host::kit_tokens) {
            tokens.set_form(form);
        }
        if let Some(engine) = self.engine() {
            host::set_token_overrides(engine, &overrides);
        }
    }

    pub fn attach(self: Pin<&mut Self>) {
        let sink = Sink { feed: self.me.clone(), in_hand: String::new(), chip_return: 0 };
        if let Some(actions) = self.engine().and_then(host::actions) {
            actions.set_sink(Some(Box::new(sink)));
        }
        self.feed_words();
        self.feed_motion();
        self.feed_swap();
        self.feed_tokens();
        self.feed_device();
        self.feed_splash();
        self.feed_library();
        self.feed_section();
        self.feed_session();
        self.feed_browse();
        self.feed_settings();
        self.feed_page();
        self.feed_desktop();
        self.feed_themes();
        self.feed_tools();
        self.feed_collections();
        self.feed_session_live();
    }

    pub fn push(self: Pin<&mut Self>, button: i32, down: bool) {
        if let Some(router) = self.engine().and_then(host::pad_router) {
            router.push(emusen_kit_qt::kinds::PadButton { repr: button }, down);
        }
    }
}

pub(crate) fn host_overrides(host: *const qobject::ThemeHost, big_screen: bool) -> Vec<(bool, String, String)> {
    let Some(host) = (unsafe { host.as_ref() }) else { return Vec::new() };
    let mut overrides = host.overrides();
    if !big_screen {
        for (light, name, value) in host.desktop_overrides() {
            if let Some(slot) = overrides.iter_mut().find(|(l, n, _)| *l == light && *n == name) {
                slot.2 = value;
            } else {
                overrides.push((light, name, value));
            }
        }
    }
    overrides
}

fn proxy_thumb_url(host: Option<&qobject::ThemeHost>, url: String) -> String {
    if url.is_empty() || url.starts_with("qrc:") || url.starts_with("image:") {
        return url;
    }
    let Some(host) = host else { return String::new() };
    if host.allows_url(&url) {
        return url;
    }
    host.allow_thumb(&url)
}

#[cfg(test)]
mod tests {
    use super::proxy_thumb_url;

    #[test]
    fn thumb_or_url_rejects_raw_file_urls_without_host() {
        assert_eq!(proxy_thumb_url(None, "file:///home/player/Roms/snes/cover.png".to_owned()), "");
        assert_eq!(proxy_thumb_url(None, "/home/player/Roms/snes/cover.png".to_owned()), "");
        assert_eq!(proxy_thumb_url(None, "image://thumb/42".to_owned()), "image://thumb/42");
        assert_eq!(proxy_thumb_url(None, "qrc:/emusen/icons/pad.svg".to_owned()), "qrc:/emusen/icons/pad.svg");
        assert_eq!(proxy_thumb_url(None, String::new()), "");
    }
}
