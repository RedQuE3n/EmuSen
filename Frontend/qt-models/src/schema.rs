//! A row of a console's settings schema as the screens say and step it (EmuSen_Qt.md §6.2 item 5, §7.7.E): the runtime's
//! kinds (`switch`, `count`, `choice`) and nothing of any core, so a core declaring a new setting needs no code here.

use emusen_crystal_client::shapes::SettingRow;

/// A value as a row says it: a choice by its label, a switch as On or Off, a count as its number.
pub fn shown(row: &SettingRow, value: &str) -> String {
    match row.kind.as_str() {
        "switch" => if is_on(value) { "On" } else { "Off" }.to_owned(),
        "choice" => {
            let choices = row.choices.as_deref().unwrap_or_default();
            let labels = row.choice_labels.as_deref().unwrap_or_default();
            choices.iter().position(|c| c == value).and_then(|at| labels.get(at)).filter(|l| !l.is_empty()).cloned().unwrap_or_else(|| value.to_owned())
        }
        _ => value.to_owned(),
    }
}

/// A switch's stored value read as on.
pub fn is_on(value: &str) -> bool {
    matches!(value.to_ascii_lowercase().as_str(), "true" | "1" | "on")
}

/// The value one step on: the next or last choice (round), a switch turned, a count within its range; none for a kind
/// that does not step.
pub fn stepped(row: &SettingRow, value: &str, by: i32) -> Option<String> {
    match row.kind.as_str() {
        "switch" => Some((!is_on(value)).to_string()),
        "choice" => {
            let choices = row.choices.as_deref().filter(|c| !c.is_empty())?;
            let next = match choices.iter().position(|c| c == value) {
                Some(at) => (at as i32 + by).rem_euclid(choices.len() as i32) as usize,
                None if by > 0 => 0,
                None => choices.len() - 1,
            };
            Some(choices[next].clone())
        }
        "count" => {
            let now: f64 = value.parse().unwrap_or(row.min.unwrap_or(0.0));
            let next = (now + f64::from(by)).clamp(row.min.unwrap_or(f64::MIN), row.max.unwrap_or(f64::MAX));
            Some(format!("{next}"))
        }
        _ => None,
    }
}

/// Whether a row means something while the keys hold their values: its `shown_with` holds, or it has none.
pub fn applies(row: &SettingRow, value: &dyn Fn(&str) -> String) -> bool {
    row.shown_with.as_ref().is_none_or(|w| value(&w.key) == w.value)
}

/// Rows as the runtime writes them, for a model that hands them on as plain data.
pub fn to_json(rows: &[SettingRow]) -> serde_json::Value {
    rows.iter()
        .map(|r| {
            serde_json::json!({ "key": r.key, "label": r.label, "hint": r.hint, "kind": r.kind, "default": r.default, "min": r.min, "max": r.max,
                                "choices": r.choices, "choice_labels": r.choice_labels, "group": r.group, "note": r.note,
                                "shown_with": r.shown_with.as_ref().map(|w| serde_json::json!({ "key": w.key, "value": w.value })) })
        })
        .collect()
}
