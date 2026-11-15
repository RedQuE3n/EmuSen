//! A look's type size never under the legibility floor, for the face Qt Quick's Text draws in by default (EmuSen_Kit.md §7.120):
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
    let face = cxx_qt_lib::QFont::default();
    emusen_kit_core::tokens::legible_text(base, text_scale, |size| ffi::cap_height(&face, size))
}
