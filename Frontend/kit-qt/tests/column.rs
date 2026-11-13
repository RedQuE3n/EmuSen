// EmuSen_Kit.md §7.2: the Tokens singleton's column is kit-core's values and nothing else, in both forms, both variants, under reduced motion and a text scale.

use emusen_kit_core::motion::{Curve, MotionToken};
use emusen_kit_core::tokens::{self, ColourToken, Form, Variant};
use emusen_kit_qt::column::{easing_type, Column};

#[test]
fn kit_7_2_every_colour_is_the_variants_token() {
    for variant in Variant::ALL {
        let column = Column::compute(Form::BigScreen, variant, 1.0, false);
        for token in ColourToken::ALL {
            assert_eq!(column.colour(token), tokens::colour(variant, token), "{variant:?} {token:?}");
        }
    }
    assert_eq!(Column::compute(Form::Desktop, Variant::Dark, 1.0, false).colour(ColourToken::Accent).to_hex(), "#4C9AE8");
}

#[test]
fn kit_7_2_metrics_follow_the_form_and_the_text_scale() {
    let big = Column::compute(Form::BigScreen, Variant::Dark, 1.0, false);
    let desktop = Column::compute(Form::Desktop, Variant::Dark, 1.0, false);
    assert_eq!(big.type_scale, tokens::type_scale(Form::BigScreen, 1.0));
    assert_eq!(big.spacing, tokens::spacing(Form::BigScreen));
    assert_eq!(big.spacing.row_height, 54.0);
    assert_eq!(desktop.spacing.row_height, 24.0);
    assert_eq!(big.header, 28.0);
    assert_eq!(desktop.header, desktop.type_scale.strong);
    assert_eq!(big.focus, tokens::focus_marks(Form::BigScreen));
    assert_eq!(big.shadow, tokens::raised_shadow(Form::BigScreen));
    assert_eq!(desktop.shadow, tokens::raised_shadow(Form::Desktop));
    let scaled = Column::compute(Form::BigScreen, Variant::Dark, 1.3, false);
    assert_eq!(scaled.type_scale, tokens::type_scale(Form::BigScreen, 1.3));
    assert!(scaled.type_scale.row > big.type_scale.row);
    assert_eq!(scaled.spacing, big.spacing, "spacing does not scale with the text");
}

#[test]
fn kit_7_2_motion_is_the_tokens_in_milliseconds_and_qt_easing_numbers() {
    let column = Column::compute(Form::BigScreen, Variant::Dark, 1.0, false);
    assert_eq!(column.duration(MotionToken::Quick), 120);
    assert_eq!(column.duration(MotionToken::Pan), 320);
    assert_eq!(column.duration(MotionToken::Poster), 1500);
    assert_eq!(column.duration(MotionToken::Instant), 0);
    assert_eq!(column.fade(MotionToken::Enter), 200);
    assert_eq!(column.curve(MotionToken::Quick), 2, "QEasingCurve::OutQuad");
    assert_eq!(column.curve(MotionToken::Exit), 1, "QEasingCurve::InQuad");
    assert_eq!(column.curve(MotionToken::Pan), 6, "QEasingCurve::OutCubic");
    assert_eq!(column.curve(MotionToken::Fade), 0, "QEasingCurve::Linear");
    assert_eq!(easing_type(Curve::QuadraticOut), 2);
    assert_eq!(column.breath_delay, 2000);
    assert_eq!(column.breath_period, 1600);
    assert_eq!(column.breath_low, 0.7);
    assert!(column.breathes);
}

#[test]
fn kit_7_2_reduced_motion_zeroes_movements_and_keeps_fades_at_120() {
    let reduced = Column::compute(Form::BigScreen, Variant::Dark, 1.0, true);
    for token in MotionToken::ALL {
        assert_eq!(reduced.duration(token), 0, "{token:?}");
        assert_eq!(reduced.fade(token), if token == MotionToken::Instant { 0 } else { 120 }, "{token:?}");
    }
    assert!(!reduced.breathes);
}

#[test]
fn kit_7_55_a_theme_s_shadow_colour_replaces_the_registry_s_and_none_keeps_it() {
    let kit = Column::compute(Form::Desktop, Variant::Dark, 1.0, false);
    let own = tokens::Colour::hex("#2A1B4C");
    let accent = tokens::Colour::hex("#B4442A");
    let themed = kit.clone().overridden(&[(ColourToken::Accent, accent)], Some(own));
    assert_eq!(themed.shadow.colour, own, "the theme's own shadow colour");
    assert_eq!(themed.colour(ColourToken::Accent), accent, "the palette's override still holds");
    assert_eq!(themed.shadow.opacity, kit.shadow.opacity, "the colour alone is the theme's");
    let plain = kit.clone().overridden(&[(ColourToken::Accent, accent)], None);
    assert_eq!(plain.shadow.colour, tokens::raised_shadow(Form::Desktop).colour, "no shadowColor, the registry's");
}

fn themed(form: Form, scale: f64, reduced: bool, given: &[(&str, &str)]) -> Column {
    let given: Vec<(String, String)> = given.iter().map(|(n, v)| ((*n).to_owned(), (*v).to_owned())).collect();
    Column::themed(form, Variant::Dark, scale, reduced, &given)
}

#[test]
fn qt_3_5_1_a_theme_s_type_size_grows_with_the_text_size_as_the_kit_s_does() {
    for form in [Form::BigScreen, Form::Desktop] {
        assert_eq!(themed(form, 1.0, false, &[("fontBody", "20")]).type_scale.body, 20.0, "{form:?}");
        assert_eq!(themed(form, 1.3, false, &[("fontBody", "20")]).type_scale.body, 26.0, "{form:?}: 20 x 1.3");
        assert_eq!(themed(form, 1.15, false, &[("fontBody", "20")]).type_scale.body, 23.0, "{form:?}: 20 x 1.15");
    }
    // The desktop's header scales as a type size, the big screen's stays put, as the kit's do.
    assert_eq!(themed(Form::Desktop, 1.3, false, &[("headerSize", "20")]).header, 26.0);
    assert_eq!(themed(Form::BigScreen, 1.3, false, &[("headerSize", "30")]).header, 30.0);
    // The kit's own sizes are the same as before.
    assert_eq!(themed(Form::BigScreen, 1.3, false, &[]), Column::compute(Form::BigScreen, Variant::Dark, 1.3, false));
}

#[test]
fn qt_3_5_1_a_theme_s_durations_and_breathing_are_reduced_as_the_kit_s_are() {
    let given = [("quick", "300"), ("quickFade", "300"), ("fade", "90"), ("breathes", "true")];
    let full = themed(Form::BigScreen, 1.0, false, &given);
    assert_eq!((full.duration(MotionToken::Quick), full.fade(MotionToken::Quick), full.fade(MotionToken::Fade)), (300, 300, 90));
    assert!(full.breathes);
    let reduced = themed(Form::BigScreen, 1.0, true, &given);
    assert_eq!(reduced.duration(MotionToken::Quick), 0, "a movement is nothing under reduced motion");
    assert_eq!((reduced.fade(MotionToken::Quick), reduced.fade(MotionToken::Fade)), (120, 120), "a fade is 120 ms");
    assert!(!reduced.breathes, "a theme's ring does not breathe under reduced motion");
    assert!(!themed(Form::BigScreen, 1.0, false, &[("breathes", "false")]).breathes, "nor where the theme stops it");
}

#[test]
fn qt_3_5_1_a_token_the_kit_neither_scales_nor_reduces_stays_the_theme_s() {
    let given = [("tileRadius", "21"), ("safeAreaX", "40"), ("quickCurve", "3"), ("breathLow", "0.5"), ("shadowBlur", "9")];
    for (scale, reduced) in [(1.0, false), (1.3, true)] {
        let c = themed(Form::BigScreen, scale, reduced, &given);
        assert_eq!((c.spacing.tile_radius, c.spacing.safe_area.left, c.spacing.safe_area.right), (21.0, 40.0, 40.0));
        assert_eq!((c.curve(MotionToken::Quick), c.breath_low, c.shadow.blur), (3, 0.5, 9.0), "{scale} {reduced}");
    }
}
