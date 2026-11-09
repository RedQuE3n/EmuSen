//! One harness test (EmuSen_Qt.md §8.1): a scene importing EmuSen.Theme alone, as a theme does, reads every list through
//! its roles by name, the words, the tokens and a screen, and places each re-exported type; the host feeds the views
//! through `host` and the actions say whether they were taken.

mod common;

use core::pin::Pin;
use cxx_qt::QObject;
use cxx_qt_lib::QString;
use emusen_client_fixture::World;
use emusen_kit_core::tokens::Variant;
use emusen_qt_test::Harness;
use emusen_theme_qt::actions::{Action, ActionSink};
use emusen_theme_qt::bells::Striker;
use emusen_theme_qt::cues::{Cue, CuePlayer};
use emusen_theme_qt::device::DeviceFacts;
use emusen_theme_qt::help::KINDS;
use emusen_theme_qt::host;
use emusen_theme_qt::browse::BrowseFacts;
use emusen_theme_qt::session::SessionFacts;
use std::sync::{Arc, Mutex};

/// A host that takes Play and declines the rest, keeping what it was asked.
struct Taker(Arc<Mutex<Vec<Action>>>);

impl ActionSink for Taker {
    fn act(&mut self, action: &Action) -> bool {
        self.0.lock().unwrap().push(action.clone());
        matches!(action, Action::Play(_))
    }

    fn would(&self, action: &Action) -> bool {
        matches!(action, Action::Play(_))
    }
}

/// A striker keeping the rungs made ready and struck.
struct Struck(Arc<Mutex<Vec<(&'static str, usize)>>>);

impl Striker for Struck {
    fn ladder(&mut self, count: usize) {
        self.0.lock().unwrap().push(("ladder", count));
    }

    fn strike(&mut self, rung: usize) {
        self.0.lock().unwrap().push(("strike", rung));
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
    assert_eq!(scene.property("orbits", "orbitCount"), "1", "the orbits read the consoles' groups when the list resets");
    assert_eq!(scene.property("figures", "count"), "1", "the figures read the consoles' groups when the list resets");
    assert_eq!(scene.property("wheels", "count"), "1", "the wheels read the consoles' groups when the list resets");
    assert_eq!(scene.property("wheels", "wheel"), "0", "the first console's wheel turned to it");
    assert_eq!(scene.property("wheels", "dialWidth"), "544", "three places and one maker on the dial");
    assert!(scene.set_property("scene", "consoleKey", "nes"));
    scene.wait(20);
    assert_eq!(scene.property("scene", "consoleGames"), "1", "gamesOf by the console's key");
    assert_eq!(scene.property("scene", "firstTitle"), "Synthetic");
    assert_eq!((scene.property("scene", "secondPlace").as_str(), scene.property("scene", "secondAttention").as_str()), ("Scraper", "true"));
    assert_eq!((scene.property("scene", "firstGroupSize").as_str(), scene.property("scene", "secondEnabled").as_str()), ("1", "true"));
    host::library(within(window)).unwrap().show_declined(&["scraper".to_owned()]);
    scene.wait(20);
    assert_eq!(scene.property("scene", "secondEnabled"), "false", "a place the host would decline reads as not available");
    assert_eq!(scene.property("scene", "secondPlace"), "Scraper", "the row changed in place");
    {
        let library = host::library(within(window)).unwrap();
        for (list, roles) in [("consoles", 16), ("places", 7), ("shelves", 4), ("recent", 18), ("shelf", 18), ("favourites", 18), ("collections", 3)] {
            assert_eq!(library.list(list).map(|l| l.roles().len()), Some(roles), "{list}'s roles");
        }
    }

    let shelved = scene.property("scene", "shelf");
    host::session(within(window)).unwrap().show(&SessionFacts {
        has_game: true,
        key: "/roms/Synthetic.nes".into(),
        title: "Synthetic".into(),
        console_id: "nes".into(),
        console: "Nintendo Entertainment System".into(),
        hue: 0.0,
        paused_since: 1,
        played: 60,
    });
    host::device(within(window)).unwrap().show(&DeviceFacts { controllers: 2, clock: "21:42".into(), ..DeviceFacts::default() });
    scene.wait(20);
    assert_eq!((scene.property("scene", "sessionTitle").as_str(), scene.property("scene", "hasGame").as_str()), ("Synthetic", "true"));
    assert_eq!(scene.property("scene", "status"), "2 controllers · 21:42");
    // A cover with no picture is the generated card, its title in the ink over art and the kit's Barlow (EmuSen_Kit.md §7.29).
    for variant in [Variant::Dark, Variant::Light] {
        scene.set_variant(variant);
        let tree = scene.tree();
        let titles: Vec<_> = tree.nodes.iter().filter(|n| n.name == "coverCardTitle").collect();
        assert_eq!(titles.len(), 3, "the plain tile's, the scaled tile's and the tile's in a scaled item");
        for title in &titles {
            assert_eq!((title.ink.as_deref(), title.face.as_deref()), (Some("#ffffffff"), Some("Barlow")), "{variant:?}");
        }
        // Drawn at 0.8, by its own scale or an ancestor's it is told of, the title's capitals on the screen are the plain tile's,
        // within the 2% a face's metrics move by between sizes (15.41 and 15.12 measured 2026-11-09; 12.3 unkept).
        let caps: Vec<f64> = titles.iter().map(|t| t.cap).collect();
        assert!(caps[0] > 0.0 && caps[1..].iter().all(|c| (c / caps[0] - 1.0).abs() < 0.03), "the title's capitals at 1, 0.8 and 0.8: {caps:?}");
    }
    assert_eq!((scene.property("scene", "sessionKey").as_str(), scene.property("scene", "sessionConsole").as_str()), ("/roms/Synthetic.nes", "Nintendo Entertainment System"));
    assert_eq!(scene.property("scene", "shelf"), shelved, "the shelf is fed by the library, the suspended game by the host");
    assert_eq!(scene.property("scene", "summary"), "2 consoles · 3 games");
    assert_eq!(scene.property("scene", "marked"), "true", "the marks are the models' paths");
    assert_eq!(scene.property("scene", "pageUp"), "8", "the pad's buttons in the kit's numbers");
    {
        let mut browse = host::browse(within(window)).unwrap();
        browse.as_mut().show(&BrowseFacts { heading: "NES".into(), next: "Game Boy".into(), focus_row: 3, ..BrowseFacts::default() });
        browse.as_mut().show_games(Vec::new());
    }
    scene.wait(20);
    assert_eq!((scene.property("scene", "browseHeading").as_str(), scene.property("scene", "browseNext").as_str()), ("NES", "Game Boy"));
    assert_eq!((scene.property("scene", "browseGames").as_str(), scene.property("scene", "focusRow").as_str()), ("0", "3"));
    assert_eq!((scene.property("scene", "wordmark").as_str(), scene.property("scene", "counted").as_str()), ("EMUSEN", "2 games"));
    assert_eq!(scene.property("scene", "accent"), scene.property("scene", "kitAccent"), "the theme's Tokens are the kit's");
    assert_eq!(scene.property("scene", "dark"), "0");
    assert_eq!((scene.property("scene", "screenRole").as_str(), scene.property("scene", "screenFocus").as_str()), ("2", "0"));
    assert_eq!(scene.property("scene", "gameHelp"), "5", "Home's shelf: Play, Game page, Favourite, Search, Home");
    assert_eq!((scene.property("scene", "placeHelp").as_str(), scene.property("scene", "placeGroup").as_str()), ("5", "Group"), "a place: Open, Back, Group, Search, Home");
    assert_eq!(scene.property("scene", "declinedHelp"), "false|true|5", "accept: false dims A's entry alone");
    assert_eq!(
        scene.property("scene", "emptyWords"),
        "No games yet|Nothing played yet|No consoles have games yet|Settings › Library adds a folder|PLACES|Continue|Recent"
    );
    assert!(scene.property("scene", "paused").starts_with("Paused "), "pausedAgo from the models' words");
    assert_eq!(scene.property("scene", "rungs"), "3", "the ladder's count, with no striker too");
    assert!(scene.property("scene", "gameActions").contains("previous PageUp"), "the frame declares the help's actions");

    let help = host::help(within(window)).unwrap();
    for kind in KINDS {
        // Safety: the help's lists are its children.
        let entries = unsafe { help.entries(kind).as_ref() }.expect("a list for every kind");
        assert!(!emusen_qt_models::help::entries(entries.screen()).is_empty(), "kind {} has entries", kind.repr);
        for (group, back, accept) in emusen_theme_qt::help::FLAGS {
            let added = unsafe { help.list(kind, group, back, accept).as_ref() }.expect("a list for every kind and flag");
            assert_eq!(added.list(), emusen_qt_models::help::entries_with(entries.screen(), group, back), "kind {}", kind.repr);
            assert_eq!(added.accepts(), accept, "kind {}", kind.repr);
        }
    }

    let mut actions = host::actions(within(window)).unwrap();
    let game = QString::from("/roms/Synthetic.nes");
    assert!(!actions.as_mut().play(&game), "with no host, nothing is taken");
    assert!(!actions.enabled(&QString::from("list")), "with no host, nothing is enabled");
    let asked = Arc::new(Mutex::new(Vec::new()));
    actions.as_mut().set_sink(Some(Box::new(Taker(Arc::clone(&asked)))));
    assert!(actions.as_mut().play(&game), "Play taken");
    assert!(!actions.enabled(&QString::from("list")), "enabled asks the host without acting");
    assert!(!actions.as_mut().back(), "Back declined");
    assert!(!actions.as_mut().step_section(1));
    assert!(!actions.as_mut().act(&QString::from("list")));
    assert!(!actions.as_mut().focus(&game));
    assert_eq!(
        *asked.lock().unwrap(),
        [Action::Play("/roms/Synthetic.nes".into()), Action::Back, Action::StepSection(1), Action::Act("list".into()), Action::Focus("/roms/Synthetic.nes".into())]
    );

    let rung = Arc::new(Mutex::new(Vec::new()));
    let mut cues = host::cues(within(window)).unwrap();
    cues.as_mut().scroll();
    cues.as_mut().set_player(Some(Box::new(Rung(Arc::clone(&rung)))));
    cues.as_mut().quicksysselect();
    assert_eq!(*rung.lock().unwrap(), [Cue::QuickSysSelect], "silence with no player, then the player rings");

    let struck = Arc::new(Mutex::new(Vec::new()));
    let mut bells = host::bells(within(window)).unwrap();
    bells.as_mut().strike(0);
    bells.as_mut().set_striker(Some(Box::new(Struck(Arc::clone(&struck)))));
    assert_eq!(bells.as_mut().ladder(13), 13);
    bells.as_mut().strike(4);
    bells.as_mut().strike(-1);
    assert_eq!(*struck.lock().unwrap(), [("ladder", 13), ("strike", 4)], "silence with no striker, then rung i rings; no rung below 0");
    scene.assert_quiet();
}
