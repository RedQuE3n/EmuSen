//! One column of tokens as the `Tokens` singleton shows it: every value computed by kit-core for a form, a variant, a text scale and reduced motion (EmuSen_Kit.md §7.2).

use emusen_kit_core::motion::{breathes, Curve, MotionToken, BREATH_DELAY, BREATH_LOW, BREATH_PERIOD};
use emusen_kit_core::tokens::{self, Colour, ColourToken, FocusMarks, Form, Spacing, TypeScale, Variant};

/// QEasingCurve::Type's number for a kit-core curve, so QML binds `easing.type` to a token.
pub fn easing_type(curve: Curve) -> i32 {
    match curve {
        Curve::Linear => 0,
        Curve::QuadraticIn => 1,
        Curve::QuadraticOut => 2,
        Curve::CubicOut => 6,
    }
}

fn millis(duration: std::time::Duration) -> i32 {
    i32::try_from(duration.as_millis()).unwrap_or(i32::MAX)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    pub colours: [Colour; 18],
    pub type_scale: TypeScale,
    pub header: f64,
    pub spacing: Spacing,
    pub focus: FocusMarks,
    pub durations: [i32; 8],
    pub fades: [i32; 8],
    pub curves: [i32; 8],
    pub breath_delay: i32,
    pub breath_period: i32,
    pub breath_low: f64,
    pub breathes: bool,
}

impl Column {
    pub fn compute(form: Form, variant: Variant, text_scale: f64, reduced: bool) -> Column {
        let colours = ColourToken::ALL.map(|token| tokens::colour(variant, token));
        let durations = MotionToken::ALL.map(|token| millis(token.duration(reduced)));
        let fades = MotionToken::ALL.map(|token| millis(token.fade_duration(reduced)));
        let curves = MotionToken::ALL.map(|token| easing_type(token.curve()));
        Column {
            colours,
            type_scale: tokens::type_scale(form, text_scale),
            header: tokens::header_size(form, text_scale),
            spacing: tokens::spacing(form),
            focus: tokens::focus_marks(form),
            durations,
            fades,
            curves,
            breath_delay: millis(BREATH_DELAY),
            breath_period: millis(BREATH_PERIOD),
            breath_low: BREATH_LOW,
            breathes: breathes(reduced),
        }
    }

    fn slot(token: MotionToken) -> usize {
        MotionToken::ALL.iter().position(|t| *t == token).unwrap_or(0)
    }

    pub fn colour(&self, token: ColourToken) -> Colour {
        self.colours[ColourToken::ALL.iter().position(|t| *t == token).unwrap_or(0)]
    }

    /// The column with a theme's colours in place of the kit's, for the variant it was computed for.
    pub fn overridden(mut self, overrides: &[(ColourToken, Colour)]) -> Column {
        for (token, colour) in overrides {
            if let Some(i) = ColourToken::ALL.iter().position(|t| t == token) {
                self.colours[i] = *colour;
            }
        }
        self
    }

    pub fn duration(&self, token: MotionToken) -> i32 {
        self.durations[Self::slot(token)]
    }

    pub fn fade(&self, token: MotionToken) -> i32 {
        self.fades[Self::slot(token)]
    }

    pub fn curve(&self, token: MotionToken) -> i32 {
        self.curves[Self::slot(token)]
    }
}
