//! The Pause Card's pages as plain data, no Qt (EmuSen_Qt.md §7.1 step 7, §8.2): Mistress's rows and words from the
//! session's facts, what a choice and B do, and the declared actions read back by the walk's parser.

use emusen_crystal_client::shapes::{Moment, RewindReel, SessionInfo, StateSlot};
use emusen_kit_core::input::{UiButton, WalkAction};
use emusen_qt_models::help::{action_lines, entries, Screen};
use emusen_qt_models::pause::{back, breadcrumb, choose, facts_of, rows, title, Choice, Facts, Page};

fn shown(page: Page, facts: &Facts) -> Vec<(String, String, bool, bool, bool)> {
    rows(page, facts).into_iter().map(|r| (r.label, r.value, r.steps, r.opens, r.section_end)).collect()
}

#[test]
fn the_first_page_holds_mistresss_rows_in_her_words() {
    let s = |t: &str| t.to_string();
    assert_eq!(
        shown(Page::First, &Facts::default()),
        vec![
            (s("Resume"), s(""), false, false, false),
            (s("Rewind"), s(""), false, false, false),
            (s("Save State"), s("Slot 1 (Empty)"), true, false, false),
            (s("Load State"), s("Slot 1 (Empty)"), true, false, false),
            (s("Speed"), s("100%"), true, false, true),
            (s("Game Options"), s(""), false, true, false),
            (s("Leave"), s(""), false, true, false),
        ]
    );
    let saved = Facts { slot: 3, saved: true, rewind_seconds: 28.6, ..Facts::default() };
    let values: Vec<String> = rows(Page::First, &saved).into_iter().map(|r| r.value).collect();
    assert_eq!(values[1..4], ["28 s", "Slot 3", "Slot 3"], "the reel's whole seconds and a saved slot");
    let withheld = Facts { rewind_why: Some("not kept for this engine".into()), rewind_seconds: 40.0, ..Facts::default() };
    assert_eq!(rows(Page::First, &withheld)[1].value, "not kept for this engine", "why it cannot open, before its seconds");
    assert_eq!(Facts::from_json(&saved.to_json()), saved, "the facts cross as plain data whole");
    assert_eq!(title(Page::First, "Cobalt Harbor (Synthetic)"), "Cobalt Harbor (Synthetic)");
    assert_eq!(breadcrumb(Page::First, "Cobalt Harbor (Synthetic)"), "");
}

#[test]
fn the_facts_are_read_as_mistress_reads_them() {
    let info = |capture: Option<bool>| SessionInfo { rewind_capture: capture, frame_rate_hz: 60.0, ..SessionInfo::default() };
    let moment = |frame: u64| Moment { frame, ..Moment::default() };
    let reel = RewindReel { snapshot_bytes: 4096, moments: vec![moment(60), moment(600), moment(1800)], ..RewindReel::default() };
    let slots = [StateSlot { slot: 1, exists: false, ..StateSlot::default() }, StateSlot { slot: 2, exists: true, ..StateSlot::default() }];
    let facts = facts_of(&info(Some(true)), Some(&reel), &slots, 2, 100);
    assert_eq!((facts.rewind_why.as_deref(), facts.rewind_seconds, facts.saved), (None, 29.0, true));
    assert!(!facts_of(&info(Some(true)), Some(&reel), &slots, 1, 100).saved, "slot 1 holds nothing");
    let empty = RewindReel::default();
    assert_eq!(facts_of(&info(Some(true)), Some(&empty), &slots, 1, 100).rewind_why.as_deref(), Some("nothing to go back to yet"));
    assert_eq!(facts_of(&info(Some(false)), Some(&reel), &slots, 1, 100).rewind_why.as_deref(), Some("not kept for this engine"));
    let withheld = SessionInfo { rewind_withheld: Some("the core keeps no states".into()), ..info(Some(false)) };
    assert_eq!(facts_of(&withheld, None, &slots, 1, 100).rewind_why.as_deref(), Some("the core keeps no states"));
}

#[test]
fn a_choice_and_b_answer_by_the_cards_rule() {
    let key_of = |page: Page, label: &str| rows(page, &Facts::default()).into_iter().find(|r| r.label == label).unwrap().key;
    assert_eq!(choose(key_of(Page::First, "Resume")), Choice::Resume);
    assert_eq!(choose(key_of(Page::First, "Leave")), Choice::Turn(Page::Leave));
    assert_eq!(choose(key_of(Page::Leave, "Back to Library")), Choice::Library);
    for owed in ["Rewind", "Save State", "Load State", "Speed", "Game Options"] {
        assert_eq!(choose(key_of(Page::First, owed)), Choice::Owed, "{owed} is a later step's");
    }
    assert_eq!(back(Page::First), None, "B on the first page resumes");
    assert_eq!(back(Page::Leave), Some((Page::First, key_of(Page::First, "Leave"))), "B on Leave goes back to its row");
    assert_eq!(shown(Page::Leave, &Facts::default()).len(), 1, "Leave holds Back to Library alone until stage 2");
    assert_eq!((title(Page::Leave, "Cobalt Harbor"), breadcrumb(Page::Leave, "Cobalt Harbor")), ("Leave".into(), "Cobalt Harbor".into()));
}

#[test]
fn the_help_bar_and_the_declared_actions_are_mistresss() {
    let labels = |screen| entries(screen).iter().map(|(b, l, _)| (*b, *l)).collect::<Vec<_>>();
    assert_eq!(
        labels(Screen::Pause),
        [(UiButton::Accept, "Select"), (UiButton::Back, "Resume"), (UiButton::Screensaver, "Screenshot"), (UiButton::Menu, "Resume")]
    );
    assert_eq!(labels(Screen::PausePage)[..2], [(UiButton::Accept, "Choose"), (UiButton::Back, "Back")]);
    let declared = WalkAction::from_lines(&action_lines(Screen::Pause));
    let keys: Vec<(&str, Option<UiButton>)> = declared.iter().map(|a| (a.key.as_str(), a.button)).collect();
    assert_eq!(keys, [("screenshot", Some(UiButton::Screensaver)), ("resume", Some(UiButton::Menu))]);
    assert_eq!(choose("resume"), Choice::Resume, "Start's declared action resumes");
}
