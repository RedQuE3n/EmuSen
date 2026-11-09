// The page frame's places held to the game page recording at 1280 x 800, and inside the screen at 16:9 (EmuSen_Kit.md §7.40).

use emusen_kit_core::audit::Rect;
use emusen_kit_core::page::{PageGeometry, page_layout};

const G: PageGeometry = PageGeometry::RECORDED;

#[test]
fn the_game_page_recording_at_1280_x_800() {
    let l = page_layout(1280.0, 800.0, 0.75, &G);
    assert_eq!(l.hero, Rect::new(0.0, 0.0, 1280.0, 360.0));
    assert_eq!(l.cover, Rect::new(64.0, 180.0, 201.0, 268.0), "a 3:4 cover at the recording's 201 x 268");
    assert_eq!((l.title.x, l.title.y), (336.0, 226.0));
    assert_eq!((l.facts.x, l.facts.y), (336.0, 286.0));
    assert_eq!(l.actions, Rect::new(336.0, 328.0, 880.0, 44.0));
    assert_eq!(l.strip, Rect::new(336.0, 396.0, 880.0, 44.0));
    assert_eq!(l.content, Rect::new(64.0, 464.0, 1152.0, 238.0));
    assert_eq!(l.rows, 4, "four whole rows of 54 in 238");
}

#[test]
fn a_wide_cover_stops_short_of_the_column_beside_it() {
    let l = page_layout(1280.0, 800.0, 4.0 / 3.0, &G);
    assert_eq!(l.cover.w, 240.0, "the column at 336 less the pane gap");
    assert!(l.cover.right() < l.title.x);
}

#[test]
fn at_16_9_the_column_and_the_content_widen_and_stay_inside() {
    let wide = 800.0 * 16.0 / 9.0;
    let l = page_layout(wide, 800.0, 0.75, &G);
    assert_eq!(l.hero.w, wide, "the hero runs edge to edge");
    for part in [l.title, l.facts, l.actions, l.strip, l.content] {
        assert!((part.right() - (wide - 64.0)).abs() < 1e-9, "{part:?} ends at the safe area's right");
    }
    assert!(l.cover.bottom() < l.content.y, "the cover clears the content");
    assert!(l.strip.bottom() < l.content.y);
}
