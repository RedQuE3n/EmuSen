//! `Tokens`: the QML singleton with one property per token; `form`, `variant`, `textScale` and `reducedMotion` switch the column (EmuSen_Kit.md §7.2).

use core::pin::Pin;

use cxx_qt::CxxQtType;

use cxx_qt_lib::{QColor, QString};
use emusen_kit_core::motion::MotionToken as M;
use emusen_kit_core::tokens::{self as core_tokens, ColourToken as C, FaceRole, WeightRole};

use crate::column::Column;
use crate::convert::qcolor;
use crate::kinds::{self, Form, Variant};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qcolor.h");
        type QColor = cxx_qt_lib::QColor;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("bundled_faces.h");
        fn bundled_families() -> QStringList;
        fn platform_family() -> QString;
        include!("emusen-kit-qt/src/kinds.cxxqt.h");
        #[namespace = "Forms"]
        type Form = crate::kinds::ffi::Form;
        #[namespace = "Variants"]
        type Variant = crate::kinds::ffi::Variant;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(Form, form)]
        #[qproperty(Variant, variant)]
        #[qproperty(f64, text_scale)]
        /// A text size by the word the setting stores; setting it sets `textScale` to that size's scale (EmuSen_Kit.md §7.47).
        #[qproperty(QString, text_size)]
        #[qproperty(bool, reduced_motion)]
        #[qproperty(QColor, surface, READ, NOTIFY)]
        #[qproperty(QColor, surface_raised, READ, NOTIFY)]
        #[qproperty(QColor, input_surface, READ, NOTIFY)]
        #[qproperty(QColor, void_fill, READ, NOTIFY)]
        #[qproperty(QColor, scrim, READ, NOTIFY)]
        #[qproperty(QColor, caption_surface, READ, NOTIFY)]
        #[qproperty(QColor, border, READ, NOTIFY)]
        #[qproperty(QColor, rule, READ, NOTIFY)]
        #[qproperty(QColor, text, READ, NOTIFY)]
        #[qproperty(QColor, text_strong, READ, NOTIFY)]
        #[qproperty(QColor, muted, READ, NOTIFY)]
        #[qproperty(QColor, accent, READ, NOTIFY)]
        #[qproperty(QColor, over_accent, READ, NOTIFY)]
        #[qproperty(QColor, success, READ, NOTIFY)]
        #[qproperty(QColor, attention, READ, NOTIFY)]
        #[qproperty(QColor, error, READ, NOTIFY)]
        #[qproperty(QColor, over_art, READ, NOTIFY)]
        #[qproperty(QColor, over_art_scrim, READ, NOTIFY)]
        #[qproperty(f64, font_display, READ, NOTIFY)]
        #[qproperty(f64, font_title, READ, NOTIFY)]
        #[qproperty(f64, font_strong, READ, NOTIFY)]
        #[qproperty(f64, font_row, READ, NOTIFY)]
        #[qproperty(f64, font_body, READ, NOTIFY)]
        #[qproperty(f64, font_caption, READ, NOTIFY)]
        #[qproperty(f64, font_glyph, READ, NOTIFY)]
        #[qproperty(f64, header_size, READ, NOTIFY)]
        #[qproperty(f64, base, READ, NOTIFY)]
        #[qproperty(f64, row_height, READ, NOTIFY)]
        #[qproperty(f64, control_height, READ, NOTIFY)]
        #[qproperty(f64, pane_gutter, READ, NOTIFY)]
        #[qproperty(f64, pane_gap, READ, NOTIFY)]
        #[qproperty(f64, card_width, READ, NOTIFY)]
        #[qproperty(f64, title_row, READ, NOTIFY)]
        #[qproperty(f64, help_bar, READ, NOTIFY)]
        #[qproperty(f64, tile_width, READ, NOTIFY)]
        #[qproperty(f64, tile_height, READ, NOTIFY)]
        #[qproperty(f64, tile_gap, READ, NOTIFY)]
        #[qproperty(f64, row_radius, READ, NOTIFY)]
        #[qproperty(f64, tile_radius, READ, NOTIFY)]
        #[qproperty(f64, safe_area_x, READ, NOTIFY)]
        #[qproperty(f64, safe_area_y, READ, NOTIFY)]
        #[qproperty(f64, ring_thickness, READ, NOTIFY)]
        #[qproperty(f64, ring_offset, READ, NOTIFY)]
        #[qproperty(f64, bar_width, READ, NOTIFY)]
        #[qproperty(f64, tile_scale, READ, NOTIFY)]
        #[qproperty(f64, hover_border, READ, NOTIFY)]
        #[qproperty(i32, instant, READ, NOTIFY)]
        #[qproperty(i32, quick, READ, NOTIFY)]
        #[qproperty(i32, enter, READ, NOTIFY)]
        #[qproperty(i32, exit, READ, NOTIFY)]
        #[qproperty(i32, pan, READ, NOTIFY)]
        #[qproperty(i32, reel, READ, NOTIFY)]
        #[qproperty(i32, fade, READ, NOTIFY)]
        #[qproperty(i32, poster, READ, NOTIFY)]
        #[qproperty(i32, quick_fade, READ, NOTIFY)]
        #[qproperty(i32, enter_fade, READ, NOTIFY)]
        #[qproperty(i32, exit_fade, READ, NOTIFY)]
        #[qproperty(i32, pan_fade, READ, NOTIFY)]
        #[qproperty(i32, reel_fade, READ, NOTIFY)]
        #[qproperty(i32, poster_fade, READ, NOTIFY)]
        #[qproperty(i32, instant_curve, READ, NOTIFY)]
        #[qproperty(i32, quick_curve, READ, NOTIFY)]
        #[qproperty(i32, enter_curve, READ, NOTIFY)]
        #[qproperty(i32, exit_curve, READ, NOTIFY)]
        #[qproperty(i32, pan_curve, READ, NOTIFY)]
        #[qproperty(i32, reel_curve, READ, NOTIFY)]
        #[qproperty(i32, fade_curve, READ, NOTIFY)]
        #[qproperty(i32, poster_curve, READ, NOTIFY)]
        #[qproperty(i32, breath_delay, READ, NOTIFY)]
        #[qproperty(i32, breath_period, READ, NOTIFY)]
        #[qproperty(f64, breath_low, READ, NOTIFY)]
        #[qproperty(bool, breathes, READ, NOTIFY)]
        #[qproperty(QColor, shadow_color, READ, NOTIFY)]
        #[qproperty(f64, shadow_opacity, READ, NOTIFY)]
        #[qproperty(f64, shadow_offset_x, READ, NOTIFY)]
        #[qproperty(f64, shadow_offset_y, READ, NOTIFY)]
        #[qproperty(f64, shadow_blur, READ, NOTIFY)]
        #[qproperty(QColor, ring_gap_color, READ, NOTIFY)]
        #[qproperty(QColor, glow_color, READ, NOTIFY)]
        #[qproperty(f64, glow_blur, READ, NOTIFY)]
        #[qproperty(f64, glow_spread, READ, NOTIFY)]
        #[qproperty(f64, button_radius, READ, NOTIFY)]
        #[qproperty(QColor, corona_color, READ, NOTIFY)]
        #[qproperty(f64, corona_rim, READ, NOTIFY)]
        #[qproperty(QColor, corona_glow_color, READ, NOTIFY)]
        #[qproperty(f64, corona_glow_blur, READ, NOTIFY)]
        #[qproperty(f64, corona_glow_spread, READ, NOTIFY)]
        #[qproperty(QColor, corona_halo_color, READ, NOTIFY)]
        #[qproperty(f64, corona_halo_blur, READ, NOTIFY)]
        #[qproperty(f64, corona_halo_spread, READ, NOTIFY)]
        #[qproperty(bool, ring_gapped, READ, NOTIFY)]
        #[qproperty(QString, face_text, READ, NOTIFY)]
        #[qproperty(QString, face_label, READ, NOTIFY)]
        #[qproperty(QString, face_interface, READ, NOTIFY)]
        #[qproperty(QString, face_code, READ, NOTIFY)]
        #[qproperty(i32, weight_title, READ, NOTIFY)]
        #[qproperty(i32, weight_sentence, READ, NOTIFY)]
        #[qproperty(i32, weight_wordmark, READ, NOTIFY)]
        type Tokens = super::TokensRust;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        /// The big screen's safe area at a screen's width: 5% of it, rounded to whole units.
        #[qinvokable]
        fn safe_area_of_width(self: &Tokens, width: f64) -> f64;
        /// The same at a screen's height.
        #[qinvokable]
        fn safe_area_of_height(self: &Tokens, height: f64) -> f64;
    }

    impl cxx_qt::Initialize for Tokens {}
}

pub struct TokensRust {
    form: Form,
    variant: Variant,
    text_scale: f64,
    text_size: QString,
    reduced_motion: bool,
    surface: QColor,
    surface_raised: QColor,
    input_surface: QColor,
    void_fill: QColor,
    scrim: QColor,
    caption_surface: QColor,
    border: QColor,
    rule: QColor,
    text: QColor,
    text_strong: QColor,
    muted: QColor,
    accent: QColor,
    over_accent: QColor,
    success: QColor,
    attention: QColor,
    error: QColor,
    over_art: QColor,
    over_art_scrim: QColor,
    font_display: f64,
    font_title: f64,
    font_strong: f64,
    font_row: f64,
    font_body: f64,
    font_caption: f64,
    font_glyph: f64,
    header_size: f64,
    base: f64,
    row_height: f64,
    control_height: f64,
    pane_gutter: f64,
    pane_gap: f64,
    card_width: f64,
    title_row: f64,
    help_bar: f64,
    tile_width: f64,
    tile_height: f64,
    tile_gap: f64,
    row_radius: f64,
    tile_radius: f64,
    safe_area_x: f64,
    safe_area_y: f64,
    ring_thickness: f64,
    ring_offset: f64,
    bar_width: f64,
    tile_scale: f64,
    hover_border: f64,
    shadow_color: QColor,
    shadow_opacity: f64,
    shadow_offset_x: f64,
    shadow_offset_y: f64,
    shadow_blur: f64,
    ring_gap_color: QColor,
    glow_color: QColor,
    glow_blur: f64,
    glow_spread: f64,
    button_radius: f64,
    corona_color: QColor,
    corona_rim: f64,
    corona_glow_color: QColor,
    corona_glow_blur: f64,
    corona_glow_spread: f64,
    corona_halo_color: QColor,
    corona_halo_blur: f64,
    corona_halo_spread: f64,
    ring_gapped: bool,
    face_text: QString,
    face_label: QString,
    face_interface: QString,
    face_code: QString,
    weight_title: i32,
    weight_sentence: i32,
    weight_wordmark: i32,
    instant: i32,
    quick: i32,
    enter: i32,
    exit: i32,
    pan: i32,
    reel: i32,
    fade: i32,
    poster: i32,
    quick_fade: i32,
    enter_fade: i32,
    exit_fade: i32,
    pan_fade: i32,
    reel_fade: i32,
    poster_fade: i32,
    instant_curve: i32,
    quick_curve: i32,
    enter_curve: i32,
    exit_curve: i32,
    pan_curve: i32,
    reel_curve: i32,
    fade_curve: i32,
    poster_curve: i32,
    breath_delay: i32,
    breath_period: i32,
    breath_low: f64,
    breathes: bool,
    // A theme's overrides per variant, a `Tokens` property and its value as theme.json writes it; set by the host on the theme's
    // engine and, for the chosen theme, on the program's (decided by the tester, 2026-11-13).
    overrides: Vec<(core_tokens::Variant, String, String)>,
}

impl Default for TokensRust {
    fn default() -> Self {
        let mut tokens = TokensRust {
            form: Form::BigScreen,
            variant: Variant::Dark,
            text_scale: 1.0,
            text_size: QString::default(),
            reduced_motion: false,
            surface: QColor::default(),
            surface_raised: QColor::default(),
            input_surface: QColor::default(),
            void_fill: QColor::default(),
            scrim: QColor::default(),
            caption_surface: QColor::default(),
            border: QColor::default(),
            rule: QColor::default(),
            text: QColor::default(),
            text_strong: QColor::default(),
            muted: QColor::default(),
            accent: QColor::default(),
            over_accent: QColor::default(),
            success: QColor::default(),
            attention: QColor::default(),
            error: QColor::default(),
            over_art: QColor::default(),
            over_art_scrim: QColor::default(),
            font_display: 0.0,
            font_title: 0.0,
            font_strong: 0.0,
            font_row: 0.0,
            font_body: 0.0,
            font_caption: 0.0,
            font_glyph: 0.0,
            header_size: 0.0,
            base: 0.0,
            row_height: 0.0,
            control_height: 0.0,
            pane_gutter: 0.0,
            pane_gap: 0.0,
            card_width: 0.0,
            title_row: 0.0,
            help_bar: 0.0,
            tile_width: 0.0,
            tile_height: 0.0,
            tile_gap: 0.0,
            row_radius: 0.0,
            tile_radius: 0.0,
            safe_area_x: 0.0,
            safe_area_y: 0.0,
            ring_thickness: 0.0,
            ring_offset: 0.0,
            bar_width: 0.0,
            tile_scale: 1.0,
            hover_border: 0.0,
            shadow_color: QColor::default(),
            shadow_opacity: 0.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_blur: 0.0,
            ring_gap_color: QColor::default(),
            glow_color: QColor::default(),
            glow_blur: 0.0,
            glow_spread: 0.0,
            button_radius: 0.0,
            corona_color: QColor::default(),
            corona_rim: 0.0,
            corona_glow_color: QColor::default(),
            corona_glow_blur: 0.0,
            corona_glow_spread: 0.0,
            corona_halo_color: QColor::default(),
            corona_halo_blur: 0.0,
            corona_halo_spread: 0.0,
            ring_gapped: false,
            face_text: QString::default(),
            face_label: QString::default(),
            face_interface: QString::default(),
            face_code: QString::default(),
            weight_title: 0,
            weight_sentence: 0,
            weight_wordmark: 0,
            instant: 0,
            quick: 0,
            enter: 0,
            exit: 0,
            pan: 0,
            reel: 0,
            fade: 0,
            poster: 0,
            quick_fade: 0,
            enter_fade: 0,
            exit_fade: 0,
            pan_fade: 0,
            reel_fade: 0,
            poster_fade: 0,
            instant_curve: 0,
            quick_curve: 0,
            enter_curve: 0,
            exit_curve: 0,
            pan_curve: 0,
            reel_curve: 0,
            fade_curve: 0,
            poster_curve: 0,
            breath_delay: 0,
            breath_period: 0,
            breath_low: 1.0,
            breathes: true,
            overrides: Vec::new(),
        };
        tokens.apply(&Column::compute(kinds::token_form(tokens.form), kinds::token_variant(tokens.variant), 1.0, false, &crate::type_size::caps));
        tokens
    }
}

impl TokensRust {
    fn apply(&mut self, c: &Column) {
        self.surface = qcolor(c.colour(C::Surface));
        self.surface_raised = qcolor(c.colour(C::SurfaceRaised));
        self.input_surface = qcolor(c.colour(C::InputSurface));
        self.void_fill = qcolor(c.colour(C::Void));
        self.scrim = qcolor(c.colour(C::Scrim));
        self.caption_surface = qcolor(c.colour(C::CaptionSurface));
        self.border = qcolor(c.colour(C::Border));
        self.rule = qcolor(c.colour(C::Rule));
        self.text = qcolor(c.colour(C::Text));
        self.text_strong = qcolor(c.colour(C::TextStrong));
        self.muted = qcolor(c.colour(C::Muted));
        self.accent = qcolor(c.colour(C::Accent));
        self.over_accent = qcolor(c.colour(C::OnAccent));
        self.success = qcolor(c.colour(C::Success));
        self.attention = qcolor(c.colour(C::Attention));
        self.error = qcolor(c.colour(C::Error));
        self.over_art = qcolor(c.colour(C::OnArt));
        self.over_art_scrim = qcolor(c.colour(C::OnArtScrim));
        self.font_display = c.type_scale.display;
        self.font_title = c.type_scale.title;
        self.font_strong = c.type_scale.strong;
        self.font_row = c.type_scale.row;
        self.font_body = c.type_scale.body;
        self.font_caption = c.type_scale.caption;
        self.font_glyph = c.type_scale.glyph;
        self.header_size = c.header;
        self.base = c.spacing.base;
        self.row_height = c.spacing.row_height;
        self.control_height = c.spacing.control_height;
        self.pane_gutter = c.spacing.pane_gutter;
        self.pane_gap = c.spacing.pane_gap;
        self.card_width = c.spacing.card_width;
        self.title_row = c.spacing.title_row;
        self.help_bar = c.spacing.help_bar;
        self.tile_width = c.spacing.tile_width;
        self.tile_height = c.spacing.tile_height;
        self.tile_gap = c.spacing.tile_gap;
        self.row_radius = c.spacing.row_radius;
        self.tile_radius = c.spacing.tile_radius;
        self.safe_area_x = c.spacing.safe_area.left;
        self.safe_area_y = c.spacing.safe_area.top;
        self.ring_thickness = c.focus.ring_thickness;
        self.ring_offset = c.focus.ring_offset;
        self.bar_width = c.focus.bar_width;
        self.tile_scale = c.focus.tile_scale;
        self.hover_border = c.focus.hover_border;
        self.shadow_color = qcolor(c.shadow.colour);
        self.shadow_opacity = c.shadow.opacity;
        self.shadow_offset_x = c.shadow.offset_x;
        self.shadow_offset_y = c.shadow.offset_y;
        self.shadow_blur = c.shadow.blur;
        self.ring_gap_color = qcolor(c.focus.gap_colour);
        self.glow_color = qcolor(c.focus.glow_colour);
        self.glow_blur = c.focus.glow_blur;
        self.glow_spread = c.focus.glow_spread;
        self.button_radius = c.spacing.button_radius;
        self.corona_color = qcolor(c.corona_colours[0]);
        self.corona_rim = c.corona.rim;
        self.corona_glow_color = qcolor(c.corona_colours[1]);
        self.corona_glow_blur = c.corona.glow_blur;
        self.corona_glow_spread = c.corona.glow_spread;
        self.corona_halo_color = qcolor(c.corona_colours[2]);
        self.corona_halo_blur = c.corona.halo_blur;
        self.corona_halo_spread = c.corona.halo_spread;
        self.ring_gapped = c.focus.gapped;
        let face = |role| c.face(role).map_or_else(qobject::platform_family, QString::from);
        self.face_text = face(FaceRole::Text);
        self.face_label = face(FaceRole::Label);
        self.face_interface = face(FaceRole::Interface);
        self.face_code = face(FaceRole::Code);
        self.weight_title = c.weight(WeightRole::Title);
        self.weight_sentence = c.weight(WeightRole::Sentence);
        self.weight_wordmark = c.weight(WeightRole::Wordmark);
        self.instant = c.duration(M::Instant);
        self.quick = c.duration(M::Quick);
        self.enter = c.duration(M::Enter);
        self.exit = c.duration(M::Exit);
        self.pan = c.duration(M::Pan);
        self.reel = c.duration(M::Reel);
        self.fade = c.fade(M::Fade);
        self.poster = c.duration(M::Poster);
        self.quick_fade = c.fade(M::Quick);
        self.enter_fade = c.fade(M::Enter);
        self.exit_fade = c.fade(M::Exit);
        self.pan_fade = c.fade(M::Pan);
        self.reel_fade = c.fade(M::Reel);
        self.poster_fade = c.fade(M::Poster);
        self.instant_curve = c.curve(M::Instant);
        self.quick_curve = c.curve(M::Quick);
        self.enter_curve = c.curve(M::Enter);
        self.exit_curve = c.curve(M::Exit);
        self.pan_curve = c.curve(M::Pan);
        self.reel_curve = c.curve(M::Reel);
        self.fade_curve = c.curve(M::Fade);
        self.poster_curve = c.curve(M::Poster);
        self.breath_delay = c.breath_delay;
        self.breath_period = c.breath_period;
        self.breath_low = c.breath_low;
        self.breathes = c.breathes;
    }
}

/// The families of the faces in the kit's resources, each file put in the application's font database the first time.
pub fn bundled_families() -> Vec<String> {
    qobject::bundled_families().iter().map(ToString::to_string).collect()
}

/// The family the application's font asks for, the face of a text that names none: the interface's and the codes' in the kit.
pub fn platform_family() -> String {
    qobject::platform_family().to_string()
}

impl cxx_qt::Initialize for qobject::Tokens {
    fn initialize(mut self: Pin<&mut Self>) {
        bundled_families();
        self.as_mut().on_form_changed(|this| this.refresh()).release();
        self.as_mut().on_variant_changed(|this| this.refresh()).release();
        self.as_mut().on_text_scale_changed(|this| this.refresh()).release();
        self.as_mut()
            .on_text_size_changed(|this| {
                let scale = core_tokens::text_scale_of(&this.text_size().to_string());
                this.set_text_scale(scale);
            })
            .release();
        self.as_mut().on_reduced_motion_changed(|this| this.refresh()).release();
    }
}

impl qobject::Tokens {
    pub fn safe_area_of_width(&self, width: f64) -> f64 {
        core_tokens::safe_area_of(width, width).left
    }

    pub fn safe_area_of_height(&self, height: f64) -> f64 {
        core_tokens::safe_area_of(height, height).top
    }

    /// A theme's overrides, each for one variant: a `Tokens` property and its value as `theme.json` writes it, made into the
    /// column by `Column::themed`, where a number or switch is the base the text scale and reduced motion act on as on the
    /// kit's. The host sets them on the theme's engine and the chosen theme's on the program's, already held to the registry
    /// and the contrast floors by theme-check (EmuSen_Kit.md §7.2, §7.55; EmuSen_Qt.md §3.5.1).
    pub fn set_overrides(mut self: Pin<&mut Self>, overrides: Vec<(core_tokens::Variant, String, String)>) {
        let given: Vec<(String, String)> = overrides.iter().map(|(_, name, value)| (name.clone(), value.clone())).collect();
        for unread in Column::unread(kinds::token_form(self.form), &given) {
            eprintln!("EmuSen.Kit: a theme's token override is not read and the kit's value stands: {unread}");
        }
        self.as_mut().rust_mut().overrides = overrides;
        self.refresh();
    }

    fn refresh(mut self: Pin<&mut Self>) {
        let variant = kinds::token_variant(self.variant);
        let mine: Vec<(String, String)> =
            self.overrides.iter().filter(|(v, _, _)| *v == variant).map(|(_, name, value)| (name.clone(), value.clone())).collect();
        // The rows' sizes are held with the capitals of the face they draw in: the look's interface face where it sets one.
        let face = mine.iter().find(|(name, _)| name == "faceInterface").map(|(_, family)| family.clone()).unwrap_or_default();
        let caps = |size: f64| crate::type_size::caps_in(&face, 400, size);
        let column = Column::themed(kinds::token_form(self.form), variant, self.text_scale, self.reduced_motion, &mine, &caps);
        self.as_mut().rust_mut().apply(&column);
        self.announce();
    }

    fn announce(mut self: Pin<&mut Self>) {
        self.as_mut().surface_changed();
        self.as_mut().surface_raised_changed();
        self.as_mut().input_surface_changed();
        self.as_mut().void_fill_changed();
        self.as_mut().scrim_changed();
        self.as_mut().caption_surface_changed();
        self.as_mut().border_changed();
        self.as_mut().rule_changed();
        self.as_mut().text_changed();
        self.as_mut().text_strong_changed();
        self.as_mut().muted_changed();
        self.as_mut().accent_changed();
        self.as_mut().over_accent_changed();
        self.as_mut().success_changed();
        self.as_mut().attention_changed();
        self.as_mut().error_changed();
        self.as_mut().over_art_changed();
        self.as_mut().over_art_scrim_changed();
        self.as_mut().font_display_changed();
        self.as_mut().font_title_changed();
        self.as_mut().font_strong_changed();
        self.as_mut().font_row_changed();
        self.as_mut().font_body_changed();
        self.as_mut().font_caption_changed();
        self.as_mut().font_glyph_changed();
        self.as_mut().header_size_changed();
        self.as_mut().base_changed();
        self.as_mut().row_height_changed();
        self.as_mut().control_height_changed();
        self.as_mut().pane_gutter_changed();
        self.as_mut().pane_gap_changed();
        self.as_mut().card_width_changed();
        self.as_mut().title_row_changed();
        self.as_mut().help_bar_changed();
        self.as_mut().tile_width_changed();
        self.as_mut().tile_height_changed();
        self.as_mut().tile_gap_changed();
        self.as_mut().row_radius_changed();
        self.as_mut().tile_radius_changed();
        self.as_mut().safe_area_x_changed();
        self.as_mut().safe_area_y_changed();
        self.as_mut().ring_thickness_changed();
        self.as_mut().ring_offset_changed();
        self.as_mut().bar_width_changed();
        self.as_mut().tile_scale_changed();
        self.as_mut().hover_border_changed();
        self.as_mut().shadow_color_changed();
        self.as_mut().shadow_opacity_changed();
        self.as_mut().shadow_offset_x_changed();
        self.as_mut().shadow_offset_y_changed();
        self.as_mut().shadow_blur_changed();
        self.as_mut().ring_gap_color_changed();
        self.as_mut().glow_color_changed();
        self.as_mut().glow_blur_changed();
        self.as_mut().glow_spread_changed();
        self.as_mut().button_radius_changed();
        self.as_mut().corona_color_changed();
        self.as_mut().corona_rim_changed();
        self.as_mut().corona_glow_color_changed();
        self.as_mut().corona_glow_blur_changed();
        self.as_mut().corona_glow_spread_changed();
        self.as_mut().corona_halo_color_changed();
        self.as_mut().corona_halo_blur_changed();
        self.as_mut().corona_halo_spread_changed();
        self.as_mut().ring_gapped_changed();
        self.as_mut().face_text_changed();
        self.as_mut().face_label_changed();
        self.as_mut().face_interface_changed();
        self.as_mut().face_code_changed();
        self.as_mut().weight_title_changed();
        self.as_mut().weight_sentence_changed();
        self.as_mut().weight_wordmark_changed();
        self.as_mut().instant_changed();
        self.as_mut().quick_changed();
        self.as_mut().enter_changed();
        self.as_mut().exit_changed();
        self.as_mut().pan_changed();
        self.as_mut().reel_changed();
        self.as_mut().fade_changed();
        self.as_mut().poster_changed();
        self.as_mut().quick_fade_changed();
        self.as_mut().enter_fade_changed();
        self.as_mut().exit_fade_changed();
        self.as_mut().pan_fade_changed();
        self.as_mut().reel_fade_changed();
        self.as_mut().poster_fade_changed();
        self.as_mut().instant_curve_changed();
        self.as_mut().quick_curve_changed();
        self.as_mut().enter_curve_changed();
        self.as_mut().exit_curve_changed();
        self.as_mut().pan_curve_changed();
        self.as_mut().reel_curve_changed();
        self.as_mut().fade_curve_changed();
        self.as_mut().poster_curve_changed();
        self.as_mut().breath_delay_changed();
        self.as_mut().breath_period_changed();
        self.as_mut().breath_low_changed();
        self.as_mut().breathes_changed();
    }
}
