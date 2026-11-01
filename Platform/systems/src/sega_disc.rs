//! The Saturn's and the Dreamcast's recognisers: the system area at the start of a disc's first data sector, whose
//! hardware identifier is `SEGA SEGASATURN ` or `SEGA SEGAKATANA `, with the product number beside it as the serial
//! (Sega's Saturn disc format standard; Marcus Comstedt, "IP.BIN", <http://mc.pp.se/dc/ip.bin.html>; EmuSen_Cores.md
//! §4.9). A Dreamcast sheet whose first track is a GD-ROM's low-density area carries no system area there, and is
//! placed by its folder instead.

use crate::media::ascii;
use crate::probe::{Head, Need, Recognition, SYNC};

/// The bytes read first: the first sector of a raw track, with room for its header.
pub const FIRST: usize = 2352;

/// The system area's identifier and the offset of its ten-byte product number, for each system.
fn layout(system: &str) -> Option<(&'static [u8; 16], usize)> {
    match system {
        "saturn" => Some((b"SEGA SEGASATURN ", 0x20)),
        "dreamcast" => Some((b"SEGA SEGAKATANA ", 0x40)),
        _ => None,
    }
}

/// The first sector's user data: after a raw sector's sync and header (mode 1 or mode 2), else the bytes as they are.
pub fn system_area(head: &[u8]) -> &[u8] {
    let at = if head.len() >= 16 && head[..12] == SYNC { if head[15] == 1 { 16 } else { 24 } } else { 0 };
    head.get(at..).unwrap_or(&[])
}

/// The system's claim on a disc whose system area names it, with its product number; none otherwise.
pub fn recognise(system: &str, head: &Head) -> Result<Option<Recognition>, Need> {
    let Some((identifier, product)) = layout(system) else { return Ok(None) };
    let area = system_area(head.read(FIRST)?);
    if area.len() < 16 || area[..16] != identifier[..] {
        return Ok(None);
    }
    let serial = area.get(product..product + 10).map(|b| ascii(b).trim().to_string()).filter(|s| !s.is_empty());
    Ok(Some(Recognition { system: system.to_string(), serial, region: None }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(identifier: &[u8; 16], product: usize, serial: &[u8]) -> Vec<u8> {
        let mut a = vec![b' '; 0x100];
        a[..16].copy_from_slice(identifier);
        a[product..product + serial.len()].copy_from_slice(serial);
        a
    }

    fn claim(system: &str, bytes: &[u8]) -> Option<Recognition> {
        recognise(system, &Head { bytes, asked: FIRST }).unwrap()
    }

    #[test]
    fn each_system_claims_its_own_system_area_and_no_other() {
        let saturn = area(b"SEGA SEGASATURN ", 0x20, b"MK-81009  ");
        let dreamcast = area(b"SEGA SEGAKATANA ", 0x40, b"MK-51000  ");
        assert_eq!(claim("saturn", &saturn), Some(Recognition { system: "saturn".into(), serial: Some("MK-81009".into()), region: None }));
        assert_eq!(claim("dreamcast", &dreamcast).and_then(|r| r.serial).as_deref(), Some("MK-51000"));
        assert_eq!(claim("saturn", &dreamcast), None);
        assert_eq!(claim("dreamcast", &saturn), None);
        assert_eq!(claim("saturn", b"SEGADISCSYSTEM  "), None, "a Sega CD's disc");
        assert_eq!(claim("saturn", b""), None);
    }

    #[test]
    fn a_raw_sector_is_read_past_its_sync_and_header() {
        for (mode, at) in [(1u8, 16usize), (2, 24)] {
            let mut raw = vec![0u8; FIRST];
            raw[..12].copy_from_slice(&SYNC);
            raw[15] = mode;
            raw[at..at + 0x100].copy_from_slice(&area(b"SEGA SEGASATURN ", 0x20, b"T-12705H  "));
            assert_eq!(claim("saturn", &raw).and_then(|r| r.serial).as_deref(), Some("T-12705H"), "mode {mode}");
        }
    }

    #[test]
    fn the_host_is_asked_for_the_first_sector() {
        assert_eq!(recognise("saturn", &Head { bytes: &[], asked: 0 }).unwrap_err().0, FIRST);
    }
}
