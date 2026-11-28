//! `Workspace`, the read-only Desktop Mode workspace, catalog, inspector, and batch-selection projection.

use crate::rows::qobject::ThemeRows;
use crate::view::Row;
use core::pin::Pin;
use cxx_qt::casting::Upcast;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QString, QStringList};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("emusen-theme-qt/src/rows.cxxqt.h");
        type ThemeRows = crate::rows::qobject::ThemeRows;
    }

    #[qenum(ThemeWorkspace)]
    enum ViewMode {
        Library,
        Table,
        Settings,
    }

    #[qenum(ThemeWorkspace)]
    enum ConsoleScope {
        Active,
        All,
        Favourites,
    }

    #[qenum(ThemeWorkspace)]
    enum DrawerTab {
        Shaders,
        States,
        Cheats,
        Controls,
    }

    #[qenum(ThemeWorkspace)]
    enum SortColumn {
        Title,
        Console,
        Core,
        Year,
        Genre,
        Played,
        SaveStates,
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element = "Workspace"]
        #[qml_singleton]
        #[qproperty(i32, view, READ, NOTIFY = changed)]
        #[qproperty(bool, sidebar_open, cxx_name = "sidebarOpen", READ, NOTIFY = changed)]
        #[qproperty(bool, inspector_open, cxx_name = "inspectorOpen", READ, NOTIFY = changed)]
        #[qproperty(bool, drawer_open, cxx_name = "drawerOpen", READ, NOTIFY = changed)]
        #[qproperty(i32, drawer_tab, cxx_name = "drawerTab", READ, NOTIFY = changed)]
        #[qproperty(QString, theme_name, cxx_name = "themeName", READ, NOTIFY = changed)]
        #[qproperty(bool, full_screen, cxx_name = "fullScreen", READ, NOTIFY = changed)]
        #[qproperty(QString, status_text, cxx_name = "statusText", READ, NOTIFY = changed)]
        #[qproperty(QString, status_summary, cxx_name = "statusSummary", READ, NOTIFY = changed)]
        #[qproperty(QString, mode_badge, cxx_name = "modeBadge", READ, NOTIFY = changed)]
        #[qproperty(QString, big_picture_hint, cxx_name = "bigPictureHint", READ, NOTIFY = changed)]
        #[qproperty(QString, status_hints, cxx_name = "statusHints", READ, NOTIFY = changed)]
        #[qproperty(QString, counts_line, cxx_name = "countsLine", READ, NOTIFY = changed)]
        #[qproperty(QString, controllers_line, cxx_name = "controllersLine", READ, NOTIFY = changed)]
        #[qproperty(QString, console_key, cxx_name = "consoleKey", READ, NOTIFY = changed)]
        #[qproperty(QString, filter_key, cxx_name = "filterKey", READ, NOTIFY = changed)]
        #[qproperty(QString, search_query, cxx_name = "searchQuery", READ, NOTIFY = changed)]
        #[qproperty(QString, console_filter, cxx_name = "consoleFilter", READ, NOTIFY = changed)]
        #[qproperty(i32, console_scope, cxx_name = "consoleScope", READ, NOTIFY = changed)]
        #[qproperty(i32, sort_index, cxx_name = "sortIndex", READ, NOTIFY = changed)]
        #[qproperty(bool, sort_ascending, cxx_name = "sortAscending", READ, NOTIFY = changed)]
        #[qproperty(QString, selected_key, cxx_name = "selectedKey", READ, NOTIFY = changed)]
        #[qproperty(QStringList, marked_keys, cxx_name = "markedKeys", READ, NOTIFY = changed)]
        #[qproperty(i32, marked_count, cxx_name = "markedCount", READ, NOTIFY = changed)]
        #[qproperty(bool, all_marked, cxx_name = "allMarked", READ, NOTIFY = changed)]
        #[qproperty(i32, count, READ, NOTIFY = changed)]
        #[qproperty(i32, total_count, cxx_name = "totalCount", READ, NOTIFY = changed)]
        #[qproperty(QString, summary_line, cxx_name = "summaryLine", READ, NOTIFY = changed)]
        #[qproperty(QString, empty_words, cxx_name = "emptyWords", READ, NOTIFY = changed)]
        #[qproperty(QString, active_heading, cxx_name = "activeHeading", READ, NOTIFY = changed)]
        #[qproperty(QString, active_core, cxx_name = "activeCore", READ, NOTIFY = changed)]
        #[qproperty(i32, active_planet, cxx_name = "activePlanet", READ, NOTIFY = changed)]
        #[qproperty(QString, active_screen, cxx_name = "activeScreen", READ, NOTIFY = changed)]
        #[qproperty(bool, has_selection, cxx_name = "hasSelection", READ, NOTIFY = changed)]
        #[qproperty(QString, inspector_key, cxx_name = "inspectorKey", READ, NOTIFY = changed)]
        #[qproperty(QString, inspector_title, cxx_name = "inspectorTitle", READ, NOTIFY = changed)]
        #[qproperty(QString, inspector_subtitle, cxx_name = "inspectorSubtitle", READ, NOTIFY = changed)]
        #[qproperty(QString, inspector_console, cxx_name = "inspectorConsole", READ, NOTIFY = changed)]
        #[qproperty(QString, inspector_system, cxx_name = "inspectorSystem", READ, NOTIFY = changed)]
        #[qproperty(QString, inspector_core, cxx_name = "inspectorCore", READ, NOTIFY = changed)]
        #[qproperty(QString, inspector_core_line, cxx_name = "inspectorCoreLine", READ, NOTIFY = changed)]
        #[qproperty(i32, inspector_planet, cxx_name = "inspectorPlanet", READ, NOTIFY = changed)]
        #[qproperty(bool, inspector_libretro, cxx_name = "inspectorLibretro", READ, NOTIFY = changed)]
        #[qproperty(QString, inspector_played_words, cxx_name = "inspectorPlayedWords", READ, NOTIFY = changed)]
        #[qproperty(QString, inspector_players, cxx_name = "inspectorPlayers", READ, NOTIFY = changed)]
        #[qproperty(QString, inspector_genre, cxx_name = "inspectorGenre", READ, NOTIFY = changed)]
        #[qproperty(QString, inspector_description, cxx_name = "inspectorDescription", READ, NOTIFY = changed)]
        #[qproperty(QString, inspector_file_name, cxx_name = "inspectorFileName", READ, NOTIFY = changed)]
        #[qproperty(QString, inspector_folder_path, cxx_name = "inspectorFolderPath", READ, NOTIFY = changed)]
        #[qproperty(QString, inspector_size_words, cxx_name = "inspectorSizeWords", READ, NOTIFY = changed)]
        #[qproperty(bool, inspector_favourite, cxx_name = "inspectorFavourite", READ, NOTIFY = changed)]
        #[qproperty(bool, inspector_running, cxx_name = "inspectorRunning", READ, NOTIFY = changed)]
        #[qproperty(QString, inspector_cover, cxx_name = "inspectorCover", READ, NOTIFY = changed)]
        #[qproperty(QString, inspector_hero, cxx_name = "inspectorHero", READ, NOTIFY = changed)]
        #[qproperty(i32, inspector_states_count, cxx_name = "inspectorStatesCount", READ, NOTIFY = changed)]
        #[qproperty(QString, inspector_states_heading, cxx_name = "inspectorStatesHeading", READ, NOTIFY = changed)]
        #[qproperty(QString, inspector_package_words, cxx_name = "inspectorPackageWords", READ, NOTIFY = changed)]
        #[qproperty(QString, inspector_cheats_words, cxx_name = "inspectorCheatsWords", READ, NOTIFY = changed)]
        #[qproperty(*mut ThemeRows, places, READ, CONSTANT)]
        #[qproperty(*mut ThemeRows, consoles, READ, CONSTANT)]
        #[qproperty(*mut ThemeRows, filters, READ, CONSTANT)]
        #[qproperty(*mut ThemeRows, sorts, READ, CONSTANT)]
        #[qproperty(*mut ThemeRows, letters, READ, CONSTANT)]
        #[qproperty(*mut ThemeRows, games, READ, CONSTANT)]
        #[qproperty(*mut ThemeRows, inspector_states, cxx_name = "inspectorStates", READ, CONSTANT)]
        #[qproperty(QString, themes_door_key, cxx_name = "themesDoorKey", READ, CONSTANT)]
        type ThemeWorkspace = super::ThemeWorkspaceRust;
    }

    unsafe extern "RustQt" {
        #[qsignal]
        fn changed(self: Pin<&mut ThemeWorkspace>);
    }

    impl cxx_qt::Initialize for ThemeWorkspace {}
}

pub const WORKSPACE_PLACE_ROLES: [&str; 3] = ["key", "label", "count"];
pub const WORKSPACE_CONSOLE_ROLES: [&str; 11] = [
    "key",
    "name",
    "maker",
    "year",
    "games",
    "engine",
    "engineKind",
    "planet",
    "libretro",
    "hasGames",
    "screen",
];
pub const WORKSPACE_FILTER_ROLES: [&str; 3] = ["key", "label", "count"];
pub const WORKSPACE_SORT_ROLES: [&str; 3] = ["index", "label", "key"];
pub const WORKSPACE_LETTER_ROLES: [&str; 1] = ["letter"];
pub const WORKSPACE_GAME_ROLES: [&str; 23] = [
    "key",
    "title",
    "system",
    "console",
    "core",
    "planet",
    "libretro",
    "year",
    "genre",
    "subtitle",
    "playedWords",
    "playedSeconds",
    "lastPlayedWords",
    "favourite",
    "marked",
    "resumeReady",
    "badge",
    "cover",
    "hero",
    "format",
    "statesCount",
    "statesLabel",
    "statusLabel",
];
pub const WORKSPACE_STATE_ROLES: [&str; 5] = ["path", "label", "whenWords", "picture", "isAuto"];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorkspaceFacts {
    pub view: i32,
    pub sidebar_open: bool,
    pub inspector_open: bool,
    pub drawer_open: bool,
    pub drawer_tab: i32,
    pub theme_name: String,
    pub full_screen: bool,
    pub status_text: String,
    pub status_summary: String,
    pub mode_badge: String,
    pub big_picture_hint: String,
    pub status_hints: String,
    pub counts_line: String,
    pub controllers_line: String,
    pub console_key: String,
    pub filter_key: String,
    pub search_query: String,
    pub console_filter: String,
    pub console_scope: i32,
    pub sort_index: i32,
    pub sort_ascending: bool,
    pub selected_key: String,
    pub marked_keys: Vec<String>,
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
    pub inspector_key: String,
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
    pub inspector_states_count: i32,
    pub inspector_states_heading: String,
    pub inspector_package_words: String,
    pub inspector_cheats_words: String,
    pub places: Vec<Row>,
    pub consoles: Vec<Row>,
    pub filters: Vec<Row>,
    pub sorts: Vec<Row>,
    pub letters: Vec<Row>,
    pub games: Vec<Row>,
    pub inspector_states: Vec<Row>,
}

pub struct ThemeWorkspaceRust {
    view: i32,
    sidebar_open: bool,
    inspector_open: bool,
    drawer_open: bool,
    drawer_tab: i32,
    theme_name: QString,
    full_screen: bool,
    status_text: QString,
    status_summary: QString,
    mode_badge: QString,
    big_picture_hint: QString,
    status_hints: QString,
    counts_line: QString,
    controllers_line: QString,
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
    count: i32,
    total_count: i32,
    summary_line: QString,
    empty_words: QString,
    active_heading: QString,
    active_core: QString,
    active_planet: i32,
    active_screen: QString,
    has_selection: bool,
    inspector_key: QString,
    inspector_title: QString,
    inspector_subtitle: QString,
    inspector_console: QString,
    inspector_system: QString,
    inspector_core: QString,
    inspector_core_line: QString,
    inspector_planet: i32,
    inspector_libretro: bool,
    inspector_played_words: QString,
    inspector_players: QString,
    inspector_genre: QString,
    inspector_description: QString,
    inspector_file_name: QString,
    inspector_folder_path: QString,
    inspector_size_words: QString,
    inspector_favourite: bool,
    inspector_running: bool,
    inspector_cover: QString,
    inspector_hero: QString,
    inspector_states_count: i32,
    inspector_states_heading: QString,
    inspector_package_words: QString,
    inspector_cheats_words: QString,
    places: *mut ThemeRows,
    consoles: *mut ThemeRows,
    filters: *mut ThemeRows,
    sorts: *mut ThemeRows,
    letters: *mut ThemeRows,
    games: *mut ThemeRows,
    inspector_states: *mut ThemeRows,
    themes_door_key: QString,
}

impl Default for ThemeWorkspaceRust {
    fn default() -> Self {
        Self {
            view: 0,
            sidebar_open: true,
            inspector_open: true,
            drawer_open: false,
            drawer_tab: 0,
            theme_name: QString::default(),
            full_screen: false,
            status_text: QString::default(),
            status_summary: QString::default(),
            mode_badge: QString::default(),
            big_picture_hint: QString::default(),
            status_hints: QString::default(),
            counts_line: QString::default(),
            controllers_line: QString::default(),
            console_key: QString::default(),
            filter_key: QString::default(),
            search_query: QString::default(),
            console_filter: QString::default(),
            console_scope: 0,
            sort_index: 0,
            sort_ascending: true,
            selected_key: QString::default(),
            marked_keys: QStringList::default(),
            marked_count: 0,
            all_marked: false,
            count: 0,
            total_count: 0,
            summary_line: QString::default(),
            empty_words: QString::default(),
            active_heading: QString::default(),
            active_core: QString::default(),
            active_planet: -1,
            active_screen: QString::default(),
            has_selection: false,
            inspector_key: QString::default(),
            inspector_title: QString::default(),
            inspector_subtitle: QString::default(),
            inspector_console: QString::default(),
            inspector_system: QString::default(),
            inspector_core: QString::default(),
            inspector_core_line: QString::default(),
            inspector_planet: -1,
            inspector_libretro: false,
            inspector_played_words: QString::default(),
            inspector_players: QString::default(),
            inspector_genre: QString::default(),
            inspector_description: QString::default(),
            inspector_file_name: QString::default(),
            inspector_folder_path: QString::default(),
            inspector_size_words: QString::default(),
            inspector_favourite: false,
            inspector_running: false,
            inspector_cover: QString::default(),
            inspector_hero: QString::default(),
            inspector_states_count: 0,
            inspector_states_heading: QString::default(),
            inspector_package_words: QString::default(),
            inspector_cheats_words: QString::default(),
            places: std::ptr::null_mut(),
            consoles: std::ptr::null_mut(),
            filters: std::ptr::null_mut(),
            sorts: std::ptr::null_mut(),
            letters: std::ptr::null_mut(),
            games: std::ptr::null_mut(),
            inspector_states: std::ptr::null_mut(),
            themes_door_key: QString::from("themes"),
        }
    }
}

impl cxx_qt::Initialize for qobject::ThemeWorkspace {
    fn initialize(mut self: Pin<&mut Self>) {
        let mut make = |roles: &'static [&'static str]| {
            let list = crate::glue::ffi::new_rows(self.as_mut().upcast_pin());
            if let Some(rows) = unsafe { list.as_mut() } {
                unsafe { Pin::new_unchecked(rows) }.set_roles(roles);
            }
            list
        };
        let (places, consoles, filters, sorts, letters, games, inspector_states) = (
            make(&WORKSPACE_PLACE_ROLES),
            make(&WORKSPACE_CONSOLE_ROLES),
            make(&WORKSPACE_FILTER_ROLES),
            make(&WORKSPACE_SORT_ROLES),
            make(&WORKSPACE_LETTER_ROLES),
            make(&WORKSPACE_GAME_ROLES),
            make(&WORKSPACE_STATE_ROLES),
        );
        let mut r = self.as_mut().rust_mut();
        r.places = places;
        r.consoles = consoles;
        r.filters = filters;
        r.sorts = sorts;
        r.letters = letters;
        r.games = games;
        r.inspector_states = inspector_states;
    }
}

impl qobject::ThemeWorkspace {
    pub fn show(mut self: Pin<&mut Self>, facts: &WorkspaceFacts) {
        let mut changed = false;

        set_plain!(self, changed, view, facts.view);
        set_plain!(self, changed, sidebar_open, facts.sidebar_open);
        set_plain!(self, changed, inspector_open, facts.inspector_open);
        set_plain!(self, changed, drawer_open, facts.drawer_open);
        set_plain!(self, changed, drawer_tab, facts.drawer_tab);
        set_text!(self, changed, theme_name, facts.theme_name);
        set_plain!(self, changed, full_screen, facts.full_screen);
        set_text!(self, changed, status_text, facts.status_text);
        set_text!(self, changed, status_summary, facts.status_summary);
        set_text!(self, changed, mode_badge, facts.mode_badge);
        set_text!(self, changed, big_picture_hint, facts.big_picture_hint);
        set_text!(self, changed, status_hints, facts.status_hints);
        set_text!(self, changed, counts_line, facts.counts_line);
        set_text!(self, changed, controllers_line, facts.controllers_line);
        set_text!(self, changed, console_key, facts.console_key);
        set_text!(self, changed, filter_key, facts.filter_key);
        set_text!(self, changed, search_query, facts.search_query);
        set_text!(self, changed, console_filter, facts.console_filter);
        set_plain!(self, changed, console_scope, facts.console_scope);
        set_plain!(self, changed, sort_index, facts.sort_index);
        set_plain!(self, changed, sort_ascending, facts.sort_ascending);
        set_text!(self, changed, selected_key, facts.selected_key);
        let marked = crate::to_qstring_list(&facts.marked_keys);
        if self.marked_keys != marked {
            self.as_mut().rust_mut().marked_keys = marked;
            changed = true;
        }
        set_plain!(self, changed, marked_count, facts.marked_count);
        set_plain!(self, changed, all_marked, facts.all_marked);
        set_plain!(self, changed, count, facts.count);
        set_plain!(self, changed, total_count, facts.total_count);
        set_text!(self, changed, summary_line, facts.summary_line);
        set_text!(self, changed, empty_words, facts.empty_words);
        set_text!(self, changed, active_heading, facts.active_heading);
        set_text!(self, changed, active_core, facts.active_core);
        set_plain!(self, changed, active_planet, facts.active_planet);
        set_text!(self, changed, active_screen, facts.active_screen);
        set_plain!(self, changed, has_selection, facts.has_selection);
        set_text!(self, changed, inspector_key, facts.inspector_key);
        set_text!(self, changed, inspector_title, facts.inspector_title);
        set_text!(self, changed, inspector_subtitle, facts.inspector_subtitle);
        set_text!(self, changed, inspector_console, facts.inspector_console);
        set_text!(self, changed, inspector_system, facts.inspector_system);
        set_text!(self, changed, inspector_core, facts.inspector_core);
        set_text!(self, changed, inspector_core_line, facts.inspector_core_line);
        set_plain!(self, changed, inspector_planet, facts.inspector_planet);
        set_plain!(self, changed, inspector_libretro, facts.inspector_libretro);
        set_text!(self, changed, inspector_played_words, facts.inspector_played_words);
        set_text!(self, changed, inspector_players, facts.inspector_players);
        set_text!(self, changed, inspector_genre, facts.inspector_genre);
        set_text!(self, changed, inspector_description, facts.inspector_description);
        set_text!(self, changed, inspector_file_name, facts.inspector_file_name);
        set_text!(self, changed, inspector_folder_path, facts.inspector_folder_path);
        set_text!(self, changed, inspector_size_words, facts.inspector_size_words);
        set_plain!(self, changed, inspector_favourite, facts.inspector_favourite);
        set_plain!(self, changed, inspector_running, facts.inspector_running);
        set_text!(self, changed, inspector_cover, facts.inspector_cover);
        set_text!(self, changed, inspector_hero, facts.inspector_hero);
        set_plain!(self, changed, inspector_states_count, facts.inspector_states_count);
        set_text!(self, changed, inspector_states_heading, facts.inspector_states_heading);
        set_text!(self, changed, inspector_package_words, facts.inspector_package_words);
        set_text!(self, changed, inspector_cheats_words, facts.inspector_cheats_words);

        let lists = [
            (self.places, &facts.places),
            (self.consoles, &facts.consoles),
            (self.filters, &facts.filters),
            (self.sorts, &facts.sorts),
            (self.letters, &facts.letters),
            (self.games, &facts.games),
            (self.inspector_states, &facts.inspector_states),
        ];
        for (ptr, rows) in lists {
            if let Some(list) = unsafe { ptr.as_mut() } {
                if list.rows() != rows.as_slice() {
                    changed = true;
                }
                unsafe { Pin::new_unchecked(list) }.replace(rows.clone());
            }
        }
        if changed {
            self.changed();
        }
    }
}

pub fn to_workspace_scope(desktop_scope: i32) -> i32 {
    match desktop_scope {
        0 => qobject::ConsoleScope::All.repr,
        1 => qobject::ConsoleScope::Active.repr,
        2 => qobject::ConsoleScope::Favourites.repr,
        _ => qobject::ConsoleScope::All.repr,
    }
}

pub fn from_workspace_scope(workspace_scope: i32) -> i32 {
    if workspace_scope == qobject::ConsoleScope::Active.repr {
        1
    } else if workspace_scope == qobject::ConsoleScope::Favourites.repr {
        2
    } else {
        0
    }
}

fn folder_leaf(path: &str) -> String {
    std::path::Path::new(path).file_name().and_then(|s| s.to_str()).unwrap_or_default().to_owned()
}

pub fn facts_of(
    data: &emusen_qt_models::desktop::DesktopWorkspaceData,
    state: &emusen_qt_models::desktop::DesktopState,
    full_screen: bool,
    url_map: &dyn Fn(String) -> String,
) -> WorkspaceFacts {
    use crate::view::Cell;
    let map_url = |u: &String| if u.is_empty() { String::new() } else { url_map(u.clone()) };
    WorkspaceFacts {
        view: state.view,
        sidebar_open: state.sidebar_open,
        inspector_open: state.inspector_open,
        drawer_open: state.drawer_open,
        drawer_tab: state.drawer_tab,
        theme_name: data.theme_name.clone(),
        full_screen,
        status_text: data.status_text.clone(),
        status_summary: data.status_summary.clone(),
        mode_badge: data.mode_badge.clone(),
        big_picture_hint: data.big_picture_hint.clone(),
        status_hints: data.status_hints.clone(),
        counts_line: data.counts_line.clone(),
        controllers_line: data.controllers_line.clone(),
        console_key: state.console_key.clone(),
        filter_key: state.filter_key.clone(),
        search_query: state.search_query.clone(),
        console_filter: state.console_filter.clone(),
        console_scope: to_workspace_scope(state.console_scope),
        sort_index: state.sort_index,
        sort_ascending: state.sort_ascending,
        selected_key: state.selected_key.clone(),
        marked_keys: state.marked_keys.clone(),
        marked_count: data.marked_count,
        all_marked: data.all_marked,
        count: data.count,
        total_count: data.total_count,
        summary_line: data.summary_line.clone(),
        empty_words: data.empty_words.clone(),
        active_heading: data.active_heading.clone(),
        active_core: data.active_core.clone(),
        active_planet: data.active_planet,
        active_screen: data.active_screen.clone(),
        has_selection: data.has_selection,
        inspector_key: data.effective_selected.clone(),
        inspector_title: data.inspector_title.clone(),
        inspector_subtitle: data.inspector_subtitle.clone(),
        inspector_console: data.inspector_console.clone(),
        inspector_system: data.inspector_system.clone(),
        inspector_core: data.inspector_core.clone(),
        inspector_core_line: data.inspector_core_line.clone(),
        inspector_planet: data.inspector_planet,
        inspector_libretro: data.inspector_libretro,
        inspector_played_words: data.inspector_played_words.clone(),
        inspector_players: data.inspector_players.clone(),
        inspector_genre: data.inspector_genre.clone(),
        inspector_description: data.inspector_description.clone(),
        inspector_file_name: data.inspector_file_name.clone(),
        inspector_folder_path: folder_leaf(&data.inspector_folder_path),
        inspector_size_words: data.inspector_size_words.clone(),
        inspector_favourite: data.inspector_favourite,
        inspector_running: data.inspector_running,
        inspector_cover: map_url(&data.inspector_cover),
        inspector_hero: map_url(&data.inspector_hero),
        inspector_states_count: data.inspector_states_count,
        inspector_states_heading: data.inspector_states_heading.clone(),
        inspector_package_words: data.inspector_package_words.clone(),
        inspector_cheats_words: data.inspector_cheats_words.clone(),
        places: data.places.iter().map(|p| vec![Cell::Text(p.key.clone()), Cell::Text(p.label.clone()), Cell::Whole(p.count)]).collect(),
        consoles: data
            .consoles
            .iter()
            .map(|c| {
                vec![
                    Cell::Text(c.key.clone()),
                    Cell::Text(c.name.clone()),
                    Cell::Text(c.maker.clone()),
                    Cell::Whole(c.year),
                    Cell::Whole(c.games),
                    Cell::Text(c.engine.clone()),
                    Cell::Text(c.engine_kind.clone()),
                    Cell::Whole(c.planet),
                    Cell::Flag(c.libretro),
                    Cell::Flag(c.has_games),
                    Cell::Text(c.screen.clone()),
                ]
            })
            .collect(),
        filters: data.filters.iter().map(|f| vec![Cell::Text(f.key.clone()), Cell::Text(f.label.clone()), Cell::Whole(f.count)]).collect(),
        sorts: data.sorts.iter().map(|s| vec![Cell::Whole(s.index), Cell::Text(s.label.clone()), Cell::Text(s.key.clone())]).collect(),
        letters: data.letters.iter().map(|l| vec![Cell::Text(l.clone())]).collect(),
        games: data
            .games
            .iter()
            .map(|g| {
                vec![
                    Cell::Text(g.key.clone()),
                    Cell::Text(g.title.clone()),
                    Cell::Text(g.system.clone()),
                    Cell::Text(g.console.clone()),
                    Cell::Text(g.core.clone()),
                    Cell::Whole(g.planet),
                    Cell::Flag(g.libretro),
                    Cell::Text(g.year.clone()),
                    Cell::Text(g.genre.clone()),
                    Cell::Text(g.subtitle.clone()),
                    Cell::Text(g.played_words.clone()),
                    Cell::Number(g.played_seconds as f64),
                    Cell::Text(g.last_played_words.clone()),
                    Cell::Flag(g.favourite),
                    Cell::Flag(g.marked),
                    Cell::Flag(g.resume_ready),
                    Cell::Text(g.badge.clone()),
                    Cell::Text(map_url(&g.cover)),
                    Cell::Text(map_url(&g.hero)),
                    Cell::Text(g.format.clone()),
                    Cell::Whole(g.states_count),
                    Cell::Text(g.states_label.clone()),
                    Cell::Text(g.status_label.clone()),
                ]
            })
            .collect(),
        inspector_states: data
            .inspector_states
            .iter()
            .enumerate()
            .map(|(i, s)| {
                vec![
                    Cell::Text(format!("state:{i}")),
                    Cell::Text(s.label.clone()),
                    Cell::Text(s.when_words.clone()),
                    Cell::Text(map_url(&s.picture)),
                    Cell::Flag(s.is_auto),
                ]
            })
            .collect(),
    }
}
