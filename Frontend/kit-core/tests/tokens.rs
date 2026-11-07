// The tokens held to their sources: EmuSen_Interface.md §5.1 to §5.5 and LunaP §206, §214.

use std::time::Duration;

use emusen_kit_core::motion::{
    BREATH_DELAY, BREATH_LOW, BREATH_PERIOD, Curve, MotionToken, REDUCED_FADE, breath_opacity, breathes,
};
use emusen_kit_core::tokens::{
    Colour, ColourToken, Form, GRAPHIC_FLOOR, Insets, MAX_TEXT_SCALE, TEXT_FLOOR, Variant, clamp_text_scale,
    colour, contrast_pairs, focus_marks, header_size, on_art_over, safe_area_of, spacing, type_scale,
};

fn close(a: f64, b: f64, within: f64) -> bool {
    (a - b).abs() <= within
}

#[test]
fn interface_5_1_the_dark_column() {
    use ColourToken::*;
    let expected = [
        (Surface, "#1B1B1E"),
        (SurfaceRaised, "#26262B"),
        (InputSurface, "#0E0E10"),
        (Void, "#000000"),
        (Scrim, "#8C000000"),
        (CaptionSurface, "#AA000000"),
        (Border, "#707070"),
        (Rule, "#333337"),
        (Text, "#D4D4D4"),
        (TextStrong, "#EEEEF0"),
        (Muted, "#8D8D91"),
        (Accent, "#4C9AE8"),
        (OnAccent, "#0E0E10"),
        (Success, "#35A064"),
        (Attention, "#DAA520"),
        (Error, "#D37070"),
        (OnArt, "#FFFFFF"),
        (OnArtScrim, "#B8000000"),
    ];
    assert_eq!(expected.len(), ColourToken::ALL.len());
    for (token, hex) in expected {
        assert_eq!(colour(Variant::Dark, token).to_hex(), hex, "{token:?}");
    }
}

#[test]
fn interface_5_1_the_light_column() {
    use ColourToken::*;
    let expected = [
        (Surface, "#F3F3F3"),
        (SurfaceRaised, "#FFFFFF"),
        (InputSurface, "#FFFFFF"),
        (Void, "#000000"),
        (Scrim, "#8C000000"),
        (CaptionSurface, "#AA000000"),
        (Border, "#8C8C8C"),
        (Rule, "#D6D6D6"),
        (Text, "#1F1F1F"),
        (TextStrong, "#000000"),
        (Muted, "#5F5F5F"),
        (Accent, "#005FB8"),
        (OnAccent, "#FFFFFF"),
        (Success, "#1B6E3C"),
        (Attention, "#7A5B00"),
        (Error, "#B3261E"),
        (OnArt, "#FFFFFF"),
        (OnArtScrim, "#B8000000"),
    ];
    for (token, hex) in expected {
        assert_eq!(colour(Variant::Light, token).to_hex(), hex, "{token:?}");
    }
}

#[test]
fn interface_5_1_the_scrims_are_55_67_and_72_percent_black() {
    assert!(close(colour(Variant::Dark, ColourToken::Scrim).alpha(), 0.55, 0.005));
    assert!(close(colour(Variant::Dark, ColourToken::CaptionSurface).alpha(), 0.67, 0.005));
    assert!(close(colour(Variant::Dark, ColourToken::OnArtScrim).alpha(), 0.72, 0.005));
}

#[test]
fn interface_5_1_every_pair_of_tokens_clears_its_contrast_floor() {
    let pairs = contrast_pairs();
    assert_eq!(pairs.len(), 50, "50 pairs in all");
    for pair in pairs {
        assert!(pair.clears(), "{:?} {:?} on {:?}: {:.2} under {}", pair.variant, pair.fore, pair.back, pair.ratio(), pair.floor);
    }
    assert_eq!(TEXT_FLOOR, 4.5);
    assert_eq!(GRAPHIC_FLOOR, 3.0);
}

#[test]
fn interface_5_1_the_2026_11_01_corrections_measure_as_recorded() {
    let raised = colour(Variant::Dark, ColourToken::SurfaceRaised);
    let ratio = |token| colour(Variant::Dark, token).contrast(raised);
    assert!(close(ratio(ColourToken::Muted), 4.55, 0.02));
    assert!(close(ratio(ColourToken::Border), 3.04, 0.02));
    assert!(close(ratio(ColourToken::Success), 4.56, 0.02));
    assert!(close(ratio(ColourToken::Error), 4.53, 0.02));
    assert!(close(Colour::hex("#86868A").contrast(raised), 4.15, 0.02), "the value the correction replaced");
}

#[test]
fn interface_5_1_on_art_over_a_white_picture_under_the_scrim_measures_9_3() {
    let foot = on_art_over(Colour::rgb(255, 255, 255));
    assert!(close(colour(Variant::Dark, ColourToken::OnArt).contrast(foot), 9.3, 0.05));
}

#[test]
fn colour_hex_round_trips_and_hsl_matches_the_css_wheel() {
    assert_eq!(Colour::hex("#4C9AE8"), Colour::rgb(0x4C, 0x9A, 0xE8));
    assert_eq!(Colour::hex("#8C000000"), Colour::rgba(0, 0, 0, 0x8C));
    assert_eq!(Colour::hsl(0.0, 1.0, 0.5, 1.0), Colour::rgb(255, 0, 0));
    assert_eq!(Colour::hsl(120.0, 1.0, 0.5, 1.0), Colour::rgb(0, 255, 0));
    assert_eq!(Colour::hsl(240.0, 1.0, 0.5, 1.0), Colour::rgb(0, 0, 255));
    assert_eq!(Colour::hsl(-120.0, 1.0, 0.5, 0.5), Colour::rgba(0, 0, 255, 128), "a hue wraps; alpha is a share");
    assert!(close(Colour::rgb(255, 255, 255).contrast(Colour::rgb(0, 0, 0)), 21.0, 0.001));
}

#[test]
fn interface_5_2_the_big_screen_type_scale_in_units() {
    let t = type_scale(Form::BigScreen, 1.0);
    assert_eq!((t.display, t.title, t.strong, t.row, t.body, t.caption, t.glyph), (44.0, 32.0, 26.0, 26.0, 22.0, 20.0, 22.0));
}

#[test]
fn interface_5_2_the_desktop_type_scale_in_pixels() {
    let t = type_scale(Form::Desktop, 1.0);
    assert_eq!((t.display, t.title, t.strong, t.row, t.body, t.caption, t.glyph), (24.0, 16.0, 14.0, 13.0, 13.0, 11.0, 13.0));
}

#[test]
fn lunap_214_1_the_text_scale_is_clamped_to_1_3_and_rounds_to_the_half_unit() {
    assert_eq!(MAX_TEXT_SCALE, 1.3);
    assert_eq!(clamp_text_scale(0.5), 1.0);
    assert_eq!(clamp_text_scale(2.0), 1.3);
    assert_eq!(clamp_text_scale(f64::NAN), 1.0);
    let t = type_scale(Form::BigScreen, 1.3);
    assert_eq!((t.display, t.title, t.body, t.caption), (57.0, 41.5, 28.5, 26.0));
    let d = type_scale(Form::Desktop, 1.3);
    assert_eq!((d.display, d.title, d.strong, d.body, d.caption), (31.0, 21.0, 18.0, 17.0, 14.5));
    assert_eq!(type_scale(Form::Desktop, 1.25).caption, 14.0, "13.75 rounds away from zero to 14");
}

#[test]
fn interface_5_3_the_big_screen_spacing() {
    let s = spacing(Form::BigScreen);
    assert_eq!(s.base, 4.0);
    assert_eq!(s.row_height, 54.0);
    assert_eq!(s.control_height, 44.0);
    assert_eq!((s.pane_gutter, s.pane_gap), (24.0, 32.0));
    assert_eq!(s.safe_area, Insets { left: 64.0, top: 40.0, right: 64.0, bottom: 40.0 });
    assert_eq!((s.card_width, s.title_row, s.help_bar), (512.0, 100.0, 58.0));
    assert_eq!((s.tile_width, s.tile_height, s.tile_gap), (180.0, 240.0, 24.0));
    assert_eq!((s.row_radius, s.tile_radius), (4.0, 8.0));
}

#[test]
fn interface_5_3_the_desktop_spacing() {
    let s = spacing(Form::Desktop);
    assert_eq!((s.row_height, s.control_height), (24.0, 28.0));
    assert_eq!((s.pane_gutter, s.pane_gap), (24.0, 16.0));
    assert_eq!(s.safe_area, Insets::default());
    assert_eq!((s.title_row, s.help_bar), (28.0, 24.0));
    assert_eq!((s.tile_width, s.tile_height, s.tile_gap), (100.0, 133.0, 16.0));
    assert_eq!((s.row_radius, s.tile_radius), (4.0, 6.0));
}

#[test]
fn interface_5_3_the_safe_area_is_5_percent_of_each_dimension() {
    assert_eq!(safe_area_of(1280.0, 800.0), Insets::symmetric(64.0, 40.0));
    assert_eq!(safe_area_of(1920.0, 1080.0), Insets::symmetric(96.0, 54.0));
    assert_eq!(safe_area_of(3840.0, 2160.0), Insets::symmetric(192.0, 108.0));
}

#[test]
fn lunap_206_2_the_header_size_is_28_on_the_big_screen_and_strong_on_the_desktop() {
    assert_eq!(header_size(Form::BigScreen, 1.0), 28.0);
    assert_eq!(header_size(Form::Desktop, 1.0), 14.0);
    assert_eq!(header_size(Form::Desktop, 1.3), 18.0);
}

#[test]
fn interface_5_4_the_focus_marks() {
    for form in Form::ALL {
        let f = focus_marks(form);
        assert_eq!((f.ring_thickness, f.ring_offset, f.bar_width, f.tile_scale, f.hover_border), (3.0, 4.0, 4.0, 1.06, 1.0));
    }
}

#[test]
fn interface_5_5_the_motion_tokens_durations_and_curves() {
    let ms = |t: MotionToken| t.nominal().as_millis();
    assert_eq!(ms(MotionToken::Instant), 0);
    assert_eq!(ms(MotionToken::Quick), 120);
    assert_eq!(ms(MotionToken::Enter), 200);
    assert_eq!(ms(MotionToken::Exit), 160);
    assert_eq!(ms(MotionToken::Pan), 320);
    assert_eq!(ms(MotionToken::Reel), 400);
    assert_eq!(ms(MotionToken::Fade), 150);
    assert_eq!(ms(MotionToken::Poster), 1500);
    assert_eq!(MotionToken::Quick.curve(), Curve::QuadraticOut);
    assert_eq!(MotionToken::Enter.curve(), Curve::QuadraticOut);
    assert_eq!(MotionToken::Exit.curve(), Curve::QuadraticIn);
    assert_eq!(MotionToken::Pan.curve(), Curve::CubicOut);
    assert_eq!(MotionToken::Reel.curve(), Curve::QuadraticOut);
    assert_eq!(MotionToken::Fade.curve(), Curve::Linear);
    assert_eq!(MotionToken::Poster.curve(), Curve::QuadraticOut);
}

#[test]
fn interface_5_5_reduced_motion_zeroes_movement_and_shortens_fades_to_120_ms() {
    assert_eq!(REDUCED_FADE, Duration::from_millis(120));
    for token in MotionToken::ALL {
        assert_eq!(token.duration(true), Duration::ZERO, "{token:?}");
        assert_eq!(token.duration(false), token.nominal(), "{token:?}");
        let fade = token.fade_duration(true);
        assert_eq!(fade, if token == MotionToken::Instant { Duration::ZERO } else { REDUCED_FADE }, "{token:?}");
    }
    assert!(breathes(false));
    assert!(!breathes(true));
}

#[test]
fn interface_5_4_the_idle_ring_breathes_to_70_percent_over_1_6_s_after_2_s() {
    assert_eq!(BREATH_DELAY, Duration::from_secs(2));
    assert_eq!(BREATH_PERIOD, Duration::from_millis(1600));
    assert_eq!(BREATH_LOW, 0.7);
    assert_eq!(breath_opacity(Duration::from_millis(1999), false), 1.0);
    assert!(close(breath_opacity(Duration::from_millis(2800), false), 0.7, 1e-9), "the bottom of the first breath");
    assert!(close(breath_opacity(Duration::from_millis(3600), false), 1.0, 1e-9), "back to full");
    assert_eq!(breath_opacity(Duration::from_millis(2800), true), 1.0, "still under reduced motion");
}

#[test]
fn motion_curves_evaluate_as_their_formulas() {
    assert_eq!(Curve::Linear.at(0.25), 0.25);
    assert_eq!(Curve::QuadraticIn.at(0.5), 0.25);
    assert_eq!(Curve::QuadraticOut.at(0.5), 0.75);
    assert_eq!(Curve::CubicOut.at(0.5), 0.875);
    assert_eq!(Curve::CubicOut.at(2.0), 1.0, "clamped");
    assert_eq!(Curve::QuadraticOut.at(-1.0), 0.0);
}
