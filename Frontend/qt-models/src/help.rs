//! The help bar's entries for each screen, as the C# screens and the mockups name them (EmuSen_Interface.md §24.1-§24.3):
//! the button by what it asks of the interface, and its label. The A/B swap is the router's job, not this list's.

use emusen_kit_core::input::UiButton;

/// A screen with a help bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Home,
    Consoles,
    Grid,
}

/// The entries, in the bar's order.
pub fn entries(screen: Screen) -> &'static [(UiButton, &'static str)] {
    match screen {
        Screen::Home => &[(UiButton::Accept, "Resume"), (UiButton::Screensaver, "Game page"), (UiButton::Menu, "Home card")],
        Screen::Consoles => {
            &[(UiButton::Accept, "Open"), (UiButton::Back, "Back"), (UiButton::Search, "Favourites"), (UiButton::Screensaver, "Random game")]
        }
        Screen::Grid => &[(UiButton::Accept, "Play"), (UiButton::Screensaver, "Game page"), (UiButton::Search, "Favourite"), (UiButton::Back, "Consoles")],
    }
}
