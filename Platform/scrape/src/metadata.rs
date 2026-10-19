//! A game's metadata as every view shows it: the player's edit, else ScreenScraper's text, else the file's,
//! `GameMetadata.Resolve` ported. The edits are the library's (lent), the scraped record the store's; the field table's
//! labels and choices are the editor's words and stay with the frontend. See EmuSen_Settings_Reference.md §4.59 and
//! EmuSen_Backend.md §30.6.

use crate::store::Record;

/// The editable fields' keys in the order ES-DE's user guide lists them, `GameMetadata.Fields`.
pub const FIELDS: [&str; 19] = [
    "name", "sortname", "collectionsortname", "description", "rating", "releasedate", "developer", "publisher", "genre", "players", "completed", "kidgame", "hidden", "broken",
    "nogamecount", "nomultiscrape", "hidemetadata", "controller", "altemulator",
];

const FLAGS: [&str; 7] = ["completed", "kidgame", "hidden", "broken", "nogamecount", "nomultiscrape", "hidemetadata"];

/// `float.ToString("0.###")` of a rating clamped to 0 to 1: seven significant digits, then three decimals, zeros dropped.
pub fn format_rating(rating: f32) -> String {
    let v = rating.clamp(0.0, 1.0) as f64;
    let seven: f64 = format!("{v:.6e}").parse().unwrap_or(v);
    let text = format!("{seven:.3}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text.is_empty() || text == "-0" { "0".to_string() } else { text.to_string() }
}

fn scraped(r: Option<&Record>, field: &str, scraped_names: bool) -> Option<String> {
    let r = r?;
    match field {
        "name" if scraped_names => r.name.clone().filter(|n| !n.is_empty()),
        "description" => r.description.clone(),
        "developer" => r.developer.clone(),
        "publisher" => r.publisher.clone(),
        "genre" => r.genre.clone(),
        "players" => r.players.clone(),
        "rating" => r.rating.map(format_rating),
        "releasedate" => r.release_date.clone(),
        _ => None,
    }
}

fn default(rom: &str, field: &str) -> Option<String> {
    match field {
        "name" => Some(emusen_galaxia::dotnet_path::file_name_without_extension(emusen_galaxia::dotnet_path::Style::HOST, rom).to_string()),
        f if FLAGS.contains(&f) => Some("0".to_string()),
        _ => None,
    }
}

/// Each field's value and its source: `edited`, `scraped` or `default`.
pub fn resolve(rom: &str, record: Option<&Record>, edits: &[(String, String)], scraped_names: bool) -> Vec<(&'static str, Option<String>, &'static str)> {
    FIELDS
        .iter()
        .map(|&field| match edits.iter().find(|(k, _)| k == field) {
            Some((_, v)) => (field, Some(v.clone()), "edited"),
            None => match scraped(record, field, scraped_names) {
                Some(s) => (field, Some(s), "scraped"),
                None => (field, default(rom, field), "default"),
            },
        })
        .collect()
}
