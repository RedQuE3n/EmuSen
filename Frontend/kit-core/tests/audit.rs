// The audit rules held to EmuSen_InterfaceSpec.md §4, EmuSen_Settings_Reference.md §4.83.2 and §4.83.7, and EmuSen_Qt.md §8.1, on hand-built trees.

use emusen_kit_core::audit::{
    Axes, DESKTOP_CAPS, Edge, Edges, Fault, Item, Kind, Legibility, Rect, Side, State, TWO_ROWS, Text, check, contrast, fit, named,
    no_chrome, pad_reach, palette_reach, pointer_reach, reached, smallest_capitals,
};
use emusen_kit_core::input::{ElementId, UiButton, WalkForm};
use emusen_kit_core::tokens::{Colour, ColourToken, Variant, colour};

const PANEL: Rect = Rect::new(0.0, 0.0, 1280.0, 800.0);
const SLACK: f64 = 1.0;

fn id(n: u32) -> ElementId {
    ElementId(n)
}

fn declared() -> Vec<Colour> {
    [ColourToken::Surface, ColourToken::SurfaceRaised, ColourToken::Text, ColourToken::Accent, ColourToken::Border]
        .into_iter()
        .map(|t| colour(Variant::Dark, t))
        .collect()
}

fn text(cap: f64, width: f64, height: f64) -> Text {
    Text { cap_height: cap, width, height, ellipsis: false }
}

fn row(n: u32, y: f64, label: &str, call: &str) -> Item {
    Item::new(n, Kind::Row, Rect::new(64.0, y, 600.0, 54.0))
        .named(label)
        .operable()
        .heard(label)
        .calls(call, call)
        .paints(&[colour(Variant::Dark, ColourToken::SurfaceRaised), colour(Variant::Dark, ColourToken::Text)])
        .child(Item::new(n + 1, Kind::Words, Rect::new(72.0, y + 12.0, 300.0, 30.0)).words(text(14.0, 280.0, 28.0)))
}

/// A clean big-screen state: a title, a scroller of two rows, a button beside them and the help bar on the foot.
fn clean() -> Item {
    Item::new(1, Kind::Panel, PANEL)
        .paints(&[colour(Variant::Dark, ColourToken::Surface)])
        .child(Item::new(2, Kind::Title, Rect::new(64.0, 40.0, 600.0, 60.0)))
        .child(
            Item::new(3, Kind::Scroller, Rect::new(64.0, 120.0, 600.0, 500.0))
                .wheel_scrolls()
                .child(row(10, 120.0, "Play", "play"))
                .child(row(12, 174.0, "Settings", "settings")),
        )
        .child(
            Item::new(20, Kind::Control, Rect::new(700.0, 120.0, 200.0, 44.0))
                .named("Back")
                .operable()
                .heard("Back")
                .calls("back", "back")
                .paints(&[colour(Variant::Dark, ColourToken::Accent)])
                .child(Item::new(21, Kind::Words, Rect::new(710.0, 128.0, 180.0, 28.0)).words(text(14.0, 150.0, 26.0))),
        )
        .child(Item::new(30, Kind::Footer, Rect::new(0.0, 742.0, 1280.0, 58.0)))
}

fn edges() -> Vec<Edge> {
    vec![
        Edge { from: id(10), button: UiButton::Down, to: id(12) },
        Edge { from: id(12), button: UiButton::Up, to: id(10) },
        Edge { from: id(10), button: UiButton::Right, to: id(20) },
        Edge { from: id(20), button: UiButton::Left, to: id(10) },
    ]
}

fn state<'a>(edges: &'a [Edge], declared: &'a [Colour], form: WalkForm) -> State<'a> {
    State { panel: PANEL, form, focus: Some(id(10)), edges, declared, slack: SLACK, smallest: smallest_capitals() }
}

fn fit_faults(root: &Item) -> Vec<Fault> {
    fit(root, PANEL, SLACK, smallest_capitals())
}

#[test]
fn interface_spec_4_a_clean_screen_passes_every_rule() {
    let edges = edges();
    let declared = declared();
    assert_eq!(check(&clean(), &state(&edges, &declared, WalkForm::BigScreen)), vec![]);
}

#[test]
fn settings_4_83_7_the_floor_is_inters_capitals_at_16() {
    assert_eq!(DESKTOP_CAPS, 0.7273);
    assert!((smallest_capitals() - 11.64).abs() < 0.005);
    assert_eq!(TWO_ROWS, 84.0);
}

#[test]
fn settings_4_83_2_past_its_panel_and_past_a_scrollers_side_but_not_past_its_foot() {
    let past_panel = clean().child(Item::new(40, Kind::Control, Rect::new(1200.0, 300.0, 200.0, 44.0)));
    assert_eq!(fit_faults(&past_panel), vec![Fault::PastItsLimit { item: id(40), limit: None }]);

    let mut below = clean();
    below.children[1] = below.children[1].clone().child(row(14, 700.0, "Below", "below"));
    assert_eq!(fit_faults(&below), vec![], "a scroller excuses what it scrolls to, up and down");

    let mut beside = clean();
    beside.children[1] = beside.children[1].clone().child(Item::new(16, Kind::Picture, Rect::new(600.0, 400.0, 120.0, 80.0)));
    assert_eq!(fit_faults(&beside), vec![Fault::PastItsLimit { item: id(16), limit: Some(id(3)) }], "past a scroller's side is cut");

    let within_slack = clean().child(Item::new(41, Kind::Control, Rect::new(1000.0, 300.0, 280.9, 44.0)));
    assert_eq!(fit_faults(&within_slack), vec![]);
}

#[test]
fn settings_4_83_2_words_past_their_button() {
    let mut tree = clean();
    tree.children[2].children[0].rect = Rect::new(690.0, 128.0, 230.0, 28.0);
    let faults = fit_faults(&tree);
    assert!(faults.contains(&Fault::WordsPastOwner { words: id(21), owner: id(20) }), "{faults:?}");
}

#[test]
fn settings_4_83_2_two_drawn_controls_overlap_unless_one_is_inside_the_other_or_the_overlap_is_within_the_slack() {
    let overlapping = clean().child(Item::new(42, Kind::Control, Rect::new(880.0, 150.0, 100.0, 44.0)));
    assert_eq!(
        fit_faults(&overlapping),
        vec![Fault::Overlap { a: id(20), b: id(42) }, Fault::Overlap { a: id(21), b: id(42) }],
        "the button and its words both"
    );

    let inside = clean().child(Item::new(43, Kind::Picture, Rect::new(720.0, 130.0, 40.0, 24.0)));
    let faults = fit_faults(&inside);
    assert!(!faults.iter().any(|f| matches!(f, Fault::Overlap { .. })), "a picture inside a control is not an overlap: {faults:?}");

    let touching = clean().child(Item::new(44, Kind::Control, Rect::new(899.5, 120.0, 100.0, 44.0)));
    assert_eq!(fit_faults(&touching), vec![], "half a pixel is within the slack");

    let words_in_row = clean();
    assert!(!fit_faults(&words_in_row).iter().any(|f| matches!(f, Fault::Overlap { .. })), "a row's own words are its descendants");
}

#[test]
fn settings_4_83_2_a_scroller_shows_two_rows_of_42() {
    let short = clean().child(Item::new(45, Kind::Scroller, Rect::new(700.0, 300.0, 300.0, 80.0)).wheel_scrolls());
    assert_eq!(fit_faults(&short), vec![Fault::ScrollsInTooLittleRoom { item: id(45), height: 80.0 }]);
    let enough = clean().child(Item::new(46, Kind::Scroller, Rect::new(700.0, 300.0, 300.0, 84.0)).wheel_scrolls());
    assert_eq!(fit_faults(&enough), vec![]);
}

#[test]
fn settings_4_83_2_cut_at_a_scrolling_edge_with_no_fade() {
    let mut cut = clean();
    cut.children[1] = cut.children[1].clone().child(row(14, 600.0, "Edge", "edge"));
    let bottom = |item: u32| Fault::CutAtEdge { item: id(item), area: id(3), side: Side::Bottom };
    assert_eq!(fit_faults(&cut), vec![bottom(14), bottom(15)], "the row and its words both cross the foot");

    let mut faded = clean();
    faded.children[1] = faded.children[1].clone().fading(Edges { bottom: true, ..Edges::default() }).child(row(14, 600.0, "Edge", "edge"));
    assert_eq!(fit_faults(&faded), vec![], "an edge that fades may slice");

    let sideways = clean().child(
        Item::new(50, Kind::Scroller, Rect::new(700.0, 300.0, 300.0, 200.0))
            .scrolling(Axes { vertical: false, horizontal: true })
            .wheel_scrolls()
            .child(Item::new(51, Kind::Picture, Rect::new(950.0, 320.0, 120.0, 80.0))),
    );
    let faults = fit_faults(&sideways);
    assert!(faults.contains(&Fault::CutAtEdge { item: id(51), area: id(50), side: Side::Right }), "{faults:?}");
    assert!(faults.contains(&Fault::PastItsLimit { item: id(51), limit: Some(id(50)) }), "and a pad never scrolls sideways: {faults:?}");
}

#[test]
fn settings_4_83_2_words_cut_with_an_ellipsis_past_their_box_at_the_foot_or_too_small() {
    let mut tree = clean();
    tree.children[2].children[0].text = Some(Text { cap_height: 11.0, width: 200.0, height: 40.0, ellipsis: true });
    let faults = fit_faults(&tree);
    assert!(faults.contains(&Fault::Ellipsis { item: id(21) }));
    assert!(faults.contains(&Fault::RunsPastItsBox { item: id(21) }));
    assert!(faults.contains(&Fault::CutAtFoot { item: id(21) }));
    assert!(faults.contains(&Fault::TooSmall { item: id(21), capitals: 11.0 }));
    assert_eq!(faults.len(), 4, "{faults:?}");

    let mut at_the_floor = clean();
    at_the_floor.children[2].children[0].text = Some(text(11.61, 150.0, 26.0));
    assert_eq!(fit_faults(&at_the_floor), vec![], "11.61 is inside the slack of 0.036 under 11.64");
    at_the_floor.children[2].children[0].text = Some(text(11.59, 150.0, 26.0));
    assert_eq!(fit_faults(&at_the_floor).len(), 1);
    assert_eq!(fit(&at_the_floor, PANEL, SLACK, 0.0), vec![], "a desktop window has no floor");
}

#[test]
fn settings_4_83_2_a_title_or_footer_past_the_panel_is_cut() {
    let mut tree = clean();
    tree.children[0].rect = Rect::new(64.0, 40.0, 1300.0, 60.0);
    tree.children[3].rect = Rect::new(0.0, 742.0, 1280.0, 70.0);
    let faults = fit_faults(&tree);
    assert_eq!(faults, vec![Fault::TitleCut { item: id(2) }, Fault::FooterCut { item: id(30) }]);
}

#[test]
fn settings_4_83_2_a_popup_and_a_hidden_subtree_are_skipped() {
    let popup = clean().child(Item::new(60, Kind::Popup, Rect::new(1100.0, 600.0, 400.0, 300.0)).child(Item::new(
        61,
        Kind::Control,
        Rect::new(1100.0, 600.0, 400.0, 44.0),
    )));
    assert_eq!(fit_faults(&popup), vec![]);
    let hidden = clean().child(Item::new(62, Kind::Panel, Rect::new(0.0, 0.0, 10.0, 10.0)).hidden().child(Item::new(
        63,
        Kind::Control,
        Rect::new(1200.0, 300.0, 200.0, 44.0),
    )));
    assert_eq!(fit_faults(&hidden), vec![]);
}

#[test]
fn interface_spec_4_2_the_pad_reaches_every_operable_control_but_an_unseen_row() {
    let edges = edges();
    assert_eq!(pad_reach(&clean(), Some(id(10)), &edges), vec![]);
    let without_button: Vec<Edge> = edges.iter().copied().filter(|e| e.to != id(20)).collect();
    assert_eq!(pad_reach(&clean(), Some(id(10)), &without_button), vec![Fault::Unreachable { item: id(20) }]);
    let mut unseen = clean();
    unseen.children[1] = unseen.children[1].clone().child(row(14, 300.0, "Kept", "kept").unseen());
    assert_eq!(pad_reach(&unseen, Some(id(10)), &edges), vec![]);
    assert_eq!(pad_reach(&clean(), None, &edges).len(), 3, "no focus: nothing is reached");
    assert_eq!(reached(Some(id(10)), &edges).len(), 3);
}

#[test]
fn qt_8_1_a_click_at_the_centre_of_a_reached_element_hits_it_and_calls_what_a_calls() {
    let edges = edges();
    let seen = reached(Some(id(10)), &edges);
    assert_eq!(pointer_reach(&clean(), &seen), vec![]);

    let covered = clean().child(Item::new(70, Kind::Control, Rect::new(0.0, 100.0, 700.0, 100.0)).named("scrim").hit_testable());
    assert_eq!(pointer_reach(&covered, &seen), vec![Fault::Covered { item: id(10), by: id(70) }]);

    let mut child_on_top = clean();
    child_on_top.children[1].children[0].children[0].hit_testable = true;
    assert_eq!(pointer_reach(&child_on_top, &seen), vec![], "a row's own words over its centre are the row's");

    let mut differs = clean();
    differs.children[2].click_call = Some("open".into());
    assert_eq!(pointer_reach(&differs, &seen), vec![Fault::ClickDiffers { item: id(20) }]);

    let mut no_wheel = clean();
    no_wheel.children[1].wheel_scrolls = false;
    assert_eq!(pointer_reach(&no_wheel, &seen), vec![Fault::WheelDoesNotMove { item: id(3) }]);
}

#[test]
fn qt_8_1_no_menu_bar_or_status_bar_in_the_big_screen_and_card_forms() {
    let chrome = clean().child(Item::new(80, Kind::StatusBar, Rect::new(0.0, 776.0, 1280.0, 24.0)));
    assert_eq!(no_chrome(&chrome, WalkForm::BigScreen), vec![Fault::Chrome { item: id(80) }]);
    assert_eq!(no_chrome(&chrome, WalkForm::Card), vec![Fault::Chrome { item: id(80) }]);
    assert_eq!(no_chrome(&chrome, WalkForm::Desktop), vec![], "desktop chrome belongs to the desktop form");
    let hidden = clean().child(Item::new(81, Kind::MenuBar, Rect::new(0.0, 0.0, 1280.0, 28.0)).hidden());
    assert_eq!(no_chrome(&hidden, WalkForm::BigScreen), vec![]);
}

#[test]
fn interface_spec_4_3_every_operable_control_has_a_name_a_reader_hears() {
    assert_eq!(named(&clean()), vec![]);
    let mut nameless = clean();
    nameless.children[2].heard = None;
    assert_eq!(named(&nameless), vec![Fault::Unnamed { item: id(20) }]);
    nameless.children[2].heard = Some("   ".into());
    assert_eq!(named(&nameless), vec![Fault::Unnamed { item: id(20) }]);
}

#[test]
fn interface_spec_4_4_every_painted_colour_is_declared_and_a_scene_is_exempt() {
    let declared = declared();
    assert_eq!(palette_reach(&clean(), &declared), vec![]);
    let stray = Colour::hex("#007ACC");
    let mut painted = clean();
    painted.children[2].paints.push(stray);
    assert_eq!(palette_reach(&painted, &declared), vec![Fault::OutsidePalette { item: id(20), colour: stray }]);
    let themed = clean().child(
        Item::new(90, Kind::Panel, Rect::new(0.0, 0.0, 1280.0, 800.0))
            .scene()
            .child(Item::new(91, Kind::Picture, Rect::new(0.0, 0.0, 100.0, 100.0)).paints(&[stray])),
    );
    assert_eq!(palette_reach(&themed, &declared), vec![]);
}

fn words_over(ink: Colour, size: f64, bold: bool, behind: &[Colour]) -> Item {
    let words =
        Item::new(2, Kind::Words, Rect::new(10.0, 10.0, 200.0, 30.0)).legible(Legibility { ink, size, bold, behind: behind.to_vec() });
    Item::new(1, Kind::Panel, PANEL).child(words)
}

fn ratio_of(faults: &[Fault]) -> Option<f64> {
    faults.iter().find_map(|f| match f {
        Fault::LowContrast { ratio, .. } => Some(*ratio),
        _ => None,
    })
}

#[test]
fn kit_7_10_words_reach_4_5_to_1_or_3_to_1_when_large_against_what_is_behind_them() {
    let grey = Colour::hex("#767676");
    let white = Colour::hex("#FFFFFF");
    assert!(contrast(&words_over(grey, 16.0, false, &[white])).is_empty(), "#767676 on white is 4.54:1");
    let faint = Colour::hex("#949494");
    let faults = contrast(&words_over(faint, 16.0, false, &[white]));
    assert_eq!(faults.len(), 1);
    assert!((ratio_of(&faults).unwrap() - 3.03).abs() < 0.02, "{faults:?}");
    assert!(contrast(&words_over(faint, 24.0, false, &[white])).is_empty(), "24 px is large: 3:1");
    assert!(contrast(&words_over(faint, 19.0, true, &[white])).is_empty(), "18.66 px bold is large");
    assert_eq!(contrast(&words_over(faint, 19.0, false, &[white])).len(), 1, "19 px regular is not");
}

#[test]
fn kit_7_10_the_ink_is_composited_and_a_spare_tenth_of_the_backdrop_may_fall_short() {
    let black = Colour::hex("#000000");
    let white = Colour::hex("#FFFFFF");
    let half_white = Colour::rgba(255, 255, 255, 128);
    let faults = contrast(&words_over(half_white, 16.0, false, &[black]));
    assert!(faults.is_empty(), "white at half over black is #808080, 5.3:1: {faults:?}");
    let quarter_white = Colour::rgba(255, 255, 255, 64);
    assert_eq!(contrast(&words_over(quarter_white, 16.0, false, &[black])).len(), 1, "at a quarter it is 2.1:1");
    let mut sky = vec![black; 19];
    sky.push(white);
    assert!(contrast(&words_over(white, 16.0, false, &sky)).is_empty(), "one star in twenty behind white words is spare");
    let mut half = vec![black; 10];
    half.extend(vec![white; 10]);
    assert_eq!(contrast(&words_over(white, 16.0, false, &half)).len(), 1, "words half over their own colour fail");
    assert!(contrast(&words_over(Colour::rgba(0, 0, 0, 0), 16.0, false, &[black])).is_empty(), "no ink, nothing drawn");
}

#[test]
fn a_fault_reads_as_one_line() {
    let line = Fault::CutAtEdge { item: id(14), area: id(3), side: Side::Bottom }.to_string();
    assert_eq!(line, "cut at the Bottom edge of 3 with no fade: 14");
    assert_eq!(
        Fault::OutsidePalette { item: id(20), colour: Colour::hex("#007ACC") }.to_string(),
        "a colour outside the palette on 20: #007ACC"
    );
}
