// Consolidated reference_soft integration tests.
#![allow(dead_code)]

extern crate emusen_qt;

#[allow(unused_imports)]
pub use emusen_qt_app_testlib::{
    bindings, common, cores_world, eclipse_board, eclipse_cards_oracle, eclipse_cheats_oracle,
    eclipse_consoles_oracle, eclipse_cores_oracle, eclipse_games_oracle, eclipse_home_oracle,
    eclipse_packages_oracle, eclipse_pad, eclipse_pad_oracle, eclipse_pause_oracle,
    eclipse_settings_cards_pages, eclipse_settings_consoles_pages, eclipse_settings_oracle,
    eclipse_settings_pages, eclipse_settings_s6_pages, eclipse_settings_s7_pages,
    eclipse_shaders_oracle, eclipse_sys1_oracle, eclipse_sys2_oracle, eclipse_table, game,
    start_held, table, tools_t4,
};

#[path = "reference_soft/card_controllers.rs"]
mod card_controllers;
#[path = "reference_soft/cards.rs"]
mod cards;
#[path = "reference_soft/cheats_get.rs"]
mod cheats_get;
#[path = "reference_soft/cheats_tool.rs"]
mod cheats_tool;
#[path = "reference_soft/console_fails.rs"]
mod console_fails;
#[path = "reference_soft/controller_failures.rs"]
mod controller_failures;
#[path = "reference_soft/cores.rs"]
mod cores;
#[path = "reference_soft/cores_fit.rs"]
mod cores_fit;
#[path = "reference_soft/desktop_face.rs"]
mod desktop_face;
#[path = "reference_soft/esde_rows.rs"]
mod esde_rows;
#[path = "reference_soft/failures.rs"]
mod failures;
#[path = "reference_soft/game_page.rs"]
mod game_page;
#[path = "reference_soft/game_page_cheats.rs"]
mod game_page_cheats;
#[path = "reference_soft/game_page_rows.rs"]
mod game_page_rows;
#[path = "reference_soft/gate_pad.rs"]
mod gate_pad;
#[path = "reference_soft/home_continue.rs"]
mod home_continue;
#[path = "reference_soft/idle.rs"]
mod idle;
#[path = "reference_soft/idle_hidden.rs"]
mod idle_hidden;
#[path = "reference_soft/page_cover.rs"]
mod page_cover;
#[path = "reference_soft/page_cover_none.rs"]
mod page_cover_none;
#[path = "reference_soft/page_doors.rs"]
mod page_doors;
#[path = "reference_soft/pause.rs"]
mod pause;
#[path = "reference_soft/pause_pages.rs"]
mod pause_pages;
#[path = "reference_soft/program.rs"]
mod program;
#[path = "reference_soft/repairs.rs"]
mod repairs;
#[path = "reference_soft/scraper.rs"]
mod scraper;
#[path = "reference_soft/screens.rs"]
mod screens;
#[path = "reference_soft/settings.rs"]
mod settings;
#[path = "reference_soft/settings_consoles.rs"]
mod settings_consoles;
#[path = "reference_soft/settings_controllers.rs"]
mod settings_controllers;
#[path = "reference_soft/settings_doors.rs"]
mod settings_doors;
#[path = "reference_soft/settings_drawings.rs"]
mod settings_drawings;
#[path = "reference_soft/settings_effects.rs"]
mod settings_effects;
#[path = "reference_soft/settings_reopen.rs"]
mod settings_reopen;
#[path = "reference_soft/settings_tester.rs"]
mod settings_tester;
#[path = "reference_soft/splash.rs"]
mod splash;
#[path = "reference_soft/splash_silent_device.rs"]
mod splash_silent_device;
#[path = "reference_soft/splash_start.rs"]
mod splash_start;
#[path = "reference_soft/start.rs"]
mod start;
#[path = "reference_soft/still.rs"]
mod still;
#[path = "reference_soft/theme.rs"]
mod theme;
#[path = "reference_soft/theme_empty_library.rs"]
mod theme_empty_library;
#[path = "reference_soft/theme_feed.rs"]
mod theme_feed;
#[path = "reference_soft/theme_library_words.rs"]
mod theme_library_words;
#[path = "reference_soft/theme_pages.rs"]
mod theme_pages;
#[path = "reference_soft/theme_pages_builtin.rs"]
mod theme_pages_builtin;
#[path = "reference_soft/theme_pages_play.rs"]
mod theme_pages_play;
#[path = "reference_soft/theme_reach.rs"]
mod theme_reach;
#[path = "reference_soft/theme_reach_pause.rs"]
mod theme_reach_pause;
#[path = "reference_soft/themes_esde.rs"]
mod themes_esde;
#[path = "reference_soft/themes_focus.rs"]
mod themes_focus;
#[path = "reference_soft/themes_tab.rs"]
mod themes_tab;
#[path = "reference_soft/tools.rs"]
mod tools;
