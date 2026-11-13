//! What a view is drawn from besides the theme: the systems and their games, the selection, the screen and the time
//! (EmuSen_BigPicture.md §13.2). Plain records the host fills from its models.

use std::collections::BTreeMap;

use crate::loader::{SystemKind, Theme, ThemeSystem};

use super::motion::Motion;

/// A calendar time with no zone, as a game's dates and the clock carry it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTime {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

impl DateTime {
    pub fn new(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> Self {
        DateTime { year, month, day, hour, minute, second }
    }

    /// Days since 1970-01-01 (Howard Hinnant's days_from_civil).
    pub fn days(&self) -> i64 {
        let y = self.year as i64 - i64::from(self.month <= 2);
        let era = (if y >= 0 { y } else { y - 399 }) / 400;
        let yoe = y - era * 400;
        let m = self.month as i64;
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + self.day as i64 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146097 + doe - 719468
    }

    pub fn seconds(&self) -> i64 {
        self.days() * 86400 + (self.hour * 3600 + self.minute * 60 + self.second) as i64
    }

    /// 0 for Sunday.
    pub fn weekday(&self) -> u32 {
        (self.days() + 4).rem_euclid(7) as u32
    }
}

/// One game as a themed view shows it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Game {
    pub name: String,
    pub file: String,
    pub description: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub genre: Option<String>,
    pub players: Option<String>,
    pub release_date: Option<DateTime>,
    pub last_played: Option<DateTime>,
    /// 0 to 1.
    pub rating: Option<f32>,
    pub play_time_seconds: Option<u64>,
    pub play_count: u32,
    pub favorite: bool,
    pub completed: bool,
    pub folder: bool,
    pub kid_game: bool,
    pub broken: bool,
    pub emulator: Option<String>,
    /// The system the game belongs to, which a collection lists it beside others.
    pub source: Option<ThemeSystem>,
    /// ES-DE's "Hide metadata fields" for this entry.
    pub hide_metadata: bool,
    /// ES-DE's "Exclude from game counter".
    pub not_counted: bool,
    /// The game's media the host found, by ES-DE's media type (cover, marquee, screenshot, ...): absolute paths.
    pub media: BTreeMap<String, String>,
    /// ES-DE's controller metadata, one of THEMES.md's customControllerIcon names.
    pub controller: Option<String>,
    pub manual: bool,
    pub alt_emulator: bool,
    /// For a folder, the entry it opens directly.
    pub folder_link: Option<String>,
    /// The entry belongs to the custom collection being edited.
    pub in_collection: bool,
    /// A custom collection listed in the grouped system: opened like a folder, drawn without the folder mark.
    pub is_collection: bool,
}

impl Game {
    pub fn new(name: &str, file: &str) -> Self {
        Game { name: name.into(), file: file.into(), ..Game::default() }
    }
}

/// A system with the theme resolved for it and its games.
#[derive(Clone, Debug)]
pub struct System {
    pub system: ThemeSystem,
    pub theme: Theme,
    pub games: Vec<Game>,
    /// What a game's systemName and systemFullname read in the gamelist instead of the system's: a grouped collection's heading.
    pub heading: Option<String>,
    /// Favourites carry the list's star (ES-DE's "Add star markings to favorites").
    pub stars: bool,
    /// Favourites are sorted first, which the quick-scroll overlay marks with a star.
    pub favorites_on_top: bool,
    /// The counts gamelistinfo shows when the host filters or enters a folder; None counts the games.
    pub counts: Option<Counts>,
    /// The system's games the list's level does not show (inside its folders, or outside the folder open), which ES-DE's
    /// gameselectors draw from too (EmuSen_Qt.md §4.11).
    pub unlisted: Vec<Game>,
}

/// gamelistinfo's numbers: games and favourites over all, games kept by a filter, and whether a folder is open.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub games: usize,
    pub favorites: usize,
    pub shown: usize,
    pub filtered: bool,
    pub in_folder: bool,
}

impl Counts {
    /// The games that count: neither folders nor excluded from the counter.
    pub fn of(games: &[Game]) -> Self {
        let counted = games.iter().filter(|g| !g.folder && !g.not_counted);
        let (n, favorites) = counted.fold((0, 0), |(n, f), g| (n + 1, f + usize::from(g.favorite)));
        Counts { games: n, favorites, shown: n, filtered: false, in_folder: false }
    }
}

/// What the device reports for systemstatus; None where it has no such radio or battery.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DeviceStatus {
    pub bluetooth: Option<bool>,
    pub wifi: Option<bool>,
    pub cellular: Option<bool>,
    pub battery_percent: Option<u8>,
    pub charging: bool,
}

/// The status indicators the player keeps, each ANDed with an element's entries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Indicators {
    pub bluetooth: bool,
    pub wifi: bool,
    pub cellular: bool,
    pub battery: bool,
    pub percentage: bool,
}

impl Default for Indicators {
    fn default() -> Self {
        Indicators { bluetooth: true, wifi: true, cellular: true, battery: true, percentage: true }
    }
}

/// ES-DE's controller types, which pick a theme's help icons by their suffix.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ControllerType {
    #[default]
    Xbox,
    PlayStation,
    Switch,
}

/// What changes the help entries: a collection being edited, the random entry, a folder to enter, quick system select, the screensaver.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HelpContext {
    pub editing: bool,
    pub random_games: bool,
    pub random_systems: bool,
    /// leftright, shoulders, triggers or none.
    pub quick_select: String,
    pub screensaver: bool,
}

impl Default for HelpContext {
    fn default() -> Self {
        HelpContext { editing: false, random_games: false, random_systems: false, quick_select: "leftright".into(), screensaver: false }
    }
}

impl System {
    pub fn new(system: ThemeSystem, theme: Theme, games: Vec<Game>) -> Self {
        System { system, theme, games, heading: None, stars: true, favorites_on_top: false, counts: None, unlisted: Vec::new() }
    }

    /// A game by its place among the listed games and then the unlisted ones, the gameselectors' numbering.
    pub fn any_game(&self, i: usize) -> Option<&Game> {
        self.games.get(i).or_else(|| self.unlisted.get(i.checked_sub(self.games.len())?))
    }

    pub fn is_collection(&self) -> bool {
        self.system.kind != SystemKind::Regular
    }
}

/// Everything a view is drawn from besides the theme.
#[derive(Clone, Debug)]
pub struct Data {
    pub systems: Vec<System>,
    pub width: f64,
    pub height: f64,
    pub system_index: usize,
    pub game_index: usize,
    /// The calendar time a clock and a relative date read.
    pub now: DateTime,
    pub hide_metadata: bool,
    pub show_clock: bool,
    pub show_help: bool,
    pub motion: Motion,
    pub status: DeviceStatus,
    pub status_shown: Indicators,
    pub controller_type: ControllerType,
    /// The A/B and X/Y functions swapped, so the help names the other buttons.
    pub swap: bool,
    pub help: HelpContext,
    /// ES-DE's quick-scroll overlay over a held textlist.
    pub scroll_overlay: bool,
    /// The gameselectors' random picks start from this, set by the host; each navigation of the view draws again.
    pub selection_seed: u64,
    /// The face of a text whose theme names none: ES-DE's own where the host found it (`fonts`), else the painter's body face.
    pub default_font: Option<String>,
    /// The face of the help where its theme names none: ES-DE's help face beside its default (`fonts::help_font`).
    pub help_font: Option<String>,
    /// The player's media viewer settings.
    pub viewer: super::viewer::ViewerSettings,
}

impl Data {
    pub fn new(systems: Vec<System>, width: f64, height: f64) -> Self {
        Data {
            systems,
            width,
            height,
            system_index: 0,
            game_index: 0,
            now: DateTime::new(2026, 9, 24, 12, 0, 0),
            hide_metadata: false,
            show_clock: true,
            show_help: true,
            motion: Motion::esde(),
            status: DeviceStatus { wifi: Some(true), battery_percent: Some(80), ..DeviceStatus::default() },
            status_shown: Indicators::default(),
            controller_type: ControllerType::Xbox,
            swap: false,
            help: HelpContext::default(),
            scroll_overlay: false,
            selection_seed: 0,
            default_font: None,
            help_font: None,
            viewer: super::viewer::ViewerSettings::default(),
        }
    }

    pub fn system(&self) -> &System {
        &self.systems[self.system_index.min(self.systems.len().saturating_sub(1))]
    }

    pub fn game(&self) -> Option<&Game> {
        let games = &self.system().games;
        games.get(self.game_index.min(games.len().saturating_sub(1)))
    }
}
