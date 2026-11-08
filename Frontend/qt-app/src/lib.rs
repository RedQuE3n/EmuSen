//! emusen-qt, the Qt front end's program (EmuSen_Qt.md §7): the module EmuSen.App and what the program reads before its
//! window opens.

// The modules the scene imports register through their crates' initializers; a crate that only names them must link them.
extern crate emusen_kit_qt;
extern crate emusen_qt_endymion;
extern crate emusen_qt_models;
extern crate emusen_qt_play;
extern crate emusen_theme_host;
extern crate emusen_theme_qt;

pub mod start;
pub mod theme;

/// Where the program's window lives in the resource system.
pub const MAIN_URL: &str = "qrc:/qt/qml/EmuSen/App/qml/Main.qml";

/// The reference theme "EmuSen", a package in the resources (Frontend/themes/emusen, EmuSen_Qt.md §3.4).
pub const REFERENCE_THEME: &str = "qrc:/emusen/themes/emusen/";
