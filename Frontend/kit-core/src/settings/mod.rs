//! The settings frame's rules (EmuSen_Kit.md §7.40–§7.42): where its strip, rows and pane stand, and the walk over its
//! rows, a stepped value, a slider and a page of choices among them. It names no setting: a host hands it rows by kind.

pub mod frame;
pub mod rows;
pub mod slider;

pub use frame::{SETTINGS, SettingsGeometry};
pub use rows::{Row, RowKind, RowOutcome, RowWalk};
pub use slider::SliderRange;
