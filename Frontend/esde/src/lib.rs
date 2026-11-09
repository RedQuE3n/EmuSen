//! `emusen-esde`: the ES-DE engine of EmuSen_Qt.md §4.2, as plain data. `loader` reads a theme folder into resolved
//! views, `scene` gives each element's state at a time and the painter's frame, `marks`
//! the built-in icons, `check` measures a folder of themes as TSV for esde-check, `fonts` finds ES-DE's own face where the player has it.
//! No Qt: the QML painter reads what these give.

pub mod check;
pub mod fonts;
pub mod loader;
pub mod marks;
pub mod scene;
