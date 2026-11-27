// Consolidated qt_gput integration tests.
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

#[path = "qt_gput/cores_play.rs"]
mod cores_play;
