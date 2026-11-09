//! `Screens`: which of the program's screens is shown and which row the grid opened, so the screens switch by calls,
//! never by logic in QML (EmuSen_Qt.md §7.1). `step` is the whole rule; the bridge only puts it on Qt, after the event
//! that asked, so one press is never answered by the screen it left and the screen it opened.

use crate::help::{HOME, NEXT, PREVIOUS};
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
    /// The Home card over Home, opened at its Power page by Home's place (EmuSen_BigPicture.md §51.2).
    HomeCard,
    /// The List card over the grid (EmuSen_Interface.md §18.1).
    List,
    /// The grid's search, the keyboard docked and holding the focus.
    Search,
    /// The grid's search, the keyboard docked and the focus on the covers (LunaP's `GalleryLayout` out of the keyboard).
    SearchGrid,
    /// The ES-DE theme's view in place of Home, Choose a console and the grid, its system and gamelist views one screen
    /// whose model says which (EmuSen_Qt.md §4.6).
    Esde,
    /// The Themes tab, from the Home card's Settings row until Settings is built (EmuSen_Qt.md §3.8).
    Themes,
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
    /// A card closed by B on its first page or by Start.
    Close,
    /// The focus went to the search's keys (true) or its covers (false) by other means than the shoulders: the pointer.
    Keys(bool),
    /// The library's look changed between the own look and an ES-DE theme: the library's screen shown is the other's.
    Restyled,
}

/// Home's Settings and Power places, by their index in `words::HOME_PLACES`.
pub const SETTINGS_PLACE: usize = 5;
pub const POWER_PLACE: usize = 6;

/// The grid's List card and the search, by the actions that open them.
pub const LIST: &str = "list";
pub const SEARCH: &str = "search";
/// A shoulder on the keyboard: the focus to the covers, while there are any.
pub const TO_GRID: &str = "toGrid";

/// The screen shown after an ask, and the grid's row (the page a card opens at, for a card); `None` leaves both as they
/// were.
pub fn step(shown: Shown, ask: &Ask) -> Option<(Shown, Option<String>)> {
    match (shown, ask) {
        (Shown::Home, Ask::Place(0)) => Some((Shown::Consoles, None)),
        // Settings is the Themes tab until Settings is built, from Home's place and the Home card's row alike.
        (Shown::Home, Ask::Place(SETTINGS_PLACE)) => Some((Shown::Themes, None)),
        (Shown::Home, Ask::Place(POWER_PLACE)) => Some((Shown::HomeCard, Some(crate::home_card::POWER.to_owned()))),
        (Shown::Home, Ask::Act(key)) if key == SEARCH => Some((Shown::Search, Some(crate::library::ALL.to_owned()))),
        // Start on Home stays on Home, which takes the focus back to where it starts.
        (Shown::Home, Ask::Act(key)) if key == HOME => Some((Shown::Home, None)),
        (Shown::Consoles | Shown::Grid | Shown::Search | Shown::SearchGrid, Ask::Act(key)) if key == HOME => Some((Shown::Home, None)),
        (Shown::HomeCard, Ask::Back | Ask::Close) => Some((Shown::Home, None)),
        (Shown::HomeCard, Ask::Act(key)) if key == crate::home_card::SETTINGS => Some((Shown::Themes, None)),
        // The card closes on its rows' actions; those of a later stage (Exit Big Picture, the tools) do nothing more yet.
        (Shown::HomeCard, Ask::Act(_)) => Some((Shown::Home, None)),
        (Shown::Themes, Ask::Back) => Some((Shown::Home, None)),
        (Shown::Themes, Ask::Act(key)) if key == HOME => Some((Shown::Home, None)),
        (Shown::Grid, Ask::Act(key)) if key == LIST => Some((Shown::List, None)),
        (Shown::List, Ask::Back | Ask::Close) => Some((Shown::Grid, None)),
        (Shown::List, Ask::Act(key)) if key == SEARCH => Some((Shown::Search, None)),
        // Every other action of the List card is the grid's, and the card closes on it.
        (Shown::List, Ask::Act(_)) => Some((Shown::Grid, None)),
        // The keyboard closed, by X keeping the words or by B on an empty field: the grid.
        (Shown::Search, Ask::Back) => Some((Shown::Grid, None)),
        // The shoulders move the focus between the keyboard and the covers, the keyboard staying docked.
        (Shown::Search, Ask::Act(key)) if key == TO_GRID => Some((Shown::SearchGrid, None)),
        (Shown::SearchGrid, Ask::Act(key)) if key == PREVIOUS || key == NEXT => Some((Shown::Search, None)),
        (Shown::Search, Ask::Keys(false)) => Some((Shown::SearchGrid, None)),
        (Shown::SearchGrid, Ask::Keys(true)) => Some((Shown::Search, None)),
        // From the covers B is the grid's: the search cleared first (GameModel), then Choose a console.
        (Shown::SearchGrid, Ask::Back) => Some((Shown::Consoles, None)),
        (Shown::SearchGrid, Ask::Act(key)) if key == LIST => Some((Shown::List, None)),
        (Shown::Consoles, Ask::Open(key)) => Some((Shown::Grid, Some(key.clone()))),
        (Shown::Consoles, Ask::Act(key)) if key == crate::library::FAVOURITES => Some((Shown::Grid, Some(key.clone()))),
        (Shown::Consoles, Ask::Back) => Some((Shown::Home, None)),
        (Shown::Grid, Ask::Back) => Some((Shown::Consoles, None)),
        (Shown::Home | Shown::Grid | Shown::Pause | Shown::List | Shown::Search | Shown::SearchGrid, Ask::Play) => Some((Shown::Game, None)),
        (Shown::Game, Ask::Menu) => Some((Shown::Pause, None)),
        (Shown::Pause, Ask::Library) => Some((Shown::Home, None)),
        (Shown::Game | Shown::Pause, Ask::Closed) => Some((Shown::Home, None)),
        _ => None,
    }
}

/// The library's screen of a look: Home for the own look, the theme's view for an ES-DE theme.
pub fn library_of(themed: bool) -> Shown {
    if themed {
        Shown::Esde
    } else {
        Shown::Home
    }
}

/// As `step`, under the library's look: with an ES-DE theme the theme's view stands where Home, Choose a console and
/// the grid stood, and the cards, the game and the Pause Card come back to it (EmuSen_Qt.md §4.6).
pub fn step_in(themed: bool, shown: Shown, ask: &Ask) -> Option<(Shown, Option<String>)> {
    let own = matches!(shown, Shown::Home | Shown::Consoles | Shown::Grid | Shown::Search | Shown::SearchGrid);
    match (themed, shown, ask) {
        (true, _, Ask::Restyled) if own => Some((Shown::Esde, None)),
        (false, Shown::Esde, Ask::Restyled) => Some((Shown::Home, None)),
        (_, _, Ask::Restyled) => None,
        (false, ..) => step(shown, ask),
        // Start opens the Home card at its first page, as ES-DE's Start opens its menu; B at the theme's view is the
        // view's own, going back from a gamelist, and does nothing more here (decided 2026-11-08).
        (true, Shown::Esde, Ask::Act(key)) if key == HOME => Some((Shown::HomeCard, Some(crate::home_card::FIRST.to_owned()))),
        (true, Shown::Esde, Ask::Act(key)) if key == LIST => Some((Shown::List, None)),
        (true, Shown::Esde | Shown::List | Shown::Pause, Ask::Play) => Some((Shown::Game, None)),
        // The cards close to the theme's view on B, Start or any of their actions, which the host carries out.
        (true, Shown::HomeCard | Shown::List, Ask::Back | Ask::Close | Ask::Act(_)) => Some((Shown::Esde, None)),
        (true, Shown::Game, Ask::Menu) => Some((Shown::Pause, None)),
        (true, Shown::Pause, Ask::Library) => Some((Shown::Esde, None)),
        (true, Shown::Game | Shown::Pause, Ask::Closed) => Some((Shown::Esde, None)),
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
        HomeCard,
        List,
        Search,
        SearchGrid,
        Esde,
        Themes,
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(Screen, shown, READ, NOTIFY)]
        #[qproperty(QString, key, READ, NOTIFY)]
        #[qproperty(bool, game_running, cxx_name = "gameRunning")]
        /// Counts the times Home is asked for while it shows, so Home takes the focus back to where it starts.
        #[qproperty(i32, home_again, cxx_name = "homeAgain", READ, NOTIFY)]
        /// The page the Home card opens at, by its key.
        #[qproperty(QString, page, READ, NOTIFY)]
        /// Whether the grid shows a game, so the keyboard's shoulders have covers to move to.
        #[qproperty(bool, found)]
        /// Whether an ES-DE theme is the library's look (LibraryModel.themed): its view stands for the library's screens.
        #[qproperty(bool, themed)]
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
        /// A card closed by B on its first page or by Start.
        #[qinvokable]
        fn close(self: Pin<&mut Screens>);
        /// Start at the ES-DE theme's view: the Home card at its first page, as ES-DE's Start opens its menu.
        #[qinvokable]
        fn home(self: Pin<&mut Screens>);
        /// Select at the ES-DE theme's view, its options: the List card.
        #[qinvokable]
        fn list(self: Pin<&mut Screens>);
        /// The search's focus taken by its keys or its covers.
        #[qinvokable]
        #[cxx_name = "keysFocused"]
        fn keys_focused(self: Pin<&mut Screens>, keys: bool);
        /// Quit EmuSen answered Yes: the program ends.
        #[qsignal]
        fn quit(self: Pin<&mut Screens>);
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
    home_again: i32,
    page: QString,
    found: bool,
    themed: bool,
}

impl Default for ScreensRust {
    fn default() -> Self {
        ScreensRust { shown: Screen::Home, key: QString::default(), game_running: false, home_again: 0, page: QString::default(), found: false, themed: false }
    }
}

impl cxx_qt::Initialize for qobject::Screens {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut()
            .on_game_running_changed(|screens| {
                let ask = if screens.game_running { Ask::Play } else { Ask::Closed };
                screens.ask(ask);
            })
            .release();
        self.on_themed_changed(|screens| screens.ask(Ask::Restyled)).release();
    }
}

fn shown_of(screen: Screen) -> Shown {
    match screen {
        Screen::Consoles => Shown::Consoles,
        Screen::Grid => Shown::Grid,
        Screen::Game => Shown::Game,
        Screen::Pause => Shown::Pause,
        Screen::HomeCard => Shown::HomeCard,
        Screen::List => Shown::List,
        Screen::Search => Shown::Search,
        Screen::SearchGrid => Shown::SearchGrid,
        Screen::Esde => Shown::Esde,
        Screen::Themes => Shown::Themes,
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
        Shown::HomeCard => Screen::HomeCard,
        Shown::List => Screen::List,
        Shown::Search => Screen::Search,
        Shown::SearchGrid => Screen::SearchGrid,
        Shown::Esde => Screen::Esde,
        Shown::Themes => Screen::Themes,
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

    pub fn act(mut self: Pin<&mut Self>, key: &QString) {
        let key = key.to_string();
        if key == crate::home_card::QUIT_EMUSEN {
            self.as_mut().quit();
        }
        if key == TO_GRID && !self.found {
            return;
        }
        self.ask(Ask::Act(key));
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

    pub fn close(self: Pin<&mut Self>) {
        self.ask(Ask::Close);
    }

    pub fn home(self: Pin<&mut Self>) {
        self.ask(Ask::Act(HOME.to_owned()));
    }

    pub fn list(self: Pin<&mut Self>) {
        self.ask(Ask::Act(LIST.to_owned()));
    }

    pub fn keys_focused(self: Pin<&mut Self>, keys: bool) {
        self.ask(Ask::Keys(keys));
    }

    fn ask(self: Pin<&mut Self>, ask: Ask) {
        let _ = self.qt_thread().queue(move |screens| screens.apply(ask));
    }

    fn apply(mut self: Pin<&mut Self>, ask: Ask) {
        let Some((shown, key)) = step_in(self.themed, shown_of(self.shown), &ask) else { return };
        if let (Shown::HomeCard, Some(page)) = (shown, &key) {
            let page = QString::from(page.as_str());
            if self.page != page {
                self.as_mut().rust_mut().page = page;
                self.as_mut().page_changed();
            }
        } else if let Some(key) = key {
            let key = QString::from(key.as_str());
            if self.key != key {
                self.as_mut().rust_mut().key = key;
                self.as_mut().key_changed();
            }
        }
        let screen = screen_of(shown);
        if self.shown == Screen::Home && screen == Screen::Home {
            let again = self.home_again.wrapping_add(1);
            self.as_mut().rust_mut().home_again = again;
            self.as_mut().home_again_changed();
        }
        if self.shown != screen {
            self.as_mut().rust_mut().shown = screen;
            self.as_mut().shown_changed();
        }
    }
}
