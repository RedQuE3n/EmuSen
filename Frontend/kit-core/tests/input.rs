// Input held to its sources: EmuSen_Settings_Reference.md §4.29, EmuSen_BigPicture.md §14.7, LunaP §202.4 and EmuSen_Qt.md §1.7.

use std::collections::HashSet;
use std::time::Duration;

use emusen_kit_core::input::{
    ElementId, Grammar, InputEvent, Intent, Key, PointerButton, Press, RepeatTiers, Role, STICK_THRESHOLD, UiButton, Walk, WalkAction,
    WalkEvent, WalkForm, intent, stick_direction,
};

fn ms(v: u64) -> Duration {
    Duration::from_millis(v)
}

/// Feeds a grammar one button held from 0 at 60 Hz until `until`, giving the time of every press it made.
fn hold(tiers: RepeatTiers, button: UiButton, until: Duration) -> Vec<(Duration, Press)> {
    let mut grammar = Grammar::new(tiers);
    let mut out = Vec::new();
    let tick = Duration::from_micros(16_667);
    let mut now = Duration::ZERO;
    while now <= until {
        for press in grammar.feed(|b| b == button, now) {
            out.push((now, *press));
        }
        now += tick;
    }
    out
}

fn at_most(a: Duration, b: Duration, slack: Duration) -> bool {
    a.abs_diff(b) <= slack
}

#[test]
fn settings_4_29_every_button_presses_once_on_the_way_down_and_only_directions_and_shoulders_repeat() {
    for button in UiButton::ALL {
        let presses = hold(RepeatTiers::OWN, button, ms(1000));
        assert_eq!(presses[0].0, Duration::ZERO, "{button:?} presses as it goes down");
        assert!(!presses[0].1.repeat);
        if button.repeats() {
            assert!(presses.len() > 1, "{button:?} repeats");
            assert!(presses[1..].iter().all(|(_, p)| p.repeat && p.jump == 1));
        } else {
            assert_eq!(presses.len(), 1, "{button:?} never repeats");
        }
    }
    let repeating: HashSet<UiButton> = UiButton::ALL.into_iter().filter(|b| b.repeats()).collect();
    let expected = HashSet::from([UiButton::Up, UiButton::Down, UiButton::Left, UiButton::Right, UiButton::PageUp, UiButton::PageDown]);
    assert_eq!(repeating, expected);
}

#[test]
fn settings_4_29_the_own_tiers_repeat_after_400_ms_and_every_80_ms() {
    let presses = hold(RepeatTiers::OWN, UiButton::Down, ms(1000));
    assert_eq!(presses.len(), 9, "0, 400, then 480 to 960");
    let mut grammar = Grammar::new(RepeatTiers::OWN);
    assert_eq!(grammar.feed(|_| true, ms(0)).len(), UiButton::ALL.len());
    assert!(grammar.feed(|_| true, ms(399)).is_empty());
    let first: Vec<UiButton> = grammar.feed(|_| true, ms(400)).iter().map(|p| p.button).collect();
    assert_eq!(first, vec![UiButton::Up, UiButton::Down, UiButton::Left, UiButton::Right, UiButton::PageUp, UiButton::PageDown]);
    assert!(grammar.feed(|_| true, ms(479)).is_empty());
    assert_eq!(grammar.feed(|_| true, ms(480)).len(), 6);
    assert_eq!(grammar.feed(|_| true, ms(560)).len(), 6);
    assert!(grammar.feed(|_| false, ms(600)).is_empty(), "let go: nothing");
    assert_eq!(grammar.feed(|_| true, ms(601)).len(), UiButton::ALL.len(), "down again: every button once more");
}

#[test]
fn bigpicture_14_7_a_list_held_steps_at_500_then_114_jumps_four_at_1703_then_every_15_9_ms() {
    let presses = hold(RepeatTiers::ESDE_LIST, UiButton::Down, ms(1800));
    let repeats: Vec<&(Duration, Press)> = presses.iter().filter(|(_, p)| p.repeat).collect();
    let slack = Duration::from_micros(17_000);
    assert!(at_most(repeats[0].0, ms(500), slack));
    for (n, (at, press)) in repeats.iter().take(11).enumerate() {
        assert!(at_most(*at, ms(500 + 114 * n as u64), slack), "step {n} at {at:?}");
        assert_eq!(press.jump, 1);
    }
    let (jump_at, jump) = repeats[11];
    assert!(at_most(*jump_at, ms(1703), slack), "the four-entry step at {jump_at:?}");
    assert_eq!(jump.jump, 4);
    let after: Vec<&&(Duration, Press)> = repeats.iter().skip(12).collect();
    assert!(after.len() > 4);
    assert!(after.iter().all(|(_, p)| p.jump == 1));
    let mut grammar = Grammar::new(RepeatTiers::ESDE_LIST);
    grammar.feed(|_| true, ms(0));
    grammar.feed(|_| true, ms(1703));
    assert!(grammar.is_fast(UiButton::Down));
    let burst = grammar.feed(|b| b == UiButton::Down, ms(1703) + Duration::from_micros(159_000)).len();
    assert_eq!(burst, 10, "ten repeats of 15.9 ms fell due in 159 ms");
}

#[test]
fn bigpicture_14_7_a_carousel_held_fast_steps_at_500_six_at_180_then_80_from_1580() {
    let presses = hold(RepeatTiers::ESDE_CAROUSEL_FAST, UiButton::Right, ms(2000));
    let times: Vec<Duration> = presses.iter().filter(|(_, p)| p.repeat).map(|(t, _)| *t).collect();
    let slack = Duration::from_micros(17_000);
    for (n, expected) in [500u64, 680, 860, 1040, 1220, 1400].into_iter().enumerate() {
        assert!(at_most(times[n], ms(expected), slack), "step {n} at {:?}", times[n]);
    }
    assert!(at_most(times[6], ms(1580), slack));
    assert!(at_most(times[7], ms(1660), slack));
    assert!(at_most(times[8], ms(1740), slack));
    assert!(presses.iter().all(|(_, p)| p.jump == 1), "the carousel's switch moves one item");
}

#[test]
fn bigpicture_14_7_a_carousel_held_without_fast_scrolling_steps_at_500_then_every_200() {
    let presses = hold(RepeatTiers::ESDE_CAROUSEL, UiButton::Right, ms(3000));
    let times: Vec<Duration> = presses.iter().filter(|(_, p)| p.repeat).map(|(t, _)| *t).collect();
    let slack = Duration::from_micros(17_000);
    assert_eq!(times.len(), 13, "500 to 2900");
    for (n, at) in times.iter().enumerate() {
        assert!(at_most(*at, ms(500 + 200 * n as u64), slack), "step {n} at {at:?}");
    }
}

#[test]
fn settings_4_29_the_menu_chord_is_guide_or_both_middle_buttons_once() {
    let mut grammar = Grammar::new(RepeatTiers::OWN);
    assert!(grammar.menu_chord(|b| b == UiButton::Guide));
    assert!(!grammar.menu_chord(|b| b == UiButton::Guide), "held, not pressed again");
    assert!(!grammar.menu_chord(|_| false));
    assert!(!grammar.menu_chord(|b| b == UiButton::Menu), "Start alone is not the chord");
    assert!(grammar.menu_chord(|b| matches!(b, UiButton::Menu | UiButton::Options)));
    assert!(!grammar.menu_chord(|b| matches!(b, UiButton::Menu | UiButton::Options)));
}

#[test]
fn settings_4_29_forget_makes_nothing_a_press_until_it_is_let_go() {
    let mut grammar = Grammar::new(RepeatTiers::OWN);
    grammar.forget(|b| b == UiButton::Back);
    assert!(grammar.feed(|b| b == UiButton::Back, ms(0)).is_empty());
    assert!(grammar.feed(|b| b == UiButton::Back, ms(500)).is_empty(), "and it does not repeat");
    grammar.feed(|_| false, ms(600));
    assert_eq!(grammar.feed(|b| b == UiButton::Back, ms(700)).len(), 1);
    grammar.forget(|b| b == UiButton::Guide);
    assert!(!grammar.menu_chord(|b| b == UiButton::Guide), "the chord that opened a menu is not a press in it");
}

#[test]
fn mainwindow_pad_the_stick_is_a_direction_past_0_55() {
    assert_eq!(STICK_THRESHOLD, 0.55);
    assert_eq!(stick_direction(0.54), 0);
    assert_eq!(stick_direction(0.55), 1);
    assert_eq!(stick_direction(-0.55), -1);
    assert_eq!(stick_direction(-0.3), 0);
}

#[test]
fn qt_1_7_the_swap_trades_accept_and_back_and_the_declined_press_becomes_a_key() {
    assert_eq!(UiButton::Accept.swapped(true), UiButton::Back);
    assert_eq!(UiButton::Back.swapped(true), UiButton::Accept);
    assert_eq!(UiButton::Menu.swapped(true), UiButton::Menu);
    assert_eq!(UiButton::Accept.swapped(false), UiButton::Accept);
    assert_eq!(UiButton::Up.key(), Some(Key::Up));
    assert_eq!(UiButton::Accept.key(), Some(Key::Return));
    assert_eq!(UiButton::Back.key(), Some(Key::Escape));
    assert_eq!(UiButton::Menu.key(), Some(Key::Menu));
    assert_eq!(UiButton::Options.key(), Some(Key::Select));
    assert_eq!(UiButton::Guide.key(), None);
}

#[test]
fn qt_1_7_pointer_and_keyboard_events_mean_what_the_pad_means() {
    let e = ElementId(7);
    assert_eq!(intent(InputEvent::Hover(e)), Some(Intent::Focus(e)), "hover moves the focus");
    assert_eq!(intent(InputEvent::Click { target: e, button: PointerButton::Primary }), Some(Intent::Activate(e)), "a click is A");
    assert_eq!(intent(InputEvent::Click { target: e, button: PointerButton::Secondary }), Some(Intent::Press(UiButton::Back)));
    assert_eq!(intent(InputEvent::Click { target: e, button: PointerButton::Back }), Some(Intent::Press(UiButton::Back)));
    assert_eq!(intent(InputEvent::Wheel { notches: 1 }), Some(Intent::Step(1)));
    assert_eq!(intent(InputEvent::Wheel { notches: -3 }), Some(Intent::Step(-3)));
    assert_eq!(intent(InputEvent::Wheel { notches: 0 }), None);
    assert_eq!(intent(InputEvent::Key { key: Key::Tab, shift: false }), Some(Intent::Walk(1)));
    assert_eq!(intent(InputEvent::Key { key: Key::Tab, shift: true }), Some(Intent::Walk(-1)));
    assert_eq!(intent(InputEvent::Key { key: Key::Return, shift: false }), Some(Intent::Press(UiButton::Accept)));
    assert_eq!(intent(InputEvent::Key { key: Key::Escape, shift: false }), Some(Intent::Press(UiButton::Back)));
    assert_eq!(intent(InputEvent::Key { key: Key::Left, shift: false }), Some(Intent::Press(UiButton::Left)));
    assert_eq!(intent(InputEvent::Key { key: Key::Home, shift: false }), Some(Intent::Press(UiButton::First)));
    assert_eq!(intent(InputEvent::Key { key: Key::Other, shift: false }), None);
    let press = Press { button: UiButton::PageDown, repeat: true, jump: 1 };
    assert_eq!(intent(InputEvent::Pad(press)), Some(Intent::Press(UiButton::PageDown)));
}

fn frame(form: WalkForm) -> Walk {
    let mut walk = Walk::new(form);
    walk.set_detail(true, false);
    walk.show_actions(true);
    walk.set_sections(vec!["games".into(), "tools".into(), "settings".into()], Some("tools"));
    walk.set_actions(vec![
        WalkAction { key: "scan".into(), button: Some(UiButton::Search), enabled: true },
        WalkAction { key: "wipe".into(), button: Some(UiButton::Options), enabled: false },
    ]);
    walk
}

#[test]
fn lunap_202_4_the_walk_steps_the_shown_roles_in_order_and_wraps_in_both_forms() {
    let mut desktop = frame(WalkForm::Desktop);
    assert!(desktop.move_focus(1));
    assert_eq!(desktop.focused(), Some(Role::Navigation));
    let mut seen = vec![Role::Navigation];
    for _ in 0..3 {
        desktop.move_focus(1);
        seen.push(desktop.focused().unwrap());
    }
    assert_eq!(seen, vec![Role::Navigation, Role::Content, Role::Detail, Role::Actions], "the desktop's detail is always beside");
    desktop.move_focus(1);
    assert_eq!(desktop.focused(), Some(Role::Navigation), "wraps");
    desktop.move_focus(-1);
    assert_eq!(desktop.focused(), Some(Role::Actions), "and back");

    let mut big = frame(WalkForm::BigScreen);
    assert!(big.move_focus(-1));
    assert_eq!(big.focused(), Some(Role::Actions), "backwards from nowhere lands on the last");
    big.move_focus(1);
    assert_eq!(big.focused(), Some(Role::Navigation));
    big.move_focus(1);
    big.move_focus(1);
    assert_eq!(big.focused(), Some(Role::Actions), "the detail is skipped until its page is open");
}

#[test]
fn lunap_202_4_the_shoulders_step_the_sections_wrapping_and_select_section_raises_nothing() {
    let mut walk = frame(WalkForm::BigScreen);
    assert!(walk.press(UiButton::PageDown));
    assert_eq!(walk.selected_section(), Some("settings"));
    assert!(walk.press(UiButton::PageDown));
    assert_eq!(walk.selected_section(), Some("games"), "wraps forward");
    assert!(walk.press(UiButton::PageUp));
    assert_eq!(walk.selected_section(), Some("settings"), "wraps back");
    assert_eq!(
        walk.take_events(),
        vec![
            WalkEvent::SectionChosen("settings".into()),
            WalkEvent::SectionChosen("games".into()),
            WalkEvent::SectionChosen("settings".into()),
        ]
    );
    assert!(walk.select_section("tools"));
    assert!(!walk.select_section("nowhere"));
    assert!(walk.take_events().is_empty(), "the application's choice raises nothing");
    let mut card = frame(WalkForm::Card);
    assert!(!card.press(UiButton::PageDown), "a card has no sections to step");
    assert_eq!(card.selected_section(), Some("tools"));
    let mut none = Walk::new(WalkForm::BigScreen);
    assert!(!none.press(UiButton::PageUp), "no sections: declined");
}

#[test]
fn lunap_202_4_a_opens_the_detail_on_the_big_screen_and_b_climbs_a_level_at_a_time() {
    let mut walk = frame(WalkForm::BigScreen);
    walk.focus_role(Role::Content);
    walk.take_events();
    assert!(walk.press(UiButton::Accept));
    assert!(walk.is_detail_open());
    assert_eq!(walk.focused(), Some(Role::Detail));
    assert_eq!(walk.take_events(), vec![WalkEvent::DetailOpened, WalkEvent::Focused(Role::Detail)]);
    assert!(!walk.press(UiButton::Accept), "A on the open detail is the host's");
    assert!(walk.press(UiButton::Back), "B closes the page");
    assert!(!walk.is_detail_open());
    assert_eq!(walk.focused(), Some(Role::Content));
    assert!(!walk.press(UiButton::Back), "B at the top is declined: the host's signal to close");
    walk.focus_role(Role::Actions);
    assert!(walk.press(UiButton::Back), "from the actions B gives the content the focus");
    assert_eq!(walk.focused(), Some(Role::Content));

    let mut desktop = frame(WalkForm::Desktop);
    desktop.focus_role(Role::Content);
    assert!(!desktop.press(UiButton::Accept), "on the desktop the detail is beside: A is the host's");
    assert!(desktop.press(UiButton::Back), "on the desktop the content gives the focus to the navigation");
    assert_eq!(desktop.focused(), Some(Role::Navigation));
    assert!(!desktop.press(UiButton::Back));

    let mut beside = frame(WalkForm::BigScreen);
    beside.set_detail(true, true);
    beside.focus_role(Role::Content);
    assert!(!beside.press(UiButton::Accept), "a detail beside the content is not opened");
    assert!(beside.is_shown(Role::Detail));
}

#[test]
fn lunap_202_4_a_rows_page_closes_before_the_detail_and_the_search_climbs_to_the_content() {
    let mut walk = frame(WalkForm::BigScreen);
    walk.show_search(true);
    walk.focus_role(Role::Content);
    walk.press(UiButton::Accept);
    walk.open_page();
    assert!(walk.is_page_open());
    walk.take_events();
    assert!(walk.press(UiButton::Back));
    assert!(!walk.is_page_open());
    assert!(walk.is_detail_open(), "the page closed, the detail stays");
    assert_eq!(walk.take_events(), vec![WalkEvent::PageClosed, WalkEvent::Focused(Role::Content)]);
    walk.focus_role(Role::Search);
    assert!(walk.press(UiButton::Back));
    assert_eq!(walk.focused(), Some(Role::Content));
}

#[test]
fn lunap_202_4_an_actions_own_button_invokes_it_and_a_disabled_one_is_declined() {
    let mut walk = frame(WalkForm::BigScreen);
    assert!(walk.press(UiButton::Search));
    assert_eq!(walk.take_events(), vec![WalkEvent::ActionInvoked("scan".into())]);
    assert!(!walk.press(UiButton::Options), "disabled: declined");
    assert!(!walk.press(UiButton::Menu), "no action owns it: declined");
    assert!(walk.take_events().is_empty());
}

#[test]
fn lunap_202_4_the_swap_is_applied_before_the_grammar() {
    let mut walk = frame(WalkForm::BigScreen);
    walk.set_swap_accept_back(true);
    walk.focus_role(Role::Content);
    assert!(walk.press(UiButton::Back), "the physical B is Accept now: it opens the detail");
    assert!(walk.is_detail_open());
    assert!(walk.press(UiButton::Accept), "and the physical A climbs");
    assert!(!walk.is_detail_open());
}

#[test]
fn lunap_202_4_a_change_of_form_keeps_the_section_and_the_focus_and_closes_the_page_on_the_desktop() {
    let mut walk = frame(WalkForm::BigScreen);
    walk.press(UiButton::PageDown);
    walk.focus_role(Role::Content);
    walk.press(UiButton::Accept);
    walk.set_form(WalkForm::Desktop);
    assert_eq!(walk.selected_section(), Some("settings"));
    assert!(!walk.is_detail_open());
    assert_eq!(walk.focused(), Some(Role::Detail), "the desktop shows the detail beside, so the focus stays in it");
    walk.set_form(WalkForm::BigScreen);
    assert_eq!(walk.focused(), Some(Role::Navigation), "the detail is not shown on the big screen until opened, so the focus moves on");
    let mut plain = Walk::new(WalkForm::BigScreen);
    plain.show_navigation(false);
    assert!(plain.move_focus(1));
    assert_eq!(plain.focused(), Some(Role::Content), "a layout without sections or detail walks its content alone");
    assert!(!plain.press(UiButton::Accept), "no detail: A is the host's");
}
