// EmuSen_Kit.md §7.22: the docked keyboard's layouts, boxes, walk and typing, held to Mistress's search keyboard (GamesSearch-1280x800.spec).

use emusen_kit_core::keyboard::{Keyboard, Layout, Outcome, DOCKED, LETTERS};

fn letters() -> Layout {
    Layout::parse(LETTERS).expect("the letters")
}

#[test]
fn the_boxes_are_mistress_s_on_a_1280_band() {
    let layout = letters();
    let at = |row, column| {
        let (x, y, w, h) = DOCKED.key_box(&layout, row, column);
        (x + 640.0, y + 398.0, w, h)
    };
    assert_eq!(at(0, 0), (244.0, 415.0, 72.0, 48.0), "1");
    assert_eq!(at(1, 0), (244.0, 471.0, 72.0, 48.0), "q");
    assert_eq!(at(3, 0), (84.0, 583.0, 152.0, 48.0), "Shift");
    assert_eq!(at(3, 11), (1044.0, 583.0, 152.0, 48.0), "Erase");
    assert_eq!(at(4, 0), (284.0, 639.0, 152.0, 48.0), "Symbols");
    assert_eq!(at(4, 1), (444.0, 639.0, 392.0, 48.0), "Space");
    assert_eq!(DOCKED.height(&layout), 304.0, "the band 398 to 702");
}

#[test]
fn the_labels_follow_the_layout_and_the_shift() {
    let mut keyboard = Keyboard::search();
    assert_eq!(keyboard.label(1, 0), "q");
    assert_eq!(keyboard.label(3, 11), "⌫");
    assert_eq!(keyboard.label(4, 0), "Symbols", "Next names the layout it shows");
    keyboard.point(3, 0);
    assert_eq!(keyboard.press(), Outcome::Moved);
    assert_eq!(keyboard.label(1, 0), "Q");
    keyboard.point(1, 0);
    keyboard.press();
    assert_eq!(keyboard.text(), "Q");
    assert_eq!(keyboard.label(1, 0), "q", "the shift lets go after one letter");
    keyboard.point(4, 0);
    keyboard.press();
    assert_eq!(keyboard.layout().name, "Symbols");
    assert_eq!(keyboard.label(4, 0), "Letters");
}

#[test]
fn the_walk_wraps_across_and_finds_the_nearest_key_up_and_down() {
    let mut keyboard = Keyboard::search();
    assert_eq!(keyboard.current(), (0, 0));
    assert_eq!(keyboard.move_by(-1, 0), Outcome::Moved);
    assert_eq!(keyboard.current(), (0, 9), "Left from 1 wraps to 0");
    assert_eq!(keyboard.move_by(0, -1), Outcome::Nothing, "the top stops");
    keyboard.point(2, 5);
    keyboard.move_by(0, 1);
    assert_eq!(keyboard.current(), (3, 6), "h down to n, the key under it");
    keyboard.move_by(0, 1);
    assert_eq!(keyboard.current(), (4, 1), "n down to Space");
    keyboard.move_by(0, -1);
    assert_eq!(keyboard.current(), (3, 5), "Space up to b, the first of b and n as near");
    keyboard.point(3, 0);
    keyboard.move_by(0, -1);
    assert_eq!(keyboard.current(), (2, 0), "Shift up to a");
    keyboard.point(4, 2);
    assert_eq!(keyboard.move_by(0, 1), Outcome::Nothing, "the foot stops");
}

#[test]
fn b_erases_then_closes_on_an_empty_field_and_done_keeps_the_text() {
    let mut keyboard = Keyboard::search();
    keyboard.set_text("gran", 4);
    assert_eq!(keyboard.back(), Outcome::Typed);
    assert_eq!(keyboard.text(), "gra");
    keyboard.set_text("ab", 1);
    keyboard.point(4, 1);
    assert_eq!(keyboard.press(), Outcome::Typed);
    assert_eq!((keyboard.text().as_str(), keyboard.caret()), ("a b", 2), "Space at the caret");
    keyboard.set_text("", 0);
    assert_eq!(keyboard.back(), Outcome::Closed { kept: false });
    keyboard.set_text("x", 1);
    keyboard.point(4, 2);
    assert_eq!(keyboard.press(), Outcome::Closed { kept: true });
    assert_eq!(keyboard.text(), "x");
}

#[test]
fn a_layout_reads_from_lines_and_none_from_nothing() {
    let layout = Layout::parse("Code\n1 2 3\nSpace:2 Erase").expect("a layout");
    assert_eq!(layout.name, "Code");
    assert_eq!(layout.rows.len(), 2);
    assert_eq!(layout.rows[1][0].width, 2);
    assert!(Layout::parse("Name only").is_none());
    assert!(Layout::parse("").is_none());
}
