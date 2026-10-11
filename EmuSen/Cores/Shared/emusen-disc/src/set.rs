//! The set module (EmuSen_CoreAPI_Extensions.md §2.3): which files a host must read beside an image, and the discs an
//! `.m3u` lists. Small readers only; what a sheet means is `cue.rs`'s and `disc.rs`'s.

use crate::cue::{CueError, Sheet};

/// The companions a sheet names, in its order: each FILE line's file, which a host passes as files
/// `EMUSEN_FILE_MEDIA + n` (Extensions §2.4).
pub fn companions(sheet: &str) -> Result<Vec<String>, CueError> {
    Ok(Sheet::parse(sheet)?.files)
}

/// An `.m3u`'s entries: one path a line, blank lines and `#` comments skipped, whitespace trimmed.
pub fn playlist(text: &str) -> Vec<String> {
    text.lines().map(|l| l.trim().trim_start_matches('\u{FEFF}')).filter(|l| !l.is_empty() && !l.starts_with('#')).map(str::to_owned).collect()
}

/// The `.sbi` found beside an image, by name: the image's stem with `.sbi` (Extensions §2.4).
pub fn sbi_beside(image: &str) -> String {
    let stem = match image.rfind('.') {
        Some(dot) if !image[dot..].contains(['/', '\\']) => &image[..dot],
        _ => image,
    };
    format!("{stem}.sbi")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sheets_companions_and_a_lists_entries() {
        let sheet = "FILE \"a (Track 1).bin\" BINARY\nTRACK 01 MODE2/2352\nINDEX 01 00:00:00\nFILE \"a (Track 2).bin\" BINARY\nTRACK 02 AUDIO\nINDEX 01 00:00:00\n";
        assert_eq!(companions(sheet).unwrap(), ["a (Track 1).bin", "a (Track 2).bin"]);
        assert_eq!(playlist("#EXTM3U\r\nGame (Disc 1).cue\r\n\r\n  Game (Disc 2).cue  \n"), ["Game (Disc 1).cue", "Game (Disc 2).cue"]);
        assert_eq!(sbi_beside("dir/Game (Europe).cue"), "dir/Game (Europe).sbi");
        assert_eq!(sbi_beside("dir.x/Game"), "dir.x/Game.sbi");
    }
}
