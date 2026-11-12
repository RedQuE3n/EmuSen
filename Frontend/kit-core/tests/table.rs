// FitsWidth: a table's columns stay inside its width at every width, held to the Cores recording's cells (EmuSen_Kit.md §7.43).

use emusen_kit_core::table::{ColumnWidth, TableMeasures, fit_columns, room};

const M: TableMeasures = TableMeasures::BIG_SCREEN;

fn fill(weight: f64, least: f64) -> ColumnWidth {
    ColumnWidth::Fill { weight, least, words: 0.0 }
}

/// The last column's right edge never passes the table's width less its trail, and no column is negative.
fn inside(width: f64, columns: &[ColumnWidth]) {
    let placed = fit_columns(width, columns, &M);
    assert_eq!(placed.len(), columns.len());
    assert!(placed.iter().all(|c| c.width >= 0.0), "a negative column at {width}: {placed:?}");
    if let Some(last) = placed.last() {
        let limit = (width - M.trail).max(M.lead + M.gap * (columns.len() - 1) as f64);
        assert!(last.x + last.width <= limit + 1e-9, "past the width at {width}: {placed:?}");
    }
    for pair in placed.windows(2) {
        assert!((pair[1].x - (pair[0].x + pair[0].width + M.gap)).abs() < 1e-9, "columns overlap or part at {width}");
    }
}

#[test]
fn the_cores_recording_at_1280_x_800() {
    // Core, Console and Status in a list 691 wide: the cells at 88, 409 and 577 from the list's 64 (Cores-1280x800.spec).
    let placed = fit_columns(691.0, &[fill(1.0, 0.0), ColumnWidth::Content(143.29), ColumnWidth::Content(153.45)], &M);
    let cells: Vec<(f64, f64)> = placed.iter().map(|c| (c.x + 64.0, c.width)).collect();
    assert_eq!(cells, vec![(88.0, 297.0), (409.0, 144.0), (577.0, 154.0)]);
}

#[test]
fn a_fill_column_takes_the_rest_and_ends_at_the_trail() {
    let placed = fit_columns(800.0, &[ColumnWidth::Content(100.0), fill(1.0, 0.0)], &M);
    assert_eq!(placed[0].width, 100.0);
    assert_eq!(placed[1].x + placed[1].width, 800.0 - M.trail, "the fill reaches the trail");
}

#[test]
fn fills_take_their_least_then_share_by_weight() {
    let placed = fit_columns(1000.0, &[fill(2.0, 100.0), fill(1.0, 100.0), ColumnWidth::Content(200.0)], &M);
    // Room 1000 - 48 - 48 = 904; the content keeps 200; 704 left, 504 over the leasts, shared 2:1.
    assert_eq!((placed[0].width, placed[1].width, placed[2].width), (436.0, 268.0, 200.0));
}

#[test]
fn wide_words_give_way_to_a_fill_s_least() {
    // Room 400 - 96 = 304; the fill keeps its 190, the two content columns share 114 in proportion to 200 and 100.
    let placed = fit_columns(400.0, &[fill(1.0, 190.0), ColumnWidth::Content(200.0), ColumnWidth::Content(100.0)], &M);
    assert_eq!(placed[0].width, 190.0);
    assert!((placed[1].width - 76.0).abs() < 1e-9 && (placed[2].width - 38.0).abs() < 1e-9, "{placed:?}");
}

#[test]
fn a_room_under_the_leasts_scales_them_into_it() {
    let placed = fit_columns(200.0, &[fill(1.0, 150.0), fill(1.0, 50.0), ColumnWidth::Content(80.0)], &M);
    // Room 200 - 96 = 104: the content gives up everything, the leasts scale 3:1.
    assert_eq!((placed[0].width, placed[1].width, placed[2].width), (78.0, 26.0, 0.0));
}

#[test]
fn content_alone_keeps_its_width_where_it_fits_and_shrinks_where_it_does_not() {
    let fits = fit_columns(600.0, &[ColumnWidth::Content(120.4), ColumnWidth::Content(80.0)], &M);
    assert_eq!((fits[0].width, fits[1].width), (121.0, 80.0), "words rounded up to whole units");
    let narrow = fit_columns(200.0, &[ColumnWidth::Content(120.0), ColumnWidth::Content(80.0)], &M);
    assert!((narrow[0].width + narrow[1].width - room(200.0, 2, &M)).abs() < 1e-9);
    assert!((narrow[0].width / narrow[1].width - 1.5).abs() < 1e-9, "shrunk in proportion");
}

#[test]
fn no_width_puts_a_column_past_the_table() {
    let sets: [&[ColumnWidth]; 5] = [
        &[fill(1.0, 0.0), ColumnWidth::Content(143.29), ColumnWidth::Content(153.45)],
        &[fill(1.0, 190.0), ColumnWidth::Content(300.0), ColumnWidth::Content(260.0), ColumnWidth::Content(90.0)],
        &[ColumnWidth::Content(500.0), ColumnWidth::Content(500.0)],
        &[fill(3.0, 40.0), fill(1.0, 400.0)],
        &[fill(0.0, 0.0), fill(0.0, 0.0), ColumnWidth::Content(10.0)],
    ];
    for columns in sets {
        let mut width = 0.0;
        while width <= 1600.0 {
            inside(width, columns);
            width += 7.5;
        }
    }
    assert!(fit_columns(500.0, &[], &M).is_empty());
}

/// Each column's width at `width`, rounded to a hundredth.
fn widths(width: f64, columns: &[ColumnWidth]) -> Vec<f64> {
    fit_columns(width, columns, &M).iter().map(|c| (c.width * 100.0).round() / 100.0).collect()
}

#[test]
fn kit_7_106_the_console_column_gives_way_and_the_others_keep_their_words() {
    // Cores at 1280 x 800 as measured on 2026-11-11: Core (fill, least 190, `Mupen64Plus-Next` 208.48), Console (four
    // consoles whole 458.09, "Super Nintendo" the widest short words 170.95) and Status ("● Default" 107.27).
    let cores = |whole: f64| [ColumnWidth::Fill { weight: 1.0, least: 190.0, words: 208.48 }, ColumnWidth::GivesWay { whole, short: 170.95 }, ColumnWidth::Content(107.27)];
    let at = widths(691.0, &cores(458.09));
    assert_eq!(at[2], 108.0, "the status words whole: {at:?}");
    assert!(at[0] >= 208.48, "the core's name whole: {at:?}");
    assert!(at[1] >= 171.0 && at[1] < 459.0, "the console column between its short and its whole words: {at:?}");
    // Room for every column's words: the console column whole, and the fill takes the rest.
    let wide = widths(1400.0, &cores(458.09));
    assert_eq!(wide[1], 459.0, "{wide:?}");
    assert!(wide[0] > 208.48, "{wide:?}");
    // Room for the short words alone: the short words, the fill and the status whole.
    let short = widths(584.0, &cores(458.09));
    assert_eq!(short[1], 171.0, "{short:?}");
    assert!(short[0] >= 190.0 && short[2] == 108.0, "{short:?}");
    // Less room than that: every column scaled into it, as before (§7.43), none past the table.
    let narrow = widths(500.0, &cores(458.09));
    assert!(narrow[1] <= 171.0, "{narrow:?}");
    inside(500.0, &cores(458.09));
    // A column whose whole words are its short ones gives nothing.
    assert_eq!(widths(691.0, &cores(170.95))[1], 171.0);
}

#[test]
fn kit_7_106_columns_that_give_way_stay_inside_the_table() {
    let set = [ColumnWidth::Fill { weight: 1.0, least: 120.0, words: 150.0 }, ColumnWidth::GivesWay { whole: 900.0, short: 60.0 }, ColumnWidth::GivesWay { whole: 300.0, short: 80.0 }, ColumnWidth::Content(90.0)];
    for width in [0.0, 100.0, 300.0, 500.0, 691.0, 900.0, 1500.0, 4000.0] {
        inside(width, &set);
    }
    let at = widths(691.0, &set);
    assert!(at[1] >= 60.0 && at[2] >= 80.0 && at[3] == 90.0 && at[0] >= 150.0, "{at:?}");
}
