// EmuSen_Kit.md §7.23, EmuSen_Interface.md §18.6: one notice at a time, a second replacing the first; over gameplay only Saved, Loaded and the screenshot.

use emusen_kit_core::notice::{Notice, Notices, Tone, HOLD_MS};

fn notice(words: &str, over_game: bool) -> Notice {
    Notice { words: words.to_string(), tone: Tone::Success, over_game }
}

#[test]
fn a_second_notice_replaces_the_first_and_the_hold_ends_it() {
    let mut notices = Notices::default();
    assert!(notices.raise(notice("Screenshot saved", true)));
    assert!(notices.raise(notice("Core approved", false)));
    assert_eq!(notices.shown().map(|n| n.words.as_str()), Some("Core approved"));
    notices.expire();
    assert!(notices.shown().is_none());
    assert_eq!(HOLD_MS, 2000);
}

#[test]
fn over_gameplay_only_the_game_s_own_notices_show_and_the_rest_wait() {
    let mut notices = Notices::default();
    assert!(!notices.set_game_running(false), "no change");
    notices.raise(notice("Core approved", false));
    assert!(notices.set_game_running(true), "an unfit notice leaves as gameplay starts");
    assert!(notices.shown().is_none());
    assert!(notices.raise(notice("State saved to Slot 3", true)));
    assert!(!notices.raise(notice("Library read", false)), "waits");
    assert_eq!(notices.shown().map(|n| n.words.as_str()), Some("State saved to Slot 3"));
    assert_eq!(notices.waiting().map(|n| n.words.as_str()), Some("Library read"), "the later waiting one replaced the first");
    notices.expire();
    assert!(notices.set_game_running(false));
    assert_eq!(notices.shown().map(|n| n.words.as_str()), Some("Library read"), "shown once a card or a screen is up");
    assert!(notices.waiting().is_none());
}

#[test]
fn a_notice_with_no_words_raises_nothing() {
    let mut notices = Notices::default();
    assert!(notices.raise(notice("Core approved", false)));
    assert!(!notices.raise(notice("", true)), "empty words say nothing");
    assert_eq!(notices.shown().map(|n| n.words.as_str()), Some("Core approved"), "the notice shown stays");
}
