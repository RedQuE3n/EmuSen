//! `Type`, the size a theme's words are drawn at (EmuSen_Qt.md §3.2, EmuSen_Kit.md §7.121): a design's size at the player's
//! text scale, never under the kit's legibility floor for the face the words are drawn in; the kit's rule, never written here.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element = "Type"]
        #[qml_singleton]
        type ThemeType = super::ThemeTypeRust;
    }

    extern "RustQt" {
        /// The size of words a design sets at `base`, at `textScale` (`Tokens.textScale`, so a binding follows it).
        #[qinvokable]
        fn size(self: &ThemeType, base: f64, text_scale: f64) -> f64;
        /// `size` for words drawn in `family` at `weight` (a look's `Tokens.faceText` and its weight), whose capitals are held to the floor.
        #[qinvokable]
        #[cxx_name = "sizeIn"]
        fn size_in(self: &ThemeType, base: f64, text_scale: f64, family: &QString, weight: i32) -> f64;
    }
}

#[derive(Default)]
pub struct ThemeTypeRust {}

impl qobject::ThemeType {
    pub fn size(&self, base: f64, text_scale: f64) -> f64 {
        emusen_kit_qt::type_size::legible(base, text_scale)
    }

    pub fn size_in(&self, base: f64, text_scale: f64, family: &qobject::QString, weight: i32) -> f64 {
        emusen_kit_qt::type_size::legible_in(base, text_scale, &family.to_string(), weight)
    }
}
