//! A game's page's source over the client crates (EmuSen_Qt.md §7.8): the metadata and pictures from the scraper's
//! sources-alone handle, the record and the edits from games.db, the states by Galaxia's save paths, the engines from
//! the runtime, the profiles from Endymion's store, the cheats from Galaxia's cheat file, the crop's and the television
//! standard's offer from Serenity's catalogue, the standard from games.db, the player's screenshots from Galaxia's
//! folder, the package from the library tool's `pkg-ls` and Galaxia's claim index, and its job (`pkg-make` or `pkg-check`)
//! started through the library tool; the crop's choices are Galaxia's Overscan row's. Nothing here needs a session.

use crate::clients::{app_settings, engine_shown, ClientSource};
use crate::display::Crop;
use crate::game_page::{Change, Choice, CropOffer, Detail, GamePageSource, Media, MediaKind, Package, PackageJob, Read, Slot, State, CHECK, DETAILS, MAKE};
use crate::library::{console_name, title_of, HERO};
use emusen_crystal_client::Scope;
use emusen_platform_client::bindings::consts::PROFILES;
use emusen_platform_client::galaxia::consts::{
    DIR_SCREENSHOTS, SAVE_PACKAGE_RESUME_STATE, SAVE_PACKAGE_STATE, SAVE_PICTURE, SAVE_RESUME_STATE, SAVE_STATE,
};
use emusen_platform_client::library::GameFile;
use emusen_platform_client::packages::Roots;
use emusen_platform_client::serenity;
use emusen_platform_client::tools::{Step, ToolJob};
use serde_json::{json, Value};
use std::path::Path;

fn words(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// The media store's types the Media section shows, in Mistress's order, the PDF manual last.
const MEDIA: [(&str, MediaKind); 6] = [
    ("screenshot", MediaKind::Screenshot),
    ("titlescreen", MediaKind::TitleScreen),
    ("video", MediaKind::Clip),
    ("marquee", MediaKind::Marquee),
    ("fanart", MediaKind::Fanart),
    ("manual", MediaKind::Manual),
];

/// A file's modified time in seconds since 1970; none for no file.
fn modified(path: &Path) -> Option<i64> {
    let at = std::fs::metadata(path).ok()?.modified().ok()?;
    Some(at.duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64))
}

impl ClientSource {
    fn media_file(&self, game: &GameFile, esde_system: &str, kind: &str) -> String {
        let Some(scraper) = self.scraper() else { return String::new() };
        let found = scraper.get(&json!({ "what": "media", "path": game.path, "system": esde_system, "type": kind }));
        found.ok().and_then(|f| f.get("path").and_then(Value::as_str).map(str::to_owned)).unwrap_or_default()
    }

    /// The states, slot by slot and then the automatic one, each where its file is, the most recent first.
    fn states(&self, game: &GameFile, settings: &Value) -> Result<Vec<State>, String> {
        let directory = settings.get("StateDirectory").and_then(Value::as_str).filter(|d| !d.trim().is_empty());
        let path_of = |slot: Slot| {
            let (n, slot_no) = match slot {
                Slot::Number(n) => (true, n as i32),
                Slot::Auto => (false, 0),
            };
            match &game.package {
                Some(folder) => self.platform.save_path(if n { SAVE_PACKAGE_STATE } else { SAVE_PACKAGE_RESUME_STATE }, folder, None, slot_no),
                None => self.platform.save_path(if n { SAVE_STATE } else { SAVE_RESUME_STATE }, &game.path, directory, slot_no),
            }
        };
        let slots = (1..=crate::pause::SLOTS).map(Slot::Number).chain([Slot::Auto]);
        let mut out = Vec::new();
        for slot in slots {
            let file = path_of(slot).map_err(words)?;
            let Some(when) = modified(&file) else { continue };
            let file = file.to_string_lossy().into_owned();
            let picture = self.platform.save_path(SAVE_PICTURE, &file, None, 0).map_err(words)?;
            let picture = if picture.exists() { picture.to_string_lossy().into_owned() } else { String::new() };
            out.push(State { path: file, slot, picture, when });
        }
        out.sort_by_key(|s| std::cmp::Reverse(s.when));
        Ok(out)
    }

    /// The player's own screenshots of the game, the newest first.
    fn screenshots(&self, game: &GameFile) -> Result<Vec<Media>, String> {
        let Some(folder) = self.platform.directory(DIR_SCREENSHOTS).map_err(words)? else { return Ok(Vec::new()) };
        Ok(crate::screenshots::of_game(&folder, &game.path).into_iter().map(|(path, _)| Media { path, kind: MediaKind::Taken }).collect())
    }

    /// The game's package and its layers: the package the scan found the game in, else one that claims the game but
    /// refused it, its file changed (Mistress's `GamePackageDetail`, `Refused`); each read by the library tool's `pkg-ls`.
    fn package(&self, game: &GameFile) -> Result<(Package, usize, Option<PackageJob>), String> {
        let (folder, refused) = match &game.package {
            Some(folder) => (folder.clone(), false),
            None => {
                let scan = self.scan_request()?;
                let library = scan.roots.first().cloned();
                // A game the library folder holds is made a package by its path there (the library tool's `pkg-make`).
                let make = library.as_deref().and_then(|root| std::path::Path::new(&game.path).strip_prefix(root).ok()).map(|rel| PackageJob {
                    key: MAKE,
                    request: json!({ "op": "pkg-make", "paths": [rel.to_string_lossy()] }),
                });
                let (Some(packages), Some(compressed)) = (scan.packages.as_deref(), scan.compressed.as_deref()) else { return Ok((Package::None, 0, make)) };
                let roots = Roots { packages, library: library.as_deref(), compressed };
                let index = self.platform.package_index(roots).map_err(words)?;
                let claims = |p: &&Value| p["claims"].as_array().is_some_and(|c| c.iter().any(|c| c.as_str() == Some(game.path.as_str())));
                match index["packages"].as_array().and_then(|all| all.iter().find(claims)).and_then(|p| p["folder"].as_str()) {
                    Some(folder) => (folder.to_owned(), true),
                    None => return Ok((Package::None, 0, make)),
                }
            }
        };
        let check = PackageJob { key: CHECK, request: json!({ "op": "pkg-check", "paths": [folder] }) };
        let job = ToolJob::start(&self.platform, &json!({ "op": "pkg-ls", "paths": [folder] })).map_err(words)?;
        let mut listed = Value::Null;
        loop {
            match job.next(1000).map_err(words)? {
                Step::Event(e) if e["event"] == "package" => listed = e,
                Step::Event(_) | Step::Waiting => {}
                Step::Ended => break,
            }
        }
        let layers = listed["patches"].as_array().map_or(0, Vec::len);
        // A package made since the scan claims the game too: it is the game's while pkg-ls finds the game as it recorded.
        let kind = match (refused && listed["state"] != "ok", listed["mode"].as_str()) {
            (true, _) => Package::Changed,
            (false, Some("self-contained")) => Package::SelfContained,
            (false, _) => Package::Linked,
        };
        Ok((kind, layers, Some(check)))
    }

    /// The game's crop as Display crop offers it: Galaxia's Overscan row's choices that hide a fixed part, each as
    /// Serenity's catalogue sizes it, and the game's own crop from games.db.
    fn crop(&self, game: &GameFile, catalogue: &Value) -> Result<CropOffer, String> {
        let rows = self.platform.settings_rows().map_err(words)?;
        let row = rows["GraphicsConfig.Consoles"].as_array().and_then(|all| all.iter().find(|r| r["key"] == "Overscan")).cloned().unwrap_or(Value::Null);
        let texts = |k: &str| row[k].as_array().into_iter().flatten().map(|v| v.as_str().unwrap_or_default().to_owned()).collect::<Vec<_>>();
        let label = row["label"].as_str().unwrap_or_default().to_owned();
        let mut offer = CropOffer { label, most: crate::display::most(catalogue), ..CropOffer::default() };
        for (value, label) in texts("choices").into_iter().zip(texts("choice_labels")) {
            match value.as_str() {
                crate::display::CUSTOM => offer.custom = label,
                _ => offer.choices.push((label, crate::display::overscan(catalogue, &value, &|_| String::new()))),
            }
        }
        offer.own = Crop::from_record(&self.library.get(&json!({ "what": "crop", "path": game.path })).map_err(words)?);
        Ok(offer)
    }

    /// Whether a scrape run is going, by the scrape service's progress.
    fn scraping(&self) -> bool {
        self.scraper().and_then(|s| s.progress(u64::MAX).ok()).is_some_and(|p| p["state"] == "running")
    }

    /// The console's profiles where it has more than one, the console's own first, and the game's choice.
    fn profiles(&self, console: &str, path: &str) -> (Option<Vec<Choice>>, Option<String>) {
        let Ok(model) = self.platform.bindings_load(PROFILES, &[console]) else { return (None, None) };
        let of = &model[console];
        let names: Vec<String> = of["Profiles"].as_object().map(|p| p.keys().cloned().collect()).unwrap_or_default();
        let chosen = of["Games"][title_of(path)].as_str().map(str::to_owned);
        if names.len() < 2 {
            return (None, chosen);
        }
        let default = of["Default"].as_str().unwrap_or("Default");
        let mut choices = vec![Choice { value: None, text: format!("Console's ({default})") }];
        choices.extend(names.into_iter().map(|n| Choice { value: Some(n.clone()), text: n }));
        (Some(choices), chosen)
    }
}

impl GamePageSource for ClientSource {
    fn read(&self, game: &GameFile) -> Result<Read, String> {
        let systems = self.runtime.systems().map_err(words)?;
        let system = systems.iter().find(|s| Some(&s.id) == game.system.as_ref());
        let settings = app_settings(&self.platform)?;
        let names = settings.get("ScrapeGameNames").and_then(Value::as_bool).unwrap_or(false);
        let metadata = match self.scraper() {
            Some(s) => s.get(&json!({ "what": "metadata", "path": game.path, "scraped_names": names })).map_err(words)?,
            None => Value::Null,
        };
        let value = |key: &str| metadata.get(key).and_then(|v| v.get("value")).and_then(Value::as_str).unwrap_or("").to_owned();
        let record = self.library.record(&game.path).map_err(words)?.unwrap_or_default();
        let esde = system.map(|s| s.esde_system.clone()).unwrap_or_default();
        let media = |kind: &str| if esde.is_empty() { String::new() } else { self.media_file(game, &esde, kind) };
        let mut read = Read {
            title: Some(value("name")).filter(|n| !n.is_empty()).unwrap_or_else(|| title_of(&game.path)),
            console: system.map(console_name).unwrap_or_default(),
            year: crate::esde_shelves::year_of(&value(crate::esde_shelves::RELEASE_DATE)).unwrap_or_default(),
            developer: value("developer"),
            publisher: value("publisher"),
            description: value("description"),
            played_seconds: record.play_seconds as i64,
            last_played: record.last_played.as_ref().and_then(Value::as_str).and_then(crate::words::parse_utc),
            favourite: record.favourite,
            cover_aspect: crate::library::cover_shape(system),
            cover: media(crate::library::COVER),
            hero: HERO.iter().map(|k| media(k)).find(|f| !f.is_empty()).unwrap_or_default(),
            screenshot: media("screenshot"),
            // Settings ▸ Display ▸ Play videos off shows the still alone (EmuSen_Interface.md §5.9).
            clip: if settings.get("PlayVideos").and_then(Value::as_bool).unwrap_or(true) { media("video") } else { String::new() },
            clip_volume: crate::esde_shelves::volume_of(&settings),
            states: self.states(game, &settings)?,
            ..Read::default()
        };
        let edited = |key: &str| metadata.get(key).and_then(|v| v.get("source")).and_then(Value::as_str) == Some("edited");
        read.details = DETAILS.iter().map(|(field, _)| Detail { field, value: value(field), edited: edited(field) }).collect();
        read.media = self.screenshots(game)?;
        read.media.extend(MEDIA.iter().filter_map(|(kind, what)| Some(media(kind)).filter(|f| !f.is_empty()).map(|path| Media { path, kind: *what })));
        read.engine = Some(value(crate::edits::ALT_EMULATOR)).filter(|e| !e.is_empty());
        (read.package, read.layers, read.package_job) = self.package(game)?;
        read.scraping = self.scraping();
        let Some(s) = system else { return Ok(read) };
        // The engines: the console's own first, then each it may run on (Mistress's IGamePageHost.Engines).
        let cores = self.runtime.cores().map_err(words)?;
        let stored = self.runtime.settings_get(0, Scope::Console, Some(crate::consoles::settings_console(s)), crate::consoles::ENGINE_KEY).map_err(words)?;
        let own = stored.filter(|e| !e.is_empty()).or_else(|| s.engines.first().cloned());
        let name = |id: &str| engine_shown(id, &cores).name;
        read.engines = match (&own, s.engines.len()) {
            (None, _) => Vec::new(),
            (Some(own), 0 | 1) => vec![Choice { value: None, text: format!("{} (the only one)", name(own)) }],
            (Some(own), _) => std::iter::once(Choice { value: None, text: format!("{} (default)", name(own)) })
                .chain(s.engines.iter().map(|e| Choice {
                    value: Some(e.clone()),
                    text: if e == own { format!("{} (the console's)", name(e)) } else { name(e) },
                }))
                .collect(),
        };
        (read.profiles, read.profile) = self.profiles(&s.console, &game.path);
        read.cheats = crate::cheats::List::of(&self.platform, &s.console, &game.path)?.map(|l| l.switches()).unwrap_or_default();
        let catalogue = serenity::catalogue(&self.platform).map_err(words)?;
        if crate::display::television(&catalogue, &s.console) {
            read.crop = Some(self.crop(game, &catalogue)?);
            // The standard is offered for a console that drove a television, as every filter of a tube is (Mistress's `StandardOffered`).
            let standard = self.library.get(&json!({ "what": "standard", "path": game.path })).map_err(words)?;
            let values: Vec<String> = standard["values"].as_array().into_iter().flatten().filter_map(|v| v.as_str().map(str::to_owned)).collect();
            read.standards = Some(crate::game_page::standard_choices(&values));
            read.standard = standard["value"].as_str().map(str::to_owned);
        }
        Ok(read)
    }

    fn start_job(&self, request: &Value) -> Result<Box<dyn crate::tools::cores::backend::Job + Send>, String> {
        crate::tools::cores::backend::sent_job(&self.platform, request)
    }

    fn other_copy(&self, game: &GameFile, state: &str) -> Result<Option<String>, String> {
        // A record the runtime cannot read is not known, as Mistress's `StateRecordsOnApi.Read` answers it.
        let Ok(Some(record)) = self.runtime.state_record(state) else { return Ok(None) };
        let Some(saved) = record.rom_md5.filter(|m| !m.is_empty()) else { return Ok(None) };
        let Ok(identity) = self.library.identity(&json!({ "path": game.path })) else { return Ok(None) };
        let same = identity["md5"].as_str().is_none_or(|md5| md5.eq_ignore_ascii_case(&saved));
        Ok((!same).then(|| record.rom_file.unwrap_or_default()))
    }

    fn change(&self, game: &GameFile, change: &Change) -> Result<(), String> {
        let path = game.path.as_str();
        match change {
            Change::Favourite => self.library.change(&json!({ "op": "toggle_favourite", "path": path })).map(drop).map_err(words),
            Change::Engine(engine) => {
                let at = crate::words::utc(crate::words::now());
                self.library.change(&json!({ "op": "save_edits", "path": path, "changes": { crate::edits::ALT_EMULATOR: engine }, "at": at })).map(drop).map_err(words)
            }
            Change::Standard(standard) => {
                let at = crate::words::utc(crate::words::now());
                self.library.change(&json!({ "op": "set_standard", "path": path, "standard": standard, "at": at })).map(drop).map_err(words)
            }
            Change::DeleteState(state) => {
                // The state, its picture by Galaxia's name for it, and the runtime's record of it (Mistress's DeleteState).
                let picture = self.platform.save_path(SAVE_PICTURE, state, None, 0).map_err(words)?;
                std::fs::remove_file(state).map_err(words)?;
                if picture.exists() {
                    std::fs::remove_file(&picture).map_err(words)?;
                }
                self.runtime.state_forget(state).map_err(words)
            }
            Change::Edit(field, value) => {
                let at = crate::words::utc(crate::words::now());
                self.library.change(&json!({ "op": "save_edits", "path": path, "changes": { *field: value }, "at": at })).map(drop).map_err(words)
            }
            Change::Cheat(index, on) => {
                let systems = self.runtime.systems().map_err(words)?;
                let console = systems.iter().find(|s| Some(&s.id) == game.system.as_ref()).map(|s| s.console.clone()).unwrap_or_default();
                let mut list = crate::cheats::List::of(&self.platform, &console, path)?.ok_or_else(|| "the game has no cheat list".to_string())?;
                if !list.set(*index, *on) {
                    return Err(format!("no cheat {index}"));
                }
                list.save(&self.platform)
            }
            Change::Crop(Some(crop)) => self.library.change(&crop.set_record(path)).map(drop).map_err(words),
            Change::Crop(None) => self.library.change(&json!({ "op": "clear_crop", "path": path })).map(drop).map_err(words),
            Change::Hide => {
                let at = crate::words::utc(crate::words::now());
                self.library.change(&json!({ "op": "save_edits", "path": path, "changes": { crate::edits::HIDDEN: crate::edits::YES }, "at": at })).map(drop).map_err(words)
            }
            Change::Profile(profile) => {
                let systems = self.runtime.systems().map_err(words)?;
                let Some(s) = systems.iter().find(|s| Some(&s.id) == game.system.as_ref()) else { return Ok(()) };
                let mut model = self.platform.bindings_load(PROFILES, &[s.console.as_str()]).map_err(words)?;
                let games = &mut model[s.console.as_str()]["Games"];
                if !games.is_object() {
                    *games = json!({});
                }
                games[title_of(path)] = profile.as_ref().map_or(Value::Null, |p| json!(p));
                self.platform.bindings_save(PROFILES, &model).map_err(words)
            }
        }
    }
}
