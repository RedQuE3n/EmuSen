//! The fields of core info the runtime reads beyond the listing's: each system's controllers with their controls and
//! ports, as the C# host's `CoreDescriptorReader` reads them, a field it does not know ignored and one absent taken at
//! its default (EmuSen_CoreAPI.md §6.3, EmuSen_Backend.md §29.2).

use emusen_native::json::Value;

/// `PadButton`'s names in its order, the canonical controls' numbers (EmuSen_CoreAPI.md §6.4).
pub const BUTTONS: [&str; 16] = ["B", "Y", "Select", "Start", "Up", "Down", "Left", "Right", "A", "X", "L", "R", "L2", "R2", "L3", "R3"];

/// `PadAxis`'s names in its order.
pub const AXES: [&str; 6] = ["LeftX", "LeftY", "RightX", "RightY", "LeftTrigger", "RightTrigger"];

/// A name as .NET's `Enum.TryParse` without regard to case off reads it, with `Enum.IsDefined` after: a member's name,
/// or a number that is a member's.
fn named(names: &[&str], text: Option<&str>) -> Option<usize> {
    let text = text?;
    if let Some(i) = names.iter().position(|n| *n == text) {
        return Some(i);
    }
    text.trim().parse::<i64>().ok().filter(|n| *n >= 0 && (*n as usize) < names.len()).map(|n| n as usize)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Controller {
    pub id: String,
    pub ports: Vec<u32>,
    /// Each button's canonical control, where it names one.
    pub buttons: Vec<Option<usize>>,
    pub axes: Vec<Option<usize>>,
}

/// The kind of a battery file, `CoreBatteryKind`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatteryKind {
    Game,
    Removable,
    Console,
}

impl BatteryKind {
    /// `CoreBatteryFile.KindNamed`: a name it does not know, or none, is the game's own.
    pub fn named(name: Option<&str>) -> BatteryKind {
        match name {
            Some("removable") => BatteryKind::Removable,
            Some("console") => BatteryKind::Console,
            _ => BatteryKind::Game,
        }
    }

    /// The kind's name in lower case, as a medium's group key spells it.
    pub fn word(self) -> &'static str {
        match self {
            BatteryKind::Game => "game",
            BatteryKind::Removable => "removable",
            BatteryKind::Console => "console",
        }
    }
}

/// A battery file a system lists, `CoreBatteryFile`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BatteryFile {
    pub which: u32,
    pub suffix: String,
    pub kind: BatteryKind,
    pub medium: Option<String>,
    pub label: Option<String>,
    pub port: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemDetail {
    pub id: String,
    pub controllers: Vec<Controller>,
    /// The system's battery files, none where it lists no `battery`.
    pub battery: Option<Vec<BatteryFile>>,
}

fn arr<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key).and_then(Value::as_array).unwrap_or(&[])
}

fn opt_str<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(Value::as_str)
}

/// Every system of the info, in its order.
pub fn systems(info: &Value) -> Vec<SystemDetail> {
    arr(info, "systems")
        .iter()
        .map(|s| SystemDetail {
            id: opt_str(s, "id").unwrap_or("").to_string(),
            controllers: arr(s, "controllers")
                .iter()
                .map(|c| Controller {
                    id: opt_str(c, "id").unwrap_or("").to_string(),
                    ports: arr(c, "ports").iter().filter_map(Value::as_i64).map(|p| p as u32).collect(),
                    buttons: arr(c, "buttons").iter().map(|b| named(&BUTTONS, opt_str(b, "control"))).collect(),
                    axes: arr(c, "axes").iter().map(|a| named(&AXES, opt_str(a, "control"))).collect(),
                })
                .collect(),
            battery: s.get("battery").and_then(Value::as_array).map(|list| {
                list.iter()
                    .map(|b| BatteryFile {
                        which: b.get("which").and_then(Value::as_i64).unwrap_or(0) as u32,
                        suffix: opt_str(b, "suffix").unwrap_or("").to_string(),
                        kind: BatteryKind::named(opt_str(b, "kind")),
                        medium: opt_str(b, "medium").map(str::to_string),
                        label: opt_str(b, "label").map(str::to_string),
                        port: b.get("port").and_then(Value::as_i64),
                    })
                    .collect()
            }),
        })
        .collect()
}

impl SystemDetail {
    /// The buttons its controllers name, each once, in the order first named.
    pub fn buttons(&self) -> Vec<usize> {
        distinct(self.controllers.iter().flat_map(|c| c.buttons.iter().flatten().copied()))
    }

    pub fn axes(&self) -> Vec<usize> {
        distinct(self.controllers.iter().flat_map(|c| c.axes.iter().flatten().copied()))
    }

    /// One past the highest port a controller names, 0 where none does.
    pub fn ports(&self) -> usize {
        self.controllers.iter().flat_map(|c| c.ports.iter()).map(|p| *p as usize + 1).max().unwrap_or(0)
    }

    pub fn has_controller(&self, id: &str) -> bool {
        self.controllers.iter().any(|c| c.id == id)
    }
}

fn distinct(items: impl Iterator<Item = usize>) -> Vec<usize> {
    let mut out = Vec::new();
    for i in items {
        if !out.contains(&i) {
            out.push(i);
        }
    }
    out
}

/// A setting of a schema as the window lists it: its key and default, hidden ones and those of a kind it does not show
/// left out (`AsCoreSetting`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Setting {
    pub key: String,
    pub default: String,
}

/// The schema's settings the window lists.
pub fn settings(schema: &Value) -> Vec<Setting> {
    schema
        .as_array()
        .unwrap_or(&[])
        .iter()
        .filter(|s| !matches!(s.get("hidden"), Some(Value::Bool(true))) && matches!(opt_str(s, "kind"), Some("switch" | "count" | "choice")))
        .map(|s| Setting { key: opt_str(s, "key").unwrap_or("").to_string(), default: opt_str(s, "default").unwrap_or("").to_string() })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use emusen_native::json::parse;

    #[test]
    fn controls_are_read_by_name_or_number_and_ports_counted() {
        let v = parse(br#"{"systems":[{"id":"s","controllers":[{"id":"p","ports":[0,1],"buttons":[{"control":"A"},{"control":"9"},{"control":"a"},{"control":null}],"axes":[{"control":"LeftY"}]}]}]}"#).unwrap();
        let s = &systems(&v)[0];
        assert_eq!(s.buttons(), [8, 9]);
        assert_eq!(s.axes(), [1]);
        assert_eq!(s.ports(), 2);
        assert!(s.has_controller("p"));
    }
}
