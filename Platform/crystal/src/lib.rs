//! Crystal, the runtime beneath the frontends, in Rust: the frontend API `emusen_crystal.h` as the C# implementation of
//! step B2 serves it, built in the parts of step B7 (EmuSen_Backend.md §29). `abi` holds the header's numbers and
//! structs; the runtime's modules follow §29.2's layout, one job each.

pub mod abi;
pub mod catalog;
pub mod firmware;
pub mod info;
pub mod json;
pub mod route;
pub mod runtime;
pub mod settings;
