//! What a view is drawn from besides the theme: the systems and their games, the selection, the screen and the time
//! (EmuSen_BigPicture.md §13.2). Plain records the host fills from its models.

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
}

impl System {
    pub fn new(system: ThemeSystem, theme: Theme, games: Vec<Game>) -> Self {
        System { system, theme, games, heading: None }
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
