//! The PlayStation's recogniser: a disc by the licence text of its system area, its market by the licence's second
//! line and its serial by SYSTEM.CNF's BOOT line (EmuSen_CoreAPI_Extensions.md §8.4, Diamond_Plan.md §4.1).

use crate::media::ascii;
use crate::probe::{DiscSectors, Head, Need, ROOT_FILE_LIMIT, Recognition, SYNC, root_file_through};
use emusen_dotnet::text::{trim, trim_start};


const LICENCE: &[u8; 32] = b"          Licensed  by          ";

/// Whether an .iso's user sectors or a raw track's 2,352-byte ones hold a PlayStation disc: the licence text at the start of sector 4's user data.
pub fn is_disc(image: &[u8]) -> bool {
    if image.len() >= 4 * 2048 + 32 && &image[4 * 2048..4 * 2048 + 32] == LICENCE {
        return true;
    }
    let raw = 4 * 2352;
    if image.len() < raw + 24 + 32 || image[..12] != SYNC || image[raw..raw + 12] != SYNC {
        return false;
    }
    let data = raw + if image[raw + 15] == 1 { 16 } else { 24 };
    &image[data..data + 32] == LICENCE
}

/// SYSTEM.CNF's BOOT line as the game's serial: "cdrom:\SLUS_007.70;1" is SLUS-00770.
pub fn serial_of(config: Option<&[u8]>) -> Option<String> {
    let text = ascii(config?);
    for line in text.split('\n') {
        let Some(eq) = line.find('=') else { continue };
        let start = trim_start(line);
        if start.len() < 4 || !start[..4].eq_ignore_ascii_case("BOOT") {
            continue;
        }
        return serial_of_file(trim(&line[eq + 1..]));
    }
    None
}

/// A boot line's program as a serial: "cdrom:\SLUS_007.70;1" is SLUS-00770, as Sapphire takes BOOT2's.
pub fn serial_of_file(file: &str) -> Option<String> {
    let cut = file.rfind('\\').max(file.rfind(':')).map_or(0, |i| i + 1);
    let file = &file[cut..];
    let file = file.split(';').next().unwrap_or("").replace('.', "").replace('_', "-");
    let file = trim(&file).to_ascii_uppercase();
    ((6..=12).contains(&file.len()) && file.contains('-')).then_some(file)
}

/// The licence's market and the BOOT line's serial; none for no PlayStation disc, or for a disc whose SYSTEM.CNF names
/// a BOOT2 program, which is the PlayStation 2's (Sapphire_Plan.md §4.1).
pub fn recognise(head: &Head) -> Result<Option<Recognition>, Need> {
    if !is_disc(head.read(12 * 1024)?) {
        return Ok(None);
    }
    let sectors = DiscSectors::new(head.read(160 * 1024)?);
    let four = sectors.sector(4);
    let line2 = if four.len() <= 32 { String::new() } else { ascii(&four[32..32 + 38.min(four.len() - 32)]) };
    let region = if line2.contains("Amer") { "Americas" } else if line2.contains("Euro") { "Europe" } else { "Japan" };
    let config = root_file_through(head, 160 * 1024, "SYSTEM.CNF", ROOT_FILE_LIMIT)?;
    if crate::ps2::disc::boot2(config.as_deref()).is_some() {
        return Ok(None);
    }
    Ok(Some(Recognition { system: "psx".into(), serial: serial_of(config.as_deref()), region: Some(region.into()) }))
}
