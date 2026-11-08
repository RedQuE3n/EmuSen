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
}

/// One entry: Accept and Back are the walk's own, so they name no action.
pub type Entry = (UiButton, &'static str, Option<&'static str>);

/// The entries, in the bar's order.
pub fn entries(screen: Screen) -> &'static [Entry] {
    match screen {
        Screen::Home => &[
            (UiButton::Accept, "Resume", None),
            (UiButton::Screensaver, "Game page", Some("gamePage")),
            (UiButton::Search, "Favourite", Some("favourite")),
            (UiButton::Options, "Search", Some("search")),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        Screen::HomeShelf => &[
            (UiButton::Accept, "Play", None),
            (UiButton::Screensaver, "Game page", Some("gamePage")),
            (UiButton::Search, "Favourite", Some("favourite")),
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
            (UiButton::Screensaver, "Random game", Some("random")),
        ],
        Screen::Grid => &[
            (UiButton::Accept, "Play", None),
            (UiButton::Screensaver, "Game page", Some("gamePage")),
            (UiButton::Search, "Favourite", Some("favourite")),
            (UiButton::PageUp, "Console", Some("previous")),
            (UiButton::Options, "List", Some("list")),
            (UiButton::Back, "Consoles", None),
            (UiButton::Menu, "Home", Some(HOME)),
        ],
        // Mistress's ShowCard and SetCardActions: B resumes on the first page and goes back on the others; Start resumes.
        Screen::Pause => &[
            (UiButton::Accept, "Select", None),
            (UiButton::Back, "Resume", None),
            (UiButton::Screensaver, "Screenshot", Some("screenshot")),
            (UiButton::Menu, "Resume", Some(crate::pause::RESUME)),
        ],
        Screen::PausePage => &[
            (UiButton::Accept, "Choose", None),
            (UiButton::Back, "Back", None),
            (UiButton::Screensaver, "Screenshot", Some("screenshot")),
            (UiButton::Menu, "Resume", Some(crate::pause::RESUME)),
        ],
        Screen::Card => &[(UiButton::Accept, "Choose", None), (UiButton::Back, "Back", None), (UiButton::Menu, "Close", Some(CLOSE))],
        Screen::Search => &[
            (UiButton::Accept, "Type", None),
            (UiButton::Back, "Delete", None),
            (UiButton::Search, "Space", Some("space")),
            (UiButton::Screensaver, "Done", Some("done")),
            (UiButton::PageUp, "Move to grid", Some("toGrid")),
            (UiButton::Menu, "Home", Some("home")),
        ],
        Screen::PauseSlots => &[
            (UiButton::Accept, "Select", None),
            (UiButton::Back, "Resume", None),
            (UiButton::Left, "Change slot", None),
            (UiButton::Screensaver, "Screenshot", Some("screenshot")),
            (UiButton::Menu, "Resume", Some(crate::pause::RESUME)),
        ],
        Screen::PauseReel => &[
            (UiButton::Accept, "Go back", None),
            (UiButton::Back, "Back", None),
            (UiButton::Left, "Moment", None),
            (UiButton::Screensaver, "Screenshot", Some("screenshot")),
        ],
    }
}

/// Start's action away from a game and a card: Home (EmuSen_BigPicture.md §51.2, §51.3).
pub const HOME: &str = "home";

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
/// `ConsolesPadCommand` has it, though the mockup's bar shows four entries.
pub fn unnamed(screen: Screen) -> &'static [Entry] {
    match screen {
        Screen::Consoles => &[(UiButton::Menu, "Home", Some(HOME))],
        _ => &[],
    }
}
