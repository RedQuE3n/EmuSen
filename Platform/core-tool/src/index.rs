//! The buildbot's nightly index, `.index-extended`: one line per core zip, `YYYY-MM-DD <crc32> <file>` (EmuSen_Cores.md
//! §4.2), the update feed of §5.4.

use emusen_core_entry::stem_of_zip;

/// One core zip in the index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexLine {
    pub date: String,
    pub crc32: u32,
    pub file: String,
    pub stem: String,
}

fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10 && b[4] == b'-' && b[7] == b'-' && b.iter().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

/// The index's core lines, and how many lines were not one (a file that is not this platform's core zip, or a line of
/// another shape).
pub fn parse(text: &str) -> (Vec<IndexLine>, usize) {
    let (mut lines, mut skipped) = (Vec::new(), 0);
    for raw in text.lines() {
        let raw = raw.trim();
        if raw.is_empty() {
            continue;
        }
        let mut words = raw.split_whitespace();
        let parsed = (|| {
            let (date, crc, file) = (words.next()?, words.next()?, words.next()?);
            if words.next().is_some() || !is_date(date) || crc.len() != 8 {
                return None;
            }
            let crc32 = u32::from_str_radix(crc, 16).ok()?;
            let stem = stem_of_zip(file)?.to_string();
            Some(IndexLine { date: date.to_string(), crc32, file: file.to_string(), stem })
        })();
        match parsed {
            Some(line) if !lines.iter().any(|l: &IndexLine| l.stem == line.stem) => lines.push(line),
            _ => skipped += 1,
        }
    }
    (lines, skipped)
}
