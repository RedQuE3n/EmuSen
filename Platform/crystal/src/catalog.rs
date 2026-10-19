//! The catalog: the systems table joined to what discovery found, as the C# `CoreCatalog` joins them, so that
//! `systems`, `shelves` and `route` answer as the C# implementation answers (EmuSen_Backend.md §29.2). The catalog's
//! four hand rows and the engines that are a console's sole engine are the C#'s until state 3, held equal by the parity
//! class.

use emusen_core_host::{Discovered, Discovery};
use emusen_systems::System;

use crate::info::{self, SystemDetail};

/// A console of the catalog: its display name, its systems, and a shelf name for each.
#[derive(Clone, Debug)]
pub struct Row {
    pub display_name: String,
    pub systems: Vec<&'static System>,
    pub shelves: Vec<String>,
    /// Whether discovery added it rather than the catalog's hand rows.
    pub discovered: bool,
}

pub const GAME_BOY_COLOR_SHELF: &str = "Game Boy Color (Mercury)";
pub const ENGINE_KEY: &str = "Engine";

/// The catalog's hand rows, `CoreCatalog.CatalogRows`: display name, shelf names, systems.
const HAND_ROWS: [(&str, &[&str], &[&str]); 4] = [
    ("SNES (Venus)", &["SNES (Venus)"], &["snes"]),
    ("NES (Moon)", &["NES (Moon)"], &["nes"]),
    ("Game Boy (Mercury)", &["Game Boy (Mercury)", GAME_BOY_COLOR_SHELF], &["gb", "gbc"]),
    ("Nintendo 64 (Mars)", &["Nintendo 64 (Mars)"], &["n64"]),
];

/// The keys a player types for the hand rows, `CatalogKeys`, by the row's place.
const HAND_KEYS: [(&str, usize); 9] = [("venus", 0), ("snes", 0), ("moon", 1), ("nes", 1), ("mercury", 2), ("gb", 2), ("gbc", 2), ("mars", 3), ("n64", 3)];

/// A console whose one engine is a discovered v1 engine, `SoleDiscoveredEngines`.
const SOLE_ENGINES: [(&str, &str); 4] = [("SNES", "VenusRT (Rust)"), ("NES", "MoonRT (Rust)"), ("GB", "MercuryRT (Rust)"), ("N64", "MarsRT (Rust)")];

/// The NES's and the SNES's pads and ports, the C# packs' constants (`NesSystem`, `SnesSystem`).
const NES_PAD: [usize; 8] = [4, 5, 6, 7, 2, 3, 0, 8];
const SNES_PAD: [usize; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

fn eq_ic(a: &str, b: &str) -> bool {
    emusen_dotnet::text::equals_ignore_case(a, b)
}

impl Row {
    pub fn console(&self) -> &str {
        &self.systems[0].console
    }

    /// Every system's extensions, each once without regard to case.
    pub fn extensions(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for e in self.systems.iter().flat_map(|s| s.extensions.iter()) {
            if !out.iter().any(|o| eq_ic(o, e)) {
                out.push(e.clone());
            }
        }
        out
    }

    pub fn supports(&self, extension: &str) -> bool {
        self.extensions().iter().any(|e| eq_ic(e, extension))
    }

    pub fn cheat_systems(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for c in self.systems.iter().flat_map(|s| s.cheat_systems.iter()) {
            if !out.contains(c) {
                out.push(c.clone());
            }
        }
        out
    }
}

/// A shelf of the library: its name, its label, the row it belongs to and its system.
#[derive(Clone, Debug)]
pub struct Shelf {
    pub name: String,
    pub label: String,
    pub row: usize,
    pub system: &'static System,
}

/// The catalog for one discovery.
pub struct Catalog<'d> {
    pub discovery: &'d Discovery,
    /// The hand rows, then the consoles discovery adds.
    pub rows: Vec<Row>,
    /// `Registry`: each key a player types and its row, in the order they were added.
    pub registry: Vec<(String, usize)>,
}

fn table() -> &'static [System] {
    emusen_systems::table().unwrap_or(&[])
}

/// The catalog console of a system id, `ConsoleForSystem`: the hand row whose systems include it.
pub fn hand_console_of(system: &str) -> Option<&'static str> {
    HAND_ROWS.iter().find(|(_, _, ids)| ids.contains(&system)).and_then(|(_, _, ids)| emusen_systems::find(ids[0])).map(|s| s.console.as_str())
}

/// The hand row's system ids for a console, `SystemIdsFor`.
pub fn system_ids_for(console: &str) -> Vec<&'static str> {
    HAND_ROWS
        .iter()
        .find(|(_, _, ids)| emusen_systems::find(ids[0]).is_some_and(|s| eq_ic(&s.console, console)))
        .map(|(_, _, ids)| ids.to_vec())
        .unwrap_or_default()
}

pub fn sole_engine(console: &str) -> Option<&'static str> {
    SOLE_ENGINES.iter().find(|(c, _)| eq_ic(c, console)).map(|(_, e)| *e)
}

/// The pack of a console no hand row runs, `PackOf`.
pub fn pack_of(console: &str) -> Option<&'static System> {
    table().iter().find(|s| eq_ic(&s.console, console) && hand_console_of(&s.id).is_none())
}

/// A system is one an engine runs only once its info says its games run there (`Runs`).
fn runs(core: &Discovered, system: &str) -> bool {
    core.sidecar.info.systems.iter().any(|s| s.id == system && !s.development)
}

impl<'d> Catalog<'d> {
    pub fn new(discovery: &'d Discovery) -> Catalog<'d> {
        let mut rows: Vec<Row> = HAND_ROWS
            .iter()
            .map(|(name, shelves, ids)| Row {
                display_name: name.to_string(),
                systems: ids.iter().filter_map(|id| emusen_systems::find(id)).collect(),
                shelves: shelves.iter().map(|s| s.to_string()).collect(),
                discovered: false,
            })
            .collect();
        for console in discovered_consoles(discovery) {
            let engine = discovered_for(discovery, &console).map(|c| c.engine_name().to_string()).unwrap_or_else(|| console.clone());
            let display = format!("{console} ({engine})");
            if let Some(pack) = pack_of(&console) {
                rows.push(Row { display_name: display.clone(), systems: vec![pack], shelves: vec![display], discovered: true });
            }
        }
        let mut registry: Vec<(String, usize)> = HAND_KEYS.iter().map(|(k, r)| (k.to_string(), *r)).collect();
        for (at, row) in rows.iter().enumerate().filter(|(_, r)| r.discovered) {
            let engine = discovered_for(discovery, row.console()).map(|c| c.engine_name().to_string());
            for name in [Some(row.console().to_string()), engine].into_iter().flatten() {
                if !name.is_empty() && !name.chars().any(char::is_whitespace) {
                    let key = emusen_dotnet::text::to_lower(&name);
                    if !registry.iter().any(|(k, _)| eq_ic(k, &key)) {
                        registry.push((key, at));
                    }
                }
            }
        }
        Catalog { discovery, rows, registry }
    }

    /// The rows in release order: by manufacturer without regard to case, then year, then console without regard to case.
    pub fn consoles_in_release_order(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.rows.len()).collect();
        order.sort_by(|&a, &b| {
            let (x, y) = (self.rows[a].systems[0], self.rows[b].systems[0]);
            emusen_dotnet::text::compare_ignore_case(&x.manufacturer, &y.manufacturer)
                .then(x.release_year.cmp(&y.release_year))
                .then(emusen_dotnet::text::compare_ignore_case(&x.console, &y.console))
        });
        order
    }

    pub fn shelves(&self) -> Vec<Shelf> {
        let mut out = Vec::new();
        for r in self.consoles_in_release_order() {
            let row = &self.rows[r];
            for (i, system) in row.systems.iter().enumerate() {
                out.push(Shelf { name: row.shelves[i].clone(), label: system.console.clone(), row: r, system });
            }
        }
        out
    }

    /// The row whose console is `console`, compared exactly, `Registry.Where(kv => kv.Value.Console == console)`.
    pub fn row_of_console(&self, console: &str) -> Option<usize> {
        self.rows.iter().position(|r| r.console() == console)
    }

    /// The system of a discovered console's engine's info, `DiscoveredSystem`.
    pub fn discovered_system(&self, console: &str) -> Option<SystemDetail> {
        let core = discovered_for(self.discovery, console)?;
        let pack = pack_of(console)?;
        info::systems(&core.sidecar.info_json).into_iter().find(|s| s.id == pack.id)
    }

    /// The system a hand console's sole engine gives in its info, `SoleEngineSystem`.
    pub fn sole_engine_system(&self, console: &str) -> Option<SystemDetail> {
        let engine = sole_engine(console)?;
        let core = self.discovery.by_engine_name(engine)?;
        let ids = system_ids_for(console);
        info::systems(&core.sidecar.info_json).into_iter().find(|s| ids.contains(&s.id.as_str()))
    }

    fn engine_system(&self, console: &str) -> Option<SystemDetail> {
        self.discovered_system(console).or_else(|| self.sole_engine_system(console))
    }

    pub fn buttons_for(&self, console: &str) -> Vec<usize> {
        if eq_ic(console, "SNES") {
            return SNES_PAD.to_vec();
        }
        if eq_ic(console, "NES") {
            return NES_PAD.to_vec();
        }
        match self.engine_system(console).map(|s| s.buttons()) {
            Some(pad) if !pad.is_empty() => pad,
            _ => (0..info::BUTTONS.len()).collect(),
        }
    }

    pub fn axes_for(&self, console: &str) -> Vec<usize> {
        self.engine_system(console).map(|s| s.axes()).unwrap_or_default()
    }

    /// `ControllerPorts.ForConsole(console)`.
    pub fn ports_for(&self, console: &str) -> usize {
        if eq_ic(console, "NES") || eq_ic(console, "SNES") {
            return 2;
        }
        self.engine_system(console).map(|s| s.ports().max(1)).unwrap_or(1)
    }

    /// The settings a console's engine offers, from its library, `SettingsFor`; opening a library under the trust rules
    /// is what the C# does here too, so a core's report then says it was loaded.
    pub fn settings_for(&self, console: &str, trust: &emusen_core_host::Trust) -> Vec<info::Setting> {
        let core = discovered_for(self.discovery, console).or_else(|| sole_engine(console).and_then(|e| self.discovery.by_engine_name(e)));
        let Some(core) = core else { return Vec::new() };
        let Ok(lib) = core.open(trust) else { return Vec::new() };
        let Ok(text) = lib.settings_text() else { return Vec::new() };
        emusen_native::json::parse(text.as_bytes()).map(|v| info::settings(&v)).unwrap_or_default()
    }

    /// Each player's controller where the console's pack counts them, `PlayerControllersFor`.
    pub fn player_controllers(&self, console: &str, stored: &dyn Fn(&str) -> Option<String>, trust: &emusen_core_host::Trust) -> Option<Vec<String>> {
        self.discovered_system(console)?;
        let pack = pack_of(console)?;
        if emusen_systems::player_controllers(&pack.id, &|_| None).is_none() {
            return None;
        }
        let settings = self.settings_for(console, trust);
        emusen_systems::player_controllers(&pack.id, &|key| stored(key).or_else(|| settings.iter().find(|s| s.key == key).map(|s| s.default.clone())))
    }

    /// `ControllerPorts.ForConsole(console, stored)`.
    pub fn players_for(&self, console: &str, stored: &dyn Fn(&str) -> Option<String>, trust: &emusen_core_host::Trust) -> usize {
        match self.player_controllers(console, stored, trust) {
            Some(players) if !players.is_empty() => players.len(),
            _ => self.ports_for(console),
        }
    }

    /// The controller a port of a discovered console is set to, `ControllerFor`.
    pub fn controller_for(&self, console: &str, port: usize, stored: &dyn Fn(&str) -> Option<String>, trust: &emusen_core_host::Trust) -> Option<String> {
        let system = self.discovered_system(console)?;
        if let Some(players) = self.player_controllers(console, stored, trust) {
            return players.get(port).filter(|id| system.has_controller(id)).cloned();
        }
        let key = format!("pad{}", port + 1);
        let setting = self.settings_for(console, trust).into_iter().find(|s| s.key == key)?;
        let value = stored(&setting.key).unwrap_or(setting.default);
        system.has_controller(&value).then_some(value)
    }

    /// The engines that run a system, `EnginesOf`: the console's sole engine, else what discovery found for it.
    pub fn engines_of(&self, console: &str, system: &str) -> Vec<String> {
        if let Some(sole) = sole_engine(console) {
            return vec![sole.to_string()];
        }
        let mut out: Vec<String> = Vec::new();
        for c in self.discovery.for_system(system) {
            if !out.iter().any(|e| e == c.engine_name()) {
                out.push(c.engine_name().to_string());
            }
        }
        out
    }

    /// The pack consoles an engine of this build starts with nothing in it, by system id, `ConsolesThatStartEmpty`.
    pub fn starts_empty(&self, system: &str) -> bool {
        self.discovery.for_system(system).any(|c| c.sidecar.info.systems.iter().any(|s| s.id == system && s.starts_empty))
    }

    /// The pack console of a system only a discovered engine serves, `DiscoveredConsoleForSystem`.
    pub fn discovered_console_for_system(system: &str) -> Option<&'static str> {
        if hand_console_of(system).is_some() {
            return None;
        }
        emusen_systems::find(system).map(|s| s.console.as_str())
    }
}

/// The engine of a console no hand row runs, `DiscoveredFor`.
pub fn discovered_for<'d>(discovery: &'d Discovery, console: &str) -> Option<&'d Discovered> {
    table()
        .iter()
        .filter(|s| eq_ic(&s.console, console) && hand_console_of(&s.id).is_none())
        .flat_map(|s| discovery.for_system(&s.id).filter(move |c| runs(c, &s.id)))
        .next()
}

/// The consoles no hand row runs whose pack names them and a discovered engine serves, in release order, each once,
/// `DiscoveredConsoles`.
pub fn discovered_consoles(discovery: &Discovery) -> Vec<String> {
    let mut packs: Vec<&'static System> = table().iter().filter(|s| hand_console_of(&s.id).is_none() && discovery.for_system(&s.id).any(|c| runs(c, &s.id))).collect();
    packs.sort_by(|x, y| emusen_dotnet::text::compare_ignore_case(&x.manufacturer, &y.manufacturer).then(x.release_year.cmp(&y.release_year)));
    let mut out: Vec<String> = Vec::new();
    for p in packs {
        if !out.iter().any(|c| eq_ic(c, &p.console)) {
            out.push(p.console.clone());
        }
    }
    out
}
