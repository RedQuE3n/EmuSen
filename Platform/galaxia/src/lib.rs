//! Galaxia, the librarian: where a file lives and how it is read and written safely.
//!
//! This is the Rust the C# `EmuSen.Galaxia` forwards to behind `EMUSEN_GALAXIA_NATIVE`, and the API a Rust DianaSh
//! calls directly. Every rule here reproduces the C#'s, which is its oracle until its gate; see EmuSen_Galaxia.md for
//! what the rules are and EmuSen_RustPlatform.md §10 for how this crate was built and checked.

pub mod archive;
pub mod atomic;
pub mod cheats;
pub mod config;
pub mod consoles;
pub mod disc_set;
pub mod dotnet_files;
pub mod dotnet_path;
pub mod dotnet_text;
pub mod game_folders;
pub mod error_log;
pub mod image_probe;
pub mod image_sets;
pub mod json;
pub mod mapped;
pub mod medium_lock;
pub mod migration;
pub mod model;
pub mod models;
pub mod rom_hash;
pub mod saves;
pub mod setting_folders;
pub mod suggestion;
pub mod tree;

#[cfg(test)]
mod test_support;
