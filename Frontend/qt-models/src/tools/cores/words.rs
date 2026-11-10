//! Cores' words: the status a row says and its tone, sizes, dates, and the core tool's refusals in a player's words
//! (EmuSen_Interface.md §15.3; EmuSen_Cores.md §2.1, §4.2), one owner each, as C#'s `CoresWords`.

use super::row::Core;
use crate::tools::Tone;

/// The status in the list, a glyph beside a word in title case (EmuSen_Interface.md §15.3).
pub fn status(row: &Core) -> String {
    if row.is_default {
        return "● Default".to_owned();
    }
    if row.awaits_approval() {
        return "⚠ To approve".to_owned();
    }
    if row.trust.as_deref() == Some("needs_refresh") {
        return "⚠ Needs refresh".to_owned();
    }
    match row.state.as_str() {
        "update" => "↑ Update".to_owned(),
        "installed" => "✓ Installed".to_owned(),
        "available" => "Available".to_owned(),
        "unsupported" | "not_read" => "Not supported".to_owned(),
        "built in" | "manifest" => "Built in".to_owned(),
        "added" | "approved" => "✓ Added by you".to_owned(),
        s => capital(s),
    }
}

/// The status's tone.
pub fn tone(row: &Core) -> Tone {
    if row.is_default {
        Tone::Text
    } else if row.awaits_approval() || row.trust.as_deref() == Some("needs_refresh") || row.state == "update" {
        Tone::Attention
    } else if matches!(row.state.as_str(), "installed" | "added" | "approved") {
        Tone::Success
    } else {
        Tone::Muted
    }
}

/// Bytes as the tool says them: 41 MB, 8 MB, 3 MB, 410 GB.
pub fn size(bytes: u64) -> String {
    let b = bytes as f64;
    match bytes {
        n if n < 1024 => format!("{n} B"),
        n if n < 1024 * 1024 => format!("{:.0} KB", b / 1024.0),
        n if n < 1024 * 1024 * 1024 => one_place(b / 1_048_576.0, "MB"),
        _ => one_place(b / 1_073_741_824.0, "GB"),
    }
}

/// At most one decimal place, none where it is zero: "0.#" in .NET's format.
fn one_place(value: f64, unit: &str) -> String {
    let rounded = (value * 10.0).round() / 10.0;
    if rounded.fract() == 0.0 {
        format!("{rounded:.0} {unit}")
    } else {
        format!("{rounded:.1} {unit}")
    }
}

const MONTHS: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

fn date(text: &str) -> Option<(u32, usize, u32)> {
    let mut parts = text.split('-');
    let (y, m, d) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() || y.len() != 4 || m.len() != 2 || d.len() != 2 {
        return None;
    }
    let (y, m, d) = (y.parse().ok()?, m.parse::<usize>().ok()?, d.parse().ok()?);
    (1..=12).contains(&m).then_some((y, m, d)).filter(|_| (1..=31).contains(&d))
}

/// A date the tool wrote as 2026-10-31, said as 31 Oct 2026; any other words as they are.
pub fn day(text: &str) -> String {
    date(text).map_or_else(|| text.to_owned(), |(y, m, d)| format!("{d} {} {y}", &MONTHS[m - 1][..3]))
}

/// A date the tool wrote as 2026-10-31, said in full as 31 October 2026; none for anything else.
pub fn long_day(text: &str) -> Option<String> {
    date(text).map(|(y, m, d)| format!("{d} {} {y}", MONTHS[m - 1]))
}

/// The consoles a row runs, as a list shows them; a dash for none.
pub fn consoles(row: &Core) -> String {
    if row.console_names.is_empty() {
        "—".to_owned()
    } else {
        row.console_names.join(", ")
    }
}

/// The tool's refusal of a whole request, as the screen says it.
pub fn refusal(tool: &str) -> String {
    if tool == "no buildbot listing is known for this platform" {
        return "libretro publishes no cores for this kind of machine that EmuSen knows of, so only EmuSen's own cores are listed.".to_owned();
    }
    if tool.starts_with("another Cores job") {
        return "Another Cores job is changing the cores folder, in this program or another. Try again when it ends.".to_owned();
    }
    let said = capital(tool);
    if said.is_empty() || said.ends_with('.') {
        said
    } else {
        said + "."
    }
}

/// "a PlayStation 2 BIOS", "an Atari BIOS": words with their article.
pub fn article(words: &str) -> String {
    let an = words.chars().next().is_some_and(|c| "aeiouAEIOU".contains(c));
    format!("{} {words}", if an { "an" } else { "a" })
}

pub fn capital(words: &str) -> String {
    let mut chars = words.chars();
    chars.next().map(|c| c.to_uppercase().chain(chars).collect()).unwrap_or_default()
}

/// The hash in groups of eight, four to a line.
pub fn grouped(sha: &str) -> String {
    let groups: Vec<&str> = sha.as_bytes().chunks(8).map(|g| std::str::from_utf8(g).unwrap_or_default()).collect();
    groups.chunks(4).map(|line| line.join(" ")).collect::<Vec<_>>().join("\n")
}
