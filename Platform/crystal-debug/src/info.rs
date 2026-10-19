//! The fields of machine info the debugger reads, as the C# `CoreDescriptorReader.MachineInfo` reads them: a field it
//! does not know ignored, one absent taken at its default (EmuSen_CoreAPI.md §6.5, §6.33, §6.34).

use crate::context::UnitInfo;
use emusen_native::json::{self, Value};

/// A memory space and its flags, `CoreSpace`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpaceInfo {
    pub id: u32,
    pub name: String,
    pub read_only: bool,
    pub side_effects: bool,
    pub reports_stores: bool,
    /// Its loads reach the read ranges of a core claiming DEBUG_READS.
    pub reports_loads: bool,
    /// A window onto other spaces' memory, holding none of its own.
    pub view: bool,
    /// "big" or "little" where the core states one.
    pub byte_order: Option<String>,
}

/// A processor, `CoreProcessor`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessorInfo {
    pub id: u32,
    pub name: String,
    pub pc_bits: i32,
    pub registers: Vec<(String, i32)>,
    pub code_space: Option<u32>,
}

/// One of the core's own counters, `CoreCounter`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CounterInfo {
    pub name: String,
    pub label: String,
    pub unit: String,
    pub kind: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Info {
    pub system: String,
    pub rate_num: u64,
    pub rate_den: u64,
    pub spaces: Vec<SpaceInfo>,
    pub processors: Vec<ProcessorInfo>,
    pub units: Vec<UnitInfo>,
    pub counters: Vec<CounterInfo>,
    pub phases: Vec<String>,
    /// The ROM patches' address range, where the core takes them.
    pub patches: Option<(i64, i64)>,
}

fn text(v: Option<&Value>, key: &str) -> String {
    v.and_then(|v| v.get(key)).and_then(Value::as_str).unwrap_or("").to_string()
}

fn int(v: Option<&Value>, key: &str, fallback: i64) -> i64 {
    v.and_then(|v| v.get(key)).and_then(Value::as_i64).unwrap_or(fallback)
}

fn arr<'a>(v: Option<&'a Value>, key: &str) -> &'a [Value] {
    v.and_then(|v| v.get(key)).and_then(Value::as_array).unwrap_or(&[])
}

fn obj<'a>(v: Option<&'a Value>, key: &str) -> Option<&'a Value> {
    v.and_then(|v| v.get(key)).filter(|o| matches!(o, Value::Object(_)))
}

fn registers(v: &Value) -> Vec<(String, i32)> {
    arr(Some(v), "registers").iter().map(|g| (text(Some(g), "name"), int(Some(g), "bits", 0) as i32)).collect()
}

impl Info {
    pub fn read(machine_info: &str) -> Info {
        let doc = json::parse(machine_info.as_bytes()).unwrap_or(Value::Null);
        let r = Some(&doc);
        let rate = obj(r, "frame_rate");
        let patches = obj(r, "patches");
        Info {
            system: text(r, "system"),
            rate_num: int(rate, "num", 60) as u64,
            rate_den: int(rate, "den", 1) as u64,
            spaces: arr(r, "spaces")
                .iter()
                .map(|s| {
                    let flags: Vec<&str> = arr(Some(s), "flags").iter().filter_map(Value::as_str).collect();
                    let has = |f: &str| flags.contains(&f);
                    SpaceInfo {
                        id: int(Some(s), "id", 0) as u32,
                        name: text(Some(s), "name"),
                        read_only: has("read_only"),
                        side_effects: has("side_effects"),
                        reports_stores: has("reports_stores"),
                        reports_loads: has("reports_loads"),
                        view: has("view"),
                        byte_order: s.get("byte_order").and_then(Value::as_str).map(str::to_string),
                    }
                })
                .collect(),
            processors: arr(r, "processors")
                .iter()
                .map(|p| ProcessorInfo {
                    id: int(Some(p), "id", 0) as u32,
                    name: text(Some(p), "name"),
                    pc_bits: int(Some(p), "pc_bits", 0) as i32,
                    registers: registers(p),
                    code_space: p.get("code_space").and_then(Value::as_i64).map(|c| c as u32),
                })
                .collect(),
            units: arr(r, "units")
                .iter()
                .map(|u| UnitInfo { id: int(Some(u), "id", 0) as u32, name: text(Some(u), "name"), kind: text(Some(u), "kind"), registers: registers(u) })
                .collect(),
            counters: arr(r, "telemetry")
                .iter()
                .map(|k| CounterInfo { name: text(Some(k), "name"), label: text(Some(k), "label"), unit: text(Some(k), "unit"), kind: text(Some(k), "kind") })
                .collect(),
            phases: arr(r, "phases").iter().filter_map(Value::as_str).map(str::to_string).collect(),
            patches: patches.map(|p| (int(Some(p), "low", 0), int(Some(p), "high", 0))),
        }
    }

    /// `FrameRateHz`: the machine's rate, 60 where it gives none.
    pub fn frame_rate_hz(&self) -> f64 {
        if self.rate_den == 0 { 60.0 } else { self.rate_num as f64 / self.rate_den as f64 }
    }

    pub fn space_named(&self, name: &str) -> Option<&SpaceInfo> {
        self.spaces.iter().find(|s| emusen_dotnet::text::equals_ignore_case(&s.name, name))
    }
}
