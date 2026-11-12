//! The states theme-audit records (man theme-check, STATES): the one list of their names and what each feeds, read by the audit,
//! printed by --list-states and held to the man page by a test. Every state reaches the theme through its own surfaces.

use emusen_theme_manifest::Screen;

use crate::world::{LONG, SUSPENDED};

/// The library a state is fed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Library {
    /// The synthetic library: eight games on four consoles, played, two hearted, each with its cover.
    Synthetic,
    /// The same eight games with no cover.
    NoCovers,
    /// The same eight, never played and none hearted.
    Unplayed,
    /// The synthetic library's games of one console alone.
    OneConsole,
    /// The eight and a game whose title is too long for any place, played last.
    Long,
    /// The synthetic library with the navigation sound at full volume.
    Loud,
    /// A catalogue of Testing/themes/catalogues, by its stem and its text.
    Catalogue(&'static str, &'static str),
}

/// One state: the screens it is recorded on and what the host feeds them.
#[derive(Debug)]
pub struct State {
    pub name: &'static str,
    pub screens: &'static [Screen],
    pub library: Library,
    /// The suspended game's title; none for no game to continue.
    pub game: Option<&'static str>,
    /// The places the host would decline, by key.
    pub declined: &'static [&'static str],
    /// The place, by key, the pad walks the focus to before the state is judged.
    pub focus: Option<&'static str>,
    /// A held press of A on the focus.
    pub press: bool,
    /// The animation clock held at this many ms after the screen is placed, motion allowed.
    pub clock: Option<i64>,
    pub words: &'static str,
}

/// The state every package is held to without --states.
pub const DEFAULT: &str = "default";

const ALL_THREE: &[Screen] = &[Screen::Home, Screen::Consoles, Screen::Games];

macro_rules! catalogue {
    ($stem:literal) => {
        Library::Catalogue($stem, include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../Testing/themes/catalogues/", $stem, ".tsv")))
    };
}

const fn state(name: &'static str, screens: &'static [Screen], library: Library, game: Option<&'static str>, words: &'static str) -> State {
    State { name, screens, library, game, declined: &[], focus: None, press: false, clock: None, words }
}

/// Every state, the default first and the held clocks last, since letting the clock go leaves Qt's own driver in place.
pub const STATES: [State; 15] = [
    state(DEFAULT, &[Screen::Home, Screen::Consoles, Screen::Games, Screen::Splash], Library::Synthetic, Some(SUSPENDED), "the synthetic library with a game suspended; the splash finished under reduced motion; Choose a console again over 120 consoles"),
    state("nogame", &[Screen::Home], Library::Synthetic, None, "no game to continue"),
    state("empty", ALL_THREE, catalogue!("empty"), None, "no console and no game; Games shows the All shelf"),
    state("one-console", ALL_THREE, Library::OneConsole, Some(SUSPENDED), "the games of one console alone, one of them suspended"),
    state("one-game", &[Screen::Games], catalogue!("one-game"), None, "a console holding one game"),
    state("many", &[Screen::Games], catalogue!("many"), None, "a console holding 9,000 games"),
    state("nocover", ALL_THREE, Library::NoCovers, Some(SUSPENDED), "no game has a cover"),
    state("norecent", ALL_THREE, Library::Unplayed, None, "nothing played and nothing hearted"),
    state("long", ALL_THREE, Library::Long, Some(LONG), "a game whose title is too long for its place, played last and suspended"),
    State { press: true, ..state("pressed", ALL_THREE, Library::Synthetic, Some(SUSPENDED), "A held on the first focus") },
    State {
        declined: &["scraper"],
        focus: Some("scraper"),
        ..state("disabled", &[Screen::Home], Library::Synthetic, Some(SUSPENDED), "the Scraper declined by the host, the focus walked to it")
    },
    state("loud", &[Screen::Home], Library::Loud, Some(SUSPENDED), "the navigation sound at full volume: the Settings place's longest line"),
    State { clock: Some(1000), ..state("splash-1s", &[Screen::Splash], Library::Synthetic, Some(SUSPENDED), "the splash 1 s after it is placed") },
    State { clock: Some(3000), ..state("splash-3s", &[Screen::Splash], Library::Synthetic, Some(SUSPENDED), "the splash at 3 s") },
    State { clock: Some(5000), ..state("splash-5s", &[Screen::Splash], Library::Synthetic, Some(SUSPENDED), "the splash at 5 s") },
];

/// The states a --states list names, `all` for every one, in the list's order; the error names one it does not know.
pub fn chosen(list: &str) -> Result<Vec<&'static State>, String> {
    let names: Vec<&str> = list.split(',').map(str::trim).filter(|n| !n.is_empty()).collect();
    if let Some(unknown) = names.iter().find(|n| **n != "all" && !STATES.iter().any(|s| s.name == **n)) {
        return Err(format!("theme-audit: {unknown} is not a state; theme-audit --list-states names them"));
    }
    Ok(STATES.iter().filter(|s| names.contains(&"all") || names.contains(&s.name)).collect())
}

/// One line a state, tab-separated: its name, its screens and its words.
pub fn listed() -> String {
    STATES
        .iter()
        .map(|s| format!("{}\t{}\t{}\n", s.name, s.screens.iter().map(|x| x.file().trim_start_matches("screens/").trim_end_matches(".qml")).collect::<Vec<_>>().join(","), s.words))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{chosen, listed, STATES};

    #[test]
    fn qt_3_7_the_man_page_lists_every_state_and_nothing_else() {
        let page = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../Docs/man/emusen.dev/theme-check.1")).unwrap();
        let section = page.split("\nSTATES\n").nth(1).and_then(|s| s.split("\nEXIT STATUS\n").next()).expect("the man page's STATES section");
        // An entry is a name in the columns 4 to 15, then its words from column 16, as the page's other lists are set.
        let entry = |l: &&str| l.len() > 16 && l.starts_with("    ") && !l[4..16].trim_end().contains(' ') && l.as_bytes()[15] == b' ' && l.as_bytes()[4] != b' ';
        let named: Vec<&str> = section.lines().filter(entry).filter_map(|l| l.split_whitespace().next()).collect();
        let owned: Vec<&str> = STATES.iter().map(|s| s.name).collect();
        assert_eq!(named, owned, "the page's states, in the list's order");
    }

    #[test]
    fn qt_3_7_a_states_list_is_read_in_the_list_s_order_and_an_unknown_name_refused() {
        assert_eq!(chosen("all").unwrap().len(), STATES.len());
        let two: Vec<&str> = chosen("splash-3s, nogame").unwrap().iter().map(|s| s.name).collect();
        assert_eq!(two, ["nogame", "splash-3s"]);
        assert!(chosen("nocovers").unwrap_err().contains("nocovers is not a state"));
        assert_eq!(listed().lines().count(), STATES.len());
        assert!(listed().starts_with("default\tHome,Consoles,Games,Splash\t"), "{}", listed());
    }
}
