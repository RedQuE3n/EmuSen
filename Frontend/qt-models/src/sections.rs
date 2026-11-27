//! The program's top sections as a nav reads them (EmuSen_Qt.md §3.4.8): their order, which the shown screen is, and the
//! screen a chosen one opens, from the screens' owner's `SECTIONS`. The shoulders step them through the kit's walk (EmuSen_Kit.md §7.27).

use crate::screens::{Section, Shown, SECTIONS};

/// Every section, in the nav's order.
pub fn all() -> &'static [Section] {
    &SECTIONS
}

/// A section by its key.
pub fn by_key(key: &str) -> Option<&'static Section> {
    SECTIONS.iter().find(|s| s.key == key)
}

/// The section the shown screen is, for a screen that draws the nav (one a section opens at no key); none for any other.
pub fn current(shown: Shown) -> Option<&'static Section> {
    SECTIONS.iter().find(|s| s.screen == shown && s.at.is_none())
}

/// The section shown on `shown` at `at` (mapping top-level Grid at `"all"` to `"library"`), falling back to `current`.
pub fn current_at(shown: Shown, at: &str) -> Option<&'static Section> {
    SECTIONS.iter().find(|s| s.screen == shown && s.at.is_some_and(|place| place == at)).or_else(|| current(shown))
}

/// The section a key opens from the shown screen: none from a screen that draws no nav, for the section already shown, or
/// for a key that names none.
pub fn opened(shown: Shown, key: &str) -> Option<&'static Section> {
    opened_at_inner(current(shown), key)
}

/// As `opened`, taking Grid's `at` key into account so top-level Library steps to sibling sections.
pub fn opened_at(shown: Shown, at: &str, key: &str) -> Option<&'static Section> {
    opened_at_inner(current_at(shown, at), key)
}

fn opened_at_inner(from: Option<&'static Section>, key: &str) -> Option<&'static Section> {
    let from = from?;
    by_key(key).filter(|s| s.key != from.key)
}
