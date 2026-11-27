// Consolidated reference_gpu integration tests.
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

#[path = "reference_gpu/acceptance.rs"]
mod acceptance;
#[path = "reference_gpu/cheats_gpu.rs"]
mod cheats_gpu;
#[path = "reference_gpu/clip_gpu.rs"]
mod clip_gpu;
#[path = "reference_gpu/console.rs"]
mod console;
#[path = "reference_gpu/controls_gpu.rs"]
mod controls_gpu;
#[path = "reference_gpu/cores_fit_gpu.rs"]
mod cores_fit_gpu;
#[path = "reference_gpu/esde.rs"]
mod esde;
#[path = "reference_gpu/full_screen_gpu.rs"]
mod full_screen_gpu;
#[path = "reference_gpu/full_screen_small_gpu.rs"]
mod full_screen_small_gpu;
#[path = "reference_gpu/game_crop_gpu.rs"]
mod game_crop_gpu;
#[path = "reference_gpu/pause_settings_gpu.rs"]
mod pause_settings_gpu;
#[path = "reference_gpu/rumble_gpu.rs"]
mod rumble_gpu;
#[path = "reference_gpu/shaders_gpu.rs"]
mod shaders_gpu;
#[path = "reference_gpu/sound_gpu.rs"]
mod sound_gpu;
