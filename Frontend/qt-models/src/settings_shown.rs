//! What a theme's Settings reads beyond a row's words (EmuSen_Qt.md §3.2, `Settings`): the page's key, a row's about line,
//! a switch's state as a state, and a console row's tile, each from the owner its row's words come from.

use crate::schema;
use crate::settings::{Page, SettingsView};
use serde_json::Value;

/// What a row shows beyond its words: its about line, a switch's state (none for a row of another kind), a console's tile.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RowFacets {
    pub about: String,
    pub switch: Option<bool>,
    pub tile: Option<ConsoleTile>,
}

/// A console row of the Consoles section as its tile reads it: the console's stored name, the system key the library's
/// consoles are keyed by, its games and the engine that runs it.
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

    /// A row's short line under its name: the description's `about` for the row, else its Galaxia row's; empty for none.
    fn about_of(&self, entry: &Value) -> String {
        let own = entry["about"].as_str().map(str::to_owned);
        own.or_else(|| self.keyed(entry).and_then(|k| k.row["about"].as_str().map(str::to_owned))).unwrap_or_default()
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
}
