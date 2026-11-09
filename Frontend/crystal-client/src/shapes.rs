//! The JSON shapes emusen_crystal.h names, as typed records read by serde. Keys a later minor adds are skipped until a
//! record names them; a key the header gives as possibly null is an `Option`. The header's comment on each call is the
//! record's definition (EmuSen_Qt.md §1.1.1).

use serde::Deserialize;
use serde_json::Value;

/// A session's id; 0 names the current session.
pub type SessionId = u32;

/// One row of `systems`: the systems table's keys, then the runtime's; a system only an engine names has the table's
/// keys at their defaults.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct SystemRow {
    pub id: String,
    pub name: String,
    pub console: String,
    pub manufacturer: String,
    pub release_year: i32,
    pub extensions: Vec<String>,
    pub cheat_systems: Vec<String>,
    pub openvgdb_systems: Vec<String>,
    pub cover_aspect: f64,
    pub esde_system: String,
    pub esde_full_name: String,
    pub self_delimiting_images: bool,
    pub attached_to: Option<String>,
    pub screenscraper_system: Option<u32>,
    pub display_name: String,
    pub firmware_name: Option<String>,
    pub cheat_space: Option<String>,
    pub media: Vec<Medium>,
    pub byte_order: Option<String>,
    pub libretro_databases: Vec<String>,
    pub libretro_systemids: Vec<String>,
    pub firmware_note: Option<String>,
    pub rumble: bool,
    pub look: Look,
    pub engines: Vec<String>,
    pub starts_empty: bool,
    pub core: Option<CoreOf>,
    pub controllers: Controllers,
}

/// A save medium as the console's manager names it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct Medium {
    pub id: String,
    pub title: String,
    pub noun: String,
}

/// How a console is drawn: its hue, its body and a planet's features.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Look {
    pub hue: f64,
    pub body: String,
    pub palette: Vec<String>,
    pub craters: f64,
    pub bands: f64,
    pub cap: f64,
    pub moons: u32,
}

/// The engine's facts riding on a system's row or a shelf: the names `core <name>` knows it by.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct CoreOf {
    pub names: Vec<String>,
    pub display_name: String,
    pub console: String,
    pub extensions: Vec<String>,
}

/// A console's pad: the canonical controls and axes, its most ports and each player's controller.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Controllers {
    pub buttons: Vec<u32>,
    pub axes: Vec<u32>,
    pub ports: u32,
    pub players: Vec<Option<String>>,
}

/// One row of `sessions`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct SessionRow {
    pub id: SessionId,
    pub engine: String,
    pub current: bool,
}

/// One of the library's shelves.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Shelf {
    pub name: String,
    pub label: String,
    pub system: String,
    pub esde_system: String,
    pub esde_full_name: String,
    pub core: CoreOf,
}

/// One engine discovery found.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Engine {
    pub name: String,
    pub id: String,
    pub version: String,
    pub systems: Vec<String>,
    pub available: bool,
    pub report: Option<String>,
    pub trust: Option<String>,
}

/// One core library found, unloaded, with what the player's approval shows.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct CoreRow {
    pub library: Option<String>,
    pub sidecar: Option<String>,
    pub sha256: Option<String>,
    pub abi: Option<Value>,
    pub development: Option<bool>,
    pub engine: Option<String>,
    pub report: Option<String>,
    pub directory: Option<String>,
    pub trust: Option<String>,
    pub name: Option<String>,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub systems: Vec<String>,
    pub libretro: Option<LibretroCore>,
    pub firmware: Option<String>,
}

/// An installed libretro core on a row of `cores`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct LibretroCore {
    pub folder: String,
    pub file: String,
    pub sha256: String,
    pub corename: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub license: Option<String>,
    pub authors: Vec<String>,
    pub source: Option<String>,
    pub build: Option<String>,
    pub trust: Option<String>,
}

/// What opens a file and with which engine, as `open` would.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Route {
    pub system: Option<String>,
    pub console: Option<String>,
    pub engine: Option<String>,
    pub reason: Option<String>,
    pub cartridge: Option<Value>,
    pub shelf: Option<String>,
    pub waiting_default: Option<WaitingDefault>,
    pub default_held: Option<WaitingDefault>,
}

/// A system's default libretro core that waits for approval, or is held; `reason` only when held.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct WaitingDefault {
    pub stem: String,
    pub sha256: String,
    pub reason: Option<String>,
}

/// The session as a front end shows it.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct SessionInfo {
    pub id: SessionId,
    pub engine: String,
    pub system: String,
    pub console: String,
    pub game: Option<Game>,
    pub run: String,
    pub halted: Option<Halted>,
    pub ports: Vec<Port>,
    pub buttons: Vec<u32>,
    pub axes: Vec<u32>,
    pub discs: Discs,
    pub picture: Picture,
    pub frame_rate_hz: f64,
    pub total_frames: u64,
    pub state_version: Option<u32>,
    pub rewind_capture: Option<bool>,
    pub rewind_withheld: Option<String>,
    pub notices: Notices,
    pub state: Option<StateFormat>,
    pub television_standard: Option<String>,
}

/// The game a session runs.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct Game {
    pub path: String,
    pub name: String,
}

/// Where a halt stopped.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct Halted {
    pub processor: u32,
    pub address: u64,
}

/// A port and the controller in it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct Port {
    pub port: u32,
    pub controller: Option<String>,
}

/// The session's discs.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Discs {
    pub count: u32,
    pub index: Option<u32>,
    pub can_change: bool,
    pub lid_open: bool,
}

/// The picture's shape.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Picture {
    pub width: i32,
    pub height: i32,
    pub row_repeat: i32,
}

/// The words a session carries for the player, each possibly none.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Notices {
    pub engine: Option<String>,
    pub firmware: Option<String>,
    pub media: Option<String>,
}

/// The machine's state format and the versions it loads.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct StateFormat {
    pub format: Option<String>,
    pub version: Option<u32>,
    pub loads_from: Vec<u32>,
}

/// One row of a console's settings schema.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct SettingRow {
    pub key: String,
    pub label: String,
    pub hint: Option<String>,
    pub kind: String,
    pub default: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub choices: Option<Vec<String>>,
    pub choice_labels: Option<Vec<String>>,
    pub group: Option<String>,
    pub note: Option<String>,
    pub shown_with: Option<ShownWith>,
    /// The running engine holds the value, its note saying why.
    pub locked: bool,
}

/// The key and value a setting means something only with.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct ShownWith {
    pub key: String,
    pub value: String,
}

/// A firmware file a game needs and the library does not hold.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct FirmwareNeed {
    pub core: String,
    pub chip: Option<String>,
    pub file: String,
    pub size: u64,
    pub purpose: Option<String>,
    pub alternate_names: Vec<String>,
    pub parts: Option<Value>,
    pub required: bool,
    pub replacement: Option<Replacement>,
    /// The entry as the runtime wrote it, which `firmware_install` takes back.
    #[serde(skip)]
    pub request: Value,
}

/// What an open replacement does in place of a firmware file, and its cost.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Replacement {
    pub effect: Option<String>,
    pub cost: Option<String>,
}

/// Each discovered system's firmware.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct FirmwareOverview {
    pub folder: String,
    pub folders: Vec<String>,
    pub systems: Vec<FirmwareSystem>,
}

/// One system of the overview.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct FirmwareSystem {
    pub name: String,
    pub engine: Option<String>,
    pub items: Vec<FirmwareItem>,
}

/// One firmware file of the overview.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct FirmwareItem {
    pub label: String,
    pub file: String,
    pub size: u64,
    pub present: bool,
    pub wrong_size: bool,
    pub effect: Option<String>,
    pub cost: Option<String>,
    pub found_in: Option<String>,
    pub wrong_size_in: Option<String>,
}

/// Each firmware file a system's engines declare, and whether the library holds it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct FirmwareStatus {
    pub system: String,
    pub firmware: Vec<FirmwareFile>,
}

/// One file of a system's firmware status; `status` is "present", "missing" or "not_needed".
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct FirmwareFile {
    pub label: String,
    pub file: String,
    pub required: bool,
    pub status: String,
    pub need: String,
}

/// The cheat codecs' catalogue and each system's pair.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct CheatCodecs {
    pub codecs: Vec<CheatCodec>,
    pub systems: Vec<CheatPair>,
}

/// One cheat codec; `kind` is "RamPoke" or "RomPatch".
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct CheatCodec {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub space: Option<String>,
}

/// A system's automatic and explicit codec.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct CheatPair {
    pub id: String,
    pub auto: Option<String>,
    pub explicit: Option<String>,
}

/// A code's answers from one codec; each part a refusal's words where the codec refused it.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct CheatDecode {
    pub can: Value,
    pub decode: Value,
    pub compare: Value,
    pub writes: Value,
}

/// One state slot, 0 the resume state.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct StateSlot {
    pub slot: u32,
    pub path: String,
    pub exists: bool,
    pub saved_at: Option<String>,
}

/// The record states.db keeps of a state file.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct StateRecord {
    pub console: Option<String>,
    pub core: Option<String>,
    pub state_version: Option<u32>,
    pub build: Option<String>,
    pub saved_at: Option<String>,
    pub rom_file: Option<String>,
    pub rom_md5: Option<String>,
    pub rom_bytes: Option<u64>,
}

/// The rewind history.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct RewindReel {
    pub enabled: bool,
    pub interval_frames: u32,
    pub budget_bytes: u64,
    pub depth: u32,
    pub buffered_bytes: u64,
    pub snapshot_bytes: u64,
    pub seconds: f64,
    pub moments: Vec<Moment>,
}

/// One moment of the history, oldest first.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Moment {
    pub frame: u64,
    pub core_frames: u64,
    pub picture: Option<Picture>,
}

/// The session's save media.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct SaveMedia {
    pub notice: Option<String>,
    pub media: Vec<SaveMedium>,
}

/// One medium of a session; `kind` "removable" or "console", `mode` "game", "shared" or "named".
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct SaveMedium {
    pub medium: String,
    pub title: String,
    pub noun: String,
    pub extension: String,
    pub kind: String,
    pub mode: String,
    pub slots: Vec<SaveSlot>,
}

/// A slot of a medium and its file.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct SaveSlot {
    pub label: String,
    pub name: String,
    pub path: String,
}

/// The session service's own reading of a session.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct ServiceInfo {
    pub id: SessionId,
    pub engine: String,
    pub frame: u64,
    pub max_sprites: Option<u32>,
    pub tilemap_entry_stride: Option<u32>,
    pub capabilities: Vec<String>,
}

/// A session's cheat codecs as its table answers them.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct SessionCodecs {
    pub auto: Option<CheatCodec>,
    pub explicit: Option<CheatCodec>,
    pub systems: Vec<String>,
}
