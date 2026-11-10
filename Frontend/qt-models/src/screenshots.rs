//! The player's own screenshots (EmuSen_Qt.md §7.8; Mistress's `MediaLibrary.Screenshots`): the one owner of their
//! names, written by `SessionModel.screenshot` and read back for a game's page, in Galaxia's screenshots folder.
//! A screenshot is "<the game's file stem> <yyyy-MM-dd HH.mm.ss.zzz>.png", the local time to the millisecond.

use std::path::Path;

/// The stamp's date and time as `QDateTime` formats them.
pub const DATE: &str = "yyyy-MM-dd";
pub const TIME: &str = "HH.mm.ss.zzz";
/// The stamp's length in characters, "2026-11-09 14.03.11.251".
const STAMP: usize = 23;
pub const EXTENSION: &str = "png";

/// A screenshot's file name from the game's file and the stamp's two parts.
pub fn name(game: &str, date: &str, time: &str) -> String {
    format!("{} {date} {time}.{EXTENSION}", stem_of(game))
}

fn stem_of(path: &str) -> String {
    Path::new(path).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

/// The stem a screenshot names: its name less the stamp, or the whole name for a picture not named so.
fn named(file: &str) -> String {
    let chars: Vec<char> = file.chars().collect();
    if chars.len() > STAMP + 1 && chars[chars.len() - STAMP - 1] == ' ' {
        chars[..chars.len() - STAMP - 1].iter().collect()
    } else {
        file.to_owned()
    }
}

/// The game's screenshots in `folder`, each file with its modified time in seconds since 1970, the newest first.
pub fn of_game(folder: &Path, game: &str) -> Vec<(String, i64)> {
    let stem = stem_of(game);
    let Ok(entries) = std::fs::read_dir(folder) else { return Vec::new() };
    let mut found: Vec<(String, i64)> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case(EXTENSION)))
        .filter(|p| p.file_stem().is_some_and(|s| named(&s.to_string_lossy()) == stem))
        .filter_map(|p| {
            let at = std::fs::metadata(&p).ok()?.modified().ok()?;
            let at = at.duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64);
            Some((p.to_string_lossy().into_owned(), at))
        })
        .collect();
    found.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.0.cmp(&a.0)));
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_read_back_to_its_game_and_another_picture_names_itself() {
        let name = name("/roms/snes/Cobalt Harbor.sfc", "2026-11-09", "14.03.11.251");
        assert_eq!(name, "Cobalt Harbor 2026-11-09 14.03.11.251.png");
        assert_eq!(named(name.trim_end_matches(".png")), "Cobalt Harbor");
        assert_eq!(named("Cobalt Harbor"), "Cobalt Harbor");
        assert_eq!(named("Cobalt Harbor 2"), "Cobalt Harbor 2");
    }
}
