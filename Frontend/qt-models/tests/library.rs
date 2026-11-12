//! The library view's behaviour with a fake client and no Qt (EmuSen_Qt.md §8.2): what rows a source's answers make,
//! that nothing per console is written by hand, and that a refresh keeps the selection.

use emusen_crystal_client::shapes::{CoreRow, LibretroCore, Look, SystemRow};
use emusen_platform_client::library::GameFile;
use emusen_qt_models::clients::engine_shown;
use emusen_qt_models::games::{from_json, to_json};
use emusen_qt_models::help::{action_lines, entries, Screen};
use emusen_qt_models::library::{games_of, refresh, title_of, without_port_mark, Engine, GameRow, Kind, LibrarySource, LibraryView};
use std::sync::Mutex;

/// A source answering what a test sets; `fail` makes the next refresh fail with its words.
#[derive(Default)]
struct Fake {
    systems: Vec<SystemRow>,
    games: Mutex<Vec<GameFile>>,
    favourites: Vec<String>,
    recent: Vec<String>,
    engines: Vec<(String, Engine)>,
    /// The media store's files, by path and type.
    media: Vec<(String, String, String)>,
    /// The games hidden in games.db, and the player's settings.
    hidden: Vec<String>,
    settings: serde_json::Value,
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
    fn favourites(&self, games: &[GameFile]) -> Result<Vec<String>, String> {
        Ok(self.favourites.iter().filter(|p| games.iter().any(|g| &g.path == *p)).cloned().collect())
    }
    fn recent(&self, games: &[GameFile]) -> Result<Vec<String>, String> {
        Ok(self.recent.iter().filter(|p| games.iter().any(|g| &g.path == *p)).cloned().collect())
    }
    fn hidden(&self, games: &[GameFile]) -> Result<Vec<String>, String> {
        Ok(self.hidden.iter().filter(|p| games.iter().any(|g| &g.path == *p)).cloned().collect())
    }
    fn settings(&self) -> Result<serde_json::Value, String> {
        Ok(self.settings.clone())
    }
    fn engines(&self, _: &[SystemRow]) -> Result<Vec<(String, Engine)>, String> {
        Ok(self.engines.clone())
    }
    fn media(&self, _: &[SystemRow], games: &[GameFile], types: &[&str]) -> Result<Vec<(String, String)>, String> {
        let first = |g: &GameFile| types.iter().find_map(|t| self.media.iter().find(|(p, k, _)| p == &g.path && k == t).map(|(_, _, f)| f.clone()));
        Ok(games.iter().filter_map(|g| first(g).map(|f| (g.path.clone(), f))).collect())
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
        .flat_map(|(s, n)| (0..*n).map(move |i| GameFile { path: format!("/roms/{s}/{i}"), system: Some(s.to_string()), ..GameFile::default() }))
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
    assert_eq!(lines, ["5 games", "Hearts on every console", "Last: 2"], "the places' own words, the game played last by its file name");
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
    let whole = GameRow { path: "/p.nes".into(), system: "nes".into(), title: "T".into(), hue: 12.5, favourite: true, cover: "c".into(), hero: "h".into(), console: "NES".into(), facts: "f".into(), played_at: 1_700_000_000, plays: 7, year: "1994".into(), rating: Some(0.8) };
    assert_eq!(from_json(&to_json([&whole])), vec![whole.clone()], "every field of a row crosses, as written straight from it");
    assert_eq!(title_of("/roms/Aurora Drift (Synthetic).sfc"), "Aurora Drift (Synthetic)");
    let mut view = LibraryView::empty();
    view.apply(Ok(read));
    assert_eq!(view.summary(), "2 consoles · 5 games");
}

#[test]
fn a_played_game_s_hero_art_is_its_fan_art_else_its_screenshot_else_its_title_screen_and_none_is_read_for_the_rest() {
    let mut source = fake();
    let file = |p: &str, k: &str| (p.to_owned(), k.to_owned(), format!("{p}.{k}.png"));
    source.media = vec![
        file("/roms/nes/2", "screenshot"),
        file("/roms/nes/2", "fanart"),
        file("/roms/gb/0", "titlescreen"),
        file("/roms/gb/0", "screenshot"),
        file("/roms/nes/0", "cover"),
        file("/roms/gb/1", "fanart"),
    ];
    let read = refresh(&source).unwrap();
    let heroes: Vec<(String, String)> = games_of("recent", &read.systems, &read.catalog).into_iter().map(|g| (g.hero, g.cover)).collect();
    assert_eq!(heroes[0].0, "/roms/nes/2.fanart.png", "fan art before a screenshot");
    assert_eq!(heroes[1].0, "/roms/gb/0.screenshot.png", "a screenshot before the title screen");
    assert_eq!(heroes[2], (String::new(), "/roms/nes/0.cover.png".into()), "a cover is no hero art");
    let unplayed = games_of("favourites", &read.systems, &read.catalog);
    assert_eq!(unplayed[0].hero, "", "a game never played has none read");
    assert_eq!(games_of("recent", &read.systems, &read.catalog)[0].console, "NES");
    let shelf = emusen_qt_models::games::without(games_of("recent", &read.systems, &read.catalog), "/roms/gb/0");
    assert_eq!(shelf.iter().map(|g| g.path.as_str()).collect::<Vec<_>>(), ["/roms/nes/2", "/roms/nes/0", "/roms/nes/1"], "the suspended game left off the shelf");
}

#[test]
fn a_paused_game_is_said_as_mistress_s_home_says_it() {
    use emusen_qt_models::words::{paused_ago, paused_facts};
    let at = 1_000_000;
    let said: Vec<String> = [0, 59, 60, 179, 3599, 3600, 7200, 86_399, 86_400, 172_799, 172_800].iter().map(|s| paused_ago(at, at + s)).collect();
    assert_eq!(
        said,
        [
            "Paused just now", "Paused just now", "Paused 1 minute ago", "Paused 2 minutes ago", "Paused 59 minutes ago",
            "Paused 1 hour ago", "Paused 2 hours ago", "Paused 23 hours ago", "Paused yesterday", "Paused yesterday",
            "Paused 2 days ago",
        ]
    );
    assert_eq!(paused_facts("Super Nintendo", at, at + 5), "Super Nintendo · Paused just now");
    assert_eq!(paused_facts("", at, at + 120), "Paused 2 minutes ago", "a game the library does not hold has no console");
    let stamp = emusen_qt_models::words::utc(1_762_638_120);
    assert_eq!((stamp.as_str(), emusen_qt_models::words::parse_utc(&stamp)), ("2025-11-08T21:42:00Z", Some(1_762_638_120)));
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
    assert_eq!(labels(Screen::HomeShelf)[0], "Accept Play", "a shelf game plays, as Mistress's Home has it");
    assert_eq!(labels(Screen::HomePlaces), ["Accept Open", "Options Search", "Menu Home"]);
    assert_eq!(labels(Screen::Grid)[0], "Accept Play");
    // B clears a search first, as Mistress's bar says; both shoulders are declared though the bar draws them as one entry.
    assert_eq!(labels(Screen::GridSearched).iter().find(|l| l.starts_with("Back")).map(String::as_str), Some("Back Clear search"));
    assert!(action_lines(Screen::Grid).contains("previous PageUp\n") && action_lines(Screen::Grid).contains("next PageDown\n"));
    assert!(action_lines(Screen::Search).contains("toGrid PageUp\n") && action_lines(Screen::Search).contains("toGrid PageDown\n"));
}

#[test]
fn each_screen_declares_its_own_actions_and_the_walk_reads_them() {
    assert_eq!(action_lines(Screen::Consoles), "favourites Search\nrandom Screensaver\nhome Menu\n", "Start goes Home, unnamed on the bar");
    for screen in [Screen::Home, Screen::HomeShelf, Screen::HomePlaces, Screen::Consoles, Screen::Grid, Screen::GridSearched, Screen::Search] {
        let read = emusen_kit_core::input::WalkAction::from_lines(&action_lines(screen));
        let named: Vec<_> = entries(screen).iter().chain(emusen_qt_models::help::unnamed(screen)).filter(|e| e.2.is_some()).map(|e| (e.2.unwrap().to_owned(), Some(e.0))).collect();
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
    *source.fail.lock().unwrap() = Some("games.db is locked".into());
    view.apply(refresh(&source));
    assert_eq!((view.rows.rows().len(), view.problem.as_str()), (5, "The library could not be read: games.db is locked"));
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
    assert_eq!(step(Shown::Home, &Ask::Play), Some((Shown::Game, None)), "a shelf game or Continue shows the game");
    assert_eq!(step(Shown::Grid, &Ask::Play), Some((Shown::Game, None)));
    assert_eq!(step(Shown::Consoles, &Ask::Play), None, "Choose a console plays nothing");
    assert_eq!(step(Shown::Game, &Ask::Menu), Some((Shown::Pause, None)), "the chord over the game is the Pause Card");
    assert_eq!(step(Shown::Game, &Ask::Back), None, "B is the game's");
    assert_eq!(step(Shown::Game, &Ask::Closed), Some((Shown::Home, None)));
    assert_eq!(step(Shown::Pause, &Ask::Play), Some((Shown::Game, None)), "the card resuming shows the game");
    assert_eq!(step(Shown::Pause, &Ask::Library), Some((Shown::Home, None)), "Back to Library: Home, the game suspended");
    assert_eq!(step(Shown::Pause, &Ask::Closed), Some((Shown::Home, None)), "a game that stops under the card");
    assert_eq!(step(Shown::Pause, &Ask::Menu), None, "the card's own Start is its action, not the chord");
    assert_eq!(step(Shown::Game, &Ask::Library), Some((Shown::Home, None)), "the Exit to Library hotkey: Home, the game suspended");
}

#[test]
fn every_row_s_games_and_its_fan_are_written_once_a_refresh_at_a_library_s_scale() {
    let mut source = fake();
    *source.games.lock().unwrap() = games(&[("nes", 6000), ("gb", 3000)]);
    source.favourites = (0..3000).step_by(7).map(|i| format!("/roms/gb/{i}")).collect();
    let read = refresh(&source).unwrap();
    assert_eq!(read.rows.len(), 5);
    for row in &read.rows {
        let games = games_of(&row.key, &read.systems, &read.catalog);
        assert_eq!(from_json(&read.lists[&row.key]), games, "{}'s games as gamesOf answered them", row.key);
        assert_eq!(from_json(&row.fan), games[..games.len().min(4)], "{}'s fan, its first four", row.key);
    }
    assert_eq!(from_json(&read.lists["all"]).len(), 9000);
    let mut view = LibraryView::empty();
    view.apply(Ok(read));
    assert_eq!(view.games_text("nowhere"), "[]", "a key that is no row leads to no games");
    assert_eq!(from_json(view.games_text("recent")).len(), 4);
}

#[test]
fn the_themes_group_and_back_entries_go_where_the_canvases_put_them() {
    use emusen_kit_core::input::UiButton;
    use emusen_qt_models::help::entries_with;
    let words = |screen, group, back| -> Vec<&str> { entries_with(screen, group, back).iter().map(|e| e.1).collect() };
    assert_eq!(words(Screen::Home, true, false), ["Resume", "Game page", "Favourite", "Group", "Search", "Home"]);
    assert_eq!(words(Screen::HomePlaces, true, false), ["Open", "Group", "Search", "Home"]);
    assert_eq!(words(Screen::Consoles, true, false), ["Open", "Back", "Group", "Favourites", "Random game"]);
    assert_eq!(words(Screen::HomeShelf, false, true), ["Play", "Game page", "Favourite", "Back", "Search", "Home"]);
    assert_eq!(words(Screen::HomePlaces, true, true), ["Open", "Back", "Group", "Search", "Home"]);
    assert_eq!(entries_with(Screen::Consoles, false, true), entries(Screen::Consoles), "a screen with B gets no second");
    for screen in [Screen::Grid, Screen::GridSearched, Screen::Search] {
        assert_eq!(entries_with(screen, true, true), entries(screen), "{screen:?}: the shoulders already mean something");
    }
    let group = entries_with(Screen::Consoles, true, false)[2];
    assert_eq!((group.0, group.2), (UiButton::PageUp, None), "the shoulders' pair, no action: the theme's walk answers it");
    assert_eq!(action_lines(Screen::Consoles), "favourites Search\nrandom Screensaver\nhome Menu\n", "the declared actions unchanged");
}

#[test]
fn qt_3_7_2_fixed_words_are_told_from_data() {
    use emusen_qt_models::words::{games, is_fixed, CHOOSE_A_CONSOLE, FAVOURITES_LINE};
    for fixed in [CHOOSE_A_CONSOLE, FAVOURITES_LINE, "Recently played", "Settings", "Art and facts"] {
        assert!(is_fixed(fixed), "{fixed} is one of the program's fixed words");
    }
    for data in ["Aurora Drift", "Super Nintendo", "Last: Aurora Drift", &games(795), ""] {
        assert!(!is_fixed(data), "{data} is data");
    }
}

#[test]
fn qt_7_8_g2_a_hidden_game_leaves_every_place_until_settings_lists_hidden_games() {
    let mut source = fake();
    source.hidden = vec!["/roms/gb/0".into(), "/roms/gb/1".into()];
    let rows = refresh(&source).unwrap().rows;
    let seen: Vec<(&str, usize)> = rows.iter().map(|r| (r.key.as_str(), r.count)).collect();
    assert_eq!(seen, [("all", 3), ("favourites", 0), ("recent", 3), ("nes", 3)], "the Game Boy's two games hidden, its console leaves too");
    source.settings = serde_json::json!({ "ShowHiddenGames": true });
    let rows = refresh(&source).unwrap().rows;
    assert_eq!(rows.iter().map(|r| r.count).collect::<Vec<_>>(), [5, 1, 4, 3, 2], "listed again with the rest");
}
