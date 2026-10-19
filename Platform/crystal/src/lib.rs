//! Crystal, the runtime beneath the frontends, in Rust: the frontend API `emusen_crystal.h` as the C# implementation of
//! step B2 serves it, built in the parts of step B7 (EmuSen_Backend.md §29). `abi` holds the header's numbers and
//! structs; the runtime's modules follow §29.2's layout, one job each.

pub mod abi;
pub mod battery;
pub mod catalog;
pub mod engine;
pub mod error_log;
pub mod events;
pub mod firmware;
pub mod info;
pub mod input;
pub mod json;
pub mod machine_info;
pub mod media;
pub mod open;
pub mod png;
pub mod rewind;
pub mod route;
pub mod runtime;
pub mod saving;
pub mod seam;
pub mod session;
pub mod settings;
pub mod settings_api;
pub mod speed;
pub mod states;
