//! The descriptors a core returns as Rust values, and the crate's writers that turn them into the JSON of
//! EmuSen_CoreAPI.md §6.3, §6.4 and §6.13, validated by `abi/v1/*.schema.json`. No core writes JSON itself.

use super::jsonw::Json;
use super::sys::{self, caps};
use crate::abi::Region;

/// `PadButton`'s names, append-only; the value is `enum emusen_control`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Control {
    B,
    Y,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
    A,
    X,
    L,
    R,
    L2,
    R2,
    L3,
    R3,
}

impl Control {
    pub const fn name(self) -> &'static str {
        ["B", "Y", "Select", "Start", "Up", "Down", "Left", "Right", "A", "X", "L", "R", "L2", "R2", "L3", "R3"][self as usize]
    }
}

/// `PadAxis`'s names; the value is `enum emusen_axis`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Axis {
    LeftX,
    LeftY,
    RightX,
    RightY,
    LeftTrigger,
    RightTrigger,
}

impl Axis {
    pub const fn name(self) -> &'static str {
        ["LeftX", "LeftY", "RightX", "RightY", "LeftTrigger", "RightTrigger"][self as usize]
    }

    pub const fn is_trigger(self) -> bool {
        matches!(self, Axis::LeftTrigger | Axis::RightTrigger)
    }
}

fn region_name(r: Region) -> &'static str {
    match r {
        Region::Ntsc => "ntsc",
        Region::Pal => "pal",
    }
}

/// Core info (§6.3); `abi` and `capabilities` are the crate's to write, from the version and `CAPABILITIES`.
#[derive(Clone, Debug, Default)]
pub struct Info {
    pub id: String,
    pub name: String,
    pub display_name: Option<String>,
    pub version: String,
    pub license: String,
    pub authors: Vec<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub systems: Vec<System>,
    pub host_requires: Vec<String>,
    pub deterministic: bool,
    pub accuracy: Option<Accuracy>,
}

#[derive(Clone, Debug, Default)]
pub struct Accuracy {
    /// `"defaults"` for a record taken with no settings (§12.4).
    pub measured_with: String,
    pub suite: String,
    pub notes: String,
}

#[derive(Clone, Debug, Default)]
pub struct System {
    pub id: String,
    pub name: String,
    pub extensions: Vec<String>,
    pub regions: Vec<Region>,
    pub controllers: Vec<Controller>,
    pub firmware: Vec<Firmware>,
}

#[derive(Clone, Debug, Default)]
pub struct Controller {
    pub id: String,
    pub label: String,
    pub ports: Vec<u32>,
    pub buttons: Vec<Button>,
    pub axes: Vec<AxisControl>,
}

/// A pad bit: `control` is its canonical name, `None` for an extra the frontend offers unbound.
#[derive(Clone, Debug)]
pub struct Button {
    pub bit: u32,
    pub control: Option<Control>,
    pub label: String,
}

/// An axis as `set_axis` receives it: `axis` is the number passed, `trigger` whether it runs 0 to 1.
#[derive(Clone, Debug)]
pub struct AxisControl {
    pub axis: u32,
    pub control: Option<Axis>,
    pub trigger: bool,
    pub label: String,
}

/// A firmware file (§6.2): passed as file `which`, or as `parts` numbered from `which` upwards.
#[derive(Clone, Debug, Default)]
pub struct Firmware {
    pub which: u32,
    pub name: String,
    pub label: String,
    pub size: u64,
    pub required: bool,
    pub parts: Vec<Vec<String>>,
    /// What the core runs in the file's place when it is absent; `None` reads as no replacement (§6.2).
    pub replacement: Option<Replacement>,
}

/// An open replacement for a firmware file, and what running on it costs, in the settings schema's words (§6.13).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Replacement {
    Exact,
    Accuracy { cost: String },
    /// No replacement: the game runs without the chip, `cost` saying so.
    None { cost: String },
}

/// Which firmware path a machine runs for file `which` (§6.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FirmwareSource {
    File,
    Replacement,
    Absent,
}

/// Machine info (§6.4). Space sizes, battery lengths and state kinds are filled by the crate from the exports.
#[derive(Clone, Debug, Default)]
pub struct MachineInfo {
    pub system: String,
    pub region: Option<Region>,
    pub frame_rate: (u64, u64),
    pub video: Video,
    pub audio: Audio,
    pub ports: Vec<Port>,
    pub spaces: Vec<Space>,
    pub processors: Vec<Processor>,
    pub battery: Vec<Battery>,
    pub state: StateFormat,
    pub phases: Vec<String>,
    pub patches: Option<(u32, u32)>,
    pub skip_rendering_state_neutral: bool,
    /// Each firmware file the game names and the path it runs on.
    pub firmware: Vec<(u32, FirmwareSource)>,
}

#[derive(Clone, Debug, Default)]
pub struct Video {
    pub base_width: u32,
    pub base_height: u32,
    pub max_width: u32,
    pub max_height: u32,
    /// The television's aspect, `(0, 0)` for square pixels.
    pub aspect: (u32, u32),
    /// `emusen_pixel_format` values the core can produce; empty means RGBA8888 only.
    pub formats: Vec<u32>,
}

#[derive(Clone, Debug, Default)]
pub struct Audio {
    pub rate: i32,
    /// What `set_mutes`' bit n stands for.
    pub channels: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Port {
    pub port: u32,
    /// A controller id from core info, `None` for an empty port.
    pub controller: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Space {
    pub id: u32,
    pub name: String,
    pub read_only: bool,
    pub side_effects: bool,
    pub reports_stores: bool,
    pub cheats: bool,
}

impl Space {
    pub fn new(id: u32, name: &str) -> Space {
        Space { id, name: name.to_owned(), read_only: false, side_effects: false, reports_stores: false, cheats: false }
    }
}

#[derive(Clone, Debug)]
pub struct Processor {
    pub id: u32,
    pub name: String,
    pub pc_bits: u32,
    /// `(name, bits)` in `debug_registers`' order.
    pub registers: Vec<(String, u32)>,
    /// The space its code is listed from, by id; absent, the host chooses by name (EmuSen_CoreAPI.md §6.4).
    pub code_space: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct Battery {
    pub which: u32,
    pub suffix: String,
}

#[derive(Clone, Debug, Default)]
pub struct StateFormat {
    /// The magic a frontend records beside a state.
    pub format: String,
    pub version: i64,
    /// The versions `state_load` reads.
    pub loads_from: Vec<i64>,
}

/// One setting (§6.13).
#[derive(Clone, Debug)]
pub struct Setting {
    pub key: String,
    pub label: String,
    pub help: String,
    pub kind: SettingKind,
    pub default: String,
    pub scope: Scope,
    pub category: Option<String>,
    pub effect: Effect,
    pub advanced: bool,
    pub hidden: bool,
    pub restart: bool,
}

#[derive(Clone, Debug)]
pub enum SettingKind {
    /// `"true"` or `"false"`, as `CoreSetting` writes a switch.
    Switch,
    Count { min: i64, max: i64, step: i64 },
    Choice(Vec<Choice>),
    Text,
}

#[derive(Clone, Debug)]
pub struct Choice {
    pub value: String,
    pub label: String,
    pub help: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// Read only at create.
    Create,
    /// Applied between frames by `set_settings`.
    Run,
}

/// What a setting trades; the accurate default is the rule for `Accuracy` and `Enhancement` (§2.1, §15 Q12).
#[derive(Clone, Debug)]
pub enum Effect {
    None,
    Exact,
    Accuracy { cost: String, accurate: String },
    Latency { cost: String },
    Enhancement { cost: String, hardware: String },
}

impl Setting {
    /// Whether `value` lies in this setting's domain.
    pub fn accepts(&self, value: &str) -> bool {
        match &self.kind {
            SettingKind::Switch => value == "true" || value == "false",
            SettingKind::Count { min, max, .. } => value.parse::<i64>().is_ok_and(|v| v >= *min && v <= *max),
            SettingKind::Choice(choices) => choices.iter().any(|c| c.value == value),
            SettingKind::Text => !value.contains(['\r', '\n']),
        }
    }

    /// C3's rules for one setting: its key, its default in its own domain, and an accurate default.
    pub fn check(&self) -> Result<(), String> {
        if self.key.is_empty() || !self.key.bytes().all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b)) {
            return Err(format!("the key {:?} is not [A-Za-z0-9_.-]+", self.key));
        }
        if !self.accepts(&self.default) {
            return Err(format!("{}: the default {:?} is outside its own domain", self.key, self.default));
        }
        match &self.effect {
            Effect::Accuracy { accurate, .. } if *accurate != self.default => Err(format!("{}: an accuracy setting's default must be its accurate value", self.key)),
            Effect::Enhancement { hardware, .. } if *hardware != self.default => Err(format!("{}: an enhancement's default must be the hardware's value", self.key)),
            Effect::Accuracy { cost, .. } | Effect::Latency { cost } | Effect::Enhancement { cost, .. } if cost.trim().is_empty() => Err(format!("{}: a trade-off needs its cost", self.key)),
            _ => Ok(()),
        }
    }
}

/// One instruction as `debug_disassemble` reports it; the core decodes, DianaOS formats.
#[derive(Clone, Debug)]
pub struct Instruction {
    pub address: u32,
    pub bytes: Vec<u8>,
    pub mnemonic: String,
    pub operands: String,
    /// A static reference knowable from the bytes alone: its kind and target.
    pub reference: Option<(Reference, u32)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reference {
    Call,
    Write,
    Read,
}

/// The names of the bits set in `bits`, for core info's `capabilities`.
pub fn capability_names(bits: u64) -> Vec<&'static str> {
    sys::CAPABILITY_NAMES.iter().filter(|(b, _)| bits & b != 0).map(|&(_, n)| n).collect()
}

fn write_firmware(j: &mut Json, f: &Firmware) {
    j.begin_object().field_uint("which", f.which as u64).field_str("name", &f.name).field_str("label", &f.label).field_uint("size", f.size).field_bool("required", f.required);
    j.key("parts").begin_array();
    for alt in &f.parts {
        j.begin_array();
        for p in alt {
            j.str(p);
        }
        j.end_array();
    }
    j.end_array();
    if let Some(r) = &f.replacement {
        j.key("replacement").begin_object();
        match r {
            Replacement::Exact => j.field_str("effect", "exact"),
            Replacement::Accuracy { cost } => j.field_str("effect", "accuracy").field_str("cost", cost),
            Replacement::None { cost } => j.field_str("effect", "none").field_str("cost", cost),
        };
        j.end_object();
    }
    j.end_object();
}

/// Core info as JSON.
pub fn info_json(info: &Info, capabilities: u64) -> String {
    let mut j = Json::new();
    j.begin_object();
    j.field_str("abi", &format!("{}.{}", sys::ABI_MAJOR, sys::ABI_MINOR));
    j.field_str("id", &info.id).field_str("name", &info.name);
    if let Some(d) = &info.display_name {
        j.field_str("display_name", d);
    }
    j.field_str("version", &info.version).field_str("license", &info.license);
    j.field_strs("authors", &info.authors);
    if let Some(u) = &info.url {
        j.field_str("url", u);
    }
    if let Some(d) = &info.description {
        j.field_str("description", d);
    }
    j.key("systems").begin_array();
    for s in &info.systems {
        j.begin_object().field_str("id", &s.id).field_str("name", &s.name).field_strs("extensions", &s.extensions);
        j.field_strs("regions", &s.regions.iter().map(|&r| region_name(r)).collect::<Vec<_>>());
        j.key("controllers").begin_array();
        for c in &s.controllers {
            j.begin_object().field_str("id", &c.id).field_str("label", &c.label);
            j.key("ports").begin_array();
            for &p in &c.ports {
                j.uint(p as u64);
            }
            j.end_array();
            j.key("buttons").begin_array();
            for b in &c.buttons {
                j.begin_object().field_uint("bit", b.bit as u64).key("control").opt_str(b.control.map(Control::name)).field_str("label", &b.label).end_object();
            }
            j.end_array();
            j.key("axes").begin_array();
            for a in &c.axes {
                j.begin_object().field_uint("axis", a.axis as u64).key("control").opt_str(a.control.map(Axis::name));
                j.field_str("kind", if a.trigger { "trigger" } else { "stick" }).field_str("label", &a.label).end_object();
            }
            j.end_array().end_object();
        }
        j.end_array();
        j.key("firmware").begin_array();
        for f in &s.firmware {
            write_firmware(&mut j, f);
        }
        j.end_array().end_object();
    }
    j.end_array();
    j.field_strs("capabilities", &capability_names(capabilities));
    j.field_strs("host_requires", &info.host_requires);
    j.field_bool("deterministic", info.deterministic);
    if let Some(a) = &info.accuracy {
        j.key("accuracy").begin_object().field_str("measured_with", &a.measured_with).field_str("suite", &a.suite).field_str("notes", &a.notes).end_object();
    }
    j.end_object();
    j.finish()
}

/// `firmware_for`'s answer.
pub fn firmware_json(list: &[Firmware]) -> String {
    let mut j = Json::new();
    j.begin_array();
    for f in list {
        write_firmware(&mut j, f);
    }
    j.end_array();
    j.finish()
}

/// What the crate reads from the exports to fill machine info, so that it cannot disagree with them.
pub struct Measured<'a> {
    pub space_size: &'a dyn Fn(u32) -> Option<i64>,
    pub battery_len: &'a dyn Fn(u32) -> Option<i64>,
    pub capabilities: u64,
}

/// Machine info as JSON.
pub fn machine_info_json(m: &MachineInfo, measured: &Measured<'_>) -> String {
    let mut j = Json::new();
    j.begin_object();
    j.field_str("system", &m.system);
    if let Some(r) = m.region {
        j.field_str("region", region_name(r));
    }
    j.key("frame_rate").begin_object().field_uint("num", m.frame_rate.0).field_uint("den", m.frame_rate.1).end_object();
    let v = &m.video;
    j.key("video").begin_object().field_uint("base_width", v.base_width as u64).field_uint("base_height", v.base_height as u64);
    j.field_uint("max_width", v.max_width as u64).field_uint("max_height", v.max_height as u64);
    j.key("aspect").begin_object().field_uint("num", v.aspect.0 as u64).field_uint("den", v.aspect.1 as u64).end_object();
    j.key("formats").begin_array();
    if v.formats.is_empty() {
        j.uint(sys::pixel::RGBA8888 as u64);
    }
    for &f in &v.formats {
        j.uint(f as u64);
    }
    j.end_array().end_object();
    j.key("audio").begin_object().field_int("rate", m.audio.rate as i64).field_strs("channels", &m.audio.channels).end_object();
    j.key("ports").begin_array();
    for p in &m.ports {
        j.begin_object().field_uint("port", p.port as u64).key("controller").opt_str(p.controller.as_deref()).end_object();
    }
    j.end_array();
    j.key("spaces").begin_array();
    for s in &m.spaces {
        let flags: Vec<&str> = [(s.read_only, "read_only"), (s.side_effects, "side_effects"), (s.reports_stores, "reports_stores"), (s.cheats, "cheats")]
            .iter()
            .filter(|f| f.0)
            .map(|f| f.1)
            .collect();
        j.begin_object().field_uint("id", s.id as u64).field_str("name", &s.name).field_int("size", (measured.space_size)(s.id).unwrap_or(0)).field_strs("flags", &flags).end_object();
    }
    j.end_array();
    j.key("processors").begin_array();
    for p in &m.processors {
        j.begin_object().field_uint("id", p.id as u64).field_str("name", &p.name).field_uint("pc_bits", p.pc_bits as u64);
        if let Some(space) = p.code_space {
            j.field_uint("code_space", space as u64);
        }
        j.key("registers").begin_array();
        for (name, bits) in &p.registers {
            j.begin_object().field_str("name", name).field_uint("bits", *bits as u64).end_object();
        }
        j.end_array().end_object();
    }
    j.end_array();
    j.key("battery").begin_array();
    for b in &m.battery {
        j.begin_object().field_uint("which", b.which as u64).field_str("suffix", &b.suffix).field_int("length", (measured.battery_len)(b.which).unwrap_or(0)).end_object();
    }
    j.end_array();
    j.key("state").begin_object().field_str("format", &m.state.format).field_int("version", m.state.version);
    j.key("kinds").begin_array().uint(sys::kind::FULL as u64);
    if measured.capabilities & caps::SNAPSHOT != 0 {
        j.uint(sys::kind::SNAPSHOT as u64);
    }
    j.end_array();
    j.key("loads_from").begin_array();
    for &v in &m.state.loads_from {
        j.int(v);
    }
    j.end_array().end_object();
    j.field_strs("phases", &m.phases);
    if let Some((low, high)) = m.patches {
        j.key("patches").begin_object().field_uint("low", low as u64).field_uint("high", high as u64).end_object();
    }
    j.field_bool("skip_rendering_state_neutral", m.skip_rendering_state_neutral);
    if !m.firmware.is_empty() {
        j.key("firmware").begin_array();
        for &(which, source) in &m.firmware {
            let source = match source {
                FirmwareSource::File => "file",
                FirmwareSource::Replacement => "replacement",
                FirmwareSource::Absent => "absent",
            };
            j.begin_object().field_uint("which", which as u64).field_str("source", source).end_object();
        }
        j.end_array();
    }
    j.end_object();
    j.finish()
}

/// The settings schema as JSON.
pub fn settings_json(settings: &[Setting]) -> String {
    let mut j = Json::new();
    j.begin_array();
    for s in settings {
        j.begin_object().field_str("key", &s.key).field_str("label", &s.label).field_str("help", &s.help);
        match &s.kind {
            SettingKind::Switch => {
                j.field_str("kind", "switch");
            }
            SettingKind::Count { min, max, step } => {
                j.field_str("kind", "count").field_int("min", *min).field_int("max", *max).field_int("step", *step);
            }
            SettingKind::Choice(choices) => {
                j.field_str("kind", "choice").key("choices").begin_array();
                for c in choices {
                    j.begin_object().field_str("value", &c.value).field_str("label", &c.label);
                    if let Some(h) = &c.help {
                        j.field_str("help", h);
                    }
                    j.end_object();
                }
                j.end_array();
            }
            SettingKind::Text => {
                j.field_str("kind", "text");
            }
        }
        j.field_str("default", &s.default).field_str("scope", if s.scope == Scope::Create { "create" } else { "run" });
        if let Some(c) = &s.category {
            j.field_str("category", c);
        }
        match &s.effect {
            Effect::None => j.field_str("effect", "none"),
            Effect::Exact => j.field_str("effect", "exact"),
            Effect::Accuracy { cost, accurate } => j.field_str("effect", "accuracy").field_str("cost", cost).field_str("accurate", accurate),
            Effect::Latency { cost } => j.field_str("effect", "latency").field_str("cost", cost),
            Effect::Enhancement { cost, hardware } => j.field_str("effect", "enhancement").field_str("cost", cost).field_str("hardware", hardware),
        };
        j.field_bool("advanced", s.advanced).field_bool("hidden", s.hidden).field_bool("restart", s.restart).end_object();
    }
    j.end_array();
    j.finish()
}

/// `setting_notes`' object, key to sentence.
pub fn notes_json(notes: &[(String, String)]) -> String {
    let mut j = Json::new();
    j.begin_object();
    for (k, v) in notes {
        j.field_str(k, v);
    }
    j.end_object();
    j.finish()
}

/// `debug_disassemble`'s records.
pub fn disassembly_json(list: &[Instruction]) -> String {
    let mut j = Json::new();
    j.begin_array();
    for i in list {
        j.begin_object().field_uint("address", i.address as u64);
        j.key("bytes").begin_array();
        for &b in &i.bytes {
            j.uint(b as u64);
        }
        j.end_array().field_str("mnemonic", &i.mnemonic).field_str("operands", &i.operands);
        j.key("reference");
        match i.reference {
            Some((kind, target)) => {
                let k = match kind {
                    Reference::Call => "call",
                    Reference::Write => "write",
                    Reference::Read => "read",
                };
                j.begin_object().field_str("kind", k).field_uint("target", target as u64).end_object();
            }
            None => {
                j.null();
            }
        }
        j.end_object();
    }
    j.end_array();
    j.finish()
}
