//! One harness test (EmuSen_Qt.md §8.1): a scene importing EmuSen.Theme alone, as a theme does, reads every list through
//! its roles by name, the words, the tokens and a screen, and places each re-exported type; the host feeds the views
//! through `host` and the actions say whether they were taken.

mod common;

use core::pin::Pin;
use cxx_qt::QObject;
use cxx_qt_lib::QString;
use emusen_client_fixture::World;
use emusen_qt_test::Harness;
use emusen_theme_qt::actions::{Action, ActionSink};
use emusen_theme_qt::cues::{Cue, CuePlayer};
use emusen_theme_qt::device::DeviceFacts;
use emusen_theme_qt::help::KINDS;
use emusen_theme_qt::host;
use emusen_theme_qt::session::SessionFacts;
use std::sync::{Arc, Mutex};

/// A host that takes Play and declines the rest, keeping what it was asked.
struct Taker(Arc<Mutex<Vec<Action>>>);

impl ActionSink for Taker {
    fn act(&mut self, action: &Action) -> bool {
        self.0.lock().unwrap().push(action.clone());
        matches!(action, Action::Play(_))
    }
}

struct Rung(Arc<Mutex<Vec<Cue>>>);

impl CuePlayer for Rung {
    fn ring(&mut self, cue: Cue) {
        self.0.lock().unwrap().push(cue);
    }
}

fn within(window: *mut QObject) -> Pin<&'static mut QObject> {
    // Safety: the scene's window lives until the harness ends with the test.
    unsafe { Pin::new_unchecked(&mut *window) }
}

#[test]
fn qt_3_2_a_theme_reads_the_views_the_host_feeds() {
    let world = World::new("theme-qml");
    let read = common::read(&world);
    let mut harness = Harness::new();
    let mut window: *mut QObject = std::ptr::null_mut();
    let file = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/qml/ThemeScene.qml");
    let mut scene = harness.load_prepared(&format!("file://{file}"), 1280, 800, |w| window = w);
    assert_eq!(scene.property("scene", "consoles"), "0", "nothing before the host feeds the library");
    assert_eq!(scene.property("scene", "places"), "7", "Home's places stand before the library is read");

    host::library(within(window)).expect("the Library singleton").show(&read.view, common::HOME_LINES);
    scene.wait(50);
    assert_eq!(scene.property("scene", "consoles"), "1");
    assert_eq!(scene.property("scene", "shelves"), "3");
    assert_eq!(scene.property("scene", "firstConsole"), "nes");
    assert!(scene.set_property("scene", "consoleKey", "nes"));
    scene.wait(20);
    assert_eq!(scene.property("scene", "consoleGames"), "1", "gamesOf by the console's key");
    assert_eq!(scene.property("scene", "firstTitle"), "Synthetic");
    assert_eq!((scene.property("scene", "secondPlace").as_str(), scene.property("scene", "secondAttention").as_str()), ("Scraper", "true"));
    {
        let library = host::library(within(window)).unwrap();
        for (list, roles) in [("consoles", 11), ("places", 6), ("shelves", 4), ("recent", 17), ("favourites", 17), ("collections", 3)] {
            assert_eq!(library.list(list).map(|l| l.roles().len()), Some(roles), "{list}'s roles");
        }
    }

    host::session(within(window)).unwrap().show(&SessionFacts {
        has_game: true,
        title: "Synthetic".into(),
        console_id: "nes".into(),
        paused_since: 1,
        played: 60,
    });
    host::device(within(window)).unwrap().show(&DeviceFacts { controllers: 2, clock: "21:42".into(), ..DeviceFacts::default() });
    scene.wait(20);
    assert_eq!((scene.property("scene", "sessionTitle").as_str(), scene.property("scene", "hasGame").as_str()), ("Synthetic", "true"));
    assert_eq!(scene.property("scene", "status"), "2 controllers · 21:42");
    assert_eq!((scene.property("scene", "wordmark").as_str(), scene.property("scene", "counted").as_str()), ("EMUSEN", "2 games"));
    assert_eq!(scene.property("scene", "accent"), scene.property("scene", "kitAccent"), "the theme's Tokens are the kit's");
    assert_eq!(scene.property("scene", "dark"), "0");
    assert_eq!((scene.property("scene", "screenRole").as_str(), scene.property("scene", "screenFocus").as_str()), ("2", "0"));
    assert_eq!(scene.property("scene", "gameHelp"), "5", "Home's shelf: Play, Game page, Favourite, Search, Home");
    assert!(scene.property("scene", "gameActions").contains("previous PageUp"), "the frame declares the help's actions");

    let help = host::help(within(window)).unwrap();
    for kind in KINDS {
        // Safety: the help's lists are its children.
        let entries = unsafe { help.entries(kind).as_ref() }.expect("a list for every kind");
        assert!(!emusen_qt_models::help::entries(entries.screen()).is_empty(), "kind {} has entries", kind.repr);
    }

    let mut actions = host::actions(within(window)).unwrap();
    let game = QString::from("/roms/Synthetic.nes");
    assert!(!actions.as_mut().play(&game), "with no host, nothing is taken");
    let asked = Arc::new(Mutex::new(Vec::new()));
    actions.as_mut().set_sink(Some(Box::new(Taker(Arc::clone(&asked)))));
    assert!(actions.as_mut().play(&game), "Play taken");
    assert!(!actions.as_mut().back(), "Back declined");
    assert!(!actions.as_mut().step_section(1));
    assert_eq!(*asked.lock().unwrap(), [Action::Play("/roms/Synthetic.nes".into()), Action::Back, Action::StepSection(1)]);

    let rung = Arc::new(Mutex::new(Vec::new()));
    let mut cues = host::cues(within(window)).unwrap();
    cues.as_mut().scroll();
    cues.as_mut().set_player(Some(Box::new(Rung(Arc::clone(&rung)))));
    cues.as_mut().quicksysselect();
    assert_eq!(*rung.lock().unwrap(), [Cue::QuickSysSelect], "silence with no player, then the player rings");
    scene.assert_quiet();
}
