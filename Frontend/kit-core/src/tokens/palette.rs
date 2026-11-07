use super::{Colour, Variant};

/// The colour tokens of EmuSen_Interface.md §5.1, one name per use.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColourToken {
    /// The screen and the desktop window.
    Surface,
    /// Cards, tiles, a focused row's bar.
    SurfaceRaised,
    /// Fields and tables.
    InputSurface,
    /// Behind a frame.
    Void,
    /// Over a frame or a theme under a card; never a blur.
    Scrim,
    /// A caption over art.
    CaptionSurface,
    /// Edges and dividers, 3:1 or better.
    Border,
    /// The line under a row (decoration, not held to 3:1).
    Rule,
    /// Body and rows.
    Text,
    /// Titles and the focused row.
    TextStrong,
    /// Status lines and facts.
    Muted,
    /// The ring, the focus bar, filled tracks, checks.
    Accent,
    /// On the accent.
    OnAccent,
    Success,
    /// A status needing the player; `Info` was its older name.
    Attention,
    Error,
    /// Words over a picture.
    OnArt,
    /// The foot of the gradient under OnArt words: black at 72% (the head is clear).
    OnArtScrim,
}

impl ColourToken {
    pub const ALL: [ColourToken; 18] = [
        ColourToken::Surface,
        ColourToken::SurfaceRaised,
        ColourToken::InputSurface,
        ColourToken::Void,
        ColourToken::Scrim,
        ColourToken::CaptionSurface,
        ColourToken::Border,
        ColourToken::Rule,
        ColourToken::Text,
        ColourToken::TextStrong,
        ColourToken::Muted,
        ColourToken::Accent,
        ColourToken::OnAccent,
        ColourToken::Success,
        ColourToken::Attention,
        ColourToken::Error,
        ColourToken::OnArt,
        ColourToken::OnArtScrim,
    ];

    /// The three surfaces words and controls are measured against.
    pub const SURFACES: [ColourToken; 3] = [ColourToken::Surface, ColourToken::SurfaceRaised, ColourToken::InputSurface];

    /// The tokens held to the words' floor against every surface.
    pub const WORDS: [ColourToken; 6] = [
        ColourToken::Text,
        ColourToken::TextStrong,
        ColourToken::Muted,
        ColourToken::Attention,
        ColourToken::Success,
        ColourToken::Error,
    ];

    /// The tokens held to the graphics' floor against every surface.
    pub const GRAPHICS: [ColourToken; 2] = [ColourToken::Accent, ColourToken::Border];
}

/// WCAG 2.1 1.4.3: words against their surface.
pub const TEXT_FLOOR: f64 = 4.5;

/// WCAG 2.1 1.4.11: a control's edge, fill or ring against its surface.
pub const GRAPHIC_FLOOR: f64 = 3.0;

/// A token's colour in a variant: the table of EmuSen_Interface.md §5.1 with its 2026-11-01 corrections.
pub const fn colour(variant: Variant, token: ColourToken) -> Colour {
    use ColourToken::*;
    match (variant, token) {
        (Variant::Dark, Surface) => Colour::hex("#1B1B1E"),
        (Variant::Dark, SurfaceRaised) => Colour::hex("#26262B"),
        (Variant::Dark, InputSurface) => Colour::hex("#0E0E10"),
        (Variant::Dark, Border) => Colour::hex("#707070"),
        (Variant::Dark, Rule) => Colour::hex("#333337"),
        (Variant::Dark, Text) => Colour::hex("#D4D4D4"),
        (Variant::Dark, TextStrong) => Colour::hex("#EEEEF0"),
        (Variant::Dark, Muted) => Colour::hex("#8D8D91"),
        (Variant::Dark, Accent) => Colour::hex("#4C9AE8"),
        (Variant::Dark, OnAccent) => Colour::hex("#0E0E10"),
        (Variant::Dark, Success) => Colour::hex("#35A064"),
        (Variant::Dark, Attention) => Colour::hex("#DAA520"),
        (Variant::Dark, Error) => Colour::hex("#D37070"),
        (Variant::Light, Surface) => Colour::hex("#F3F3F3"),
        (Variant::Light, SurfaceRaised) => Colour::hex("#FFFFFF"),
        (Variant::Light, InputSurface) => Colour::hex("#FFFFFF"),
        (Variant::Light, Border) => Colour::hex("#8C8C8C"),
        (Variant::Light, Rule) => Colour::hex("#D6D6D6"),
        (Variant::Light, Text) => Colour::hex("#1F1F1F"),
        (Variant::Light, TextStrong) => Colour::hex("#000000"),
        (Variant::Light, Muted) => Colour::hex("#5F5F5F"),
        (Variant::Light, Accent) => Colour::hex("#005FB8"),
        (Variant::Light, OnAccent) => Colour::hex("#FFFFFF"),
        (Variant::Light, Success) => Colour::hex("#1B6E3C"),
        (Variant::Light, Attention) => Colour::hex("#7A5B00"),
        (Variant::Light, Error) => Colour::hex("#B3261E"),
        (_, Void) => Colour::hex("#000000"),
        (_, Scrim) => Colour::hex("#8C000000"),
        (_, CaptionSurface) => Colour::hex("#AA000000"),
        (_, OnArt) => Colour::hex("#FFFFFF"),
        (_, OnArtScrim) => Colour::hex("#B8000000"),
    }
}

/// One pair the palette is held to: the fore token over the back token, at a floor, in a variant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContrastPair {
    pub variant: Variant,
    pub fore: ColourToken,
    pub back: ColourToken,
    pub floor: f64,
}

impl ContrastPair {
    /// The pair's measured contrast.
    pub fn ratio(self) -> f64 {
        colour(self.variant, self.fore).contrast(colour(self.variant, self.back))
    }

    /// Whether the pair clears its floor.
    pub fn clears(self) -> bool {
        self.ratio() >= self.floor
    }
}

/// The 50 pairs of EmuSen_Interface.md §5.1: words at 4.5:1 and graphics at 3:1 on each surface, and OnAccent on Accent, in both variants.
pub fn contrast_pairs() -> Vec<ContrastPair> {
    let mut pairs = Vec::with_capacity(50);
    for variant in Variant::ALL {
        for back in ColourToken::SURFACES {
            for fore in ColourToken::WORDS {
                pairs.push(ContrastPair { variant, fore, back, floor: TEXT_FLOOR });
            }
            for fore in ColourToken::GRAPHICS {
                pairs.push(ContrastPair { variant, fore, back, floor: GRAPHIC_FLOOR });
            }
        }
        pairs.push(ContrastPair { variant, fore: ColourToken::OnAccent, back: ColourToken::Accent, floor: TEXT_FLOOR });
    }
    pairs
}

/// The surface OnArt words sit on at the gradient's foot: the picture under the scrim.
pub fn on_art_over(picture: Colour) -> Colour {
    colour(Variant::Dark, ColourToken::OnArtScrim).over(picture)
}
