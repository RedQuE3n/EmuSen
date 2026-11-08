//! `scene`: each element's rectangle, opacity, offset and text at a time, for a resolved view, the selection and the
//! durations measured from ES-DE (EmuSen_BigPicture.md §13, §14, §16; EmuSen_Qt.md §4.2). Plain records out; the QML
//! painter binds them and animates nothing of its own.

pub mod data;
pub mod grid;
pub mod mapping;
pub mod motion;
pub mod stage;
pub mod view;
pub mod words;

pub use data::{Data, DateTime, Game, System};
pub use motion::{ms, Easing, Glide, Motion, RepeatRule, TextScroll, Time};
pub use stage::{Layer, Stage};
pub use view::{ElementState, GridFocus, Rect, SceneView, Step};
