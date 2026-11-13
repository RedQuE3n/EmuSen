//! The synthetic library a theme is audited on (man theme-check): eight games with made-up titles, two on each of four
//! consoles, all played three hours apart, two hearted, read through the real clients as the program reads them; and the
//! same library with 120 consoles (`crowd`), the one Choose a console is held to as well; each game has a cover in an ES-DE media
//! folder, and the states' shapes of it (man theme-check, STATES) are read from the same world.

use emusen_client_fixture::{library, World};
use emusen_crystal_client::Config;
use emusen_platform_client::galaxia::consts::{FILE_GAMES, MODEL_APP_SETTINGS};
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt_models::clients::ClientSource;
use emusen_qt_models::library::{refresh, Kind, LibrarySource, LibraryView};
use emusen_qt_models::words;
use crate::catalogue::{Catalogue, Entry};
use crate::crowd::Crowd;
use crate::shape::Shaped;
use crate::states::Library as Shape;
use emusen_qt_test::Harness;
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

/// The game suspended, unless a state says otherwise.
pub const SUSPENDED: &str = "Fable of Tiles";

/// The state long's game, played last: a title too long for any place a design gives one.
pub const LONG: &str = "The Lighthouse Keeper's Unfinished Chronicle of the Seventh Northern Winter";
const LONG_EXT: &str = "sfc";

/// The covers' size in pixels, three by four, which each design crops to its console's shape.
const COVER: (u32, u32) = (450, 600);

/// The world, the library's view as the program's model holds it, and what the host feeds a theme beside it.
pub struct Synthetic {
    pub world: World,
    /// The clients the states' shapes are read through.
    source: ClientSource,
    /// The eight games' files and the long one's.
    eight: Vec<String>,
    long: String,
    /// The folder the covers are in, which the host lets a theme show.
    pub media: PathBuf,
    pub view: LibraryView,
    pub crowd: LibraryView,
    /// The crowd's consoles by the names a tile shows.
    pub crowd_names: Vec<String>,
    /// The library with a catalogue's consoles in place of its own, and their names, when one is given.
    pub catalogue: Option<(LibraryView, Vec<String>)>,
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

/// The library made in a scratch tree and read, and over `entries` when given; the error names the step that failed. The covers
/// are drawn by `harness` before the theme's scene is loaded.
pub fn make(harness: &mut Harness, entries: Option<&[Entry]>) -> Result<Synthetic, String> {
    let world = World::new("theme-audit");
    let all: Vec<(&str, &str)> = GAMES.into_iter().chain([(LONG, LONG_EXT)]).collect();
    for (title, ext) in &all {
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
    // The long title is played last, so it leads every recent list it is in.
    for (i, (title, ext)) in all.iter().enumerate() {
        let at = if *title == LONG { now - 3600 } else { now - 3 * 3600 * (i as i64 + 1) };
        let path = rom(&world, title, ext).to_string_lossy().into_owned();
        games.change(&json!({ "op": "started", "path": path, "at": words::utc(at) })).map_err(|e| format!("a game played: {e}"))?;
        if HEARTED.contains(title) {
            games.change(&json!({ "op": "toggle_favourite", "path": path })).map_err(|e| format!("a heart: {e}"))?;
        }
    }
    // Each game's cover in an ES-DE media folder under its console's id, which a first reading names.
    let mut first = LibraryView::empty();
    first.apply(refresh(&source));
    let media = world.dir("Media");
    let mut covers = Vec::new();
    for (i, (title, ext)) in all.iter().enumerate() {
        let key = rom(&world, title, ext).to_string_lossy().into_owned();
        let system = first.game(&key).map(|g| g.system).ok_or(format!("{title}: not in the library"))?;
        covers.push((media.join(&system).join("covers").join(format!("{title} (Synthetic).png")), COVER, i, format!("{system} covers\n{title}")));
    }
    // A folder of its own: Qt caches a folder's listing once it loads a file from it, and the audit's scene comes later.
    emusen_qt_test::covers::draw(harness, &world.dir("Covers"), &covers, |_| {});
    settings["EsdeMediaDirectory"] = json!(media.to_string_lossy());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).map_err(|e| format!("the settings: {e}"))?;
    // The scraper reads the media folder when it opens, so the clients are opened again once the covers are in it.
    drop(first);
    drop(source);
    let source = ClientSource::open(&path, &none).map_err(|e| format!("the clients: {e}"))?;
    let eight: Vec<String> = GAMES.iter().map(|(t, e)| rom(&world, t, e).to_string_lossy().into_owned()).collect();
    let long = rom(&world, LONG, LONG_EXT).to_string_lossy().into_owned();
    let mut synthetic = Synthetic {
        world,
        source,
        eight,
        long,
        media,
        view: LibraryView::empty(),
        crowd: LibraryView::empty(),
        crowd_names: Vec::new(),
        catalogue: None,
        device: DeviceFacts { controllers: 1, clock: "21:42".into(), battery: 64, ..DeviceFacts::default() },
    };
    synthetic.view = synthetic.shaped(Shape::Synthetic)?;
    let names = |view: &LibraryView| -> Vec<String> {
        view.rows.rows().iter().filter(|r| r.kind == Some(Kind::Console)).map(|r| r.name.clone()).collect()
    };
    let default = synthetic.default_source();
    let mut crowd = LibraryView::empty();
    crowd.apply(refresh(&Crowd::new(&default, &synthetic.world.root)?));
    if !crowd.problem.is_empty() {
        return Err(format!("the library of 120 consoles: {}", crowd.problem));
    }
    let crowd_names = names(&crowd);
    let catalogue = match entries {
        None => None,
        Some(entries) => {
            let listed = synthetic.listed(entries)?;
            let shown = names(&listed);
            Some((listed, shown))
        }
    };
    drop(default);
    (synthetic.crowd, synthetic.crowd_names, synthetic.catalogue) = (crowd, crowd_names, catalogue);
    Ok(synthetic)
}

impl Synthetic {
    /// The eight games with their covers and plays: the default library, which the crowd and a catalogue are read over.
    fn default_source(&self) -> Shaped<'_> {
        Shaped { inner: &self.source, keep: self.eight.clone(), covers: true, played: true }
    }

    /// The library over a catalogue's consoles in place of its own.
    fn listed(&self, entries: &[Entry]) -> Result<LibraryView, String> {
        let default = self.default_source();
        let mut listed = LibraryView::empty();
        listed.apply(refresh(&Catalogue::new(&default, &self.world.root, entries)?));
        if !listed.problem.is_empty() {
            return Err(format!("the catalogue's library: {}", listed.problem));
        }
        Ok(listed)
    }

    /// The library a state is fed, read afresh through the clients.
    pub fn shaped(&self, shape: Shape) -> Result<LibraryView, String> {
        let mut source = self.default_source();
        match shape {
            Shape::Catalogue(name, text) => {
                let entries = crate::catalogue::parse(text).map_err(|e| format!("the catalogue {name}: {e}"))?;
                return self.listed(&entries);
            }
            Shape::Synthetic => {}
            Shape::NoCovers => source.covers = false,
            Shape::Unplayed => source.played = false,
            Shape::Long => source.keep.push(self.long.clone()),
            Shape::Loud => {}
            Shape::OneConsole => {
                let suspended = self.key_of(SUSPENDED).unwrap_or_default();
                let ext = suspended.rsplit('.').next().unwrap_or_default().to_owned();
                source.keep.retain(|k| k.ends_with(&format!(".{ext}")));
            }
        }
        let mut view = LibraryView::empty();
        view.apply(refresh(&source));
        if shape == Shape::Loud {
            view.catalog.settings["BigPictureInterface"]["NavigationVolume"] = json!(100);
            view.catalog.settings["NavigationSounds"] = json!(true);
        }
        if view.problem.is_empty() { Ok(view) } else { Err(format!("the library: {}", view.problem)) }
    }

    /// A game's file by its title.
    fn key_of(&self, title: &str) -> Option<String> {
        self.eight.iter().chain([&self.long]).find(|k| k.contains(&format!("/{title} (Synthetic)."))).cloned()
    }

    /// Home's lines over a view in the dark or the light variant, as the program's host makes them.
    pub fn home_lines(&self, view: &LibraryView, dark: bool) -> String {
        let settings_line = words::settings_line(dark, &view.catalog.settings);
        let lines: Vec<serde_json::Value> = words::home_lines(view.catalog.games.len(), &serde_json::Value::Null, &settings_line)
            .into_iter()
            .map(|(status, attention)| json!({ "status": status, "attention": attention }))
            .collect();
        serde_json::Value::Array(lines).to_string()
    }

    /// The session with `title` suspended for 12 minutes after 90 of play, or with no game; the game must be in the view.
    pub fn session(&self, view: &LibraryView, title: Option<&str>) -> Result<SessionFacts, String> {
        let Some(title) = title else { return Ok(SessionFacts::default()) };
        let key = self.key_of(title).ok_or(format!("the suspended game {title}: not a synthetic game"))?;
        let found = view.game(&key).ok_or(format!("the suspended game {title}: not in the library"))?;
        Ok(SessionFacts {
            has_game: true,
            title: found.title.clone(),
            console_id: found.system.clone(),
            console: found.console.clone(),
            hue: found.hue,
            key,
            paused_since: words::now() - 720,
            played: 5400,
        })
    }
}

