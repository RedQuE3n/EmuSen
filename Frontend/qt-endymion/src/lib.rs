//! Endymion for the interface (EmuSen_Qt.md §1.7, EmuSen_Kit.md §7.15), the module `EmuSen.Endymion` 1.0: its pads read
//! into the kit's PadRouter with the help bar's family, and the kit's bells through its interface player.
//! `reading` and `reader` are plain Rust; `pad_reader` and `bell_player` only put them on Qt.

// The kit's QML module registers through its crate's initializer; this module's types name its enumerations.
extern crate emusen_kit_qt;

pub mod bell_player;
pub mod pad_reader;
pub mod reader;
pub mod reading;
