// EmuSen_Kit.md §7.4: the router's queue takes effect at a pump, the grammar's repeats run on the stated clock, and the swap is applied once, here.

use std::time::Duration;

use emusen_kit_core::input::{RepeatTiers, UiButton};
use emusen_kit_qt::router::{PadEvent, Router};

fn ms(v: u64) -> Duration {
    Duration::from_millis(v)
}

fn buttons(router: &mut Router, now: u64) -> Vec<(UiButton, bool, u32)> {
    router.pump(ms(now)).presses.into_iter().map(|p| (p.button, p.repeat, p.jump)).collect()
}

#[test]
fn kit_7_4_a_queued_event_takes_effect_at_the_next_pump_only() {
    let mut router = Router::new(RepeatTiers::OWN);
    router.push(PadEvent { button: UiButton::Accept, down: true });
    assert!(!router.is_held(UiButton::Accept), "nothing changes before a pump");
    assert_eq!(buttons(&mut router, 0), vec![(UiButton::Accept, false, 1)]);
    assert!(router.is_held(UiButton::Accept));
    assert_eq!(buttons(&mut router, 100), vec![], "Accept never repeats");
    router.push(PadEvent { button: UiButton::Accept, down: false });
    assert_eq!(buttons(&mut router, 120), vec![]);
    assert!(!router.is_held(UiButton::Accept));
}

#[test]
fn kit_7_4_a_held_direction_repeats_on_the_own_tiers() {
    let mut router = Router::new(RepeatTiers::OWN);
    router.push(PadEvent { button: UiButton::Down, down: true });
    assert_eq!(buttons(&mut router, 0), vec![(UiButton::Down, false, 1)]);
    assert_eq!(buttons(&mut router, 399), vec![]);
    assert_eq!(buttons(&mut router, 400), vec![(UiButton::Down, true, 1)]);
    assert_eq!(buttons(&mut router, 479), vec![]);
    assert_eq!(buttons(&mut router, 480), vec![(UiButton::Down, true, 1)]);
}

#[test]
fn kit_7_4_the_swap_is_applied_to_the_presses_once() {
    let mut router = Router::new(RepeatTiers::OWN);
    router.set_swap_accept_back(true);
    router.push(PadEvent { button: UiButton::Accept, down: true });
    router.push(PadEvent { button: UiButton::Up, down: true });
    let pressed: Vec<UiButton> = buttons(&mut router, 0).into_iter().map(|(b, _, _)| b).collect();
    assert_eq!(pressed, vec![UiButton::Up, UiButton::Accept.swapped(true)]);
    assert_eq!(pressed[1], UiButton::Back);
    assert!(router.is_held(UiButton::Accept), "the held state is the physical button");
}

#[test]
fn kit_7_4_the_menu_chord_is_reported_once_and_its_buttons_forgotten() {
    let mut router = Router::new(RepeatTiers::OWN);
    router.push(PadEvent { button: UiButton::Guide, down: true });
    let pumped = router.pump(ms(0));
    assert!(pumped.menu_chord);
    assert!(pumped.presses.is_empty(), "the button that opened the menu reaches nothing under it");
    let again = router.pump(ms(50));
    assert!(!again.menu_chord);
    assert!(again.presses.is_empty());
}

#[test]
fn kit_7_4_the_tiers_can_be_changed() {
    let mut router = Router::new(RepeatTiers::OWN);
    router.set_tiers(RepeatTiers::ESDE_CAROUSEL);
    router.push(PadEvent { button: UiButton::Right, down: true });
    assert_eq!(buttons(&mut router, 0).len(), 1);
    assert_eq!(buttons(&mut router, 450), vec![], "the carousel waits 500 ms");
    assert_eq!(buttons(&mut router, 500), vec![(UiButton::Right, true, 1)]);
}
