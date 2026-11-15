//! The program's top sections (EmuSen_Qt.md §3.2, R-20261115-7), without a window: each is a screen the program opens, its
//! name one of the program's own words, the shown screen's section, and the screen a chosen one opens.

use emusen_qt_models::screens::{step, step_in, Ask, Shown, SECTIONS};
use emusen_qt_models::{help, sections, words};

#[test]
fn qt_3_2_every_section_is_a_screen_home_reaches() {
    assert_eq!(sections::all().iter().map(|s| s.key).collect::<Vec<_>>(), ["home", "consoles"], "Home and Consoles, in the nav's order");
    for section in sections::all() {
        assert!(words::is_fixed(section.name), "{} is one of the program's fixed words", section.name);
        let reached = section.screen == Shown::Home || (0..words::HOME_PLACES.len()).any(|at| step(Shown::Home, &Ask::Place(at)).map(|s| s.0) == Some(section.screen));
        assert!(reached, "{}: Home opens its screen by one of its places", section.key);
    }
    assert_eq!(sections::all().iter().map(|s| s.name).collect::<Vec<_>>(), ["Home", "Consoles"]);
}

#[test]
fn qt_3_2_the_shown_screen_names_its_section() {
    assert_eq!(sections::current(Shown::Home).map(|s| s.key), Some("home"));
    assert_eq!(sections::current(Shown::Consoles).map(|s| s.key), Some("consoles"));
    for other in [Shown::Grid, Shown::Game, Shown::Settings, Shown::HomeCard, Shown::Esde, Shown::Tool] {
        assert_eq!(sections::current(other), None, "{other:?} is no section");
    }
}

#[test]
fn qt_3_2_a_section_chosen_opens_its_screen() {
    assert_eq!(step(Shown::Home, &Ask::Section("consoles".into())), Some((Shown::Consoles, None)));
    assert_eq!(step(Shown::Consoles, &Ask::Section("home".into())), Some((Shown::Home, None)));
    assert_eq!(step(Shown::Home, &Ask::Section("home".into())), None, "the section shown stays");
    assert_eq!(step(Shown::Home, &Ask::Section("tools".into())), None, "a key that names no section");
    assert_eq!(step(Shown::Grid, &Ask::Section("home".into())), None, "a screen that draws no nav");
    assert_eq!(step_in(true, Shown::Esde, &Ask::Section("consoles".into())), None, "an ES-DE theme's view has no sections");
    // Consoles by its section is the screen Home's Games place opens.
    assert_eq!(step(Shown::Home, &Ask::Section("consoles".into())), step(Shown::Home, &Ask::Place(0)));
}

#[test]
fn qt_3_2_the_help_entry_is_the_shoulders_with_no_action() {
    assert_eq!(help::SECTIONS.1, "Sections");
    assert_eq!(help::SECTIONS.2, None, "the walk steps the sections, so the entry declares nothing");
    assert_eq!(help::pair_of(help::SECTIONS.0), Some(emusen_kit_core::input::UiButton::PageDown), "drawn as LB RB");
    assert_eq!(SECTIONS.len(), sections::all().len());
}
