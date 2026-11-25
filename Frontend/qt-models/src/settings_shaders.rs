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
            return Ok(levers_from_json(&read));
        }
        let Some(filter) = self.builtin(console, &stored) else { return Ok(Vec::new()) };
        Ok(levers_from_builtin(filter, console))
    }

    /// A lever's value on `console`: the one stored, else the shader's own.
    fn lever_value(&self, console: &str, lever: &Lever) -> f64 {
        let stored = self.shader_of(console);
        stored_lever_value(&self.read.graphics, console, &stored, lever)
    }

    /// The console's CRT levers ordered with board #59's seven levers first, then the rest in Serenity's order.
    fn ordered_levers(&self, console: &str) -> Result<Vec<Lever>, String> {
        let list = self.levers(console)?;
        if !self.themed_settings || pack_preset(&self.shader_of(console)).is_some() {
            return Ok(list);
        }
        Ok(order_crt_levers(list))
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

    /// A lever's value saved in `ShaderParameters` under `console`'s `stored` filter, forgotten at its own value.
    pub(crate) fn store_lever_for(&mut self, console: &str, stored: &str, lever: &Lever, value: f64) -> Outcome {
        let text = decimal(value);
        let parameters = &mut self.read.graphics["ShaderParameters"];
        if !parameters.is_object() {
            *parameters = json!({});
        }
        if text == decimal(lever.own) {
            if let Some(held) = parameters[console][stored].as_object_mut() {
                held.remove(&lever.id);
                if held.is_empty() {
                    parameters[console].as_object_mut().map(|c| c.remove(stored));
                }
            }
            if parameters[console].as_object().is_some_and(Map::is_empty) {
                parameters.as_object_mut().map(|p| p.remove(console));
            }
        } else {
            parameters[console][stored][&lever.id] = json!(text);
        }
        let value = text.parse::<f32>().unwrap_or(value as f32);
        Outcome::All(vec![
            Outcome::Saved(Model::Graphics, self.read.graphics.clone()),
            Outcome::Lever { console: console.to_owned(), id: lever.id.clone(), value },
        ])
    }

    fn store_lever(&mut self, console: &str, lever: &Lever, value: f64) -> Outcome {
        let stored = self.shader_of(console);
        self.store_lever_for(console, &stored, lever, value)
    }

    /// Saves `stored` into `Consoles.<console>.ScreenFilter` and updates `RecentShaders.<console>`.
    pub(crate) fn apply_shader_choice(&mut self, console: &str, stored: &str) -> Outcome {
        set_at(&mut self.read.graphics, &format!("Consoles.{console}.ScreenFilter"), json!(stored));
        let recent = &mut self.read.graphics["RecentShaders"];
        if !recent.is_object() {
            *recent = json!({});
        }
        let mut list: Vec<Value> = recent[console].as_array().cloned().unwrap_or_default();
        list.retain(|s| s != stored);
        list.insert(0, json!(stored));
        list.truncate(RECENT_KEPT);
        recent[console] = Value::Array(list);
        let saved = Outcome::Saved(Model::Graphics, self.read.graphics.clone());
        if pack_preset(stored).is_some() { Outcome::All(vec![saved, Outcome::Reread]) } else { saved }
    }

    /// Whether `index` on the active page is Display's `Shaders` door.
    pub(crate) fn is_shaders_door(&self, index: usize) -> bool {
        self.entries().get(index).and_then(|e| e["door"].as_str()) == Some("shaders")
    }

    /// Opens `#60` (`ShaderParams`) or `#59` (`Levers`) for `console` directly from `#20`.
    pub(crate) fn open_shader_settings_page(&mut self, console: &str, stored: &str, index: usize) -> Outcome {
        self.param_search.clear();
        if pack_preset(stored).is_some() {
            self.open(Page::ShaderParams(console.to_owned()), "Parameters".to_owned(), index)
        } else {
            self.open(Page::Levers(console.to_owned()), "Shader levers".to_owned(), index)
        }
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
            return Some(self.apply_shader_choice(&console, stored));
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

/// The eight Eclipse #20 category filters in board order (`EclipseShaders.dc.html:113`).
pub const SHADER_CATEGORIES: [(&str, &str); 8] = [
    ("favourites", "Favourites"),
    ("crt", "CRT"),
    ("handheld", "Handheld panels"),
    ("broadcast", "Broadcast and composite"),
    ("scaling", "Sharpening and scaling"),
    ("colour", "Colour and film"),
    ("bezel", "Borders and bezels"),
    ("console", "Made for one console"),
];

const MONTHS: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

#[derive(Clone, Debug, Default)]
pub struct PresetMeta {
    pub passes: Option<usize>,
    pub levers: Vec<Lever>,
    pub error: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ShaderItem {
    pub stored: String,
    pub name: String,
    pub category: usize,
    pub passes: Option<usize>,
    pub settings_count: usize,
    pub builtin_key: Option<String>,
}

#[derive(Clone, Debug)]
pub enum ShadersOutcome {
    None,
    Changed,
    Saved(Outcome),
    Search { title: String, prompt: String, start: String },
    OpenSettingsPage { console: String, stored: String, return_row: usize },
    RemoveConfirmed,
}

/// Interactive state for Eclipse board #20 (`ShadersEclipse.qml`).
#[derive(Clone, Debug)]
pub struct ShadersBoard {
    pub return_row: usize,
    pub return_params: bool,
    pub source: usize,
    pub category: usize,
    pub scope: usize,
    pub console_index: usize,
    pub consoles: Vec<(String, String)>,
    pub focus_zone: usize,
    pub focused_preset: usize,
    pub focused_slider: usize,
    pub search: String,
    pub split: f64,
    pub comparing: bool,
    pub confirm_remove: bool,
    pub favourites: Vec<String>,
    pub pack_dir: Option<std::path::PathBuf>,
    pub pack_bytes: u64,
    pub pack_date: String,
    pub meta_cache: std::collections::BTreeMap<String, PresetMeta>,
    pub preview_stored: Option<String>,
}

impl ShadersBoard {
    pub fn open(view: &SettingsView, platform_library: &str, return_row: usize) -> Self {
        let consoles: Vec<(String, String)> = view
            .consoles("every")
            .into_iter()
            .map(|c| (c.clone(), view.console_name(&c)))
            .collect();
        let running = view.facts["system"].as_str().filter(|s| !s.is_empty());
        let console_index = running
            .and_then(|r| consoles.iter().position(|(c, _)| c == r))
            .or_else(|| {
                let systems = view.facts["systems"].as_object()?;
                consoles.iter().position(|(c, _)| systems.get(c).and_then(Value::as_u64).unwrap_or(0) > 0)
            })
            .or_else(|| consoles.iter().position(|(c, _)| c == "SNES"))
            .unwrap_or(0);
        let active = consoles
            .get(console_index)
            .map(|(c, _)| view.shader_of(c))
            .unwrap_or_default();
        let favourites = load_favourites(platform_library);
        let (pack_dir, pack_bytes, pack_date) = scan_pack_dir(platform_library);
        let has_pack = !view.read.pack.is_empty();
        let source = if pack_preset(&active).is_some() || (active == "None" && has_pack) { 1 } else { 0 };
        let category = default_category(view, source, &active);
        let mut board = ShadersBoard {
            return_row,
            return_params: false,
            source,
            category,
            scope: 1,
            console_index,
            consoles,
            focus_zone: 4,
            focused_preset: 0,
            focused_slider: 0,
            search: String::new(),
            split: 0.5,
            comparing: false,
            confirm_remove: false,
            favourites,
            pack_dir,
            pack_bytes,
            pack_date,
            meta_cache: std::collections::BTreeMap::new(),
            preview_stored: None,
        };
        let list = board.visible_items(view);
        if let Some(pos) = list.iter().position(|i| i.stored == active) {
            board.focused_preset = pos;
        }
        board.sync_preview_stored(view, platform_library);
        board
    }

    pub fn active_console(&self) -> Option<&str> {
        self.consoles.get(self.console_index).map(|(c, _)| c.as_str())
    }

    pub fn active_console_name(&self) -> &str {
        self.consoles.get(self.console_index).map_or("Every console", |(_, n)| n.as_str())
    }

    fn builtin_items(&self, view: &SettingsView) -> Vec<ShaderItem> {
        let console = if self.scope == 1 { self.active_console() } else { None };
        let filters = view.read.catalogue["filters"].as_array().map(Vec::as_slice).unwrap_or_default();
        filters
            .iter()
            .filter(|f| {
                let key = text_of(&f["key"]);
                if key.starts_with("effect.") && key.ends_with(".linear") {
                    return false;
                }
                console.is_none_or(|c| f["consoles"].as_array().is_none_or(|list| list.iter().any(|x| x == c)))
            })
            .map(|f| {
                let key = text_of(&f["key"]);
                let stored = text_of(&f["name"]);
                let passes = f["passes"].as_array().map(Vec::len);
                let settings_count = f["settings"].as_array().map_or(0, Vec::len);
                ShaderItem {
                    category: categorize_builtin(&key, f["consoles"].as_array().is_some()),
                    name: stored.clone(),
                    stored,
                    passes: passes.filter(|&p| p > 0 || !f["built"].as_bool().unwrap_or(false)),
                    settings_count,
                    builtin_key: Some(key),
                }
            })
            .collect()
    }

    fn pack_items(&self, view: &SettingsView) -> Vec<ShaderItem> {
        view.read
            .pack
            .iter()
            .map(|path| {
                let stored = format!("{PACK_PREFIX}{path}");
                let cached = self.meta_cache.get(&stored);
                ShaderItem {
                    name: SettingsView::shader_name(&stored),
                    category: categorize_pack(path),
                    passes: cached.and_then(|m| m.passes),
                    settings_count: cached.map_or(0, |m| m.levers.len()),
                    stored,
                    builtin_key: None,
                }
            })
            .collect()
    }

    pub fn source_items(&self, view: &SettingsView, source: usize) -> Vec<ShaderItem> {
        if source == 0 { self.builtin_items(view) } else { self.pack_items(view) }
    }

    pub fn category_counts(&self, view: &SettingsView) -> [usize; 8] {
        let items = self.source_items(view, self.source);
        let mut counts = [0usize; 8];
        for item in &items {
            if self.is_favourite(&item.stored) {
                counts[0] += 1;
            }
            if item.category < 8 {
                counts[item.category] += 1;
            }
        }
        counts
    }

    pub fn is_favourite(&self, stored: &str) -> bool {
        self.favourites.iter().any(|f| f == stored)
    }

    pub fn visible_items(&self, view: &SettingsView) -> Vec<ShaderItem> {
        let q = self.search.trim().to_ascii_lowercase();
        self.source_items(view, self.source)
            .into_iter()
            .filter(|item| {
                let in_cat = if self.category == 0 { self.is_favourite(&item.stored) } else { item.category == self.category };
                in_cat && (q.is_empty() || item.name.to_ascii_lowercase().contains(&q) || item.stored.to_ascii_lowercase().contains(&q))
            })
            .collect()
    }

    pub fn focused_item(&self, view: &SettingsView) -> Option<ShaderItem> {
        let list = self.visible_items(view);
        list.get(self.focused_preset.min(list.len().saturating_sub(1))).cloned()
    }

    pub fn sync_preview_stored(&mut self, view: &SettingsView, platform_library: &str) {
        let items = self.visible_items(view);
        if items.is_empty() {
            self.focused_preset = 0;
            self.preview_stored = None;
            return;
        }
        self.focused_preset = self.focused_preset.min(items.len() - 1);
        for item in items.iter().take(24) {
            if item.builtin_key.is_none() {
                self.ensure_meta(view, platform_library, &item.stored);
            }
        }
        let target = &items[self.focused_preset];
        if target.builtin_key.is_none() {
            self.ensure_meta(view, platform_library, &target.stored);
        }
        self.preview_stored = Some(target.stored.clone());
        let slider_len = self.sliders(view, platform_library).len();
        if slider_len == 0 {
            self.focused_slider = 0;
        } else {
            self.focused_slider = self.focused_slider.min(slider_len - 1);
        }
    }

    pub fn ensure_meta(&mut self, view: &SettingsView, platform_library: &str, stored: &str) {
        if self.meta_cache.contains_key(stored) {
            return;
        }
        let Some(pack_rel) = pack_preset(stored) else { return };
        if let Some(Ok(raw_params)) = view.read.levers.get(stored) {
            let passes = read_pack_passes(platform_library, pack_rel);
            self.meta_cache.insert(stored.to_owned(), PresetMeta { passes, levers: levers_from_json(raw_params), error: None });
            return;
        }
        if platform_library.is_empty() {
            return;
        }
        let meta = read_pack_meta(platform_library, pack_rel);
        self.meta_cache.insert(stored.to_owned(), meta);
    }

    fn levers_for(&mut self, view: &SettingsView, platform_library: &str, stored: &str, console: &str) -> (Option<usize>, Vec<Lever>) {
        if pack_preset(stored).is_some() {
            self.ensure_meta(view, platform_library, stored);
            return self.meta_cache.get(stored).map_or((None, Vec::new()), |m| (m.passes, m.levers.clone()));
        }
        let Some(filter) = view.builtin(console, stored) else {
            let any_console = view.read.catalogue["filters"].as_array().and_then(|fs| fs.iter().find(|f| f["name"] == stored || f["key"] == stored));
            let Some(f) = any_console else { return (None, Vec::new()) };
            return (f["passes"].as_array().map(Vec::len).filter(|&p| p > 0), levers_from_builtin(f, console));
        };
        let passes = filter["passes"].as_array().map(Vec::len).filter(|&p| p > 0 || !filter["built"].as_bool().unwrap_or(false));
        let mut list = levers_from_builtin(filter, console);
        if stored == "CRT" {
            list = order_crt_levers(list);
        }
        (passes, list)
    }

    fn current_console(&self) -> &str {
        self.active_console().unwrap_or("SNES")
    }

    fn current_stored(&self, view: &SettingsView) -> String {
        self.preview_stored.clone().unwrap_or_else(|| view.shader_of(self.current_console()))
    }

    fn stored_value(&self, view: &SettingsView, console: &str, stored: &str, lever: &Lever) -> f64 {
        stored_lever_value(&view.read.graphics, console, stored, lever)
    }

    pub fn sliders(&mut self, view: &SettingsView, platform_library: &str) -> Vec<(Lever, f64)> {
        let stored = self.current_stored(view);
        let console = self.current_console().to_owned();
        let (_, levers) = self.levers_for(view, platform_library, &stored, &console);
        levers
            .into_iter()
            .filter(|l| !l.heading && l.choices.is_empty() && l.range.maximum > l.range.minimum)
            .take(3)
            .map(|l| {
                let v = self.stored_value(view, &console, &stored, &l);
                (l, v)
            })
            .collect()
    }

    pub fn preview_target(&mut self, view: &SettingsView, platform_library: &str) -> crate::shader_preview::PreviewTarget {
        let stored = self.current_stored(view);
        let console = self.current_console().to_owned();
        let (_, levers) = self.levers_for(view, platform_library, &stored, &console);
        let mut values = std::collections::BTreeMap::new();
        for l in &levers {
            if !l.heading {
                values.insert(l.id.clone(), self.stored_value(view, &console, &stored, l) as f32);
            }
        }
        crate::shader_preview::PreviewTarget { stored, console: (self.scope == 1).then_some(console), values }
    }

    pub fn nav(&mut self, view: &SettingsView, platform_library: &str, dx: i32, dy: i32) -> ShadersOutcome {
        let sliders_len = self.sliders(view, platform_library).len();
        let visible_len = self.visible_items(view).len();
        let stored = self.current_stored(view);
        let console = self.current_console().to_owned();
        let total_settings = self.levers_for(view, platform_library, &stored, &console).1.iter().filter(|l| !l.heading).count();
        let has_remove = !view.read.pack.is_empty();
        match self.focus_zone {
            0 => {
                if dx != 0 {
                    self.set_source(view, platform_library, 1 - self.source);
                    return ShadersOutcome::Changed;
                }
                if dy > 0 {
                    self.focus_zone = 1;
                    return ShadersOutcome::Changed;
                }
            }
            1 => {
                if dy < 0 {
                    if self.category > 0 {
                        self.set_category(view, platform_library, self.category - 1);
                    } else {
                        self.focus_zone = 0;
                    }
                    return ShadersOutcome::Changed;
                }
                if dy > 0 {
                    if self.category + 1 < SHADER_CATEGORIES.len() {
                        self.set_category(view, platform_library, self.category + 1);
                    } else if has_remove {
                        self.focus_zone = 2;
                    }
                    return ShadersOutcome::Changed;
                }
                if dx > 0 {
                    self.focus_zone = if visible_len > 0 { 4 } else { 3 };
                    return ShadersOutcome::Changed;
                }
            }
            2 => {
                if dy < 0 {
                    self.focus_zone = 1;
                    return ShadersOutcome::Changed;
                }
                if dx > 0 {
                    self.focus_zone = if visible_len > 0 { 4 } else { 3 };
                    return ShadersOutcome::Changed;
                }
            }
            3 => {
                if dx < 0 {
                    self.focus_zone = 1;
                    return ShadersOutcome::Changed;
                }
                if dx > 0 {
                    self.focus_zone = 5;
                    return ShadersOutcome::Changed;
                }
                if dy > 0 && visible_len > 0 {
                    self.focus_zone = 4;
                    return ShadersOutcome::Changed;
                }
            }
            4 => {
                if dy < 0 {
                    if self.focused_preset > 0 {
                        self.focused_preset -= 1;
                        self.sync_preview_stored(view, platform_library);
                    } else {
                        self.focus_zone = 3;
                    }
                    return ShadersOutcome::Changed;
                }
                if dy > 0 && self.focused_preset + 1 < visible_len {
                    self.focused_preset += 1;
                    self.sync_preview_stored(view, platform_library);
                    return ShadersOutcome::Changed;
                }
                if dx < 0 {
                    self.focus_zone = 1;
                    return ShadersOutcome::Changed;
                }
                if dx > 0 {
                    self.focus_zone = if sliders_len > 0 {
                        self.focused_slider = 0;
                        6
                    } else if total_settings > 0 {
                        7
                    } else {
                        5
                    };
                    return ShadersOutcome::Changed;
                }
            }
            5 => {
                if dx != 0 {
                    let next = if dx > 0 { (self.scope + 1).min(1) } else { self.scope.saturating_sub(1) };
                    if next != self.scope {
                        self.set_scope(view, platform_library, next);
                        return ShadersOutcome::Changed;
                    }
                    if dx < 0 {
                        self.focus_zone = 3;
                        return ShadersOutcome::Changed;
                    }
                }
                if dy > 0 {
                    self.focus_zone = if sliders_len > 0 {
                        self.focused_slider = 0;
                        6
                    } else if total_settings > 0 {
                        7
                    } else {
                        5
                    };
                    return ShadersOutcome::Changed;
                }
            }
            6 => {
                if dy < 0 {
                    if self.focused_slider > 0 {
                        self.focused_slider -= 1;
                    } else {
                        self.focus_zone = 5;
                    }
                    return ShadersOutcome::Changed;
                }
                if dy > 0 {
                    if self.focused_slider + 1 < sliders_len {
                        self.focused_slider += 1;
                    } else if total_settings > 0 {
                        self.focus_zone = 7;
                    }
                    return ShadersOutcome::Changed;
                }
            }
            7 => {
                if dy < 0 {
                    self.focus_zone = if sliders_len > 0 {
                        self.focused_slider = sliders_len - 1;
                        6
                    } else {
                        5
                    };
                    return ShadersOutcome::Changed;
                }
                if dx < 0 {
                    self.focus_zone = 4;
                    return ShadersOutcome::Changed;
                }
            }
            _ => {}
        }
        ShadersOutcome::None
    }

    pub fn set_source(&mut self, view: &SettingsView, platform_library: &str, source: usize) {
        self.source = source.min(1);
        self.focused_preset = 0;
        self.sync_preview_stored(view, platform_library);
    }

    pub fn set_category(&mut self, view: &SettingsView, platform_library: &str, cat: usize) {
        self.category = cat.min(SHADER_CATEGORIES.len() - 1);
        self.focused_preset = 0;
        self.sync_preview_stored(view, platform_library);
    }

    pub fn set_scope(&mut self, view: &SettingsView, platform_library: &str, scope: usize) {
        self.scope = scope.min(1);
        self.sync_preview_stored(view, platform_library);
    }

    pub fn step_console(&mut self, view: &SettingsView, platform_library: &str, by: i32) -> bool {
        if self.consoles.len() < 2 {
            return false;
        }
        self.console_index = wrapped(self.console_index, by, self.consoles.len());
        self.scope = 1;
        self.sync_preview_stored(view, platform_library);
        true
    }

    pub fn apply_current(&self, view: &mut SettingsView) -> Outcome {
        let Some(stored) = self.preview_stored.clone() else { return Outcome::Nothing };
        let targets: Vec<String> = if self.scope == 0 {
            self.consoles.iter().filter(|(c, _)| pack_preset(&stored).is_some() || view.builtin(c, &stored).is_some()).map(|(c, _)| c.clone()).collect()
        } else {
            self.active_console().map(|c| vec![c.to_owned()]).unwrap_or_default()
        };
        if targets.is_empty() {
            return Outcome::Nothing;
        }
        let mut all = Vec::new();
        for c in targets {
            let outcome = view.apply_shader_choice(&c, &stored);
            if matches!(outcome, Outcome::All(_)) {
                all.push(Outcome::Reread);
            }
        }
        let saved = Outcome::Saved(Model::Graphics, view.read.graphics.clone());
        if all.is_empty() { saved } else { Outcome::All(vec![saved, Outcome::Reread]) }
    }

    pub fn accept(&mut self, view: &mut SettingsView, _platform_library: &str) -> ShadersOutcome {
        match self.focus_zone {
            2 if !view.read.pack.is_empty() => {
                self.confirm_remove = true;
                ShadersOutcome::Changed
            }
            3 => ShadersOutcome::Search {
                title: "Search filters".to_owned(),
                prompt: self.search_placeholder(view),
                start: self.search.clone(),
            },
            7 => {
                let stored = self.current_stored(view);
                let console = self.current_console().to_owned();
                if view.shader_of(&console) != stored {
                    view.apply_shader_choice(&console, &stored);
                }
                self.return_params = true;
                ShadersOutcome::OpenSettingsPage { console, stored, return_row: self.return_row }
            }
            0 | 1 | 4 | 5 => {
                let outcome = self.apply_current(view);
                if outcome == Outcome::Nothing { ShadersOutcome::None } else { ShadersOutcome::Saved(outcome) }
            }
            _ => ShadersOutcome::None,
        }
    }

    pub fn toggle_favourite(&mut self, view: &SettingsView, platform_library: &str) -> bool {
        let Some(stored) = self.preview_stored.clone() else { return false };
        if let Some(pos) = self.favourites.iter().position(|f| f == &stored) {
            self.favourites.remove(pos);
        } else {
            self.favourites.push(stored);
        }
        save_favourites(platform_library, &self.favourites);
        self.sync_preview_stored(view, platform_library);
        true
    }

    pub fn step_split(&mut self, delta: f64) -> bool {
        let next = (self.split + delta).clamp(0.05, 0.95);
        if (next - self.split).abs() < 1e-6 {
            return false;
        }
        self.split = next;
        true
    }

    pub fn set_split(&mut self, value: f64) -> bool {
        if !value.is_finite() {
            return false;
        }
        let next = value.clamp(0.05, 0.95);
        if (next - self.split).abs() < 1e-6 {
            return false;
        }
        self.split = next;
        true
    }

    pub fn step_slider(&mut self, view: &mut SettingsView, platform_library: &str, slot: usize, by: i32) -> Option<(String, f32, Outcome)> {
        let sliders = self.sliders(view, platform_library);
        let (lever, now) = sliders.get(slot)?;
        let next = lever.range.stepped(*now, by);
        self.commit_slider(view, lever, next)
    }

    pub fn slide_slider(&mut self, view: &mut SettingsView, platform_library: &str, slot: usize, norm: f64) -> Option<(String, f32, Outcome)> {
        if !norm.is_finite() {
            return None;
        }
        let sliders = self.sliders(view, platform_library);
        let (lever, _) = sliders.get(slot)?;
        let (low, high) = (lever.range.minimum.min(lever.range.maximum), lever.range.minimum.max(lever.range.maximum));
        let raw = low + norm.clamp(0.0, 1.0) * (high - low);
        let next = if lever.range.step > 0.0 {
            let steps = ((raw - low) / lever.range.step).round();
            (low + steps * lever.range.step).clamp(low, high)
        } else {
            raw.clamp(low, high)
        };
        self.commit_slider(view, lever, next)
    }

    fn commit_slider(&self, view: &mut SettingsView, lever: &Lever, value: f64) -> Option<(String, f32, Outcome)> {
        let stored = self.current_stored(view);
        let console = self.current_console().to_owned();
        let targets: Vec<String> = if self.scope == 0 {
            self.consoles.iter().map(|(c, _)| c.clone()).collect()
        } else {
            vec![console]
        };
        let mut all = Vec::new();
        for c in targets {
            let one = view.store_lever_for(&c, &stored, lever, value);
            if let Outcome::All(parts) = one {
                for part in parts {
                    if !matches!(part, Outcome::Saved(..)) {
                        all.push(part);
                    }
                }
            }
        }
        all.insert(0, Outcome::Saved(Model::Graphics, view.read.graphics.clone()));
        Some((lever.id.clone(), value as f32, Outcome::All(all)))
    }

    pub fn remove_pack(&mut self, view: &mut SettingsView, platform_library: &str) -> Result<(), String> {
        let Some(dir) = self.pack_dir.clone() else {
            self.confirm_remove = false;
            return Ok(());
        };
        if dir.is_dir() {
            for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())?.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    std::fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
                } else {
                    std::fs::remove_file(&path).map_err(|e| e.to_string())?;
                }
            }
        }
        view.read.pack.clear();
        self.meta_cache.clear();
        self.pack_bytes = 0;
        self.pack_date.clear();
        self.confirm_remove = false;
        self.set_source(view, platform_library, 0);
        Ok(())
    }

    fn search_placeholder(&self, view: &SettingsView) -> String {
        let cat_count = self.category_counts(view)[self.category];
        if cat_count == 0 {
            return "Search filters".to_owned();
        }
        let tag = match self.category {
            0 => "favourite",
            1 => "CRT",
            2 => "handheld",
            3 => "broadcast",
            4 => "scaling",
            5 => "colour",
            6 => "bezel",
            _ => "console",
        };
        format!("Search {} {tag} filters", crate::words::grouped(cat_count))
    }

    pub fn to_json(&mut self, view: &SettingsView, platform_library: &str, comparing: bool) -> Value {
        let emusen_total = self.source_items(view, 0).len();
        let libretro_total = self.source_items(view, 1).len();
        let counts = self.category_counts(view);
        let visible = self.visible_items(view);
        let active_console = self.current_console().to_owned();
        let chosen = view.shader_of(&active_console);
        let stored = self.current_stored(view);
        let is_none = stored == "None" || stored == crate::display::PLAIN;
        let is_pack = pack_preset(&stored).is_some();
        let (_, all_levers) = self.levers_for(view, platform_library, &stored, &active_console);
        let passes = visible
            .get(self.focused_preset)
            .and_then(|i| i.passes)
            .or_else(|| self.meta_cache.get(&stored).and_then(|m| m.passes));
        let total_settings = all_levers.iter().filter(|l| !l.heading).count();
        let sliders_vec = self.sliders(view, platform_library);
        let display_name = if visible.is_empty() { "No filter".to_owned() } else { SettingsView::shader_name(&stored) };
        let source_words = if is_pack { "From libretro's pack" } else { "Built into EmuSen" };
        let pass_words = passes.map(|p| crate::words::counted(p, "pass", "passes")).unwrap_or_default();
        let settings_words = crate::words::counted(total_settings, "setting", "settings");
        let subtitle = if pass_words.is_empty() {
            format!("{source_words} · {settings_words}")
        } else {
            format!("{source_words} · {pass_words} · {settings_words}")
        };
        let pack_installed = !view.read.pack.is_empty();
        let pack_status = if pack_installed {
            let filters_words = crate::words::counted(libretro_total, "filter", "filters");
            let size_words = format_bytes(self.pack_bytes);
            if self.pack_date.is_empty() {
                format!("{size_words} on disk · {filters_words}.")
            } else {
                format!("Yours is from {}, {size_words} on disk · {filters_words}.", self.pack_date)
            }
        } else {
            "No shader pack installed in Shaders/.".to_owned()
        };
        let sliders_json: Vec<Value> = sliders_vec
            .iter()
            .enumerate()
            .map(|(i, (lever, val))| {
                let (low, high) = (lever.range.minimum.min(lever.range.maximum), lever.range.minimum.max(lever.range.maximum));
                let ratio = if high > low { ((val - low) / (high - low)).clamp(0.0, 1.0) } else { 0.0 };
                let shown = if is_pack { format_param_value(lever, *val) } else { format_crt_value(lever, *val) };
                json!({
                    "slot": i,
                    "id": lever.id,
                    "label": lever.label,
                    "value": shown,
                    "ratio": ratio,
                    "focused": self.focus_zone == 6 && self.focused_slider == i,
                    "boxName": format!("shaderSlider.{i}"),
                    "ringName": format!("shaderSliderRing.{i}"),
                    "labelName": format!("shaderSliderName.{i}"),
                    "valName": format!("shaderSliderVal.{i}"),
                    "knobName": format!("shaderSliderKnob.{i}"),
                })
            })
            .collect();
        let categories_json: Vec<Value> = SHADER_CATEGORIES
            .iter()
            .enumerate()
            .map(|(i, (key, label))| {
                json!({
                    "index": i,
                    "key": key,
                    "name": label,
                    "count": crate::words::grouped(counts[i]),
                    "active": self.category == i,
                    "focused": self.focus_zone == 1 && self.category == i,
                    "rowName": format!("shaderCat.{i}"),
                    "ringName": format!("shaderCatRing.{i}"),
                    "nameName": format!("shaderCatName.{i}"),
                    "countName": format!("shaderCatCount.{i}"),
                })
            })
            .collect();
        let presets_json: Vec<Value> = visible
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let meta = preset_row_meta(item, self.meta_cache.get(&item.stored));
                json!({
                    "index": i,
                    "stored": item.stored,
                    "name": item.name,
                    "meta": meta,
                    "favourite": self.is_favourite(&item.stored),
                    "selected": item.stored == chosen,
                    "scanlines": item.stored != "None" && item.stored != crate::display::PLAIN,
                    "focused": self.focus_zone == 4 && self.focused_preset == i,
                    "rowName": format!("shaderRow.{i}"),
                    "ringName": format!("shaderRowRing.{i}"),
                    "nameName": format!("shaderRowName.{i}"),
                    "metaName": format!("shaderRowMeta.{i}"),
                    "starName": format!("shaderRowStar.{i}"),
                })
            })
            .collect();
        let footer_help = vec![
            json!({ "button": emusen_kit_core::input::UiButton::Accept.index() as i32, "label": "Use this", "pair": -1 }),
            json!({ "button": emusen_kit_core::input::UiButton::Screensaver.index() as i32, "label": "Hold to see it off", "pair": -1 }),
            json!({ "button": emusen_kit_core::input::UiButton::Search.index() as i32, "label": "Favourite", "pair": -1 }),
            json!({ "button": emusen_kit_core::input::UiButton::PageUp.index() as i32, "label": "Move the line", "pair": emusen_kit_core::input::UiButton::PageDown.index() as i32 }),
            json!({ "button": emusen_kit_core::input::UiButton::Back.index() as i32, "label": "Back", "pair": -1 }),
        ];
        json!({
            "focusZone": self.focus_zone,
            "labels": {
                "settings": "Settings",
                "slash": "/",
                "picture": "Picture",
                "filter": "Screen filter",
                "packTitle": "libretro's shader pack",
                "removePack": "Remove the pack",
                "noFilter": "No filter",
                "confirmTitle": "Remove libretro's shader pack?",
                "confirmMessage": "Removes the pack files in your Shaders folder. Built-in EmuSen filters stay.",
                "confirmCancel": "Cancel",
                "confirmOk": "Remove",
                "confirmChoices": ["Cancel", "Remove"],
            },
            "focusedPreset": self.focused_preset,
            "sources": [
                {
                    "index": 0,
                    "text": format!("EmuSen's · {}", crate::words::grouped(emusen_total)),
                    "active": self.source == 0,
                    "focused": self.focus_zone == 0 && self.source == 0,
                    "pillName": "sourceTab.0",
                    "ringName": "sourceRing.0",
                    "textName": "sourceWords.0",
                },
                {
                    "index": 1,
                    "text": format!("libretro · {}", crate::words::grouped(libretro_total)),
                    "active": self.source == 1,
                    "focused": self.focus_zone == 0 && self.source == 1,
                    "pillName": "sourceTab.1",
                    "ringName": "sourceRing.1",
                    "textName": "sourceWords.1",
                },
            ],
            "categories": categories_json,
            "pack": {
                "installed": pack_installed,
                "status": pack_status,
                "removeFocused": self.focus_zone == 2,
            },
            "search": {
                "text": self.search,
                "placeholder": self.search_placeholder(view),
                "hasText": !self.search.is_empty(),
                "focused": self.focus_zone == 3,
            },
            "presets": presets_json,
            "emptyListText": if !visible.is_empty() { "" } else if self.category == 0 { "No favourite shaders yet. Press Y on any filter." } else if !self.search.is_empty() { "No matching filters." } else { "No filters in this category." },
            "scopes": [
                {
                    "index": 0,
                    "text": "Every console",
                    "active": self.scope == 0,
                    "focused": self.focus_zone == 5 && self.scope == 0,
                    "pillName": "scopeTab.0",
                    "ringName": "scopeRing.0",
                    "textName": "scopeWords.0",
                },
                {
                    "index": 1,
                    "text": self.active_console_name(),
                    "active": self.scope == 1,
                    "focused": self.focus_zone == 5 && self.scope == 1,
                    "pillName": "scopeTab.1",
                    "ringName": "scopeRing.1",
                    "textName": "scopeWords.1",
                },
            ],
            "preview": {
                "split": self.split,
                "comparing": comparing,
                "afterLabel": if comparing || is_none || visible.is_empty() { "No filter".to_owned() } else { display_name.clone() },
                "heading": display_name,
                "subtitle": subtitle,
                "hasFiltered": !comparing && !is_none && !visible.is_empty(),
            },
            "sliders": sliders_json,
            "allSettings": {
                "visible": total_settings > 0 && !visible.is_empty(),
                "text": format!("All {} settings", crate::words::grouped(total_settings)),
                "focused": self.focus_zone == 7,
            },
            "confirmOpen": self.confirm_remove,
            "footerHelp": footer_help,
        })
    }
}

fn default_category(view: &SettingsView, source: usize, active: &str) -> usize {
    if source == 1 {
        return pack_preset(active).map_or(1, categorize_pack);
    }
    if let Some(f) = view.read.catalogue["filters"].as_array().and_then(|fs| fs.iter().find(|f| f["name"] == active || f["key"] == active)) {
        let key = text_of(&f["key"]);
        let cat = categorize_builtin(&key, f["consoles"].as_array().is_some());
        if active != "None" && key != crate::display::PLAIN {
            return cat;
        }
    }
    1
}

fn categorize_builtin(key: &str, console_limited: bool) -> usize {
    if key == "CRT" || key == "CRT (Lottes)" || key.contains("Crt") || key.contains("Scanlines") {
        1
    } else if console_limited || key.contains("LCD") {
        2
    } else {
        4
    }
}

fn categorize_pack(path: &str) -> usize {
    let p = path.to_ascii_lowercase();
    let has = |terms: &[&str]| terms.iter().any(|t| p.contains(t));
    if has(&["bezel", "border", "mega_bezel"]) {
        6
    } else if has(&["handheld", "lcd", "dotmatrix", "gameboy", "gba"]) {
        2
    } else if has(&["ntsc", "pal", "vhs", "blargg", "composite", "svideo", "s-video", "broadcast"]) {
        3
    } else if has(&["interpolation", "pixel-art-scaling", "scalefx", "xbr", "nnedi", "anti-aliasing", "sharpen", "fsr", "omniscale", "sabr", "eagle", "hqx"]) {
        4
    } else if has(&["film", "color", "colour", "misc", "deband", "dither", "cel"]) {
        5
    } else if p.starts_with("crt/") || p.contains("/crt") || p.contains("crt") {
        1
    } else if has(&["presets/", "snes/", "nes/", "psx/", "n64/", "genesis/", "megadrive/", "saturn/", "dc/", "psp/", "nds/"]) {
        7
    } else {
        5
    }
}

fn preset_row_meta(item: &ShaderItem, cached: Option<&PresetMeta>) -> String {
    if item.builtin_key.is_some() {
        return if item.settings_count > 0 {
            format!("Built into EmuSen · {}", crate::words::counted(item.settings_count, "setting", "settings"))
        } else {
            "Built into EmuSen".to_owned()
        };
    }
    if let Some(meta) = cached {
        if meta.error.is_some() {
            return "Could not read preset".to_owned();
        }
        if let Some(passes) = meta.passes {
            let tier = match passes {
                1 => "Very light",
                2..=3 => "Light",
                4..=7 => "Medium",
                _ => "Heavy",
            };
            return format!("{tier} · {}", crate::words::counted(passes, "pass", "passes"));
        }
    }
    "libretro's pack".to_owned()
}

fn stored_lever_value(graphics: &Value, console: &str, stored: &str, lever: &Lever) -> f64 {
    let held = &graphics["ShaderParameters"][console][stored][&lever.id];
    text_of(held).trim().replace(',', ".").parse::<f64>().ok().filter(|v| v.is_finite()).unwrap_or(lever.own)
}

fn order_crt_levers(mut list: Vec<Lever>) -> Vec<Lever> {
    let mut head = Vec::with_capacity(list.len());
    for id in CRT_BOARD_ORDER {
        if let Some(pos) = list.iter().position(|l| l.id == id) {
            head.push(list.remove(pos));
        }
    }
    head.extend(list);
    head
}

fn levers_from_builtin(filter: &Value, console: &str) -> Vec<Lever> {
    let defaults = &filter["defaults"];
    filter["settings"]
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
        .collect()
}

fn levers_from_json(raw: &[Value]) -> Vec<Lever> {
    raw.iter()
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
        .collect()
}

fn read_pack_passes(platform_library: &str, pack_rel: &str) -> Option<usize> {
    if platform_library.is_empty() {
        return None;
    }
    let p = emusen_platform_client::Platform::load(platform_library).ok()?;
    let root = p.directory(emusen_platform_client::galaxia::consts::DIR_SHADERS).ok().flatten()?;
    let preset = emusen_platform_client::serenity::preset_load(&p, &root.join(pack_rel).to_string_lossy()).ok()?;
    preset["passes"].as_array().map(Vec::len)
}

fn read_pack_meta(platform_library: &str, pack_rel: &str) -> PresetMeta {
    let Ok(p) = emusen_platform_client::Platform::load(platform_library) else {
        return PresetMeta::default();
    };
    let Ok(Some(root)) = p.directory(emusen_platform_client::galaxia::consts::DIR_SHADERS) else {
        return PresetMeta::default();
    };
    let Ok(preset) = emusen_platform_client::serenity::preset_load(&p, &root.join(pack_rel).to_string_lossy()) else {
        return PresetMeta { error: Some("Could not read preset".to_owned()), ..Default::default() };
    };
    if let Some(msg) = preset.get("message").and_then(Value::as_str).filter(|_| preset.get("fail").is_some()) {
        return PresetMeta { error: Some(msg.to_owned()), ..Default::default() };
    }
    let passes = preset["passes"].as_array().map(Vec::len);
    let mut sources = Vec::new();
    for pass in preset["passes"].as_array().into_iter().flatten() {
        if let Ok(source) = emusen_platform_client::serenity::source_load(&p, &text_of(&pass["shader"])) {
            if source.get("fail").is_none() {
                sources.push(source["parameters"].clone());
            }
        }
    }
    let merged = emusen_platform_client::serenity::parameters_merge(&p, &json!({ "sources": sources, "overrides": preset["parameters"] })).unwrap_or(Value::Null);
    let levers = merged.as_array().map(|a| levers_from_json(a)).unwrap_or_default();
    PresetMeta { passes, levers, error: None }
}

fn favourites_file(platform_library: &str) -> Option<std::path::PathBuf> {
    if platform_library.is_empty() {
        return None;
    }
    let p = emusen_platform_client::Platform::load(platform_library).ok()?;
    let graphics = p.file(emusen_platform_client::galaxia::consts::FILE_GRAPHICS).ok()?;
    Some(graphics.parent()?.join("shader_favourites.json"))
}

fn load_favourites(platform_library: &str) -> Vec<String> {
    let Some(path) = favourites_file(platform_library) else { return Vec::new() };
    let Ok(text) = std::fs::read_to_string(path) else { return Vec::new() };
    serde_json::from_str::<Value>(&text)
        .ok()
        .and_then(|v| v["favourites"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect()))
        .unwrap_or_default()
}

fn save_favourites(platform_library: &str, favourites: &[String]) {
    let Some(path) = favourites_file(platform_library) else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let tmp = path.with_extension("json.tmp");
    let body = serde_json::to_string_pretty(&json!({ "favourites": favourites })).unwrap_or_default();
    if std::fs::write(&tmp, body).is_ok() {
        let _ = std::fs::rename(&tmp, &path);
    }
}

fn scan_pack_dir(platform_library: &str) -> (Option<std::path::PathBuf>, u64, String) {
    if platform_library.is_empty() {
        return (None, 0, String::new());
    }
    let Ok(p) = emusen_platform_client::Platform::load(platform_library) else { return (None, 0, String::new()) };
    let Ok(Some(root)) = p.directory(emusen_platform_client::galaxia::consts::DIR_SHADERS) else { return (None, 0, String::new()) };
    let mut bytes = 0u64;
    let mut newest = 0i64;
    walk_pack(&root, &mut bytes, &mut newest);
    let date = format_day_month(newest);
    (Some(root), bytes, date)
}

fn walk_pack(dir: &std::path::Path, bytes: &mut u64, newest: &mut i64) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.filter_map(Result::ok) {
        let Ok(meta) = entry.metadata() else { continue };
        if meta.is_dir() {
            walk_pack(&entry.path(), bytes, newest);
        } else if meta.is_file() {
            *bytes = bytes.saturating_add(meta.len());
            if let Ok(m) = meta.modified() {
                let secs = m.duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64);
                *newest = (*newest).max(secs);
            }
        }
    }
}

fn format_day_month(secs: i64) -> String {
    if secs <= 0 {
        return String::new();
    }
    let iso = crate::words::utc(secs);
    let month = iso.get(5..7).and_then(|s| s.parse::<usize>().ok()).and_then(|m| MONTHS.get(m.saturating_sub(1))).copied();
    let day = iso.get(8..10).and_then(|s| s.parse::<u32>().ok());
    match (day, month) {
        (Some(d), Some(m)) => format!("{d} {m}"),
        _ => String::new(),
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{} MB", (bytes + 524_288) / 1_048_576)
    } else if bytes >= 1024 {
        format!("{} KB", (bytes + 512) / 1024)
    } else {
        format!("{bytes} B")
    }
}

