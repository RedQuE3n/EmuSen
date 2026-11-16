//! Eclipse's games view narrowed (EmuSen_Qt.md §3.4.10): over a library with a game played, three never played and two
//! in a collection, Not played and the collection pressed on the chips and their lists and counts read; the filter, the
//! sort and the letter set on the List card and read in the games view, and the reverse; a search with no game says so.

mod common;
#[path = "eclipse/table.rs"]
mod table;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt::MAIN_URL;
use emusen_qt_models::listing::Sort;
use emusen_qt_models::{themes, words};
use emusen_qt_test::Scene;
use serde_json::json;

/// The games, by name; the second is played and the last two are in the collection.
const GAMES: [&str; 4] = ["Aurora Drift", "Brass Lantern", "Cinder Mile", "Dune Signal"];
const PLAYED: &str = "Brass Lantern";
const IN_COLLECTION: [&str; 2] = ["Cinder Mile", "Dune Signal"];
const COLLECTION: &str = "Couch";
/// Qt::Key_Z and Qt::Key_Q, typed as a search no game's name holds.
const KEYS: [i32; 2] = [0x5A, 0x51];

/// games.db where Galaxia names it, as the program opens it.
fn records() -> Library {
    let platform = Platform::load(library()).unwrap();
    let sqlite = Lent::open("libsqlite3.so.0").unwrap();
    Library::open(&platform, &platform.file(emusen_platform_client::galaxia::consts::FILE_GAMES).unwrap(), &sqlite).unwrap()
}

/// An item's words and its ink, read from the scene's tree: a view's delegates are not found by name otherwise.
fn words_of(scene: &mut Scene<'_>, name: &str) -> (String, String) {
    let tree = scene.tree();
    let node = tree.find_named(name).map(|i| &tree.nodes[i as usize]).unwrap_or_else(|| panic!("{name} is in the scene"));
    (node.text.clone().unwrap_or_default(), node.ink.clone().unwrap_or_default())
}

/// The shown games' titles, in the order shown.
fn titles(scene: &mut Scene<'_>) -> Vec<String> {
    let count: usize = scene.property("gamesGrid", "count").parse().unwrap();
    (0..count).map(|i| words_of(scene, &format!("tileTitle.{i}")).0).collect()
}

/// Presses a chip as a hand does: the focus on it, then A.
fn press_chip(scene: &mut Scene<'_>, index: usize) {
    let tree = scene.tree();
    let chip = tree.find_named(&format!("gamesFilter.{index}")).expect("the chip");
    assert!(scene.focus_item(&tree, chip));
    scene.pad(UiButton::Accept);
}

/// The List card's rows as it shows them, "label | value".
fn card_rows(scene: &mut Scene<'_>) -> Vec<String> {
    common::console::card_words(scene).into_iter().map(|(label, value)| if value.is_empty() { label } else { format!("{label} | {value}") }).collect()
}

/// An object of the theme's engine by its number in the tree.
fn object(id: u64) -> core::pin::Pin<&'static mut cxx_qt::QObject> {
    // Safety: the id was read from the live tree, and the screen it names lives until the host loads again.
    unsafe { core::pin::Pin::new_unchecked(&mut *(id as *mut cxx_qt::QObject)) }
}

/// The Actions the program gives the theme, as the games view calls them.
fn theme_actions(scene: &mut Scene<'_>) -> core::pin::Pin<&'static mut emusen_theme_qt::actions::qobject::ThemeActions> {
    let tree = scene.tree();
    let screen: u64 = tree.nodes[tree.find_named("gridScreen").expect("Eclipse's games view") as usize].id.parse().unwrap();
    emusen_theme_qt::host::actions(object(screen)).expect("the theme's Actions")
}

fn list_facts(scene: &mut Scene<'_>) -> serde_json::Value {
    serde_json::from_str(&scene.property("gridGames", "listFacts")).expect("the List card's facts")
}

#[test]
fn qt_3_4_10_eclipses_chips_and_the_list_card_narrow_one_list() {
    let files: Vec<String> = GAMES.iter().map(|t| format!("{t}.sfc")).collect();
    let world = sandbox("qt-app-eclipse-games-lists", &files.iter().map(|f| ("SNES", f.as_str())).collect::<Vec<_>>());
    let path = |title: &str| world.root.join("Roms/SNES").join(format!("{title}.sfc")).to_str().unwrap().to_owned();
    let games = records();
    games.change(&json!({ "op": "started", "path": path(PLAYED), "at": "2026-11-13T10:00:00Z" })).unwrap();
    let id = games.change(&json!({ "op": "create_collection", "name": COLLECTION, "at": "2026-11-13T10:00:00Z" })).unwrap();
    for title in IN_COLLECTION {
        games.change(&json!({ "op": "add_to_collection", "id": id, "path": path(title) })).unwrap();
    }
    drop(games);

    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "screens", "shown", "Home", 3000);
    // A game played puts Home's focus on it; Games is the first place.
    let mut games_place = None;
    for _ in 0..40 {
        games_place = scene.tree().find_named("place.0");
        if games_place.is_some() {
            break;
        }
        scene.wait(50);
    }
    let tree = scene.tree();
    assert!(scene.focus_item(&tree, games_place.expect("Home's Games place")));
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Consoles", 2000);
    let tree = scene.tree();
    let all = tree.find_named("tile.0").expect("All games on Choose a console");
    assert!(scene.focus_item(&tree, all));
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);
    assert!(scene.set_property("program", "theme", &themes::built_in_folder("eclipse")));
    wait_for(&mut scene, "host.theme", "ready", "true", 3000);
    wait_for(&mut scene, "gamesGrid", "count", &GAMES.len().to_string(), 3000);
    scene.wait(300);

    // The chips count each list as the board names them: all, no favourite, three never played (R-20261115-10); the
    // collection is the List card's alone, no chip.
    let chips: Vec<String> = (0..3).map(|i| words_of(&mut scene, &format!("gamesFilterWords.{i}")).0).collect();
    let never = GAMES.len() - 1;
    assert_eq!(chips, [words::choice_line(words::ALL, GAMES.len()), words::choice_line(words::FAVOURITES_FILTER, 0), words::choice_line(words::NOT_PLAYED, never)]);
    assert!(!scene.tree().nodes.iter().any(|n| n.name == "gamesFilter.3" && n.visible), "no chip for the collection");

    // Not played pressed: the three never played, the played one left out.
    press_chip(&mut scene, 2);
    wait_for(&mut scene, "gamesGrid", "count", &never.to_string(), 2000);
    assert_eq!(titles(&mut scene), GAMES.iter().filter(|t| **t != PLAYED).map(|t| t.to_string()).collect::<Vec<_>>());
    // The collection set as a theme sets it (Actions.setFilter): its games.
    let key = scene.tree().nodes.iter().find(|n| n.name == "gamesFilter.3").map(|n| n.id.parse::<u64>().unwrap()).expect("the collection's choice");
    let key = emusen_qt_test::ffi::ffi::harness_property(object(key), &cxx_qt_lib::QString::from("gamesFilter.3"), &cxx_qt_lib::QString::from("key")).to_string();
    assert!(theme_actions(&mut scene).set_filter(&cxx_qt_lib::QString::from(key.as_str())));
    wait_for(&mut scene, "gamesGrid", "count", &IN_COLLECTION.len().to_string(), 2000);
    assert_eq!(titles(&mut scene), IN_COLLECTION);
    // The collections are read once for each change of them, never again as the focus moves while one narrows the list.
    let read = scene.property("gridGames", "collectionsRead");
    assert_ne!(read, "0", "the collections were read");
    let tree = scene.tree();
    assert!(scene.focus_item(&tree, tree.find_named("gamesTile.0").unwrap()));
    for (button, title) in [(UiButton::Right, IN_COLLECTION[1]), (UiButton::Left, IN_COLLECTION[0]), (UiButton::Right, IN_COLLECTION[1]), (UiButton::Left, IN_COLLECTION[0])] {
        scene.pad(button);
        wait_for(&mut scene, "gamesTitle", "text", title, 1000);
    }
    assert_eq!(focus_name(&mut scene), "gamesTile.0");
    assert_eq!(scene.property("gridGames", "focused"), "0", "the model's focus moved and came back");
    assert_eq!(scene.property("gridGames", "collectionsRead"), read, "no read of the collections for a move of the focus");

    // Set in the games view, read on the List card: the collection, the sort the pill stepped to, the focused letter.
    assert!(theme_actions(&mut scene).step_sort(1));
    wait_for(&mut scene, "gamesSortWords", "text", &emusen_qt_models::listing::sort_line(Sort::LastPlayed.words()), 2000);
    scene.pad(UiButton::Options);
    wait_for(&mut scene, "screens", "shown", "List", 2000);
    scene.wait(300);
    let rows = card_rows(&mut scene);
    assert_eq!(rows[1], format!("Sort | {}", Sort::LastPlayed.words()), "{rows:?}");
    assert_eq!(rows[3], format!("Jump to | {}", emusen_qt_models::listing::letter(IN_COLLECTION[0])), "{rows:?}");
    assert_eq!(list_facts(&mut scene)["filter"], COLLECTION, "the card's Filter says the collection");

    // Set on the List card, read in the games view: the sort stepped, then Not played chosen on the Filter page.
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Right);
    scene.wait(200);
    assert_eq!(card_rows(&mut scene)[1], format!("Sort | {}", Sort::MostPlayed.words()));
    until(&mut scene, "gamesSortWords", &emusen_qt_models::listing::sort_line(Sort::MostPlayed.words()));
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    scene.wait(300);
    let filters = card_rows(&mut scene);
    let not_played = filters.iter().position(|r| r == words::NOT_PLAYED).unwrap_or_else(|| panic!("Not played on the Filter page: {filters:?}"));
    for _ in 0..not_played {
        scene.pad(UiButton::Down);
    }
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);
    wait_for(&mut scene, "gamesGrid", "count", &never.to_string(), 2000);
    assert_ne!(words_of(&mut scene, "gamesFilterWords.2").1, words_of(&mut scene, "gamesFilterWords.0").1, "Not played drawn as the chip in force");
    assert_eq!(list_facts(&mut scene)["filter"], words::NOT_PLAYED);

    // The letter stepped on the card's Jump to moves the games view's focus and its rail.
    scene.pad(UiButton::Options);
    wait_for(&mut scene, "screens", "shown", "List", 2000);
    scene.wait(300);
    for _ in 0..3 {
        scene.pad(UiButton::Down);
    }
    let before = card_rows(&mut scene)[3].clone();
    scene.pad(UiButton::Right);
    scene.wait(200);
    let after = card_rows(&mut scene)[3].clone();
    assert_ne!(before, after, "Jump to stepped");
    let letter = after.rsplit(" | ").next().unwrap().to_owned();
    until(&mut scene, "gamesRailLetterWords", &letter);
    assert_eq!(emusen_qt_models::listing::letter(&scene.property("gamesTitle", "text")), letter, "the focused game is the letter's");

    // The reverse: the games view's letter step (RT) is the card's Jump to.
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);
    let tree = scene.tree();
    assert!(scene.focus_item(&tree, tree.find_named("gamesTile.0").unwrap()));
    scene.wait(100);
    scene.pad(UiButton::Last);
    scene.wait(200);
    let rail = words_of(&mut scene, "gamesRailLetterWords").0;
    scene.pad(UiButton::Options);
    wait_for(&mut scene, "screens", "shown", "List", 2000);
    scene.wait(300);
    assert_eq!(card_rows(&mut scene)[3], format!("Jump to | {rail}"));

    // A search no name holds leaves the list empty, and it says so (R-20261115-13).
    scene.pad(UiButton::Up);
    scene.pad(UiButton::Up);
    scene.pad(UiButton::Up);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Search", 2000);
    for key in KEYS {
        scene.key(key, 0);
    }
    wait_for(&mut scene, "gamesGrid", "count", "0", 2000);
    let rows = table::built();
    let (held, item, said) = table::state_rows(&rows, "search");
    wait_for(&mut scene, &item, "text", said, 2000);
    let report = table::differences(&mut scene, &held, 1280.0, 800.0);
    assert!(report.is_empty(), "search: {}", report.join("\n"));
}

fn focus_name(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default()
}

/// Waits until an item's words, read from the tree, are `words`.
fn until(scene: &mut Scene<'_>, name: &str, words: &str) {
    for _ in 0..40 {
        if scene.tree().find_named(name).is_some() && words_of(scene, name).0 == words {
            return;
        }
        scene.wait(50);
    }
    assert_eq!(words_of(scene, name).0, words, "{name}'s words");
}
