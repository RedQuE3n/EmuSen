//! `Words`, every word a theme may show that no row carries (EmuSen_Qt.md §3.2; §1.3 bans words in QML): the labels of
//! the three screens and the counted and timed phrases, each from emusen-qt-models' words.rs, none written here.

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use emusen_qt_models::words;

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
        #[qml_element = "Words"]
        #[qml_singleton]
        #[qproperty(QString, title, READ, CONSTANT)]
        #[qproperty(QString, games_heading, cxx_name = "gamesHeading", READ, CONSTANT)]
        #[qproperty(QString, choose_a_console, cxx_name = "chooseAConsole", READ, CONSTANT)]
        #[qproperty(QString, games_path, cxx_name = "gamesPath", READ, CONSTANT)]
        #[qproperty(QString, continue_playing, cxx_name = "continuePlaying", READ, CONSTANT)]
        #[qproperty(QString, recent, READ, CONSTANT)]
        #[qproperty(QString, between, READ, CONSTANT)]
        #[qproperty(QString, search, READ, CONSTANT)]
        #[qproperty(QString, emusen_tag, cxx_name = "emusenTag", READ, CONSTANT)]
        #[qproperty(QString, libretro_tag, cxx_name = "libretroTag", READ, CONSTANT)]
        #[qproperty(QString, no_games, cxx_name = "noGames", READ, CONSTANT)]
        #[qproperty(QString, no_recent, cxx_name = "noRecent", READ, CONSTANT)]
        #[qproperty(QString, no_consoles, cxx_name = "noConsoles", READ, CONSTANT)]
        #[qproperty(QString, add_folder, cxx_name = "addFolder", READ, CONSTANT)]
        #[qproperty(QString, places, READ, CONSTANT)]
        #[qproperty(QString, continue_entry, cxx_name = "continueEntry", READ, CONSTANT)]
        #[qproperty(QString, recent_entry, cxx_name = "recentEntry", READ, CONSTANT)]
        #[qproperty(QStringList, fact_labels, cxx_name = "factLabels", READ, CONSTANT)]
        #[qproperty(QStringList, console_shows, cxx_name = "consoleShows", READ, CONSTANT)]
        #[qproperty(QStringList, game_fact_labels, cxx_name = "gameFactLabels", READ, CONSTANT)]
        #[qproperty(QString, crumb, READ, CONSTANT)]
        #[qproperty(QString, no_games_line, cxx_name = "noGamesLine", READ, CONSTANT)]
        #[qproperty(QStringList, console_sorts, cxx_name = "consoleSorts", READ, CONSTANT)]
        /// The time the timed phrases are said against, in seconds since 1970; 0, the default, is the wall clock. The
        /// program sets it from its clock, which a test holds (decided 2026-11-09 by the tester); a theme only reads it.
        #[qproperty(f64, now, READ, NOTIFY)]
        #[qproperty(QString, name, READ, CONSTANT)]
        /// From 1.11: Left with Right named as one key, "◀ ▶", as the Settings boards' help line draws it.
        #[qproperty(QString, key_pair, cxx_name = "keyPair", READ, CONSTANT)]
        /// From 1.23: Desktop Mode labels owned by emusen-qt-models::words.
        #[qproperty(QStringList, desktop_nav_tabs, cxx_name = "desktopNavTabs", READ, CONSTANT)]
        #[qproperty(QStringList, desktop_view_modes, cxx_name = "desktopViewModes", READ, CONSTANT)]
        #[qproperty(QStringList, desktop_sort_labels, cxx_name = "desktopSortLabels", READ, CONSTANT)]
        #[qproperty(QStringList, desktop_table_columns, cxx_name = "desktopTableColumns", READ, CONSTANT)]
        #[qproperty(QStringList, desktop_drawer_tabs, cxx_name = "desktopDrawerTabs", READ, CONSTANT)]
        #[qproperty(QString, desktop_search_placeholder, cxx_name = "desktopSearchPlaceholder", READ, CONSTANT)]
        #[qproperty(QString, desktop_big_picture_label, cxx_name = "desktopBigPictureLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_big_picture_hint, cxx_name = "desktopBigPictureHint", READ, CONSTANT)]
        #[qproperty(QString, desktop_resume_label, cxx_name = "desktopResumeLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_sidebar_tip, cxx_name = "desktopSidebarTip", READ, CONSTANT)]
        #[qproperty(QString, desktop_inspector_tip, cxx_name = "desktopInspectorTip", READ, CONSTANT)]
        #[qproperty(QString, desktop_constellation_heading, cxx_name = "desktopConstellationHeading", READ, CONSTANT)]
        #[qproperty(QString, desktop_all_games_label, cxx_name = "desktopAllGamesLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_recent_label, cxx_name = "desktopRecentLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_consoles_heading, cxx_name = "desktopConsolesHeading", READ, CONSTANT)]
        #[qproperty(QString, desktop_scope_active, cxx_name = "desktopScopeActive", READ, CONSTANT)]
        #[qproperty(QString, desktop_console_filter_placeholder, cxx_name = "desktopConsoleFilterPlaceholder", READ, CONSTANT)]
        #[qproperty(QString, desktop_idle_status, cxx_name = "desktopIdleStatus", READ, CONSTANT)]
        #[qproperty(QString, desktop_running_status, cxx_name = "desktopRunningStatus", READ, CONSTANT)]
        #[qproperty(QString, desktop_paused_status, cxx_name = "desktopPausedStatus", READ, CONSTANT)]
        #[qproperty(QString, desktop_no_controllers, cxx_name = "desktopNoControllers", READ, CONSTANT)]
        #[qproperty(QString, desktop_batch_guarantee_title, cxx_name = "desktopBatchGuaranteeTitle", READ, CONSTANT)]
        #[qproperty(QString, desktop_contract_note, cxx_name = "desktopContractNote", READ, CONSTANT)]
        #[qproperty(QString, desktop_filter_resume_ready, cxx_name = "desktopFilterResumeReady", READ, CONSTANT)]
        #[qproperty(QString, desktop_filter_unplayed, cxx_name = "desktopFilterUnplayed", READ, CONSTANT)]
        #[qproperty(QString, desktop_sort_label, cxx_name = "desktopSortLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_console_options_label, cxx_name = "desktopConsoleOptionsLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_resume_badge, cxx_name = "desktopResumeBadge", READ, CONSTANT)]
        #[qproperty(QString, desktop_inspector_heading, cxx_name = "desktopInspectorHeading", READ, CONSTANT)]
        #[qproperty(QString, desktop_play_label, cxx_name = "desktopPlayLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_save_states_label, cxx_name = "desktopSaveStatesLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_no_states_label, cxx_name = "desktopNoStatesLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_program_facts_label, cxx_name = "desktopProgramFactsLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_fact_core, cxx_name = "desktopFactCore", READ, CONSTANT)]
        #[qproperty(QString, desktop_fact_played, cxx_name = "desktopFactPlayed", READ, CONSTANT)]
        #[qproperty(QString, desktop_fact_players, cxx_name = "desktopFactPlayers", READ, CONSTANT)]
        #[qproperty(QString, desktop_fact_genre, cxx_name = "desktopFactGenre", READ, CONSTANT)]
        #[qproperty(QString, desktop_fact_file, cxx_name = "desktopFactFile", READ, CONSTANT)]
        #[qproperty(QString, desktop_fact_size, cxx_name = "desktopFactSize", READ, CONSTANT)]
        #[qproperty(QString, desktop_fact_last_played, cxx_name = "desktopFactLastPlayed", READ, CONSTANT)]
        #[qproperty(QString, desktop_favourite_label, cxx_name = "desktopFavouriteLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_unfavourite_label, cxx_name = "desktopUnfavouriteLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_details_label, cxx_name = "desktopDetailsLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_folder_label, cxx_name = "desktopFolderLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_table_heading, cxx_name = "desktopTableHeading", READ, CONSTANT)]
        #[qproperty(QString, desktop_clear_filters_label, cxx_name = "desktopClearFiltersLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_clear_label, cxx_name = "desktopClearLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_unmark_fav_label, cxx_name = "desktopUnmarkFavLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_dash, cxx_name = "desktopDash", READ, CONSTANT)]
        #[qproperty(QString, desktop_strip_pause, cxx_name = "desktopStripPause", READ, CONSTANT)]
        #[qproperty(QString, desktop_strip_rewind, cxx_name = "desktopStripRewind", READ, CONSTANT)]
        #[qproperty(QString, desktop_strip_states, cxx_name = "desktopStripStates", READ, CONSTANT)]
        #[qproperty(QString, desktop_strip_shaders, cxx_name = "desktopStripShaders", READ, CONSTANT)]
        #[qproperty(QString, desktop_strip_cheats, cxx_name = "desktopStripCheats", READ, CONSTANT)]
        #[qproperty(QString, desktop_strip_shot, cxx_name = "desktopStripShot", READ, CONSTANT)]
        #[qproperty(QString, desktop_strip_fullscreen, cxx_name = "desktopStripFullscreen", READ, CONSTANT)]
        #[qproperty(QString, desktop_quick_save_label, cxx_name = "desktopQuickSaveLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_quick_load_label, cxx_name = "desktopQuickLoadLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_slot_prefix, cxx_name = "desktopSlotPrefix", READ, CONSTANT)]
        #[qproperty(QString, desktop_empty_slot_label, cxx_name = "desktopEmptySlotLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_rewind_available, cxx_name = "desktopRewindAvailable", READ, CONSTANT)]
        #[qproperty(QString, desktop_no_rewind_label, cxx_name = "desktopNoRewindLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_master_cheats_label, cxx_name = "desktopMasterCheatsLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_no_cheats_label, cxx_name = "desktopNoCheatsLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_shaders_heading, cxx_name = "desktopShadersHeading", READ, CONSTANT)]
        #[qproperty(QString, desktop_no_display_rows_label, cxx_name = "desktopNoDisplayRowsLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_speed_label, cxx_name = "desktopSpeedLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_swap_players_label, cxx_name = "desktopSwapPlayersLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_restart_label, cxx_name = "desktopRestartLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_quit_game_label, cxx_name = "desktopQuitGameLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_settings_filter_placeholder, cxx_name = "desktopSettingsFilterPlaceholder", READ, CONSTANT)]
        #[qproperty(QString, desktop_settings_heading, cxx_name = "desktopSettingsHeading", READ, CONSTANT)]
        #[qproperty(QString, desktop_tools_heading, cxx_name = "desktopToolsHeading", READ, CONSTANT)]
        #[qproperty(QString, desktop_modified_only_label, cxx_name = "desktopModifiedOnlyLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_back_label, cxx_name = "desktopBackLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_themes_heading, cxx_name = "desktopThemesHeading", READ, CONSTANT)]
        #[qproperty(QString, desktop_themes_subtitle, cxx_name = "desktopThemesSubtitle", READ, CONSTANT)]
        #[qproperty(QString, desktop_use_theme_label, cxx_name = "desktopUseThemeLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_in_use_label, cxx_name = "desktopInUseLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_dual_face_badge, cxx_name = "desktopDualFaceBadge", READ, CONSTANT)]
        #[qproperty(QString, desktop_big_picture_badge, cxx_name = "desktopBigPictureBadge", READ, CONSTANT)]
        #[qproperty(QString, desktop_esde_badge, cxx_name = "desktopEsdeBadge", READ, CONSTANT)]
        #[qproperty(QString, desktop_check_theme_label, cxx_name = "desktopCheckThemeLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_about_detail_heading, cxx_name = "desktopAboutDetailHeading", READ, CONSTANT)]
        #[qproperty(QString, desktop_status_hints, cxx_name = "desktopStatusHints", READ, CONSTANT)]
        #[qproperty(QString, desktop_never_played, cxx_name = "desktopNeverPlayed", READ, CONSTANT)]
        #[qproperty(QString, desktop_read_in_place, cxx_name = "desktopReadInPlace", READ, CONSTANT)]
        #[qproperty(QString, desktop_none_label, cxx_name = "desktopNoneLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_library_manager_label, cxx_name = "desktopLibraryManagerLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_resume_auto_label, cxx_name = "desktopResumeAutoLabel", READ, CONSTANT)]
        #[qproperty(QString, desktop_speed_max_label, cxx_name = "desktopSpeedMaxLabel", READ, CONSTANT)]
        type ThemeWords = super::ThemeWordsRust;
    }

    extern "RustQt" {
        /// "1 game", "1,566 games".
        #[qinvokable]
        fn games(self: &ThemeWords, count: i32) -> QString;
        /// A count grouped by thousands: "1,214".
        #[qinvokable]
        fn count(self: &ThemeWords, count: i32) -> QString;
        /// Parts said as one line, the empty ones left out: "Nintendo · 1990".
        #[qinvokable]
        fn line(self: &ThemeWords, parts: &QStringList) -> QString;
        /// A battery's charge: "82%"; empty below 0, where the machine has none.
        #[qinvokable]
        fn percent(self: &ThemeWords, charge: i32) -> QString;
        /// Where an item stands among its list: "3 of 14", for the item at `index` from 0.
        #[qinvokable]
        fn place(self: &ThemeWords, index: i32, count: i32) -> QString;
        /// How long ago a time in seconds since 1970 was: "12 min ago".
        #[qinvokable]
        fn ago(self: &ThemeWords, at: f64) -> QString;
        /// Continue's facts: the console's name and how long ago the game was paused.
        #[qinvokable]
        #[cxx_name = "pausedFacts"]
        fn paused_facts(self: &ThemeWords, console: &QString, paused_since: f64) -> QString;
        /// A console's facts: "Nintendo · 1983 · 2 games · runs on Moon, an EmuSen core", the empty parts left out.
        #[qinvokable]
        fn facts(self: &ThemeWords, maker: &QString, year: i32, count: i32, runs_on: &QString) -> QString;
        /// The facts' last words, from a console's `engineName` and `engineKind`.
        #[qinvokable]
        #[cxx_name = "runsOn"]
        fn runs_on(self: &ThemeWords, engine_name: &QString, engine_kind: &QString) -> QString;
        /// Home's status line: the controllers connected, where there are any, and the clock.
        #[qinvokable]
        #[cxx_name = "statusLine"]
        fn status_line(self: &ThemeWords, controllers: i32, clock: &QString) -> QString;
        /// The splash's caption: the consoles' count in capitals.
        #[qinvokable]
        #[cxx_name = "splashCaption"]
        fn splash_caption(self: &ThemeWords, consoles: i32) -> QString;
        /// The counts under Eclipse's splash wordmark (1.10): "1,284 GAMES · 8 CORES".
        #[qinvokable]
        #[cxx_name = "splashCounts"]
        fn splash_counts(self: &ThemeWords, games: i32, cores: i32) -> QString;
        /// Choose a console's summary: "11 consoles · 1,566 games".
        #[qinvokable]
        fn summary(self: &ThemeWords, consoles: i32, games: i32) -> QString;
        /// How long ago the suspended game was paused: "Paused 12 minutes ago".
        #[qinvokable]
        #[cxx_name = "pausedAgo"]
        fn paused_ago(self: &ThemeWords, paused_since: f64) -> QString;
        /// A game's last play and its time played: "Last played yesterday · 3 h 20 min played".
        #[qinvokable]
        #[cxx_name = "playedFacts"]
        fn played_facts(self: &ThemeWords, at: f64, seconds: f64) -> QString;
        /// The line over a suspended game's name: "Continue · Super Nintendo".
        #[qinvokable]
        #[cxx_name = "continueLine"]
        fn continue_line(self: &ThemeWords, console: &QString) -> QString;

        /// "Resume point · saved 9:12 PM": a suspended game's picture and when it was paused, in the machine's short form.
        #[qinvokable]
        #[cxx_name = "resumePoint"]
        fn resume_point(self: &ThemeWords, paused_since: f64) -> QString;
        /// The art behind Eclipse's disc named: "Game art · Aurora Drift".
        #[qinvokable]
        #[cxx_name = "gameArt"]
        fn game_art(self: &ThemeWords, title: &QString) -> QString;
        /// A moment, in seconds since 1970, in the machine's zone and short time form: "9:12 PM" under en_US.
        #[qinvokable]
        #[cxx_name = "shortTime"]
        fn short_time(self: &ThemeWords, at: f64) -> QString;
    }
}

pub struct ThemeWordsRust {
    title: QString,
    name: QString,
    games_heading: QString,
    choose_a_console: QString,
    games_path: QString,
    continue_playing: QString,
    recent: QString,
    between: QString,
    search: QString,
    emusen_tag: QString,
    libretro_tag: QString,
    no_games: QString,
    no_recent: QString,
    no_consoles: QString,
    add_folder: QString,
    places: QString,
    continue_entry: QString,
    recent_entry: QString,
    fact_labels: cxx_qt_lib::QStringList,
    console_shows: cxx_qt_lib::QStringList,
    game_fact_labels: cxx_qt_lib::QStringList,
    crumb: QString,
    no_games_line: QString,
    console_sorts: cxx_qt_lib::QStringList,
    key_pair: QString,
    now: f64,
    desktop_nav_tabs: cxx_qt_lib::QStringList,
    desktop_view_modes: cxx_qt_lib::QStringList,
    desktop_sort_labels: cxx_qt_lib::QStringList,
    desktop_table_columns: cxx_qt_lib::QStringList,
    desktop_drawer_tabs: cxx_qt_lib::QStringList,
    desktop_search_placeholder: QString,
    desktop_big_picture_label: QString,
    desktop_big_picture_hint: QString,
    desktop_resume_label: QString,
    desktop_sidebar_tip: QString,
    desktop_inspector_tip: QString,
    desktop_constellation_heading: QString,
    desktop_all_games_label: QString,
    desktop_recent_label: QString,
    desktop_consoles_heading: QString,
    desktop_scope_active: QString,
    desktop_console_filter_placeholder: QString,
    desktop_idle_status: QString,
    desktop_running_status: QString,
    desktop_paused_status: QString,
    desktop_no_controllers: QString,
    desktop_batch_guarantee_title: QString,
    desktop_contract_note: QString,
    desktop_filter_resume_ready: QString,
    desktop_filter_unplayed: QString,
    desktop_sort_label: QString,
    desktop_console_options_label: QString,
    desktop_resume_badge: QString,
    desktop_inspector_heading: QString,
    desktop_play_label: QString,
    desktop_save_states_label: QString,
    desktop_no_states_label: QString,
    desktop_program_facts_label: QString,
    desktop_fact_core: QString,
    desktop_fact_played: QString,
    desktop_fact_players: QString,
    desktop_fact_genre: QString,
    desktop_fact_file: QString,
    desktop_fact_size: QString,
    desktop_fact_last_played: QString,
    desktop_favourite_label: QString,
    desktop_unfavourite_label: QString,
    desktop_details_label: QString,
    desktop_folder_label: QString,
    desktop_table_heading: QString,
    desktop_clear_filters_label: QString,
    desktop_clear_label: QString,
    desktop_unmark_fav_label: QString,
    desktop_dash: QString,
    desktop_strip_pause: QString,
    desktop_strip_rewind: QString,
    desktop_strip_states: QString,
    desktop_strip_shaders: QString,
    desktop_strip_cheats: QString,
    desktop_strip_shot: QString,
    desktop_strip_fullscreen: QString,
    desktop_quick_save_label: QString,
    desktop_quick_load_label: QString,
    desktop_slot_prefix: QString,
    desktop_empty_slot_label: QString,
    desktop_rewind_available: QString,
    desktop_no_rewind_label: QString,
    desktop_master_cheats_label: QString,
    desktop_no_cheats_label: QString,
    desktop_shaders_heading: QString,
    desktop_no_display_rows_label: QString,
    desktop_speed_label: QString,
    desktop_swap_players_label: QString,
    desktop_restart_label: QString,
    desktop_quit_game_label: QString,
    desktop_settings_filter_placeholder: QString,
    desktop_settings_heading: QString,
    desktop_tools_heading: QString,
    desktop_modified_only_label: QString,
    desktop_back_label: QString,
    desktop_themes_heading: QString,
    desktop_themes_subtitle: QString,
    desktop_use_theme_label: QString,
    desktop_in_use_label: QString,
    desktop_dual_face_badge: QString,
    desktop_big_picture_badge: QString,
    desktop_esde_badge: QString,
    desktop_check_theme_label: QString,
    desktop_about_detail_heading: QString,
    desktop_status_hints: QString,
    desktop_never_played: QString,
    desktop_read_in_place: QString,
    desktop_none_label: QString,
    desktop_library_manager_label: QString,
    desktop_resume_auto_label: QString,
    desktop_speed_max_label: QString,
}

impl Default for ThemeWordsRust {
    fn default() -> Self {
        ThemeWordsRust {
            title: QString::from(words::TITLE),
            name: QString::from(words::NAME),
            games_heading: QString::from(words::GAMES),
            choose_a_console: QString::from(words::CHOOSE_A_CONSOLE),
            games_path: QString::from(words::GAMES_PATH),
            continue_playing: QString::from(words::CONTINUE_PLAYING),
            recent: QString::from(words::RECENT),
            between: QString::from(words::BETWEEN),
            search: QString::from(words::SEARCH),
            emusen_tag: QString::from(words::EMUSEN_TAG),
            libretro_tag: QString::from(words::LIBRETRO_TAG),
            no_games: QString::from(words::NO_GAMES),
            no_recent: QString::from(words::NO_RECENT),
            no_consoles: QString::from(words::NO_CONSOLES),
            add_folder: QString::from(words::ADD_FOLDER),
            places: QString::from(words::PLACES),
            continue_entry: QString::from(words::CONTINUE_ENTRY),
            recent_entry: QString::from(words::RECENT_ENTRY),
            fact_labels: string_list(&words::FACT_LABELS),
            console_shows: string_list(&words::CONSOLE_SHOWS),
            game_fact_labels: string_list(&words::GAME_FACT_LABELS),
            crumb: QString::from(words::CRUMB),
            no_games_line: QString::from(words::NO_GAMES_LINE),
            console_sorts: string_list(&words::CONSOLE_SORTS),
            key_pair: QString::from(words::KEY_PAIR),
            now: 0.0,
            desktop_nav_tabs: string_list(&words::DESKTOP_NAV_TABS),
            desktop_view_modes: string_list(&words::DESKTOP_VIEW_MODES),
            desktop_sort_labels: string_list(&words::DESKTOP_SORT_LABELS),
            desktop_table_columns: string_list(&words::DESKTOP_TABLE_COLUMNS),
            desktop_drawer_tabs: string_list(&words::DESKTOP_DRAWER_TABS),
            desktop_search_placeholder: QString::from(words::DESKTOP_SEARCH_PLACEHOLDER),
            desktop_big_picture_label: QString::from(words::DESKTOP_BIG_PICTURE_LABEL),
            desktop_big_picture_hint: QString::from(words::DESKTOP_BIG_PICTURE_HINT),
            desktop_resume_label: QString::from(words::DESKTOP_RESUME_LABEL),
            desktop_sidebar_tip: QString::from(words::DESKTOP_SIDEBAR_TIP),
            desktop_inspector_tip: QString::from(words::DESKTOP_INSPECTOR_TIP),
            desktop_constellation_heading: QString::from(words::DESKTOP_CONSTELLATION_HEADING),
            desktop_all_games_label: QString::from(words::DESKTOP_ALL_GAMES_LABEL),
            desktop_recent_label: QString::from(words::DESKTOP_RECENT_LABEL),
            desktop_consoles_heading: QString::from(words::DESKTOP_CONSOLES_HEADING),
            desktop_scope_active: QString::from(words::DESKTOP_SCOPE_ACTIVE),
            desktop_console_filter_placeholder: QString::from(words::DESKTOP_CONSOLE_FILTER_PLACEHOLDER),
            desktop_idle_status: QString::from(words::DESKTOP_IDLE_STATUS),
            desktop_running_status: QString::from(words::DESKTOP_RUNNING_STATUS),
            desktop_paused_status: QString::from(words::DESKTOP_PAUSED_STATUS),
            desktop_no_controllers: QString::from(words::DESKTOP_NO_CONTROLLERS),
            desktop_batch_guarantee_title: QString::from(words::DESKTOP_BATCH_GUARANTEE_TITLE),
            desktop_contract_note: QString::from(words::DESKTOP_CONTRACT_NOTE),
            desktop_filter_resume_ready: QString::from(words::DESKTOP_FILTER_RESUME_READY),
            desktop_filter_unplayed: QString::from(words::DESKTOP_FILTER_UNPLAYED),
            desktop_sort_label: QString::from(words::DESKTOP_SORT_LABEL),
            desktop_console_options_label: QString::from(words::DESKTOP_CONSOLE_OPTIONS_LABEL),
            desktop_resume_badge: QString::from(words::DESKTOP_RESUME_BADGE),
            desktop_inspector_heading: QString::from(words::DESKTOP_INSPECTOR_HEADING),
            desktop_play_label: QString::from(words::DESKTOP_PLAY_LABEL),
            desktop_save_states_label: QString::from(words::DESKTOP_SAVE_STATES_LABEL),
            desktop_no_states_label: QString::from(words::DESKTOP_NO_STATES_LABEL),
            desktop_program_facts_label: QString::from(words::DESKTOP_PROGRAM_FACTS_LABEL),
            desktop_fact_core: QString::from(words::DESKTOP_FACT_CORE),
            desktop_fact_played: QString::from(words::DESKTOP_FACT_PLAYED),
            desktop_fact_players: QString::from(words::DESKTOP_FACT_PLAYERS),
            desktop_fact_genre: QString::from(words::DESKTOP_FACT_GENRE),
            desktop_fact_file: QString::from(words::DESKTOP_FACT_FILE),
            desktop_fact_size: QString::from(words::DESKTOP_FACT_SIZE),
            desktop_fact_last_played: QString::from(words::DESKTOP_FACT_LAST_PLAYED),
            desktop_favourite_label: QString::from(words::DESKTOP_FAVOURITE_LABEL),
            desktop_unfavourite_label: QString::from(words::DESKTOP_UNFAVOURITE_LABEL),
            desktop_details_label: QString::from(words::DESKTOP_DETAILS_LABEL),
            desktop_folder_label: QString::from(words::DESKTOP_FOLDER_LABEL),
            desktop_table_heading: QString::from(words::DESKTOP_TABLE_HEADING),
            desktop_clear_filters_label: QString::from(words::DESKTOP_CLEAR_FILTERS_LABEL),
            desktop_clear_label: QString::from(words::DESKTOP_CLEAR_LABEL),
            desktop_unmark_fav_label: QString::from(words::DESKTOP_UNMARK_FAV_LABEL),
            desktop_dash: QString::from(words::DESKTOP_DASH),
            desktop_strip_pause: QString::from(words::DESKTOP_STRIP_PAUSE),
            desktop_strip_rewind: QString::from(words::DESKTOP_STRIP_REWIND),
            desktop_strip_states: QString::from(words::DESKTOP_STRIP_STATES),
            desktop_strip_shaders: QString::from(words::DESKTOP_STRIP_SHADERS),
            desktop_strip_cheats: QString::from(words::DESKTOP_STRIP_CHEATS),
            desktop_strip_shot: QString::from(words::DESKTOP_STRIP_SHOT),
            desktop_strip_fullscreen: QString::from(words::DESKTOP_STRIP_FULLSCREEN),
            desktop_quick_save_label: QString::from(words::DESKTOP_QUICK_SAVE_LABEL),
            desktop_quick_load_label: QString::from(words::DESKTOP_QUICK_LOAD_LABEL),
            desktop_slot_prefix: QString::from(words::DESKTOP_SLOT_PREFIX),
            desktop_empty_slot_label: QString::from(words::DESKTOP_EMPTY_SLOT_LABEL),
            desktop_rewind_available: QString::from(words::DESKTOP_REWIND_AVAILABLE),
            desktop_no_rewind_label: QString::from(words::DESKTOP_NO_REWIND_LABEL),
            desktop_master_cheats_label: QString::from(words::DESKTOP_MASTER_CHEATS_LABEL),
            desktop_no_cheats_label: QString::from(words::DESKTOP_NO_CHEATS_LABEL),
            desktop_shaders_heading: QString::from(words::DESKTOP_SHADERS_HEADING),
            desktop_no_display_rows_label: QString::from(words::DESKTOP_NO_DISPLAY_ROWS_LABEL),
            desktop_speed_label: QString::from(words::DESKTOP_SPEED_LABEL),
            desktop_swap_players_label: QString::from(words::DESKTOP_SWAP_PLAYERS_LABEL),
            desktop_restart_label: QString::from(words::DESKTOP_RESTART_LABEL),
            desktop_quit_game_label: QString::from(words::DESKTOP_QUIT_GAME_LABEL),
            desktop_settings_filter_placeholder: QString::from(words::DESKTOP_SETTINGS_FILTER_PLACEHOLDER),
            desktop_settings_heading: QString::from(words::DESKTOP_SETTINGS_HEADING),
            desktop_tools_heading: QString::from(words::DESKTOP_TOOLS_HEADING),
            desktop_modified_only_label: QString::from(words::DESKTOP_MODIFIED_ONLY_LABEL),
            desktop_back_label: QString::from(words::DESKTOP_BACK_LABEL),
            desktop_themes_heading: QString::from(words::DESKTOP_THEMES_HEADING),
            desktop_themes_subtitle: QString::from(words::DESKTOP_THEMES_SUBTITLE),
            desktop_use_theme_label: QString::from(words::DESKTOP_USE_THEME_LABEL),
            desktop_in_use_label: QString::from(words::DESKTOP_IN_USE_LABEL),
            desktop_dual_face_badge: QString::from(words::DESKTOP_DUAL_FACE_BADGE),
            desktop_big_picture_badge: QString::from(words::DESKTOP_BIG_PICTURE_BADGE),
            desktop_esde_badge: QString::from(words::DESKTOP_ESDE_BADGE),
            desktop_check_theme_label: QString::from(words::DESKTOP_CHECK_THEME_LABEL),
            desktop_about_detail_heading: QString::from(words::DESKTOP_ABOUT_DETAIL_HEADING),
            desktop_status_hints: QString::from(words::DESKTOP_STATUS_HINTS),
            desktop_never_played: QString::from(words::DESKTOP_NEVER_PLAYED),
            desktop_read_in_place: QString::from(words::DESKTOP_READ_IN_PLACE),
            desktop_none_label: QString::from(words::DESKTOP_NONE_LABEL),
            desktop_library_manager_label: QString::from(words::DESKTOP_LIBRARY_MANAGER_LABEL),
            desktop_resume_auto_label: QString::from(words::DESKTOP_RESUME_AUTO_LABEL),
            desktop_speed_max_label: QString::from(words::DESKTOP_SPEED_MAX_LABEL),
        }
    }
}

fn said(text: String) -> QString {
    QString::from(text.as_str())
}

impl qobject::ThemeWords {
    /// The host's: the time the phrases are said against, 0 for the wall clock.
    pub fn show_now(mut self: core::pin::Pin<&mut Self>, now: f64) {
        if self.now != now {
            self.as_mut().rust_mut().now = now;
            self.now_changed();
        }
    }

    /// Now, held or the wall clock's.
    fn at(&self) -> i64 {
        if self.now > 0.0 { self.now as i64 } else { words::now() }
    }

    pub fn games(&self, count: i32) -> QString {
        said(words::games(count.max(0) as usize))
    }

    pub fn line(&self, parts: &cxx_qt_lib::QStringList) -> QString {
        let parts: Vec<String> = cxx_qt_lib::QList::<QString>::from(parts).iter().map(|p| p.to_string()).collect();
        said(words::line(&parts.iter().map(String::as_str).collect::<Vec<_>>()))
    }

    pub fn count(&self, count: i32) -> QString {
        said(words::grouped(count.max(0) as usize))
    }

    pub fn percent(&self, charge: i32) -> QString {
        said(words::percent(charge))
    }

    pub fn place(&self, index: i32, count: i32) -> QString {
        said(emusen_qt_models::game_page::place(index.max(0) as usize, count.max(0) as usize))
    }

    pub fn ago(&self, at: f64) -> QString {
        said(words::ago(at as i64, self.at()))
    }

    pub fn paused_facts(&self, console: &QString, paused_since: f64) -> QString {
        said(words::paused_facts(&console.to_string(), paused_since as i64, self.at()))
    }

    pub fn played_facts(&self, at: f64, seconds: f64) -> QString {
        said(words::played_facts(at as i64, self.at(), seconds as i64))
    }

    pub fn continue_line(&self, console: &QString) -> QString {
        said(words::continue_line(&console.to_string()))
    }

    pub fn resume_point(&self, paused_since: f64) -> QString {
        said(words::resume_point(&emusen_qt_models::clock::short_time(paused_since as i64)))
    }

    pub fn game_art(&self, title: &QString) -> QString {
        said(words::game_art(&title.to_string()))
    }

    pub fn short_time(&self, at: f64) -> QString {
        QString::from(emusen_qt_models::clock::short_time(at as i64).as_str())
    }

    pub fn facts(&self, maker: &QString, year: i32, count: i32, runs_on: &QString) -> QString {
        said(words::facts(&maker.to_string(), year, count.max(0) as usize, &runs_on.to_string()))
    }

    pub fn runs_on(&self, engine_name: &QString, engine_kind: &QString) -> QString {
        let name = engine_name.to_string();
        if name.is_empty() {
            return QString::default();
        }
        said(words::runs_on(&name, engine_kind.to_string() == crate::view::engine_kind(true)))
    }

    pub fn status_line(&self, controllers: i32, clock: &QString) -> QString {
        said(words::status_line(controllers.max(0) as usize, &clock.to_string()))
    }

    pub fn summary(&self, consoles: i32, games: i32) -> QString {
        said(words::summary(consoles.max(0) as usize, games.max(0) as usize))
    }

    pub fn paused_ago(&self, paused_since: f64) -> QString {
        said(words::paused_ago(paused_since as i64, self.at()))
    }

    pub fn splash_caption(&self, consoles: i32) -> QString {
        said(words::splash_caption(consoles.max(0) as usize))
    }

    pub fn splash_counts(&self, games: i32, cores: i32) -> QString {
        said(words::splash_counts(games.max(0) as usize, cores.max(0) as usize))
    }
}

/// Words as a QML list of strings.
fn string_list(words: &[&str]) -> cxx_qt_lib::QStringList {
    let mut list = cxx_qt_lib::QList::<QString>::default();
    words.iter().for_each(|w| list.append(QString::from(*w)));
    cxx_qt_lib::QStringList::from(&list)
}
