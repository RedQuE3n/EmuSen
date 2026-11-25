//! Settings ▸ Display ▸ Shaders and Shader levers, and a console's Shader levers (EmuSen_Qt.md §7.7): the shaders a
//! console may draw with as a page of choices (its recent ones, Serenity's built-in filters that suit it, the shader
//! pack's presets), stored in its `ScreenFilter`; and the levers of the one it draws with, each a parameter as the base
//! reports it (a built-in filter's settings from Serenity's catalogue, a pack preset's as its passes declare them),
//! stored in `ShaderParameters` and set on a running game's chain at once.

use crate::display::{pack_preset, PACK_PREFIX};
use crate::settings::{at, set_at, text_of, wrapped, Model, Outcome, Page, Pane, Preview, Row, SettingsView};
use crate::settings_controllers::Typed;
use crate::settings_shown::{BoardAside, BoardRow, Form};
use emusen_kit_core::settings::{RowKind, SliderRange};
use serde_json::{json, Map, Value};

/// How many shaders a console's recent list keeps, newest first.
pub const RECENT_KEPT: usize = 5;

/// The seven CRT levers board #59 puts first ahead of Serenity's remaining CRT settings.
pub const CRT_BOARD_ORDER: [&str; 7] = ["signal", "standard", "curvature", "level", "mask", "maskDepth", "glare"];

/// Whether a numeric CRT lever is a relative multiplier shown as a percentage on board #59.
fn crt_percent_lever(id: &str) -> bool {
    !matches!(id, "displayNits" | "tubeNits" | "gamma")
}

/// Whether a pack shader parameter is a 0/1 boolean toggle shown as On/Off (board #60).
pub(crate) fn is_bool_param(lever: &Lever) -> bool {
    lever.choices.is_empty() && !lever.heading && lever.range.minimum == 0.0 && lever.range.maximum == 1.0 && (lever.range.step - 1.0).abs() < 1e-6
}

/// Formats a CRT lever's value for Eclipse board #59: choice text, percentage, or decimal.
pub(crate) fn format_crt_value(lever: &Lever, value: f64) -> String {
    if !lever.choices.is_empty() {
        let at = (value.round().max(0.0) as usize).min(lever.choices.len() - 1);
        return lever.choices[at].clone();
    }
    if crt_percent_lever(&lever.id) {
        format!("{}%", (value * 100.0).round() as i64)
    } else {
        decimal(value)
    }
}

/// Formats a pack shader parameter's value for Eclipse board #60: On/Off for 0..1 step 1, else decimal.
pub(crate) fn format_param_value(lever: &Lever, value: f64) -> String {
    if is_bool_param(lever) {
        if value >= 0.5 { "On" } else { "Off" }.to_owned()
    } else {
        decimal(value)
    }
}

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

    /// The console's CRT levers ordered with board #59's seven levers first, then the rest in Serenity's order.
    fn ordered_levers(&self, console: &str) -> Result<Vec<Lever>, String> {
        let mut list = self.levers(console)?;
        if !self.themed_settings || pack_preset(&self.shader_of(console)).is_some() {
            return Ok(list);
        }
        let mut head = Vec::with_capacity(list.len());
        for id in CRT_BOARD_ORDER {
            if let Some(pos) = list.iter().position(|l| l.id == id) {
                head.push(list.remove(pos));
            }
        }
        head.extend(list);
        Ok(head)
    }

    /// The rows of a shader page, or none for another page.
    pub(crate) fn shaders_entries(&self, page: &Page) -> Option<Vec<Value>> {
        match page {
            Page::ShaderConsoles { .. } => {
                Some(self.consoles("every").into_iter().map(|c| json!({ "id": format!("shaders:{c}"), "shader_console": c })).collect())
            }
            Page::Shaders(console) => {
                let mut out = Vec::new();
                if self.themed_settings && pack_preset(&self.shader_of(console)).is_some() {
                    out.push(json!({ "id": "shaders:params", "shader_params": console }));
                }
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
            Page::Levers(console) => Some(match self.ordered_levers(console) {
                Ok(levers) if !levers.is_empty() => {
                    let mut out: Vec<Value> = levers.iter().map(|l| json!({ "id": format!("lever:{}", l.id), "lever": l.id })).collect();
                    if self.themed_settings {
                        out.push(json!({ "id": "levers:reset", "levers_reset": true }));
                    }
                    out
                }
                Ok(_) => vec![json!({ "id": "levers:none", "heading": self.shaders_word("none") })],
                Err(words) => vec![json!({ "id": "levers:failed", "heading": words })],
            }),
            Page::ShaderParams(console) => {
                let mut out = vec![
                    json!({ "id": "params:search", "params_search": true }),
                    json!({ "id": "params:reset", "params_reset": true }),
                ];
                let query = self.param_search.to_lowercase();
                match self.levers(console) {
                    Ok(levers) => {
                        let filtered: Vec<&Lever> = levers
                            .iter()
                            .filter(|l| {
                                if query.is_empty() {
                                    true
                                } else {
                                    !l.heading && (l.label.to_lowercase().contains(&query) || l.id.to_lowercase().contains(&query))
                                }
                            })
                            .collect();
                        if filtered.is_empty() {
                            let words = if query.is_empty() { self.shaders_word("none") } else { "No parameters match.".to_owned() };
                            out.push(json!({ "id": "params:none", "heading": words }));
                        } else {
                            out.extend(filtered.into_iter().map(|l| json!({ "id": format!("lever:{}", l.id), "lever": l.id })));
                        }
                    }
                    Err(words) => out.push(json!({ "id": "params:failed", "heading": words })),
                }
                Some(out)
            }
            _ => None,
        }
    }

    fn lever_of(&self, entry: &Value) -> Option<(String, Lever)> {
        let console = match self.page() {
            Some(Page::Levers(c) | Page::ShaderParams(c)) => c,
            _ => return None,
        };
        let id = entry["lever"].as_str()?;
        let lever = self.levers(console).ok()?.into_iter().find(|l| l.id == id)?;
        Some((console.clone(), lever))
    }

    pub(crate) fn shaders_row(&self, entry: &Value) -> Option<Row> {
        let id = text_of(&entry["id"]);
        let row = |label: String, value: String, kind: RowKind, held: bool, marked: bool| Some(Row { id: id.clone(), label, value, kind, held, marked, code: false });
        if let Some(console) = entry["shader_console"].as_str() {
            return row(self.console_name(console), Self::shader_name(&self.shader_of(console)), RowKind::Opens, false, false);
        }
        if let Some(console) = entry["shader_params"].as_str() {
            return row("Parameters".to_owned(), Self::shader_name(&self.shader_of(console)), RowKind::Opens, false, false);
        }
        if entry["levers_reset"] == true {
            return row("Reset to the tier's values".to_owned(), String::new(), RowKind::Action, false, false);
        }
        if entry["params_search"] == true {
            let value = if self.param_search.is_empty() { "…".to_owned() } else { self.param_search.clone() };
            return row("Search parameters".to_owned(), value, RowKind::Opens, false, false);
        }
        if entry["params_reset"] == true {
            return row("Reset all".to_owned(), String::new(), RowKind::Action, false, false);
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
            if matches!(self.page(), Some(Page::ShaderParams(_))) {
                let kind = if is_bool_param(&lever) { RowKind::Steps } else { RowKind::Slider };
                return row(lever.label.clone(), format_param_value(&lever, value), kind, false, false);
            }
            if lever.choices.is_empty() {
                return row(lever.label, decimal(value), RowKind::Slider, false, false);
            }
            let at = (value.round().max(0.0) as usize).min(lever.choices.len() - 1);
            return row(lever.label, lever.choices[at].clone(), RowKind::Steps, false, false);
        }
        if matches!(self.page(), Some(Page::Shaders(_) | Page::Levers(_) | Page::ShaderParams(_) | Page::ShaderConsoles { .. })) {
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
        if lever.heading || !lever.choices.is_empty() || (matches!(self.page(), Some(Page::ShaderParams(_))) && is_bool_param(&lever)) {
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
        if let Some(console) = entry["shader_params"].as_str() {
            let sentence = format!("The parameters of {}, for the {}.", Self::shader_name(&self.shader_of(console)), self.console_name(console));
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
        let pane = self.description_at(&["shaders", "pane"]);
        if entry["levers_reset"] == true {
            return Some(Pane {
                sentence: "Every lever back to what the tier sets.".to_owned(),
                preview: Preview::Picture,
                caption: self.fill(&text_of(&pane["caption"]), ""),
                ..Pane::default()
            });
        }
        if matches!(self.page(), Some(Page::ShaderParams(_))) {
            let console = match self.page() {
                Some(Page::ShaderParams(c)) => c,
                _ => unreachable!(),
            };
            let sentence = format!("The parameters of {}, for the {}.", Self::shader_name(&self.shader_of(console)), self.console_name(console));
            return Some(Pane {
                sentence,
                preview: Preview::Picture,
                caption: self.fill(&text_of(&pane["caption"]), ""),
                ..Pane::default()
            });
        }
        let (_, lever) = self.lever_of(entry)?;
        let own = if lever.choices.is_empty() { decimal(lever.own) } else { lever.choices.get(lever.own.round().max(0.0) as usize).cloned().unwrap_or_default() };
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
        let next = if !lever.choices.is_empty() {
            wrapped((now.round().max(0.0) as usize).min(lever.choices.len() - 1), by, lever.choices.len()) as f64
        } else if matches!(self.page(), Some(Page::ShaderParams(_))) && is_bool_param(&lever) {
            if now >= 0.5 { 0.0 } else { 1.0 }
        } else {
            lever.range.stepped(now, by)
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

    /// Resets stored shader parameters for `console` back to their defaults, optionally preserving `keep` (CRT `quality`).
    fn reset_levers(&mut self, console: &str, keep: Option<&str>) -> Outcome {
        let stored = self.shader_of(console);
        let levers = self.levers(console).unwrap_or_default();
        if let Some(map) = self.read.graphics["ShaderParameters"][console][&stored].as_object_mut() {
            map.retain(|k, _| keep == Some(k.as_str()));
        }
        if self.read.graphics["ShaderParameters"][console][&stored].as_object().is_some_and(Map::is_empty) {
            if let Some(c) = self.read.graphics["ShaderParameters"][console].as_object_mut() {
                c.remove(&stored);
            }
        }
        if self.read.graphics["ShaderParameters"][console].as_object().is_some_and(Map::is_empty) {
            if let Some(p) = self.read.graphics["ShaderParameters"].as_object_mut() {
                p.remove(console);
            }
        }
        let mut all = vec![Outcome::Saved(Model::Graphics, self.read.graphics.clone())];
        for l in levers.into_iter().filter(|l| !l.heading && keep != Some(l.id.as_str())) {
            all.push(Outcome::Lever { console: console.to_owned(), id: l.id, value: l.own as f32 });
        }
        Outcome::All(all)
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
            if levers && self.themed_settings && pack_preset(&self.shader_of(console)).is_some() {
                self.param_search.clear();
                return Some(self.open(Page::ShaderParams(console.to_owned()), "Parameters".to_owned(), index));
            }
            let page = if levers { Page::Levers(console.to_owned()) } else { Page::Shaders(console.to_owned()) };
            return Some(self.open(page, self.console_name(console), index));
        }
        if let Some(console) = entry["shader_params"].as_str() {
            self.param_search.clear();
            return Some(self.open(Page::ShaderParams(console.to_owned()), "Parameters".to_owned(), index));
        }
        if entry["levers_reset"] == true {
            let Some(Page::Levers(console)) = self.page().cloned() else { return None };
            return Some(self.reset_levers(&console, Some("quality")));
        }
        if entry["params_search"] == true {
            return Some(Outcome::Type {
                title: "Search parameters".to_owned(),
                prompt: "Words in a parameter's name.".to_owned(),
                start: self.param_search.clone(),
                typed: Typed::SearchParameters,
            });
        }
        if entry["params_reset"] == true {
            let Some(Page::ShaderParams(console)) = self.page().cloned() else { return None };
            return Some(self.reset_levers(&console, None));
        }
        if let Some(stored) = entry["shader"].as_str() {
            let Some(Page::Shaders(console)) = self.page().cloned() else { return None };
            if self.themed_settings && pack_preset(stored).is_some() && self.shader_of(&console) == stored {
                self.param_search.clear();
                return Some(self.open(Page::ShaderParams(console), "Parameters".to_owned(), index));
            }
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
        if entry.get("lever").is_some() {
            if self.themed_settings {
                return self.shaders_step(entry, 1);
            }
            return Some(Outcome::Nothing);
        }
        if entry.get("heading").is_some() && matches!(self.page(), Some(Page::Levers(_) | Page::ShaderParams(_))) {
            return Some(Outcome::Nothing);
        }
        None
    }

    /// Eclipse's board-exact row projection for Levers (#59) and Parameters (#60).
    pub(crate) fn shaders_board_row(&self, row: &Row, entry: &Value) -> Option<BoardRow> {
        let mut out = BoardRow {
            label: row.label.clone(),
            about: String::new(),
            value: row.value.clone(),
            form: Form::Door,
            planet: -3,
            subtitle: String::new(),
            chip: String::new(),
            chip_attention: false,
            runs: true,
        };
        if entry["levers_reset"] == true {
            out.about = "Every lever back to what the tier sets.".to_owned();
            return Some(out);
        }
        if entry["params_search"] == true {
            out.about = "Words in a parameter's name.".to_owned();
            return Some(out);
        }
        if entry["params_reset"] == true {
            out.about = "Every parameter back to the shader's own value.".to_owned();
            return Some(out);
        }
        if let Some(console) = entry["shader_params"].as_str() {
            out.about = format!("The parameters of {}, for the {}.", Self::shader_name(&self.shader_of(console)), self.console_name(console));
            return Some(out);
        }
        let (console, lever) = self.lever_of(entry)?;
        let cname = self.console_name(&console);
        if lever.heading {
            out.form = Form::Fact;
            return Some(out);
        }
        let val = self.lever_value(&console, &lever);
        match self.page() {
            Some(Page::Levers(_)) => {
                out.about = format!("{}, for the {cname}.", lever.label);
                out.value = format_crt_value(&lever, val);
                out.form = Form::Choice;
                Some(out)
            }
            Some(Page::ShaderParams(_)) => {
                let own = format_param_value(&lever, lever.own);
                out.about = format!("{}, for the {cname}; {own} is the shader's own.", lever.label);
                out.value = format_param_value(&lever, val);
                out.form = Form::Choice;
                Some(out)
            }
            _ => None,
        }
    }

    /// Eclipse's board-exact header, glow and aside projection for Levers (#59) and Parameters (#60).
    pub(crate) fn shaders_board_aside(&self, focused: usize, board_rows: &[BoardRow], pane: &Pane, out: &mut BoardAside) -> bool {
        match self.page() {
            Some(Page::Levers(_)) => {
                let on_display = self.section_keys().get(self.section).map(String::as_str) == Some("display");
                out.trail_line = if on_display { "Display".to_owned() } else { self.trail_line() };
                out.heading = "Shader levers".to_owned();
                out.page_glow = "#6A4CF0".to_owned();
                out.about_heading.clear();
                out.lines.clear();
                out.muted.clear();
                out.wrap_about = true;
                out.cards_help = true;
                out.box_menu = true;
                if let Some(br) = board_rows.get(focused) {
                    out.sentence = if br.form == Form::Choice {
                        format!("{} at {}. Each lever redraws the preview; the tier's own values come back with Reset.", br.label, br.value)
                    } else {
                        pane.sentence.clone()
                    };
                }
                true
            }
            Some(Page::ShaderParams(console)) => {
                let b = crate::words::TRAIL_BETWEEN;
                out.trail_line = format!("Display{b}Shaders{b}{}", self.console_name(console));
                out.heading = "Parameters".to_owned();
                out.page_glow = "#6A4CF0".to_owned();
                out.about_heading.clear();
                out.sentence = pane.sentence.clone();
                out.lines.clear();
                out.muted.clear();
                out.wrap_about = true;
                out.cards_help = true;
                out.box_menu = true;
                true
            }
            Some(Page::ShaderConsoles { .. } | Page::Shaders(_)) => {
                out.page_glow = "#6A4CF0".to_owned();
                true
            }
            _ => false,
        }
    }
}
