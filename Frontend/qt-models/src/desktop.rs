//! Reference Desktop Mode view model (`EmuSen.Models` 1.0): chrome strings, view state, multi-select, and real data projections.

use crate::game_model::cover_url;
use crate::library::{games_of_at, seconds_played, GameFacts, GameRow, Kind, LibraryView, ALL};
use crate::pause::Facts as PauseFacts;
use crate::search_eclipse::cover_swatch;
use crate::settings_model::qobject::SettingsModel;
use crate::settings_shown::planet_of;
use crate::words;
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QList, QString, QStringList};
use emusen_kit_core::settings::RowKind;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("emusen-qt-models/src/library_model.cxxqt.h");
        type LibraryModel = crate::library_model::qobject::LibraryModel;
        include!("emusen-qt-models/src/session_model.cxxqt.h");
        type SessionModel = crate::session_model::qobject::SessionModel;
        include!("emusen-qt-models/src/settings_model.cxxqt.h");
        type SettingsModel = crate::settings_model::qobject::SettingsModel;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(*mut LibraryModel, library)]
        #[qproperty(*mut SessionModel, session)]
        #[qproperty(*mut SettingsModel, settings)]
        #[qproperty(QString, theme_name, cxx_name = "themeName")]
        #[qproperty(bool, has_game, cxx_name = "hasGame")]
        #[qproperty(bool, game_paused, cxx_name = "gamePaused")]
        #[qproperty(bool, game_shown, cxx_name = "gameShown")]
        #[qproperty(bool, full_screen, cxx_name = "fullScreen")]
        #[qproperty(i32, controllers)]
        #[qproperty(QString, clock_text, cxx_name = "clockText")]
        #[qproperty(i32, battery)]
        #[qproperty(i32, view, READ, WRITE = set_view, NOTIFY)]
        #[qproperty(bool, sidebar_open, cxx_name = "sidebarOpen", READ, WRITE = set_sidebar_open, NOTIFY)]
        #[qproperty(bool, inspector_open, cxx_name = "inspectorOpen", READ, WRITE = set_inspector_open, NOTIFY)]
        #[qproperty(bool, drawer_open, cxx_name = "drawerOpen", READ, WRITE = set_drawer_open, NOTIFY)]
        #[qproperty(i32, drawer_tab, cxx_name = "drawerTab", READ, WRITE = set_drawer_tab, NOTIFY)]
        #[qproperty(QString, console_key, cxx_name = "consoleKey", READ, WRITE = set_console_key, NOTIFY)]
        #[qproperty(QString, filter_key, cxx_name = "filterKey", READ, WRITE = set_filter_key, NOTIFY)]
        #[qproperty(QString, search_query, cxx_name = "searchQuery", READ, WRITE = set_search_query, NOTIFY)]
        #[qproperty(QString, console_filter, cxx_name = "consoleFilter", READ, WRITE = set_console_filter, NOTIFY)]
        #[qproperty(i32, console_scope, cxx_name = "consoleScope", READ, WRITE = set_console_scope, NOTIFY)]
        #[qproperty(i32, sort_index, cxx_name = "sortIndex", READ, WRITE = set_sort_index, NOTIFY)]
        #[qproperty(bool, sort_ascending, cxx_name = "sortAscending", READ, WRITE = set_sort_ascending, NOTIFY)]
        #[qproperty(QString, selected_key, cxx_name = "selectedKey", READ, WRITE = set_selected_key, NOTIFY)]
        #[qproperty(QStringList, marked_keys, cxx_name = "markedKeys", READ, NOTIFY)]
        #[qproperty(i32, marked_count, cxx_name = "markedCount", READ, NOTIFY)]
        #[qproperty(bool, all_marked, cxx_name = "allMarked", READ, NOTIFY)]
        #[qproperty(i32, settings_tool_index, cxx_name = "settingsToolIndex", READ, NOTIFY)]
        #[qproperty(QString, settings_filter, cxx_name = "settingsFilter", READ, WRITE = set_settings_filter, NOTIFY)]
        #[qproperty(bool, settings_modified_only, cxx_name = "settingsModifiedOnly", READ, WRITE = set_settings_modified_only, NOTIFY)]
        #[qproperty(QString, chrome_json, cxx_name = "chromeJson", READ, CONSTANT)]
        #[qproperty(QString, consoles_json, cxx_name = "consolesJson", READ, NOTIFY)]
        #[qproperty(QString, games_json, cxx_name = "gamesJson", READ, NOTIFY)]
        #[qproperty(QString, inspector_json, cxx_name = "inspectorJson", READ, NOTIFY)]
        #[qproperty(QString, playing_json, cxx_name = "playingJson", READ, NOTIFY)]
        #[qproperty(QString, settings_json, cxx_name = "settingsJson", READ, NOTIFY)]
        #[qproperty(QString, status_text, cxx_name = "statusText", READ, NOTIFY)]
        #[qproperty(QString, mode_badge, cxx_name = "modeBadge", READ, NOTIFY)]
        #[qproperty(QString, status_summary, cxx_name = "statusSummary", READ, NOTIFY)]
        type DesktopModel = super::DesktopModelRust;
    }

    extern "RustQt" {
        #[cxx_name = "setView"]
        fn set_view(self: Pin<&mut DesktopModel>, view: i32);

        #[cxx_name = "setSidebarOpen"]
        fn set_sidebar_open(self: Pin<&mut DesktopModel>, open: bool);

        #[cxx_name = "setInspectorOpen"]
        fn set_inspector_open(self: Pin<&mut DesktopModel>, open: bool);

        #[cxx_name = "setDrawerOpen"]
        fn set_drawer_open(self: Pin<&mut DesktopModel>, open: bool);

        #[cxx_name = "setDrawerTab"]
        fn set_drawer_tab(self: Pin<&mut DesktopModel>, tab: i32);

        #[cxx_name = "setConsoleKey"]
        fn set_console_key(self: Pin<&mut DesktopModel>, key: QString);

        #[cxx_name = "setFilterKey"]
        fn set_filter_key(self: Pin<&mut DesktopModel>, key: QString);

        #[cxx_name = "setSearchQuery"]
        fn set_search_query(self: Pin<&mut DesktopModel>, query: QString);

        #[cxx_name = "setConsoleFilter"]
        fn set_console_filter(self: Pin<&mut DesktopModel>, filter: QString);

        #[cxx_name = "setConsoleScope"]
        fn set_console_scope(self: Pin<&mut DesktopModel>, scope: i32);

        #[cxx_name = "setSortIndex"]
        fn set_sort_index(self: Pin<&mut DesktopModel>, index: i32);

        #[cxx_name = "setSortAscending"]
        fn set_sort_ascending(self: Pin<&mut DesktopModel>, ascending: bool);

        #[cxx_name = "setSelectedKey"]
        fn set_selected_key(self: Pin<&mut DesktopModel>, key: QString);

        #[cxx_name = "setSettingsFilter"]
        fn set_settings_filter(self: Pin<&mut DesktopModel>, filter: QString);

        #[cxx_name = "setSettingsModifiedOnly"]
        fn set_settings_modified_only(self: Pin<&mut DesktopModel>, modified: bool);

        #[qinvokable]
        #[cxx_name = "toggleSidebar"]
        fn toggle_sidebar(self: Pin<&mut DesktopModel>);

        #[qinvokable]
        #[cxx_name = "toggleInspector"]
        fn toggle_inspector(self: Pin<&mut DesktopModel>);

        #[qinvokable]
        #[cxx_name = "toggleDrawer"]
        fn toggle_drawer(self: Pin<&mut DesktopModel>);

        #[qinvokable]
        #[cxx_name = "openDrawerTab"]
        fn open_drawer_tab(self: Pin<&mut DesktopModel>, tab: i32);

        #[qinvokable]
        #[cxx_name = "selectConsole"]
        fn select_console(self: Pin<&mut DesktopModel>, key: &QString);

        #[qinvokable]
        #[cxx_name = "stepSort"]
        fn step_sort(self: Pin<&mut DesktopModel>);

        #[qinvokable]
        #[cxx_name = "sortByColumn"]
        fn sort_by_column(self: Pin<&mut DesktopModel>, column: i32);

        #[qinvokable]
        #[cxx_name = "selectGame"]
        fn select_game(self: Pin<&mut DesktopModel>, path: &QString);

        #[qinvokable]
        #[cxx_name = "jumpLetter"]
        fn jump_letter(self: &DesktopModel, letter: &QString) -> i32;

        #[qinvokable]
        #[cxx_name = "markToggle"]
        fn mark_toggle(self: Pin<&mut DesktopModel>, path: &QString);

        #[qinvokable]
        #[cxx_name = "markAll"]
        fn mark_all(self: Pin<&mut DesktopModel>);

        #[qinvokable]
        #[cxx_name = "clearMarked"]
        fn clear_marked(self: Pin<&mut DesktopModel>);

        #[qinvokable]
        #[cxx_name = "batchFavourite"]
        fn batch_favourite(self: Pin<&mut DesktopModel>, on: bool);

        #[qinvokable]
        #[cxx_name = "toggleSelectedFavourite"]
        fn toggle_selected_favourite(self: Pin<&mut DesktopModel>);

        #[qinvokable]
        #[cxx_name = "openSelectedFolder"]
        fn open_selected_folder(self: Pin<&mut DesktopModel>);

        #[qinvokable]
        #[cxx_name = "openConsoleOptions"]
        fn open_console_options(self: Pin<&mut DesktopModel>, key: &QString);

        #[qinvokable]
        #[cxx_name = "selectSettingsSection"]
        fn select_settings_section(self: Pin<&mut DesktopModel>, index: i32);

        #[qinvokable]
        #[cxx_name = "openSettingsTool"]
        fn open_settings_tool(self: Pin<&mut DesktopModel>, index: i32);

        #[qinvokable]
        #[cxx_name = "settingsBack"]
        fn settings_back(self: Pin<&mut DesktopModel>);

        #[qinvokable]
        #[cxx_name = "settingsChoose"]
        fn settings_choose(self: Pin<&mut DesktopModel>, row: i32);

        #[qinvokable]
        #[cxx_name = "settingsStep"]
        fn settings_step(self: Pin<&mut DesktopModel>, row: i32, by: i32);

        #[qinvokable]
        #[cxx_name = "settingsSlide"]
        fn settings_slide(self: Pin<&mut DesktopModel>, row: i32, amount: f64);

        #[qinvokable]
        #[cxx_name = "settingsFocus"]
        fn settings_focus(self: Pin<&mut DesktopModel>, row: i32);

        #[qinvokable]
        #[cxx_name = "handleSettingsDoor"]
        fn handle_settings_door(self: Pin<&mut DesktopModel>, name: &QString);

        #[qinvokable]
        #[cxx_name = "clearFilters"]
        fn clear_filters(self: Pin<&mut DesktopModel>);

        #[qinvokable]
        fn refresh(self: Pin<&mut DesktopModel>);

        #[qsignal]
        #[cxx_name = "openFolderRequested"]
        fn open_folder_requested(self: Pin<&mut DesktopModel>, path: QString);

        #[qsignal]
        #[cxx_name = "launchRequested"]
        fn launch_requested(self: Pin<&mut DesktopModel>, path: QString);

        #[qsignal]
        #[cxx_name = "launchFromRequested"]
        fn launch_from_requested(self: Pin<&mut DesktopModel>, path: QString, state: QString);

        #[qsignal]
        #[cxx_name = "resumeRequested"]
        fn resume_requested(self: Pin<&mut DesktopModel>);

        #[qsignal]
        #[cxx_name = "pageRequested"]
        fn page_requested(self: Pin<&mut DesktopModel>, path: QString);

        #[qsignal]
        #[cxx_name = "doorRequested"]
        fn door_requested(self: Pin<&mut DesktopModel>, name: QString);

        #[qsignal]
        #[cxx_name = "toggleFullScreenRequested"]
        fn toggle_full_screen_requested(self: Pin<&mut DesktopModel>);
    }

    impl cxx_qt::Initialize for DesktopModel {}
    impl cxx_qt::Threading for DesktopModel {}
}

/// Pure Rust filter/sort/selection state for Desktop Mode projections and unit testing.
#[derive(Clone, Debug)]
pub struct DesktopState {
    pub view: i32,
    pub sidebar_open: bool,
    pub inspector_open: bool,
    pub drawer_open: bool,
    pub drawer_tab: i32,
    pub console_key: String,
    pub filter_key: String,
    pub search_query: String,
    pub console_filter: String,
    pub console_scope: i32,
    pub sort_index: i32,
    pub sort_ascending: bool,
    pub selected_key: String,
    pub marked_keys: Vec<String>,
    pub settings_tool_index: i32,
    pub settings_filter: String,
    pub settings_modified_only: bool,
}

impl Default for DesktopState {
    fn default() -> Self {
        DesktopState {
            view: 0,
            sidebar_open: true,
            inspector_open: true,
            drawer_open: false,
            drawer_tab: 0,
            console_key: ALL.to_owned(),
            filter_key: ALL.to_owned(),
            search_query: String::new(),
            console_filter: String::new(),
            console_scope: 1,
            sort_index: 0,
            sort_ascending: true,
            selected_key: String::new(),
            marked_keys: Vec::new(),
            settings_tool_index: -1,
            settings_filter: String::new(),
            settings_modified_only: false,
        }
    }
}

impl DesktopState {
    pub fn set_view(&mut self, view: i32) -> bool {
        let clamped = view.clamp(0, 2);
        if self.view != clamped {
            self.view = clamped;
            true
        } else {
            false
        }
    }

    pub fn toggle_mark(&mut self, path: &str) {
        if path.is_empty() {
            return;
        }
        if let Some(pos) = self.marked_keys.iter().position(|k| k == path) {
            self.marked_keys.remove(pos);
        } else {
            self.marked_keys.push(path.to_owned());
        }
    }

    pub fn mark_all(&mut self, visible_paths: &[String]) {
        if visible_paths.is_empty() {
            return;
        }
        let all_selected = visible_paths.iter().all(|p| self.marked_keys.contains(p));
        if all_selected {
            self.marked_keys.retain(|k| !visible_paths.contains(k));
        } else {
            for p in visible_paths {
                if !self.marked_keys.contains(p) {
                    self.marked_keys.push(p.clone());
                }
            }
        }
    }

    pub fn clear_marks(&mut self) {
        self.marked_keys.clear();
    }
}

/// Live or snapshot info about the running emulation session for Desktop Mode projections.
#[derive(Clone, Debug, Default)]
pub struct SessionSnapshot {
    pub running: bool,
    pub paused: bool,
    pub game: String,
    pub console: String,
    pub fps_words: String,
    pub status: String,
    pub slot: u32,
    pub speed: u32,
    pub master_cheats: bool,
    pub card_facts: String,
}

/// Cached per-game inspector details fetched off the GUI thread.
#[derive(Clone, Debug, Default)]
pub struct CachedInspector {
    pub path: String,
    pub facts: GameFacts,
    pub size_bytes: Option<u64>,
    pub cheats_count: usize,
    pub states: Vec<crate::game_page::State>,
}

/// Structured Workspace projection shared by `DesktopModel` JSON serialization and `ThemeFeed`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesktopPlace {
    pub key: String,
    pub label: String,
    pub count: i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesktopConsole {
    pub key: String,
    pub name: String,
    pub full_name: String,
    pub maker: String,
    pub year: i32,
    pub games: i32,
    pub played: i32,
    pub engine: String,
    pub engine_kind: String,
    pub planet: i32,
    pub libretro: bool,
    pub has_games: bool,
    pub screen: String,
    pub hue: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesktopFilter {
    pub key: String,
    pub label: String,
    pub count: i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesktopSort {
    pub index: i32,
    pub label: String,
    pub key: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesktopGame {
    pub key: String,
    pub title: String,
    pub system: String,
    pub console: String,
    pub core: String,
    pub planet: i32,
    pub libretro: bool,
    pub year: String,
    pub genre: String,
    pub subtitle: String,
    pub played_words: String,
    pub played_seconds: i64,
    pub last_played_words: String,
    pub favourite: bool,
    pub marked: bool,
    pub resume_ready: bool,
    pub badge: String,
    pub cover: String,
    pub hero: String,
    pub swatch: Value,
    pub format: String,
    pub states_count: i32,
    pub states_label: String,
    pub status_label: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesktopStateEntry {
    pub path: String,
    pub label: String,
    pub when_words: String,
    pub picture: String,
    pub is_auto: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesktopWorkspaceData {
    pub theme_name: String,
    pub status_text: String,
    pub status_summary: String,
    pub mode_badge: String,
    pub big_picture_hint: String,
    pub status_hints: String,
    pub counts_line: String,
    pub controllers_line: String,
    pub marked_count: i32,
    pub all_marked: bool,
    pub count: i32,
    pub total_count: i32,
    pub summary_line: String,
    pub empty_words: String,
    pub active_heading: String,
    pub active_core: String,
    pub active_planet: i32,
    pub active_screen: String,
    pub has_selection: bool,
    pub effective_selected: String,
    pub inspector_title: String,
    pub inspector_subtitle: String,
    pub inspector_console: String,
    pub inspector_system: String,
    pub inspector_core: String,
    pub inspector_core_line: String,
    pub inspector_planet: i32,
    pub inspector_libretro: bool,
    pub inspector_played_words: String,
    pub inspector_players: String,
    pub inspector_genre: String,
    pub inspector_description: String,
    pub inspector_file_name: String,
    pub inspector_folder_path: String,
    pub inspector_size_words: String,
    pub inspector_favourite: bool,
    pub inspector_running: bool,
    pub inspector_cover: String,
    pub inspector_hero: String,
    pub inspector_swatch: Value,
    pub inspector_states_count: i32,
    pub inspector_states_heading: String,
    pub inspector_package_words: String,
    pub inspector_cheats_words: String,
    pub places: Vec<DesktopPlace>,
    pub consoles: Vec<DesktopConsole>,
    pub filters: Vec<DesktopFilter>,
    pub sorts: Vec<DesktopSort>,
    pub letters: Vec<String>,
    pub games: Vec<DesktopGame>,
    pub inspector_states: Vec<DesktopStateEntry>,
    pub visible_paths: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesktopPlayingSlot {
    pub slot: i32,
    pub exists: bool,
    pub when: String,
    pub picture: String,
    pub active: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesktopPlayingCheat {
    pub index: i32,
    pub description: String,
    pub on: bool,
    pub code: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesktopPlayingConsoleOption {
    pub key: String,
    pub label: String,
    pub value: String,
    pub display_value: String,
    pub choice_count: i32,
    pub choices: Vec<String>,
    pub choice_labels: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesktopPlayingRewindMark {
    pub frame: f64,
    pub label: String,
    pub short: String,
    pub picture: String,
    pub now: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesktopPlayingData {
    pub running: bool,
    pub paused: bool,
    pub game: String,
    pub system: String,
    pub title: String,
    pub console: String,
    pub core: String,
    pub planet: i32,
    pub fps_words: String,
    pub status: String,
    pub slot: i32,
    pub speed: i32,
    pub master_cheats: bool,
    pub has_rewind: bool,
    pub latest_rewind_frame: f64,
    pub slots: Vec<DesktopPlayingSlot>,
    pub cheats: Vec<DesktopPlayingCheat>,
    pub console_options: Vec<DesktopPlayingConsoleOption>,
    pub rewind_marks: Vec<DesktopPlayingRewindMark>,
}

/// Output bundle of `project_desktop`.
#[derive(Clone, Debug)]
pub struct DesktopProjection {
    pub consoles_json: String,
    pub games_json: String,
    pub inspector_json: String,
    pub playing_json: String,
    pub status_text: String,
    pub mode_badge: String,
    pub status_summary: String,
    pub effective_selected: String,
    pub visible_paths: Vec<String>,
    pub all_marked: bool,
}

const SORT_LABELS: [&str; 7] = words::DESKTOP_SORT_LABELS;
const SORT_KEYS: [&str; 7] = ["title", "console", "core", "year", "genre", "played", "states"];
const TOOL_KEYS: [&str; 6] = ["cores", "shaders", "packages", "scraper", "library", "themes"];

/// Static dictionary of every user-facing string and icon path in Reference Desktop Mode.
pub fn chrome_json() -> String {
    let mut map = serde_json::Map::new();
    let merge = |dst: &mut serde_json::Map<String, Value>, part: Value| {
        if let Value::Object(obj) = part {
            dst.extend(obj);
        }
    };
    merge(
        &mut map,
        json!({
            "wordmark": words::NAME,
            "navTabs": words::DESKTOP_NAV_TABS,
            "viewModes": words::DESKTOP_VIEW_MODES,
            "searchPlaceholder": words::DESKTOP_SEARCH_PLACEHOLDER,
            "bigPictureLabel": words::DESKTOP_BIG_PICTURE_LABEL,
            "bigPictureHint": words::DESKTOP_BIG_PICTURE_HINT,
            "resumeLabel": words::DESKTOP_RESUME_LABEL,
            "sidebarTip": words::DESKTOP_SIDEBAR_TIP,
            "inspectorTip": words::DESKTOP_INSPECTOR_TIP,
            "constellationHeading": words::DESKTOP_CONSTELLATION_HEADING,
            "continueLabel": words::CONTINUE_ENTRY,
            "allGamesLabel": words::DESKTOP_ALL_GAMES_LABEL,
            "favouritesLabel": words::FAVOURITES_FILTER,
            "recentLabel": words::DESKTOP_RECENT_LABEL,
            "consolesHeading": words::DESKTOP_CONSOLES_HEADING,
            "scopeAll": words::ALL,
            "scopeActive": words::DESKTOP_SCOPE_ACTIVE,
            "consoleFilterPlaceholder": words::DESKTOP_CONSOLE_FILTER_PLACEHOLDER,
            "idleStatus": words::DESKTOP_IDLE_STATUS,
            "runningStatus": words::DESKTOP_RUNNING_STATUS,
            "pausedStatus": words::DESKTOP_PAUSED_STATUS,
            "noControllers": words::DESKTOP_NO_CONTROLLERS,
            "batchGuaranteeTitle": words::DESKTOP_BATCH_GUARANTEE_TITLE,
            "contractNote": words::DESKTOP_CONTRACT_NOTE
        }),
    );
    merge(
        &mut map,
        json!({
            "filters": [
                { "key": "all", "label": words::ALL },
                { "key": "favourites", "label": words::FAVOURITES_FILTER },
                { "key": "resume", "label": words::DESKTOP_FILTER_RESUME_READY },
                { "key": "unplayed", "label": words::DESKTOP_FILTER_UNPLAYED }
            ],
            "sortLabel": words::DESKTOP_SORT_LABEL,
            "sortOptions": SORT_LABELS,
            "consoleOptionsLabel": words::DESKTOP_CONSOLE_OPTIONS_LABEL,
            "resumeBadge": words::DESKTOP_RESUME_BADGE,
            "emptyLibrary": words::NO_GAMES,
            "emptyFiltered": words::NO_GAMES_FOUND,
            "inspectorHeading": words::DESKTOP_INSPECTOR_HEADING,
            "playLabel": words::DESKTOP_PLAY_LABEL,
            "resumeSlotLabel": words::DESKTOP_RESUME_LABEL,
            "saveStatesLabel": words::DESKTOP_SAVE_STATES_LABEL,
            "noStatesLabel": words::DESKTOP_NO_STATES_LABEL,
            "programFactsLabel": words::DESKTOP_PROGRAM_FACTS_LABEL,
            "factCore": words::DESKTOP_FACT_CORE,
            "factPlayed": words::DESKTOP_FACT_PLAYED,
            "factPlayers": words::DESKTOP_FACT_PLAYERS,
            "factGenre": words::DESKTOP_FACT_GENRE,
            "factFile": words::DESKTOP_FACT_FILE,
            "factSize": words::DESKTOP_FACT_SIZE,
            "factLastPlayed": words::DESKTOP_FACT_LAST_PLAYED,
            "favouriteLabel": words::DESKTOP_FAVOURITE_LABEL,
            "unfavouriteLabel": words::DESKTOP_UNFAVOURITE_LABEL,
            "detailsLabel": words::DESKTOP_DETAILS_LABEL,
            "folderLabel": words::DESKTOP_FOLDER_LABEL,
            "libretroSuffix": words::DESKTOP_LIBRETRO_SUFFIX,
            "nativeSuffix": words::DESKTOP_NATIVE_SUFFIX
        }),
    );
    merge(
        &mut map,
        json!({
            "tableHeading": words::DESKTOP_TABLE_HEADING,
            "clearFiltersLabel": words::DESKTOP_CLEAR_FILTERS_LABEL,
            "columns": words::DESKTOP_TABLE_COLUMNS,
            "sortAsc": words::DESKTOP_SORT_ASC,
            "sortDesc": words::DESKTOP_SORT_DESC,
            "statusResume": words::DESKTOP_RESUME_LABEL,
            "statusFav": words::DESKTOP_FAVOURITE_LABEL,
            "statusReady": words::DESKTOP_STATUS_READY,
            "selectedSuffix": words::DESKTOP_SELECTED_SUFFIX,
            "clearSelectionLabel": words::DESKTOP_CLEAR_LABEL,
            "batchMarkFav": words::DESKTOP_FAVOURITE_LABEL,
            "batchUnmarkFav": words::DESKTOP_UNMARK_FAV_LABEL,
            "batchPage": words::DESKTOP_DETAILS_LABEL,
            "batchPlay": words::DESKTOP_PLAY_LABEL,
            "batchClear": words::DESKTOP_CLEAR_LABEL,
            "dash": words::DESKTOP_DASH
        }),
    );
    merge(
        &mut map,
        json!({
            "stripLibrary": words::LIBRARY_SECTION,
            "stripPause": words::DESKTOP_STRIP_PAUSE,
            "stripResume": words::DESKTOP_RESUME_LABEL,
            "stripRewind": words::DESKTOP_STRIP_REWIND,
            "stripStates": words::DESKTOP_STRIP_STATES,
            "stripShaders": words::DESKTOP_STRIP_SHADERS,
            "stripCheats": words::DESKTOP_STRIP_CHEATS,
            "stripControls": words::OPTIONS,
            "stripScreenshot": words::DESKTOP_STRIP_SHOT,
            "stripFullscreen": words::DESKTOP_STRIP_FULLSCREEN,
            "stripBigPicture": words::DESKTOP_BIG_PICTURE_LABEL,
            "drawerTabs": words::DESKTOP_DRAWER_TABS,
            "quickSaveLabel": words::DESKTOP_QUICK_SAVE_LABEL,
            "quickLoadLabel": words::DESKTOP_QUICK_LOAD_LABEL,
            "slotPrefix": words::DESKTOP_SLOT_PREFIX,
            "emptySlotLabel": words::DESKTOP_EMPTY_SLOT_LABEL,
            "rewindNowLabel": words::DESKTOP_REWIND_AVAILABLE,
            "noRewindLabel": words::DESKTOP_NO_REWIND_LABEL,
            "masterCheatsLabel": words::DESKTOP_MASTER_CHEATS_LABEL,
            "noCheatsLabel": words::DESKTOP_NO_CHEATS_LABEL,
            "shadersHeading": words::DESKTOP_SHADERS_HEADING,
            "noDisplayRowsLabel": words::DESKTOP_NO_DISPLAY_ROWS_LABEL,
            "speedLabel": words::DESKTOP_SPEED_LABEL,
            "speeds": [
                { "percent": 25, "label": "25%" },
                { "percent": 100, "label": "100%" },
                { "percent": 300, "label": "300%" },
                { "percent": 0, "label": words::DESKTOP_SPEED_MAX_LABEL }
            ],
            "swapPlayersLabel": words::DESKTOP_SWAP_PLAYERS_LABEL,
            "restartLabel": words::DESKTOP_RESTART_LABEL,
            "quitGameLabel": words::DESKTOP_QUIT_GAME_LABEL
        }),
    );
    merge(
        &mut map,
        json!({
            "settingsFilterPlaceholder": words::DESKTOP_SETTINGS_FILTER_PLACEHOLDER,
            "settingsHeading": words::DESKTOP_SETTINGS_HEADING,
            "toolsHeading": words::DESKTOP_TOOLS_HEADING,
            "modifiedOnlyLabel": words::DESKTOP_MODIFIED_ONLY_LABEL,
            "backLabel": words::DESKTOP_BACK_LABEL,
            "themesHeading": words::DESKTOP_THEMES_HEADING,
            "themesSubtitle": words::DESKTOP_THEMES_SUBTITLE,
            "useThemeLabel": words::DESKTOP_USE_THEME_LABEL,
            "inUseLabel": words::DESKTOP_IN_USE_LABEL,
            "dualFaceBadge": words::DESKTOP_DUAL_FACE_BADGE,
            "bigPictureBadge": words::DESKTOP_BIG_PICTURE_BADGE,
            "esdeBadge": words::DESKTOP_ESDE_BADGE,
            "navObjectNames": ["desktopNavLibrary", "desktopNavTable", "desktopNavSettings"],
            "allKey": "all",
            "sep": " · ",
            "percent": "%",
            "batteryPrefix": "BAT ",
            "checkThemeLabel": words::DESKTOP_CHECK_THEME_LABEL,
            "toolsList": [
                { "key": "cores", "label": words::HOME_PLACES[4] },
                { "key": "shaders", "label": words::DESKTOP_STRIP_SHADERS },
                { "key": "packages", "label": words::HOME_PLACES[2] },
                { "key": "scraper", "label": words::HOME_PLACES[1] },
                { "key": "library", "label": words::DESKTOP_LIBRARY_MANAGER_LABEL },
                { "key": "themes", "label": words::DESKTOP_THEMES_HEADING }
            ],
            "aboutDetailHeading": words::DESKTOP_ABOUT_DETAIL_HEADING,
            "statusHints": words::DESKTOP_STATUS_HINTS,
            "icons": {
                "search": "M11 4a7 7 0 1 0 0 14a7 7 0 1 0 0 -14M16.5 16.5L21 21",
                "heart": "M12 20s-7-4.5-9-9a5 5 0 0 1 9-3a5 5 0 0 1 9 3c-2 4.5-9 9-9 9z",
                "play": "M7 5l12 7l-12 7z",
                "pause": "M7 5h3v14H7zM14 5h3v14h-3z",
                "chevronLeft": "M15 6l-6 6l6 6",
                "chevronRight": "M9 6l6 6l-6 6",
                "check": "M5 12l4.5 4.5L19 7",
                "close": "M6 6l12 12M18 6l-12 12",
                "folder": "M3 6h6l2 2h10v10H3z",
                "sidebar": "M4 5h16v14H4zM9 5v14",
                "inspector": "M4 5h16v14H4zM15 5v14"
            }
        }),
    );
    Value::Object(map).to_string()
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{} KB", bytes / 1024)
    } else {
        format!("{bytes} B")
    }
}

fn path_extension(path: &str) -> String {
    Path::new(path).extension().and_then(|e| e.to_str()).map(|e| format!(".{}", e.to_ascii_lowercase())).unwrap_or_default()
}

fn path_file_name(path: &str) -> String {
    Path::new(path).file_name().and_then(|e| e.to_str()).unwrap_or(path).to_owned()
}

fn path_folder(path: &str) -> String {
    Path::new(path).parent().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default()
}

fn meta_str(meta: &[(String, BTreeMap<String, String>)], path: &str, key: &str) -> String {
    meta.iter().find(|(p, _)| p == path).and_then(|(_, m)| m.get(key)).cloned().unwrap_or_default()
}

fn game_seed(path: &str) -> usize {
    path.bytes().fold(2166136261usize, |h, b| h.wrapping_mul(16777619) ^ usize::from(b))
}

fn system_core_info(view: &LibraryView, system: &str) -> (String, i32, bool) {
    if let Some(row) = view.rows.rows().iter().find(|r| r.kind == Some(Kind::Console) && r.key == system) {
        let planet = planet_of(&row.engine_name, row.libretro, !row.engine.is_empty());
        return (row.engine_name.clone(), planet, row.libretro);
    }
    if let Some(sys) = view.systems.iter().find(|s| s.id == system) {
        let engine = sys.engines.first().cloned().unwrap_or_default();
        let planet = planet_of(&engine, false, !engine.is_empty());
        return (engine, planet, false);
    }
    (String::new(), -2, false)
}

fn row_matches_filter(g: &GameRow, filter_key: &str, session: &SessionSnapshot, states_count: usize) -> bool {
    match filter_key {
        "favourites" => g.favourite,
        "resume" => (session.running && session.game == g.path) || states_count > 0,
        "unplayed" => g.played_at == 0,
        _ => true,
    }
}

fn row_matches_search(g: &GameRow, core: &str, genre: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let q = query.trim().to_lowercase();
    q.is_empty()
        || g.title.to_lowercase().contains(&q)
        || g.console.to_lowercase().contains(&q)
        || core.to_lowercase().contains(&q)
        || genre.to_lowercase().contains(&q)
        || g.year.to_lowercase().contains(&q)
}

/// Computes structured Desktop Workspace projections from the real library, session, saved states, and cached inspector state.
#[allow(clippy::too_many_arguments)]
pub fn project_workspace(
    view: &LibraryView,
    extra_meta: &[(String, BTreeMap<String, String>)],
    state: &DesktopState,
    session: &SessionSnapshot,
    state_counts: &BTreeMap<String, usize>,
    cached: Option<&CachedInspector>,
    theme_name: &str,
    controllers: i32,
) -> DesktopWorkspaceData {
    let now = words::now();
    let meta = if !view.catalog.metadata.is_empty() { view.catalog.metadata.as_slice() } else { extra_meta };
    let has_running = session.running && !session.game.is_empty();
    let theme_label = if theme_name.trim().is_empty() { words::NAME.to_owned() } else { theme_name.trim().to_owned() };
    let mode_badge = theme_label.to_ascii_uppercase();
    let status_summary = words::desktop_status_summary(&theme_label);

    let all_games = games_of_at(ALL, &view.systems, &view.catalog, now);
    let recent_count = view.catalog.recent.len();
    let fav_count = all_games.iter().filter(|g| g.favourite).count();
    let fav_systems: HashSet<&str> = all_games.iter().filter(|g| g.favourite).map(|g| g.system.as_str()).collect();

    let console_q = state.console_filter.trim().to_lowercase();
    let source_consoles: Vec<_> = if state.console_scope == 0 {
        let mut all = view.cored.clone();
        for r in view.rows.rows().iter().filter(|r| r.kind == Some(Kind::Console)) {
            if !all.iter().any(|c| c.key == r.key) {
                all.push(r.clone());
            }
        }
        all
    } else if state.console_scope == 2 {
        view.rows.rows().iter().filter(|r| r.kind == Some(Kind::Console) && fav_systems.contains(r.key.as_str())).cloned().collect()
    } else {
        view.rows.rows().iter().filter(|r| r.kind == Some(Kind::Console)).cloned().collect()
    };
    let consoles: Vec<DesktopConsole> = source_consoles
        .iter()
        .filter(|r| console_q.is_empty() || r.name.to_lowercase().contains(&console_q) || r.full_name.to_lowercase().contains(&console_q))
        .map(|r| {
            let planet = planet_of(&r.engine_name, r.libretro, !r.engine.is_empty());
            let engine_kind = if r.engine_name.is_empty() {
                String::new()
            } else if r.libretro {
                words::DESKTOP_LIBRETRO_SUFFIX.to_owned()
            } else {
                words::DESKTOP_NATIVE_SUFFIX.to_owned()
            };
            DesktopConsole {
                key: r.key.clone(),
                name: r.name.clone(),
                full_name: r.full_name.clone(),
                maker: r.maker.clone(),
                year: r.year,
                games: r.count as i32,
                played: r.played as i32,
                engine: r.engine_name.clone(),
                engine_kind,
                planet,
                libretro: r.libretro,
                has_games: r.count > 0,
                screen: r.screen.clone(),
                hue: r.hue,
            }
        })
        .collect();

    let mut places = Vec::new();
    if has_running {
        places.push(DesktopPlace { key: "continue".to_owned(), label: words::CONTINUE_ENTRY.to_owned(), count: 1 });
    }
    places.push(DesktopPlace { key: ALL.to_owned(), label: words::DESKTOP_ALL_GAMES_LABEL.to_owned(), count: all_games.len() as i32 });
    places.push(DesktopPlace { key: crate::library::FAVOURITES.to_owned(), label: words::FAVOURITES_FILTER.to_owned(), count: fav_count as i32 });
    places.push(DesktopPlace { key: crate::library::RECENT.to_owned(), label: words::DESKTOP_RECENT_LABEL.to_owned(), count: recent_count as i32 });

    let active_row = view.rows.rows().iter().find(|r| r.key == state.console_key);
    let active_heading = match state.console_key.as_str() {
        "continue" => words::CONTINUE_ENTRY.to_owned(),
        ALL => words::DESKTOP_ALL_GAMES_LABEL.to_owned(),
        crate::library::FAVOURITES => words::FAVOURITES_FILTER.to_owned(),
        crate::library::RECENT => words::DESKTOP_RECENT_LABEL.to_owned(),
        _ => active_row.map(|r| r.name.clone()).unwrap_or_else(|| words::DESKTOP_ALL_GAMES_LABEL.to_owned()),
    };
    let (active_core, active_planet, _) = if active_row.is_some_and(|r| r.kind == Some(Kind::Console)) {
        system_core_info(view, &state.console_key)
    } else {
        (String::new(), -2, false)
    };
    let active_screen = active_row.map(|r| r.screen.clone()).unwrap_or_default();
    let total_consoles = view.systems.len();
    let active_consoles = view.rows.rows().iter().filter(|r| r.kind == Some(Kind::Console)).count();
    let counts_line = format!("{active_consoles} / {total_consoles}");
    let controllers_line = words::desktop_controllers_line(controllers);

    let base_list: Vec<GameRow> = match state.console_key.as_str() {
        "continue" => {
            if has_running {
                all_games.iter().filter(|g| g.path == session.game).cloned().collect()
            } else {
                Vec::new()
            }
        }
        ALL => all_games.clone(),
        crate::library::FAVOURITES => games_of_at(crate::library::FAVOURITES, &view.systems, &view.catalog, now),
        crate::library::RECENT => games_of_at(crate::library::RECENT, &view.systems, &view.catalog, now),
        system => games_of_at(system, &view.systems, &view.catalog, now),
    };

    let state_count_of = |path: &str| -> usize {
        state_counts
            .get(path)
            .copied()
            .or_else(|| cached.filter(|c| c.path == path).map(|c| c.facts.states))
            .unwrap_or(0)
    };
    let f_all = base_list.len();
    let f_fav = base_list.iter().filter(|g| g.favourite).count();
    let f_res = base_list.iter().filter(|g| (has_running && session.game == g.path) || state_count_of(&g.path) > 0).count();
    let f_unp = base_list.iter().filter(|g| g.played_at == 0).count();

    struct Enriched {
        row: GameRow,
        core: String,
        planet: i32,
        libretro: bool,
        genre: String,
        year: String,
        played_seconds: i64,
        played_words: String,
        last_played_words: String,
        states_count: usize,
        resume_ready: bool,
        format: String,
    }

    let mut filtered: Vec<Enriched> = base_list
        .into_iter()
        .filter_map(|g| {
            let states_count = state_count_of(&g.path);
            if !row_matches_filter(&g, &state.filter_key, session, states_count) {
                return None;
            }
            let (core, planet, libretro) = system_core_info(view, &g.system);
            let mut genre = meta_str(meta, &g.path, "genre");
            if genre.is_empty() {
                if let Some(c) = cached.filter(|c| c.path == g.path) {
                    genre = c.facts.genre.clone();
                }
            }
            let mut year = g.year.clone();
            if year.is_empty() {
                if let Some(d) = crate::esde_shelves::release_date(meta.iter().find(|(p, _)| p == &g.path).map(|(_, m)| m).unwrap_or(&BTreeMap::new())) {
                    year = crate::esde_shelves::year_of(&d).unwrap_or_default();
                }
            }
            if year.is_empty() {
                if let Some(c) = cached.filter(|c| c.path == g.path) {
                    year = c.facts.year.clone();
                }
            }
            if !row_matches_search(&g, &core, &genre, &state.search_query) {
                return None;
            }
            let played_seconds = seconds_played(&view.catalog, &g.path);
            let played_words = words::time_played(played_seconds).unwrap_or_default();
            let last_played_words = if g.played_at > 0 { words::ago(g.played_at, now) } else { String::new() };
            let resume_ready = (has_running && session.game == g.path) || states_count > 0;
            let format = path_extension(&g.path);
            Some(Enriched {
                row: g,
                core,
                planet,
                libretro,
                genre,
                year,
                played_seconds,
                played_words,
                last_played_words,
                states_count,
                resume_ready,
                format,
            })
        })
        .collect();

    filtered.sort_by(|a, b| {
        let ord = match state.sort_index {
            1 => a.row.console.to_lowercase().cmp(&b.row.console.to_lowercase()),
            2 => a.core.to_lowercase().cmp(&b.core.to_lowercase()),
            3 => a.year.cmp(&b.year),
            4 => a.genre.to_lowercase().cmp(&b.genre.to_lowercase()),
            5 => a.played_seconds.cmp(&b.played_seconds).then(a.row.played_at.cmp(&b.row.played_at)),
            6 => a.states_count.cmp(&b.states_count),
            _ => a.row.title.to_lowercase().cmp(&b.row.title.to_lowercase()),
        }
        .then_with(|| a.row.title.to_lowercase().cmp(&b.row.title.to_lowercase()));
        if state.sort_ascending { ord } else { ord.reverse() }
    });

    let mut letters = Vec::<String>::new();
    for item in &filtered {
        let l = crate::listing::letter(&item.row.title);
        if !letters.contains(&l) {
            letters.push(l);
        }
    }

    let visible_paths: Vec<String> = filtered.iter().map(|e| e.row.path.clone()).collect();
    let marked_set: HashSet<&str> = state.marked_keys.iter().map(String::as_str).collect();
    let all_marked = !visible_paths.is_empty() && visible_paths.iter().all(|p| marked_set.contains(p.as_str()));

    let games: Vec<DesktopGame> = filtered
        .iter()
        .map(|e| {
            let cover = cover_url(&e.row.cover);
            let hero = cover_url(if e.row.hero.is_empty() { &e.row.cover } else { &e.row.hero });
            let swatch = cover_swatch(&cover, game_seed(&e.row.path));
            let badge = if has_running && session.game == e.row.path {
                words::DESKTOP_RESUME_BADGE.to_owned()
            } else if e.states_count > 0 {
                words::counted(e.states_count, "STATE", "STATES")
            } else {
                String::new()
            };
            let status_label = if e.resume_ready {
                words::DESKTOP_RESUME_LABEL.to_owned()
            } else if e.row.favourite {
                words::DESKTOP_FAVOURITE_LABEL.to_owned()
            } else {
                words::DESKTOP_STATUS_READY.to_owned()
            };
            let states_label = if e.states_count > 0 { words::counted(e.states_count, "state", "states") } else { words::DESKTOP_DASH.to_owned() };
            let subtitle = words::line(&[&e.year, &e.genre]);
            let subtitle = if subtitle.is_empty() { e.row.console.clone() } else { subtitle };
            DesktopGame {
                key: e.row.path.clone(),
                title: e.row.title.clone(),
                system: e.row.system.clone(),
                console: e.row.console.clone(),
                core: e.core.clone(),
                planet: e.planet,
                libretro: e.libretro,
                year: e.year.clone(),
                genre: e.genre.clone(),
                subtitle,
                played_words: if e.played_words.is_empty() { words::DESKTOP_DASH.to_owned() } else { e.played_words.clone() },
                played_seconds: e.played_seconds,
                last_played_words: e.last_played_words.clone(),
                favourite: e.row.favourite,
                marked: marked_set.contains(e.row.path.as_str()),
                resume_ready: e.resume_ready,
                badge,
                cover,
                hero,
                swatch,
                format: e.format.clone(),
                states_count: e.states_count as i32,
                states_label,
                status_label,
            }
        })
        .collect();

    let effective_selected = if !state.selected_key.is_empty() && visible_paths.contains(&state.selected_key) {
        state.selected_key.clone()
    } else {
        visible_paths.first().cloned().unwrap_or_default()
    };

    let selected_item = filtered.iter().find(|e| e.row.path == effective_selected);
    let mut data = DesktopWorkspaceData {
        theme_name: theme_label,
        status_text: String::new(),
        status_summary,
        mode_badge,
        big_picture_hint: words::DESKTOP_BIG_PICTURE_HINT.to_owned(),
        status_hints: words::DESKTOP_STATUS_HINTS.to_owned(),
        counts_line,
        controllers_line,
        marked_count: state.marked_keys.len() as i32,
        all_marked,
        count: filtered.len() as i32,
        total_count: f_all as i32,
        summary_line: words::desktop_summary_line(filtered.len(), f_all, state.filter_key != ALL || !state.search_query.trim().is_empty()),
        empty_words: if state.filter_key != ALL || !state.search_query.trim().is_empty() {
            words::NO_GAMES_FOUND.to_owned()
        } else if state.console_key == crate::library::FAVOURITES {
            words::NO_FAVOURITES.to_owned()
        } else if state.console_key == crate::library::RECENT {
            words::NO_RECENT.to_owned()
        } else {
            words::NO_GAMES.to_owned()
        },
        active_heading: active_heading.clone(),
        active_core,
        active_planet,
        active_screen,
        has_selection: selected_item.is_some(),
        effective_selected,
        places,
        consoles,
        filters: vec![
            DesktopFilter { key: "all".to_owned(), label: words::ALL.to_owned(), count: f_all as i32 },
            DesktopFilter { key: "favourites".to_owned(), label: words::FAVOURITES_FILTER.to_owned(), count: f_fav as i32 },
            DesktopFilter { key: "resume".to_owned(), label: words::DESKTOP_FILTER_RESUME_READY.to_owned(), count: f_res as i32 },
            DesktopFilter { key: "unplayed".to_owned(), label: words::DESKTOP_FILTER_UNPLAYED.to_owned(), count: f_unp as i32 },
        ],
        sorts: SORT_LABELS
            .iter()
            .zip(SORT_KEYS)
            .enumerate()
            .map(|(index, (label, key))| DesktopSort { index: index as i32, label: (*label).to_owned(), key: key.to_owned() })
            .collect(),
        letters,
        games,
        inspector_states: Vec::new(),
        visible_paths,
        ..DesktopWorkspaceData::default()
    };

    if let Some(item) = selected_item {
        let c_facts = cached.filter(|c| c.path == item.row.path);
        let cover = cover_url(&item.row.cover);
        let hero = cover_url(if item.row.hero.is_empty() { &item.row.cover } else { &item.row.hero });
        let swatch = cover_swatch(&hero, game_seed(&item.row.path));
        let players = c_facts.map(|c| c.facts.players.clone()).filter(|p| !p.is_empty()).unwrap_or_else(|| words::players(&meta_str(meta, &item.row.path, "players")));
        let description = c_facts.map(|c| c.facts.description.clone()).filter(|d| !d.is_empty()).unwrap_or_else(|| meta_str(meta, &item.row.path, "description"));
        let size_words = c_facts.and_then(|c| c.size_bytes).map(format_bytes).unwrap_or_else(|| words::DESKTOP_DASH.to_owned());
        let played_line = match (item.played_words.as_str(), item.last_played_words.as_str()) {
            ("", "") => words::DESKTOP_NEVER_PLAYED.to_owned(),
            ("", last) => last.to_owned(),
            (p, "") => p.to_owned(),
            (p, last) => format!("{p} · {last}"),
        };
        let subtitle = words::line(&[&item.row.console, &item.year, &item.genre]);
        let core_kind = if item.core.is_empty() {
            words::DESKTOP_DASH.to_owned()
        } else if item.libretro {
            format!("{} ({})", item.core, words::DESKTOP_LIBRETRO_SUFFIX)
        } else {
            format!("{} ({})", item.core, words::DESKTOP_NATIVE_SUFFIX)
        };
        let package_name = view.catalog.games.iter().find(|g| g.path == item.row.path).and_then(|g| g.package.as_deref()).unwrap_or_default();
        let cheats_count = c_facts.map_or(0, |c| c.cheats_count);
        let inspector_states: Vec<DesktopStateEntry> = c_facts
            .map(|c| c.states.as_slice())
            .unwrap_or(&[])
            .iter()
            .map(|s| {
                let label = match s.slot {
                    crate::game_page::Slot::Auto => words::DESKTOP_RESUME_AUTO_LABEL.to_owned(),
                    crate::game_page::Slot::Number(n) => format!("{} {n}", words::DESKTOP_SLOT_PREFIX),
                };
                DesktopStateEntry {
                    path: s.path.clone(),
                    label,
                    when_words: words::ago(s.when, now),
                    picture: cover_url(&s.picture),
                    is_auto: matches!(s.slot, crate::game_page::Slot::Auto),
                }
            })
            .collect();

        data.inspector_title = item.row.title.clone();
        data.inspector_subtitle = subtitle;
        data.inspector_console = item.row.console.clone();
        data.inspector_system = item.row.system.clone();
        data.inspector_core = item.core.clone();
        data.inspector_core_line = core_kind;
        data.inspector_planet = item.planet;
        data.inspector_libretro = item.libretro;
        data.inspector_played_words = played_line;
        data.inspector_players = if players.is_empty() { words::DESKTOP_DASH.to_owned() } else { players };
        data.inspector_genre = if item.genre.is_empty() { words::DESKTOP_DASH.to_owned() } else { item.genre.clone() };
        data.inspector_description = description;
        data.inspector_file_name = path_file_name(&item.row.path);
        data.inspector_folder_path = path_folder(&item.row.path);
        data.inspector_size_words = size_words;
        data.inspector_favourite = item.row.favourite;
        data.inspector_running = has_running && session.game == item.row.path;
        data.inspector_cover = cover;
        data.inspector_hero = hero;
        data.inspector_swatch = swatch;
        data.inspector_states_count = item.states_count as i32;
        data.inspector_states_heading = if item.states_count > 0 {
            format!("{} · {}", words::DESKTOP_SAVE_STATES_LABEL, item.states_count)
        } else {
            words::DESKTOP_SAVE_STATES_LABEL.to_owned()
        };
        data.inspector_states = inspector_states;
        data.inspector_package_words = if package_name.is_empty() { words::DESKTOP_READ_IN_PLACE.to_owned() } else { package_name.to_owned() };
        data.inspector_cheats_words = if cheats_count > 0 { words::counted(cheats_count, "code", "codes") } else { words::DESKTOP_NONE_LABEL.to_owned() };
    }

    data.status_text = if !state.marked_keys.is_empty() && state.view == 1 {
        format!("{} of {} selected ({} total in library)", words::grouped(state.marked_keys.len()), words::grouped(filtered.len()), words::grouped(all_games.len()))
    } else if let Some(item) = selected_item {
        format!("{active_heading} · {} · Selected: {}", words::games(filtered.len()), item.row.title)
    } else {
        format!("{active_heading} · {}", words::games(filtered.len()))
    };

    data
}

/// Computes all Desktop Mode JSON projections from the real library, session, saved states, and cached inspector state.
#[allow(clippy::too_many_arguments)]
pub fn project_desktop(
    view: &LibraryView,
    extra_meta: &[(String, BTreeMap<String, String>)],
    state: &DesktopState,
    session: &SessionSnapshot,
    state_counts: &BTreeMap<String, usize>,
    cached: Option<&CachedInspector>,
    theme_name: &str,
    controllers: i32,
) -> DesktopProjection {
    let ws = project_workspace(view, extra_meta, state, session, state_counts, cached, theme_name, controllers);
    let places_json: Vec<Value> = ws.places.iter().map(|p| json!({ "key": p.key, "label": p.label, "count": p.count })).collect();
    let consoles_list: Vec<Value> = ws
        .consoles
        .iter()
        .map(|c| {
            json!({
                "key": c.key,
                "name": c.name,
                "fullName": c.full_name,
                "count": c.games,
                "played": c.played,
                "planet": c.planet,
                "engine": c.engine,
                "libretro": c.libretro,
                "screen": c.screen,
                "hue": c.hue,
            })
        })
        .collect();
    let consoles_json = json!({
        "places": places_json,
        "consoles": consoles_list,
        "activeHeading": ws.active_heading,
        "activeCore": ws.active_core,
        "activePlanet": ws.active_planet,
        "activeScreen": ws.active_screen,
        "countsLine": ws.counts_line,
        "controllersLine": ws.controllers_line,
    })
    .to_string();

    let rows_json: Vec<Value> = ws
        .games
        .iter()
        .map(|g| {
            json!({
                "path": g.key,
                "title": g.title,
                "system": g.system,
                "console": g.console,
                "core": g.core,
                "planet": g.planet,
                "libretro": g.libretro,
                "year": g.year,
                "genre": g.genre,
                "subtitle": g.subtitle,
                "playedWords": g.played_words,
                "playedSeconds": g.played_seconds,
                "lastPlayedWords": g.last_played_words,
                "favourite": g.favourite,
                "marked": g.marked,
                "resumeReady": g.resume_ready,
                "badge": g.badge,
                "coverUrl": g.cover,
                "heroUrl": g.hero,
                "swatch": g.swatch,
                "format": g.format,
                "statesCount": g.states_count,
                "statesLabel": g.states_label,
                "statusLabel": g.status_label,
            })
        })
        .collect();

    let inspector_json = if ws.has_selection {
        let states_json: Vec<Value> = ws
            .inspector_states
            .iter()
            .map(|s| {
                json!({
                    "path": s.path,
                    "label": s.label,
                    "whenWords": s.when_words,
                    "pictureUrl": s.picture,
                    "isAuto": s.is_auto,
                })
            })
            .collect();
        json!({
            "hasGame": true,
            "path": ws.effective_selected,
            "title": ws.inspector_title,
            "subtitle": ws.inspector_subtitle,
            "console": ws.inspector_console,
            "system": ws.inspector_system,
            "core": ws.inspector_core,
            "coreLine": ws.inspector_core_line,
            "planet": ws.inspector_planet,
            "libretro": ws.inspector_libretro,
            "playedWords": ws.inspector_played_words,
            "players": ws.inspector_players,
            "genre": ws.inspector_genre,
            "description": ws.inspector_description,
            "fileName": ws.inspector_file_name,
            "folderPath": ws.inspector_folder_path,
            "sizeWords": ws.inspector_size_words,
            "favourite": ws.inspector_favourite,
            "running": ws.inspector_running,
            "coverUrl": ws.inspector_cover,
            "heroUrl": ws.inspector_hero,
            "swatch": ws.inspector_swatch,
            "statesCount": ws.inspector_states_count,
            "statesHeading": ws.inspector_states_heading,
            "states": states_json,
            "packageWords": ws.inspector_package_words,
            "cheatsWords": ws.inspector_cheats_words,
        })
    } else {
        json!({ "hasGame": false })
    }
    .to_string();

    let f_all = ws.filters.first().map_or(0, |f| f.count);
    let f_fav = ws.filters.get(1).map_or(0, |f| f.count);
    let f_res = ws.filters.get(2).map_or(0, |f| f.count);
    let f_unp = ws.filters.get(3).map_or(0, |f| f.count);
    let games_json = json!({
        "count": ws.count,
        "totalCount": ws.total_count,
        "summaryLine": ws.summary_line,
        "filterCounts": { "all": f_all, "favourites": f_fav, "resume": f_res, "unplayed": f_unp },
        "rows": rows_json,
        "letters": ws.letters,
        "allMarked": ws.all_marked,
    })
    .to_string();

    let pause_facts = if session.card_facts.is_empty() { None } else { Some(PauseFacts::from_json(&session.card_facts)) };
    let playing_json = build_playing_json(view, session, pause_facts.as_ref());

    DesktopProjection {
        consoles_json,
        games_json,
        inspector_json,
        playing_json,
        status_text: ws.status_text,
        mode_badge: ws.mode_badge,
        status_summary: ws.status_summary,
        effective_selected: ws.effective_selected,
        visible_paths: ws.visible_paths,
        all_marked: ws.all_marked,
    }
}

/// Computes structured playing HUD data from the active session and pause facts.
pub fn project_playing(view: &LibraryView, session: &SessionSnapshot, facts: Option<&PauseFacts>) -> DesktopPlayingData {
    let title = if session.game.is_empty() {
        if session.console.is_empty() { String::new() } else { crate::session::console_session_name(&session.console) }
    } else {
        crate::library::title_of(&session.game)
    };
    let console = if session.game.is_empty() {
        view.rows.rows().iter().find(|r| r.key == session.console).map(|r| r.name.clone()).unwrap_or_else(|| session.console.clone())
    } else {
        view.console_of(&session.game)
    };
    let system = if session.game.is_empty() {
        session.console.clone()
    } else {
        view.catalog.games.iter().find(|g| g.path == session.game).and_then(|g| g.system.clone()).unwrap_or_default()
    };
    let (core, planet, _) = system_core_info(view, &system);
    let slots: Vec<DesktopPlayingSlot> = facts
        .map(|f| {
            f.slots
                .iter()
                .map(|s| DesktopPlayingSlot {
                    slot: s.slot as i32,
                    exists: s.exists,
                    when: s.when.clone(),
                    picture: s.picture.clone(),
                    active: s.slot == session.slot,
                })
                .collect()
        })
        .unwrap_or_else(|| {
            (1..=8)
                .map(|slot| DesktopPlayingSlot {
                    slot,
                    exists: false,
                    when: String::new(),
                    picture: String::new(),
                    active: slot as u32 == session.slot,
                })
                .collect()
        });
    let cheats: Vec<DesktopPlayingCheat> = facts
        .map(|f| {
            f.cheats
                .iter()
                .enumerate()
                .map(|(i, c)| DesktopPlayingCheat { index: i as i32, description: c.description.clone(), on: c.on, code: String::new() })
                .collect()
        })
        .unwrap_or_default();
    let console_options: Vec<DesktopPlayingConsoleOption> = facts
        .map(|f| {
            f.cards
                .schema
                .iter()
                .map(|r| {
                    let value = f.cards.value(&r.key);
                    let choices = r.choices.clone().unwrap_or_default();
                    let choice_labels = r.choice_labels.clone().unwrap_or_default();
                    let display_value = choices
                        .iter()
                        .position(|c| c == &value)
                        .and_then(|i| choice_labels.get(i))
                        .cloned()
                        .unwrap_or_else(|| value.clone());
                    DesktopPlayingConsoleOption {
                        key: r.key.clone(),
                        label: r.label.clone(),
                        value,
                        display_value,
                        choice_count: choices.len() as i32,
                        choices,
                        choice_labels,
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    let rewind_marks: Vec<DesktopPlayingRewindMark> = facts
        .filter(|f| f.rewind_why.is_none() && !f.held.is_empty())
        .map(|f| {
            crate::reel::marks_of(&f.held, f.now, f.hz)
                .into_iter()
                .map(|m| DesktopPlayingRewindMark {
                    frame: m.frame as f64,
                    label: m.label(),
                    short: m.short(),
                    picture: f.moment_picture(&m),
                    now: m.now,
                })
                .collect()
        })
        .unwrap_or_default();
    let latest_rewind = facts.and_then(|f| f.held.last().copied()).unwrap_or(0);
    let has_rewind = facts.is_some_and(|f| f.rewind_why.is_none() && !f.held.is_empty());

    DesktopPlayingData {
        running: session.running,
        paused: session.paused,
        game: session.game.clone(),
        system,
        title,
        console,
        core,
        planet,
        fps_words: session.fps_words.clone(),
        status: session.status.clone(),
        slot: session.slot as i32,
        speed: session.speed as i32,
        master_cheats: session.master_cheats,
        has_rewind,
        latest_rewind_frame: latest_rewind as f64,
        slots,
        cheats,
        console_options,
        rewind_marks,
    }
}

fn build_playing_json(view: &LibraryView, session: &SessionSnapshot, facts: Option<&PauseFacts>) -> String {
    let p = project_playing(view, session, facts);
    let slots: Vec<Value> = p.slots.iter().map(|s| json!({ "slot": s.slot, "exists": s.exists, "when": s.when, "picture": s.picture })).collect();
    let cheats: Vec<Value> = p.cheats.iter().map(|c| json!({ "index": c.index, "description": c.description, "on": c.on })).collect();
    let console_options: Vec<Value> = p
        .console_options
        .iter()
        .map(|o| {
            json!({
                "key": o.key,
                "label": o.label,
                "value": o.value,
                "choices": o.choices,
                "choiceLabels": o.choice_labels,
            })
        })
        .collect();
    json!({
        "running": p.running,
        "paused": p.paused,
        "game": p.game,
        "system": p.system,
        "title": p.title,
        "console": p.console,
        "core": p.core,
        "planet": p.planet,
        "fpsWords": p.fps_words,
        "status": p.status,
        "slot": p.slot,
        "speed": p.speed,
        "masterCheats": p.master_cheats,
        "slots": slots,
        "cheats": cheats,
        "consoleOptions": console_options,
        "hasRewind": p.has_rewind,
        "latestRewindFrame": p.latest_rewind_frame as u64,
    })
    .to_string()
}

fn build_settings_json(settings: Option<&SettingsModel>, filter: &str, tool_index: i32) -> String {
    let Some(model) = settings else {
        return json!({
            "section": 0,
            "trailLine": "",
            "heading": "Controllers",
            "canBack": false,
            "toolIndex": tool_index,
            "sections": [],
            "rows": [],
            "asideHeading": "",
            "asideSentence": "",
            "asideLines": [],
            "asideMuted": [],
            "preview": 0,
            "meter": 0.0,
            "caption": "",
            "samples": [],
        })
        .to_string();
    };
    let view = model.view();
    let sections: Vec<Value> = view
        .sections()
        .into_iter()
        .enumerate()
        .map(|(i, name)| json!({ "index": i, "name": name }))
        .collect();
    let shown = model.shown_rows();
    let facets = view.facets();
    let q = filter.trim().to_lowercase();
    let rows: Vec<Value> = shown
        .iter()
        .enumerate()
        .filter(|(_, r)| q.is_empty() || r.label.to_lowercase().contains(&q) || r.value.to_lowercase().contains(&q))
        .map(|(i, r)| {
            let facet = facets.get(i);
            let slider = view.slider(i);
            json!({
                "index": i,
                "id": r.id,
                "label": r.label,
                "value": r.value,
                "about": facet.map_or("", |f| f.about.as_str()),
                "kind": r.kind.index(),
                "isSwitch": facet.is_some_and(|f| f.switch.is_some()),
                "switchOn": facet.is_some_and(|f| f.switch == Some(true)),
                "isChoice": matches!(r.kind, RowKind::Steps),
                "isSlider": matches!(r.kind, RowKind::Slider),
                "isDoor": matches!(r.kind, RowKind::Opens | RowKind::Asks) || (matches!(r.kind, RowKind::Action) && r.value.is_empty()),
                "code": r.code,
                "marked": r.marked,
                "sliderMin": slider.map_or(0.0, |s| s[0]),
                "sliderMax": slider.map_or(1.0, |s| s[1]),
                "sliderStep": slider.map_or(0.1, |s| s[2]),
                "sliderAmount": slider.map_or(0.0, |s| s[3]),
            })
        })
        .collect();
    let trail: Vec<String> = model.trail().iter().map(|t| t.to_string()).collect();
    let lines: Vec<String> = model.pane_lines().iter().map(|t| t.to_string()).collect();
    let muted: Vec<String> = model.pane_muted().iter().map(|t| t.to_string()).collect();
    let samples: Vec<String> = model.samples().iter().map(|t| t.to_string()).collect();
    let focused = *model.focused();
    let aside_heading = usize::try_from(focused).ok().and_then(|i| shown.get(i)).map(|r| r.label.clone()).unwrap_or_default();

    json!({
        "section": *model.section(),
        "trailLine": trail.join(" › "),
        "heading": view.heading(),
        "canBack": !trail.is_empty() || tool_index >= 0,
        "toolIndex": tool_index,
        "sections": sections,
        "rows": rows,
        "focused": focused,
        "asideHeading": aside_heading,
        "asideSentence": model.sentence().to_string(),
        "asideLines": lines,
        "asideMuted": muted,
        "preview": *model.preview(),
        "meter": *model.meter(),
        "caption": model.caption().to_string(),
        "samples": samples,
    })
    .to_string()
}

pub struct DesktopModelRust {
    library: *mut qobject::LibraryModel,
    session: *mut qobject::SessionModel,
    settings: *mut qobject::SettingsModel,
    theme_name: QString,
    has_game: bool,
    game_paused: bool,
    game_shown: bool,
    full_screen: bool,
    controllers: i32,
    clock_text: QString,
    battery: i32,
    view: i32,
    sidebar_open: bool,
    inspector_open: bool,
    drawer_open: bool,
    drawer_tab: i32,
    console_key: QString,
    filter_key: QString,
    search_query: QString,
    console_filter: QString,
    console_scope: i32,
    sort_index: i32,
    sort_ascending: bool,
    selected_key: QString,
    marked_keys: QStringList,
    marked_count: i32,
    all_marked: bool,
    settings_tool_index: i32,
    settings_filter: QString,
    settings_modified_only: bool,
    chrome_json: QString,
    consoles_json: QString,
    games_json: QString,
    inspector_json: QString,
    playing_json: QString,
    settings_json: QString,
    status_text: QString,
    mode_badge: QString,
    status_summary: QString,
    state: DesktopState,
    visible_paths: Vec<String>,
    state_counts: BTreeMap<String, usize>,
    counts_sig: Option<(usize, bool, String, bool)>,
    cached: Option<CachedInspector>,
    asking_path: String,
}

impl Default for DesktopModelRust {
    fn default() -> Self {
        let state = DesktopState::default();
        let empty_counts = BTreeMap::new();
        let proj = project_desktop(&LibraryView::empty(), &[], &state, &SessionSnapshot::default(), &empty_counts, None, "EmuSen", 0);
        DesktopModelRust {
            library: std::ptr::null_mut(),
            session: std::ptr::null_mut(),
            settings: std::ptr::null_mut(),
            theme_name: QString::from("EmuSen"),
            has_game: false,
            game_paused: false,
            game_shown: false,
            full_screen: false,
            controllers: 0,
            clock_text: QString::default(),
            battery: -1,
            view: state.view,
            sidebar_open: state.sidebar_open,
            inspector_open: state.inspector_open,
            drawer_open: state.drawer_open,
            drawer_tab: state.drawer_tab,
            console_key: QString::from(state.console_key.as_str()),
            filter_key: QString::from(state.filter_key.as_str()),
            search_query: QString::default(),
            console_filter: QString::default(),
            console_scope: state.console_scope,
            sort_index: state.sort_index,
            sort_ascending: state.sort_ascending,
            selected_key: QString::default(),
            marked_keys: QStringList::default(),
            marked_count: 0,
            all_marked: false,
            settings_tool_index: state.settings_tool_index,
            settings_filter: QString::default(),
            settings_modified_only: state.settings_modified_only,
            chrome_json: QString::from(chrome_json().as_str()),
            consoles_json: QString::from(proj.consoles_json.as_str()),
            games_json: QString::from(proj.games_json.as_str()),
            inspector_json: QString::from(proj.inspector_json.as_str()),
            playing_json: QString::from(proj.playing_json.as_str()),
            settings_json: QString::from(build_settings_json(None, "", -1).as_str()),
            status_text: QString::from(proj.status_text.as_str()),
            mode_badge: QString::from(proj.mode_badge.as_str()),
            status_summary: QString::from(proj.status_summary.as_str()),
            state,
            visible_paths: proj.visible_paths,
            state_counts: empty_counts,
            counts_sig: None,
            cached: None,
            asking_path: String::new(),
        }
    }
}

impl cxx_qt::Initialize for qobject::DesktopModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_library_changed(|m| m.refresh()).release();
        self.as_mut().on_session_changed(|m| m.refresh()).release();
        self.as_mut().on_settings_changed(|m| m.refresh()).release();
        self.as_mut().on_theme_name_changed(|m| m.refresh()).release();
        self.as_mut().on_has_game_changed(|m| m.refresh()).release();
        self.as_mut().on_game_paused_changed(|m| m.refresh()).release();
        self.as_mut().on_game_shown_changed(|m| m.refresh()).release();
        self.as_mut().on_full_screen_changed(|m| m.refresh()).release();
        self.as_mut().on_controllers_changed(|m| m.refresh()).release();
    }
}

fn to_qstring_list(items: &[String]) -> QStringList {
    let mut out = QList::<QString>::default();
    for item in items {
        out.append(QString::from(item.as_str()));
    }
    QStringList::from(&out)
}

impl qobject::DesktopModel {
    fn session_snapshot(&self) -> SessionSnapshot {
        let Some(s) = (unsafe { self.session.as_ref() }) else {
            return SessionSnapshot::default();
        };
        SessionSnapshot {
            running: *s.running(),
            paused: *s.paused(),
            game: s.game().to_string(),
            console: s.console().to_string(),
            fps_words: s.fps_words().to_string(),
            status: s.status().to_string(),
            slot: *s.slot(),
            speed: *s.speed(),
            master_cheats: *s.master_cheats(),
            card_facts: s.card_facts().to_string(),
        }
    }

    pub fn set_view(mut self: Pin<&mut Self>, view: i32) {
        let changed = self.as_mut().rust_mut().state.set_view(view);
        if changed {
            let next = self.state.view;
            self.as_mut().rust_mut().view = next;
            self.as_mut().view_changed();
            self.refresh();
        }
    }

    pub fn set_sidebar_open(mut self: Pin<&mut Self>, open: bool) {
        if self.sidebar_open != open {
            self.as_mut().rust_mut().state.sidebar_open = open;
            self.as_mut().rust_mut().sidebar_open = open;
            self.as_mut().sidebar_open_changed();
        }
    }

    pub fn set_inspector_open(mut self: Pin<&mut Self>, open: bool) {
        if self.inspector_open != open {
            self.as_mut().rust_mut().state.inspector_open = open;
            self.as_mut().rust_mut().inspector_open = open;
            self.as_mut().inspector_open_changed();
        }
    }

    pub fn set_drawer_open(mut self: Pin<&mut Self>, open: bool) {
        if self.drawer_open != open {
            self.as_mut().rust_mut().state.drawer_open = open;
            self.as_mut().rust_mut().drawer_open = open;
            self.as_mut().drawer_open_changed();
        }
    }

    pub fn set_drawer_tab(mut self: Pin<&mut Self>, tab: i32) {
        let tab = tab.clamp(0, 3);
        if self.drawer_tab != tab {
            self.as_mut().rust_mut().state.drawer_tab = tab;
            self.as_mut().rust_mut().drawer_tab = tab;
            self.as_mut().drawer_tab_changed();
        }
    }

    pub fn set_console_key(mut self: Pin<&mut Self>, key: QString) {
        if self.console_key != key {
            self.as_mut().rust_mut().state.console_key = key.to_string();
            self.as_mut().rust_mut().console_key = key;
            self.as_mut().console_key_changed();
            self.refresh();
        }
    }

    pub fn set_filter_key(mut self: Pin<&mut Self>, key: QString) {
        if self.filter_key != key {
            self.as_mut().rust_mut().state.filter_key = key.to_string();
            self.as_mut().rust_mut().filter_key = key;
            self.as_mut().filter_key_changed();
            self.refresh();
        }
    }

    pub fn set_search_query(mut self: Pin<&mut Self>, query: QString) {
        if self.search_query != query {
            self.as_mut().rust_mut().state.search_query = query.to_string();
            self.as_mut().rust_mut().search_query = query;
            self.as_mut().search_query_changed();
            self.refresh();
        }
    }

    pub fn set_console_filter(mut self: Pin<&mut Self>, filter: QString) {
        if self.console_filter != filter {
            self.as_mut().rust_mut().state.console_filter = filter.to_string();
            self.as_mut().rust_mut().console_filter = filter;
            self.as_mut().console_filter_changed();
            self.refresh();
        }
    }

    pub fn set_console_scope(mut self: Pin<&mut Self>, scope: i32) {
        let scope = scope.clamp(0, 2);
        if self.console_scope != scope {
            self.as_mut().rust_mut().state.console_scope = scope;
            self.as_mut().rust_mut().console_scope = scope;
            self.as_mut().console_scope_changed();
            self.refresh();
        }
    }

    pub fn set_sort_index(mut self: Pin<&mut Self>, index: i32) {
        let index = index.clamp(0, (SORT_LABELS.len() - 1) as i32);
        if self.sort_index != index {
            self.as_mut().rust_mut().state.sort_index = index;
            self.as_mut().rust_mut().sort_index = index;
            self.as_mut().sort_index_changed();
            self.refresh();
        }
    }

    pub fn set_sort_ascending(mut self: Pin<&mut Self>, ascending: bool) {
        if self.sort_ascending != ascending {
            self.as_mut().rust_mut().state.sort_ascending = ascending;
            self.as_mut().rust_mut().sort_ascending = ascending;
            self.as_mut().sort_ascending_changed();
            self.refresh();
        }
    }

    pub fn set_selected_key(mut self: Pin<&mut Self>, key: QString) {
        if self.selected_key != key {
            self.as_mut().rust_mut().state.selected_key = key.to_string();
            self.as_mut().rust_mut().selected_key = key;
            self.as_mut().selected_key_changed();
            self.refresh();
        }
    }

    pub fn set_settings_filter(mut self: Pin<&mut Self>, filter: QString) {
        if self.settings_filter != filter {
            self.as_mut().rust_mut().state.settings_filter = filter.to_string();
            self.as_mut().rust_mut().settings_filter = filter;
            self.as_mut().settings_filter_changed();
            self.refresh();
        }
    }

    pub fn set_settings_modified_only(mut self: Pin<&mut Self>, modified: bool) {
        if self.settings_modified_only != modified {
            self.as_mut().rust_mut().state.settings_modified_only = modified;
            self.as_mut().rust_mut().settings_modified_only = modified;
            self.as_mut().settings_modified_only_changed();
            self.refresh();
        }
    }

    pub fn toggle_sidebar(mut self: Pin<&mut Self>) {
        let next = !self.sidebar_open;
        self.as_mut().set_sidebar_open(next);
    }

    pub fn toggle_inspector(mut self: Pin<&mut Self>) {
        let next = !self.inspector_open;
        self.as_mut().set_inspector_open(next);
    }

    pub fn toggle_drawer(mut self: Pin<&mut Self>) {
        let next = !self.drawer_open;
        self.as_mut().set_drawer_open(next);
        if next {
            if let Some(s) = unsafe { self.session.as_mut() } {
                unsafe { Pin::new_unchecked(s) }.refresh_facts();
            }
        }
    }

    pub fn open_drawer_tab(mut self: Pin<&mut Self>, tab: i32) {
        let tab = tab.clamp(0, 3);
        if self.drawer_open && self.drawer_tab == tab {
            self.as_mut().set_drawer_open(false);
        } else {
            self.as_mut().set_drawer_tab(tab);
            self.as_mut().set_drawer_open(true);
            if let Some(s) = unsafe { self.session.as_mut() } {
                unsafe { Pin::new_unchecked(s) }.refresh_facts();
            }
        }
    }

    pub fn select_console(self: Pin<&mut Self>, key: &QString) {
        self.set_console_key(key.clone());
    }

    pub fn step_sort(mut self: Pin<&mut Self>) {
        let next = (self.sort_index + 1) % SORT_LABELS.len() as i32;
        self.as_mut().set_sort_index(next);
    }

    pub fn sort_by_column(mut self: Pin<&mut Self>, column: i32) {
        let column = column.clamp(0, (SORT_LABELS.len() - 1) as i32);
        if self.sort_index == column {
            let asc = !self.sort_ascending;
            self.as_mut().set_sort_ascending(asc);
        } else {
            self.as_mut().rust_mut().state.sort_ascending = true;
            self.as_mut().rust_mut().sort_ascending = true;
            self.as_mut().sort_ascending_changed();
            self.as_mut().set_sort_index(column);
        }
    }

    pub fn select_game(self: Pin<&mut Self>, path: &QString) {
        self.set_selected_key(path.clone());
    }

    pub fn jump_letter(&self, letter: &QString) -> i32 {
        let target = letter.to_string().to_uppercase();
        let Some(lib) = (unsafe { self.library.as_ref() }) else { return -1 };
        let view = lib.view();
        for (idx, path) in self.visible_paths.iter().enumerate() {
            if let Some(g) = view.game(path) {
                if crate::listing::letter(&g.title) == target {
                    return idx as i32;
                }
            }
        }
        -1
    }

    pub fn mark_toggle(mut self: Pin<&mut Self>, path: &QString) {
        self.as_mut().rust_mut().state.toggle_mark(&path.to_string());
        self.as_mut().sync_marks();
        self.refresh();
    }

    pub fn mark_all(mut self: Pin<&mut Self>) {
        let paths = self.visible_paths.clone();
        self.as_mut().rust_mut().state.mark_all(&paths);
        self.as_mut().sync_marks();
        self.refresh();
    }

    pub fn clear_marked(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().state.clear_marks();
        self.as_mut().sync_marks();
        self.refresh();
    }

    fn sync_marks(mut self: Pin<&mut Self>) {
        let qlist = to_qstring_list(&self.state.marked_keys);
        let count = self.state.marked_keys.len() as i32;
        self.as_mut().rust_mut().marked_keys = qlist;
        self.as_mut().marked_keys_changed();
        if self.marked_count != count {
            self.as_mut().rust_mut().marked_count = count;
            self.as_mut().marked_count_changed();
        }
    }

    fn effective_selected_path(&self) -> String {
        let selected = self.selected_key.to_string();
        if !selected.is_empty() && self.visible_paths.iter().any(|p| p == &selected) {
            return selected;
        }
        self.visible_paths.first().cloned().unwrap_or_default()
    }

    pub fn batch_favourite(self: Pin<&mut Self>, on: bool) {
        let Some(lib) = (unsafe { self.library.as_mut() }) else { return };
        let favs: HashSet<String> = lib.view().catalog.favourites.iter().cloned().collect();
        let targets: Vec<String> = self
            .state
            .marked_keys
            .iter()
            .filter(|p| favs.contains(*p) != on)
            .cloned()
            .collect();
        unsafe { Pin::new_unchecked(lib) }.batch_favourite(targets, on);
    }

    pub fn toggle_selected_favourite(self: Pin<&mut Self>) {
        let target = self.effective_selected_path();
        if target.is_empty() {
            return;
        }
        if let Some(lib) = unsafe { self.library.as_mut() } {
            unsafe { Pin::new_unchecked(lib) }.toggle_favourite(&QString::from(target.as_str()));
        }
    }

    pub fn open_selected_folder(self: Pin<&mut Self>) {
        let target = self.effective_selected_path();
        let folder = path_folder(&target);
        if !folder.is_empty() {
            self.open_folder_requested(QString::from(folder.as_str()));
        }
    }

    fn rewind_settings_trail(mut s: Pin<&mut SettingsModel>) {
        for _ in 0..8 {
            if s.trail().is_empty() || !s.as_mut().back() {
                break;
            }
        }
    }

    pub fn open_console_options(mut self: Pin<&mut Self>, key: &QString) {
        let console_id = key.to_string();
        if let Some(settings) = unsafe { self.settings.as_mut() } {
            let mut s = unsafe { Pin::new_unchecked(settings) };
            Self::rewind_settings_trail(s.as_mut());
            if !console_id.is_empty() && console_id != ALL && console_id != crate::library::FAVOURITES && console_id != crate::library::RECENT {
                s.as_mut().open_console(&QString::from(console_id.as_str()));
            } else {
                s.as_mut().set_section(3);
            }
        }
        self.as_mut().set_settings_tool(-1);
        self.as_mut().set_view(2);
    }

    fn set_settings_tool(mut self: Pin<&mut Self>, index: i32) {
        if self.settings_tool_index != index {
            self.as_mut().rust_mut().state.settings_tool_index = index;
            self.as_mut().rust_mut().settings_tool_index = index;
            self.as_mut().settings_tool_index_changed();
        }
    }

    pub fn select_settings_section(mut self: Pin<&mut Self>, index: i32) {
        self.as_mut().set_settings_tool(-1);
        if let Some(settings) = unsafe { self.settings.as_mut() } {
            let mut s = unsafe { Pin::new_unchecked(settings) };
            Self::rewind_settings_trail(s.as_mut());
            s.as_mut().set_section(index);
        }
        self.refresh();
    }

    pub fn open_settings_tool(mut self: Pin<&mut Self>, index: i32) {
        let Some(key) = TOOL_KEYS.get(usize::try_from(index).ok().unwrap_or(usize::MAX)).copied() else {
            return;
        };
        if key == "themes" {
            self.as_mut().set_settings_tool(index);
            self.refresh();
            return;
        }
        self.as_mut().set_settings_tool(-1);
        if let Some(settings) = unsafe { self.settings.as_mut() } {
            let mut s = unsafe { Pin::new_unchecked(settings) };
            Self::rewind_settings_trail(s.as_mut());
            match key {
                "cores" => s.as_mut().set_section(3),
                "shaders" => s.as_mut().set_section(1),
                "scraper" => {
                    s.as_mut().set_section(4);
                    if let Some(row) = s.shown_rows().iter().position(|r| r.id == "scraping") {
                        s.as_mut().choose(row as i32);
                    }
                }
                _ => s.as_mut().set_section(4),
            }
        }
        self.refresh();
    }

    pub fn settings_back(mut self: Pin<&mut Self>) {
        if self.settings_tool_index >= 0 {
            self.as_mut().set_settings_tool(-1);
            self.refresh();
            return;
        }
        if let Some(settings) = unsafe { self.settings.as_mut() } {
            unsafe { Pin::new_unchecked(settings) }.back();
        }
        self.refresh();
    }

    pub fn settings_choose(self: Pin<&mut Self>, row: i32) {
        if let Some(settings) = unsafe { self.settings.as_mut() } {
            unsafe { Pin::new_unchecked(settings) }.choose(row);
        }
        self.refresh();
    }

    pub fn settings_step(self: Pin<&mut Self>, row: i32, by: i32) {
        if let Some(settings) = unsafe { self.settings.as_mut() } {
            unsafe { Pin::new_unchecked(settings) }.step(row, by);
        }
        self.refresh();
    }

    pub fn settings_slide(self: Pin<&mut Self>, row: i32, amount: f64) {
        if let Some(settings) = unsafe { self.settings.as_mut() } {
            unsafe { Pin::new_unchecked(settings) }.slide(row, amount);
        }
        self.refresh();
    }

    pub fn settings_focus(self: Pin<&mut Self>, row: i32) {
        if let Some(settings) = unsafe { self.settings.as_mut() } {
            unsafe { Pin::new_unchecked(settings) }.set_focused(row);
        }
        self.refresh();
    }

    pub fn handle_settings_door(self: Pin<&mut Self>, name: &QString) {
        self.open_door(&name.to_string());
    }

    pub fn open_door(mut self: Pin<&mut Self>, key: &str) -> bool {
        match key {
            "themes" => {
                self.as_mut().set_settings_tool(5);
                self.refresh();
                true
            }
            "shaders" => {
                self.as_mut().set_settings_tool(-1);
                if let Some(settings) = unsafe { self.settings.as_mut() } {
                    unsafe { Pin::new_unchecked(settings) }.open_shaders_door();
                }
                self.refresh();
                true
            }
            "bindings" | "keyboard" => {
                self.as_mut().set_settings_tool(-1);
                if let Some(settings) = unsafe { self.settings.as_mut() } {
                    let mut s = unsafe { Pin::new_unchecked(settings) };
                    if let Some(row) = s.as_mut().focus_section_row(0, key) {
                        s.as_mut().choose(row as i32);
                    }
                }
                self.refresh();
                true
            }
            "levers" => {
                self.as_mut().set_settings_tool(-1);
                if let Some(settings) = unsafe { self.settings.as_mut() } {
                    let mut s = unsafe { Pin::new_unchecked(settings) };
                    if let Some(row) = s.as_mut().focus_section_row(1, "levers") {
                        s.as_mut().choose(row as i32);
                    }
                }
                self.refresh();
                true
            }
            "cores" => {
                self.as_mut().select_settings_section(3);
                true
            }
            "library" | "packages" => {
                self.as_mut().select_settings_section(4);
                true
            }
            "scraper" | "compressed" | "cheats" => {
                self.as_mut().door_requested(QString::from(key));
                true
            }
            _ => false,
        }
    }

    pub fn open_folder_for(self: Pin<&mut Self>, key: &str) -> bool {
        let target = if key.is_empty() { self.effective_selected_path() } else { key.to_owned() };
        let folder = path_folder(&target);
        if folder.is_empty() {
            return false;
        }
        self.open_folder_requested(QString::from(folder.as_str()));
        true
    }

    pub fn launch_from(self: Pin<&mut Self>, path: &str, state: &str) -> bool {
        if path.is_empty() {
            return false;
        }
        self.launch_from_requested(QString::from(path), QString::from(state));
        true
    }

    pub fn add_games_to_collection(self: Pin<&mut Self>, games: Vec<String>, collection_id: &str) -> bool {
        let Some(lib) = (unsafe { self.library.as_mut() }) else {
            return false;
        };
        if games.is_empty() {
            return false;
        }
        let pin = unsafe { Pin::new_unchecked(lib) };
        if collection_id == "favourites" {
            pin.batch_favourite(games, true);
            return true;
        }
        let raw = collection_id.strip_prefix("collection:").unwrap_or(collection_id);
        let Ok(id) = raw.parse::<i64>() else {
            return false;
        };
        pin.batch_add_to_collection(id, games);
        true
    }

    pub fn workspace_data(&self) -> DesktopWorkspaceData {
        let lib = unsafe { self.library.as_ref() };
        let empty = LibraryView::empty();
        let (view, meta) = match lib {
            Some(l) => (l.view(), l.effective_metadata()),
            None => (&empty, &[][..]),
        };
        let session = self.session_snapshot();
        let theme_name = self.theme_name.to_string();
        project_workspace(view, meta, &self.state, &session, &self.state_counts, self.cached.as_ref(), &theme_name, self.controllers)
    }

    pub fn state(&self) -> &DesktopState {
        &self.state
    }

    pub fn playing_data(&self) -> DesktopPlayingData {
        let lib = unsafe { self.library.as_ref() };
        let empty = LibraryView::empty();
        let view = lib.map_or(&empty, |l| l.view());
        let session = self.session_snapshot();
        let facts = if session.card_facts.is_empty() { None } else { Some(PauseFacts::from_json(&session.card_facts)) };
        project_playing(view, &session, facts.as_ref())
    }

    pub fn clear_filters(mut self: Pin<&mut Self>) {
        let mut changed = false;
        if self.state.filter_key != ALL {
            self.as_mut().rust_mut().state.filter_key = ALL.to_owned();
            self.as_mut().rust_mut().filter_key = QString::from(ALL);
            self.as_mut().filter_key_changed();
            changed = true;
        }
        if !self.state.search_query.is_empty() {
            self.as_mut().rust_mut().state.search_query.clear();
            self.as_mut().rust_mut().search_query = QString::default();
            self.as_mut().search_query_changed();
            changed = true;
        }
        if changed {
            self.refresh();
        }
    }

    pub fn refresh(mut self: Pin<&mut Self>) {
        let lib = unsafe { self.library.as_ref() };
        let empty = LibraryView::empty();
        let (view, meta, lib_ready) = match lib {
            Some(l) => (l.view(), l.effective_metadata(), !*l.loading()),
            None => (&empty, &[][..], false),
        };
        let game_count = view.catalog.games.len();
        let session = self.session_snapshot();
        let theme_name = self.theme_name.to_string();
        let proj = project_desktop(view, meta, &self.state, &session, &self.state_counts, self.cached.as_ref(), &theme_name, self.controllers);
        let effective = proj.effective_selected.clone();

        {
            let mut r = self.as_mut().rust_mut();
            r.visible_paths = proj.visible_paths;
            r.consoles_json = QString::from(proj.consoles_json.as_str());
            r.games_json = QString::from(proj.games_json.as_str());
            r.inspector_json = QString::from(proj.inspector_json.as_str());
            r.playing_json = QString::from(proj.playing_json.as_str());
            r.status_text = QString::from(proj.status_text.as_str());
            r.mode_badge = QString::from(proj.mode_badge.as_str());
            r.status_summary = QString::from(proj.status_summary.as_str());
            r.all_marked = proj.all_marked;
            let settings_ref = unsafe { r.settings.as_ref() };
            r.settings_json = QString::from(build_settings_json(settings_ref, &r.state.settings_filter, r.state.settings_tool_index).as_str());
        }
        self.as_mut().consoles_json_changed();
        self.as_mut().games_json_changed();
        self.as_mut().inspector_json_changed();
        self.as_mut().playing_json_changed();
        self.as_mut().settings_json_changed();
        self.as_mut().status_text_changed();
        self.as_mut().mode_badge_changed();
        self.as_mut().status_summary_changed();
        self.as_mut().all_marked_changed();

        let sig = (game_count, session.running, session.game.clone(), session.paused);
        if lib_ready && self.counts_sig.as_ref() != Some(&sig) {
            self.as_mut().rust_mut().counts_sig = Some(sig.clone());
            self.as_mut().ask_state_counts(sig);
        }
        if !effective.is_empty() && self.asking_path != effective && self.cached.as_ref().is_none_or(|c| c.path != effective) {
            self.ask_inspector_facts(&effective);
        }
    }

    fn ask_state_counts(self: Pin<&mut Self>, sig: (usize, bool, String, bool)) {
        let Some(lib) = (unsafe { self.library.as_ref() }) else { return };
        let Some(source) = lib.source() else { return };
        let games = lib.view().catalog.games.clone();
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let mut counts = BTreeMap::new();
            for game in &games {
                let count = source.state_count(game).unwrap_or(0);
                if count > 0 {
                    counts.insert(game.path.clone(), count);
                }
            }
            let _ = qt.queue(move |mut model| {
                if model.counts_sig.as_ref() == Some(&sig) && model.state_counts != counts {
                    model.as_mut().rust_mut().state_counts = counts;
                    model.refresh();
                }
            });
        });
    }

    fn ask_inspector_facts(mut self: Pin<&mut Self>, path: &str) {
        let Some(lib) = (unsafe { self.library.as_ref() }) else { return };
        let Some(source) = lib.source() else { return };
        let view = lib.view();
        let Some(game) = view.catalog.games.iter().find(|g| g.path == path).cloned() else { return };
        let seconds = seconds_played(&view.catalog, path);
        let console = view.systems.iter().find(|s| Some(&s.id) == game.system.as_ref()).map(|s| s.console.clone()).unwrap_or_default();
        self.as_mut().rust_mut().asking_path = path.to_owned();
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let facts = crate::library::facts_of(source.as_ref(), &game, seconds).unwrap_or_else(|_| GameFacts { path: game.path.clone(), ..Default::default() });
            let size_bytes = std::fs::metadata(&game.path).ok().map(|m| m.len());
            let cheats_count = if console.is_empty() { 0 } else { source.cheats(&console, &game.path).unwrap_or(0) };
            let states = source.states(&game).unwrap_or_default();
            let cached = CachedInspector { path: game.path, facts, size_bytes, cheats_count, states };
            let _ = qt.queue(move |mut model| {
                if model.asking_path == cached.path {
                    model.as_mut().rust_mut().cached = Some(cached);
                    model.refresh();
                }
            });
        });
    }
}
