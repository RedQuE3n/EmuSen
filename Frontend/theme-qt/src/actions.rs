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
        /// A console's options: Settings opened at the console's page.
        #[qinvokable]
        #[cxx_name = "consoleOptions"]
        fn console_options(self: Pin<&mut ThemeActions>, id: &QString) -> bool;
        /// Opens a place of `places` or `shelves` by its id, of any kind.
        #[qinvokable]
        #[cxx_name = "openPlace"]
        fn open_place(self: Pin<&mut ThemeActions>, id: &QString) -> bool;
        /// Opens one of `Library.sections` by its key (1.8).
        #[qinvokable]
        #[cxx_name = "openSection"]
        fn open_section(self: Pin<&mut ThemeActions>, key: &QString) -> bool;
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
        /// Narrows the section shown to one of `Browse.filters` by its `key` (1.7, R-20261115-10).
        #[qinvokable]
        #[cxx_name = "setFilter"]
        fn set_filter(self: Pin<&mut ThemeActions>, key: &QString) -> bool;
        /// Steps the sort shown by `by`, as the List card's Sort row steps it (1.7, R-20261115-11).
        #[qinvokable]
        #[cxx_name = "stepSort"]
        fn step_sort(self: Pin<&mut ThemeActions>, by: i32) -> bool;
        /// Moves the focus to the first game of the letter `by` letters on, as the List card's Jump to (1.7, R-20261115-12).
        #[qinvokable]
        #[cxx_name = "stepLetter"]
        fn step_letter(self: Pin<&mut ThemeActions>, by: i32) -> bool;
        /// Whether the host would take `act(key)` now, without acting: a declined action's item and help entry look disabled.
        #[qinvokable]
        fn enabled(self: &ThemeActions, key: &QString) -> bool;
        /// Requests a face switch between Big Picture (`true`) and Desktop (`false`) modes (1.22).
        #[qinvokable]
        #[cxx_name = "setBigScreen"]
        fn set_big_screen(self: Pin<&mut ThemeActions>, big_screen: bool) -> bool;

        /// Settings' calls, each the program's own (EmuSen_Qt.md §3.2, `Settings`).
        #[qinvokable]
        #[cxx_name = "settingsStep"]
        fn settings_step(self: Pin<&mut ThemeActions>, row: i32, by: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "settingsChoose"]
        fn settings_choose(self: Pin<&mut ThemeActions>, row: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "settingsSlide"]
        fn settings_slide(self: Pin<&mut ThemeActions>, row: i32, amount: f64) -> bool;
        #[qinvokable]
        #[cxx_name = "settingsSection"]
        fn settings_section(self: Pin<&mut ThemeActions>, section: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "settingsFocus"]
        fn settings_focus(self: Pin<&mut ThemeActions>, row: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "settingsShoulder"]
        fn settings_shoulder(self: Pin<&mut ThemeActions>, by: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "settingsBack"]
        fn settings_back(self: Pin<&mut ThemeActions>) -> bool;
        /// Selects an inline kind pill on Settings -> Library -> Themes (1.21, board #61).
        #[qinvokable]
        #[cxx_name = "settingsInlinePill"]
        fn settings_inline_pill(self: Pin<&mut ThemeActions>, index: i32) -> bool;

        /// A game's page's calls, each the program's own (EmuSen_Qt.md §3.2, `Page`).
        #[qinvokable]
        #[cxx_name = "pageSection"]
        fn page_section(self: Pin<&mut ThemeActions>, section: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "pageAct"]
        fn page_act(self: Pin<&mut ThemeActions>, action: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "pageStepAction"]
        fn page_step_action(self: Pin<&mut ThemeActions>, action: i32, by: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "pageChoose"]
        fn page_choose(self: Pin<&mut ThemeActions>, row: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "pageStep"]
        fn page_step(self: Pin<&mut ThemeActions>, row: i32, by: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "pageFocus"]
        fn page_focus(self: Pin<&mut ThemeActions>, action: i32, row: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "pageDeclared"]
        fn page_declared(self: Pin<&mut ThemeActions>, key: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "pageAnswer"]
        fn page_answer(self: Pin<&mut ThemeActions>, choice: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "pageBack"]
        fn page_back(self: Pin<&mut ThemeActions>) -> bool;
        /// One of `Page.tabs` chosen by its place (1.13).
        #[qinvokable]
        #[cxx_name = "pageTab"]
        fn page_tab(self: Pin<&mut ThemeActions>, tab: i32) -> bool;
        /// A key on the media viewer (1.19, R-20261123-32).
        #[qinvokable]
        #[cxx_name = "pageViewKey"]
        fn page_view_key(self: Pin<&mut ThemeActions>, key: i32, modifiers: i32) -> bool;
        /// The PDF manual's page count reported to the viewer (1.19, R-20261123-32).
        #[qinvokable]
        #[cxx_name = "pageViewPages"]
        fn page_view_pages(self: Pin<&mut ThemeActions>, pages: i32) -> bool;
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
    ConsoleOptions(String),
    OpenPlace(String),
    OpenSection(String),
    Home,
    Back,
    StepSection(i32),
    OpenHomeCard,
    OpenListCard,
    Search,
    Act(String),
    Focus(String),
    SetFilter(String),
    StepSort(i32),
    StepLetter(i32),
}

/// A change a theme's Settings or game page asks for, each a call of the program's model for that screen.
#[derive(Clone, Debug, PartialEq)]
pub enum ScreenAction {
    SettingsStep(i32, i32),
    SettingsChoose(i32),
    SettingsSlide(i32, f64),
    SettingsSection(i32),
    SettingsFocus(i32),
    SettingsShoulder(i32),
    SettingsBack,
    SettingsInlinePill(i32),
    PageSection(i32),
    PageAct(i32),
    PageStepAction(i32, i32),
    PageChoose(i32),
    PageStep(i32, i32),
    PageFocus(i32, i32),
    PageDeclared(String),
    PageAnswer(i32),
    PageBack,
    PageTab(i32),
    PageViewKey(i32, i32),
    PageViewPages(i32),
}

/// The host's side: takes an action, or declines it, and says which.
pub trait ActionSink {
    fn act(&mut self, action: &Action) -> bool;
    /// A screen's call; a host that draws neither screen for a theme declines them all.
    fn screen(&mut self, _action: &ScreenAction) -> bool {
        false
    }
    /// Switches between Big Picture and Desktop faces; a host that does not toggle faces declines it.
    fn set_big_screen(&mut self, _big_screen: bool) -> bool {
        false
    }
    /// Whether `act` would take the action, asked without acting; a host that declines nothing need not say.
    fn would(&self, _action: &Action) -> bool {
        true
    }
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

    fn call(mut self: Pin<&mut Self>, action: ScreenAction) -> bool {
        self.as_mut().rust_mut().sink.as_mut().is_some_and(|sink| sink.screen(&action))
    }

    pub fn settings_step(self: Pin<&mut Self>, row: i32, by: i32) -> bool {
        self.call(ScreenAction::SettingsStep(row, by))
    }

    pub fn settings_choose(self: Pin<&mut Self>, row: i32) -> bool {
        self.call(ScreenAction::SettingsChoose(row))
    }

    pub fn settings_slide(self: Pin<&mut Self>, row: i32, amount: f64) -> bool {
        self.call(ScreenAction::SettingsSlide(row, amount))
    }

    pub fn settings_section(self: Pin<&mut Self>, section: i32) -> bool {
        self.call(ScreenAction::SettingsSection(section))
    }

    pub fn settings_focus(self: Pin<&mut Self>, row: i32) -> bool {
        self.call(ScreenAction::SettingsFocus(row))
    }

    pub fn settings_shoulder(self: Pin<&mut Self>, by: i32) -> bool {
        self.call(ScreenAction::SettingsShoulder(by))
    }

    pub fn settings_back(self: Pin<&mut Self>) -> bool {
        self.call(ScreenAction::SettingsBack)
    }

    pub fn settings_inline_pill(self: Pin<&mut Self>, index: i32) -> bool {
        self.call(ScreenAction::SettingsInlinePill(index))
    }

    pub fn page_section(self: Pin<&mut Self>, section: i32) -> bool {
        self.call(ScreenAction::PageSection(section))
    }

    pub fn page_act(self: Pin<&mut Self>, action: i32) -> bool {
        self.call(ScreenAction::PageAct(action))
    }

    pub fn page_step_action(self: Pin<&mut Self>, action: i32, by: i32) -> bool {
        self.call(ScreenAction::PageStepAction(action, by))
    }

    pub fn page_choose(self: Pin<&mut Self>, row: i32) -> bool {
        self.call(ScreenAction::PageChoose(row))
    }

    pub fn page_step(self: Pin<&mut Self>, row: i32, by: i32) -> bool {
        self.call(ScreenAction::PageStep(row, by))
    }

    pub fn page_focus(self: Pin<&mut Self>, action: i32, row: i32) -> bool {
        self.call(ScreenAction::PageFocus(action, row))
    }

    pub fn page_declared(self: Pin<&mut Self>, key: &QString) -> bool {
        self.call(ScreenAction::PageDeclared(key.to_string()))
    }

    pub fn page_answer(self: Pin<&mut Self>, choice: i32) -> bool {
        self.call(ScreenAction::PageAnswer(choice))
    }

    pub fn page_back(self: Pin<&mut Self>) -> bool {
        self.call(ScreenAction::PageBack)
    }

    pub fn page_tab(self: Pin<&mut Self>, tab: i32) -> bool {
        self.call(ScreenAction::PageTab(tab))
    }

    pub fn page_view_key(self: Pin<&mut Self>, key: i32, modifiers: i32) -> bool {
        self.call(ScreenAction::PageViewKey(key, modifiers))
    }

    pub fn page_view_pages(self: Pin<&mut Self>, pages: i32) -> bool {
        self.call(ScreenAction::PageViewPages(pages))
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

    pub fn console_options(self: Pin<&mut Self>, id: &QString) -> bool {
        self.take(Action::ConsoleOptions(id.to_string()))
    }

    pub fn open_place(self: Pin<&mut Self>, id: &QString) -> bool {
        self.take(Action::OpenPlace(id.to_string()))
    }

    pub fn open_section(self: Pin<&mut Self>, key: &QString) -> bool {
        self.take(Action::OpenSection(key.to_string()))
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

    pub fn set_filter(self: Pin<&mut Self>, key: &QString) -> bool {
        self.take(Action::SetFilter(key.to_string()))
    }

    pub fn step_sort(self: Pin<&mut Self>, by: i32) -> bool {
        self.take(Action::StepSort(by))
    }

    pub fn step_letter(self: Pin<&mut Self>, by: i32) -> bool {
        self.take(Action::StepLetter(by))
    }

    pub fn enabled(&self, key: &QString) -> bool {
        self.sink.as_ref().is_some_and(|sink| sink.would(&Action::Act(key.to_string())))
    }

    pub fn set_big_screen(mut self: Pin<&mut Self>, big_screen: bool) -> bool {
        self.as_mut().rust_mut().sink.as_mut().is_some_and(|sink| sink.set_big_screen(big_screen))
    }
}
