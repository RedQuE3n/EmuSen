//! The Settings screen as plain data (EmuSen_Qt.md §7.7, EmuSen_Interface.md §2.7): the six sections and their pages,
//! each row from the screen's description (`data/settings_screen.json`) and, for a key's row, Galaxia's settings rows
//! (EmuSen_Galaxia.md §3.15), the one path a core's `settings_schema` rows take too. No row is written in code: the
//! description says where a row stands and what a door opens, the rows say a key's words and values, and the models
//! hold what is chosen. Tested without Qt over a fake source.

use crate::words;
use emusen_kit_core::settings::RowKind;
use emusen_kit_core::tokens::text_scale_of;
use serde_json::{json, Value};

/// The screen's description: sections, their rows in order, the doors' words and the panes.
pub const DESCRIPTION: &str = include_str!("../data/settings_screen.json");

/// A settings model a row's key belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Model {
    /// `AppSettings`, the settings file.
    App,
    /// `GraphicsConfig`, its file by Galaxia's number, with each console's keys in its `Consoles` map.
    Graphics,
}

/// What the screen reads and writes: Galaxia's rows and models, Serenity's catalogue of screens and filters, and the
/// machine's sound devices.
pub trait SettingsSource {
    fn rows(&self) -> Result<Value, String>;
    fn load(&self, model: Model) -> Result<Value, String>;
    fn save(&self, model: Model, document: &Value) -> Result<(), String>;
    fn catalogue(&self) -> Result<Value, String>;
    fn outputs(&self) -> Vec<String>;
}

/// One read of everything the screen shows.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Read {
    pub rows: Value,
    pub app: Value,
    pub graphics: Value,
    pub catalogue: Value,
    pub outputs: Vec<String>,
}

/// Reads the source once; a model with no file yet reads as null, and its keys show their defaults.
pub fn read(source: &dyn SettingsSource) -> Result<Read, String> {
    Ok(Read {
        rows: source.rows()?,
        app: source.load(Model::App)?,
        graphics: source.load(Model::Graphics)?,
        catalogue: source.catalogue()?,
        outputs: source.outputs(),
    })
}

/// A row as the frame draws it.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub id: String,
    pub label: String,
    pub value: String,
    pub kind: RowKind,
    pub held: bool,
}

/// What the pane holds over its words.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Preview {
    #[default]
    None,
    /// The picture of the game, named by its caption (drawn from piece J on).
    Picture,
    /// A meter of the row's share.
    Meter,
    /// The two variants side by side.
    Variants,
    /// A line at each text size.
    Sizes,
}

/// The focused row's pane: the sentence, plain lines after it, muted lines after those, and the preview.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pane {
    pub sentence: String,
    pub lines: Vec<String>,
    pub muted: Vec<String>,
    pub preview: Preview,
    pub caption: String,
    pub meter: f64,
    /// A line and its scale for each text size, or the two variants' names.
    pub samples: Vec<(String, f64)>,
}

/// What a press asked for, beyond the rows read again.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    Nothing,
    /// A value changed: the model's whole document, to be saved.
    Saved(Model, Value),
    /// A page opened under the section.
    Opened,
    /// A door to another screen, by its name in the description.
    Door(String),
}

/// The screen: the description, the last read, the section and the pages open under it.
#[derive(Clone, Debug)]
pub struct SettingsView {
    description: Value,
    pub read: Read,
    pub section: usize,
    /// The pages open, outermost first: a group of Galaxia's rows and the label of the row that opened it.
    pages: Vec<(String, String)>,
    /// The program's facts a pane names: `games`, the games in the library; `running`, the game's title.
    pub facts: Value,
}

impl Default for SettingsView {
    fn default() -> Self {
        SettingsView::new()
    }
}

/// A stored value as text: a string as it is, a number by its value, a switch as `true` or `false`.
fn text_of(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.as_f64().map(|f| if f.fract() == 0.0 { format!("{f:.0}") } else { f.to_string() }).unwrap_or_default(),
        _ => String::new(),
    }
}

/// Whether a stored text is a choice's value: numbers by value, so 1.0 is "1"; anything else exactly.
fn same(stored: &str, choice: &str) -> bool {
    match (stored.parse::<f64>(), choice.parse::<f64>()) {
        (Ok(a), Ok(b)) => (a - b).abs() < 1e-9,
        _ => stored == choice,
    }
}

/// A member of a model by its key, a nested model's as `Model.Field`.
fn at<'a>(document: &'a Value, key: &str) -> Option<&'a Value> {
    key.split('.').try_fold(document, |v, part| v.get(part))
}

fn set_at(document: &mut Value, key: &str, value: Value) {
    let mut here = document;
    let parts: Vec<&str> = key.split('.').collect();
    for part in &parts[..parts.len() - 1] {
        if !here.get(*part).is_some_and(Value::is_object) {
            here[*part] = json!({});
        }
        here = &mut here[*part];
    }
    if !here.is_object() {
        *here = json!({});
    }
    here[parts[parts.len() - 1]] = value;
}

/// A text written back in the stored value's own type, so a share stays a number and a switch a boolean.
fn typed(old: Option<&Value>, kind: &str, text: &str) -> Value {
    match old {
        _ if kind == "switch" => Value::Bool(text == "true"),
        Some(Value::Number(n)) if n.is_f64() => text.parse::<f64>().map_or(json!(text), |f| json!(f)),
        Some(Value::Number(_)) => text.parse::<i64>().map(|i| json!(i)).or_else(|_| text.parse::<f64>().map(|f| json!(f))).unwrap_or(json!(text)),
        Some(Value::Bool(_)) => Value::Bool(text == "true"),
        _ => json!(text),
    }
}

fn strings(value: &Value) -> Vec<String> {
    value.as_array().map(|a| a.iter().map(text_of).collect()).unwrap_or_default()
}

/// A step through `n` values from `at` by `by`, round from either end.
fn wrapped(at: usize, by: i32, n: usize) -> usize {
    (at as i64 + i64::from(by)).rem_euclid(n as i64) as usize
}

/// A key's row resolved: its model, its row of Galaxia's, and the consoles it is set for (none: the model's own key).
struct Keyed<'a> {
    model: &'a str,
    row: &'a Value,
    consoles: Vec<String>,
}

impl SettingsView {
    pub fn new() -> SettingsView {
        let description = serde_json::from_str(DESCRIPTION).expect("the settings screen's description is JSON");
        SettingsView { description, read: Read::default(), section: 0, pages: Vec::new(), facts: Value::Null }
    }

    fn word(&self, name: &str) -> String {
        self.description["words"][name].as_str().unwrap_or_default().to_owned()
    }

    pub fn sections(&self) -> Vec<String> {
        self.description["sections"].as_array().map(|s| s.iter().map(|s| text_of(&s["name"])).collect()).unwrap_or_default()
    }

    /// The pages' trail: empty on a section, else the section's name and each page's.
    pub fn trail(&self) -> Vec<String> {
        if self.pages.is_empty() {
            return Vec::new();
        }
        let mut trail = vec![self.sections().get(self.section).cloned().unwrap_or_default()];
        trail.extend(self.pages.iter().map(|(_, label)| label.clone()));
        trail
    }

    pub fn page_open(&self) -> bool {
        !self.pages.is_empty()
    }

    /// The section chosen, its pages closed.
    pub fn choose_section(&mut self, section: usize) {
        self.section = section.min(self.sections().len().saturating_sub(1));
        self.pages.clear();
    }

    /// B: the page closed; false on a section, where B is the screens'.
    pub fn back(&mut self) -> bool {
        self.pages.pop().is_some()
    }

    /// The rows shown: the section's from the description, or a page's, every row of Galaxia's in its group.
    fn entries(&self) -> Vec<Value> {
        if let Some((group, _)) = self.pages.last() {
            let mut rows = Vec::new();
            if let Some(models) = self.read.rows.as_object() {
                for (model, list) in models {
                    for row in list.as_array().into_iter().flatten().filter(|r| r["group"] == group.as_str()) {
                        rows.push(json!({ "id": text_of(&row["key"]).to_lowercase(), "row": format!("{model} {}", text_of(&row["key"])) }));
                    }
                }
            }
            return rows;
        }
        self.description["sections"][self.section]["rows"].as_array().cloned().unwrap_or_default()
    }

    /// The consoles a row is set for: every console Serenity has a screen for, or those whose screen is a television.
    fn consoles(&self, which: &str) -> Vec<String> {
        let screens = self.read.catalogue["screens"].as_array().cloned().unwrap_or_default();
        let mut out: Vec<String> = Vec::new();
        for s in screens.iter().filter(|s| which == "every" || s["kind"] == "Television") {
            let console = text_of(&s["console"]);
            if !out.contains(&console) {
                out.push(console);
            }
        }
        out
    }

    fn keyed(&self, entry: &Value) -> Option<Keyed<'_>> {
        let reference = entry["row"].as_str()?;
        let (model, key) = reference.split_once(' ')?;
        let (model_name, list) = self.read.rows.as_object()?.iter().find(|(m, _)| m.as_str() == model)?;
        let row = list.as_array()?.iter().find(|r| r["key"] == key)?;
        let consoles = if model == "GraphicsConfig.Consoles" { self.consoles(entry["consoles"].as_str().unwrap_or("every")) } else { Vec::new() };
        Some(Keyed { model: model_name.as_str(), row, consoles })
    }

    fn document(&self, model: &str) -> &Value {
        if model.starts_with("GraphicsConfig") {
            &self.read.graphics
        } else {
            &self.read.app
        }
    }

    /// The stored texts of a key's row: one, or one per console it is set for, a blank or a console without one at the
    /// default.
    fn stored(&self, keyed: &Keyed<'_>) -> Vec<String> {
        let key = text_of(&keyed.row["key"]);
        let default = text_of(&keyed.row["default"]);
        if keyed.consoles.is_empty() {
            return vec![at(self.document(keyed.model), &key).map(text_of).filter(|t| !t.is_empty()).unwrap_or(default)];
        }
        let held = |c: &String| at(&self.read.graphics, &format!("Consoles.{c}.{key}")).map(text_of).filter(|t| !t.is_empty());
        keyed.consoles.iter().map(|c| held(c).unwrap_or_else(|| default.clone())).collect()
    }

    /// A key's values and their words: a switch's two, a choice's, the machine's devices where the row asks for them,
    /// and a stored value none of them is, kept as itself.
    fn choices(&self, entry: &Value, keyed: &Keyed<'_>) -> Vec<(String, String)> {
        let mut choices: Vec<(String, String)> = match keyed.row["kind"].as_str() {
            Some("switch") => vec![("false".into(), self.word("off")), ("true".into(), self.word("on"))],
            _ => strings(&keyed.row["choices"]).into_iter().zip(strings(&keyed.row["choice_labels"])).collect(),
        };
        if entry["more"] == "outputs" {
            for device in &self.read.outputs {
                if !choices.iter().any(|(v, _)| v == device) {
                    choices.push((device.clone(), device.clone()));
                }
            }
        }
        for stored in self.stored(keyed) {
            if keyed.row["kind"] != "count" && !choices.iter().any(|(v, _)| same(&stored, v)) {
                choices.push((stored.clone(), stored));
            }
        }
        choices
    }

    /// The CRT row's choices: off, then each of the filter's setting's named values.
    fn filter_choices(&self, entry: &Value) -> Vec<String> {
        let filter = self.filter(entry);
        let setting = filter.and_then(|f| f["settings"].as_array()?.iter().find(|s| s["id"] == entry["setting"]));
        let mut out = vec![self.word("off")];
        out.extend(setting.map(|s| strings(&s["choices"])).unwrap_or_default());
        out
    }

    fn filter(&self, entry: &Value) -> Option<&Value> {
        self.filter_keyed(&entry["filter"])
    }

    fn filter_keyed(&self, key: &Value) -> Option<&Value> {
        self.read.catalogue["filters"].as_array()?.iter().find(|f| f["key"] == *key)
    }

    /// The word a console's `ScreenFilter` holds for a filter of the catalogue: the filter's name there.
    fn filter_name(&self, key: &Value) -> String {
        self.filter_keyed(key).map_or_else(|| text_of(key), |f| text_of(&f["name"]))
    }

    /// Each console's choice on the CRT row: 0 off, else 1 and the setting's value there or its initial value.
    fn filter_at(&self, entry: &Value) -> Vec<usize> {
        let name = self.filter_name(&entry["filter"]);
        let setting = text_of(&entry["setting"]);
        let initial = self
            .filter(entry)
            .and_then(|f| f["settings"].as_array()?.iter().find(|s| s["id"] == entry["setting"]))
            .and_then(|s| s["initial"].as_u64())
            .map_or(0.0, f64::from_bits);
        let consoles = self.consoles(entry["consoles"].as_str().unwrap_or("every"));
        consoles
            .iter()
            .map(|c| {
                let chosen = at(&self.read.graphics, &format!("Consoles.{c}.ScreenFilter")).map(text_of).unwrap_or_default();
                if chosen != name {
                    return 0;
                }
                let held = self.read.graphics.get("ShaderParameters").and_then(|p| p.get(c)).and_then(|p| p.get(&name)).and_then(|p| p.get(&setting));
                1 + held.map(text_of).and_then(|t| t.parse::<f64>().ok()).unwrap_or(initial).max(0.0) as usize
            })
            .collect()
    }

    /// One index for a row set across consoles, where they agree.
    fn agreed(at: &[usize]) -> Option<usize> {
        let first = *at.first()?;
        at.iter().all(|i| *i == first).then_some(first)
    }

    /// A row's value words and, for a row of values, the chosen index among its choices.
    fn value(&self, entry: &Value) -> (String, Option<usize>, usize) {
        if let Some(keyed) = self.keyed(entry) {
            if keyed.row["kind"] == "count" {
                return (self.stored(&keyed).first().cloned().unwrap_or_default(), None, 0);
            }
            let choices = self.choices(entry, &keyed);
            let held: Vec<usize> = self.stored(&keyed).iter().filter_map(|s| choices.iter().position(|(v, _)| same(s, v))).collect();
            return match Self::agreed(&held) {
                Some(i) => (choices[i].1.clone(), Some(i), choices.len()),
                None if keyed.consoles.is_empty() => (String::new(), None, choices.len()),
                None => (self.word("every"), None, choices.len()),
            };
        }
        if entry.get("filter").is_some() {
            let choices = self.filter_choices(entry);
            return match Self::agreed(&self.filter_at(entry)) {
                Some(i) => (choices.get(i).cloned().unwrap_or_default(), Some(i), choices.len()),
                None => (self.word("every"), None, choices.len()),
            };
        }
        let spec = &entry["value"];
        if let Some(fact) = spec["fact"].as_str() {
            return (self.fact(fact), None, 0);
        }
        let Some(from) = spec["from"].as_str() else { return (String::new(), None, 0) };
        let (model, key) = from.split_once(' ').unwrap_or((from, ""));
        let text = at(self.document(model), key).map(text_of).unwrap_or_default();
        let words = spec["words"].get(&text).and_then(Value::as_str);
        let shown = match (words, text.is_empty()) {
            (Some(w), _) => w.to_owned(),
            (None, true) => text_of(&spec["blank"]),
            (None, false) => text,
        };
        (shown, None, 0)
    }

    fn fact(&self, name: &str) -> String {
        match name {
            "version" => words::version_line(),
            "running" => match self.facts["running"].as_str() {
                Some(t) if !t.is_empty() => t.to_owned(),
                _ => self.word("last_game"),
            },
            "games" => words::counted(self.facts["games"].as_u64().unwrap_or(0) as usize, &self.word("game"), &self.word("games")),
            _ => String::new(),
        }
    }

    /// The rows shown.
    pub fn rows(&self) -> Vec<Row> {
        self.entries().iter().map(|e| self.row(e)).collect()
    }

    fn row(&self, entry: &Value) -> Row {
        let keyed = self.keyed(entry);
        let label = keyed.as_ref().map_or_else(|| text_of(&entry["label"]), |k| text_of(&k.row["label"]));
        let kind = if keyed.is_some() || entry.get("filter").is_some() {
            RowKind::Steps
        } else if entry.get("page").is_some() || entry.get("door").is_some() {
            RowKind::Opens
        } else {
            RowKind::Action
        };
        let held = keyed.as_ref().is_some_and(|k| k.row["locked"] == true);
        Row { id: text_of(&entry["id"]), label, value: self.value(entry).0, kind, held }
    }

    /// The pane of a row shown, by its index.
    pub fn pane(&self, index: usize) -> Pane {
        let Some(entry) = self.entries().get(index).cloned() else { return Pane::default() };
        let keyed = self.keyed(&entry);
        let (value, chosen, count) = self.value(&entry);
        let pane = &entry["pane"];
        let fill = |line: &str| self.fill(line, &value);
        let hint = keyed.as_ref().map_or_else(|| text_of(&entry["hint"]), |k| text_of(&k.row["hint"]));
        let by_value = pane["by_value"].get(&value).or_else(|| pane["by_value"].get("*"));
        let sentence = by_value.or(pane.get("sentence")).map_or(hint, text_of);
        let preview = match pane["preview"].as_str() {
            Some("picture") => Preview::Picture,
            Some("meter") => Preview::Meter,
            Some("variants") => Preview::Variants,
            Some("sizes") => Preview::Sizes,
            _ => Preview::None,
        };
        let meter = match (chosen, count) {
            (Some(i), n) if n > 1 => i as f64 / (n - 1) as f64,
            _ => 0.0,
        };
        let samples = match (&keyed, preview) {
            // Each size's scale is the kit's, which applies the text size to every screen.
            (Some(k), Preview::Sizes) => self.choices(&entry, k).into_iter().map(|(v, w)| (format!("{w} · {}", self.word("sample")), text_scale_of(&v))).collect(),
            (Some(k), Preview::Variants) => self.choices(&entry, k).into_iter().take(2).map(|(_, w)| (w, 1.0)).collect(),
            _ => Vec::new(),
        };
        Pane {
            sentence: fill(&sentence),
            lines: strings(&pane["lines"]).iter().map(|l| fill(l)).collect(),
            muted: strings(&pane["muted"]).iter().map(|l| fill(l)).collect(),
            preview,
            caption: pane["caption"].as_str().map(fill).unwrap_or_default(),
            meter,
            samples,
        }
    }

    /// A line's names filled: `{value}` the row's words, a fact by its name, and `{Model Key}` a key's words.
    fn fill(&self, line: &str, value: &str) -> String {
        let mut out = String::new();
        let mut rest = line;
        while let Some(open) = rest.find('{') {
            out.push_str(&rest[..open]);
            let Some(close) = rest[open..].find('}') else { break };
            let name = &rest[open + 1..open + close];
            let said = match name {
                "value" => value.to_owned(),
                n if n.contains(' ') => self.value(&json!({ "row": n, "value": { "from": n } })).0,
                n => self.fact(n),
            };
            out.push_str(&said);
            rest = &rest[open + close + 1..];
        }
        out.push_str(rest);
        out
    }

    /// Left or Right on a row of values: the next value, saved; a row set across consoles takes it on each.
    pub fn step(&mut self, index: usize, by: i32) -> Outcome {
        let Some(entry) = self.entries().get(index).cloned() else { return Outcome::Nothing };
        let (_, chosen, count) = self.value(&entry);
        if let Some(keyed) = self.keyed(&entry) {
            if keyed.row["locked"] == true {
                return Outcome::Nothing;
            }
            let key = text_of(&keyed.row["key"]);
            let kind = text_of(&keyed.row["kind"]);
            let model = keyed.model.to_owned();
            let consoles = keyed.consoles.clone();
            let text = if kind == "count" {
                let now = self.stored(&keyed).first().and_then(|t| t.parse::<f64>().ok()).unwrap_or(0.0);
                let (low, high) = (keyed.row["min"].as_f64().unwrap_or(f64::MIN), keyed.row["max"].as_f64().unwrap_or(f64::MAX));
                format!("{}", (now + f64::from(by)).clamp(low, high))
            } else {
                let choices = self.choices(&entry, &keyed);
                if choices.is_empty() {
                    return Outcome::Nothing;
                }
                let next = chosen.map_or(if by > 0 { 0 } else { count - 1 }, |i| wrapped(i, by, count));
                choices[next].0.clone()
            };
            return if consoles.is_empty() {
                let document = if model.starts_with("GraphicsConfig") { &mut self.read.graphics } else { &mut self.read.app };
                let old = at(document, &key).cloned();
                set_at(document, &key, typed(old.as_ref(), &kind, &text));
                let saved = if model.starts_with("GraphicsConfig") { Model::Graphics } else { Model::App };
                Outcome::Saved(saved, document.clone())
            } else {
                for c in consoles {
                    set_at(&mut self.read.graphics, &format!("Consoles.{c}.{key}"), json!(text));
                }
                Outcome::Saved(Model::Graphics, self.read.graphics.clone())
            };
        }
        if entry.get("filter").is_some() && count > 0 {
            let next = chosen.map_or(if by > 0 { 0 } else { count - 1 }, |i| wrapped(i, by, count));
            let name = self.filter_name(&entry["filter"]);
            let off = self.filter_name(&entry["off"]);
            let setting = text_of(&entry["setting"]);
            for c in self.consoles(entry["consoles"].as_str().unwrap_or("every")) {
                if next == 0 {
                    set_at(&mut self.read.graphics, &format!("Consoles.{c}.ScreenFilter"), json!(off));
                } else {
                    set_at(&mut self.read.graphics, &format!("Consoles.{c}.ScreenFilter"), json!(name));
                    let parameters = &mut self.read.graphics["ShaderParameters"];
                    if !parameters.is_object() {
                        *parameters = json!({});
                    }
                    parameters[&c][&name][&setting] = json!((next - 1).to_string());
                }
            }
            return Outcome::Saved(Model::Graphics, self.read.graphics.clone());
        }
        Outcome::Nothing
    }

    /// A on a row: a value steps on, a page opens, a door is the screens' to open.
    pub fn choose(&mut self, index: usize) -> Outcome {
        let Some(entry) = self.entries().get(index).cloned() else { return Outcome::Nothing };
        if self.keyed(&entry).is_some() || entry.get("filter").is_some() {
            return self.step(index, 1);
        }
        if let Some(group) = entry["page"].as_str() {
            self.pages.push((group.to_owned(), text_of(&entry["label"])));
            return Outcome::Opened;
        }
        entry["door"].as_str().map_or(Outcome::Nothing, |d| Outcome::Door(d.to_owned()))
    }

    /// A key's stored text, by its `Model Key`, as the description's `applies` names it.
    fn applied(&self, name: &str) -> Option<String> {
        let (model, key) = self.description["applies"][name].as_str().or_else(|| self.description["applies"][name][0].as_str())?.split_once(' ')?;
        Some(at(self.document(model), key).map(text_of).unwrap_or_default())
    }

    /// The text size chosen, by the word the setting stores; the kit gives its scale (`text_scale_of`).
    pub fn text_size(&self) -> String {
        self.applied("text_size").unwrap_or_default()
    }

    fn applies_as(&self, name: &str) -> bool {
        let wanted = text_of(&self.description["applies"][name][1]);
        self.applied(name).is_some_and(|v| v == wanted)
    }

    /// Whether the light variant is chosen; System reads as dark until the desktop's choice has a reader.
    pub fn light(&self) -> bool {
        self.applies_as("light")
    }

    /// Whether reduced motion is chosen; System reads as full motion, as the program's start does.
    pub fn reduced_motion(&self) -> bool {
        self.applies_as("reduced_motion")
    }
}
