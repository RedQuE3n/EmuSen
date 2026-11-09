//! Settings ▸ Consoles' source over the frontend API (EmuSen_Qt.md §7.7.E): the runtime's systems, cores, settings
//! schema and values, firmware status and overview, and the card manager's list, each through `emusen-crystal-client`.

use crate::consoles::ConsoleSource;
use emusen_crystal_client::shapes::{CoreRow, FirmwareFile, FirmwareItem, SettingRow, SystemRow};
use emusen_crystal_client::consts::{CARDS_LIST, CARDS_NEW};
use emusen_crystal_client::{Runtime, Scope};
use serde_json::{json, Value};

pub struct ClientConsoles {
    pub runtime: Runtime,
}

fn words(e: impl std::fmt::Display) -> String {
    e.to_string()
}

impl ClientConsoles {
    /// The runtime opened on the platform library, with the cores folder a test or a build names (`clients::runtime_config`).
    pub fn open(platform_library: &str, cores: &str) -> Result<ClientConsoles, String> {
        let platform = emusen_platform_client::Platform::load(platform_library).map_err(words)?;
        let config = crate::clients::runtime_config(&platform, cores)?;
        let api = emusen_crystal_client::Api::load(platform_library).map_err(words)?;
        Ok(ClientConsoles { runtime: Runtime::new(&api, &config).map_err(words)? })
    }

    /// A console's setting stored, or forgotten with none.
    pub fn set(&self, console: &str, key: &str, value: Option<&str>) -> Result<(), String> {
        self.runtime.settings_set(0, Scope::Console, Some(console), key, value).map_err(words)
    }

    /// A formatted card made at `path` by the card manager, which answers in words whether it was.
    pub fn new_card(&self, path: &str) -> Result<(), String> {
        let answer = self.runtime.cards_op(CARDS_NEW, &json!({ "path": path })).map_err(words)?.unwrap_or(Value::Null);
        if answer["done"] == true { Ok(()) } else { Err(answer["words"].as_str().unwrap_or("The card was not made.").to_owned()) }
    }
}

impl ConsoleSource for ClientConsoles {
    fn systems(&self) -> Result<Vec<SystemRow>, String> {
        self.runtime.systems().map_err(words)
    }

    fn cores(&self) -> Vec<CoreRow> {
        self.runtime.cores().unwrap_or_default()
    }

    fn schema(&self, console: &str) -> Vec<SettingRow> {
        self.runtime.settings_schema(0, Some(console), None).unwrap_or_default()
    }

    fn setting(&self, console: &str, key: &str) -> Option<String> {
        self.runtime.settings_get(0, Scope::Console, Some(console), key).ok().flatten()
    }

    fn firmware(&self, system: &str) -> Option<Vec<FirmwareFile>> {
        self.runtime.firmware_status(system).ok().flatten().map(|s| s.firmware)
    }

    fn overview(&self) -> Vec<FirmwareItem> {
        self.runtime.firmware_overview().map(|o| o.systems.into_iter().flat_map(|s| s.items).collect()).unwrap_or_default()
    }

    fn cards(&self, console: &str) -> Value {
        self.runtime.cards_op(CARDS_LIST, &json!({ "console": console })).ok().flatten().unwrap_or(Value::Null)
    }
}

/// The consoles as a file holds them in place of the runtime, as Mistress's recordings script their world: `systems`,
/// `cores`, each console's `schema` and stored `settings`, each system's `firmware`, the `overview`'s files and each
/// console's `cards` (EmuSen_Qt.md §7.7.E). A change is kept in the rows read, never written to the file.
pub struct ScriptedConsoles {
    world: Value,
}

impl ScriptedConsoles {
    pub fn open(path: &str) -> Result<ScriptedConsoles, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        Ok(ScriptedConsoles { world: serde_json::from_str(&text).map_err(|e| format!("{path}: {e}"))? })
    }

    fn of<T: serde::de::DeserializeOwned + Default>(&self, value: &Value) -> T {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }
}

impl ConsoleSource for ScriptedConsoles {
    fn systems(&self) -> Result<Vec<SystemRow>, String> {
        Ok(self.of(&self.world["systems"]))
    }

    fn cores(&self) -> Vec<CoreRow> {
        self.of(&self.world["cores"])
    }

    fn schema(&self, console: &str) -> Vec<SettingRow> {
        self.of(&self.world["schema"][console])
    }

    fn setting(&self, console: &str, key: &str) -> Option<String> {
        self.world["settings"][console][key].as_str().map(str::to_owned)
    }

    fn firmware(&self, system: &str) -> Option<Vec<FirmwareFile>> {
        Some(self.of(&self.world["firmware"][system]))
    }

    fn overview(&self) -> Vec<FirmwareItem> {
        self.of(&self.world["overview"])
    }

    fn cards(&self, console: &str) -> Value {
        self.world["cards"][console].clone()
    }
}
