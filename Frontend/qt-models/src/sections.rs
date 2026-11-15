//! The program's top sections as a nav reads them (EmuSen_Qt.md §3.2): their order, which the shown screen is, and the screen
//! a chosen one opens, from the screens' owner's `SECTIONS`. The shoulders step them through the kit's walk (EmuSen_Kit.md §7.27).

use crate::screens::{Section, Shown, SECTIONS};

/// Every section, in the nav's order.
pub fn all() -> &'static [Section] {
    &SECTIONS
}

/// The section the shown screen is; none for a screen that is no section.
pub fn current(shown: Shown) -> Option<&'static Section> {
    SECTIONS.iter().find(|s| s.screen == shown)
}

/// The screen a section's key opens from the shown screen: none from a screen that is no section, for the section already
/// shown, or for a key that names none.
pub fn opened(shown: Shown, key: &str) -> Option<Shown> {
    current(shown)?;
    SECTIONS.iter().find(|s| s.key == key).map(|s| s.screen).filter(|s| *s != shown)
}
