// Consolidated eclipse_gpu integration tests.
#![allow(dead_code)]

extern crate emusen_qt;

#[allow(unused_imports)]
pub use emusen_qt_app_testlib::{
    bindings, common, cores_world, eclipse_board, eclipse_cards_oracle, eclipse_cheats_oracle,
    eclipse_consoles_oracle, eclipse_cores_oracle, eclipse_games_oracle, eclipse_home_oracle,
    eclipse_packages_oracle, eclipse_pad, eclipse_pad_oracle, eclipse_pause_oracle,
    eclipse_resume_oracle, eclipse_settings_cards_pages, eclipse_settings_consoles_pages, eclipse_settings_oracle,
    eclipse_settings_pages, eclipse_settings_s6_pages, eclipse_settings_s7_pages,
    eclipse_shaders_oracle, eclipse_sys1_oracle, eclipse_sys2_oracle, eclipse_table, game,
    start_held, table, tools_t4,
};

#[path = "eclipse_gpu/eclipse_consoles_gpu.rs"]
mod eclipse_consoles_gpu;
#[path = "eclipse_gpu/eclipse_game_gpu.rs"]
mod eclipse_game_gpu;
#[path = "eclipse_gpu/eclipse_home_gpu.rs"]
mod eclipse_home_gpu;
#[path = "eclipse_gpu/eclipse_resume_gpu.rs"]
mod eclipse_resume_gpu;
#[path = "eclipse_gpu/eclipse_rhi_smoke.rs"]
mod eclipse_rhi_smoke;
#[path = "eclipse_gpu/eclipse_shaders_gpu.rs"]
mod eclipse_shaders_gpu;
#[path = "eclipse_gpu/eclipse_splash.rs"]
mod eclipse_splash;
#[path = "eclipse_gpu/eclipse_splash_planets.rs"]
mod eclipse_splash_planets;
