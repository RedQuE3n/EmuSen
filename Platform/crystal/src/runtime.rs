//! The runtime: what it was told when it was made (the frontend API's 1.2 config), the engines its cores directories
//! hold, found by `emusen-core-host` and opened under its trust rules, and the answers about systems, engines and a
//! file's route, which run no core (EmuSen_Backend.md §29.2, §29.6).

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use emusen_core_host::{Discovery, Trust, listing};
use emusen_native::core::jsonw::Json as HostJson;

use crate::catalog::Catalog;
use crate::json::Json;

/// What a runtime is told when it is made.
#[derive(Clone, Debug, Default)]
pub struct Config {
    /// The cores directories in their order; none for the program's directory and its `cores` folder.
    pub cores: Option<Vec<String>>,
    /// Whether cores in development are listed; none for `EMUSEN_DEVELOPMENT_CORES`' answer.
    pub development: Option<bool>,
}

pub struct Runtime {
    directories: Vec<PathBuf>,
    development: bool,
    trust: Trust,
    discovery: Mutex<Arc<Discovery>>,
    /// The runtime's own lines no session said, kept until the events raise them on session 0 (§29.11, Q2).
    said: Mutex<Vec<String>>,
    pub sessions: crate::open::Sessions,
}

impl Runtime {
    pub fn new(config: &Config) -> Runtime {
        let directories: Vec<PathBuf> = match &config.cores {
            Some(dirs) => dirs.iter().map(PathBuf::from).collect(),
            None => {
                let base = emusen_galaxia::tree::base_directory();
                vec![PathBuf::from(&base), PathBuf::from(emusen_galaxia::dotnet_path::combine(emusen_galaxia::dotnet_path::Style::HOST, &base, "cores"))]
            }
        };
        let development = config.development.unwrap_or_else(|| std::env::var(emusen_core_host::discovery::DEVELOPMENT_VARIABLE).as_deref() == Ok("1"));
        let trust = Trust::new(&directories, &[]);
        let discovery = Mutex::new(Arc::new(Discovery::scan(&directories, development)));
        Runtime { directories, development, trust, discovery, said: Mutex::new(Vec::new()), sessions: crate::open::Sessions::default() }
    }

    /// A line of the runtime's own that no session said.
    pub fn say(&self, line: String) {
        self.said.lock().unwrap_or_else(|p| p.into_inner()).push(line);
    }

    /// The lines said since the last call.
    pub fn take_said(&self) -> Vec<String> {
        std::mem::take(&mut *self.said.lock().unwrap_or_else(|p| p.into_inner()))
    }

    pub fn trust(&self) -> &Trust {
        &self.trust
    }

    /// The discovery the runtime answers from now; a rescan replaces it, and a session already open keeps its engine.
    pub fn discovery(&self) -> Arc<Discovery> {
        self.discovery.lock().unwrap_or_else(|p| p.into_inner()).clone()
    }

    pub fn rescan(&self) {
        let fresh = Arc::new(Discovery::scan(&self.directories, self.development));
        *self.discovery.lock().unwrap_or_else(|p| p.into_inner()) = fresh;
    }

    /// `[{"name","id","version","systems","available","report"}]`.
    pub fn engines_json(&self) -> String {
        let mut j = HostJson::new();
        listing::engines(&mut j, &self.discovery());
        j.finish()
    }

    /// `[{"library","sidecar","sha256","abi","development","engine","report"}]`.
    pub fn cores_json(&self) -> String {
        let mut j = HostJson::new();
        listing::cores(&mut j, &self.discovery());
        j.finish()
    }

    /// The systems table's systems with their engines and the engine's facts, then those only an engine's info names.
    pub fn systems_json(&self) -> String {
        let discovery = self.discovery();
        let catalog = Catalog::new(&discovery);
        let stored = emusen_galaxia::consoles::load(&emusen_galaxia::tree::tree());
        let mut j = Json::new();
        j.begin_array();
        let mut listed: Vec<&str> = Vec::new();
        for e in emusen_systems::table().unwrap_or(&[]) {
            listed.push(&e.id);
            j.begin_object()
                .str("id", &e.id)
                .str("name", &e.name)
                .str("console", &e.console)
                .str("manufacturer", &e.manufacturer)
                .int("release_year", e.release_year as i64)
                .strs("extensions", &e.extensions)
                .strs("cheat_systems", &e.cheat_systems)
                .strs("openvgdb_systems", &e.openvgdb_systems)
                .f64("cover_aspect", e.cover_aspect)
                .str("esde_system", &e.esde_system)
                .str("esde_full_name", &e.esde_full_name)
                .bool("self_delimiting_images", e.self_delimiting_images)
                .str_or_null("attached_to", e.attached_to.as_deref());
            match e.screenscraper_system {
                Some(id) => j.int("screenscraper_system", id as i64),
                None => j.null("screenscraper_system"),
            };
            j.strs("engines", catalog.engines_of(&e.console, &e.id)).bool("starts_empty", catalog.starts_empty(&e.id));
            self.core_of(&mut j, &catalog, &e.console, &e.extensions);
            self.controllers(&mut j, &catalog, &e.console, &stored);
            j.end_object();
        }
        let mut groups: Vec<(&emusen_core_host::descriptors::CoreSystem, Vec<String>, bool)> = Vec::new();
        for c in discovery.found() {
            for s in c.sidecar.info.systems.iter().filter(|s| discovery.offered(s) && !listed.contains(&s.id.as_str())) {
                match groups.iter_mut().find(|(g, _, _)| g.id == s.id) {
                    Some((_, engines, empty)) => {
                        if !engines.iter().any(|e| e == c.engine_name()) {
                            engines.push(c.engine_name().to_string());
                        }
                        *empty |= s.starts_empty;
                    }
                    None => groups.push((s, vec![c.engine_name().to_string()], s.starts_empty)),
                }
            }
        }
        for (s, engines, empty) in groups {
            j.begin_object().str("id", &s.id).str("name", &s.name).strs("extensions", &s.extensions).strs("engines", &engines).bool("starts_empty", empty);
            let mut names = vec![s.id.clone()];
            for n in engines.iter().filter(|n| !n.chars().any(char::is_whitespace)).map(|n| emusen_dotnet::text::to_lower(n)) {
                if !names.contains(&n) {
                    names.push(n);
                }
            }
            j.key_object("core").strs("names", &names).str("display_name", &format!("{} ({})", s.name, engines.join(", "))).str("console", &s.id).strs("extensions", &s.extensions).end_object();
            let console = Catalog::discovered_console_for_system(&s.id).map(str::to_string).unwrap_or_else(|| s.id.clone());
            self.controllers(&mut j, &catalog, &console, &stored);
            j.end_object();
        }
        j.end_array();
        j.finish()
    }

    /// The names `core <name>` knows a system's core by, and its display name, console and extensions, `CoreOf`.
    fn core_of(&self, j: &mut Json, catalog: &Catalog, console: &str, extensions: &[String]) {
        let mut named: Vec<&(String, usize)> = catalog.registry.iter().filter(|(_, r)| catalog.rows[*r].console() == console).collect();
        if named.is_empty() {
            named = catalog.registry.iter().filter(|(_, r)| extensions.iter().all(|e| catalog.rows[*r].supports(e))).collect();
        }
        let Some(&&(_, row)) = named.first() else {
            j.null("core");
            return;
        };
        let r = &catalog.rows[row];
        j.key_object("core")
            .strs("names", named.iter().filter(|(_, at)| *at == row).map(|(k, _)| k.as_str()))
            .str("display_name", &r.display_name)
            .str("console", r.console())
            .strs("extensions", r.extensions())
            .end_object();
    }

    /// A console's pad, its most ports and each player's controller as its stored settings stand.
    fn controllers(&self, j: &mut Json, catalog: &Catalog, console: &str, stored: &emusen_galaxia::consoles::Consoles) {
        let value = |key: &str| stored.value(console, key).map(str::to_string);
        j.key_object("controllers").key_array("buttons");
        for b in catalog.buttons_for(console) {
            j.int_value(b as i64);
        }
        j.end_array().key_array("axes");
        for a in catalog.axes_for(console) {
            j.int_value(a as i64);
        }
        j.end_array().int("ports", catalog.ports_for(console) as i64).key_array("players");
        for p in 0..catalog.players_for(console, &value, &self.trust) {
            match catalog.controller_for(console, p, &value, &self.trust) {
                Some(id) => j.str_value(&id),
                None => j.null_value(),
            };
        }
        j.end_array().end_object();
    }

    /// `{"system","console","engine","reason","cartridge","shelf"}` for a file, as `open` would route it.
    pub fn route_json(&self, at: &str) -> String {
        let discovery = self.discovery();
        let catalog = Catalog::new(&discovery);
        let tree = emusen_galaxia::tree::tree();
        crate::route::route_json(&catalog, &discovery, at, &|game, console, system| crate::settings::engine_for(&tree, game, console, Some(system)))
    }

    /// The library's shelves in release order.
    pub fn shelves_json(&self) -> String {
        let discovery = self.discovery();
        let catalog = Catalog::new(&discovery);
        let mut j = Json::new();
        j.begin_array();
        for s in catalog.shelves() {
            let row = &catalog.rows[s.row];
            j.begin_object()
                .str("name", &s.name)
                .str("label", &s.label)
                .str("system", &s.system.id)
                .str("esde_system", &s.system.esde_system)
                .str("esde_full_name", &s.system.esde_full_name)
                .key_object("core")
                .str("display_name", &row.display_name)
                .str("console", row.console())
                .strs("extensions", row.extensions())
                .end_object()
                .end_object();
        }
        j.end_array();
        j.finish()
    }
}

/// A refusal: its status and words.
pub type Refused = (i32, String);

impl Runtime {
    pub fn open(&self, asked: &crate::open::Asked) -> Result<u32, Refused> {
        crate::open::open(self, &self.sessions, asked).map(|s| s.id)
    }

    /// The battery save written, the save media let go, the session ended, `ManagedSession.Close`.
    pub fn close(&self, id: u32) -> Result<(), Refused> {
        let s = self.sessions.get(id)?;
        s.close(|e| {
            e.save_sram();
            e.dispose();
            for line in e.take_said() {
                self.say(line);
            }
        })?;
        self.sessions.remove(s.id);
        Ok(())
    }

    /// The console's reset, `ManagedSession.Reset`.
    pub fn reset(&self, id: u32) -> Result<(), Refused> {
        self.sessions.get(id)?.change(|e, run| {
            e.reset().map_err(|r| (r.status, r.words))?;
            run.halted = false;
            e.refresh_debugger();
            Ok(())
        })
    }

    /// `[{"id","engine","current"}]`, `ManagedRuntime.SessionsJson`.
    pub fn sessions_json(&self) -> String {
        let (all, current) = self.sessions.all_and_current();
        let mut j = crate::json::Json::new();
        j.begin_array();
        for s in all {
            let name = s.engine.lock().unwrap_or_else(|p| p.into_inner()).name.clone();
            j.begin_object().int("id", s.id as i64).str("engine", &name).bool("current", s.id == current).end_object();
        }
        j.end_array();
        j.finish()
    }

    pub fn session_info(&self, id: u32) -> Result<String, Refused> {
        Ok(self.sessions.get(id)?.info_json())
    }

    pub fn save_media(&self, id: u32) -> Result<String, Refused> {
        let s = self.sessions.get(id)?;
        let discovery = self.discovery();
        let catalog = Catalog::new(&discovery);
        let tree = emusen_galaxia::tree::tree();
        Ok(s.save_media_json(&catalog, &tree))
    }

    /// A change to a session's drive: the drive refused where the engine changes no discs.
    fn disc(&self, id: u32, change: impl FnOnce(&mut crate::engine::Engine, &emusen_galaxia::tree::Tree, &dyn Fn(&str) -> bool) -> Result<(), Refused>) -> Result<(), Refused> {
        let s = self.sessions.get(id)?;
        let discovery = self.discovery();
        let catalog = Catalog::new(&discovery);
        let tree = emusen_galaxia::tree::tree();
        let is_image = |e: &str| crate::route::is_image_extension(&catalog, &discovery, e);
        s.change(|e, _| {
            if !e.can_change_disc() {
                return Err((emusen_session_service::abi::status::NOT_SUPPORTED, format!("{} has no drive whose disc can be changed", e.name)));
            }
            change(e, &tree, &is_image)
        })
    }

    pub fn disc_insert(&self, id: u32, at: &str) -> Result<(), Refused> {
        self.disc(id, |e, tree, is_image| e.insert_path(tree, at, is_image).map_err(|r| (r.status, r.words)))
    }

    pub fn disc_eject(&self, id: u32) -> Result<(), Refused> {
        self.disc(id, |e, tree, is_image| e.change_disc(tree, -1, is_image).map_err(|r| (r.status, r.words)))
    }

    pub fn disc_change(&self, id: u32, index: u32) -> Result<(), Refused> {
        self.disc(id, |e, tree, is_image| {
            if index as usize >= e.discs.len() {
                return Err((emusen_session_service::abi::status::BAD_ARGUMENT, format!("no disc {index}; the game has {}", e.discs.len())));
            }
            e.change_disc(tree, index as i32, is_image).map_err(|r| (r.status, r.words))
        })
    }

    pub fn disc_lid(&self, id: u32, open: bool) -> Result<(), Refused> {
        self.disc(id, |e, _, _| e.set_lid(open).map_err(|r| (r.status, r.words)))
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        for s in self.sessions.all() {
            let _ = self.close(s.id);
        }
    }
}
