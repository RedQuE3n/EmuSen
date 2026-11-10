//! Settings ▸ Controllers' own pages (EmuSen_Qt.md §7.7.H; EmuSen_Input.md §8.9, §10.5, §10.6; Mistress's
//! `SettingsScreen.Controllers` and `.Keys`): Players, Profiles and a profile's page, Hotkeys, and the bindings page with
//! the console's controller drawn, for the pad or the keyboard. Every word is the description's (`controllers` in
//! `data/settings_screen.json`); the seats are the pads' reader's; the bindings are Endymion's models (`bindings.rs`),
//! a map chosen by Endymion's `bindings_choose`.

use crate::bindings::{pad_words, Bindings, Choice, File};
use crate::consoles::Console;
use crate::controllers;
use crate::settings::{Outcome, Page, Pane, Preview, Row, SettingsView};
use crate::tools::Tone;
use emusen_kit_core::settings::RowKind;
use emusen_qt_endymion::seats::{Ask, Connected, Player};
use serde_json::{json, Value};

/// The seats and the pads connected, as the pads' reader last gave them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Seated {
    pub players: Vec<Player>,
    pub pads: Vec<Connected>,
    /// Endymion's stick deadzone, past which a stick's direction is held.
    pub deadzone: f64,
}

/// The bindings page's state: the console, the profile and player whose bindings it shows, the control focused, and
/// whether A listens for a press, the tester runs, or it is the keyboard's page.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Binding {
    pub console: String,
    pub profile: String,
    pub player: u32,
    pub focus: usize,
    pub capturing: bool,
    pub testing: bool,
    pub keys: bool,
}

/// Endymion's choice of a player's map (`bindings_choose`), over the models as they stand; and the gamepad button each
/// canonical button is bound to for a player as the files stand (`bindings_bound`), SDL's number or -1.
pub trait Chooser: Send + Sync {
    fn choose(&self, gamepad: &Value, profiles: &Value, console: &str, player: u32, profile: Option<&str>, game: Option<&str>) -> Choice;
    fn bound(&self, console: &str, player: u32) -> [i32; 16];
    /// The name a game is known by in a console's profiles, from its path: Endymion's.
    fn game_key(&self, path: &str) -> String;
    /// Each control's word in the binding files, with the button it is or the stick's axis and way: Endymion's.
    fn controls(&self) -> &[ControlWord];
}

/// A control as the binding files name it (`emusen_endymion_bindings_controls`): its word, and the core's button it is
/// or the axis a stick's direction pushes and which way.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ControlWord {
    pub word: String,
    pub button: Option<u32>,
    pub axis: Option<u32>,
    pub toward: i32,
}

impl ControlWord {
    /// Endymion's answer read into words; an entry with no word is passed over.
    pub fn read(answer: &Value) -> Vec<ControlWord> {
        let number = |e: &Value, k: &str| e[k].as_u64().map(|n| n as u32);
        answer.as_array().into_iter().flatten().filter_map(|e| {
            Some(ControlWord { word: e["word"].as_str()?.to_owned(), button: number(e, "button"), axis: number(e, "axis"), toward: e["toward"].as_i64().unwrap_or(0) as i32 })
        }).collect()
    }
}

/// The bits a pad's triggers take beside SDL's buttons in what it holds: Endymion's (`PADS_HELD`).
pub use emusen_platform_client::pads::consts::{HELD_LEFT_TRIGGER as LEFT_TRIGGER_BIT, HELD_RIGHT_TRIGGER as RIGHT_TRIGGER_BIT};

/// What the drawing shows: the drawing file (empty for the generic one), the controls, the words on each label, the
/// control focused and those held, and the strip over it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Drawn {
    pub drawing: String,
    pub controls: Vec<String>,
    pub texts: Vec<String>,
    pub focus: i32,
    pub held: Vec<String>,
    /// How far each control is pushed, 0 to 1, in the controls' order; empty while the tester does not run.
    pub pushes: Vec<f64>,
    pub tabs: Vec<String>,
    pub caption: String,
    /// What a reader calls the drawing.
    pub name: String,
}

/// What a typed answer is for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Typed {
    NewProfile { console: String, from: String },
    Rename { console: String, profile: String },
}

fn fill(line: &str, names: &[(&str, &str)]) -> String {
    names.iter().fold(line.to_owned(), |l, (name, value)| l.replace(&format!("{{{name}}}"), value))
}

impl SettingsView {
    fn cw(&self, name: &str) -> String {
        self.description_at(&["controllers", "words", name]).as_str().unwrap_or_default().to_owned()
    }

    /// The consoles with an engine and a pad, in the order Consoles lists them.
    pub fn pad_consoles(&self) -> Vec<&Console> {
        self.read.consoles.consoles.iter().filter(|c| c.runs() && !controllers::controls(&c.controllers).is_empty()).collect()
    }

    /// The console of the game running, by its system, from the program's facts.
    pub(crate) fn running_console(&self) -> Option<String> {
        let system = self.facts["system"].as_str().filter(|s| !s.is_empty())?;
        self.read.consoles.consoles.iter().find(|c| c.systems.iter().any(|s| s == system)).map(|c| c.console.clone())
    }

    /// The console Profiles and the bindings page open on: the one stepped to, else the game running's, else the first.
    pub fn pad_console(&self) -> Option<String> {
        let consoles: Vec<String> = self.pad_consoles().iter().map(|c| c.console.clone()).collect();
        if let Some(chosen) = self.stepped_console.as_ref().filter(|c| consoles.contains(c)) {
            return Some(chosen.clone());
        }
        self.running_console().filter(|r| consoles.contains(r)).or_else(|| consoles.first().cloned())
    }

    fn console_name(&self, console: &str) -> String {
        self.read.consoles.console(console).map_or_else(|| console.to_owned(), |c| c.name.clone())
    }

    pub(crate) fn controls_of(&self, console: &str) -> Vec<String> {
        self.read.consoles.console(console).map(|c| controllers::controls(&c.controllers)).unwrap_or_default()
    }

    /// A control in words: a button by its name, a stick's direction by the description's words for it.
    fn control_words(&self, control: &str) -> String {
        self.description_at(&["controllers", "sticks", control]).as_str().map_or_else(|| control.to_owned(), str::to_owned)
    }

    fn players_of(&self, console: &str) -> u32 {
        self.read.consoles.console(console).map_or(1, |c| controllers::players(&c.controllers))
    }

    /// The map a player plays with in a profile, as Endymion chooses it.
    fn choice(&self, console: &str, player: u32, profile: &str) -> Choice {
        let b = &self.read.bindings;
        match &self.chooser {
            Some(chooser) => chooser.choose(&b.gamepad, &b.profiles, console, player, Some(profile), None),
            None => Choice { profile: profile.to_owned(), map_player: 1 },
        }
    }

    /// The emulator's actions in the screen's order, each with its words: those of the description that Endymion's
    /// hotkeys have, then any other it has, by its own name.
    fn actions(&self) -> Vec<(String, String)> {
        let known = self.read.bindings.actions();
        let mut out: Vec<(String, String)> = self
            .description_at(&["controllers", "actions"])
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|pair| Some((pair[0].as_str()?.to_owned(), pair[1].as_str()?.to_owned())))
            .filter(|(a, _)| known.contains(a))
            .collect();
        out.extend(known.into_iter().filter(|a| !out.iter().any(|(o, _)| o == a)).map(|a| (a.clone(), a)).collect::<Vec<_>>());
        out
    }

    fn action_words(&self, action: &str) -> String {
        self.actions().into_iter().find(|(a, _)| a == action).map_or_else(|| action.to_owned(), |(_, w)| w)
    }

    /// The rows of Controllers' own pages.
    pub(crate) fn controller_entries(&self, page: &Page) -> Option<Vec<Value>> {
        Some(match page {
            Page::Players => {
                let mut rows: Vec<Value> = (0..self.seats.pads.len()).map(|i| json!({ "id": format!("pad{i}"), "pad": i })).collect();
                for p in self.seats.players.iter().filter(|p| p.kept && p.pad.is_some()) {
                    rows.push(json!({ "id": format!("kept{}", p.player), "kept": p.player }));
                }
                let group = self.cw("players_group");
                if let Some(models) = self.read.rows.as_object() {
                    for (model, list) in models {
                        for row in list.as_array().into_iter().flatten().filter(|r| r["group"] == group.as_str()) {
                            let key = row["key"].as_str().unwrap_or_default();
                            rows.push(json!({ "id": key.to_lowercase(), "row": format!("{model} {key}") }));
                        }
                    }
                }
                rows
            }
            Page::Profiles => {
                let Some(console) = self.pad_console() else { return Some(Vec::new()) };
                let mut rows = vec![json!({ "id": "pconsole", "pconsole": true })];
                rows.extend(self.read.bindings.profiles(&console).into_iter().map(|p| json!({ "id": format!("profile:{p}"), "profile": p })));
                rows.push(json!({ "id": "newprofile", "newprofile": true }));
                rows
            }
            Page::Profile(..) => ["pbindings", "pdefault", "prename", "pdelete"].iter().map(|k| json!({ "id": k, *k: true })).collect(),
            Page::Hotkeys => self.actions().into_iter().map(|(a, _)| json!({ "id": format!("hk:{a}"), "hotkey": a })).collect(),
            Page::Bindings => Vec::new(),
            _ => return None,
        })
    }

    /// Whether a section row of the description is shown: Rumble where a console's games drive a motor (the game
    /// running's console, else any).
    pub(crate) fn controller_shown(&self, entry: &Value) -> bool {
        if entry["controllers"] != "rumble" {
            return true;
        }
        let shaking: Vec<&Console> = self.read.consoles.consoles.iter().filter(|c| c.rumble).collect();
        match self.running_console() {
            Some(running) => shaking.iter().any(|c| c.console == running),
            None => !shaking.is_empty(),
        }
    }

    fn profile_page(&self) -> Option<(String, String)> {
        match self.page()? {
            Page::Profile(c, p) => Some((c.clone(), p.clone())),
            _ => None,
        }
    }

    /// A row of Controllers' own that the description alone does not resolve.
    pub(crate) fn controller_row(&self, entry: &Value) -> Option<Row> {
        let id = entry["id"].as_str().unwrap_or_default().to_owned();
        let row = |label: String, value: String, kind: RowKind, held: bool| Some(Row { id: id.clone(), label, value, kind, held, marked: false });
        let label = || entry["label"].as_str().unwrap_or_default().to_owned();
        match entry["controllers"].as_str() {
            Some("players") => {
                let set = self.seats.players.iter().filter(|p| p.pad.is_some()).count();
                let count = set.max(self.seats_shown());
                return row(label(), fill(&self.cw("players_value"), &[("set", &set.to_string()), ("count", &count.to_string())]), RowKind::Opens, false);
            }
            Some("bindings" | "keys") => return row(label(), String::new(), RowKind::Opens, self.pad_console().is_none()),
            Some("profiles") => {
                let console = self.pad_console();
                let value = console.as_ref().map_or_else(|| self.read.bindings.default_profile.clone(), |c| self.read.bindings.console_default(c));
                return row(label(), value, RowKind::Opens, console.is_none());
            }
            Some("hotkeys") => return row(label(), String::new(), RowKind::Opens, false),
            Some("reset") => return row(label(), String::new(), RowKind::Action, false),
            _ => {}
        }
        if let Some(i) = entry["pad"].as_u64() {
            let pad = self.seats.pads.get(i as usize)?;
            let value = self.player_words(pad.player);
            return row(pad.name.clone(), if pad.pressed { format!("{}{value}", self.cw("pressed")) } else { value }, RowKind::Steps, false);
        }
        if let Some(n) = entry["kept"].as_u64() {
            let p = self.seats.players.iter().find(|p| p.player == n as usize)?;
            let label = fill(&self.cw("kept_label"), &[("n", &n.to_string()), ("pad", p.pad.as_deref().unwrap_or_default())]);
            return row(label, self.cw("forget"), RowKind::Action, false);
        }
        if entry["pconsole"] == true {
            return row(self.cw("console_label"), self.pad_console().map(|c| self.console_name(&c)).unwrap_or_default(), RowKind::Steps, false);
        }
        if let Some(profile) = entry["profile"].as_str() {
            let console = self.pad_console()?;
            let value = if self.read.bindings.console_default(&console) == profile { self.cw("starts") } else { String::new() };
            return row(profile.to_owned(), value, RowKind::Opens, false);
        }
        if entry["newprofile"] == true {
            return row(self.cw("new_label"), String::new(), RowKind::Asks, false);
        }
        if let Some((console, profile)) = self.profile_page() {
            let is_default = *profile == self.read.bindings.default_profile;
            let chosen = self.read.bindings.console_default(&console) == profile;
            for (key, words, kind, held, value) in [
                ("pbindings", "pbindings_label", RowKind::Opens, false, String::new()),
                ("pdefault", "pdefault_label", RowKind::Action, chosen, if chosen { self.cw("chosen") } else { String::new() }),
                ("prename", "rename_label", RowKind::Asks, is_default, String::new()),
                ("pdelete", "delete_label", RowKind::Action, is_default, String::new()),
            ] {
                if entry[key] == true {
                    return row(self.cw(words), value, kind, held);
                }
            }
        }
        if let Some(action) = entry["hotkey"].as_str() {
            return row(self.action_words(action), self.hotkey_value(action), RowKind::Action, false);
        }
        None
    }

    /// How many seats the Players row and page count at least (the description's, Mistress's four).
    fn seats_shown(&self) -> usize {
        (self.description_at(&["controllers", "seats_shown"]).as_u64().unwrap_or(0) as usize).max(self.seats.players.len())
    }

    fn player_words(&self, player: usize) -> String {
        if player == 0 { self.cw("none") } else { fill(&self.cw("player"), &[("n", &player.to_string())]) }
    }

    /// The consoles whose controls are bound to a key, by name, of those with a pad.
    fn consoles_holding(&self, key: &str) -> Vec<String> {
        let consoles: Vec<(String, Vec<String>)> = self.pad_consoles().iter().map(|c| (c.console.clone(), controllers::controls(&c.controllers))).collect();
        self.read.bindings.consoles_holding(key, &consoles).into_iter().map(|c| self.console_name(c)).collect()
    }

    fn hotkey_value(&self, action: &str) -> String {
        if self.listening.as_deref() == Some(action) {
            return self.cw("hotkey_listening");
        }
        let Some(key) = self.read.bindings.hotkey(action) else { return self.cw("hotkey_unbound") };
        let twice = self.read.bindings.actions().iter().filter(|a| self.read.bindings.hotkey(a).as_deref() == Some(key.as_str())).count() > 1;
        if twice || !self.consoles_holding(&key).is_empty() { fill(&self.cw("hotkey_clash"), &[("key", &key)]) } else { key }
    }

    /// The pane of a row of Controllers' own, or of the bindings page.
    pub(crate) fn controller_pane(&self, entry: Option<&Value>) -> Option<Pane> {
        if let Some(b) = &self.binding {
            return Some(self.binding_pane(b));
        }
        let entry = entry?;
        let said = |sentence: String, muted: Vec<String>| Some(Pane { sentence, muted, ..Pane::default() });
        match entry["controllers"].as_str() {
            Some("players") => {
                let profile = self.pad_console().map(|c| fill(&self.cw("profile_mark"), &[("profile", &self.read.bindings.console_default(&c))])).unwrap_or_default();
                let mut lines: Vec<String> = self
                    .seats
                    .players
                    .iter()
                    .filter_map(|p| {
                        let pad = p.pad.as_ref()?;
                        let kept = if p.kept { self.cw("kept_mark") } else { String::new() };
                        Some(format!("{}{kept}{profile}", fill(&self.cw("players_line"), &[("n", &p.player.to_string()), ("pad", pad)])))
                    })
                    .collect();
                let free: Vec<String> = (1..=self.seats_shown()).filter(|n| self.seats.players.iter().all(|p| p.player != *n || p.pad.is_none())).map(|n| n.to_string()).collect();
                let mut muted = Vec::new();
                if !free.is_empty() {
                    muted.push(fill(&self.cw("free"), &[("list", &free.join(&self.cw("and")))]));
                }
                muted.push(self.cw("players_muted"));
                let sentence = if lines.is_empty() { String::new() } else { lines.remove(0) };
                return Some(Pane { sentence, lines, muted, tones: vec![Tone::Text], ..Pane::default() });
            }
            Some("profiles") => {
                let Some(console) = self.pad_console() else { return said(self.cw("no_console"), Vec::new()) };
                let n = self.read.bindings.profiles(&console).len();
                let count = crate::words::counted(n, &self.cw("profile"), &self.cw("profiles"));
                let line = fill(&self.cw("profiles_line"), &[("console", &self.console_name(&console)), ("count", &count), ("chosen", &self.read.bindings.console_default(&console))]);
                return said(line, vec![self.cw("profiles_muted")]);
            }
            Some("rumble") => {
                let value = self.app_value("Rumble");
                let by = &entry["pane"]["by_value"];
                let sentence = by[if value == "true" { "On" } else { "Off" }].as_str().unwrap_or_default().to_owned();
                let mut names: Vec<String> = Vec::new();
                for c in self.read.consoles.consoles.iter().filter(|c| c.rumble) {
                    if !names.contains(&c.name) {
                        names.push(c.name.clone());
                    }
                }
                return said(sentence, vec![fill(&self.cw("rumble_muted"), &[("names", &names.join(", "))])]);
            }
            _ => {}
        }
        if let Some(i) = entry["pad"].as_u64() {
            let pad = self.seats.pads.get(i as usize)?;
            return said(self.cw(if pad.player == 0 { "pad_none" } else { "pad_seated" }), Vec::new());
        }
        if entry.get("kept").is_some() {
            return said(self.cw("kept_hint"), Vec::new());
        }
        if entry["pconsole"] == true {
            return said(self.cw("console_hint"), Vec::new());
        }
        if let Some(profile) = entry["profile"].as_str() {
            let console = self.pad_console()?;
            let name = self.console_name(&console);
            let chosen = self.read.bindings.console_default(&console) == profile;
            let line = fill(&self.cw(if chosen { "starts_line" } else { "kept_line" }), &[("profile", profile), ("console", &name)]);
            return said(line, vec![self.cw("profile_muted")]);
        }
        if entry["newprofile"] == true {
            let console = self.pad_console()?;
            return said(fill(&self.cw("new_hint"), &[("chosen", &self.read.bindings.console_default(&console)), ("console", &self.console_name(&console))]), Vec::new());
        }
        if let Some((console, profile)) = self.profile_page() {
            let names = [("console", self.console_name(&console)), ("profile", profile.clone())];
            let names: Vec<(&str, &str)> = names.iter().map(|(k, v)| (*k, v.as_str())).collect();
            let is_default = *profile == self.read.bindings.default_profile;
            for (key, words) in [
                ("pbindings", "pbindings_hint"),
                ("pdefault", "pdefault_hint"),
                ("prename", if is_default { "rename_default" } else { "rename_hint" }),
                ("pdelete", if is_default { "delete_default" } else { "delete_hint" }),
            ] {
                if entry[key] == true {
                    return said(fill(&self.cw(words), &names), Vec::new());
                }
            }
        }
        if let Some(action) = entry["hotkey"].as_str() {
            return Some(self.hotkey_pane(action));
        }
        None
    }

    fn app_value(&self, key: &str) -> String {
        match &self.read.app[key] {
            Value::Bool(b) => b.to_string(),
            Value::String(s) => s.clone(),
            other => other.to_string(),
        }
    }

    fn hotkey_pane(&self, action: &str) -> Pane {
        let mut lines = Vec::new();
        let mut tones = vec![Tone::Text];
        let held = self.description_at(&["controllers", "held_actions"]).as_array().is_some_and(|a| a.iter().any(|x| x == action));
        let sentence = self.cw(if held { "hotkey_held" } else { "hotkey_press" });
        if self.listening.as_deref() == Some(action) {
            lines.push(format!("{} {}", self.cw("listening_mark"), self.cw("hotkey_listen")));
            tones.push(Tone::Attention);
        } else if let Some(key) = self.read.bindings.hotkey(action) {
            lines.push(fill(&self.cw("bound_key"), &[("key", &key)]));
            tones.push(Tone::Text);
            for console in self.consoles_holding(&key) {
                lines.push(format!("{} {}", self.cw("warning_mark"), fill(&self.cw("hotkey_conflict"), &[("console", &console), ("key", &key)])));
                tones.push(Tone::Attention);
            }
        } else {
            lines.push(self.cw("no_key"));
            tones.push(Tone::Text);
        }
        Pane { sentence, lines, muted: vec![self.cw("hotkey_muted")], tones, ..Pane::default() }
    }

    /// Left or Right on a row of Controllers' own.
    pub(crate) fn controller_step(&mut self, entry: &Value, by: i32) -> Option<Outcome> {
        if let Some(i) = entry["pad"].as_u64() {
            let pad = self.seats.pads.get(i as usize)?.clone();
            let count = self.seats_shown().max(pad.player) + 1;
            let player = (pad.player as i64 + i64::from(by)).rem_euclid(count as i64) as usize;
            return Some(Outcome::Seat(Ask::Move { key: pad.key, player }));
        }
        if entry["pconsole"] == true {
            let consoles: Vec<String> = self.pad_consoles().iter().map(|c| c.console.clone()).collect();
            let at = self.pad_console().and_then(|c| consoles.iter().position(|x| *x == c)).unwrap_or(0);
            self.stepped_console = consoles.get((at as i64 + i64::from(by)).rem_euclid(consoles.len().max(1) as i64) as usize).cloned();
            return Some(Outcome::Shown);
        }
        None
    }

    /// A on a row of Controllers' own.
    pub(crate) fn controller_choose(&mut self, entry: &Value, index: usize) -> Option<Outcome> {
        let label = self.controller_row(entry).map(|r| r.label).unwrap_or_default();
        match entry["controllers"].as_str() {
            Some("players") => return Some(self.open(Page::Players, label, index)),
            Some("bindings") => return Some(self.open_bindings(None, false, label, index)),
            Some("keys") => return Some(self.open_bindings(None, true, label, index)),
            Some("profiles") if self.pad_console().is_some() => return Some(self.open(Page::Profiles, label, index)),
            Some("hotkeys") => return Some(self.open(Page::Hotkeys, label, index)),
            Some("reset") => return Some(Outcome::Bindings(self.read.bindings.reset_all())),
            Some(_) => return Some(Outcome::Nothing),
            None => {}
        }
        if entry.get("pad").is_some() || entry["pconsole"] == true {
            return self.controller_step(entry, 1);
        }
        if let Some(n) = entry["kept"].as_u64() {
            return Some(Outcome::Seat(Ask::Forget { player: n as usize }));
        }
        if let Some(profile) = entry["profile"].as_str() {
            let console = self.pad_console()?;
            return Some(self.open(Page::Profile(console, profile.to_owned()), profile.to_owned(), index));
        }
        if entry["newprofile"] == true {
            let console = self.pad_console()?;
            let from = self.read.bindings.console_default(&console);
            let prompt = fill(&self.cw("new_prompt"), &[("console", &self.console_name(&console))]);
            return Some(Outcome::Type { title: self.cw("new_title"), prompt, start: String::new(), typed: Typed::NewProfile { console, from } });
        }
        if let Some((console, profile)) = self.profile_page() {
            if entry["pbindings"] == true {
                return Some(self.open_bindings(Some((console, profile)), false, self.cw("bindings_title"), index));
            }
            if entry["pdefault"] == true {
                return Some(Outcome::Bindings(vec![self.read.bindings.choose_default(&console, &profile)]));
            }
            if *profile == self.read.bindings.default_profile {
                return Some(Outcome::Nothing);
            }
            if entry["prename"] == true {
                let prompt = fill(&self.cw("rename_prompt"), &[("profile", &profile)]);
                return Some(Outcome::Type { title: self.cw("rename_label"), prompt, start: profile.clone(), typed: Typed::Rename { console, profile } });
            }
            if entry["pdelete"] == true {
                let file = self.read.bindings.delete(&console, &profile);
                self.back();
                return Some(Outcome::Bindings(vec![file]));
            }
        }
        if let Some(action) = entry["hotkey"].as_str() {
            self.listening = if self.listening.as_deref() == Some(action) { None } else { Some(action.to_owned()) };
            return Some(Outcome::Shown);
        }
        None
    }

    /// A typed answer taken: a new profile made, or one renamed and its page following it.
    pub fn typed(&mut self, typed: &Typed, words: &str) -> Outcome {
        let words = words.trim();
        if words.is_empty() {
            return Outcome::Nothing;
        }
        match typed {
            Typed::NewProfile { console, from } => Outcome::Bindings(vec![self.read.bindings.create(console, words, from)]),
            Typed::Rename { console, profile } => {
                let file = self.read.bindings.rename(console, profile, words);
                if let Some(last) = self.pages.last_mut() {
                    if last.0 == Page::Profile(console.clone(), profile.clone()) {
                        *last = (Page::Profile(console.clone(), words.to_owned()), words.to_owned(), last.2);
                    }
                }
                Outcome::Bindings(vec![file])
            }
        }
    }

    /// The bindings page opened on a console's profile, or on the pad console's own, its first control focused.
    fn open_bindings(&mut self, on: Option<(String, String)>, keys: bool, label: String, index: usize) -> Outcome {
        let Some((console, profile)) = on.or_else(|| self.pad_console().map(|c| (c.clone(), self.read.bindings.console_default(&c)))) else {
            return Outcome::Nothing;
        };
        self.binding = Some(Binding { console, profile, player: 1, focus: 0, capturing: false, testing: false, keys });
        let title = if keys { self.cw("keys_title") } else { label };
        self.open(Page::Bindings, title, index)
    }

    /// The bindings page closed with its page, or a listening row's press let go: whether B was taken here.
    pub(crate) fn controller_back(&mut self) -> Option<bool> {
        if self.listening.take().is_some() {
            return Some(true);
        }
        let b = self.binding.as_mut()?;
        if b.capturing || b.testing {
            b.capturing = false;
            b.testing = false;
            self.held.clear();
            self.pushes.clear();
            self.keys_held.clear();
            return Some(true);
        }
        None
    }

    /// The help bar the bindings page and Hotkeys take.
    pub(crate) fn controller_help(&self) -> Option<crate::help::Screen> {
        use crate::help::Screen;
        if self.listening.is_some() {
            return Some(Screen::SettingsListening);
        }
        if matches!(self.page(), Some(Page::Hotkeys)) {
            return Some(Screen::SettingsHotkeys);
        }
        let b = self.binding.as_ref().filter(|_| matches!(self.page(), Some(Page::Bindings)))?;
        Some(if b.capturing {
            Screen::SettingsListening
        } else if b.testing {
            Screen::SettingsTesting
        } else if b.keys {
            Screen::SettingsKeys
        } else if self.players_of(&b.console) > 1 {
            Screen::SettingsBindings
        } else {
            Screen::SettingsBindingsOne
        })
    }

    /// The focused control of the bindings page.
    fn focused_control(&self) -> Option<String> {
        let b = self.binding.as_ref()?;
        self.controls_of(&b.console).get(b.focus).cloned()
    }

    /// A button's pad binding in words; L2 and R2 are always their trigger as well (EmuSen_Input.md §7.3).
    fn bound_pad(&self, b: &Binding, control: &str) -> Option<String> {
        if !controllers::is_button(control) {
            return None;
        }
        let choice = self.choice(&b.console, b.player, &b.profile);
        let bound = self.read.bindings.bound(&b.console, &choice, control).map(|w| pad_words(&w));
        let trigger = self.description_at(&["controllers", "triggers", control]).as_str().map(|w| self.cw(w));
        match (bound, trigger) {
            (Some(b), Some(t)) => Some(fill(&self.cw("trigger_or"), &[("pad", &b), ("trigger", &t)])),
            (b, t) => b.or(t),
        }
    }

    fn binding_pane(&self, b: &Binding) -> Pane {
        let console = self.console_name(&b.console);
        if b.testing {
            let held: Vec<String> = self.held.iter().map(|c| self.control_words(c)).collect();
            let line = if held.is_empty() { self.cw("press_anything") } else { fill(&self.cw("held"), &[("list", &held.join(", "))]) };
            let player = fill(&self.cw("player"), &[("n", &b.player.to_string())]);
            return Pane {
                preview: Preview::Heading,
                caption: self.cw("testing"),
                sentence: format!("{console} · {player}"),
                lines: vec![line],
                muted: vec![self.cw("testing_muted")],
                tones: vec![Tone::Muted, Tone::Text],
                ..Pane::default()
            };
        }
        let Some(control) = self.focused_control() else {
            return Pane { sentence: fill(&self.cw("no_controls"), &[("console", &console)]), ..Pane::default() };
        };
        let name = self.control_words(&control);
        let mut lines = Vec::new();
        let mut tones = vec![Tone::Muted];
        let mut muted = Vec::new();
        let sentence = if b.keys {
            if b.capturing {
                lines.push(format!("{} {}", self.cw("listening_mark"), fill(&self.cw("listen_key"), &[("control", &name)])));
                tones.push(Tone::Attention);
            } else {
                lines.push(self.read.bindings.key(&b.console, &self.file_control(&control)).map_or_else(|| self.cw("no_key"), |k| fill(&self.cw("bound_key"), &[("key", &k)])));
                tones.push(Tone::Text);
                muted.push(self.cw("keys_muted"));
            }
            format!("{console} · {} · {name}", self.cw("keyboard_tab"))
        } else {
            let player = (b.player > 1).then(|| fill(&self.cw("player"), &[("n", &b.player.to_string())]));
            let profile = (b.profile != self.read.bindings.default_profile).then(|| b.profile.clone());
            let place: Vec<String> = [Some(console.clone()), profile, player, Some(name.clone())].into_iter().flatten().collect();
            if !controllers::is_button(&control) {
                lines.push(self.cw("stick_line"));
                tones.push(Tone::Text);
            } else if b.capturing {
                lines.push(format!("{} {}", self.cw("listening_mark"), fill(&self.cw("listen_pad"), &[("control", &name)])));
                tones.push(Tone::Attention);
            } else {
                let words = match self.bound_pad(b, &control) {
                    Some(pad) if pad.ends_with(&self.cw("left_trigger")) || pad.ends_with(&self.cw("right_trigger")) => fill(&self.cw("bound_trigger"), &[("pad", &pad)]),
                    Some(pad) => fill(&self.cw("bound_button"), &[("pad", &pad)]),
                    None => self.cw("not_bound"),
                };
                lines.push(words);
                tones.push(Tone::Text);
            }
            if !b.capturing {
                muted.push(self.cw("pad_muted"));
            }
            place.join(" · ")
        };
        if let Some(conflict) = self.conflict_words(&b.console) {
            lines.push(format!("{} {conflict}", self.cw("warning_mark")));
            tones.push(Tone::Attention);
        }
        Pane { preview: Preview::Heading, caption: name, sentence, lines, muted, tones, ..Pane::default() }
    }

    /// The word the binding files give a control the drawing names by the core's button or a stick's way (Endymion's
    /// `bindings_controls`); the control's own name where Endymion names no such control.
    fn file_control(&self, control: &str) -> String {
        let words = self.chooser.as_ref().map_or(&[][..], |c| c.controls());
        let found = match controllers::control_number(control) {
            Some(n) => words.iter().find(|w| w.button == Some(n)),
            None => controllers::stick_way(control).and_then(|(axis, toward)| words.iter().find(|w| w.axis == Some(axis) && w.toward == toward)),
        };
        found.map_or_else(|| control.to_owned(), |w| w.word.clone())
    }

    fn conflict_words(&self, console: &str) -> Option<String> {
        let controls: Vec<(String, String)> = self.controls_of(console).into_iter().map(|c| (self.file_control(&c), self.control_words(&c))).collect();
        let found = self.read.bindings.conflicts(console, &controls, &|a| self.action_words(a));
        if found.is_empty() {
            return None;
        }
        let list: Vec<String> = found.iter().map(|(k, owners)| fill(&self.cw("clash"), &[("key", k), ("owners", &owners.join(&self.cw("and")))])).collect();
        Some(fill(&self.cw("conflict"), &[("console", &self.console_name(console)), ("list", &list.join("; "))]))
    }

    /// What the bindings page's drawing shows: the console's controller for the player, its controls, and each one's
    /// binding on its label.
    pub fn drawn(&self) -> Option<Drawn> {
        let b = self.binding.as_ref().filter(|_| matches!(self.page(), Some(Page::Bindings)))?;
        let console = self.read.consoles.console(&b.console)?;
        let controls = controllers::controls(&console.controllers);
        let id = controllers::controller_of(&console.controllers, b.player);
        let drawing = id.and_then(controllers::drawing).unwrap_or_default().to_owned();
        let choice = self.choice(&b.console, b.player, &b.profile);
        let texts = controls
            .iter()
            .map(|c| {
                if b.keys {
                    self.read.bindings.key(&b.console, &self.file_control(c)).unwrap_or_default()
                } else if controllers::is_button(c) {
                    self.read.bindings.bound(&b.console, &choice, c).map(|w| pad_words(&w)).unwrap_or_default()
                } else {
                    String::new()
                }
            })
            .collect();
        let first = if b.keys { self.cw("keyboard_tab") } else { fill(&self.cw("player"), &[("n", &b.player.to_string())]) };
        let consoles: Vec<&Console> = self.pad_consoles();
        let at = consoles.iter().position(|c| c.console == b.console).unwrap_or(0);
        let mut tabs = vec![first];
        tabs.extend(consoles[at..].iter().chain(consoles[..at].iter()).map(|c| c.name.clone()));
        Some(Drawn {
            drawing,
            controls,
            texts,
            focus: if b.testing { -1 } else { b.focus as i32 },
            held: self.held.clone(),
            pushes: if b.testing { self.pushes.clone() } else { Vec::new() },
            tabs,
            caption: self.cw("caption"),
            name: String::new(),
        })
    }

    /// Whether the bindings page is shown.
    pub fn binding_open(&self) -> bool {
        self.binding.is_some() && matches!(self.page(), Some(Page::Bindings))
    }

    /// A press newly down while the pad's page listens, as a pad's buttons and trigger bits: the lowest button, or a
    /// trigger.
    pub fn heard_down(&mut self, down: u32) -> Outcome {
        if down & LEFT_TRIGGER_BIT != 0 {
            return self.heard_pad(None, Some("L2"));
        }
        if down & RIGHT_TRIGGER_BIT != 0 {
            return self.heard_pad(None, Some("R2"));
        }
        self.heard_pad(Some(down.trailing_zeros() as i32), None)
    }

    /// What the tester lights: a control whose key is held on the keyboard, and the page's player's pad as the game
    /// would hear it, a canonical button by its bound gamepad button, L2 and R2 by their triggers past half, and a
    /// stick's directions past Endymion's stick deadzone.
    pub fn held_from(&self, seated: &Seated) -> Vec<String> {
        let Some(b) = self.binding.as_ref().filter(|b| b.testing) else { return Vec::new() };
        let pad = self.tester_pad(seated, b.player);
        let bound = match (pad, self.chooser.as_ref()) {
            (Some(_), Some(chooser)) => chooser.bound(&b.console, b.player),
            _ => [-1; 16],
        };
        let mut held = Vec::new();
        for control in self.controls_of(&b.console) {
            let by_key = self.read.bindings.key(&b.console, &self.file_control(&control)).is_some_and(|k| self.keys_held.iter().any(|h| h.eq_ignore_ascii_case(&k)));
            let by_pad = pad.is_some_and(|pad| {
                if let Some(n) = controllers::control_number(&control) {
                    let by_button = bound.get(n as usize).is_some_and(|&sdl| sdl >= 0 && pad.buttons & (1 << sdl) != 0);
                    by_button || (control == "L2" && pad.buttons & LEFT_TRIGGER_BIT != 0) || (control == "R2" && pad.buttons & RIGHT_TRIGGER_BIT != 0)
                } else {
                    controllers::stick_held(&control, &pad.axes, seated.deadzone)
                }
            });
            if by_key || by_pad {
                held.push(control);
            }
        }
        held
    }

    /// How far the tester sees each control pushed, in the controls' order: a stick's directions by the page's player's
    /// stick, so the drawing's knob stands where the stick does; every other control 0.
    pub fn pushes_from(&self, seated: &Seated) -> Vec<f64> {
        let Some(b) = self.binding.as_ref().filter(|b| b.testing) else { return Vec::new() };
        let pad = self.tester_pad(seated, b.player);
        self.controls_of(&b.console).iter().map(|c| pad.map_or(0.0, |p| controllers::stick_push(c, &p.axes))).collect()
    }

    /// The pad seated as a player, if one is connected there.
    fn tester_pad<'a>(&self, seated: &'a Seated, player: u32) -> Option<&'a Connected> {
        seated.pads.iter().find(|p| p.player == player as usize)
    }

    /// A key down or up on the keyboard while the tester runs, by the binding files' word for it: what is held then.
    pub fn tester_key(&mut self, word: &str, down: bool) -> bool {
        if !self.testing() {
            return false;
        }
        self.keys_held.retain(|k| !k.eq_ignore_ascii_case(word));
        if down {
            self.keys_held.push(word.to_owned());
        }
        let seated = self.seats.clone();
        self.held = self.held_from(&seated);
        true
    }

    /// A direction on the bindings page (`dx`, `dy`, one of the four): the nearest declared control that way on the
    /// drawing, as Mistress's page moved; false, and the focus kept, where nothing lies that way.
    pub fn binding_move(&mut self, dx: i32, dy: i32) -> bool {
        let Some(b) = self.binding.as_ref().filter(|b| !b.capturing && !b.testing) else { return false };
        let controls = self.controls_of(&b.console);
        let file = self.read.consoles.console(&b.console).and_then(|c| controllers::controller_of(&c.controllers, b.player)).and_then(controllers::drawing);
        let (drawing, _) = emusen_kit_core::controller::drawing_for(file, &controls);
        let Some(next) = emusen_kit_core::controller::toward(&drawing, &controls, b.focus, f64::from(dx), f64::from(dy)) else { return false };
        if let Some(b) = self.binding.as_mut() {
            b.focus = next;
        }
        true
    }

    /// A control focused by the pointer.
    pub fn binding_focus(&mut self, index: usize) -> bool {
        let count = self.binding.as_ref().map(|b| self.controls_of(&b.console).len()).unwrap_or(0);
        match self.binding.as_mut().filter(|b| !b.capturing && !b.testing && index < count) {
            Some(b) => {
                b.focus = index;
                true
            }
            None => false,
        }
    }

    /// The shoulders on the bindings page: the console before or after, its own default profile, its first control.
    pub fn binding_shoulder(&mut self, by: i32) -> bool {
        let consoles: Vec<String> = self.pad_consoles().iter().map(|c| c.console.clone()).collect();
        let Some(b) = self.binding.as_ref().filter(|b| !b.capturing && !b.testing) else { return false };
        let at = consoles.iter().position(|c| *c == b.console).unwrap_or(0);
        let Some(next) = consoles.get((at as i64 + i64::from(by)).rem_euclid(consoles.len().max(1) as i64) as usize).cloned() else { return false };
        let profile = self.read.bindings.console_default(&next);
        let players = self.players_of(&next);
        let b = self.binding.as_mut().expect("a bindings page");
        *b = Binding { player: b.player.min(players), profile, console: next, focus: 0, capturing: false, testing: false, keys: b.keys };
        true
    }

    /// A on the bindings page: listening for a press, on the keyboard's page for any control, on the pad's for a button.
    pub fn binding_accept(&mut self) -> bool {
        let Some(control) = self.focused_control() else { return false };
        let Some(b) = self.binding.as_mut().filter(|b| !b.testing) else { return false };
        if b.keys || controllers::is_button(&control) {
            b.capturing = true;
            return true;
        }
        false
    }

    /// One of the page's declared actions: Y resets the focused control, X the next player (the pad's page) or clears
    /// the key (the keyboard's), Select the tester.
    pub fn binding_act(&mut self, action: &str) -> Outcome {
        let Some(control) = self.focused_control() else { return Outcome::Nothing };
        let Some(b) = self.binding.clone().filter(|b| !b.capturing) else { return Outcome::Nothing };
        match action {
            crate::help::RESET if b.keys => {
                let word = self.file_control(&control);
                Outcome::Bindings(vec![self.read.bindings.reset_key(&b.console, &word)])
            }
            crate::help::RESET if controllers::is_button(&control) => {
                let choice = self.choice(&b.console, b.player, &b.profile);
                Outcome::Bindings(vec![self.read.bindings.reset_button(&b.console, b.player, &choice, &control)])
            }
            crate::help::CLEAR if b.keys => {
                let word = self.file_control(&control);
                Outcome::Bindings(vec![self.read.bindings.clear_key(&b.console, &word)])
            }
            crate::help::PLAYER if !b.keys => {
                let players = self.players_of(&b.console);
                if let Some(page) = self.binding.as_mut() {
                    page.player = page.player % players + 1;
                }
                Outcome::Shown
            }
            crate::help::TEST if !b.keys => {
                if let Some(page) = self.binding.as_mut() {
                    page.testing = !page.testing;
                }
                self.held.clear();
                self.pushes.clear();
                self.keys_held.clear();
                Outcome::Shown
            }
            _ => Outcome::Nothing,
        }
    }

    /// The gamepad button heard while the pad's page listens, by SDL's number, or a trigger (L2 or R2 for the left or
    /// right) pulled: the focused button bound to it in the profile's map for the player.
    pub fn heard_pad(&mut self, pad: Option<i32>, trigger: Option<&str>) -> Outcome {
        let Some(control) = self.focused_control() else { return Outcome::Nothing };
        let Some(b) = self.binding.clone().filter(|b| b.capturing && !b.keys) else { return Outcome::Nothing };
        if let Some(page) = self.binding.as_mut() {
            page.capturing = false;
        }
        let choice = self.choice(&b.console, b.player, &b.profile);
        match (pad, trigger) {
            // A trigger pulled for L2 or R2 binds it alone, as the trigger is always read (EmuSen_Input.md §7.3).
            (_, Some(t)) if t == control => Outcome::Bindings(vec![self.read.bindings.unbind(&b.console, b.player, &choice, &control)]),
            (Some(button), _) => Outcome::Bindings(vec![self.read.bindings.rebind(&b.console, b.player, &choice, &control, button)]),
            _ => Outcome::Shown,
        }
    }

    /// The key heard while the keyboard's page or a hotkey listens, in the binding files' word for it; none for Escape,
    /// which cancels.
    pub fn heard_key(&mut self, key: Option<&str>) -> Outcome {
        if let Some(action) = self.listening.take() {
            return match key {
                Some(k) => Outcome::Bindings(self.read.bindings.rebind_hotkey(&action, k)),
                None => Outcome::Shown,
            };
        }
        let Some(control) = self.focused_control() else { return Outcome::Nothing };
        let Some(b) = self.binding.clone().filter(|b| b.capturing && b.keys) else { return Outcome::Nothing };
        if let Some(page) = self.binding.as_mut() {
            page.capturing = false;
        }
        match key {
            Some(k) => {
                let word = self.file_control(&control);
                Outcome::Bindings(self.read.bindings.rebind_key(&b.console, &word, k))
            }
            None => Outcome::Shown,
        }
    }

    /// X on a hotkey's row: its key cleared.
    pub fn clear_hotkey(&mut self, index: usize) -> Outcome {
        let entries = self.controller_entries(&Page::Hotkeys).unwrap_or_default();
        match entries.get(index).and_then(|e| e["hotkey"].as_str()) {
            Some(action) if matches!(self.page(), Some(Page::Hotkeys)) => Outcome::Bindings(vec![self.read.bindings.clear_hotkey(action)]),
            _ => Outcome::Nothing,
        }
    }

    /// Whether a press is listened for, so the keyboard's keys are the page's.
    pub fn listening(&self) -> bool {
        self.listening.is_some() || self.binding.as_ref().is_some_and(|b| b.capturing && matches!(self.page(), Some(Page::Bindings)))
    }

    /// Whether the pad's page listens for a gamepad's button.
    pub fn listening_pad(&self) -> bool {
        self.binding.as_ref().is_some_and(|b| b.capturing && !b.keys && matches!(self.page(), Some(Page::Bindings)))
    }

    /// How long B is held to leave the tester, since a short press of B is a button being tried.
    pub fn tester_hold_ms(&self) -> i32 {
        self.description_at(&["controllers", "tester_hold_ms"]).as_i64().map_or(0, |ms| ms as i32)
    }

    /// Whether the tester runs.
    pub fn testing(&self) -> bool {
        self.binding.as_ref().is_some_and(|b| b.testing && matches!(self.page(), Some(Page::Bindings)))
    }

    /// The bindings models saved: the files a change touched.
    pub fn bindings_files(files: &[File]) -> Vec<u32> {
        files.iter().map(|f| match f {
            File::Keys => emusen_platform_client::bindings::consts::KEYS,
            File::Gamepad => emusen_platform_client::bindings::consts::GAMEPAD,
            File::Profiles => emusen_platform_client::bindings::consts::PROFILES,
            File::Hotkeys => emusen_platform_client::bindings::consts::HOTKEYS,
        }).collect()
    }

    /// The model of a file, as it stands.
    pub fn bindings_model(bindings: &Bindings, file: File) -> &Value {
        match file {
            File::Keys => &bindings.keys,
            File::Gamepad => &bindings.gamepad,
            File::Profiles => &bindings.profiles,
            File::Hotkeys => &bindings.hotkeys,
        }
    }
}

/// A row of the Pause Card's Game Options ▸ Controllers (EmuSen_Interface.md §20.3), with its pane's words, as the card
/// draws it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GameRow {
    pub key: String,
    pub label: String,
    pub value: String,
    pub steps: bool,
    pub opens: bool,
    pub dimmed: bool,
    pub sentence: String,
    pub muted: String,
}

impl GameRow {
    pub fn to_json(&self) -> Value {
        json!({ "key": self.key, "label": self.label, "value": self.value, "steps": self.steps, "opens": self.opens,
                "dimmed": self.dimmed, "sentence": self.sentence, "muted": self.muted })
    }

    /// The rows as `SettingsModel.cardControllers` gives them.
    pub fn list(v: &Value) -> Vec<GameRow> {
        let text = |r: &Value, k: &str| r[k].as_str().unwrap_or_default().to_owned();
        v.as_array()
            .into_iter()
            .flatten()
            .map(|r| GameRow {
                key: text(r, "key"),
                label: text(r, "label"),
                value: text(r, "value"),
                steps: r["steps"] == true,
                opens: r["opens"] == true,
                dimmed: r["dimmed"] == true,
                sentence: text(r, "sentence"),
                muted: text(r, "muted"),
            })
            .collect()
    }
}

/// The card's Bindings: the door that opens Settings' drawing page over the card.
pub const CARD_BINDINGS: &str = "cardBindings";
/// The card's Swap Players, which the card's own Swap carries out.
pub const SWAP: &str = "gswap";

impl SettingsView {
    fn card_word(&self, name: &str) -> String {
        self.description_at(&["controllers", "card", name]).as_str().unwrap_or_default().to_owned()
    }

    /// The game's console and file while one runs.
    fn running_game(&self) -> Option<(String, String)> {
        let game = self.facts["game"].as_str().filter(|g| !g.is_empty())?.to_owned();
        Some((self.running_console()?, game))
    }

    /// The profile the running game plays with, as Endymion resolves it.
    fn game_profile(&self, console: &str, game: &str) -> String {
        let b = &self.read.bindings;
        match &self.chooser {
            Some(c) => c.choose(&b.gamepad, &b.profiles, console, 1, None, Some(game)).profile,
            None => b.console_default(console),
        }
    }

    /// The card's page: each player the console's ports hold (two at least) with the pad they play on, the game's
    /// profile, the drawing, and the swap (Mistress's `GameControllerRows`).
    pub fn game_rows(&self) -> Vec<GameRow> {
        let mut rows = Vec::new();
        let running = self.running_game();
        let players = running.as_ref().map_or(1, |(c, _)| self.players_of(c)).max(2);
        let profile = running.as_ref().map(|(c, g)| self.game_profile(c, g));
        let profile_words = profile.as_ref().map(|p| fill(&self.card_word("seat_profile"), &[("profile", p)])).unwrap_or_default();
        for p in 1..=players as usize {
            let n = p.to_string();
            let seat = self.seats.players.iter().find(|s| s.player == p);
            let value = match seat {
                Some(s) if s.kept => fill(&self.card_word("away"), &[("pad", s.pad.as_deref().unwrap_or_default())]),
                Some(s) => s.pad.clone().unwrap_or_else(|| self.cw("none")),
                None => self.cw("none"),
            };
            let sentence = match seat.and_then(|s| s.pad.as_ref()).filter(|_| !seat.is_some_and(|s| s.kept)) {
                Some(pad) => fill(&self.card_word("seat_line"), &[("n", &n), ("pad", pad), ("profile", &profile_words)]),
                None => fill(&self.card_word("seat_nobody"), &[("n", &n)]),
            };
            rows.push(GameRow {
                key: format!("seat{p}"),
                label: fill(&self.card_word("player_label"), &[("n", &n)]),
                value,
                steps: true,
                sentence,
                muted: self.card_word("seat_muted"),
                ..GameRow::default()
            });
        }
        if let (Some((console, _)), Some(profile)) = (&running, &profile) {
            let names = [("console", self.console_name(console)), ("profile", profile.clone())];
            let names: Vec<(&str, &str)> = names.iter().map(|(k, v)| (*k, v.as_str())).collect();
            rows.push(GameRow {
                key: "gprofile".into(),
                label: self.card_word("profile_label"),
                value: profile.clone(),
                steps: true,
                sentence: fill(&self.card_word("profile_line"), &names),
                muted: self.card_word("profile_muted"),
                ..GameRow::default()
            });
            rows.push(GameRow {
                key: "gbindings".into(),
                label: self.card_word("bindings_label"),
                opens: true,
                sentence: fill(&self.card_word("bindings_line"), &names),
                muted: self.card_word("bindings_muted"),
                ..GameRow::default()
            });
        }
        let second = self.seats.players.iter().any(|s| s.player == 2 && s.pad.is_some() && !s.kept);
        rows.push(GameRow { key: SWAP.into(), label: self.card_word("swap_label"), dimmed: !second, sentence: self.card_word("swap_hint"), ..GameRow::default() });
        rows
    }

    /// Left or Right (`by` ±1), or A (`by` 0), on a row of the card's page.
    pub fn game_act(&mut self, index: usize, by: i32) -> Outcome {
        let Some(row) = self.game_rows().into_iter().nth(index) else { return Outcome::Nothing };
        if let Some(p) = row.key.strip_prefix("seat").and_then(|p| p.parse::<usize>().ok()) {
            // None, then each pad connected; Left and Right seat the next one as this player.
            let pads = self.seats.pads.clone();
            let at = pads.iter().position(|c| c.player == p).map_or(0, |i| i + 1);
            let next = (at as i64 + i64::from(if by == 0 { 1 } else { by })).rem_euclid(pads.len() as i64 + 1) as usize;
            return match next {
                0 => pads.iter().find(|c| c.player == p).map_or(Outcome::Nothing, |c| Outcome::Seat(Ask::Move { key: c.key, player: 0 })),
                n => Outcome::Seat(Ask::Move { key: pads[n - 1].key, player: p }),
            };
        }
        let Some((console, game)) = self.running_game() else { return Outcome::Nothing };
        match row.key.as_str() {
            "gprofile" => {
                let profiles = self.read.bindings.profiles(&console);
                let now = self.game_profile(&console, &game);
                let at = profiles.iter().position(|p| *p == now).unwrap_or(0);
                let next = profiles[(at as i64 + i64::from(if by == 0 { 1 } else { by })).rem_euclid(profiles.len() as i64) as usize].clone();
                let own = (next != self.read.bindings.console_default(&console)).then_some(next.as_str());
                let Some(key) = self.chooser.as_ref().map(|c| c.game_key(&game)) else { return Outcome::Nothing };
                Outcome::Bindings(vec![self.read.bindings.set_game_profile(&console, &key, own)])
            }
            "gbindings" if by == 0 => {
                self.open_card_bindings(&console);
                Outcome::Door(CARD_BINDINGS.into())
            }
            SWAP if by == 0 && !row.dimmed => Outcome::Door(SWAP.into()),
            _ => Outcome::Nothing,
        }
    }

    /// Settings' drawing page for a console as the only page, from the card: B closes Settings back to the card.
    pub fn open_card_bindings(&mut self, console: &str) {
        self.choose_section(0);
        let profile = self.running_game().map_or_else(|| self.read.bindings.console_default(console), |(c, g)| self.game_profile(&c, &g));
        self.binding = Some(Binding { console: console.to_owned(), profile, player: 1, focus: 0, capturing: false, testing: false, keys: false });
        self.pages.push((Page::Bindings, self.cw("bindings_title"), 0));
        self.from_card = true;
    }
}
