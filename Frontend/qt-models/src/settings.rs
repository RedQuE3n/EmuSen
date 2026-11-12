//! The Settings screen as plain data (EmuSen_Qt.md §7.7, EmuSen_Interface.md §2.7): the six sections and their pages,
//! each row from the screen's description (`data/settings_screen.json`) and, for a key's row, Galaxia's settings rows
//! (EmuSen_Galaxia.md §3.15), the one path a core's `settings_schema` rows take too. No row is written in code: the
//! description says where a row stands and what a door opens, the rows say a key's words and values, and the models
//! hold what is chosen. Tested without Qt over a fake source.

use crate::consoles::{self, Console, ConsoleRead};
use crate::tools::Tone;
use crate::{cards, firmware, schema, words};
use emusen_kit_core::glyph::PadFamily;
use emusen_kit_core::settings::RowKind;
use emusen_qt_endymion::game::PadSettings;
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
    /// EmuSen's own folders a row may open, by Galaxia's names for them (`firmware`).
    fn folders(&self) -> std::collections::BTreeMap<String, String>;
}

/// One read of everything the screen shows.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Read {
    pub rows: Value,
    pub app: Value,
    pub graphics: Value,
    pub catalogue: Value,
    pub outputs: Vec<String>,
    pub folders: std::collections::BTreeMap<String, String>,
    /// The consoles and what their engines declare, read through the runtime (EmuSen_Qt.md §7.7.E).
    pub consoles: ConsoleRead,
    /// Endymion's four binding models (EmuSen_Qt.md §7.7.H).
    pub bindings: crate::bindings::Bindings,
}

/// Reads the source once; a model with no file yet reads as null, and its keys show their defaults.
pub fn read(source: &dyn SettingsSource) -> Result<Read, String> {
    Ok(Read {
        rows: source.rows()?,
        app: source.load(Model::App)?,
        graphics: source.load(Model::Graphics)?,
        catalogue: source.catalogue()?,
        outputs: source.outputs(),
        folders: source.folders(),
        consoles: ConsoleRead::default(),
        bindings: crate::bindings::Bindings::default(),
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
    /// The choice in force on a page of choices.
    pub marked: bool,
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
    /// The focused control's name, large, as the bindings page's pane begins (its words are the caption).
    Heading,
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
    /// The sentence's tone, then each plain line's, by what it says.
    pub tones: Vec<Tone>,
}

/// What a press asked for, beyond the rows read again.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    Nothing,
    /// A value changed: the model's whole document, to be saved.
    Saved(Model, Value),
    /// A page opened under the section.
    Opened,
    /// The rows' values changed in place, as a card chosen on the card manager's page: read again, the focus kept.
    Shown,
    /// A door to another screen, by its name in the description.
    Door(String),
    /// A console's setting stored through the runtime, or forgotten with none; the rows already show it.
    Set { console: String, key: String, value: Option<String> },
    /// A new card made at a path through the card manager; the consoles are read again after.
    NewCard(String),
    /// The consoles read again, as after a file is put in a firmware folder.
    Reread,
    /// A folder shown in the system's file manager.
    OpenFolder(String),
    /// One of the card manager's changes (`CARDS_COPY`, `_MOVE`, `_DELETE`, `_IMPORT`) and its arguments; its words are
    /// told back through `cards_said`, and the consoles read again.
    Cards { op: u32, args: Value },
    /// A folder to choose in the system's folder dialog, for a key (`Model Key`), starting at the folder it holds.
    ChooseFolder { key: String, start: String },
    /// Several at once, in order: a console reset forgets its keys and its picture.
    All(Vec<Outcome>),
    /// One of Endymion's binding files changed, to be saved and read again (EmuSen_Qt.md §7.7.H).
    Bindings(Vec<crate::bindings::File>),
    /// An ask of the seats, for the pads' reader.
    Seat(emusen_qt_endymion::seats::Ask),
    /// Words to type for a page: the question's title, its prompt and the words it starts with.
    Type { title: String, prompt: String, start: String, typed: crate::settings_controllers::Typed },
    /// Nothing done, and the words of what failed, said as the program's other failures are (EmuSen_Qt.md §7.9).
    Failed(String),
}

/// A page open under a section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Page {
    /// A group of Galaxia's rows.
    Group(String),
    /// A console's page, by the console's stored name.
    Console(String),
    /// A console's engines as a page of choices (EmuSen_Interface.md §18.11).
    Engine(String),
    Firmware(String),
    Cards(String),
    /// A medium's drawer of cards, by the console and the medium.
    Drawer(String, String),
    /// The card manager's page of a medium: a card chosen and the saves on it (Mistress's Memory Cards window).
    Saves(String, String),
    /// A save on the chosen card, by its id: copy, move or delete it.
    Save(String, String, String),
    /// The question before a save is deleted, by the save's id.
    Delete(String, String, String),
    /// Controllers ▸ Players, Profiles, a console's profile, Hotkeys, and the bindings page (EmuSen_Qt.md §7.7.H).
    Players,
    Profiles,
    Profile(String, String),
    Hotkeys,
    Bindings,
}

impl Page {
    fn console(&self) -> Option<&str> {
        match self {
            Page::Group(_) | Page::Players | Page::Profiles | Page::Profile(..) | Page::Hotkeys | Page::Bindings => None,
            Page::Console(c) | Page::Engine(c) | Page::Firmware(c) | Page::Cards(c) | Page::Drawer(c, _) => Some(c),
            Page::Saves(c, _) | Page::Save(c, _, _) | Page::Delete(c, _, _) => Some(c),
        }
    }
}

/// The screen: the description, the last read, the section and the pages open under it.
#[derive(Clone)]
pub struct SettingsView {
    description: Value,
    pub read: Read,
    pub section: usize,
    /// The pages open, outermost first, each with the label of the row that opened it and that row's index.
    pub(crate) pages: Vec<(Page, String, usize)>,
    /// The program's facts a pane names: `games`, the games in the library; `running`, the game's title.
    pub facts: Value,
    /// The card manager's page: the card chosen and the card a save goes to, by path, and the last change's words.
    pub(crate) chosen_card: Option<String>,
    pub(crate) target_card: Option<String>,
    pub(crate) said: String,
    /// The seats and the pads connected, as the pads' reader gave them (EmuSen_Qt.md §7.7.H).
    pub seats: crate::settings_controllers::Seated,
    /// Endymion's choice of a player's map, over the platform library.
    pub chooser: Option<std::sync::Arc<dyn crate::settings_controllers::Chooser>>,
    /// The controls the tester sees held, how far each is pushed (a stick's directions), and the keys held, by the
    /// binding files' words.
    pub held: Vec<String>,
    pub pushes: Vec<f64>,
    pub keys_held: Vec<String>,
    pub(crate) stepped_console: Option<String>,
    pub(crate) binding: Option<crate::settings_controllers::Binding>,
    /// The hotkey whose row listens for a key.
    pub(crate) listening: Option<String>,
    /// Opened from the Pause Card on its drawing page: B leaves Settings for the card.
    pub(crate) from_card: bool,
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
        SettingsView {
            description,
            read: Read::default(),
            section: 0,
            pages: Vec::new(),
            facts: Value::Null,
            chosen_card: None,
            target_card: None,
            said: String::new(),
            seats: Default::default(),
            chooser: None,
            held: Vec::new(),
            pushes: Vec::new(),
            keys_held: Vec::new(),
            stepped_console: None,
            binding: None,
            listening: None,
            from_card: false,
        }
    }

    /// A member of the description by its path.
    pub(crate) fn description_at(&self, path: &[&str]) -> &Value {
        path.iter().fold(&self.description, |v, k| &v[*k])
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
        trail.extend(self.pages.iter().map(|(_, label, _)| label.clone()));
        trail
    }

    /// The help bar the page shown takes.
    pub fn help(&self) -> crate::help::Screen {
        use crate::help::Screen;
        if let Some(screen) = self.controller_help() {
            return screen;
        }
        match self.page() {
            None => Screen::Settings,
            Some(Page::Console(_)) if self.pages.len() == 1 => Screen::SettingsConsole,
            Some(Page::Engine(_)) => Screen::SettingsChoices,
            Some(_) => Screen::SettingsPage,
        }
    }

    /// The page open innermost, if any.
    pub fn page(&self) -> Option<&Page> {
        self.pages.last().map(|(p, _, _)| p)
    }

    pub fn page_open(&self) -> bool {
        !self.pages.is_empty()
    }

    /// The section chosen, its pages closed.
    pub fn choose_section(&mut self, section: usize) {
        self.section = section.min(self.sections().len().saturating_sub(1));
        self.pages.clear();
        self.binding = None;
        self.listening = None;
        self.from_card = false;
    }

    /// B: the page closed, and the index of the row that opened it; none on a section, where B is the screens'.
    pub fn back(&mut self) -> Option<usize> {
        if self.from_card {
            // From the card, B leaves Settings on the drawing page, back to the card.
            self.from_card = false;
            self.pages.clear();
            self.binding = None;
            self.held.clear();
            return None;
        }
        let (page, _, at) = self.pages.pop()?;
        if page == Page::Bindings {
            self.binding = None;
            self.held.clear();
        }
        Some(at)
    }

    /// B taken by the page itself: a press listened for, or the tester, let go of; else B closes the page.
    pub fn back_taken(&mut self) -> bool {
        self.controller_back().unwrap_or(false)
    }

    pub(crate) fn open(&mut self, page: Page, label: String, at: usize) -> Outcome {
        self.pages.push((page, label, at));
        Outcome::Opened
    }

    /// The rows shown: the section's from the description, or a page's: a group's every row of Galaxia's, or a console's.
    fn entries(&self) -> Vec<Value> {
        if let Some(rows) = self.page().and_then(|p| self.controller_entries(p)) {
            return rows;
        }
        match self.page() {
            Some(Page::Group(group)) => {
                let mut rows = Vec::new();
                if let Some(models) = self.read.rows.as_object() {
                    for (model, list) in models {
                        for row in list.as_array().into_iter().flatten().filter(|r| r["group"] == group.as_str()) {
                            rows.push(json!({ "id": text_of(&row["key"]).to_lowercase(), "row": format!("{model} {}", text_of(&row["key"])) }));
                        }
                    }
                }
                rows
            }
            Some(page) => self.console_entries(page),
            None if self.description["sections"][self.section]["from"] == "consoles" => {
                self.read.consoles.consoles.iter().map(|c| json!({ "id": format!("console:{}", c.console), "console": c.console })).collect()
            }
            None => self.description["sections"][self.section]["rows"].as_array().into_iter().flatten().filter(|e| self.controller_shown(e)).cloned().collect(),
        }
    }

    /// The console a page is of.
    pub(crate) fn console(&self) -> Option<&Console> {
        self.page().and_then(Page::console).and_then(|c| self.read.consoles.console(c))
    }

    fn console_word(&self, name: &str) -> String {
        self.description["consoles"]["words"][name].as_str().unwrap_or_default().to_owned()
    }

    /// The keys a console's page draws in rows of its own, so the schema's rows leave them out: those the description
    /// names, and the media's.
    fn own_keys(&self, console: &Console) -> Vec<String> {
        let named = ["page", "firmware"].iter().flat_map(|p| self.description["consoles"][p].as_array().cloned().unwrap_or_default());
        let mut keys: Vec<String> = named.filter_map(|e| e["schema"].as_str().filter(|k| *k != "*").map(str::to_owned)).collect();
        keys.extend(console.media_keys());
        keys
    }

    /// Whether a console's screen is a television, by Serenity's catalogue.
    fn television(&self, console: &str) -> bool {
        self.read.catalogue["screens"].as_array().into_iter().flatten().any(|s| s["console"] == console && s["kind"] == "Television")
    }

    /// A console page's rows: its own page from the description with the schema's other rows in place of `*`, each
    /// group under a heading; the engines; the firmware's needs and rows; the media's.
    fn console_entries(&self, page: &Page) -> Vec<Value> {
        let Some(console) = self.console() else { return Vec::new() };
        let value = |k: &str| console.value(k);
        match page {
            Page::Console(_) => {
                let television = self.television(&console.console);
                let own = self.own_keys(console);
                let mut out = Vec::new();
                for e in self.description["consoles"]["page"].as_array().into_iter().flatten() {
                    match e["screen"].as_str() {
                        Some("Television") if !television => continue,
                        Some("other") if television => continue,
                        _ => {}
                    }
                    match e["schema"].as_str() {
                        Some("*") => {
                            let mut group: Option<String> = None;
                            for r in console.schema.iter().filter(|r| !own.contains(&r.key) && schema::applies(r, &value)) {
                                if r.group.is_some() && r.group != group {
                                    let g = r.group.clone().unwrap_or_default();
                                    out.push(json!({ "id": format!("group:{g}"), "heading": g }));
                                }
                                group = r.group.clone();
                                out.push(json!({ "id": r.key, "schema": r.key }));
                            }
                        }
                        Some(key) if console.row(key).is_none_or(|r| r.choices.as_ref().is_none_or(Vec::is_empty)) => {}
                        _ => out.push(e.clone()),
                    }
                }
                out
            }
            Page::Engine(_) => {
                let choices = console.row(consoles::ENGINE_KEY).and_then(|r| r.choices.clone()).unwrap_or_default();
                choices.iter().map(|c| json!({ "id": format!("engine:{c}"), "choice": c })).collect()
            }
            Page::Firmware(_) => {
                let mut out = Vec::new();
                for e in self.description["consoles"]["firmware"].as_array().into_iter().flatten() {
                    if e["needs"] == true {
                        out.extend(console.needs().iter().enumerate().map(|(at, n)| json!({ "id": format!("fw:{}", n.name), "need": at })));
                    } else if e["schema"].as_str().is_none_or(|k| console.row(k).is_some()) {
                        out.push(e.clone());
                    }
                }
                out
            }
            Page::Cards(_) => cards::rows(&console.media, &console.schema, &value, false).iter().enumerate().map(|(at, r)| json!({ "id": r.id, "card": at })).collect(),
            Page::Drawer(_, medium) => {
                let m = console.media.iter().find(|m| &m.medium == medium);
                m.map(cards::drawer_rows).unwrap_or_default().iter().enumerate().map(|(at, r)| json!({ "id": r.id, "drawer": at })).collect()
            }
            Page::Saves(..) | Page::Save(..) | Page::Delete(..) => self.saves_entries(),
            _ => Vec::new(),
        }
    }

    /// Whether a page of the card manager is shown, whose rows `settings_saves.rs` resolves.
    fn on_saves(&self) -> bool {
        matches!(self.page(), Some(Page::Saves(..) | Page::Save(..) | Page::Delete(..)))
    }

    fn card_row(&self, entry: &Value) -> Option<cards::Row> {
        let console = self.console()?;
        let at = entry["card"].as_u64()? as usize;
        cards::rows(&console.media, &console.schema, &|k| console.value(k), false).into_iter().nth(at)
    }

    /// The consoles a row is set for: every console Serenity has a screen for, those whose screen is a television, or
    /// the console whose page is open.
    fn consoles(&self, which: &str) -> Vec<String> {
        if which == "this" {
            return self.page().and_then(Page::console).map(|c| vec![c.to_owned()]).unwrap_or_default();
        }
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
        self.consoles(entry["consoles"].as_str().unwrap_or("every")).iter().map(|c| self.filter_tier(entry, c)).collect()
    }

    /// A console's choice on the CRT row: 0 off, else 1 and the setting's value there or its initial value.
    fn filter_tier(&self, entry: &Value, c: &str) -> usize {
        let name = self.filter_name(&entry["filter"]);
        let setting = text_of(&entry["setting"]);
        let initial = self
            .filter(entry)
            .and_then(|f| f["settings"].as_array()?.iter().find(|s| s["id"] == entry["setting"]))
            .and_then(|s| s["initial"].as_u64())
            .map_or(0.0, f64::from_bits);
        let chosen = at(&self.read.graphics, &format!("Consoles.{c}.ScreenFilter")).map(text_of).unwrap_or_default();
        if chosen != name {
            return 0;
        }
        let held = self.read.graphics.get("ShaderParameters").and_then(|p| p.get(c)).and_then(|p| p.get(&name)).and_then(|p| p.get(&setting));
        1 + held.map(text_of).and_then(|t| t.parse::<f64>().ok()).unwrap_or(initial).max(0.0) as usize
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
        if let Some(row) = self.controller_row(entry) {
            return row;
        }
        if let Some(row) = self.console_row(entry) {
            return row;
        }
        let keyed = self.keyed(entry);
        let label = keyed.as_ref().map_or_else(|| text_of(&entry["label"]), |k| text_of(&k.row["label"]));
        let kind = if keyed.is_some() || entry.get("filter").is_some() {
            RowKind::Steps
        } else if entry.get("page").is_some() || entry.get("door").is_some() || entry.get("folder").is_some() {
            RowKind::Opens
        } else {
            RowKind::Action
        };
        let held = keyed.as_ref().is_some_and(|k| k.row["locked"] == true) || self.declined(entry);
        Row { id: text_of(&entry["id"]), label: self.fill(&label, ""), value: self.value(entry).0, kind, held, marked: false }
    }

    /// A row of the description a console's state declines: a door whose filter is not chosen, a folder to forget that
    /// is not set.
    fn declined(&self, entry: &Value) -> bool {
        if let Some(filter) = entry["while"].as_str() {
            let name = self.filter_name(&json!(filter));
            return !self.consoles("this").iter().all(|c| at(&self.read.graphics, &format!("Consoles.{c}.ScreenFilter")).map(text_of).as_deref() == Some(name.as_str()));
        }
        if let Some(clear) = entry["clear"].as_str() {
            let (model, key) = clear.split_once(' ').unwrap_or((clear, ""));
            return at(self.document(model), key).map(text_of).unwrap_or_default().is_empty();
        }
        false
    }

    /// A row of the Consoles section or a console's page that the description alone does not resolve.
    fn console_row(&self, entry: &Value) -> Option<Row> {
        if self.on_saves() {
            return self.saves_row(entry);
        }
        let id = text_of(&entry["id"]);
        let row = |label: String, value: String, kind: RowKind, held: bool| Some(Row { id: id.clone(), label, value, kind, held, marked: false });
        if let Some(name) = entry["console"].as_str() {
            let c = self.read.consoles.console(name)?;
            return row(c.name.clone(), if c.runs() { String::new() } else { self.console_word("unsupported") }, RowKind::Opens, !c.runs());
        }
        if let Some(group) = entry["heading"].as_str() {
            return row(group.to_owned(), String::new(), RowKind::Action, true);
        }
        let console = self.console()?;
        if let Some(key) = entry["schema"].as_str() {
            let r = console.row(key)?;
            let label = entry["label"].as_str().map_or_else(|| r.label.clone(), str::to_owned);
            return row(label, self.schema_value(console, entry), RowKind::Steps, r.locked);
        }
        if let Some(choice) = entry["choice"].as_str() {
            let chosen = console.engine() == choice;
            let value = if chosen { self.console_word("chosen") } else { self.console_word("installed") };
            return Some(Row { marked: chosen, ..row(self.read.consoles.engine_name(choice), value, RowKind::Choice, false)? });
        }
        if let Some(at) = entry["need"].as_u64() {
            let need = console.needs().into_iter().nth(at as usize)?;
            return row(need.name.clone(), firmware::status(&need, console.starts_empty), RowKind::Action, false);
        }
        if entry.get("card").is_some() {
            let r = self.card_row(entry)?;
            let kind = if r.steps || r.held { RowKind::Steps } else if r.opens { RowKind::Opens } else if r.asks { RowKind::Asks } else { RowKind::Action };
            return row(r.label, r.value, kind, r.held);
        }
        if let Some(at) = entry["drawer"].as_u64() {
            let medium = match self.page() {
                Some(Page::Drawer(_, m)) => m.clone(),
                _ => return None,
            };
            let r = console.media.iter().find(|m| m.medium == medium).map(cards::drawer_rows)?.into_iter().nth(at as usize)?;
            return row(r.label, r.value, RowKind::Action, false);
        }
        match entry["page"].as_str() {
            Some("firmware") => row(text_of(&entry["label"]), firmware::value(&console.needs(), console.starts_empty), RowKind::Opens, false),
            Some("cards") => row(text_of(&entry["label"]), String::new(), RowKind::Opens, console.media.is_empty()),
            _ if entry["filters"] == "suiting" => {
                let (shown, _) = self.screen_choice(console, entry);
                row(text_of(&entry["label"]), shown, RowKind::Steps, false)
            }
            _ => None,
        }
    }

    /// A schema row's value as its row says it: the description's word for it, an engine by its name, else the schema's.
    fn schema_value(&self, console: &Console, entry: &Value) -> String {
        let key = entry["schema"].as_str().unwrap_or_default();
        let value = console.value(key);
        if let Some(word) = entry["words"][&value].as_str() {
            return word.to_owned();
        }
        if key == consoles::ENGINE_KEY {
            return self.read.consoles.engine_name(&console.engine());
        }
        console.row(key).map(|r| schema::shown(r, &value)).unwrap_or(value)
    }

    /// The filters a console's screen may be drawn with (those of every console, and those naming it) and the one
    /// chosen, by name: the console's `ScreenFilter`, else the plain picture's.
    fn screen_filters(&self, console: &str) -> Vec<String> {
        let filters = self.read.catalogue["filters"].as_array().cloned().unwrap_or_default();
        filters
            .iter()
            .filter(|f| f["consoles"].as_array().is_none_or(|list| list.iter().any(|c| c == console)))
            .map(|f| text_of(&f["name"]))
            .collect()
    }

    fn screen_choice(&self, console: &Console, entry: &Value) -> (String, Option<usize>) {
        let names = self.screen_filters(&console.console);
        let off = self.filter_name(&entry["off"]);
        let chosen = at(&self.read.graphics, &format!("Consoles.{}.ScreenFilter", console.console)).map(text_of).filter(|t| !t.is_empty()).unwrap_or(off);
        let index = names.iter().position(|n| *n == chosen);
        (chosen, index)
    }

    /// The pane of a row shown, by its index.
    pub fn pane(&self, index: usize) -> Pane {
        let entry = self.entries().get(index).cloned();
        if let Some(pane) = self.controller_pane(entry.as_ref()) {
            return pane;
        }
        let Some(entry) = entry else { return Pane::default() };
        if let Some(words) = self.console_pane(&entry) {
            let mut lines = words.lines.into_iter();
            return Pane { sentence: lines.next().unwrap_or_default(), lines: lines.collect(), muted: words.muted, tones: words.tones, ..Pane::default() };
        }
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
            tones: Vec::new(),
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
                "console" => self.console().map(|c| c.name.clone()).unwrap_or_default(),
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
        if let Some(outcome) = self.controller_step(&entry, by) {
            return outcome;
        }
        if let Some(outcome) = self.console_step(&entry, by) {
            return outcome;
        }
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
        if let Some(outcome) = self.controller_choose(&entry, index) {
            return outcome;
        }
        if let Some(outcome) = self.console_choose(&entry, index) {
            return outcome;
        }
        if self.keyed(&entry).is_some() || entry.get("filter").is_some() {
            return self.step(index, 1);
        }
        if let Some(group) = entry["page"].as_str() {
            return self.open(Page::Group(group.to_owned()), text_of(&entry["label"]), index);
        }
        if let Some(key) = entry["folder"].as_str() {
            let (model, field) = key.split_once(' ').unwrap_or((key, ""));
            let start = at(self.document(model), field).map(text_of).unwrap_or_default();
            return Outcome::ChooseFolder { key: key.to_owned(), start };
        }
        if let Some(folder) = entry["open"].as_str() {
            return self.read.folders.get(folder).map_or(Outcome::Nothing, |path| Outcome::OpenFolder(path.clone()));
        }
        entry["door"].as_str().map_or(Outcome::Nothing, |d| Outcome::Door(d.to_owned()))
    }

    /// The folder chosen in the dialog for a folder row's key, saved as its model's every other value is.
    pub fn set_folder(&mut self, key: &str, path: &str) -> Outcome {
        let Some((model, field)) = key.split_once(' ') else { return Outcome::Nothing };
        let graphics = model.starts_with("GraphicsConfig");
        let document = if graphics { &mut self.read.graphics } else { &mut self.read.app };
        set_at(document, field, json!(path));
        Outcome::Saved(if graphics { Model::Graphics } else { Model::App }, document.clone())
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

    /// Whether A and B, and X and Y, trade places in the menus (Swap A and B in menus).
    pub fn swap_buttons(&self) -> bool {
        self.applies_as("swap_buttons")
    }

    /// The help bar's pad family chosen by Controller type; none for Automatic, which follows the pad last pressed.
    pub fn controller_family(&self) -> Option<PadFamily> {
        PadFamily::named(&self.applied("controller_type")?)
    }

    /// Whether a pad's coming and going raises a notice (Controller notices).
    pub fn controller_notices(&self) -> bool {
        self.applies_as("controller_notices")
    }

    /// Whether the controllers' motors run where a game drives them (Rumble).
    pub fn rumble(&self) -> bool {
        self.applies_as("rumble")
    }

    /// The settings the pads act on: Only the first controller, Left stick as the D-pad, Stick deadzone, Mirror player 1
    /// to player 2; none until the settings are read. A stored deadzone that is not a number is left unset, in words.
    pub fn pad_settings(&self) -> Option<(PadSettings, Option<String>)> {
        let stored = self.applied("stick_deadzone").filter(|v| !v.is_empty())?;
        let deadzone = stored.parse::<f64>().ok();
        let settings = PadSettings {
            first_controller_only: self.applies_as("first_controller_only"),
            stick_as_dpad: self.applies_as("stick_as_dpad"),
            stick_deadzone: deadzone,
            mirror: self.applies_as("mirror"),
        };
        Some((settings, deadzone.is_none().then(|| unread_deadzone(&stored))))
    }
}

/// The words for a stored stick deadzone that is not a number, said as the program's other failures are.
pub fn unread_deadzone(stored: &str) -> String {
    format!("The stick deadzone could not be read, so it was left as it was: \"{stored}\" is not a number")
}

/// A console's words with its names filled: `{console}` and the names given.
fn said(line: &str, names: &[(&str, &str)]) -> String {
    names.iter().fold(line.to_owned(), |l, (name, value)| l.replace(&format!("{{{name}}}"), value))
}

impl SettingsView {
    /// The games a console's systems hold, as the library counted them.
    fn games_of(&self, console: &Console) -> usize {
        console.systems.iter().map(|s| self.facts["systems"][s].as_u64().unwrap_or(0) as usize).sum()
    }

    /// A console in the list: its games, its engine, its firmware, its CRT and its first choice, and what they are
    /// (Mistress's `ConsolePane`).
    fn list_pane(&self, c: &Console) -> firmware::Words {
        let name = c.name.as_str();
        if !c.runs() {
            return firmware::Words { lines: vec![said(&self.console_word("no_engine"), &[("console", name)])], ..Default::default() };
        }
        let games = words::counted(self.games_of(c), &self.word("game"), &self.word("games"));
        let engine = self.read.consoles.engine_name(&c.engine());
        let mut lines = vec![said(&self.console_word("facts"), &[("console", name), ("games", &games), ("engine", &engine)]), firmware::line(&c.needs(), c.starts_empty)];
        let mut facts = Vec::new();
        if self.television(&c.console) {
            if let Some(crt) = self.description["consoles"]["page"].as_array().and_then(|p| p.iter().find(|e| e.get("filter").is_some())) {
                let word = self.filter_choices(crt).get(self.filter_tier(crt, &c.console)).cloned().unwrap_or_default();
                facts.push(said(&self.console_word("crt"), &[("crt", &word)]));
            }
        }
        let own = self.own_keys(c);
        if let Some(r) = c.schema.iter().find(|r| r.kind == "choice" && !own.contains(&r.key) && schema::applies(r, &|k| c.value(k))) {
            facts.push(format!("{} {}", r.label, schema::shown(r, &c.value(&r.key))));
        }
        if !facts.is_empty() {
            lines.push(facts.join(" · "));
        }
        let tones = vec![Tone::Text, firmware::line_tone(&c.needs(), c.starts_empty)];
        firmware::Words { lines, muted: vec![said(&self.console_word("shares"), &[("console", name)])], tones }
    }

    /// The pane of a row the description alone does not word: the list's consoles, a console's schema and engines, its
    /// firmware and its cards.
    fn console_pane(&self, entry: &Value) -> Option<firmware::Words> {
        if self.on_saves() {
            return self.saves_pane(entry);
        }
        let one = |line: String| Some(firmware::Words { lines: vec![line], ..Default::default() });
        if let Some(name) = entry["console"].as_str() {
            return Some(self.list_pane(self.read.consoles.console(name)?));
        }
        let console = self.console()?;
        let name = console.name.as_str();
        if let Some(group) = entry["heading"].as_str() {
            return one(said(&self.console_word("heading"), &[("console", name), ("group", &group.to_lowercase())]));
        }
        if let Some(key) = entry["schema"].as_str() {
            let row = console.row(key)?;
            if key == consoles::ENGINE_KEY {
                let engine = self.read.consoles.engine_name(&console.engine());
                let chosen = console.engine();
                let others: Vec<String> = row.choices.iter().flatten().filter(|c| **c != chosen).map(|c| self.read.consoles.engine_name(c)).collect();
                let muted = if others.is_empty() { Vec::new() } else { vec![said(&self.console_word("engine_others"), &[("others", &others.join(", "))])] };
                return Some(firmware::Words { lines: vec![said(&self.console_word("engine"), &[("console", name), ("engine", &engine)])], muted, ..Default::default() });
            }
            let value = self.schema_value(console, entry);
            let pane = &entry["pane"];
            let hint = entry["hint"].as_str().map(str::to_owned).unwrap_or_else(|| {
                let note = row.note.clone().filter(|n| !n.is_empty()).map(|n| format!(" {n}")).unwrap_or_default();
                format!("{}{note}", row.hint.clone().unwrap_or_default())
            });
            let sentence = pane["by_value"][&value].as_str().map_or(hint, str::to_owned);
            return Some(firmware::Words { lines: vec![self.fill(&sentence, &value)], muted: strings(&pane["muted"]), ..Default::default() });
        }
        if let Some(choice) = entry["choice"].as_str() {
            let word = if console.engine() == choice { "runs" } else { "would_run" };
            let engine = self.read.consoles.engine_name(choice);
            return Some(firmware::Words { lines: vec![said(&self.console_word(word), &[("choice", &engine), ("console", name)])], muted: vec![self.console_word("choose")], ..Default::default() });
        }
        if let Some(at) = entry["need"].as_u64() {
            let need = console.needs().into_iter().nth(at as usize)?;
            let engine = self.read.consoles.engine_name(&console.engine());
            let replaced = console.value("FirmwarePreference") == "replacement";
            return Some(firmware::need_pane(&need, console.starts_empty, name, &engine, replaced, &self.read.consoles.overview));
        }
        if entry.get("card").is_some() {
            let row = self.card_row(entry)?;
            return Some(cards::pane(&console.media, &row, &console.schema, &|k| console.value(k), name, false));
        }
        if entry.get("drawer").is_some() {
            return one(cards::DRAWER_CARD.to_owned());
        }
        match entry["page"].as_str() {
            Some("firmware") => Some(firmware::pane(&console.needs(), console.starts_empty, name)),
            Some("cards") => match console.media.first() {
                Some(m) => Some(cards::mode_pane(m, &console.schema, &|k| console.value(k), name)),
                None => one(said(&self.console_word("cartridge"), &[("console", name)])),
            },
            _ if entry["filters"] == "suiting" => one(self.fill(&text_of(&entry["hint"]), "")),
            _ => None,
        }
    }

    /// A console's key changed in the rows read, then stored through the runtime.
    fn set(&mut self, console: &str, key: &str, value: Option<String>) -> Outcome {
        if let Some(c) = self.read.consoles.console_mut(console) {
            match &value {
                Some(v) => c.values.insert(key.to_owned(), v.clone()),
                None => c.values.remove(key),
            };
        }
        Outcome::Set { console: console.to_owned(), key: key.to_owned(), value }
    }

    /// Left or Right on a console's row: a schema row, a card's mode or slot, a screen's filter.
    fn console_step(&mut self, entry: &Value, by: i32) -> Option<Outcome> {
        if self.on_saves() {
            return self.saves_step(entry, by);
        }
        let console = self.console()?.clone();
        let value = |k: &str| console.value(k);
        if let Some(key) = entry["schema"].as_str() {
            let row = console.row(key)?;
            if row.locked {
                return Some(Outcome::Nothing);
            }
            return Some(match schema::stepped(row, &value(key), by) {
                Some(next) => self.set(&console.console, key, Some(next)),
                None => Outcome::Nothing,
            });
        }
        if entry.get("card").is_some() {
            let row = self.card_row(entry)?;
            return Some(match cards::step(&row, &console.schema, &value, by) {
                Some((key, next)) => self.set(&console.console, &key, Some(next)),
                None => Outcome::Nothing,
            });
        }
        if entry["filters"] == "suiting" {
            let names = self.screen_filters(&console.console);
            if names.is_empty() {
                return Some(Outcome::Nothing);
            }
            let (_, chosen) = self.screen_choice(&console, entry);
            let next = chosen.map_or(if by > 0 { 0 } else { names.len() - 1 }, |i| wrapped(i, by, names.len()));
            set_at(&mut self.read.graphics, &format!("Consoles.{}.ScreenFilter", console.console), json!(names[next]));
            return Some(Outcome::Saved(Model::Graphics, self.read.graphics.clone()));
        }
        if entry.get("console").is_some() || entry.get("heading").is_some() || entry.get("need").is_some() || entry.get("drawer").is_some() {
            return Some(Outcome::Nothing);
        }
        None
    }

    /// A on a console's row: a console's page, the engines' page, a choice taken, a page of firmware or cards, a card
    /// made, the console reset, the folders read again.
    fn console_choose(&mut self, entry: &Value, index: usize) -> Option<Outcome> {
        if self.on_saves() {
            return self.saves_choose(entry, index);
        }
        if let Some(name) = entry["console"].as_str() {
            let c = self.read.consoles.console(name)?;
            return Some(if c.runs() { self.open(Page::Console(c.console.clone()), c.name.clone(), index) } else { Outcome::Nothing });
        }
        let console = self.console()?.clone();
        if let Some(key) = entry["schema"].as_str() {
            if entry["page"] == "engine" {
                let label = console.row(key).map(|r| r.label.clone()).unwrap_or_default();
                return Some(self.open(Page::Engine(console.console.clone()), label, index));
            }
            return self.console_step(entry, 1);
        }
        if let Some(choice) = entry["choice"].as_str() {
            let outcome = self.set(&console.console, consoles::ENGINE_KEY, Some(choice.to_owned()));
            self.pages.pop();
            return Some(outcome);
        }
        if let Some(card) = self.card_row(entry) {
            return Some(match card.part {
                cards::Part::Drawer => {
                    let title = console.media.iter().find(|m| m.medium == card.medium).map(|m| format!("{}s", m.title)).unwrap_or_default();
                    self.open(Page::Drawer(console.console.clone(), card.medium.clone()), title, index)
                }
                cards::Part::Saves => {
                    let title = console.media.iter().find(|m| m.medium == card.medium).map(|m| m.title.clone()).unwrap_or_default();
                    self.chosen_card = None;
                    self.target_card = None;
                    self.said.clear();
                    self.open(Page::Saves(console.console.clone(), card.medium.clone()), title, index)
                }
                cards::Part::New => console.media.iter().find(|m| m.medium == card.medium).and_then(|m| m.new_card_path()).map_or(Outcome::Nothing, Outcome::NewCard),
                _ => self.console_step(entry, 1).unwrap_or(Outcome::Nothing),
            });
        }
        let label = text_of(&entry["label"]);
        match entry["page"].as_str() {
            Some("firmware") => return Some(self.open(Page::Firmware(console.console.clone()), label, index)),
            Some("cards") if !console.media.is_empty() => return Some(self.open(Page::Cards(console.console.clone()), label, index)),
            Some("cards") => return Some(Outcome::Nothing),
            _ => {}
        }
        if entry["reset"] == true {
            let mut all: Vec<Outcome> = console.schema.iter().map(|r| self.set(&console.console, &r.key, None)).collect();
            if let Some(map) = self.read.graphics.get_mut("Consoles").and_then(Value::as_object_mut) {
                map.remove(&console.console);
            }
            if let Some(map) = self.read.graphics.get_mut("ShaderParameters").and_then(Value::as_object_mut) {
                map.remove(&console.console);
            }
            all.push(Outcome::Saved(Model::Graphics, self.read.graphics.clone()));
            return Some(Outcome::All(all));
        }
        if entry["reread"] == true {
            return Some(Outcome::Reread);
        }
        if let Some(clear) = entry["clear"].as_str() {
            if self.declined(entry) {
                return Some(Outcome::Nothing);
            }
            let (model, key) = clear.split_once(' ').unwrap_or((clear, ""));
            let graphics = model.starts_with("GraphicsConfig");
            let document = if graphics { &mut self.read.graphics } else { &mut self.read.app };
            set_at(document, key, Value::Null);
            return Some(Outcome::Saved(if graphics { Model::Graphics } else { Model::App }, document.clone()));
        }
        if entry.get("while").is_some() && self.declined(entry) {
            return Some(Outcome::Nothing);
        }
        if entry.get("need").is_some() || entry.get("heading").is_some() || entry.get("drawer").is_some() {
            return Some(Outcome::Nothing);
        }
        None
    }

    /// LB or RB on a console's page: the console before or after it among those an engine runs, its page in place of
    /// this one; elsewhere nothing, the shoulders being the strip's.
    pub fn shoulder(&mut self, by: i32) -> bool {
        let Some((Page::Console(from), _, at)) = self.pages.first().cloned() else { return false };
        if self.pages.len() != 1 {
            return false;
        }
        let running: Vec<(String, String)> = self.read.consoles.running().iter().map(|c| (c.console.clone(), c.name.clone())).collect();
        let Some(here) = running.iter().position(|(c, _)| *c == from) else { return false };
        if running.len() < 2 {
            return false;
        }
        let (to, name) = running[wrapped(here, by, running.len())].clone();
        let row = self.read.consoles.consoles.iter().position(|c| c.console == to).unwrap_or(at);
        self.pages[0] = (Page::Console(to), name, row);
        true
    }
}
