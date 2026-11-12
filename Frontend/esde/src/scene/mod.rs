//! `scene`: each element's rectangle, opacity, offset and text at a time, for a resolved view, the selection and the
//! durations measured from ES-DE (EmuSen_BigPicture.md §13, §14, §16; EmuSen_Qt.md §4.2). Plain records out; the QML
//! painter binds them and animates nothing of its own.

pub mod data;
pub mod draw;
pub mod frame;
pub mod grid;
pub mod help;
pub mod mapping;
pub mod motion;
pub mod paint;
pub mod primary;
pub mod selector;
pub mod stage;
pub mod view;
pub mod viewer;
pub mod words;

pub use data::{ControllerType, Counts, Data, DateTime, DeviceStatus, Game, HelpContext, Indicators, System};
pub use frame::*;
pub use motion::{ms, Easing, Glide, Motion, RepeatRule, TextScroll, Time};
pub use stage::{Layer, Stage};
pub use view::{ElementState, GridFocus, Rect, SceneView, Step};
pub use viewer::{HelpPlace, Key, ManualView, Viewer, ViewerSettings};
