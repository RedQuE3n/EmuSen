// ES-DE's text rules (EmuSen_Qt.md §4.9.1), each held on the kit's body face read from its file as a fixture.

use emusen_esde_text::{layout, whole, Align, Face, Spec, ELLIPSIS};

fn face() -> Face {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../kit-qt/fonts/Barlow-Regular.ttf");
    Face::new(std::fs::read(path).unwrap()).unwrap()
}

fn spec(text: &str, px: f64, width: f64, height: f64) -> Spec<'_> {
    Spec { text, px, own_face: true, width, height, line_spacing: 1.5, wrap: true, ellipsis: false, align: Align::Start }
}

fn words(l: &emusen_esde_text::Line) -> String {
    l.glyphs.iter().map(|g| g.text.as_str()).collect()
}

#[test]
fn qt_4_9_the_size_is_rounded_up_to_a_whole_pixel() {
    let sizes = [whole(0.04583333 * 1200.0 * 0.7), whole(25.6667), whole(14.4), whole(19.2), whole(28.8), whole(0.025 * 800.0), whole(0.2)];
    assert_eq!(sizes, [39, 26, 15, 20, 29, 20, 1]);
    assert_eq!(whole(0.02916667 * 1200.0), 35, "35.000004 px is 35, as ES-DE draws Art Book Next's release date");
    let a = layout(&face(), &spec("MENU", 14.4, 0.0, 0.0));
    let b = layout(&face(), &spec("MENU", 15.0, 0.0, 0.0));
    assert_eq!(a, b, "ES-DE's own face at 14.4 px is drawn at 15");
    let mut theirs = spec("MENU", 14.4, 0.0, 0.0);
    theirs.own_face = false;
    assert_eq!(layout(&face(), &theirs).size, 14.4, "a theme's face is drawn at its size");
}

#[test]
fn qt_4_9_each_pen_is_the_sum_of_the_shaped_advances_rounded() {
    let f = face();
    let pieces = f.shape("SCREENSAVER", 39.0, true);
    let l = layout(&f, &spec("SCREENSAVER", 39.0, 0.0, 0.0));
    let mut pen = 0;
    for (g, p) in l.lines[0].glyphs.iter().zip(&pieces) {
        assert_eq!(g.x, pen);
        pen += p.advance;
    }
    assert_eq!(l.width, pen);
    // Each advance is the shaped one rounded: the face's at 10000 px scaled down, to the nearest pixel.
    let fine = f.shape("SCREENSAVER", 10000.0, true);
    for (p, q) in pieces.iter().zip(&fine) {
        assert_eq!(p.advance, (f64::from(q.advance) * 39.0 / 10000.0).round() as i32, "{}", p.text);
    }
    // The face's kerning is applied: A before V stands closer than A alone advances, where the face kerns the pair.
    let alone = f.shape("A", 39.0, true)[0].advance;
    let paired = f.shape("AV", 39.0, true)[0].advance;
    assert!(paired <= alone, "A–V kerned: {paired} against {alone}");
}

#[test]
fn qt_4_9_es_de_s_own_face_rounds_a_space_down() {
    let f = face();
    // The space's advance in 1/10000 em, from the face at 10000 px.
    let em = f64::from(f.shape("a a", 10000.0, false)[1].advance) / 10000.0;
    for size in [20.0, 26.0, 29.0, 39.0] {
        let exact = em * size;
        assert_eq!(f.shape("a a", size, false)[1].advance, exact.round() as i32, "a theme's face rounds at {size}");
        assert_eq!(f.shape("a a", size, true)[1].advance, exact.floor() as i32, "ES-DE's own rounds down at {size}");
        assert_eq!(f.shape("a a", size, true)[0].advance, f.shape("a a", size, false)[0].advance, "a letter rounds in both");
    }
}

#[test]
fn qt_4_9_words_break_at_spaces_within_the_width() {
    let text = "The Super Nintendo Entertainment System is a 16-bit home video game console developed by Nintendo";
    let l = layout(&face(), &spec(text, 29.0, 300.0, 0.0));
    assert!(l.lines.len() > 2);
    for line in &l.lines {
        assert!(line.width <= 300, "{} is {} wide", words(line), line.width);
        assert!(!words(line).starts_with(' '));
    }
    let joined: Vec<String> = l.lines.iter().map(|l| words(l).trim_end().to_owned()).collect();
    assert_eq!(joined.join(" "), text);
}

#[test]
fn qt_4_9_the_pitch_is_the_whole_size_times_line_spacing() {
    let mut s = spec("a b c d", 28.8, 1.0, 0.0);
    s.line_spacing = 1.1;
    let l = layout(&face(), &s);
    assert!((l.pitch - 31.9).abs() < 1e-9);
    assert_eq!(l.lines.iter().map(|l| l.y).collect::<Vec<_>>(), [0, 32, 64, 96]);
}

#[test]
fn qt_4_9_a_box_of_fixed_height_shows_whole_lines_only() {
    let mut s = spec("a b c d e f g h i", 29.0, 1.0, 216.0);
    s.line_spacing = 1.1;
    s.ellipsis = true;
    let l = layout(&face(), &s);
    assert_eq!(l.lines.len(), 6, "216 px holds 6.77 pitches of 31.9");
    assert!(l.lines.iter().all(|l| !words(l).contains(ELLIPSIS)));
}

#[test]
fn qt_4_9_a_box_one_line_high_cuts_the_line_with_an_ellipsis() {
    let mut s = spec("Aurora Drift is a game invented for\nthe tests", 20.0, 120.0, 20.0);
    s.ellipsis = true;
    let l = layout(&face(), &s);
    assert_eq!(l.lines.len(), 1);
    assert!(words(&l.lines[0]).ends_with(ELLIPSIS));
    assert!(l.lines[0].width <= 120);
    s.text = "Short";
    assert_eq!(words(&layout(&face(), &s).lines[0]), "Short", "a line that fits is left whole");
}

#[test]
fn qt_4_9_a_line_stands_by_its_alignment_at_whole_pixels() {
    let mut s = spec("MENU", 20.0, 101.0, 0.0);
    s.wrap = false;
    let start = layout(&face(), &s).lines[0].clone();
    s.align = Align::End;
    assert_eq!(layout(&face(), &s).lines[0].x, 101 - start.width);
    s.align = Align::Center;
    assert_eq!(layout(&face(), &s).lines[0].x, (f64::from(101 - start.width) / 2.0).round() as i32);
}

// The kit's body face's advances, from HarfBuzz 14.1.0's `hb_shape` at a scale of 65536 to the pixel on 2026-11-10, each
// rounded half up from its exact value (EmuSen_Qt.md §4.12.1), and in ES-DE's own face a space rounded down. AVATAR
// carries the face's A-V and V-A kerning; COLLECTION at 14.4 px has I and N at 3.4992 and 9.4896, which 64ths of a pixel
// make 3.5 and 9.5 and round up.
const ADVANCES: [(&str, f64, &[i32], &[i32]); 6] = [
    ("Options", 26.0, &[15, 14, 8, 7, 14, 14, 12], &[15, 14, 8, 7, 14, 14, 12]),
    ("AVATAR", 39.0, &[21, 20, 20, 17, 23, 23], &[21, 20, 20, 17, 23, 23]),
    ("View Media", 9.216, &[5, 2, 5, 7, 2, 6, 5, 5, 2, 5], &[5, 2, 5, 7, 1, 6, 5, 5, 2, 5]),
    ("Favorites", 21.6, &[10, 11, 10, 11, 8, 5, 7, 11, 10], &[10, 11, 10, 11, 8, 5, 7, 11, 10]),
    ("Screensaver", 14.4, &[8, 7, 5, 7, 7, 8, 7, 7, 7, 7, 5], &[8, 7, 5, 7, 7, 8, 7, 7, 7, 7, 5]),
    ("COLLECTION", 14.4, &[8, 9, 8, 8, 8, 8, 8, 3, 9, 9], &[8, 9, 8, 8, 8, 8, 8, 3, 9, 9]),
];

#[test]
fn qt_4_12_each_advance_is_its_exact_value_rounded() {
    let f = face();
    for (text, size, whole, own) in ADVANCES {
        let theirs: Vec<i32> = f.shape(text, size, false).iter().map(|p| p.advance).collect();
        assert_eq!(theirs, whole, "{text} at {size} px");
        let mine: Vec<i32> = f.shape(text, size, true).iter().map(|p| p.advance).collect();
        assert_eq!(mine, own, "{text} at {size} px in ES-DE's own face");
    }
}

#[test]
fn qt_4_9_the_rules_shape_with_harfrust_alone() {
    let manifest = include_str!("../Cargo.toml");
    let table = manifest.split("\n[").find(|t| t.starts_with("dependencies]")).unwrap_or_default();
    let names: Vec<&str> = table.lines().skip(1).filter(|l| !l.starts_with('#')).filter_map(|l| l.split_once('=')).map(|(n, _)| n.trim()).collect();
    assert_eq!(names, ["harfrust"], "the shaping approved 2026-11-11 (R-20261111-74), and through it read-fonts (EmuSen_Qt.md §4.9.4)");
}
