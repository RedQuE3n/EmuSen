//! The grid's listing as a theme reads and drives it (EmuSen_Qt.md §3.4.10; R-20261115-10 to -13): Not played narrows to the games with no played time, the Filter page's choices with their counts are one list for the List card and a theme, a list with nothing in it says its own words, a section is told by its kind, and the triggers step letters where a bar asks for them.

use emusen_kit_core::input::UiButton;
use emusen_qt_models::card::pages_of;
use emusen_qt_models::help::{self, Screen};
use emusen_qt_models::library::{self, GameRow};
use emusen_qt_models::list_card::{self, action_of, Listed};
use emusen_qt_models::listing::{choices, empty_words, shown, Choice, Listing};
use emusen_qt_models::words;

fn game(title: &str, favourite: bool, played_at: i64) -> GameRow {
    GameRow { path: format!("/r/{title}.nes"), title: title.into(), favourite, played_at, ..GameRow::default() }
}

fn games() -> [GameRow; 4] {
    [game("brass", false, 30), game("Aurora", true, 0), game("cobalt", true, 20), game("2 Tails", false, 0)]
}

#[test]
fn not_played_narrows_to_the_games_with_no_played_time() {
    let games = games();
    let listing = Listing { not_played: true, ..Listing::default() };
    let titles: Vec<String> = shown(&games, "nes", &listing).iter().map(|g| g.title.clone()).collect();
    assert_eq!(titles, ["2 Tails", "Aurora"]);
    assert!(listing.narrowed(), "the count says how many of the section are shown");
}

#[test]
fn the_filter_choices_are_counted_once_and_are_the_filter_pages_rows() {
    let games = games();
    let saved = Listed { key: "collection:7".into(), name: "Couch".into(), members: vec!["/r/brass.nes".into()] };
    let made = choices(&games, "nes", std::slice::from_ref(&saved));
    let c = |key: &str, name: &str, count: usize| Choice { key: key.into(), name: name.into(), count };
    assert_eq!(
        made,
        [
            c(list_card::EVERY_GAME, words::EVERY_GAME, 4),
            c(list_card::FAVOURITES_ONLY, words::FAVOURITES_FILTER, 2),
            c(list_card::NOT_PLAYED, words::NOT_PLAYED, 2),
            c("collection:7", "Couch", 1)
        ]
    );
    assert!(choices(&games, library::FAVOURITES, &[]).iter().all(|c| c.key != list_card::FAVOURITES_ONLY), "the Favourites section has no Favourites choice");
    // The List card's Filter page shows the same choices, Not played among them.
    let facts = serde_json::json!({ "filters": made, "section": "NES", "count": "4 games" });
    let page = pages_of(list_card::CARD).unwrap().page(list_card::FILTER, &facts);
    let rows: Vec<(String, String)> = page.rows.iter().map(|r| (r.key.clone(), r.label.clone())).collect();
    assert_eq!(rows, made.iter().map(|c| (c.key.clone(), c.name.clone())).collect::<Vec<_>>());
    let first = pages_of(list_card::CARD).unwrap().page(list_card::FIRST, &serde_json::json!({ "filter": words::NOT_PLAYED, "sort": "Name" }));
    assert!(first.rows.iter().any(|r| r.value == words::NOT_PLAYED), "the Filter row says Not played");
}

#[test]
fn a_list_with_nothing_says_its_own_words() {
    assert_eq!(empty_words(library::FAVOURITES, false), words::NO_FAVOURITES);
    assert_eq!(empty_words(library::RECENT, false), words::NO_RECENT);
    assert_eq!(empty_words("nes", true), words::NO_GAMES_FOUND);
    assert_eq!(empty_words(library::FAVOURITES, true), words::NO_GAMES_FOUND, "a search or a filter that leaves none");
    assert_eq!(empty_words(library::ALL, false), words::NO_GAMES);
    assert_eq!((words::NO_FAVOURITES, words::NO_GAMES_FOUND, words::NOT_PLAYED), ("No favourites yet", "No games found", "Not played"));
    assert!(words::is_fixed(words::NO_GAMES_FOUND) && words::is_fixed(words::NOT_PLAYED));
    assert_eq!(words::choice_line(words::NOT_PLAYED, 1530), "Not played · 1,530", "a chip's line");
}

#[test]
fn a_section_is_told_by_its_kind() {
    assert_eq!(library::section_kind(library::ALL), "all");
    assert_eq!(library::section_kind(library::FAVOURITES), "favourites");
    assert_eq!(library::section_kind(library::RECENT), "recent");
    assert_eq!(library::section_kind("snes"), library::CONSOLE_KIND);
    assert_eq!(library::section_kind(""), "");
}

#[test]
fn the_triggers_step_letters_where_the_bar_asks_and_keep_their_meaning_elsewhere() {
    assert_eq!(action_of(list_card::JUMP_BACK), (list_card::JUMP, -1));
    assert_eq!(action_of(list_card::JUMP_ON), (list_card::JUMP, 1));
    assert_eq!(action_of(list_card::SORT_ON), (list_card::SORT, 1));
    let plain = help::entries_with(Screen::Grid, false, false);
    assert_eq!(help::entries_with_letters(Screen::Grid, false, false, false), plain, "not asked, nothing added");
    let lettered = help::entries_with_letters(Screen::Grid, false, false, true);
    let console = plain.iter().position(|e| e.0 == UiButton::PageUp).unwrap();
    assert_eq!(lettered[console + 1], help::LETTER, "after the shoulders' Console");
    assert_eq!(lettered.len(), plain.len() + 1);
    assert_eq!(help::pair_of(UiButton::First), Some(UiButton::Last), "LT and RT drawn as one entry");
    let lines = help::action_lines_with(Screen::Grid, true);
    assert!(lines.contains(&format!("{} First\n", list_card::JUMP_BACK)) && lines.contains(&format!("{} Last\n", list_card::JUMP_ON)), "{lines}");
    assert_eq!(help::action_lines_with(Screen::Grid, false), help::action_lines(Screen::Grid));
    assert_eq!(help::action_lines_with(Screen::Home, true), help::action_lines(Screen::Home), "a bar with no letters' place");
    assert!(!help::action_lines(Screen::Grid).contains("First"), "the grid's own bar leaves the triggers alone");
}

#[test]
fn a_collections_key_is_never_read_as_a_step_whatever_its_id() {
    // games.db's collection ids are any i64: a negative one puts '-' in the key, which a step's reading split.
    let text = list_card::collections_text(&[emusen_qt_models::gamelists::Collection { id: -3, name: "Couch".into(), members: vec![] }]);
    let key = list_card::collections_of(&text)[0].key.clone();
    assert_eq!(action_of(&key), (key.as_str(), 0));
    assert_eq!(action_of(&list_card::stepped(list_card::JUMP, -1)), (list_card::JUMP, -1), "a step is still a step");
}

#[test]
fn eclipses_games_board_names_its_chips_its_order_and_its_foot() {
    // The chips: every game as All, Favourites and Not played as the Filter page names them, a collection none.
    assert_eq!(emusen_qt_models::listing::chip_name(list_card::EVERY_GAME), Some(words::ALL));
    assert_eq!(emusen_qt_models::listing::chip_name(list_card::FAVOURITES_ONLY), Some(words::FAVOURITES_FILTER));
    assert_eq!(emusen_qt_models::listing::chip_name(list_card::NOT_PLAYED), Some(words::NOT_PLAYED));
    let listed = Listed { key: "collection:7".into(), name: "Couch".into(), members: Vec::new() };
    let collection = choices(&games(), library::ALL, &[listed]).pop().unwrap();
    assert_eq!(emusen_qt_models::listing::chip_name(&collection.key), None, "a collection has no chip");
    // The order with its direction; one that names its own as it is.
    assert_eq!(emusen_qt_models::listing::sort_line("Name"), "Name, A to Z");
    assert_eq!(emusen_qt_models::listing::sort_line("Last played"), "Last played, newest first");
    assert_eq!(emusen_qt_models::listing::sort_line("Name, ascending"), "Name, ascending");
    // The foot: A Start, B Back, Y Favourite, in the board's order, the actions kept.
    let foot = help::games_named(help::entries(Screen::Grid).to_vec());
    let named: Vec<(UiButton, &str)> = foot.iter().map(|e| (e.0, e.1)).collect();
    assert_eq!(named, [(UiButton::Accept, words::START), (UiButton::Back, help::BACK.1), (UiButton::Search, "Favourite")]);
    assert_eq!(foot[2].2, Some(help::FAVOURITE));
}
