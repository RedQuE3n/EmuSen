//! The kit's gallery as a QML module, `EmuSen.Gallery`: the scene the gate runs on and the program shows (EmuSen_Kit.md §7.10).

// The bridge's one unsafe call, QImage::save, takes a C string; its contract is cxx's, whose generated wrapper carries no doc.
#![allow(clippy::missing_safety_doc)]

// The kit's QML module registers through its crate's initializer; a crate that only names it in QML must still link it.
extern crate emusen_kit_qt;
// Endymion's pads for the gallery's own walk (EmuSen_Kit.md §7.15).
extern crate emusen_qt_endymion;

/// Where the gallery's scene lives in the resource system.
pub const GALLERY_URL: &str = "qrc:/qt/qml/EmuSen/Gallery/qml/Gallery.qml";

pub mod grab;
