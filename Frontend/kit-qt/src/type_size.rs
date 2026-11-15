//! A look's type size never under the legibility floor, for the face Qt Quick's Text draws in by default (EmuSen_Kit.md §7.121):
//! kit-core's `legible_text` with the face's capitals measured by Qt.

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qfont.h");
        type QFont = cxx_qt_lib::QFont;
        include!("text_width.h");
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        fn cap_height(font: &QFont, size: f64) -> f64;
        fn cap_height_in(family: &QString, weight: i32, size: f64) -> f64;
    }
}

/// The size words of a design's `base` are drawn at, at `text_scale`, in the application's face, the one a Text with no
/// family set takes.
pub fn legible(base: f64, text_scale: f64) -> f64 {
    emusen_kit_core::tokens::legible_text(base, text_scale, caps)
}

/// The capitals' height of the application's face at `size`, as Qt draws it: the `caps_at` the kit's `Tokens` sizes are held with.
pub fn caps(size: f64) -> f64 {
    ffi::cap_height(&cxx_qt_lib::QFont::default(), size)
}

/// The size words of a design's `base` are drawn at, at `text_scale`, in `family` at `weight`: the face a look's Text sets,
/// whose capitals, not the application face's, are held to the floor.
pub fn legible_in(base: f64, text_scale: f64, family: &str, weight: i32) -> f64 {
    emusen_kit_core::tokens::legible_text(base, text_scale, |size| caps_in(family, weight, size))
}

/// The capitals' height of `family` at `weight` and `size`, as Qt draws it; the application's face for an empty family.
pub fn caps_in(family: &str, weight: i32, size: f64) -> f64 {
    if family.is_empty() {
        return caps(size);
    }
    ffi::cap_height_in(&cxx_qt_lib::QString::from(family), weight, size)
}
