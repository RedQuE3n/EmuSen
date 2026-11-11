//! The painter's state without Qt: the systems with their resolved themes, the stage, its clock in microseconds, and the
//! pad's grammar on the ES-DE view (EmuSen_Qt.md §4.8). `EsdeModel` puts it on Qt; everything here is plain Rust.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

use emusen_esde::fonts;
use emusen_esde::loader::{self, Choices, MediaPresence, Theme, ThemeSystem};
use emusen_esde::scene::{Data, DateTime, DeviceStatus, Frame, HelpContext, Key, Layout, PadButton, Row, Rows, Stage, System, Time};

use emusen_kit_core::input::{PadControl, UiButton};

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
            Some(System { stars: s.stars, favorites_on_top: s.favorites_on_top, counts: s.counts, unlisted: s.unlisted, ..System::new(system, theme, s.games) })
        })
        .collect()
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
    held: Option<UiButton>,
    pub settings: Settings,
    /// Counts installs, so a list model knows its rows may have changed under the same `Rows`.
    pub generation: u64,
    /// The player's face-button swap, which the help's buttons follow.
    pub swap: bool,
    /// What the host reports of the device for systemstatus; none until it reports.
    pub device: DeviceStatus,
    /// ES-DE's default face as the host gave it; none, the one `fonts` finds.
    pub default_font: Option<String>,
    /// The gameselectors' seed as the host gave it; none, the install's time.
    pub selection_seed: Option<u64>,
}

impl Default for Painter {
    fn default() -> Self {
        Painter::new(Settings::default())
    }
}

impl Painter {
    pub fn new(settings: Settings) -> Self {
        Painter { stage: None, now: 0, frame: Frame::default(), held: None, settings, generation: 0, swap: false, device: DeviceStatus::default(), default_font: None, selection_seed: None }
    }

    /// Installs a refresh's systems, keeping the view and the selection where they still exist.
    pub fn install(&mut self, systems: Vec<System>, w: f64, h: f64, now: DateTime) {
        self.generation += 1;
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
        // The gameselectors' random picks start from the host's seed, else the install's time, so a pinned time pins them too.
        data.selection_seed = self.selection_seed.unwrap_or(now.seconds() as u64);
        data.default_font = self.default_font.clone().or_else(|| fonts::default_font().map(|p| p.to_string_lossy().into_owned()));
        data.help_font = data.default_font.as_deref().and_then(|d| fonts::help_font(std::path::Path::new(d))).map(|p| p.to_string_lossy().into_owned());
        data.show_clock = self.settings.show_clock;
        data.show_help = self.settings.show_help;
        data.scroll_overlay = self.settings.scroll_overlay;
        data.viewer = self.settings.viewer;
        data.swap = self.swap;
        data.status = self.device;
        data.help = HelpContext {
            random_games: self.settings.random != "disabled",
            random_systems: self.settings.random == "gamessystems",
            quick_select: self.settings.quick_select.clone(),
            screensaver: self.settings.screensaver_controls,
            ..HelpContext::default()
        };
        let view = if view == "gamelist" { "gamelist" } else { "system" };
        match &mut self.stage {
            Some(stage) => stage.restart(data, view, self.now),
            None => self.stage = Some(Stage::new(data, view, self.now)),
        }
        self.held = None;
        self.refresh();
    }

    /// The player's swap: the help names the buttons the functions moved to.
    pub fn set_swap(&mut self, swap: bool) {
        self.swap = swap;
        if let Some(stage) = &mut self.stage {
            stage.current.data.swap = swap;
        }
        self.refresh();
    }

    /// The device's radios and battery as the host reports them.
    pub fn set_device(&mut self, device: DeviceStatus) {
        self.device = device;
        if let Some(stage) = &mut self.stage {
            stage.current.data.status = device;
        }
        self.refresh();
    }

    /// The wall clock the clock element and the dates read.
    pub fn set_now(&mut self, now: DateTime) {
        if let Some(stage) = &mut self.stage {
            stage.current.data.now = now;
        }
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
        stage.restart(data, "gamelist", self.now);
    }

    /// A press from the router; the scene repeats a held direction itself, so the router's repeats are not passed on.
    pub fn pressed(&mut self, button: UiButton, repeat: bool) -> Option<Action> {
        if repeat || self.stage.is_none() {
            return None;
        }
        let now = self.now;
        let vertical = self.vertical();
        let grid = self.primary_kind() == Some("grid");
        let gamelist = self.view() == "gamelist";
        let quick = self.settings.quick_select.clone();
        if let Some(viewer) = self.stage.as_mut()?.viewer.as_mut() {
            // The viewer's keys (scene::viewer::Key); any other button closes it.
            let key = match button {
                UiButton::Left => Some(Key::Left),
                UiButton::Right => Some(Key::Right),
                UiButton::Up => Some(Key::Up),
                UiButton::Down => Some(Key::Down),
                UiButton::First => Some(Key::First),
                UiButton::Last => Some(Key::Last),
                UiButton::PageUp => Some(Key::ZoomOut),
                UiButton::PageDown => Some(Key::ZoomIn),
                _ => None,
            };
            let moved = match key.and_then(|k| viewer.press(k)) {
                Some(moved) => moved,
                None => {
                    self.stage.as_mut()?.viewer = None;
                    false
                }
            };
            self.refresh();
            return moved.then_some(Action::Cue("navigate"));
        }
        let action = match button {
            UiButton::Up | UiButton::Down | UiButton::Left | UiButton::Right => {
                let along = matches!(button, UiButton::Up | UiButton::Down) == vertical || grid;
                let dir = if matches!(button, UiButton::Up | UiButton::Left) { -1 } else { 1 };
                if along {
                    let rows = grid && matches!(button, UiButton::Up | UiButton::Down);
                    self.stage.as_mut()?.current.press(dir, now, rows);
                    self.held = Some(button);
                    Some(Action::Cue("navigate"))
                } else if gamelist && quick.starts_with("leftright") && matches!(button, UiButton::Left | UiButton::Right) {
                    self.change_system(dir as i64);
                    Some(Action::Cue("quicksysselect"))
                } else {
                    None
                }
            }
            UiButton::Accept if gamelist => self.game_file().map(Action::Played),
            UiButton::Accept => {
                self.stage.as_mut()?.switch(now);
                Some(Action::Cue("select"))
            }
            UiButton::Back if gamelist => {
                self.stage.as_mut()?.switch(now);
                Some(Action::Cue("back"))
            }
            UiButton::PageUp | UiButton::PageDown if gamelist && quick == "leftrightshoulders" => {
                self.change_system(if button == UiButton::PageUp { -1 } else { 1 });
                Some(Action::Cue("quicksysselect"))
            }
            UiButton::First | UiButton::Last if gamelist && quick == "leftrighttriggers" => {
                self.change_system(if button == UiButton::First { -1 } else { 1 });
                Some(Action::Cue("quicksysselect"))
            }
            UiButton::PageUp | UiButton::PageDown => {
                let moved = self.stage.as_mut()?.current.jump(if button == UiButton::PageUp { -10 } else { 10 }, now);
                moved.then_some(Action::Cue("navigate"))
            }
            UiButton::First | UiButton::Last => {
                let current = &mut self.stage.as_mut()?.current;
                let n = current.count() as i32;
                current.jump(if button == UiButton::First { -n } else { n }, now).then_some(Action::Cue("navigate"))
            }
            UiButton::Search if gamelist => self.game_file().map(Action::Favourite),
            UiButton::Screensaver if gamelist => self.stage.as_mut()?.open_viewer().then_some(Action::Cue("select")),
            UiButton::Menu => Some(Action::Menu),
            UiButton::Options => Some(Action::Options),
            _ => None,
        };
        self.refresh();
        action
    }

    /// A release from the router: a held direction stops and the metadata fades back in.
    pub fn released(&mut self, button: UiButton) {
        if self.held == Some(button) {
            self.held = None;
            let now = self.now;
            if let Some(stage) = &mut self.stage {
                stage.current.release(now);
            }
            self.refresh();
        }
    }

    /// The pointer over a row selects it, as hovering a walkable focuses it; not while the items glide, since an item
    /// moving under a still pointer is no choice of the player's and would chase the selection (EmuSen_Qt.md §4.8.5).
    pub fn hover(&mut self, row: i32) {
        if self.stage.as_ref().is_some_and(|s| !s.current.is_animating()) {
            self.select(row);
        }
        self.refresh();
    }

    /// A row made the selection, as a step to it, and the frame at once.
    pub fn choose(&mut self, row: i32) {
        self.select(row);
        self.refresh();
    }

    /// The shown system's shelf name and the primary's selected row; none before an install.
    pub fn selection(&self) -> (&str, i32) {
        self.stage.as_ref().map_or(("", -1), |s| (s.current.data.system().system.name.as_str(), s.current.index() as i32))
    }

    // A row made the selection, as a step to it.
    fn select(&mut self, row: i32) {
        let now = self.now;
        if let Some(stage) = &mut self.stage {
            let current = &mut stage.current;
            let delta = row - current.index() as i32;
            if delta != 0 && row >= 0 && (row as usize) < current.count() {
                current.step(delta, now);
            }
        }
    }

    /// A tap selects its row and takes A there, the same call as the pad's, glide or not.
    pub fn tap(&mut self, row: i32) -> Option<Action> {
        self.select(row);
        self.refresh();
        self.pressed(UiButton::Accept, false)
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

    pub fn manual_pages(&mut self, pages: usize) -> bool {
        let changed = self.stage.as_mut().is_some_and(|s| s.manual_pages(pages));
        if changed {
            self.refresh();
        }
        changed
    }

    pub fn rows(&self, rows: Rows) -> Vec<Row> {
        self.stage.as_ref().and_then(|s| s.rows(rows)).unwrap_or_default()
    }
}

/// A help entry's place as the kit's button that draws there under the swap, and its pair's (-1 for none): the kit's
/// `PadGlyph` draws `UiButton::control(swap)`, so the press of the place, swapped, draws the place itself.
pub fn help_buttons(place: PadButton, pair: bool, swap: bool) -> (i32, i32) {
    let control = match place {
        PadButton::DpadUpDown => PadControl::Up,
        PadButton::DpadLeftRight => PadControl::Left,
        PadButton::South => PadControl::South,
        PadButton::East => PadControl::East,
        PadButton::West => PadControl::West,
        PadButton::North => PadControl::North,
        PadButton::LeftShoulder => PadControl::LeftShoulder,
        PadButton::RightShoulder => PadControl::RightShoulder,
        PadButton::LeftTrigger => PadControl::LeftTrigger,
        PadButton::RightTrigger => PadControl::RightTrigger,
        PadButton::Select => PadControl::Select,
        PadButton::Start => PadControl::Start,
        PadButton::StickClick => PadControl::LeftStick,
        PadButton::Dpad => PadControl::Up,
    };
    let number = |c: PadControl| c.press().map_or(-1, |b| b.swapped(swap).index() as i32);
    let second = match (place, pair) {
        (PadButton::DpadUpDown, true) => number(PadControl::Down),
        (PadButton::DpadLeftRight, true) => number(PadControl::Right),
        (PadButton::LeftShoulder, true) => number(PadControl::RightShoulder),
        (PadButton::LeftTrigger, true) => number(PadControl::RightTrigger),
        (PadButton::Dpad, true) => number(PadControl::Left),
        _ => -1,
    };
    (number(control), second)
}

/// The navigation sounds of the first system's theme as `{name: file}`, the files the theme names that exist; ES-DE
/// reads them once from its first system's theme.
pub fn sounds(painter: &Painter) -> String {
    let Some(stage) = &painter.stage else { return "{}".into() };
    let Some(first) = stage.current.data.systems.first() else { return "{}".into() };
    let found: serde_json::Map<String, serde_json::Value> =
        first.theme.sounds().into_iter().filter(|(_, p)| p.exists).map(|(n, p)| (n.to_owned(), p.absolute.clone().into())).collect();
    serde_json::Value::Object(found).to_string()
}

/// The first error the loader met in a system's theme, or a missing theme file, as the host's notice words it.
pub fn problem(systems: &[System]) -> String {
    use emusen_esde::loader::Severity;
    systems
        .iter()
        .find_map(|s| match s.theme.entry_file {
            None => Some(format!("no theme.xml for {}", s.theme.system.name)),
            Some(_) => s.theme.diagnostics.iter().find(|d| d.severity == Severity::Error).map(|d| d.to_string()),
        })
        .unwrap_or_default()
}
