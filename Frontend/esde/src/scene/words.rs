//! What a text, datetime or clock element says: a game's field, the system's numbers, a date by strftime, in the
//! project's own words where ES-DE prints its own resources (EmuSen_BigPicture.md §13.2, §39).

use crate::loader::Element;

use super::data::{Counts, DateTime, Game, System};

pub const UNKNOWN: &str = "unknown";
pub const NEVER: &str = "never";
pub const YES: &str = "yes";
pub const NO: &str = "no";

/// "1 game", "12 games": a count and its noun, singular for one.
pub fn count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// THEMES.md's letterCase: uppercase, lowercase, capitalize, or none. Capitalize lowers the text and raises the first
/// letter of each word, a word being bounded as Unicode's word segmentation bounds it (ES-DE 3.4.1
/// `Utils::String::toCapitalized`, which walks ICU's word break iterator).
pub fn cased(text: &str, letter_case: Option<&str>) -> String {
    match letter_case {
        Some("uppercase") => text.to_uppercase(),
        Some("lowercase") => text.to_lowercase(),
        Some("capitalize") => capitalized(text),
        _ => text.to_owned(),
    }
}

fn capitalized(text: &str) -> String {
    let lower: Vec<char> = text.to_lowercase().chars().collect();
    let word = |c: char| c.is_alphanumeric();
    // UAX #29's WB6/WB7 and WB11/WB12: an apostrophe, full stop or colon between letters, or a comma or full stop
    // between digits, stays inside the word.
    let joins = |i: usize| {
        let (Some(&a), Some(&m), Some(&b)) = (lower.get(i.wrapping_sub(2)), lower.get(i - 1), lower.get(i)) else { return false };
        (a.is_alphabetic() && b.is_alphabetic() && matches!(m, '\'' | '\u{2019}' | '.' | ':' | '\u{b7}'))
            || (a.is_numeric() && b.is_numeric() && matches!(m, ',' | '.' | ';' | '\''))
    };
    let mut out = String::with_capacity(text.len());
    for (i, &c) in lower.iter().enumerate() {
        let starts = i == 0 || !(word(lower[i - 1]) && word(c) || joins(i));
        if starts {
            out.extend(c.to_uppercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// The collection suffix of a game's name: its own system's short name in brackets, upper case or capitalized as
/// letterCaseSystemNameSuffix says and as written for lowercase (ES-DE 3.4.1 `GamelistBase::populateList`'s nameSuffixFunc).
pub fn system_suffix(system: &str, letter_case: Option<&str>) -> String {
    let name = match letter_case.unwrap_or("uppercase") {
        "uppercase" => system.to_uppercase(),
        "capitalize" => capitalized(system),
        _ => system.to_owned(),
    };
    format!(" [{name}]")
}

const MONTHS: [&str; 12] =
    ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
const DAYS: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

/// A time by THEMES.md's strftime subset: %Y %y %m %d %e %H %I %M %S %p %b %h %B %a %A and %%; any other % pair stays as written.
pub fn strftime(value: &DateTime, format: &str) -> String {
    let mut out = String::new();
    let mut chars = format.chars();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        let Some(token) = chars.next() else {
            out.push('%');
            break;
        };
        let month = MONTHS[(value.month.clamp(1, 12) - 1) as usize];
        let day = DAYS[value.weekday() as usize];
        let hour12 = if value.hour.is_multiple_of(12) { 12 } else { value.hour % 12 };
        match token {
            'Y' => out.push_str(&format!("{:04}", value.year)),
            'y' => out.push_str(&format!("{:02}", value.year.rem_euclid(100))),
            'm' => out.push_str(&format!("{:02}", value.month)),
            'd' => out.push_str(&format!("{:02}", value.day)),
            'e' => out.push_str(&value.day.to_string()),
            'H' => out.push_str(&format!("{:02}", value.hour)),
            'I' => out.push_str(&format!("{hour12:02}")),
            'M' => out.push_str(&format!("{:02}", value.minute)),
            'S' => out.push_str(&format!("{:02}", value.second)),
            'p' => out.push_str(if value.hour < 12 { "AM" } else { "PM" }),
            'b' | 'h' => out.push_str(&month[..3]),
            'B' => out.push_str(month),
            'a' => out.push_str(&day[..3]),
            'A' => out.push_str(day),
            '%' => out.push('%'),
            other => {
                out.push('%');
                out.push(other);
            }
        }
    }
    out
}

/// The project's own words for a date relative to now.
pub fn relative(then: &DateTime, now: &DateTime) -> String {
    let ago = now.seconds() - then.seconds();
    let n = |unit: i64| ago / unit;
    let ago_in = |value: i64, unit: &str| format!("{value} {unit}{} ago", if value == 1 { "" } else { "s" });
    match ago {
        _ if ago < 0 => "in the future".into(),
        _ if ago < 60 => "just now".into(),
        _ if ago < 3600 => ago_in(n(60), "minute"),
        _ if ago < 86400 => ago_in(n(3600), "hour"),
        _ if ago < 30 * 86400 => ago_in(n(86400), "day"),
        _ if ago < 365 * 86400 => ago_in(n(86400) / 30, "month"),
        _ => ago_in(n(86400) / 365, "year"),
    }
}

pub fn play_time(seconds: u64) -> String {
    if seconds >= 3600 {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    } else if seconds >= 60 {
        format!("{}m", seconds / 60)
    } else {
        format!("{seconds}s")
    }
}

/// A game's name with the collection suffix THEMES.md's systemNameSuffix adds, in a collection only.
pub fn name_with_suffix(e: &Element, game: &Game, system: &System) -> String {
    if !game.folder && e.bool("systemNameSuffix") == Some(true) && system.is_collection() {
        let source = game.source.as_ref().unwrap_or(&system.system);
        return format!("{}{}", game.name, system_suffix(&source.name, e.string("letterCaseSystemNameSuffix")));
    }
    game.name.clone()
}

/// What a text element says, before its letter case: its text, else its metadata or system data, else its default; None draws nothing.
pub fn text(e: &Element, view: &str, system: &System, game: Option<&Game>) -> Option<String> {
    let value = e.string("text").map(str::to_owned).or_else(|| metadata(e, view, system, game)).or_else(|| system_data(e, system));
    let value = match value {
        Some(v) if !v.is_empty() => v,
        _ => e.string("defaultValue").map(str::to_owned).or_else(|| missing(e, game))?,
    };
    (!value.is_empty()).then_some(value)
}

/// What a datetime element says, before its letter case.
pub fn date_time(e: &Element, view: &str, game: Option<&Game>, now: &DateTime) -> Option<String> {
    let field = e.string("metadata");
    if game.is_none() && view == "system" && matches!(field, Some("releasedate" | "lastplayed")) {
        return None;
    }
    let date = game.and_then(|g| match field {
        Some("releasedate") => g.release_date,
        Some("lastplayed") => g.last_played,
        _ => None,
    });
    let value = match date {
        Some(d) if e.bool("displayRelative") == Some(true) => Some(relative(&d, now)),
        Some(d) => Some(strftime(&d, e.string("format").unwrap_or("%Y-%m-%d"))),
        None => e.string("defaultValue").map(str::to_owned).or_else(|| {
            game.and(match field {
                Some("releasedate") => Some(UNKNOWN.to_owned()),
                Some("lastplayed") => Some(NEVER.to_owned()),
                _ => None,
            })
        }),
    }?;
    (!value.is_empty()).then_some(value)
}

/// What a clock says at a time.
pub fn clock(e: &Element, now: &DateTime) -> String {
    strftime(now, e.string("format").unwrap_or("%H:%M"))
}

// What a game's empty field shows, where ES-DE 3.4.1 was measured to show a word (§39).
fn missing(e: &Element, game: Option<&Game>) -> Option<String> {
    (game.is_some() && matches!(e.string("metadata"), Some("developer" | "publisher" | "genre" | "players" | "playtime")))
        .then(|| UNKNOWN.to_owned())
}

fn metadata(e: &Element, view: &str, system: &System, game: Option<&Game>) -> Option<String> {
    let field = e.string("metadata")?;
    if matches!(field, "systemName" | "systemFullname") && view == "gamelist" {
        if let Some(heading) = &system.heading {
            return (!heading.is_empty()).then(|| heading.clone());
        }
    }
    let source = || game.and_then(|g| g.source.as_ref()).unwrap_or(&system.system);
    match field {
        "systemName" => return Some(system.system.name.clone()),
        "systemFullname" => return Some(system.system.full_name.clone()),
        "sourceSystemName" => return Some(source().name.clone()),
        "sourceSystemFullname" => return Some(source().full_name.clone()),
        _ => {}
    }
    let g = game?;
    let yes_no = |v: bool| Some(if v { YES } else { NO }.to_owned());
    let file_name = || std::path::Path::new(&g.file).file_name().map(|n| n.to_string_lossy().into_owned());
    match field {
        "name" => Some(name_with_suffix(e, g, system)),
        "description" => g.description.clone(),
        "rating" => g.rating.map(|r| format!("{}/5", one_decimal(r * 5.0))),
        "developer" => g.developer.clone(),
        "publisher" => g.publisher.clone(),
        "genre" => g.genre.clone(),
        "players" => g.players.clone(),
        "favorite" => yes_no(g.favorite),
        "completed" => yes_no(g.completed),
        "kidgame" => yes_no(g.kid_game),
        "broken" => yes_no(g.broken),
        "playcount" => Some(g.play_count.to_string()),
        "playtime" => g.play_time_seconds.filter(|&s| s > 0).map(play_time),
        "altemulator" | "emulator" => g.emulator.clone(),
        "physicalName" => std::path::Path::new(&g.file).file_stem().map(|n| n.to_string_lossy().into_owned()),
        "physicalNameExtension" => file_name(),
        _ => None,
    }
}

// "0.#": one decimal at most, none when it is zero.
fn one_decimal(value: f32) -> String {
    let text = format!("{value:.1}");
    text.strip_suffix(".0").map(str::to_owned).unwrap_or(text)
}

fn system_data(e: &Element, system: &System) -> Option<String> {
    let field = e.string("systemdata")?;
    let c = system.counts.unwrap_or_else(|| Counts::of(&system.games));
    let (games, favorites) = (c.games, c.favorites);
    Some(match field {
        "name" => system.system.name.clone(),
        "fullname" => system.system.full_name.clone(),
        // THEMES.md: "Number of favorites is printed inside brackets if applicable".
        "gamecount" => {
            let favs = if favorites > 0 { format!(" ({})", count(favorites, "favorite", "favorites")) } else { String::new() };
            format!("{}{favs}", count(games, "game", "games"))
        }
        "gamecountGames" => count(games, "game", "games"),
        "gamecountGamesNoText" => games.to_string(),
        "gamecountFavorites" => count(favorites, "favorite", "favorites"),
        "gamecountFavoritesNoText" => favorites.to_string(),
        _ => return None,
    })
}
