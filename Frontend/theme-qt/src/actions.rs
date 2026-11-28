//! `Actions`, every change a theme can ask for (EmuSen_Qt.md §3.2): each passed to the host's sink, which says whether it
//! was taken. With no sink nothing is taken. The models stay read-only; this is the one way out of a theme.

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QString, QStringList};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
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

        /// Desktop Workspace calls (1.23).
        #[qinvokable]
        #[cxx_name = "workspaceView"]
        fn workspace_view(self: Pin<&mut ThemeActions>, view: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "workspaceSetSidebar"]
        fn workspace_set_sidebar(self: Pin<&mut ThemeActions>, open: bool) -> bool;
        #[qinvokable]
        #[cxx_name = "workspaceSetInspector"]
        fn workspace_set_inspector(self: Pin<&mut ThemeActions>, open: bool) -> bool;
        #[qinvokable]
        #[cxx_name = "workspaceSetDrawer"]
        fn workspace_set_drawer(self: Pin<&mut ThemeActions>, open: bool) -> bool;
        #[qinvokable]
        #[cxx_name = "workspaceSetDrawerTab"]
        fn workspace_set_drawer_tab(self: Pin<&mut ThemeActions>, tab: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "workspaceSelectConsole"]
        fn workspace_select_console(self: Pin<&mut ThemeActions>, key: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "workspaceSetConsoleFilter"]
        fn workspace_set_console_filter(self: Pin<&mut ThemeActions>, filter: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "workspaceSetConsoleScope"]
        fn workspace_set_console_scope(self: Pin<&mut ThemeActions>, scope: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "workspaceSetFilter"]
        fn workspace_set_filter(self: Pin<&mut ThemeActions>, key: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "workspaceSetSearch"]
        fn workspace_set_search(self: Pin<&mut ThemeActions>, query: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "workspaceSetSort"]
        fn workspace_set_sort(self: Pin<&mut ThemeActions>, index: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "workspaceSortByColumn"]
        fn workspace_sort_by_column(self: Pin<&mut ThemeActions>, column: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "workspaceSelectGame"]
        fn workspace_select_game(self: Pin<&mut ThemeActions>, key: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "workspaceMarkToggle"]
        fn workspace_mark_toggle(self: Pin<&mut ThemeActions>, key: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "workspaceMarkAll"]
        fn workspace_mark_all(self: Pin<&mut ThemeActions>) -> bool;
        #[qinvokable]
        #[cxx_name = "workspaceClearMarked"]
        fn workspace_clear_marked(self: Pin<&mut ThemeActions>) -> bool;
        #[qinvokable]
        #[cxx_name = "workspaceBatchFavourite"]
        fn workspace_batch_favourite(self: Pin<&mut ThemeActions>, on: bool) -> bool;
        #[qinvokable]
        #[cxx_name = "workspaceClearFilters"]
        fn workspace_clear_filters(self: Pin<&mut ThemeActions>) -> bool;
        #[qinvokable]
        #[cxx_name = "workspaceOpenFolder"]
        fn workspace_open_folder(self: Pin<&mut ThemeActions>, key: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "playFrom"]
        fn play_from(self: Pin<&mut ThemeActions>, game: &QString, state: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "openDesktopDoor"]
        fn open_desktop_door(self: Pin<&mut ThemeActions>, key: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "openTool"]
        fn open_tool(self: Pin<&mut ThemeActions>, key: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "toggleFullScreen"]
        fn toggle_full_screen(self: Pin<&mut ThemeActions>) -> bool;
        #[qinvokable]
        #[cxx_name = "addGamesToCollection"]
        fn add_games_to_collection(self: Pin<&mut ThemeActions>, games: &QStringList, collection_id: &QString) -> bool;
        /// Desktop Workspace library and shaders doors (1.24).
        #[qinvokable]
        #[cxx_name = "workspaceLibrary"]
        fn workspace_library(self: Pin<&mut ThemeActions>) -> bool;
        #[qinvokable]
        #[cxx_name = "workspaceShaders"]
        fn workspace_shaders(self: Pin<&mut ThemeActions>) -> bool;

        /// Live Session calls (1.23).
        #[qinvokable]
        #[cxx_name = "sessionPause"]
        fn session_pause(self: Pin<&mut ThemeActions>) -> bool;
        #[qinvokable]
        #[cxx_name = "sessionResume"]
        fn session_resume(self: Pin<&mut ThemeActions>) -> bool;
        #[qinvokable]
        #[cxx_name = "sessionRestart"]
        fn session_restart(self: Pin<&mut ThemeActions>) -> bool;
        #[qinvokable]
        #[cxx_name = "sessionClose"]
        fn session_close(self: Pin<&mut ThemeActions>) -> bool;
        #[qinvokable]
        #[cxx_name = "sessionScreenshot"]
        fn session_screenshot(self: Pin<&mut ThemeActions>) -> bool;
        #[qinvokable]
        #[cxx_name = "sessionChooseSlot"]
        fn session_choose_slot(self: Pin<&mut ThemeActions>, slot: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "sessionSaveState"]
        fn session_save_state(self: Pin<&mut ThemeActions>, slot: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "sessionLoadState"]
        fn session_load_state(self: Pin<&mut ThemeActions>, slot: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "sessionChooseSpeed"]
        fn session_choose_speed(self: Pin<&mut ThemeActions>, percent: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "sessionRewindTo"]
        fn session_rewind_to(self: Pin<&mut ThemeActions>, frame: f64) -> bool;
        #[qinvokable]
        #[cxx_name = "sessionSetMasterCheats"]
        fn session_set_master_cheats(self: Pin<&mut ThemeActions>, on: bool) -> bool;
        #[qinvokable]
        #[cxx_name = "sessionSwitchCheat"]
        fn session_switch_cheat(self: Pin<&mut ThemeActions>, index: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "sessionStepConsoleOption"]
        fn session_step_console_option(self: Pin<&mut ThemeActions>, key: &QString, by: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "sessionSwapPlayers"]
        fn session_swap_players(self: Pin<&mut ThemeActions>) -> bool;
        #[qinvokable]
        #[cxx_name = "sessionAnswerResume"]
        fn session_answer_resume(self: Pin<&mut ThemeActions>, resume: bool) -> bool;

        /// Desktop Settings, Themes, and Tools calls (1.23).
        #[qinvokable]
        #[cxx_name = "settingsSetFilter"]
        fn settings_set_filter(self: Pin<&mut ThemeActions>, filter: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "settingsSetModifiedOnly"]
        fn settings_set_modified_only(self: Pin<&mut ThemeActions>, on: bool) -> bool;
        #[qinvokable]
        #[cxx_name = "settingsOpenTool"]
        fn settings_open_tool(self: Pin<&mut ThemeActions>, index: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "themesSetKind"]
        fn themes_set_kind(self: Pin<&mut ThemeActions>, kind: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "themesFocus"]
        fn themes_focus(self: Pin<&mut ThemeActions>, row: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "themesUse"]
        fn themes_use(self: Pin<&mut ThemeActions>, id: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "themesStep"]
        fn themes_step(self: Pin<&mut ThemeActions>, row: i32, by: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "themesCheck"]
        fn themes_check(self: Pin<&mut ThemeActions>, id: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "toolsFocus"]
        fn tools_focus(self: Pin<&mut ThemeActions>, index: i32) -> bool;
        #[qinvokable]
        #[cxx_name = "toolsOpen"]
        fn tools_open(self: Pin<&mut ThemeActions>, key: &QString) -> bool;
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
    SettingsSetFilter(String),
    SettingsSetModifiedOnly(bool),
    SettingsOpenTool(i32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkspaceAction {
    View(i32),
    SetSidebar(bool),
    SetInspector(bool),
    SetDrawer(bool),
    SetDrawerTab(i32),
    SelectConsole(String),
    SetConsoleFilter(String),
    SetConsoleScope(i32),
    SetFilter(String),
    SetSearch(String),
    SetSort(i32),
    SortByColumn(i32),
    SelectGame(String),
    MarkToggle(String),
    MarkAll,
    ClearMarked,
    BatchFavourite(bool),
    ClearFilters,
    OpenFolder(String),
    PlayFrom(String, String),
    OpenDesktopDoor(String),
    OpenTool(String),
    ToggleFullScreen,
    AddGamesToCollection(Vec<String>, String),
    OpenLibrary,
    OpenShaders,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SessionAction {
    Pause,
    Resume,
    Restart,
    Close,
    Screenshot,
    ChooseSlot(i32),
    SaveState(i32),
    LoadState(i32),
    ChooseSpeed(i32),
    RewindTo(f64),
    SetMasterCheats(bool),
    SwitchCheat(i32),
    StepConsoleOption(String, i32),
    SwapPlayers,
    AnswerResume(bool),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThemesAction {
    SetKind(String),
    Focus(i32),
    Use(String),
    Step(i32, i32),
    Check(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolsAction {
    Focus(i32),
    Open(String),
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
    fn workspace(&mut self, _action: &WorkspaceAction) -> bool {
        false
    }
    fn session(&mut self, _action: &SessionAction) -> bool {
        false
    }
    fn themes(&mut self, _action: &ThemesAction) -> bool {
        false
    }
    fn tools(&mut self, _action: &ToolsAction) -> bool {
        false
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

    fn call_workspace(mut self: Pin<&mut Self>, action: WorkspaceAction) -> bool {
        self.as_mut().rust_mut().sink.as_mut().is_some_and(|sink| sink.workspace(&action))
    }

    fn call_session(mut self: Pin<&mut Self>, action: SessionAction) -> bool {
        self.as_mut().rust_mut().sink.as_mut().is_some_and(|sink| sink.session(&action))
    }

    fn call_themes(mut self: Pin<&mut Self>, action: ThemesAction) -> bool {
        self.as_mut().rust_mut().sink.as_mut().is_some_and(|sink| sink.themes(&action))
    }

    fn call_tools(mut self: Pin<&mut Self>, action: ToolsAction) -> bool {
        self.as_mut().rust_mut().sink.as_mut().is_some_and(|sink| sink.tools(&action))
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

    pub fn settings_set_filter(self: Pin<&mut Self>, filter: &QString) -> bool {
        self.call(ScreenAction::SettingsSetFilter(filter.to_string()))
    }

    pub fn settings_set_modified_only(self: Pin<&mut Self>, on: bool) -> bool {
        self.call(ScreenAction::SettingsSetModifiedOnly(on))
    }

    pub fn settings_open_tool(self: Pin<&mut Self>, index: i32) -> bool {
        self.call(ScreenAction::SettingsOpenTool(index))
    }

    pub fn workspace_view(self: Pin<&mut Self>, view: i32) -> bool {
        self.call_workspace(WorkspaceAction::View(view))
    }

    pub fn workspace_set_sidebar(self: Pin<&mut Self>, open: bool) -> bool {
        self.call_workspace(WorkspaceAction::SetSidebar(open))
    }

    pub fn workspace_set_inspector(self: Pin<&mut Self>, open: bool) -> bool {
        self.call_workspace(WorkspaceAction::SetInspector(open))
    }

    pub fn workspace_set_drawer(self: Pin<&mut Self>, open: bool) -> bool {
        self.call_workspace(WorkspaceAction::SetDrawer(open))
    }

    pub fn workspace_set_drawer_tab(self: Pin<&mut Self>, tab: i32) -> bool {
        self.call_workspace(WorkspaceAction::SetDrawerTab(tab))
    }

    pub fn workspace_select_console(self: Pin<&mut Self>, key: &QString) -> bool {
        self.call_workspace(WorkspaceAction::SelectConsole(key.to_string()))
    }

    pub fn workspace_set_console_filter(self: Pin<&mut Self>, filter: &QString) -> bool {
        self.call_workspace(WorkspaceAction::SetConsoleFilter(filter.to_string()))
    }

    pub fn workspace_set_console_scope(self: Pin<&mut Self>, scope: i32) -> bool {
        self.call_workspace(WorkspaceAction::SetConsoleScope(scope))
    }

    pub fn workspace_set_filter(self: Pin<&mut Self>, key: &QString) -> bool {
        self.call_workspace(WorkspaceAction::SetFilter(key.to_string()))
    }

    pub fn workspace_set_search(self: Pin<&mut Self>, query: &QString) -> bool {
        self.call_workspace(WorkspaceAction::SetSearch(query.to_string()))
    }

    pub fn workspace_set_sort(self: Pin<&mut Self>, index: i32) -> bool {
        self.call_workspace(WorkspaceAction::SetSort(index))
    }

    pub fn workspace_sort_by_column(self: Pin<&mut Self>, column: i32) -> bool {
        self.call_workspace(WorkspaceAction::SortByColumn(column))
    }

    pub fn workspace_select_game(self: Pin<&mut Self>, key: &QString) -> bool {
        self.call_workspace(WorkspaceAction::SelectGame(key.to_string()))
    }

    pub fn workspace_mark_toggle(self: Pin<&mut Self>, key: &QString) -> bool {
        self.call_workspace(WorkspaceAction::MarkToggle(key.to_string()))
    }

    pub fn workspace_mark_all(self: Pin<&mut Self>) -> bool {
        self.call_workspace(WorkspaceAction::MarkAll)
    }

    pub fn workspace_clear_marked(self: Pin<&mut Self>) -> bool {
        self.call_workspace(WorkspaceAction::ClearMarked)
    }

    pub fn workspace_batch_favourite(self: Pin<&mut Self>, on: bool) -> bool {
        self.call_workspace(WorkspaceAction::BatchFavourite(on))
    }

    pub fn workspace_clear_filters(self: Pin<&mut Self>) -> bool {
        self.call_workspace(WorkspaceAction::ClearFilters)
    }

    pub fn workspace_open_folder(self: Pin<&mut Self>, key: &QString) -> bool {
        self.call_workspace(WorkspaceAction::OpenFolder(key.to_string()))
    }

    pub fn play_from(self: Pin<&mut Self>, game: &QString, state: &QString) -> bool {
        self.call_workspace(WorkspaceAction::PlayFrom(game.to_string(), state.to_string()))
    }

    pub fn open_desktop_door(self: Pin<&mut Self>, key: &QString) -> bool {
        self.call_workspace(WorkspaceAction::OpenDesktopDoor(key.to_string()))
    }

    pub fn open_tool(self: Pin<&mut Self>, key: &QString) -> bool {
        self.call_workspace(WorkspaceAction::OpenTool(key.to_string()))
    }

    pub fn toggle_full_screen(self: Pin<&mut Self>) -> bool {
        self.call_workspace(WorkspaceAction::ToggleFullScreen)
    }

    pub fn add_games_to_collection(self: Pin<&mut Self>, games: &QStringList, collection_id: &QString) -> bool {
        let list = games.iter().map(|g| g.to_string()).collect();
        self.call_workspace(WorkspaceAction::AddGamesToCollection(list, collection_id.to_string()))
    }

    pub fn workspace_library(self: Pin<&mut Self>) -> bool {
        self.call_workspace(WorkspaceAction::OpenLibrary)
    }

    pub fn workspace_shaders(self: Pin<&mut Self>) -> bool {
        self.call_workspace(WorkspaceAction::OpenShaders)
    }

    pub fn session_pause(self: Pin<&mut Self>) -> bool {
        self.call_session(SessionAction::Pause)
    }

    pub fn session_resume(self: Pin<&mut Self>) -> bool {
        self.call_session(SessionAction::Resume)
    }

    pub fn session_restart(self: Pin<&mut Self>) -> bool {
        self.call_session(SessionAction::Restart)
    }

    pub fn session_close(self: Pin<&mut Self>) -> bool {
        self.call_session(SessionAction::Close)
    }

    pub fn session_screenshot(self: Pin<&mut Self>) -> bool {
        self.call_session(SessionAction::Screenshot)
    }

    pub fn session_choose_slot(self: Pin<&mut Self>, slot: i32) -> bool {
        self.call_session(SessionAction::ChooseSlot(slot))
    }

    pub fn session_save_state(self: Pin<&mut Self>, slot: i32) -> bool {
        self.call_session(SessionAction::SaveState(slot))
    }

    pub fn session_load_state(self: Pin<&mut Self>, slot: i32) -> bool {
        self.call_session(SessionAction::LoadState(slot))
    }

    pub fn session_choose_speed(self: Pin<&mut Self>, percent: i32) -> bool {
        self.call_session(SessionAction::ChooseSpeed(percent))
    }

    pub fn session_rewind_to(self: Pin<&mut Self>, frame: f64) -> bool {
        self.call_session(SessionAction::RewindTo(frame))
    }

    pub fn session_set_master_cheats(self: Pin<&mut Self>, on: bool) -> bool {
        self.call_session(SessionAction::SetMasterCheats(on))
    }

    pub fn session_switch_cheat(self: Pin<&mut Self>, index: i32) -> bool {
        self.call_session(SessionAction::SwitchCheat(index))
    }

    pub fn session_step_console_option(self: Pin<&mut Self>, key: &QString, by: i32) -> bool {
        self.call_session(SessionAction::StepConsoleOption(key.to_string(), by))
    }

    pub fn session_swap_players(self: Pin<&mut Self>) -> bool {
        self.call_session(SessionAction::SwapPlayers)
    }

    pub fn session_answer_resume(self: Pin<&mut Self>, resume: bool) -> bool {
        self.call_session(SessionAction::AnswerResume(resume))
    }

    pub fn themes_set_kind(self: Pin<&mut Self>, kind: &QString) -> bool {
        self.call_themes(ThemesAction::SetKind(kind.to_string()))
    }

    pub fn themes_focus(self: Pin<&mut Self>, row: i32) -> bool {
        self.call_themes(ThemesAction::Focus(row))
    }

    pub fn themes_use(self: Pin<&mut Self>, id: &QString) -> bool {
        self.call_themes(ThemesAction::Use(id.to_string()))
    }

    pub fn themes_step(self: Pin<&mut Self>, row: i32, by: i32) -> bool {
        self.call_themes(ThemesAction::Step(row, by))
    }

    pub fn themes_check(self: Pin<&mut Self>, id: &QString) -> bool {
        self.call_themes(ThemesAction::Check(id.to_string()))
    }

    pub fn tools_focus(self: Pin<&mut Self>, index: i32) -> bool {
        self.call_tools(ToolsAction::Focus(index))
    }

    pub fn tools_open(self: Pin<&mut Self>, key: &QString) -> bool {
        self.call_tools(ToolsAction::Open(key.to_string()))
    }
}
