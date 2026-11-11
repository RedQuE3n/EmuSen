//! The library as an ES-DE theme sees it, with a fake client and no window (EmuSen_Qt.md §4.6): the shelves' text and
//! its fields, the media index and presence, the settings, the theme's screen among the others, and the GUI thread's
//! part of a refresh at a library's scale.

use emusen_crystal_client::shapes::SystemRow;
use emusen_platform_client::library::GameFile;
use emusen_qt_models::esde_shelves::{metadata_of, names_of, Metadata, settings_of, theme_key, MediaIndex, Plays};
use cxx_qt_lib::QString;
use emusen_qt_models::esde_texts::{take, EsdeTexts};
use emusen_qt_models::library::{refresh, Engine, FirstLook, LibrarySource, LibraryView};
use emusen_qt_models::screens::{step, step_in, Ask, Shown};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A source answering what a test sets, counting the metadata it is asked for.
#[derive(Default)]
struct Fake {
    systems: Vec<SystemRow>,
    games: Vec<GameFile>,
    favourites: Vec<String>,
    played: Vec<(String, i64)>,
    plays: Vec<(String, Plays)>,
    metadata: Vec<(String, Value)>,
    covers: Vec<(String, String)>,
    index: MediaIndex,
    settings: Value,
    asked: AtomicUsize,
}

impl LibrarySource for Fake {
    fn systems(&self) -> Result<Vec<SystemRow>, String> {
        Ok(self.systems.clone())
    }
    fn games(&self) -> Result<Vec<GameFile>, String> {
        Ok(self.games.clone())
    }
    fn favourites(&self, _: &[GameFile]) -> Result<Vec<String>, String> {
        Ok(self.favourites.clone())
    }
    fn recent(&self, _: &[GameFile]) -> Result<Vec<String>, String> {
        Ok(self.played.iter().map(|(p, _)| p.clone()).collect())
    }
    fn engines(&self, _: &[SystemRow]) -> Result<Vec<(String, Engine)>, String> {
        Ok(Vec::new())
    }
    fn media(&self, _: &[SystemRow], games: &[GameFile], types: &[&str]) -> Result<Vec<(String, String)>, String> {
        let wanted = types.contains(&"cover");
        Ok(self.covers.iter().filter(|(p, _)| wanted && games.iter().any(|g| &g.path == p)).cloned().collect())
    }
    fn played(&self, _: &[GameFile]) -> Result<Vec<(String, i64)>, String> {
        Ok(self.played.clone())
    }
    fn shelves(&self) -> Result<Vec<String>, String> {
        Ok(vec!["snes".into(), "nes".into()])
    }
    fn settings(&self) -> Result<Value, String> {
        Ok(self.settings.clone())
    }
    fn metadata(&self, games: &[GameFile]) -> Result<Vec<(String, Metadata)>, String> {
        self.asked.fetch_add(games.len(), Ordering::Relaxed);
        Ok(self.metadata.iter().filter(|(p, _)| games.iter().any(|g| &g.path == p)).map(|(p, m)| (p.clone(), metadata_of(m))).collect())
    }
    fn plays(&self, _: &[GameFile]) -> Result<Vec<(String, Plays)>, String> {
        Ok(self.plays.clone())
    }
    fn media_index(&self, _: &[String]) -> Result<MediaIndex, String> {
        Ok(self.index.clone())
    }
}

fn system(id: &str, esde: &str, full: &str) -> SystemRow {
    SystemRow { id: id.into(), name: full.into(), esde_system: esde.into(), esde_full_name: full.into(), engines: vec!["x".into()], ..SystemRow::default() }
}

fn game(path: &str, system: &str) -> GameFile {
    let folder = path.strip_prefix(&format!("/roms/{system}/")).and_then(|rest| rest.rsplit_once('/')).map_or("", |(f, _)| f).to_owned();
    GameFile { path: path.into(), system: Some(system.into()), package: None, folder }
}

/// The scraper's `metadata` answer for the fields given, each as an edit.
fn metadata(fields: &[(&str, &str)]) -> Value {
    let mut out = serde_json::Map::new();
    for (k, v) in fields {
        out.insert((*k).into(), json!({ "value": v, "source": "edited" }));
    }
    Value::Object(out)
}

fn themed() -> Value {
    json!({
        "LibraryStyle": "Theme",
        "BigPictureTheme": "/themes/art-book-next/",
        "BigPicture": { "/themes/art-book-next": { "Variant": "textlist", "ColorScheme": "dark" } },
        "BigPictureInterface": { "DisplayClock": true, "StartupView": "gamelist" },
        "ShowHiddenGames": false,
    })
}

/// Two systems, a game in a folder, a favourite with its metadata and plays, and a game never played with none.
fn fake() -> Fake {
    let mut index = MediaIndex::default();
    index.insert("snes", "screenshot", "Brass Lantern (Synthetic)", "/media/snes/screenshots/Brass Lantern (Synthetic).png");
    index.insert("snes", "manual", "Aurora Drift (Synthetic)", "/media/snes/manuals/Aurora Drift (Synthetic).pdf");
    index.insert("snes", "manual", "Quartz Spire (Synthetic)", "/media/snes/manuals/Tower Set/Quartz Spire (Synthetic).pdf");
    index.insert("snes", "marquee", "Quartz Spire (Synthetic)", "/media/snes/marquees/Tower Set/Quartz Spire (Synthetic).png");
    Fake {
        systems: vec![system("nes", "nes", "Nintendo Entertainment System"), system("snes", "snes", "Super Nintendo Entertainment System"), system("gb", "gb", "Game Boy")],
        games: vec![
            game("/roms/snes/Aurora Drift (Synthetic).sfc", "snes"),
            game("/roms/snes/Brass Lantern (Synthetic).sfc", "snes"),
            game("/roms/snes/Tower Set/Quartz Spire (Synthetic).sfc", "snes"),
            game("/roms/nes/Cobalt Harbor (Synthetic).nes", "nes"),
        ],
        favourites: vec!["/roms/snes/Aurora Drift (Synthetic).sfc".into()],
        played: vec![("/roms/snes/Aurora Drift (Synthetic).sfc".into(), 1_790_000_000)],
        plays: vec![("/roms/snes/Aurora Drift (Synthetic).sfc".into(), Plays { count: 3, seconds: 5400 })],
        metadata: vec![(
            "/roms/snes/Aurora Drift (Synthetic).sfc".into(),
            metadata(&[
                ("name", "Aurora Drift"),
                ("sortname", "Drift, Aurora"),
                ("description", "A game invented for EmuSen's tests."),
                ("developer", "Studio A"),
                ("rating", "0.7"),
                ("releasedate", "1990-01-01"),
                ("completed", "1"),
                ("kidgame", "0"),
                ("hidemetadata", "1"),
                ("controller", "gamepad_nintendo_snes"),
                ("altemulator", "snes9x"),
            ]),
        )],
        covers: vec![("/roms/nes/Cobalt Harbor (Synthetic).nes".into(), "/art/nes/Cobalt Harbor.png".into())],
        index,
        settings: themed(),
        ..Fake::default()
    }
}

fn shelves(text: &str) -> Vec<Value> {
    serde_json::from_str::<Value>(text).unwrap().as_array().unwrap().clone()
}

#[test]
fn each_shelf_carries_its_es_de_names_and_its_games_every_field() {
    let read = refresh(&fake()).unwrap();
    let shelves = shelves(&read.esde.shelves);
    let seen: Vec<(&str, &str, &str, &str, &str)> = shelves
        .iter()
        .map(|s| (s["name"].as_str().unwrap(), s["system"].as_str().unwrap(), s["full_name"].as_str().unwrap(), s["kind"].as_str().unwrap(), s["theme"].as_str().unwrap()))
        .collect();
    assert_eq!(
        seen,
        [
            ("snes", "snes", "Super Nintendo Entertainment System", "regular", "snes"),
            ("nes", "nes", "Nintendo Entertainment System", "regular", "nes"),
            ("all", "", "all games", "auto", "auto-allgames"),
            ("favorites", "", "favorites", "auto", "auto-favorites"),
            ("recent", "", "last played", "auto", "auto-lastplayed"),
        ],
        "the shelves' order, the Game Boy left out with no games, then the automatic collections"
    );
    let names = |s: &Value| s["games"].as_array().unwrap().iter().map(|g| g["name"].as_str().unwrap().to_owned()).collect::<Vec<_>>();
    assert_eq!(names(&shelves[0]), ["Tower Set", "Aurora Drift", "Brass Lantern (Synthetic)"], "the folder on top, then the favourite, then by name");
    let aurora = &shelves[0]["games"][1];
    let expected = json!({
        "file": "/roms/snes/Aurora Drift (Synthetic).sfc", "name": "Aurora Drift", "esde_system": "snes", "esde_full_name": "Super Nintendo Entertainment System",
        "sort_name": "Drift, Aurora", "custom_sort_name": null,
        "developer": "Studio A", "publisher": null, "genre": null, "players": null,
        "release_date": "1990-01-01", "rating": 0.7, "last_played": 1_790_000_000, "play_count": 3, "play_time": 5400, "favourite": true,
        "completed": true, "kid_game": false, "broken": false, "hidden": false, "not_counted": false, "hide_metadata": true,
        "controller": "gamepad_nintendo_snes", "alt_emulator": true, "emulator": "snes9x", "manual": true, "folder": false,
        "folder_path": "", "folder_link": null, "is_collection": false, "face": null,
    });
    assert_eq!(aurora, &expected, "every field but the description, given on demand");
    assert_eq!(read.esde_state.description_of("/roms/snes/Aurora Drift (Synthetic).sfc"), Some("A game invented for EmuSen's tests."));
    let never = &shelves[0]["games"][2];
    assert_eq!(
        (never["name"].as_str(), never["last_played"].as_i64(), never["play_count"].as_i64(), never["favourite"].as_bool(), never["rating"].is_null(), never["manual"].as_bool()),
        (Some("Brass Lantern (Synthetic)"), Some(0), Some(0), Some(false), true, Some(false)),
        "a game with no metadata is named by its file, never played and no favourite"
    );
    let refs: Vec<&str> = shelves[2]["games"].as_array().unwrap().iter().map(|g| g["file"].as_str().unwrap()).collect();
    assert_eq!(refs.len(), 4, "a collection lists its games by file, each row once in its console's shelf");
    assert!(shelves[2]["games"][0].get("name").is_none());
    let quartz = read.esde_state.shelf("snes").unwrap().games.iter().find(|g| g.name == "Quartz Spire (Synthetic)").unwrap();
    assert_eq!((quartz.folder_path.as_str(), quartz.manual, quartz.favourite), ("Tower Set", true, false), "a folder, a manual, no favourite");
}

#[test]
fn the_media_index_and_presence_are_written_beside_the_shelves_with_the_scraper_s_cover() {
    let read = refresh(&fake()).unwrap();
    let media: Value = serde_json::from_str(&read.esde.media).unwrap();
    assert_eq!(media["nes"]["cover"]["Cobalt Harbor (Synthetic)"], "/art/nes/Cobalt Harbor.png", "the cover the scraper's order chose");
    assert_eq!(media["snes"]["screenshot"]["Brass Lantern (Synthetic)"], "/media/snes/screenshots/Brass Lantern (Synthetic).png");
    let presence: Value = serde_json::from_str(&read.esde.presence).unwrap();
    assert_eq!(presence["snes"], json!(["manual", "marquee", "screenshot"]), "the variant triggers per shelf");
    assert_eq!(presence["nes"], json!(["cover"]));
    assert_eq!(presence["all"], json!(["cover", "manual", "marquee", "screenshot"]), "a collection's, each game under its own console");
}

#[test]
fn the_first_look_lists_every_shelf_and_game_reading_no_metadata() {
    let source = fake();
    let first = refresh(&FirstLook(&source)).unwrap();
    assert_eq!(source.asked.load(Ordering::Relaxed), 0, "no game's metadata is read for the first look");
    let whole = refresh(&source).unwrap();
    let files = |text: &str| -> Vec<(String, Vec<String>)> {
        shelves(text).iter().map(|s| (s["name"].to_string(), s["games"].as_array().unwrap().iter().map(|g| g["file"].to_string()).collect())).collect()
    };
    let mut a = files(&first.esde.shelves);
    let mut b = files(&whole.esde.shelves);
    a.iter_mut().chain(b.iter_mut()).for_each(|(_, g)| g.sort());
    assert_eq!(a, b, "the same shelves and games, which the whole read names and sorts by their metadata");
}

#[test]
fn the_own_look_asks_for_no_metadata_and_writes_empty_texts() {
    let mut source = fake();
    source.settings["LibraryStyle"] = json!("Classic");
    let read = refresh(&source).unwrap();
    assert!(!read.esde_settings.themed);
    assert_eq!((read.esde.shelves.as_str(), read.esde.media.as_str(), read.esde.presence.as_str()), ("[]", "{}", "{}"));
    assert_eq!(source.asked.load(Ordering::Relaxed), 0, "no game's metadata is read for a view not shown");
    source.settings["LibraryStyle"] = json!("Theme");
    source.settings["BigPictureTheme"] = json!(" ");
    assert!(!refresh(&source).unwrap().esde_settings.themed, "a theme style with no theme named is the own look");
}

#[test]
fn the_settings_are_the_theme_s_stored_choices_and_the_interface_s() {
    let settings = settings_of(&themed());
    assert!(settings.themed);
    let text: Value = serde_json::from_str(&settings.text).unwrap();
    assert_eq!(text["choices"], json!({ "Variant": "textlist", "ColorScheme": "dark" }), "keyed by the folder less its trailing separator");
    assert_eq!(text["interface"]["StartupView"], "gamelist");
    assert_eq!((text["style"].as_str(), text["show_hidden"].as_bool()), (Some("Theme"), Some(false)));
    assert_eq!(theme_key("/t/x//"), "/t/x");
    assert_eq!(theme_key(""), "");
    let none = settings_of(&Value::Null);
    assert!(!none.themed);
    assert_eq!(serde_json::from_str::<Value>(&none.text).unwrap()["choices"], json!({}));
}

#[test]
fn the_index_walks_each_root_s_type_folders_the_first_root_winning() {
    let root = std::env::temp_dir().join(format!("esde-index-{}", std::process::id()));
    let (store, esde) = (root.join("store"), root.join("esde"));
    for (dir, file) in [
        (&store, "snes/covers/Aurora Drift.png"),
        (&esde, "snes/covers/Aurora Drift.png"),
        (&esde, "snes/covers/Brass Lantern.jpg"),
        (&esde, "snes/screenshots/Tower Set/Quartz Spire.png"),
        (&esde, "nes/covers/Cobalt Harbor.png"),
    ] {
        let path = dir.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"").unwrap();
    }
    let mut index = MediaIndex::default();
    for r in [&store, &esde] {
        index.walk(r, "snes", "cover", "covers");
        index.walk(r, "snes", "screenshot", "screenshots");
    }
    let at = |p: &std::path::Path, f: &str| p.join(f).to_string_lossy().into_owned();
    assert_eq!(index.find("snes", "cover", &names_of("/r/snes/Aurora Drift.sfc", "")), Some(at(&store, "snes/covers/Aurora Drift.png").as_str()), "the store's first");
    assert_eq!(index.find("snes", "cover", &names_of("/r/snes/Brass Lantern.sfc", "")), Some(at(&esde, "snes/covers/Brass Lantern.jpg").as_str()));
    assert_eq!(index.find("snes", "screenshot", &names_of("/r/snes/Tower Set/Quartz Spire.sfc", "Tower Set")), Some(at(&esde, "snes/screenshots/Tower Set/Quartz Spire.png").as_str()), "a folder's game by its folder and stem");
    assert_eq!(index.find("nes", "cover", &names_of("/r/nes/Cobalt Harbor.nes", "")), None, "only the shelves asked for are walked");
    let names = ["Brass Lantern".to_owned()].into_iter().collect();
    assert_eq!(index.present("snes", &names), ["cover"]);
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn the_theme_s_view_stands_for_the_library_s_screens() {
    let home = Some(emusen_qt_models::home_card::FIRST.to_owned());
    let table: [(Shown, Ask, Option<(Shown, Option<String>)>, &str); 16] = [
        (Shown::Home, Ask::Restyled, Some((Shown::Esde, None)), "Moonrise gives way to the theme's view"),
        (Shown::Grid, Ask::Restyled, Some((Shown::Esde, None)), "a library screen of the own look"),
        (Shown::Game, Ask::Restyled, None, "the game stays"),
        (Shown::Esde, Ask::Act("home".into()), Some((Shown::HomeCard, home)), "Start: the Home card at its first page"),
        (Shown::HomeCard, Ask::Close, Some((Shown::Esde, None)), "the card closes to the view"),
        (Shown::HomeCard, Ask::Back, Some((Shown::Esde, None)), ""),
        (Shown::HomeCard, Ask::Act("search".into()), Some((Shown::Esde, None)), "the card closes on its action"),
        (Shown::Esde, Ask::Back, None, "B at the system view does nothing (decided 2026-11-08)"),
        (Shown::Esde, Ask::Act("list".into()), Some((Shown::List, None)), "Select: the List card, ES-DE's options"),
        (Shown::List, Ask::Back, Some((Shown::Esde, None)), ""),
        (Shown::List, Ask::Act("sort".into()), Some((Shown::Esde, None)), "the List card closes on its action"),
        (Shown::Esde, Ask::Play, Some((Shown::Game, None)), "a game played"),
        (Shown::Game, Ask::Menu, Some((Shown::Pause, None)), ""),
        (Shown::Pause, Ask::Library, Some((Shown::Esde, None)), "Back to Library"),
        (Shown::Game, Ask::Closed, Some((Shown::Esde, None)), "the game closed"),
        (Shown::Esde, Ask::Place(0), None, "Home's places are not the theme's"),
    ];
    for (shown, ask, after, why) in table {
        assert_eq!(step_in(true, shown, &ask), after, "{shown:?} {ask:?}: {why}");
    }
    assert_eq!(step_in(false, Shown::Esde, &Ask::Restyled), Some((Shown::Home, None)), "the own look again: Home");
    assert_eq!(step_in(false, Shown::Home, &Ask::Restyled), None);
    for (shown, ask) in [(Shown::Pause, Ask::Library), (Shown::HomeCard, Ask::Close), (Shown::Home, Ask::Place(0)), (Shown::Grid, Ask::Back)] {
        assert_eq!(step_in(false, shown, &ask), step(shown, &ask), "the own look's table unchanged");
    }
}

#[test]
fn the_gui_thread_s_part_of_a_refresh_stays_under_two_milliseconds_at_nine_thousand_games() {
    let mut source = fake();
    let description = "A game invented for EmuSen's tests. It has no story and no pictures of its own; every image shown for it is a flat colour with its name written on it. This second sentence is here so that the description is long enough to wrap.";
    source.games = (0..9000).map(|i| game(&format!("/roms/{}/Game {i}.x", if i % 3 == 0 { "nes" } else { "snes" }), if i % 3 == 0 { "nes" } else { "snes" })).collect();
    source.metadata = Vec::new();
    let paths: Vec<String> = source.games.iter().map(|g| g.path.clone()).collect();
    source.favourites = paths.iter().step_by(5).cloned().collect();
    let mut index = MediaIndex::default();
    for p in &paths {
        let system = if p.contains("/nes/") { "nes" } else { "snes" };
        for kind in ["cover", "screenshot", "marquee", "miximage"] {
            index.insert(system, kind, &names_of(p, "")[0], &format!("/media/{system}/{kind}s/{p}.png"));
        }
    }
    source.index = index;
    let fields = |p: &str| metadata(&[("name", p), ("description", description), ("developer", "Studio A"), ("publisher", "Publisher P"), ("genre", "Puzzle"), ("rating", "0.5"), ("releasedate", "1990-01-01")]);
    let all = source.games.clone();
    let source = Fake { metadata: all.iter().map(|g| (g.path.clone(), fields(&g.path))).collect(), ..source };
    let fast = MetadataByMap(source);
    let first = refresh(&fast).unwrap();
    assert!(first.esde.shelves.len() > 2_000_000, "a library's scale: {} bytes", first.esde.shelves.len());
    // The next refresh: one more favourite, so the shelves' text is replaced and the other three compare equal at length.
    let mut more = fast;
    more.0.favourites.push(paths[1].clone());
    let next = refresh(&more).unwrap();
    let mut best = std::time::Duration::MAX;
    for _ in 0..3 {
        // A view already holding the library, as the GUI thread holds it after the first read.
        let mut view = LibraryView::empty();
        let mut shown: [QString; 4] = Default::default();
        let texts = EsdeTexts::of(&first.esde_settings, &first.esde);
        let [a, b, c, d] = &mut shown;
        assert_eq!(take(&mut view, [a, b, c, d], Ok(first.clone()), Some(texts)), [true; 4]);
        let read = [&first.esde_settings.text, &first.esde.shelves, &first.esde.media, &first.esde.presence];
        for (i, (q, t)) in shown.iter().zip(read).enumerate() {
            assert!(q.to_string() == *t, "text {i} published in its own place");
        }
        // The QStrings are made on the reading thread (esde_texts.rs); the GUI thread's part is `take`, dropping what it replaces.
        let (refreshed, texts) = (Ok(next.clone()), Some(EsdeTexts::of(&next.esde_settings, &next.esde)));
        let made = texts.as_ref().map(|t| t.0[1].as_slice().as_ptr());
        let [a, b, c, d] = &mut shown;
        let started = std::time::Instant::now();
        let changed = take(&mut view, [a, b, c, d], refreshed, texts);
        best = best.min(started.elapsed());
        assert_eq!(changed, [false, true, false, false]);
        assert!(shown[1].to_string() == view.esde.shelves && view.esde.shelves == next.esde.shelves, "the shelves' new text moved in");
        assert_eq!(Some(shown[1].as_slice().as_ptr()), made, "the shelves' QString is the one the reading thread made, not one made again");
    }
    eprintln!("the GUI thread's part at 9,000 games: {best:?}");
    assert!(best < std::time::Duration::from_millis(2), "{best:?}");
}

/// The fake with its metadata found by a map, so the scale test measures the GUI thread's part, not a slow fake.
struct MetadataByMap(Fake);

impl LibrarySource for MetadataByMap {
    fn systems(&self) -> Result<Vec<SystemRow>, String> {
        self.0.systems()
    }
    fn games(&self) -> Result<Vec<GameFile>, String> {
        self.0.games()
    }
    fn favourites(&self, g: &[GameFile]) -> Result<Vec<String>, String> {
        self.0.favourites(g)
    }
    fn recent(&self, g: &[GameFile]) -> Result<Vec<String>, String> {
        self.0.recent(g)
    }
    fn engines(&self, s: &[SystemRow]) -> Result<Vec<(String, Engine)>, String> {
        self.0.engines(s)
    }
    fn shelves(&self) -> Result<Vec<String>, String> {
        self.0.shelves()
    }
    fn settings(&self) -> Result<Value, String> {
        self.0.settings()
    }
    fn metadata(&self, _: &[GameFile]) -> Result<Vec<(String, Metadata)>, String> {
        Ok(self.0.metadata.iter().map(|(p, m)| (p.clone(), metadata_of(m))).collect())
    }
    fn media_index(&self, s: &[String]) -> Result<MediaIndex, String> {
        self.0.media_index(s)
    }
}
