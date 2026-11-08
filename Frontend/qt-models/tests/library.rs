//! The library view's behaviour with a fake client and no Qt (EmuSen_Qt.md §8.2): what rows a source's answers make,
//! that nothing per console is written by hand, and that a refresh keeps the selection.

use emusen_crystal_client::shapes::{Look, SystemRow};
use emusen_platform_client::library::GameFile;
use emusen_qt_models::library::{refresh, Kind, LibrarySource, LibraryView};
use std::sync::Mutex;

/// A source answering what a test sets; `fail` makes the next refresh fail with its words.
#[derive(Default)]
struct Fake {
    systems: Vec<SystemRow>,
    games: Mutex<Vec<GameFile>>,
    favourites: usize,
    recent: usize,
    fail: Mutex<Option<String>>,
}

impl LibrarySource for Fake {
    fn systems(&self) -> Result<Vec<SystemRow>, String> {
        Ok(self.systems.clone())
    }
    fn games(&self) -> Result<Vec<GameFile>, String> {
        match self.fail.lock().unwrap().take() {
            Some(words) => Err(words),
            None => Ok(self.games.lock().unwrap().clone()),
        }
    }
    fn favourites(&self, _: &[GameFile]) -> Result<usize, String> {
        Ok(self.favourites)
    }
    fn recent(&self, _: &[GameFile]) -> Result<usize, String> {
        Ok(self.recent)
    }
}

fn system(id: &str, display: &str, hue: f64, engine: &str) -> SystemRow {
    SystemRow {
        id: id.into(),
        name: format!("{id} system"),
        display_name: display.into(),
        manufacturer: "Maker".into(),
        release_year: 1990,
        look: Look { hue, body: "planet".into(), ..Look::default() },
        engines: vec![engine.into()],
        ..SystemRow::default()
    }
}

fn games(of: &[(&str, usize)]) -> Vec<GameFile> {
    of.iter()
        .flat_map(|(s, n)| (0..*n).map(move |i| GameFile { path: format!("/roms/{s}/{i}"), system: Some(s.to_string()), package: None }))
        .collect()
}

fn fake() -> Fake {
    Fake {
        systems: vec![
            system("nes", "NES", 330.0, "Moon"),
            system("snes", "Super Nintendo", 38.0, "Venus"),
            system("gb", "Game Boy", 212.0, "Mercury"),
        ],
        games: Mutex::new(games(&[("nes", 3), ("gb", 2)])),
        favourites: 1,
        recent: 4,
        ..Fake::default()
    }
}

#[test]
fn the_places_come_first_then_each_console_with_games_in_the_table_s_order() {
    let rows = refresh(&fake()).unwrap();
    let seen: Vec<(Option<Kind>, &str, &str, usize)> = rows.iter().map(|r| (r.kind, r.key.as_str(), r.name.as_str(), r.count)).collect();
    assert_eq!(
        seen,
        [
            (Some(Kind::Place), "all", "All games", 5),
            (Some(Kind::Place), "favourites", "Favourites", 1),
            (Some(Kind::Place), "recent", "Recently played", 4),
            (Some(Kind::Console), "nes", "NES", 3),
            (Some(Kind::Console), "gb", "Game Boy", 2),
        ],
        "the Super Nintendo has no games and is not shown"
    );
    let gb = &rows[4];
    assert_eq!((gb.hue, gb.body.as_str(), gb.maker.as_str(), gb.year, gb.engine.as_str()), (212.0, "planet", "Maker", 1990, "Mercury"));
}

#[test]
fn a_console_only_an_engine_declares_shows_by_its_declared_name_with_nothing_written_for_it() {
    let declared = SystemRow {
        id: "test".into(),
        name: "Test system".into(),
        engines: vec!["V1TestCore".into()],
        look: Look { hue: 77.0, body: "planet".into(), ..Look::default() },
        ..SystemRow::default()
    };
    let source = Fake { systems: vec![declared], games: Mutex::new(games(&[("test", 1)])), ..Fake::default() };
    let rows = refresh(&source).unwrap();
    let test = rows.iter().find(|r| r.key == "test").expect("the declared console");
    assert_eq!((test.name.as_str(), test.hue, test.engine.as_str(), test.count), ("Test system", 77.0, "V1TestCore", 1));
}

#[test]
fn a_refresh_keeps_the_selection_by_its_key_and_near_its_place_when_it_is_gone() {
    let source = fake();
    let mut view = LibraryView::empty();
    assert_eq!((view.rows.rows().len(), view.rows.selected()), (3, Some(0)), "the places alone at first, the first selected");
    view.apply(refresh(&source));
    assert!(view.rows.select(4));
    assert_eq!(view.rows.selected_key(), Some("gb"));

    // A console appears before the selected one: the selection follows the Game Boy.
    *source.games.lock().unwrap() = games(&[("nes", 3), ("snes", 1), ("gb", 2)]);
    view.apply(refresh(&source));
    assert_eq!((view.rows.selected(), view.rows.selected_key()), (Some(5), Some("gb")));

    // The Game Boy's games are gone: the selection stays at its place, clamped to the last row.
    *source.games.lock().unwrap() = games(&[("nes", 3)]);
    view.apply(refresh(&source));
    assert_eq!((view.rows.selected(), view.rows.selected_key()), (Some(3), Some("nes")));
}

#[test]
fn a_failed_refresh_keeps_the_rows_and_shows_the_source_s_words() {
    let source = fake();
    let mut view = LibraryView::empty();
    view.apply(refresh(&source));
    *source.fail.lock().unwrap() = Some("The library could not be read.".into());
    view.apply(refresh(&source));
    assert_eq!((view.rows.rows().len(), view.problem.as_str()), (5, "The library could not be read."));
    view.apply(refresh(&source));
    assert!(view.problem.is_empty(), "the next good refresh clears it");
}
