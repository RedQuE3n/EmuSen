//! `ThemeFeed`: the program's side of an EmuSen theme (EmuSen_Qt.md §3.5.2). It hands the theme's engine the views of
//! `EmuSen.Theme` from the program's models, carries the theme's `Actions` back to the window as a signal, and keeps the
//! kit's per-engine singletons in step: reduced motion, and the pad's presses while a theme's screen holds the focus.

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use cxx_qt::casting::Upcast;
use emusen_qt_models::library_model::qobject::LibraryModel;
use emusen_qt_models::{help, screens};
use emusen_theme_qt::view;
use emusen_theme_qt::actions::{Action, ActionSink};
use emusen_theme_qt::device::DeviceFacts;
use emusen_theme_qt::host;
use emusen_theme_qt::session::SessionFacts;
use std::cell::Cell;
use std::rc::Rc;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
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
        type ThemeFeed = super::ThemeFeedRust;
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

        /// The pads connected, their glyphs' family, and the time as Home's status line says it.
        #[qinvokable]
        #[cxx_name = "showDevice"]
        fn show_device(self: Pin<&mut ThemeFeed>, controllers: i32, family: i32, clock: &QString);

        /// Reduced motion, as the window binds the kit's tokens.
        #[qinvokable]
        #[cxx_name = "showMotion"]
        fn show_motion(self: Pin<&mut ThemeFeed>, reduced: bool);

        /// The theme's `Actions` carried by this feed, in the host's engine as it is now.
        #[qinvokable]
        fn attach(self: Pin<&mut ThemeFeed>);

        /// A pad's button down or up, for the theme's engine's router.
        #[qinvokable]
        fn push(self: Pin<&mut ThemeFeed>, button: i32, down: bool);

        /// The window declines the action being carried; it is taken otherwise.
        #[qinvokable]
        fn decline(self: Pin<&mut ThemeFeed>);

        /// The theme's `Actions`, each as the window carries it out: a game played or resumed by its file.
        #[qsignal]
        fn played(self: Pin<&mut ThemeFeed>, game: QString);
        /// One of Home's places, by its index in Home's order.
        #[qsignal]
        #[cxx_name = "placeChosen"]
        fn place_chosen(self: Pin<&mut ThemeFeed>, index: i32);
        /// A console's or a shelf's grid, by its key.
        #[qsignal]
        fn opened(self: Pin<&mut ThemeFeed>, key: QString);
        /// B that the theme's screen did not keep.
        #[qsignal]
        fn backed(self: Pin<&mut ThemeFeed>);
        /// A screen's declared action, by the help bar's key (`emusen_qt_models::help`).
        #[qsignal]
        fn acted(self: Pin<&mut ThemeFeed>, key: QString);
    }

    impl cxx_qt::Initialize for ThemeFeed {}
}

/// The feed's state: the window's host and library, and whether the window declined the action it is carrying.
pub struct ThemeFeedRust {
    host: *mut qobject::ThemeHost,
    library: *mut LibraryModel,
    declined: bool,
    /// The feed as the sink sees it, emptied when the feed goes, so a sink that outlives it takes nothing.
    me: Rc<Cell<*mut qobject::ThemeFeed>>,
}

impl Default for ThemeFeedRust {
    fn default() -> Self {
        ThemeFeedRust {
            host: std::ptr::null_mut(),
            library: std::ptr::null_mut(),
            declined: false,
            me: Rc::new(Cell::new(std::ptr::null_mut())),
        }
    }
}

impl Drop for ThemeFeedRust {
    fn drop(&mut self) {
        self.me.set(std::ptr::null_mut());
    }
}

/// The theme's `Actions` as the window's signal.
struct Sink(Rc<Cell<*mut qobject::ThemeFeed>>);

/// What the window does for an action: each of the theme's `Actions` as one of the feed's signals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Carried {
    Played(String),
    Place(i32),
    Opened(String),
    Backed,
    Acted(&'static str),
}

/// The signal an action becomes; none for one the program has no answer to yet (the Home card opened by a theme).
pub fn carried(action: &Action) -> Option<Carried> {
    Some(match action {
        Action::Play(key) => Carried::Played(key.clone()),
        Action::OpenPage(_) => Carried::Acted(help::GAME_PAGE),
        Action::Favourite(_) => Carried::Acted(help::FAVOURITE),
        Action::Random(_) => Carried::Acted(help::RANDOM),
        Action::OpenConsole(key) => Carried::Opened(key.clone()),
        Action::OpenPlace(key) => match view::PLACE_IDS.iter().position(|id| id == key) {
            Some(i) => Carried::Place(i as i32),
            None => Carried::Opened(key.clone()),
        },
        Action::Home => Carried::Acted(help::HOME),
        Action::Back => Carried::Backed,
        Action::StepSection(by) => Carried::Acted(if *by < 0 { help::PREVIOUS } else { help::NEXT }),
        Action::OpenHomeCard => return None,
        Action::OpenListCard => Carried::Acted(screens::LIST),
        Action::Search => Carried::Acted(screens::SEARCH),
    })
}

impl ActionSink for Sink {
    fn act(&mut self, action: &Action) -> bool {
        // Safety: the cell holds the feed while it lives and null after; the theme's engine runs on the feed's thread.
        let Some(feed) = (unsafe { self.0.get().as_mut() }) else { return false };
        let mut feed = unsafe { Pin::new_unchecked(feed) };
        let Some(carried) = carried(action) else { return false };
        feed.as_mut().rust_mut().declined = false;
        match carried {
            Carried::Played(game) => feed.as_mut().played(QString::from(game.as_str())),
            Carried::Place(index) => feed.as_mut().place_chosen(index),
            Carried::Opened(key) => feed.as_mut().opened(QString::from(key.as_str())),
            Carried::Backed => feed.as_mut().backed(),
            Carried::Acted(key) => feed.as_mut().acted(QString::from(key)),
        }
        !feed.declined
    }
}

impl cxx_qt::Initialize for qobject::ThemeFeed {
    fn initialize(mut self: Pin<&mut Self>) {
        let me = unsafe { self.as_mut().get_unchecked_mut() } as *mut qobject::ThemeFeed;
        self.rust().me.set(me);
        self.as_mut().on_host_changed(|feed| feed.attach()).release();
    }
}

impl qobject::ThemeFeed {
    /// The theme's engine, or none while no theme is loaded.
    fn engine(&self) -> Option<Pin<&mut cxx_qt::QObject>> {
        // Safety: the host is the window's, alive as long as the feed it is handed to.
        let host = unsafe { self.host.as_ref() }?;
        unsafe { host.engine().as_mut() }.map(|e| unsafe { Pin::new_unchecked(e) })
    }

    /// # Safety
    /// `library` is the window's `LibraryModel`, alive as long as the feed.
    pub unsafe fn show_library(mut self: Pin<&mut Self>, library: *mut cxx_qt::QObject) {
        // Safety: the window hands its LibraryModel; a list model's QObject base sits at its address (single inheritance).
        let list = library as *const emusen_qt_models::library_model::qobject::QAbstractListModel;
        self.as_mut().rust_mut().library = unsafe { LibraryModel::from_base_ptr(list) } as *mut LibraryModel;
        // Safety: as above.
        let (Some(library), Some(engine)) = (unsafe { self.library.as_ref() }, self.engine()) else { return };
        if let Some(view) = host::library(engine) {
            view.show(library.view(), &library.home_lines().to_string());
        }
    }

    /// # Safety
    /// `frame` is the game's picture item, alive while the session runs.
    pub unsafe fn show_session(self: Pin<&mut Self>, game: &QString, running: bool, paused_at: f64, frame: *mut qobject::QQuickItem) {
        // Safety: the library model is the window's, alive as long as the feed.
        let library = unsafe { self.library.as_ref() };
        let found = library.and_then(|l| l.view().catalog.games.iter().find(|g| g.path == game.to_string())).filter(|_| running);
        let facts = SessionFacts {
            has_game: running,
            title: library.filter(|_| running).map(|l| l.title_of(game).to_string()).unwrap_or_default(),
            console_id: found.and_then(|g| g.system.clone()).unwrap_or_default(),
            paused_since: if running { paused_at as i64 } else { 0 },
            played: 0,
        };
        let Some(engine) = self.engine() else { return };
        if let Some(mut session) = host::session(engine) {
            session.as_mut().show(&facts);
            session.show_frame(if running { frame } else { std::ptr::null_mut() });
        }
    }

    pub fn show_device(self: Pin<&mut Self>, controllers: i32, family: i32, clock: &QString) {
        let facts = DeviceFacts { controllers, family, clock: clock.to_string(), ..DeviceFacts::default() };
        let Some(engine) = self.engine() else { return };
        if let Some(device) = host::device(engine) {
            device.show(&facts);
        }
    }

    pub fn show_motion(self: Pin<&mut Self>, reduced: bool) {
        let Some(engine) = self.engine() else { return };
        if let Some(tokens) = host::kit_tokens(engine) {
            tokens.set_reduced_motion(reduced);
        }
    }

    pub fn attach(self: Pin<&mut Self>) {
        let sink = Sink(self.me.clone());
        let Some(engine) = self.engine() else { return };
        if let Some(actions) = host::actions(engine) {
            actions.set_sink(Some(Box::new(sink)));
        }
    }

    pub fn push(self: Pin<&mut Self>, button: i32, down: bool) {
        let Some(engine) = self.engine() else { return };
        if let Some(router) = host::pad_router(engine) {
            router.push(emusen_kit_qt::kinds::PadButton { repr: button }, down);
        }
    }

    pub fn decline(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().declined = true;
    }
}

