//! The library's texts as the scene reads them: `LibraryModel.esdeShelves`, `esdeMedia` and `esdeSettings` parsed into
//! the scene's systems and games and the loader's choices (EmuSen_Qt.md §4.6, §4.8). Plain Rust; no Qt.

use std::collections::BTreeMap;

use emusen_esde::loader::{Choices, MediaPresence, SystemKind, ThemeSystem};
use emusen_esde::scene::{Counts, DateTime, DeviceStatus, Game};
use serde_json::Value;

/// One shelf of the shelves' text, without its theme: the system it is, its games and its list's facts.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Shelf {
    pub system: Option<ThemeSystem>,
    pub games: Vec<Game>,
    pub stars: bool,
    pub favorites_on_top: bool,
    pub counts: Option<Counts>,
}

/// The ES-DE view's settings: the theme folder, the choices stored for it, and the interface's switches.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub theme: String,
    pub choices: Choices,
    pub show_clock: bool,
    pub show_help: bool,
    pub scroll_overlay: bool,
    pub quick_select: String,
    pub startup_view: String,
    /// The random entry's button: games, gamessystems or disabled (ES-DE's Random entry button).
    pub random: String,
    /// X starts the screensaver from the system view (ES-DE's Enable screensaver controls).
    pub screensaver_controls: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: String::new(),
            choices: Choices::default(),
            show_clock: false,
            show_help: true,
            scroll_overlay: false,
            quick_select: "leftrightshoulders".into(),
            startup_view: "system".into(),
            random: "games".into(),
            screensaver_controls: true,
        }
    }
}

fn text(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).filter(|s| !s.is_empty()).map(str::to_owned)
}

fn flag(v: &Value, key: &str) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(false)
}

/// A stored choice by Mistress's key (`Variant`, `ColorScheme`, …), or its lower-camel form.
fn choice(v: &Value, key: &str) -> Option<String> {
    let lower = format!("{}{}", key[..1].to_lowercase(), &key[1..]);
    text(v, key).or_else(|| text(v, &lower))
}

/// `esdeSettings` as `esde_shelves::settings_of` writes it; a text that does not read gives the defaults.
pub fn settings(text_in: &str) -> Settings {
    let v: Value = serde_json::from_str(text_in).unwrap_or(Value::Null);
    let c = v.get("choices").cloned().unwrap_or(Value::Null);
    let i = v.get("interface").cloned().unwrap_or(Value::Null);
    let on = |key: &str, default: bool| i.get(key).and_then(Value::as_bool).unwrap_or(default);
    let d = Settings::default();
    Settings {
        theme: text(&v, "theme").unwrap_or_default(),
        choices: Choices {
            variant: choice(&c, "Variant"),
            color_scheme: choice(&c, "ColorScheme"),
            font_size: choice(&c, "FontSize"),
            language: choice(&c, "Language"),
            aspect_ratio: choice(&c, "AspectRatio"),
            transitions: choice(&c, "Transitions"),
            ..Choices::default()
        },
        show_clock: on("DisplayClock", d.show_clock),
        show_help: on("DisplayHelp", d.show_help),
        scroll_overlay: on("ListScrollOverlay", d.scroll_overlay),
        quick_select: text(&i, "QuickSystemSelect").unwrap_or(d.quick_select),
        startup_view: text(&i, "StartupView").unwrap_or(d.startup_view),
        random: v.get("collections").and_then(|c| text(c, "RandomEntryButton")).unwrap_or(d.random),
        screensaver_controls: on("ScreensaverControls", d.screensaver_controls),
    }
}

/// The device's status as the host writes it, `{"wifi":true,"battery":80,"charging":false}`: a key left out is a radio
/// or battery the device lacks, and an empty text shows no indicator at all.
pub fn device(text_in: &str) -> DeviceStatus {
    let v: Value = serde_json::from_str(text_in).unwrap_or(Value::Null);
    let radio = |key: &str| v.get(key).and_then(Value::as_bool);
    DeviceStatus {
        bluetooth: radio("bluetooth"),
        wifi: radio("wifi"),
        cellular: radio("cellular"),
        battery_percent: v.get("battery").and_then(Value::as_u64).map(|b| b.min(100) as u8),
        charging: radio("charging").unwrap_or(false),
    }
}

/// A date of the metadata's `yyyy-MM-dd` (or `yyyyMMddTHHmmss`) form.
pub fn date(text_in: &str) -> Option<DateTime> {
    let digits: String = text_in.chars().filter(char::is_ascii_digit).collect();
    let year = digits.get(0..4)?.parse().ok()?;
    let month = digits.get(4..6).and_then(|m| m.parse().ok()).unwrap_or(1);
    let day = digits.get(6..8).and_then(|d| d.parse().ok()).unwrap_or(1);
    Some(DateTime::new(year, month, day, 0, 0, 0))
}

/// Seconds since 1970 as a civil date and time in UTC (Howard Hinnant's days-to-civil).
pub fn from_unix(seconds: i64) -> DateTime {
    let days = seconds.div_euclid(86_400);
    let rest = seconds.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = (yoe + era * 400 + i64::from(month <= 2)) as i32;
    DateTime::new(year, month, day, (rest / 3600) as u32, (rest % 3600 / 60) as u32, (rest % 60) as u32)
}

/// A game's media by ES-DE type: its folder and stem first, then its stem, under its own console's folder.
fn media_of(media: &Value, system: &str, folder_path: &str, file: &str) -> BTreeMap<String, String> {
    let stem = std::path::Path::new(file).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let keyed = if folder_path.is_empty() { None } else { Some(format!("{}/{stem}", folder_path.trim_matches('/'))) };
    let mut out = BTreeMap::new();
    if let Some(types) = media.get(system).and_then(Value::as_object) {
        for (kind, names) in types {
            let found = keyed.as_deref().and_then(|k| names.get(k)).or_else(|| names.get(&stem)).and_then(Value::as_str);
            if let Some(path) = found {
                out.insert(kind.clone(), path.to_owned());
            }
        }
    }
    out
}

fn game_of(v: &Value, media: &Value) -> Game {
    let file = text(v, "file").unwrap_or_default();
    let name = text(v, "name").unwrap_or_else(|| file.clone());
    let system = text(v, "esde_system").unwrap_or_default();
    let folder_path = text(v, "folder_path").unwrap_or_default();
    let count = |key: &str| v.get(key).and_then(Value::as_u64).unwrap_or(0);
    let last = v.get("last_played").and_then(Value::as_i64).unwrap_or(0);
    Game {
        description: text(v, "description"),
        developer: text(v, "developer"),
        publisher: text(v, "publisher"),
        genre: text(v, "genre"),
        players: text(v, "players"),
        release_date: text(v, "release_date").and_then(|d| date(&d)),
        last_played: (last > 0).then(|| from_unix(last)),
        rating: v.get("rating").and_then(Value::as_f64).map(|r| r as f32),
        play_time_seconds: v.get("play_time").and_then(Value::as_u64),
        play_count: count("play_count") as u32,
        favorite: flag(v, "favourite"),
        completed: flag(v, "completed"),
        folder: flag(v, "folder"),
        kid_game: flag(v, "kid_game"),
        broken: flag(v, "broken"),
        emulator: text(v, "emulator"),
        hide_metadata: flag(v, "hide_metadata"),
        not_counted: flag(v, "not_counted"),
        media: media_of(media, &system, &folder_path, &file),
        controller: text(v, "controller"),
        manual: flag(v, "manual"),
        alt_emulator: flag(v, "alt_emulator"),
        folder_link: text(v, "folder_link"),
        is_collection: flag(v, "is_collection"),
        in_collection: flag(v, "in_collection"),
        ..Game::new(&name, &file)
    }
}

/// `esdeShelves` with `esdeMedia`: each shelf's system and games; a collection's game by its file alone takes the row its
/// console's shelf carries (EmuSen_Qt.md §4.6).
pub fn shelves(shelves_text: &str, media_text: &str) -> Vec<Shelf> {
    let all: Value = serde_json::from_str(shelves_text).unwrap_or(Value::Null);
    let media: Value = serde_json::from_str(media_text).unwrap_or(Value::Null);
    let list = all.as_array().cloned().unwrap_or_default();
    let mut whole: BTreeMap<String, Value> = BTreeMap::new();
    for shelf in &list {
        for g in shelf.get("games").and_then(Value::as_array).into_iter().flatten() {
            if g.get("name").is_some() {
                if let Some(file) = text(g, "file") {
                    whole.entry(file).or_insert_with(|| g.clone());
                }
            }
        }
    }
    let systems: BTreeMap<String, ThemeSystem> = list
        .iter()
        .filter_map(|s| Some((text(s, "name")?, ThemeSystem::new(&text(s, "name")?, &text(s, "full_name")?, &text(s, "theme")?))))
        .collect();
    list.iter()
        .map(|s| {
            let kind = match text(s, "kind").as_deref() {
                Some("auto") => SystemKind::AutoCollection,
                Some("custom") => SystemKind::CustomCollection,
                _ => SystemKind::Regular,
            };
            let system = (|| Some(ThemeSystem::of_kind(&text(s, "name")?, &text(s, "full_name")?, &text(s, "theme")?, kind)))();
            let games = s
                .get("games")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(|g| {
                    let row = if g.get("name").is_some() { g } else { text(g, "file").and_then(|f| whole.get(&f)).unwrap_or(g) };
                    let mut game = game_of(row, &media);
                    if kind != SystemKind::Regular {
                        game.source = text(row, "esde_system").and_then(|n| systems.get(&n).cloned());
                    }
                    game
                })
                .collect();
            let counts = s.get("counts").map(|c| Counts {
                games: c.get("games").and_then(Value::as_u64).unwrap_or(0) as usize,
                favorites: c.get("favorites").and_then(Value::as_u64).unwrap_or(0) as usize,
                shown: c.get("shown").and_then(Value::as_u64).unwrap_or(0) as usize,
                filtered: flag(c, "filtered"),
                in_folder: flag(c, "in_folder"),
            });
            Shelf { system, games, stars: flag(s, "stars"), favorites_on_top: flag(s, "favourites_on_top"), counts }
        })
        .collect()
}

/// `esdePresence`: the media types some game of each shelf has, for the variant triggers.
pub fn presence(text_in: &str) -> BTreeMap<String, MediaPresence> {
    let v: Value = serde_json::from_str(text_in).unwrap_or(Value::Null);
    let mut out = BTreeMap::new();
    for (shelf, types) in v.as_object().into_iter().flatten() {
        let types: Vec<&str> = types.as_array().into_iter().flatten().filter_map(Value::as_str).collect();
        out.insert(shelf.clone(), MediaPresence::of(&types));
    }
    out
}
