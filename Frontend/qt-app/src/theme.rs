//! `ThemeFeed`: the program's side of an EmuSen theme (EmuSen_Qt.md §3.5.2). It hands the theme's engine the views of
//! `EmuSen.Theme` from the program's models, carries the theme's `Actions` back to the window as signals, and keeps the
//! kit's per-engine singletons in step: reduced motion, and the pad's presses while a theme's screen holds the focus.
//! What it last showed is kept, so a new engine is fed it all at once.

use core::pin::Pin;
use cxx_qt::casting::Upcast;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QList, QString, QStringList};
use emusen_qt_models::game_model::qobject::GameModel;
use emusen_qt_models::game_page_model::qobject::GamePageModel;
use emusen_qt_models::library_model::qobject::{LibraryModel, QAbstractListModel};
use emusen_qt_models::settings_model::qobject::SettingsModel;
use emusen_qt_models::{help, list_card, screens, themes, words};
use emusen_theme_host::host::qobject::Role;
use emusen_theme_qt::actions::{Action, ActionSink, ScreenAction};
use emusen_theme_qt::browse::facts_of;
use emusen_theme_qt::device::DeviceFacts;
use emusen_theme_qt::host;
use emusen_theme_qt::page::{action_rows, board_row_rows, card_rows, content_rows, media_tile_rows, pill_rows, PageFacts};
use emusen_theme_qt::session::SessionFacts;
use emusen_theme_qt::view;
use std::cell::Cell;
use std::rc::Rc;

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
        BuiltInThemesRust { reference: QString::from(crate::reference_theme().as_str()), manifests: QStringList::from(&manifests) }
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

/// The feed's state: the window's host and models, and what it last showed.
pub struct ThemeFeedRust {
    host: *mut qobject::ThemeHost,
    library: *const LibraryModel,
    /// The window's Settings and page models, which a theme's screens read and act through.
    settings: *mut SettingsModel,
    page: *mut GamePageModel,
    /// A feed of Settings or of the page queued and not yet run, so the changes of one press feed once.
    settings_queued: bool,
    page_queued: bool,
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
    facts_for: std::cell::RefCell<String>,
    shown_section: QString,
    splash_shown: bool,
    splash_time: f64,
    splash_ease: f64,
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
            settings_queued: false,
            page_queued: false,
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
            facts_for: std::cell::RefCell::new(String::new()),
            shown_section: QString::default(),
            splash_shown: false,
            splash_time: 0.0,
            splash_ease: 0.0,
            splash: Cell::new((-1, std::ptr::null_mut())),
            me: Rc::new(Cell::new(std::ptr::null_mut())),
        }
    }
}

impl Drop for ThemeFeedRust {
    fn drop(&mut self) {
        self.me.set(std::ptr::null_mut());
    }
}

/// A model the window hands as a QObject, as its own type: a list model's QObject base sits at its address.
///
/// # Safety
/// `object` is a `T` or null.
unsafe fn model<T: Upcast<QAbstractListModel>>(object: *mut cxx_qt::QObject) -> *const T {
    unsafe { T::from_base_ptr(object as *const QAbstractListModel) }
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
        // X back to default, Consoles Y filter, and LT/RT letter jump on a theme's Settings (R-20261116-22).
        if matches!(action, Action::Act(key) if matches!(key.as_str(), help::DEFAULT | help::CONSOLES_FILTER | help::CONSOLES_LETTER_BACK | help::CONSOLES_LETTER_ON)) {
            let settings = feed.taken(Role::Settings).then(|| unsafe { feed.settings.as_mut() }).flatten().map(|m| unsafe { Pin::new_unchecked(m) });
            return settings.is_some_and(|s| match action {
                Action::Act(key) if key == help::DEFAULT => {
                    let row = (*s.focused()).max(s.view().firmware_need_count() as i32);
                    s.to_default(row)
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
        let settings = feed.taken(Role::Settings).then(|| unsafe { feed.settings.as_mut() }).flatten().map(|m| unsafe { Pin::new_unchecked(m) });
        let page = feed.taken(Role::Game).then(|| unsafe { feed.page.as_mut() }).flatten().map(|m| unsafe { Pin::new_unchecked(m) });
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
            (ScreenAction::SettingsBack, Some(s), _) => {
                if let Some(start) = s.view().chip_start().filter(|&st| *s.focused() >= st as i32) {
                    s.set_focused(self.chip_return.clamp(0, start.saturating_sub(1) as i32));
                    return true;
                }
                self.chip_return = 0;
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
                let took = (*action == -1 || within(*action, actions)) && (*row == 0 || within(*row, p.shown_items().len()));
                if took {
                    p.focus(*action, *row);
                }
                took
            }
            (ScreenAction::PageDeclared(key), _, Some(p)) => p.declared(&QString::from(key.as_str())),
            (ScreenAction::PageAnswer(choice), _, Some(p)) => p.answer(*choice),
            (ScreenAction::PageBack, _, Some(p)) => p.back(),
            (ScreenAction::PageTab(tab), _, Some(p)) => p.choose_tab(*tab),
            _ => false,
        }
    }
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
        let (Some(library), Some(engine)) = (self.library(), self.engine()) else { return };
        if let Some(mut view) = host::library(engine) {
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
        let open = (section.searching, section.typing, section.list_open);
        let searched = !games.search().is_empty();
        let mut facts = facts_of(library.view(), &section.key, &games.count_line().to_string(), *games.focused(), open, searched);
        let (letters, letter) = games.letters();
        facts.empty_words = games.empty_words().to_owned();
        facts.filter = games.filter_key();
        facts.sort = games.sort_words().to_owned();
        facts.letter = letter;
        let rows = view::games(library.view(), games.rows(), self.at());
        let (filters, letters) = (view::filters(games.choices()), view::letters(&letters));
        let focused = usize::try_from(*games.focused()).ok().and_then(|i| games.rows().get(i)).map(|g| g.path.clone()).unwrap_or_default();
        self.ask_facts(library, focused);
        let Some(engine) = self.engine() else { return };
        if let Some(mut browse) = host::browse(engine) {
            browse.as_mut().show_games(rows);
            browse.show_listing(filters, letters);
            browse.show(&facts);
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

    /// Whether a theme's call of the screen's is taken: the theme draws it and the window shows it.
    fn taken(&self, role: Role) -> bool {
        let shown = (role == Role::Settings && self.settings_shown) || (role == Role::Game && self.page_shown);
        shown && self.draws(role)
    }

    fn feed_settings(&self) {
        // Safety: the model is the window's, alive as long as the feed.
        let Some(settings) = (unsafe { self.settings.as_ref() }) else { return };
        if !self.draws(Role::Settings) {
            return;
        }
        let facts = match emusen_theme_qt::settings::facts_of(settings.view(), settings.shown_rows(), *settings.focused()) {
            Ok(facts) => facts,
            Err(why) => {
                eprintln!("theme feed: Settings not shown to the theme: {why}");
                return;
            }
        };
        if let Some(view) = self.engine().and_then(host::settings) {
            view.show(&facts);
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

    fn feed_page(&self) {
        // Safety: the model is the window's, alive as long as the feed.
        let Some(page) = (unsafe { self.page.as_ref() }) else { return };
        if !self.draws(Role::Game) {
            return;
        }
        let text = |t: &QString| t.to_string();
        let words = |l: &QStringList| QList::<QString>::from(l).iter().map(|w| w.to_string()).collect::<Vec<_>>();
        let (row_sentence, row_place) = page.board_aside();
        let facts = PageFacts {
            ready: *page.ready(),
            key: page.game_key().to_owned(),
            title: text(page.title()),
            facts: text(page.facts()),
            cover: text(page.cover()),
            hero: text(page.hero()),
            clip: text(page.clip()),
            actions: action_rows(&page.shown_actions()),
            sections: words(page.sections()),
            section: *page.section(),
            shows: *page.shows(),
            words: text(page.words()),
            picture: text(page.picture()),
            rows: content_rows(page.shown_items()),
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
            cards: card_rows(&page.cards()),
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
            media_tiles: media_tile_rows(&page.media_cards()),
            media_note: page.media_note().to_owned(),
            coverable: page.coverable(),
            subpage: page.subpage() as i32,
            subpage_title: page.subpage_title().to_owned(),
            subpage_subtitle: page.subpage_subtitle().to_owned(),
        };
        if let Some(view) = self.engine().and_then(host::page) {
            view.show(&facts);
        }
    }

    /// Reads the focused game's facts off the GUI thread when the focus has moved to another game, and feeds them to
    /// `Browse` when they come back for the game still focused (EmuSen_Qt.md §3.4.10).
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
        std::thread::spawn(move || {
            let facts = emusen_qt_models::library::facts_of(source.as_ref(), &game, seconds).unwrap_or_else(|_| emusen_qt_models::library::GameFacts { path: game.path.clone(), ..Default::default() });
            let _ = qt.queue(move |feed| {
                if *feed.facts_for.borrow() == facts.path {
                    feed.show_facts(facts);
                }
            });
        });
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
        let facts = DeviceFacts { controllers: self.controllers, family: self.family, clock: self.clock.to_string(),
            short_time: self.short_time.to_string(), battery: self.battery, charging: self.charging,
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
        // Safety: the host is the window's, alive as long as the feed it is handed to.
        let overrides = unsafe { self.host.as_ref() }.map(|h| h.overrides()).unwrap_or_default();
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
    }

    pub fn push(self: Pin<&mut Self>, button: i32, down: bool) {
        if let Some(router) = self.engine().and_then(host::pad_router) {
            router.push(emusen_kit_qt::kinds::PadButton { repr: button }, down);
        }
    }
}
