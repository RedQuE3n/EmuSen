//! `emusen-esde`: the ES-DE engine of EmuSen_Qt.md §4.2, as plain data. `loader` reads a theme folder into resolved
//! views, `scene` gives each element's state at a time, `check` measures a folder of themes as TSV for esde-check.
//! No Qt: the QML painter reads what these give.

pub mod check;
pub mod loader;
pub mod scene;
