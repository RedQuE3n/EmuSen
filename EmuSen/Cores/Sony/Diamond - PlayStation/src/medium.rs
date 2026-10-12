//! The disc a create opens (Diamond_CDROM.md §8): a cue sheet with its tracks as companion files, a CHD, an `.iso` or a
//! lone raw track, each through emusen-disc as CD-XA, with an `.sbi` among the companions; its market read from the
//! licence text in sector 4, as `media.rs` reads a raw image's.

use emusen_disc::cue::Sheet;
use emusen_disc::{Disc, IsoForm, ecma130};

use crate::media::{Kind, Media, Refusal, licensed};

/// What opening found: no disc (a PS-EXE), or the disc and what it is.
pub enum Opened {
    Program,
    Disc(Box<Disc>, Media),
}

#[derive(Debug, PartialEq, Eq)]
pub enum DiscRefusal {
    Media(Refusal),
    /// A sheet whose track files were not all given.
    Missing { needs: usize, has: usize },
    /// A sheet or an image emusen-disc refuses, in its words.
    Bad(String),
}

/// Whether `image` reads as a cue sheet: text whose first command is FILE, REM or CATALOG.
pub fn is_sheet(image: &[u8]) -> bool {
    let head = &image[..image.len().min(4096)];
    let Ok(text) = std::str::from_utf8(head.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(head)) else { return false };
    text.lines().map(str::trim).find(|l| !l.is_empty()).is_some_and(|l| ["FILE", "REM", "CATALOG", "TITLE", "PERFORMER"].iter().any(|k| l.to_ascii_uppercase().starts_with(k)))
}

/// Opens `image` with its companions; a PS-EXE is left to `media.rs`.
pub fn open(image: &[u8], companions: &[&[u8]]) -> Result<Opened, DiscRefusal> {
    if image.starts_with(crate::media::EXE_MAGIC) {
        return Media::read(image).map(|_| Opened::Program).map_err(DiscRefusal::Media);
    }
    let (mut disc, used) = if is_sheet(image) {
        let sheet = Sheet::parse(&String::from_utf8_lossy(image)).map_err(|e| DiscRefusal::Bad(e.to_string()))?;
        let needs = sheet.files.len();
        if companions.len() < needs {
            return Err(DiscRefusal::Missing { needs, has: companions.len() });
        }
        let files = companions[..needs].iter().map(|c| c.to_vec()).collect();
        (Disc::from_sheet(&sheet, files, IsoForm::Mode2Form1).map_err(|e| DiscRefusal::Bad(e.to_string()))?, needs)
    } else {
        let opened = match Media::read(image) {
            Ok(Media { kind: Kind::Disc { raw: false }, .. }) => Disc::from_user_image(image.to_vec(), IsoForm::Mode2Form1),
            _ => Disc::from_image(image.to_vec(), IsoForm::Mode2Form1),
        };
        match opened {
            Ok(d) => (d, 0),
            Err(_) => return Err(DiscRefusal::Media(Refusal::NotPlayStation)),
        }
    };
    if let Some(sbi) = companions[used..].iter().find(|c| c.starts_with(b"SBI\0")) {
        disc.apply_sbi(sbi).map_err(|e| DiscRefusal::Bad(e.to_string()))?;
    }
    let mut raw = [0u8; ecma130::RAW];
    let start = disc.tracks()[0].start;
    disc.read(start + 4, &mut raw);
    let user = if raw[15] == 1 { &raw[16..] } else { &raw[24..] };
    match licensed(user) {
        Some(market) => Ok(Opened::Disc(Box::new(disc), Media { kind: Kind::Disc { raw: true }, market })),
        None => Err(DiscRefusal::Media(Refusal::NotPlayStation)),
    }
}

/// A disc's fingerprint, never 0: FNV-1a over its tracks' starts and kinds, its lead-out and the raw sectors 4 and 16 to
/// 23 of its first track, the licence and the volume descriptors with the root directory that usually follows them,
/// so that a state is held to the disc it was made on.
pub fn fingerprint(disc: &mut Disc) -> u64 {
    let mut h = 0xCBF2_9CE4_8422_2325u64;
    let mut eat = |bytes: &[u8]| bytes.iter().for_each(|&b| h = (h ^ b as u64).wrapping_mul(0x0100_0000_01B3));
    let tracks: Vec<(u32, bool)> = disc.tracks().iter().map(|t| (t.start, t.audio)).collect();
    for (start, audio) in &tracks {
        eat(&start.to_le_bytes());
        eat(&[*audio as u8]);
    }
    eat(&disc.lead_out().to_le_bytes());
    let first = tracks.first().map_or(0, |t| t.0);
    let mut raw = [0u8; ecma130::RAW];
    for sector in std::iter::once(4).chain(16..24) {
        disc.read(first + sector, &mut raw);
        eat(&raw);
    }
    h.max(1)
}
