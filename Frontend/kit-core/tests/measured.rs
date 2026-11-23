// EmuSen_Kit.md §7.122: the rows' walk over rows of their own heights, with no geometry of its own.

use emusen_kit_core::input::UiButton;
use emusen_kit_core::settings::{MeasuredWalk, Row, RowKind, RowOutcome};

fn walk(heights: &[f64], window: f64, gap: f64) -> MeasuredWalk {
    let mut walk = MeasuredWalk::new();
    walk.set_rows(heights.iter().map(|_| Row::of(RowKind::Steps)).collect(), false);
    walk.set_heights(heights.to_vec());
    walk.set_gap(gap);
    walk.set_window(window);
    walk
}

#[test]
fn rows_of_different_heights_move_the_window_only_when_the_focus_leaves_it() {
    let mut w = walk(&[84.0, 120.0, 84.0, 160.0, 84.0], 300.0, 10.0);
    assert_eq!((w.current(), w.first()), (0, 0));
    assert_eq!(w.press(UiButton::Down), RowOutcome::Moved(1));
    assert_eq!(w.first(), 0, "84 + 10 + 120 fits in 300");
    assert_eq!(w.press(UiButton::Down), RowOutcome::Moved(2));
    assert_eq!(w.first(), 1, "rows 0 to 2 take 308, so the window leaves row 0");
    assert_eq!(w.offset(), 94.0, "the window starts one row and its gap down");
    assert_eq!(w.press(UiButton::Down), RowOutcome::Moved(3));
    assert_eq!(w.first(), 2, "84 + 10 + 160 fits, 120 more does not");
    assert_eq!(w.offset(), 94.0 + 130.0);
    assert_eq!(w.press(UiButton::Down), RowOutcome::Moved(4));
    assert_eq!(w.first(), 3, "rows 2 to 4 take 84 + 160 + 84 + 20 = 348, more than 300");
    assert_eq!(w.press(UiButton::Up), RowOutcome::Moved(3));
    assert_eq!(w.first(), 3, "inside the window the focus moves alone");
    assert_eq!(w.press(UiButton::Up), RowOutcome::Moved(2));
    assert_eq!(w.first(), 2, "above it, the window follows a row at a time");
}

#[test]
fn a_tall_row_alone_fills_the_window_and_the_last_rows_leave_no_space_below() {
    let mut w = walk(&[50.0, 400.0, 50.0, 50.0], 300.0, 0.0);
    w.press(UiButton::Down);
    assert_eq!(w.first(), 1, "a row taller than the window is shown from its top");
    w.press(UiButton::Down);
    assert_eq!(w.first(), 2, "the row under it takes the window's top");
    assert!(w.scroll_to(0), "the wheel moves the window");
    assert_eq!(w.current(), 2, "and leaves the focus");
    assert!(!w.scroll_to(9) || w.first() == 2, "past the end the window holds at the last full view");
    assert_eq!(w.first(), 2, "rows 2 and 3 take 100: starting any later would show empty space");
}

#[test]
fn a_height_measured_late_moves_the_window_and_none_shows_every_row() {
    let mut w = walk(&[100.0, 100.0, 100.0], 0.0, 0.0);
    w.press(UiButton::Down);
    w.press(UiButton::Down);
    assert_eq!(w.first(), 0, "no window: every row is shown from the first");
    w.set_window(150.0);
    assert_eq!(w.first(), 2, "a window given later is kept around the focus");
    w.set_height(2, 40.0);
    assert_eq!(w.first(), 1, "a shorter last row lets the row above show too");
    w.set_rows(vec![Row::of(RowKind::Action); 3], false);
    assert_eq!((w.current(), w.first()), (0, 0), "new rows start at the top");
}

#[test]
fn the_grammar_is_the_row_walk_s() {
    let mut w = walk(&[60.0, 90.0], 500.0, 4.0);
    assert_eq!(w.press(UiButton::Right), RowOutcome::Stepped { row: 0, by: 1 });
    assert_eq!(w.press(UiButton::Accept), RowOutcome::Chosen(0));
    w.set_sections(3);
    assert_eq!(w.press(UiButton::PageDown), RowOutcome::Section(1));
    w.set_trail(true);
    assert_eq!(w.press(UiButton::PageDown), RowOutcome::Declined, "under a trail the shoulders are the host's");
    assert!(w.focus(1));
    assert_eq!(w.current(), 1);
}

#[test]
fn rows_that_fill_the_window_exactly_all_show_though_their_float_sum_runs_a_hair_over() {
    // Eclipse's Settings: nine 84 px rows 10 apart in a window to the ninth's foot, in canvas units of 2/3 (EmuSen_Qt.md §3.4.11).
    let unit = 2.0 / 3.0;
    let heights = [84.0 * unit; 9];
    let span = heights.iter().sum::<f64>() + 8.0 * 10.0 * unit;
    let mut w = walk(&heights, span * (1.0 - 1e-12), 10.0 * unit);
    for _ in 0..8 {
        w.press(UiButton::Down);
    }
    assert_eq!((w.current(), w.first()), (8, 0), "the ninth focused, the first still shown");
    let mut w = walk(&[84.0 * unit; 10], span * (1.0 - 1e-12), 10.0 * unit);
    for _ in 0..9 {
        w.press(UiButton::Down);
    }
    assert_eq!((w.current(), w.first()), (9, 1), "a tenth row moves the window by one");
}
