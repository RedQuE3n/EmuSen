//! Unit tests for Wave 6 sys1: Power runner mock, Eclipse Power/Quit HomeCard pages, NoticePresenter parsing, and #14 Delete Slot wording.

use emusen_qt_models::card::{pages_of, Choice, YES};
use emusen_qt_models::game_page::{delete_when, GamePage, Outcome, Read, Slot, State};
use emusen_qt_models::help::Screen;
use emusen_qt_models::home_card::{self, Facts};
use emusen_qt_models::marks;
use emusen_qt_models::notice_presenter::{icon_for, split_words, take_pending_action, OPEN_ITS_FOLDER};
use emusen_qt_models::power::{execute, execute_desktop, Action, Runner, SystemctlRunner};
use emusen_qt_models::question::back_label;
use std::sync::Mutex;

struct MockRunner {
    log: Mutex<Vec<String>>,
    fail: Option<String>,
}

impl Runner for MockRunner {
    fn run(&self, verb: &str) -> Result<(), String> {
        self.log.lock().unwrap().push(verb.to_owned());
        match &self.fail {
            Some(why) => Err(why.clone()),
            None => Ok(()),
        }
    }
}

#[test]
fn power_execute_saves_before_running_verb_and_aborts_on_save_error() {
    for (action, want_close, want_verb) in [
        (Action::Sleep, false, "suspend"),
        (Action::Restart, true, "reboot"),
        (Action::ShutDown, true, "poweroff"),
    ] {
        let runner = MockRunner { log: Mutex::new(Vec::new()), fail: None };
        let res = execute(action, &runner, |close| {
            assert_eq!(close, want_close);
            runner.log.lock().unwrap().push(if close { "save:close".into() } else { "save:keep".into() });
            Ok(())
        });
        assert!(res.is_ok(), "{res:?}");
        let want_save = if want_close { "save:close" } else { "save:keep" };
        assert_eq!(*runner.log.lock().unwrap(), [want_save, want_verb]);
    }
    let runner = MockRunner { log: Mutex::new(Vec::new()), fail: None };
    let err = execute(Action::ShutDown, &runner, |_| Err("disk full".into())).unwrap_err();
    assert!(runner.log.lock().unwrap().is_empty(), "runner never invoked when save fails");
    assert!(err.contains("Shut down cancelled") && err.contains("disk full"), "{err}");
    let failing = MockRunner { log: Mutex::new(Vec::new()), fail: Some("permission denied".into()) };
    let err = execute(Action::Restart, &failing, |_| Ok(())).unwrap_err();
    assert_eq!(*failing.log.lock().unwrap(), ["reboot"]);
    assert_eq!(err, "Restart could not run\npermission denied");
}

#[test]
fn desktop_saves_before_minimizing_and_systemctl_is_blocked_in_tests() {
    let order = Mutex::new(Vec::new());
    let ok = execute_desktop(
        || {
            order.lock().unwrap().push("save:keep");
            Ok(())
        },
        || order.lock().unwrap().push("desktop"),
    );
    assert!(ok.is_ok(), "{ok:?}");
    assert_eq!(*order.lock().unwrap(), ["save:keep", "desktop"]);

    let mut minimized = false;
    let err = execute_desktop(|| Err("disk full".into()), || minimized = true).unwrap_err();
    assert!(!minimized, "minimize never called when save fails");
    assert_eq!(err, "Back to the desktop cancelled\nCould not save the suspended game: disk full");

    let blocked = SystemctlRunner.run("poweroff").unwrap_err();
    assert!(blocked.contains("disabled in tests"), "{blocked}");
}

#[test]
fn eclipse_power_page_projects_board_15_while_reference_stays_untouched() {
    let home = pages_of(home_card::CARD).unwrap();
    let suspended: serde_json::Value = serde_json::from_str(&Facts { suspended: "Aurora Drift".into(), ..Facts::default() }.to_json()).unwrap();
    let idle: serde_json::Value = serde_json::from_str(&Facts::default().to_json()).unwrap();

    // Reference theme power page stays untouched (R-20261114-21).
    let ref_power = home.page_for(home_card::POWER, &suspended, false);
    assert!(!ref_power.power && ref_power.rows.iter().map(|r| r.label.as_str()).collect::<Vec<_>>() == ["Quit EmuSen..."]);

    // Eclipse Power with suspended game draws all 5 board #15 rows, icons, hints, and subtitle.
    let ep = home.page_for(home_card::POWER, &suspended, true);
    assert!(ep.power && ep.title == "Power");
    assert_eq!(ep.subtitle, "Aurora Drift is suspended. It is saved before anything here happens.");
    assert_eq!(
        ep.rows.iter().map(|r| (r.key.as_str(), r.label.as_str(), r.value.as_str(), r.icon.as_str())).collect::<Vec<_>>(),
        [
            (home_card::POWER_SLEEP, "Sleep", "Wakes where you are", "power.sleep"),
            (home_card::POWER_RESTART, "Restart", "", "power.restart"),
            (home_card::POWER_SHUTDOWN, "Shut down", "", "power.shutdown"),
            (home_card::POWER_DESKTOP, "Back to the desktop", "EmuSen keeps running", "power.desktop"),
            (home_card::QUIT, "Quit EmuSen", "", "power.quit"),
        ]
    );
    for row in &ep.rows {
        assert!(!marks::path(&row.icon).is_empty(), "icon path exists for {}", row.icon);
    }
    assert_eq!(home.help_for(home_card::POWER, true), Screen::QuestionBack);
    assert_eq!(home.choose(home_card::POWER, home_card::POWER_SLEEP), Choice::Act(home_card::POWER_SLEEP.into()));
    assert_eq!(home.choose(home_card::POWER, home_card::POWER_DESKTOP), Choice::Act(home_card::POWER_DESKTOP.into()));
    assert_eq!(home.choose(home_card::POWER, home_card::QUIT), Choice::Turn(home_card::QUIT.into()));

    // Suspended Quit turns to circular dialog; idle Quit acts immediately without confirmation.
    let eq = home.page_for(home_card::QUIT, &suspended, true);
    assert!(eq.dialog && eq.title == "Quit EmuSen?" && eq.subtitle == "The game closes, then EmuSen.");
    assert_eq!(home.choose(home_card::QUIT, YES), Choice::Act(home_card::QUIT_EMUSEN.into()));

    let ep_idle = home.page_for(home_card::POWER, &idle, true);
    assert!(ep_idle.subtitle.is_empty());
    assert_eq!(ep_idle.rows[4].key, home_card::QUIT_EMUSEN);
    assert_eq!(home.choose(home_card::POWER, home_card::QUIT_EMUSEN), Choice::Act(home_card::QUIT_EMUSEN.into()));

    let menu_idle = home.page_for(home_card::FIRST, &idle, true);
    assert_eq!(menu_idle.rows.last().unwrap().key, home_card::QUIT_EMUSEN);
}

#[test]
fn notice_presenter_splits_heading_body_and_picks_icons() {
    assert_eq!(
        split_words("Tower Set could not start\nIts second disc is missing."),
        ("Tower Set could not start".into(), "Its second disc is missing.".into())
    );
    assert_eq!(
        split_words("Tower Set could not start: Its second disc is missing: Disc 2 is gone."),
        ("Tower Set could not start".into(), "Its second disc is missing: Disc 2 is gone.".into())
    );
    assert_eq!(split_words("State saved to slot 1"), ("State saved to slot 1".into(), String::new()));
    assert_eq!((icon_for(true), icon_for(false)), ("notice.warning", "notice.ok"));
    assert!(!marks::path("notice.warning").is_empty() && !marks::path("notice.ok").is_empty());

    // Scoped pending folder action attaches only to matching attention words and clears on unrelated attention notices.
    let mut words = "Failed to load Missing: not found".to_owned();
    let mut action = Some((OPEN_ITS_FOLDER.to_owned(), "/roms/snes".to_owned()));
    assert_eq!(take_pending_action(&mut words, &mut action, "State saved", false), None);
    assert_eq!(
        take_pending_action(&mut words, &mut action, "Failed to load Missing: not found", true),
        Some((OPEN_ITS_FOLDER.to_owned(), "/roms/snes".to_owned()))
    );
    assert!(words.is_empty() && action.is_none());

    let mut words = "Failed to load Missing: not found".to_owned();
    let mut action = Some((OPEN_ITS_FOLDER.to_owned(), "/roms/snes".to_owned()));
    assert_eq!(take_pending_action(&mut words, &mut action, "Settings could not save", true), None);
    assert!(words.is_empty() && action.is_none(), "unrelated attention notice clears stale pending action");
}

#[test]
fn eclipse_delete_slot_uses_board_14_wording_thumbnail_and_back_label() {
    assert_eq!(delete_when((2026, 11, 24), (2026, 11, 25), (2026, 11, 24), "8:40 PM"), "yesterday at 8:40 PM");
    assert_eq!(delete_when((2026, 11, 25), (2026, 11, 25), (2026, 11, 24), "9:15 PM"), "today at 9:15 PM");
    assert_eq!(delete_when((2026, 11, 12), (2026, 11, 25), (2026, 11, 24), "6:05 PM"), "12 November at 6:05 PM");

    let mut page = GamePage::new(Read {
        title: "Aurora Drift".into(),
        states: vec![State { path: "/s/slot1.state".into(), slot: Slot::Number(1), picture: "/s/slot1.png".into(), when: 1_000 }],
        ..Read::default()
    }, 90_000);
    page.section = 1;
    let Outcome::Ask(q, _) = page.delete_for(0, true, &|_| "yesterday at 8:40 PM".into()) else { panic!("expected Ask") };
    assert_eq!(q.title, "Delete Slot 1?");
    assert_eq!(q.sentences, ["Saved yesterday at 8:40 PM. The state and its picture go. Your in-game saves stay."]);
    assert_eq!(q.choices, ["Keep it", "Delete"]);
    assert_eq!(q.back, Some(0));
    assert_eq!(q.picture, "/s/slot1.png");
    assert_eq!(back_label(&q.choices, 0), "Keep it");
    assert_eq!(back_label(&q.choices, -1), "Back");
}
