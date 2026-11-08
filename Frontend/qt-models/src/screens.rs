//! `Screens`: which of the program's screens is shown and which row the grid opened, so the screens switch by calls,
//! never by logic in QML (EmuSen_Qt.md §7.1). `step` is the whole rule; the bridge only puts it on Qt, after the event
//! that asked, so one press is never answered by the screen it left and the screen it opened.

use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;

/// A screen of the program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shown {
    Home,
    Consoles,
    Grid,
    /// The game, its picture filling the window.
    Game,
    /// The Pause Card over the paused game.
    Pause,
}

/// What asked for a change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    /// One of Home's places, by its index in `words::HOME_PLACES`; the first, Games, is the only one built yet.
    Place(usize),
    /// A row chosen on Choose a console.
    Open(String),
    /// B that no screen kept.
    Back,
    /// A screen's declared action (crate::help); the favourites' action opens their grid, the others are a later stage's.
    Act(String),
    /// A game chosen on Home's shelf or the grid, Continue chosen, or the Pause Card resuming: the game is shown.
    Play,
    /// The menu chord over the game: the Pause Card, as Mistress's `OpenPadMenu` opens it over a game.
    Menu,
    /// The Pause Card's Back to Library: Home, the game suspended behind it.
    Library,
    /// The game closed: Home.
    Closed,
}

/// The screen shown after an ask, and the grid's row; `None` leaves both as they were.
pub fn step(shown: Shown, ask: &Ask) -> Option<(Shown, Option<String>)> {
    match (shown, ask) {
        (Shown::Home, Ask::Place(0)) => Some((Shown::Consoles, None)),
        (Shown::Consoles, Ask::Open(key)) => Some((Shown::Grid, Some(key.clone()))),
        (Shown::Consoles, Ask::Act(key)) if key == crate::library::FAVOURITES => Some((Shown::Grid, Some(key.clone()))),
        (Shown::Consoles, Ask::Back) => Some((Shown::Home, None)),
        (Shown::Grid, Ask::Back) => Some((Shown::Consoles, None)),
        (Shown::Home | Shown::Grid | Shown::Pause, Ask::Play) => Some((Shown::Game, None)),
        (Shown::Game, Ask::Menu) => Some((Shown::Pause, None)),
        (Shown::Pause, Ask::Library) => Some((Shown::Home, None)),
        (Shown::Game | Shown::Pause, Ask::Closed) => Some((Shown::Home, None)),
        _ => None,
    }
}

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    /// The program's screens.
    #[qenum(Screens)]
    enum Screen {
        Home,
        Consoles,
        Grid,
        Game,
        Pause,
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(Screen, shown, READ, NOTIFY)]
        #[qproperty(QString, key, READ, NOTIFY)]
        #[qproperty(bool, game_running, cxx_name = "gameRunning")]
        type Screens = super::ScreensRust;
    }

    extern "RustQt" {
        /// One of Home's places, by its index.
        #[qinvokable]
        fn place(self: Pin<&mut Screens>, index: i32);
        /// A row chosen on Choose a console: its grid.
        #[qinvokable]
        fn open(self: Pin<&mut Screens>, key: &QString);
        /// B that the shown screen did not keep.
        #[qinvokable]
        fn back(self: Pin<&mut Screens>);
        /// A screen's declared action, by its key.
        #[qinvokable]
        fn act(self: Pin<&mut Screens>, key: &QString);
        /// A game chosen, or Continue: the game's screen.
        #[qinvokable]
        fn play(self: Pin<&mut Screens>);
        /// The menu chord over the game: the Pause Card.
        #[qinvokable]
        fn menu(self: Pin<&mut Screens>);
        /// The Pause Card's Back to Library: Home over the suspended game.
        #[qinvokable]
        fn library(self: Pin<&mut Screens>);
    }

    impl cxx_qt::Threading for Screens {}
    impl cxx_qt::Initialize for Screens {}
}

use qobject::Screen;

pub struct ScreensRust {
    shown: Screen,
    key: QString,
    /// Whether the program's game runs: a game that starts is shown, and when it stops its screen gives way to Home.
    game_running: bool,
}

impl Default for ScreensRust {
    fn default() -> Self {
        ScreensRust { shown: Screen::Home, key: QString::default(), game_running: false }
    }
}

impl cxx_qt::Initialize for qobject::Screens {
    fn initialize(self: Pin<&mut Self>) {
        self.on_game_running_changed(|screens| {
            let ask = if screens.game_running { Ask::Play } else { Ask::Closed };
            screens.ask(ask);
        })
        .release();
    }
}

fn shown_of(screen: Screen) -> Shown {
    match screen {
        Screen::Consoles => Shown::Consoles,
        Screen::Grid => Shown::Grid,
        Screen::Game => Shown::Game,
        Screen::Pause => Shown::Pause,
        _ => Shown::Home,
    }
}

fn screen_of(shown: Shown) -> Screen {
    match shown {
        Shown::Home => Screen::Home,
        Shown::Consoles => Screen::Consoles,
        Shown::Grid => Screen::Grid,
        Shown::Game => Screen::Game,
        Shown::Pause => Screen::Pause,
    }
}

impl qobject::Screens {
    pub fn place(self: Pin<&mut Self>, index: i32) {
        self.ask(Ask::Place(index.max(0) as usize));
    }

    pub fn open(self: Pin<&mut Self>, key: &QString) {
        self.ask(Ask::Open(key.to_string()));
    }

    pub fn back(self: Pin<&mut Self>) {
        self.ask(Ask::Back);
    }

    pub fn act(self: Pin<&mut Self>, key: &QString) {
        self.ask(Ask::Act(key.to_string()));
    }

    pub fn play(self: Pin<&mut Self>) {
        self.ask(Ask::Play);
    }

    pub fn menu(self: Pin<&mut Self>) {
        self.ask(Ask::Menu);
    }

    pub fn library(self: Pin<&mut Self>) {
        self.ask(Ask::Library);
    }

    fn ask(self: Pin<&mut Self>, ask: Ask) {
        let _ = self.qt_thread().queue(move |screens| screens.apply(ask));
    }

    fn apply(mut self: Pin<&mut Self>, ask: Ask) {
        let Some((shown, key)) = step(shown_of(self.shown), &ask) else { return };
        if let Some(key) = key {
            let key = QString::from(key.as_str());
            if self.key != key {
                self.as_mut().rust_mut().key = key;
                self.as_mut().key_changed();
            }
        }
        let screen = screen_of(shown);
        if self.shown != screen {
            self.as_mut().rust_mut().shown = screen;
            self.as_mut().shown_changed();
        }
    }
}
