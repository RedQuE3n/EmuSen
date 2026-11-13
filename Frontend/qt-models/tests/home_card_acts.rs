//! The Home card's rows act (EmuSen_Qt.md §7.9, items 2 and 3): Continue shows the suspended game, Start Console lists
//! the consoles that start empty and starts the one chosen, from the Home card and the List card alike, and Tools ▸
//! Cores and ▸ Themes open those screens in both looks, B coming back to where the card was opened.

use emusen_qt_models::card::{pages_of, Choice};
use emusen_qt_models::home_card::{self, console_to_start, start_console_action, Facts, Startable};
use emusen_qt_models::list_card;
use emusen_qt_models::help::Screen;
use emusen_qt_models::screens::{step, step_in, tool_back, Ask, Shown, THEMES};
use serde_json::{json, Value};

fn act(key: &str) -> Ask {
    Ask::Act(key.to_owned())
}

fn startable() -> Vec<Startable> {
    vec![
        Startable { console: "PlayStation".into(), name: "PlayStation".into(), engine: "Diamond".into(), firmware: "✓ Firmware found.".into(), system: "psx".into() },
        Startable { console: "Saturn".into(), name: "Saturn".into(), engine: "Topaz".into(), firmware: String::new(), system: "saturn".into() },
    ]
}

#[test]
fn continue_shows_the_suspended_game_in_both_looks() {
    assert_eq!(step(Shown::HomeCard, &act(home_card::RESUME)), Some((Shown::Game, None)));
    assert_eq!(step_in(true, Shown::HomeCard, &act(home_card::RESUME)), Some((Shown::Game, None)));
    let home = pages_of(home_card::CARD).unwrap();
    assert_eq!(home.choose(home_card::FIRST, home_card::RESUME), Choice::Act(home_card::RESUME.into()));
}

#[test]
fn start_console_lists_each_console_that_starts_empty_and_starts_the_one_chosen() {
    let home = pages_of(home_card::CARD).unwrap();
    let f: Value = serde_json::from_str(&Facts { startable: startable(), ..Facts::default() }.to_json()).unwrap();
    // The row and its page read one fact: offered while a console is listed, and not over an empty page.
    let rows_of = |f: &Value| home.page(home_card::FIRST, f).rows.iter().map(|r| r.key.clone()).collect::<Vec<_>>();
    assert!(rows_of(&f).iter().any(|k| k == home_card::START_CONSOLE));
    let none: Value = serde_json::from_str(&Facts::default().to_json()).unwrap();
    assert!(!rows_of(&none).iter().any(|k| k == home_card::START_CONSOLE), "no row over an empty page");
    assert_eq!(home.choose(home_card::FIRST, home_card::START_CONSOLE), Choice::Turn(home_card::START_CONSOLE.into()), "the row opens its page");
    let page = home.page(home_card::START_CONSOLE, &f);
    assert_eq!((page.title.as_str(), page.breadcrumb.as_str()), ("Start Console", "Home"));
    assert_eq!(page.rows.iter().map(|r| (r.label.as_str(), r.value.as_str())).collect::<Vec<_>>(), [("PlayStation", "Diamond"), ("Saturn", "Topaz")]);
    assert_eq!(
        page.rows[0].pane.lines[0],
        ("Starts a PlayStation with no disc in it, on Diamond.".to_owned(), "Its BIOS screen comes up; Change Disc on the Pause Card puts a game in.".to_owned())
    );
    // The firmware line ends the pane where the console has one (Mistress's `FirmwareLine`).
    assert_eq!(page.rows[0].pane.lines.get(1), Some(&("✓ Firmware found.".to_owned(), String::new())));
    assert_eq!(page.rows[1].pane.lines.len(), 1);
    // A starts the console, and the help bar says so; every other page of the card says Choose.
    assert_eq!(home.help(home_card::START_CONSOLE), Screen::CardStart);
    assert_eq!(home.help(home_card::FIRST), Screen::Card);
    assert_eq!(emusen_qt_models::help::entries(Screen::CardStart)[0].1, "Start");
    let chosen = home.choose(home_card::START_CONSOLE, &page.rows[1].key);
    assert_eq!(chosen, Choice::Act(start_console_action("Saturn")));
    assert_eq!(console_to_start(&start_console_action("Saturn")), Some("Saturn"));
    assert_eq!((console_to_start(home_card::START_CONSOLE), console_to_start(home_card::RESUME)), (None, None));
    // The card closes on it to the console's screen in both looks, from the Home card and the List card, as a game's
    // launch asks for it first; a start that fails gives way to the library's screen.
    for themed in [false, true] {
        for card in [Shown::HomeCard, Shown::List] {
            assert_eq!(step_in(themed, card, &act(&start_console_action("Saturn"))), Some((Shown::Game, None)), "{card:?} themed {themed}");
        }
        assert_eq!(step_in(themed, Shown::Game, &Ask::Closed).map(|s| s.0), Some(if themed { Shown::Esde } else { Shown::Home }));
    }

    // The List card's Start Console is the same page, under the List.
    let list = pages_of(list_card::CARD).unwrap();
    let lf = json!({ "section": "All", "count": "8 games", "sort": "Name", "filter": "", "letter": "A", "letters": "A", "startable": startable() });
    let first = list.page(list_card::FIRST, &lf);
    assert!(first.rows.iter().any(|r| r.key == list_card::START_CONSOLE), "offered where a console starts empty");
    assert_eq!(list.choose(list_card::FIRST, list_card::START_CONSOLE), Choice::Turn(list_card::START_CONSOLE.into()));
    let page = list.page(list_card::START_CONSOLE, &lf);
    assert_eq!((page.breadcrumb.as_str(), page.rows.len()), ("List", 2));
    assert_eq!(list.choose(list_card::START_CONSOLE, &page.rows[0].key), Choice::Act(start_console_action("PlayStation")));
    let none = json!({ "section": "All", "count": "8 games", "sort": "Name", "filter": "", "letter": "A", "letters": "A", "startable": [] });
    assert!(!list.page(list_card::FIRST, &none).rows.iter().any(|r| r.key == list_card::START_CONSOLE), "not offered with none");
    assert_eq!(list.help(list_card::START_CONSOLE), Screen::CardStart);

    // A theme's gamelist of a console that starts empty offers it first in Select's menu (Mistress's
    // `AddStartConsoleOption`), in its words, starting it at once.
    let ef = json!({ "section": "PlayStation", "count": "8 games", "sort": "Name", "filter": "", "letter": "A", "letters": "A", "console": startable()[0] });
    let first = list.page(list_card::FIRST, &ef);
    assert_eq!((first.rows[0].key.as_str(), first.rows[0].label.as_str()), (start_console_action("PlayStation").as_str(), "Start PlayStation With No Disc"));
    assert_eq!(first.rows[0].pane.lines[0].0, "Starts a PlayStation with no disc in it, on Diamond.");
    assert_eq!(list.choose(list_card::FIRST, &first.rows[0].key), Choice::Act(start_console_action("PlayStation")));
    assert_eq!(list.page(list_card::FIRST, &lf).rows[0].key, list_card::SEARCH, "a gamelist of another console leads with Search");
    // A console the facts name that cannot be read is said under the rows, not dropped unsaid.
    let bad = json!({ "section": "PlayStation", "count": "8 games", "sort": "Name", "filter": "", "letter": "A", "letters": "A", "console": { "console": 7 } });
    let first = list.page(list_card::FIRST, &bad);
    assert_eq!(first.rows[0].key, list_card::SEARCH);
    assert!(first.footer.starts_with(&list_card::console_unread("")), "said: {:?}", first.footer);
    assert!(list.page(list_card::FIRST, &ef).footer.is_empty() && list.page(list_card::FIRST, &lf).footer.is_empty());
}

#[test]
fn a_console_session_is_named_as_mistress_named_it() {
    assert_eq!(emusen_qt_models::session::console_session_name("PlayStation"), "PlayStation (no disc)");
    let home = pages_of(home_card::CARD).unwrap();
    let f: Value = serde_json::from_str(&Facts { suspended: emusen_qt_models::session::console_session_name("PlayStation"), ..Facts::default() }.to_json()).unwrap();
    let first = home.page(home_card::FIRST, &f);
    assert_eq!((first.rows[0].key.as_str(), first.rows[0].value.as_str()), (home_card::RESUME, "PlayStation (no disc)"));
}

#[test]
fn tools_cores_and_themes_open_their_screens_and_b_comes_back_to_where_the_card_was_opened() {
    let home = pages_of(home_card::CARD).unwrap();
    assert_eq!(home.choose(home_card::TOOLS, home_card::CORES), Choice::Act(home_card::CORES.into()));
    assert_eq!(home.choose(home_card::TOOLS, home_card::THEMES), Choice::Act(home_card::THEMES.into()));
    assert_eq!(THEMES, home_card::THEMES, "Settings' door and the tool's row open the same tab");
    for themed in [true, false] {
        let library = if themed { Shown::Esde } else { Shown::Home };
        assert_eq!(step_in(themed, Shown::HomeCard, &act(home_card::CORES)), Some((Shown::Cores, None)), "themed {themed}");
        assert_eq!(step_in(themed, Shown::HomeCard, &act(home_card::THEMES)), Some((Shown::Themes, None)), "themed {themed}");
        assert_eq!(tool_back(themed, Some(Shown::HomeCard), Shown::Cores, &Ask::Back), Some(library));
        assert_eq!(tool_back(themed, Some(Shown::HomeCard), Shown::Themes, &Ask::Back), Some(library));
        // A tool the card does not have a screen for closes the card to the library's screen, as before.
        assert_eq!(step_in(themed, Shown::HomeCard, &act("collections")), Some((library, None)));
        assert_eq!(step_in(themed, Shown::HomeCard, &act(home_card::CHEATS)), Some((Shown::Tool, Some(home_card::CHEATS.into()))), "the Cheats row opens its screen");
        assert_eq!(step_in(themed, Shown::HomeCard, &act(home_card::SCRAPER)), Some((Shown::Tool, Some(home_card::SCRAPER.into()))), "the Scraper's row opens its screen");
    }
    // Opened otherwise, each screen's own way back stands: the Themes tab to Settings, Cores from Home's place to Home.
    assert_eq!(tool_back(true, Some(Shown::Settings), Shown::Themes, &Ask::Back), None);
    assert_eq!(step_in(true, Shown::Themes, &Ask::Back), Some((Shown::Settings, None)));
    assert_eq!(tool_back(false, Some(Shown::Home), Shown::Cores, &Ask::Back), Some(Shown::Home));
}

#[test]
fn qt_7_9_lists_the_home_card_cannot_read_are_said_under_its_rows() {
    let home = pages_of(home_card::CARD).unwrap();
    let (startable, tools, unread) = home_card::read_lists(&serde_json::to_string(&startable()).unwrap(), "{}");
    assert_eq!((startable.len(), tools, unread.as_str()), (2, json!({}), ""), "read whole, nothing said");
    // Empty texts, as the library holds before a tool has given its line: none, and nothing said.
    let (startable, tools, unread) = home_card::read_lists("", "");
    assert_eq!((startable.len(), tools, unread.as_str()), (0, Value::Null, ""), "empty texts are none, not unread");
    let (startable, tools, unread) = home_card::read_lists("[{\"console\":", "not json");
    assert!(startable.is_empty() && tools.is_null());
    assert!(unread.starts_with("The consoles that start with no disc could not be read: "), "{unread}");
    assert!(unread.contains(" The tools' lines could not be read: "), "{unread}");
    let f: Value = serde_json::from_str(&Facts { unread: unread.clone(), ..Facts::default() }.to_json()).unwrap();
    assert_eq!(home.page(home_card::FIRST, &f).footer, unread, "said under the first page's rows");
}
