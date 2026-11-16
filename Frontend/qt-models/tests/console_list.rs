//! Choose a console's chips and Y over the consoles a core runs (EmuSen_Qt.md §3.4.9): which each chip holds and counts,
//! and each order, each maker's consoles together; and Eclipse's Y and X declared by the help (`options_named`).

use emusen_qt_models::console_list::{counts, listed, Order, Show, ORDERS, SHOWS};
use emusen_qt_models::help::{self, options_named, Screen};
use emusen_qt_models::library::LibraryRow;
use emusen_qt_models::words;

fn row(key: &str, maker: &str, year: i32, count: usize, favourites: usize) -> LibraryRow {
    LibraryRow { key: key.into(), name: key.into(), maker: maker.into(), year, count, favourites, ..LibraryRow::default() }
}

fn keys(rows: &[&LibraryRow]) -> Vec<String> {
    rows.iter().map(|r| r.key.clone()).collect()
}

fn library() -> Vec<LibraryRow> {
    vec![
        row("snes", "Nintendo", 1990, 214, 2),
        row("md", "Sega", 1988, 40, 0),
        row("fds", "Nintendo", 1986, 0, 0),
        row("Gb", "Nintendo", 1989, 96, 0),
        row("sms", "Sega", 1985, 0, 0),
        row("arcade", "", 1978, 12, 1),
        row("psx", "Sony", 1994, 300, 0),
    ]
}

#[test]
fn qt_3_4_9_each_chip_holds_and_counts_its_consoles_and_each_order_keeps_a_maker_s_together() {
    let all = library();
    assert_eq!(counts(&all), [7, 5, 2], "All, With games, Favourites");
    assert_eq!(keys(&listed(&all, Show::All, Order::Maker)), ["snes", "fds", "Gb", "md", "sms", "arcade", "psx"]);
    assert_eq!(keys(&listed(&all, Show::WithGames, Order::Maker)), ["snes", "Gb", "md", "arcade", "psx"]);
    assert_eq!(keys(&listed(&all, Show::Favourites, Order::Maker)), ["snes", "arcade"]);
    assert_eq!(keys(&listed(&all, Show::All, Order::Year)), ["fds", "Gb", "snes", "sms", "md", "arcade", "psx"]);
    assert_eq!(keys(&listed(&all, Show::All, Order::Name)), ["fds", "Gb", "snes", "md", "sms", "arcade", "psx"]);
    assert_eq!(keys(&listed(&all, Show::All, Order::Games)), ["snes", "Gb", "fds", "md", "sms", "arcade", "psx"]);
    assert_eq!((SHOWS.len(), ORDERS.len()), (words::CONSOLE_SHOWS.len(), words::CONSOLE_SORTS.len()), "a word for each");
}

#[test]
fn qt_3_4_9_eclipse_s_y_and_x_are_the_sort_and_the_console_s_options_with_their_action_lines() {
    let named = options_named(help::entries_grouped(Screen::Consoles, Some(help::MAKER), false));
    let lines = help::lines_of(Screen::Consoles, &named);
    assert!(named.contains(&(emusen_kit_core::input::UiButton::Search, words::SORT, Some(help::SORT))), "{named:?}");
    assert!(named.contains(&(emusen_kit_core::input::UiButton::Screensaver, words::OPTIONS, Some(help::CONSOLE_OPTIONS))), "{named:?}");
    assert!(lines.contains("sort Search\n") && lines.contains("consoleOptions Screensaver\n"), "{lines}");
    assert!(!lines.contains("favourites") && !lines.contains("random"), "Favourites and Random game gave way: {lines}");
    assert_eq!(help::action_lines(Screen::Consoles), help::lines_of(Screen::Consoles, help::entries(Screen::Consoles)));
    assert!(help::action_lines(Screen::Consoles).contains("random Screensaver\n"), "the own look keeps Random game");
}

#[test]
fn qt_3_4_9_a_console_s_options_go_from_choose_a_console_to_settings() {
    use emusen_qt_models::screens::{console_options, step, Ask, Shown};
    let asked = format!("{}:snes", help::CONSOLE_OPTIONS);
    assert_eq!(console_options(&asked), Some("snes"));
    assert_eq!(console_options(help::CONSOLE_OPTIONS), None, "no console named");
    assert_eq!(step(Shown::Consoles, &Ask::Act(asked.clone())), Some((Shown::Settings, None)));
    assert_eq!(step(Shown::Grid, &Ask::Act(asked)), None, "asked of Choose a console alone");
}
