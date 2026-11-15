//! `Type`, the size a theme's words are drawn at (EmuSen_Qt.md §3.2, EmuSen_Kit.md §7.121): a design's size at the player's
//! text scale, never under the kit's legibility floor for the face the words are drawn in; the kit's rule, never written here.

#[cxx_qt::bridge]
pub mod qobject {
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
    }
}

#[derive(Default)]
pub struct ThemeTypeRust {}

impl qobject::ThemeType {
    pub fn size(&self, base: f64, text_scale: f64) -> f64 {
        emusen_kit_qt::type_size::legible(base, text_scale)
    }
}
