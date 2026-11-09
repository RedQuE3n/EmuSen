//! The interface's tokens: colour in two variants, type, spacing and focus in two forms, and motion (EmuSen_Interface.md §5, EmuSen_Kit.md §1).

mod colour;
mod metrics;
mod palette;

pub use colour::Colour;
pub use metrics::{
    FocusMarks, Insets, MAX_TEXT_SCALE, SAFE_AREA_FRACTION, Spacing, TEXT_SIZES, TypeScale, clamp_text_scale, focus_marks, header_size,
    safe_area_of, spacing, text_scale_of, type_scale,
};
pub use palette::{ColourToken, ContrastPair, GRAPHIC_FLOOR, TEXT_FLOOR, colour, contrast_pairs, on_art_over};

/// Which rendering of the interface a set of metrics is for (LunaP's `LunaForm`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Form {
    /// A desktop window, in device pixels at 1x.
    Desktop,
    /// The big screen and the handheld, in design units: a pixel of a screen 800 high (1280 × 800), scaled with the screen's height.
    BigScreen,
}

/// Which column of the colour tokens is read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Variant {
    Dark,
    Light,
}

impl Form {
    pub const ALL: [Form; 2] = [Form::Desktop, Form::BigScreen];
}

impl Variant {
    pub const ALL: [Variant; 2] = [Variant::Dark, Variant::Light];
}
