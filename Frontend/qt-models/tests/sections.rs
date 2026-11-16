//! The program's top sections (EmuSen_Qt.md §3.4.8, R-20261115-7), without a window: the boards' five in their order, each
//! name one of the program's own words, the shown screen's section, and the working screen a chosen one opens.

use emusen_qt_models::screens::{step, step_in, Ask, Shown, SECTIONS};
use emusen_qt_models::{help, home_card, library, list_card, sections, words};

#[test]
fn qt_3_4_8_the_sections_are_the_boards_five_in_order() {
    assert_eq!(sections::all().iter().map(|s| s.key).collect::<Vec<_>>(), ["home", "library", "consoles", "collections", "tools"]);
    assert_eq!(sections::all().iter().map(|s| s.name).collect::<Vec<_>>(), ["Home", "Library", "Consoles", "Collections", "Tools"]);
    for section in sections::all() {
        assert!(words::is_fixed(section.name), "{} is one of the program's fixed words", section.name);
    }
}

#[test]
fn qt_3_4_8_the_shown_screen_names_its_section() {
    assert_eq!(sections::current(Shown::Home).map(|s| s.key), Some("home"));
    assert_eq!(sections::current(Shown::Consoles).map(|s| s.key), Some("consoles"));
    for other in [Shown::Grid, Shown::List, Shown::Game, Shown::Settings, Shown::HomeCard, Shown::Esde, Shown::Tool] {
        assert_eq!(sections::current(other), None, "{other:?} draws no nav");
    }
}

#[test]
fn qt_3_4_8_each_section_opens_a_screen_the_program_has() {
    for from in [Shown::Home, Shown::Consoles] {
        assert_eq!(step(from, &Ask::Section("library".into())), Some((Shown::Grid, Some(library::ALL.to_owned()))), "Library: every game");
        assert_eq!(step(from, &Ask::Section("collections".into())), Some((Shown::List, Some(library::ALL.to_owned()))), "Collections: the List card");
        assert_eq!(step(from, &Ask::Section("tools".into())), Some((Shown::HomeCard, Some(home_card::TOOLS.to_owned()))), "Tools: the Home card's page");
    }
    assert_eq!(sections::by_key("collections").and_then(|s| s.page), Some(list_card::FILTER), "at its Filter page, which lists the collections");
    assert_eq!(step(Shown::Home, &Ask::Section("consoles".into())), Some((Shown::Consoles, None)));
    assert_eq!(step(Shown::Consoles, &Ask::Section("home".into())), Some((Shown::Home, None)));
    // Consoles by its section is the screen Home's Games place opens.
    assert_eq!(step(Shown::Home, &Ask::Section("consoles".into())), step(Shown::Home, &Ask::Place(0)));
}

#[test]
fn qt_3_4_8_a_section_is_declined_where_it_cannot_open() {
    assert_eq!(step(Shown::Home, &Ask::Section("home".into())), None, "the section shown stays");
    assert_eq!(step(Shown::Consoles, &Ask::Section("consoles".into())), None, "the section shown stays");
    assert_eq!(step(Shown::Home, &Ask::Section("nowhere".into())), None, "a key that names no section");
    assert_eq!(step(Shown::Grid, &Ask::Section("home".into())), None, "a screen that draws no nav");
    assert_eq!(step_in(true, Shown::Esde, &Ask::Section("consoles".into())), None, "an ES-DE theme's view has no sections");
}

#[test]
fn qt_3_4_8_the_help_entry_is_the_shoulders_with_no_action() {
    assert_eq!(help::SECTIONS.1, "Sections");
    assert_eq!(help::SECTIONS.2, None, "the walk steps the sections, so the entry declares nothing");
    assert_eq!(help::pair_of(help::SECTIONS.0), Some(emusen_kit_core::input::UiButton::PageDown), "drawn as LB RB");
    assert_eq!(SECTIONS.len(), sections::all().len());
}
