// EmuSen_Kit.md §7.4: the router's queue takes effect at a pump, the grammar's repeats run on the stated clock, and the swap is applied once, here.

use std::time::Duration;

use emusen_kit_core::input::{RepeatTiers, UiButton};
use emusen_kit_qt::qt_keys;
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

#[test]
fn kit_7_17_while_a_game_owns_the_pad_only_the_chord_leaves_and_nothing_held_then_presses_after() {
    let mut router = Router::new(RepeatTiers::OWN);
    router.set_chord_only(true);
    router.push(PadEvent { button: UiButton::Down, down: true });
    assert_eq!(buttons(&mut router, 0), vec![], "the walk sees no game input");
    assert_eq!(buttons(&mut router, 1000), vec![], "nor a repeat of it");
    router.push(PadEvent { button: UiButton::Guide, down: true });
    assert!(router.pump(ms(1016)).menu_chord, "the chord reaches the program");
    router.set_chord_only(false);
    assert_eq!(buttons(&mut router, 1032), vec![], "what was held over the game is not pressed when the interface has the pad");
    router.push(PadEvent { button: UiButton::Down, down: false });
    router.push(PadEvent { button: UiButton::Guide, down: false });
    assert_eq!(buttons(&mut router, 1048), vec![]);
    router.push(PadEvent { button: UiButton::Down, down: true });
    assert_eq!(buttons(&mut router, 1064), vec![(UiButton::Down, false, 1)]);
}

// EmuSen_Kit.md §7.17: the chord that takes the pad from a game is one chord for as long as it is held, whoever owns the pad
// at the next pump; reported at every pump it paused a game and resumed it again while the button was still down.
#[test]
fn kit_7_17_the_chord_held_over_a_game_is_reported_once_until_let_go() {
    for takes_the_pad in [true, false] {
        let mut router = Router::new(RepeatTiers::OWN);
        router.set_chord_only(true);
        router.push(PadEvent { button: UiButton::Guide, down: true });
        assert!(router.pump(ms(0)).menu_chord, "the chord reaches the program");
        router.set_chord_only(!takes_the_pad);
        assert!(!router.pump(ms(16)).menu_chord, "still held at the next pump, the interface owning the pad: {takes_the_pad}");
        assert!(!router.pump(ms(32)).menu_chord, "and at the one after");
        router.push(PadEvent { button: UiButton::Guide, down: false });
        assert!(!router.pump(ms(48)).menu_chord, "let go");
        router.push(PadEvent { button: UiButton::Guide, down: true });
        assert!(router.pump(ms(64)).menu_chord, "pressed again, a chord again");
        // Let go and pressed again between two pumps is a new chord too: the queue shows what the state at the pump cannot.
        router.push(PadEvent { button: UiButton::Guide, down: false });
        router.push(PadEvent { button: UiButton::Guide, down: true });
        assert!(router.pump(ms(80)).menu_chord, "let go and pressed again before the next pump");
        assert!(!router.pump(ms(96)).menu_chord, "and one chord while that press is held");
    }
    // The two middle buttons together are the chord of a pad with no guide button, whether or not Guide is read for its chords.
    for guide_chords in [false, true] {
        let mut router = Router::new(RepeatTiers::OWN);
        router.set_guide_chords(guide_chords);
        router.set_chord_only(true);
        router.push(PadEvent { button: UiButton::Options, down: true });
        router.push(PadEvent { button: UiButton::Menu, down: true });
        assert!(router.pump(ms(0)).menu_chord, "Guide's chords read: {guide_chords}");
        assert!(!router.pump(ms(16)).menu_chord, "the two middle buttons still held, the game still owning the pad: {guide_chords}");
        router.set_chord_only(false);
        assert!(!router.pump(ms(32)).menu_chord, "and with the interface owning it: {guide_chords}");
    }
}

// EmuSen_Kit.md §7.4: the router has work only while an event waits or a button is held, so a frame's clock sleeps otherwise.
#[test]
fn kit_7_4_the_router_is_pending_only_while_an_event_waits_or_a_button_is_held() {
    let mut router = Router::new(RepeatTiers::OWN);
    assert!(!router.pending(), "idle");
    router.push(PadEvent { button: UiButton::Down, down: true });
    assert!(router.pending(), "an event waits");
    router.pump(Duration::from_millis(0));
    assert!(router.pending(), "Down is held, its repeats are due");
    router.push(PadEvent { button: UiButton::Down, down: false });
    router.pump(Duration::from_millis(10));
    assert!(!router.pending(), "let go and applied");
}

// EmuSen_Kit.md §7.26: with the Guide chords on over a game, Guide alone opens the menu as it is let go, Guide with Up,
// Down or the right shoulder is a chord and no menu, and Options with Menu open it at once only while Guide is up.
#[test]
fn kit_7_26_over_a_game_guide_is_read_for_its_chords() {
    let mut router = Router::new(RepeatTiers::OWN);
    router.set_chord_only(true);
    router.set_guide_chords(true);
    let mut at = 0;
    let mut press = |router: &mut Router, button: UiButton, down: bool| {
        router.push(PadEvent { button, down });
        at += 16;
        router.pump(ms(at))
    };
    assert!(!router.guide_held());
    let pumped = press(&mut router, UiButton::Guide, true);
    assert!(!pumped.menu_chord, "Guide going down opens nothing yet");
    assert!(router.guide_held(), "the game is to hear nothing while Guide is held (EmuSen_Kit.md §7.26)");
    let pumped = press(&mut router, UiButton::Up, true);
    assert_eq!((pumped.guide_chord, pumped.menu_chord), (Some(UiButton::Up), false));
    assert!(pumped.presses.is_empty(), "the chord is no press");
    press(&mut router, UiButton::Up, false);
    let pumped = press(&mut router, UiButton::PageDown, true);
    assert_eq!(pumped.guide_chord, Some(UiButton::PageDown));
    press(&mut router, UiButton::PageDown, false);
    let pumped = press(&mut router, UiButton::Guide, false);
    assert!(!pumped.menu_chord, "a chord made, so Guide let go opens nothing");
    assert!(!router.guide_held());

    press(&mut router, UiButton::Guide, true);
    let pumped = press(&mut router, UiButton::Guide, false);
    assert!(pumped.menu_chord, "Guide alone opens the menu as it is let go");

    press(&mut router, UiButton::Options, true);
    let pumped = press(&mut router, UiButton::Menu, true);
    assert!(pumped.menu_chord, "the two middle buttons open it at once");
    press(&mut router, UiButton::Options, false);
    press(&mut router, UiButton::Menu, false);

    router.set_guide_chords(false);
    let pumped = press(&mut router, UiButton::Guide, true);
    assert!(pumped.menu_chord, "with the chords off Guide opens the menu as it goes down");
    assert!(!router.guide_held(), "and the game is not kept from the pad");
    assert_eq!(press(&mut router, UiButton::Up, true).guide_chord, None);
}

// EmuSen_Kit.md §7.28: a release is raised once, at the pump that applies it, only after a press that was raised, and
// with the swap as the press had it; nothing held over a game or into the menu chord is released.
#[test]
fn kit_7_28_a_release_raises_once_after_its_press() {
    let mut router = Router::new(RepeatTiers::OWN);
    router.set_swap_accept_back(true);
    router.push(PadEvent { button: UiButton::Down, down: true });
    router.push(PadEvent { button: UiButton::Accept, down: true });
    assert!(router.pump(ms(0)).released.is_empty(), "nothing let go yet");
    assert!(router.pump(ms(500)).released.is_empty(), "a repeat is no release");
    router.push(PadEvent { button: UiButton::Down, down: false });
    router.push(PadEvent { button: UiButton::Accept, down: false });
    assert_eq!(router.pump(ms(510)).released, vec![UiButton::Down, UiButton::Back], "once each, Accept swapped as its press was");
    assert!(router.pump(ms(520)).released.is_empty(), "and only once");
    router.push(PadEvent { button: UiButton::Up, down: false });
    assert!(router.pump(ms(530)).released.is_empty(), "a button never pressed is never released");

    router.set_chord_only(true);
    router.push(PadEvent { button: UiButton::Left, down: true });
    router.pump(ms(540));
    router.set_chord_only(false);
    router.push(PadEvent { button: UiButton::Left, down: false });
    assert!(router.pump(ms(550)).released.is_empty(), "held over a game: no press, so no release");

    router.push(PadEvent { button: UiButton::Guide, down: true });
    assert!(router.pump(ms(560)).menu_chord);
    router.push(PadEvent { button: UiButton::Guide, down: false });
    assert!(router.pump(ms(570)).released.is_empty(), "the chord's button reached nothing, so is not released");
}

// EmuSen_Kit.md §7.28: a key is the pad's press by the kit's key map, through the same queue: Return is Accept whatever
// the swap, a key the map does not know is declined, and a key reaches nothing while a game owns the pad.
#[test]
fn kit_7_28_a_key_is_a_press_and_its_release_through_the_queue() {
    const RETURN: i32 = 0x0100_0004;
    const ESCAPE: i32 = 0x0100_0000;
    const TAB: i32 = 0x0100_0001;
    const LETTER_A: i32 = 0x41;
    let mut router = Router::new(RepeatTiers::OWN);
    router.set_swap_accept_back(true);
    assert!(router.push_key(RETURN, 0));
    assert!(router.pending(), "a key waits for a pump");
    let pumped = router.pump(ms(0));
    assert_eq!((pumped.presses.len(), pumped.keyed.clone()), (0, vec![UiButton::Accept]), "Return is Accept, not swapped");
    assert!(pumped.released.is_empty(), "a key's release is its own press's, raised with it");
    assert!(!router.pending(), "nothing held after a key");
    assert!(!router.push_key(TAB, 0), "Tab walks and is no press");
    assert!(!router.push_key(LETTER_A, 0), "a key the map does not know");
    assert!(router.pump(ms(10)).keyed.is_empty());
    assert!(router.push_key(qt_keys::KEY_ENTER, 0));
    assert!(router.push_key(ESCAPE, 0));
    assert_eq!(router.pump(ms(20)).keyed, vec![UiButton::Accept, UiButton::Back]);
    router.set_chord_only(true);
    assert!(router.push_key(RETURN, 0));
    assert!(router.pump(ms(30)).keyed.is_empty(), "nothing while a game owns the pad");
}
