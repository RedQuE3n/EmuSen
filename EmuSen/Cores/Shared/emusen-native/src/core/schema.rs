//! A validator for the subset of JSON Schema that `abi/v1/*.schema.json` use: `type`, `enum`, `pattern` (anchored
//! classes and literals with `+`), `minimum`, `required`, `properties`, `additionalProperties`, `items` and
//! same-document `$ref`. Unknown members are allowed, as every reader of a descriptor must allow them (§4.2).

use crate::json::Value;

/// The schemas as committed beside the header.
pub const INFO: &str = include_str!("../../abi/v1/info.schema.json");
pub const MACHINE_INFO: &str = include_str!("../../abi/v1/machine-info.schema.json");
pub const SETTINGS: &str = include_str!("../../abi/v1/settings.schema.json");
pub const FIRMWARE: &str = include_str!("../../abi/v1/firmware.schema.json");
pub const SETTING_NOTES: &str = include_str!("../../abi/v1/setting-notes.schema.json");
pub const DISASSEMBLY: &str = include_str!("../../abi/v1/disassembly.schema.json");

/// Every violation of `schema` in `document`, each with its path; empty when it conforms.
pub fn validate(schema: &str, document: &str) -> Vec<String> {
    let root = match crate::json::parse(schema.as_bytes()) {
        Ok(v) => v,
        Err(e) => return vec![format!("the schema is not JSON at byte {}", e.at)],
    };
    let doc = match crate::json::parse(document.as_bytes()) {
        Ok(v) => v,
        Err(e) => return vec![format!("the document is not JSON at byte {}", e.at)],
    };
    let mut errors = Vec::new();
    check(&root, &root, &doc, "$", &mut errors);
    errors
}

/// The schema `$ref` names within `root`.
pub fn resolve<'a>(root: &'a Value, s: &'a Value) -> &'a Value {
    match s.get("$ref").and_then(Value::as_str).and_then(|r| r.strip_prefix("#/$defs/")) {
        Some(name) => root.get("$defs").and_then(|d| d.get(name)).map_or(s, |t| resolve(root, t)),
        None => s,
    }
}

fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Int(_) => "integer",
        Value::Float(_) => "number",
        Value::Str(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn check(root: &Value, s: &Value, v: &Value, path: &str, errors: &mut Vec<String>) {
    let s = resolve(root, s);
    if let Some(t) = s.get("type") {
        let allowed: Vec<&str> = match t {
            Value::Str(one) => vec![one.as_str()],
            Value::Array(many) => many.iter().filter_map(Value::as_str).collect(),
            _ => vec![],
        };
        let actual = type_name(v);
        if !allowed.iter().any(|&a| a == actual || (a == "number" && actual == "integer")) {
            errors.push(format!("{path}: {actual}, not {}", allowed.join(" or ")));
            return;
        }
    }
    if let (Some(e), Some(text)) = (s.get("enum").and_then(Value::as_array), v.as_str())
        && !e.iter().any(|x| x.as_str() == Some(text))
    {
        errors.push(format!("{path}: {text:?} is not one of the enumeration"));
    }
    if let (Some(p), Some(text)) = (s.get("pattern").and_then(Value::as_str), v.as_str()) {
        match matches(p, text) {
            Some(true) => {}
            Some(false) => errors.push(format!("{path}: {text:?} does not match {p}")),
            None => errors.push(format!("{path}: the pattern {p} is outside the supported subset")),
        }
    }
    if let (Some(min), Some(n)) = (s.get("minimum").and_then(Value::as_i64), v.as_i64())
        && n < min
    {
        errors.push(format!("{path}: {n} is below {min}"));
    }
    if let Value::Object(members) = v {
        for r in s.get("required").and_then(Value::as_array).unwrap_or(&[]) {
            if let Some(key) = r.as_str()
                && !members.iter().any(|(k, _)| k == key)
            {
                errors.push(format!("{path}: {key} is required"));
            }
        }
        let props = s.get("properties");
        for (k, child) in members {
            match props.and_then(|p| p.get(k)) {
                Some(cs) => check(root, cs, child, &format!("{path}.{k}"), errors),
                None => {
                    if let Some(extra) = s.get("additionalProperties") {
                        check(root, extra, child, &format!("{path}.{k}"), errors);
                    }
                }
            }
        }
    }
    if let (Value::Array(items), Some(item)) = (v, s.get("items")) {
        for (i, child) in items.iter().enumerate() {
            check(root, item, child, &format!("{path}[{i}]"), errors);
        }
    }
}

/// An anchored pattern of literals and `[...]` classes, each optionally `+`; `None` for anything else.
pub fn matches(pattern: &str, text: &str) -> Option<bool> {
    let body = pattern.strip_prefix('^')?.strip_suffix('$')?;
    let mut atoms: Vec<(Vec<(char, char)>, bool)> = Vec::new();
    let mut cs = body.chars().peekable();
    while let Some(c) = cs.next() {
        let set = if c == '[' {
            let mut set = Vec::new();
            let inner: Vec<char> = cs.by_ref().take_while(|&c| c != ']').collect();
            let mut i = 0;
            while i < inner.len() {
                if i + 2 < inner.len() && inner[i + 1] == '-' {
                    set.push((inner[i], inner[i + 2]));
                    i += 3;
                } else {
                    set.push((inner[i], inner[i]));
                    i += 1;
                }
            }
            set
        } else if "()|*?{}\\.".contains(c) {
            return None;
        } else {
            vec![(c, c)]
        };
        let plus = cs.peek() == Some(&'+');
        if plus {
            cs.next();
        }
        atoms.push((set, plus));
    }
    // Classes here never overlap their neighbours, so a greedy match is exact.
    let t: Vec<char> = text.chars().collect();
    let mut at = 0;
    for (set, plus) in &atoms {
        let hit = |c: char| set.iter().any(|&(lo, hi)| c >= lo && c <= hi);
        if at >= t.len() || !hit(t[at]) {
            return Some(false);
        }
        at += 1;
        while *plus && at < t.len() && hit(t[at]) {
            at += 1;
        }
    }
    Some(at == t.len())
}
