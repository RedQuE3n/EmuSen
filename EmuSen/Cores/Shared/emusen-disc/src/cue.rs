//! The cue sheet, parsed in full (psx-spx "CDROM File Formats", .CUE/.BIN): its files, tracks, modes, indexes, pregaps
//! and postgaps, in the order written. Placing them on the disc is `disc.rs`'s; which files to read is `set.rs`'s.

use crate::msf::Msf;

/// A track's data as its file stores it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Audio,
    Mode1User,
    Mode1Raw,
    Mode2Xa,
    Mode2Raw,
}

impl Mode {
    /// The bytes each sector takes in its file.
    pub fn bytes(self) -> usize {
        match self {
            Mode::Mode1User => 2048,
            Mode::Mode2Xa => 2336,
            _ => 2352,
        }
    }

    pub fn audio(self) -> bool {
        self == Mode::Audio
    }

    fn parse(word: &str) -> Option<Mode> {
        Some(match word.to_ascii_uppercase().as_str() {
            "AUDIO" => Mode::Audio,
            "MODE1/2048" => Mode::Mode1User,
            "MODE1/2352" => Mode::Mode1Raw,
            "MODE2/2336" | "CDI/2336" => Mode::Mode2Xa,
            "MODE2/2352" | "CDI/2352" => Mode::Mode2Raw,
            _ => return None,
        })
    }
}

/// The FLAGS line's bits, as the control nibble takes them: DCP (copy permitted), 4CH, PRE.
pub mod flags {
    pub const PRE: u8 = 1 << 0;
    pub const DCP: u8 = 1 << 1;
    pub const FOUR_CHANNEL: u8 = 1 << 3;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Track {
    pub number: u8,
    /// The sheet's file this track's sectors are in.
    pub file: usize,
    pub mode: Mode,
    /// Each INDEX as its number and its frame in the file.
    pub indexes: Vec<(u8, u32)>,
    /// Sectors before and after the track that the file does not hold.
    pub pregap: u32,
    pub postgap: u32,
    pub flags: u8,
}

impl Track {
    /// The frame in its file where the track's stored sectors begin: its first INDEX.
    pub fn first_frame(&self) -> u32 {
        self.indexes.first().map_or(0, |i| i.1)
    }

    pub fn index(&self, n: u8) -> Option<u32> {
        self.indexes.iter().find(|i| i.0 == n).map(|i| i.1)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sheet {
    /// Each FILE line's name, as written.
    pub files: Vec<String>,
    pub tracks: Vec<Track>,
    pub catalog: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CueError {
    pub line: usize,
    pub what: String,
}

impl std::fmt::Display for CueError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "cue sheet line {}: {}", self.line, self.what)
    }
}

/// The words of a line, a quoted name kept whole.
fn words(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = line.trim();
    while !rest.is_empty() {
        if let Some(r) = rest.strip_prefix('"') {
            let end = r.find('"').unwrap_or(r.len());
            out.push(&r[..end]);
            rest = r.get(end + 1..).unwrap_or("").trim_start();
        } else {
            let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            out.push(&rest[..end]);
            rest = rest[end..].trim_start();
        }
    }
    out
}

impl Sheet {
    pub fn parse(text: &str) -> Result<Sheet, CueError> {
        let mut sheet = Sheet::default();
        let fail = |line: usize, what: &str| Err(CueError { line, what: what.into() });
        for (n, raw) in text.lines().enumerate() {
            let line = n + 1;
            let w = words(raw.trim_start_matches('\u{FEFF}'));
            let Some(command) = w.first() else { continue };
            match command.to_ascii_uppercase().as_str() {
                "FILE" => {
                    let Some(name) = w.get(1) else { return fail(line, "FILE names no file") };
                    let kind = w.get(2).map(|k| k.to_ascii_uppercase());
                    if !matches!(kind.as_deref(), Some("BINARY") | None) {
                        return fail(line, &format!("FILE of type {} is not a raw image", kind.unwrap_or_default()));
                    }
                    sheet.files.push((*name).to_owned());
                }
                "TRACK" => {
                    if sheet.files.is_empty() {
                        return fail(line, "TRACK before any FILE");
                    }
                    let number = w.get(1).and_then(|t| t.parse::<u8>().ok()).filter(|&t| (1..=99).contains(&t));
                    let mode = w.get(2).and_then(|m| Mode::parse(m));
                    let (Some(number), Some(mode)) = (number, mode) else { return fail(line, "TRACK needs a number 1-99 and a known mode") };
                    if sheet.tracks.last().is_some_and(|t| t.number >= number) {
                        return fail(line, "tracks must be numbered upward");
                    }
                    sheet.tracks.push(Track { number, file: sheet.files.len() - 1, mode, indexes: Vec::new(), pregap: 0, postgap: 0, flags: 0 });
                }
                "INDEX" | "PREGAP" | "POSTGAP" | "FLAGS" => {
                    let Some(t) = sheet.tracks.last_mut() else { return fail(line, "an entry before any TRACK") };
                    match command.to_ascii_uppercase().as_str() {
                        "INDEX" => {
                            let i = w.get(1).and_then(|i| i.parse::<u8>().ok()).filter(|&i| i <= 99);
                            let at = w.get(2).and_then(|m| Msf::parse(m));
                            let (Some(i), Some(at)) = (i, at) else { return fail(line, "INDEX needs a number and a time") };
                            if t.indexes.last().is_some_and(|l| l.0 >= i || l.1 > at.sector()) {
                                return fail(line, "indexes must run upward");
                            }
                            t.indexes.push((i, at.sector()));
                        }
                        "FLAGS" => {
                            for f in &w[1..] {
                                t.flags |= match f.to_ascii_uppercase().as_str() {
                                    "PRE" => flags::PRE,
                                    "DCP" => flags::DCP,
                                    "4CH" => flags::FOUR_CHANNEL,
                                    _ => 0,
                                };
                            }
                        }
                        gap => {
                            let Some(at) = w.get(1).and_then(|m| Msf::parse(m)) else { return fail(line, "a gap needs a time") };
                            if gap == "PREGAP" { t.pregap = at.sector() } else { t.postgap = at.sector() }
                        }
                    }
                }
                "CATALOG" => sheet.catalog = w.get(1).map(|c| (*c).to_owned()),
                _ => {}
            }
        }
        if sheet.tracks.is_empty() {
            return fail(text.lines().count(), "no TRACK");
        }
        if let Some(t) = sheet.tracks.iter().find(|t| t.index(1).is_none()) {
            return fail(0, &format!("track {} has no INDEX 01", t.number));
        }
        Ok(sheet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DUMP: &str = "FILE \"Game (Track 1).bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\nFILE \"Game (Track 2).bin\" BINARY\n  TRACK 02 AUDIO\n    INDEX 00 00:00:00\n    INDEX 01 00:02:00\n";

    #[test]
    fn a_redump_sheet_reads_whole() {
        let s = Sheet::parse(DUMP).unwrap();
        assert_eq!(s.files, ["Game (Track 1).bin", "Game (Track 2).bin"]);
        assert_eq!(s.tracks.len(), 2);
        assert_eq!((s.tracks[1].file, s.tracks[1].mode, s.tracks[1].indexes.clone()), (1, Mode::Audio, vec![(0, 0), (1, 150)]));
    }

    #[test]
    fn psx_spx_example_with_pregaps_and_flags() {
        let s = Sheet::parse("FILE \"PATH\\FILENAME.BIN\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n  TRACK 02 AUDIO\n    FLAGS DCP PRE\n    PREGAP 00:02:00\n    INDEX 01 08:09:29\n   TRACK 3 AUDIO\n   INDEX 1 14:02:29\nREM a comment\n").unwrap();
        assert_eq!(s.tracks[1].pregap, 150);
        assert_eq!(s.tracks[1].flags, flags::DCP | flags::PRE);
        assert_eq!(s.tracks[2].first_frame(), Msf::new(14, 2, 29).sector());
    }

    #[test]
    fn malformed_sheets_are_refused_with_the_line() {
        assert_eq!(Sheet::parse("TRACK 01 AUDIO\n").unwrap_err().line, 1);
        assert!(Sheet::parse("FILE a.bin BINARY\nTRACK 01 MODE3/2352\nINDEX 01 00:00:00\n").is_err());
        assert!(Sheet::parse("FILE a.bin BINARY\nTRACK 01 AUDIO\nINDEX 00 00:00:00\n").is_err(), "no INDEX 01");
        assert!(Sheet::parse("FILE a.wav WAVE\nTRACK 01 AUDIO\nINDEX 01 00:00:00\n").is_err());
        assert!(Sheet::parse("FILE a.bin BINARY\nTRACK 02 AUDIO\nINDEX 01 00:00:00\nTRACK 01 AUDIO\nINDEX 01 00:00:00\n").is_err());
    }
}
