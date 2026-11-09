//! The help bar's entries for each screen, as the C# screens and the mockups name them (EmuSen_Interface.md §24.1-§24.3;
//! Home's by the row focused, as `HomeScreen.HintsFor` gives them):
//! the button by what it asks of the interface, its label, and the action it invokes where the walk does not answer it
//! itself. The A/B swap is the router's job, not this list's.

use emusen_kit_core::input::UiButton;

/// A screen with a help bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    /// Home with Continue focused.
    Home,
    /// Home with a game of the shelf focused.
    HomeShelf,
    /// Home with a place focused.
    HomePlaces,
    Consoles,
    Grid,
    /// The grid with words searched for: B clears them (`GamesScreen.HintsFor` with `searched`).
    GridSearched,
    /// The Pause Card's first page.
    Pause,
    /// A page of the Pause Card past the first.
    PausePage,
    /// A card over the library: the Home card and the List card (Mistress's `ShowCard` away from a game).
    Card,
    /// The grid's search with the keyboard docked (`GamesScreen.HintsFor` in the keyboard).
    Search,
    /// The Pause Card's first page on Save State or Load State: Left and Right change the slot.
    PauseSlots,
    /// The Pause Card's reel (EmuSen_Interface.md §20.1): A goes back, Left and Right step the moment, Start is not offered.
    PauseReel,
    /// A question whose B answers No (EmuSen_Interface.md §15.6).
    Question,
    /// A question whose B goes back, the Resume question's (§18.3).
    QuestionBack,
    /// The Pause Card's Change Disc (EmuSen_Interface.md §20.2): A puts the disc in, Start is not offered.
    PauseDisc,
    /// The Themes tab on a theme Use is offered for (EmuSen_Qt.md §3.8).
    Themes,
    /// The Themes tab on a theme in use, not checked clean, or of a kind not chosen here: Check alone.
    ThemesChecked,
    /// Settings on a section (EmuSen_BigPicture.md §52): the shoulders step the sections.
    Settings,
    /// Settings on a page under a section: the shoulders are the page's.
    SettingsPage,
    /// A tool's search with the keyboard docked: Type, Delete, Space and Done (EmuSen_Interface.md §15.2).
    ToolSearch,
}

/// One entry: Accept and Back are the walk's own, so they name no action.
pub type Entry = (UiButton, &'static str, Option<&'static str>);

/// The entries, in the bar's order.
pub fn entries(screen: Screen) -> &'static [Entry] {
    match screen {
        Screen::Home => &[
            (UiButton::Accept, "Resume", None),
            (UiButton::Screensaver, "Game page", Some(GAME_PAGE)),
            (UiButton::Search, "Favourite", Some(FAVOURITE)),
            (UiButton::Options, "Search", Some("search")),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        Screen::HomeShelf => &[
            (UiButton::Accept, "Play", None),
            (UiButton::Screensaver, "Game page", Some(GAME_PAGE)),
            (UiButton::Search, "Favourite", Some(FAVOURITE)),
            (UiButton::Options, "Search", Some("search")),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        Screen::HomePlaces => &[
            (UiButton::Accept, "Open", None),
            (UiButton::Options, "Search", Some("search")),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        Screen::Consoles => &[
            (UiButton::Accept, "Open", None),
            (UiButton::Back, "Back", None),
            (UiButton::Search, "Favourites", Some(crate::library::FAVOURITES)),
            (UiButton::Screensaver, "Random game", Some(RANDOM)),
        ],
        Screen::Grid => &[
            (UiButton::Accept, "Play", None),
            (UiButton::Screensaver, "Game page", Some(GAME_PAGE)),
            (UiButton::Search, "Favourite", Some(FAVOURITE)),
            (UiButton::PageUp, "Console", Some(PREVIOUS)),
            (UiButton::Options, "List", Some("list")),
            (UiButton::Back, "Consoles", None),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        Screen::GridSearched => &[
            (UiButton::Accept, "Play", None),
            (UiButton::Screensaver, "Game page", Some(GAME_PAGE)),
            (UiButton::Search, "Favourite", Some(FAVOURITE)),
            (UiButton::PageUp, "Console", Some(PREVIOUS)),
            (UiButton::Options, "List", Some("list")),
            (UiButton::Back, "Clear search", None),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        // Mistress's ShowCard and SetCardActions: B resumes on the first page and goes back on the others; Start resumes.
        Screen::Pause => &[
            (UiButton::Accept, "Select", None),
            (UiButton::Back, "Resume", None),
            (UiButton::Screensaver, "Screenshot", Some(crate::pause::SCREENSHOT)),
            (UiButton::Menu, "Resume", Some(crate::pause::RESUME)),
        ],
        Screen::PausePage => &[
            (UiButton::Accept, "Choose", None),
            (UiButton::Back, "Back", None),
            (UiButton::Screensaver, "Screenshot", Some(crate::pause::SCREENSHOT)),
            (UiButton::Menu, "Resume", Some(crate::pause::RESUME)),
        ],
        Screen::Question => &[(UiButton::Accept, "Choose", None), (UiButton::Back, "No", None)],
        Screen::QuestionBack => &[(UiButton::Accept, "Choose", None), (UiButton::Back, "Back", None)],
        Screen::Card => &[(UiButton::Accept, "Choose", None), (UiButton::Back, "Back", None), (UiButton::Menu, "Close", Some(CLOSE))],
        Screen::Search => &[
            (UiButton::Accept, "Type", None),
            (UiButton::Back, "Delete", None),
            (UiButton::Search, "Space", Some("space")),
            (UiButton::Screensaver, "Done", Some("done")),
            (UiButton::PageUp, "Move to grid", Some(crate::screens::TO_GRID)),
            (UiButton::Menu, "Home", Some("home")),
        ],
        Screen::PauseSlots => &[
            (UiButton::Accept, "Select", None),
            (UiButton::Back, "Resume", None),
            (UiButton::Left, "Change slot", None),
            (UiButton::Screensaver, "Screenshot", Some(crate::pause::SCREENSHOT)),
            (UiButton::Menu, "Resume", Some(crate::pause::RESUME)),
        ],
        Screen::PauseReel => &[
            (UiButton::Accept, "Go back", None),
            (UiButton::Back, "Back", None),
            (UiButton::Left, "Moment", None),
            (UiButton::Screensaver, "Screenshot", Some(crate::pause::SCREENSHOT)),
        ],
        Screen::PauseDisc => &[
            (UiButton::Accept, "Put in", None),
            (UiButton::Back, "Back", None),
            (UiButton::Screensaver, "Screenshot", Some(crate::pause::SCREENSHOT)),
        ],
        Screen::Themes => &[
            (UiButton::Accept, "Use", None),
            (UiButton::Screensaver, "Check", Some(CHECK)),
            (UiButton::PageUp, "Kind", None),
            (UiButton::Back, "Back", None),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        Screen::ThemesChecked => &[
            (UiButton::Screensaver, "Check", Some(CHECK)),
            (UiButton::PageUp, "Kind", None),
            (UiButton::Back, "Back", None),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        // Mistress's SettingsView hints: Change, Value, Section, Back, Home.
        Screen::Settings => &[
            (UiButton::Accept, "Change", None),
            (UiButton::Left, "Value", None),
            (UiButton::PageUp, "Section", None),
            (UiButton::Back, "Back", None),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        Screen::SettingsPage => &[
            (UiButton::Accept, "Change", None),
            (UiButton::Left, "Value", None),
            (UiButton::Back, "Back", None),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        Screen::ToolSearch => &[
            (UiButton::Accept, "Type", None),
            (UiButton::Back, "Delete", None),
            (UiButton::Search, "Space", Some("space")),
            (UiButton::Screensaver, "Done", Some("done")),
        ],
    }
}

/// The Themes tab's check of the focused theme.
pub const CHECK: &str = "check";

/// The shoulders' entry where a screen's shoulders jump a group of its own row, and B's where a screen has none (the
/// themes' help bars, EmuSen_Qt.md §3.2): no action, since the walk and the theme answer them.
pub const GROUP: Entry = (UiButton::PageUp, "Group", None);
pub const BACK: Entry = (UiButton::Back, "Back", None);

/// Where the group entry goes in a screen's bar, after the entries that act on the focused item; none where the shoulders
/// already mean something (the grid's Console, the search's Move to grid) or the screen has no row of groups.
pub fn group_at(screen: Screen) -> Option<usize> {
    match screen {
        Screen::Home | Screen::HomeShelf => Some(3),
        Screen::HomePlaces => Some(1),
        Screen::Consoles => Some(2),
        _ => None,
    }
}

/// Where B Back goes in a screen's bar that has no B: after the entries that act on the focused item.
pub fn back_at(screen: Screen) -> Option<usize> {
    match screen {
        Screen::Home | Screen::HomeShelf => Some(3),
        Screen::HomePlaces => Some(1),
        _ => None,
    }
}

/// A screen's entries with the group's and B's added where the screen takes them, B before the group.
pub fn entries_with(screen: Screen, group: bool, back: bool) -> Vec<Entry> {
    let mut list = entries(screen).to_vec();
    let has_back = list.iter().any(|e| e.0 == UiButton::Back);
    let back_place = back_at(screen).filter(|_| back && !has_back);
    if let Some(at) = back_place {
        list.insert(at, BACK);
    }
    if let Some(at) = group_at(screen).filter(|_| group) {
        list.insert(at + usize::from(back_place.is_some_and(|b| b <= at)), GROUP);
    }
    list
}

/// Start's action away from a game and a card: Home (EmuSen_BigPicture.md §51.2, §51.3).
pub const HOME: &str = "home";

/// The focused game's page, its heart, and a random game: actions a later stage carries out.
pub const GAME_PAGE: &str = "gamePage";
pub const FAVOURITE: &str = "favourite";
pub const RANDOM: &str = "random";

/// The shoulders on the grid: the console before and the console after (Mistress's `StepSection`).
pub const PREVIOUS: &str = "previous";
pub const NEXT: &str = "next";

/// A card's declared action that closes it, as Start closes a card away from a game.
pub const CLOSE: &str = "close";

/// The second button an entry names: the shoulders are one entry, LB and RB, as the accepted render draws them, and
/// the d-pad's Left and Right one entry.
pub fn pair_of(button: UiButton) -> Option<UiButton> {
    match button {
        UiButton::PageUp => Some(UiButton::PageDown),
        UiButton::Left => Some(UiButton::Right),
        _ => None,
    }
}

/// The screen's actions as the frame declares them: `key Button`, one a line (EmuSen_Kit.md §7.19).
pub fn action_lines(screen: Screen) -> String {
    entries(screen).iter().chain(unnamed(screen)).filter_map(|(button, _, action)| action.map(|key| format!("{key} {button:?}\n"))).collect()
}

/// A screen's actions its help bar does not name: Start on Choose a console goes Home, as Mistress's
/// `ConsolesPadCommand` has it, though the mockup's bar shows four entries; the right shoulder of a pair the bar draws
/// as one entry (the grid's Console, the search's Move to grid).
pub fn unnamed(screen: Screen) -> &'static [Entry] {
    match screen {
        Screen::Consoles => &[(UiButton::Menu, "Home", Some(HOME))],
        Screen::Grid | Screen::GridSearched => &[(UiButton::PageDown, "Console", Some(NEXT))],
        Screen::Search => &[(UiButton::PageDown, "Move to grid", Some(crate::screens::TO_GRID))],
        _ => &[],
    }
}
