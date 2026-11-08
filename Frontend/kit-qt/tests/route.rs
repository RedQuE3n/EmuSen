// EmuSen_Qt.md §1.7, EmuSen_Kit.md §7.3: a press goes to the open sheet, then the walk, then the focused control as the key it becomes.

use emusen_kit_core::input::{Key, UiButton, Walk, WalkForm};
use emusen_kit_qt::frame::{route, sheet_route, Routed, SheetRouted};

#[test]
fn qt_1_7_an_open_sheet_takes_every_press_before_the_walk() {
    let mut walk = Walk::new(WalkForm::BigScreen);
    walk.set_detail(true, false);
    assert_eq!(route(true, &mut walk, UiButton::Accept), Routed::Sheet);
    assert!(!walk.is_detail_open(), "the walk never saw the press");
    assert_eq!(route(true, &mut walk, UiButton::Back), Routed::Sheet);
}

#[test]
fn qt_1_7_the_walk_takes_what_its_grammar_means() {
    let mut walk = Walk::new(WalkForm::BigScreen);
    walk.set_detail(true, false);
    assert_eq!(route(false, &mut walk, UiButton::Accept), Routed::Walk, "A on the content opens the detail's page");
    assert!(walk.is_detail_open());
    assert_eq!(route(false, &mut walk, UiButton::Back), Routed::Walk, "B climbs");
    assert!(!walk.is_detail_open());
}

#[test]
fn qt_1_7_a_declined_press_becomes_the_key_a_stock_control_reads() {
    let mut walk = Walk::new(WalkForm::BigScreen);
    assert_eq!(route(false, &mut walk, UiButton::Back), Routed::Declined(Some(Key::Escape)), "B at the top is the host's");
    assert_eq!(route(false, &mut walk, UiButton::Down), Routed::Declined(Some(Key::Down)));
    assert_eq!(route(false, &mut walk, UiButton::Options), Routed::Declined(Some(Key::Select)));
    assert_eq!(route(false, &mut walk, UiButton::Guide), Routed::Declined(None), "no key stands for the guide button");
    walk.set_detail(false, false);
    assert_eq!(route(false, &mut walk, UiButton::Accept), Routed::Declined(Some(Key::Return)), "no detail: A reaches the control");
}

#[test]
fn kit_7_8_the_open_sheet_closes_on_b_and_passes_the_rest_as_keys() {
    assert_eq!(sheet_route(UiButton::Back), SheetRouted::Dismiss);
    assert_eq!(sheet_route(UiButton::Accept), SheetRouted::Pass(Some(Key::Return)), "A reaches the sheet's focused button");
    assert_eq!(sheet_route(UiButton::Left), SheetRouted::Pass(Some(Key::Left)));
    assert_eq!(sheet_route(UiButton::Guide), SheetRouted::Pass(None));
}
