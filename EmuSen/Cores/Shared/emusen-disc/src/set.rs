//! The set module (EmuSen_CoreAPI_Extensions.md §2.3): which files a host must read beside an image, and the discs an
//! `.m3u` lists. Small readers only; what a sheet means is `cue.rs`'s and `disc.rs`'s.

use crate::cue::{CueError, FileKind, Sheet};

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

/// Where one track's bytes lie in the file that holds it, as a dump's per-track hashes take them (EmuSen_Disc.md §12).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrackSpan {
    pub number: u8,
    /// The file's position among the sheet's FILE lines.
    pub file: usize,
    /// The first byte: the track's first INDEX in its file, 00 where it has one.
    pub offset: u64,
    /// Up to the next track's first byte in the same file; `None` to the file's end.
    pub length: Option<u64>,
}

/// Each track of a sheet as a byte range of its file, in the sheet's order. A WAVE file's one track is the whole file,
/// as it was dumped; a WAVE file naming several tracks has no ranges in its own bytes and is refused.
pub fn track_spans(sheet: &str) -> Result<Vec<TrackSpan>, String> {
    let sheet = Sheet::parse(sheet).map_err(|e| e.to_string())?;
    let mut spans: Vec<TrackSpan> = Vec::with_capacity(sheet.tracks.len());
    for (k, t) in sheet.tracks.iter().enumerate() {
        let prior = k.checked_sub(1).map(|p| &sheet.tracks[p]).filter(|p| p.file == t.file);
        if sheet.kinds[t.file] == FileKind::Wave {
            if prior.is_some() {
                return Err(format!("the WAVE file {} holds more than one track", sheet.files[t.file]));
            }
            spans.push(TrackSpan { number: t.number, file: t.file, offset: 0, length: None });
            continue;
        }
        let offset = match prior {
            Some(p) => spans[k - 1].offset + (t.first_frame() - p.first_frame()) as u64 * p.mode.bytes() as u64,
            None => t.first_frame() as u64 * t.mode.bytes() as u64,
        };
        if prior.is_some() {
            spans[k - 1].length = Some(offset - spans[k - 1].offset);
        }
        spans.push(TrackSpan { number: t.number, file: t.file, offset, length: None });
    }
    Ok(spans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tracks_span_is_its_first_index_to_the_next_tracks_in_its_file() {
        let redump = "FILE \"a (Track 1).bin\" BINARY\nTRACK 01 MODE2/2352\nINDEX 01 00:00:00\nFILE \"a (Track 2).bin\" BINARY\nTRACK 02 AUDIO\nINDEX 00 00:00:00\nINDEX 01 00:02:00\n";
        assert_eq!(
            track_spans(redump).unwrap(),
            [TrackSpan { number: 1, file: 0, offset: 0, length: None }, TrackSpan { number: 2, file: 1, offset: 0, length: None }]
        );
        let one = "FILE a.bin BINARY\nTRACK 01 MODE1/2048\nINDEX 01 00:00:00\nTRACK 02 AUDIO\nINDEX 00 00:00:10\nINDEX 01 00:00:12\nTRACK 03 AUDIO\nINDEX 01 00:00:20\n";
        assert_eq!(
            track_spans(one).unwrap(),
            [
                TrackSpan { number: 1, file: 0, offset: 0, length: Some(10 * 2048) },
                TrackSpan { number: 2, file: 0, offset: 10 * 2048, length: Some(10 * 2352) },
                TrackSpan { number: 3, file: 0, offset: 10 * 2048 + 10 * 2352, length: None },
            ]
        );
        let late = "FILE a.bin BINARY\nTRACK 01 MODE2/2352\nINDEX 01 00:00:05\n";
        assert_eq!(track_spans(late).unwrap()[0].offset, 5 * 2352);
        let wave = "FILE a.bin BINARY\nTRACK 01 MODE1/2352\nINDEX 01 00:00:00\nFILE b.wav WAVE\nTRACK 02 AUDIO\nINDEX 01 00:00:00\n";
        assert_eq!(track_spans(wave).unwrap()[1], TrackSpan { number: 2, file: 1, offset: 0, length: None });
        assert!(track_spans("FILE b.wav WAVE\nTRACK 01 AUDIO\nINDEX 01 00:00:00\nTRACK 02 AUDIO\nINDEX 01 00:00:10\n").unwrap_err().contains("more than one track"));
        assert!(track_spans("TRACK 01 AUDIO\n").is_err());
    }

    #[test]
    fn a_sheets_companions_and_a_lists_entries() {
        let sheet = "FILE \"a (Track 1).bin\" BINARY\nTRACK 01 MODE2/2352\nINDEX 01 00:00:00\nFILE \"a (Track 2).bin\" BINARY\nTRACK 02 AUDIO\nINDEX 01 00:00:00\n";
        assert_eq!(companions(sheet).unwrap(), ["a (Track 1).bin", "a (Track 2).bin"]);
        assert_eq!(playlist("#EXTM3U\r\nGame (Disc 1).cue\r\n\r\n  Game (Disc 2).cue  \n"), ["Game (Disc 1).cue", "Game (Disc 2).cue"]);
        assert_eq!(sbi_beside("dir/Game (Europe).cue"), "dir/Game (Europe).sbi");
        assert_eq!(sbi_beside("dir.x/Game"), "dir.x/Game.sbi");
    }
}
