//! Settings ▸ Consoles as plain data (EmuSen_Qt.md §7.7.E, EmuSen_BigPicture.md §52.4): every console the runtime lists,
//! its facts, and its settings schema, values, firmware and cards, read once each time Settings opens. A console's page
//! is derived from what its engines declare (EmuSen_Qt.md §6.2): nothing here names a console or a core.

use crate::cards::{self, Medium};
use crate::firmware::{self, Need};
use emusen_crystal_client::shapes::{CoreRow, FirmwareFile, FirmwareItem, SettingRow, SystemRow};
use serde_json::Value;
use std::collections::BTreeMap;

/// The runtime's key for a console's engine, the value Cores' Make Default writes too (`emusen_crystal.h`).
pub const ENGINE_KEY: &str = "Engine";

/// What the Consoles section reads of the runtime, through the frontend API.
pub trait ConsoleSource {
    fn systems(&self) -> Result<Vec<SystemRow>, String>;
    fn cores(&self) -> Vec<CoreRow>;
    /// The console's settings schema, as its chosen engine declares it.
    fn schema(&self, console: &str) -> Vec<SettingRow>;
    /// A key's value as the console holds it, stored or defaulted.
    fn setting(&self, console: &str, key: &str) -> Option<String>;
    /// A system's firmware files and whether each is found; none where the runtime cannot say.
    fn firmware(&self, system: &str) -> Option<Vec<FirmwareFile>>;
    /// Every file of the runtime's firmware overview, where each was found and what its open version changes.
    fn overview(&self) -> Vec<FirmwareItem>;
    /// The card manager's list of the console's media.
    fn cards(&self, console: &str) -> Value;
    /// A card as the card manager opens it (`CARDS_OPEN`); null for a file of no medium.
    fn open_card(&self, path: &str) -> Value;
}

/// One console as Settings shows it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Console {
    /// The console's name as the settings store it (the systems table's `console`).
    pub console: String,
    /// Its name in a list: the table's display name, else its name.
    pub name: String,
    pub systems: Vec<String>,
    pub engines: Vec<String>,
    /// A console that starts with no game in it, whose firmware every game needs.
    pub starts_empty: bool,
    pub year: i32,
    pub firmware_name: Option<String>,
    pub schema: Vec<SettingRow>,
    pub values: BTreeMap<String, String>,
    pub firmware: Vec<(String, FirmwareFile)>,
    pub media: Vec<Medium>,
    /// The console's pad as its engine declares it: the canonical controls, the axes, the most ports and each player's
    /// controller (EmuSen_Qt.md §6.1).
    pub controllers: emusen_crystal_client::shapes::Controllers,
    /// Whether the console's games drive a motor (the systems table's `rumble`).
    pub rumble: bool,
}

impl Console {
    /// Whether an engine runs it, so its page opens.
    pub fn runs(&self) -> bool {
        !self.engines.is_empty()
    }

    /// A key's value as the console holds it, else its schema default.
    pub fn value(&self, key: &str) -> String {
        self.values.get(key).cloned().or_else(|| self.row(key).and_then(|r| r.default.clone())).unwrap_or_default()
    }

    pub fn row(&self, key: &str) -> Option<&SettingRow> {
        self.schema.iter().find(|r| r.key == key)
    }

    pub fn needs(&self) -> Vec<Need> {
        firmware::needs(&self.firmware, self.firmware_name.as_deref())
    }

    /// The engine's value, the schema's Engine row's chosen value, else the first engine listed.
    pub fn engine(&self) -> String {
        Some(self.value(ENGINE_KEY)).filter(|v| !v.is_empty()).or_else(|| self.engines.first().cloned()).unwrap_or_default()
    }

    /// The keys its media's rows are stored under: each mode, and the rows shown with it.
    pub fn media_keys(&self) -> Vec<String> {
        let mut keys = Vec::new();
        for m in &self.media {
            let k = cards::keys(m, &self.schema);
            keys.extend(k.mode.map(|r| r.key.clone()));
            keys.extend(k.slots.iter().map(|r| r.key.clone()));
        }
        keys
    }
}

/// Everything the section shows, in the list's order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConsoleRead {
    pub consoles: Vec<Console>,
    pub cores: Vec<CoreRow>,
    pub overview: Vec<FirmwareItem>,
}

impl ConsoleRead {
    pub fn console(&self, console: &str) -> Option<&Console> {
        self.consoles.iter().find(|c| c.console == console)
    }

    pub fn console_mut(&mut self, console: &str) -> Option<&mut Console> {
        self.consoles.iter_mut().find(|c| c.console == console)
    }

    /// An engine by its stored value, named as the other screens name it (`clients::engine_shown`).
    pub fn engine_name(&self, engine: &str) -> String {
        crate::clients::engine_shown(engine, &self.cores).name
    }

    /// The consoles an engine runs, in the list's order, which LB and RB step through on a console's page.
    pub fn running(&self) -> Vec<&Console> {
        self.consoles.iter().filter(|c| c.runs()).collect()
    }
}

/// Reads every console: each once, by the first system naming it; those an engine runs first, then by release year and
/// name (EmuSen_BigPicture.md §52.4). A console no engine runs is listed with no schema read.
pub fn read(source: &dyn ConsoleSource) -> Result<ConsoleRead, String> {
    let systems = source.systems()?;
    let mut consoles: Vec<Console> = Vec::new();
    for s in &systems {
        let console = if s.console.is_empty() { s.id.clone() } else { s.console.clone() };
        if let Some(c) = consoles.iter_mut().find(|c| c.console.eq_ignore_ascii_case(&console)) {
            c.systems.push(s.id.clone());
            c.starts_empty |= s.starts_empty;
            c.rumble |= s.rumble;
            if c.controllers.buttons.is_empty() {
                c.controllers = s.controllers.clone();
            }
            for e in &s.engines {
                if !c.engines.contains(e) {
                    c.engines.push(e.clone());
                }
            }
            continue;
        }
        let name = if s.display_name.is_empty() { s.name.clone() } else { s.display_name.clone() };
        consoles.push(Console {
            console,
            name,
            systems: vec![s.id.clone()],
            engines: s.engines.clone(),
            starts_empty: s.starts_empty,
            year: if s.release_year > 0 { s.release_year } else { i32::MAX },
            firmware_name: s.firmware_name.clone().filter(|f| !f.is_empty()),
            controllers: s.controllers.clone(),
            rumble: s.rumble,
            ..Console::default()
        });
    }
    consoles.sort_by(|a, b| (!a.runs(), a.year, a.name.to_lowercase()).cmp(&(!b.runs(), b.year, b.name.to_lowercase())));
    for c in consoles.iter_mut().filter(|c| c.runs()) {
        c.schema = source.schema(&c.console);
        for row in &c.schema {
            if let Some(v) = source.setting(&c.console, &row.key) {
                c.values.insert(row.key.clone(), v);
            }
        }
        for system in &c.systems {
            c.firmware.extend(source.firmware(system).unwrap_or_default().into_iter().map(|f| (system.clone(), f)));
        }
        c.media = cards::media(&source.cards(&c.console));
        for m in &mut c.media {
            m.opened = m.cards.iter().filter_map(|card| Some((card.path.clone(), serde_json::from_value(source.open_card(&card.path)).ok()?))).collect();
        }
    }
    Ok(ConsoleRead { consoles, cores: source.cores(), overview: source.overview() })
}
