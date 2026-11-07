//! The Qt half of the `EmuSen.Kit` QML module (EmuSen_Qt.md §1.4, EmuSen_Kit.md §7): cxx-qt bridges over `emusen-kit-core`.
//! Every decision is kit-core's; this crate only translates Qt's types and events to and from it.

// The unsafe invokables take the item pointers QML hands them; the contract is cxx-qt's, not a section per function.
#![allow(clippy::missing_safety_doc)]

pub mod column;
pub mod frame;
pub mod kinds;
pub mod pad;
pub mod qt_keys;
pub mod router;
pub mod tokens;
