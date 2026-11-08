//! `Actions`, every change a theme can ask for (EmuSen_Qt.md §3.2): each passed to the host's sink, which says whether it
//! was taken. With no sink nothing is taken. The models stay read-only; this is the one way out of a theme.

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element = "Actions"]
        #[qml_singleton]
        type ThemeActions = super::ThemeActionsRust;
    }

    extern "RustQt" {
        /// Plays the game of a `gamesOf` row's `id`.
        #[qinvokable]
        fn play(self: Pin<&mut ThemeActions>, game: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "openPage"]
        fn open_page(self: Pin<&mut ThemeActions>, game: &QString) -> bool;
        #[qinvokable]
        fn favourite(self: Pin<&mut ThemeActions>, game: &QString) -> bool;
        /// A random game of a console, or of every console for an empty id.
        #[qinvokable]
        fn random(self: Pin<&mut ThemeActions>, console: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "openConsole"]
        fn open_console(self: Pin<&mut ThemeActions>, id: &QString) -> bool;
        /// Opens a place of `places` or `shelves` by its id, of any kind.
        #[qinvokable]
        #[cxx_name = "openPlace"]
        fn open_place(self: Pin<&mut ThemeActions>, id: &QString) -> bool;
        #[qinvokable]
        fn home(self: Pin<&mut ThemeActions>) -> bool;
        #[qinvokable]
        fn back(self: Pin<&mut ThemeActions>) -> bool;
        #[qinvokable]
        #[cxx_name = "stepSection"]
        fn step_section(self: Pin<&mut ThemeActions>, by: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "openHomeCard"]
        fn open_home_card(self: Pin<&mut ThemeActions>) -> bool;
        #[qinvokable]
        #[cxx_name = "openListCard"]
        fn open_list_card(self: Pin<&mut ThemeActions>) -> bool;
        #[qinvokable]
        fn search(self: Pin<&mut ThemeActions>) -> bool;
        /// A help bar's declared action, by the key `Frame`'s `actionInvoked` gives (EmuSen_Kit.md §7.19).
        #[qinvokable]
        fn act(self: Pin<&mut ThemeActions>, key: &QString) -> bool;
        /// The game the focus is on, by its row's `key`, so the cards and the shoulders act on it.
        #[qinvokable]
        fn focus(self: Pin<&mut ThemeActions>, game: &QString) -> bool;
    }
}

/// A change a theme asks for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Play(String),
    OpenPage(String),
    Favourite(String),
    Random(String),
    OpenConsole(String),
    OpenPlace(String),
    Home,
    Back,
    StepSection(i32),
    OpenHomeCard,
    OpenListCard,
    Search,
    Act(String),
    Focus(String),
}

/// The host's side: takes an action, or declines it, and says which.
pub trait ActionSink {
    fn act(&mut self, action: &Action) -> bool;
}

#[derive(Default)]
pub struct ThemeActionsRust {
    sink: Option<Box<dyn ActionSink>>,
}

impl qobject::ThemeActions {
    /// The host's sink; none declines every action.
    pub fn set_sink(mut self: Pin<&mut Self>, sink: Option<Box<dyn ActionSink>>) {
        self.as_mut().rust_mut().sink = sink;
    }

    fn take(mut self: Pin<&mut Self>, action: Action) -> bool {
        self.as_mut().rust_mut().sink.as_mut().is_some_and(|sink| sink.act(&action))
    }

    pub fn play(self: Pin<&mut Self>, game: &QString) -> bool {
        self.take(Action::Play(game.to_string()))
    }

    pub fn open_page(self: Pin<&mut Self>, game: &QString) -> bool {
        self.take(Action::OpenPage(game.to_string()))
    }

    pub fn favourite(self: Pin<&mut Self>, game: &QString) -> bool {
        self.take(Action::Favourite(game.to_string()))
    }

    pub fn random(self: Pin<&mut Self>, console: &QString) -> bool {
        self.take(Action::Random(console.to_string()))
    }

    pub fn open_console(self: Pin<&mut Self>, id: &QString) -> bool {
        self.take(Action::OpenConsole(id.to_string()))
    }

    pub fn open_place(self: Pin<&mut Self>, id: &QString) -> bool {
        self.take(Action::OpenPlace(id.to_string()))
    }

    pub fn home(self: Pin<&mut Self>) -> bool {
        self.take(Action::Home)
    }

    pub fn back(self: Pin<&mut Self>) -> bool {
        self.take(Action::Back)
    }

    pub fn step_section(self: Pin<&mut Self>, by: i32) -> bool {
        self.take(Action::StepSection(by))
    }

    pub fn open_home_card(self: Pin<&mut Self>) -> bool {
        self.take(Action::OpenHomeCard)
    }

    pub fn open_list_card(self: Pin<&mut Self>) -> bool {
        self.take(Action::OpenListCard)
    }

    pub fn search(self: Pin<&mut Self>) -> bool {
        self.take(Action::Search)
    }

    pub fn act(self: Pin<&mut Self>, key: &QString) -> bool {
        self.take(Action::Act(key.to_string()))
    }

    pub fn focus(self: Pin<&mut Self>, game: &QString) -> bool {
        self.take(Action::Focus(game.to_string()))
    }
}
