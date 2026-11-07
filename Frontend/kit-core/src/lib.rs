//! The pure-Rust heart of the `EmuSen.Kit` QML module, as EmuSen_Qt.md §1.4 defines it and EmuSen_Kit.md records it.
//! Since 2026-11-07 this crate is the one owner of the interface's tokens; LunaP's C# copy is retired at stage 8.
//! It names no Qt type, no runtime and no console: hosts hand it roles, looks and times, and it hands back decisions and geometry.

pub mod audit;
pub mod input;
pub mod motion;
pub mod tokens;
