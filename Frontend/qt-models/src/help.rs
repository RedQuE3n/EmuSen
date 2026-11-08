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
            (UiButton::Menu, "Home", Some("homeCard")),
        ],
        Screen::HomeShelf => &[
            (UiButton::Accept, "Play", None),
            (UiButton::Screensaver, "Game page", Some("gamePage")),
            (UiButton::Search, "Favourite", Some("favourite")),
            (UiButton::Options, "Search", Some("search")),
            (UiButton::Menu, "Home", Some("homeCard")),
        ],
        Screen::HomePlaces => &[
            (UiButton::Accept, "Open", None),
            (UiButton::Options, "Search", Some("search")),
            (UiButton::Menu, "Home", Some("homeCard")),
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
            (UiButton::Menu, "Home", Some("homeCard")),
        ],
    }
}

/// The second button an entry names: the shoulders are one entry, LB and RB, as the accepted render draws them.
pub fn pair_of(button: UiButton) -> Option<UiButton> {
    (button == UiButton::PageUp).then_some(UiButton::PageDown)
}

/// The screen's actions as the frame declares them: `key Button`, one a line (EmuSen_Kit.md §7.19).
pub fn action_lines(screen: Screen) -> String {
    entries(screen).iter().filter_map(|(button, _, action)| action.map(|key| format!("{key} {button:?}\n"))).collect()
}
