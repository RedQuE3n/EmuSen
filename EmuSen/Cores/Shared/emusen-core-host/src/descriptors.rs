//! The fields of core info a host reads to list and route a core, with the C# host's defaults: a field it does not know
//! is ignored, one it needs and is absent takes its default (EmuSen_CoreAPI.md §6.3, `CoreDescriptors.cs`).

use emusen_native::json::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoreSystem {
    pub id: String,
    pub name: String,
    /// Lowercased, as routing compares them.
    pub extensions: Vec<String>,
    pub development: bool,
    pub starts_empty: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoreInfo {
    pub abi: String,
    pub id: String,
    pub name: String,
    /// `display_name`, or `name` where the info has none: the engine's name a player sees.
    pub display_name: String,
    pub version: String,
    pub license: String,
    pub systems: Vec<CoreSystem>,
    pub capabilities: Vec<String>,
    pub host_requires: Vec<String>,
}

fn str_or(v: &Value, key: &str, fallback: &str) -> String {
    v.get(key).and_then(Value::as_str).unwrap_or(fallback).to_owned()
}

fn arr<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key).and_then(Value::as_array).unwrap_or(&[])
}

fn strs(v: &Value, key: &str) -> Vec<String> {
    arr(v, key).iter().filter_map(Value::as_str).map(str::to_owned).collect()
}

fn flag(v: &Value, key: &str) -> bool {
    matches!(v.get(key), Some(Value::Bool(true)))
}

/// The C# reader throws where it reads a value as an integer that is not one; such an info is not read at all.
fn integers_where_read(system: &Value) -> bool {
    let ports = arr(system, "controllers").iter().all(|c| arr(c, "ports").iter().all(|p| p.as_i64().is_some()));
    let parts = arr(system, "firmware").iter().all(|f| arr(f, "parts").iter().all(|p| p.as_array().is_some()));
    let battery_ports = arr(system, "battery").iter().all(|b| !matches!(b.get("port"), Some(Value::Float(_))));
    ports && parts && battery_ports
}

impl CoreInfo {
    /// The info `v` holds, or `None` where it is not an object or the C# reader would refuse it.
    pub fn read(v: &Value) -> Option<CoreInfo> {
        if !matches!(v, Value::Object(_)) {
            return None;
        }
        let mut systems = Vec::new();
        for s in arr(v, "systems") {
            if !integers_where_read(s) {
                return None;
            }
            systems.push(CoreSystem {
                id: str_or(s, "id", ""),
                name: str_or(s, "name", ""),
                extensions: strs(s, "extensions").into_iter().map(|e| e.to_lowercase()).collect(),
                development: flag(s, "development"),
                starts_empty: flag(s, "starts_empty"),
            });
        }
        let name = str_or(v, "name", "");
        Some(CoreInfo {
            abi: str_or(v, "abi", ""),
            id: str_or(v, "id", ""),
            display_name: v.get("display_name").and_then(Value::as_str).map_or_else(|| name.clone(), str::to_owned),
            name,
            version: str_or(v, "version", ""),
            license: str_or(v, "license", ""),
            systems,
            capabilities: strs(v, "capabilities"),
            host_requires: strs(v, "host_requires"),
        })
    }

    /// Whether a system lists `extension`, without regard to case.
    pub fn claims(&self, extension: &str) -> bool {
        self.system_for(extension).is_some()
    }

    pub fn system_for(&self, extension: &str) -> Option<&CoreSystem> {
        let e = extension.to_lowercase();
        self.systems.iter().find(|s| s.extensions.contains(&e))
    }
}

/// Two JSON values equal as .NET's `JsonNode.DeepEquals` compares them: an object's members without regard to order.
pub fn deep_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => x.len() == y.len() && x.iter().all(|(k, v)| y.iter().any(|(l, w)| k == l && deep_equal(v, w))),
        (Value::Array(x), Value::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(v, w)| deep_equal(v, w)),
        (Value::Int(x), Value::Float(y)) | (Value::Float(y), Value::Int(x)) => *x as f64 == *y,
        _ => a == b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use emusen_native::json::parse;

    #[test]
    fn the_display_name_falls_back_to_the_name_and_extensions_are_lowercased() {
        let v = parse(br#"{"abi":"1.14","id":"x","name":"X","version":"1","license":"L","systems":[{"id":"s","name":"S","extensions":[".GB",".gbc"],"development":true}],"capabilities":[]}"#).unwrap();
        let i = CoreInfo::read(&v).unwrap();
        assert_eq!(i.display_name, "X");
        assert_eq!(i.systems[0].extensions, [".gb", ".gbc"]);
        assert!(i.systems[0].development && !i.systems[0].starts_empty);
        assert!(i.claims(".Gb") && !i.claims(".nes"));
    }

    #[test]
    fn an_info_the_csharp_reader_throws_on_is_not_read() {
        let v = parse(br#"{"systems":[{"controllers":[{"ports":[0,"one"]}]}]}"#).unwrap();
        assert!(CoreInfo::read(&v).is_none());
        assert!(CoreInfo::read(&parse(b"[1]").unwrap()).is_none());
    }

    #[test]
    fn objects_compare_without_regard_to_member_order() {
        let a = parse(br#"{"a":1,"b":[1,{"c":2,"d":3}]}"#).unwrap();
        let b = parse(br#"{"b":[1,{"d":3,"c":2}],"a":1}"#).unwrap();
        let c = parse(br#"{"b":[{"d":3,"c":2},1],"a":1}"#).unwrap();
        assert!(deep_equal(&a, &b));
        assert!(!deep_equal(&a, &c));
    }
}
