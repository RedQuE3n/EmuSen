use super::{Colour, Form};

/// The type scale for one form: each role's size in that form's units (EmuSen_Interface.md §5.2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeScale {
    pub display: f64,
    pub title: f64,
    pub strong: f64,
    pub row: f64,
    pub body: f64,
    pub caption: f64,
    pub glyph: f64,
}

/// Margins, in a form's units.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Insets {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl Insets {
    pub const fn symmetric(horizontal: f64, vertical: f64) -> Insets {
        Insets { left: horizontal, top: vertical, right: horizontal, bottom: vertical }
    }
}

/// The spacing for one form, in that form's units (EmuSen_Interface.md §5.3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spacing {
    pub base: f64,
    pub row_height: f64,
    pub control_height: f64,
    pub pane_gutter: f64,
    pub pane_gap: f64,
    pub safe_area: Insets,
    pub card_width: f64,
    pub title_row: f64,
    pub help_bar: f64,
    pub tile_width: f64,
    pub tile_height: f64,
    pub tile_gap: f64,
    pub row_radius: f64,
    pub tile_radius: f64,
}

/// How the focus is marked, the same in both forms' units (EmuSen_Interface.md §5.4).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FocusMarks {
    pub ring_thickness: f64,
    pub ring_offset: f64,
    pub bar_width: f64,
    pub tile_scale: f64,
    pub hover_border: f64,
}

/// The one shadow a raised surface casts: its colour and opacity, its offset and its blur, the same in both forms' units
/// (EmuSen_Interface.md §5.4).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub colour: Colour,
    pub opacity: f64,
    pub offset_x: f64,
    pub offset_y: f64,
    pub blur: f64,
}

impl Shadow {
    /// The name the kit's `Tokens` singleton gives the shadow's colour: a colour token after the palette's (rule Q).
    pub const COLOUR_PROPERTY: &'static str = "shadowColor";
}

/// The largest text scale a player may choose.
pub const MAX_TEXT_SCALE: f64 = 1.3;

/// The text sizes a player chooses, by the word the setting stores (`AppSettings.TextSize`), and the type scale each
/// asks of the kit (EmuSen_Kit.md §7.47).
pub const TEXT_SIZES: [(&str, f64); 3] = [("default", 1.0), ("large", 1.15), ("larger", MAX_TEXT_SCALE)];

/// The type scale of a text size by its word; 1 for a word the kit does not know.
pub fn text_scale_of(size: &str) -> f64 {
    TEXT_SIZES.iter().find(|(word, _)| *word == size).map_or(1.0, |(_, scale)| *scale)
}

/// The share of each dimension kept clear at the big screen's edges.
pub const SAFE_AREA_FRACTION: f64 = 0.05;

const BIG_SCREEN_TYPE: TypeScale =
    TypeScale { display: 44.0, title: 32.0, strong: 26.0, row: 26.0, body: 22.0, caption: 20.0, glyph: 22.0 };

const DESKTOP_TYPE: TypeScale = TypeScale { display: 24.0, title: 16.0, strong: 14.0, row: 13.0, body: 13.0, caption: 11.0, glyph: 13.0 };

const BIG_SCREEN_SPACING: Spacing = Spacing {
    base: 4.0,
    row_height: 54.0,
    control_height: 44.0,
    pane_gutter: 24.0,
    pane_gap: 32.0,
    safe_area: Insets::symmetric(64.0, 40.0),
    card_width: 512.0,
    title_row: 100.0,
    help_bar: 58.0,
    tile_width: 180.0,
    tile_height: 240.0,
    tile_gap: 24.0,
    row_radius: 4.0,
    tile_radius: 8.0,
};

const DESKTOP_SPACING: Spacing = Spacing {
    base: 4.0,
    row_height: 24.0,
    control_height: 28.0,
    pane_gutter: 24.0,
    pane_gap: 16.0,
    safe_area: Insets::symmetric(0.0, 0.0),
    card_width: 512.0,
    title_row: 28.0,
    help_bar: 24.0,
    tile_width: 100.0,
    tile_height: 133.0,
    tile_gap: 16.0,
    row_radius: 4.0,
    tile_radius: 6.0,
};

const FOCUS_MARKS: FocusMarks = FocusMarks { ring_thickness: 3.0, ring_offset: 4.0, bar_width: 4.0, tile_scale: 1.06, hover_border: 1.0 };

const RAISED_SHADOW: Shadow = Shadow { colour: Colour::rgb(0, 0, 0), opacity: 0.5, offset_x: 0.0, offset_y: 8.0, blur: 24.0 };

/// A text scale as the kit keeps it: 1 to 1.3, and 1 for anything that is not a number.
pub fn clamp_text_scale(scale: f64) -> f64 {
    if scale.is_finite() { scale.clamp(1.0, MAX_TEXT_SCALE) } else { 1.0 }
}

/// A form's type sizes at text scale 1: the bases `type_scale` scales.
pub const fn base_type(form: Form) -> TypeScale {
    match form {
        Form::BigScreen => BIG_SCREEN_TYPE,
        Form::Desktop => DESKTOP_TYPE,
    }
}

/// A type size from its base at a text scale: the base times the clamped scale, rounded to the nearest half unit, halves away
/// from zero.
pub fn scaled_text(base: f64, text_scale: f64) -> f64 {
    (base * clamp_text_scale(text_scale) * 2.0).round() / 2.0
}

impl TypeScale {
    /// Every size of these bases at a text scale (`scaled_text`).
    pub fn scaled(&self, text_scale: f64) -> TypeScale {
        let r = |v: f64| scaled_text(v, text_scale);
        TypeScale {
            display: r(self.display),
            title: r(self.title),
            strong: r(self.strong),
            row: r(self.row),
            body: r(self.body),
            caption: r(self.caption),
            glyph: r(self.glyph),
        }
    }
}

/// A form's type scale at a text scale, each size rounded to the nearest half unit, halves away from zero.
pub fn type_scale(form: Form, text_scale: f64) -> TypeScale {
    base_type(form).scaled(text_scale)
}

/// A form's spacing.
pub const fn spacing(form: Form) -> Spacing {
    match form {
        Form::BigScreen => BIG_SCREEN_SPACING,
        Form::Desktop => DESKTOP_SPACING,
    }
}

/// A form's focus marks.
pub const fn focus_marks(_form: Form) -> FocusMarks {
    FOCUS_MARKS
}

/// A form's raised shadow, cast by a focused tile, a card's open edge and a page's cover.
pub const fn raised_shadow(_form: Form) -> Shadow {
    RAISED_SHADOW
}

/// A section heading's size at text scale 1: the menus' 28 on the big screen, the desktop's Strong.
pub const fn header_base(form: Form) -> f64 {
    match form {
        Form::BigScreen => 28.0,
        Form::Desktop => DESKTOP_TYPE.strong,
    }
}

/// A section heading's size from its base at a text scale: fixed on the big screen, scaled as a type size on the desktop.
pub fn header_of(form: Form, base: f64, text_scale: f64) -> f64 {
    match form {
        Form::BigScreen => base,
        Form::Desktop => scaled_text(base, text_scale),
    }
}

/// A section heading's size until the header role replaces it: the desktop's Strong, the menus' 28 on the big screen.
pub fn header_size(form: Form, text_scale: f64) -> f64 {
    header_of(form, header_base(form), text_scale)
}

/// The big screen's safe area for a screen of a size: 5% of each dimension, rounded to whole units.
pub fn safe_area_of(width: f64, height: f64) -> Insets {
    Insets::symmetric((width * SAFE_AREA_FRACTION).round(), (height * SAFE_AREA_FRACTION).round())
}
