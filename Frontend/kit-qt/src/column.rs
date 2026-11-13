//! One column of tokens as the `Tokens` singleton shows it: every value computed by kit-core for a form, a variant, a text scale and reduced motion (EmuSen_Kit.md §7.2).

use std::time::Duration;

use emusen_kit_core::motion::{breathes_if, movement, Curve, MotionToken, BREATH_DELAY, BREATH_LOW, BREATH_PERIOD};
use emusen_kit_core::tokens::{self, Colour, ColourToken, FocusMarks, Form, Shadow, Spacing, TypeScale, Variant};

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
    pub shadow: Shadow,
    pub durations: [i32; 8],
    pub fades: [i32; 8],
    pub curves: [i32; 8],
    pub breath_delay: i32,
    pub breath_period: i32,
    pub breath_low: f64,
    pub breathes: bool,
}

/// The values a column is made from before the text scale and reduced motion: kit-core's, or a theme's in their place.
struct Bases {
    type_scale: TypeScale,
    header: f64,
    spacing: Spacing,
    focus: FocusMarks,
    shadow: Shadow,
    durations: [i32; 8],
    fades: [i32; 8],
    curves: [i32; 8],
    breath_delay: i32,
    breath_period: i32,
    breath_low: f64,
    breathes: bool,
}

/// Where a number or switch token's base is held, by the kind an override of it is read as.
enum Slot<'a> {
    Number(&'a mut f64),
    Pair(&'a mut f64, &'a mut f64),
    Whole(&'a mut i32),
    Switch(&'a mut bool),
}

impl Bases {
    fn of(form: Form) -> Bases {
        let nominal = MotionToken::ALL.map(|token| millis(token.nominal()));
        Bases {
            type_scale: tokens::base_type(form),
            header: tokens::header_base(form),
            spacing: tokens::spacing(form),
            focus: tokens::focus_marks(form),
            shadow: tokens::raised_shadow(form),
            durations: nominal,
            fades: nominal,
            curves: MotionToken::ALL.map(|token| easing_type(token.curve())),
            breath_delay: millis(BREATH_DELAY),
            breath_period: millis(BREATH_PERIOD),
            breath_low: BREATH_LOW,
            breathes: true,
        }
    }

    /// The base a `Tokens` property is made from; the colours are the palette's (`ColourToken::named`).
    fn slot(&mut self, property: &str) -> Option<Slot<'_>> {
        use MotionToken as M;
        let (t, sp, f, sh) = (&mut self.type_scale, &mut self.spacing, &mut self.focus, &mut self.shadow);
        let i = Column::slot;
        Some(match property {
            "fontDisplay" => Slot::Number(&mut t.display),
            "fontTitle" => Slot::Number(&mut t.title),
            "fontStrong" => Slot::Number(&mut t.strong),
            "fontRow" => Slot::Number(&mut t.row),
            "fontBody" => Slot::Number(&mut t.body),
            "fontCaption" => Slot::Number(&mut t.caption),
            "fontGlyph" => Slot::Number(&mut t.glyph),
            "headerSize" => Slot::Number(&mut self.header),
            "base" => Slot::Number(&mut sp.base),
            "rowHeight" => Slot::Number(&mut sp.row_height),
            "controlHeight" => Slot::Number(&mut sp.control_height),
            "paneGutter" => Slot::Number(&mut sp.pane_gutter),
            "paneGap" => Slot::Number(&mut sp.pane_gap),
            "cardWidth" => Slot::Number(&mut sp.card_width),
            "titleRow" => Slot::Number(&mut sp.title_row),
            "helpBar" => Slot::Number(&mut sp.help_bar),
            "tileWidth" => Slot::Number(&mut sp.tile_width),
            "tileHeight" => Slot::Number(&mut sp.tile_height),
            "tileGap" => Slot::Number(&mut sp.tile_gap),
            "rowRadius" => Slot::Number(&mut sp.row_radius),
            "tileRadius" => Slot::Number(&mut sp.tile_radius),
            "safeAreaX" => Slot::Pair(&mut sp.safe_area.left, &mut sp.safe_area.right),
            "safeAreaY" => Slot::Pair(&mut sp.safe_area.top, &mut sp.safe_area.bottom),
            "ringThickness" => Slot::Number(&mut f.ring_thickness),
            "ringOffset" => Slot::Number(&mut f.ring_offset),
            "barWidth" => Slot::Number(&mut f.bar_width),
            "tileScale" => Slot::Number(&mut f.tile_scale),
            "hoverBorder" => Slot::Number(&mut f.hover_border),
            "shadowOpacity" => Slot::Number(&mut sh.opacity),
            "shadowOffsetX" => Slot::Number(&mut sh.offset_x),
            "shadowOffsetY" => Slot::Number(&mut sh.offset_y),
            "shadowBlur" => Slot::Number(&mut sh.blur),
            "instant" => Slot::Whole(&mut self.durations[i(M::Instant)]),
            "quick" => Slot::Whole(&mut self.durations[i(M::Quick)]),
            "enter" => Slot::Whole(&mut self.durations[i(M::Enter)]),
            "exit" => Slot::Whole(&mut self.durations[i(M::Exit)]),
            "pan" => Slot::Whole(&mut self.durations[i(M::Pan)]),
            "reel" => Slot::Whole(&mut self.durations[i(M::Reel)]),
            "poster" => Slot::Whole(&mut self.durations[i(M::Poster)]),
            "fade" => Slot::Whole(&mut self.fades[i(M::Fade)]),
            "quickFade" => Slot::Whole(&mut self.fades[i(M::Quick)]),
            "enterFade" => Slot::Whole(&mut self.fades[i(M::Enter)]),
            "exitFade" => Slot::Whole(&mut self.fades[i(M::Exit)]),
            "panFade" => Slot::Whole(&mut self.fades[i(M::Pan)]),
            "reelFade" => Slot::Whole(&mut self.fades[i(M::Reel)]),
            "posterFade" => Slot::Whole(&mut self.fades[i(M::Poster)]),
            "instantCurve" => Slot::Whole(&mut self.curves[i(M::Instant)]),
            "quickCurve" => Slot::Whole(&mut self.curves[i(M::Quick)]),
            "enterCurve" => Slot::Whole(&mut self.curves[i(M::Enter)]),
            "exitCurve" => Slot::Whole(&mut self.curves[i(M::Exit)]),
            "panCurve" => Slot::Whole(&mut self.curves[i(M::Pan)]),
            "reelCurve" => Slot::Whole(&mut self.curves[i(M::Reel)]),
            "fadeCurve" => Slot::Whole(&mut self.curves[i(M::Fade)]),
            "posterCurve" => Slot::Whole(&mut self.curves[i(M::Poster)]),
            "breathDelay" => Slot::Whole(&mut self.breath_delay),
            "breathPeriod" => Slot::Whole(&mut self.breath_period),
            "breathLow" => Slot::Number(&mut self.breath_low),
            "breathes" => Slot::Switch(&mut self.breathes),
            _ => return None,
        })
    }

    /// A theme's value in place of the base, read as the token's kind; a value not of the kind is left alone.
    fn set(&mut self, property: &str, value: &str) {
        let number = value.parse::<f64>().ok().filter(|v| v.is_finite());
        match self.slot(property) {
            Some(Slot::Number(held)) => *held = number.unwrap_or(*held),
            Some(Slot::Pair(a, b)) => {
                if let Some(v) = number {
                    (*a, *b) = (v, v);
                }
            }
            Some(Slot::Whole(held)) => *held = value.parse::<i32>().unwrap_or(*held),
            Some(Slot::Switch(held)) => *held = value.parse::<bool>().unwrap_or(*held),
            None => {}
        }
    }
}

fn length(ms: i32) -> Duration {
    Duration::from_millis(u64::try_from(ms).unwrap_or(0))
}

impl Column {
    pub fn compute(form: Form, variant: Variant, text_scale: f64, reduced: bool) -> Column {
        Column::themed(form, variant, text_scale, reduced, &[])
    }

    /// The column with a theme's overrides for this variant, each a `Tokens` property and its value as `theme.json` writes it.
    /// A number or switch becomes the base kit-core makes the token from, so the text scale and reduced motion act on it as on
    /// the kit's own; a colour replaces the palette's or the raised shadow's (EmuSen_Kit.md §7.2, §7.55; EmuSen_Qt.md §3.5.1).
    pub fn themed(form: Form, variant: Variant, text_scale: f64, reduced: bool, given: &[(String, String)]) -> Column {
        let mut b = Bases::of(form);
        for (name, value) in given {
            b.set(name, value);
        }
        let colours: Vec<(ColourToken, Colour)> =
            given.iter().filter_map(|(name, value)| Some((ColourToken::named(name)?, Colour::parse(value)?))).collect();
        let shadow = given.iter().find(|(name, _)| name == Shadow::COLOUR_PROPERTY).and_then(|(_, value)| Colour::parse(value));
        let durations = MotionToken::ALL.map(|token| millis(movement(length(b.durations[Self::slot(token)]), reduced)));
        let fades = MotionToken::ALL.map(|token| millis(token.fade_of(length(b.fades[Self::slot(token)]), reduced)));
        Column {
            colours: ColourToken::ALL.map(|token| tokens::colour(variant, token)),
            type_scale: b.type_scale.scaled(text_scale),
            header: tokens::header_of(form, b.header, text_scale),
            spacing: b.spacing,
            focus: b.focus,
            shadow: b.shadow,
            durations,
            fades,
            curves: b.curves,
            breath_delay: b.breath_delay,
            breath_period: b.breath_period,
            breath_low: b.breath_low,
            breathes: breathes_if(b.breathes, reduced),
        }
        .overridden(&colours, shadow)
    }

    fn slot(token: MotionToken) -> usize {
        MotionToken::ALL.iter().position(|t| *t == token).unwrap_or(0)
    }

    pub fn colour(&self, token: ColourToken) -> Colour {
        self.colours[ColourToken::ALL.iter().position(|t| *t == token).unwrap_or(0)]
    }

    /// The column with a theme's colours in place of the kit's, for the variant it was computed for: the palette's, and the
    /// raised shadow's colour where the theme sets its own (EmuSen_Kit.md §7.55).
    pub fn overridden(mut self, overrides: &[(ColourToken, Colour)], shadow: Option<Colour>) -> Column {
        for (token, colour) in overrides {
            if let Some(i) = ColourToken::ALL.iter().position(|t| t == token) {
                self.colours[i] = *colour;
            }
        }
        if let Some(colour) = shadow {
            self.shadow.colour = colour;
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
