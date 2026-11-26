//! Unit tests for Reference Desktop Mode projections, view switching, search/filtering, and multi-select marks.

use emusen_crystal_client::shapes::{Look, SystemRow};
use emusen_platform_client::library::GameFile;
use emusen_qt_models::desktop::{chrome_json, project_desktop, CachedInspector, DesktopState, SessionSnapshot};
use emusen_qt_models::esde_shelves::Plays;
use emusen_qt_models::library::{refresh, Engine, GameFacts, LibrarySource, LibraryView};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Default)]
struct FakeLibrary {
    systems: Vec<SystemRow>,
    games: Vec<GameFile>,
    favourites: Vec<String>,
    recent: Vec<String>,
    engines: Vec<(String, Engine)>,
    played: Vec<(String, i64)>,
    plays: Vec<(String, Plays)>,
    metadata: Vec<(String, BTreeMap<String, String>)>,
}

impl LibrarySource for FakeLibrary {
    fn systems(&self) -> Result<Vec<SystemRow>, String> {
        Ok(self.systems.clone())
    }
    fn games(&self) -> Result<Vec<GameFile>, String> {
        Ok(self.games.clone())
    }
    fn favourites(&self, games: &[GameFile]) -> Result<Vec<String>, String> {
        Ok(self.favourites.iter().filter(|p| games.iter().any(|g| &g.path == *p)).cloned().collect())
    }
    fn recent(&self, games: &[GameFile]) -> Result<Vec<String>, String> {
        Ok(self.recent.iter().filter(|p| games.iter().any(|g| &g.path == *p)).cloned().collect())
    }
    fn engines(&self, _: &[SystemRow]) -> Result<Vec<(String, Engine)>, String> {
        Ok(self.engines.clone())
    }
    fn played(&self, games: &[GameFile]) -> Result<Vec<(String, i64)>, String> {
        Ok(self.played.iter().filter(|(p, _)| games.iter().any(|g| &g.path == p)).cloned().collect())
    }
    fn plays(&self, games: &[GameFile]) -> Result<Vec<(String, Plays)>, String> {
        Ok(self.plays.iter().filter(|(p, _)| games.iter().any(|g| &g.path == p)).cloned().collect())
    }
    fn metadata(&self, _: &[GameFile]) -> Result<Vec<(String, BTreeMap<String, String>)>, String> {
        Ok(self.metadata.clone())
    }
}

fn sample_view() -> LibraryView {
    let mut snes_meta = BTreeMap::new();
    snes_meta.insert("genre".into(), "Action".into());
    snes_meta.insert("releasedate".into(), "19940101T000000".into());
    snes_meta.insert("players".into(), "1-2".into());
    let mut psx_meta = BTreeMap::new();
    psx_meta.insert("genre".into(), "Tactics".into());
    psx_meta.insert("releasedate".into(), "19980101T000000".into());
    let source = FakeLibrary {
        systems: vec![
            SystemRow { id: "snes".into(), name: "Super Nintendo".into(), display_name: "Super Nintendo".into(), look: Look { hue: 260.0, body: "planet".into(), ..Look::default() }, engines: vec!["Venus".into()], ..SystemRow::default() },
            SystemRow { id: "psx".into(), name: "PlayStation".into(), display_name: "PlayStation".into(), look: Look { hue: 205.0, body: "planet".into(), ..Look::default() }, engines: vec!["Diamond".into()], ..SystemRow::default() },
            SystemRow { id: "gba".into(), name: "Game Boy Advance".into(), display_name: "Game Boy Advance".into(), look: Look { hue: 140.0, body: "planet".into(), ..Look::default() }, engines: vec!["mGBA".into()], ..SystemRow::default() },
        ],
        games: vec![
            GameFile { path: "/roms/snes/Aurora Drift.sfc".into(), system: Some("snes".into()), ..GameFile::default() },
            GameFile { path: "/roms/snes/Brass Lantern.sfc".into(), system: Some("snes".into()), ..GameFile::default() },
            GameFile { path: "/roms/psx/Basalt Kings.chd".into(), system: Some("psx".into()), ..GameFile::default() },
        ],
        favourites: vec!["/roms/snes/Aurora Drift.sfc".into()],
        recent: vec!["/roms/snes/Aurora Drift.sfc".into()],
        engines: vec![
            ("snes".into(), Engine { id: "Venus".into(), name: "Venus".into(), libretro: false }),
            ("psx".into(), Engine { id: "Diamond".into(), name: "Diamond".into(), libretro: false }),
        ],
        played: vec![("/roms/snes/Aurora Drift.sfc".into(), 1_700_000_000)],
        plays: vec![("/roms/snes/Aurora Drift.sfc".into(), Plays { count: 4, seconds: 3600 })],
        metadata: vec![
            ("/roms/snes/Aurora Drift.sfc".into(), snes_meta),
            ("/roms/psx/Basalt Kings.chd".into(), psx_meta),
        ],
    };
    let mut view = LibraryView::empty();
    view.apply(refresh(&source));
    view
}

#[test]
fn desktop_state_handles_view_switching_and_multi_select_marks() {
    let mut state = DesktopState::default();
    assert_eq!(state.view, 0);
    assert!(state.set_view(1));
    assert_eq!(state.view, 1);
    assert!(!state.set_view(1), "unchanged view reports false");
    assert!(state.set_view(9), "clamps upper bound to Settings (2)");
    assert_eq!(state.view, 2);
    assert!(state.set_view(-3), "clamps lower bound to Library (0)");
    assert_eq!(state.view, 0);

    let rows = vec!["/roms/a.sfc".to_owned(), "/roms/b.chd".to_owned()];
    state.toggle_mark("/roms/a.sfc");
    assert_eq!(state.marked_keys, vec!["/roms/a.sfc"]);
    state.toggle_mark("/roms/a.sfc");
    assert!(state.marked_keys.is_empty());

    state.mark_all(&rows);
    assert_eq!(state.marked_keys, rows);
    state.mark_all(&rows);
    assert!(state.marked_keys.is_empty(), "second mark_all clears all marked rows");

    state.toggle_mark("/roms/b.chd");
    state.clear_marks();
    assert!(state.marked_keys.is_empty());
}

#[test]
fn desktop_projections_filter_sort_and_project_real_library_and_inspector_data() {
    let view = sample_view();
    let extra_meta = view.catalog.metadata.clone();
    let mut state = DesktopState::default();
    let session = SessionSnapshot {
        running: true,
        paused: true,
        game: "/roms/snes/Aurora Drift.sfc".into(),
        console: "snes".into(),
        fps_words: "60.0 fps".into(),
        status: "Paused".into(),
        slot: 2,
        speed: 100,
        master_cheats: true,
        card_facts: String::new(),
    };

    let state_counts = BTreeMap::from([("/roms/snes/Brass Lantern.sfc".to_owned(), 2usize)]);
    let proj = project_desktop(&view, &extra_meta, &state, &session, &state_counts, None, "Eclipse", 1);
    let games: Value = serde_json::from_str(&proj.games_json).unwrap();
    let consoles: Value = serde_json::from_str(&proj.consoles_json).unwrap();
    let inspector: Value = serde_json::from_str(&proj.inspector_json).unwrap();
    assert_eq!(games["count"], 3);
    assert_eq!(games["filterCounts"]["resume"], 2, "running Aurora Drift + saved Brass Lantern");
    assert_eq!(consoles["activeHeading"], "All Games");
    assert_eq!(consoles["consoles"].as_array().unwrap().len(), 2, "active scope shows consoles with games");
    assert_eq!(consoles["places"][0]["key"], "continue", "running game inserts Continue entry");
    assert_eq!(inspector["hasGame"], true);
    assert_eq!(inspector["title"], "Aurora Drift");
    assert_eq!(inspector["running"], true);
    assert_eq!(inspector["coreLine"], "Venus (EmuSen native)");

    state.filter_key = "favourites".into();
    let fav_proj = project_desktop(&view, &extra_meta, &state, &session, &state_counts, None, "Eclipse", 1);
    let fav_games: Value = serde_json::from_str(&fav_proj.games_json).unwrap();
    assert_eq!(fav_games["count"], 1);
    assert_eq!(fav_games["rows"][0]["title"], "Aurora Drift");

    state.filter_key = "unplayed".into();
    let unplayed_proj = project_desktop(&view, &extra_meta, &state, &session, &state_counts, None, "Eclipse", 1);
    let unplayed_games: Value = serde_json::from_str(&unplayed_proj.games_json).unwrap();
    assert_eq!(unplayed_games["count"], 2);

    state.filter_key = "all".into();
    state.console_key = "snes".into();
    state.search_query = "brass".into();
    let cached = CachedInspector {
        path: "/roms/snes/Brass Lantern.sfc".into(),
        facts: GameFacts { path: "/roms/snes/Brass Lantern.sfc".into(), states: 2, players: "1 player".into(), ..GameFacts::default() },
        size_bytes: Some(524_288),
        cheats_count: 3,
        states: vec![emusen_qt_models::game_page::State {
            path: "/saves/Brass Lantern.state".into(),
            slot: emusen_qt_models::game_page::Slot::Auto,
            picture: String::new(),
            when: 1_700_000_000,
        }],
    };
    let search_proj = project_desktop(&view, &extra_meta, &state, &session, &state_counts, Some(&cached), "Dawn", 0);
    let search_games: Value = serde_json::from_str(&search_proj.games_json).unwrap();
    let search_inspector: Value = serde_json::from_str(&search_proj.inspector_json).unwrap();
    assert_eq!(search_games["count"], 1);
    assert_eq!(search_games["rows"][0]["title"], "Brass Lantern");
    assert_eq!(search_inspector["sizeWords"], "512 KB");
    assert_eq!(search_inspector["cheatsWords"], "3 codes");
    assert_eq!(search_inspector["states"][0]["label"], "Resume (Auto)");
    assert_eq!(search_proj.mode_badge, "DAWN");

    let chrome: Value = serde_json::from_str(&chrome_json()).unwrap();
    assert_eq!(chrome["navTabs"].as_array().unwrap().len(), 3);
    assert_eq!(chrome["bigPictureHint"], "F10");
}
