// The tool screen's places held to the Cores recording, and the pane spanning the list and the bar at every size (EmuSen_Kit.md §7.39).

use emusen_kit_core::audit::Rect;
use emusen_kit_core::tool::{ToolGeometry, ToolLayout, tool_layout};

const G: ToolGeometry = ToolGeometry::RECORDED;

fn r(x: f64, y: f64, w: f64, h: f64) -> Rect {
    Rect::new(x, y, w, h)
}

/// The rules every layout keeps: the pane spans the list and the bar, the bar stands on the list's foot, the list is
/// whole rows, and everything lies inside the safe area above the help bar.
fn spans(width: f64, height: f64, l: &ToolLayout) {
    let at = format!("{width} x {height}");
    assert_eq!(l.pane.y, l.list.y, "the pane starts with the list at {at}");
    assert_eq!(l.pane.bottom(), l.bar.bottom(), "the pane ends with the bar at {at}");
    assert_eq!(l.bar.y, l.list.bottom(), "the bar stands on the list's foot at {at}");
    assert_eq!((l.bar.x, l.bar.w), (l.list.x, l.list.w), "the bar belongs to the list at {at}");
    assert_eq!(l.list.h, l.rows as f64 * G.spacing.row_height, "whole rows at {at}");
    assert_eq!(l.pane.x, l.list.right() + G.gap, "the pane beside the list at {at}");
    let s = G.spacing.safe_area;
    let foot = height - s.bottom - G.spacing.help_bar;
    for (name, part) in [("header", l.header), ("strip", l.strip), ("search", l.search), ("list", l.list), ("bar", l.bar), ("pane", l.pane)]
    {
        assert!(part.x >= s.left && part.right() <= width - s.right + 1e-9, "{name} across the safe area at {at}: {part:?}");
        assert!(part.y >= s.top && part.bottom() <= foot + 1e-9, "{name} below the help bar at {at}: {part:?}");
    }
    assert_eq!(l.strip.right(), l.search.x, "the search at the strip's right at {at}");
}

#[test]
fn the_cores_recording_at_1280_x_800() {
    let l = tool_layout(1280.0, 800.0, &G);
    assert_eq!(l.header, r(64.0, 40.0, 1152.0, 48.0));
    assert_eq!(l.strip, r(64.0, 100.0, 912.0, 44.0));
    assert_eq!(l.search, r(976.0, 100.0, 240.0, 44.0));
    assert_eq!((l.list, l.rows), (r(64.0, 160.0, 691.0, 432.0), 8), "eight rows of 54");
    assert_eq!(l.bar, r(64.0, 592.0, 691.0, 110.0));
    assert_eq!(l.pane, r(771.0, 160.0, 445.0, 542.0), "the recording's detail, spanning list and bar");
    spans(1280.0, 800.0, &l);
}

#[test]
fn at_16_9_the_extra_width_goes_to_the_list_and_the_pane_in_their_shares() {
    let wide = 800.0 * 16.0 / 9.0;
    let l = tool_layout(wide, 800.0, &G);
    let across = wide - 128.0;
    assert_eq!(l.list.w, (across * 0.6).round());
    assert!((l.pane.w - (across - l.list.w - 16.0)).abs() < 1e-9);
    assert_eq!(l.rows, 8, "the same rows: the design is 800 high at every size");
    spans(wide, 800.0, &l);
}

#[test]
fn the_pane_spans_the_list_and_the_bar_at_every_size() {
    let mut height = 400.0;
    while height <= 1000.0 {
        let mut width = 960.0;
        while width <= 1800.0 {
            let l = tool_layout(width, height, &G);
            spans(width, height, &l);
            width += 37.0;
        }
        height += 13.0;
    }
}

#[test]
fn a_shorter_area_shows_fewer_whole_rows_never_a_cut_one() {
    let l = tool_layout(1280.0, 760.0, &G);
    assert_eq!(l.rows, 7, "40 units less: one row less, none cut");
    assert_eq!(l.pane.h, 7.0 * 54.0 + 110.0);
}
