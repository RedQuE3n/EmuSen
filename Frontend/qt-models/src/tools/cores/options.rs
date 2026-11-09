//! An engine's options as rows of a page, from the runtime's settings schema (EmuSen_Interface.md §15.5, Options ▸):
//! each row its label, its value in the schema's words, ◂ ▸ for a switch, a count or a choice, and its hint and note in
//! the pane; a row shown only with another key's value is shown while that key holds it. The schema's kinds are the
//! runtime's (`switch`, `count`, `choice`); nothing here knows a core.

use crate::card::{Pane, Row};
use emusen_crystal_client::shapes::SettingRow;

/// A value as the page says it: a choice by its label, a switch as On or Off, a count as its number.
pub fn shown(row: &SettingRow, value: &str) -> String {
    match row.kind.as_str() {
        "switch" => if value.eq_ignore_ascii_case("true") { "On" } else { "Off" }.to_owned(),
        "choice" => {
            let choices = row.choices.as_deref().unwrap_or_default();
            let labels = row.choice_labels.as_deref().unwrap_or_default();
            choices.iter().position(|c| c == value).and_then(|at| labels.get(at)).cloned().unwrap_or_else(|| value.to_owned())
        }
        _ => value.to_owned(),
    }
}

/// The value one step on: the next or last choice (round), a switch turned, a count within its range; none for a
/// kind that does not step.
pub fn stepped(row: &SettingRow, value: &str, by: i32) -> Option<String> {
    match row.kind.as_str() {
        "switch" => Some((!value.eq_ignore_ascii_case("true")).to_string()),
        "choice" => {
            let choices = row.choices.as_deref().filter(|c| !c.is_empty())?;
            let at = choices.iter().position(|c| c == value).unwrap_or(0) as i32;
            Some(choices[(at + by).rem_euclid(choices.len() as i32) as usize].clone())
        }
        "count" => {
            let now: f64 = value.parse().unwrap_or(row.min.unwrap_or(0.0));
            let next = (now + f64::from(by)).clamp(row.min.unwrap_or(f64::MIN), row.max.unwrap_or(f64::MAX));
            Some(format!("{next}"))
        }
        _ => None,
    }
}

/// The page's rows, keyed `<prefix><key>`, each in its value as `value` reads it; the last row of a group ends a section.
pub fn rows(schema: &[SettingRow], value: &dyn Fn(&str, &Option<String>) -> String, prefix: &str) -> Vec<Row> {
    let shown_rows: Vec<&SettingRow> = schema
        .iter()
        .filter(|s| s.shown_with.as_ref().is_none_or(|w| value(&w.key, &schema.iter().find(|o| o.key == w.key).and_then(|o| o.default.clone())) == w.value))
        .collect();
    shown_rows
        .iter()
        .enumerate()
        .map(|(at, s)| {
            let now = value(&s.key, &s.default);
            let steps = matches!(s.kind.as_str(), "switch" | "count" | "choice");
            let lines = [s.hint.clone(), s.note.clone()].into_iter().flatten().filter(|l| !l.is_empty()).map(|l| (l, String::new())).collect();
            let section_end = shown_rows.get(at + 1).is_some_and(|next| next.group != s.group);
            Row {
                key: format!("{prefix}{}", s.key),
                label: s.label.clone(),
                value: shown(s, &now),
                steps,
                section_end,
                pane: Pane { lines, ..Pane::default() },
                ..Row::default()
            }
        })
        .collect()
}
