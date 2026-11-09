//! The Qt half of the `EmuSen.Kit` QML module (EmuSen_Qt.md §1.4, EmuSen_Kit.md §7): cxx-qt bridges over `emusen-kit-core`.
//! Every decision is kit-core's; this crate only translates Qt's types and events to and from it.

// The unsafe invokables take the item pointers QML hands them; the contract is cxx-qt's, not a section per function.
#![allow(clippy::missing_safety_doc)]
// A world's body is eight plain numbers in one QML call (MoonriseModel.addBody); a struct would need a QML type of its own.
#![allow(clippy::too_many_arguments)]

pub mod base;
pub mod bellcache;
pub mod bells;
pub mod body;
pub mod column;
pub mod convert;
pub mod figures;
pub mod frame;
pub mod glyph;
pub mod item_model;
pub mod keyboard;
pub mod keypass;
pub mod kinds;
pub mod moonrise;
pub mod notices;
pub mod pad;
pub mod picture;
pub mod planets;
pub mod qt_keys;
pub mod router;
pub mod sky;
pub mod stars;
pub mod tokens;
pub mod world;
