//! `Screens`: which of the program's screens is shown and which row the grid opened, so the screens switch by calls,
//! never by logic in QML (EmuSen_Qt.md §7.1). `step` is the whole rule; the bridge only puts it on Qt, after the event
//! that asked, so one press is never answered by the screen it left and the screen it opened.

use crate::help::{HOME, NEXT, PREVIOUS};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;

/// A screen of the program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shown {
    Home,
    Consoles,
    Grid,
    /// The game, its picture filling the window.
    Game,
    /// The Pause Card over the paused game.
    Pause,
    /// The Home card over Home, opened at its Power page by Home's place (EmuSen_BigPicture.md §51.2) and at its first
    /// page by Start on Home.
    HomeCard,
    /// The List card over the grid (EmuSen_Interface.md §18.1).
    List,
    /// The grid's search, the keyboard docked and holding the focus.
    Search,
    /// The grid's search, the keyboard docked and the focus on the covers (LunaP's `GalleryLayout` out of the keyboard).
    SearchGrid,
    /// The ES-DE theme's view in place of Home, Choose a console and the grid, its system and gamelist views one screen
    /// whose model says which (EmuSen_Qt.md §4.6).
    Esde,
    /// The Themes tab, under Settings ▸ Library ▸ Themes (EmuSen_Qt.md §3.8, §7.7).
    Themes,
    /// Settings, from Home's Settings place and the Home card's Settings row (EmuSen_Qt.md §7.7).
    Settings,
    /// Cores on the tool screen, from Home's place (EmuSen_Interface.md §15.8).
    Cores,
    /// A tool of jobs on the tool screen (Library Manager, Compressed Discs, Packages), from Home's place or the Home card's
    /// Tools page; `Screens.tool` names which.
    Tool,
    /// A game's page, from X on Home or the grid, a theme's `openPage` and the Home card's Selected game (EmuSen_Qt.md
    /// §7.8); B returns to where it was opened.
    Page,
    /// Eclipse Collections hub (#11).
    Collections,
    /// Eclipse Tools hub (#9).
    Tools,
}

/// What asked for a change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    /// One of Home's places, by its index in `words::HOME_PLACES`.
    Place(usize),
    /// A row chosen on Choose a console or a collection opened on Collections.
    Open(String),
    /// A game's page opened directly by path, as from Search results.
    GamePage(String),
    /// B that no screen kept.
    Back,
    /// A screen's declared action (crate::help) or a card's row's action, by its key.
    Act(String),
    /// A game chosen on Home's shelf or the grid, Continue chosen, or the Pause Card resuming: the game is shown.
    Play,
    /// Resuming an already running session from Continue or Home card; treated as started play.
    ResumePlay,
    /// Playback began after launch/resume prompts cleared.
    Started,
    /// Launching or opening the game failed.
    Failed,
    /// The menu chord over the game: the Pause Card, as Mistress's `OpenPadMenu` opens it over a game.
    Menu,
    /// The Pause Card's Back to Library: Home, the game suspended behind it.
    Library,
    /// The game closed: Home.
    Closed,
    /// A card closed by B on its first page or by Start.
    Close,
    /// The focus went to the search's keys (true) or its covers (false) by other means than the shoulders: the pointer.
    Keys(bool),
    /// The library's look changed between the own look and an ES-DE theme: the library's screen shown is the other's.
    Restyled,
    /// One of the program's sections chosen on a nav, by its key (crate::sections).
    Section(String),
    /// A Settings door opened from Desktop Mode (`scraper`, `compressed`, `cheats`).
    DesktopDoor(String),
}

/// A top section of the program: its key, its name as a nav says it, the screen it opens, the key that screen is opened
/// at (a grid's, the Home card's page) and the List card's page, where it opens one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Section {
    pub key: &'static str,
    pub name: &'static str,
    pub screen: Shown,
    pub at: Option<&'static str>,
    pub page: Option<&'static str>,
}

/// The program's sections in a nav's order (Main.dc.html:34-38, R-20261115-7), each opening the program's screen that holds
/// it today (EmuSen_Qt.md §3.4.8).
/// The Consoles section's key, which the game page's trail opens.
pub const CONSOLES_KEY: &str = "consoles";

pub const SECTIONS: [Section; 5] = [
    Section { key: HOME, name: crate::words::HOME_SECTION, screen: Shown::Home, at: None, page: None },
    Section { key: "library", name: crate::words::LIBRARY_SECTION, screen: Shown::Grid, at: Some(crate::library::ALL), page: None },
    Section { key: CONSOLES_KEY, name: crate::words::CONSOLES_SECTION, screen: Shown::Consoles, at: None, page: None },
    Section {
        key: "collections",
        name: crate::words::COLLECTIONS_SECTION,
        screen: Shown::List,
        at: Some(crate::library::ALL),
        page: Some(crate::list_card::FILTER),
    },
    Section { key: "tools", name: crate::words::TOOLS_SECTION, screen: Shown::HomeCard, at: Some(crate::home_card::TOOLS), page: None },
];

/// Home's Settings and Power places, by their index in `words::HOME_PLACES`.
pub const SETTINGS_PLACE: usize = 5;
pub const POWER_PLACE: usize = 6;
pub const CORES_PLACE: usize = 4;

/// The screen a tool opens, by its key: Cores, the Themes tab, or the tool screen for the Scraper, Cheats or a tool of jobs; none for a tool with
/// no screen yet, which Home and the Home card do not offer.
pub fn tool_screen(key: &str) -> Option<Shown> {
    match key {
        crate::home_card::CORES => Some(Shown::Cores),
        crate::home_card::THEMES => Some(Shown::Themes),
        crate::home_card::SCRAPER | crate::home_card::CHEATS => Some(Shown::Tool),
        _ if crate::tools::jobs::tools().iter().any(|t| t == key) => Some(Shown::Tool),
        _ => None,
    }
}

/// A tool's screen with the tool named, by its key; none for a tool with no screen.
fn open_tool(key: &str) -> Option<(Shown, Option<String>)> {
    tool_screen(key).map(|shown| (shown, (shown == Shown::Tool).then(|| key.to_owned())))
}

/// The grid's List card and the search, by the actions that open them.
pub const LIST: &str = "list";
pub const SEARCH: &str = "search";
/// A shoulder on the keyboard: the focus to the results (the covers, a tool's rows), while there are any.
pub const TO_GRID: &str = "toGrid";

/// The rule of a search with its keyboard docked (EmuSen_Qt.md §7.6, R-20261111-31): the shoulders move the focus between
/// the keys and the results, the keyboard staying docked; whether the keys hold the focus after `key`, none if no move.
pub fn search_shoulder(on_keys: bool, key: &str) -> Option<bool> {
    match (on_keys, key) {
        (true, TO_GRID) => Some(false),
        (false, k) if k == PREVIOUS || k == NEXT => Some(true),
        _ => None,
    }
}
/// The console `consoleOptions:<console>` names (Eclipse's X, EclipseConsoles:94); none for every other action.
pub fn console_options(action: &str) -> Option<&str> {
    action.strip_prefix(crate::help::CONSOLE_OPTIONS)?.strip_prefix(':').filter(|c| !c.is_empty())
}

/// Settings' door to the Themes tab, by its name in the screen's description.
pub const THEMES: &str = "themes";
/// Settings' door to the cheats of the game last played: that game's page, opened at Options' Cheats.
pub const CHEATS: &str = crate::game_page::CHEATS;

/// What a door of Settings opens beside the Themes tab: a tool it names, or the last game's cheats; none for a door with
/// no screen yet.
fn settings_door(key: &str) -> Option<(Shown, Option<String>)> {
    if key == CHEATS {
        return Some((Shown::Page, Some(CHEATS.to_owned())));
    }
    open_tool(key)
}
/// The Pause Card's Controllers ▸ Bindings: Settings' drawing page over the card, B back to it (EmuSen_Interface.md §20.3).
pub const CARD_BINDINGS: &str = crate::settings_controllers::CARD_BINDINGS;

/// The screen shown after an ask, and the grid's row (the page a card opens at, for a card); `None` leaves both as they
/// were.
pub fn step(shown: Shown, ask: &Ask) -> Option<(Shown, Option<String>)> {
    step_at(shown, "", ask)
}

/// As `step`, with the shown screen's current key `at` so top-level Library maps to `"library"`.
pub fn step_at(shown: Shown, at: &str, ask: &Ask) -> Option<(Shown, Option<String>)> {
    match (shown, ask) {
        (_, Ask::Section(key)) => crate::sections::opened_at(shown, at, key).map(|s| (s.screen, s.at.map(str::to_owned))),
        (Shown::Home, Ask::Place(0)) => Some((Shown::Consoles, None)),
        (Shown::Home, Ask::Place(SETTINGS_PLACE)) => Some((Shown::Settings, None)),
        (Shown::Home, Ask::Place(POWER_PLACE)) => Some((Shown::HomeCard, Some(crate::home_card::POWER.to_owned()))),
        (Shown::Home | Shown::Grid | Shown::SearchGrid | Shown::HomeCard, Ask::Act(key)) if key == GAME_PAGE => Some((Shown::Page, None)),
        (Shown::Home, Ask::Act(key)) if key == SEARCH => Some((Shown::Search, Some(crate::library::ALL.to_owned()))),
        // Start on Home opens the Home card at its first page (decided 2026-11-11); Mistress's did so on the desktop, her big screen's focused Home's start.
        (Shown::Home, Ask::Act(key)) if key == HOME => Some((Shown::HomeCard, Some(crate::home_card::FIRST.to_owned()))),
        (Shown::Consoles | Shown::Grid | Shown::Search | Shown::SearchGrid, Ask::Act(key)) if key == HOME => Some((Shown::Home, None)),
        (Shown::HomeCard, Ask::Back | Ask::Close) => Some((Shown::Home, None)),
        (Shown::HomeCard, Ask::Act(key)) if key == SEARCH => Some((Shown::Search, Some(crate::library::ALL.to_owned()))),
        (Shown::HomeCard, Ask::Act(key)) if key == crate::home_card::LIBRARY => Some((Shown::Grid, Some(crate::library::ALL.to_owned()))),
        (Shown::HomeCard, Ask::Act(key)) if key == crate::home_card::SETTINGS => Some((Shown::Settings, None)),
        (Shown::HomeCard, Ask::Act(key)) => card_act(false, key),
        (Shown::Settings, Ask::Back) => Some((Shown::Home, None)),
        (Shown::Settings | Shown::Themes, Ask::Act(key)) if key == HOME => Some((Shown::Home, None)),
        (Shown::Settings, Ask::Act(key)) if key == THEMES => Some((Shown::Themes, None)),
        (Shown::Settings, Ask::Act(key)) => settings_door(key),
        // The Themes tab is a page of Settings: B goes back to it.
        (Shown::Themes, Ask::Back) => Some((Shown::Settings, None)),
        (Shown::Grid, Ask::Act(key)) if key == LIST => Some((Shown::List, None)),
        (Shown::List, Ask::Back | Ask::Close) => Some((Shown::Grid, None)),
        (Shown::List, Ask::Act(key)) if key == SEARCH => Some((Shown::Search, None)),
        // A console started from its Start Console page is the game's screen, as a game's launch asks for it first.
        (Shown::List, Ask::Act(key)) if crate::home_card::console_to_start(key).is_some() => Some((Shown::Game, None)),
        // Every other action of the List card is the grid's, and the card closes on it.
        (Shown::List, Ask::Act(_)) => Some((Shown::Grid, None)),
        // The keyboard closed, by X keeping the words or by B on an empty field: the grid.
        (Shown::Search, Ask::Back) => Some((Shown::Grid, None)),
        // The shoulders move the focus between the keyboard and the covers, the keyboard staying docked.
        (Shown::Search, Ask::Act(key)) if search_shoulder(true, key) == Some(false) => Some((Shown::SearchGrid, None)),
        (Shown::SearchGrid, Ask::Act(key)) if search_shoulder(false, key) == Some(true) => Some((Shown::Search, None)),
        (Shown::Search, Ask::Keys(false)) => Some((Shown::SearchGrid, None)),
        (Shown::SearchGrid, Ask::Keys(true)) => Some((Shown::Search, None)),
        // From the covers B is the grid's: the search cleared first (GameModel), then Choose a console.
        (Shown::SearchGrid, Ask::Back) => Some((Shown::Consoles, None)),
        (Shown::SearchGrid, Ask::Act(key)) if key == LIST => Some((Shown::List, None)),
        (Shown::Consoles, Ask::Open(key)) => Some((Shown::Grid, Some(key.clone()))),
        (Shown::Consoles, Ask::Act(key)) if key == crate::library::FAVOURITES => Some((Shown::Grid, Some(key.clone()))),
        (Shown::Consoles, Ask::Act(key)) if console_options(key).is_some() => Some((Shown::Settings, None)),
        (Shown::Consoles, Ask::Back) => Some((Shown::Home, None)),
        (Shown::Grid, Ask::Back) => Some((Shown::Consoles, None)),
        (Shown::Home | Shown::Grid | Shown::Pause | Shown::List | Shown::Search | Shown::SearchGrid, Ask::Play) => Some((Shown::Game, None)),
        (Shown::Game, Ask::Menu) => Some((Shown::Pause, None)),
        (Shown::Pause, Ask::Act(key)) if key == CARD_BINDINGS => Some((Shown::Settings, None)),
        (Shown::Pause, Ask::Act(key)) if key == crate::home_card::CHEATS => Some((Shown::Tool, Some(crate::home_card::CHEATS.to_owned()))),
        (Shown::Game | Shown::Pause, Ask::Library) => Some((Shown::Home, None)),
        (Shown::Game | Shown::Pause, Ask::Closed) => Some((Shown::Home, None)),
        (Shown::Home, Ask::Place(at)) if (1..=crate::words::HOME_TOOLS.len()).contains(at) => open_tool(crate::words::HOME_TOOLS[at - 1]),
        (Shown::Cores | Shown::Tool, Ask::Back) => Some((Shown::Home, None)),
        _ => None,
    }
}

/// The Home card's row's action besides Settings and a game's page: Continue the game and a console started its screen,
/// a tool of the Tools page its screen; every other row closes the card to the library's screen (EmuSen_Qt.md §7.9).
fn card_act(themed: bool, key: &str) -> Option<(Shown, Option<String>)> {
    if let Some(tool) = open_tool(key) {
        return Some(tool);
    }
    let shown = match key {
        crate::home_card::RESUME => Shown::Game,
        _ if crate::home_card::console_to_start(key).is_some() => Shown::Game,
        _ => library_of(themed),
    };
    Some((shown, None))
}

/// Where B on Cores or the Themes tab returns where the Home card opened it: the library's screen, the card having
/// closed as it opened it; Settings where its door opened a tool; `None` where something else opened it, whose own rule
/// stands.
pub fn tool_back(themed: bool, opened_over: Option<Shown>, shown: Shown, ask: &Ask) -> Option<Shown> {
    match (opened_over, shown, ask) {
        (Some(Shown::Settings), Shown::Cores | Shown::Tool, Ask::Back) => Some(Shown::Settings),
        (Some(Shown::Tools), Shown::Cores | Shown::Themes | Shown::Tool, Ask::Back) => Some(Shown::Tools),
        (Some(Shown::Page), Shown::Tool, Ask::Back) => Some(Shown::Page),
        (Some(Shown::Pause), Shown::Tool, Ask::Back) => Some(Shown::Pause),
        (Some(Shown::HomeCard), Shown::Cores | Shown::Themes | Shown::Tool, Ask::Back) => Some(library_of(themed)),
        (_, Shown::Cores | Shown::Tool, Ask::Back) => Some(library_of(themed)),
        _ => None,
    }
}

/// The game's page by the help bar's action that opens it.
pub const GAME_PAGE: &str = crate::help::GAME_PAGE;

/// Where B on a game's page returns: the screen it was opened over, the library's screen for a card (the card closed
/// as it opened the page).
pub fn page_origin(themed: bool, opened_over: Shown) -> Shown {
    match opened_over {
        Shown::HomeCard => library_of(themed),
        other => other,
    }
}

/// A game's page's way out: B to where it was opened, Start to the library's screen, a game started to the game, and a
/// theme's trail to Choose a console or a console's games.
pub fn page_step(themed: bool, origin: Shown, ask: &Ask) -> Option<Shown> {
    match ask {
        Ask::Back => Some(origin),
        Ask::Act(key) if key == HOME => Some(library_of(themed)),
        Ask::Act(key) if key == crate::home_card::CHEATS => Some(Shown::Tool),
        Ask::Play => Some(Shown::Game),
        // A theme's trail (Eclipse's game page, EclipseGame:30-32): a section that opens at no key, and a console's games.
        Ask::Section(key) => crate::sections::by_key(key).filter(|s| s.at.is_none() && s.page.is_none()).map(|s| s.screen),
        Ask::Open(_) => Some(Shown::Grid),
        _ => None,
    }
}

/// The library's screen of a look: Home for the own look, the theme's view for an ES-DE theme.
pub fn library_of(themed: bool) -> Shown {
    if themed {
        Shown::Esde
    } else {
        Shown::Home
    }
}

/// As `step`, under the library's look: with an ES-DE theme the theme's view stands where Home, Choose a console and
/// the grid stood, and the cards, the game and the Pause Card come back to it (EmuSen_Qt.md §4.6).
pub fn step_in(themed: bool, shown: Shown, ask: &Ask) -> Option<(Shown, Option<String>)> {
    step_in_at(themed, "", shown, ask)
}

/// Records whether playback already started and which screen should regain focus if opening fails or is cancelled.
fn record_play_origin(themed: bool, was: Shown, ask: &Ask) -> (bool, Option<Shown>) {
    if *ask == Ask::ResumePlay {
        return (true, None);
    }
    let origin = match was {
        Shown::List | Shown::HomeCard => library_of(themed),
        other => other,
    };
    (false, Some(origin))
}

/// Returns the screen `B` returns to from `Shown::Grid` based on how it was entered.
fn grid_origin_for(was: Shown, ask: &Ask) -> Shown {
    match was {
        Shown::Consoles if !matches!(ask, Ask::Section(_)) => Shown::Consoles,
        Shown::Collections if !matches!(ask, Ask::Section(_)) => Shown::Collections,
        Shown::Home | Shown::HomeCard | Shown::Tools => Shown::Home,
        _ if matches!(ask, Ask::Section(_)) => Shown::Home,
        _ => Shown::Consoles,
    }
}

fn next_grid_origin(was: Shown, shown: Shown, ask: &Ask, current: Option<Shown>) -> Option<Shown> {
    let keep_origin = matches!(was, Shown::List | Shown::Search | Shown::SearchGrid | Shown::Game | Shown::Pause)
        || (was == Shown::Page && *ask == Ask::Back);
    if shown == Shown::Grid && was != Shown::Grid {
        if keep_origin { current } else { Some(grid_origin_for(was, ask)) }
    } else if matches!(shown, Shown::Grid | Shown::List | Shown::Search | Shown::SearchGrid | Shown::Page | Shown::Game | Shown::Pause) {
        current
    } else {
        None
    }
}

/// Returns the destination screen for a failure or unstarted close from Game/Pause.
fn close_origin(themed: bool, play_origin: Option<Shown>, play_started: bool, ask: &Ask) -> Option<Shown> {
    if *ask == Ask::Failed || (*ask == Ask::Closed && !play_started) {
        Some(play_origin.unwrap_or_else(|| library_of(themed)))
    } else {
        None
    }
}

fn step_in_at(themed: bool, at: &str, shown: Shown, ask: &Ask) -> Option<(Shown, Option<String>)> {
    let own = matches!(shown, Shown::Home | Shown::Consoles | Shown::Grid | Shown::Search | Shown::SearchGrid | Shown::Collections | Shown::Tools);
    match (themed, shown, ask) {
        (true, _, Ask::Restyled) if own => Some((Shown::Esde, None)),
        (false, Shown::Esde, Ask::Restyled) => Some((Shown::Home, None)),
        (_, _, Ask::Restyled) => None,
        (false, ..) => step_at(shown, at, ask),
        // Start opens the Home card at its first page, as ES-DE's Start opens its menu; B at the theme's view is the
        // view's own, going back from a gamelist, and does nothing more here (decided 2026-11-08).
        (true, Shown::Esde, Ask::Act(key)) if key == HOME => Some((Shown::HomeCard, Some(crate::home_card::FIRST.to_owned()))),
        (true, Shown::Esde | Shown::HomeCard, Ask::Act(key)) if key == GAME_PAGE => Some((Shown::Page, None)),
        (true, Shown::Esde, Ask::Act(key)) if key == LIST => Some((Shown::List, None)),
        (true, Shown::Esde | Shown::List | Shown::Pause, Ask::Play) => Some((Shown::Game, None)),
        (true, Shown::HomeCard, Ask::Act(key)) if key == crate::home_card::SETTINGS => Some((Shown::Settings, None)),
        (true, Shown::HomeCard, Ask::Act(key)) => card_act(true, key),
        (true, Shown::List, Ask::Act(key)) if crate::home_card::console_to_start(key).is_some() => Some((Shown::Game, None)),
        (true, Shown::Settings, Ask::Back) => Some((Shown::Esde, None)),
        (true, Shown::Settings | Shown::Themes, Ask::Act(key)) if key == HOME => Some((Shown::Esde, None)),
        (true, Shown::Settings, Ask::Act(key)) if key == THEMES => Some((Shown::Themes, None)),
        (true, Shown::Settings, Ask::Act(key)) => settings_door(key),
        (true, Shown::Themes, Ask::Back) => Some((Shown::Settings, None)),
        // The cards close to the theme's view on B, Start or any of their actions, which the host carries out.
        (true, Shown::HomeCard | Shown::List, Ask::Back | Ask::Close | Ask::Act(_)) => Some((Shown::Esde, None)),
        (true, Shown::Game, Ask::Menu) => Some((Shown::Pause, None)),
        (true, Shown::Pause, Ask::Act(key)) if key == CARD_BINDINGS => Some((Shown::Settings, None)),
        (true, Shown::Pause, Ask::Act(key)) if key == crate::home_card::CHEATS => Some((Shown::Tool, Some(crate::home_card::CHEATS.to_owned()))),
        (true, Shown::Game | Shown::Pause, Ask::Library) => Some((Shown::Esde, None)),
        (true, Shown::Game | Shown::Pause, Ask::Closed) => Some((Shown::Esde, None)),
        _ => None,
    }
}

fn eclipse_section_at(shown: Shown, key: &str) -> Option<&'static str> {
    match shown {
        Shown::Home => Some(HOME),
        Shown::Grid if key == crate::library::ALL => Some("library"),
        Shown::Consoles => Some(CONSOLES_KEY),
        Shown::Collections => Some("collections"),
        Shown::Tools => Some("tools"),
        _ => None,
    }
}

/// Eclipse-aware screen step, routing Collections and Tools hubs when `eclipse` is active.
pub fn step_in_look(eclipse: bool, themed: bool, shown: Shown, ask: &Ask) -> Option<(Shown, Option<String>)> {
    step_in_look_at(eclipse, themed, shown, "", ask)
}

/// As `step_in_look`, carrying Grid's `at` key so top-level Library maps to `"library"`.
pub fn step_in_look_at(eclipse: bool, themed: bool, shown: Shown, at: &str, ask: &Ask) -> Option<(Shown, Option<String>)> {
    if matches!(ask, Ask::GamePage(_)) && matches!(shown, Shown::SearchGrid | Shown::Grid | Shown::Home | Shown::HomeCard) {
        return Some((Shown::Page, None));
    }
    if !eclipse || themed {
        return step_in_at(themed, at, shown, ask);
    }
    match (shown, ask) {
        (_, Ask::Section(key)) => {
            let current = eclipse_section_at(shown, at)?;
            if current == key.as_str() {
                return None;
            }
            match key.as_str() {
                HOME => Some((Shown::Home, None)),
                "library" => Some((Shown::Grid, Some(crate::library::ALL.to_owned()))),
                CONSOLES_KEY => Some((Shown::Consoles, None)),
                "collections" => Some((Shown::Collections, None)),
                "tools" => Some((Shown::Tools, None)),
                _ => None,
            }
        }
        (Shown::Grid, Ask::Act(key)) if at == crate::library::ALL && key == PREVIOUS => Some((Shown::Home, None)),
        (Shown::Grid, Ask::Act(key)) if at == crate::library::ALL && key == NEXT => Some((Shown::Consoles, None)),
        (Shown::Collections, Ask::Open(key)) => Some((Shown::Grid, Some(key.clone()))),
        (Shown::Collections | Shown::Tools, Ask::Back) => Some((Shown::Home, None)),
        (Shown::Collections | Shown::Tools, Ask::Act(key)) if key == HOME => Some((Shown::Home, None)),
        (Shown::Tools, Ask::Act(key)) if key == THEMES || key == "cards" => Some((Shown::Settings, None)),
        (Shown::Tools, Ask::Act(key)) if key == crate::home_card::SETTINGS => Some((Shown::Settings, None)),
        (Shown::Tools, Ask::Act(key)) if console_options(key).is_some() => Some((Shown::Settings, None)),
        (Shown::Tools, Ask::Act(key)) => open_tool(key),
        _ => step_in_at(false, at, shown, ask),
    }
}

/// Returns the adjacent shelf key from `rows` around `current` when stepping by `delta`.
pub fn neighbour_shelf(rows: &[crate::library::LibraryRow], current: &str, delta: i32) -> Option<String> {
    if rows.len() < 2 {
        return None;
    }
    let at = rows.iter().position(|r| r.key == current)?;
    let next = (at as isize + delta as isize).rem_euclid(rows.len() as isize) as usize;
    Some(rows[next].key.clone())
}

/// Face-aware screen step allowing Desktop launches and Game Page transitions from any non-running Desktop view.
pub fn step_for_face(big_screen: bool, eclipse: bool, themed: bool, shown: Shown, ask: &Ask) -> Option<(Shown, Option<String>)> {
    step_for_face_at(big_screen, eclipse, themed, shown, "", ask)
}

/// As `step_for_face`, carrying Grid's active shelf key `at`.
pub fn step_for_face_at(big_screen: bool, eclipse: bool, themed: bool, shown: Shown, at: &str, ask: &Ask) -> Option<(Shown, Option<String>)> {
    if let Ask::DesktopDoor(key) = ask {
        return settings_door(key);
    }
    if !big_screen {
        match ask {
            Ask::Play
                if matches!(
                    shown,
                    Shown::Home
                        | Shown::Consoles
                        | Shown::Grid
                        | Shown::Pause
                        | Shown::List
                        | Shown::Search
                        | Shown::SearchGrid
                        | Shown::Settings
                        | Shown::Themes
                        | Shown::Collections
                        | Shown::Tools
                        | Shown::Cores
                        | Shown::Tool
                        | Shown::Esde
                        | Shown::Page
                ) =>
            {
                return Some((Shown::Game, None));
            }
            Ask::GamePage(_) if shown != Shown::Game => return Some((Shown::Page, None)),
            _ => {}
        }
    }
    step_in_look_at(eclipse, themed, shown, at, ask)
}

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("emusen-qt-models/src/library_model.cxxqt.h");
        type LibraryModel = crate::library_model::qobject::LibraryModel;
        include!("emusen-qt-models/src/settings_model.cxxqt.h");
        type SettingsModel = crate::settings_model::qobject::SettingsModel;
    }

    /// The program's screens.
    #[qenum(Screens)]
    enum Screen {
        Home,
        Consoles,
        Grid,
        Game,
        Pause,
        HomeCard,
        List,
        Search,
        SearchGrid,
        Esde,
        Themes,
        Settings,
        Cores,
        Page,
        Tool,
        Collections,
        Tools,
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(Screen, shown, READ, NOTIFY)]
        #[qproperty(QString, key, READ, NOTIFY)]
        #[qproperty(bool, game_running, cxx_name = "gameRunning")]
        /// The page the Home card opens at, by its key.
        #[qproperty(QString, page, READ, NOTIFY)]
        /// Whether the grid shows a game, so the keyboard's shoulders have covers to move to.
        #[qproperty(bool, found)]
        /// Whether an ES-DE theme is the library's look (LibraryModel.themed): its view stands for the library's screens.
        #[qproperty(bool, themed)]
        /// Whether Big Picture face is active; false allows Desktop launch and Game Page transitions from any Desktop screen.
        #[qproperty(bool, big_screen, cxx_name = "bigScreen")]
        /// Whether Eclipse chrome is active (Kit.Tokens.coronaRim > 0).
        #[qproperty(bool, eclipse)]
        /// The tool of jobs the tool screen shows, by its key.
        #[qproperty(QString, tool, READ, NOTIFY)]
        /// The game last played (LibraryModel.lastPlayed), whose page Settings' Cheats opens.
        #[qproperty(QString, last_played, cxx_name = "lastPlayed")]
        /// The game a door opened a page for, the game in hand's page where empty; and the part of the page it opened
        /// at (`cheats`), empty for the page's first.
        #[qproperty(QString, page_game, cxx_name = "pageGame", READ, NOTIFY)]
        #[qproperty(QString, page_at, cxx_name = "pageAt", READ, NOTIFY)]
        /// The console whose page Settings opens at, asked for from Choose a console; empty where Settings opens at its start.
        #[qproperty(QString, settings_at, cxx_name = "settingsAt", READ, NOTIFY)]
        /// Settings target opened from Tools (`"themes"` or `"cards"`), cleared when leaving Settings.
        #[qproperty(QString, settings_tool, cxx_name = "settingsTool", READ, NOTIFY)]
        /// The page the List card opens at, a section's (crate::sections), empty for its first.
        #[qproperty(QString, list_page, cxx_name = "listPage", READ, NOTIFY)]
        /// The key of the program's section the shown screen is (crate::sections), empty for a screen that is none.
        #[qproperty(QString, section, READ, NOTIFY)]
        /// The game shown on Home when opening its page without a specific grid path.
        #[qproperty(QString, in_hand, cxx_name = "inHand")]
        /// The running session's game path when opening Cheats from Pause Card.
        #[qproperty(QString, pause_game, cxx_name = "pauseGame")]
        /// The game path passed to the Cheats tool; empty opens the library overview.
        #[qproperty(QString, cheats_game, cxx_name = "cheatsGame", READ, NOTIFY)]
        /// Library model used to step adjacent game shelves on Grid screens.
        #[qproperty(*mut LibraryModel, library_model, cxx_name = "libraryModel")]
        /// Settings model targeted by Tools -> Themes / Memory cards navigation.
        #[qproperty(*mut SettingsModel, settings_model, cxx_name = "settingsModel")]
        type Screens = super::ScreensRust;
    }

    extern "RustQt" {
        /// One of Home's places, by its index.
        #[qinvokable]
        fn place(self: Pin<&mut Screens>, index: i32);
        /// A row chosen on Choose a console: its grid.
        #[qinvokable]
        fn open(self: Pin<&mut Screens>, key: &QString);
        /// B that the shown screen did not keep.
        #[qinvokable]
        fn back(self: Pin<&mut Screens>);
        /// Opens the Cheats tool from the active screen context without hardcoding an action string in QML.
        #[qinvokable]
        #[cxx_name = "openCheats"]
        fn open_cheats(self: Pin<&mut Screens>);
        /// A screen's declared action, by its key.
        #[qinvokable]
        fn act(self: Pin<&mut Screens>, key: &QString);
        /// A game chosen, or Continue: the game's screen.
        #[qinvokable]
        fn play(self: Pin<&mut Screens>);
        /// The suspended game shown again and resumed: a library's Continue, or Exit to Library's key over the library.
        #[qinvokable]
        fn resume(self: Pin<&mut Screens>);
        /// The menu chord over the game: the Pause Card.
        #[qinvokable]
        fn menu(self: Pin<&mut Screens>);
        /// The Pause Card's Back to Library: Home over the suspended game.
        #[qinvokable]
        fn library(self: Pin<&mut Screens>);
        /// A card closed by B on its first page or by Start.
        #[qinvokable]
        fn close(self: Pin<&mut Screens>);
        /// Start at the ES-DE theme's view: the Home card at its first page, as ES-DE's Start opens its menu.
        #[qinvokable]
        fn home(self: Pin<&mut Screens>);
        /// Select at the ES-DE theme's view, its options: the List card.
        #[qinvokable]
        fn list(self: Pin<&mut Screens>);
        /// The game failed to open or stopped on a failure: returns to the screen that launched it.
        #[qinvokable]
        fn failed(self: Pin<&mut Screens>);
        /// Marks the active game session as genuinely started after prompt resolution.
        #[qinvokable]
        #[cxx_name = "markPlaying"]
        fn mark_playing(self: Pin<&mut Screens>);
        /// Applies pending Tools -> Settings target (`themes` or `cards`) once Settings is ready.
        #[qinvokable]
        #[cxx_name = "applySettingsTool"]
        fn apply_settings_tool(self: Pin<&mut Screens>);
        /// Closes Tools-opened Settings when Back pops above its entry page.
        #[qinvokable]
        #[cxx_name = "checkToolsSettingsTrail"]
        fn check_tools_settings_trail(self: Pin<&mut Screens>);
        /// The search's focus taken by its keys or its covers.
        #[qinvokable]
        #[cxx_name = "keysFocused"]
        fn keys_focused(self: Pin<&mut Screens>, keys: bool);
        /// One of the program's sections chosen on a nav, by its key: its screen.
        #[qinvokable]
        #[cxx_name = "openSection"]
        fn open_section(self: Pin<&mut Screens>, key: &QString);
        /// Steps to the previous (-1) or next (+1) section from a nav-bearing screen.
        #[qinvokable]
        #[cxx_name = "stepSection"]
        fn step_section(self: Pin<&mut Screens>, delta: i32);
        /// Opens a game's page directly by its ROM path.
        #[qinvokable]
        #[cxx_name = "openGame"]
        fn open_game(self: Pin<&mut Screens>, path: &QString);
        /// Opens a Settings tool or game door triggered from Desktop Settings.
        #[qinvokable]
        #[cxx_name = "openDesktopDoor"]
        fn open_desktop_door(self: Pin<&mut Screens>, key: &QString);
        /// Quit EmuSen answered Yes: the program ends.
        #[qsignal]
        fn quit(self: Pin<&mut Screens>);
        /// The Home card's Continue or resume(): the suspended game resumes as its screen is shown.
        #[qsignal]
        fn resumed(self: Pin<&mut Screens>);
        /// Start Console's row chosen: the console starts with nothing in it, by its key.
        #[qsignal]
        #[cxx_name = "consoleStarted"]
        fn console_started(self: Pin<&mut Screens>, key: QString);
        /// Eclipse Power dialog row chosen: Sleep, Restart, Shut down, or Back to the desktop, by its key.
        #[qsignal]
        #[cxx_name = "powerAsked"]
        fn power_asked(self: Pin<&mut Screens>, key: QString);
    }

    impl cxx_qt::Threading for Screens {}
    impl cxx_qt::Initialize for Screens {}
}

use qobject::Screen;

pub struct ScreensRust {
    shown: Screen,
    key: QString,
    /// Whether the program's game runs: a game that starts is shown, and when it stops its screen gives way to Home.
    game_running: bool,
    page: QString,
    found: bool,
    themed: bool,
    big_screen: bool,
    eclipse: bool,
    tool: QString,
    last_played: QString,
    page_game: QString,
    page_at: QString,
    section: QString,
    /// The screen a game's page was opened over.
    page_origin: Shown,
    /// The screen Cores or the Themes tab was opened over.
    tool_origin: Option<Shown>,
    /// Settings opened over the Pause Card, whose B returns to it.
    over_card: bool,
    /// Settings opened from Tools, whose B returns to it.
    over_tools: bool,
    settings_at: QString,
    settings_tool: QString,
    tool_trail_depth: usize,
    list_page: QString,
    /// Settings opened at a console's page from Choose a console, whose B returns to it.
    over_consoles: bool,
    /// Grid opened from Collections, whose B returns to Collections.
    grid_origin: Option<Shown>,
    /// Screen Search was opened over.
    search_origin: Option<Shown>,
    /// Screen that launched the current game session, restored on cancel/failure.
    play_origin: Option<Shown>,
    /// Whether the active launch actually started playing beyond launch/resume prompts.
    play_started: bool,
    in_hand: QString,
    pause_game: QString,
    cheats_game: QString,
    library_model: *mut qobject::LibraryModel,
    settings_model: *mut qobject::SettingsModel,
}

impl Default for ScreensRust {
    fn default() -> Self {
        ScreensRust {
            shown: Screen::Home,
            key: QString::default(),
            game_running: false,
            page: QString::default(),
            found: false,
            themed: false,
            big_screen: true,
            eclipse: false,
            tool: QString::default(),
            last_played: QString::default(),
            page_game: QString::default(),
            page_at: QString::default(),
            section: QString::from(SECTIONS[0].key),
            page_origin: Shown::Home,
            tool_origin: None,
            over_card: false,
            over_tools: false,
            settings_at: QString::default(),
            settings_tool: QString::default(),
            tool_trail_depth: 0,
            list_page: QString::default(),
            over_consoles: false,
            grid_origin: None,
            search_origin: None,
            play_origin: None,
            play_started: false,
            in_hand: QString::default(),
            pause_game: QString::default(),
            cheats_game: QString::default(),
            library_model: std::ptr::null_mut(),
            settings_model: std::ptr::null_mut(),
        }
    }
}

impl cxx_qt::Initialize for qobject::Screens {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut()
            .on_game_running_changed(|screens| {
                let ask = if screens.game_running { Ask::Play } else { Ask::Closed };
                screens.ask(ask);
            })
            .release();
        self.on_themed_changed(|screens| screens.ask(Ask::Restyled)).release();
    }
}

fn shown_of(screen: Screen) -> Shown {
    match screen {
        Screen::Consoles => Shown::Consoles,
        Screen::Grid => Shown::Grid,
        Screen::Game => Shown::Game,
        Screen::Pause => Shown::Pause,
        Screen::HomeCard => Shown::HomeCard,
        Screen::List => Shown::List,
        Screen::Search => Shown::Search,
        Screen::SearchGrid => Shown::SearchGrid,
        Screen::Esde => Shown::Esde,
        Screen::Themes => Shown::Themes,
        Screen::Settings => Shown::Settings,
        Screen::Cores => Shown::Cores,
        Screen::Page => Shown::Page,
        Screen::Tool => Shown::Tool,
        Screen::Collections => Shown::Collections,
        Screen::Tools => Shown::Tools,
        _ => Shown::Home,
    }
}

fn screen_of(shown: Shown) -> Screen {
    match shown {
        Shown::Home => Screen::Home,
        Shown::Consoles => Screen::Consoles,
        Shown::Grid => Screen::Grid,
        Shown::Game => Screen::Game,
        Shown::Pause => Screen::Pause,
        Shown::HomeCard => Screen::HomeCard,
        Shown::List => Screen::List,
        Shown::Search => Screen::Search,
        Shown::SearchGrid => Screen::SearchGrid,
        Shown::Esde => Screen::Esde,
        Shown::Themes => Screen::Themes,
        Shown::Settings => Screen::Settings,
        Shown::Cores => Screen::Cores,
        Shown::Page => Screen::Page,
        Shown::Tool => Screen::Tool,
        Shown::Collections => Screen::Collections,
        Shown::Tools => Screen::Tools,
    }
}

impl qobject::Screens {
    pub fn place(self: Pin<&mut Self>, index: i32) {
        self.ask(Ask::Place(index.max(0) as usize));
    }

    pub fn open(self: Pin<&mut Self>, key: &QString) {
        self.ask(Ask::Open(key.to_string()));
    }

    pub fn back(self: Pin<&mut Self>) {
        self.ask(Ask::Back);
    }

    pub fn open_cheats(self: Pin<&mut Self>) {
        self.ask(Ask::Act(crate::home_card::CHEATS.to_owned()));
    }

    pub fn act(mut self: Pin<&mut Self>, key: &QString) {
        let key = key.to_string();
        if key == crate::home_card::QUIT_EMUSEN {
            self.as_mut().quit();
        }
        if key == crate::home_card::POWER_DESKTOP || crate::power::Action::from_key(&key).is_some() {
            self.as_mut().power_asked(QString::from(key.as_str()));
        }
        if key == crate::home_card::RESUME && self.shown == Screen::HomeCard {
            if !self.game_running {
                return self.ask(Ask::Close);
            }
            self.as_mut().resumed();
        }
        if let Some(console) = crate::home_card::console_to_start(&key).map(QString::from) {
            // The game's screen asked for first, as a game's launch does, so a start that fails at once is answered after it.
            self.as_mut().ask(Ask::Act(key));
            return self.console_started(console);
        }
        if key == TO_GRID && !self.found {
            return;
        }
        self.ask(Ask::Act(key));
    }

    pub fn play(self: Pin<&mut Self>) {
        self.ask(Ask::Play);
    }

    pub fn resume(mut self: Pin<&mut Self>) {
        self.as_mut().resumed();
        self.ask(Ask::ResumePlay);
    }

    pub fn menu(self: Pin<&mut Self>) {
        self.ask(Ask::Menu);
    }

    pub fn library(self: Pin<&mut Self>) {
        self.ask(Ask::Library);
    }

    pub fn close(self: Pin<&mut Self>) {
        self.ask(Ask::Close);
    }

    pub fn home(self: Pin<&mut Self>) {
        self.ask(Ask::Act(HOME.to_owned()));
    }

    pub fn list(self: Pin<&mut Self>) {
        self.ask(Ask::Act(LIST.to_owned()));
    }

    pub fn failed(self: Pin<&mut Self>) {
        self.ask(Ask::Failed);
    }

    pub fn mark_playing(self: Pin<&mut Self>) {
        self.ask(Ask::Started);
    }

    pub fn apply_settings_tool(mut self: Pin<&mut Self>) {
        if self.tool_trail_depth > 0 || self.shown != Screen::Settings {
            return;
        }
        let tool = self.settings_tool.to_string();
        if tool.is_empty() {
            return;
        }
        // Safety: settings_model is the window's SettingsModel when wired in Main.qml.
        let Some(mut settings) = unsafe { self.settings_model.as_mut() }.map(|p| unsafe { Pin::new_unchecked(p) }) else { return };
        if let Some(depth) = crate::tools_hub::open_tools_settings(&tool, settings.as_mut()) {
            self.as_mut().rust_mut().tool_trail_depth = depth;
        }
    }

    pub fn check_tools_settings_trail(mut self: Pin<&mut Self>) {
        let depth = self.tool_trail_depth;
        if depth == 0 || self.settings_tool.is_empty() || self.shown != Screen::Settings {
            return;
        }
        // Safety: settings_model is the window's SettingsModel when wired in Main.qml.
        let Some(mut settings) = unsafe { self.settings_model.as_mut() }.map(|p| unsafe { Pin::new_unchecked(p) }) else { return };
        if settings.view().trail().len() < depth {
            self.as_mut().rust_mut().tool_trail_depth = 0;
            settings.as_mut().set_section(0);
            self.as_mut().back();
        }
    }

    pub fn keys_focused(self: Pin<&mut Self>, keys: bool) {
        self.ask(Ask::Keys(keys));
    }

    pub fn open_section(self: Pin<&mut Self>, key: &QString) {
        self.ask(Ask::Section(key.to_string()));
    }

    pub fn step_section(self: Pin<&mut Self>, delta: i32) {
        let key = self.key.to_string();
        let Some(current) = eclipse_section_at(shown_of(self.shown), &key) else { return };
        let Some(at) = SECTIONS.iter().position(|s| s.key == current) else { return };
        let next = (at as isize + delta as isize).rem_euclid(SECTIONS.len() as isize) as usize;
        self.ask(Ask::Section(SECTIONS[next].key.to_owned()));
    }

    fn shelf_step(&self, current: &str, delta: i32) -> Option<String> {
        // Safety: library_model is the window's LibraryModel, alive as long as Screens.
        let library = unsafe { self.library_model.as_ref() }?;
        neighbour_shelf(library.view().rows.rows(), current, delta)
    }

    pub fn open_game(self: Pin<&mut Self>, path: &QString) {
        let p = path.to_string();
        if !p.is_empty() {
            self.ask(Ask::GamePage(p));
        }
    }

    pub fn open_desktop_door(self: Pin<&mut Self>, key: &QString) {
        let k = key.to_string();
        if !k.is_empty() {
            self.ask(Ask::DesktopDoor(k));
        }
    }

    fn ask(self: Pin<&mut Self>, ask: Ask) {
        let _ = self.qt_thread().queue(move |screens| screens.apply(ask));
    }

    fn apply(mut self: Pin<&mut Self>, ask: Ask) {
        if ask == Ask::Started {
            self.as_mut().rust_mut().play_started = true;
            return;
        }
        let was = shown_of(self.shown);
        let current_key = self.key.to_string();
        let step_ask = if ask == Ask::ResumePlay { Ask::Play } else { ask.clone() };
        let stepped = match was {
            Shown::Page => page_step(self.themed, self.page_origin, &step_ask).map(|s| {
                let key = match &step_ask {
                    Ask::Open(key) => Some(key.clone()),
                    Ask::Act(key) if s == Shown::Tool => Some(key.clone()),
                    _ => None,
                };
                (s, key)
            }),
            Shown::Settings if self.over_card && step_ask == Ask::Back => Some((Shown::Pause, None)),
            Shown::Settings if self.over_tools && step_ask == Ask::Back => Some((Shown::Tools, None)),
            Shown::Settings if self.over_consoles && step_ask == Ask::Back => Some((Shown::Consoles, None)),
            Shown::Grid if step_ask == Ask::Back => self
                .grid_origin
                .map(|origin| (origin, None))
                .or_else(|| step_for_face_at(self.big_screen, self.eclipse, self.themed, was, &current_key, &step_ask)),
            Shown::Search | Shown::SearchGrid if self.eclipse && self.search_origin == Some(Shown::Home) && step_ask == Ask::Back => Some((Shown::Home, None)),
            Shown::Game | Shown::Pause if close_origin(self.themed, self.play_origin, self.play_started, &step_ask).is_some() => {
                close_origin(self.themed, self.play_origin, self.play_started, &step_ask).map(|s| (s, None))
            }
            _ => tool_back(self.themed, self.tool_origin, was, &step_ask)
                .map(|s| (s, None))
                .or_else(|| step_for_face_at(self.big_screen, self.eclipse, self.themed, was, &current_key, &step_ask))
                .or_else(|| match (was, &step_ask) {
                    (Shown::Grid, Ask::Act(k)) if k == PREVIOUS => self.shelf_step(&current_key, -1).map(|s| (Shown::Grid, Some(s))),
                    (Shown::Grid, Ask::Act(k)) if k == NEXT => self.shelf_step(&current_key, 1).map(|s| (Shown::Grid, Some(s))),
                    _ => None,
                }),
        };
        let Some((shown, key)) = stepped else { return };
        if shown == Shown::Game && !matches!(was, Shown::Game | Shown::Pause | Shown::Settings | Shown::Tool) {
            let (started, origin) = record_play_origin(self.themed, was, &ask);
            self.as_mut().rust_mut().play_started = started;
            self.as_mut().rust_mut().play_origin = origin;
        } else if !matches!(shown, Shown::Game | Shown::Pause) && !matches!((was, shown), (Shown::Pause, Shown::Settings | Shown::Tool)) {
            self.as_mut().rust_mut().play_origin = None;
            self.as_mut().rust_mut().play_started = false;
        }
        if matches!(shown, Shown::Cores | Shown::Themes | Shown::Tool) && shown != was {
            self.as_mut().rust_mut().tool_origin = Some(was);
        }
        let over_tools = (was == Shown::Tools && shown == Shown::Settings) || (self.over_tools && shown == Shown::Settings);
        self.as_mut().rust_mut().over_tools = over_tools;
        let existing_tool = self.settings_tool.to_string();
        let next_tool = if shown == Shown::Settings {
            if was == Shown::Tools {
                match &ask {
                    Ask::Act(k) if k == THEMES => THEMES,
                    Ask::Act(k) if k == "cards" => "cards",
                    _ => "",
                }
                .to_owned()
            } else if was == Shown::Settings {
                existing_tool
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        let next_tool_q = QString::from(next_tool.as_str());
        let tool_changed = self.settings_tool != next_tool_q;
        if tool_changed || next_tool.is_empty() {
            self.as_mut().rust_mut().tool_trail_depth = 0;
        }
        if tool_changed {
            self.as_mut().rust_mut().settings_tool = next_tool_q;
        }
        self.as_mut().rust_mut().grid_origin = next_grid_origin(was, shown, &ask, self.grid_origin);
        let search_origin = if shown == Shown::Search && !matches!(was, Shown::Search | Shown::SearchGrid | Shown::Page) {
            Some(was)
        } else if matches!(shown, Shown::Search | Shown::SearchGrid | Shown::Page) {
            self.search_origin
        } else {
            None
        };
        self.as_mut().rust_mut().search_origin = search_origin;
        if shown != was && (shown == Shown::Settings || was == Shown::Settings) {
            let at = match (&ask, shown) {
                (Ask::Act(key), Shown::Settings) => console_options(key).unwrap_or_default(),
                _ => "",
            };
            self.as_mut().rust_mut().over_consoles = was == Shown::Consoles && !at.is_empty();
            let at = QString::from(at);
            if self.settings_at != at {
                self.as_mut().rust_mut().settings_at = at;
                self.as_mut().settings_at_changed();
            }
        }
        if shown == Shown::List && was != Shown::List {
            let page = QString::from(match &ask {
                Ask::Section(key) => crate::sections::by_key(key).and_then(|s| s.page).unwrap_or_default(),
                _ => "",
            });
            if self.list_page != page {
                self.as_mut().rust_mut().list_page = page;
                self.as_mut().list_page_changed();
            }
        }
        let over_card = was == Shown::Pause && shown == Shown::Settings;
        if self.over_card != over_card && (over_card || shown != Shown::Settings) {
            self.as_mut().rust_mut().over_card = over_card;
        }
        if shown == Shown::Page && was != Shown::Page {
            if was == Shown::Tool {
                let at = QString::from(if self.tool.to_string() == crate::home_card::CHEATS { CHEATS } else { "" });
                if self.page_at != at {
                    self.as_mut().rust_mut().page_at = at;
                    self.as_mut().page_at_changed();
                }
            } else {
                self.as_mut().rust_mut().page_origin = if !self.big_screen { library_of(self.themed) } else { page_origin(self.themed, was) };
                let (at, game) = match &ask {
                    Ask::GamePage(path) => (QString::default(), QString::from(path.as_str())),
                    _ => {
                        let at = QString::from(key.as_deref().unwrap_or_default());
                        let game = if key.is_some() { self.last_played.clone() } else { QString::default() };
                        (at, game)
                    }
                };
                if self.page_at != at {
                    self.as_mut().rust_mut().page_at = at;
                    self.as_mut().page_at_changed();
                }
                if self.page_game != game {
                    self.as_mut().rust_mut().page_game = game;
                    self.as_mut().page_game_changed();
                }
            }
        }
        let section_key = if shown == Shown::Grid { key.clone().unwrap_or_else(|| current_key.clone()) } else { String::new() };
        if let (Shown::HomeCard, Some(page)) = (shown, &key) {
            let page = QString::from(page.as_str());
            if self.page != page {
                self.as_mut().rust_mut().page = page;
                self.as_mut().page_changed();
            }
        } else if let (Shown::Tool, Some(tool)) = (shown, &key) {
            if tool == crate::home_card::CHEATS && was != Shown::Tool {
                let target = match was {
                    Shown::Page => if !self.page_game.is_empty() { self.page_game.clone() } else { self.in_hand.clone() },
                    Shown::Pause => self.pause_game.clone(),
                    _ => QString::default(),
                };
                if self.cheats_game != target {
                    self.as_mut().rust_mut().cheats_game = target;
                    self.as_mut().cheats_game_changed();
                }
            }
            let tool = QString::from(tool.as_str());
            if self.tool != tool {
                self.as_mut().rust_mut().tool = tool;
                self.as_mut().tool_changed();
            }
        } else if let Some(key) = key.filter(|_| shown != Shown::Page) {
            let key = QString::from(key.as_str());
            if self.key != key {
                self.as_mut().rust_mut().key = key;
                self.as_mut().key_changed();
            }
        }
        if shown != Shown::Tool && was == Shown::Tool && !self.cheats_game.is_empty() {
            self.as_mut().rust_mut().cheats_game = QString::default();
            self.as_mut().cheats_game_changed();
        }
        let section = QString::from(eclipse_section_at(shown, &section_key).unwrap_or_else(|| crate::sections::current_at(shown, &section_key).map_or("", |s| s.key)));
        if self.section != section {
            self.as_mut().rust_mut().section = section;
            self.as_mut().section_changed();
        }
        let screen = screen_of(shown);
        if self.shown != screen {
            self.as_mut().rust_mut().shown = screen;
            self.as_mut().shown_changed();
        }
        if tool_changed {
            self.as_mut().settings_tool_changed();
        }
        if !next_tool.is_empty() {
            self.as_mut().apply_settings_tool();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_face_allows_play_and_game_page_from_any_non_running_screen() {
        for screen in [
            Shown::Home,
            Shown::Consoles,
            Shown::Grid,
            Shown::Pause,
            Shown::Settings,
            Shown::Tools,
            Shown::Tool,
            Shown::Cores,
            Shown::Themes,
            Shown::Esde,
        ] {
            assert_eq!(step_for_face(false, true, false, screen, &Ask::Play), Some((Shown::Game, None)));
            assert_eq!(
                step_for_face(false, true, false, screen, &Ask::GamePage("/rom/Game.sfc".to_owned())),
                Some((Shown::Page, None))
            );
        }
        assert_eq!(step_for_face(false, true, false, Shown::Game, &Ask::GamePage("/rom/Game.sfc".to_owned())), None);
        assert_eq!(step_for_face(true, true, false, Shown::Consoles, &Ask::Play), None);
        assert_eq!(step_for_face(true, true, false, Shown::Settings, &Ask::GamePage("/rom/Game.sfc".to_owned())), None);
    }

    #[test]
    fn desktop_game_page_returns_to_library_of_look() {
        assert_eq!(page_origin(false, Shown::HomeCard), Shown::Home);
        assert_eq!(library_of(false), Shown::Home);
        assert_eq!(library_of(true), Shown::Esde);
    }

    #[test]
    fn eclipse_top_level_library_steps_between_home_and_consoles() {
        assert_eq!(eclipse_section_at(Shown::Grid, crate::library::ALL), Some("library"));
        assert_eq!(eclipse_section_at(Shown::Grid, "snes"), None);
        assert_eq!(step_in_look_at(true, false, Shown::Grid, crate::library::ALL, &Ask::Act(PREVIOUS.into())), Some((Shown::Home, None)));
        assert_eq!(step_in_look_at(true, false, Shown::Grid, crate::library::ALL, &Ask::Act(NEXT.into())), Some((Shown::Consoles, None)));
        assert_eq!(step_in_look_at(true, false, Shown::Grid, crate::library::ALL, &Ask::Section("tools".into())), Some((Shown::Tools, None)));
        assert_eq!(step_in_look_at(true, false, Shown::Grid, crate::library::ALL, &Ask::Section("library".into())), None);
        assert_eq!(step_in_look_at(true, false, Shown::Grid, "snes", &Ask::Act(PREVIOUS.into())), None);
    }

    #[test]
    fn eclipse_tools_route_themes_and_cards_to_settings_and_return_to_tools() {
        assert_eq!(step_in_look_at(true, false, Shown::Tools, "", &Ask::Act(THEMES.into())), Some((Shown::Settings, None)));
        assert_eq!(step_in_look_at(true, false, Shown::Tools, "", &Ask::Act("cards".into())), Some((Shown::Settings, None)));
        assert_eq!(tool_back(false, Some(Shown::Tools), Shown::Tool, &Ask::Back), Some(Shown::Tools));
        assert_eq!(tool_back(false, Some(Shown::Tools), Shown::Cores, &Ask::Back), Some(Shown::Tools));
    }

    #[test]
    fn play_origin_restores_launch_screen_until_playback_starts() {
        for origin in [Shown::Grid, Shown::Page, Shown::SearchGrid, Shown::Consoles] {
            let (started, saved) = record_play_origin(false, origin, &Ask::Play);
            assert!(!started);
            assert_eq!(saved, Some(origin));
            assert_eq!(close_origin(false, saved, false, &Ask::Failed), Some(origin));
            assert_eq!(close_origin(false, saved, false, &Ask::Closed), Some(origin));
            assert_eq!(close_origin(false, saved, true, &Ask::Closed), None);
        }
        assert_eq!(record_play_origin(false, Shown::List, &Ask::Act("console:snes".into())), (false, Some(Shown::Home)));
        assert_eq!(record_play_origin(true, Shown::HomeCard, &Ask::Act("console:snes".into())), (false, Some(Shown::Esde)));
        assert_eq!(record_play_origin(false, Shown::Home, &Ask::ResumePlay), (true, None));
    }

    #[test]
    fn library_back_origin_tracks_home_consoles_and_collections() {
        assert_eq!(grid_origin_for(Shown::Home, &Ask::Section("library".into())), Shown::Home);
        assert_eq!(grid_origin_for(Shown::Consoles, &Ask::Section("library".into())), Shown::Home);
        assert_eq!(grid_origin_for(Shown::Consoles, &Ask::Open("snes".into())), Shown::Consoles);
        assert_eq!(grid_origin_for(Shown::Collections, &Ask::Open("favs".into())), Shown::Collections);
        assert_eq!(grid_origin_for(Shown::Tools, &Ask::Section("library".into())), Shown::Home);

        // Preserve grid_origin across launch failure/cancel and clean return from Page, clearing once settled or exited.
        let origin = next_grid_origin(Shown::Collections, Shown::Grid, &Ask::Open("favs".into()), None);
        assert_eq!(origin, Some(Shown::Collections));
        let in_game = next_grid_origin(Shown::Grid, Shown::Game, &Ask::Play, origin);
        assert_eq!(in_game, Some(Shown::Collections));
        let after_fail = next_grid_origin(Shown::Game, Shown::Grid, &Ask::Failed, in_game);
        assert_eq!(after_fail, Some(Shown::Collections));
        assert_eq!(next_grid_origin(Shown::Grid, Shown::Page, &Ask::GamePage("g".into()), origin), Some(Shown::Collections));
        assert_eq!(next_grid_origin(Shown::Page, Shown::Grid, &Ask::Back, origin), Some(Shown::Collections));
        assert_eq!(next_grid_origin(Shown::Page, Shown::Grid, &Ask::Open("snes".into()), origin), Some(Shown::Consoles));
        assert_eq!(next_grid_origin(Shown::Game, Shown::Home, &Ask::Closed, in_game), None);
    }
}
