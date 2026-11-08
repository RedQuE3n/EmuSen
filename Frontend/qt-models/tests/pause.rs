//! The Pause Card's pages as plain data, no Qt (EmuSen_Qt.md §7.1 step 7, §7.2, §8.2): Mistress's rows and words from
//! the session's facts, the card's pane, the reel, Save and Load with their slots, what a choice, a step and B do, and the
//! declared actions read back by the walk's parser.

use emusen_crystal_client::shapes::{Moment, Picture, RewindReel, SessionInfo};
use emusen_kit_core::input::{UiButton, WalkAction};
use emusen_qt_models::help::{action_lines, entries, Screen};
use emusen_qt_models::pause::{facts_of, Card, Facts, Outcome, Page, PaneKind, SlotFact, Tile};
use emusen_qt_models::reel::{interval_for, marks_of, stops_of, Reel};

fn card(facts: Facts) -> Card {
    Card { facts, ..Card::default() }
}

fn shown(card: &Card) -> Vec<(String, String, bool, bool, bool)> {
    card.rows().into_iter().map(|r| (r.label, r.value, r.steps, r.opens, r.section_end)).collect()
}

fn at(card: &Card, label: &str) -> usize {
    card.rows().iter().position(|r| r.label == label).unwrap()
}

/// A reel of one moment with a picture every half second for `seconds`, at 60 frames a second, the present at its end.
fn reeling(seconds: u64) -> Facts {
    let held: Vec<u64> = (0..=seconds * 2).map(|i| 1000 + i * 30).collect();
    let now = *held.last().unwrap() + 10;
    Facts { held, now, rewind_seconds: seconds as f64, pictures: "image://session/1/".into(), ..Facts::default() }
}

fn saved(slots: &[u32]) -> Facts {
    let slots = (1..=8)
        .map(|slot| {
            let exists = slots.contains(&slot);
            SlotFact {
                slot,
                exists,
                when: if exists { format!("Sun 8 Nov, 21:4{slot}") } else { String::new() },
                picture: if exists { format!("image://session/1/file/s{slot}.png") } else { String::new() },
            }
        })
        .collect();
    Facts { slots, ..Facts::default() }
}

#[test]
fn the_first_page_holds_mistresss_rows_in_her_words() {
    let s = |t: &str| t.to_string();
    assert_eq!(
        shown(&card(Facts::default())),
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
    let three = Facts { slot: 3, rewind_seconds: 28.6, ..saved(&[3]) };
    let values: Vec<String> = card(three.clone()).rows().into_iter().map(|r| r.value).collect();
    assert_eq!(values[1..4], ["28 s", "Slot 3", "Slot 3"], "the reel's whole seconds and a saved slot");
    let withheld = Facts { rewind_why: Some("not kept for this engine".into()), rewind_seconds: 40.0, ..Facts::default() };
    assert_eq!(card(withheld).rows()[1].value, "not kept for this engine", "why it cannot open, before its seconds");
    assert_eq!(Facts::from_json(&three.to_json()), three, "the facts cross as plain data whole");
    assert_eq!(Facts::from_json(&reeling(4).to_json()), reeling(4));
    let c = card(Facts::default());
    assert_eq!((c.title("Cobalt Harbor (Synthetic)"), c.breadcrumb("Cobalt Harbor (Synthetic)")), ("Cobalt Harbor (Synthetic)".into(), String::new()));
}

#[test]
fn the_facts_are_read_as_mistress_reads_them() {
    let info = |capture: Option<bool>| SessionInfo { rewind_capture: capture, frame_rate_hz: 60.0, total_frames: 1810, ..SessionInfo::default() };
    let moment = |frame: u64, pictured: bool| Moment { frame, picture: pictured.then(Picture::default), ..Moment::default() };
    let reel = RewindReel { snapshot_bytes: 4096, moments: vec![moment(60, false), moment(600, true), moment(1800, true)], ..RewindReel::default() };
    let facts = facts_of(&info(Some(true)), Some(&reel));
    assert_eq!((facts.rewind_why.as_deref(), facts.rewind_seconds), (None, 29.0));
    assert_eq!((facts.held.as_slice(), facts.now, facts.hz), ([600, 1800].as_slice(), 1810, 60.0), "the moments with a picture and the present");
    let empty = RewindReel::default();
    assert_eq!(facts_of(&info(Some(true)), Some(&empty)).rewind_why.as_deref(), Some("nothing to go back to yet"));
    assert_eq!(facts_of(&info(Some(false)), Some(&reel)).rewind_why.as_deref(), Some("not kept for this engine"));
    let withheld = SessionInfo { rewind_withheld: Some("the core keeps no states".into()), ..info(Some(false)) };
    assert_eq!(facts_of(&withheld, None).rewind_why.as_deref(), Some("the core keeps no states"));
    assert_eq!((interval_for(60.0), interval_for(50.0), interval_for(59.94), interval_for(1.0)), (30, 25, 30, 1), "two snapshots a second");
}

#[test]
fn a_choice_a_step_and_b_answer_by_the_cards_rule() {
    let mut c = card(Facts::default());
    assert_eq!(c.choose(at(&c, "Resume")), Outcome::Resume);
    assert_eq!(c.choose(at(&c, "Leave")), Outcome::Shown);
    assert_eq!((c.page, c.title("Cobalt Harbor"), c.breadcrumb("Cobalt Harbor")), (Page::Leave, "Leave".into(), "Cobalt Harbor".into()));
    assert_eq!(c.choose(at(&c, "Back to Library")), Outcome::Library);
    assert_eq!(c.back(), Outcome::Shown);
    assert_eq!((c.page, c.current), (Page::First, 6), "B on Leave goes back to its row");
    assert_eq!(c.back(), Outcome::Resume, "B on the first page resumes");
    assert_eq!(c.choose(at(&c, "Rewind")), Outcome::Nothing, "a reel with nothing in it does not open");
    assert_eq!(c.choose(at(&c, "Game Options")), Outcome::Nothing, "a later step's");
    assert_eq!(c.act("resume"), Outcome::Resume, "Start's declared action resumes");

    let save = at(&c, "Save State");
    assert_eq!(c.step(save, -1, false), Outcome::Slot(8), "Left from slot 1 wraps to 8, as Mistress's slots do");
    assert_eq!(c.step(save, 1, false), Outcome::Slot(2));
    assert_eq!(c.choose(save), Outcome::Save(1), "A saves to the slot and closes the card");
    let load = at(&c, "Load State");
    assert_eq!(c.choose(load), Outcome::Nothing, "an empty slot loads nothing");
    let mut c = card(Facts { slot: 4, ..saved(&[4]) });
    assert_eq!(c.choose(load), Outcome::Load(4));
    let speed = at(&c, "Speed");
    assert_eq!((c.step(speed, 1, false), c.step(speed, -1, false), c.choose(speed)), (Outcome::Speed(300), Outcome::Speed(25), Outcome::Speed(300)));
    let mut fast = card(Facts { speed: 300, ..Facts::default() });
    assert_eq!(fast.step(speed, 1, false), Outcome::Nothing, "the fastest stays");
}

#[test]
fn save_and_load_show_three_slots_around_the_current_and_say_what_a_does() {
    let mut c = card(Facts { slot: 1, ..saved(&[2]) });
    c.focus(at(&c, "Save State"));
    let pane = c.pane();
    assert_eq!(pane.kind, PaneKind::Slots);
    let tile = |name: &str, when: &str, picture: &str, ringed: bool| Tile { picture: picture.into(), name: name.into(), when: when.into(), ringed };
    assert_eq!(
        pane.tiles,
        [tile("Slot 1", "Empty", "", true), tile("Slot 2", "Sun 8 Nov, 21:42", "image://session/1/file/s2.png", false), tile("Slot 3", "Empty", "", false)]
    );
    assert_eq!(pane.sentence, "A saves to Slot 1.");
    assert_eq!(c.help(), Screen::PauseSlots, "Change slot on the bar");
    c.facts.slot = 5;
    assert_eq!(c.pane().tiles.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(), ["Slot 4", "Slot 5", "Slot 6"], "the current in the middle");
    c.facts.slot = 8;
    assert_eq!(c.pane().tiles.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(), ["Slot 6", "Slot 7", "Slot 8"], "as near as the end allows");
    c.facts.slot = 2;
    assert_eq!(c.pane().sentence, "A saves to Slot 2, replacing it.");
    c.focus(at(&c, "Load State"));
    assert_eq!(c.pane().sentence, "A loads Slot 2.");
    c.facts.slot = 3;
    assert_eq!(c.pane().sentence, "Slot 3 is empty.");
    c.focus(at(&c, "Speed"));
    assert_eq!((c.pane().kind, c.help()), (PaneKind::None, Screen::Pause));
}

#[test]
fn the_rewind_row_shows_the_newest_pictures_or_why_it_cannot() {
    let mut c = card(reeling(28));
    c.focus(at(&c, "Rewind"));
    let pane = c.pane();
    assert_eq!(pane.kind, PaneKind::Reel);
    let names: Vec<&str> = pane.tiles.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["1 s ago", "0.5 s ago", "Now"], "the three newest, oldest first, named back from the newest");
    assert_eq!(pane.tiles[2].picture, format!("image://session/1/reel/{}", 1000 + 56 * 30));
    assert_eq!(pane.sentence, "28 s available. A opens the reel; Left and Right step through it.");
    let mut short = card(Facts { held: vec![100], now: 110, rewind_seconds: 0.2, ..reeling(0) });
    short.focus(1);
    assert_eq!(short.pane().sentence, "Under a second available. A opens the reel; Left and Right step through it.");
    let mut none = card(Facts { rewind_why: Some("nothing to go back to yet".into()), ..Facts::default() });
    none.focus(1);
    assert_eq!((none.pane().tiles.len(), none.pane().sentence), (0, "Rewind: nothing to go back to yet".to_string()));
}

#[test]
fn the_reel_opens_on_its_first_stop_steps_by_stops_and_goes_back() {
    let mut c = card(reeling(28));
    assert_eq!(c.choose(at(&c, "Rewind")), Outcome::Shown);
    assert_eq!((c.page, c.current, c.title("G"), c.breadcrumb("G")), (Page::Reel, 0, "Rewind".into(), "G".into()));
    let words = |c: &Card| c.rows().into_iter().map(|r| format!("{}|{}|{}", r.label, r.value, r.dimmed)).collect::<Vec<_>>();
    assert_eq!(words(&c), ["Go back to 1.7 s ago||false", "Moment|1.7 s ago|false", "Hold ◂ to scrub||true"]);
    assert_eq!(c.help(), Screen::PauseReel);
    let strip = |c: &Card| c.pane().tiles.iter().map(|t| format!("{}{}", t.name, if t.ringed { "*" } else { "" })).collect::<Vec<_>>();
    assert_eq!(strip(&c), ["Now", "1.7 s*", "4.2 s", "8.2 s", "12 s", "28 s"], "the present, the stops nearest 1.5, 4, 8 and 12 s, the oldest");
    let pane = c.pane();
    assert_eq!((pane.kind, pane.big.as_str()), (PaneKind::Moment, pane.tiles[1].picture.as_str()));
    assert_eq!(pane.sentence, "1.7 s ago. A goes back there; the moments after it are kept until you play on.");
    assert_eq!(c.step(1, 1, false), Outcome::Shown);
    assert_eq!(c.step(1, 1, false), Outcome::Shown);
    assert_eq!(c.step(1, 1, false), Outcome::Shown);
    assert_eq!(c.step(1, 1, false), Outcome::Shown);
    assert_eq!(c.rows()[1].value, "28 s ago", "28 s back is four presses from the first stop");
    assert_eq!(c.step(1, 1, false), Outcome::Nothing, "the oldest stays");
    assert_eq!(c.step(1, -1, true), Outcome::Shown);
    assert_eq!(strip(&c).last().unwrap(), "27 s*", "a moment scrubbed to stands in for the nearer stop");
    assert_eq!(c.stride(-1, false), Outcome::Shown);
    assert_eq!(c.rows()[1].value, "22 s ago", "a shoulder strides five seconds");
    assert_eq!(c.stride(-1, true), Outcome::Shown);
    assert_eq!((c.rows()[0].label.as_str(), c.choose(0)), ("Stay at Now", Outcome::Resume), "A at Now closes the card and changes nothing");
    c.stride(1, true);
    assert_eq!(c.choose(0), Outcome::RewindTo(1000), "A goes back to the moment's frame");
    assert_eq!(c.back(), Outcome::Shown);
    assert_eq!((c.page, c.current, c.reel.is_none()), (Page::First, 1, true), "B goes back to Rewind's row");
}

#[test]
fn the_reels_stops_follow_mistresss_rule() {
    let marks = marks_of(&[0, 30, 60, 90], 100, 60.0);
    assert_eq!(marks.iter().map(|m| m.frame).collect::<Vec<_>>(), [100, 90, 60, 30, 0], "newest first, the present first");
    assert_eq!(stops_of(&marks), [0, 4], "the moment nearest 1.5 s back is the oldest, so the strip is the present and it");
    assert!(Reel::open(marks_of(&[], 100, 60.0)).is_none(), "nothing before the present opens no reel");
}

#[test]
fn the_help_bar_and_the_declared_actions_are_mistresss() {
    let labels = |screen| entries(screen).iter().map(|(b, l, _)| (*b, *l)).collect::<Vec<_>>();
    assert_eq!(
        labels(Screen::Pause),
        [(UiButton::Accept, "Select"), (UiButton::Back, "Resume"), (UiButton::Screensaver, "Screenshot"), (UiButton::Menu, "Resume")]
    );
    assert_eq!(labels(Screen::PausePage)[..2], [(UiButton::Accept, "Choose"), (UiButton::Back, "Back")]);
    assert_eq!(labels(Screen::PauseSlots)[2], (UiButton::Left, "Change slot"));
    assert_eq!(labels(Screen::PauseReel), [(UiButton::Accept, "Go back"), (UiButton::Back, "Back"), (UiButton::Left, "Moment"), (UiButton::Screensaver, "Screenshot")]);
    let declared = WalkAction::from_lines(&action_lines(Screen::Pause));
    let keys: Vec<(&str, Option<UiButton>)> = declared.iter().map(|a| (a.key.as_str(), a.button)).collect();
    assert_eq!(keys, [("screenshot", Some(UiButton::Screensaver)), ("resume", Some(UiButton::Menu))]);
}

#[test]
fn leave_asks_each_end_first_on_the_card_forms_question_page() {
    let mut c = card(Facts::default());
    c.choose(at(&c, "Leave"));
    let labels: Vec<String> = c.rows().into_iter().map(|r| r.label).collect();
    assert_eq!(labels, ["Back to Library", "Restart Game...", "Quit Game...", "Quit EmuSen..."], "Mistress's LeaveRows");
    for (row, title, sentence, yes) in [
        (1, "Restart Game?", "The game starts again from the beginning. Progress since your last save is lost.", Outcome::Restart),
        (2, "Quit Game?", "The game closes and the library comes back.", Outcome::QuitGame),
        (3, "Quit EmuSen?", "The game closes, then EmuSen.", Outcome::QuitEmuSen),
    ] {
        assert_eq!(c.choose(row), Outcome::Shown);
        assert_eq!((c.title("G"), c.breadcrumb("G"), c.current), (title.to_string(), sentence.to_string(), 0), "No first");
        assert_eq!(c.rows().into_iter().map(|r| r.label).collect::<Vec<_>>(), ["No", "Yes"]);
        assert_eq!(c.choose(0), Outcome::Shown, "No goes back");
        assert_eq!((c.page, c.current), (Page::Leave, row), "on the row that asked");
        c.choose(row);
        assert_eq!(c.back(), Outcome::Shown, "B goes back too");
        assert_eq!((c.page, c.current), (Page::Leave, row));
        c.choose(row);
        assert_eq!(c.choose(1), yes);
        c.back();
    }
}
