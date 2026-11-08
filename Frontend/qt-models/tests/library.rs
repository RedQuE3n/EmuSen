//! The library view's behaviour with a fake client and no Qt (EmuSen_Qt.md §8.2): what rows a source's answers make,
//! that nothing per console is written by hand, and that a refresh keeps the selection.

use emusen_crystal_client::shapes::{CoreRow, LibretroCore, Look, SystemRow};
use emusen_platform_client::library::GameFile;
use emusen_qt_models::clients::engine_shown;
use emusen_qt_models::games::{from_json, to_json};
use emusen_qt_models::help::{action_lines, entries, Screen};
use emusen_qt_models::library::{games_of, refresh, title_of, without_port_mark, Engine, Kind, LibrarySource, LibraryView};
use std::sync::Mutex;

/// A source answering what a test sets; `fail` makes the next refresh fail with its words.
#[derive(Default)]
struct Fake {
    systems: Vec<SystemRow>,
    games: Mutex<Vec<GameFile>>,
    favourites: Vec<String>,
    recent: Vec<String>,
    engines: Vec<(String, Engine)>,
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
    fn favourites(&self, _: &[GameFile]) -> Result<Vec<String>, String> {
        Ok(self.favourites.clone())
    }
    fn recent(&self, _: &[GameFile]) -> Result<Vec<String>, String> {
        Ok(self.recent.clone())
    }
    fn engines(&self, _: &[SystemRow]) -> Result<Vec<(String, Engine)>, String> {
        Ok(self.engines.clone())
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
        favourites: vec!["/roms/gb/1".into()],
        recent: vec!["/roms/nes/2".into(), "/roms/gb/0".into(), "/roms/nes/0".into(), "/roms/nes/1".into()],
        engines: vec![
            ("nes".into(), Engine { id: "Moon".into(), name: "Moon".into(), libretro: false }),
            ("gb".into(), Engine { id: "sameboy".into(), name: "SameBoy".into(), libretro: true }),
        ],
        ..Fake::default()
    }
}

#[test]
fn the_places_come_first_then_each_console_with_games_in_the_table_s_order() {
    let rows = refresh(&fake()).unwrap().rows;
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
    assert_eq!((gb.hue, gb.body.as_str(), gb.maker.as_str(), gb.year), (212.0, "planet", "Maker", 1990));
    assert_eq!((gb.engine.as_str(), gb.engine_name.as_str(), gb.libretro), ("sameboy", "SameBoy", true), "the engine the source says runs it now");
    assert_eq!((gb.full_name.as_str(), gb.status.as_str()), ("gb system", "2 games"));
    assert_eq!(gb.facts, "Maker · 1990 · 2 games · runs on SameBoy, a libretro core");
    assert_eq!(rows[3].facts, "Maker · 1990 · 3 games · runs on Moon, an EmuSen core");
    let lines: Vec<&str> = rows[..3].iter().map(|r| r.status.as_str()).collect();
    assert_eq!(lines, ["5 games", "Your hearts, every console", "Last: 2"], "the places' own words, the game played last by its file name");
}

#[test]
fn a_row_leads_to_its_games_by_title_and_recent_by_when() {
    let source = fake();
    let read = refresh(&source).unwrap();
    let titles = |key: &str| -> Vec<String> {
        games_of(key, &read.systems, &read.catalog).iter().map(|g| format!("{}/{}", g.system, g.title)).collect()
    };
    assert_eq!(titles("nes"), ["nes/0", "nes/1", "nes/2"]);
    assert_eq!(titles("recent"), ["nes/2", "gb/0", "nes/0", "nes/1"]);
    assert_eq!(titles("favourites"), ["gb/1"]);
    assert_eq!(titles("all").len(), 5);
    let gb = games_of("gb", &read.systems, &read.catalog);
    assert_eq!((gb[1].hue, gb[1].favourite, gb[0].favourite), (212.0, true, false));
    assert_eq!(from_json(&to_json(&gb)), gb, "the plain data between the two models reads back whole");
    assert_eq!(title_of("/roms/Aurora Drift (Synthetic).sfc"), "Aurora Drift (Synthetic)");
    let mut view = LibraryView::empty();
    view.apply(Ok(read));
    assert_eq!(view.summary(), "2 consoles · 5 games");
}

#[test]
fn an_engine_is_named_by_its_own_info_and_the_port_s_mark_is_dropped() {
    assert_eq!((without_port_mark("VenusRT"), without_port_mark("PCSX2"), without_port_mark("RT")), ("Venus", "PCSX2", "RT"));
    let own = CoreRow { engine: Some("Venus".into()), name: Some("VenusRT".into()), ..CoreRow::default() };
    let hosted = CoreRow {
        engine: Some("pcsx2".into()),
        name: Some("pcsx2_libretro".into()),
        libretro: Some(LibretroCore { corename: Some("PCSX2".into()), ..LibretroCore::default() }),
        ..CoreRow::default()
    };
    let cores = [own, hosted];
    assert_eq!(engine_shown("Venus", &cores), Engine { id: "Venus".into(), name: "Venus".into(), libretro: false });
    assert_eq!(engine_shown("pcsx2", &cores), Engine { id: "pcsx2".into(), name: "PCSX2".into(), libretro: true });
    assert_eq!(engine_shown("Moon", &cores).name, "Moon", "an engine the listing lacks keeps its stored name");
}

#[test]
fn each_screen_s_help_bar_names_its_presses() {
    let labels = |s| entries(s).iter().map(|(b, l, _)| format!("{b:?} {l}")).collect::<Vec<_>>();
    assert_eq!(labels(Screen::Consoles), ["Accept Open", "Back Back", "Search Favourites", "Screensaver Random game"]);
    assert_eq!(labels(Screen::Home)[0], "Accept Resume");
    assert_eq!(labels(Screen::Grid)[0], "Accept Play");
}

#[test]
fn each_screen_declares_its_own_actions_and_the_walk_reads_them() {
    assert_eq!(action_lines(Screen::Consoles), "favourites Search\nrandom Screensaver\n");
    for screen in [Screen::Home, Screen::Consoles, Screen::Grid] {
        let read = emusen_kit_core::input::WalkAction::from_lines(&action_lines(screen));
        let named: Vec<_> = entries(screen).iter().filter(|e| e.2.is_some()).map(|e| (e.2.unwrap().to_owned(), Some(e.0))).collect();
        assert_eq!(read.into_iter().map(|a| (a.key, a.button)).collect::<Vec<_>>(), named, "{screen:?}");
    }
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
    let rows = refresh(&source).unwrap().rows;
    let test = rows.iter().find(|r| r.key == "test").expect("the declared console");
    assert_eq!((test.name.as_str(), test.hue, test.count), ("Test system", 77.0, 1));
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

#[test]
fn moonrise_has_a_planet_for_each_system_with_an_engine_games_or_not() {
    let mut systems = fake().systems;
    systems.push(SystemRow { engines: Vec::new(), ..system("pico", "Pico", 99.0, "") });
    systems[0].look.palette = vec!["#FFF8FB".into(), "#F3CFE3".into(), "#A9789F".into()];
    systems[0].look.craters = 0.28;
    systems[1].look.body = "gem".into();
    let worlds = emusen_qt_models::library::splash_worlds(&systems);
    assert_eq!(
        worlds,
        vec!["330 planet 0.28 0 0 none 0 #FFF8FB #F3CFE3 #A9789F", "38 gem 0 0 0 none 0", "212 planet 0 0 0 none 0"],
        "in the table's order, each from the table's look, none without an engine"
    );
    assert_eq!(emusen_qt_models::words::splash_caption(3), "3 CONSOLES");
    assert_eq!(emusen_qt_models::words::splash_caption(1), "1 CONSOLE");
    assert_eq!(emusen_qt_models::words::counted(1566, "game", "games"), "1,566 games");
    assert_eq!(emusen_qt_models::words::grouped(1_234_567), "1,234,567");
}

#[test]
fn the_screens_switch_by_what_was_asked() {
    use emusen_qt_models::screens::{step, Ask, Shown};
    assert_eq!(step(Shown::Home, &Ask::Place(0)), Some((Shown::Consoles, None)));
    assert_eq!(step(Shown::Home, &Ask::Place(1)), None, "the scraper is a later stage's");
    assert_eq!(step(Shown::Consoles, &Ask::Open("nes".into())), Some((Shown::Grid, Some("nes".into()))));
    assert_eq!(step(Shown::Consoles, &Ask::Act("favourites".into())), Some((Shown::Grid, Some("favourites".into()))));
    assert_eq!(step(Shown::Consoles, &Ask::Act("random".into())), None, "a later stage's action changes nothing yet");
    assert_eq!(step(Shown::Grid, &Ask::Back), Some((Shown::Consoles, None)));
    assert_eq!(step(Shown::Consoles, &Ask::Back), Some((Shown::Home, None)));
    assert_eq!(step(Shown::Home, &Ask::Back), None, "B on Home stays");
}
