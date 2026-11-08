//! The help bar's entries for each screen, as the C# screens and the mockups name them (EmuSen_Interface.md §24.1-§24.3):
//! the button by what it asks of the interface, its label, and the action it invokes where the walk does not answer it
//! itself. The A/B swap is the router's job, not this list's.

use emusen_kit_core::input::UiButton;

/// A screen with a help bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Home,
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
            (UiButton::Menu, "Home card", Some("homeCard")),
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
            (UiButton::Back, "Consoles", None),
        ],
    }
}

/// The screen's actions as the frame declares them: `key Button`, one a line (EmuSen_Kit.md §7.17).
pub fn action_lines(screen: Screen) -> String {
    entries(screen).iter().filter_map(|(button, _, action)| action.map(|key| format!("{key} {button:?}\n"))).collect()
}
