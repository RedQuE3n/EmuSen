//! A disc from an image and the files beside it, as a host passes them (EmuSen_Disc.md §9.2): a cue sheet with its
//! tracks, a lone raw track, an `.iso`, user sectors with no volume descriptor, or a CHD, and an `.sbi` among the
//! companions. The console's own signature, its market and its refusals stay with the console.

use crate::cue::Sheet;
use crate::disc::{Disc, IsoForm};

/// What a lone image is taken for where it is no sheet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lone {
    /// A CHD, a raw track or an `.iso`, told by its own bytes (§4).
    Detect,
    /// 2,048-byte user sectors, which the console's signature has shown them to be though no volume descriptor says so.
    User,
}

/// Why an image and its companions make no disc, with this crate's words where it has them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// A sheet whose files were not all given.
    Missing { needs: usize, has: usize },
    /// A sheet that does not parse, or whose files do not hold its tracks.
    Sheet(String),
    /// One file that is no image this crate reads.
    Image(String),
    /// An `.sbi` among the companions that does not parse.
    Sbi(String),
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Refusal::Missing { needs, has } => write!(f, "the cue sheet names {needs} files and {has} were given"),
            Refusal::Sheet(s) | Refusal::Image(s) | Refusal::Sbi(s) => f.write_str(s),
        }
    }
}

/// The `.sbi`'s identifier, by which it is told among the companions.
pub const SBI_MAGIC: &[u8; 4] = b"SBI\0";

/// Whether `image` reads as a cue sheet: text whose first command is one a sheet opens with.
pub fn is_sheet(image: &[u8]) -> bool {
    let head = &image[..image.len().min(4096)];
    let Ok(text) = std::str::from_utf8(head.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(head)) else { return false };
    text.lines().map(str::trim).find(|l| !l.is_empty()).is_some_and(|l| ["FILE", "REM", "CATALOG", "TITLE", "PERFORMER"].iter().any(|k| l.to_ascii_uppercase().starts_with(k)))
}

/// The disc `image` and its `companions` make: a sheet takes its files from the first companions, in its order; any
/// other image stands alone; an `.sbi` among the companions left over is applied. Opening reads no track's sectors.
pub fn open<B: AsRef<[u8]>>(image: B, mut companions: Vec<B>, iso: IsoForm, lone: Lone) -> Result<Disc<B>, Refusal> {
    let (mut disc, rest) = if is_sheet(image.as_ref()) {
        let sheet = Sheet::parse(&String::from_utf8_lossy(image.as_ref())).map_err(|e| Refusal::Sheet(e.to_string()))?;
        let needs = sheet.files.len();
        if companions.len() < needs {
            return Err(Refusal::Missing { needs, has: companions.len() });
        }
        let rest = companions.split_off(needs);
        (Disc::from_sheet(&sheet, companions, iso).map_err(|e| Refusal::Sheet(e.to_string()))?, rest)
    } else {
        let opened = match lone {
            Lone::User => Disc::from_user_image(image, iso),
            Lone::Detect => Disc::from_image(image, iso),
        };
        (opened.map_err(|e| Refusal::Image(e.to_string()))?, companions)
    };
    if let Some(sbi) = rest.iter().find(|c| c.as_ref().starts_with(SBI_MAGIC)) {
        disc.apply_sbi(sbi.as_ref()).map_err(|e| Refusal::Sbi(e.to_string()))?;
    }
    Ok(disc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sheet_is_told_by_its_first_command() {
        assert!(is_sheet(b"FILE \"a.bin\" BINARY\n"));
        assert!(is_sheet(b"\xEF\xBB\xBF\r\n  rem made by a tool\r\nFILE \"a.bin\" BINARY\r\n"));
        assert!(is_sheet(b"CATALOG 0000000000000\n") && is_sheet(b"TITLE \"x\"\n") && is_sheet(b"PERFORMER \"x\"\n"));
        assert!(!is_sheet(b"") && !is_sheet(b"\n\n") && !is_sheet(&[0x00, 0xFF, 0xFF, 0xFF]) && !is_sheet(b"TRACK 01 AUDIO\n"));
    }
}
