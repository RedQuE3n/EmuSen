//! The painter's state without Qt: the systems with their resolved themes, the stage, its clock in microseconds, and the
//! pad's grammar on the ES-DE view (EmuSen_Qt.md §4.8). `EsdeModel` puts it on Qt; everything here is plain Rust.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

use emusen_esde::loader::{self, Choices, MediaPresence, Theme, ThemeSystem};
use emusen_esde::scene::{Data, DateTime, Frame, Layout, Row, Rows, Stage, System, Time};

use crate::shelves::{Settings, Shelf};

/// A resolved theme per (system, choices, media presence), kept while the folder is unchanged, as `ThemedLibrary._themes` was.
#[derive(Clone, Default)]
pub struct ThemeCache {
    themes: Arc<Mutex<HashMap<String, Theme>>>,
}

impl ThemeCache {
    /// The theme for a system, loaded once per key.
    pub fn theme(&self, folder: &str, system: &ThemeSystem, choices: &Choices, media: Option<&MediaPresence>) -> Theme {
        let mut types: Vec<&String> = media.map(|m| m.types.iter().collect()).unwrap_or_default();
        types.sort();
        let key = format!("{folder}\u{1f}{system:?}\u{1f}{choices:?}\u{1f}{types:?}");
        let mut themes = self.themes.lock().unwrap_or_else(|e| e.into_inner());
        themes.entry(key).or_insert_with(|| loader::load(folder, system, choices, media)).clone()
    }

    /// Forgets every theme, as an update of the folder on disk asks.
    pub fn forget(&self) {
        self.themes.lock().unwrap_or_else(|e| e.into_inner()).clear();
    }
}

/// The systems a refresh hands the GUI thread: the shelves each with its theme, built on the data thread.
pub fn systems(cache: &ThemeCache, settings: &Settings, shelves: Vec<Shelf>, presence: &BTreeMap<String, MediaPresence>, w: i32, h: i32) -> Vec<System> {
    let choices = Choices { screen_width: w, screen_height: h, ..settings.choices.clone() };
    shelves
        .into_iter()
        .filter_map(|s| {
            let system = s.system?;
            let theme = cache.theme(&settings.theme, &system, &choices, presence.get(&system.name));
            Some(System { stars: s.stars, favorites_on_top: s.favorites_on_top, counts: s.counts, ..System::new(system, theme, s.games) })
        })
        .collect()
}

/// The kit's pad buttons by their numbers in `Kit.PadButton` (kit-qt's `kinds.rs`), the one owner of the order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    Up,
    Down,
    Left,
    Right,
    Accept,
    Back,
    Menu,
    Options,
    PageUp,
    PageDown,
    First,
    Last,
    Guide,
    Search,
    Random,
    Screensaver,
}

impl Button {
    pub const ALL: [Button; 16] = [
        Button::Up,
        Button::Down,
        Button::Left,
        Button::Right,
        Button::Accept,
        Button::Back,
        Button::Menu,
        Button::Options,
        Button::PageUp,
        Button::PageDown,
        Button::First,
        Button::Last,
        Button::Guide,
        Button::Search,
        Button::Random,
        Button::Screensaver,
    ];

    pub fn of(number: i32) -> Option<Button> {
        usize::try_from(number).ok().and_then(|i| Button::ALL.get(i)).copied()
    }
}

/// What a press asks of the host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Played(String),
    Favourite(String),
    Menu,
    Options,
    /// The view stepped, opened or went back: the cue the host plays.
    Cue(&'static str),
}

pub struct Painter {
    pub stage: Option<Stage>,
    /// The stage's clock, microseconds since the painter began.
    pub now: Time,
    pub frame: Frame,
    held: Option<Button>,
    pub settings: Settings,
}

impl Default for Painter {
    fn default() -> Self {
        Painter::new(Settings::default())
    }
}

impl Painter {
    pub fn new(settings: Settings) -> Self {
        Painter { stage: None, now: 0, frame: Frame::default(), held: None, settings }
    }

    /// Installs a refresh's systems, keeping the view and the selection where they still exist.
    pub fn install(&mut self, systems: Vec<System>, w: f64, h: f64, now: DateTime) {
        if systems.is_empty() {
            self.stage = None;
            self.frame = Frame::default();
            return;
        }
        let (view, system, game) = match &self.stage {
            Some(s) => (s.current.view.clone(), s.current.data.system_index, s.current.data.game_index),
            None => (self.settings.startup_view.clone(), 0, 0),
        };
        let mut data = Data::new(systems, w, h);
        data.system_index = system.min(data.systems.len() - 1);
        data.game_index = game;
        data.now = now;
        data.show_clock = self.settings.show_clock;
        data.show_help = self.settings.show_help;
        data.scroll_overlay = self.settings.scroll_overlay;
        let view = if view == "gamelist" { "gamelist" } else { "system" };
        self.stage = Some(Stage::new(data, view, self.now));
        self.held = None;
        self.refresh();
    }

    /// The frame at the stage's time.
    pub fn refresh(&mut self) {
        if let Some(stage) = &mut self.stage {
            stage.advance(self.now);
            self.frame = stage.frame();
        }
    }

    /// The clock moved on by `ms` milliseconds.
    pub fn tick(&mut self, ms: f64) {
        self.now += (ms.max(0.0) * 1000.0).round() as Time;
        self.refresh();
    }

    pub fn view(&self) -> &str {
        self.stage.as_ref().map_or("", |s| s.current.view.as_str())
    }

    fn primary_kind(&self) -> Option<&'static str> {
        match self.frame.primary.as_ref().map(|p| &p.layout) {
            Some(Layout::Carousel(_)) => Some("carousel"),
            Some(Layout::TextList(_)) => Some("textlist"),
            Some(Layout::Grid(_)) => Some("grid"),
            None => None,
        }
    }

    // Whether the primary element moves along the vertical: a list, a vertical carousel or wheel; a grid moves both ways.
    fn vertical(&self) -> bool {
        let Some(stage) = &self.stage else { return false };
        match self.primary_kind() {
            Some("textlist") => true,
            Some("carousel") => stage.current.theme_view().primary().and_then(|p| p.string("type")).is_some_and(|t| t.starts_with("vertical")),
            _ => false,
        }
    }

    fn game_file(&self) -> Option<String> {
        let stage = self.stage.as_ref()?;
        (stage.current.view == "gamelist").then(|| stage.current.data.game().map(|g| g.file.clone())).flatten()
    }

    // Another system's gamelist, as ES-DE's quick system select opens it.
    fn change_system(&mut self, delta: i64) {
        let Some(stage) = &mut self.stage else { return };
        let mut data = stage.current.data.clone();
        let n = data.systems.len() as i64;
        data.system_index = (data.system_index as i64 + delta).rem_euclid(n) as usize;
        data.game_index = 0;
        *stage = Stage::new(data, "gamelist", self.now);
    }

    /// A press from the router; the scene repeats a held direction itself, so the router's repeats are not passed on.
    pub fn pressed(&mut self, button: Button, repeat: bool) -> Option<Action> {
        if repeat || self.stage.is_none() {
            return None;
        }
        let now = self.now;
        let vertical = self.vertical();
        let grid = self.primary_kind() == Some("grid");
        let gamelist = self.view() == "gamelist";
        let quick = self.settings.quick_select.clone();
        let action = match button {
            Button::Up | Button::Down | Button::Left | Button::Right => {
                let along = matches!(button, Button::Up | Button::Down) == vertical || grid;
                let dir = if matches!(button, Button::Up | Button::Left) { -1 } else { 1 };
                if along {
                    let rows = grid && matches!(button, Button::Up | Button::Down);
                    self.stage.as_mut()?.current.press(dir, now, rows);
                    self.held = Some(button);
                    Some(Action::Cue("navigate"))
                } else if gamelist && quick.starts_with("leftright") && matches!(button, Button::Left | Button::Right) {
                    self.change_system(dir as i64);
                    Some(Action::Cue("quicksysselect"))
                } else {
                    None
                }
            }
            Button::Accept if gamelist => self.game_file().map(Action::Played),
            Button::Accept => {
                self.stage.as_mut()?.switch(now);
                Some(Action::Cue("select"))
            }
            Button::Back if gamelist => {
                self.stage.as_mut()?.switch(now);
                Some(Action::Cue("back"))
            }
            Button::PageUp | Button::PageDown if gamelist && quick == "leftrightshoulders" => {
                self.change_system(if button == Button::PageUp { -1 } else { 1 });
                Some(Action::Cue("quicksysselect"))
            }
            Button::First | Button::Last if gamelist && quick == "leftrighttriggers" => {
                self.change_system(if button == Button::First { -1 } else { 1 });
                Some(Action::Cue("quicksysselect"))
            }
            Button::PageUp | Button::PageDown => {
                let moved = self.stage.as_mut()?.current.jump(if button == Button::PageUp { -10 } else { 10 }, now);
                moved.then_some(Action::Cue("navigate"))
            }
            Button::First | Button::Last => {
                let current = &mut self.stage.as_mut()?.current;
                let n = current.count() as i32;
                current.jump(if button == Button::First { -n } else { n }, now).then_some(Action::Cue("navigate"))
            }
            Button::Search if gamelist => self.game_file().map(Action::Favourite),
            Button::Menu => Some(Action::Menu),
            Button::Options => Some(Action::Options),
            _ => None,
        };
        self.refresh();
        action
    }

    /// A release from the router: a held direction stops and the metadata fades back in.
    pub fn released(&mut self, button: Button) {
        if self.held == Some(button) {
            self.held = None;
            let now = self.now;
            if let Some(stage) = &mut self.stage {
                stage.current.release(now);
            }
            self.refresh();
        }
    }

    /// The pointer over a row selects it, as hovering a walkable focuses it.
    pub fn hover(&mut self, row: i32) {
        let now = self.now;
        if let Some(stage) = &mut self.stage {
            let current = &mut stage.current;
            let delta = row - current.index() as i32;
            if delta != 0 && row >= 0 && (row as usize) < current.count() {
                current.step(delta, now);
            }
        }
        self.refresh();
    }

    /// A tap selects its row and takes A there, the same call as the pad's.
    pub fn tap(&mut self, row: i32) -> Option<Action> {
        self.hover(row);
        self.pressed(Button::Accept, false)
    }

    /// One wheel notch steps one item: back for a notch up, on for a notch down.
    pub fn wheel(&mut self, notches: i32) {
        let now = self.now;
        if let (Some(stage), true) = (&mut self.stage, notches != 0) {
            stage.current.step(-notches.signum(), now);
        }
        self.refresh();
    }

    /// The painter's measurement of an element; true when the frame changed.
    pub fn measured(&mut self, id: u64, w: f64, h: f64) -> bool {
        let changed = self.stage.as_mut().is_some_and(|s| s.measured(id, w, h));
        if changed {
            self.refresh();
        }
        changed
    }

    pub fn rows(&self, rows: Rows) -> Vec<Row> {
        self.stage.as_ref().and_then(|s| s.rows(rows)).unwrap_or_default()
    }
}
