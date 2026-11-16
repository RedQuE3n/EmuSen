//! A face's capitals as Qt draws them, which kit-core's `legible_text` holds the Tokens column's sizes with (EmuSen_Kit.md §7.121).

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

/// The capitals' height of the application's face at `size`, as Qt draws it: the `caps_at` the kit's `Tokens` sizes are held with.
pub fn caps(size: f64) -> f64 {
    ffi::cap_height(&cxx_qt_lib::QFont::default(), size)
}

/// The capitals' height of `family` at `weight` and `size`, as Qt draws it; the application's face for an empty family.
pub fn caps_in(family: &str, weight: i32, size: f64) -> f64 {
    if family.is_empty() {
        return caps(size);
    }
    ffi::cap_height_in(&cxx_qt_lib::QString::from(family), weight, size)
}
