//! The library's texts as the scene reads them: `LibraryModel.esdeShelves`, `esdeMedia` and `esdeSettings` parsed into
//! the scene's systems and games and the loader's choices (EmuSen_Qt.md §4.6, §4.8). Plain Rust; no Qt.

use std::collections::BTreeMap;

use emusen_esde::loader::{Choices, MediaPresence, SystemKind, ThemeSystem};
use emusen_esde::scene::{Counts, DateTime, DeviceStatus, Game, HelpPlace, ViewerSettings};
use emusen_esde::shelves::{GameRow, ShelfText, Text};
use serde_json::Value;

/// One shelf of the shelves' text, without its theme: the system it is, its games and its list's facts.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Shelf {
    pub system: Option<ThemeSystem>,
    pub games: Vec<Game>,
    pub stars: bool,
    pub favorites_on_top: bool,
    pub counts: Option<Counts>,
    /// The games the listed level does not show: inside its folders, or outside the folder open (EmuSen_Qt.md §4.11).
    pub unlisted: Vec<Game>,
}

/// The ES-DE view's settings: the theme folder, the choices stored for it, and the interface's switches.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub theme: String,
    pub choices: Choices,
    pub show_clock: bool,
    pub show_help: bool,
    pub scroll_overlay: bool,
    /// ES-DE's ViewsVideoAudio: a theme's videos in the system and gamelist views play their sound.
    pub views_video_audio: bool,
    pub quick_select: String,
    pub startup_view: String,
    /// The random entry's button: games, gamessystems or disabled (ES-DE's Random entry button).
    pub random: String,
    /// X starts the screensaver from the system view (ES-DE's Enable screensaver controls).
    pub screensaver_controls: bool,
    /// The media viewer's settings (ES-DE's Media viewer settings).
    pub viewer: ViewerSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: String::new(),
            choices: Choices::default(),
            show_clock: false,
            show_help: true,
            scroll_overlay: false,
            views_video_audio: true,
            quick_select: "leftrightshoulders".into(),
            startup_view: "system".into(),
            random: "games".into(),
            screensaver_controls: true,
            viewer: ViewerSettings::default(),
        }
    }
}

fn text(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).filter(|s| !s.is_empty()).map(str::to_owned)
}

/// A stored choice by Mistress's key (`Variant`, `ColorScheme`, …), or its lower-camel form.
fn choice(v: &Value, key: &str) -> Option<String> {
    let lower = format!("{}{}", key[..1].to_lowercase(), &key[1..]);
    text(v, key).or_else(|| text(v, &lower))
}

/// One rule for the view's four texts and the device's: no text is the default (the property's own, before the host sets
/// it), and a text that does not read is the words why, never read as the default (EmuSen_Qt.md §4.15).
fn read(text_in: &str, what: &str) -> Result<Value, String> {
    match text_in.trim() {
        "" => Ok(Value::Null),
        text => serde_json::from_str(text).map_err(|e| format!("{what} could not be read: {e}")),
    }
}

/// `esdeSettings` as `esde_shelves::settings_of` writes it; no text gives the defaults.
pub fn settings(text_in: &str) -> Result<Settings, String> {
    let v = read(text_in, "The view's settings")?;
    let c = v.get("choices").cloned().unwrap_or(Value::Null);
    let i = v.get("interface").cloned().unwrap_or(Value::Null);
    let on = |key: &str, default: bool| i.get(key).and_then(Value::as_bool).unwrap_or(default);
    let d = Settings::default();
    Ok(Settings {
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
        views_video_audio: on("ViewsVideoAudio", d.views_video_audio),
        quick_select: text(&i, "QuickSystemSelect").unwrap_or(d.quick_select),
        startup_view: text(&i, "StartupView").unwrap_or(d.startup_view),
        random: v.get("collections").and_then(|c| text(c, "RandomEntryButton")).unwrap_or(d.random),
        screensaver_controls: on("ScreensaverControls", d.screensaver_controls),
        viewer: ViewerSettings {
            help: text(&i, "MediaViewerHelpPrompts").map_or(d.viewer.help, |w| HelpPlace::of(&w)),
            show_types: on("MediaViewerShowTypes", d.viewer.show_types),
            keep_video_running: on("MediaViewerKeepVideoRunning", d.viewer.keep_video_running),
            stretch_videos: on("MediaViewerStretchVideos", d.viewer.stretch_videos),
            blur: on("MediaViewerVideoBlur", d.viewer.blur),
            audio: on("MediaViewerVideoAudio", d.viewer.audio),
            volume: v.get("volume").and_then(Value::as_f64).map_or(d.viewer.volume, |x| x.clamp(0.0, 1.0)),
        },
    })
}

/// The device's status as the host writes it, `{"wifi":true,"battery":80,"charging":false}`: a key left out is a radio
/// or battery the device lacks, and no text shows no indicator at all.
pub fn device(text_in: &str) -> Result<DeviceStatus, String> {
    let v = read(text_in, "The device's status")?;
    let radio = |key: &str| v.get(key).and_then(Value::as_bool);
    Ok(DeviceStatus {
        bluetooth: radio("bluetooth"),
        wifi: radio("wifi"),
        cellular: radio("cellular"),
        battery_percent: v.get("battery").and_then(Value::as_u64).map(|b| b.min(100) as u8),
        charging: radio("charging").unwrap_or(false),
    })
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

// A text field as the scene reads it: none where empty.
fn some(t: &Option<Text>) -> Option<String> {
    t.as_deref().filter(|s| !s.is_empty()).map(str::to_owned)
}

fn game_of(v: &GameRow, media: &Value) -> Game {
    let file = v.file.to_string();
    let name = if v.name.is_empty() { file.clone() } else { v.name.to_string() };
    let last = v.last_played;
    Game {
        description: some(&v.description),
        developer: some(&v.developer),
        publisher: some(&v.publisher),
        genre: some(&v.genre),
        players: some(&v.players),
        release_date: some(&v.release_date).and_then(|d| date(&d)),
        last_played: (last > 0).then(|| from_unix(last)),
        rating: v.rating.map(|r| r as f32),
        play_time_seconds: Some(u64::try_from(v.play_time).unwrap_or(0)),
        play_count: u32::try_from(v.play_count).unwrap_or(if v.play_count < 0 { 0 } else { u32::MAX }),
        favorite: v.favourite,
        completed: v.completed,
        folder: v.folder,
        kid_game: v.kid_game,
        broken: v.broken,
        emulator: some(&v.emulator),
        hide_metadata: v.hide_metadata,
        not_counted: v.not_counted,
        media: media_of(media, &v.esde_system, &v.folder_path, &file),
        controller: some(&v.controller),
        manual: v.manual,
        alt_emulator: v.alt_emulator,
        folder_link: some(&v.folder_link),
        is_collection: v.is_collection,
        in_collection: v.in_collection,
        ..Game::new(&name, &file)
    }
}

/// `esdeShelves` with `esdeMedia`: each shelf's system and games; a collection's game by its file alone takes the row its
/// console's shelf carries (EmuSen_Qt.md §4.6). The text is read straight into rows, never into a tree of values, since
/// at a library's scale the tree was 150 MiB for a 6.6 MB text (§4.15). A text that does not read is the words why, which
/// the model shows, never an empty library; no shelves text is no shelves, and no media index text no media (a host that
/// gives none).
pub fn shelves(shelves_text: &str, media_text: &str) -> Result<Vec<Shelf>, String> {
    let list: Vec<ShelfText> = match shelves_text.trim() {
        "" => Vec::new(),
        text => serde_json::from_str(text).map_err(|e| format!("The library's shelves could not be read: {e}"))?,
    };
    let media = read(media_text, "The library's media index")?;
    let mut whole: BTreeMap<&str, &GameRow> = BTreeMap::new();
    for g in list.iter().flat_map(|s| &s.games).filter(|g| g.is_whole() && !g.file.is_empty()) {
        whole.entry(&g.file).or_insert(g);
    }
    let named = |t: &str| (!t.is_empty()).then(|| t.to_owned());
    let system_of = |s: &ShelfText, kind: SystemKind| Some(ThemeSystem::of_kind(&named(&s.name)?, &named(&s.full_name)?, &named(&s.theme)?, kind));
    let systems: BTreeMap<String, ThemeSystem> =
        list.iter().filter_map(|s| Some((named(&s.name)?, system_of(s, SystemKind::Regular)?))).collect();
    Ok(list
        .iter()
        .map(|s| {
            let kind = match &*s.kind {
                "auto" => SystemKind::AutoCollection,
                "custom" => SystemKind::CustomCollection,
                _ => SystemKind::Regular,
            };
            let games = s
                .games
                .iter()
                .map(|g| {
                    let row = if g.is_whole() { g } else { whole.get(&*g.file).copied().unwrap_or(g) };
                    let mut game = game_of(row, &media);
                    if kind != SystemKind::Regular {
                        game.source = systems.get(&*row.esde_system).cloned();
                    }
                    game
                })
                .collect();
            let counts = s.counts.map(|c| Counts { games: c.games, favorites: c.favorites, shown: c.shown, filtered: c.filtered, in_folder: c.in_folder });
            let unlisted = s.unlisted.iter().map(|g| game_of(g, &media)).collect();
            Shelf { system: system_of(s, kind), games, stars: s.stars, favorites_on_top: s.favourites_on_top, counts, unlisted }
        })
        .collect())
}

/// `esdePresence`: the media types some game of each shelf has, for the variant triggers; no text is no media, and a text
/// that does not read is said in the words returned.
pub fn presence(text_in: &str) -> Result<BTreeMap<String, MediaPresence>, String> {
    let v = read(text_in, "The library's media presence")?;
    let mut out = BTreeMap::new();
    for (shelf, types) in v.as_object().into_iter().flatten() {
        let types: Vec<&str> = types.as_array().into_iter().flatten().filter_map(Value::as_str).collect();
        out.insert(shelf.clone(), MediaPresence::of(&types));
    }
    Ok(out)
}
