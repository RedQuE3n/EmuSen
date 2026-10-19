//! Machine info and the settings schema as the runtime reads them, the C# `CoreDescriptorReader.MachineInfo` and
//! `Settings`: a field it does not know ignored, one absent taken at its default (EmuSen_CoreAPI.md §6.3, §6.5).

use emusen_native::json::{self, Value};

use crate::info::BatteryKind;

fn arr<'a>(v: Option<&'a Value>, key: &str) -> &'a [Value] {
    v.and_then(|v| v.get(key)).and_then(Value::as_array).unwrap_or(&[])
}

fn int(v: Option<&Value>, key: &str, fallback: i64) -> i64 {
    v.and_then(|v| v.get(key)).and_then(Value::as_i64).unwrap_or(fallback)
}

fn text<'a>(v: Option<&'a Value>, key: &str) -> Option<&'a str> {
    v.and_then(|v| v.get(key)).and_then(Value::as_str)
}

fn obj<'a>(v: Option<&'a Value>, key: &str) -> Option<&'a Value> {
    v.and_then(|v| v.get(key)).filter(|o| matches!(o, Value::Object(_)))
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Port {
    pub port: u32,
    pub controller: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Drive {
    pub drive: u32,
    pub lid_open: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MachineBattery {
    pub which: u32,
    pub suffix: String,
    pub kind: BatteryKind,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Space {
    pub id: u32,
    pub name: String,
    pub byte_order: Option<String>,
}

/// What a machine says of itself.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MachineInfo {
    pub system: String,
    pub region: Option<String>,
    pub rate_num: u64,
    pub rate_den: u64,
    pub aspect_num: u32,
    pub aspect_den: u32,
    pub ports: Vec<Port>,
    pub spaces: Vec<Space>,
    pub battery: Vec<MachineBattery>,
    pub state_format: String,
    pub state_version: i64,
    pub loads_from: Vec<i64>,
    pub patch_low: Option<i64>,
    pub patch_high: Option<i64>,
    pub drives: Vec<Drive>,
    /// Each firmware entry's source, `absent` where none says.
    pub firmware: Vec<(u32, String)>,
    pub processors: Vec<(u32, String)>,
    /// The frame's phases' names, in the order `phases` times them.
    pub phases: Vec<String>,
    /// The raw pictures' encodings, in their order.
    pub raw: Vec<String>,
}

impl MachineInfo {
    pub fn read(text_json: &str) -> MachineInfo {
        let doc = json::parse(text_json.as_bytes()).unwrap_or(Value::Null);
        let r = Some(&doc);
        let rate = obj(r, "frame_rate");
        let video = obj(r, "video");
        let aspect = obj(video, "aspect");
        let state = obj(r, "state");
        let patches = obj(r, "patches");
        MachineInfo {
            system: text(r, "system").unwrap_or("").to_string(),
            region: text(r, "region").map(str::to_string),
            rate_num: int(rate, "num", 60) as u64,
            rate_den: int(rate, "den", 1) as u64,
            aspect_num: int(aspect, "num", 0) as u32,
            aspect_den: int(aspect, "den", 0) as u32,
            ports: arr(r, "ports").iter().map(|p| Port { port: int(Some(p), "port", 0) as u32, controller: text(Some(p), "controller").map(str::to_string) }).collect(),
            spaces: arr(r, "spaces")
                .iter()
                .map(|s| Space { id: int(Some(s), "id", 0) as u32, name: text(Some(s), "name").unwrap_or("").to_string(), byte_order: text(Some(s), "byte_order").map(str::to_string) })
                .collect(),
            battery: arr(r, "battery")
                .iter()
                .map(|b| MachineBattery {
                    which: int(Some(b), "which", 0) as u32,
                    suffix: text(Some(b), "suffix").unwrap_or(".srm").to_string(),
                    kind: BatteryKind::named(text(Some(b), "kind")),
                })
                .collect(),
            state_format: text(state, "format").unwrap_or("").to_string(),
            state_version: int(state, "version", 0),
            loads_from: arr(state, "loads_from").iter().filter_map(Value::as_i64).collect(),
            patch_low: patches.map(|p| int(Some(p), "low", 0)),
            patch_high: patches.map(|p| int(Some(p), "high", 0)),
            drives: arr(r, "drives").iter().map(|d| Drive { drive: int(Some(d), "drive", 0) as u32, lid_open: text(Some(d), "lid") == Some("open") }).collect(),
            firmware: arr(r, "firmware").iter().map(|f| (int(Some(f), "which", 0) as u32, text(Some(f), "source").unwrap_or("absent").to_string())).collect(),
            phases: arr(r, "phases").iter().filter_map(Value::as_str).map(str::to_string).collect(),
            raw: arr(video, "raw").iter().map(|e| text(Some(e), "encoding").unwrap_or("").to_string()).collect(),
            processors: arr(r, "processors").iter().map(|p| (int(Some(p), "id", 0) as u32, text(Some(p), "name").unwrap_or("").to_string())).collect(),
        }
    }

    pub fn frame_rate_hz(&self) -> f64 {
        if self.rate_den == 0 { 60.0 } else { self.rate_num as f64 / self.rate_den as f64 }
    }
}

/// One setting of a schema, `CoreSettingDescriptor`.
#[derive(Clone, Debug, PartialEq)]
pub struct SettingDesc {
    pub key: String,
    pub label: String,
    pub help: String,
    pub kind: String,
    pub default: String,
    pub min: i64,
    pub max: i64,
    pub choices: Vec<String>,
    pub choice_labels: Vec<String>,
    pub create_scope: bool,
    pub effect: String,
    pub cost: Option<String>,
    pub hidden: bool,
}

impl SettingDesc {
    /// Whether the value is one the setting takes, `Accepts`.
    pub fn accepts(&self, value: &str) -> bool {
        match self.kind.as_str() {
            "switch" => value == "true" || value == "false",
            "count" => emusen_dotnet::number::parse_i32(value).is_some_and(|n| n as i64 >= self.min && n as i64 <= self.max),
            "choice" => self.choices.iter().any(|c| c == value),
            _ => !value.contains('\n') && !value.contains('\r'),
        }
    }

    /// Whether a window lists it, `AsCoreSetting` not null.
    pub fn listed(&self) -> bool {
        !self.hidden && matches!(self.kind.as_str(), "switch" | "count" | "choice")
    }
}

/// A schema's settings, every one, in its order.
pub fn settings(text_json: &str) -> Vec<SettingDesc> {
    let doc = json::parse(text_json.as_bytes()).unwrap_or(Value::Null);
    doc.as_array()
        .unwrap_or(&[])
        .iter()
        .map(|s| {
            let s = Some(s);
            let choices = arr(s, "choices");
            SettingDesc {
                key: text(s, "key").unwrap_or("").to_string(),
                label: text(s, "label").unwrap_or("").to_string(),
                help: text(s, "help").unwrap_or("").to_string(),
                kind: text(s, "kind").unwrap_or("").to_string(),
                default: text(s, "default").unwrap_or("").to_string(),
                min: int(s, "min", 0) as i32 as i64,
                max: int(s, "max", 0) as i32 as i64,
                choices: choices.iter().map(|c| text(Some(c), "value").unwrap_or("").to_string()).collect(),
                choice_labels: choices.iter().map(|c| text(Some(c), "label").or(text(Some(c), "value")).unwrap_or("").to_string()).collect(),
                create_scope: text(s, "scope") == Some("create"),
                effect: text(s, "effect").unwrap_or("none").to_string(),
                cost: text(s, "cost").map(str::to_string),
                hidden: matches!(s.and_then(|v| v.get("hidden")), Some(Value::Bool(true))),
            }
        })
        .collect()
}

/// `key -> sentence`, from `setting_notes`.
pub fn notes(text_json: &str) -> Vec<(String, String)> {
    match json::parse(text_json.as_bytes()) {
        Ok(Value::Object(members)) => members.into_iter().filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string()))).collect(),
        _ => Vec::new(),
    }
}
