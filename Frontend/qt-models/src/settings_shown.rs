//! What a theme's Settings reads beyond a row's words (EmuSen_Qt.md §3.2, `Settings`).

use crate::{cards, clients, consoles, firmware, schema, words};
use crate::settings::{Page, Pane, Row, SettingsView};
use serde_json::Value;

/// Planet shade colours matching Eclipse's PlanetInk (0..=7), with Diamond's #4A7890 as fallback (boards #27-#29, #57).
const PLANET_SHADES: [&str; 8] = ["#5E6272", "#7A5416", "#6B1E16", "#2C4670", "#1C4A36", "#274A1E", "#4A7890", "#162C6B"];
const CONSOLES_GLOW_FALLBACK: &str = "#4A7890";

/// Maps a console's core to its Eclipse planet mark: -2 unsupported, -1 libretro checkerboard, 0..=7 PlanetInk triple.
pub fn planet_of(engine_name: &str, libretro: bool, runs: bool) -> i32 {
    if !runs {
        return -2;
    }
    if libretro {
        return -1;
    }
    match engine_name.to_ascii_lowercase().as_str() {
        "moon" => 0,
        "venus" => 1,
        "mars" => 2,
        "mercury" => 3,
        "endou" => 4,
        "nephrite" => 5,
        "diamond" => 6,
        "sapphire" => 7,
        other => (other.bytes().fold(2166136261u32, |h, b| h.wrapping_mul(16777619) ^ u32::from(b)) % 8) as i32,
    }
}

/// The page glow colour for a planet mark, falling back to #4A7890 when the console has no planet.
pub fn planet_shade(planet: i32) -> &'static str {
    usize::try_from(planet).ok().and_then(|i| PLANET_SHADES.get(i)).copied().unwrap_or(CONSOLES_GLOW_FALLBACK)
}

/// Joins core names in plain English ("A", "A and B", "A, B and C") as board #57 writes them.
fn join_and(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [a] => a.clone(),
        [a, b] => format!("{a} and {b}"),
        many => format!("{} and {}", many[..many.len() - 1].join(", "), many[many.len() - 1]),
    }
}

/// Eclipse-specific board projection of one Settings row, keeping SettingsModel's own fields intact for the reference theme.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BoardRow {
    pub label: String,
    pub about: String,
    pub value: String,
    pub form: Form,
    pub planet: i32,
    pub subtitle: String,
    pub chip: String,
    pub chip_attention: bool,
    pub runs: bool,
}

/// Eclipse-specific board projection of the page header, glow, About panel, core card and firmware file list.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BoardAside {
    pub heading: String,
    pub about_heading: String,
    pub sentence: String,
    pub lines: Vec<String>,
    pub muted: Vec<String>,
    pub consoles_subtitle: String,
    pub page_glow: String,
    pub card_planet: i32,
    pub card_title: String,
    pub card_subtitle: String,
    pub firmware_files: Vec<firmware::FileStatus>,
    pub wrap_about: bool,
    pub cards_help: bool,
    pub box_menu: bool,
}

/// What a row shows beyond its words: its about line, a switch's state (none for a row of another kind), a console's tile.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RowFacets {
    pub about: String,
    pub switch: Option<bool>,
    pub tile: Option<ConsoleTile>,
}

/// How a theme draws a row's value, as the Settings boards do (EmuSen_Qt.md §3.4.11): between chevrons, a switch, a door's chevron, or words alone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Form {
    Choice,
    Switch,
    Door,
    #[default]
    Fact,
}

impl Form {
    /// A row's form: a switch; a stepped value or a slider a choice; a page, a question or an action with no value a door; else a fact.
    pub fn of(row: &crate::settings::Row, switch: Option<bool>) -> Form {
        use emusen_kit_core::settings::RowKind;
        match row.kind {
            _ if switch.is_some() => Form::Switch,
            RowKind::Steps | RowKind::Slider => Form::Choice,
            RowKind::Opens | RowKind::Asks => Form::Door,
            RowKind::Action if row.value.is_empty() => Form::Door,
            _ => Form::Fact,
        }
    }

    /// Its name as a theme reads it in `Settings.rows`' `form`.
    pub fn word(self) -> &'static str {
        match self {
            Form::Choice => "choice",
            Form::Switch => "switch",
            Form::Door => "door",
            Form::Fact => "fact",
        }
    }
}

/// A console row's tile: its stored name, its system key, its games and the engine that runs it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConsoleTile {
    pub console: String,
    pub system: String,
    pub games: usize,
    pub engine: String,
}

impl Page {
    /// The page's kind and what it is of, joined by `:`: `console:SNES`, `group:Scraping`, `players`.
    pub fn key(&self) -> String {
        let (kind, of): (&str, Vec<&str>) = match self {
            Page::Group(g) => ("group", vec![g]),
            Page::Console(c) => ("console", vec![c]),
            Page::Engine(c) => ("engine", vec![c]),
            Page::Firmware(c) => ("firmware", vec![c]),
            Page::Cards(c) => ("cards", vec![c]),
            Page::Drawer(c, m) => ("drawer", vec![c, m]),
            Page::Saves(c, m) => ("saves", vec![c, m]),
            Page::Save(c, m, s) => ("save", vec![c, m, s]),
            Page::Delete(c, m, s) => ("delete", vec![c, m, s]),
            Page::Players => ("players", vec![]),
            Page::Profiles => ("profiles", vec![]),
            Page::Profile(a, b) => ("profile", vec![a, b]),
            Page::Hotkeys => ("hotkeys", vec![]),
            Page::Bindings => ("bindings", vec![]),
            Page::ShaderConsoles { levers: false } => ("shaderConsoles", vec![]),
            Page::ShaderConsoles { levers: true } => ("leverConsoles", vec![]),
            Page::Shaders(c) => ("shaders", vec![c]),
            Page::Levers(c) => ("levers", vec![c]),
            Page::Output => ("output", vec![]),
        };
        std::iter::once(kind).chain(of).collect::<Vec<_>>().join(":")
    }
}

impl SettingsView {
    /// The page shown: the innermost page's key, else the section's id from the description (`display`, `consoles`).
    pub fn page_key(&self) -> String {
        match self.page() {
            Some(page) => page.key(),
            None => self.description_at(&["sections"]).get(self.section).and_then(|s| s["id"].as_str()).unwrap_or_default().to_owned(),
        }
    }

    /// Each row's facets beyond its words, in the rows' order, the description's entries read once.
    pub fn facets(&self) -> Vec<RowFacets> {
        self.entries().iter().map(|entry| RowFacets { about: self.about_of(entry), switch: self.switch_of(entry), tile: self.tile_of(entry) }).collect()
    }

    /// A row's short line under its name: the description's `about`, else its Galaxia row's, else its hint (R-20261123-31).
    fn about_of(&self, entry: &Value) -> String {
        let keyed = self.keyed(entry);
        let own = entry["about"].as_str().or_else(|| keyed.as_ref().and_then(|k| k.row["about"].as_str()));
        let hint = keyed.as_ref().map_or(&entry["hint"], |k| &k.row["hint"]).as_str().unwrap_or_default();
        self.fill(own.unwrap_or(hint), "")
    }

    /// A Galaxia row or a console's setting of the kind `switch`: on where every place it is stored for holds it on.
    fn switch_of(&self, entry: &Value) -> Option<bool> {
        if let Some(keyed) = self.keyed(entry) {
            return (keyed.row["kind"] == "switch").then(|| self.stored(&keyed).iter().all(|s| schema::is_on(s)));
        }
        let key = entry["schema"].as_str().filter(|_| !self.on_saves())?;
        let console = self.console()?;
        let row = console.row(key)?;
        (row.kind == "switch").then(|| schema::is_on(&console.value(key)))
    }

    /// A console row of the Consoles section.
    fn tile_of(&self, entry: &Value) -> Option<ConsoleTile> {
        let name = entry["console"].as_str().filter(|_| self.page().is_none())?;
        let c = self.read.consoles.console(name)?;
        Some(ConsoleTile {
            console: c.console.clone(),
            system: c.systems.first().cloned().unwrap_or_default(),
            games: self.games_of(c),
            engine: if c.runs() { self.read.consoles.engine_name(&c.engine()) } else { String::new() },
        })
    }

    /// Eclipse's board-exact row projections, aligned 1:1 with `rows` and `facets` (boards #27, #28, #57, #29).
    pub fn board_rows(&self, rows: &[Row], facets: &[RowFacets]) -> Vec<BoardRow> {
        let entries = self.entries();
        rows.iter()
            .zip(facets)
            .zip(entries.iter())
            .map(|((row, facet), entry)| self.board_row(row, facet, entry))
            .collect()
    }

    fn board_row(&self, row: &Row, facet: &RowFacets, entry: &Value) -> BoardRow {
        let mut out = BoardRow {
            label: row.label.clone(),
            about: facet.about.clone(),
            value: row.value.clone(),
            form: Form::of(row, facet.switch),
            planet: -3,
            subtitle: String::new(),
            chip: String::new(),
            chip_attention: false,
            runs: true,
        };
        if let Some(name) = entry["console"].as_str().filter(|_| self.page().is_none()) {
            if let Some(c) = self.read.consoles.console(name) {
                let engine = clients::engine_shown(&c.engine(), &self.read.consoles.cores);
                let games = words::games(self.games_of(c));
                out.runs = c.runs();
                out.planet = planet_of(&engine.name, engine.libretro, c.runs());
                out.subtitle = format!("{games} · {}", if c.runs() { engine.name.as_str() } else { "no core yet" });
                if !c.runs() {
                    out.chip = "Not supported yet".to_owned();
                } else if c.starts_empty && c.needs().iter().any(|n| n.missing()) {
                    out.chip = "Needs firmware".to_owned();
                    out.chip_attention = true;
                }
            }
            return out;
        }
        match self.page() {
            Some(Page::Players) => {
                if let Some(i) = entry["pad"].as_u64() {
                    let none = self.seats.pads.get(i as usize).is_some_and(|p| p.player == 0);
                    out.about = self.cw(if none { "pad_none" } else { "pad_seated" });
                } else if entry.get("kept").is_some() {
                    out.about = "Kept for this controller, which gets the number back when it reconnects.".to_owned();
                }
                return out;
            }
            Some(Page::Profiles) => {
                let console = self.pad_console();
                let name = console.as_deref().map(|c| self.console_name(c)).unwrap_or_default();
                if entry["pconsole"] == true {
                    out.about = self.cw("console_hint");
                } else if let Some(profile) = entry["profile"].as_str() {
                    out.about = if profile == self.read.bindings.default_profile {
                        self.cw("default_hint")
                    } else {
                        self.cw("own_hint").replace("{console}", &name)
                    };
                } else if entry["newprofile"] == true {
                    let chosen = console.as_deref().map(|c| self.read.bindings.console_default(c)).unwrap_or_else(|| self.read.bindings.default_profile.clone());
                    out.about = self.cw("new_hint").replace("{chosen}", &chosen).replace("{console}", &name);
                    out.value = "…".to_owned();
                }
                return out;
            }
            Some(Page::Profile(console, profile)) => {
                let name = self.console_name(console);
                let is_default = *profile == self.read.bindings.default_profile;
                if entry["pbindings"] == true {
                    out.about = self.cw("pbindings_hint").replace("{console}", &name);
                } else if entry["pdefault"] == true {
                    out.about = self.cw("pdefault_hint").replace("{console}", &name).replace("{profile}", profile);
                } else if entry["prename"] == true {
                    out.about = self.cw(if is_default { "rename_default" } else { "rename_hint" });
                    if !is_default {
                        out.value = "…".to_owned();
                    }
                } else if entry["pdelete"] == true {
                    out.about = self.cw(if is_default { "delete_default" } else { "delete_hint" }).replace("{profile}", profile);
                }
                return out;
            }
            Some(Page::Hotkeys) => {
                if let Some(action) = entry["hotkey"].as_str() {
                    let held = emusen_qt_endymion::hotkeys::Hotkey::named(action).is_some_and(emusen_qt_endymion::hotkeys::Hotkey::held);
                    out.about = self.cw(if held { "hotkey_held" } else { "hotkey_press" });
                }
                return out;
            }
            Some(Page::Output) => {
                if entry["output"].is_string() {
                    out.about = self.output_sentence(&out.label);
                }
                return out;
            }
            _ => {}
        }
        let Some(console) = self.console() else { return out };
        match self.page() {
            Some(Page::Console(_)) => {
                if row.id == "engine" {
                    out.label = "Core".to_owned();
                    out.about = format!("The core that runs the {}'s games; A opens its own settings.", console.name);
                    out.form = Form::Door;
                } else if row.id == "firmware" {
                    let (found, total, _) = firmware::file_statuses(&console.firmware, &self.read.consoles.overview);
                    out.about = format!("The firmware the {}'s cores use, found by its content in your folders.", console.name);
                    out.value = firmware::count_value(found, total, false);
                } else if row.id == "cards" {
                    if let Some(mode_row) = console.media.first().and_then(|m| cards::keys(m, &console.schema).mode) {
                        let shown = schema::shown(mode_row, &console.value(&mode_row.key));
                        out.value = if shown == "Per game" { "One per game".to_owned() } else { shown };
                        out.about = format!("The {}'s cards: shared by every game as the console's were, or one per game.", console.name);
                        out.form = Form::Choice;
                    } else {
                        out.value.clear();
                        out.about = format!("The {}'s games keep their saves in the cartridge.", console.name);
                        out.form = Form::Fact;
                    }
                } else if row.id == "consolecrop" {
                    out.about = "The whole picture, the part a television showed, or a crop of your own.".to_owned();
                } else if row.id == "consoleshape" {
                    out.about = "The screen the console was made for, or each pixel square.".to_owned();
                } else if row.id == "reset" {
                    out.about = format!("Every {} setting back to its default: the core's rows, its picture and its shaders.", console.name);
                } else if entry["heading"].is_string() {
                    out.about.clear();
                    out.form = Form::Fact;
                } else if let Some(key) = entry["schema"].as_str() {
                    out.about = console.row(key).and_then(|r| r.hint.clone()).unwrap_or_default();
                }
            }
            Some(Page::Engine(_)) => {
                out.about = if row.marked {
                    format!("{} runs every {} game.", row.label, console.name)
                } else {
                    format!("{} would run every {} game.", row.label, console.name)
                };
                if row.marked {
                    out.value = format!("● {}", row.value);
                }
            }
            Some(Page::Firmware(_)) => {
                if row.id == "fwreplacement" {
                    out.about = "Whether the console's open replacement runs in place of firmware of your own.".to_owned();
                }
            }
            Some(Page::Cards(_)) => {
                if let Some(card_row) = self.card_row(entry) {
                    if let Some(m) = console.media.iter().find(|m| m.medium == card_row.medium) {
                        let noun = m.noun_in_text();
                        out.about = match card_row.part {
                            cards::Part::Mode => format!("Per game, shared by every game as the console's were, or a drawer of named {noun}s."),
                            cards::Part::Slot(_) => format!("The {noun} in this slot when a game starts."),
                            cards::Part::Drawer => format!("Every {noun} in the drawer, and the saves on each."),
                            cards::Part::Saves => format!("Each {noun}'s saves, copied, moved or deleted."),
                            cards::Part::New => format!("A blank {noun} in the drawer, named for you."),
                        };
                    }
                }
            }
            Some(Page::Drawer(_, _)) => {
                out.about = cards::DRAWER_CARD.to_owned();
                out.form = Form::Fact;
            }
            Some(Page::Saves(_, _)) => {
                if let Some(m) = self.saves_medium() {
                    if entry.get("savescard").is_some() {
                        out.about = format!("Left and Right step every {}, the drawer's and each game's own.", m.noun_in_text());
                    } else if let Some(save) = entry["save"].as_str().and_then(|id| self.save(id)) {
                        let unit = m.unit.as_deref().unwrap_or_default();
                        let blocks = words::counted(save.units as usize, unit, &format!("{unit}s"));
                        out.about = "Choose a save to copy, move or delete it.".to_owned();
                        out.value = if save.damaged { format!("{blocks} · damaged") } else { blocks };
                    } else if entry.get("import").is_some() {
                        out.about = "Every save on every game's own card copied onto the chosen card, all or none; each game's own card stays as it is.".to_owned();
                    }
                }
            }
            Some(Page::Save(_, _, _)) => {
                if entry["saveop"] == "target" {
                    if let Some(m) = self.saves_medium() {
                        out.about = format!("The {} the save is copied or moved to.", m.noun_in_text());
                    }
                }
            }
            _ => {}
        }
        out
    }

    /// Eclipse's board-exact page heading, glow, About panel, core card and firmware list (boards #27-#29, #57, #58, #63, #64).
    pub fn board_aside(&self, focused: usize, rows: &[Row], board_rows: &[BoardRow], pane: &Pane) -> BoardAside {
        let on_consoles = self.section_keys().get(self.section).map(String::as_str) == Some("consoles");
        let heading = match self.page() {
            Some(Page::Engine(_)) => "Core".to_owned(),
            _ => self.heading(),
        };
        let consoles_subtitle = if self.page().is_none() && on_consoles {
            format!("{}. Each one's core, firmware, memory cards and picture.", words::counted(self.listed_consoles().len(), "console", "consoles"))
        } else {
            String::new()
        };
        let page_glow = if on_consoles {
            let active = self.console().or_else(|| self.listed_consoles().get(focused).copied());
            let planet = active.map_or(-2, |c| {
                let engine = clients::engine_shown(&c.engine(), &self.read.consoles.cores);
                planet_of(&engine.name, engine.libretro, c.runs())
            });
            planet_shade(planet).to_owned()
        } else if matches!(self.page(), Some(Page::Players | Page::Profiles | Page::Profile(..) | Page::Hotkeys)) {
            "#3F6FE0".to_owned()
        } else if matches!(self.page(), Some(Page::Output)) {
            "#B8862B".to_owned()
        } else {
            String::new()
        };
        let cards_page = matches!(self.page(), Some(Page::Cards(_) | Page::Drawer(..) | Page::Saves(..) | Page::Save(..) | Page::Delete(..)));
        let s6_page = matches!(self.page(), Some(Page::Players | Page::Profiles | Page::Profile(..) | Page::Hotkeys | Page::Output));
        let s6_cards_help = matches!(self.page(), Some(Page::Players | Page::Profiles | Page::Profile(..)));
        let mut out = BoardAside {
            heading,
            about_heading: if pane.preview == crate::settings::Preview::Chips {
                pane.caption.clone()
            } else {
                board_rows.get(focused).map(|r| r.label.clone()).unwrap_or_default()
            },
            sentence: if pane.preview == crate::settings::Preview::Chips { String::new() } else { pane.sentence.clone() },
            lines: pane.lines.clone(),
            muted: pane.muted.clone(),
            consoles_subtitle,
            page_glow,
            card_planet: -3,
            card_title: String::new(),
            card_subtitle: String::new(),
            firmware_files: Vec::new(),
            wrap_about: cards_page || s6_page,
            cards_help: cards_page || s6_cards_help,
            box_menu: cards_page || s6_page,
        };
        match self.page() {
            Some(Page::Players) => {
                out.about_heading = self.heading();
                let profile = self.pad_console().map(|c| self.cw("profile_mark").replace("{profile}", &self.read.bindings.console_default(&c))).unwrap_or_default();
                let mut lines: Vec<String> = self
                    .seats
                    .players
                    .iter()
                    .filter_map(|p| {
                        let pad = p.pad.as_ref()?;
                        let base = self.cw("players_line").replace("{n}", &p.player.to_string()).replace("{pad}", pad);
                        Some(if p.kept { format!("{base}{}", self.cw("kept_mark")) } else { format!("{base}{profile}") })
                    })
                    .collect();
                let free: Vec<String> = (1..=self.seats_shown()).filter(|n| self.seats.players.iter().all(|p| p.player != *n || p.pad.is_none())).map(|n| n.to_string()).collect();
                let mut muted = Vec::new();
                if !free.is_empty() {
                    muted.push(self.cw("free").replace("{list}", &free.join(&self.cw("and"))));
                }
                muted.push(self.cw("players_muted"));
                out.sentence = if lines.is_empty() { String::new() } else { lines.remove(0) };
                out.lines = lines;
                out.muted = muted;
                return out;
            }
            Some(Page::Profiles) => {
                if let Some(console) = self.pad_console() {
                    let count = words::counted(self.read.bindings.profiles(&console).len(), &self.cw("profile"), &self.cw("profiles"));
                    let summary = self
                        .cw("profiles_line")
                        .replace("{console}", &self.console_name(&console))
                        .replace("{count}", &count)
                        .replace("{chosen}", &self.read.bindings.console_default(&console));
                    out.muted.push(summary);
                }
                return out;
            }
            Some(Page::Profile(..)) => {
                out.muted = vec![self.cw("profiles_muted")];
                return out;
            }
            Some(Page::Hotkeys) => {
                out.sentence = pane.lines.first().cloned().unwrap_or_default();
                let mut lines = Vec::new();
                if !pane.sentence.is_empty() {
                    lines.push(pane.sentence.clone());
                }
                if pane.lines.len() > 1 {
                    lines.extend(pane.lines[1..].iter().cloned());
                }
                out.lines = lines;
                return out;
            }
            Some(Page::Output) => {
                out.about_heading = self.heading();
                return out;
            }
            _ => {}
        }
        let Some(console) = self.console() else { return out };
        let other_cores = |chosen: &str| -> Vec<String> {
            console
                .row(consoles::ENGINE_KEY)
                .and_then(|r| r.choices.as_ref())
                .into_iter()
                .flatten()
                .filter(|c| c.as_str() != chosen)
                .map(|c| self.read.consoles.engine_name(c))
                .collect()
        };
        match self.page() {
            Some(Page::Console(_)) => {
                if rows.get(focused).is_some_and(|r| r.id == "engine") {
                    let engine = clients::engine_shown(&console.engine(), &self.read.consoles.cores);
                    out.card_planet = planet_of(&engine.name, engine.libretro, true);
                    out.card_title = engine.name.clone();
                    out.card_subtitle = if engine.libretro { "Libretro core, in use" } else { "EmuSen's own core, in use" }.to_owned();
                    out.about_heading = "Core".to_owned();
                    out.sentence = format!("The core that runs the {}'s games. Its own settings are the rows marked with its planet.", console.name);
                    let mut facts = format!("{} · {} · core {}", console.name, words::games(self.games_of(console)), engine.name);
                    if self.television(&console.console) {
                        if let Some(crt) = self.description_at(&["consoles", "page"]).as_array().and_then(|p| p.iter().find(|e| e.get("filter").is_some())) {
                            if let Some(word) = self.filter_choices(crt).get(self.filter_tier(crt, &console.console)) {
                                facts.push_str(&format!(" · CRT profile {word}"));
                            }
                        }
                    }
                    let mut muted = vec![facts];
                    let others = other_cores(&console.engine());
                    if !others.is_empty() {
                        muted.push(format!("{} can be chosen here too; this is the same setting as Cores' Make Default.", join_and(&others)));
                    }
                    out.muted = muted;
                } else if rows.get(focused).is_some_and(|r| r.id == "reset") {
                    out.sentence = out.sentence.replace("the engine's rows", "the core's rows").replace("the engine holds", "the core holds");
                }
            }
            Some(Page::Engine(_)) => {
                if let Some(row) = rows.get(focused) {
                    out.about_heading = row.label.clone();
                    out.sentence = if row.marked {
                        format!("{}, the {}'s core: every {} game opens on it unless the game's own options choose another.", row.label, console.name, console.name)
                    } else {
                        format!("{} would run every {} game.", row.label, console.name)
                    };
                    let chosen_id = row.id.strip_prefix("engine:").unwrap_or_default();
                    let others = other_cores(chosen_id);
                    let mut muted = Vec::new();
                    if !others.is_empty() {
                        muted.push(format!("{} can be chosen here too; this is the same setting as Cores' Make Default.", join_and(&others)));
                    }
                    muted.push(self.console_word("choose"));
                    out.muted = muted;
                }
            }
            Some(Page::Firmware(_)) => {
                let (found, total, files) = firmware::file_statuses(&console.firmware, &self.read.consoles.overview);
                out.about_heading = firmware::count_value(found, total, true);
                out.sentence.clear();
                out.muted = vec![firmware::CONTENT_NOTE.to_owned()];
                out.firmware_files = files;
            }
            Some(Page::Saves(..)) => {
                if let Some(m) = self.saves_medium() {
                    out.sentence = m.mode_text();
                    let mut muted = Vec::new();
                    if let Some(card) = self.chosen() {
                        out.about_heading = card.label.clone();
                        let unit = m.unit.as_deref().unwrap_or_default();
                        match (m.opened(&card.path), m.capacity) {
                            (None, _) => muted.push("No card of this medium is in the drawer or a game's yet.".to_owned()),
                            (Some(o), None) => muted.push(format!("{} bytes. No format module reads this medium yet, so its saves are not listed; a new card can still be made.", words::grouped(o.length as usize))),
                            (Some(o), Some(capacity)) if o.formatted => {
                                let saves = words::counted(o.saves.len(), "save", "saves");
                                let units = if capacity == 1 { unit.to_owned() } else { format!("{unit}s") };
                                muted.push(format!("{saves}, {} of {capacity} {units} free.", o.free));
                            }
                            (Some(o), Some(_)) => {
                                if let Some(problem) = &o.problem {
                                    if !problem.is_empty() {
                                        muted.push(problem.clone());
                                    }
                                }
                            }
                        }
                        if let Some(holder) = &card.held_by {
                            muted.push(format!("In use by {holder}, which is running: its saves can be looked at, and changed when the game is closed."));
                        }
                        if let Some(id) = rows.get(focused).and_then(|r| r.id.strip_prefix("save:")) {
                            if let Some(game) = self.save(id).and_then(|s| s.game.clone()) {
                                muted.push(game);
                            }
                        }
                    } else {
                        muted.push("No card of this medium is in the drawer or a game's yet.".to_owned());
                    }
                    if !self.said.is_empty() {
                        muted.push(self.said.clone());
                    }
                    out.muted = muted;
                }
            }
            Some(Page::Save(..)) => {
                out.about_heading = self.page_save().and_then(|id| self.save(&id)).map(|s| s.title.clone()).unwrap_or_else(|| self.heading());
            }
            Some(Page::Delete(..)) => {
                out.about_heading = self.heading();
            }
            _ => {}
        }
        out
    }
}
