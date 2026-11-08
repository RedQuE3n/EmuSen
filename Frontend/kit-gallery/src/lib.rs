//! The kit's gallery as a QML module, `EmuSen.Gallery`: the scene the gate runs on and the program shows (EmuSen_Kit.md §7.10).

// The kit's QML module registers through its crate's initializer; a crate that only names it in QML must still link it.
extern crate emusen_kit_qt;

/// Where the gallery's scene lives in the resource system.
pub const GALLERY_URL: &str = "qrc:/qt/qml/EmuSen/Gallery/qml/Gallery.qml";
