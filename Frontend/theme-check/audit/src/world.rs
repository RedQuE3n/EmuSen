//! The synthetic library a theme is audited on (man theme-check): eight games with made-up titles, two on each of four
//! consoles, all played three hours apart, two hearted, read through the real clients as the program reads them; and the
//! same library with 120 consoles (`crowd`), the one Choose a console is held to as well.

use emusen_client_fixture::{library, World};
use emusen_crystal_client::Config;
use emusen_platform_client::galaxia::consts::{FILE_GAMES, MODEL_APP_SETTINGS};
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt_models::clients::ClientSource;
use emusen_qt_models::library::{refresh, Kind, LibrarySource, LibraryView};
use emusen_qt_models::words;
use crate::crowd::Crowd;
use emusen_theme_qt::device::DeviceFacts;
use emusen_theme_qt::session::SessionFacts;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

/// Each game's title and an extension only its console's table entry lists, so the console follows from the file alone; four
/// consoles, since the test world shelved none of the Mega Drive's, PlayStation's, PlayStation 2's or Dreamcast's (measured on 2026-11-08).
const GAMES: [(&str, &str); 8] = [
    ("Aurora Drift", "sfc"),
    ("Brass Lantern", "nes"),
    ("Cobalt Harbor", "z64"),
    ("Dune Relay", "gb"),
    ("Ember Circuit", "sfc"),
    ("Fable of Tiles", "nes"),
    ("Granite Choir", "z64"),
    ("Hollow Comet", "gb"),
];

/// The hearted games, by title.
const HEARTED: [&str; 2] = ["Brass Lantern", "Fable of Tiles"];

/// The world, the library's view as the program's model holds it, and what the host feeds a theme beside it.
pub struct Synthetic {
    pub world: World,
    pub view: LibraryView,
    pub crowd: LibraryView,
    /// The crowd's consoles by the names a tile shows.
    pub crowd_names: Vec<String>,
    pub home_lines: String,
    pub session: SessionFacts,
    pub device: DeviceFacts,
}

fn rom(world: &World, title: &str, ext: &str) -> PathBuf {
    world.dir("Roms").join(format!("{title} (Synthetic).{ext}"))
}

fn records(platform: &Arc<Platform>) -> Result<Library, String> {
    let sqlite = Lent::open("libsqlite3.so.0").map_err(|e| format!("SQLite: {e}"))?;
    let file = platform.file(FILE_GAMES).map_err(|e| format!("the games' file: {e}"))?;
    Library::open(platform, &file, &sqlite).map_err(|e| format!("the games' records: {e}"))
}

/// The library made in a scratch tree and read; the error names the step that failed.
pub fn make() -> Result<Synthetic, String> {
    let world = World::new("theme-audit");
    for (title, ext) in GAMES {
        std::fs::write(rom(&world, title, ext), [0u8; 4096]).map_err(|e| format!("a synthetic game: {e}"))?;
    }
    let platform = Platform::load(library()).map_err(|e| format!("the platform library: {e}"))?;
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).map_err(|e| format!("the settings: {e}"))?;
    settings["RomDirectory"] = json!(world.root.join("Roms").to_string_lossy());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).map_err(|e| format!("the settings: {e}"))?;
    let path = library().to_string_lossy().into_owned();
    let none = Config { development: false, cores: Some(Vec::new()), player_cores: Some(Vec::new()) };
    let source = ClientSource::open(&path, &none).map_err(|e| format!("the clients: {e}"))?;
    source.games().map_err(|e| format!("the games: {e}"))?;
    let now = words::now();
    let games = records(&platform)?;
    for (i, (title, ext)) in GAMES.iter().enumerate() {
        let at = words::utc(now - 3 * 3600 * (i as i64 + 1));
        let path = rom(&world, title, ext).to_string_lossy().into_owned();
        games.change(&json!({ "op": "started", "path": path, "at": at })).map_err(|e| format!("a game played: {e}"))?;
        if HEARTED.contains(title) {
            games.change(&json!({ "op": "toggle_favourite", "path": path })).map_err(|e| format!("a heart: {e}"))?;
        }
    }
    let mut view = LibraryView::empty();
    view.apply(refresh(&source));
    if !view.problem.is_empty() {
        return Err(format!("the library: {}", view.problem));
    }
    let mut crowd = LibraryView::empty();
    crowd.apply(refresh(&Crowd::new(&source, &world.root)?));
    if !crowd.problem.is_empty() {
        return Err(format!("the library of 120 consoles: {}", crowd.problem));
    }
    let crowd_names = crowd.rows.rows().iter().filter(|r| r.kind == Some(Kind::Console)).map(|r| r.name.clone()).collect();
    let settings_line = words::settings_line(true, &view.catalog.settings);
    let lines: Vec<serde_json::Value> = words::home_lines(view.catalog.games.len(), &serde_json::Value::Null, &settings_line)
        .into_iter()
        .map(|(status, attention)| json!({ "status": status, "attention": attention }))
        .collect();
    let key = rom(&world, "Fable of Tiles", "nes").to_string_lossy().into_owned();
    let found = view.game(&key).ok_or("the suspended game: not in the library")?;
    let session = SessionFacts {
        has_game: true,
        title: found.title.clone(),
        console_id: found.system.clone(),
        console: found.console.clone(),
        hue: found.hue,
        key,
        paused_since: now - 720,
        played: 5400,
    };
    let device = DeviceFacts { controllers: 1, clock: "21:42".into(), battery: 64, ..DeviceFacts::default() };
    Ok(Synthetic { world, view, crowd, crowd_names, home_lines: serde_json::Value::Array(lines).to_string(), session, device })
}
