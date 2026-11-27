// Consolidated eclipse_soft integration tests.
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

#[path = "eclipse_soft/eclipse.rs"]
mod eclipse;
#[path = "eclipse_soft/eclipse_bindings.rs"]
mod eclipse_bindings;
#[path = "eclipse_soft/eclipse_cards.rs"]
mod eclipse_cards;
#[path = "eclipse_soft/eclipse_cheats.rs"]
mod eclipse_cheats;
#[path = "eclipse_soft/eclipse_consoles.rs"]
mod eclipse_consoles;
#[path = "eclipse_soft/eclipse_cores.rs"]
mod eclipse_cores;
#[path = "eclipse_soft/eclipse_faces.rs"]
mod eclipse_faces;
#[path = "eclipse_soft/eclipse_game.rs"]
mod eclipse_game;
#[path = "eclipse_soft/eclipse_game_cheats.rs"]
mod eclipse_game_cheats;
#[path = "eclipse_soft/eclipse_game_cover.rs"]
mod eclipse_game_cover;
#[path = "eclipse_soft/eclipse_game_crop.rs"]
mod eclipse_game_crop;
#[path = "eclipse_soft/eclipse_game_details.rs"]
mod eclipse_game_details;
#[path = "eclipse_soft/eclipse_game_edit.rs"]
mod eclipse_game_edit;
#[path = "eclipse_soft/eclipse_game_media.rs"]
mod eclipse_game_media;
#[path = "eclipse_soft/eclipse_game_never.rs"]
mod eclipse_game_never;
#[path = "eclipse_soft/eclipse_game_options.rs"]
mod eclipse_game_options;
#[path = "eclipse_soft/eclipse_game_overview.rs"]
mod eclipse_game_overview;
#[path = "eclipse_soft/eclipse_game_package.rs"]
mod eclipse_game_package;
#[path = "eclipse_soft/eclipse_game_pad.rs"]
mod eclipse_game_pad;
#[path = "eclipse_soft/eclipse_game_scrape.rs"]
mod eclipse_game_scrape;
#[path = "eclipse_soft/eclipse_game_states.rs"]
mod eclipse_game_states;
#[path = "eclipse_soft/eclipse_game_viewer.rs"]
mod eclipse_game_viewer;
#[path = "eclipse_soft/eclipse_games.rs"]
mod eclipse_games;
#[path = "eclipse_soft/eclipse_games_empty.rs"]
mod eclipse_games_empty;
#[path = "eclipse_soft/eclipse_games_lists.rs"]
mod eclipse_games_lists;
#[path = "eclipse_soft/eclipse_home.rs"]
mod eclipse_home;
#[path = "eclipse_soft/eclipse_home_pad.rs"]
mod eclipse_home_pad;
#[path = "eclipse_soft/eclipse_nav.rs"]
mod eclipse_nav;
#[path = "eclipse_soft/eclipse_packages.rs"]
mod eclipse_packages;
#[path = "eclipse_soft/eclipse_pause.rs"]
mod eclipse_pause;
#[path = "eclipse_soft/eclipse_settings.rs"]
mod eclipse_settings;
#[path = "eclipse_soft/eclipse_settings_cards.rs"]
mod eclipse_settings_cards;
#[path = "eclipse_soft/eclipse_settings_collections.rs"]
mod eclipse_settings_collections;
#[path = "eclipse_soft/eclipse_settings_console.rs"]
mod eclipse_settings_console;
#[path = "eclipse_soft/eclipse_settings_consoles.rs"]
mod eclipse_settings_consoles;
#[path = "eclipse_soft/eclipse_settings_controllers.rs"]
mod eclipse_settings_controllers;
#[path = "eclipse_soft/eclipse_settings_core.rs"]
mod eclipse_settings_core;
#[path = "eclipse_soft/eclipse_settings_emusen.rs"]
mod eclipse_settings_emusen;
#[path = "eclipse_soft/eclipse_settings_firmware.rs"]
mod eclipse_settings_firmware;
#[path = "eclipse_soft/eclipse_settings_library.rs"]
mod eclipse_settings_library;
#[path = "eclipse_soft/eclipse_settings_s6.rs"]
mod eclipse_settings_s6;
#[path = "eclipse_soft/eclipse_settings_s7.rs"]
mod eclipse_settings_s7;
#[path = "eclipse_soft/eclipse_settings_saves.rs"]
mod eclipse_settings_saves;
#[path = "eclipse_soft/eclipse_settings_scraping.rs"]
mod eclipse_settings_scraping;
#[path = "eclipse_soft/eclipse_settings_sound.rs"]
mod eclipse_settings_sound;
#[path = "eclipse_soft/eclipse_shaders.rs"]
mod eclipse_shaders;
#[path = "eclipse_soft/eclipse_sys1.rs"]
mod eclipse_sys1;
#[path = "eclipse_soft/eclipse_sys2.rs"]
mod eclipse_sys2;
#[path = "eclipse_soft/eclipse_tokens_agnostic.rs"]
mod eclipse_tokens_agnostic;
#[path = "eclipse_soft/eclipse_tools_t4.rs"]
mod eclipse_tools_t4;
