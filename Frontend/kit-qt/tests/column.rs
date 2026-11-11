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
