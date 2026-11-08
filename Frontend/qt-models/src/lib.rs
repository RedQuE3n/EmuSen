//! emusen-qt-models: the view models of EmuSen_Qt.md §1.2, the module EmuSen.Models 1.0. Each model's behaviour is plain
//! Rust over a source the client crates or a test's fake answer; its bridge only puts that on Qt.

pub mod clients;
pub mod clock;
pub mod game_model;
pub mod games;
pub mod help;
pub mod help_model;
pub mod keyed;
pub mod library;
pub mod library_model;
pub mod screens;
pub mod words;
pub mod words_object;
