//! The words the screens show that no row carries, written once (EmuSen_Qt.md §1.2: QML holds no words), with the
//! counted phrases the rows' words are made of.

/// The wordmark: Moonrise's title and Home's (EmuSen_Interface.md §22, §24.3).
pub const TITLE: &str = "EMUSEN";

/// "1 game", "1,566 games": a count with its noun, grouped by thousands as the mockups write them.
pub fn counted(n: usize, one: &str, many: &str) -> String {
    format!("{} {}", grouped(n), if n == 1 { one } else { many })
}

/// A number grouped by thousands with commas.
pub fn grouped(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Moonrise's caption: the count of planets in capitals.
pub fn splash_caption(planets: usize) -> String {
    counted(planets, "CONSOLE", "CONSOLES")
}

/// Choose a console's title and line, as the courts canvas words them (EmuSen_Interface.md §24.1).
pub const GAMES: &str = "GAMES";
pub const CHOOSE_A_CONSOLE: &str = "Choose a console";
/// The grid's path before the console's name (§24.2).
pub const GAMES_PATH: &str = "GAMES ▸";
/// Home's headings (§24.3).
pub const CONTINUE_PLAYING: &str = "CONTINUE PLAYING";
pub const RECENT: &str = "RECENT";
/// Home's seven places, in order, as the accepted render words them (EmuSen_Qt.md §7.1).
pub const HOME_PLACES: [&str; 7] = ["Games", "Scraper", "Packages", "Library", "Cores", "Settings", "Power"];
/// The tools' keys among Home's places, in order, as a tool's summary names them.
pub const HOME_TOOLS: [&str; 4] = ["scraper", "packages", "library", "cores"];
/// What each tool does, said under its place until it has said its state.
pub const HOME_TOOL_ABOUT: [&str; 4] = ["Art and facts", "Game packages", "Checks dumps", "The engines"];
/// The Power place's line on the big screen.
pub const POWER_LINE: &str = "Exit";
/// What stands between the grid's two neighbours (§24.2).
pub const BETWEEN: &str = "·";
/// The search field's name, as a screen reader says it.
pub const SEARCH: &str = "Search";

/// The program's name and version, as the Home card's Power page says it under its rows (Mistress's `BigMenuFooter`).
pub fn version_line() -> String {
    format!("EmuSen {}", env!("EMUSEN_VERSION"))
}
/// The engine line's tags (§24.6).
pub const EMUSEN_TAG: &str = "EMUSEN";
pub const LIBRETRO_TAG: &str = "LIBRETRO";
/// The Favourites place's line.
pub const FAVOURITES_LINE: &str = "Your hearts, every console";

/// The empty states, as the themes' canvases word them: no game at all, nothing played, no console with games, and how
/// the player adds a folder.
pub const NO_GAMES: &str = "No games yet";
pub const NO_RECENT: &str = "Nothing played yet";
pub const NO_CONSOLES: &str = "No consoles have games yet";
pub const ADD_FOLDER: &str = "Settings › Library adds a folder";

/// The eyebrow over Home's places.
pub const PLACES: &str = "PLACES";

/// Continue and the Recent shelf as a list of destinations names them, beside the places (Meridian's rail).
pub const CONTINUE_ENTRY: &str = "Continue";
pub const RECENT_ENTRY: &str = "Recent";

/// A count of games.
pub fn games(n: usize) -> String {
    counted(n, "game", "games")
}

/// The Recently played place's line: the game played last, or none yet.
pub fn last_played(title: Option<&str>) -> String {
    title.map_or_else(|| "Nothing played yet".to_owned(), |t| format!("Last: {t}"))
}

/// Choose a console's summary at the top right: "11 consoles · 1,566 games".
pub fn summary(consoles: usize, games: usize) -> String {
    format!("{} · {}", counted(consoles, "console", "consoles"), self::games(games))
}

/// The facts' last words: which engine runs the console now.
pub fn runs_on(name: &str, libretro: bool) -> String {
    format!("runs on {name}, {}", if libretro { "a libretro core" } else { "an EmuSen core" })
}

/// A focused console's facts: "Nintendo · 1983 · 2 games · runs on Moon, an EmuSen core", the empty parts left out.
pub fn facts(maker: &str, year: i32, games: usize, runs_on: &str) -> String {
    let year = if year > 0 { year.to_string() } else { String::new() };
    [maker, year.as_str(), self::games(games).as_str(), runs_on].iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(" · ")
}

/// Now, in seconds since 1970.
pub fn now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64)
}

/// How long ago `at` was at `now`, both in seconds since 1970: "just now", "12 min ago", "3 h ago", "2 days ago".
pub fn ago(at: i64, now: i64) -> String {
    let minutes = (now - at).max(0) / 60;
    match minutes {
        0 => "just now".into(),
        1..=59 => format!("{minutes} min ago"),
        60..=1439 => format!("{} h ago", minutes / 60),
        _ => counted((minutes / 1440) as usize, "day ago", "days ago"),
    }
}

/// How long ago a game was paused, as Mistress's Home says it (HomeScreen.PausedAgo): "Paused just now", "Paused 1 minute
/// ago", "Paused 12 minutes ago", "Paused 3 hours ago", "Paused yesterday", "Paused 4 days ago".
pub fn paused_ago(at: i64, now: i64) -> String {
    let minutes = (now - at).max(0) / 60;
    match minutes {
        0 => "Paused just now".into(),
        1..=59 => format!("Paused {}", counted(minutes as usize, "minute ago", "minutes ago")),
        60..=1439 => format!("Paused {}", counted((minutes / 60) as usize, "hour ago", "hours ago")),
        1440..=2879 => "Paused yesterday".into(),
        _ => format!("Paused {} days ago", minutes / 1440),
    }
}

/// Continue's facts for a paused game: its console, where the library knows it, and how long ago it was paused.
pub fn paused_facts(console: &str, at: i64, now: i64) -> String {
    if console.is_empty() {
        paused_ago(at, now)
    } else {
        format!("{console} · {}", paused_ago(at, now))
    }
}

/// An ISO 8601 time in UTC, "2026-11-08T21:42:00Z" or with its fraction, as seconds since 1970; none for other text.
pub fn parse_utc(text: &str) -> Option<i64> {
    let b = text.as_bytes();
    let num = |r: std::ops::Range<usize>| text.get(r)?.parse::<i64>().ok();
    if b.len() < 19 || b[4] != b'-' || b[7] != b'-' || b[13] != b':' {
        return None;
    }
    let (y, m, d) = (num(0..4)?, num(5..7)?, num(8..10)?);
    let (hh, mm, ss) = (num(11..13)?, num(14..16)?, num(17..19)?);
    // Days from the civil date, Howard Hinnant's algorithm (http://howardhinnant.github.io/date_algorithms.html).
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let days = era * 146_097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719_468;
    Some(days * 86_400 + hh * 3_600 + mm * 60 + ss)
}

/// Seconds since 1970 as an ISO 8601 time in UTC, "2026-11-08T21:42:00Z", which `parse_utc` reads back.
pub fn utc(at: i64) -> String {
    let (days, secs) = (at.div_euclid(86_400), at.rem_euclid(86_400));
    // The civil date of a day count, Howard Hinnant's algorithm, the inverse of parse_utc's.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", secs / 3600, secs % 3600 / 60, secs % 60)
}

/// The Settings place's line: the look and the navigation sound, "Dark · 60% sound" (the C#'s SettingsLine).
pub fn settings_line(dark: bool, settings: &serde_json::Value) -> String {
    let sounds = settings.get("NavigationSounds").and_then(serde_json::Value::as_bool).unwrap_or(true);
    let volume = settings.pointer("/BigPictureInterface/NavigationVolume").and_then(serde_json::Value::as_i64).unwrap_or(100).clamp(0, 100);
    let sound = if sounds { format!("{volume}% sound") } else { "sound off".into() };
    format!("{} · {sound}", if dark { "Dark" } else { "Light" })
}

/// Home's places' second lines, each with whether it needs the player: the games' count, each tool's brief from its
/// summary (`{"scraper": {"brief": "41 no art", "attention": false}, ...}`) or what it does, the settings, and Exit.
pub fn home_lines(games: usize, tools: &serde_json::Value, settings: &str) -> Vec<(String, bool)> {
    let mut out = vec![(self::games(games), false)];
    for (key, about) in HOME_TOOLS.iter().zip(HOME_TOOL_ABOUT) {
        let summary = tools.get(*key);
        let brief = summary.and_then(|s| s.get("brief")).and_then(serde_json::Value::as_str).filter(|b| !b.is_empty());
        let attention = summary.and_then(|s| s.get("attention")).and_then(serde_json::Value::as_bool).unwrap_or(false);
        out.push(brief.map_or_else(|| (about.to_owned(), false), |b| (b.to_owned(), attention)));
    }
    out.push((settings.to_owned(), false));
    out.push((POWER_LINE.to_owned(), false));
    out
}

/// Home's status line: the controllers connected, when there are any, and the time (the C#'s device line).
pub fn status_line(controllers: usize, time: &str) -> String {
    if controllers == 0 {
        time.to_owned()
    } else {
        format!("{} · {time}", counted(controllers, "controller", "controllers"))
    }
}
