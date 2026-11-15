//! A look's type size never under the legibility floor, for the face Qt Quick's Text draws in by default (EmuSen_Kit.md §7.121):
//! kit-core's `legible_text` with the face's capitals measured by Qt.

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qfont.h");
        type QFont = cxx_qt_lib::QFont;
        include!("text_width.h");
        fn cap_height(font: &QFont, size: f64) -> f64;
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
