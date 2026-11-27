// Shared Qt front-end test helpers and screen oracles.
#![allow(dead_code)]

pub mod bindings;
pub mod common;
pub mod cores_world;
pub mod eclipse_board;
pub mod eclipse_cards_oracle;
pub mod eclipse_cheats_oracle;
pub mod eclipse_consoles_oracle;
pub mod eclipse_cores_oracle;
pub mod eclipse_games_oracle;
pub mod eclipse_home_oracle;
pub mod eclipse_packages_oracle;
pub mod eclipse_pad;
pub mod eclipse_pad_oracle;
pub mod eclipse_pause_oracle;
pub mod eclipse_settings_cards_pages;
pub mod eclipse_settings_consoles_pages;
pub mod eclipse_settings_oracle;
pub mod eclipse_settings_pages;
pub mod eclipse_settings_s6_pages;
pub mod eclipse_settings_s7_pages;
pub mod eclipse_shaders_oracle;
pub mod eclipse_sys1_oracle;
pub mod eclipse_sys2_oracle;
pub mod eclipse_table;
pub mod start_held;

pub use eclipse_board::{game, tools_t4};
pub use eclipse_table as table;
