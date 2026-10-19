//! The cheat files and the player's cheat database, the C# `ChtFile`, `CheatDatabase`, `CheatDatabasePruner` and
//! `CheatDatabaseInstaller.Install` (EmuSen_Backend.md §15.3). The saved cheat lists, `cheats/<game>.json`, are
//! Galaxia's already (its `cheats` module); the registry a session applies is Crystal's (B7).

pub mod cht;
pub mod database;
pub mod install;
pub mod prune;
pub mod registry;
