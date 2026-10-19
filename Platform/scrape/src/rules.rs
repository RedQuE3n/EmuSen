//! The region, language, genre, rating and date rules that decide what is kept from an answer, `ScrapeRules` ported.
//! Each is held to the C# by `tests/rules.rs`, whose tables were written by the C# on 2026-10-18. See
//! EmuSen_BigPicture.md §5.4 and §17.6.

use crate::answer::{Genre, Localised, Media};
use crate::kinds::{Kind, Payload};
use emusen_dotnet::text::{equals_ignore_case, is_blank};

pub const AUTOMATIC_REGION: &str = "auto";

/// ES-DE's documented fallback after the preferred region: world, USA, EU, Japan, then ScreenScraper's own.
pub const FALLBACK_REGIONS: [&str; 5] = ["wor", "us", "eu", "jp", "ss"];

/// No-Intro's country names to ScreenScraper's region codes.
const COUNTRIES: [(&str, &str); 18] = [
    ("World", "wor"), ("USA", "us"), ("Europe", "eu"), ("Japan", "jp"), ("Germany", "de"), ("France", "fr"), ("Spain", "sp"), ("Italy", "it"),
    ("Netherlands", "nl"), ("Sweden", "se"), ("Australia", "au"), ("Korea", "kr"), ("Brazil", "br"), ("China", "cn"), ("Asia", "asi"),
    ("Taiwan", "tw"), ("UK", "uk"), ("Canada", "ca"),
];

fn country(name: &str) -> Option<&'static str> {
    COUNTRIES.iter().find(|(n, _)| equals_ignore_case(n, name)).map(|(_, code)| *code)
}

/// The text inside each `(...)` holding no parenthesis, left to right, as the regex `\(([^()]*)\)` finds them.
fn tags(text: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let mut open: Option<usize> = None;
    for (i, c) in text.char_indices() {
        match c {
            '(' => open = Some(i + 1),
            ')' => {
                if let Some(start) = open.take() {
                    found.push(&text[start..i]);
                }
            }
            _ => {}
        }
    }
    found
}

/// The first tag whose every name is a country: `(USA, Europe)` is `us`, `(En,Fr,De)` is not a region.
pub fn region_of_file_name(file_name: &str) -> Option<&'static str> {
    for tag in tags(file_name) {
        let names: Vec<&str> = tag.split(',').map(|n| n.trim_matches(char::is_whitespace)).filter(|n| !n.is_empty()).collect();
        if !names.is_empty() && names.iter().all(|n| country(n).is_some()) {
            return country(names[0]);
        }
    }
    None
}

/// The regions tried, in order: the player's choice or the file's tag, then with the fallback on ES-DE's list.
pub fn region_order(file_name: &str, choice: &str, fallback: bool) -> Vec<String> {
    let preferred = if choice != AUTOMATIC_REGION && !is_blank(Some(choice)) { choice.to_string() } else { region_of_file_name(file_name).unwrap_or("wor").to_string() };
    let mut order = vec![preferred.clone()];
    if fallback {
        order.extend(FALLBACK_REGIONS.iter().filter(|r| **r != preferred).map(|r| r.to_string()));
    }
    order
}

pub fn language_order(preferred: &str) -> Vec<String> {
    if is_blank(Some(preferred)) || preferred == "en" { vec!["en".into()] } else { vec![preferred.to_string(), "en".into()] }
}

/// The file of one kind to fetch: each ScreenScraper type in turn, by region order; then one with no region; then any
/// region, only with the fallback on.
pub fn choose_media<'a>(media: &'a [Media], kind: &Kind, regions: &[String], fallback: bool) -> Option<&'a Media> {
    for ty in kind.screenscraper {
        let of_type: Vec<&Media> = media.iter().filter(|m| m.ty == *ty).collect();
        if of_type.is_empty() {
            continue;
        }
        for region in regions {
            if let Some(hit) = of_type.iter().find(|m| m.region.as_deref() == Some(region.as_str())) {
                return Some(hit);
            }
        }
        if let Some(plain) = of_type.iter().find(|m| m.region.as_deref().is_none_or(str::is_empty)) {
            return Some(plain);
        }
        if fallback {
            return Some(of_type[0]);
        }
    }
    None
}

/// A text by key order, then, only with the fallback on, any at all.
pub fn choose_text<'a>(texts: &'a [Localised], order: &[String], fallback: bool) -> Option<&'a Localised> {
    for key in order {
        if let Some(hit) = texts.iter().find(|t| t.key == *key) {
            return Some(hit);
        }
    }
    if fallback { texts.first() } else { None }
}

/// The main genre's name in the language order, else the first genre's.
pub fn genre(genres: &[Genre], languages: &[String]) -> Option<String> {
    let genre = genres.iter().find(|g| g.main).or_else(|| genres.first())?;
    choose_text(&genre.names, languages, true).map(|t| t.text.clone())
}

/// `Math.Round(value, 1, MidpointRounding.AwayFromZero)`: scaled by ten, a fraction of a half or more taken away from zero.
fn round_tenth_away(value: f64) -> f64 {
    let scaled = value * 10.0;
    let whole = scaled.trunc();
    let fraction = scaled - whole;
    let rounded = if fraction.abs() >= 0.5 { whole + fraction.signum() } else { whole };
    rounded / 10.0
}

/// ScreenScraper's note is out of 20; ES-DE keeps a fraction shown in half stars: note / 20 to the nearest tenth, as a
/// `float`, as `ScrapeRules.Rating` casts it.
pub fn rating(note: Option<f64>) -> Option<f32> {
    let n = note?;
    if !(n >= 0.0) {
        return None;
    }
    let clamped = if n > 20.0 { 20.0 } else { n };
    Some(round_tenth_away(clamped / 20.0) as f32)
}

/// A date as `yyyy-MM-dd`, `yyyy-MM` or `yyyy` exactly, ASCII digits, a real day; the text trimmed first.
pub fn date(text: Option<&str>) -> Option<(i32, u32, u32)> {
    let text = text.filter(|t| !is_blank(Some(t)))?.trim_matches(char::is_whitespace);
    let digits = |s: &str| -> Option<u32> { (s.bytes().all(|b| b.is_ascii_digit())).then(|| s.parse().ok()).flatten() };
    let parts: Vec<&str> = text.split('-').collect();
    let (year, month, day) = match parts.as_slice() {
        [y] if y.len() == 4 => (digits(y)?, 1, 1),
        [y, m] if y.len() == 4 && m.len() == 2 => (digits(y)?, digits(m)?, 1),
        [y, m, d] if y.len() == 4 && m.len() == 2 && d.len() == 2 => (digits(y)?, digits(m)?, digits(d)?),
        _ => return None,
    };
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return None,
    };
    (year >= 1 && (1..=days).contains(&day)).then_some((year as i32, month, day))
}

pub fn date_text(date: (i32, u32, u32)) -> String {
    format!("{:04}-{:02}-{:02}", date.0, date.1, date.2)
}

/// The extension a media file is written with: a manual's is pdf and a clip's mp4; a picture's ScreenScraper's format
/// when it is jpg, else png.
pub fn extension(format: Option<&str>, payload: Payload) -> &'static str {
    match payload {
        Payload::Pdf => ".pdf",
        Payload::Video => ".mp4",
        Payload::Image => match format.map(emusen_dotnet::text::to_lower).as_deref() {
            Some("jpg" | "jpeg") => ".jpg",
            _ => ".png",
        },
    }
}

/// ES-DE's search text from a file's name: the extension and every `(...)` and `[...]` taken out, underscores as spaces.
pub fn search_name(file_name: &str) -> String {
    let stem = emusen_galaxia::dotnet_path::file_name_without_extension(emusen_galaxia::dotnet_path::Style::HOST, file_name);
    let mut bare = String::with_capacity(stem.len());
    let chars: Vec<char> = stem.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let close = match chars[i] {
            '(' => chars[i + 1..].iter().position(|&c| c == ')' || c == '(').filter(|&p| chars[i + 1 + p] == ')'),
            '[' => chars[i + 1..].iter().position(|&c| c == ']' || c == '[').filter(|&p| chars[i + 1 + p] == ']'),
            _ => None,
        };
        match close {
            Some(p) => {
                bare.push(' ');
                i += p + 2;
            }
            None => {
                bare.push(if chars[i] == '_' { ' ' } else { chars[i] });
                i += 1;
            }
        }
    }
    let mut out = String::with_capacity(bare.len());
    let mut space = false;
    for c in bare.chars() {
        if c.is_whitespace() {
            if !space {
                out.push(' ');
            }
            space = true;
        } else {
            out.push(c);
            space = false;
        }
    }
    out.trim_matches(char::is_whitespace).to_string()
}
