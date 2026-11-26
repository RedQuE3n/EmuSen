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

const SORT_LABELS: [&str; 7] = ["Title", "Console", "Core", "Year", "Genre", "Played", "States"];
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
            "navTabs": ["Library", "Table", "Settings"],
            "viewModes": ["Grid", "Table", "Settings"],
            "searchPlaceholder": "Search games, consoles, or filters...",
            "bigPictureLabel": "Big Picture",
            "bigPictureHint": "F10",
            "resumeLabel": "Resume",
            "sidebarTip": "Toggle sidebar",
            "inspectorTip": "Toggle inspector",
            "constellationHeading": "CONSTELLATION",
            "continueLabel": words::CONTINUE_ENTRY,
            "allGamesLabel": "All Games",
            "favouritesLabel": words::FAVOURITES_FILTER,
            "recentLabel": "Recently Played",
            "consolesHeading": "CONSOLES",
            "scopeAll": words::ALL,
            "scopeActive": "Active",
            "consoleFilterPlaceholder": "Filter consoles...",
            "idleStatus": "Idle",
            "runningStatus": "Running",
            "pausedStatus": "Paused",
            "noControllers": "No controller",
            "batchGuaranteeTitle": "Read-only library",
            "contractNote": "Library folders are read in place; source ROMs are never modified."
        }),
    );
    merge(
        &mut map,
        json!({
            "filters": [
                { "key": "all", "label": "All" },
                { "key": "favourites", "label": "Favourites" },
                { "key": "resume", "label": "Resume Ready" },
                { "key": "unplayed", "label": "Unplayed" }
            ],
            "sortLabel": "Sort:",
            "sortOptions": SORT_LABELS,
            "consoleOptionsLabel": "Console Options",
            "resumeBadge": "RESUME",
            "emptyLibrary": words::NO_GAMES,
            "emptyFiltered": words::NO_GAMES_FOUND,
            "inspectorHeading": "INSPECTOR",
            "playLabel": "Play",
            "resumeSlotLabel": "Resume",
            "saveStatesLabel": "SAVE STATES",
            "noStatesLabel": "No saved states yet",
            "programFactsLabel": "PROGRAM FACTS",
            "factCore": "Core",
            "factPlayed": "Time played",
            "factPlayers": "Players",
            "factGenre": "Genre",
            "factFile": "File",
            "factSize": "Size",
            "factLastPlayed": "Last played",
            "favouriteLabel": "Favourite",
            "unfavouriteLabel": "Favourited",
            "detailsLabel": "Page",
            "folderLabel": "Folder",
            "libretroSuffix": "libretro",
            "nativeSuffix": "EmuSen native"
        }),
    );
    merge(
        &mut map,
        json!({
            "tableHeading": "Library Table",
            "clearFiltersLabel": "Clear filters",
            "columns": ["Title", "Console", "Core", "Year", "Genre", "Time Played", "States", "Format", "Status"],
            "sortAsc": "↑",
            "sortDesc": "↓",
            "statusResume": "Resume",
            "statusFav": "Favourite",
            "statusReady": "Ready",
            "selectedSuffix": "selected",
            "clearSelectionLabel": "Clear",
            "batchMarkFav": "Favourite",
            "batchUnmarkFav": "Unfavourite",
            "batchPage": "Page",
            "batchPlay": "Play",
            "batchClear": "Clear",
            "dash": "—"
        }),
    );
    merge(
        &mut map,
        json!({
            "stripLibrary": "Library",
            "stripPause": "Pause",
            "stripResume": "Resume",
            "stripRewind": "Rewind",
            "stripStates": "States",
            "stripShaders": "Shaders",
            "stripCheats": "Cheats",
            "stripControls": "Options",
            "stripScreenshot": "Shot",
            "stripFullscreen": "Fullscreen",
            "stripBigPicture": "Big Picture",
            "drawerTabs": ["Shaders", "States", "Cheats", "Options"],
            "quickSaveLabel": "Save Slot",
            "quickLoadLabel": "Load Slot",
            "slotPrefix": "Slot",
            "emptySlotLabel": "Empty",
            "rewindNowLabel": "Rewind",
            "noRewindLabel": "Rewind unavailable",
            "masterCheatsLabel": "Master Cheats",
            "noCheatsLabel": "No cheats for this game",
            "shadersHeading": "Display & Shaders",
            "noDisplayRowsLabel": "Open Settings for per-console shader configuration",
            "speedLabel": "Speed",
            "speeds": [
                { "percent": 25, "label": "25%" },
                { "percent": 100, "label": "100%" },
                { "percent": 300, "label": "300%" },
                { "percent": 0, "label": "Max" }
            ],
            "swapPlayersLabel": "Swap P1 / P2",
            "restartLabel": "Restart",
            "quitGameLabel": "Close Game"
        }),
    );
    merge(
        &mut map,
        json!({
            "settingsFilterPlaceholder": "Filter settings...",
            "settingsHeading": "SETTINGS",
            "toolsHeading": "TOOLS",
            "modifiedOnlyLabel": "Modified only",
            "backLabel": "Back",
            "themesHeading": "Themes",
            "themesSubtitle": "Choose an EmuSen or ES-DE look. Changes apply immediately.",
            "useThemeLabel": "Use",
            "inUseLabel": "In use",
            "dualFaceBadge": "Dual Face",
            "bigPictureBadge": "Big Picture",
            "esdeBadge": "ES-DE",
            "navObjectNames": ["desktopNavLibrary", "desktopNavTable", "desktopNavSettings"],
            "allKey": "all",
            "sep": " · ",
            "percent": "%",
            "batteryPrefix": "BAT ",
            "checkThemeLabel": "Check",
            "toolsList": [
                { "key": "cores", "label": "Cores" },
                { "key": "shaders", "label": "Shaders" },
                { "key": "packages", "label": "Packages" },
                { "key": "scraper", "label": "Scraper" },
                { "key": "library", "label": "Library Manager" },
                { "key": "themes", "label": "Themes" }
            ],
            "aboutDetailHeading": "DETAILS",
            "statusHints": "Double-click: Play · F10: Big Picture · F11: Fullscreen",
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

/// Computes all Desktop Mode JSON projections from the real library, session, saved states, and cached inspector state.
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
    let now = words::now();
    let meta = if !view.catalog.metadata.is_empty() { view.catalog.metadata.as_slice() } else { extra_meta };
    let has_running = session.running && !session.game.is_empty();
    let theme_label = if theme_name.trim().is_empty() { "EmuSen".to_owned() } else { theme_name.trim().to_owned() };
    let mode_badge = theme_label.to_ascii_uppercase();
    let status_summary = format!("{} · {} Desktop", words::version_line(), theme_label);

    let all_games = games_of_at(ALL, &view.systems, &view.catalog, now);
    let recent_count = view.catalog.recent.len();
    let fav_count = all_games.iter().filter(|g| g.favourite).count();

    let console_q = state.console_filter.trim().to_lowercase();
    let source_consoles = if state.console_scope == 0 {
        let mut all = view.cored.clone();
        for r in view.rows.rows().iter().filter(|r| r.kind == Some(Kind::Console)) {
            if !all.iter().any(|c| c.key == r.key) {
                all.push(r.clone());
            }
        }
        all
    } else {
        view.rows.rows().iter().filter(|r| r.kind == Some(Kind::Console)).cloned().collect()
    };
    let consoles: Vec<Value> = source_consoles
        .iter()
        .filter(|r| console_q.is_empty() || r.name.to_lowercase().contains(&console_q) || r.full_name.to_lowercase().contains(&console_q))
        .map(|r| {
            let planet = planet_of(&r.engine_name, r.libretro, !r.engine.is_empty());
            json!({
                "key": r.key,
                "name": r.name,
                "fullName": r.full_name,
                "count": r.count,
                "played": r.played,
                "planet": planet,
                "engine": r.engine_name,
                "libretro": r.libretro,
                "screen": r.screen,
                "hue": r.hue,
            })
        })
        .collect();

    let mut places = Vec::new();
    if has_running {
        places.push(json!({ "key": "continue", "label": words::CONTINUE_ENTRY, "count": 1 }));
    }
    places.push(json!({ "key": ALL, "label": "All Games", "count": all_games.len() }));
    places.push(json!({ "key": crate::library::FAVOURITES, "label": words::FAVOURITES_FILTER, "count": fav_count }));
    places.push(json!({ "key": crate::library::RECENT, "label": "Recently Played", "count": recent_count }));

    let active_row = view.rows.rows().iter().find(|r| r.key == state.console_key);
    let active_heading = match state.console_key.as_str() {
        "continue" => words::CONTINUE_ENTRY.to_owned(),
        ALL => "All Games".to_owned(),
        crate::library::FAVOURITES => words::FAVOURITES_FILTER.to_owned(),
        crate::library::RECENT => "Recently Played".to_owned(),
        _ => active_row.map(|r| r.name.clone()).unwrap_or_else(|| "All Games".to_owned()),
    };
    let (active_core, active_planet, _) = if active_row.is_some_and(|r| r.kind == Some(Kind::Console)) {
        system_core_info(view, &state.console_key)
    } else {
        (String::new(), -2, false)
    };
    let active_screen = active_row.map(|r| r.screen.clone()).unwrap_or_default();
    let total_consoles = view.systems.len();
    let active_consoles = view.rows.rows().iter().filter(|r| r.kind == Some(Kind::Console)).count();

    let consoles_json = json!({
        "places": places,
        "consoles": consoles,
        "activeHeading": active_heading,
        "activeCore": active_core,
        "activePlanet": active_planet,
        "activeScreen": active_screen,
        "countsLine": format!("{active_consoles} / {total_consoles}"),
        "controllersLine": if controllers > 0 { words::counted(controllers as usize, "controller", "controllers") } else { "No controller".to_owned() },
    })
    .to_string();

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

    let rows_json: Vec<Value> = filtered
        .iter()
        .map(|e| {
            let cover = cover_url(&e.row.cover);
            let hero = cover_url(if e.row.hero.is_empty() { &e.row.cover } else { &e.row.hero });
            let swatch = cover_swatch(&cover, game_seed(&e.row.path));
            let badge = if has_running && session.game == e.row.path {
                "RESUME".to_owned()
            } else if e.states_count > 0 {
                words::counted(e.states_count, "STATE", "STATES")
            } else {
                String::new()
            };
            let status_label = if e.resume_ready {
                "Resume"
            } else if e.row.favourite {
                "Favourite"
            } else {
                "Ready"
            };
            let states_label = if e.states_count > 0 { words::counted(e.states_count, "state", "states") } else { "—".to_owned() };
            let subtitle = words::line(&[&e.year, &e.genre]);
            let subtitle = if subtitle.is_empty() { e.row.console.clone() } else { subtitle };
            json!({
                "path": e.row.path,
                "title": e.row.title,
                "system": e.row.system,
                "console": e.row.console,
                "core": e.core,
                "planet": e.planet,
                "libretro": e.libretro,
                "year": e.year,
                "genre": e.genre,
                "subtitle": subtitle,
                "playedWords": if e.played_words.is_empty() { "—".to_owned() } else { e.played_words.clone() },
                "playedSeconds": e.played_seconds,
                "lastPlayedWords": e.last_played_words,
                "favourite": e.row.favourite,
                "marked": marked_set.contains(e.row.path.as_str()),
                "resumeReady": e.resume_ready,
                "badge": badge,
                "coverUrl": cover,
                "heroUrl": hero,
                "swatch": swatch,
                "format": e.format,
                "statesCount": e.states_count,
                "statesLabel": states_label,
                "statusLabel": status_label,
            })
        })
        .collect();

    let effective_selected = if !state.selected_key.is_empty() && visible_paths.contains(&state.selected_key) {
        state.selected_key.clone()
    } else {
        visible_paths.first().cloned().unwrap_or_default()
    };

    let selected_item = filtered.iter().find(|e| e.row.path == effective_selected);
    let inspector_json = if let Some(item) = selected_item {
        let c_facts = cached.filter(|c| c.path == item.row.path);
        let cover = cover_url(&item.row.cover);
        let hero = cover_url(if item.row.hero.is_empty() { &item.row.cover } else { &item.row.hero });
        let swatch = cover_swatch(&hero, game_seed(&item.row.path));
        let players = c_facts.map(|c| c.facts.players.clone()).filter(|p| !p.is_empty()).unwrap_or_else(|| words::players(&meta_str(meta, &item.row.path, "players")));
        let description = c_facts.map(|c| c.facts.description.clone()).filter(|d| !d.is_empty()).unwrap_or_else(|| meta_str(meta, &item.row.path, "description"));
        let size_words = c_facts.and_then(|c| c.size_bytes).map(format_bytes).unwrap_or_else(|| "—".to_owned());
        let played_line = match (item.played_words.as_str(), item.last_played_words.as_str()) {
            ("", "") => "Never played".to_owned(),
            ("", last) => last.to_owned(),
            (p, "") => p.to_owned(),
            (p, last) => format!("{p} · {last}"),
        };
        let subtitle = words::line(&[&item.row.console, &item.year, &item.genre]);
        let core_kind = if item.core.is_empty() {
            "—".to_owned()
        } else if item.libretro {
            format!("{} (libretro)", item.core)
        } else {
            format!("{} (EmuSen native)", item.core)
        };
        let package_name = view.catalog.games.iter().find(|g| g.path == item.row.path).and_then(|g| g.package.as_deref()).unwrap_or_default();
        let cheats_count = c_facts.map_or(0, |c| c.cheats_count);
        let states_json: Vec<Value> = c_facts
            .map(|c| c.states.as_slice())
            .unwrap_or(&[])
            .iter()
            .map(|s| {
                let label = match s.slot {
                    crate::game_page::Slot::Auto => "Resume (Auto)".to_owned(),
                    crate::game_page::Slot::Number(n) => format!("Slot {n}"),
                };
                json!({
                    "path": s.path,
                    "label": label,
                    "whenWords": words::ago(s.when, now),
                    "pictureUrl": cover_url(&s.picture),
                    "isAuto": matches!(s.slot, crate::game_page::Slot::Auto),
                })
            })
            .collect();
        json!({
            "hasGame": true,
            "path": item.row.path,
            "title": item.row.title,
            "subtitle": subtitle,
            "console": item.row.console,
            "system": item.row.system,
            "core": item.core,
            "coreLine": core_kind,
            "planet": item.planet,
            "libretro": item.libretro,
            "playedWords": played_line,
            "players": if players.is_empty() { "—".to_owned() } else { players },
            "genre": if item.genre.is_empty() { "—".to_owned() } else { item.genre.clone() },
            "description": description,
            "fileName": path_file_name(&item.row.path),
            "folderPath": path_folder(&item.row.path),
            "sizeWords": size_words,
            "favourite": item.row.favourite,
            "running": has_running && session.game == item.row.path,
            "coverUrl": cover,
            "heroUrl": hero,
            "swatch": swatch,
            "statesCount": item.states_count,
            "statesHeading": if item.states_count > 0 { format!("SAVE STATES · {}", item.states_count) } else { "SAVE STATES".to_owned() },
            "states": states_json,
            "packageWords": if package_name.is_empty() { "Read in place".to_owned() } else { package_name.to_owned() },
            "cheatsWords": if cheats_count > 0 { words::counted(cheats_count, "code", "codes") } else { "None".to_owned() },
        })
    } else {
        json!({ "hasGame": false })
    }
    .to_string();

    let narrowed = state.filter_key != ALL || !state.search_query.trim().is_empty();
    let games_summary = if narrowed {
        format!("Showing {} of {} games", words::grouped(filtered.len()), words::grouped(f_all))
    } else {
        words::games(filtered.len())
    };
    let games_json = json!({
        "count": filtered.len(),
        "totalCount": f_all,
        "summaryLine": games_summary,
        "filterCounts": { "all": f_all, "favourites": f_fav, "resume": f_res, "unplayed": f_unp },
        "rows": rows_json,
        "letters": letters,
        "allMarked": all_marked,
    })
    .to_string();

    let pause_facts = if session.card_facts.is_empty() { None } else { Some(PauseFacts::from_json(&session.card_facts)) };
    let playing_json = build_playing_json(view, session, pause_facts.as_ref());

    let status_text = if !state.marked_keys.is_empty() && state.view == 1 {
        format!("{} of {} selected ({} total in library)", words::grouped(state.marked_keys.len()), words::grouped(filtered.len()), words::grouped(all_games.len()))
    } else if let Some(item) = selected_item {
        format!("{active_heading} · {} · Selected: {}", words::games(filtered.len()), item.row.title)
    } else {
        format!("{active_heading} · {}", words::games(filtered.len()))
    };

    DesktopProjection {
        consoles_json,
        games_json,
        inspector_json,
        playing_json,
        status_text,
        mode_badge,
        status_summary,
        effective_selected,
        visible_paths,
        all_marked,
    }
}

fn build_playing_json(view: &LibraryView, session: &SessionSnapshot, facts: Option<&PauseFacts>) -> String {
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
    let slots: Vec<Value> = facts
        .map(|f| {
            f.slots
                .iter()
                .map(|s| json!({ "slot": s.slot, "exists": s.exists, "when": s.when, "picture": s.picture }))
                .collect()
        })
        .unwrap_or_else(|| (1..=8).map(|slot| json!({ "slot": slot, "exists": false, "when": "", "picture": "" })).collect());
    let cheats: Vec<Value> = facts
        .map(|f| {
            f.cheats
                .iter()
                .enumerate()
                .map(|(i, c)| json!({ "index": i, "description": c.description, "on": c.on }))
                .collect()
        })
        .unwrap_or_default();
    let console_options: Vec<Value> = facts
        .map(|f| {
            f.cards
                .schema
                .iter()
                .map(|r| {
                    json!({
                        "key": r.key,
                        "label": r.label,
                        "value": f.cards.value(&r.key),
                        "choices": r.choices.clone().unwrap_or_default(),
                        "choiceLabels": r.choice_labels.clone().unwrap_or_default(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let latest_rewind = facts.and_then(|f| f.held.last().copied()).unwrap_or(0);
    let has_rewind = facts.is_some_and(|f| f.rewind_why.is_none() && !f.held.is_empty());

    json!({
        "running": session.running,
        "paused": session.paused,
        "game": session.game,
        "system": system,
        "title": title,
        "console": console,
        "core": core,
        "planet": planet,
        "fpsWords": session.fps_words,
        "status": session.status,
        "slot": session.slot,
        "speed": session.speed,
        "masterCheats": session.master_cheats,
        "slots": slots,
        "cheats": cheats,
        "consoleOptions": console_options,
        "hasRewind": has_rewind,
        "latestRewindFrame": latest_rewind,
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
        let scope = scope.clamp(0, 1);
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

    pub fn handle_settings_door(mut self: Pin<&mut Self>, name: &QString) {
        let key = name.to_string();
        match key.as_str() {
            "themes" => {
                self.as_mut().set_settings_tool(5);
                self.refresh();
            }
            "shaders" => {
                self.as_mut().set_settings_tool(-1);
                if let Some(settings) = unsafe { self.settings.as_mut() } {
                    unsafe { Pin::new_unchecked(settings) }.open_shaders_door();
                }
                self.refresh();
            }
            "bindings" | "keyboard" => {
                self.as_mut().set_settings_tool(-1);
                if let Some(settings) = unsafe { self.settings.as_mut() } {
                    let mut s = unsafe { Pin::new_unchecked(settings) };
                    if let Some(row) = s.as_mut().focus_section_row(0, &key) {
                        s.as_mut().choose(row as i32);
                    }
                }
                self.refresh();
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
            }
            "cores" => self.as_mut().select_settings_section(3),
            "library" | "packages" => self.as_mut().select_settings_section(4),
            "scraper" | "compressed" | "cheats" => self.as_mut().door_requested(name.clone()),
            _ => {}
        }
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
