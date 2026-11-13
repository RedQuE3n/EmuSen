//! Settings ▸ Display ▸ Shaders and Shader levers, and a console's Shader levers (EmuSen_Qt.md §7.7): the shaders a
//! console may draw with as a page of choices (its recent ones, Serenity's built-in filters that suit it, the shader
//! pack's presets), stored in its `ScreenFilter`; and the levers of the one it draws with, each a parameter as the base
//! reports it (a built-in filter's settings from Serenity's catalogue, a pack preset's as its passes declare them),
//! stored in `ShaderParameters` and set on a running game's chain at once.

use crate::display::{pack_preset, PACK_PREFIX};
use crate::settings::{at, set_at, text_of, wrapped, Model, Outcome, Page, Pane, Preview, Row, SettingsView};
use emusen_kit_core::settings::{RowKind, SliderRange};
use serde_json::{json, Map, Value};

/// How many shaders a console's recent list keeps, newest first.
pub const RECENT_KEPT: usize = 5;

/// The `ScreenFilter` values of every console that draws with a pack preset, each once.
pub fn chosen_presets(graphics: &Value) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for console in graphics["Consoles"].as_object().into_iter().flat_map(Map::values) {
        let stored = text_of(&console["ScreenFilter"]);
        if pack_preset(&stored).is_some() && !out.contains(&stored) {
            out.push(stored);
        }
    }
    out
}

/// A file or folder name as words: the preset's extension dropped, underscores as spaces, runs of spaces as one.
fn readable(name: &str) -> String {
    let name = if name.to_ascii_lowercase().ends_with(".slangp") { &name[..name.len() - ".slangp".len()] } else { name };
    name.replace('_', " ").split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A float as the value stored and shown: at most six places, no trailing zeros.
fn decimal(value: f64) -> String {
    let text = format!("{:.6}", value);
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" { "0".to_owned() } else { text.to_owned() }
}

fn bits(value: &Value) -> f64 {
    value.as_u64().map_or(0.0, |b| f64::from(f32::from_bits(b as u32)))
}

/// One lever: a parameter as Serenity gives one, and its named values where it has them.
#[derive(Clone, Debug, PartialEq)]
pub struct Lever {
    pub id: String,
    pub label: String,
    /// The shader's own value on this console.
    pub own: f64,
    pub range: SliderRange,
    pub choices: Vec<String>,
    /// Whether the parameter only heads a group, as a range with no width does.
    pub heading: bool,
}

impl SettingsView {
    fn shader_of(&self, console: &str) -> String {
        let stored = at(&self.read.graphics, &format!("Consoles.{console}.ScreenFilter")).map(text_of).unwrap_or_default();
        if stored.is_empty() { self.filter_name(&json!(crate::display::PLAIN)) } else { stored }
    }

    /// A stored `ScreenFilter` as the words a row shows: a built-in filter's name, a preset's file name.
    fn shader_name(stored: &str) -> String {
        match pack_preset(stored) {
            Some(path) => readable(path.rsplit('/').next().unwrap_or(path)),
            None => stored.to_owned(),
        }
    }

    fn shaders_word(&self, name: &str) -> String {
        text_of(&self.description_at(&["shaders", "words", name]))
    }

    /// The built-in filter a stored value names, by its name or key, if it suits the console.
    fn builtin(&self, console: &str, stored: &str) -> Option<&Value> {
        let filters = self.read.catalogue["filters"].as_array()?;
        let suits = |f: &&Value| f["consoles"].as_array().is_none_or(|list| list.iter().any(|c| c == console));
        filters.iter().filter(suits).find(|f| f["name"] == stored || f["key"] == stored)
    }

    /// Whether a stored value is one the console may draw with now.
    fn offered(&self, console: &str, stored: &str) -> bool {
        match pack_preset(stored) {
            Some(path) => self.read.pack.iter().any(|p| p == path),
            None => self.builtin(console, stored).is_some(),
        }
    }

    /// The levers of the shader `console` draws with, in the order the base gives them, or why they could not be read.
    pub fn levers(&self, console: &str) -> Result<Vec<Lever>, String> {
        let stored = self.shader_of(console);
        if pack_preset(&stored).is_some() {
            let read = self.read.levers.get(&stored).cloned().unwrap_or(Ok(Vec::new()))?;
            return Ok(read
                .iter()
                .map(|p| {
                    let (minimum, maximum) = (bits(&p["minimum"]), bits(&p["maximum"]));
                    let description = text_of(&p["description"]);
                    let label = if description.trim().is_empty() { text_of(&p["id"]) } else { description.trim().to_owned() };
                    Lever {
                        id: text_of(&p["id"]),
                        label,
                        own: bits(&p["initial"]),
                        range: SliderRange { minimum, maximum, step: bits(&p["step"]) },
                        choices: Vec::new(),
                        heading: maximum <= minimum,
                    }
                })
                .collect());
        }
        let Some(filter) = self.builtin(console, &stored) else { return Ok(Vec::new()) };
        let defaults = &filter["defaults"];
        Ok(filter["settings"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|s| {
                let id = text_of(&s["id"]);
                let own = [console, ""].iter().find_map(|c| defaults[*c][&id].as_u64()).map_or_else(|| bits(&s["initial"]), |b| bits(&json!(b)));
                Lever {
                    label: text_of(&s["description"]),
                    own,
                    range: SliderRange { minimum: bits(&s["minimum"]), maximum: bits(&s["maximum"]), step: bits(&s["step"]) },
                    choices: s["choices"].as_array().map(|c| c.iter().map(text_of).collect()).unwrap_or_default(),
                    heading: false,
                    id,
                }
            })
            .collect())
    }

    /// A lever's value on `console`: the one stored, else the shader's own.
    fn lever_value(&self, console: &str, lever: &Lever) -> f64 {
        let stored = self.shader_of(console);
        let held = self.read.graphics["ShaderParameters"][console][&stored][&lever.id].clone();
        text_of(&held).trim().replace(',', ".").parse::<f64>().ok().filter(|v| v.is_finite()).unwrap_or(lever.own)
    }

    /// The rows of a shader page, or none for another page.
    pub(crate) fn shaders_entries(&self, page: &Page) -> Option<Vec<Value>> {
        match page {
            Page::ShaderConsoles { .. } => {
                Some(self.consoles("every").into_iter().map(|c| json!({ "id": format!("shaders:{c}"), "shader_console": c })).collect())
            }
            Page::Shaders(console) => {
                let mut out = Vec::new();
                let recent: Vec<String> = self.read.graphics["RecentShaders"][console]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(text_of)
                    .filter(|s| !s.is_empty() && self.offered(console, s))
                    .collect();
                if !recent.is_empty() {
                    out.push(json!({ "id": "heading:recent", "heading": self.shaders_word("recent") }));
                    out.extend(recent.iter().map(|s| json!({ "id": format!("recent:{s}"), "shader": s })));
                }
                out.push(json!({ "id": "heading:builtin", "heading": self.shaders_word("builtin") }));
                out.extend(self.screen_filters(console).into_iter().map(|s| json!({ "id": format!("shader:{s}"), "shader": s })));
                // The pack's presets under a heading for each folder, those at its top under the pack's own, first.
                let folder_of = |path: &String| path.rsplit_once('/').map(|(f, _)| f.split('/').map(readable).collect::<Vec<_>>().join(" / "));
                let mut presets: Vec<(Option<String>, &String)> = self.read.pack.iter().map(|p| (folder_of(p), p)).collect();
                presets.sort_by_key(|(folder, _)| folder.is_some());
                let mut heading: Option<Option<String>> = None;
                for (folder, path) in presets {
                    if heading.as_ref() != Some(&folder) {
                        let words = folder.clone().unwrap_or_else(|| self.shaders_word("pack"));
                        out.push(json!({ "id": format!("heading:{words}"), "heading": words }));
                        heading = Some(folder);
                    }
                    out.push(json!({ "id": format!("shader:{PACK_PREFIX}{path}"), "shader": format!("{PACK_PREFIX}{path}") }));
                }
                Some(out)
            }
            Page::Levers(console) => Some(match self.levers(console) {
                Ok(levers) if !levers.is_empty() => levers.iter().map(|l| json!({ "id": format!("lever:{}", l.id), "lever": l.id })).collect(),
                Ok(_) => vec![json!({ "id": "levers:none", "heading": self.shaders_word("none") })],
                Err(words) => vec![json!({ "id": "levers:failed", "heading": words })],
            }),
            _ => None,
        }
    }

    fn lever_of(&self, entry: &Value) -> Option<(String, Lever)> {
        let Some(Page::Levers(console)) = self.page() else { return None };
        let id = entry["lever"].as_str()?;
        let lever = self.levers(console).ok()?.into_iter().find(|l| l.id == id)?;
        Some((console.clone(), lever))
    }

    pub(crate) fn shaders_row(&self, entry: &Value) -> Option<Row> {
        let id = text_of(&entry["id"]);
        let row = |label: String, value: String, kind: RowKind, held: bool, marked: bool| Some(Row { id: id.clone(), label, value, kind, held, marked });
        if let Some(console) = entry["shader_console"].as_str() {
            return row(self.console_name(console), Self::shader_name(&self.shader_of(console)), RowKind::Opens, false, false);
        }
        if let Some(stored) = entry["shader"].as_str() {
            let Some(Page::Shaders(console)) = self.page() else { return None };
            let chosen = self.shader_of(console) == stored;
            let value = if chosen { self.console_word("chosen") } else { String::new() };
            return row(Self::shader_name(stored), value, RowKind::Choice, false, chosen);
        }
        if entry.get("lever").is_some() {
            let (console, lever) = self.lever_of(entry)?;
            if lever.heading {
                return row(lever.label, String::new(), RowKind::Action, true, false);
            }
            let value = self.lever_value(&console, &lever);
            if lever.choices.is_empty() {
                return row(lever.label, decimal(value), RowKind::Slider, false, false);
            }
            let at = (value.round().max(0.0) as usize).min(lever.choices.len() - 1);
            return row(lever.label, lever.choices[at].clone(), RowKind::Steps, false, false);
        }
        if matches!(self.page(), Some(Page::Shaders(_) | Page::Levers(_) | Page::ShaderConsoles { .. })) {
            if let Some(words) = entry["heading"].as_str() {
                return row(words.to_owned(), String::new(), RowKind::Action, true, false);
            }
        }
        None
    }

    /// A slider row's range and amount, by its index: minimum, maximum, the pad's step and the value.
    pub fn slider(&self, index: usize) -> Option<[f64; 4]> {
        let entry = self.entries().get(index).cloned()?;
        let (console, lever) = self.lever_of(&entry)?;
        if lever.heading || !lever.choices.is_empty() {
            return None;
        }
        let r = lever.range;
        Some([r.minimum, r.maximum, r.pad_step(), self.lever_value(&console, &lever)])
    }

    pub(crate) fn shaders_pane(&self, entry: &Value) -> Option<Pane> {
        if let Some(console) = entry["shader_console"].as_str() {
            let sentence = format!("{} · {}", self.console_name(console), Self::shader_name(&self.shader_of(console)));
            return Some(Pane { sentence, ..Pane::default() });
        }
        if let Some(stored) = entry["shader"].as_str() {
            // A pack preset by its path in the pack; a built-in filter by its credit in Serenity's catalogue.
            let Some(Page::Shaders(console)) = self.page() else { return None };
            let sentence = match pack_preset(stored) {
                Some(path) => path.to_owned(),
                None => self.builtin(console, stored).map(|f| text_of(&f["credit"])).unwrap_or_default(),
            };
            return Some(Pane { sentence, ..Pane::default() });
        }
        let (_, lever) = self.lever_of(entry)?;
        let own = if lever.choices.is_empty() { decimal(lever.own) } else { lever.choices.get(lever.own.round().max(0.0) as usize).cloned().unwrap_or_default() };
        let pane = self.description_at(&["shaders", "pane"]);
        Some(Pane {
            sentence: if lever.heading { String::new() } else { self.fill(&self.shaders_word("own"), &own) },
            preview: Preview::Picture,
            caption: self.fill(&text_of(&pane["caption"]), &own),
            ..Pane::default()
        })
    }

    /// Left or Right on a lever: its next value, saved and set on the picture.
    pub(crate) fn shaders_step(&mut self, entry: &Value, by: i32) -> Option<Outcome> {
        let (console, lever) = self.lever_of(entry)?;
        if lever.heading {
            return Some(Outcome::Nothing);
        }
        let now = self.lever_value(&console, &lever);
        let next = if lever.choices.is_empty() {
            lever.range.stepped(now, by)
        } else {
            wrapped((now.round().max(0.0) as usize).min(lever.choices.len() - 1), by, lever.choices.len()) as f64
        };
        Some(self.store_lever(&console, &lever, next))
    }

    /// A slider moved to `amount` by the pad's walk or the pointer.
    pub fn slide(&mut self, index: usize, amount: f64) -> Outcome {
        let Some(entry) = self.entries().get(index).cloned() else { return Outcome::Nothing };
        let Some((console, lever)) = self.lever_of(&entry) else { return Outcome::Nothing };
        if lever.heading || !lever.choices.is_empty() || !amount.is_finite() {
            return Outcome::Nothing;
        }
        let (low, high) = (lever.range.minimum.min(lever.range.maximum), lever.range.minimum.max(lever.range.maximum));
        self.store_lever(&console, &lever, amount.clamp(low, high))
    }

    /// A lever's value saved in `ShaderParameters` under the console's shader, forgotten at the shader's own value.
    fn store_lever(&mut self, console: &str, lever: &Lever, value: f64) -> Outcome {
        let stored = self.shader_of(console);
        let text = decimal(value);
        let parameters = &mut self.read.graphics["ShaderParameters"];
        if !parameters.is_object() {
            *parameters = json!({});
        }
        if text == decimal(lever.own) {
            if let Some(held) = parameters[console][&stored].as_object_mut() {
                held.remove(&lever.id);
                if held.is_empty() {
                    parameters[console].as_object_mut().map(|c| c.remove(&stored));
                }
            }
            if parameters[console].as_object().is_some_and(Map::is_empty) {
                parameters.as_object_mut().map(|p| p.remove(console));
            }
        } else {
            parameters[console][&stored][&lever.id] = json!(text);
        }
        let value = text.parse::<f32>().unwrap_or(value as f32);
        Outcome::All(vec![
            Outcome::Saved(Model::Graphics, self.read.graphics.clone()),
            Outcome::Lever { console: console.to_owned(), id: lever.id.clone(), value },
        ])
    }

    /// A on a shader page's row, or on the doors that open them.
    pub(crate) fn shaders_choose(&mut self, entry: &Value, index: usize) -> Option<Outcome> {
        let label = text_of(&entry["label"]);
        match entry["door"].as_str() {
            Some("shaders") => return Some(self.open(Page::ShaderConsoles { levers: false }, label, index)),
            Some("levers") => {
                let console = self.page().and_then(|p| match p {
                    Page::Console(c) => Some(c.clone()),
                    _ => None,
                });
                return Some(match console {
                    Some(c) => self.open(Page::Levers(c), self.fill(&label, ""), index),
                    None => self.open(Page::ShaderConsoles { levers: true }, label, index),
                });
            }
            _ => {}
        }
        if let Some(console) = entry["shader_console"].as_str() {
            let levers = matches!(self.page(), Some(Page::ShaderConsoles { levers: true }));
            let page = if levers { Page::Levers(console.to_owned()) } else { Page::Shaders(console.to_owned()) };
            return Some(self.open(page, self.console_name(console), index));
        }
        if let Some(stored) = entry["shader"].as_str() {
            let Some(Page::Shaders(console)) = self.page().cloned() else { return None };
            set_at(&mut self.read.graphics, &format!("Consoles.{console}.ScreenFilter"), json!(stored));
            let recent = &mut self.read.graphics["RecentShaders"];
            if !recent.is_object() {
                *recent = json!({});
            }
            let mut list: Vec<Value> = recent[&console].as_array().cloned().unwrap_or_default();
            list.retain(|s| s != stored);
            list.insert(0, json!(stored));
            list.truncate(RECENT_KEPT);
            recent[&console] = Value::Array(list);
            let saved = Outcome::Saved(Model::Graphics, self.read.graphics.clone());
            // A pack preset's levers are read from its files, so the settings are read again for them.
            return Some(if pack_preset(stored).is_some() { Outcome::All(vec![saved, Outcome::Reread]) } else { saved });
        }
        if entry.get("lever").is_some() || entry.get("heading").is_some() && matches!(self.page(), Some(Page::Levers(_))) {
            return Some(Outcome::Nothing);
        }
        None
    }
}
