// EmuSen_Kit.md §7.40–§7.42: the settings frame's measures held to Mistress's recordings (SettingsSound-1280x800.spec,
// SettingsDisplay-1280x800.spec), and the walk over its rows, a slider row and a page of choices.

use emusen_kit_core::input::UiButton;
use emusen_kit_core::settings::{Row, RowKind, RowOutcome, RowWalk, SETTINGS, SliderRange};
use emusen_kit_core::tokens::{Form, spacing};

const ROW: f64 = spacing(Form::BigScreen).row_height;

fn rows(kinds: &[RowKind]) -> Vec<Row> {
    kinds.iter().map(|k| Row::of(*k)).collect()
}

fn walk(kinds: &[RowKind], shown: usize) -> RowWalk {
    let mut walk = RowWalk::new(shown);
    walk.set_rows(rows(kinds), false);
    walk
}

#[test]
fn the_measures_are_mistress_s_at_1280_by_800() {
    let g = SETTINGS;
    assert_eq!((g.side, g.strip_top, g.strip_height), (64.0, 40.0, 44.0), "the strip at 64,40, 44 high");
    assert_eq!((g.strip_gap, g.tab_padding), (8.0, 16.0), "CONTROLLERS at 64 with its words at 80, DISPLAY at 231");
    assert_eq!(g.content_top, 108.0, "the rows and the pane at 108");
    assert_eq!(g.rows_width(1280.0), 704.0, "rows 704 wide");
    assert_eq!((g.pane_left(1280.0), g.pane_width, g.pane_height), (800.0, 416.0, 578.0), "the pane at 800,108, 416 x 578");
    assert_eq!((g.pane_inset, g.slot_width()), (24.0, 368.0), "the pane's words at 824,132, 368 wide");
    assert_eq!(g.slot_gap, 12.0, "a picture 276 high at 132, its sentence at 420");
    assert_eq!(g.rows_shown(ROW), 10, "ten rows from 108 to 648");
}

#[test]
fn a_wider_canvas_widens_the_rows_and_keeps_the_pane() {
    let wide = 1280.0 * 1920.0 / 1080.0 * 800.0 / 1200.0;
    assert!((SETTINGS.pane_left(wide) - (wide - 480.0)).abs() < 1e-9, "the pane stays 64 from the right");
    assert!((SETTINGS.rows_width(wide) - (wide - 576.0)).abs() < 1e-9, "the rows take the rest");
    assert_eq!(SETTINGS.rows_width(300.0), 0.0, "never a negative width");
    assert_eq!(SETTINGS.rows_shown(0.0), 0);
}

#[test]
fn up_and_down_move_and_stop_at_the_ends() {
    let mut w = walk(&[RowKind::Steps, RowKind::Opens, RowKind::Action], 10);
    assert_eq!(w.current(), 0);
    assert_eq!(w.press(UiButton::Up), RowOutcome::Kept, "the first row does not wrap to the last");
    assert_eq!(w.press(UiButton::Down), RowOutcome::Moved(1));
    assert_eq!(w.press(UiButton::Down), RowOutcome::Moved(2));
    assert_eq!(w.press(UiButton::Down), RowOutcome::Kept, "the last row does not wrap to the first");
    assert_eq!(w.press(UiButton::Up), RowOutcome::Moved(1));
}

#[test]
fn the_window_moves_only_when_the_focus_leaves_it_a_row_at_a_time() {
    let mut w = walk(&[RowKind::Action; 12], 10);
    for row in 1..10 {
        assert_eq!(w.press(UiButton::Down), RowOutcome::Moved(row));
        assert_eq!(w.first(), 0, "row {row} is inside the window");
    }
    assert_eq!(w.press(UiButton::Down), RowOutcome::Moved(10));
    assert_eq!(w.first(), 1, "the eleventh row moves the window by one");
    w.press(UiButton::Down);
    assert_eq!(w.first(), 2);
    for _ in 0..9 {
        w.press(UiButton::Up);
    }
    assert_eq!((w.current(), w.first()), (2, 2), "up inside the window leaves it");
    w.press(UiButton::Up);
    assert_eq!((w.current(), w.first()), (1, 1));
    assert!(w.scroll_to(9), "the wheel moves it");
    assert_eq!(w.first(), 2, "held so the last row is the window's last");
    assert_eq!(w.current(), 1, "the wheel leaves the focus where it was");
    assert!(w.focus(7) && !w.focus(7) && !w.focus(12), "the pointer focuses a row, once, and only a row there is");
    w.set_shown(5);
    assert_eq!((w.current(), w.first()), (7, 3), "a smaller window still holds the focus");
}

#[test]
fn left_and_right_step_a_stepped_value_or_a_slider_and_nothing_else() {
    let mut w = walk(&[RowKind::Steps, RowKind::Slider, RowKind::Opens, RowKind::Action, RowKind::Choice], 10);
    assert_eq!(w.press(UiButton::Left), RowOutcome::Stepped { row: 0, by: -1 });
    assert_eq!(w.press(UiButton::Right), RowOutcome::Stepped { row: 0, by: 1 });
    w.press(UiButton::Down);
    assert_eq!(w.press(UiButton::Right), RowOutcome::Slid { row: 1, amount: 0.01 }, "a slider without a step takes a hundredth");
    assert_eq!(w.press(UiButton::Left), RowOutcome::Slid { row: 1, amount: 0.0 });
    assert_eq!(w.press(UiButton::Left), RowOutcome::Kept, "and stops at its minimum");
    for _ in 2..5 {
        w.press(UiButton::Down);
        assert_eq!(w.press(UiButton::Left), RowOutcome::Kept, "{:?} does not step", w.rows()[w.current()].kind);
        assert_eq!(w.press(UiButton::Right), RowOutcome::Kept);
    }
}

#[test]
fn a_held_row_takes_no_change_and_a_chooses_the_others() {
    let mut list = rows(&[RowKind::Steps, RowKind::Slider, RowKind::Opens, RowKind::Asks, RowKind::Action]);
    list[0].held = true;
    list[1].held = true;
    let mut w = RowWalk::new(10);
    w.set_rows(list, false);
    assert_eq!(w.press(UiButton::Right), RowOutcome::Kept, "a held value");
    assert_eq!(w.press(UiButton::Accept), RowOutcome::Kept, "a held row is not chosen");
    w.press(UiButton::Down);
    assert_eq!(w.press(UiButton::Left), RowOutcome::Kept, "a held slider");
    for row in 2..5 {
        w.press(UiButton::Down);
        assert_eq!(w.press(UiButton::Accept), RowOutcome::Chosen(row));
    }
    assert_eq!(w.marked(), None, "choosing a row that is no choice marks nothing");
}

#[test]
fn a_page_of_choices_opens_on_its_mark_and_a_moves_the_mark() {
    let mut list = rows(&[RowKind::Choice; 3]);
    list[2].marked = true;
    let mut w = RowWalk::new(10);
    w.set_rows(list.clone(), false);
    assert_eq!((w.current(), w.marked()), (2, Some(2)), "the focus on the choice in force");
    w.press(UiButton::Up);
    w.press(UiButton::Up);
    assert_eq!(w.press(UiButton::Accept), RowOutcome::Chosen(0));
    assert_eq!(w.marked(), Some(0), "one mark, on the choice made");
    assert_eq!(w.rows().iter().filter(|r| r.marked).count(), 1);
    w.press(UiButton::Down);
    w.set_rows(list, true);
    assert_eq!((w.current(), w.marked()), (1, Some(2)), "the host's rows read again keep the focus and take the host's mark");
}

#[test]
fn a_long_page_of_choices_shows_its_mark() {
    let mut list = rows(&[RowKind::Choice; 14]);
    list[12].marked = true;
    let mut w = RowWalk::new(10);
    w.set_rows(list, false);
    assert_eq!((w.current(), w.first()), (12, 3), "the mark is in the window");
}

#[test]
fn the_rest_is_declined_and_no_rows_decline_everything() {
    let mut w = walk(&[RowKind::Steps], 10);
    for button in [UiButton::Back, UiButton::PageUp, UiButton::PageDown, UiButton::Menu, UiButton::Options] {
        assert_eq!(w.press(button), RowOutcome::Declined, "{button:?} is the frame's or the host's");
    }
    let mut none = RowWalk::new(10);
    for button in [UiButton::Up, UiButton::Down, UiButton::Left, UiButton::Accept] {
        assert_eq!(none.press(button), RowOutcome::Declined);
    }
}

#[test]
fn the_kinds_are_numbered_as_the_qml_enumeration() {
    for (i, kind) in RowKind::ALL.iter().enumerate() {
        assert_eq!(RowKind::from_index(i as i64), *kind);
    }
    assert_eq!(RowKind::from_index(4), RowKind::Slider);
    assert_eq!(RowKind::from_index(-1), RowKind::Action, "an unknown kind is an action");
    assert_eq!(RowKind::from_index(99), RowKind::Action);
}

#[test]
fn the_shoulders_step_the_sections_wrapping_and_are_the_host_s_under_a_trail() {
    let mut w = walk(&[RowKind::Action; 3], 10);
    w.set_sections(6);
    assert_eq!(w.press(UiButton::PageUp), RowOutcome::Section(5), "LB from the first section wraps to the last");
    assert_eq!(w.press(UiButton::PageDown), RowOutcome::Section(0), "RB from the last wraps to the first");
    assert_eq!(w.press(UiButton::PageDown), RowOutcome::Section(1));
    assert!(w.select_section(4) && !w.select_section(6), "the host chooses a section there is");
    assert_eq!(w.section(), 4);
    w.set_trail(true);
    assert_eq!(w.press(UiButton::PageDown), RowOutcome::Declined, "a page's shoulders are the host's");
    assert_eq!(w.section(), 4, "and the section stays");
    w.set_trail(false);
    w.set_sections(3);
    assert_eq!(w.section(), 0, "a section no longer there gives way to the first");
    w.set_sections(1);
    assert_eq!(w.press(UiButton::PageUp), RowOutcome::Declined, "one section does not step");
}

#[test]
fn a_slider_steps_on_its_grid_inside_its_range() {
    let range = SliderRange { minimum: 0.0, maximum: 25.0, step: 0.5 };
    let slider = |amount: f64, held: bool| Row { kind: RowKind::Slider, held, range, amount, ..Row::default() };
    let mut w = RowWalk::new(10);
    w.set_rows(vec![slider(2.0, false), slider(24.5, false), slider(2.3, false), slider(8.0, true)], false);
    assert_eq!(w.press(UiButton::Right), RowOutcome::Slid { row: 0, amount: 2.5 });
    assert_eq!(w.press(UiButton::Right), RowOutcome::Slid { row: 0, amount: 3.0 }, "the walk keeps the amount it gave");
    assert_eq!(w.press(UiButton::Left), RowOutcome::Slid { row: 0, amount: 2.5 });
    w.press(UiButton::Down);
    assert_eq!(w.press(UiButton::Right), RowOutcome::Slid { row: 1, amount: 25.0 });
    assert_eq!(w.press(UiButton::Right), RowOutcome::Kept, "the maximum holds");
    w.press(UiButton::Down);
    assert_eq!(w.press(UiButton::Right), RowOutcome::Slid { row: 2, amount: 3.0 }, "an amount off the grid steps onto it");
    w.press(UiButton::Down);
    assert_eq!(w.press(UiButton::Left), RowOutcome::Kept, "a held slider");
    let reversed = SliderRange { minimum: 10.0, maximum: 0.0, step: 3.0 };
    assert_eq!(reversed.stepped(9.0, 1), 9.0, "the last whole step below the top of a range given high to low");
    assert_eq!(reversed.stepped(9.0, -4), 0.0);
    assert!(SliderRange::default().stepped(f64::NAN, 1).is_nan(), "nothing made of no number");
}
