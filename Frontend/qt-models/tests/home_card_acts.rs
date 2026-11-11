//! The Home card's rows act (EmuSen_Qt.md §7.9, items 2 and 3): Continue shows the suspended game, Start Console lists
//! the consoles that start empty and starts the one chosen, from the Home card and the List card alike, and Tools ▸
//! Cores and ▸ Themes open those screens in both looks, B coming back to where the card was opened.

use emusen_qt_models::card::{pages_of, Choice};
use emusen_qt_models::home_card::{self, console_to_start, start_console_action, Facts, Startable};
use emusen_qt_models::list_card;
use emusen_qt_models::screens::{step, step_in, tool_back, Ask, Shown, THEMES};
use serde_json::{json, Value};

fn act(key: &str) -> Ask {
    Ask::Act(key.to_owned())
}

fn startable() -> Vec<Startable> {
    vec![
        Startable { console: "PlayStation".into(), name: "PlayStation".into(), engine: "Diamond".into() },
        Startable { console: "Saturn".into(), name: "Saturn".into(), engine: "Topaz".into() },
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
    let f: Value = serde_json::from_str(&Facts { start_console: true, startable: startable(), ..Facts::default() }.to_json()).unwrap();
    assert_eq!(home.choose(home_card::FIRST, home_card::START_CONSOLE), Choice::Turn(home_card::START_CONSOLE.into()), "the row opens its page");
    let page = home.page(home_card::START_CONSOLE, &f);
    assert_eq!((page.title.as_str(), page.breadcrumb.as_str()), ("Start Console", "Home"));
    assert_eq!(page.rows.iter().map(|r| (r.label.as_str(), r.value.as_str())).collect::<Vec<_>>(), [("PlayStation", "Diamond"), ("Saturn", "Topaz")]);
    assert_eq!(
        page.rows[0].pane.lines[0],
        ("Starts a PlayStation with no disc in it, on Diamond.".to_owned(), "Its BIOS screen comes up; Change Disc on the Pause Card puts a game in.".to_owned())
    );
    let chosen = home.choose(home_card::START_CONSOLE, &page.rows[1].key);
    assert_eq!(chosen, Choice::Act(start_console_action("Saturn")));
    assert_eq!(console_to_start(&start_console_action("Saturn")), Some("Saturn"));
    assert_eq!((console_to_start(home_card::START_CONSOLE), console_to_start(home_card::RESUME)), (None, None));
    // The card closes on it to the library's screen, where the console's start shows the game's screen.
    assert_eq!(step(Shown::HomeCard, &act(&start_console_action("Saturn"))), Some((Shown::Home, None)));
    assert_eq!(step(Shown::Home, &Ask::Play), Some((Shown::Game, None)));

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
        assert_eq!(step_in(themed, Shown::HomeCard, &act("scraper")), Some((library, None)));
    }
    // Opened otherwise, each screen's own way back stands: the Themes tab to Settings, Cores from Home's place to Home.
    assert_eq!(tool_back(true, Some(Shown::Settings), Shown::Themes, &Ask::Back), None);
    assert_eq!(step_in(true, Shown::Themes, &Ask::Back), Some((Shown::Settings, None)));
    assert_eq!(tool_back(false, Some(Shown::Home), Shown::Cores, &Ask::Back), Some(Shown::Home));
}
