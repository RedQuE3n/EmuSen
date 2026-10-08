//! Galaxia, the librarian: where a file lives and how it is read and written safely.
//!
//! This is the Rust the C# `EmuSen.Galaxia` forwards to behind `EMUSEN_GALAXIA_NATIVE`, and the API a Rust DianaOS
//! calls directly. Every rule here reproduces the C#'s, which is its oracle until its gate; see EmuSen_Galaxia.md for
//! what the rules are and EmuSen_RustPlatform.md §10 for how this crate was built and checked.

pub mod atomic;
pub mod dotnet_path;
pub mod migration;
pub mod rom_hash;
pub mod saves;
pub mod tree;

#[cfg(test)]
mod test_support;
