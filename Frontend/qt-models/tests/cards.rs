//! The Card form's pages as plain data (EmuSen_Qt.md §7.1, stage 2): the Home card's and the List card's rows in
//! Mistress's words, what each row does, the deck's way back, the grid's listing, and the screens' rule for the cards.

use emusen_qt_models::card::{pages_of, question, Choice, Deck, YES};
use emusen_qt_models::home_card::{self, Facts};
use emusen_qt_models::library::GameRow;
use emusen_qt_models::list_card::{self, action_of, stepped};
use emusen_qt_models::listing::{count_line, jump_letter, letter_rows, shown, step_sort, Listing};
use emusen_qt_models::screens::{step, Ask, Shown, POWER_PLACE};
use serde_json::{json, Value};

fn labels(card: &str, page: &str, facts: &Value) -> Vec<String> {
    pages_of(card).unwrap().page(page, facts).rows.iter().map(|r| r.label.clone()).collect()
}

fn facts(f: Facts) -> Value {
    serde_json::from_str(&f.to_json()).unwrap()
}

#[test]
fn the_home_card_has_mistress_rows_and_pages() {
    let home = pages_of(home_card::CARD).unwrap();
    // The Power page (HomeCardPower): Quit alone, Big Picture being the program on every machine (the tester decided, 2026-11-13).
    let power = home.page(home_card::POWER, &facts(Facts::default()));
    assert_eq!((power.title.as_str(), power.breadcrumb.as_str()), ("Power", "Home"));
    assert!(power.footer.starts_with("EmuSen ") && power.footer.len() > 7, "the program's version under Power: {}", power.footer);
    assert_eq!(labels(home_card::CARD, home_card::POWER, &facts(Facts::default())), ["Quit EmuSen..."]);
    assert_eq!(labels(home_card::CARD, home_card::POWER, &facts(Facts { game_mode: true, ..Facts::default() })), ["Quit EmuSen..."]);
    assert_eq!(home.choose(home_card::POWER, "bigPicture"), Choice::Owed, "no row of Power's leaves Big Picture");
    // Quit asks first (HomeCardQuit): No, then Yes; its sentence says whether a game closes too.
    let quit = home.page(home_card::QUIT, &facts(Facts::default()));
    assert_eq!(quit, question("Quit EmuSen?", "EmuSen closes."));
    assert_eq!(home.page(home_card::QUIT, &facts(Facts { suspended: "Cobalt Harbor".into(), ..Facts::default() })).breadcrumb, "The game closes, then EmuSen.");
    assert_eq!(home.choose(home_card::POWER, home_card::QUIT), Choice::Turn(home_card::QUIT.into()));
    assert_eq!(home.choose(home_card::QUIT, "no"), Choice::Back);
    assert_eq!(home.choose(home_card::QUIT, YES), Choice::Act(home_card::QUIT_EMUSEN.into()));

    // The first page in §2.3's order, over a suspended game with a game in hand (HomeCardDesktop).
    let f = facts(Facts { suspended: "Cobalt Harbor".into(), paused: "SNES · Paused just now".into(), in_hand: "Aurora Drift".into(), search: true, startable: vec![home_card::Startable { console: "PlayStation".into(), name: "PlayStation".into(), engine: "Diamond".into(), ..Default::default() }], ..Facts::default() });
    let first = home.page(home_card::FIRST, &f);
    assert_eq!(first.title, "Home");
    assert_eq!(labels(home_card::CARD, home_card::FIRST, &f), ["Continue", "Selected game", "Search", "Start Console", "Tools", "Settings", "Power"]);
    assert_eq!(first.rows[0].value, "Cobalt Harbor");
    assert_eq!(first.rows[0].pane.caption, "SNES · Paused just now");
    let ends: Vec<bool> = first.rows.iter().map(|r| r.section_end).collect();
    assert_eq!(ends, [false, false, false, true, false, true, false], "Tools and Power each start a section");
    // With nothing suspended and nothing in hand, Tools leads and starts no section.
    assert_eq!(labels(home_card::CARD, home_card::FIRST, &facts(Facts::default())), ["Tools", "Settings", "Power"]);
    assert!(!home.page(home_card::FIRST, &facts(Facts::default())).rows[0].section_end);

    // Tools: each tool's line as its value, what it does in the pane until it has said its state.
    let tools = json!({ "cores": { "brief": "1 to approve", "attention": true, "sentence": "A core waits." } });
    let t = home.page(home_card::TOOLS, &facts(Facts { tools: tools.clone(), ..Facts::default() }));
    // A tool with no screen yet (Collections, Shell) is not listed.
    assert_eq!(t.rows.iter().map(|r| r.label.as_str()).collect::<Vec<_>>(), ["Scraper", "Packages", "Library Manager", "Cores", "Cheats", "Themes", "Compressed Discs"]);
    assert_eq!(t.rows[3].value, "1 to approve");
    assert_eq!(t.rows[3].pane.lines[0], ("Cores".to_owned(), "A core waits.".to_owned()));
    assert_eq!(t.rows[1].pane.lines[0].1, "Makes, checks and zips game packages, and converts discs to CHD.");
    assert_eq!(home.page(home_card::FIRST, &facts(Facts { tools, ..Facts::default() })).rows[0].value, "1 to approve", "Tools names the first tool that needs the player");
}

#[test]
fn the_deck_goes_back_to_the_row_that_opened_a_page() {
    let mut deck = Deck::default();
    deck.open(home_card::POWER);
    assert_eq!((deck.page(), deck.depth()), (home_card::POWER, 1));
    deck.turn(home_card::QUIT, 1);
    assert_eq!(deck.page(), home_card::QUIT);
    assert_eq!(deck.back(), Some((home_card::POWER.to_owned(), 1)), "B goes back to Quit EmuSen...");
    assert_eq!(deck.back(), None, "B on the page the card opened at closes it");
}

fn game(title: &str, favourite: bool, played_at: i64) -> GameRow {
    GameRow { path: format!("/r/{title}.nes"), title: title.into(), favourite, played_at, ..GameRow::default() }
}

#[test]
fn the_list_card_and_the_listing_are_mistress_games_screen() {
    let games = [game("brass", false, 30), game("Aurora", true, 10), game("cobalt", true, 20), game("2 Tails", false, 0)];
    let titles = |l: &Listing, section: &str| shown(&games, section, l).iter().map(|g| g.title.clone()).collect::<Vec<_>>();
    let by_name = Listing::default();
    assert_eq!(titles(&by_name, "all"), ["2 Tails", "Aurora", "brass", "cobalt"], "by name, in any case");
    assert_eq!(titles(&Listing { sort: 1, ..Listing::default() }, "all"), ["brass", "cobalt", "Aurora", "2 Tails"], "Last played, newest first");
    assert_eq!(titles(&by_name, "recent"), ["brass", "Aurora", "cobalt", "2 Tails"], "the recently played keep their order under Name");
    assert_eq!(titles(&Listing { favourites_only: true, ..Listing::default() }, "all"), ["Aurora", "cobalt"]);
    assert_eq!(titles(&Listing { search: " AUR ".into(), ..Listing::default() }, "all"), ["Aurora"]);
    assert_eq!((step_sort(0, -1), step_sort(4, 1)), (4, 0), "Sort wraps both ways");
    assert_eq!(count_line(1284, 12, true), "12 of 1,284 games");
    assert_eq!(count_line(8, 8, false), "8 games");
    let letters = letter_rows(&shown(&games, "all", &by_name));
    assert_eq!(letters, [("#".to_owned(), 0), ("A".to_owned(), 1), ("B".to_owned(), 2), ("C".to_owned(), 3)]);
    assert_eq!((jump_letter(&letters, "B", 1), jump_letter(&letters, "#", -1)), (Some(3), Some(3)));

    // The card: GamesListCard's rows, the title from the section and its count, the pane per row.
    let list = pages_of(list_card::CARD).unwrap();
    let f = json!({ "section": "All", "count": "8 games", "sort": "Name", "filter": "", "letter": "A", "letters": "A B C", "favouritesSection": false });
    let page = list.page(list_card::FIRST, &f);
    assert_eq!((page.title.as_str(), page.breadcrumb.as_str()), ("All · 8 games", "List"));
    assert_eq!(page.rows.iter().map(|r| r.label.as_str()).collect::<Vec<_>>(), ["Search", "Sort", "Filter", "Jump to", "Random game", "Collections"]);
    assert_eq!(page.rows.iter().map(|r| (r.steps, r.opens)).collect::<Vec<_>>(), [(false, true), (true, false), (false, true), (true, false), (false, false), (false, true)]);
    assert_eq!((page.rows[1].value.as_str(), page.rows[3].value.as_str()), ("Name", "A"));
    assert_eq!(page.rows[0].pane.lines[0], ("Type part of a name; the covers narrow as you type.".to_owned(), "The keyboard docks at the foot of the screen.".to_owned()));
    assert_eq!(page.rows[1].pane.lines[0].1, "Also: Last played · Most played · Year · Rating");
    let mut startable = f.clone();
    startable["startable"] = json!([{ "console": "PlayStation", "name": "PlayStation", "engine": "Diamond" }]);
    assert_eq!(list.page(list_card::FIRST, &startable).rows[5].label, "Start Console", "offered where a console starts empty, before Collections");
    assert_eq!(list.choose(list_card::FIRST, list_card::SORT), Choice::Stay("sort+1".into()));
    assert_eq!(list.step(list_card::FIRST, list_card::JUMP, -1), Some("jump-1".into()));
    assert_eq!(list.step(list_card::FIRST, list_card::SEARCH, 1), None, "Search steps nothing");
    assert_eq!(action_of(&stepped("jump", -1)), ("jump", -1));
    assert_eq!(list.choose(list_card::FIRST, list_card::FILTER), Choice::Turn(list_card::FILTER.into()));
    assert_eq!(list.page(list_card::FILTER, &f).rows.iter().map(|r| r.label.as_str()).collect::<Vec<_>>(), ["Every game", "Favourites"]);
    assert_eq!(list.page(list_card::FILTER, &json!({ "favouritesSection": true })).rows.len(), 1, "the Favourites section is not offered Favourites");
    assert_eq!(list.choose(list_card::FILTER, list_card::FAVOURITES_ONLY), Choice::Act(list_card::FAVOURITES_ONLY.into()));
}

#[test]
fn start_and_the_cards_follow_mistress_routes() {
    let act = |k: &str| Ask::Act(k.into());
    assert_eq!(step(Shown::Home, &Ask::Place(POWER_PLACE)), Some((Shown::HomeCard, Some("power".into()))), "Power opens the Home card at Power");
    assert_eq!(step(Shown::HomeCard, &Ask::Back), Some((Shown::Home, None)));
    assert_eq!(step(Shown::HomeCard, &Ask::Close), Some((Shown::Home, None)));
    assert_eq!(step(Shown::Home, &act("home")), Some((Shown::HomeCard, Some(emusen_qt_models::home_card::FIRST.to_owned()))), "Start on Home opens the Home card at its first page");
    assert_eq!(step(Shown::Grid, &act("home")), Some((Shown::Home, None)), "Start on the grid goes Home");
    assert_eq!(step(Shown::Consoles, &act("home")), Some((Shown::Home, None)));
    assert_eq!(step(Shown::Grid, &act("list")), Some((Shown::List, None)), "Select opens the List card");
    assert_eq!(step(Shown::List, &Ask::Back), Some((Shown::Grid, None)));
    assert_eq!(step(Shown::List, &act("search")), Some((Shown::Search, None)));
    assert_eq!(step(Shown::Home, &act("search")), Some((Shown::Search, Some("all".into()))), "Home's Select searches every game");
    assert_eq!(step(Shown::Search, &act("space")), None, "the keyboard's own keys change no screen");
    assert_eq!(step(Shown::Search, &act("done")), None, "X is the keyboard's, whose close is the screen's Back");
    assert_eq!(step(Shown::Search, &Ask::Play), Some((Shown::Game, None)), "a result plays without closing the keyboard first");
    // The shoulders move the focus between the keys and the covers, the keyboard staying docked (LunaP's GalleryLayout).
    assert_eq!(step(Shown::Search, &act("toGrid")), Some((Shown::SearchGrid, None)));
    assert_eq!(step(Shown::SearchGrid, &act("previous")), Some((Shown::Search, None)));
    assert_eq!(step(Shown::SearchGrid, &act("next")), Some((Shown::Search, None)));
    assert_eq!(step(Shown::Search, &Ask::Back), Some((Shown::Grid, None)), "B on the empty field closes the keyboard");
    assert_eq!(step(Shown::SearchGrid, &Ask::Back), Some((Shown::Consoles, None)), "B the grid passed on goes to Choose a console");
    assert_eq!(step(Shown::SearchGrid, &act("home")), Some((Shown::Home, None)));
    assert_eq!(step(Shown::SearchGrid, &Ask::Play), Some((Shown::Game, None)));
    assert_eq!(step(Shown::Search, &Ask::Keys(false)), Some((Shown::SearchGrid, None)), "the pointer on a cover");
    assert_eq!(step(Shown::SearchGrid, &Ask::Keys(true)), Some((Shown::Search, None)), "a click on the band");
    assert_eq!(step(Shown::Grid, &Ask::Keys(false)), None);
    assert_eq!(step(Shown::Game, &act("home")), None, "inside a game Start is the Pause Card's chord, not Home");
}

#[test]
fn collections_opens_the_filter_page_whose_collections_narrow_the_section_to_their_members() {
    use emusen_qt_models::gamelists::Collection;
    let list = pages_of(list_card::CARD).unwrap();
    assert_eq!(list.choose(list_card::FIRST, list_card::COLLECTIONS), Choice::Turn(list_card::FILTER.to_owned()));
    let text = list_card::collections_text(&[Collection { id: 7, name: "Couch".into(), members: vec!["/r/brass.nes".into()] }]);
    let listed = list_card::collections_of(&text);
    let f = json!({ "favouritesSection": false, "collections": [{ "key": listed[0].key, "name": listed[0].name }] });
    assert_eq!(labels(list_card::CARD, list_card::FILTER, &f), ["Every game", "Favourites", "Couch"]);
    let key = listed[0].key.clone();
    assert!(list_card::is_collection(&key) && !list_card::is_collection(list_card::FAVOURITES_ONLY));
    assert_eq!(list.choose(list_card::FILTER, &key), Choice::Act(key.clone()), "the grid narrows to it");
    assert_eq!(action_of(&key), (key.as_str(), 0), "a collection's key is no step");
    let games = [game("brass", false, 0), game("Aurora", true, 0)];
    let only = Listing { collection: Some(std::sync::Arc::new(listed[0].members.iter().cloned().collect())), ..Listing::default() };
    assert_eq!(shown(&games, "all", &only).iter().map(|g| g.title.as_str()).collect::<Vec<_>>(), ["brass"]);
    assert!(only.narrowed(), "the count says how many of the section");
}

#[test]
fn settings_doors_open_the_compressed_discs_tool_and_the_last_games_cheats_and_b_returns_to_settings() {
    use emusen_qt_models::screens::{step_in, tool_back};
    let act = |k: &str| Ask::Act(k.to_owned());
    assert_eq!(step(Shown::Settings, &act("compressed")), Some((Shown::Tool, Some("compressed".into()))));
    assert_eq!(step_in(true, Shown::Settings, &act("compressed")), Some((Shown::Tool, Some("compressed".into()))), "under an ES-DE theme too");
    assert_eq!(tool_back(false, Some(Shown::Settings), Shown::Tool, &Ask::Back), Some(Shown::Settings));
    assert_eq!(tool_back(true, Some(Shown::Settings), Shown::Tool, &Ask::Back), Some(Shown::Settings));
    assert_eq!(tool_back(false, Some(Shown::Home), Shown::Tool, &Ask::Back), Some(Shown::Home), "from Home, Home");
    assert_eq!(step(Shown::Settings, &act("cheats")), Some((Shown::Page, Some("cheats".into()))));
    assert_eq!(step(Shown::Settings, &act("logging")), None, "a door with no screen yet opens nothing");
}

#[test]
fn eclipse_cards_project_boards_5_44_45_and_46() {
    use emusen_qt_models::help::Screen;
    let home = pages_of(home_card::CARD).unwrap();
    let f = facts(Facts {
        suspended: "Chrono Trigger".into(),
        suspended_console: "Super Nintendo".into(),
        suspended_picture: "file:///ct.png".into(),
        total_games: 1284,
        in_hand: "Super Metroid".into(),
        search: true,
        startable: vec![
            home_card::Startable { console: "PlayStation".into(), name: "PlayStation".into(), engine: "Diamond".into(), firmware: "✓ Firmware found.".into(), system: "psx".into(), libretro: false },
            home_card::Startable { console: "Dreamcast".into(), name: "Dreamcast".into(), engine: "Flycast".into(), firmware: "Firmware missing.".into(), system: "dreamcast".into(), libretro: true },
        ],
        ..Facts::default()
    });
    // Board #5 with suspended game and game in hand: Menu title, Continue card + 7 drawn rows.
    let menu = home.page_for(home_card::FIRST, &f, true);
    assert!(menu.menu && menu.title == "Menu");
    assert_eq!(menu.continue_card.as_ref().map(|c| (c.title.as_str(), c.subtitle.as_str())), Some(("Chrono Trigger", "Suspended · Super Nintendo")));
    assert_eq!(
        menu.rows.iter().map(|r| (r.label.as_str(), r.value.as_str(), r.icon.as_str())).collect::<Vec<_>>(),
        [
            ("Continue", "Chrono Trigger", ""),
            ("Game page", "Super Metroid", "menu.gamePage"),
            ("Search", "1,284 games", "menu.search"),
            ("Library", "", "menu.library"),
            ("Tools", "Cores, Scraper, Packages", "menu.tools"),
            ("Settings", "", "menu.settings"),
            ("Power", "Sleep, restart, shut down", "menu.power"),
            ("Quit EmuSen", "The game closes first", "menu.quit"),
        ]
    );
    assert_eq!(home.help_for(home_card::FIRST, true), Screen::CardHome);
    assert_eq!(home.help_for(home_card::FIRST, false), Screen::Card);
    assert_eq!(home.choose(home_card::FIRST, home_card::LIBRARY), Choice::Act(home_card::LIBRARY.into()));
    assert_eq!(step(Shown::HomeCard, &Ask::Act(home_card::LIBRARY.into())), Some((Shown::Grid, Some("all".into()))));
    assert_eq!(step(Shown::HomeCard, &Ask::Act("library".into())), Some((Shown::Tool, Some("library".into()))));
    assert_eq!(step(Shown::HomeCard, &Ask::Act(home_card::SEARCH.into())), Some((Shown::Search, Some("all".into()))));
    // Undrawn state: no suspended game and no game in hand omits Continue and Game page and clears Quit hint.
    let idle = home.page_for(home_card::FIRST, &facts(Facts { total_games: 12, ..Facts::default() }), true);
    assert!(idle.continue_card.is_none());
    assert_eq!(idle.rows.iter().map(|r| r.label.as_str()).collect::<Vec<_>>(), ["Search", "Library", "Tools", "Settings", "Power", "Quit EmuSen"]);
    assert_eq!(idle.rows.last().unwrap().value, "");
    // Board #46 Start Console populates pane core, planet, and firmware status/ok.
    let sc = home.page_for(home_card::START_CONSOLE, &f, true);
    assert_eq!((sc.rows[0].pane.core.as_str(), sc.rows[0].pane.status.as_str(), sc.rows[0].pane.status_ok), ("Diamond", "✓ Firmware found.", true));
    assert!(sc.rows[0].pane.planet >= 0);
    assert_eq!((sc.rows[1].pane.core.as_str(), sc.rows[1].pane.planet, sc.rows[1].pane.status.as_str(), sc.rows[1].pane.status_ok), ("Flycast", -1, "Firmware missing.", false));
    // Board #44 Eclipse List Card shows full sort list and section divider before Start Console.
    let list = pages_of(list_card::CARD).unwrap();
    let lf = json!({
        "section": "Super Nintendo", "count": "148 games", "sort": "Name", "filter": "", "letter": "C", "letters": "A B C",
        "startable": [{ "console": "PlayStation", "name": "PlayStation", "engine": "Diamond" }],
        "filters": [
            { "key": "every", "name": "Every game", "count": 4 }, { "key": "favourites", "name": "Favourites", "count": 1 },
            { "key": "notPlayed", "name": "Not played", "count": 2 }, { "key": "collection:1", "name": "Co-op", "count": 1 }
        ]
    });
    let lp = list.page_for(list_card::FIRST, &lf, true);
    assert_eq!(lp.rows[1].pane.lines[0].1, "Name · Last played · Most played · Year · Rating");
    assert!(lp.rows[4].section_end && !lp.rows[5].section_end);
    // Board #45 Eclipse Filter attaches detail pane per row and divider before collections.
    let fp = list.page_for(list_card::FILTER, &lf, true);
    assert_eq!(fp.rows.iter().map(|r| (r.label.as_str(), r.section_end)).collect::<Vec<_>>(), [("Every game", false), ("Favourites", false), ("Not played", true), ("Co-op", false)]);
    assert_eq!(fp.rows[1].pane.lines[0].0, "Shows the section's favourites.");
    assert_eq!(fp.rows[2].pane.lines[0].0, "Shows the section's games never played.");
    assert_eq!(fp.rows[3].pane.lines[0].0, "Shows Co-op.");
}
