//! Opening a session, the C# `ManagedRuntime.Open` and `CoreFactory`'s v1 branch: a game on the engine its route and
//! the settings' layers choose, its firmware asked first, or a console started empty; the engine made with the
//! console's stored settings and media, the layered settings applied (EmuSen_Backend.md §29.2).

use std::sync::{Arc, Mutex};

use emusen_core_host::{Discovered, Discovery, Library, Trust};
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::tree::Tree;
use emusen_session_service::abi::status as svc;

use crate::catalog::{self, Catalog};
use crate::engine::{Ctx, Engine, Options};
use crate::info::SystemDetail;
use crate::session::{OpenedWith, Session};

/// A refusal: its status and words.
pub type Refused = (i32, String);

/// The system of an engine's info a file runs as, `CoreCatalog.SystemOfFile(path, info)`.
pub fn system_of_info(catalog: &Catalog, discovery: &Discovery, at: &str, core: &Discovered) -> Option<SystemDetail> {
    let (row, decision) = crate::route::route(catalog, discovery, at);
    let id = decision.as_ref().and_then(|d| d.system.map(|s| s.id.clone())).or_else(|| row.and_then(|r| catalog::pack_of(catalog.rows[r].console()).map(|p| p.id.clone())));
    let systems = crate::info::systems(&core.sidecar.info_json);
    if let Some(id) = id
        && let Some(s) = systems.iter().find(|s| s.id == id)
    {
        return Some(s.clone());
    }
    let extension = crate::route::image_extension(catalog, discovery, at);
    let id = core.sidecar.info.system_for(&extension)?.id.clone();
    systems.into_iter().find(|s| s.id == id)
}

/// A console's sole engine missing, in the player's words, `CoreFactory.Missing`.
fn missing(discovery: &Discovery, console: &str, engine: &str) -> Refused {
    let report = discovery.by_engine_name(engine).map(|c| c.report()).unwrap_or_else(|| "no sidecar for it was found beside the program".to_string());
    (crate::abi::status::NO_ENGINE, format!("This build has no {console} core: {engine} is missing ({report})."))
}

/// The library a game opens on, `CoreFactory.Create`'s v1 branch and its refusals.
fn library_for<'d>(catalog: &Catalog, discovery: &'d Discovery, trust: &Trust, at: &str, engine: Option<&str>) -> Result<(&'d Discovered, &'static Library), Refused> {
    let found = crate::route::v1_engine(catalog, discovery, at, engine).map_err(|why| (crate::abi::status::NO_ENGINE, why))?;
    if let Some(core) = found {
        match core.open(trust) {
            Ok(lib) => return Ok((core, lib)),
            Err(why) if core.awaits_approval() => return Err((crate::abi::status::NOT_APPROVED, why.to_string())),
            Err(_) => {}
        }
    }
    let extension = crate::route::image_extension(catalog, discovery, at);
    Err(match extension.as_str() {
        ".smc" | ".sfc" => missing(discovery, "SNES", "VenusRT (Rust)"),
        ".nes" => missing(discovery, "NES", "MoonRT (Rust)"),
        ".gb" | ".gbc" => missing(discovery, "Game Boy", "MercuryRT (Rust)"),
        ".z64" | ".n64" | ".v64" => missing(discovery, "N64", "MarsRT (Rust)"),
        other => (crate::abi::status::NO_ENGINE, format!("No core in this build handles '{other}' - see CoreCatalog for what is registered.")),
    })
}

/// The console's stored values of the library's settings, `CoreFactory.StoredSettings`.
fn stored_settings(engine_settings: &[crate::machine_info::SettingDesc], stored: &emusen_galaxia::consoles::Consoles, console: Option<&str>) -> Vec<(String, String)> {
    let Some(console) = console else { return Vec::new() };
    engine_settings.iter().filter_map(|s| stored.value(console, &s.key).map(|v| (s.key.clone(), v.to_string()))).collect()
}

/// The runtime's sessions by id, the current one the last opened that is still open.
#[derive(Default)]
pub struct Sessions {
    list: Mutex<(Vec<Arc<Session>>, u32, u32)>,
}

impl Sessions {
    pub fn get(&self, id: u32) -> Result<Arc<Session>, Refused> {
        let list = self.list.lock().unwrap_or_else(|p| p.into_inner());
        let key = if id == 0 { list.2 } else { id };
        list.0.iter().find(|s| s.id == key && key != 0).cloned().ok_or_else(|| (svc::NO_SESSION, if id == 0 { "no session is open".to_string() } else { format!("no session {id}") }))
    }

    fn next_id(&self) -> u32 {
        let mut list = self.list.lock().unwrap_or_else(|p| p.into_inner());
        if list.1 == 0 {
            list.1 = 1;
        }
        let id = list.1;
        list.1 += 1;
        id
    }

    fn add(&self, s: Arc<Session>) {
        let mut list = self.list.lock().unwrap_or_else(|p| p.into_inner());
        list.2 = s.id;
        list.0.push(s);
    }

    pub fn remove(&self, id: u32) {
        let mut list = self.list.lock().unwrap_or_else(|p| p.into_inner());
        list.0.retain(|s| s.id != id);
        if list.2 == id {
            list.2 = list.0.iter().map(|s| s.id).max().unwrap_or(0);
        }
    }

    /// Every session by id, and the current one's id, read together.
    pub fn all_and_current(&self) -> (Vec<Arc<Session>>, u32) {
        let list = self.list.lock().unwrap_or_else(|p| p.into_inner());
        let mut all = list.0.clone();
        all.sort_by_key(|s| s.id);
        (all, list.2)
    }

    pub fn all(&self) -> Vec<Arc<Session>> {
        let mut all = self.list.lock().unwrap_or_else(|p| p.into_inner()).0.clone();
        all.sort_by_key(|s| s.id);
        all
    }
}

/// The empty consoles in release order, `ConsolesThatStartEmpty`.
fn empty_starts(catalog: &Catalog) -> Vec<(String, String)> {
    let mut packs: Vec<&emusen_systems::System> = emusen_systems::table().unwrap_or(&[]).iter().filter(|e| catalog.starts_empty(&e.id)).collect();
    packs.sort_by(|x, y| emusen_dotnet::text::compare_ignore_case(&x.manufacturer, &y.manufacturer).then(x.release_year.cmp(&y.release_year)));
    packs.into_iter().map(|e| (e.console.clone(), e.id.clone())).collect()
}

/// What `open` is asked.
pub struct Asked<'a> {
    pub path: Option<&'a str>,
    pub console: Option<&'a str>,
    pub engine: Option<&'a str>,
    pub opened: OpenedWith,
}

/// A session opened, `ManagedRuntime.Open`.
pub fn open(rt: &crate::runtime::Runtime, sessions: &Sessions, asked: &Asked) -> Result<Arc<Session>, Refused> {
    if asked.path.is_none() == asked.console.is_none() {
        return Err((svc::BAD_ARGUMENT, "a session opens a game or starts a console: one of the two is given".to_string()));
    }
    let tree = emusen_galaxia::tree::tree();
    let discovery = rt.discovery();
    let catalog = Catalog::new(&discovery);
    let folders = crate::firmware::folders(&tree);
    let options = Options { battery_disabled: asked.opened.flags & crate::abi::OPEN_NO_BATTERY != 0, audio_buffer_max_samples: asked.opened.audio_buffer_max_samples, device: asked.opened.device };
    let is_image = |e: &str| crate::route::is_image_extension(&catalog, &discovery, e);
    let ctx = Ctx { tree: &tree, is_image: &is_image, folders: &folders };
    let stored = emusen_galaxia::consoles::load(&tree);
    let resolved = match asked.path {
        Some(at) => Some(emusen_galaxia::claims::resolve(&emusen_galaxia::claims::Places::of(&tree).roots(), at).map_err(|words| (crate::abi::status::IO, words))?),
        None => None,
    };
    let path = resolved.as_ref().map(|r| r.game.as_str());
    let package = resolved.as_ref().and_then(|r| r.package.clone());
    let (engine, console_asked) = match path {
        Some(at) => (open_game(rt, &catalog, &discovery, &tree, &ctx, &stored, at, asked.engine, options, package.clone())?, None),
        None => {
            let (e, console) = start_console(rt, &catalog, &discovery, &tree, &folders, &stored, asked.console.unwrap_or(""), asked.engine, options)?;
            (e, Some(console))
        }
    };
    let name = if !engine.console.is_empty() {
        engine.console.clone()
    } else if let Some(c) = &console_asked {
        c.clone()
    } else {
        path.and_then(|p| crate::route::route(&catalog, &discovery, p).0).map(|r| catalog.rows[r].console().to_string()).unwrap_or_else(|| engine.display_name.clone())
    };
    let empty = if asked.path.is_none() { console_asked } else { None };
    let mut session = Session::new(sessions.next_id(), engine, path.map(str::to_string), name, empty, asked.opened, rt.events.clone());
    session.package = package;
    let session = Arc::new(session);
    apply_stored(&session, &ctx, &stored);
    sessions.add(session.clone());
    Ok(session)
}

/// Every listed setting applied as the layers give it, game over console, `ApplyStored`.
pub fn apply_stored(s: &Session, ctx: &Ctx, stored: &emusen_galaxia::consoles::Consoles) {
    let mut e = s.engine.lock().unwrap_or_else(|p| p.into_inner());
    s.raise_said(&e);
    let system = e.machine.as_ref().map(|_| e.mi.system.clone());
    let game = crate::settings::game_values(ctx.tree, system.as_deref(), &s.game_key());
    for setting in e.listed() {
        let own = game.iter().find(|(k, _)| *k == setting.key).map(|(_, v)| v.clone());
        let layered = match own {
            Some(v) => Some(v),
            None => stored.value(&s.console, &setting.key).map(str::to_string),
        };
        let layered = if setting.key == catalog::ENGINE_KEY { crate::settings::current(layered) } else { layered };
        if e.value(&setting.key) == Some(layered.as_deref().unwrap_or(&setting.default)) {
            continue;
        }
        e.apply(ctx, &setting.key, layered.as_deref());
        s.raise_said(&e);
    }
}

#[allow(clippy::too_many_arguments)]
fn open_game(
    rt: &crate::runtime::Runtime,
    catalog: &Catalog,
    discovery: &Discovery,
    tree: &Tree,
    ctx: &Ctx,
    stored: &emusen_galaxia::consoles::Consoles,
    at: &str,
    engine: Option<&str>,
    options: Options,
    package: Option<String>,
) -> Result<Engine, Refused> {
    if !emusen_galaxia::dotnet_files::file_exists(at) {
        return Err((crate::abi::status::IO, format!("{at} does not exist")));
    }
    let chosen = engine.map(str::to_string).or_else(|| {
        let (row, _) = crate::route::route(catalog, discovery, at);
        let system = crate::route::system_of_file(catalog, discovery, at);
        crate::settings::engine_for(tree, at, row.map(|r| catalog.rows[r].console()), system.as_deref())
    });
    let (core_name, missing) = crate::firmware::needed(rt, tree, at, chosen.as_deref());
    if !missing.is_empty() {
        let list: Vec<String> = missing.iter().map(|m| format!("{core_name} {} firmware - {}, {} bytes", m.label, m.name, crate::firmware::n0(m.size))).collect();
        return Err((crate::abi::status::FIRMWARE, format!("{} needs firmware the library does not hold: {}", path::file_name(Style::HOST, at), list.join("; "))));
    }
    let (core, lib) = library_for(catalog, discovery, &rt.trust(), at, chosen.as_deref())?;
    let system = system_of_info(catalog, discovery, at, core);
    let (row, _) = crate::route::route(catalog, discovery, at);
    let row_console = row.map(|r| catalog.rows[r].console().to_string());
    let stored_console = row_console.clone().or_else(|| system.as_ref().and_then(|s| Catalog::discovered_console_for_system(&s.id)).map(str::to_string));
    let mut e = Engine::new(lib, core.sidecar.info_json.clone(), &[], options);
    let values = stored_settings(&e.settings, stored, stored_console.as_deref());
    e = Engine::new(lib, core.sidecar.info_json.clone(), &values, options);
    if let Some(console) = &stored_console {
        let media = crate::media::media_of(console, system.as_ref());
        e.media_choices = crate::media::stored_choices(&media, &|k| stored.value(console, k).map(str::to_string));
    }
    let console = system
        .as_ref()
        .and_then(|s| Catalog::discovered_console_for_system(&s.id).map(str::to_string).or_else(|| Some(emusen_dotnet::text::to_upper(&s.id))))
        .or(row_console)
        .unwrap_or_else(|| core.sidecar.info.id.clone());
    e.device_lender = rt.lender.clone().filter(|_| options.device != 0).map(|l| (l, options.device));
    e.package = package;
    e.load_rom(ctx.tree, at, system, console, ctx.is_image, ctx.folders).map_err(|r| (r.status, r.words))?;
    Ok(e)
}

#[allow(clippy::too_many_arguments)]
fn start_console(
    rt: &crate::runtime::Runtime,
    catalog: &Catalog,
    discovery: &Discovery,
    tree: &Tree,
    folders: &[String],
    stored: &emusen_galaxia::consoles::Consoles,
    asked: &str,
    engine: Option<&str>,
    options: Options,
) -> Result<(Engine, String), Refused> {
    let starts = empty_starts(catalog);
    let console = starts
        .iter()
        .find(|(c, id)| emusen_dotnet::text::equals_ignore_case(c, asked) || emusen_dotnet::text::equals_ignore_case(id, asked))
        .map(|(c, _)| c.clone())
        .ok_or_else(|| (crate::abi::status::NO_ENGINE, format!("no console named \"{asked}\" starts with nothing in it in this build")))?;
    let chosen = engine.map(str::to_string).or_else(|| crate::settings::current(stored.value(&console, catalog::ENGINE_KEY).map(str::to_string)));
    let not_here = || (crate::abi::status::NO_ENGINE, format!("No core in this build starts the {console} with nothing in it."));
    let (_, system_id) = starts.iter().find(|(c, _)| emusen_dotnet::text::equals_ignore_case(c, &console)).cloned().ok_or_else(not_here)?;
    let mut candidates: Vec<&Discovered> =
        discovery.for_system(&system_id).filter(|c| c.sidecar.info.systems.iter().any(|s| s.id == system_id && s.starts_empty)).collect();
    candidates.sort_by_key(|c| if Some(c.engine_name()) == chosen.as_deref() { 0 } else { 1 });
    let core = *candidates.first().ok_or_else(not_here)?;
    let lib = core.open(&rt.trust()).map_err(|why| {
        let status = if core.awaits_approval() { crate::abi::status::NOT_APPROVED } else { crate::abi::status::NO_ENGINE };
        (status, format!("{} is not in use: {why}", core.engine_name()))
    })?;
    let probe = Engine::new(lib, core.sidecar.info_json.clone(), &[], options);
    let values = stored_settings(&probe.settings, stored, Some(&console));
    let mut e = Engine::new(lib, core.sidecar.info_json.clone(), &values, options);
    let system = crate::info::systems(&core.sidecar.info_json).into_iter().find(|s| s.id == system_id);
    let media = crate::media::media_of(&console, system.as_ref());
    e.media_choices = crate::media::stored_choices(&media, &|k| stored.value(&console, k).map(str::to_string));
    e.device_lender = rt.lender.clone().filter(|_| options.device != 0).map(|l| (l, options.device));
    e.start_console(tree, &system_id, console.clone(), folders).map_err(|r| (r.status, r.words))?;
    Ok((e, console))
}
