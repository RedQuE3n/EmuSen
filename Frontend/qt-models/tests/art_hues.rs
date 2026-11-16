//! EmuSen_Qt.md §3.4.8: a picture's hues, most prominent first, for a theme's corona and backdrop (R-20261115-27, -28).

use emusen_kit_core::tokens::Colour;
use emusen_qt_models::art_hues::{hues, hues_of_file, DISTINCT};

fn pixels(parts: &[(Colour, usize)]) -> Vec<Colour> {
    parts.iter().flat_map(|(c, n)| std::iter::repeat_n(*c, *n)).collect()
}

fn near(a: f64, b: f64) -> bool {
    emusen_kit_core::world::wrap_hue(a - b).abs() < 1.0
}

#[test]
fn qt_3_4_8_one_hue_is_one_hue() {
    let teal = Colour::hsl(168.0, 0.7, 0.45, 1.0);
    let found = hues(&pixels(&[(teal, 400)]), 3);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(near(found[0], teal.to_hsl().0), "{found:?}");
}

#[test]
fn qt_3_4_8_three_hues_come_most_prominent_first() {
    let (violet, teal, blue) = (Colour::hsl(251.0, 0.85, 0.62, 1.0), Colour::hsl(168.0, 0.73, 0.45, 1.0), Colour::hsl(218.0, 0.89, 0.53, 1.0));
    let found = hues(&pixels(&[(teal, 100), (violet, 300), (blue, 200)]), 3);
    assert_eq!(found.len(), 3, "{found:?}");
    for (got, want) in found.iter().zip([violet, blue, teal]) {
        assert!(near(*got, want.to_hsl().0), "{found:?}");
    }
    assert_eq!(hues(&pixels(&[(teal, 100), (violet, 300), (blue, 200)]), 2).len(), 2, "no more than asked");
}

#[test]
fn qt_3_4_8_close_hues_are_one_and_a_trace_is_none() {
    let a = Colour::hsl(200.0, 0.8, 0.5, 1.0);
    let b = Colour::hsl(200.0 + DISTINCT / 2.0, 0.8, 0.5, 1.0);
    let red = Colour::hsl(0.0, 0.8, 0.5, 1.0);
    let found = hues(&pixels(&[(a, 300), (b, 250), (red, 10)]), 3);
    assert_eq!(found.len(), 1, "two hues closer than {DISTINCT} degrees are one, and 2% of red is none: {found:?}");
}

#[test]
fn qt_3_4_8_greys_black_white_and_nothing_have_no_hue() {
    let greys = pixels(&[(Colour::hex("#808080"), 100), (Colour::hex("#000000"), 100), (Colour::hex("#FFFFFF"), 100), (Colour::hex("#8A8090"), 100)]);
    assert!(hues(&greys, 3).is_empty());
    assert!(hues(&[], 3).is_empty());
    assert!(hues_of_file("/nonexistent/cover.png", 3).is_empty(), "a picture Qt cannot read has none");
}
