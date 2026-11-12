//! How far a box sized by its words may move between two window sizes of one aspect ratio (decided 2026-11-11 by the
//! tester, R-20261111-112): half a pixel for each letter it shows plus one pixel, in pixels of the smaller window. ES-DE
//! 3.4.1 rounds each glyph's advance to a whole pixel (es-core/src/resources/Font.cpp, `Font::shapeText`), so a run of
//! n letters may stand up to n halves of a pixel from proportional, and places at whole pixels
//! (es-core/src/GuiComponent.cpp, `GuiComponent::getTransform`), the one pixel (EmuSen_Qt.md §4.8.5).

/// The letters a text shows: its characters, white space left out.
pub fn letters(text: &str) -> usize {
    text.chars().filter(|c| !c.is_whitespace()).count()
}

/// The most a box of `letters` letters may move, in pixels of the smaller window.
pub fn allowance(letters: usize) -> f64 {
    0.5 * letters as f64 + 1.0
}

/// Whether a box's largest edge difference, in pixels of the smaller window, is within its allowance.
pub fn within(most_px: f64, letters: usize) -> bool {
    most_px <= allowance(letters)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The figures of the ruling's table: 27 letters 14.5 px, a five-letter clock 3.5, a box with no letters 1.
    #[test]
    fn half_a_pixel_a_letter_and_one() {
        assert_eq!(letters("BACK  SELECT\tA"), 11);
        assert_eq!((allowance(27), allowance(5), allowance(0)), (14.5, 3.5, 1.0));
        assert!(within(14.5, 27) && within(3.33, 27) && within(0.0, 0));
        assert!(!within(14.67, 27), "Art Book Next's system help at 1280 x 720 against 1920 x 1080");
        assert!(!within(1.01, 0) && !within(4.0, 5));
    }
}
